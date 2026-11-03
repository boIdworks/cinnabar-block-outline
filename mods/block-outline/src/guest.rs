//! Block outline mod: draws a coloured outline around the block the player is looking at.
//!
//! Each frame reads the eye pose, samples the selection shapes of the terrain around the player,
//! raycasts to the targeted block and draws its edges as depth-tested lines. F8 opens a settings
//! menu for the colour, opacity and thickness; changes save to the mod's settings file.

use std::cell::{Cell as StdCell, RefCell};

use mod_api::bindings::{
    Guest,
    cinnabar::extension::{
        budget, environment, gameplay, hud, input, panel as host_panel, player_state,
        render::{self, Line, LinePattern, Rgba, Vector3},
        settings,
        terrain::{
            self, Geometry, Position, ReadResult, Region, RegionQuery, ResolvedShape, ShapeBinding,
            Stamp,
        },
    },
};

use mod_api::drawing::{self, api as draw, shapes};

use crate::outline::{self, Cell, Hit, KeyName, Settings};
use crate::panel::{self, MenuState};

/// No block layer in a terrain slot.
const NO_LAYER: u16 = u16::MAX;
/// How long a status label stays on the HUD.
const LABEL_SECONDS: f32 = 5.0;
/// How long the pointer must rest on a control before its help appears.
const HOVER_SECONDS: f32 = 0.5;

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
    /// Unsaved menu state: key capture and whether the colour tools edit the fill.
    static MENU: StdCell<MenuState> = const { StdCell::new(MenuState { capturing: false, capturing_fullbright: false, editing_fill: false, editing_aim: false, page: panel::Page::Home, colour_open: false, typing: None }) };
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
    static LAST_ACTORS: RefCell<Vec<(Vec<[f64; 6]>, Option<bool>)>> = const { RefCell::new(Vec::new()) };
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
    /// The help entry on screen and where, so the box is only updated when it changes.
    static TIP_SHOWN: StdCell<Option<(usize, [f32; 2])>> = const { StdCell::new(None) };
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
                "Block Outline: press {} for the outline menu",
                settings.menu_key.display()
            )),
            Err(error) => show_label(&format!("Block Outline: settings unavailable ({error})")),
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
            if let Err(error) =
                host_panel::update_content(&panel::update(&settings, seconds, state))
            {
                show_label(&format!(
                    "Block Outline: allow the Controls permission for the menu ({error})"
                ));
            }
        }

        let menu_open = controls.as_ref().is_some_and(|c| c.panel_open);
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
            hover_help(controls, seconds);
            // Committed hex codes arrive separately from the numeric control events.
            let text = if menu_open {
                input::read_text_events().unwrap_or_default()
            } else {
                Vec::new()
            };
            apply_menu_events(controls, &text, seconds);
        }
        let settings = SETTINGS.with(StdCell::get);
        if menu_open && (settings.rainbow || settings.mob.rainbow) {
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
                let observed = nearby_actor_boxes(eye);
                LAST_ACTORS.with(|last| last.borrow_mut().clone_from(&observed));
                observed
            } else {
                LAST_ACTORS.with(|last| last.borrow().clone())
            };
            // Cinnabar says which actor its crosshair would hit (attack reach, blocks in the
            // way); use that when known, so a mob is only aimed when it can really be hit.
            let native_aim = observed.iter().any(|(_, hittable)| hittable.is_some());
            let aimed_actor: Vec<Vec<[f64; 6]>> = observed
                .iter()
                .filter(|(_, hittable)| *hittable == Some(true))
                .map(|(boxes, _)| boxes.clone())
                .take(1)
                .collect();
            let actors: Vec<Vec<[f64; 6]>> = observed.into_iter().map(|(boxes, _)| boxes).collect();
            let hit = SAMPLE
                .with(|sample| {
                    let sample = sample.borrow();
                    // After a dimension change or reconnect the last sample describes another world
                    // until the new one arrives; outlining from it would show phantom blocks.
                    let sample = sample.as_ref().filter(|sample| {
                        sample.frame_session == frame.session
                            && sample.stamp.dimension == frame.dimension
                    })?;
                    if native_aim {
                        // The aimed actor, wherever it is; otherwise blocks only.
                        if let Some(boxes) = aimed_actor.first() {
                            let mut min = [f64::INFINITY; 3];
                            let mut max = [f64::NEG_INFINITY; 3];
                            for b in boxes {
                                for axis in 0..3 {
                                    min[axis] = min[axis].min(b[axis]);
                                    max[axis] = max[axis].max(b[axis + 3]);
                                }
                            }
                            if min[0].is_finite() {
                                return Some(outline::Target::Actor(Hit {
                                    block: [0; 3],
                                    min,
                                    max,
                                    distance: 0.0,
                                    direct: false,
                                    boxes: outline::Boxes::single(min, max),
                                }));
                            }
                        }
                        return outline::target_with_actors(
                            eye,
                            dir,
                            f64::from(settings.reach),
                            |block| cell_at(sample, block),
                            &[],
                        );
                    }
                    outline::target_with_actors(
                        eye,
                        dir,
                        f64::from(settings.reach),
                        |block| cell_at(sample, block),
                        &actors,
                    )
                })
                .map(|target| match target {
                    outline::Target::Block(hit) => (hit, false),
                    outline::Target::Actor(hit) => (hit, true),
                });
            // While the blocks for a new view are still loading (a frame or two after a big
            // turn), keep the last block outlined rather than flicker off.
            let hit = hit.or_else(|| {
                let covered = SAMPLE.with(|sample| {
                    sample.borrow().as_ref().is_some_and(|sample| {
                        let max = [0, 1, 2].map(|a| sample.min[a] + sample.size[a]);
                        outline::region_covers(
                            (sample.min, max),
                            eye,
                            dir,
                            f64::from(settings.reach),
                        )
                    })
                });
                if covered {
                    return None;
                }
                LAST_HIT.with(|last| {
                    last.borrow()
                        .as_ref()
                        .filter(|last| last.dimension == frame.dimension)
                        .and_then(|last| last.hit)
                        .filter(|(_, actor)| !actor)
                })
            });
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
}

