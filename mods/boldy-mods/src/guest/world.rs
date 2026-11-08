//! The world around the player: the terrain sample, nearby mobs, the shapes to outline and drawing them.

use super::*;

/// The latest terrain sample and the query that produced it.
pub(super) struct Sample {
    /// The gameplay frame's session when this sample arrived. The stamp's own session is a
    /// different identifier (the world observation's), so it cannot be compared with frames.
    pub(super) frame_session: u64,
    pub(super) min: [i32; 3],
    pub(super) size: [i32; 3],
    pub(super) stamp: Stamp,
    pub(super) blocks: Blocks,
}

/// The parts of a terrain region the outline uses: layer 0's states, which shape each state
/// selects with, per-cell overrides and the shapes. The region itself also carries every state's
/// name and tags, which are costly to keep and to free.
pub(super) struct Blocks {
    pub(super) loaded: Vec<u64>,
    /// Palette index per cell, layer 0 only.
    pub(super) states: Vec<u16>,
    pub(super) palette: Vec<PaletteEntry>,
    /// Layer 0 slots with their own selection shape, sorted by slot.
    pub(super) overrides: Vec<(u32, Option<u32>)>,
    pub(super) shape_offsets: Vec<u32>,
    pub(super) shape_boxes: Vec<f64>,
}

/// A block state's selection shape and, for states without one, whether it has no outline.
#[derive(Clone, Copy)]
pub(super) struct PaletteEntry {
    pub(super) selection: Selection,
    /// Air, a liquid, or nameless: no outline when no shape is known.
    pub(super) empty: bool,
}

#[derive(Clone, Copy)]
pub(super) enum Selection {
    /// One shape for every occurrence; `None` when unavailable.
    Fixed(Option<u32>),
    PerCell,
    NotRequested,
}

/// Whether a block without known shape data has no outline: air, a liquid, or nameless.
/// Exact names, not substring searches: this runs for every state of every changed region.
pub(super) fn empty_block(id: &str) -> bool {
    matches!(
        id,
        "" | "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:structure_void"
            | "minecraft:light_block"
            | "minecraft:water"
            | "minecraft:flowing_water"
            | "minecraft:lava"
            | "minecraft:flowing_lava"
    ) || id.starts_with("minecraft:light_block")
}

impl Blocks {
    pub(super) fn from_region(region: &Region, cells: usize) -> Self {
        let palette = region
            .palette
            .iter()
            .map(|state| PaletteEntry {
                selection: match &state.selection {
                    ShapeBinding::Fixed(shape) => Selection::Fixed(known(shape)),
                    ShapeBinding::PerCell => Selection::PerCell,
                    ShapeBinding::NotRequested => Selection::NotRequested,
                },
                empty: empty_block(state.identifier.as_deref().unwrap_or("")),
            })
            .collect();
        Self {
            loaded: region.loaded.clone(),
            states: region
                .states
                .get(..cells)
                .unwrap_or(&region.states[..])
                .to_vec(),
            palette,
            overrides: region
                .overrides
                .iter()
                .filter(|o| (o.slot as usize) < cells)
                .map(|o| (o.slot, o.selection.as_ref().and_then(known)))
                .collect(),
            shape_offsets: region.shape_offsets.clone(),
            shape_boxes: region.shape_boxes.clone(),
        }
    }
}

/// What the crosshair is on this frame, from the last terrain sample, and whether it is a mob
/// (`true`) or a block. `aimed` is `Some` when Cinnabar says which actor it would hit (the boxes
/// of that actor, or `None` for none); without it every actor in `actors` can be the target.
pub(super) fn aimed_target(
    session: u64,
    dimension: i32,
    [eye, dir]: [[f64; 3]; 2],
    reach: f32,
    aimed: Option<Option<&[[f64; 6]]>>,
    actors: &[outline::ActorBox],
) -> Option<(Hit, bool)> {
    let target = SAMPLE.with(|sample| {
        let sample = sample.borrow();
        // After a dimension change or reconnect the last sample describes another world until
        // the new one arrives; outlining from it would show phantom blocks.
        let sample = sample.as_ref().filter(|sample| {
            sample.frame_session == session && sample.stamp.dimension == dimension
        })?;
        let lookup = |block| cell_at(sample, block);
        let reach = f64::from(reach);
        match aimed {
            // The aimed actor, wherever it is; otherwise blocks only.
            Some(aimed) => {
                if let Some(bounds) = aimed.and_then(bounds_of) {
                    return Some(outline::Target::Actor(Hit {
                        block: [0; 3],
                        min: bounds.0,
                        max: bounds.1,
                        distance: 0.0,
                        direct: false,
                        boxes: outline::Boxes::single(bounds.0, bounds.1),
                    }));
                }
                outline::target_with_actors(eye, dir, reach, lookup, outline::NO_ACTORS)
            }
            None => outline::target_with_actors(eye, dir, reach, lookup, actors),
        }
    })?;
    Some(match target {
        outline::Target::Block(hit) => (hit, false),
        outline::Target::Actor(hit) => (hit, true),
    })
}

