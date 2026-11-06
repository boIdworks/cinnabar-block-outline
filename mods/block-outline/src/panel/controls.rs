//! The menu's controls as data: where each one sits, what its help says and what the keyboard
//! does to it. Hover help, the hover highlight, the focus ring and Tab / arrow navigation all
//! read this one table, and a test ties it to the buttons of the real document, so a control
//! cannot move without its help and its keyboard stop following.

use std::sync::OnceLock;

use super::*;

/// Which view a control belongs to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum View {
    /// The title, close button and bottom line: always shown.
    Always,
    Settings,
    Colour,
    /// Shown while the fill has its own colour: the half-width Line and Fill colour buttons.
    FillOwn,
    /// Shown while the fill matches the line: the wide Line colour button.
    FillMatch,
    /// The start page.
    Home,
    /// Any page but the start page.
    Page,
    /// The Block outline page's settings view.
    BlockOnly,
    /// The Mob hitbox page's settings view.
    MobOnly,
    /// The Mob hitbox page's Line / Aimed colour buttons, in either view.
    MobTop,
    /// The Arrow counter page.
    Counter,
    /// The Zoom page.
    Zoom,
}

/// What a control is, which is also what the keyboard does to it.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Action {
    /// Help only: a number box, a group or a - / + pair. The keyboard skips it.
    Help,
    /// The bottom line, which moves with the card's height. The keyboard skips it.
    Hint,
    /// A button. Enter or Space presses it.
    Press(String),
    /// A switch. Enter or Space flips it; Left turns it off, Right on.
    Toggle(&'static str),
    /// A slider row. Left and Right step it, Enter types its number.
    Slider(usize),
    /// A value with - and + beside it. Left and Right press those, Enter the value (its reset).
    Stepper {
        value: &'static str,
        less: &'static str,
        more: &'static str,
    },
    /// The hex code box. Enter types into it.
    Hex,
    /// A key button. Enter waits for the next key.
    Key(&'static str),
}

/// One control: where it is on the card (`[x, y, w, h]`), when it is shown and what to do.
pub(super) struct Control {
    pub view: View,
    pub area: [f32; 4],
    pub tip: &'static str,
    pub action: Action,
}

/// The bottom line's help; the line moves with the card's height.
pub(super) const HINT_TIP: &str = "Menu key opens; Esc closes";

/// Every control, in the order the keyboard visits them: top buttons, the page, then Back and
/// Close. Built once.
pub(super) fn controls() -> &'static [Control] {
    static TABLE: OnceLock<Vec<Control>> = OnceLock::new();
    TABLE.get_or_init(build)
}

