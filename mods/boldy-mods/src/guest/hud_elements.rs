//! The HUD elements: the arrow counter, the CPS readout, the FPS counter and the editor that
//! moves them. The
//! shared state (settings, menu, the published layer) lives in the parent module.

use super::*;

/// How long to wait before publishing the HUD layer again after the host refused it.
const HUD_RETRY_SECONDS: f32 = 5.0;

/// Fuel a frame needs left to publish the HUD layer; with less, a later frame does.
const SURFACE_RESERVE: u64 = 100_000;

/// Fuel a frame needs left to read the inventory after it changed; with less, a later frame does.
const INVENTORY_RESERVE: u64 = 90_000;

/// Fuel a frame needs left to read the combat feed; with less, a later frame does.
const COMBAT_RESERVE: u64 = 40_000;

/// Fuel a frame keeps over what sending the crosshair is expected to cost
/// ([`crosshair::patch_cost`]), for the rest of the frame. With less, a later frame sends it.
const CROSSHAIR_SPARE: u64 = 25_000;

/// Fuel a frame needs left to save the settings (a new jump reset log row); with less, a later
/// frame does.
const LOG_SAVE_RESERVE: u64 = 120_000;

/// Width of `arrows` written out, at text scale 1, measured once per new number.
fn count_width(arrows: u32) -> f32 {
    if let Some((known, width)) = COUNT_WIDTH.with(StdCell::get)
        && known == arrows
    {
        return width;
    }
    let mut text = String::new();
    crate::panel::digits(&mut text, u64::from(arrows));
    let width = hud::measure_text(&text, hud::Font::Native)
        .ok()
        .filter(|width| width.is_finite() && *width > 0.0)
        .unwrap_or_else(|| counter::guess_width(counter::digits(arrows)));
    COUNT_WIDTH.with(|w| w.set(Some((arrows, width))));
    width
}

/// While moving, the arrow counter, CPS, FPS and armor HUD all show their frames, and the editor ([`hudedit`]) moves or resizes what is dragged. Enter,
/// Escape or the right button finish. Returns how each frame should look, in that order.
fn drag_elements(
    controls: &input::Controls,
    settings: &mut Settings,
    viewport: [f32; 2],
    // The unscaled size of each element, including the armor HUD.
    bases: [[f32; 2]; 4],
    seconds: f32,
) -> [counter::Edit; 4] {
    let before = (settings.counter, settings.cps, settings.fps, settings.armor);
    let (armor_corner, armor_size) = crate::armor::frame(settings.armor, viewport);
    let mut items = [
        hudedit::Item {
            pos: settings.counter.pos,
            scale: settings.counter.scale,
        },
        hudedit::Item {
            pos: settings.cps.pos,
            scale: settings.cps.scale,
        },
        hudedit::Item {
            pos: settings.fps.pos,
            scale: settings.fps.scale,
        },
        hudedit::Item {
            pos: counter::centre(armor_corner, armor_size, viewport),
            scale: settings.armor.scale,
        },
    ];
    let mut state = MENU.with(StdCell::get);
    let mut editor = EDITOR.with(StdCell::get);
    // The element grabbed last (or whose button opened this) takes the keys.
    editor.selected = if state.moving_armor {
        3
    } else if state.moving_fps {
        2
    } else {
        usize::from(state.moving_cps)
    };
    let pressed = |key: &str| controls.keys_pressed.iter().any(|k| k == key);
    let input = hudedit::Input {
        pointer: controls.pointer.map(|(x, y)| [x, y]),
        pressed: pressed("MouseLeft"),
        held: controls.keys_held.iter().any(|k| k == "MouseLeft"),
        steps: hudedit::resize_steps(controls.wheel, &controls.keys_pressed),
    };
    // The guide box (at the top of the screen) and a little room around it.
    let guide = [panel::GUIDE_W, panel::GUIDE_H + 16.0];
    let out = editor.step(&mut items, bases, viewport, input, guide);
    EDITOR.with(|e| e.set(editor));
    (settings.counter.pos, settings.counter.scale) = (items[0].pos, items[0].scale);
    (settings.cps.pos, settings.cps.scale) = (items[1].pos, items[1].scale);
    (settings.fps.pos, settings.fps.scale) = (items[2].pos, items[2].scale);
    let new_size = bases[3].map(|side| side * items[3].scale);
    let new_corner = counter::place(items[3].pos, new_size, viewport);
    if new_corner != armor_corner || items[3].scale != settings.armor.scale {
        settings.armor.scale = items[3].scale;
        let offset = crate::armor::corner_offset(settings.armor);
        settings.armor.pos = counter::Pos::anchored(
            [
                new_corner[0] - offset[0] * items[3].scale,
                new_corner[1] - offset[1] * items[3].scale,
            ],
            viewport,
        );
    }
    if (settings.counter, settings.cps, settings.fps, settings.armor) != before {
        set_settings(*settings);
        COUNTER_UNSAVED.with(|unsaved| unsaved.set(true));
    }
    state.moving_cps = editor.selected == 1;
    state.moving_fps = editor.selected == 2;
    state.moving_armor = editor.selected == 3;
    if ["Escape", "Enter", "NumpadEnter", "MouseRight"]
        .iter()
        .any(|key| pressed(key))
    {
        state.moving = false;
        state.moving_cps = false;
        state.moving_fps = false;
        state.moving_armor = false;
        MENU.with(|menu| menu.set(state));
        EDITOR.with(|e| e.set(hudedit::Editor::default()));
        push_menu_with(seconds, settings, state, true);
    } else {
        let moved_guide = state.guide_bottom != out.guide_blocked;
        state.guide_bottom = out.guide_blocked;
        MENU.with(|menu| menu.set(state));
        if moved_guide {
            let _ = host_panel::update_content(&panel::guide_update(state));
        }
    }
    out.edits
}