/// The box around all of `boxes`, `None` when there are none.
pub(super) fn bounds_of(boxes: &[[f64; 6]]) -> Option<([f64; 3], [f64; 3])> {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for b in boxes {
        for axis in 0..3 {
            min[axis] = min[axis].min(b[axis]);
            max[axis] = max[axis].max(b[axis + 3]);
        }
    }
    min[0].is_finite().then_some((min, max))
}

/// While the blocks for a new view are still loading (a frame or two after a big turn), the last
/// block that was outlined, rather than a flicker off. `None` once the view is covered by the
/// sample (nothing is there) or when the last target was a mob.
pub(super) fn last_block_while_loading(
    eye: [f64; 3],
    dir: [f64; 3],
    reach: f32,
    dimension: i32,
) -> Option<(Hit, bool)> {
    let covered = SAMPLE.with(|sample| {
        sample.borrow().as_ref().is_some_and(|sample| {
            let max = [0, 1, 2].map(|a| sample.min[a] + sample.size[a]);
            outline::region_covers((sample.min, max), eye, dir, f64::from(reach))
        })
    });
    if covered {
        return None;
    }
    LAST_HIT.with(|last| {
        last.borrow()
            .as_ref()
            .filter(|last| last.dimension == dimension)
            .and_then(|last| last.hit)
            .filter(|(_, actor)| !actor)
    })
}

/// The mob hitboxes within `MOB_RANGE` of `eye`, nearest first, at most `MAX_MOB_BOXES`. With a
/// view direction (first person, where the camera is at the eye) the ones wholly behind the
/// viewer are left out: they cannot be seen, and each costs about ten thousand fuel to draw, so
/// the limit then goes to mobs that can.
pub(super) fn nearest_boxes(
    actors: &[outline::ActorBox],
    eye: [f64; 3],
    view: Option<[f64; 3]>,
) -> Vec<[[f64; 3]; 2]> {
    outline::nearest_mob_boxes(actors, eye, view, MOB_RANGE, MAX_MOB_BOXES)
}

/// Everything to draw this frame: other nearby mobs in the mob look, then the aimed block in
/// the block look or the aimed mob in the aimed colour (or the mob look when only the aimed mob
/// is outlined).
pub(super) fn shapes(target: &Target, settings: &Settings) -> Vec<Shape> {
    let mobs = settings.for_mobs();
    // A look that is switched off draws nothing: not the aimed block, nor the mobs.
    let aimed = target.hit.as_ref().filter(|(_, actor)| {
        if *actor {
            settings.mob_on
        } else {
            settings.block_on
        }
    });
    let aimed = aimed.map(|(hit, actor)| {
        let look = match (actor, settings.show_all_mobs) {
            (false, _) => *settings,
            (true, true) => settings.aimed(),
            (true, false) => mobs,
        };
        Shape {
            min: hit.min,
            max: hit.max,
            boxes: hit.boxes,
            settings: look,
            direct: hit.direct,
            block: !actor,
        }
    });
    let mut shapes: Vec<Shape> = target
        .mobs
        .iter()
        .filter(|_| settings.mob_on)
        .map(|[min, max]| Shape {
            min: *min,
            max: *max,
            boxes: outline::Boxes::single(*min, *max),
            settings: mobs,
            direct: false,
            block: false,
        })
        // The aimed mob is drawn once, in its aimed look.
        .filter(|shape| !aimed.is_some_and(|aimed| !aimed.block && aimed.same_box(shape)))
        .collect();
    shapes.extend(aimed);
    shapes
}