fn build() -> Vec<Control> {
    let mut table: Vec<Control> = Vec::with_capacity(112);
    let mut add = |view: View, area: [f32; 4], tip: &'static str, action: Action| {
        table.push(Control { view, area, tip, action });
    };
    let press = |id: &str| Action::Press(id.to_owned());
    let full = SETTINGS_W - 20.0;
    let right = 10.0 + HALF + 4.0;
    let row = |y: f32| [10.0, y, full, BUTTON_H];
    let half = |y: f32| [10.0, y, HALF, BUTTON_H];
    let half_right = |y: f32| [right, y, HALF, BUTTON_H];
    // A value with - and + beside it: the row (for the keyboard), the value and the pair.
    let value_area = |y: f32| [10.0, y, HIGHLIGHT_W, BUTTON_H];
    let pair_area = |y: f32| [10.0 + HIGHLIGHT_W, y, SETTINGS_W - 20.0 - HIGHLIGHT_W, BUTTON_H];
    const STEPPER_TIP: &str = "Left/Right change it; Enter resets it";
    const TYPE: &str = "Click, type a number, press Enter";

    // The start page.
    add(View::Home, [10.0, HOME_BLOCK_Y, full, HOME_BUTTON_H], "Colours and size of the block outline", press("open_block"));
    add(View::Home, [10.0, HOME_MOB_Y, full, HOME_BUTTON_H], "Colours and size of mob hitboxes", press("open_mob"));
    add(View::Home, [10.0, HOME_COUNTER_Y, full, HOME_BUTTON_H], "Shows how many arrows you carry", press("open_counter"));
    add(View::Home, [10.0, HOME_ZOOM_Y, full, HOME_BUTTON_H], "Hold a key to zoom in smoothly", press("open_zoom"));
    add(View::Home, row(HOME_FULLBRIGHT_Y), "Lights up caves and nights (only you)", Action::Toggle("fullbright"));
    add(View::Home, row(HOME_FULLBRIGHT_KEY_Y), "Click, press a key; Del clears it", Action::Key("fullbright_key"));
    add(View::Home, row(HOME_KEY_Y), "Click, press a key; Esc cancels", Action::Key("menu_key"));

    // The Block outline and Mob hitbox pages: the colour buttons at the top.
    add(View::FillOwn, half(EDIT_Y), "Shows or hides the line colour tools", press("edit_line"));
    add(View::FillOwn, half_right(EDIT_Y), "Shows or hides the fill colour tools", press("edit_fill"));
    add(View::FillMatch, row(EDIT_Y), "Colour tools; the fill uses it too", press("edit_line"));
    add(View::MobTop, half(EDIT_Y), "Hitbox colour; the fill uses it too", press("edit_line"));
    add(View::MobTop, half_right(EDIT_Y), "Colour of the mob you aim at", press("edit_aim"));

    // Their settings view: opacity, thickness and fill rows, then the options.
    let slider_row = |index: usize| {
        let (_, y) = slider_place(index);
        [10.0, y, TRACK_X + track_width() - 10.0, ROW_HEIGHT]
    };
    let number_box = |index: usize| {
        let (_, y) = slider_place(index);
        [VALUE_X, y, VALUE_WIDTH, ROW_HEIGHT]
    };
    add(View::Settings, number_box(3), TYPE, Action::Help);
    add(View::Settings, slider_row(3), "How solid the lines are (0 = invisible)", Action::Slider(3));
    add(View::Settings, number_box(4), TYPE, Action::Help);
    add(View::Settings, slider_row(4), "Line thickness; thins out far away", Action::Slider(4));
    add(View::BlockOnly, number_box(5), TYPE, Action::Help);
    add(View::BlockOnly, slider_row(5), "Tints the faces (0 = off)", Action::Slider(5));
    add(View::BlockOnly, half(TOGGLES_Y), "Lines cycle through every colour", Action::Toggle("rainbow"));
    add(View::BlockOnly, half_right(TOGGLES_Y), "The outline gently fades in and out", Action::Toggle("pulse"));
    // Mob hitboxes have no pulse: their Rainbow switch takes the whole row.
    add(View::MobOnly, row(TOGGLES_Y), "Lines cycle through every colour", Action::Toggle("rainbow"));
    add(View::BlockOnly, half(BOTTOM_Y), "On: the fill uses the line colour", Action::Toggle("fill_match"));
    add(View::MobOnly, half(BOTTOM_Y), "Off: only the mob you aim at", Action::Toggle("show_all_mobs"));
    add(View::Settings, half_right(BOTTOM_Y), "Resets this page; click twice to confirm", press("reset"));
    add(View::BlockOnly, value_area(HIGHLIGHT_Y), "Hides the game's highlight; click: on/off", Action::Help);
    add(View::BlockOnly, pair_area(HIGHLIGHT_Y), "5%% weaker or stronger", Action::Help);
    add(
        View::BlockOnly,
        row(HIGHLIGHT_Y),
        "Left/Right change it; Enter: on/off",
        Action::Stepper { value: "hide_game_highlight", less: "highlight_less", more: "highlight_more" },
    );

    // Their colour view.
    for (index, tip) in [
        "Red part of the colour being edited",
        "Green part of the colour being edited",
        "Blue part of the colour being edited",
    ]
    .into_iter()
    .enumerate()
    {
        add(View::Colour, number_box(index), TYPE, Action::Help);
        add(View::Colour, slider_row(index), tip, Action::Slider(index));
    }
    let [wx, wy, side] = WHEEL;
    add(View::Colour, [wx, wy, side, side], "Angle picks the hue; centre is paler", Action::Help);
    let hex_w = SETTINGS_W - 10.0 - HEX_X;
    add(View::Colour, [HEX_X, HEX_Y, hex_w, 58.0], "Click, type e.g. FF8800, press Enter", Action::Help);
    add(View::Colour, [HEX_X, HEX_Y + 12.0, hex_w, BUTTON_H], "Click, type e.g. FF8800, press Enter", Action::Hex);
    let swatch_width = (SETTINGS_W - 20.0) / SWATCHES.len() as f32;
    for (index, (name, _)) in SWATCHES.iter().enumerate() {
        add(
            View::Colour,
            [10.0 + swatch_width * index as f32, SWATCH_Y + 12.0, swatch_width - 3.0, BUTTON_H],
            "Click a colour to use it straight away",
            press(&format!("swatch_{}", name.to_lowercase())),
        );
    }
    for slot in 0..FAVOURITES {
        add(
            View::Colour,
            [10.0 + 34.0 * slot as f32, FAVOURITE_Y + 12.0, 30.0, BUTTON_H],
            "Click a saved colour to use it",
            press(&format!("favourite_{slot}")),
        );
    }
    let save_x = 10.0 + 34.0 * FAVOURITES as f32;
    add(
        View::Colour,
        [save_x, FAVOURITE_Y + 12.0, SETTINGS_W - 10.0 - save_x, BUTTON_H],
        "Saves the colour being edited",
        press("save_favourite"),
    );

    // The Arrow counter page.
    add(View::Counter, row(COUNTER_ON_Y), "Shows or hides the arrow counter", Action::Toggle("counter"));
    add(View::Counter, row(COUNTER_BOW_Y), "Only while a bow or crossbow is held", Action::Toggle("counter_bow"));
    add(View::Counter, row(COUNTER_MOVE_Y), "Drag it anywhere; scroll to resize", press("counter_move"));
    add(View::Counter, value_area(COUNTER_SIZE_Y), "Click the size to reset it to 100%%", Action::Help);
    add(View::Counter, pair_area(COUNTER_SIZE_Y), "10%% smaller or bigger", Action::Help);
    add(
        View::Counter,
        row(COUNTER_SIZE_Y),
        STEPPER_TIP,
        Action::Stepper { value: "counter_size", less: "counter_smaller", more: "counter_bigger" },
    );
    add(View::Counter, row(COUNTER_RESET_Y), "Beside the hotbar at 100%%; click twice", press("counter_reset"));

    // The Zoom page.
    add(View::Zoom, row(ZOOM_ON_Y), "Turns the zoom key on or off", Action::Toggle("zoom"));
    add(View::Zoom, row(ZOOM_KEY_Y), "Click, press a key; Del clears it", Action::Key("zoom_key"));
    add(View::Zoom, row(ZOOM_MODE_Y), "Hold the key, or press to toggle", Action::Toggle("zoom_mode"));
    add(View::Zoom, row(ZOOM_SCROLL_Y), "Scroll to zoom; moves hotbar slot too", Action::Toggle("zoom_scroll"));
    add(View::Zoom, row(ZOOM_SENS_Y), "Look more slowly while zoomed in", Action::Toggle("zoom_sens"));
    for (y, value_tip, pair_tip, (value, less, more)) in [
        (
            ZOOM_LEVEL_Y,
            "Zoom factor; click to reset it",
            "0.5x less or more",
            ("zoom_level", "zoom_level_less", "zoom_level_more"),
        ),
        (
            ZOOM_TIME_Y,
            "Seconds to zoom in; click to reset it",
            "0.05 seconds shorter or longer",
            ("zoom_time", "zoom_time_less", "zoom_time_more"),
        ),
        (
            ZOOM_STEP_Y,
            "Zoom change per scroll; click to reset it",
            "5%% smaller or bigger",
            ("zoom_step", "zoom_step_less", "zoom_step_more"),
        ),
    ] {
        add(View::Zoom, value_area(y), value_tip, Action::Help);
        add(View::Zoom, pair_area(y), pair_tip, Action::Help);
        add(View::Zoom, row(y), STEPPER_TIP, Action::Stepper { value, less, more });
    }
    add(View::Zoom, row(ZOOM_RESET_Y), "Zoom back to defaults; click twice", press("zoom_reset"));

    // Back and Close come last for the keyboard; the title and bottom line are help only.
    add(View::Page, BACK_AREA, "Goes back one step", press("back"));
    add(View::Always, CLOSE_AREA, "Closes the menu (Esc does too)", press("close"));
    add(View::Always, [10.0, 4.0, 150.0, 18.0], "Outlines, counter, zoom and fullbright", Action::Help);
    add(View::Always, [10.0, CARD_H - 18.0, full, 14.0], HINT_TIP, Action::Hint);
    table
}

