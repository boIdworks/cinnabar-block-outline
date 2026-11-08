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
use crate::crosshair;
use crate::fps;
use crate::hudedit;
use crate::jumpreset;
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
type Observed = (Vec<outline::ActorBox>, Vec<Option<bool>>);
/// A hover lookup: its inputs, the pointer on the card and the control under it.
type HoverLookup = (
    (Option<(f32, f32)>, MenuState, bool),
    Option<[f32; 2]>,
    Option<usize>,
);

mod fuel;
mod hud_elements;
mod menu;
mod world;
use fuel::*;
use hud_elements::{
    arrow_counter, cps_frame, crosshair_frame, draw_grid_frame, fps_frame, jump_reset_frame,
};
use menu::*;
use world::*;

/// A copy of all the settings. That is nearly 400 bytes, a few hundred fuel to copy, so code that
/// runs every frame reads only the fields it needs with [`settings_with`].
fn settings() -> Settings {
    SETTINGS.with(|settings| *settings.borrow())
}

/// Replaces the settings.
fn set_settings(settings: Settings) {
    SETTINGS.with(|current| *current.borrow_mut() = settings);
}

/// Reads from the settings in place, copying only what `read` returns.
fn settings_with<R>(read: impl FnOnce(&Settings) -> R) -> R {
    SETTINGS.with(|settings| read(&settings.borrow()))
}

/// Everything the CPS readout depends on: the click counts, the HUD's size, the settings and the
/// edit frame. While none of it changes (and the readout is up) nothing is worked out again.
type CpsKey = (
    (u32, Option<u32>),
    Option<[f32; 2]>,
    cps::Cps,
    Option<counter::Edit>,
);

/// A changed set of bindings being read: its revision, the next offset, and the attack keys and
/// jump keys found so far.
type BindingsRead = (u64, u32, Vec<String>, Vec<String>);

/// What the HUD layer has of a crosshair or the drawing grid: the header's hash (none sent
/// yet) and each part's (none: still empty, as the layer starts).
type Sent = (Option<u64>, [Option<u64>; crosshair::PARTS]);
/// Where the drawing grid's highlight is: the pixel (row, column), Mirror, the resolution.
type GridCursor = (Option<(usize, usize)>, bool, usize);
/// The drawing and colour the menu's grid shows, and the plan worked out for them.
type GridPlan = ((crosshair::Drawing, [u8; 3]), crosshair::Plan);
/// A crosshair's view, and the shape and plan worked out for it.
type CrosshairPlan = (crosshair::View, crosshair::Shape, crosshair::Plan);
const NOTHING_SENT: Sent = (None, [None; crosshair::PARTS]);

/// The latest jump reset timing verdict and knockback outcome, with when each was made.
type JumpVerdicts = (
    Option<(jumpreset::Timing, f32)>,
    Option<(jumpreset::Outcome, f32)>,
);