/// Hitboxes of nearby mobs and players, as Cinnabar uses for picking. Empty without the
/// Players and Entities permissions; the host only returns the kinds this mod may see.
/// Each nearby actor's boxes and whether Cinnabar's crosshair would hit it (`None`: unknown).
pub(super) fn nearby_actor_boxes(eye: [f64; 3]) -> Observed {
    let around_eye = |actor: &player_state::Actor| {
        actor.boxes.iter().any(|b| {
            let (min, max) = ([b.min.0, b.min.1, b.min.2], [b.max.0, b.max.1, b.max.2]);
            (0..3)
                .all(|axis| f64::from(min[axis]) <= eye[axis] && eye[axis] <= f64::from(max[axis]))
        })
    };
    // The host lists the nearest first and every actor read costs about a thousand fuel, so only
    // the nearest `ACTOR_LIMIT` are asked for: with at most 8 outlined (and only those within
    // `MOB_RANGE`) and the aimed one always close, the rest of a big crowd is never used.
    crate::arena::scratch(
        || player_state::read_actors_limited(ACTOR_LIMIT),
        |observed| {
            observed
                .as_ref()
                .map(|observed| boxes_of(observed.iter().map(|o| &o.actor), &around_eye))
                .unwrap_or_default()
        },
    )
}

/// Each pickable, visible actor's hitbox as `[min x, min y, min z, max x, max y, max z]`: one
/// box around all of its boxes, as the game shows it. Keeping every part box made crowds (16
/// players on a server hub) cost a large share of the frame's fuel.
pub(super) fn boxes_of<'a>(
    actors: impl Iterator<Item = &'a player_state::Actor>,
    around_eye: &dyn Fn(&player_state::Actor) -> bool,
) -> Observed {
    let mut boxes = Vec::with_capacity(16);
    let mut hittable = Vec::with_capacity(16);
    // A box around the eye is the player's own body, not something to outline.
    for actor in actors.filter(|actor| actor.pickable && !actor.invisible && !around_eye(actor)) {
        let mut bounds = [
            f32::INFINITY,
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ];
        for b in &actor.boxes {
            let (lo, hi) = ([b.min.0, b.min.1, b.min.2], [b.max.0, b.max.1, b.max.2]);
            for axis in 0..3 {
                bounds[axis] = bounds[axis].min(lo[axis]);
                bounds[axis + 3] = bounds[axis + 3].max(hi[axis]);
            }
        }
        if bounds[0].is_finite() {
            boxes.push([bounds.map(f64::from)]);
            hittable.push(actor.hittable);
        }
    }
    (boxes, hittable)
}

/// Lifts the lines just off the block faces so the depth test does not let the faces cut into
/// them, which makes thin lines shimmer.
pub(super) const INFLATE: f64 = 0.003;

/// Uploads the cube while it is missing, at most four times a second: drawing is refused before a world
/// is joined, and a dimension change or loading gap invalidates every handle (the host bumps its
/// drawing generation) without running `init` again.
pub(super) fn ensure_cube(seconds: f32) {
    let missing = CUBE.with(|cube| cube.borrow().len() != outline::EDGES.len())
        || FACE_QUADS.with(|faces| faces.borrow().len() != outline::FACES.len());
    // In a frame to itself: after the HUD layer or the menu patch, a later frame uploads it.
    if !missing
        || seconds < UPLOAD_RETRY_AT.with(StdCell::get)
        || HEAVY_FRAME.with(StdCell::get)
        || headroom() < CUBE_RESERVE
    {
        return;
    }
    UPLOAD_RETRY_AT.with(|at| at.set(seconds + 0.25));
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    upload_cube();
}

/// Forgets handles the host no longer recognises, so the next frame uploads fresh ones.
pub(super) fn forget_cube() {
    CUBE.with(|cube| cube.borrow_mut().clear());
    FACE_QUADS.with(|faces| faces.borrow_mut().clear());
    UPLOAD_RETRY_AT.with(|at| at.set(0.0));
}

