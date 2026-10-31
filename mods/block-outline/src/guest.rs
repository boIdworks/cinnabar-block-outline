//! Block outline mod: draws a coloured outline around the block the player is looking at.
//!
//! Each frame reads the eye pose, samples the selection shapes of the terrain around the player,
//! raycasts to the targeted block and draws its edges as depth-tested lines. F8 opens a settings
//! menu for the colour, opacity and thickness; changes save to the mod's settings file.

use std::cell::{Cell as StdCell, RefCell};

use mod_api::bindings::{
    Guest,
    cinnabar::extension::{
        gameplay, hud, input, panel as host_panel,
        render::{self, Line, LinePattern, Rgba, Vector3},
        settings,
        terrain::{
            self, Geometry, Position, ReadResult, Region, RegionQuery, ResolvedShape, ShapeBinding,
            Stamp,
        },
    },
};

use mod_api::drawing::{self, api as draw, shapes};

use crate::outline::{self, Cell, Hit, Settings};
use crate::panel;

/// No block layer in a terrain slot.
const NO_LAYER: u16 = u16::MAX;
/// How long a status label stays on the HUD.
const LABEL_SECONDS: f32 = 5.0;

/// The latest terrain sample and the query that produced it.
struct Sample {
    min: [i32; 3],
    size: [i32; 3],
    stamp: Stamp,
    region: Region,
}

thread_local! {
    static SETTINGS: StdCell<Settings> = StdCell::new(Settings::default());
    static SAMPLE: RefCell<Option<Sample>> = const { RefCell::new(None) };
    /// The query last read, so an unchanged query can pass its stamp.
    static QUERY: StdCell<Option<([i32; 3], [i32; 3])>> = const { StdCell::new(None) };
    /// The last targeted block, kept on screen while the menu owns input as a live preview.
    static LAST_HIT: RefCell<Option<Target>> = const { RefCell::new(None) };
    /// Unit-cube edge geometry, uploaded once; empty when world drawing is unavailable.
    static CUBE: RefCell<Vec<draw::Handle>> = const { RefCell::new(Vec::new()) };
    /// Unit-cube face quads for the fill, in `outline::FACES` order; empty when unavailable.
    static FACE_QUADS: RefCell<Vec<draw::Handle>> = const { RefCell::new(Vec::new()) };
    static SECONDS: StdCell<f32> = const { StdCell::new(0.0) };
    static LABEL_UNTIL: StdCell<Option<f32>> = const { StdCell::new(None) };
    static REPORTED_ERROR: StdCell<bool> = const { StdCell::new(false) };
    /// When to next try uploading the cube while it is missing, in accumulated seconds.
    static UPLOAD_RETRY_AT: StdCell<f32> = const { StdCell::new(0.0) };
}

/// What to outline and where it is seen from.
struct Target {
    hit: Hit,
    dimension: i32,
    /// The rendered camera, which differs from the eye in third person.
    camera: [f64; 3],
}

struct BlockOutline;

impl Guest for BlockOutline {
    fn init() {
        let loaded = settings::load().map(|json| Settings::parse(&json));
        let settings = loaded.as_ref().copied().unwrap_or_default();
        SETTINGS.with(|s| s.set(settings));
        // May be refused before a world is joined; `draw_outline` retries until it succeeds.
        upload_cube();
        // The packer embeds the menu itself; only the saved values are patched in here, because
        // `init` runs under the same small fuel budget as a frame.
        let menu = host_panel::update_content(&panel::update(&settings, 0.0));
        match (loaded, menu) {
            (Ok(_), Ok(())) => show_label("Block Outline: press F8 for the outline menu"),
            (Err(error), _) => show_label(&format!("Block Outline: settings unavailable ({error})")),
            (_, Err(error)) => show_label(&format!(
                "Block Outline: allow the Controls permission for the F8 menu ({error})"
            )),
        }
    }

