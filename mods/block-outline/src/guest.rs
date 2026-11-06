//! Boldy Mods: draws a coloured outline around the block or mob the player is looking at, and an
//! arrow counter on the HUD.
//!
//! Each frame reads the eye pose, samples the selection shapes of the terrain around the player,
//! raycasts to the targeted block and draws its edges as depth-tested lines. F8 opens a settings
//! menu for the colours, sizes and the arrow counter; changes save to the mod's settings file.

use std::cell::{Cell as StdCell, RefCell};

use mod_api::bindings::{
    Guest,
    cinnabar::extension::{
        budget, camera, environment, gameplay, hud, input, panel as host_panel, player_state,
        render::{self, Line, LinePattern, Rgba, Vector3},
        settings,
        terrain::{
            self, Geometry, Position, ReadResult, Region, RegionQuery, ResolvedShape, ShapeBinding,
            Stamp,
        },
    },
};

use mod_api::drawing::{self, api as draw, shapes};

use crate::counter::{self, Quiver};
use crate::cps;
use crate::hudedit;
use crate::outline::{self, Cell, Hit, KeyName, Settings};
use crate::panel::{self, MenuState};
use crate::zoom;

/// No block layer in a terrain slot.
const NO_LAYER: u16 = u16::MAX;
/// How long a status label stays on the HUD.
const LABEL_SECONDS: f32 = 5.0;
/// How long the pointer must rest on a control before its help appears.
const HOVER_SECONDS: f32 = 0.5;
/// Each nearby actor's box (as `[min, max]` per axis) and whether Cinnabar says its crosshair
/// would hit it (`None`: unknown).
type Observed = Vec<(Vec<[f64; 6]>, Option<bool>)>;
/// A hover lookup: its inputs, the pointer on the card and the control under it.
type HoverLookup = (
    (Option<(f32, f32)>, MenuState, bool),
    Option<[f32; 2]>,
    Option<usize>,
);

/// The latest terrain sample and the query that produced it.
struct Sample {
    /// The gameplay frame's session when this sample arrived. The stamp's own session is a
    /// different identifier (the world observation's), so it cannot be compared with frames.
    frame_session: u64,
    min: [i32; 3],
    size: [i32; 3],
    stamp: Stamp,
    blocks: Blocks,
}

/// The parts of a terrain region the outline uses: layer 0's states, which shape each state
/// selects with, per-cell overrides and the shapes. The region itself also carries every state's
/// name and tags, which are costly to keep and to free.
struct Blocks {
    loaded: Vec<u64>,
    /// Palette index per cell, layer 0 only.
    states: Vec<u16>,
    palette: Vec<PaletteEntry>,
    /// Layer 0 slots with their own selection shape, sorted by slot.
    overrides: Vec<(u32, Option<u32>)>,
    shape_offsets: Vec<u32>,
    shape_boxes: Vec<f64>,
}

/// A block state's selection shape and, for states without one, whether it has no outline.
#[derive(Clone, Copy)]
struct PaletteEntry {
    selection: Selection,
    /// Air, a liquid, or nameless: no outline when no shape is known.
    empty: bool,
}

#[derive(Clone, Copy)]
enum Selection {
    /// One shape for every occurrence; `None` when unavailable.
    Fixed(Option<u32>),
    PerCell,
    NotRequested,
}

