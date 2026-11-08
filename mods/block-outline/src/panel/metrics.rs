//! Positions, sizes and colours of the menu's pieces.

use super::*;

/// Width of the small - and + buttons.
pub(super) const STEP_W: f32 = 26.0;

/// The highlight fix's Off / On button, beside its - and + buttons.
pub(super) const HIGHLIGHT_W: f32 = SETTINGS_W - 20.0 - 2.0 * (STEP_W + 4.0);

// Layout. Settings positions are relative to the settings column, colour positions to the
// colour column.
pub(super) const EDIT_Y: f32 = 31.0;

/// The start page: a button for each page, then the menu key. Nine buttons and the Menu
/// section just fill the card's height.
pub(super) const HOME_BLOCK_Y: f32 = 34.0;

pub(super) const HOME_PITCH: f32 = 25.0;

pub(super) const HOME_MOB_Y: f32 = HOME_BLOCK_Y + HOME_PITCH;

pub(super) const HOME_COUNTER_Y: f32 = HOME_MOB_Y + HOME_PITCH;

pub(super) const HOME_CPS_Y: f32 = HOME_COUNTER_Y + HOME_PITCH;

pub(super) const HOME_FPS_Y: f32 = HOME_CPS_Y + HOME_PITCH;

pub(super) const HOME_ZOOM_Y: f32 = HOME_FPS_Y + HOME_PITCH;

pub(super) const HOME_LIGHT_Y: f32 = HOME_ZOOM_Y + HOME_PITCH;

pub(super) const HOME_JUMP_Y: f32 = HOME_LIGHT_Y + HOME_PITCH;

pub(super) const HOME_CROSS_Y: f32 = HOME_JUMP_Y + HOME_PITCH;

pub(super) const HOME_BUTTON_H: f32 = 23.0;

/// Below the page buttons: a Menu section.
pub(super) const HOME_MENU_CAPTION_Y: f32 = HOME_CROSS_Y + HOME_BUTTON_H + 9.0;

pub(super) const HOME_KEY_Y: f32 = HOME_MENU_CAPTION_Y + 14.0;

/// The mob page has no Fill row, so its options sit one row higher.
pub(super) const MOB_OPTIONS_SHIFT: f32 = ROW_STEP;

/// Room the start page keeps for its fuel line, between its last control and the hint.
pub(super) const FUEL_LINE: f32 = 12.0;

/// Room below a page's last control for the hint line.
pub(super) const CARD_FOOT: f32 = 28.0;

/// The Arrow counter page's rows.
pub(super) const COUNTER_ON_Y: f32 = 34.0;

pub(super) const COUNTER_BOW_Y: f32 = COUNTER_ON_Y + ROW_PITCH;

pub(super) const COUNTER_CAPTION_Y: f32 = COUNTER_BOW_Y + ROW_PITCH + SECTION_GAP;

pub(super) const COUNTER_MOVE_Y: f32 = COUNTER_CAPTION_Y + 14.0;

pub(super) const COUNTER_SIZE_Y: f32 = COUNTER_MOVE_Y + ROW_PITCH;

pub(super) const COUNTER_RESET_Y: f32 = COUNTER_SIZE_Y + ROW_PITCH;

/// The CPS page's rows.
pub(super) const CPS_ON_Y: f32 = 34.0;

pub(super) const CPS_RIGHT_Y: f32 = CPS_ON_Y + ROW_PITCH;

pub(super) const CPS_ZEROS_Y: f32 = CPS_RIGHT_Y + ROW_PITCH;

pub(super) const CPS_CAPTION_Y: f32 = CPS_ZEROS_Y + ROW_PITCH + SECTION_GAP;

pub(super) const CPS_MOVE_Y: f32 = CPS_CAPTION_Y + 14.0;

pub(super) const CPS_SIZE_Y: f32 = CPS_MOVE_Y + ROW_PITCH;

pub(super) const CPS_RESET_Y: f32 = CPS_SIZE_Y + ROW_PITCH;

/// The FPS counter page's rows.
pub(super) const FPS_ON_Y: f32 = 34.0;

pub(super) const FPS_CAPTION_Y: f32 = FPS_ON_Y + ROW_PITCH + SECTION_GAP;

pub(super) const FPS_MOVE_Y: f32 = FPS_CAPTION_Y + 14.0;

pub(super) const FPS_SIZE_Y: f32 = FPS_MOVE_Y + ROW_PITCH;

pub(super) const FPS_RESET_Y: f32 = FPS_SIZE_Y + ROW_PITCH;

/// The Lighting page's rows.
pub(super) const LIGHT_ON_Y: f32 = 34.0;

pub(super) const LIGHT_KEY_Y: f32 = LIGHT_ON_Y + ROW_PITCH;

/// The Jump reset page's rows: the switch, then the timing section with its value, and a note.
pub(super) const JUMP_ON_Y: f32 = 34.0;