/// Keeps the HUD arrow counter up to date: publishes its layer, recounts the arrows after the
/// inventory changes, lets the player drag and resize it while moving, and patches the HUD only
/// when what it shows changed.
pub(super) fn arrow_counter(
    controls: Option<&input::Controls>,
    seconds: f32,
    hud_size: Option<[f32; 3]>,
) {
    if let Some(v) = hud_size {
        let mut watch = VIEWPORT_WATCH.with(StdCell::get);
        let reaction = watch.observe(v, seconds);
        VIEWPORT_WATCH.with(|w| w.set(watch));
        if reaction.remeasure {
            // Text widths are in GUI units and change with the GUI scale: measure again, and
            // send every binding again in case the host laid the layer out afresh.
            COUNT_WIDTH.with(|w| w.set(None));
            COUNTER_SHOWN.with(|shown| shown.set(None));
            CPS_WIDTHS.with(|probe| {
                let mut reset = probe.get();
                reset.reset();
                probe.set(reset);
            });
            CPS_SHOWN.with(|shown| shown.set(None));
            FPS_SHOWN.with(|shown| shown.set(None));
            JUMP_SHOWN.with(|shown| shown.set(None));
        }
        if reaction.republish {
            HUD_PUBLISHED.with(|published| published.set(false));
            HUD_RETRY_AT.with(|at| at.set(0.0));
        }
    }
    if !HUD_PUBLISHED.with(StdCell::get) {
        if seconds < HUD_RETRY_AT.with(StdCell::get) {
            return;
        }
        // Publishing writes the layer's starting bindings: wait for a frame with fuel to spare,
        // and with no other heavy job (the menu patch) in it.
        if HEAVY_FRAME.with(StdCell::get) || headroom() < SURFACE_RESERVE {
            return;
        }
        HEAVY_FRAME.with(|heavy| heavy.set(true));
        COUNTER_SHOWN.with(|shown| shown.set(None));
        CPS_SHOWN.with(|shown| shown.set(None));
        FPS_SHOWN.with(|shown| shown.set(None));
        EFFECTS_SHOWN.with(|shown| shown.set(None));
        HOTBAR_SHOWN.with(|shown| shown.set(None));
        ARMOR_DIRTY.with(|dirty| dirty.set(true));
        JUMP_SHOWN.with(|shown| shown.set(None));
        // A new layer starts with the crosshair and the grid empty.
        CROSSHAIR_SENT.with(|sent| sent.set(NOTHING_SENT));
        GRID_SENT.with(|sent| sent.set(NOTHING_SENT));
        GRID_CURSOR.with(|cursor| cursor.set(None));
        CROSSHAIR_VISIBLE.with(|visible| visible.set(None));
        if !hud_write() {
            return;
        }
        if let Err(error) = hud::set_surface(&counter::surface()) {
            HUD_RETRY_AT.with(|at| at.set(seconds + HUD_RETRY_SECONDS));
            if settings_with(|s| s.counter.on) {
                report_once(&format!(
                    "Boldy Mods: allow the HUD permission for the arrow counter ({error})"
                ));
            }
            return;
        }
        HUD_PUBLISHED.with(|published| published.set(true));
    }
    let state = MENU.with(StdCell::get);
    let panel_open = controls.is_some_and(|c| c.panel_open);
    let moving = state.moving && panel_open;
    let viewport = hud_size.map(|v| [v[0], v[1]]);
    // Only the counter's own settings, in place; the whole copy is made on the paths that change
    // settings (a position to pin, an element being dragged, a save).
    let mut counter = settings_with(|s| s.counter);
    if let Some(viewport) = viewport.filter(|_| counter.pos.is_fraction()) {
        // An older settings file kept the position as a share of the screen: pin it to the
        // nearest edge or the middle now, where it is on screen, and save that.
        let mut settings = settings();
        settings.counter.pos = counter::Pos::anchored(settings.counter.pos.at(viewport), viewport);
        counter = settings.counter;
        set_settings(settings);
        COUNTER_UNSAVED.with(|unsaved| unsaved.set(true));
    }
    let quiver = QUIVER.with(StdCell::get).map(|(_, _, quiver)| quiver);
    let arrows = quiver.map_or(0, |quiver| quiver.arrows);
    let text_width = count_width(arrows);
    let mut edits = [counter::Edit::default(); 4];
    if let (true, Some(controls), Some(viewport)) = (moving, controls, viewport) {
        let mut settings = settings();
        let (_, _, layout) = cps_reading(seconds, &settings.cps);
        let (fps_width, coords) = fps_lines(settings.fps, FPS.with(StdCell::get).fps(), true);
        let bases = [
            counter::size(1.0, text_width),
            cps::base(layout.total),
            fps::base(fps_width, coords.map_or(0.0, |(_, width)| width)),
            crate::armor::base(settings.armor),
        ];
        edits = drag_elements(controls, &mut settings, viewport, bases, seconds);
        counter = settings.counter;
    }
    timed(S_ARMOR, || {
        armor_frame(seconds, viewport, moving.then_some(edits[3]));
    });
    // A moved counter is saved once, when moving ends, not on every step of a drag.
    let moving = MENU.with(StdCell::get).moving && panel_open;
    let moving_others = moving;
    EDIT_CPS.with(|edit| edit.set(moving_others.then_some(edits[1])));
    EDIT_FPS.with(|edit| edit.set(moving_others.then_some(edits[2])));
    if !moving && COUNTER_UNSAVED.with(StdCell::get) {
        // The drag is over whether or not the save fits in this frame; the new place is already
        // on screen, and only writing it down waits.
        EDITOR.with(|editor| editor.set(hudedit::Editor::default()));
        save_moved_hud(seconds);
    }
    let on_page = panel_open && state.page == panel::Page::Counter;
    let holding = || {
        !counter.bow_only
            || quiver.is_some_and(|quiver| {
                let selected = selected_slot().map(|(_, _, slot)| slot);
                quiver.holding_bow(selected)
            })
    };
    // Without the Player status permission the count is unknown: show a ? so the counter can
    // still be found and placed, and the reason is clear.
    let unknown = quiver.is_none() && NO_PLAYER_STATUS.with(StdCell::get);
    // While the HUD is edited every element is shown, so each can be placed.
    let visible = moving_others
        || (counter.on && (quiver.is_some() || unknown) && (on_page || unknown || holding()));
    let view = match viewport.filter(|_| visible) {
        Some(viewport) => {
            let size = counter::size(counter.scale, text_width);
            counter::View {
                visible: true,
                arrows: (!unknown).then_some(arrows),
                offset: counter::place(counter.pos, size, viewport),
                size,
                scale: counter.scale,
                text_width,
                edit: moving_others.then_some(edits[0]),
            }
        }
        None => counter::View {
            visible: false,
            arrows: Some(0),
            offset: [0.0; 2],
            size: [0.0; 2],
            scale: 1.0,
            text_width: 0.0,
            edit: None,
        },
    };
    let shown = COUNTER_SHOWN.with(StdCell::get);
    // Hidden is hidden, whatever it would have shown.
    if shown.is_some_and(|shown| shown == view || (!shown.visible && !view.visible)) {
        return;
    }
    if !hud_write() {
        return;
    }
    match hud::update_surface(&counter::patch(&view)) {
        Ok(()) => COUNTER_SHOWN.with(|s| s.set(Some(view))),
        Err(error) => {
            // The layer is gone (or was refused): publish it again later.
            HUD_PUBLISHED.with(|published| published.set(false));
            HUD_RETRY_AT.with(|at| at.set(seconds + HUD_RETRY_SECONDS));
            report_once(&format!("Boldy Mods: arrow counter failed ({error})"));
        }
    }
}