    fn frame() {
        let frame = gameplay::read_frame().ok().flatten();
        let controls = input::read_controls().ok();
        let delta = controls
            .as_ref()
            .map(|c| c.seconds)
            .or(frame.as_ref().map(|f| f.frame_seconds))
            .filter(|s| s.is_finite())
            .unwrap_or(0.0)
            .max(0.0);
        let seconds = SECONDS.with(|s| {
            s.set(s.get() + delta);
            s.get()
        });
        expire_label(seconds);

        let menu_open = controls.as_ref().is_some_and(|c| c.panel_open);
        if let Some(controls) = &controls {
            apply_menu_events(&controls.events, seconds);
        }
        let settings = SETTINGS.with(StdCell::get);
        if menu_open && settings.rainbow {
            let _ = host_panel::update_content(&panel::preview_update(&settings, seconds));
        }

        if let Some(frame) = frame {
            let eye = [f64::from(frame.eye.x), f64::from(frame.eye.y), f64::from(frame.eye.z)];
            refresh_terrain(eye, settings.reach);
            let dir = outline::look_direction(frame.yaw, frame.pitch);
            let hit = SAMPLE.with(|sample| {
                let sample = sample.borrow();
                // After a dimension change or reconnect the last sample describes another world
                // until the new one arrives; outlining from it would show phantom blocks.
                let sample = sample.as_ref().filter(|sample| {
                    sample.stamp.session == frame.session && sample.stamp.dimension == frame.dimension
                })?;
                outline::target(eye, dir, f64::from(settings.reach), |block| cell_at(sample, block))
            });
            let camera = gameplay::read_camera()
                .ok()
                .flatten()
                .map_or(eye, |pose| {
                    [f64::from(pose.position.x), f64::from(pose.position.y), f64::from(pose.position.z)]
                });
            LAST_HIT.with(|last| {
                *last.borrow_mut() = hit.map(|hit| Target {
                    hit,
                    dimension: frame.dimension,
                    camera,
                });
            });
        } else if !menu_open {
            // Another screen owns input; show nothing until gameplay resumes.
            return;
        }

        LAST_HIT.with(|last| {
            if let Some(target) = last.borrow().as_ref() {
                draw_outline(target, &settings, seconds);
            }
        });
    }
}

/// Applies the menu's control events, then saves and refreshes the menu once if anything changed.
fn apply_menu_events(events: &[input::Event], seconds: f32) {
    if events.is_empty() {
        return;
    }
    let mut settings = SETTINGS.with(StdCell::get);
    let mut changed = false;
    for event in events {
        changed |= panel::apply(&mut settings, &event.id, event.value);
    }
    if !changed {
        return;
    }
    SETTINGS.with(|s| s.set(settings));
    if let Err(error) = settings::save(&settings.to_json()) {
        report_once(&format!("Block Outline: settings not saved ({error})"));
    }
    if let Err(error) = host_panel::update_content(&panel::update(&settings, seconds)) {
        report_once(&format!("Block Outline: menu update failed ({error})"));
    }
}

/// Lifts the lines just off the block faces so the depth test does not let the faces cut into
/// them, which makes thin lines shimmer.
const INFLATE: f64 = 0.003;

/// Uploads the cube while it is missing, at most once a second: drawing is refused before a world
/// is joined, and a dimension change or loading gap invalidates every handle (the host bumps its
/// drawing generation) without running `init` again.
fn ensure_cube(seconds: f32) {
    let missing = CUBE.with(|cube| cube.borrow().len() != outline::EDGES.len())
        || FACE_QUADS.with(|faces| faces.borrow().len() != outline::FACES.len());
    if !missing || seconds < UPLOAD_RETRY_AT.with(StdCell::get) {
        return;
    }
    UPLOAD_RETRY_AT.with(|at| at.set(seconds + 1.0));
    upload_cube();
}

/// Forgets handles the host no longer recognises, so the next frame uploads fresh ones.
fn forget_cube() {
    CUBE.with(|cube| cube.borrow_mut().clear());
    FACE_QUADS.with(|faces| faces.borrow_mut().clear());
    UPLOAD_RETRY_AT.with(|at| at.set(0.0));
}

/// Uploads each unit-cube edge as its own geometry, so a frame can draw only the visible ones.
fn upload_cube() {
    forget_cube_silently();
    let mut handles = Vec::with_capacity(outline::EDGES.len());
    for edge in outline::EDGES {
        let Ok(spec) = shapes::polyline(&edge.unit_points(), false, [1.0; 4]) else {
            release(&handles);
            return;
        };
        let Some(Ok(handle)) = spec.first().map(draw::create_geometry) else {
            // World drawing is not granted yet: `draw_outline` falls back to `render.draw-lines`.
            release(&handles);
            return;
        };
        handles.push(handle);
    }
    let mut quads = Vec::with_capacity(outline::FACES.len());
    for face in outline::FACES {
        let vertices = face
            .unit_corners()
            .map(|[x, y, z]| draw::Vertex {
                position: (x, y, z),
                color: (1.0, 1.0, 1.0, 1.0),
                uv: (0.0, 0.0),
                data: (0.0, 0.0, 0.0, 0.0),
            })
            .to_vec();
        // Both windings, so the quad shows whichever way it faces.
        let indices = vec![0, 1, 2, 0, 2, 3, 0, 2, 1, 0, 3, 2];
        let mesh = draw::GeometrySpec::Triangles(draw::Mesh { vertices, indices });
        let Ok(handle) = draw::create_geometry(&mesh) else {
            release(&handles);
            release(&quads);
            return;
        };
        quads.push(handle);
    }
    // Commit both sets together, so a partial upload is retried as a whole.
    CUBE.with(|cube| *cube.borrow_mut() = handles);
    FACE_QUADS.with(|faces| *faces.borrow_mut() = quads);
}