pub(super) const JUMP_CAPTION_Y: f32 = JUMP_ON_Y + ROW_PITCH + SECTION_GAP;

pub(super) const JUMP_WINDOW_Y: f32 = JUMP_CAPTION_Y + 14.0;

pub(super) const JUMP_NOTE_Y: f32 = JUMP_WINDOW_Y + ROW_PITCH + 6.0;

/// What the page says under its value; each line has to fit the card's width.
pub(super) const JUMP_NOTE: [&str; 3] = [
    "Jump the moment you get hit.",
    "Shows early or late by the crosshair",
    "and whether you took knockback.",
];

/// The Crosshair page's rows, below its colour button: the shape section, then the options.
pub(super) const CROSS_STYLE_Y: f32 = ROW_Y;

pub(super) const CROSS_LENGTH_Y: f32 = CROSS_STYLE_Y + ROW_PITCH;

pub(super) const CROSS_THICK_Y: f32 = CROSS_LENGTH_Y + ROW_PITCH;

pub(super) const CROSS_GAP_Y: f32 = CROSS_THICK_Y + ROW_PITCH;

pub(super) const CROSS_OPTIONS_CAPTION_Y: f32 = CROSS_GAP_Y + ROW_PITCH + SECTION_GAP;

pub(super) const CROSS_OPACITY_Y: f32 = CROSS_OPTIONS_CAPTION_Y + 14.0;

pub(super) const CROSS_TOGGLES_Y: f32 = CROSS_OPACITY_Y + ROW_PITCH;

pub(super) const CROSS_RESET_Y: f32 = CROSS_TOGGLES_Y + ROW_PITCH;

/// The drawing view: the grid's top-left corner and side, then a line of help and the Mirror
/// and Clear buttons under it. Each of the grid's pixels is a square `DRAW_GRID[2]` divided by
/// the drawing's resolution wide.
pub(super) const DRAW_GRID: [f32; 3] = [
    (SETTINGS_W - crate::crosshair::GRID_VIEW) / 2.0,
    53.0,
    crate::crosshair::GRID_VIEW,
];

pub(super) const DRAW_NOTE_Y: f32 = DRAW_GRID[1] + DRAW_GRID[2] + 3.0;

pub(super) const DRAW_BUTTONS_Y: f32 = DRAW_NOTE_Y + 12.0;

/// The top row's right half, beside the Colour button: Draw (on the page and in the colour
/// tools), or the resolution with its - and + (in the drawing view).
pub(super) const TOP_RIGHT_X: f32 = 10.0 + HALF + 4.0;

pub(super) const RES_VALUE_W: f32 = HALF - 2.0 * (STEP_W + 4.0);

/// The Zoom page's rows.
pub(super) const ZOOM_ON_Y: f32 = 34.0;

pub(super) const ZOOM_KEY_Y: f32 = ZOOM_ON_Y + ROW_PITCH;

pub(super) const ZOOM_MODE_Y: f32 = ZOOM_KEY_Y + ROW_PITCH;

pub(super) const ZOOM_SCROLL_Y: f32 = ZOOM_MODE_Y + ROW_PITCH;

pub(super) const ZOOM_SENS_Y: f32 = ZOOM_SCROLL_Y + ROW_PITCH;

pub(super) const ZOOM_CAPTION_Y: f32 = ZOOM_SENS_Y + ROW_PITCH + SECTION_GAP;

pub(super) const ZOOM_LEVEL_Y: f32 = ZOOM_CAPTION_Y + 14.0;

pub(super) const ZOOM_TIME_Y: f32 = ZOOM_LEVEL_Y + ROW_PITCH;

pub(super) const ZOOM_STEP_Y: f32 = ZOOM_TIME_Y + ROW_PITCH;

pub(super) const ZOOM_RESET_Y: f32 = ZOOM_STEP_Y + ROW_PITCH;

/// The guide shown at the top of the screen while the counter is moved.
pub const GUIDE_W: f32 = 240.0;

/// Where the guide's text starts, after its accent.
pub(super) const GUIDE_TEXT_X: f32 = 20.0;

/// What the guide says; each line has to fit the guide's width (about 5.9 units a letter).
pub(super) const GUIDE_LINES: [&str; 3] = [
    "Drag a HUD element to move it",
    "Drag any corner square to resize it",
    "Snaps to the hotbar and screen middle",
];

pub const GUIDE_H: f32 = 68.0;