/// The menu opens on its start page: once closed, forget the page that was shown.
fn back_to_start_when_closed(controls: &input::Controls, seconds: f32) {
    let state = MENU.with(StdCell::get);
    if controls.panel_open
        || (state.page == panel::Page::Home && !state.colour_open && state.typing.is_none())
    {
        return;
    }
    let state = MenuState {
        page: panel::Page::Home,
        colour_open: false,
        editing_fill: false,
        editing_aim: false,
        typing: None,
        ..state
    };
    MENU.with(|m| m.set(state));
    let settings = SETTINGS.with(StdCell::get);
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    let _ = host_panel::update_content(&panel::update(&settings, seconds, state));
}

/// Captures a new menu key while the "Menu key" button waits for one; otherwise opens or closes
/// the menu when the chosen key is pressed. The panel's own toggle key is a placeholder, because
/// a panel cannot change its toggle key after loading.
fn handle_menu_key(controls: &input::Controls, seconds: f32) {
    if MENU.with(StdCell::get).capturing_fullbright {
        capture_fullbright_key(controls, seconds);
        return;
    }
    if MENU.with(StdCell::get).capturing {
        if !controls.panel_open {
            // The menu closed mid-capture; keep the old key.
            set_capturing(false, seconds);
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
            if let Err(error) = settings::save(&settings.to_json()) {
                report_once(&format!("Block Outline: settings not saved ({error})"));
            }
            reserve_keys(&settings);
        }
        set_capturing(false, seconds);
        return;
    }
    if MENU.with(StdCell::get).typing.is_some() {
        // Keys are being typed into a box; the menu key must not close the menu.
        return;
    }
    toggle_fullbright_by_key(controls);
    let key = SETTINGS.with(StdCell::get).menu_key;
    if !controls.focused
        || !controls
            .keys_pressed
            .iter()
            .any(|pressed| pressed == key.as_str())
    {
        return;
    }
    if controls.panel_open {
        let _ = host_panel::set_open(false);
    } else if controls.gameplay {
        if let Err(error) = host_panel::set_open(true) {
            report_once(&format!("Block Outline: menu unavailable ({error})"));
        }
    }
}

/// Starts or ends waiting for a new menu key and redraws the button for it.
fn set_capturing(capturing: bool, seconds: f32) {
    let state = MENU.with(|menu| {
        let mut state = menu.get();
        state.capturing = capturing;
        menu.set(state);
        state
    });
    let settings = SETTINGS.with(StdCell::get);
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    if let Err(error) = host_panel::update_content(&panel::update(&settings, seconds, state)) {
        report_once(&format!("Block Outline: menu update failed ({error})"));
    }
}