thread_local! {
    static SETTINGS: RefCell<Settings> = RefCell::new(Settings::default());
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
    static MENU: StdCell<MenuState> = const { StdCell::new(MenuState { capturing: false, capturing_fullbright: false, capturing_zoom: false, editing_fill: false, editing_aim: false, page: panel::Page::Home, colour_open: false, typing: None, confirm: panel::Reset::None, focus: None, moving: false, moving_cps: false, moving_fps: false, guide_bottom: false, fuel_k: [0, 0], drawing: false, draw_full: false }) };
    /// The player's attack keys (`MouseLeft` until read) and the bindings revision they came
    /// from, re-read every `BINDINGS_SECONDS`.
    static ATTACK_KEYS: RefCell<(Option<u64>, Vec<String>)> = const { RefCell::new((None, Vec::new())) };
    static BINDINGS_CHECK_AT: StdCell<f32> = const { StdCell::new(0.0) };
    /// A changed set of bindings being read, one page per frame: its revision, the next offset
    /// and the attack keys found so far.
    static BINDINGS_READ: RefCell<Option<BindingsRead>> = const { RefCell::new(None) };
    /// The player's jump keys, read with the attack keys; empty until read (Space is assumed).
    static JUMP_KEYS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// Jump reset: jumps and hits waiting for each other, the combat feed's position and the
    /// session they belong to.
    static JUMP_RESET: StdCell<(jumpreset::Tracker, jumpreset::Feed, u64)> =
        const { StdCell::new((jumpreset::Tracker::new(), jumpreset::Feed::new(), 0)) };
    /// Your movement after a hit, watched for knockback.
    static JUMP_WATCH: StdCell<jumpreset::Watch> = const { StdCell::new(jumpreset::Watch::new()) };
    /// The latest timing verdict and knockback outcome, with when each was made, in accumulated
    /// seconds.
    static JUMP_VERDICT: StdCell<JumpVerdicts> = const { StdCell::new((None, None)) };
    /// The last hits' recorded movement, as `knockback|jump offset|movement` text each, written
    /// to the settings file with the log.
    static JUMP_TRACES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// The last jump resets in full (the same rows as `JUMP_TRACES`), kept across restarts: read
    /// back from the settings file at start.
    static JUMP_RESETS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// A hit was added to the jump reset log and the settings file does not have it yet.
    static JUMP_LOG_UNSAVED: StdCell<bool> = const { StdCell::new(false) };
    /// What the jump reset readout last showed, so it is only patched when something changes.
    static JUMP_SHOWN: StdCell<Option<jumpreset::View>> = const { StdCell::new(None) };
    /// The fullbright state last sent to Cinnabar and the session it was sent in, so it is sent
    /// once per change and again after joining another world.
    static FULLBRIGHT_SENT: StdCell<Option<(bool, u64)>> = const { StdCell::new(None) };
    /// The last open or close the menu key asked for, and when: the host shows the result a few
    /// frames later, and pressing again in between must act on what was asked, not on the old state.
    static MENU_REQUEST: StdCell<Option<(bool, f32)>> = const { StdCell::new(None) };
    /// Whether the menu was open in the last frame that looked, and when it was last seen closing.
    static MENU_SEEN_OPEN: StdCell<bool> = const { StdCell::new(false) };
    static MENU_CLOSED_AT: StdCell<f32> = const { StdCell::new(-100.0) };
    /// The menu still shows the embedded defaults; the first frame patches in the settings.
    static MENU_STALE: StdCell<bool> = const { StdCell::new(true) };
    /// Fuel the last changed terrain region cost per block; unknown until one is read.
    static READ_COST: StdCell<Option<f64>> = const { StdCell::new(None) };
    /// The last actors read, reused by frames with too little fuel left to read them.
    static LAST_ACTORS: RefCell<Observed> = const { RefCell::new((Vec::new(), Vec::new())) };
    /// This frame already did a costly job (a menu patch or a geometry upload), so a changed
    /// terrain region, the other costly job, waits for the next frame.
    static HEAVY_FRAME: StdCell<bool> = const { StdCell::new(false) };
    /// The last merged outline worked out, by its inflated boxes: shapes with several boxes cost
    /// a little to merge, so a block's outline is only worked out when the target changes.
    static SEGMENTS: RefCell<Option<(outline::Boxes, Vec<outline::Segment>)>> = const { RefCell::new(None) };
    /// The game's HUD elements when last read: whether the whole HUD was hidden (F1), whether
    /// the game would draw its crosshair, and when they were read.
    static HUD_HIDDEN: StdCell<(bool, bool, f32)> =
        const { StdCell::new((false, true, f32::NEG_INFINITY)) };
    /// Whether this mod has the game's own crosshair hidden, in place of its custom one.
    static VANILLA_CROSSHAIR_HIDDEN: StdCell<bool> = const { StdCell::new(false) };
    /// HUD writes (surface, patches, hidden elements, label) made in this callback.
    static HUD_WRITES: StdCell<u32> = const { StdCell::new(0) };
    /// The stroke being drawn on the crosshair grid: whether it draws (left button) or erases
    /// (right), and the last pixel painted, which the next frame's line starts from.
    static DRAW_STROKE: StdCell<Option<(bool, (usize, usize))>> = const { StdCell::new(None) };
    /// The settings changed (a menu change, or a drawing stroke) and the settings file does not
    /// have them yet: saved in the first frame with the fuel for it, after a stroke ends.
    static SETTINGS_UNSAVED: StdCell<bool> = const { StdCell::new(false) };
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
    static CLICKS: RefCell<(cps::Clicks, cps::Clicks)> =
        const { RefCell::new((cps::Clicks::new(), cps::Clicks::new())) };
    /// What the CPS readout last showed, so it is only patched when something changes.
    static CPS_SHOWN: StdCell<Option<cps::View>> = const { StdCell::new(None) };
    /// What the CPS readout was last worked out from.
    static CPS_KEY: StdCell<Option<CpsKey>> = const { StdCell::new(None) };
    /// The CPS text's measured pieces, measured again when the GUI scale changes.
    static CPS_WIDTHS: StdCell<Option<cps::Widths>> = const { StdCell::new(None) };
    /// Counts frames for the FPS counter.
    static FPS: StdCell<fps::Meter> = const { StdCell::new(fps::Meter::new()) };
    /// What the FPS counter last showed, so it is only patched when something changes.
    static FPS_SHOWN: StdCell<Option<fps::View>> = const { StdCell::new(None) };
    /// The most fuel any frame has used since the mod started, and when the start page's fuel
    /// line was last refreshed.
    static FUEL_PEAK: StdCell<u64> = const { StdCell::new(0) };
    /// The fuel each feature has used (see `SECTIONS`), and when it was last saved.
    static FUEL_STATS: RefCell<[Stat; SECTIONS.len()]> =
        const { RefCell::new([Stat { sum: 0, max: 0, calls: 0 }; SECTIONS.len()]) };
    static FUEL_SAVED_AT: StdCell<f32> = const { StdCell::new(0.0) };
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
    /// The same for the FPS counter's frame.
    static EDIT_FPS: StdCell<Option<counter::Edit>> = const { StdCell::new(None) };
    /// The counter was moved or resized but not saved yet; saved once moving ends.
    static COUNTER_UNSAVED: StdCell<bool> = const { StdCell::new(false) };
    /// The zoom's easing state.
    static ZOOM: StdCell<zoom::Engine> = StdCell::new(zoom::Engine::default());
    /// Zoom mode "press to toggle": whether the zoom is switched on.
    static ZOOM_TOGGLED: StdCell<bool> = const { StdCell::new(false) };
    /// The hotbar slot last seen while zooming, to turn a scroll into zoom steps.
    static ZOOM_SLOT: StdCell<Option<u8>> = const { StdCell::new(None) };
}