/// The card's height for what is shown: each page is only as tall as its controls, with the
/// hint line below them; the colour tools use the full height.
pub(super) fn card_height(state: MenuState) -> f32 {
    match state.page {
        _ if state.colour_open && state.page.has_colour() => CARD_H,
        Page::Crosshair if state.drawing => DRAW_BUTTONS_Y + BUTTON_H + CARD_FOOT,
        Page::Home => HOME_KEY_Y + BUTTON_H + CARD_FOOT + FUEL_LINE,
        Page::Counter => COUNTER_RESET_Y + BUTTON_H + CARD_FOOT,
        Page::Cps => CPS_RESET_Y + BUTTON_H + CARD_FOOT,
        Page::Fps => FPS_RESET_Y + BUTTON_H + CARD_FOOT,
        Page::Zoom => ZOOM_RESET_Y + BUTTON_H + CARD_FOOT,
        Page::Lighting => LIGHT_KEY_Y + BUTTON_H + CARD_FOOT,
        Page::JumpReset => JUMP_NOTE_Y + 12.0 * JUMP_NOTE.len() as f32 + CARD_FOOT,
        Page::Crosshair => CROSS_RESET_Y + BUTTON_H + CARD_FOOT,
        Page::Block => HIGHLIGHT_Y + BUTTON_H + CARD_FOOT,
        Page::Mob => BOTTOM_Y - MOB_OPTIONS_SHIFT + BUTTON_H + CARD_FOOT,
    }
}

/// The colour view's Back button, top right.
/// The close button, top right on every page, and Back beside it.
pub(super) const CLOSE_AREA: [f32; 4] = [SETTINGS_W - 30.0, 5.0, 20.0, BUTTON_H];

pub(super) const BACK_AREA: [f32; 4] = [SETTINGS_W - 84.0, 5.0, 50.0, BUTTON_H];

/// Section captions of the settings view, above the bars and the options.
pub(super) const LOOK_CAPTION_Y: f32 = 55.0;

pub(super) const ROW_Y: f32 = 69.0;

pub(super) const ROW_STEP: f32 = 24.0;

pub(super) const ROW_HEIGHT: f32 = BUTTON_H;

pub(super) const OPTIONS_CAPTION_Y: f32 = ROW_Y + ROW_STEP * 3.0 + SECTION_GAP;

pub(super) const TOGGLES_Y: f32 = OPTIONS_CAPTION_Y + 14.0;

pub(super) const BOTTOM_Y: f32 = TOGGLES_Y + ROW_PITCH;

pub(super) const KEY_Y: f32 = BOTTOM_Y + ROW_PITCH;

/// The block page's game highlight fix, where the menu key was before it moved to the start.
pub(super) const HIGHLIGHT_Y: f32 = KEY_Y;

/// Slider rows: name, typed number box and bar. The colour column's bars are shorter.
pub(super) const VALUE_X: f32 = 66.0;

/// Where the button of a "label, then key button" row starts: room for "Fullbright key".
pub(super) const KEY_X: f32 = 92.0;

pub(super) const VALUE_WIDTH: f32 = 36.0;

pub(super) const TRACK_X: f32 = 108.0;

pub(super) const fn track_width() -> f32 {
    SETTINGS_W - 10.0 - TRACK_X
}

/// The colour tools, below the Line / Fill colour buttons in place of the settings.
pub(super) const RGB_Y: f32 = 54.0;

/// Wheel square's top-left corner and side, and the size of its colour cells.
pub(super) const WHEEL: [f32; 3] = [10.0, 122.0, 90.0];

pub(super) const WHEEL_CELL: f32 = 3.6;

/// A wheel cell is a little larger than the grid step, so neighbours overlap and leave no seams;
/// a quarter cell (on the rim) is a little over half of that. Each size is set once, on the
/// template its cells inherit, which keeps hundreds of cells small.
pub(super) const WHEEL_CELL_SIZE: f32 = WHEEL_CELL + 0.25;

pub(super) const WHEEL_QUARTER_SIZE: f32 = WHEEL_CELL_SIZE / 2.0 + 0.2;

pub(super) const MARKER: f32 = 7.0;

pub(super) const HEX_X: f32 = WHEEL[0] + WHEEL[2] + 10.0;

pub(super) const HEX_Y: f32 = 124.0;

pub(super) const SWATCH_Y: f32 = 224.0;

pub(super) const FAVOURITE_Y: f32 = 264.0;

pub(super) const TEXT: [f32; 3] = [0.92, 0.93, 0.95];

pub(super) const DIM: [f32; 3] = [0.74, 0.76, 0.8];

pub(super) const TRACK: [f32; 4] = [0.235, 0.245, 0.29, 1.0];

/// The "on" colour: dark enough for the light text on it to pass WCAG AA (4.5:1), see the test.
pub(super) const ACTIVE: [f32; 4] = [0.45, 0.24, 0.75, 1.0];

/// A reset button waiting for its second click.
pub(super) const DANGER: [f32; 4] = [0.62, 0.2, 0.18, 1.0];

/// A switch's pill and knob, in panel units; the knob's left edge is `KNOB_OFF` off, and
/// `PILL_W - KNOB - KNOB_OFF` on.
pub(super) const PILL_W: f32 = 28.0;