/// Whether `view` is shown for `state`.
fn shown(view: View, state: MenuState, fill_matches: bool) -> bool {
    let page = state.page != Page::Home;
    let outline = state.page.outline();
    let colour_open = state.colour_open;
    match view {
        View::Always => true,
        View::Home => !page,
        View::Page => page,
        View::Settings => outline && !colour_open,
        View::BlockOnly => state.page == Page::Block && !colour_open,
        View::MobOnly => state.page == Page::Mob && !colour_open,
        View::MobTop => state.page == Page::Mob,
        View::Counter => state.page == Page::Counter,
        View::Zoom => state.page == Page::Zoom,
        View::Colour => outline && colour_open,
        View::FillOwn => state.page == Page::Block && !fill_matches,
        View::FillMatch => state.page == Page::Block && fill_matches,
    }
}

/// Control `index`'s area on the card and help text, or `None` while it is hidden.
pub(super) fn tip_area(
    index: usize,
    state: MenuState,
    fill_matches: bool,
) -> Option<([f32; 4], &'static str)> {
    let control = controls().get(index)?;
    // The card is hidden while the counter is moved.
    if state.moving || !shown(control.view, state, fill_matches) {
        return None;
    }
    let mut area = control.area;
    if control.action == Action::Hint {
        area[1] = card_height(state) - 18.0;
    }
    // The mob page's options sit one row higher, where its Fill row would be.
    if state.page == Page::Mob
        && matches!(control.view, View::Settings | View::MobOnly)
        && area[1] >= OPTIONS_CAPTION_Y
    {
        area[1] -= MOB_OPTIONS_SHIFT;
    }
    Some((area, control.tip))
}