thread_local! {
    /// The crosshair being shown: its view, and the shape and plan worked out for it once.
    static CROSSHAIR_PLAN: RefCell<Option<CrosshairPlan>> = const { RefCell::new(None) };
    /// What the HUD has of the crosshair, and of the menu's drawing grid: header and parts.
    static CROSSHAIR_SENT: StdCell<Sent> = const { StdCell::new(NOTHING_SENT) };
    static GRID_SENT: StdCell<Sent> = const { StdCell::new(NOTHING_SENT) };
    /// Whether the crosshair was last shown or hidden.
    static CROSSHAIR_VISIBLE: StdCell<Option<bool>> = const { StdCell::new(None) };
    /// The drawing grid's plan, by the drawing and colour it shows.
    static GRID_PLAN: RefCell<Option<GridPlan>> = const { RefCell::new(None) };
    /// The pixel the grid's highlight was last put on (none: hidden), with Mirror and the
    /// resolution: sent again only when one of them changes.
    static GRID_CURSOR: StdCell<Option<GridCursor>> = const { StdCell::new(None) };
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
/// How many of the nearest actors are read each frame (the host allows 16).
const ACTOR_LIMIT: u32 = 12;

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
        HUD_WRITES.with(|writes| writes.set(0));
        let text = settings::load();
        // The file is split into its fields once: every lookup in it used to scan the whole
        // text, which cost most of `init`'s fuel.
        let mut resets = None;
        let loaded = text
            .as_ref()
            .map(|json| {
                let fields = outline::Fields::new(json);
                resets = fields.text_span("jump_reset_resets");
                Settings::from_fields(&fields)
            })
            .map_err(String::clone);
        if let (Ok(json), Some(span)) = (text, resets) {
            restore_resets(json, span);
        }
        let settings = loaded.as_ref().copied().unwrap_or_default();
        set_settings(settings);
        // The cube is not uploaded here: `init` is short of fuel and drawing may be refused until
        // a world is joined anyway. `ensure_cube` uploads it in the first frame in a world.
        // The packer embeds the menu itself; the saved values are patched in by the first frame,
        // not here: `init` also runs when Cinnabar restarts the mod, and failing it then would
        // switch the mod off, so it stays well inside its fuel.
        reserve_keys(&settings);
        let _ = match loaded {
            Ok(_) => show_label(&format!(
                "Boldy Mods: press {} for the menu",
                settings.menu_key.display()
            )),
            Err(error) => show_label(&format!("Boldy Mods: settings unavailable ({error})")),
        };
    }

    fn frame() {
        HUD_WRITES.with(|writes| writes.set(0));
        let start = budget::remaining_fuel();
        FRAME_START.with(|at| at.set(start));
        frame_body();
        note_fuel(start);
    }
}