/// Counts left and right clicks and keeps the CPS readout on the HUD up to date. Clicks count
/// only in gameplay with the menu closed, and only while the readout is on.
pub(super) fn cps_frame(
    controls: Option<&input::Controls>,
    seconds: f32,
    hud_size: Option<[f32; 3]>,
) {
    let settings = settings_with(|s| s.cps);
    if settings.on
        && let Some(controls) = controls.filter(|c| c.focused && c.gameplay && !c.panel_open)
    {
        CLICKS.with(|clicks| {
            let (left, right) = &mut *clicks.borrow_mut();
            for key in &controls.keys_pressed {
                match key.as_str() {
                    "MouseLeft" => left.click(seconds),
                    "MouseRight" => right.click(seconds),
                    _ => {}
                }
            }
        });
    }
    // The layer is published by the arrow counter; patch it only once it is up.
    if !HUD_PUBLISHED.with(StdCell::get) {
        CPS_SHOWN.with(|shown| shown.set(None));
        return;
    }
    let viewport = hud_size.map(|v| [v[0], v[1]]);
    let edit = EDIT_CPS.with(StdCell::get);
    // The counts first, which are cheap: with them, the place, the settings and the frame as
    // last time, and the readout on screen, there is nothing to work out or compare.
    let counts = cps_counts(seconds, &settings);
    let key = (counts, viewport, settings, edit);
    if CPS_KEY.with(StdCell::get) == Some(key) && CPS_SHOWN.with(|shown| shown.get().is_some()) {
        return;
    }
    CPS_KEY.with(|last| last.set(Some(key)));
    let view = match viewport.filter(|_| settings.on || edit.is_some()) {
        Some(viewport) => {
            let ((left, right), layout) = (counts, cps_widths().layout(counts.0, counts.1));
            let scale = cps::clamp_scale(settings.scale);
            cps::View {
                visible: true,
                left,
                right,
                zeros: settings.zeros,
                scale,
                layout,
                offset: counter::place(settings.pos, cps::size(scale, layout.total), viewport),
                edit,
            }
        }
        None => cps::View::HIDDEN,
    };
    let shown = CPS_SHOWN.with(StdCell::get);
    // Hidden is hidden, whatever it would have shown.
    if shown.is_some_and(|shown| shown == view || (!shown.visible && !view.visible)) {
        return;
    }
    if !hud_write() {
        return;
    }
    match hud::update_surface(&cps::patch(&view)) {
        Ok(()) => CPS_SHOWN.with(|s| s.set(Some(view))),
        Err(error) => {
            // The layer is gone (or was refused): the arrow counter publishes it again.
            HUD_PUBLISHED.with(|published| published.set(false));
            HUD_RETRY_AT.with(|at| at.set(seconds + HUD_RETRY_SECONDS));
            report_once(&format!("Boldy Mods: CPS counter failed ({error})"));
        }
    }
}

thread_local! {
    /// The block the player stands in, as last read; `None` when not known or not wanted.
    static COORDS: StdCell<Option<fps::Coords>> = const { StdCell::new(None) };
    /// The time before which the position is not read again.
    static COORDS_NEXT: StdCell<f32> = const { StdCell::new(0.0) };
}

/// Fuel a frame needs left to read the position; with less, the last one stays and a later frame
/// reads it.
const COORDS_RESERVE: u64 = 30_000;

/// Reads where the player stands, at most every [`fps::COORDS_EVERY`] seconds and only while the
/// coordinates are wanted (on, in a world): the one host call is skipped otherwise, and the last
/// answer is dropped so a later start does not show an old position.
fn refresh_coords(seconds: f32, wanted: bool) {
    if !wanted {
        if COORDS.with(StdCell::get).is_some() {
            COORDS.with(|coords| coords.set(None));
        }
        return;
    }
    if seconds < COORDS_NEXT.with(StdCell::get) || headroom() < COORDS_RESERVE {
        return;
    }
    COORDS_NEXT.with(|next| next.set(seconds + fps::COORDS_EVERY));
    note_read();
    let at = match player_state::read_world() {
        Ok(Some(world)) => fps::block_of(world.position),
        _ => None,
    };
    COORDS.with(|coords| coords.set(at));
}

/// What the counter shows: the width of the FPS text (0 when it is not shown) and the coordinates
/// line with its width (`None` when it is off or the position is not known). While the HUD is
/// being edited (`editing`) a counter with nothing to show still has its FPS text, and unknown
/// coordinates read as zeros, so there is something to grab.
fn fps_lines(settings: fps::Fps, fps_now: u32, editing: bool) -> (f32, Option<(fps::Coords, f32)>) {
    let widths = cps_widths();
    let show_fps = settings.on || (editing && !settings.coords);
    let width = if show_fps {
        widths.fps_width(fps_now)
    } else {
        0.0
    };
    let coords = settings
        .coords
        .then(|| COORDS.with(StdCell::get).or(editing.then_some([0; 3])))
        .flatten()
        .map(|coords| (coords, fps::coords_width(&widths, coords)));
    (width, coords)
}

/// Keeps the FPS counter on the HUD up to date. The frames are counted in `frame_body`; the
/// number changes every [`fps::EVERY`] seconds, so the HUD is patched about twice a second. With
/// the coordinates on, the block the player stands in is read ten times a second and the line is
/// patched only when it changes.
pub(super) fn fps_frame(seconds: f32, hud_size: Option<[f32; 3]>, in_world: bool) {
    let settings = settings_with(|s| s.fps);
    timed(S_COORDS, || {
        refresh_coords(seconds, settings.coords && in_world);
    });
    // The layer is published by the arrow counter; patch it only once it is up.
    if !HUD_PUBLISHED.with(StdCell::get) {
        FPS_SHOWN.with(|shown| shown.set(None));
        return;
    }
    let edit = EDIT_FPS.with(StdCell::get);
    let fps = FPS.with(StdCell::get).fps();
    let (width, coords) = fps_lines(settings, fps, edit.is_some());
    let view = match hud_size.filter(|_| width > 0.0 || coords.is_some()) {
        Some(viewport) => {
            let scale = cps::clamp_scale(settings.scale);
            let coords_width = coords.map_or(0.0, |(_, width)| width);
            fps::View {
                visible: true,
                fps,
                scale,
                width,
                coords,
                offset: counter::place(
                    settings.pos,
                    fps::size(scale, width, coords_width),
                    [viewport[0], viewport[1]],
                ),
                edit,
            }
        }
        None => fps::View::HIDDEN,
    };
    let shown = FPS_SHOWN.with(StdCell::get);
    // Hidden is hidden, whatever it would have shown.
    if shown.is_some_and(|shown| shown == view || (!shown.visible && !view.visible)) {
        return;
    }
    if !hud_write() {
        return;
    }
    match hud::update_surface(&fps::patch(&view)) {
        Ok(()) => FPS_SHOWN.with(|s| s.set(Some(view))),
        Err(error) => {
            // The layer is gone (or was refused): the arrow counter publishes it again.
            HUD_PUBLISHED.with(|published| published.set(false));
            HUD_RETRY_AT.with(|at| at.set(seconds + HUD_RETRY_SECONDS));
            report_once(&format!("Boldy Mods: FPS counter failed ({error})"));
        }
    }
}