/// Which control is under `point` on the card, if any: the smallest one, so a number box wins
/// over its row and a - / + pair over the row it sits in.
pub fn tip_at(point: [f32; 2], state: MenuState, fill_matches: bool) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for index in 0..controls().len() {
        let Some(([x, y, w, h], _)) = tip_area(index, state, fill_matches) else {
            continue;
        };
        if point[0] >= x && point[0] < x + w && point[1] >= y && point[1] < y + h {
            let size = w * h;
            if best.is_none_or(|(_, smallest)| size < smallest) {
                best = Some((index, size));
            }
        }
    }
    best.map(|(index, _)| index)
}

/// Whether the keyboard can stop on control `index` now.
fn is_stop(index: usize, state: MenuState, fill_matches: bool) -> bool {
    !matches!(controls()[index].action, Action::Help | Action::Hint)
        && tip_area(index, state, fill_matches).is_some()
}

/// The focused control's area and help, or `None` when nothing is focused or it is hidden.
pub(super) fn focus_area(state: MenuState, fill_matches: bool) -> Option<([f32; 4], &'static str)> {
    let index = usize::from(state.focus?);
    if !is_stop(index, state, fill_matches) {
        return None;
    }
    tip_area(index, state, fill_matches)
}

/// Moves focus to the page's first stop when the focused control is no longer shown (the page or
/// view changed under it).
pub(super) fn settle_focus(settings: &Settings, state: &mut MenuState) {
    let Some(focus) = state.focus else { return };
    let matching = fill_matches(settings, *state);
    if !is_stop(usize::from(focus), *state, matching) {
        state.focus = (0..controls().len())
            .find(|&index| is_stop(index, *state, matching))
            .and_then(|index| u8::try_from(index).ok());
    }
}

/// Whether switch `id` is on, as the shown page has it.
fn toggle_on(settings: &Settings, state: MenuState, id: &str) -> bool {
    let shown = view(settings, state);
    match id {
        "rainbow" => shown.rainbow,
        "pulse" => shown.pulse,
        "fill_match" => shown.fill_rgb.is_none(),
        "show_all_mobs" => shown.show_all_mobs,
        "fullbright" => shown.fullbright,
        "counter" => shown.counter.on,
        "counter_bow" => shown.counter.bow_only,
        "zoom" => shown.zoom.on,
        "zoom_mode" => shown.zoom.toggle,
        "zoom_scroll" => shown.zoom.scroll,
        "zoom_sens" => shown.zoom.sensitivity,
        _ => false,
    }
}

/// How far an arrow key moves slider `index`; Shift takes five times as big a step.
fn slider_step(index: usize, shift: bool) -> f32 {
    let step = if index == 4 { 0.5 } else { 5.0 };
    if shift { step * 5.0 } else { step }
}

