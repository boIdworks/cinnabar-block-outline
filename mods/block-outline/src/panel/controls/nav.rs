//! Which control is shown, focus and keyboard navigation.

use super::*;

/// Whether `view` is shown for `state`.
pub(in crate::panel) fn shown(view: View, state: MenuState, fill_matches: bool) -> bool {
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
        View::Cps => state.page == Page::Cps,
        View::Fps => state.page == Page::Fps,
        View::Zoom => state.page == Page::Zoom,
        View::Lighting => state.page == Page::Lighting,
        View::JumpReset => state.page == Page::JumpReset,
        View::Crosshair => state.page == Page::Crosshair && !colour_open && !state.drawing,
        View::Draw => state.page == Page::Crosshair && state.drawing,
        View::CrosshairTop => state.page == Page::Crosshair,
        View::CrosshairDrawButton => state.page == Page::Crosshair && !state.drawing,
        View::Colour => state.page.has_colour() && colour_open,
        View::FillOwn => state.page == Page::Block && !fill_matches,
        View::FillMatch => state.page == Page::Block && fill_matches,
    }
}

/// Control `index`'s area on the card and help text, or `None` while it is hidden.
pub(in crate::panel) fn tip_area(
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
pub(in crate::panel) fn is_stop(index: usize, state: MenuState, fill_matches: bool) -> bool {
    !matches!(controls()[index].action, Action::Help | Action::Hint)
        && tip_area(index, state, fill_matches).is_some()
}

/// The focused control's area and help, or `None` when nothing is focused or it is hidden.
pub(in crate::panel) fn focus_area(
    state: MenuState,
    fill_matches: bool,
) -> Option<([f32; 4], &'static str)> {
    let index = usize::from(state.focus?);
    if !is_stop(index, state, fill_matches) {
        return None;
    }
    tip_area(index, state, fill_matches)
}

/// Moves focus to the page's first stop when the focused control is no longer shown (the page or
/// view changed under it).
pub(in crate::panel) fn settle_focus(settings: &Settings, state: &mut MenuState) {
    let Some(focus) = state.focus else { return };
    let matching = fill_matches(settings, *state);
    if !is_stop(usize::from(focus), *state, matching) {
        state.focus = (0..controls().len())
            .find(|&index| is_stop(index, *state, matching))
            .and_then(|index| u8::try_from(index).ok());
    }
}

/// Whether switch `id` is on, as the shown page has it.
pub(in crate::panel) fn toggle_on(settings: &Settings, state: MenuState, id: &str) -> bool {
    let shown = view(settings, state);
    match id {
        "rainbow" => shown.rainbow,
        "pulse" => shown.pulse,
        "fill_match" => shown.fill_rgb.is_none(),
        "show_all_mobs" => shown.show_all_mobs,
        "fullbright" | "home_light" => shown.fullbright,
        "home_block" => shown.block_on,
        "home_mob" => shown.mob_on,
        "home_counter" => shown.counter.on,
        "home_cps" => shown.cps.on,
        "home_fps" => shown.fps.on,
        "home_zoom" => shown.zoom.on,
        "jump_on" | "home_jump" => shown.jump_reset.on,
        "crosshair_on" | "home_crosshair" => shown.crosshair.on,
        "crosshair_outline" => shown.crosshair.outline,
        "crosshair_mirror" => shown.crosshair.mirror,
        "counter" => shown.counter.on,
        "counter_bow" => shown.counter.bow_only,
        "cps_on" => shown.cps.on,
        "cps_right" => shown.cps.right,
        "cps_zeros" => shown.cps.zeros,
        "fps_on" => shown.fps.on,
        "zoom" => shown.zoom.on,
        "zoom_mode" => shown.zoom.toggle,
        "zoom_scroll" => shown.zoom.scroll,
        "zoom_sens" => shown.zoom.sensitivity,
        _ => false,
    }
}

/// How far an arrow key moves slider `index`; Shift takes five times as big a step.
pub(in crate::panel) fn slider_step(index: usize, shift: bool) -> f32 {
    let step = if index == 4 { 0.5 } else { 5.0 };
    if shift { step * 5.0 } else { step }
}

#[derive(Clone, Copy, PartialEq)]
pub(in crate::panel) enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// The stop nearest `from` in direction `dir`, as the card is laid out: the one straight above
/// or below (or the closest in that direction), or beside it on the same row. `None` when there
/// is nothing that way.
pub(in crate::panel) fn neighbour(
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
                let dir = if key == "ArrowRight" {
                    Dir::Right
                } else {
                    Dir::Left
                };
                let focus = state
                    .focus
                    .map(usize::from)
                    .filter(|&f| is_stop(f, *state, matching));
                let changes_value = focus.is_some_and(|f| {
                    matches!(
                        controls()[f].action,
                        Action::Slider(_) | Action::Stepper { .. }
                    )
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
                    if first {
                        0
                    } else {
                        stops.len() - 1
                    }
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
            ("ArrowLeft" | "ArrowRight", Action::Stepper { less, more, .. }) => Some((
                (if key == "ArrowRight" { *more } else { *less }).into(),
                1.0,
            )),
            ("ArrowLeft" | "ArrowRight", Action::Slider(index)) => {
                let shown = view(settings, *state);
                let value = slider_value(&shown, *state, *index);
                let step = slider_step(*index, shift);
                let value = if key == "ArrowRight" {
                    value + step
                } else {
                    value - step
                };
                Some((SLIDERS[*index].0.into(), value))
            }
            _ => None,
        };
        out.events.extend(event);
    }
    out
}