thread_local! {
    /// The fuel this callback started with: what [`headroom`] counts the frame's spending from.
    static FRAME_START: StdCell<u64> = const { StdCell::new(0) };
}

thread_local! {
    /// The saved resets' text as the settings file had it, until the first time they are needed
    /// (a new reset, or a save): splitting it into rows at start used a fair share of `init`.
    /// Apart from the block above, whose macro cannot take many more statics.
    static JUMP_RESETS_RAW: RefCell<Option<(String, std::ops::Range<usize>)>> = const { RefCell::new(None) };
}

/// Takes the saved jump resets back from the settings file: the file's text and where the resets
/// lie in it are only kept, and split into rows by [`resets_with`] when they are first needed, so
/// the start costs little fuel.
fn restore_resets(json: String, span: std::ops::Range<usize>) {
    if !span.is_empty() {
        JUMP_RESETS_RAW.with(|raw| *raw.borrow_mut() = Some((json, span)));
    }
}

/// Gives `use_rows` the saved jump resets, splitting the text kept by [`restore_resets`] into rows
/// first if that has not happened yet.
fn resets_with<R>(use_rows: impl FnOnce(&mut Vec<String>) -> R) -> R {
    if let Some((json, span)) = JUMP_RESETS_RAW.with(|raw| raw.borrow_mut().take()) {
        let mut rows: Vec<String> = json
            .get(span)
            .unwrap_or_default()
            .split('/')
            .filter(|row| !row.is_empty())
            .map(str::to_owned)
            .collect();
        let extra = rows.len().saturating_sub(jumpreset::RESETS_KEPT);
        rows.drain(..extra);
        JUMP_RESETS.with(|saved| *saved.borrow_mut() = rows);
    }
    JUMP_RESETS.with(|rows| use_rows(&mut rows.borrow_mut()))
}