/// The next step of sending `plan` over what the layer has (`sent`): the first part that
/// changed, else the header (`None`) if it did. Parts first: they are only drawn once the header
/// places them, so a new crosshair appears whole, in the frame its header is sent.
fn next_step(plan: &crosshair::Plan, sent: &Sent) -> Option<Option<usize>> {
    let empty = crosshair::empty_part();
    if let Some(index) =
        (0..crosshair::PARTS).find(|&index| sent.1[index].unwrap_or(empty) != plan.parts[index])
    {
        return Some(Some(index));
    }
    (sent.0 != Some(plan.header)).then_some(None)
}

/// Whether this frame can afford a costly patch: no other this frame, `cost` fuel and some to
/// spare, and a HUD write left. Taking it marks the frame as having done one.
fn afford(cost: u64) -> bool {
    if HEAVY_FRAME.with(StdCell::get) || headroom() < cost + CROSSHAIR_SPARE || !hud_write() {
        return false;
    }
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    true
}

/// The host replaces the entire hidden list at once, so keep both features together.
fn hidden_elements(crosshair: bool, hunger: bool) -> Vec<hud::VanillaElement> {
    let mut hidden = Vec::with_capacity(2);
    if crosshair {
        hidden.push(hud::VanillaElement::Crosshair);
    }
    if hunger {
        hidden.push(hud::VanillaElement::Hunger);
    }
    hidden
}

/// The custom crosshair: shown in place of the game's own while it is switched on, and only
/// where the game would draw its own (not with the HUD hidden or in third person). On its menu
/// page it shows whatever the game does, so changes can be seen as they are made. The game's
/// crosshair is hidden only while this one shows, so there is never no crosshair at all.
///
/// A changed crosshair is sent in steps, one a frame: its header, then each part that changed
/// (see [`crosshair`]); showing and hiding it is a binding. A refused patch is reported once
/// and counted as sent, so it never turns into a patch every frame.
pub(super) fn crosshair_frame(controls: Option<&input::Controls>, hud_size: Option<[f32; 3]>) {
    // The look is read in place: it holds the whole drawing (over 500 bytes), and copying and
    // comparing it every frame was most of this section's cost. A copy is made only to work out
    // a new plan.
    let on = settings_with(|s| s.crosshair.on);
    let published = HUD_PUBLISHED.with(StdCell::get);
    // The game's crosshair gives way only while this one shows: not while it is still being
    // sent, nor where it cannot show (the HUD's size unknown), so there is always one.
    let replace = on && published && CROSSHAIR_VISIBLE.with(StdCell::get) == Some(true);
    timed(S_XH_HIDE, || {
        let hunger = settings_with(|s| s.hide_hunger);
        if (VANILLA_CROSSHAIR_HIDDEN.with(StdCell::get) != replace
            || VANILLA_HUNGER_HIDDEN.with(StdCell::get) != hunger)
            && hud_write()
            && hud::set_hidden_elements(&hidden_elements(replace, hunger)).is_ok()
        {
            VANILLA_CROSSHAIR_HIDDEN.with(|cell| cell.set(replace));
            VANILLA_HUNGER_HIDDEN.with(|cell| cell.set(hunger));
        }
    });
    // The layer is published by the arrow counter; patch it only once it is up.
    if !published {
        return;
    }
    let previewing = controls.is_some_and(|c| c.panel_open)
        && MENU.with(StdCell::get).page == panel::Page::Crosshair;
    let viewport =
        hud_size.filter(|_| on && (previewing || timed(S_XH_SHOWN, vanilla_crosshair_shown)));
    let visible = CROSSHAIR_VISIBLE.with(StdCell::get);
    let Some(viewport) = viewport else {
        if visible != Some(false) && hud_write() {
            CROSSHAIR_VISIBLE.with(|shown| shown.set(Some(false)));
            send(&crosshair::visibility(false), "crosshair");
        }
        return;
    };
    // The shape and plan, worked out once for a view: a large drawing costs to work out.
    // Settings that were not replaced since the plan was last checked against them need no
    // comparing: the revision says so for a few instructions, and the comparison (which still
    // costs about 1,900 fuel) is only made after a change.
    let revision = SETTINGS_REV.with(StdCell::get);
    let planned = timed(S_XH_PLAN, || {
        if CROSSHAIR_CHECKED.with(StdCell::get) == Some((revision, viewport)) {
            return true;
        }
        let same = settings_with(|s| {
            CROSSHAIR_PLAN.with(|plan| {
                plan.borrow().as_ref().is_some_and(|(known, ..)| {
                    known.viewport == viewport && known.look == s.crosshair
                })
            })
        });
        if same {
            CROSSHAIR_CHECKED.with(|checked| checked.set(Some((revision, viewport))));
        }
        same
    });
    if !planned {
        let view = crosshair::View {
            look: settings_with(|s| s.crosshair),
            viewport,
        };
        if HEAVY_FRAME.with(StdCell::get)
            || headroom() < crosshair::plan_cost(&view.look) + CROSSHAIR_SPARE
        {
            return;
        }
        let shape = crosshair::view_shape(&view);
        let plan = crosshair::plan(&view, &shape);
        CROSSHAIR_PLAN.with(|known| *known.borrow_mut() = Some((view, shape, plan)));
        // Made from the settings as they are now, so it matches them at this revision.
        CROSSHAIR_CHECKED.with(|checked| checked.set(Some((revision, viewport))));
    }
    let sent = CROSSHAIR_SENT.with(StdCell::get);
    let patch = CROSSHAIR_PLAN.with(|known| {
        let known = known.borrow();
        // The stored view is the current one: it was just checked, or just made.
        let (view, shape, plan) = known.as_ref()?;
        let step = next_step(plan, &sent)?;
        let cost = match step {
            None => crosshair::header_cost(shape),
            Some(index) => crosshair::part_cost(shape, index),
        };
        if !afford(cost) {
            return Some(None);
        }
        let mut sent = sent;
        let patch = match step {
            None => {
                sent.0 = Some(plan.header);
                crosshair::header_patch(view, shape)
            }
            Some(index) => {
                sent.1[index] = Some(plan.parts[index]);
                crosshair::part_patch(view, shape, index)
            }
        };
        CROSSHAIR_SENT.with(|cell| cell.set(sent));
        Some(Some((patch, step.is_none())))
    });
    match patch {
        // A step this frame could not afford: a later one sends it.
        Some(None) => {}
        Some(Some((patch, header))) => {
            if header {
                // The header shows the crosshair.
                CROSSHAIR_VISIBLE.with(|shown| shown.set(Some(true)));
            }
            send(&patch, "crosshair");
        }
        // Everything sent: only showing it may be left.
        None => {
            if visible != Some(true) && hud_write() {
                CROSSHAIR_VISIBLE.with(|shown| shown.set(Some(true)));
                send(&crosshair::visibility(true), "crosshair");
            }
        }
    }
}