/// Whether a block without known shape data has no outline: air, a liquid, or nameless.
/// Exact names, not substring searches: this runs for every state of every changed region.
fn empty_block(id: &str) -> bool {
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
    fn from_region(region: &Region, cells: usize) -> Self {
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

mod hud_elements;
use hud_elements::{arrow_counter, cps_frame};

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
    /// A hash of the last problem shown, so each problem is shown once rather than every frame.
    static REPORTED_ERROR: StdCell<u64> = const { StdCell::new(0) };
    /// Reading the inventory was refused: the Player status permission is not allowed.
    static NO_PLAYER_STATUS: StdCell<bool> = const { StdCell::new(false) };
    /// When to next try uploading the cube while it is missing, in accumulated seconds.
    static UPLOAD_RETRY_AT: StdCell<f32> = const { StdCell::new(0.0) };
    /// Unsaved menu state: key capture and whether the colour tools edit the fill.
    static MENU: StdCell<MenuState> = const { StdCell::new(MenuState { capturing: false, capturing_fullbright: false, capturing_zoom: false, editing_fill: false, editing_aim: false, page: panel::Page::Home, colour_open: false, typing: None, confirm: panel::Reset::None, focus: None, moving: false, moving_cps: false, guide_bottom: false, fuel_k: [0, 0] }) };
    /// The player's attack keys (`MouseLeft` until read) and the bindings revision they came
    /// from, re-read every `BINDINGS_SECONDS`.
    static ATTACK_KEYS: RefCell<(Option<u64>, Vec<String>)> = const { RefCell::new((None, Vec::new())) };
    static BINDINGS_CHECK_AT: StdCell<f32> = const { StdCell::new(0.0) };
    /// A changed set of bindings being read, one page per frame: its revision, the next offset
    /// and the attack keys found so far.
    static BINDINGS_READ: RefCell<Option<(u64, u32, Vec<String>)>> = const { RefCell::new(None) };
    /// The fullbright state last sent to Cinnabar and the session it was sent in, so it is sent
    /// once per change and again after joining another world.
    static FULLBRIGHT_SENT: StdCell<Option<(bool, u64)>> = const { StdCell::new(None) };
    /// The menu still shows the embedded defaults; the first frame patches in the settings.
    static MENU_STALE: StdCell<bool> = const { StdCell::new(true) };
    /// Fuel the last changed terrain region cost per block; unknown until one is read.
    static READ_COST: StdCell<Option<f64>> = const { StdCell::new(None) };
    /// The last actors read, reused by frames with too little fuel left to read them.
    static LAST_ACTORS: RefCell<Observed> = const { RefCell::new(Vec::new()) };
    /// This frame already did a costly job (a menu patch or a geometry upload), so a changed
    /// terrain region, the other costly job, waits for the next frame.
    static HEAVY_FRAME: StdCell<bool> = const { StdCell::new(false) };
    /// The last merged outline worked out, by its inflated boxes: shapes with several boxes cost
    /// a little to merge, so a block's outline is only worked out when the target changes.
    static SEGMENTS: RefCell<Option<(outline::Boxes, Vec<outline::Segment>)>> = const { RefCell::new(None) };
    /// Whether the HUD was hidden (F1) when last checked, and when.
    static HUD_HIDDEN: StdCell<(bool, f32)> = const { StdCell::new((false, f32::NEG_INFINITY)) };
    /// The control under the pointer and when the pointer arrived on it.
    static HOVER: StdCell<Option<(usize, f32)>> = const { StdCell::new(None) };
    /// When the rainbow preview in the menu was last sent.
    static PREVIEW_AT: StdCell<f32> = const { StdCell::new(f32::NEG_INFINITY) };
    /// When a reset button started waiting for its second click.
    static CONFIRM_AT: StdCell<f32> = const { StdCell::new(0.0) };
    /// The arrow key being held (1 + its place in `ARROWS`, 0 for none) and when it repeats next.
    static ARROW_REPEAT: StdCell<(u8, f32)> = const { StdCell::new((0, 0.0)) };
    /// The last hover lookup: what it was for (pointer, menu state, fill matches line), the
    /// pointer on the card and the control under it.
    static HOVER_LOOKUP: StdCell<Option<HoverLookup>> = const { StdCell::new(None) };
    /// The help entry on screen and where, so the box is only updated when it changes.
    static TIP_SHOWN: StdCell<Option<(usize, [f32; 2])>> = const { StdCell::new(None) };
    /// Whether the arrow counter's HUD layer was published since the mod (re)started.
    static HUD_PUBLISHED: StdCell<bool> = const { StdCell::new(false) };
    /// When to try publishing it again after the host refused it (no HUD permission).
    static HUD_RETRY_AT: StdCell<f32> = const { StdCell::new(0.0) };
    /// The arrows carried and the inventory they were counted from: its session and equipment
    /// revision, so the inventory is only read again after it changes.
    static QUIVER: StdCell<Option<(u64, u64, Quiver)>> = const { StdCell::new(None) };
    /// What the HUD last showed, so it is only patched when something changes.
    static COUNTER_SHOWN: StdCell<Option<counter::View>> = const { StdCell::new(None) };
    /// When the left and right mouse buttons were clicked recently, for the CPS readout.
    static CLICKS: StdCell<(cps::Clicks, cps::Clicks)> =
        const { StdCell::new((cps::Clicks::new(), cps::Clicks::new())) };
    /// What the CPS readout last showed, so it is only patched when something changes.
    static CPS_SHOWN: StdCell<Option<cps::View>> = const { StdCell::new(None) };
    /// The CPS text's measured pieces, measured again when the GUI scale changes.
    static CPS_WIDTHS: StdCell<Option<cps::Widths>> = const { StdCell::new(None) };
    /// The most fuel any frame has used since the mod started, and when the start page's fuel
    /// line was last refreshed.
    static FUEL_PEAK: StdCell<u64> = const { StdCell::new(0) };
    static FUEL_SHOWN_AT: StdCell<f32> = const { StdCell::new(0.0) };
    /// Whether the menu was open this frame.
    static MENU_OPEN: StdCell<bool> = const { StdCell::new(false) };
    /// Watches the HUD root's size and GUI scale for changes.
    static VIEWPORT_WATCH: StdCell<counter::ViewportWatch> =
        const { StdCell::new(counter::ViewportWatch::new()) };
    /// The last number measured and its width at text scale 1.
    static COUNT_WIDTH: StdCell<Option<(u32, f32)>> = const { StdCell::new(None) };
    /// While dragging the counter: where the pointer holds it, from its top-left corner.
    /// The part being dragged and how.
    static EDITOR: StdCell<hudedit::Editor> =
        const { StdCell::new(hudedit::Editor { selected: 0, drag: None }) };
    /// How the CPS readout's frame looks while the HUD is edited, set by the arrow counter's
    /// frame, which handles the pointer for both elements.
    static EDIT_CPS: StdCell<Option<counter::Edit>> = const { StdCell::new(None) };
    /// The counter was moved or resized but not saved yet; saved once moving ends.
    static COUNTER_UNSAVED: StdCell<bool> = const { StdCell::new(false) };
    /// The zoom's easing state.
    static ZOOM: StdCell<zoom::Engine> = StdCell::new(zoom::Engine::default());
    /// Zoom mode "press to toggle": whether the zoom is switched on.
    static ZOOM_TOGGLED: StdCell<bool> = const { StdCell::new(false) };
    /// The hotbar slot last seen while zooming, to turn a scroll into zoom steps.
    static ZOOM_SLOT: StdCell<Option<u8>> = const { StdCell::new(None) };
}

/// Mob hitboxes outlined around the player: within this many blocks of the eye, nearest first,
/// and at most `MAX_MOB_BOXES`, which keeps a crowd inside the frame's fuel.
const MOB_RANGE: f64 = 12.0;
/// Fuel kept back after a terrain read for the rest of the frame (actors and drawing).
const READ_RESERVE: f64 = 70_000.0;
/// Fuel a frame needs left to read the nearby actors; with less, the last ones are reused.
const ACTOR_RESERVE: u64 = 110_000;
/// Fuel kept back while adding mob outlines, for sending them and the rest of the frame.
const DRAW_RESERVE: u64 = 25_000;
/// About what sending one line instance costs.
const SEND_FUEL: u64 = 400;
const MAX_MOB_BOXES: usize = 8;

/// One box to draw and how.
#[derive(Clone, Copy)]
struct Shape {
    min: [f64; 3],
    max: [f64; 3],
    /// The shape's boxes, outlined as one merged shape.
    boxes: outline::Boxes,
    settings: Settings,
    /// Cinnabar's own selected block, whose highlight can be cancelled.
    direct: bool,
    /// A block rather than a mob: its fill gives way to the cracks while mining.
    block: bool,
}

impl Shape {
    fn same_box(&self, other: &Shape) -> bool {
        let close = |a: [f64; 3], b: [f64; 3]| (0..3).all(|axis| (a[axis] - b[axis]).abs() < 1e-6);
        close(self.min, other.min) && close(self.max, other.max)
    }
}

/// What to outline and where it is seen from.
struct Target {
    /// The block or mob aimed at, and whether it is a mob.
    hit: Option<(Hit, bool)>,
    /// Other nearby mob hitboxes as `[min, max]`, while every nearby mob is outlined.
    mobs: Vec<[[f64; 3]; 2]>,
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
        // The packer embeds the menu itself; the saved values are patched in by the first frame,
        // not here: `init` also runs when Cinnabar restarts the mod, and failing it then would
        // switch the mod off, so it stays well inside its fuel.
        reserve_keys(&settings);
        match loaded {
            Ok(_) => show_label(&format!(
                "Boldy Mods: press {} for the menu",
                settings.menu_key.display()
            )),
            Err(error) => show_label(&format!("Boldy Mods: settings unavailable ({error})")),
        }
    }

    fn frame() {
        let start = budget::remaining_fuel();
        frame_body();
        note_fuel(start);
    }
}