/// Uploads each unit-cube edge as its own geometry, so a frame can draw only the visible ones.
pub(super) fn upload_cube() {
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
pub(super) fn release(handles: &[draw::Handle]) {
    for handle in handles {
        let _ = draw::remove(*handle);
    }
}

pub(super) fn forget_cube_silently() {
    CUBE.with(|cube| cube.borrow_mut().clear());
    FACE_QUADS.with(|faces| faces.borrow_mut().clear());
}

pub(super) fn draw_outlines(
    shapes: &[Shape],
    dimension: i32,
    camera: [f64; 3],
    seconds: f32,
    mining: bool,
) {
    ensure_cube(seconds);
    // Each shape's outline, worked out (and cached) before the arena below is switched on.
    let segments: Vec<Vec<outline::Segment>> = shapes
        .iter()
        .map(|shape| segments_for(&shape.boxes.inflated(INFLATE)))
        .collect();
    let drawn = CUBE.with(|cube| {
        let cube = cube.borrow();
        if cube.len() != outline::EDGES.len() {
            return Ok(false);
        }
        // Every line is an instance with its own small parameter list; building them in the
        // scratch arena makes those allocations nearly free, which matters with a crowd of
        // mobs outlined.
        crate::arena::scratch(
            || {
                let mut instances: Vec<draw::Instance> = Vec::with_capacity(16 * shapes.len());
                // The aimed outline (last) first, then the nearest mobs, while the frame can
                // afford them: each line costs about a thousand fuel to build and send, and a
                // frame that runs out stops the mod.
                let order = (0..shapes.len())
                    .rev()
                    .take(1)
                    .chain(0..shapes.len().saturating_sub(1));
                for (drawn, index) in order.enumerate() {
                    let needed = DRAW_RESERVE + SEND_FUEL * (instances.len() as u64 + 16);
                    if drawn > 0 && headroom() < needed {
                        break;
                    }
                    let (shape, segments) = (&shapes[index], &segments[index]);
                    push_shape(
                        &mut instances,
                        &cube,
                        shape,
                        segments,
                        dimension,
                        camera,
                        seconds,
                        mining,
                    );
                }
                let result = draw::replace_frame_group("outline", &instances);
                (instances, result)
            },
            |(_, result)| result.clone().map(|()| true),
        )
    });
    let drawn = match drawn {
        Ok(drawn) => drawn,
        Err(error) if error.code == draw::ErrorCode::StaleHandle => {
            // The host dropped our geometry; re-upload it and use plain lines this frame.
            forget_cube();
            false
        }
        Err(error) => {
            report_once(&format!("Boldy Mods: drawing failed ({})", error.detail));
            false
        }
    };
    if drawn {
        return;
    }
    // The plain-line fallback has a small shared budget: the aimed outline (drawn last) first,
    // then the nearest mobs, and only whole boxes.
    let mut lines: Vec<Line> = Vec::new();
    let order = (0..shapes.len())
        .rev()
        .take(1)
        .chain(0..shapes.len().saturating_sub(1));
    for index in order {
        let shape = &shapes[index];
        let min = shape.min.map(|v| v - INFLATE);
        let max = shape.max.map(|v| v + INFLATE);
        let width = outline::width_at(shape.settings.width, min, max, camera);
        let [r, g, b, a] = shape.settings.linear_rgba(seconds);
        let a = a * shape.settings.pulse_factor(seconds);
        let edges = segments[index]
            .iter()
            .filter(|segment| segment.visible(camera));
        if lines.len() + edges.clone().count() > mod_api::MAX_RENDER_RIBBONS {
            break;
        }
        lines.extend(edges.map(|segment| {
            Line {
                points: [segment.start, segment.end()]
                    .into_iter()
                    .map(|[x, y, z]| Vector3 {
                        x: x as f32,
                        y: y as f32,
                        z: z as f32,
                    })
                    .collect(),
                width,
                color: Rgba { r, g, b, a },
                pattern: LinePattern::Solid,
            }
        }));
    }
    if let Err(error) = render::draw_lines(&lines) {
        report_once(&format!("Boldy Mods: drawing failed ({error})"));
    }
}

/// The merged outline of `boxes`: computed once and reused while the same shape is outlined.
pub(super) fn segments_for(boxes: &outline::Boxes) -> Vec<outline::Segment> {
    if boxes.as_slice().len() <= 1 {
        // One box is cheap; keep the cache for the block's shape.
        return outline::shape_edges(boxes.as_slice());
    }
    SEGMENTS.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some((cached, segments)) = cache.as_ref()
            && cached == boxes
        {
            return segments.clone();
        }
        let segments = outline::shape_edges(boxes.as_slice());
        *cache = Some((*boxes, segments.clone()));
        segments
    })
}