/// The drawing grid's highlight: on the pixel under the pointer (and with Mirror on, on its
/// reflections), so it is plain where a click lands on a fine grid. Hidden off the grid and with
/// the grid not shown (`controls` none). A small patch, sent only when the pixel changes.
fn grid_cursor(controls: Option<&input::Controls>) {
    let (size, mirror) = settings_with(|s| (s.crosshair.pixels.size(), s.crosshair.mirror));
    let cell = controls
        .and_then(super::card_pointer)
        .and_then(|point| panel::draw_cell(point, size));
    let key = (cell, mirror, size);
    if GRID_CURSOR.with(StdCell::get) == Some(key) || !hud_write() {
        return;
    }
    GRID_CURSOR.with(|cursor| cursor.set(Some(key)));
    send(
        &crosshair::grid_cursor_patch(size, cell, mirror),
        "drawing grid",
    );
}

/// Sends a crosshair or grid patch, reporting a refusal once.
fn send(patch: &str, what: &str) {
    if let Err(error) = hud::update_surface(patch) {
        report_once(&format!("Boldy Mods: {what} failed ({error})"));
    }
}

/// The menu's drawing grid: the HUD's `drawgrid`, which the Crosshair page embeds, sent in
/// steps like the crosshair whenever the drawing, its resolution or its colour changed while the
/// grid shows.
pub(super) fn draw_grid_frame(controls: Option<&input::Controls>) {
    let state = MENU.with(StdCell::get);
    let showing = controls.is_some_and(|c| c.panel_open)
        && state.page == panel::Page::Crosshair
        && state.drawing;
    if !HUD_PUBLISHED.with(StdCell::get) {
        return;
    }
    grid_cursor(controls.filter(|_| showing));
    if !showing || HEAVY_FRAME.with(StdCell::get) {
        return;
    }
    let key = settings_with(|s| (s.crosshair.pixels, s.crosshair.rgb));
    let planned = GRID_PLAN.with(|plan| {
        plan.borrow()
            .as_ref()
            .is_some_and(|(known, _)| *known == key)
    });
    if !planned {
        if headroom() < crosshair::grid_plan_cost(&key.0) + CROSSHAIR_SPARE {
            return;
        }
        let plan = crosshair::grid_plan(&key.0, key.1);
        GRID_PLAN.with(|known| *known.borrow_mut() = Some((key, plan)));
    }
    let Some(plan) = GRID_PLAN.with(|known| known.borrow().as_ref().map(|(_, plan)| plan.clone()))
    else {
        return;
    };
    let mut sent = GRID_SENT.with(StdCell::get);
    let Some(step) = next_step(&plan, &sent) else {
        return;
    };
    let (drawing, rgb) = key;
    let cost = match step {
        None => crosshair::grid_header_cost(),
        Some(index) => crosshair::grid_part_cost(&drawing, index),
    };
    if !afford(cost) {
        return;
    }
    let patch = match step {
        None => {
            sent.0 = Some(plan.header);
            crosshair::grid_header_patch(&drawing, rgb)
        }
        Some(index) => {
            sent.1[index] = Some(plan.parts[index]);
            crosshair::grid_part_patch(&drawing, index)
        }
    };
    GRID_SENT.with(|cell| cell.set(sent));
    send(&patch, "drawing grid");
}

/// What the jump reset reads of its settings every frame.
struct JumpLive {
    on: bool,
    window: u16,
    knockback: f32,
}