/// One frame's work; [`Guest::frame`] measures the fuel it takes.
fn frame_body() {
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
    // Like Simple Fullbright, ask for fullbright first thing in every frame: the host keeps
    // it only while callbacks succeed, so a request each frame keeps it on.
    let session = frame.as_ref().map_or(0, |frame| frame.session);
    apply_fullbright(SETTINGS.with(StdCell::get).fullbright, session);
    HEAVY_FRAME.with(|heavy| heavy.set(false));
    if MENU_STALE.with(StdCell::get) {
        MENU_STALE.with(|stale| stale.set(false));
        HEAVY_FRAME.with(|heavy| heavy.set(true));
        let settings = SETTINGS.with(StdCell::get);
        let state = MENU.with(StdCell::get);
        if let Err(error) = host_panel::update_content(&panel::update(&settings, seconds, state)) {
            show_label(&format!(
                "Boldy Mods: allow the Controls permission for the menu ({error})"
            ));
        }
    }

    let menu_open = controls.as_ref().is_some_and(|c| c.panel_open);
    MENU_OPEN.with(|open| open.set(menu_open));
    // Holding attack in gameplay mines the block; the fill would cover its cracks.
    refresh_attack_keys(seconds);
    let mining = controls.as_ref().is_some_and(|c| {
        !c.panel_open
            && ATTACK_KEYS.with(|keys| {
                let keys = &keys.borrow().1;
                c.keys_admitted.iter().any(|held| {
                    if keys.is_empty() {
                        held == "MouseLeft"
                    } else {
                        keys.contains(held)
                    }
                })
            })
    });
    if let Some(controls) = &controls {
        handle_menu_key(controls, seconds);
        back_to_start_when_closed(controls, seconds);
        expire_confirm(seconds);
        hover_help(controls, seconds);
        // Committed hex codes arrive separately from the numeric control events.
        let text = if menu_open {
            input::read_text_events().unwrap_or_default()
        } else {
            Vec::new()
        };
        apply_menu_events(controls, &text, seconds);
    }
    // Before the early returns below: the host keeps the counter on screen, patched here.
    // One read of the HUD's size for both elements.
    let hud_size = hud::read_viewport()
        .filter(|v| {
            v.width.is_finite()
                && v.height.is_finite()
                && v.scale.is_finite()
                && v.width > 0.0
                && v.height > 0.0
        })
        .map(|v| [v.width, v.height, v.scale]);
    arrow_counter(controls.as_ref(), seconds, hud_size);
    cps_frame(controls.as_ref(), seconds, hud_size);
    zoom_frame(controls.as_ref(), delta);
    let settings = SETTINGS.with(StdCell::get);
    if menu_open
        && (settings.rainbow || settings.mob.rainbow)
        && seconds - PREVIEW_AT.with(StdCell::get) >= PREVIEW_SECONDS
    {
        PREVIEW_AT.with(|at| at.set(seconds));
        let state = MENU.with(StdCell::get);
        let _ = host_panel::update_content(&panel::preview_update(&settings, seconds, state));
    }

    if let Some(frame) = frame {
        let eye = [
            f64::from(frame.eye.x),
            f64::from(frame.eye.y),
            f64::from(frame.eye.z),
        ];
        // Re-upload lost geometry first, so that frame skips the terrain read.
        ensure_cube(seconds);
        let dir = outline::look_direction(frame.yaw, frame.pitch);
        if !HEAVY_FRAME.with(StdCell::get) {
            refresh_terrain(eye, dir, settings.reach, frame.session);
        }
        // Reading a crowd (up to 16 players on a server hub) costs a fair share of a
        // frame; after a costly terrain read, the last frame's actors do.
        let observed = if budget::remaining_fuel() >= ACTOR_RESERVE {
            nearby_actor_boxes(eye)
        } else {
            LAST_ACTORS.with(|last| std::mem::take(&mut *last.borrow_mut()))
        };
        // Cinnabar says which actor its crosshair would hit (attack reach, blocks in the
        // way); use that when known, so a mob is only aimed when it can really be hit.
        let (actors, hittable): (Vec<Vec<[f64; 6]>>, Vec<Option<bool>>) =
            observed.into_iter().unzip();
        let native_aim = hittable.iter().any(Option::is_some);
        let aimed = hittable.iter().position(|hit| *hit == Some(true));
        let hit = aimed_target(
            frame.session,
            frame.dimension,
            [eye, dir],
            settings.reach,
            native_aim.then(|| aimed.map(|index| actors[index].as_slice())),
            &actors,
        )
        .or_else(|| last_block_while_loading(eye, dir, settings.reach, frame.dimension));
        let camera = gameplay::read_camera().ok().flatten().map_or(eye, |pose| {
            [
                f64::from(pose.position.x),
                f64::from(pose.position.y),
                f64::from(pose.position.z),
            ]
        });
        let mobs = if settings.show_all_mobs {
            nearest_boxes(&actors, eye)
        } else {
            Vec::new()
        };
        // Kept for a frame with too little fuel to read them again (moved, not copied).
        LAST_ACTORS.with(|last| *last.borrow_mut() = actors.into_iter().zip(hittable).collect());
        LAST_HIT.with(|last| {
            *last.borrow_mut() = Some(Target {
                hit,
                mobs,
                dimension: frame.dimension,
                camera,
            });
        });
    } else if !menu_open {
        // Another screen owns input; show nothing until gameplay resumes.
        return;
    }
    // Like Cinnabar's own selection, hide with the HUD (F1). An open menu of ours also hides
    // the HUD, but there the outline is the live preview.
    if !menu_open && hud_hidden() {
        return;
    }

    LAST_HIT.with(|last| {
        if let Some(target) = last.borrow().as_ref() {
            let shapes = shapes(target, &settings);
            if !shapes.is_empty() {
                draw_outlines(&shapes, target.dimension, target.camera, seconds, mining);
            }
        }
    });
}

