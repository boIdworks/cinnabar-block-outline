//! The settings menu: opening and closing it, its keyboard and pointer events, key capture and saving.

use super::*;

/// The menu opens on its start page: once closed, forget the page that was shown.
pub(super) fn back_to_start_when_closed(controls: &input::Controls, seconds: f32) {
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
        moving_fps: false,
        drawing: false,
        draw_full: false,
        ..state
    };
    MENU.with(|m| m.set(state));
    push_menu(seconds, state);
}

/// Where the pointer is on the menu's card: hover help works it out first, every frame it moves.
pub(super) fn card_pointer(controls: &input::Controls) -> Option<[f32; 2]> {
    HOVER_LOOKUP
        .with(StdCell::get)
        .and_then(|((pointer, _, _), point, _)| {
            point.filter(|_| pointer.is_some() && pointer == controls.pointer)
        })
}

/// Captures a new menu key while the "Menu key" button waits for one; otherwise opens or closes
/// the menu when the chosen key is pressed (and says so, `true`, when it did). The panel's own
/// toggle key is a placeholder, because a panel cannot change its toggle key after loading.
pub(super) fn handle_menu_key(controls: &input::Controls, seconds: f32) -> bool {
    // Remember when the menu closed: for a moment after it, the game is not yet back in gameplay,
    // and a quick second press (to reopen it) must still be answered.
    if MENU_SEEN_OPEN.with(|seen| seen.replace(controls.panel_open)) && !controls.panel_open {
        MENU_CLOSED_AT.with(|at| at.set(seconds));
    }
    let state = MENU.with(StdCell::get);
    if state.capturing_fullbright {
        capture_optional_key(KeyButton::Fullbright, controls, seconds);
        return false;
    }
    if state.capturing_zoom {
        capture_optional_key(KeyButton::Zoom, controls, seconds);
        return false;
    }
    if state.capturing {
        if !controls.panel_open {
            // The menu closed mid-capture; keep the old key.
            set_capturing(KeyButton::Menu, false, seconds);
            return false;
        }
        let Some(key) = controls
            .keys_pressed
            .iter()
            .find(|key| !key.starts_with("Mouse"))
        else {
            return false;
        };
        if key != "Escape"
            && let Some(name) = KeyName::new(key)
        {
            let mut settings = settings();
            settings.menu_key = name;
            set_settings(settings);
            save_settings(&settings);
            reserve_keys(&settings);
        }
        set_capturing(KeyButton::Menu, false, seconds);
        return false;
    }
    // Keys are being typed into a box: the menu key must not close the menu.
    if state.typing.is_some() {
        return false;
    }
    toggle_fullbright_by_key(controls);
    if !controls.focused {
        return false;
    }
    let key = settings_with(|s| s.menu_key);
    if !controls
        .keys_pressed
        .iter()
        .any(|pressed| pressed == key.as_str())
    {
        return false;
    }
    // A press just after the last one acts on what that one asked for: the panel's state in
    // `controls` can lag a few frames behind a request.
    let asked = MENU_REQUEST
        .with(StdCell::get)
        .filter(|(_, at)| seconds - at < REQUEST_SECONDS);
    let open = asked.map_or(controls.panel_open, |(open, _)| open);
    let reopening = seconds - MENU_CLOSED_AT.with(StdCell::get) < REOPEN_SECONDS;
    if open {
        let _ = host_panel::close();
        MENU_REQUEST.with(|request| request.set(Some((false, seconds))));
        MENU_CLOSED_AT.with(|at| at.set(seconds));
        true
    } else if controls.gameplay || reopening {
        match host_panel::open() {
            Ok(()) => {
                MENU_REQUEST.with(|request| request.set(Some((true, seconds))));
                true
            }
            Err(error) => {
                report_once(&format!("Boldy Mods: menu unavailable ({error})"));
                false
            }
        }
    } else {
        false
    }
}

/// How long the panel's reported state may lag behind a request of the menu key, in seconds.
pub(super) const REQUEST_SECONDS: f32 = 0.35;

/// How long after the menu closed a press may open it again even before gameplay resumes.
pub(super) const REOPEN_SECONDS: f32 = 1.0;