/// One frame's work; [`Guest::frame`] measures the fuel it takes.
fn frame_body() {
    let (frame, controls) = timed(S_READS, || {
        (
            gameplay::read_frame().ok().flatten(),
            input::read_controls().ok(),
        )
    });
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
    // Every frame counts, also those that return early below.
    FPS.with(|meter| {
        let mut m = meter.get();
        m.tick(delta);
        meter.set(m);
    });
    expire_label(seconds);
    // Like Simple Fullbright, ask for fullbright first thing in every frame: the host keeps
    // it only while callbacks succeed, so a request each frame keeps it on.
    let session = frame.as_ref().map_or(0, |frame| frame.session);
    timed(S_FULLBRIGHT, || {
        apply_fullbright(settings_with(|s| s.fullbright), session);
    });
    HEAVY_FRAME.with(|heavy| heavy.set(false));
    // The menu key comes first, and a frame that opens or closes the menu does nothing else: a
    // frame that runs out of fuel is undone by the host, request and all, so a press must never
    // share a frame with the menu redraw or the outline.
    if let Some(controls) = &controls
        && handle_menu_key(controls, seconds)
    {
        return;
    }
    let menu_open = controls.as_ref().is_some_and(|c| c.panel_open);
    // The saved values are patched into the menu when it is open, so it never shows the packed
    // defaults. A closed menu is patched in a frame of its own once the HUD and the outline's
    // geometry are up: with them all in the first frame of a world it came to over twice what a
    // frame is meant to spend.
    if MENU_STALE.with(StdCell::get)
        && (menu_open || startup_done())
        && !HEAVY_FRAME.with(StdCell::get)
        && headroom() >= MENU_PATCH_RESERVE
    {
        MENU_STALE.with(|stale| stale.set(false));
        HEAVY_FRAME.with(|heavy| heavy.set(true));
        let settings = settings();
        let state = MENU.with(StdCell::get);
        if let Err(error) = host_panel::update_content(&panel::update(&settings, seconds, state)) {
            show_label(&format!(
                "Boldy Mods: allow the Controls permission for the menu ({error})"
            ));
        }
    }

    MENU_OPEN.with(|open| open.set(menu_open));
    // Holding attack in gameplay mines the block; the fill would cover its cracks.
    timed(S_KEYS, || refresh_attack_keys(seconds));
    let mining = timed(S_MINING, || {
        controls.as_ref().is_some_and(|c| {
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
        })
    });
    timed(S_MENU, || {
        if let Some(controls) = &controls {
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
            draw_frame(controls);
        }
    });
    // Before the early returns below: the host keeps the counter on screen, patched here.
    // One read of the HUD's size for both elements.
    let hud_size = timed(S_VIEWPORT, || {
        hud::read_viewport()
            .filter(|v| {
                v.width.is_finite()
                    && v.height.is_finite()
                    && v.scale.is_finite()
                    && v.width > 0.0
                    && v.height > 0.0
            })
            .map(|v| [v.width, v.height, v.scale])
    });
    timed(S_ARROW, || {
        arrow_counter(controls.as_ref(), seconds, hud_size)
    });
    // Before the other HUD elements, which cost little: a changed crosshair or drawing grid is
    // the costliest patch, and needs the most fuel left.
    timed(S_CROSSHAIR, || {
        draw_grid_frame(controls.as_ref());
        crosshair_frame(controls.as_ref(), hud_size);
    });
    timed(S_CPS, || {
        cps_frame(controls.as_ref(), seconds, hud_size);
        fps_frame(seconds, hud_size);
    });
    timed(S_JUMP, || {
        jump_reset_frame(controls.as_ref(), delta, seconds, session);
    });
    timed(S_ZOOM, || zoom_frame(controls.as_ref(), delta));
    let settings = settings();
    if menu_open
        && (settings.rainbow || settings.mob.rainbow)
        && seconds - PREVIEW_AT.with(StdCell::get) >= PREVIEW_SECONDS
    {
        PREVIEW_AT.with(|at| at.set(seconds));
        let state = MENU.with(StdCell::get);
        let _ = host_panel::update_content(&panel::preview_update(&settings, seconds, state));
    }

    // Nothing is drawn with the HUD hidden (F1) or with both outlines switched off, so skip the
    // terrain and crowd reads that would only feed a drawing that never happens. (An open menu
    // previews the outline, so it counts as showing, but not with both outlines off.)
    if !(settings.block_on || settings.mob_on) || (!menu_open && timed(S_HIDDEN, hud_hidden)) {
        return;
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
            timed(S_TERRAIN, || {
                refresh_terrain(eye, dir, settings.reach, frame.session);
            });
        }
        // Reading a crowd (up to 16 players on a server hub) costs a fair share of a
        // frame; after a costly terrain read, the last frame's actors do.
        let (actors, hittable) = if headroom() >= ACTOR_RESERVE {
            timed(S_ACTORS, || nearby_actor_boxes(eye))
        } else {
            LAST_ACTORS.with(|last| std::mem::take(&mut *last.borrow_mut()))
        };
        // Cinnabar says which actor its crosshair would hit (attack reach, blocks in the
        // way); use that when known, so a mob is only aimed when it can really be hit.
        let native_aim = hittable.iter().any(Option::is_some);
        let aimed = hittable.iter().position(|hit| *hit == Some(true));
        let hit = timed(S_AIM, || {
            aimed_target(
                frame.session,
                frame.dimension,
                [eye, dir],
                settings.reach,
                native_aim.then(|| aimed.map(|index| actors[index].as_slice())),
                &actors,
            )
            .or_else(|| last_block_while_loading(eye, dir, settings.reach, frame.dimension))
        });
        let camera = timed(S_CAMERA, gameplay::read_camera)
            .ok()
            .flatten()
            .map_or(eye, |pose| {
                [
                    f64::from(pose.position.x),
                    f64::from(pose.position.y),
                    f64::from(pose.position.z),
                ]
            });
        let mobs = if settings.show_all_mobs {
            // In first person the camera is at the eye, so what is behind it is not on screen;
            // in third person it sits elsewhere, and nothing is left out.
            let first_person = (0..3).all(|axis| (camera[axis] - eye[axis]).abs() < 0.05);
            timed(S_BOXES, || {
                nearest_boxes(&actors, eye, first_person.then_some(dir))
            })
        } else {
            Vec::new()
        };
        // Kept for a frame with too little fuel to read them again (moved, not copied).
        timed(S_KEEP, || {
            LAST_ACTORS.with(|last| *last.borrow_mut() = (actors, hittable));
        });
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
    LAST_HIT.with(|last| {
        if let Some(target) = last.borrow().as_ref() {
            let shapes = timed(S_SHAPES, || shapes(target, &settings));
            if !shapes.is_empty() {
                timed(S_DRAW, || {
                    draw_outlines(&shapes, target.dimension, target.camera, seconds, mining);
                });
            }
        }
    });
}