/// Fuel a frame may use before the mod says so: the host allows 200,000 and quarantines the mod
/// after three frames that run out.
const FUEL_WARN: u64 = 150_000;
/// How often the menu's fuel line is refreshed, in seconds.
const FUEL_SHOWN_EVERY: f32 = 1.0;

/// Notes the fuel the frame that started with `start` left over: the start page shows the last
/// and the busiest frame, and a frame near the limit is reported once. A frame has little fuel,
/// so this watches it rather than letting a trap be the first sign.
fn note_fuel(start: u64) {
    let used = start.saturating_sub(budget::remaining_fuel());
    let peak = FUEL_PEAK.with(|peak| {
        peak.set(peak.get().max(used));
        peak.get()
    });
    if used > FUEL_WARN {
        report_once("Boldy Mods: a frame used over 150k of its 200k fuel");
    }
    let seconds = SECONDS.with(StdCell::get);
    let mut state = MENU.with(StdCell::get);
    if state.page != panel::Page::Home
        || !MENU_OPEN.with(StdCell::get)
        || seconds - FUEL_SHOWN_AT.with(StdCell::get) < FUEL_SHOWN_EVERY
    {
        return;
    }
    FUEL_SHOWN_AT.with(|at| at.set(seconds));
    let thousands = |fuel: u64| u16::try_from(fuel / 1000).unwrap_or(u16::MAX);
    state.fuel_k = [thousands(used), thousands(peak)];
    MENU.with(|menu| menu.set(state));
    let _ = host_panel::update_content(&panel::fuel_update(state));
}