/// Jump reset: notes your jumps and the hits you take, and shows how far apart they were. Hits
/// come from the host's combat feed (the local player being hurt); jumps from the jump key's
/// presses in gameplay. Only an unused jump or hit within `jumpreset::RELEVANT` of the other
/// counts, so ordinary jumping says nothing.
pub(super) fn jump_reset_frame(
    controls: Option<&input::Controls>,
    delta: f32,
    seconds: f32,
    session: u64,
) {
    // Only the three values this uses: the settings hold the hit log too, which is not worth
    // copying every frame.
    let live = settings_with(|s| JumpLive {
        on: s.jump_reset.on,
        window: s.jump_reset.window,
        knockback: s.jump_reset.knockback,
    });
    let (mut tracker, mut feed, known_session) = JUMP_RESET.with(StdCell::get);
    let mut watch = JUMP_WATCH.with(StdCell::get);
    let (mut verdict, mut outcome) = JUMP_VERDICT.with(StdCell::get);
    if !live.on || session != known_session {
        // Off, or another world: start clean, so switching on never judges something old.
        (tracker, feed, watch) = (
            jumpreset::Tracker::new(),
            jumpreset::Feed::new(),
            jumpreset::Watch::new(),
        );
        (verdict, outcome) = (None, None);
    }
    if live.on {
        tracker.step(delta);
        if headroom() >= COMBAT_RESERVE {
            const LIMIT: usize = mod_api::MAX_COMBAT_READ_EVENTS;
            note_read();
            if let Ok(events) = player_state::read_combat(feed.after, LIMIT as u32) {
                let mut batch = [(0u64, false); LIMIT];
                let count = events.len().min(LIMIT);
                for (slot, event) in batch.iter_mut().zip(&events) {
                    *slot = (
                        event.sequence,
                        event.local && event.kind == player_state::CombatKind::Hurt,
                    );
                }
                for _ in 0..feed.take(&batch[..count], LIMIT) {
                    // Fall damage is the same kind of event but no hit: landing stops you dead,
                    // which would read as a perfect reset.
                    if watch.falling() {
                        continue;
                    }
                    // A new hit: what the last one showed is over, and the movement after this
                    // one is watched for knockback.
                    (verdict, outcome) = (None, None);
                    watch.hit();
                    if let Some(timing) = tracker.hit(live.window) {
                        verdict = Some((timing, seconds));
                    }
                }
            }
        }
        // Your velocity every frame, so the frame of a hit has one before it to compare with.
        let motion = if headroom() >= COMBAT_RESERVE {
            note_read();
            player_state::read_motion().ok().flatten()
        } else {
            None
        };
        let velocity = motion
            .as_ref()
            .map(|motion| [motion.velocity.0, motion.velocity.1, motion.velocity.2]);
        watch.record(velocity, motion.as_ref().map(|motion| motion.on_ground));
        if let Some(found) = watch.sample(delta, velocity, live.knockback) {
            // A reset needs a jump near the hit too; a wall or corner also kills a shove.
            let found = found.judged(verdict.map(|(timing, _)| timing));
            outcome = Some((found, seconds));
            // Keep the numbers for tuning: written to the settings file below.
            let timing = verdict.map(|(timing, _)| timing);
            {
                let offset = timing.map_or(String::from("x"), |t| {
                    format!(
                        "{}",
                        if t.before {
                            -i32::from(t.ms)
                        } else {
                            i32::from(t.ms)
                        }
                    )
                });
                let row = format!(
                    "{}|{offset}|{}",
                    (found.knockback * 1000.0).round(),
                    watch.recording()
                );
                // A jump reset is also kept in a list of its own, so it is not pushed out by the
                // ordinary hits after it.
                if found.reset {
                    resets_with(|resets| {
                        jumpreset::push_capped(resets, row.clone(), jumpreset::RESETS_KEPT);
                    });
                }
                JUMP_TRACES.with(|traces| {
                    jumpreset::push_capped(&mut traces.borrow_mut(), row, jumpreset::LOG_LEN);
                });
            }
            let mut all = settings();
            all.jump_reset
                .log
                .push(jumpreset::Row::new(timing, found, watch.trace()));
            set_settings(all);
            JUMP_LOG_UNSAVED.with(|unsaved| unsaved.set(true));
        }
        watch.ground(motion.as_ref().map(|motion| motion.on_ground));
        // A hurt that never showed the shove (fire, poison, ...) was no hit: show nothing.
        if watch.take_dropped() {
            verdict = None;
        }
        // Presses count only in gameplay with the menu closed.
        if let Some(controls) = controls.filter(|c| c.focused && c.gameplay && !c.panel_open) {
            let pressed = JUMP_KEYS.with(|keys| {
                let keys = keys.borrow();
                controls.keys_pressed.iter().any(|key| {
                    if keys.is_empty() {
                        key == "Space"
                    } else {
                        keys.contains(key)
                    }
                })
            });
            if pressed && let Some(timing) = tracker.press(live.window) {
                verdict = Some((timing, seconds));
            }
        }
    }
    JUMP_RESET.with(|state| state.set((tracker, feed, session)));
    JUMP_WATCH.with(|stored| stored.set(watch));
    JUMP_VERDICT.with(|stored| stored.set((verdict, outcome)));
    if JUMP_LOG_UNSAVED.with(StdCell::get) && headroom() >= LOG_SAVE_RESERVE {
        JUMP_LOG_UNSAVED.with(|unsaved| unsaved.set(false));
        save_settings(&settings());
    }
    // The layer is published by the arrow counter; patch it only once it is up.
    if !HUD_PUBLISHED.with(StdCell::get) {
        JUMP_SHOWN.with(|shown| shown.set(None));
        return;
    }
    // On its own page the readout shows a sample, so it can be seen without being hit.
    let previewing = live.on
        && controls.is_some_and(|c| c.panel_open)
        && MENU.with(StdCell::get).page == panel::Page::JumpReset;
    let view = if previewing {
        jumpreset::SAMPLE
    } else {
        // Each part stays up for `SHOW` seconds after it was made.
        let fresh = |at: f32| live.on && seconds - at <= jumpreset::SHOW;
        jumpreset::View {
            // The timing waits until the hit is known to be a hit: its shove has shown.
            timing: verdict
                .filter(|(_, at)| fresh(*at) && (outcome.is_some() || watch.shoved()))
                .map(|(timing, _)| timing),
            outcome: outcome.filter(|(_, at)| fresh(*at)).map(|(found, _)| found),
        }
    };
    let shown = JUMP_SHOWN.with(StdCell::get);
    // Hidden is hidden, whatever it would have shown.
    if shown.is_some_and(|shown| shown == view || (shown.is_hidden() && view.is_hidden())) {
        return;
    }
    if !hud_write() {
        return;
    }
    match hud::update_surface(&jumpreset::patch(&view)) {
        Ok(()) => JUMP_SHOWN.with(|s| s.set(Some(view))),
        Err(error) => {
            // The layer is gone (or was refused): the arrow counter publishes it again.
            HUD_PUBLISHED.with(|published| published.set(false));
            HUD_RETRY_AT.with(|at| at.set(seconds + HUD_RETRY_SECONDS));
            report_once(&format!("Boldy Mods: jump reset failed ({error})"));
        }
    }
}

/// The clicks of each button in the last second (the right one only when it is shown) and the
/// readout's layout at text scale 1.
fn cps_reading(seconds: f32, settings: &cps::Cps) -> (u32, Option<u32>, cps::Layout) {
    let (left_now, right_now) = cps_counts(seconds, settings);
    (
        left_now,
        right_now,
        cps_widths().layout(left_now, right_now),
    )
}

/// The clicks of each button in the last second (the right one only when it is shown).
fn cps_counts(seconds: f32, settings: &cps::Cps) -> (u32, Option<u32>) {
    CLICKS.with(|clicks| {
        let (left, right) = &mut *clicks.borrow_mut();
        let left_now = left.per_second(seconds);
        (left_now, settings.right.then(|| right.per_second(seconds)))
    })
}

/// The CPS text's pieces at text scale 1: `CPS:`, `FPS:`, a digit, a space and the bar.
///
/// The host may not be able to measure yet, or may measure before its fonts are ready, so a
/// reading is not trusted until a second one agrees (see [`cps::WidthProbe`]); once they do, this
/// is only a cell read. When the widths change, what was laid out with the old ones is redone.
fn cps_widths() -> cps::Widths {
    let mut probe = CPS_WIDTHS.with(StdCell::get);
    let now = SECONDS.with(StdCell::get);
    if probe.due(now) {
        if probe.observe(now, measure_cps_widths()) {
            CPS_SHOWN.with(|shown| shown.set(None));
            FPS_SHOWN.with(|shown| shown.set(None));
        }
        CPS_WIDTHS.with(|cell| cell.set(probe));
    }
    probe.widths()
}

/// Measures the pieces of the CPS and FPS text, `None` unless every one could be measured. Each is
/// measured with a digit beside it, as a trailing space may not count.
fn measure_cps_widths() -> Option<cps::Widths> {
    let measure = |text: &str| {
        hud::measure_text(text, hud::Font::Native)
            .ok()
            .filter(|width| width.is_finite() && *width > 0.0)
    };
    let digit = measure("0")?;
    Some(cps::Widths {
        caption: (measure("CPS:0")? - digit).max(0.0),
        fps_caption: (measure("FPS:0")? - digit).max(0.0),
        digit,
        space: (measure(" 0")? - digit).max(1.0),
        bar: (measure("|0")? - digit).max(1.0),
    })
}

/// How long to wait before trying again after the settings file could not be written, in seconds.
const SAVE_RETRY_SECONDS: f32 = 5.0;

thread_local! {
    /// The time before which a failed save of a moved HUD element is not tried again.
    static COUNTER_SAVE_AT: StdCell<f32> = const { StdCell::new(0.0) };
}