/// Frees geometry from an incomplete upload, so retries cannot pile up host resources.
fn release(handles: &[draw::Handle]) {
    for handle in handles {
        let _ = draw::remove(*handle);
    }
}

fn forget_cube_silently() {
    CUBE.with(|cube| cube.borrow_mut().clear());
    FACE_QUADS.with(|faces| faces.borrow_mut().clear());
}

fn draw_outline(target: &Target, settings: &Settings, seconds: f32) {
    ensure_cube(seconds);
    let min = target.hit.min.map(|v| v - INFLATE);
    let max = target.hit.max.map(|v| v + INFLATE);
    let visible = outline::EDGES.map(|edge| edge.visible(min, max, target.camera));
    let width = outline::width_at(settings.width, min, max, target.camera);
    let [r, g, b, a] = settings.linear_rgba(seconds);
    let drawn = CUBE.with(|cube| {
        let cube = cube.borrow();
        if cube.len() != outline::EDGES.len() {
            return Ok(false);
        }
        let anchor = draw::Anchor::World(draw::WorldAnchor {
            dimension: target.dimension,
            origin: (min[0], min[1], min[2]),
        });
        let scale = (
            (max[0] - min[0]) as f32,
            (max[1] - min[1]) as f32,
            (max[2] - min[2]) as f32,
        );
        let mut instances: Vec<draw::Instance> = Vec::with_capacity(15);
        if settings.fill > 0 {
            let alpha = f32::from(settings.fill) / 100.0;
            FACE_QUADS.with(|faces| {
                for (quad, face) in faces.borrow().iter().zip(outline::FACES) {
                    if face.visible(min, max, target.camera) {
                        let mut fill = drawing::instance(*quad, anchor);
                        fill.transform.scale = scale;
                        fill.tint = (r * alpha, g * alpha, b * alpha, alpha);
                        instances.push(fill);
                    }
                }
            });
        }
        instances.extend(cube
            .iter()
            .zip(visible)
            .filter(|(_, visible)| *visible)
            .map(|(geometry, _)| {
                let mut line = drawing::instance(*geometry, anchor);
                line.transform.scale = scale;
                // Premultiplied blending: colour scaled by opacity.
                line.tint = (r * a, g * a, b * a, a);
                line.width = width;
                line.width_mode = draw::WidthMode::Pixels;
                line.join = draw::Join::Round;
                line.cap = draw::Cap::Round;
                line
            }));
        match draw::replace_frame_group("outline", &instances) {
            Ok(()) => Ok(true),
            Err(error) => Err(error),
        }
    });
    let drawn = match drawn {
        Ok(drawn) => drawn,
        Err(error) if error.code == draw::ErrorCode::StaleHandle => {
            // The host dropped our geometry; re-upload it and use plain lines this frame.
            forget_cube();
            false
        }
        Err(error) => {
            report_once(&format!("Block Outline: drawing failed ({})", error.detail));
            false
        }
    };
    if drawn {
        return;
    }
    let lines: Vec<Line> = outline::EDGES
        .into_iter()
        .zip(visible)
        .filter(|(_, visible)| *visible)
        .map(|(edge, _)| Line {
            points: edge
                .points(min, max)
                .into_iter()
                .map(|[x, y, z]| Vector3 { x, y, z })
                .collect(),
            width,
            color: Rgba { r, g, b, a },
            pattern: LinePattern::Solid,
        })
        .collect();
    if let Err(error) = render::draw_lines(&lines) {
        report_once(&format!("Block Outline: drawing failed ({error})"));
    }
}