/// Appends one shape's retained-geometry instances: the highlight cancel and the fill on each
/// box's visible faces, then the merged outline's visible lines.
#[allow(clippy::too_many_arguments)]
pub(super) fn push_shape(
    instances: &mut Vec<draw::Instance>,
    cube: &[draw::Handle],
    shape: &Shape,
    segments: &[outline::Segment],
    dimension: i32,
    camera: [f64; 3],
    seconds: f32,
    mining: bool,
) {
    let settings = &shape.settings;
    let min = shape.min.map(|v| v - INFLATE);
    let max = shape.max.map(|v| v + INFLATE);
    let width = outline::width_at(settings.width, min, max, camera);
    let [r, g, b, a] = settings.linear_rgba(seconds);
    let a = a * settings.pulse_factor(seconds);
    let boxes = shape.boxes.inflated(INFLATE);
    let anchor = |origin: [f64; 3]| {
        draw::Anchor::World(draw::WorldAnchor {
            dimension,
            origin: (origin[0], origin[1], origin[2]),
        })
    };
    // Each box's faces that face the camera; faces between boxes lie inside the block and are
    // hidden by it.
    let faces = |instances: &mut Vec<draw::Instance>, tint: (f32, f32, f32, f32)| {
        FACE_QUADS.with(|quads| {
            for b in boxes.as_slice() {
                let (lo, hi) = ([b[0], b[1], b[2]], [b[3], b[4], b[5]]);
                let scale = (
                    (hi[0] - lo[0]) as f32,
                    (hi[1] - lo[1]) as f32,
                    (hi[2] - lo[2]) as f32,
                );
                for (quad, face) in quads.borrow().iter().zip(outline::FACES) {
                    if face.visible(lo, hi, camera) {
                        let mut instance = drawing::instance(*quad, anchor(lo));
                        instance.transform.scale = scale;
                        instance.tint = tint;
                        instances.push(instance);
                    }
                }
            }
        });
    };
    if settings.highlight_cancel > 0 && shape.direct {
        // Cinnabar brightens its selected block's faces by 1.3 in highlight mode; darkening the
        // same faces by 1 / 1.3 first undoes it. Never on the bridging support or actors, which
        // Cinnabar does not highlight.
        let alpha = outline::highlight_cancel_alpha(settings.highlight_cancel);
        faces(instances, (0.0, 0.0, 0.0, alpha));
    }
    // No fill while mining, so the block's cracks stay visible.
    if settings.fill > 0 && !(mining && shape.block) {
        let alpha = f32::from(settings.fill) / 100.0 * settings.pulse_factor(seconds);
        let [fr, fg, fb] = settings.fill_srgb(seconds).map(outline::srgb_to_linear);
        faces(instances, (fr * alpha, fg * alpha, fb * alpha, alpha));
    }
    for segment in segments {
        if !segment.visible(camera) {
            continue;
        }
        // The unit edge along this axis from the origin, stretched to the segment.
        let Some(geometry) = cube.get(segment.axis * 4) else {
            continue;
        };
        let mut line = drawing::instance(*geometry, anchor(segment.start));
        let mut scale = [1.0f32; 3];
        scale[segment.axis] = segment.length as f32;
        line.transform.scale = (scale[0], scale[1], scale[2]);
        // Premultiplied blending: colour scaled by opacity.
        line.tint = (r * a, g * a, b * a, a);
        line.width = width;
        line.width_mode = draw::WidthMode::Pixels;
        line.join = draw::Join::Round;
        line.cap = draw::Cap::Round;
        instances.push(line);
    }
}