/// Writes down a moved HUD element's place once a frame can afford it (a save is about 65,000
/// fuel, and more with a big crosshair drawing) and has no other heavy job. Until it is written
/// the change stays pending: a frame short of fuel, or a failed write, leaves it for a later one,
/// and a further move meanwhile is saved by the same write.
fn save_moved_hud(seconds: f32) {
    if seconds < COUNTER_SAVE_AT.with(StdCell::get)
        || HEAVY_FRAME.with(StdCell::get)
        || headroom() < SAVE_RESERVE
    {
        return;
    }
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    if save_settings(&settings()) {
        COUNTER_UNSAVED.with(|unsaved| unsaved.set(false));
    } else {
        COUNTER_SAVE_AT.with(|at| at.set(seconds + SAVE_RETRY_SECONDS));
    }
}

/// Forgets the armor bindings built so far, so the next equipment read builds them again.
/// Their display has to be refreshed too.
fn armor_forget() {
    ARMOR_PRINT.with(|print| print.set(None));
    ARMOR_PENDING.with(|pending| {
        if pending.borrow_mut().take().is_some() {
            ARMOR_DIRTY.with(|dirty| dirty.set(true));
        }
    });
}

/// Recounts the arrows when the inventory changed since they were last counted. The equipment
/// revision is a cheap check every frame; the inventory itself is only read after a change and
/// with enough fuel left, otherwise in a later frame.
///
/// The armor display is built from the same read, but only while the armor HUD is on and only
/// when what its four cells show changed (by their [`crate::armor::Print`]), so an arrow used or an
/// unrelated item moved formats nothing. Read while it is off, the armor is left unbuilt, and
/// turning it on reads the equipment again.
pub(super) fn refresh_quiver() {
    note_read();
    let revisions = match player_state::read_revisions() {
        Ok(Some(revisions)) => revisions,
        Ok(None) => {
            // Not in a world.
            QUIVER.with(|q| q.set(None));
            armor_forget();
            return;
        }
        Err(_) => {
            // Shown on the counter itself as a yellow ?, with this message once.
            if !NO_PLAYER_STATUS.with(|n| n.replace(true)) {
                show_label(
                    "Boldy Mods: allow the Player status permission (Mods tab) to count arrows",
                );
            }
            QUIVER.with(|q| q.set(None));
            armor_forget();
            return;
        }
    };
    NO_PLAYER_STATUS.with(|n| n.set(false));
    let armor_on = settings_with(|s| s.armor.on);
    // The armor wants a read when it is on but has nothing built for the current equipment.
    let armor_missing = armor_on && ARMOR_PRINT.with(StdCell::get).is_none();
    let known = QUIVER.with(StdCell::get);
    if !armor_missing
        && known.is_some_and(|(session, revision, _)| {
            session == revisions.session && revision == revisions.equipment
        })
    {
        return;
    }
    if headroom() < INVENTORY_RESERVE {
        return;
    }
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    note_read();
    let counted = crate::arena::scratch(player_state::read_equipment, |equipment| {
        let Ok(Some(equipment)) = equipment else {
            return None;
        };
        fn cell(slot: &player_state::Slot) -> Option<(&str, u16)> {
            let item = slot.item.as_ref()?;
            Some((item.identifier.as_deref()?, item.count))
        }
        let quiver = Quiver::count(
            equipment.inventory.iter().map(cell),
            cell(&equipment.offhand),
        );
        let armor = if armor_on {
            let print = armor_print(equipment);
            // Unchanged armor builds nothing, unless the bindings it would be are not there.
            let unchanged = ARMOR_PRINT.with(StdCell::get) == Some(print)
                && ARMOR_PENDING.with(|pending| pending.borrow().is_some());
            (!unchanged).then(|| (print, armor_bindings(equipment)))
        } else {
            None
        };
        Some((quiver, armor))
    });
    // Keyed by the revision checked above: a change since then is caught next frame.
    let Some((quiver, armor)) = counted else {
        return;
    };
    QUIVER.with(|q| q.set(Some((revisions.session, revisions.equipment, quiver))));
    if !armor_on {
        // The armor changed unseen, so it is built from a fresh read when it is switched on.
        ARMOR_PRINT.with(|print| print.set(None));
    } else if let Some((print, (bindings, percents))) = armor {
        ARMOR_PERCENTS.with(|stored| stored.set(percents));
        ARMOR_PENDING.with(|pending| *pending.borrow_mut() = Some(bindings));
        ARMOR_PRINT.with(|stored| stored.set(Some(print)));
        ARMOR_DIRTY.with(|dirty| dirty.set(true));
    }
}

/// What the armor cells would show, as a summary to compare, without building the bindings.
fn armor_print(equipment: &player_state::Equipment) -> u64 {
    let mut print = crate::armor::Print::new();
    for slot in equipment.armor.iter().take(4) {
        print.cell(shown_armor(slot).map(|item| {
            (
                item.identifier.as_deref().unwrap_or_default(),
                item.metadata,
                crate::armor::durability(item.damage, item.max_durability),
            )
        }));
    }
    print.finish(equipment.armor.len().min(4))
}

/// The item an armor cell shows: a known cell holding an item the host named with a safe
/// identifier.
fn shown_armor(slot: &player_state::Slot) -> Option<&player_state::Item> {
    slot.item.as_ref().filter(|item| {
        slot.known
            && item
                .identifier
                .as_deref()
                .is_some_and(crate::armor::valid_id)
    })
}