/// Saves `settings`, telling the player once if that fails.
pub(super) fn save_settings(settings: &Settings) {
    let mut json = settings.to_json();
    // Extras ride along: the jump resets in full (read back at start), the recorded movement of
    // the last hits, and the fuel each feature uses (neither read back). The file may hold
    // `MAX_SETTINGS_BYTES`, so what does not fit is left out, the oldest first and the traces
    // before the resets.
    let report = fuel_report();
    let mut room = mod_api::MAX_SETTINGS_BYTES.saturating_sub(json.len() + report.len() + 600);
    let resets = resets_with(|rows| jumpreset::joined_within(rows, room));
    room = room.saturating_sub(resets.len());
    let traces = JUMP_TRACES.with(|rows| jumpreset::joined_within(&rows.borrow(), room));
    let mut extras = String::new();
    for (key, value) in [
        ("jump_reset_resets", &resets),
        ("jump_reset_trace", &traces),
        ("fuel_report", &report),
    ] {
        if !value.is_empty() {
            extras.push_str(",\n  \"");
            extras.push_str(key);
            extras.push_str("\": \"");
            extras.push_str(value);
            extras.push('"');
        }
    }
    if let Some(end) = json.rfind('}').filter(|_| !extras.is_empty()) {
        json.truncate(end);
        let kept = json.trim_end().len();
        json.truncate(kept);
        json.push_str(&extras);
        json.push_str("\n}\n");
    }
    if let Err(error) = settings::save(&json) {
        report_once(&format!("Boldy Mods: settings not saved ({error})"));
    }
}

/// Sends the menu for `state` to the host. A frame that did this is a heavy one, which leaves the
/// terrain read for the next. A failure is ignored: the menu may just have closed.
pub(super) fn push_menu(seconds: f32, state: MenuState) {
    push_menu_with(seconds, &settings(), state, false);
}

/// [`push_menu`] for `settings`, telling the player once when `report` and the host refused.
pub(super) fn push_menu_with(seconds: f32, settings: &Settings, state: MenuState, report: bool) {
    HEAVY_FRAME.with(|heavy| heavy.set(true));
    if let Err(error) = host_panel::update_content(&panel::update(settings, seconds, state))
        && report
    {
        report_once(&format!("Boldy Mods: menu update failed ({error})"));
    }
}

/// The three buttons that wait for a key.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum KeyButton {
    Menu,
    Fullbright,
    Zoom,
}

/// Starts or ends waiting for a new key on `button` and redraws the menu for it.
pub(super) fn set_capturing(button: KeyButton, capturing: bool, seconds: f32) {
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
pub(super) fn capture_optional_key(button: KeyButton, controls: &input::Controls, seconds: f32) {
    if !controls.panel_open {
        set_capturing(button, false, seconds);
        return;
    }
    let mut settings = settings();
    let key = match button {
        KeyButton::Zoom => &mut settings.zoom.key,
        _ => &mut settings.fullbright_key,
    };
    let Some(chosen) = zoom::captured_key(&controls.keys_pressed, *key, settings.menu_key) else {
        return;
    };
    if chosen != *key {
        *key = chosen;
        set_settings(settings);
        save_settings(&settings);
        reserve_keys(&settings);
    }
    set_capturing(button, false, seconds);
}

/// Shows a short explanation next to the pointer once it has rested on a control for
/// `HOVER_SECONDS`, and hides it when the pointer moves off or the menu closes.
pub(super) fn hover_help(controls: &input::Controls, seconds: f32) {
    // In gameplay, which is most of the time, there is nothing to look up or to hide.
    if !controls.panel_open
        && HOVER.with(StdCell::get).is_none()
        && TIP_SHOWN.with(StdCell::get).is_none()
    {
        return;
    }
    let state = MENU.with(StdCell::get);
    let matching = panel::fill_matches(&settings(), state);
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
pub(super) const PREVIEW_SECONDS: f32 = 0.05;

/// The arrow keys, in the order `ARROW_REPEAT` counts them.
pub(super) const ARROWS: [&str; 4] = ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"];

/// A held arrow key repeats after this many seconds, then every `ARROW_REPEAT_EVERY`.
pub(super) const ARROW_REPEAT_DELAY: f32 = 0.4;

pub(super) const ARROW_REPEAT_EVERY: f32 = 0.06;

/// The arrow key to act on again because it is being held. The host only sends a key's first
/// press, so holding Left or Right on a slider would otherwise move it once.
pub(super) fn arrow_repeat(controls: &input::Controls, seconds: f32) -> Option<&'static str> {
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
pub(super) fn expire_confirm(seconds: f32) {
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
pub(super) fn apply_menu_events(
    controls: &input::Controls,
    text: &[input::TextEvent],
    seconds: f32,
) {
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
    let before = settings();
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
        set_settings(settings);
        if settings.zoom != before.zoom {
            // The zoom key is reserved only while the zoom is on.
            reserve_keys(&settings);
        }
        // Saved in a frame with the fuel for it: this one has the menu's redraw to pay for,
        // and a large drawing to rescale or write out costs a lot more.
        SETTINGS_UNSAVED.with(|unsaved| unsaved.set(true));
    }
    push_menu_with(seconds, &settings, state, true);
    // The update hid the highlight and help; forget them so the next frame shows them again
    // where they now belong (opening the colour tools moves every settings control).
    HOVER.with(|h| h.set(None));
    TIP_SHOWN.with(|t| t.set(None));
}