/// Shows a short explanation next to the pointer once it has rested on a control for
/// `HOVER_SECONDS`, and hides it when the pointer moves off or the menu closes.
fn hover_help(controls: &input::Controls, seconds: f32) {
    // Positions depend on whether the colour tools widen the card.
    let state = MENU.with(StdCell::get);
    let open = state.colour_open;
    let matching = panel::fill_matches(&SETTINGS.with(StdCell::get), state);
    let point = controls
        .pointer
        .filter(|_| controls.panel_open && !MENU.with(StdCell::get).capturing)
        .and_then(|(x, y)| {
            let layout = host_panel::read_layout()?;
            panel::card_point(
                [x, y],
                [layout.width, layout.height],
                [layout.offset.0, layout.offset.1],
                layout.scale,
                open,
            )
        });
    let under = point.and_then(|point| panel::tip_at(point, state, matching));
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

/// Applies the menu's control events, then saves and refreshes the menu once if anything changed.
fn apply_menu_events(controls: &input::Controls, text: &[input::TextEvent], seconds: f32) {
    let events = &controls.events;
    let mut state = MENU.with(StdCell::get);
    if !controls.panel_open && state.typing.is_some() {
        // The menu closed mid-typing: keep the old value.
        state.typing = None;
        MENU.with(|menu| menu.set(state));
    }
    let typing = state.typing.is_some() && !state.capturing;
    if events.is_empty() && text.is_empty() && !(typing && !controls.keys_pressed.is_empty()) {
        return;
    }
    let before = SETTINGS.with(StdCell::get);
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
    for event in events {
        if event.id == "close" {
            // The X button: the menu's state resets to the start page once it has closed.
            let _ = host_panel::set_open(false);
            continue;
        }
        if event.id == "fullbright_key" {
            // The "Light key" button: wait for the next key press.
            set_capturing_fullbright(true, seconds);
            state = MENU.with(StdCell::get);
            continue;
        }
        if event.id == "menu_key" {
            // The "Menu key" button: wait for the next key press.
            set_capturing(true, seconds);
            state = MENU.with(StdCell::get);
            continue;
        }
        refresh |= panel::apply(&mut settings, &mut state, &event.id, event.value);
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
        return;
    }
    if changed {
        SETTINGS.with(|s| s.set(settings));
        if let Err(error) = settings::save(&settings.to_json()) {
            report_once(&format!("Block Outline: settings not saved ({error})"));
        }
    }
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    if let Err(error) = host_panel::update_content(&panel::update(&settings, seconds, state)) {
        report_once(&format!("Block Outline: menu update failed ({error})"));
    }
    // The update hid the highlight and help; forget them so the next frame shows them again
    // where they now belong (opening the colour tools moves every settings control).
    HOVER.with(|h| h.set(None));
    TIP_SHOWN.with(|t| t.set(None));
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
            "Block Outline: allow the Fullbright permission for fullbright ({error})"
        ));
    }
}

/// Reserves the menu key and the fullbright key, so the game does not act on them too (such as
/// E opening the inventory).
fn reserve_keys(settings: &Settings) {
    let mut keys = vec![settings.menu_key.as_str().to_owned()];
    keys.extend(settings.fullbright_key.map(|key| key.as_str().to_owned()));
    let _ = input::reserve_keys(&keys);
}

/// Waits for the new fullbright key while its button is active: Escape keeps the old key,
/// Backspace or Delete removes it, any other key (except the menu key) becomes it.
fn capture_fullbright_key(controls: &input::Controls, seconds: f32) {
    if !controls.panel_open {
        set_capturing_fullbright(false, seconds);
        return;
    }
    let Some(key) = controls
        .keys_pressed
        .iter()
        .find(|key| !key.starts_with("Mouse"))
    else {
        return;
    };
    let mut settings = SETTINGS.with(StdCell::get);
    let chosen = match key.as_str() {
        "Escape" => settings.fullbright_key,
        "Backspace" | "Delete" => None,
        key if key == settings.menu_key.as_str() => settings.fullbright_key,
        key => KeyName::new(key).or(settings.fullbright_key),
    };
    if chosen != settings.fullbright_key {
        settings.fullbright_key = chosen;
        SETTINGS.with(|s| s.set(settings));
        if let Err(error) = settings::save(&settings.to_json()) {
            report_once(&format!("Block Outline: settings not saved ({error})"));
        }
        reserve_keys(&settings);
    }
    set_capturing_fullbright(false, seconds);
}

/// Starts or ends waiting for a new fullbright key and redraws the button for it.
fn set_capturing_fullbright(capturing: bool, seconds: f32) {
    let state = MENU.with(|menu| {
        let mut state = menu.get();
        state.capturing_fullbright = capturing;
        menu.set(state);
        state
    });
    let settings = SETTINGS.with(StdCell::get);
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    let _ = host_panel::update_content(&panel::update(&settings, seconds, state));
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
    if let Err(error) = settings::save(&settings.to_json()) {
        report_once(&format!("Block Outline: settings not saved ({error})"));
    }
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
fn nearby_actor_boxes(eye: [f64; 3]) -> Vec<(Vec<[f64; 6]>, Option<bool>)> {
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
            report_once(&format!("Block Outline: drawing failed ({})", error.detail));
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
        report_once(&format!("Block Outline: drawing failed ({error})"));
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
                .map(|sample| sample.stamp.clone())
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
            Ok(ReadResult::Changed(region)) => Ok(Some((
                region.stamp.clone(),
                Blocks::from_region(region, cells),
            ))),
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
        Err(error) => report_once(&format!("Block Outline: terrain unavailable ({error})")),
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
    if !REPORTED_ERROR.with(|r| r.replace(true)) {
        show_label(text);
    }
}

mod_api::bindings::export!(BlockOutline with_types_in mod_api::bindings);