pub(super) const PILL_H: f32 = 14.0;

pub(super) const KNOB: f32 = 10.0;

pub(super) const KNOB_OFF: f32 = 2.0;

/// A slider's thin bar, its round knob and the bar's empty colour.
pub(super) const TRACK_H: f32 = 6.0;

pub(super) const SLIDER_KNOB: f32 = 12.0;

pub(super) const SLIDER_TRACK: [f32; 4] = [0.22, 0.23, 0.28, 1.0];

/// The switches: the pill's colour while off (on is `ACTIVE`), and the header accent.
pub(super) const SWITCH_OFF: [f32; 4] = [0.43, 0.44, 0.52, 1.0];

/// The start page's switches sit right of the page buttons: this wide, after a small gap.
pub(super) const HOME_SWITCH_W: f32 = 40.0;

pub(super) const HOME_GAP: f32 = 4.0;

/// The width of a start page button.
pub(super) const HOME_BUTTON_W: f32 = SETTINGS_W - 20.0 - HOME_SWITCH_W - HOME_GAP;

/// What each start page button says under its name (Block, Mob, Counter, Zoom, CPS, Lighting,
/// Jump reset, Crosshair).
pub(super) const HOME_ABOUT: [&str; 9] = [
    "The block you look at",
    "Mobs and players you look at",
    "Arrows left, on your HUD",
    "Hold a key to zoom in",
    "Left/right clicks per second",
    "Fullbright for caves, nights",
    "Times your jump after a hit",
    "Your own shape and colour",
    "Frames per second, on your HUD",
];

/// Settings sit in lighter inset panels, `SECTION_PAD` wider than their rows; the gap between two
/// panels is `SECTION_GAP` more than the rows' own spacing.
pub(super) const SECTION: [f32; 4] = [0.135, 0.145, 0.185, 1.0];

pub(super) const SECTION_PAD: f32 = 5.0;

pub(super) const SECTION_GAP: f32 = 6.0;

/// Section captions are small capitals in a soft accent; the title is drawn this much larger.
pub(super) const CAPTION_TEXT: [f32; 3] = [0.72, 0.66, 0.94];

pub(super) const TITLE_SCALE: f32 = 1.25;

/// The tile behind a home button's icon, and the bottom lip that makes a key look like a key.
pub(super) const ICON_BG: [f32; 4] = [0.12, 0.13, 0.17, 1.0];

pub(super) const KEY_LIP: [f32; 4] = [0.1, 0.11, 0.14, 1.0];

pub(super) const KEY_LIP_H: f32 = 2.0;

/// The home buttons' badge, and the room the header accent takes before the title.
pub(super) const BADGE: f32 = 22.0;

pub(super) const TITLE_X: f32 = 8.0;

pub(super) const ACCENT: [f32; 4] = [0.66, 0.5, 0.97, 1.0];

/// The switch rows: bound by `#<name>_bg` (pill colour) and `#<name>_knob` (knob position).
pub(super) const SWITCHES: [&str; 28] = [
    "home_block",
    "home_mob",
    "home_counter",
    "home_cps",
    "home_zoom",
    "home_light",
    "rainbow",
    "pulse",
    "fill_match",
    "show_all_mobs",
    "fullbright",
    "counter",
    "counter_bow",
    "zoom",
    "zoom_mode",
    "zoom_scroll",
    "zoom_sens",
    "cps_on",
    "cps_right",
    "cps_zeros",
    "fps_on",
    "home_fps",
    "home_jump",
    "jump_on",
    "home_crosshair",
    "crosshair_on",
    "crosshair_outline",
    "crosshair_mirror",
];

/// The card's outline and the keyboard focus ring.
pub(super) const BORDER: [f32; 4] = [0.3, 0.31, 0.37, 1.0];

/// The keyboard focus frame: a thin outline `FOCUS_GAP` outside the control, and a faint tint
/// inside it that covers nothing.
pub(super) const FOCUS: [f32; 4] = [0.72, 0.62, 0.97, 0.85];

pub(super) const FOCUS_TINT: [f32; 4] = [0.78, 0.66, 1.0, 0.12];

pub(super) const FOCUS_GAP: f32 = 2.0;

pub(super) const FOCUS_LINE: f32 = 1.5;

pub(super) const EMPTY_SLOT: [f32; 4] = [0.12, 0.13, 0.15, 1.0];

/// Gap between the card and the right edge of the screen.
pub(super) const SCREEN_MARGIN: f32 = 12.0;

pub(super) const NUMBER_BOX: [f32; 4] = [0.2, 0.21, 0.26, 1.0];

/// Thin dividers: under the title and beside section captions.
pub(super) const RULE: [f32; 4] = [0.26, 0.27, 0.31, 1.0];