/// What the crosshair is on this frame, from the last terrain sample, and whether it is a mob
/// (`true`) or a block. `aimed` is `Some` when Cinnabar says which actor it would hit (the boxes
/// of that actor, or `None` for none); without it every actor in `actors` can be the target.
fn aimed_target(
    session: u64,
    dimension: i32,
    [eye, dir]: [[f64; 3]; 2],
    reach: f32,
    aimed: Option<Option<&[[f64; 6]]>>,
    actors: &[Vec<[f64; 6]>],
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
                outline::target_with_actors(eye, dir, reach, lookup, &[])
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
fn bounds_of(boxes: &[[f64; 6]]) -> Option<([f64; 3], [f64; 3])> {
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
fn last_block_while_loading(
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

/// The menu opens on its start page: once closed, forget the page that was shown.
fn back_to_start_when_closed(controls: &input::Controls, seconds: f32) {
    let state = MENU.with(StdCell::get);
    if controls.panel_open
        || (state.page == panel::Page::Home
            && !state.colour_open
            && state.typing.is_none()
            && state.confirm == panel::Reset::None
            && state.focus.is_none()
            && !state.moving)
    {
        return;
    }
    let state = MenuState {
        page: panel::Page::Home,
        colour_open: false,
        editing_fill: false,
        editing_aim: false,
        typing: None,
        confirm: panel::Reset::None,
        focus: None,
        moving: false,
        moving_cps: false,
        ..state
    };
    MENU.with(|m| m.set(state));
    push_menu(seconds, state);
}

/// Captures a new menu key while the "Menu key" button waits for one; otherwise opens or closes
/// the menu when the chosen key is pressed. The panel's own toggle key is a placeholder, because
/// a panel cannot change its toggle key after loading.
fn handle_menu_key(controls: &input::Controls, seconds: f32) {
    let state = MENU.with(StdCell::get);
    if state.capturing_fullbright {
        capture_optional_key(KeyButton::Fullbright, controls, seconds);
        return;
    }
    if state.capturing_zoom {
        capture_optional_key(KeyButton::Zoom, controls, seconds);
        return;
    }
    if state.capturing {
        if !controls.panel_open {
            // The menu closed mid-capture; keep the old key.
            set_capturing(KeyButton::Menu, false, seconds);
            return;
        }
        let Some(key) = controls
            .keys_pressed
            .iter()
            .find(|key| !key.starts_with("Mouse"))
        else {
            return;
        };
        if key != "Escape"
            && let Some(name) = KeyName::new(key)
        {
            let mut settings = SETTINGS.with(StdCell::get);
            settings.menu_key = name;
            SETTINGS.with(|s| s.set(settings));
            save_settings(&settings);
            reserve_keys(&settings);
        }
        set_capturing(KeyButton::Menu, false, seconds);
        return;
    }
    // Keys are being typed into a box: the menu key must not close the menu.
    if state.typing.is_some() {
        return;
    }
    toggle_fullbright_by_key(controls);
    if !controls.focused {
        return;
    }
    let key = SETTINGS.with(StdCell::get).menu_key;
    if !controls
        .keys_pressed
        .iter()
        .any(|pressed| pressed == key.as_str())
    {
        return;
    }
    if controls.panel_open {
        let _ = host_panel::close();
    } else if controls.gameplay
        && let Err(error) = host_panel::open()
    {
        report_once(&format!("Boldy Mods: menu unavailable ({error})"));
    }
}

/// Saves `settings`, telling the player once if that fails.
fn save_settings(settings: &Settings) {
    if let Err(error) = settings::save(&settings.to_json()) {
        report_once(&format!("Boldy Mods: settings not saved ({error})"));
    }
}

/// Sends the menu for `state` to the host. A frame that did this is a heavy one, which leaves the
/// terrain read for the next. A failure is ignored: the menu may just have closed.
fn push_menu(seconds: f32, state: MenuState) {
    push_menu_with(seconds, &SETTINGS.with(StdCell::get), state, false);
}

/// [`push_menu`] for `settings`, telling the player once when `report` and the host refused.
fn push_menu_with(seconds: f32, settings: &Settings, state: MenuState, report: bool) {
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    if let Err(error) = host_panel::update_content(&panel::update(settings, seconds, state))
        && report
    {
        report_once(&format!("Boldy Mods: menu update failed ({error})"));
    }
}

/// The three buttons that wait for a key.
#[derive(Clone, Copy, PartialEq)]
enum KeyButton {
    Menu,
    Fullbright,
    Zoom,
}

/// Starts or ends waiting for a new key on `button` and redraws the menu for it.
fn set_capturing(button: KeyButton, capturing: bool, seconds: f32) {
    let state = MENU.with(|menu| {
        let mut state = menu.get();
        match button {
            KeyButton::Menu => state.capturing = capturing,
            KeyButton::Fullbright => state.capturing_fullbright = capturing,
            KeyButton::Zoom => state.capturing_zoom = capturing,
        }
        menu.set(state);
        state
    });
    push_menu(seconds, state);
}

/// Waits for the new key of an optional key button (Fullbright or Zoom): Backspace or Delete
/// unbinds it, Escape cancels, the menu key is refused (the old key stays), any other key becomes
/// the key.
fn capture_optional_key(button: KeyButton, controls: &input::Controls, seconds: f32) {
    if !controls.panel_open {
        set_capturing(button, false, seconds);
        return;
    }
    let mut settings = SETTINGS.with(StdCell::get);
    let key = match button {
        KeyButton::Zoom => &mut settings.zoom.key,
        _ => &mut settings.fullbright_key,
    };
    let Some(chosen) = zoom::captured_key(&controls.keys_pressed, *key, settings.menu_key) else {
        return;
    };
    if chosen != *key {
        *key = chosen;
        SETTINGS.with(|s| s.set(settings));
        save_settings(&settings);
        reserve_keys(&settings);
    }
    set_capturing(button, false, seconds);
}

/// Shows a short explanation next to the pointer once it has rested on a control for
/// `HOVER_SECONDS`, and hides it when the pointer moves off or the menu closes.
fn hover_help(controls: &input::Controls, seconds: f32) {
    // In gameplay, which is most of the time, there is nothing to look up or to hide.
    if !controls.panel_open
        && HOVER.with(StdCell::get).is_none()
        && TIP_SHOWN.with(StdCell::get).is_none()
    {
        return;
    }
    let state = MENU.with(StdCell::get);
    let matching = panel::fill_matches(&SETTINGS.with(StdCell::get), state);
    // Which control the pointer is on only changes when the pointer, the menu's state or the
    // fill's mode do, so the host's layout is not read again while the pointer rests.
    let lookup = (
        controls
            .pointer
            .filter(|_| controls.panel_open && !state.capturing),
        state,
        matching,
    );
    let cached = HOVER_LOOKUP.with(StdCell::get);
    let (point, under) = match cached {
        Some((key, point, under)) if key == lookup => (point, under),
        _ => {
            let point = lookup.0.and_then(|(x, y)| {
                let layout = host_panel::read_layout()?;
                panel::card_point(
                    [x, y],
                    [layout.width, layout.height],
                    [layout.offset.0, layout.offset.1],
                    layout.scale,
                )
            });
            let under = point.and_then(|point| panel::tip_at(point, state, matching));
            HOVER_LOOKUP.with(|last| last.set(Some((lookup, point, under))));
            (point, under)
        }
    };
    // The highlight follows the pointer at once; the help text waits for a short rest.
    if HOVER.with(StdCell::get).map(|(index, _)| index) != under {
        let _ = host_panel::update_content(&panel::highlight_update(under, state, matching));
    }
    let since = match (HOVER.with(StdCell::get), under) {
        (Some((previous, since)), Some(now)) if previous == now => Some(since),
        (_, Some(_)) => Some(seconds),
        (_, None) => None,
    };
    HOVER.with(|h| h.set(under.zip(since)));
    let show = under.filter(|_| since.is_some_and(|since| seconds - since >= HOVER_SECONDS));
    let wanted = show.zip(point);
    let shown = TIP_SHOWN.with(StdCell::get);
    let moved = match (shown, wanted) {
        (Some((a, p)), Some((b, q))) => a != b || (p[0] - q[0]).abs() + (p[1] - q[1]).abs() > 2.0,
        (None, None) => false,
        _ => true,
    };
    if moved {
        TIP_SHOWN.with(|t| t.set(wanted));
        let _ = host_panel::update_content(&panel::tip_update(
            show,
            point.unwrap_or_default(),
            state,
            matching,
        ));
    }
}

/// How often the menu's rainbow swatches are sent a new colour: smooth enough at twenty a second.
const PREVIEW_SECONDS: f32 = 0.05;

/// The arrow keys, in the order `ARROW_REPEAT` counts them.
const ARROWS: [&str; 4] = ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"];
/// A held arrow key repeats after this many seconds, then every `ARROW_REPEAT_EVERY`.
const ARROW_REPEAT_DELAY: f32 = 0.4;
const ARROW_REPEAT_EVERY: f32 = 0.06;

/// The arrow key to act on again because it is being held. The host only sends a key's first
/// press, so holding Left or Right on a slider would otherwise move it once.
fn arrow_repeat(controls: &input::Controls, seconds: f32) -> Option<&'static str> {
    let Some(held) = ARROWS
        .iter()
        .position(|arrow| controls.keys_held.iter().any(|key| key == arrow))
    else {
        ARROW_REPEAT.with(|repeat| repeat.set((0, 0.0)));
        return None;
    };
    let tag = held as u8 + 1;
    let (current, next) = ARROW_REPEAT.with(StdCell::get);
    let fresh = controls.keys_pressed.iter().any(|key| key == ARROWS[held]);
    if fresh || current != tag {
        // A new press already counts once; the repeat starts after a short wait.
        ARROW_REPEAT.with(|repeat| repeat.set((tag, seconds + ARROW_REPEAT_DELAY)));
        return None;
    }
    if seconds < next {
        return None;
    }
    ARROW_REPEAT.with(|repeat| repeat.set((tag, seconds + ARROW_REPEAT_EVERY)));
    Some(ARROWS[held])
}

/// Takes back a reset button's "Sure? Click again" once `CONFIRM_SECONDS` have passed.
fn expire_confirm(seconds: f32) {
    let mut state = MENU.with(StdCell::get);
    if state.confirm == panel::Reset::None
        || seconds - CONFIRM_AT.with(StdCell::get) < panel::CONFIRM_SECONDS
    {
        return;
    }
    state.confirm = panel::Reset::None;
    MENU.with(|menu| menu.set(state));
    push_menu(seconds, state);
}

/// Applies the menu's control events and key presses, then saves and refreshes the menu once if
/// anything changed.
fn apply_menu_events(controls: &input::Controls, text: &[input::TextEvent], seconds: f32) {
    let events = &controls.events;
    let mut state = MENU.with(StdCell::get);
    if !controls.panel_open && state.typing.is_some() {
        // The menu closed mid-typing: keep the old value.
        state.typing = None;
        MENU.with(|menu| menu.set(state));
    }
    if !controls.panel_open && events.is_empty() && text.is_empty() {
        return;
    }
    let typing = state.typing.is_some() && !state.capturing;
    // The keys that count this frame: those pressed, and an arrow key that is being held.
    let mut pressed = controls.keys_pressed.clone();
    if controls.panel_open && !typing {
        pressed.extend(arrow_repeat(controls, seconds).map(str::to_owned));
    }
    let keys = controls.panel_open && !pressed.is_empty();
    if events.is_empty() && text.is_empty() && !keys {
        return;
    }
    let before = SETTINGS.with(StdCell::get);
    let confirm_before = state.confirm;
    let mut settings = before;
    // A rejected hex code still refreshes the menu, putting the field back to the current code.
    let mut refresh = false;
    // Keys go to the box being typed into before clicks, so a click on another box first
    // applies what was typed here.
    if typing {
        for key in &controls.keys_pressed {
            refresh |= panel::type_key(&mut settings, &mut state, key);
        }
    }
    // A click puts the keyboard focus away; otherwise the keys move or press the focused control.
    let mut queue: Vec<(String, f32)> = Vec::new();
    let mut moved = false;
    if !events.is_empty() && state.focus.take().is_some() {
        refresh = true;
    }
    if keys && !typing {
        let shift = controls
            .keys_held
            .iter()
            .any(|key| key == "ShiftLeft" || key == "ShiftRight");
        let keyed = panel::keys(&settings, &mut state, &pressed, shift);
        queue = keyed.events;
        moved = keyed.moved;
        refresh |= keyed.redraw;
    }
    queue.extend(events.iter().map(|event| (event.id.clone(), event.value)));
    for (id, value) in &queue {
        match id.as_str() {
            "close" => {
                // The X button: the menu's state resets to the start page once it has closed.
                let _ = host_panel::close();
            }
            // A key button: wait for the next key press.
            id @ ("zoom_key" | "fullbright_key" | "menu_key") => {
                let button = match id {
                    "zoom_key" => KeyButton::Zoom,
                    "fullbright_key" => KeyButton::Fullbright,
                    _ => KeyButton::Menu,
                };
                set_capturing(button, true, seconds);
                state = MENU.with(StdCell::get);
            }
            _ => refresh |= panel::apply(&mut settings, &mut state, id, *value),
        }
    }
    if state.confirm != panel::Reset::None && state.confirm != confirm_before {
        CONFIRM_AT.with(|at| at.set(seconds));
    }
    for event in text {
        match panel::apply_text(&mut settings, state, &event.id, &event.value) {
            Some(applied) => refresh |= applied,
            None => refresh = true,
        }
    }
    MENU.with(|menu| menu.set(state));
    let changed = settings != before;
    if !changed && !refresh {
        if moved {
            // Only the focus ring moved: a small patch is enough.
            let matching = panel::fill_matches(&settings, state);
            let _ = host_panel::update_content(&panel::focus_update(state, matching));
        }
        return;
    }
    if changed {
        SETTINGS.with(|s| s.set(settings));
        if settings.zoom != before.zoom {
            // The zoom key is reserved only while the zoom is on.
            reserve_keys(&settings);
        }
        save_settings(&settings);
    }
    push_menu_with(seconds, &settings, state, true);
    // The update hid the highlight and help; forget them so the next frame shows them again
    // where they now belong (opening the colour tools moves every settings control).
    HOVER.with(|h| h.set(None));
    TIP_SHOWN.with(|t| t.set(None));
}

/// The zoom, once per frame: the zoom key (held, or pressed to toggle) narrows the view with an
/// ease, scrolling (read from the hotbar slot moving) or the = and - keys change how far, and
/// the camera is asked for the view and look scales every frame it is zoomed; omitting them
/// restores the normal view. Works only in focused gameplay with no menu open.
fn zoom_frame(controls: Option<&input::Controls>, delta: f32) {
    let zoom = SETTINGS.with(StdCell::get).zoom;
    let gameplay = controls.is_some_and(|c| c.focused && c.gameplay && !c.panel_open);
    if !(zoom.on && gameplay) {
        ZOOM_TOGGLED.with(|t| t.set(false));
        ZOOM_SLOT.with(|s| s.set(None));
        // A menu or another screen restores the view by itself: start the next zoom from 1.
        ZOOM.with(|engine| {
            if engine.get().divisor > 1.0 {
                engine.set(zoom::Engine::default());
            }
        });
        return;
    }
    let Some(controls) = controls else { return };
    // Without a key (unbound) there is nothing to hold or press, so the zoom never starts.
    let key = zoom.key.as_ref().map_or("", KeyName::as_str);
    let active = if key.is_empty() {
        ZOOM_TOGGLED.with(|t| t.set(false));
        false
    } else if zoom.toggle {
        if controls.keys_pressed.iter().any(|k| k == key) {
            ZOOM_TOGGLED.with(|t| t.set(!t.get()));
        }
        ZOOM_TOGGLED.with(StdCell::get)
    } else {
        ZOOM_TOGGLED.with(|t| t.set(false));
        controls.keys_held.iter().any(|k| k == key)
    };
    let mut steps = 0;
    if active {
        for pressed in &controls.keys_pressed {
            match pressed.as_str() {
                "Equal" | "NumpadAdd" => steps += 1,
                "Minus" | "NumpadSubtract" => steps -= 1,
                _ => {}
            }
        }
        if zoom.scroll {
            steps += scroll_from_hotbar(controls);
        }
    } else {
        ZOOM_SLOT.with(|s| s.set(None));
    }
    let mut engine = ZOOM.with(StdCell::get);
    if !active && engine.divisor <= 1.0 {
        return;
    }
    let output = engine.tick(&zoom, active, steps, delta);
    ZOOM.with(|e| e.set(engine));
    if output.applying
        && let Err(error) = camera::set_view_scale(output.fov_scale, output.look_scale)
    {
        report_once(&format!(
            "Boldy Mods: allow the Camera rotation permission for zoom ({error})"
        ));
    }
}

/// The scroll notches since the last frame while zooming, positive zooming in, from the
/// selected hotbar slot moving: a mod cannot read the wheel itself during gameplay.
fn scroll_from_hotbar(controls: &input::Controls) -> i32 {
    let Some(now) = player_state::read_selected_slot()
        .ok()
        .flatten()
        .map(|selected| selected.slot)
    else {
        return 0;
    };
    let last = ZOOM_SLOT.with(|slot| slot.replace(Some(now)));
    // A number key jumps straight to a slot: that is not a scroll.
    let jumped = controls.keys_pressed.iter().any(|key| {
        key.strip_prefix("Digit")
            .or_else(|| key.strip_prefix("Numpad"))
            .is_some_and(|rest| rest.len() == 1 && rest != "0")
    });
    match last {
        Some(last) if !jumped => zoom::scroll_steps(last, now),
        _ => 0,
    }
}

/// Asks Cinnabar for fullbright every frame while it is on (one cheap call), and switches it off
/// once per change or world. A refusal (the Fullbright permission not allowed) is shown once
/// per change, not every frame.
fn apply_fullbright(enabled: bool, session: u64) {
    let changed = FULLBRIGHT_SENT.with(StdCell::get) != Some((enabled, session));
    if !enabled && !changed {
        return;
    }
    FULLBRIGHT_SENT.with(|sent| sent.set(Some((enabled, session))));
    if let Err(error) = environment::set_fullbright(enabled)
        && enabled
        && changed
    {
        show_label(&format!(
            "Boldy Mods: allow the Fullbright permission for fullbright ({error})"
        ));
    }
}

/// Reserves the menu key and the fullbright key, so the game does not act on them too (such as
/// E opening the inventory).
fn reserve_keys(settings: &Settings) {
    let mut keys = vec![settings.menu_key.as_str().to_owned()];
    keys.extend(settings.fullbright_key.map(|key| key.as_str().to_owned()));
    if settings.zoom.on
        && let Some(key) = settings.zoom.key
    {
        keys.push(key.as_str().to_owned());
    }
    let _ = input::reserve_keys(&keys);
}

/// Switches fullbright when its key is pressed in game or with the menu open, with a message.
fn toggle_fullbright_by_key(controls: &input::Controls) {
    let mut settings = SETTINGS.with(StdCell::get);
    let Some(key) = settings.fullbright_key else {
        return;
    };
    if !controls.focused
        || !controls
            .keys_pressed
            .iter()
            .any(|pressed| pressed == key.as_str())
    {
        return;
    }
    settings.fullbright = !settings.fullbright;
    SETTINGS.with(|s| s.set(settings));
    save_settings(&settings);
    show_label(if settings.fullbright {
        "Fullbright: On"
    } else {
        "Fullbright: Off"
    });
    // An open menu shows the new state on its next frame.
    MENU_STALE.with(|stale| stale.set(true));
}

/// How often to check whether the player changed their key bindings.
const BINDINGS_SECONDS: f32 = 2.0;

/// Re-reads the player's attack keys when their bindings changed, checking at most every
/// `BINDINGS_SECONDS`. Reads at most one page per frame, because control reads are capped per
/// callback and going over stops the mod. Keeps the previous keys when they cannot be read.
fn refresh_attack_keys(seconds: f32) {
    let limit = mod_api::MAX_INPUT_BINDINGS_PER_READ as u32;
    let pending = BINDINGS_READ.with(|read| read.borrow_mut().take());
    let (revision, offset, mut keys, page) = match pending {
        Some((revision, offset, keys)) => match input::read_bindings(None, offset, limit) {
            Ok(Some(page)) if page.revision == revision => (revision, offset, keys, page),
            // Changed or unreadable mid-read: start again at the next check.
            _ => return,
        },
        None => {
            if seconds < BINDINGS_CHECK_AT.with(StdCell::get) {
                return;
            }
            BINDINGS_CHECK_AT.with(|at| at.set(seconds + BINDINGS_SECONDS));
            let known = ATTACK_KEYS.with(|keys| keys.borrow().0);
            let Ok(Some(page)) = input::read_bindings(known, 0, limit) else {
                return;
            };
            (page.revision, 0, Vec::new(), page)
        }
    };
    let next = offset + page.entries.len() as u32;
    let total = page.total;
    keys.extend(
        page.entries
            .into_iter()
            .filter(|binding| {
                binding.action == "attack" && binding.context == input::BindingContext::Gameplay
            })
            .map(|binding| binding.key),
    );
    if next < total && next > offset {
        BINDINGS_READ.with(|read| *read.borrow_mut() = Some((revision, next, keys)));
    } else {
        ATTACK_KEYS.with(|stored| *stored.borrow_mut() = (Some(revision), keys));
    }
}

/// The mob hitboxes within `MOB_RANGE` of `eye`, nearest first, at most `MAX_MOB_BOXES`.
fn nearest_boxes(actors: &[Vec<[f64; 6]>], eye: [f64; 3]) -> Vec<[[f64; 3]; 2]> {
    let mut boxes: Vec<(f64, [[f64; 3]; 2])> = actors
        .iter()
        .flatten()
        .map(|b| {
            let (min, max) = ([b[0], b[1], b[2]], [b[3], b[4], b[5]]);
            // Distance from the eye to the nearest point of the box.
            let gap: f64 = (0..3)
                .map(|axis| {
                    (min[axis] - eye[axis])
                        .max(eye[axis] - max[axis])
                        .max(0.0)
                        .powi(2)
                })
                .sum();
            (gap, [min, max])
        })
        .filter(|(gap, _)| *gap <= MOB_RANGE * MOB_RANGE)
        .collect();
    boxes.sort_by(|a, b| a.0.total_cmp(&b.0));
    boxes
        .into_iter()
        .take(MAX_MOB_BOXES)
        .map(|(_, b)| b)
        .collect()
}

/// Everything to draw this frame: other nearby mobs in the mob look, then the aimed block in
/// the block look or the aimed mob in the aimed colour (or the mob look when only the aimed mob
/// is outlined).
fn shapes(target: &Target, settings: &Settings) -> Vec<Shape> {
    let mobs = settings.for_mobs();
    let aimed = target.hit.as_ref().map(|(hit, actor)| {
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
fn nearby_actor_boxes(eye: [f64; 3]) -> Observed {
    let around_eye = |actor: &player_state::Actor| {
        actor.boxes.iter().any(|b| {
            let (min, max) = ([b.min.0, b.min.1, b.min.2], [b.max.0, b.max.1, b.max.2]);
            (0..3)
                .all(|axis| f64::from(min[axis]) <= eye[axis] && eye[axis] <= f64::from(max[axis]))
        })
    };
    crate::arena::scratch(player_state::read_actors, |actors| {
        actors
            .as_ref()
            .map(|actors| boxes_of(actors, &around_eye))
            .unwrap_or_default()
    })
}

/// Each pickable, visible actor's hitbox as `[min x, min y, min z, max x, max y, max z]`: one
/// box around all of its boxes, as the game shows it. Keeping every part box made crowds (16
/// players on a server hub) cost a large share of the frame's fuel.
fn boxes_of(
    actors: &[player_state::Actor],
    around_eye: &dyn Fn(&player_state::Actor) -> bool,
) -> Vec<(Vec<[f64; 6]>, Option<bool>)> {
    actors
        .iter()
        // A box around the eye is the player's own body, not something to outline.
        .filter(|actor| actor.pickable && !actor.invisible && !around_eye(actor))
        .filter_map(|actor| {
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
            bounds[0]
                .is_finite()
                .then(|| (vec![bounds.map(f64::from)], actor.hittable))
        })
        .collect()
}

/// Whether Cinnabar hides its HUD (F1, or the death screen): it then reports every vanilla HUD
/// element as not drawn. Needs the HUD permission; without it the outline always shows.
/// Checked about five times a second, not every frame: the reply lists every HUD element by
/// name, which costs fuel to read.
fn hud_hidden() -> bool {
    let seconds = SECONDS.with(StdCell::get);
    let (hidden, checked_at) = HUD_HIDDEN.with(StdCell::get);
    if (seconds - checked_at).abs() < 0.2 {
        return hidden;
    }
    let hidden = crate::arena::scratch(hud::read_elements, |elements| {
        elements.as_ref().is_ok_and(|elements| {
            !elements.is_empty() && elements.iter().all(|element| !element.visible)
        })
    });
    HUD_HIDDEN.with(|cell| cell.set((hidden, seconds)));
    hidden
}

/// Lifts the lines just off the block faces so the depth test does not let the faces cut into
/// them, which makes thin lines shimmer.
const INFLATE: f64 = 0.003;

/// Uploads the cube while it is missing, at most four times a second: drawing is refused before a world
/// is joined, and a dimension change or loading gap invalidates every handle (the host bumps its
/// drawing generation) without running `init` again.
fn ensure_cube(seconds: f32) {
    let missing = CUBE.with(|cube| cube.borrow().len() != outline::EDGES.len())
        || FACE_QUADS.with(|faces| faces.borrow().len() != outline::FACES.len());
    if !missing || seconds < UPLOAD_RETRY_AT.with(StdCell::get) {
        return;
    }
    UPLOAD_RETRY_AT.with(|at| at.set(seconds + 0.25));
    HEAVY_FRAME.with(|heavy| heavy.set(true));
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

fn draw_outlines(shapes: &[Shape], dimension: i32, camera: [f64; 3], seconds: f32, mining: bool) {
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
                    if drawn > 0 && budget::remaining_fuel() < needed {
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
fn segments_for(boxes: &outline::Boxes) -> Vec<outline::Segment> {
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
fn push_shape(
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
fn refresh_terrain(eye: [f64; 3], dir: [f64; 3], reach: f32, frame_session: u64) {
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
        if (budget::remaining_fuel() as f64) < needed {
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

fn known(shape: &ResolvedShape) -> Option<u32> {
    match shape {
        ResolvedShape::Known(index) => Some(*index),
        ResolvedShape::Unavailable => None,
    }
}

fn shape_boxes(blocks: &Blocks, shape: u32) -> Vec<[f64; 6]> {
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
    // FNV-1a: cheap, and enough to tell problems apart.
    let hash = text.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
    });
    if REPORTED_ERROR.with(|r| r.replace(hash)) != hash {
        show_label(text);
    }
}

mod_api::bindings::export!(BlockOutline with_types_in mod_api::bindings);