/// Reads the blocks along the look ray, keeping the previous sample while a new query is pending.
pub(super) fn refresh_terrain(eye: [f64; 3], dir: [f64; 3], reach: f32, frame_session: u64) {
    let reach = f64::from(reach);
    let per_cell = READ_COST.with(StdCell::get);
    let slack = outline::region_slack(per_cell, |slack| {
        outline::region_cells(outline::ray_region(eye, dir, reach, 1 + slack))
    });
    let (min, max) = outline::region_for(QUERY.with(StdCell::get), eye, dir, reach, slack);
    let cells = outline::region_cells((min, max));
    // A changed region costs fuel for every kind of block in it, and a frame that runs out
    // stops the mod. Leave it to a later frame when this one has too little left (other work
    // came first); each frame that waits trusts the old estimate a little less, so it is tried
    // again soon.
    if let Some(per_cell) = per_cell {
        let needed = per_cell * cells as f64 + READ_RESERVE;
        if (headroom() as f64) < needed {
            READ_COST.with(|cost| cost.set(Some(per_cell * 0.98)));
            return;
        }
    }
    let query = RegionQuery {
        min: Position {
            x: min[0],
            y: min[1],
            z: min[2],
        },
        max: Position {
            x: max[0],
            y: max[1],
            z: max[2],
        },
        geometry: Geometry::SELECTION,
    };
    let same_query = QUERY.with(|q| q.replace(Some((min, max)))) == Some((min, max));
    let after = if same_query {
        SAMPLE.with(|s| {
            s.borrow()
                .as_ref()
                .filter(|sample| sample.min == min)
                .map(|sample| sample.stamp)
        })
    } else {
        None
    };
    let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let before = budget::remaining_fuel();
    // The reply is lifted into the scratch arena; only the compact copy is kept.
    let read = crate::arena::scratch(
        || terrain::read_region(query, after),
        |result| match result {
            Ok(ReadResult::Changed(region)) => {
                Ok(Some((region.stamp, Blocks::from_region(region, cells))))
            }
            Ok(ReadResult::Pending | ReadResult::Unchanged(_)) => Ok(None),
            Err(error) => Err(format!("{error:?}")),
        },
    );
    match read {
        Ok(Some((stamp, blocks))) => {
            // What this region cost per block sizes the next one.
            let used = before.saturating_sub(budget::remaining_fuel());
            READ_COST.with(|cost| cost.set(Some(used as f64 / cells.max(1) as f64)));
            // Drop the old copy first: it is small now, unlike the old full region.
            SAMPLE.with(|s| {
                *s.borrow_mut() = Some(Sample {
                    frame_session,
                    min,
                    size,
                    stamp,
                    blocks,
                });
            });
        }
        Ok(None) => {}
        Err(error) => report_once(&format!("Boldy Mods: terrain unavailable ({error})")),
    }
}

/// The selection boxes of the block at `block`, from layer 0 of the sample.
pub(super) fn cell_at(sample: &Sample, block: [i32; 3]) -> Cell {
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
    let blocks = &sample.blocks;
    let loaded = blocks
        .loaded
        .get(cell / 64)
        .is_some_and(|word| (*word >> (cell % 64)) & 1 == 1);
    if !loaded {
        return Cell::Unknown;
    }
    // Layer 0 lives at slot `cell`.
    let Some(&index) = blocks.states.get(cell) else {
        return Cell::Unknown;
    };
    if index == NO_LAYER {
        return Cell::Boxes(Vec::new());
    }
    let Some(state) = blocks.palette.get(usize::from(index)) else {
        return Cell::Unknown;
    };
    let shape = match state.selection {
        Selection::Fixed(shape) => shape,
        Selection::PerCell => blocks
            .overrides
            .binary_search_by_key(&(cell as u32), |o| o.0)
            .ok()
            .and_then(|i| blocks.overrides[i].1),
        Selection::NotRequested => None,
    };
    match shape {
        Some(shape) => Cell::Boxes(shape_boxes(blocks, shape)),
        // No shape data: outline the full block unless it is air or a liquid.
        None if state.empty => Cell::Boxes(Vec::new()),
        None => Cell::Boxes(vec![[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]),
    }
}

pub(super) fn known(shape: &ResolvedShape) -> Option<u32> {
    match shape {
        ResolvedShape::Known(index) => Some(*index),
        ResolvedShape::Unavailable => None,
    }
}

pub(super) fn shape_boxes(blocks: &Blocks, shape: u32) -> Vec<[f64; 6]> {
    let shape = shape as usize;
    let (Some(&start), Some(&end)) = (
        blocks.shape_offsets.get(shape),
        blocks.shape_offsets.get(shape + 1),
    ) else {
        return Vec::new();
    };
    (start as usize..end as usize)
        .filter_map(|b| {
            let values = blocks.shape_boxes.get(b * 6..b * 6 + 6)?;
            Some([
                values[0], values[1], values[2], values[3], values[4], values[5],
            ])
        })
        .collect()
}

#[cfg(test)]
mod tests;
