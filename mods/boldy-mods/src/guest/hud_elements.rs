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

/// While moving, with the mouse alone: the arrow counter, the CPS readout and the FPS counter all
/// show their frames, and the editor ([`hudedit`]) moves or resizes what is dragged. Enter,
/// Escape or the right button finish. Returns how each frame should look, in that order.
fn drag_elements(
    controls: &input::Controls,
    settings: &mut Settings,
    viewport: [f32; 2],
    // Each element's size at scale 1: the arrow counter, the CPS readout, the FPS counter.
    bases: [[f32; 2]; 3],
    seconds: f32,
) -> [counter::Edit; 3] {
    let before = (settings.counter, settings.cps, settings.fps);
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
    ];
    let mut state = MENU.with(StdCell::get);
    let mut editor = EDITOR.with(StdCell::get);
    // The element grabbed last (or whose button opened this) takes the keys.
    editor.selected = if state.moving_fps {
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
    if (settings.counter, settings.cps, settings.fps) != before {
        set_settings(*settings);
        COUNTER_UNSAVED.with(|unsaved| unsaved.set(true));
    }
    state.moving_cps = editor.selected == 1;
    state.moving_fps = editor.selected == 2;
    if ["Escape", "Enter", "NumpadEnter", "MouseRight"]
        .iter()
        .any(|key| pressed(key))
    {
        state.moving = false;
        state.moving_cps = false;
        state.moving_fps = false;
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
            CPS_WIDTHS.with(|w| w.set(None));
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
    let mut settings = settings();
    if let Some(viewport) = viewport.filter(|_| settings.counter.pos.is_fraction()) {
        // An older settings file kept the position as a share of the screen: pin it to the
        // nearest edge or the middle now, where it is on screen, and save that.
        settings.counter.pos = counter::Pos::anchored(settings.counter.pos.at(viewport), viewport);
        set_settings(settings);
        COUNTER_UNSAVED.with(|unsaved| unsaved.set(true));
    }
    if settings.counter.on {
        refresh_quiver();
    }
    let quiver = QUIVER.with(StdCell::get).map(|(_, _, quiver)| quiver);
    let arrows = quiver.map_or(0, |quiver| quiver.arrows);
    let text_width = count_width(arrows);
    let mut edits = [counter::Edit::default(); 3];
    if let (true, Some(controls), Some(viewport)) = (moving, controls, viewport) {
        let (_, _, layout) = cps_reading(seconds, &settings.cps);
        let fps_width = cps_widths().fps_width(FPS.with(StdCell::get).fps());
        let bases = [
            counter::size(1.0, text_width),
            cps::base(layout.total),
            fps::base(fps_width),
        ];
        edits = drag_elements(controls, &mut settings, viewport, bases, seconds);
    }
    // A moved counter is saved once, when moving ends, not on every step of a drag.
    let moving = MENU.with(StdCell::get).moving && panel_open;
    EDIT_CPS.with(|edit| edit.set(moving.then_some(edits[1])));
    EDIT_FPS.with(|edit| edit.set(moving.then_some(edits[2])));
    if !moving && COUNTER_UNSAVED.with(StdCell::get) {
        COUNTER_UNSAVED.with(|unsaved| unsaved.set(false));
        EDITOR.with(|editor| editor.set(hudedit::Editor::default()));
        save_settings(&settings);
    }
    let counter = settings.counter;
    let on_page = panel_open && state.page == panel::Page::Counter;
    let holding = || {
        !counter.bow_only
            || quiver.is_some_and(|quiver| {
                let selected = player_state::read_selected_slot()
                    .ok()
                    .flatten()
                    .map(|selected| selected.slot);
                quiver.holding_bow(selected)
            })
    };
    // Without the Player status permission the count is unknown: show a ? so the counter can
    // still be found and placed, and the reason is clear.
    let unknown = quiver.is_none() && NO_PLAYER_STATUS.with(StdCell::get);
    // While the HUD is edited every element is shown, so each can be placed.
    let visible = moving
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
                edit: moving.then_some(edits[0]),
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

/// Keeps the FPS counter on the HUD up to date. The frames are counted in `frame_body`; the
/// number changes every [`fps::EVERY`] seconds, so the HUD is patched about twice a second.
pub(super) fn fps_frame(seconds: f32, hud_size: Option<[f32; 3]>) {
    let settings = settings_with(|s| s.fps);
    // The layer is published by the arrow counter; patch it only once it is up.
    if !HUD_PUBLISHED.with(StdCell::get) {
        FPS_SHOWN.with(|shown| shown.set(None));
        return;
    }
    let edit = EDIT_FPS.with(StdCell::get);
    let view = match hud_size.filter(|_| settings.on || edit.is_some()) {
        Some(viewport) => {
            let fps = FPS.with(StdCell::get).fps();
            let width = cps_widths().fps_width(fps);
            let scale = cps::clamp_scale(settings.scale);
            fps::View {
                visible: true,
                fps,
                scale,
                width,
                offset: counter::place(
                    settings.pos,
                    fps::size(scale, width),
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

/// The custom crosshair: shown in place of the game's own while it is switched on, and only
/// where the game would draw its own (not with the HUD hidden or in third person). On its menu
/// page it shows whatever the game does, so changes can be seen as they are made. The game's
/// crosshair is hidden only while this one shows, so there is never no crosshair at all.
///
/// A changed crosshair is sent in steps, one a frame: its header, then each part that changed
/// (see [`crosshair`]); showing and hiding it is a binding. A refused patch is reported once
/// and counted as sent, so it never turns into a patch every frame.
pub(super) fn crosshair_frame(controls: Option<&input::Controls>, hud_size: Option<[f32; 3]>) {
    let look = settings_with(|s| s.crosshair);
    let published = HUD_PUBLISHED.with(StdCell::get);
    // The game's crosshair gives way only while this one shows: not while it is still being
    // sent, nor where it cannot show (the HUD's size unknown), so there is always one.
    let replace = look.on && published && CROSSHAIR_VISIBLE.with(StdCell::get) == Some(true);
    if VANILLA_CROSSHAIR_HIDDEN.with(StdCell::get) != replace && hud_write() {
        let hidden: &[hud::VanillaElement] = if replace {
            &[hud::VanillaElement::Crosshair]
        } else {
            &[]
        };
        if hud::set_hidden_elements(hidden).is_ok() {
            VANILLA_CROSSHAIR_HIDDEN.with(|cell| cell.set(replace));
        }
    }
    // The layer is published by the arrow counter; patch it only once it is up.
    if !published {
        return;
    }
    let previewing = controls.is_some_and(|c| c.panel_open)
        && MENU.with(StdCell::get).page == panel::Page::Crosshair;
    let view = match hud_size {
        Some(viewport) if look.on && (previewing || vanilla_crosshair_shown()) => {
            Some(crosshair::View { look, viewport })
        }
        _ => None,
    };
    let visible = CROSSHAIR_VISIBLE.with(StdCell::get);
    let Some(view) = view else {
        if visible != Some(false) && hud_write() {
            CROSSHAIR_VISIBLE.with(|shown| shown.set(Some(false)));
            send(&crosshair::visibility(false), "crosshair");
        }
        return;
    };
    // The shape and plan, worked out once for a view: a large drawing costs to work out.
    let planned = CROSSHAIR_PLAN.with(|plan| {
        plan.borrow()
            .as_ref()
            .is_some_and(|(known, ..)| *known == view)
    });
    if !planned {
        if HEAVY_FRAME.with(StdCell::get)
            || headroom() < crosshair::plan_cost(&view.look) + CROSSHAIR_SPARE
        {
            return;
        }
        let shape = crosshair::view_shape(&view);
        let plan = crosshair::plan(&view, &shape);
        CROSSHAIR_PLAN.with(|known| *known.borrow_mut() = Some((view, shape, plan)));
    }
    let sent = CROSSHAIR_SENT.with(StdCell::get);
    let patch = CROSSHAIR_PLAN.with(|known| {
        let known = known.borrow();
        let (_, shape, plan) = known.as_ref()?;
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
                crosshair::header_patch(&view, shape)
            }
            Some(index) => {
                sent.1[index] = Some(plan.parts[index]);
                crosshair::part_patch(&view, shape, index)
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

/// The CPS text's pieces measured at text scale 1, once per GUI scale: `CPS:`, `FPS:`, a digit,
/// a space and the bar. Measured with a digit beside them, as a trailing space may not count.
fn cps_widths() -> cps::Widths {
    if let Some(widths) = CPS_WIDTHS.with(StdCell::get) {
        return widths;
    }
    let measure = |text: &str| {
        hud::measure_text(text, hud::Font::Native)
            .ok()
            .filter(|width| width.is_finite() && *width > 0.0)
    };
    let widths = match (
        measure("0"),
        measure("CPS:0"),
        measure("FPS:0"),
        measure(" 0"),
        measure("|0"),
    ) {
        (Some(digit), Some(caption), Some(fps_caption), Some(space), Some(bar)) => cps::Widths {
            caption: (caption - digit).max(0.0),
            fps_caption: (fps_caption - digit).max(0.0),
            digit,
            space: (space - digit).max(1.0),
            bar: (bar - digit).max(1.0),
        },
        _ => cps::Widths::GUESS,
    };
    CPS_WIDTHS.with(|w| w.set(Some(widths)));
    widths
}

/// Recounts the arrows when the inventory changed since they were last counted. The equipment
/// revision is a cheap check every frame; the inventory itself is only read after a change and
/// with enough fuel left, otherwise in a later frame.
fn refresh_quiver() {
    let revisions = match player_state::read_revisions() {
        Ok(Some(revisions)) => revisions,
        Ok(None) => {
            // Not in a world.
            QUIVER.with(|q| q.set(None));
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
            return;
        }
    };
    NO_PLAYER_STATUS.with(|n| n.set(false));
    let known = QUIVER.with(StdCell::get);
    if known.is_some_and(|(session, revision, _)| {
        session == revisions.session && revision == revisions.equipment
    }) {
        return;
    }
    if headroom() < INVENTORY_RESERVE {
        return;
    }
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    let counted = crate::arena::scratch(player_state::read_equipment, |equipment| {
        let Ok(Some(equipment)) = equipment else {
            return None;
        };
        fn cell(slot: &player_state::Slot) -> Option<(&str, u16)> {
            let item = slot.item.as_ref()?;
            Some((item.identifier.as_deref()?, item.count))
        }
        Some(Quiver::count(
            equipment.inventory.iter().map(cell),
            cell(&equipment.offhand),
        ))
    });
    // Keyed by the revision checked above: a change since then is caught next frame.
    QUIVER.with(|q| q.set(counted.map(|quiver| (revisions.session, revisions.equipment, quiver))));
}

#[cfg(test)]
mod tests;