#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// The stop nearest `from` in direction `dir`, as the card is laid out: the one straight above
/// or below (or the closest in that direction), or beside it on the same row. `None` when there
/// is nothing that way.
fn neighbour(
    from: usize,
    dir: Dir,
    stops: &[usize],
    state: MenuState,
    fill_matches: bool,
) -> Option<usize> {
    let [fx, fy, fw, fh] = tip_area(from, state, fill_matches)?.0;
    let (fcx, fcy) = (fx + fw / 2.0, fy + fh / 2.0);
    // The distance between two intervals on one axis, 0 where they overlap.
    let gap = |a0: f32, a1: f32, b0: f32, b1: f32| (b0 - a1).max(a0 - b1).max(0.0);
    let mut best: Option<(usize, f32)> = None;
    for &index in stops.iter().filter(|&&index| index != from) {
        let Some(([x, y, w, h], _)) = tip_area(index, state, fill_matches) else {
            continue;
        };
        let (cx, cy) = (x + w / 2.0, y + h / 2.0);
        let (along, across) = match dir {
            Dir::Down => (cy - fcy, gap(fx, fx + fw, x, x + w)),
            Dir::Up => (fcy - cy, gap(fx, fx + fw, x, x + w)),
            Dir::Right => (cx - fcx, gap(fy, fy + fh, y, y + h)),
            Dir::Left => (fcx - cx, gap(fy, fy + fh, y, y + h)),
        };
        // Strictly in that direction; left and right stay on the same row.
        if along <= 2.0 || (matches!(dir, Dir::Left | Dir::Right) && across > 0.0) {
            continue;
        }
        // Prefer what is near, and what lines up with the focused control.
        let score = along + 3.0 * across;
        if best.is_none_or(|(_, lowest)| score < lowest) {
            best = Some((index, score));
        }
    }
    best.map(|(index, _)| index)
}

/// What the keyboard did in one frame.
#[derive(Debug, Default, PartialEq)]
pub struct Keyed {
    /// Control events to apply, as if the controls had been clicked.
    pub events: Vec<(String, f32)>,
    /// Focus moved: patch the focus ring ([`focus_update`](super::focus_update)).
    pub moved: bool,
    /// Something besides the focus ring changed: redraw the menu.
    pub redraw: bool,
}