/// Reads the region around the eye, keeping the previous sample while a new query is pending.
fn refresh_terrain(eye: [f64; 3], reach: f32) {
    let radius = reach.ceil() as i32;
    let (min, max) = outline::region_for(
        QUERY.with(StdCell::get),
        eye.map(|v| v.floor() as i32),
        radius,
    );
    let query = RegionQuery {
        min: Position { x: min[0], y: min[1], z: min[2] },
        max: Position { x: max[0], y: max[1], z: max[2] },
        geometry: Geometry::SELECTION,
    };
    let same_query = QUERY.with(|q| q.replace(Some((min, max)))) == Some((min, max));
    let after = if same_query {
        SAMPLE.with(|s| {
            s.borrow()
                .as_ref()
                .filter(|sample| sample.min == min)
                .map(|sample| sample.stamp.clone())
        })
    } else {
        None
    };
    match terrain::read_region(query, after) {
        Ok(ReadResult::Changed(region)) => SAMPLE.with(|s| {
            *s.borrow_mut() = Some(Sample {
                min,
                size: [max[0] - min[0], max[1] - min[1], max[2] - min[2]],
                stamp: region.stamp.clone(),
                region,
            });
        }),
        Ok(ReadResult::Pending | ReadResult::Unchanged(_)) => {}
        Err(error) => report_once(&format!("Block Outline: terrain unavailable ({error:?})")),
    }
}

/// The selection boxes of the block at `block`, from layer 0 of the sample.
fn cell_at(sample: &Sample, block: [i32; 3]) -> Cell {
    let rel = [
        block[0] - sample.min[0],
        block[1] - sample.min[1],
        block[2] - sample.min[2],
    ];
    if (0..3).any(|axis| rel[axis] < 0 || rel[axis] >= sample.size[axis]) {
        return Cell::Unknown;
    }
    let [_, size_y, size_z] = sample.size.map(|v| v as usize);
    let [x, y, z] = rel.map(|v| v as usize);
    let cell = (x * size_z + z) * size_y + y;
    let region = &sample.region;
    let loaded = region
        .loaded
        .get(cell / 64)
        .is_some_and(|word| (*word >> (cell % 64)) & 1 == 1);
    if !loaded {
        return Cell::Unknown;
    }
    // Layer 0 lives at slot `cell`.
    let Some(&index) = region.states.get(cell) else {
        return Cell::Unknown;
    };
    if index == NO_LAYER {
        return Cell::Boxes(Vec::new());
    }
    let Some(state) = region.palette.get(usize::from(index)) else {
        return Cell::Unknown;
    };
    let shape = match &state.selection {
        ShapeBinding::Fixed(shape) => known(shape),
        ShapeBinding::PerCell => region
            .overrides
            .binary_search_by_key(&(cell as u32), |o| o.slot)
            .ok()
            .and_then(|i| region.overrides[i].selection.as_ref().and_then(known)),
        ShapeBinding::NotRequested => None,
    };
    match shape {
        Some(shape) => Cell::Boxes(shape_boxes(region, shape)),
        // No shape data: outline the full block unless it is air or a liquid.
        _ => {
            let id = state.identifier.as_deref().unwrap_or("");
            if id.is_empty() || ["air", "water", "lava"].iter().any(|n| id.contains(n)) {
                Cell::Boxes(Vec::new())
            } else {
                Cell::Boxes(vec![[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]])
            }
        }
    }
}

fn known(shape: &ResolvedShape) -> Option<u32> {
    match shape {
        ResolvedShape::Known(index) => Some(*index),
        ResolvedShape::Unavailable => None,
    }
}

fn shape_boxes(region: &Region, shape: u32) -> Vec<[f64; 6]> {
    let shape = shape as usize;
    let (Some(&start), Some(&end)) = (
        region.shape_offsets.get(shape),
        region.shape_offsets.get(shape + 1),
    ) else {
        return Vec::new();
    };
    (start as usize..end as usize)
        .filter_map(|b| {
            let values = region.shape_boxes.get(b * 6..b * 6 + 6)?;
            Some([values[0], values[1], values[2], values[3], values[4], values[5]])
        })
        .collect()
}

fn show_label(text: &str) {
    let text: String = text.chars().filter(|c| !c.is_control()).take(120).collect();
    if hud::set_label(&text).is_ok() {
        let now = SECONDS.with(StdCell::get);
        LABEL_UNTIL.with(|until| until.set(Some(now + LABEL_SECONDS)));
    }
}

fn expire_label(seconds: f32) {
    let expired = LABEL_UNTIL.with(|until| until.get().is_some_and(|t| seconds >= t));
    if expired && hud::set_label("").is_ok() {
        LABEL_UNTIL.with(|until| until.set(None));
    }
}

/// Shows the first persistent failure only, so a denied permission does not spam the HUD.
fn report_once(text: &str) {
    if !REPORTED_ERROR.with(|r| r.replace(true)) {
        show_label(text);
    }
}

mod_api::bindings::export!(BlockOutline with_types_in mod_api::bindings);
