//! Hover help and the focus ring: the patches that show them. The controls themselves, with
//! their areas and help texts, are in `controls.rs`.

use super::*;

/// Width of the hover help box, in panel units.
pub(super) const TIP_WIDTH: f32 = 236.0;

/// The bottom line while nothing is hovered or focused.
pub(super) const HINT_DEFAULT: &str = "Hover or press Tab for help. Esc closes.";

/// Converts a pointer in HUD units to card coordinates, using `panel.read-layout`: the panel
/// root's size and offset in HUD units, its scale, and the card at the root's right edge,
/// `SCREEN_MARGIN` in, vertically centred.
pub fn card_point(
    pointer: [f32; 2],
    root: [f32; 2],
    offset: [f32; 2],
    scale: f32,
) -> Option<[f32; 2]> {
    if scale.is_nan() || scale <= 0.0 {
        return None;
    }
    let origin = [
        root[0] - SETTINGS_W - SCREEN_MARGIN,
        (root[1] - CARD_H) / 2.0,
    ];
    Some(std::array::from_fn(|axis| {
        (pointer[axis] - offset[axis]) / scale - origin[axis]
    }))
}

/// A patch showing help entry `tip` beside `point`, or hiding the help when `None`.
pub fn tip_update(
    tip: Option<usize>,
    point: [f32; 2],
    state: MenuState,
    fill_matches: bool,
) -> String {
    let Some((_, text)) = tip.and_then(|index| tip_area(index, state, fill_matches)) else {
        return r##"{"binding_changes":{"#tip_visible":false}}"##.into();
    };
    // Below and right of the pointer, kept inside the card.
    let x = (point[0] + 8.0).clamp(4.0, (SETTINGS_W - TIP_WIDTH - 4.0).max(4.0));
    let y = if point[1] + 30.0 > CARD_H {
        point[1] - 20.0
    } else {
        point[1] + 14.0
    };
    let mut out = String::with_capacity(160);
    out.push_str(r##"{"binding_changes":{"#tip_visible":true,"#tip_text":""##);
    out.push_str(text);
    out.push_str(r##"","#tip_offset":["##);
    fixed(&mut out, x);
    out.push(',');
    fixed(&mut out, y);
    out.push_str("]}}");
    out
}

/// What the bottom line says: the help of what the pointer is on, else of the focused control,
/// else how to get help.
pub(super) fn hint_text(
    hovered: Option<usize>,
    state: MenuState,
    fill_matches: bool,
) -> &'static str {
    hovered
        .and_then(|index| tip_area(index, state, fill_matches))
        .or_else(|| focus_area(state, fill_matches))
        .map_or(HINT_DEFAULT, |(_, text)| text)
}

/// Appends `"#<prefix>_visible"`, `_offset`, `_width` and `_height` for a ring or highlight
/// drawn `grow` units outside `area`, or just the hidden flag when there is none.
fn ring(out: &mut String, prefix: &str, area: Option<[f32; 4]>, grow: f32) {
    out.push_str("\"#");
    out.push_str(prefix);
    out.push_str("_visible\":");
    let Some([x, y, w, h]) = area else {
        out.push_str("false");
        return;
    };
    out.push_str("true,\"#");
    out.push_str(prefix);
    out.push_str("_offset\":[");
    fixed(out, x - grow);
    out.push(',');
    fixed(out, y - grow);
    out.push_str("],\"#");
    out.push_str(prefix);
    out.push_str("_width\":");
    fixed(out, w + 2.0 * grow);
    out.push_str(",\"#");
    out.push_str(prefix);
    out.push_str("_height\":");
    fixed(out, h + 2.0 * grow);
}

/// Appends the focus ring and the bottom line's text, as the menu shows them when nothing is
/// hovered. Used by every full redraw and by [`focus_update`].
pub(super) fn focus_bindings(out: &mut String, state: MenuState, fill_matches: bool) {
    let area = focus_area(state, fill_matches).map(|(area, _)| area);
    // The tint, then the four sides of the outline: `FOCUS_GAP` outside the control, each
    // `FOCUS_LINE` thick.
    ring(out, "focus", area, FOCUS_GAP);
    if let Some([x, y, w, h]) = area {
        let (x0, y0) = (x - FOCUS_GAP, y - FOCUS_GAP);
        let (wide, tall) = (w + 2.0 * FOCUS_GAP, h + 2.0 * FOCUS_GAP);
        let mut place = |name: &str, at: [f32; 2]| {
            out.push_str(",\"#focus_");
            out.push_str(name);
            out.push_str("\":[");
            fixed(out, at[0]);
            out.push(',');
            fixed(out, at[1]);
            out.push(']');
        };
        place("top", [x0, y0]);
        place("bottom", [x0, y0 + tall - FOCUS_LINE]);
        place("left", [x0, y0]);
        place("right", [x0 + wide - FOCUS_LINE, y0]);
        out.push_str(",\"#focus_wide\":");
        fixed(out, wide);
        out.push_str(",\"#focus_tall\":");
        fixed(out, tall);
    }
    out.push_str(",\"#hint_text\":\"");
    out.push_str(hint_text(None, state, fill_matches));
    out.push('"');
}

/// A patch moving the focus ring onto the focused control (or hiding it) and telling its help
/// on the bottom line.
pub fn focus_update(state: MenuState, fill_matches: bool) -> String {
    let mut out = String::with_capacity(220);
    out.push_str(r##"{"binding_changes":{"##);
    focus_bindings(&mut out, state, fill_matches);
    out.push_str("}}");
    out
}

/// A patch moving the hover highlight onto control `tip`'s area, or hiding it, and putting its
/// help on the bottom line at once (the floating box waits for the pointer to rest).
pub fn highlight_update(tip: Option<usize>, state: MenuState, fill_matches: bool) -> String {
    let area = tip.and_then(|index| tip_area(index, state, fill_matches)).map(|(area, _)| area);
    let mut out = String::with_capacity(200);
    out.push_str(r##"{"binding_changes":{"##);
    ring(&mut out, "hover", area, 2.0);
    out.push_str(",\"#hint_text\":\"");
    out.push_str(hint_text(tip, state, fill_matches));
    out.push_str("\"}}");
    out
}