/// Feeds the keys pressed this frame to the menu (`shift`: Shift is held).
///
/// - The arrow keys move the focus frame the way they point: Up and Down to the control above or
///   below, Left and Right to the one beside it. Past the last control they wrap round.
/// - On a slider or a value with - and + (and on a switch with nothing beside it) Left and Right
///   change the value instead; Shift makes the step five times as big.
/// - Tab and Shift+Tab walk every control in reading order; Home and End jump to the first and
///   last one.
/// - Enter or Space press the focused control, Backspace goes back a page.
///
/// Does nothing while a box is typed into, a key is awaited or the counter is moved.
pub fn keys(settings: &Settings, state: &mut MenuState, pressed: &[String], shift: bool) -> Keyed {
    let mut out = Keyed::default();
    if state.typing.is_some()
        || state.capturing
        || state.capturing_fullbright
        || state.capturing_zoom
        || state.moving
    {
        return out;
    }
    for key in pressed {
        let matching = fill_matches(settings, *state);
        // Where to move the frame: along the reading order, or the way an arrow points.
        enum Move {
            Order(i32),
            Point(Dir),
            Ends(bool),
        }
        let movement = match key.as_str() {
            "Tab" => Some(Move::Order(if shift { -1 } else { 1 })),
            "ArrowDown" => Some(Move::Point(Dir::Down)),
            "ArrowUp" => Some(Move::Point(Dir::Up)),
            "ArrowLeft" | "ArrowRight" => {
                // Beside a neighbour, the frame moves to it; on a slider, a - / + value or a
                // switch with nothing beside it, the arrow changes the value (below).
                let dir = if key == "ArrowRight" { Dir::Right } else { Dir::Left };
                let focus = state
                    .focus
                    .map(usize::from)
                    .filter(|&f| is_stop(f, *state, matching));
                let changes_value = focus.is_some_and(|f| {
                    matches!(controls()[f].action, Action::Slider(_) | Action::Stepper { .. })
                });
                if focus.is_none() {
                    Some(Move::Order(1))
                } else if changes_value {
                    None
                } else {
                    let stops: Vec<usize> = (0..controls().len())
                        .filter(|&index| is_stop(index, *state, matching))
                        .collect();
                    // With anything beside it on its row, a control's Left and Right always
                    // move the frame (even at the row's end); only a control alone on its row
                    // lets them change its value.
                    let mate = focus.is_some_and(|f| {
                        [Dir::Left, Dir::Right]
                            .iter()
                            .any(|&side| neighbour(f, side, &stops, *state, matching).is_some())
                    });
                    mate.then_some(Move::Point(dir))
                }
            }
            "Backspace" => {
                if state.page != Page::Home {
                    out.events.push(("back".into(), 1.0));
                }
                continue;
            }
            "Home" => Some(Move::Ends(true)),
            "End" => Some(Move::Ends(false)),
            _ => None,
        };
        if let Some(movement) = movement {
            let stops: Vec<usize> = (0..controls().len())
                .filter(|&index| is_stop(index, *state, matching))
                .collect();
            if stops.is_empty() {
                continue;
            }
            let at = state
                .focus
                .and_then(|focus| stops.iter().position(|&index| index == usize::from(focus)));
            let walk = |step: i32| match at {
                None if step > 0 => 0,
                None => stops.len() - 1,
                Some(at) => (at as i32 + step).rem_euclid(stops.len() as i32) as usize,
            };
            let to = match (movement, at) {
                (Move::Ends(first), _) => {
                    if first { 0 } else { stops.len() - 1 }
                }
                (Move::Order(step), _) => walk(step),
                // The first arrow press only brings the frame in.
                (Move::Point(_), None) => 0,
                (Move::Point(dir), Some(at)) => {
                    match neighbour(stops[at], dir, &stops, *state, matching) {
                        Some(index) => stops.iter().position(|&s| s == index).unwrap_or(at),
                        // Nothing that way: Up and Down wrap round the whole card.
                        None if dir == Dir::Down => walk(1),
                        None if dir == Dir::Up => walk(-1),
                        None => at,
                    }
                }
            };
            state.focus = u8::try_from(stops[to]).ok();
            // Moving on cancels a reset that was waiting for its second click.
            if state.confirm != Reset::None {
                state.confirm = Reset::None;
                out.redraw = true;
            }
            out.moved = true;
            continue;
        }
        let Some(focus) = state.focus.map(usize::from) else {
            continue;
        };
        if !is_stop(focus, *state, matching) {
            continue;
        }
        let action = &controls()[focus].action;
        let event = match (key.as_str(), action) {
            ("Enter" | "NumpadEnter" | "Space", Action::Press(id)) => Some((id.clone(), 1.0)),
            ("Enter" | "NumpadEnter" | "Space", Action::Toggle(id)) => {
                let on = toggle_on(settings, *state, id);
                Some(((*id).into(), if on { 0.0 } else { 1.0 }))
            }
            ("Enter" | "NumpadEnter" | "Space", Action::Slider(index)) => {
                Some((format!("type_{}", SLIDERS[*index].0), 1.0))
            }
            ("Enter" | "NumpadEnter" | "Space", Action::Stepper { value, .. }) => {
                Some(((*value).into(), 1.0))
            }
            ("Enter" | "NumpadEnter" | "Space", Action::Hex) => Some(("type_hex".into(), 1.0)),
            ("Enter" | "NumpadEnter" | "Space", Action::Key(id)) => Some(((*id).into(), 1.0)),
            ("ArrowLeft" | "ArrowRight", Action::Toggle(id)) => {
                Some(((*id).into(), if key == "ArrowRight" { 1.0 } else { 0.0 }))
            }
            ("ArrowLeft" | "ArrowRight", Action::Stepper { less, more, .. }) => {
                Some(((if key == "ArrowRight" { *more } else { *less }).into(), 1.0))
            }
            ("ArrowLeft" | "ArrowRight", Action::Slider(index)) => {
                let shown = view(settings, *state);
                let value = slider_value(&shown, *state, *index);
                let step = slider_step(*index, shift);
                let value = if key == "ArrowRight" { value + step } else { value - step };
                Some((SLIDERS[*index].0.into(), value))
            }
            _ => None,
        };
        out.events.extend(event);
    }
    out
}
