//! Hover help: which help entry is under the pointer and the patches that show it.

use super::*;

/// Width of the hover help box, in panel units.
pub(super) const TIP_WIDTH: f32 = 236.0;

/// Card width; the same whichever view is shown.
pub fn card_width(_colour_open: bool) -> f32 {
    SETTINGS_W
}

/// Which view a help entry belongs to.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum View {
    /// The title, preview, Line / Fill colour buttons and the bottom line: always shown.
    Always,
    Settings,
    Colour,
    /// Shown while the fill has its own colour: the half-width Line and Fill colour buttons.
    FillOwn,
    /// Shown while the fill matches the line: the wide Line colour button.
    FillMatch,
    /// The start page.
    Home,
    /// Either page, whichever view.
    Page,
    /// The Block outline page's settings view.
    BlockOnly,
    /// The Mob hitbox page's settings view.
    MobOnly,
    /// The Mob hitbox page's Line / Aimed colour buttons, in either view.
    MobTop,
}

/// Hover help: its view, its area on the card as `[x, y, w, h]`, and its text.
pub(super) fn tips() -> [(View, [f32; 4], &'static str); 39] {
    let view = |colour: bool| if colour { View::Colour } else { View::Settings };
    let row = |index: usize| {
        let (colour, y) = slider_place(index);
        (
            view(colour),
            [10.0, y, TRACK_X + track_width() - 10.0, ROW_HEIGHT],
        )
    };
    let number = |index: usize| {
        let (colour, y) = slider_place(index);
        (view(colour), [VALUE_X, y, VALUE_WIDTH, ROW_HEIGHT])
    };
    let right = 10.0 + HALF + 4.0;
    const TYPE: &str = "Click, type a number, press Enter";
    let [wx, wy, side] = WHEEL;
    let tip = |(view, area): (View, [f32; 4]), text| (view, area, text);
    [
        tip(number(0), TYPE),
        tip(number(1), TYPE),
        tip(number(2), TYPE),
        tip(number(3), TYPE),
        tip(number(4), TYPE),
        (View::BlockOnly, number(5).1, TYPE),
        (
            View::FillOwn,
            [10.0, EDIT_Y, HALF, 15.0],
            "Shows or hides the line colour tools",
        ),
        (
            View::FillOwn,
            [right, EDIT_Y, HALF, 15.0],
            "Shows or hides the fill colour tools",
        ),
        tip(row(0), "Red part of the colour being edited"),
        tip(row(1), "Green part of the colour being edited"),
        tip(row(2), "Blue part of the colour being edited"),
        tip(row(3), "How solid the lines are (0 = invisible)"),
        tip(row(4), "Line thickness; thins out far away"),
        (View::BlockOnly, row(5).1, "Tints the faces (0 = off)"),
        (
            View::Settings,
            [10.0, TOGGLES_Y, HALF, 15.0],
            "Lines cycle through every colour",
        ),
        (
            View::BlockOnly,
            [right, TOGGLES_Y, HALF, 15.0],
            "The outline gently fades in and out",
        ),
        (
            View::Colour,
            [10.0, SWATCH_Y, SETTINGS_W - 20.0, 30.0],
            "Click a colour to use it straight away",
        ),
        (
            View::Colour,
            [10.0, FAVOURITE_Y, 4.0 * 34.0, 30.0],
            "Click a saved colour to use it",
        ),
        (
            View::Colour,
            [
                10.0 + 4.0 * 34.0,
                FAVOURITE_Y,
                SETTINGS_W - 20.0 - 4.0 * 34.0,
                30.0,
            ],
            "Saves the colour being edited",
        ),
        (
            View::BlockOnly,
            [10.0, BOTTOM_Y, HALF, 15.0],
            "On: the fill uses the line colour",
        ),
        (
            View::Settings,
            [right, BOTTOM_Y, HALF, 15.0],
            "This page's defaults; keeps saved",
        ),
        (
            View::Colour,
            [wx, wy, side, side],
            "Angle picks the hue; centre is paler",
        ),
        (
            View::Colour,
            [HEX_X, HEX_Y, SETTINGS_W - 10.0 - HEX_X, 52.0],
            "Click, type e.g. FF8800, press Enter",
        ),
        (
            View::Home,
            [10.0, HOME_KEY_Y, SETTINGS_W - 20.0, 15.0],
            "Click, then press your new menu key",
        ),
        (
            View::Always,
            [10.0, CARD_H - 18.0, SETTINGS_W - 20.0, 14.0],
            HINT_TIP,
        ),
        (
            View::Always,
            [10.0, 4.0, 150.0, 18.0],
            "Outlines the block you are looking at",
        ),
        (
            View::BlockOnly,
            [10.0, HIGHLIGHT_Y, HIGHLIGHT_W, 15.0],
            "Cancels the game's tint; click: off/on",
        ),
        (
            View::FillMatch,
            [10.0, EDIT_Y, SETTINGS_W - 20.0, 15.0],
            "Colour tools; the fill uses it too",
        ),
        (View::Page, BACK_AREA, "Goes back one step"),
        (View::Always, CLOSE_AREA, "Closes the menu (Esc does too)"),
        (
            View::Home,
            [10.0, HOME_BLOCK_Y, SETTINGS_W - 20.0, HOME_BUTTON_H],
            "Colours and size of the block outline",
        ),
        (
            View::Home,
            [10.0, HOME_MOB_Y, SETTINGS_W - 20.0, HOME_BUTTON_H],
            "Colours and size of mob hitboxes",
        ),
        (
            View::MobTop,
            [right, EDIT_Y, HALF, 15.0],
            "Colour of the mob you aim at",
        ),
        (
            View::MobOnly,
            [10.0, BOTTOM_Y, HALF, 15.0],
            "Off: only the mob you aim at",
        ),
        (
            View::MobOnly,
            [right, TOGGLES_Y, HALF, 15.0],
            "Lines cycle through every colour",
        ),
        (
            View::Home,
            [10.0, HOME_FULLBRIGHT_Y, SETTINGS_W - 20.0, 15.0],
            "Lights up caves and nights (only you)",
        ),
        (
            View::Home,
            [10.0, HOME_FULLBRIGHT_KEY_Y, SETTINGS_W - 20.0, 15.0],
            "Toggles fullbright; Backspace clears",
        ),
        (
            View::BlockOnly,
            [
                10.0 + HIGHLIGHT_W,
                HIGHLIGHT_Y,
                SETTINGS_W - 20.0 - HIGHLIGHT_W,
                15.0,
            ],
            "5% weaker or stronger",
        ),
        (
            View::MobTop,
            [10.0, EDIT_Y, HALF, 15.0],
            "Hitbox colour; the fill uses it too",
        ),
    ]
}

/// Help entry `index`'s area on the card, or `None` while its view is hidden.
/// The hint line's help; the line moves with the card's height.
const HINT_TIP: &str = "Menu key or F10 opens; Esc closes";

pub(super) fn tip_area(
    index: usize,
    state: MenuState,
    fill_matches: bool,
) -> Option<([f32; 4], &'static str)> {
    let (view, area, text) = *tips().get(index)?;
    let page = state.page != Page::Home;
    let colour_open = state.colour_open;
    let shown = match view {
        View::Always => true,
        View::Home => !page,
        View::Page => page,
        View::Settings => page && !colour_open,
        View::BlockOnly => state.page == Page::Block && !colour_open,
        View::MobOnly => state.page == Page::Mob && !colour_open,
        View::MobTop => state.page == Page::Mob,
        View::Colour => page && colour_open,
        View::FillOwn => state.page == Page::Block && !fill_matches,
        View::FillMatch => state.page == Page::Block && fill_matches,
    };
    let mut area = area;
    if text == HINT_TIP {
        area[1] = card_height(state) - 18.0;
    }
    // The mob page's options sit one row higher, where its Fill row would be.
    if state.page == Page::Mob
        && matches!(view, View::Settings | View::MobOnly)
        && area[1] >= OPTIONS_CAPTION_Y
    {
        area[1] -= MOB_OPTIONS_SHIFT;
    }
    shown.then_some((area, text))
}

/// Converts a pointer in HUD units to card coordinates, using `panel.read-layout`: the panel
/// root's size and offset in HUD units, its scale, and the card at the root's right edge,
/// `SCREEN_MARGIN` in, vertically centred.
pub fn card_point(
    pointer: [f32; 2],
    root: [f32; 2],
    offset: [f32; 2],
    scale: f32,
    colour_open: bool,
) -> Option<[f32; 2]> {
    if !(scale > 0.0) {
        return None;
    }
    let origin = [
        root[0] - card_width(colour_open) - SCREEN_MARGIN,
        (root[1] - CARD_H) / 2.0,
    ];
    Some(std::array::from_fn(|axis| {
        (pointer[axis] - offset[axis]) / scale - origin[axis]
    }))
}

/// Which help entry is under `point` on the card, if any.
pub fn tip_at(point: [f32; 2], state: MenuState, fill_matches: bool) -> Option<usize> {
    (0..tips().len()).find(|&index| {
        tip_area(index, state, fill_matches).is_some_and(|([x, y, w, h], _)| {
            point[0] >= x && point[0] < x + w && point[1] >= y && point[1] < y + h
        })
    })
}

/// A patch showing help entry `tip` beside `point`, or hiding the help when `None`.
pub fn tip_update(
    tip: Option<usize>,
    point: [f32; 2],
    state: MenuState,
    fill_matches: bool,
) -> String {
    let colour_open = state.colour_open;
    let Some((_, text)) = tip.and_then(|index| tip_area(index, state, fill_matches)) else {
        return r##"{"binding_changes":{"#tip_visible":false}}"##.into();
    };
    // Below and right of the pointer, kept inside the card.
    let x = (point[0] + 8.0)
        .min(card_width(colour_open) - TIP_WIDTH - 4.0)
        .max(4.0);
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

/// A patch moving the hover highlight onto help entry `tip`'s area, or hiding it.
pub fn highlight_update(tip: Option<usize>, state: MenuState, fill_matches: bool) -> String {
    let Some(([x, y, w, h], _)) = tip.and_then(|index| tip_area(index, state, fill_matches)) else {
        return r##"{"binding_changes":{"#hover_visible":false}}"##.into();
    };
    let mut out = String::with_capacity(140);
    out.push_str(r##"{"binding_changes":{"#hover_visible":true,"#hover_offset":["##);
    fixed(&mut out, x - 2.0);
    out.push(',');
    fixed(&mut out, y - 2.0);
    out.push_str(r##"],"#hover_width":"##);
    fixed(&mut out, w + 4.0);
    out.push_str(r##","#hover_height":"##);
    fixed(&mut out, h + 4.0);
    out.push_str("}}");
    out
}