/// Fuel a frame needs left to patch the menu with the settings: the patch costs about 160,000.
const MENU_PATCH_RESERVE: u64 = 175_000;
/// Fuel a frame needs left to upload the outline's geometry (18 pieces, about 2,300 each).
const CUBE_RESERVE: u64 = 70_000;

/// Whether the heavy start-up work is done: the HUD layer is published and the outline's
/// geometry is uploaded. Each of them takes a frame to itself, the menu patch the one after.
fn startup_done() -> bool {
    HUD_PUBLISHED.with(StdCell::get)
        && CUBE.with(|cube| cube.borrow().len() == outline::EDGES.len())
}

/// Fuel a painting frame keeps for itself beyond finding the drawing's rectangles.
const DRAW_SPARE: u64 = 40_000;

/// Fuel a frame needs left to save the settings file: about 65,000, and up to 40,000 more to
/// write out a 64 by 64 drawing.
const SAVE_RESERVE: u64 = 140_000;

/// Drawing on the Crosshair page's grid: holding the left button over it draws the pixels the
/// pointer passes, and holding the right button erases them. With both held, the one pressed
/// last wins. With Mirror on, every pixel is painted in all four quarters. A pixel that would make the
/// drawing too detailed to send is refused, and the help line says so. The grid (the HUD's
/// `drawgrid`, which the menu embeds) follows in `draw_grid_frame`; the settings file is
/// written when the stroke ends.
fn draw_frame(controls: &input::Controls) {
    let mut state = MENU.with(StdCell::get);
    let is = |keys: &[String], name: &str| keys.iter().any(|key| key == name);
    let (left_held, right_held) = (
        is(&controls.keys_held, "MouseLeft"),
        is(&controls.keys_held, "MouseRight"),
    );
    let (left_pressed, right_pressed) = (
        is(&controls.keys_pressed, "MouseLeft"),
        is(&controls.keys_pressed, "MouseRight"),
    );
    let held = left_held || right_held;
    let pressed = left_pressed || right_pressed;
    let drawing = controls.panel_open
        && state.page == panel::Page::Crosshair
        && state.drawing
        && state.typing.is_none();
    if !drawing || !(held || pressed) {
        DRAW_STROKE.with(|stroke| stroke.set(None));
        if SETTINGS_UNSAVED.with(StdCell::get) && headroom() >= SAVE_RESERVE {
            SETTINGS_UNSAVED.with(|unsaved| unsaved.set(false));
            save_settings(&settings());
        }
        return;
    }
    let point = card_pointer(controls);
    let mut settings = settings();
    let look = settings.crosshair;
    let Some((row, col)) = point.and_then(|point| panel::draw_cell(point, look.pixels.size()))
    else {
        return;
    };
    // Left draws, right erases: a button pressed now decides, else the one still held. A new
    // stroke (or the other button) starts here; else the line goes on from the last pixel.
    let stroke = DRAW_STROKE.with(StdCell::get);
    let (on, from) = if left_pressed {
        (true, (row, col))
    } else if right_pressed {
        (false, (row, col))
    } else {
        match stroke {
            Some((on, last)) if (on && left_held) || (!on && right_held) => (on, last),
            _ => (left_held, (row, col)),
        }
    };
    let pixels = look.pixels.painted_line(from, (row, col), on, look.mirror);
    if pixels == look.pixels {
        DRAW_STROKE.with(|cell| cell.set(Some((on, (row, col)))));
        return;
    }
    // Telling whether it is too detailed means finding its rectangles, which a large drawing
    // pays for: only with the fuel for it. The pointer is still there next frame.
    if headroom() < crosshair::rects_cost(&pixels) + DRAW_SPARE {
        return;
    }
    let refused = crosshair::too_detailed(&pixels);
    if refused != state.draw_full {
        // Only the help line changes in the menu: the grid is the HUD's, sent in
        // `draw_grid_frame`.
        state.draw_full = refused;
        MENU.with(|menu| menu.set(state));
        let _ = host_panel::update_content(&panel::draw_update(&settings, state));
    }
    if refused {
        return;
    }
    // Only now does the line move on: a frame that could not paint leaves it to the next.
    DRAW_STROKE.with(|cell| cell.set(Some((on, (row, col)))));
    settings.crosshair.pixels = pixels;
    settings.crosshair.style = crosshair::CUSTOM;
    set_settings(settings);
    SETTINGS_UNSAVED.with(|unsaved| unsaved.set(true));
}

