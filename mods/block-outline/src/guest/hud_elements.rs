//! The HUD elements: the arrow counter, the CPS readout and the editor that moves them. The
//! shared state (settings, menu, the published layer) lives in the parent module.

use super::*;

/// How long to wait before publishing the HUD layer again after the host refused it.
const HUD_RETRY_SECONDS: f32 = 5.0;

/// Fuel a frame needs left to publish the HUD layer; with less, a later frame does.
const SURFACE_RESERVE: u64 = 100_000;

/// Fuel a frame needs left to read the inventory after it changed; with less, a later frame does.
const INVENTORY_RESERVE: u64 = 90_000;

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

/// While moving, with the mouse alone: the arrow counter and the CPS readout both show their
/// frames, and the editor ([`hudedit`]) moves or resizes what is dragged. Enter, Escape or the
/// right button finish. Returns how each frame should look: the arrow counter's, then the CPS
/// readout's.
fn drag_elements(
    controls: &input::Controls,
    settings: &mut Settings,
    viewport: [f32; 2],
    // Each element's size at scale 1: the arrow counter, then the CPS readout.
    bases: [[f32; 2]; 2],
    seconds: f32,
) -> [counter::Edit; 2] {
    let before = (settings.counter, settings.cps);
    let mut items = [
        hudedit::Item {
            pos: settings.counter.pos,
            scale: settings.counter.scale,
        },
        hudedit::Item {
            pos: settings.cps.pos,
            scale: settings.cps.scale,
        },
    ];
    let mut state = MENU.with(StdCell::get);
    let mut editor = EDITOR.with(StdCell::get);
    // The element grabbed last (or whose button opened this) takes the keys.
    editor.selected = usize::from(state.moving_cps);
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
    if (settings.counter, settings.cps) != before {
        SETTINGS.with(|s| s.set(*settings));
        COUNTER_UNSAVED.with(|unsaved| unsaved.set(true));
    }
    state.moving_cps = editor.selected == 1;
    if ["Escape", "Enter", "NumpadEnter", "MouseRight"]
        .iter()
        .any(|key| pressed(key))
    {
        state.moving = false;
        state.moving_cps = false;
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
        // Publishing writes the layer's starting bindings: wait for a frame with fuel to spare.
        if budget::remaining_fuel() < SURFACE_RESERVE {
            return;
        }
        COUNTER_SHOWN.with(|shown| shown.set(None));
        CPS_SHOWN.with(|shown| shown.set(None));
        if let Err(error) = hud::set_surface(&counter::surface()) {
            HUD_RETRY_AT.with(|at| at.set(seconds + HUD_RETRY_SECONDS));
            if SETTINGS.with(StdCell::get).counter.on {
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
    let mut settings = SETTINGS.with(StdCell::get);
    if let Some(viewport) = viewport.filter(|_| settings.counter.pos.is_fraction()) {
        // An older settings file kept the position as a share of the screen: pin it to the
        // nearest edge or the middle now, where it is on screen, and save that.
        settings.counter.pos = counter::Pos::anchored(settings.counter.pos.at(viewport), viewport);
        SETTINGS.with(|s| s.set(settings));
        COUNTER_UNSAVED.with(|unsaved| unsaved.set(true));
    }
    if settings.counter.on {
        refresh_quiver();
    }
    let quiver = QUIVER.with(StdCell::get).map(|(_, _, quiver)| quiver);
    let arrows = quiver.map_or(0, |quiver| quiver.arrows);
    let text_width = count_width(arrows);
    let mut edits = [counter::Edit::default(); 2];
    if let (true, Some(controls), Some(viewport)) = (moving, controls, viewport) {
        let (_, _, layout) = cps_reading(seconds, &settings.cps);
        let bases = [counter::size(1.0, text_width), cps::base(layout.total)];
        edits = drag_elements(controls, &mut settings, viewport, bases, seconds);
    }
    // A moved counter is saved once, when moving ends, not on every step of a drag.
    let moving = MENU.with(StdCell::get).moving && panel_open;
    EDIT_CPS.with(|edit| edit.set(moving.then_some(edits[1])));
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
    let settings = SETTINGS.with(StdCell::get).cps;
    if settings.on
        && let Some(controls) = controls.filter(|c| c.focused && c.gameplay && !c.panel_open)
    {
        let (mut left, mut right) = CLICKS.with(StdCell::get);
        let mut clicked = false;
        for key in &controls.keys_pressed {
            match key.as_str() {
                "MouseLeft" => left.click(seconds),
                "MouseRight" => right.click(seconds),
                _ => continue,
            }
            clicked = true;
        }
        if clicked {
            CLICKS.with(|clicks| clicks.set((left, right)));
        }
    }
    // The layer is published by the arrow counter; patch it only once it is up.
    if !HUD_PUBLISHED.with(StdCell::get) {
        CPS_SHOWN.with(|shown| shown.set(None));
        return;
    }
    let viewport = hud_size.map(|v| [v[0], v[1]]);
    let edit = EDIT_CPS.with(StdCell::get);
    let view = match viewport.filter(|_| settings.on || edit.is_some()) {
        Some(viewport) => {
            let (left, right, layout) = cps_reading(seconds, &settings);
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

/// The clicks of each button in the last second (the right one only when it is shown) and the
/// readout's layout at text scale 1.
fn cps_reading(seconds: f32, settings: &cps::Cps) -> (u32, Option<u32>, cps::Layout) {
    let (mut left, mut right) = CLICKS.with(StdCell::get);
    let left_now = left.per_second(seconds);
    let right_now = settings.right.then(|| right.per_second(seconds));
    CLICKS.with(|clicks| clicks.set((left, right)));
    (
        left_now,
        right_now,
        cps_widths().layout(left_now, right_now),
    )
}

/// The CPS text's pieces measured at text scale 1, once per GUI scale: `CPS:`, a digit, a
/// space and the bar. Measured with a digit beside them, as a trailing space may not count.
fn cps_widths() -> cps::Widths {
    if let Some(widths) = CPS_WIDTHS.with(StdCell::get) {
        return widths;
    }
    let measure = |text: &str| {
        hud::measure_text(text, hud::Font::Native)
            .ok()
            .filter(|width| width.is_finite() && *width > 0.0)
    };
    let widths = match (measure("0"), measure("CPS:0"), measure(" 0"), measure("|0")) {
        (Some(digit), Some(caption), Some(space), Some(bar)) => cps::Widths {
            caption: (caption - digit).max(0.0),
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
    if budget::remaining_fuel() < INVENTORY_RESERVE {
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