/// The armor HUD's bindings for the equipment, and each cell's durability percent for its bar.
fn armor_bindings(equipment: &player_state::Equipment) -> (String, [Option<u32>; 4]) {
    use std::fmt::Write;
    // The four slots always show while the armor HUD is on, empty or not.
    let mut armor = String::from(r##""#armor_any":true,"##);
    let mut percents = [None; 4];
    for (index, slot) in equipment.armor.iter().take(4).enumerate() {
        if index > 0 {
            armor.push(',');
        }
        let item = shown_armor(slot);
        let _ = write!(
            armor,
            r##""#armor_visible_{index}":{},"#armor_text_{index}":""##,
            item.is_some()
        );
        if let Some(item) = item {
            if let Some(percent) = crate::armor::durability(item.damage, item.max_durability) {
                let _ = write!(armor, "{percent}%");
            }
            if let Some(id) = item.identifier.as_deref() {
                let _ = write!(
                    armor,
                    r##"","#armor_item_{index}":{{"item":{{"identifier":"{id}","metadata":{}}}}}"##,
                    item.metadata
                );
            }
        } else {
            armor.push('"');
        }
        let percent =
            item.and_then(|item| crate::armor::durability(item.damage, item.max_durability));
        let _ = write!(
            armor,
            r##","#armor_bar_visible_{index}":{}"##,
            percent.is_some()
        );
        if let Some(percent) = percent {
            percents[index] = Some(percent);
            let color = if percent > 50 {
                "[0.25,1,0.1,1]"
            } else if percent > 20 {
                "[1,0.75,0.1,1]"
            } else {
                "[1,0.15,0.1,1]"
            };
            let _ = write!(armor, r##","#armor_bar_color_{index}":{color}"##);
        }
    }
    // Missing cells must clear their old display too.
    for index in equipment.armor.len().min(4)..4 {
        if index > 0 {
            armor.push(',');
        }
        let _ = write!(
            armor,
            r##""#armor_visible_{index}":false,"#armor_bar_visible_{index}":false,"#armor_text_{index}":"""##
        );
    }
    (armor, percents)
}

#[cfg(test)]
mod tests;

thread_local! {
    static EFFECTS_NEXT: StdCell<f32> = const { StdCell::new(0.0) };
    /// What the timers last sent, as a [`crate::effects::fingerprint`]; `None` is blank.
    static EFFECTS_SHOWN: StdCell<Option<u64>> = const { StdCell::new(None) };
}

/// Poll at five Hz so newly applied effects appear promptly; an unchanged poll sends nothing.
/// Switched off or out of a world it makes no host call, and blanks the labels once.
pub(super) fn effects_frame(seconds: f32, in_world: bool) {
    if !HUD_PUBLISHED.with(StdCell::get) {
        return;
    }
    let shown = EFFECTS_SHOWN
        .with(StdCell::get)
        .unwrap_or(crate::effects::BLANK);
    let wanted = in_world && settings_with(|s| s.effect_timers);
    if !wanted && shown == crate::effects::BLANK {
        return;
    }
    if (wanted && seconds < EFFECTS_NEXT.with(StdCell::get))
        || headroom() < 90_000
        || HEAVY_FRAME.with(StdCell::get)
    {
        return;
    }
    let mut codes = [0; crate::effects::SLOTS];
    if wanted {
        EFFECTS_NEXT.with(|next| next.set(seconds + 0.2));
        note_read();
        match player_state::read_effects() {
            Ok(Some(value)) => crate::effects::fill(
                &mut codes,
                value
                    .effects
                    .iter()
                    .map(|effect| (effect.effect_id, effect.remaining_ticks)),
            ),
            Ok(None) => {}
            Err(_) => return,
        }
    }
    let print = crate::effects::fingerprint(&codes);
    if print == shown {
        return;
    }
    if !hud_write() {
        EFFECTS_NEXT.with(|next| next.set(seconds));
        return;
    }
    let patch = format!(
        r#"{{"binding_changes":{{{}}}}}"#,
        crate::effects::bindings(&codes)
    );
    if hud::update_surface(&patch).is_ok() {
        EFFECTS_SHOWN.with(|cell| cell.set(Some(print)));
    } else {
        HUD_PUBLISHED.with(|published| published.set(false));
    }
}

type ArmorShown = (crate::armor::Armor, [f32; 2], Option<counter::Edit>);

thread_local! {
    static ARMOR_PENDING: RefCell<Option<String>> = const { RefCell::new(None) };
    static ARMOR_DIRTY: StdCell<bool> = const { StdCell::new(true) };
    /// The [`crate::armor::Print`] the pending bindings were built from; `None` while nothing is
    /// built for the current equipment (armor off when it last changed, or not in a world).
    static ARMOR_PRINT: StdCell<Option<u64>> = const { StdCell::new(None) };
    static ARMOR_PERCENTS: StdCell<[Option<u32>; 4]> = const { StdCell::new([None; 4]) };
    static ARMOR_SETTINGS: StdCell<Option<ArmorShown>> = const { StdCell::new(None) };
}

/// Publish only when the shared equipment snapshot changes.
fn armor_frame(seconds: f32, viewport: Option<[f32; 2]>, edit: Option<counter::Edit>) {
    let Some(viewport) = viewport else {
        return;
    };
    let settings = settings_with(|s| s.armor);
    if ARMOR_SETTINGS.with(StdCell::get) != Some((settings, viewport, edit)) {
        ARMOR_DIRTY.with(|dirty| dirty.set(true));
    }
    if !ARMOR_DIRTY.with(StdCell::get) || HEAVY_FRAME.with(StdCell::get) || headroom() < 40_000 {
        return;
    }
    if !hud_write() {
        return;
    }
    let result = ARMOR_PENDING.with(|value| {
        let value = value.borrow();
        let fallback;
        // Bindings not built for the current equipment (armor was off when it changed) are not
        // shown; the next equipment read builds them and marks the display dirty again.
        let built = ARMOR_PRINT.with(StdCell::get).is_some();
        let pending = match value.as_deref().filter(|_| settings.on && built) {
            Some(pending) => pending,
            None => {
                fallback = crate::armor::empty_bindings();
                &fallback
            }
        };
        let layout = crate::armor::layout_bindings(
            settings,
            viewport,
            edit,
            ARMOR_PERCENTS.with(StdCell::get),
        );
        hud::update_surface(&format!(r#"{{"binding_changes":{{{pending},{layout}}}}}"#))
    });
    if result.is_ok() {
        ARMOR_DIRTY.with(|dirty| dirty.set(false));
        ARMOR_SETTINGS.with(|seen| seen.set(Some((settings, viewport, edit))));
    } else {
        HUD_PUBLISHED.with(|published| published.set(false));
        HUD_RETRY_AT.with(|at| at.set(seconds + HUD_RETRY_SECONDS));
    }
}

/// Publish only after bindings or session visibility changes.
pub(super) fn hotbar_frame(seconds: f32, in_world: bool) {
    if !HUD_PUBLISHED.with(StdCell::get) {
        return;
    }
    let revision = HOTBAR_KEYS.with(|keys| keys.borrow().0);
    let visible = in_world && revision.is_some() && settings_with(|s| s.hotbar_keys);
    let key = (revision.unwrap_or(0), visible);
    if HOTBAR_SHOWN.with(StdCell::get) == Some(key)
        || HEAVY_FRAME.with(StdCell::get)
        || headroom() < 40_000
    {
        return;
    }
    if !hud_write() {
        return;
    }
    let patch = HOTBAR_KEYS.with(|keys| {
        use std::fmt::Write;
        let keys = keys.borrow();
        let mut out = format!(r##"{{"binding_changes":{{"#hotkeys_visible":{}"##, key.1);
        for (i, label) in keys.1.iter().enumerate() {
            let value = crate::panel::json_string(label.as_deref().unwrap_or(""));
            let _ = write!(out, r##","#hotkey_{i}":{value}"##);
        }
        out.push_str("}}");
        out
    });
    if hud::update_surface(&patch).is_ok() {
        HOTBAR_SHOWN.with(|shown| shown.set(Some(key)));
    } else {
        HUD_PUBLISHED.with(|published| published.set(false));
        HUD_RETRY_AT.with(|at| at.set(seconds + HUD_RETRY_SECONDS));
    }
}