/// The zoom, once per frame: the zoom key (held, or pressed to toggle) narrows the view with an
/// ease, scrolling (read from the hotbar slot moving) or the = and - keys change how far, and
/// the camera is asked for the view and look scales every frame it is zoomed; omitting them
/// restores the normal view. Works only in focused gameplay with no menu open.
fn zoom_frame(controls: Option<&input::Controls>, delta: f32) {
    let zoom = settings_with(|s| s.zoom);
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
    let mut settings = settings();
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
    set_settings(settings);
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

/// Re-reads the player's attack and jump keys when their bindings changed, checking at most
/// every `BINDINGS_SECONDS`. Reads at most one page per frame, because control reads are capped
/// per callback and going over stops the mod. Keeps the previous keys when they cannot be read.
fn refresh_attack_keys(seconds: f32) {
    let limit = mod_api::MAX_INPUT_BINDINGS_PER_READ as u32;
    let pending = BINDINGS_READ.with(|read| read.borrow_mut().take());
    let (revision, offset, mut keys, mut jump, page) = match pending {
        Some((revision, offset, keys, jump)) => match input::read_bindings(None, offset, limit) {
            Ok(Some(page)) if page.revision == revision => (revision, offset, keys, jump, page),
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
            (page.revision, 0, Vec::new(), Vec::new(), page)
        }
    };
    let next = offset + page.entries.len() as u32;
    let total = page.total;
    for binding in page.entries {
        if binding.context != input::BindingContext::Gameplay {
            continue;
        }
        match binding.action.as_str() {
            "attack" => keys.push(binding.key),
            "jump" => jump.push(binding.key),
            _ => {}
        }
    }
    if next < total && next > offset {
        BINDINGS_READ.with(|read| *read.borrow_mut() = Some((revision, next, keys, jump)));
    } else {
        ATTACK_KEYS.with(|stored| *stored.borrow_mut() = (Some(revision), keys));
        JUMP_KEYS.with(|stored| *stored.borrow_mut() = jump);
    }
}

/// Whether Cinnabar hides its HUD (F1, or the death screen): it then reports every vanilla HUD
/// element as not drawn. Needs the HUD permission; without it the outline always shows.
/// Checked about five times a second, not every frame: the reply lists every HUD element by
/// name, which costs fuel to read.
fn hud_hidden() -> bool {
    hud_elements_read(0.2).0
}

/// Whether the game would draw its own crosshair now, hidden by this mod or not: not with the
/// HUD hidden (F1), in third person or on the death screen. The custom crosshair follows it, so
/// it is checked more often than [`hud_hidden`], ten times a second. Without the HUD
/// permission it counts as shown.
fn vanilla_crosshair_shown() -> bool {
    hud_elements_read(0.1).1
}

/// The game's HUD elements, read again once the last read is `max_age` seconds old: whether
/// they are all hidden, and whether the crosshair is shown.
fn hud_elements_read(max_age: f32) -> (bool, bool) {
    let seconds = SECONDS.with(StdCell::get);
    let (hidden, crosshair, checked_at) = HUD_HIDDEN.with(StdCell::get);
    if (seconds - checked_at).abs() < max_age {
        return (hidden, crosshair);
    }
    let (hidden, crosshair) =
        crate::arena::scratch(hud::read_elements, |elements| match elements {
            Ok(elements) => (
                !elements.is_empty() && elements.iter().all(|element| !element.visible),
                elements
                    .iter()
                    .find(|element| element.element == hud::VanillaElement::Crosshair)
                    .is_none_or(|element| element.visible),
            ),
            Err(_) => (false, true),
        });
    HUD_HIDDEN.with(|cell| cell.set((hidden, crosshair, seconds)));
    (hidden, crosshair)
}

/// The most HUD writes this mod makes in one callback. Cinnabar allows 8 and quarantines a mod
/// that makes a ninth, so one is kept in hand: a write held back here is made in a later frame.
const HUD_WRITE_LIMIT: u32 = 7;

/// Counts one HUD write (a surface, a patch, the hidden elements or the label), or returns
/// false when this callback has made all it may.
fn hud_write() -> bool {
    HUD_WRITES.with(|writes| {
        let made = writes.get();
        if made >= HUD_WRITE_LIMIT {
            return false;
        }
        writes.set(made + 1);
        true
    })
}

/// Shows `text` on the HUD for a while; false when it could not (no HUD write left this frame,
/// or no permission).
fn show_label(text: &str) -> bool {
    let text: String = text.chars().filter(|c| !c.is_control()).take(120).collect();
    if hud_write() && hud::set_label(&text).is_ok() {
        let now = SECONDS.with(StdCell::get);
        LABEL_UNTIL.with(|until| until.set(Some(now + LABEL_SECONDS)));
        return true;
    }
    false
}

fn expire_label(seconds: f32) {
    let expired = LABEL_UNTIL.with(|until| until.get().is_some_and(|t| seconds >= t));
    if expired && hud_write() && hud::set_label("").is_ok() {
        LABEL_UNTIL.with(|until| until.set(None));
    }
}

/// Shows the first persistent failure only, so a denied permission does not spam the HUD.
fn report_once(text: &str) {
    // FNV-1a: cheap, and enough to tell problems apart.
    let hash = text.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
    });
    // Counted as reported only once it was shown: a frame out of HUD writes tries again later.
    if REPORTED_ERROR.with(StdCell::get) != hash && show_label(text) {
        REPORTED_ERROR.with(|r| r.set(hash));
    }
}

mod_api::bindings::export!(BlockOutline with_types_in mod_api::bindings);
