//! Control ids and the sizes the menu is laid out in.

use super::*;

/// The menu's width: one narrow column at the right edge of the screen, so the middle of the
/// screen, where the outline is, stays clear.
pub(super) const SETTINGS_W: f32 = 264.0;

/// Card height, the same for the settings and the colour tools so nothing moves when switching.
/// (Twelve page buttons and the Menu section fill the start page.)
pub(super) const CARD_H: f32 = 413.0;

/// Height of every button and small control row; tall enough to hit comfortably.
pub(super) const BUTTON_H: f32 = 18.0;

/// Vertical distance between two rows of buttons.
pub(super) const ROW_PITCH: f32 = 24.0;

/// Width of one half of the settings column, for rows holding two buttons.
pub(super) const HALF: f32 = (SETTINGS_W - 24.0) / 2.0;

/// Slider rows: control id and bar colour.
pub(super) const SLIDERS: [(&str, [f32; 4]); 6] = [
    ("red", [0.86, 0.24, 0.24, 1.0]),
    ("green", [0.28, 0.78, 0.32, 1.0]),
    ("blue", [0.26, 0.46, 0.92, 1.0]),
    ("opacity", ACCENT),
    ("width", ACCENT),
    ("fill", ACCENT),
];

// Control indices. New controls are appended so existing indices never move.
pub(super) const RAINBOW: usize = SLIDERS.len();

pub(super) const FIRST_SWATCH: usize = RAINBOW + 1;

/// Hidden sliders the wheel's `mod.vector` route drives.
pub(super) const WHEEL_X: usize = FIRST_SWATCH + SWATCHES.len();

pub(super) const WHEEL_Y: usize = WHEEL_X + 1;

pub(super) const HEX: usize = WHEEL_Y + 1;

pub(super) const MENU_KEY: usize = HEX + 1;

pub(super) const PULSE: usize = MENU_KEY + 1;

pub(super) const RESET: usize = PULSE + 1;

pub(super) const EDIT_LINE: usize = RESET + 1;

pub(super) const EDIT_FILL: usize = EDIT_LINE + 1;

pub(super) const FILL_MATCH: usize = EDIT_FILL + 1;

pub(super) const FIRST_FAVOURITE: usize = FILL_MATCH + 1;

pub(super) const SAVE_FAVOURITE: usize = FIRST_FAVOURITE + FAVOURITES;

/// Buttons that start typing into a slider's number box, then the hex code box.
pub(super) const FIRST_TYPE: usize = SAVE_FAVOURITE + 1;

pub(super) const HIDE_HIGHLIGHT: usize = FIRST_TYPE + SLIDERS.len() + 1;

/// Leaves the colour tools for the settings.
pub(super) const BACK: usize = HIDE_HIGHLIGHT + 1;

/// The start page's buttons.
pub(super) const OPEN_BLOCK: usize = BACK + 1;

pub(super) const OPEN_MOB: usize = OPEN_BLOCK + 1;

/// The Mob hitbox page's All mobs switch and Aimed colour button.
pub(super) const SHOW_ALL: usize = OPEN_MOB + 1;

pub(super) const EDIT_AIM: usize = SHOW_ALL + 1;

/// The game highlight fix's - and + buttons.
pub(super) const EDIT_ARROW: usize = EDIT_AIM + 1;

pub(super) const HIGHLIGHT_LESS: usize = EDIT_ARROW + 1;

pub(super) const HIGHLIGHT_MORE: usize = HIGHLIGHT_LESS + 1;

/// The start page's Fullbright switch.
pub(super) const FULLBRIGHT: usize = HIGHLIGHT_MORE + 1;

pub(super) const FULLBRIGHT_KEY: usize = FULLBRIGHT + 1;

/// Closes the menu.
pub(super) const CLOSE: usize = FULLBRIGHT_KEY + 1;

/// The start page's Arrow counter button and that page's controls.
pub(super) const OPEN_COUNTER: usize = CLOSE + 1;

pub(super) const COUNTER: usize = OPEN_COUNTER + 1;

pub(super) const COUNTER_BOW: usize = COUNTER + 1;

pub(super) const COUNTER_MOVE: usize = COUNTER_BOW + 1;

/// The size button (back to 100%) and its - and + buttons.
pub(super) const COUNTER_SIZE: usize = COUNTER_MOVE + 1;

pub(super) const COUNTER_SMALLER: usize = COUNTER_SIZE + 1;

pub(super) const COUNTER_BIGGER: usize = COUNTER_SMALLER + 1;

pub(super) const COUNTER_RESET: usize = COUNTER_BIGGER + 1;

/// Ends moving the counter.
pub(super) const ARMOR_HORIZONTAL: usize = COUNTER_RESET + 1;
pub(super) const ARMOR_PERCENT: usize = ARMOR_HORIZONTAL + 1;
pub(super) const ARMOR_BACKGROUND: usize = ARMOR_PERCENT + 1;
pub(super) const COUNTER_DONE: usize = ARMOR_BACKGROUND + 1;

/// The start page's Zoom button and that page's controls: switches, the zoom key and three
/// values, each a button that resets it beside its - and + buttons.
pub(super) const OPEN_ZOOM: usize = COUNTER_DONE + 1;

pub(super) const ZOOM_ON: usize = OPEN_ZOOM + 1;

pub(super) const ZOOM_MODE: usize = ZOOM_ON + 1;

pub(super) const ZOOM_SCROLL: usize = ZOOM_MODE + 1;

pub(super) const ZOOM_SENS: usize = ZOOM_SCROLL + 1;

pub(super) const ZOOM_KEY: usize = ZOOM_SENS + 1;

pub(super) const ZOOM_LEVEL: usize = ZOOM_KEY + 1;

pub(super) const ZOOM_LEVEL_LESS: usize = ZOOM_LEVEL + 1;

pub(super) const ZOOM_LEVEL_MORE: usize = ZOOM_LEVEL_LESS + 1;

pub(super) const ZOOM_TIME: usize = ZOOM_LEVEL_MORE + 1;

pub(super) const ZOOM_TIME_LESS: usize = ZOOM_TIME + 1;

pub(super) const ZOOM_TIME_MORE: usize = ZOOM_TIME_LESS + 1;

pub(super) const ZOOM_STEP: usize = ZOOM_TIME_MORE + 1;

pub(super) const ZOOM_STEP_LESS: usize = ZOOM_STEP + 1;

pub(super) const ZOOM_STEP_MORE: usize = ZOOM_STEP_LESS + 1;

pub(super) const ZOOM_RESET: usize = ZOOM_STEP_MORE + 1;

/// The start page's CPS button and that page's controls: two switches, moving, and the size
/// value beside its - and + buttons.
pub(super) const OPEN_CPS: usize = ZOOM_RESET + 1;

pub(super) const CPS_ON: usize = OPEN_CPS + 1;

pub(super) const CPS_RIGHT: usize = CPS_ON + 1;

pub(super) const CPS_MOVE: usize = CPS_RIGHT + 1;

pub(super) const CPS_SIZE: usize = CPS_MOVE + 1;

pub(super) const CPS_SMALLER: usize = CPS_SIZE + 1;

pub(super) const CPS_BIGGER: usize = CPS_SMALLER + 1;

pub(super) const CPS_RESET: usize = CPS_BIGGER + 1;

/// The CPS page's Leading zeros switch, added last so earlier indices never move.
pub(super) const CPS_ZEROS: usize = CPS_RESET + 1;

/// The start page's Lighting button. Its page holds Fullbright and its key (earlier controls).
pub(super) const OPEN_LIGHT: usize = CPS_ZEROS + 1;

/// The start page's on/off switches, one beside each page button (Block, Mob, Counter, CPS,
/// Zoom, Lighting).
pub(super) const HOME_BLOCK_ON: usize = OPEN_LIGHT + 1;

pub(super) const HOME_MOB_ON: usize = HOME_BLOCK_ON + 1;

pub(super) const HOME_COUNTER_ON: usize = HOME_MOB_ON + 1;

pub(super) const HOME_CPS_ON: usize = HOME_COUNTER_ON + 1;

pub(super) const HOME_ZOOM_ON: usize = HOME_CPS_ON + 1;

pub(super) const HOME_LIGHT_ON: usize = HOME_ZOOM_ON + 1;

/// The start page's Jump reset button, that page's switch and perfect-window value (with its
/// - and + buttons), and the button's own switch on the start page.
pub(super) const OPEN_JUMP: usize = HOME_LIGHT_ON + 1;

pub(super) const JUMP_ON: usize = OPEN_JUMP + 1;

pub(super) const JUMP_WINDOW: usize = JUMP_ON + 1;

pub(super) const JUMP_WINDOW_LESS: usize = JUMP_WINDOW + 1;

pub(super) const JUMP_WINDOW_MORE: usize = JUMP_WINDOW_LESS + 1;

pub(super) const HOME_JUMP_ON: usize = JUMP_WINDOW_MORE + 1;

/// The start page's Crosshair button and its switch, then that page's controls: the colour
/// button, four values with - and + (style, length, thickness, gap), opacity likewise, two
/// switches and reset.
pub(super) const OPEN_CROSS: usize = HOME_JUMP_ON + 1;

pub(super) const HOME_CROSS_ON: usize = OPEN_CROSS + 1;

pub(super) const CROSS_COLOUR: usize = HOME_CROSS_ON + 1;

pub(super) const CROSS_STYLE: usize = CROSS_COLOUR + 1;

pub(super) const CROSS_LENGTH: usize = CROSS_STYLE + 3;

pub(super) const CROSS_THICK: usize = CROSS_LENGTH + 3;

pub(super) const CROSS_GAP: usize = CROSS_THICK + 3;

pub(super) const CROSS_OPACITY: usize = CROSS_GAP + 3;

pub(super) const CROSS_ON: usize = CROSS_OPACITY + 3;

pub(super) const CROSS_OUTLINE: usize = CROSS_ON + 1;

pub(super) const CROSS_RESET: usize = CROSS_OUTLINE + 1;

/// The Crosshair page's Draw button, and the drawing view's Mirror switch and Clear button.
pub(super) const CROSS_DRAW: usize = CROSS_RESET + 1;

pub(super) const DRAW_MIRROR: usize = CROSS_DRAW + 1;

pub(super) const DRAW_CLEAR: usize = DRAW_MIRROR + 1;

/// The drawing view's resolution, with its - and + buttons.
pub(super) const DRAW_RES: usize = DRAW_CLEAR + 1;

/// The Crosshair page's values: id, caption, first control index (the value; its - and +
/// follow) and row.
pub(super) const CROSS_VALUES: [(&str, &str, usize, f32); 5] = [
    ("crosshair_style", "Style", CROSS_STYLE, CROSS_STYLE_Y),
    ("crosshair_length", "Length", CROSS_LENGTH, CROSS_LENGTH_Y),
    (
        "crosshair_thickness",
        "Thickness",
        CROSS_THICK,
        CROSS_THICK_Y,
    ),
    ("crosshair_gap", "Gap", CROSS_GAP, CROSS_GAP_Y),
    (
        "crosshair_opacity",
        "Opacity",
        CROSS_OPACITY,
        CROSS_OPACITY_Y,
    ),
];

/// The FPS counter page's controls, added last so earlier indices never move: the switch,
/// moving, the size value beside its - and + buttons, and reset.
pub(super) const FPS_ON: usize = DRAW_RES + 3;

pub(super) const FPS_MOVE: usize = FPS_ON + 1;

pub(super) const FPS_SIZE: usize = FPS_MOVE + 1;

pub(super) const FPS_SMALLER: usize = FPS_SIZE + 1;

pub(super) const FPS_BIGGER: usize = FPS_SMALLER + 1;

pub(super) const FPS_RESET: usize = FPS_BIGGER + 1;

/// The start page's FPS counter button and its switch.
pub(super) const OPEN_FPS: usize = FPS_RESET + 1;

pub(super) const HOME_FPS_ON: usize = OPEN_FPS + 1;

pub(super) const OPEN_ARMOR: usize = HOME_FPS_ON + 1;
pub(super) const HOME_ARMOR_ON: usize = OPEN_ARMOR + 1;

pub(super) const OPEN_VANILLA: usize = HOME_ARMOR_ON + 1;
pub(super) const SHOW_HUNGER: usize = OPEN_VANILLA + 1;

pub(super) const EFFECT_TIMERS: usize = SHOW_HUNGER + 1;
pub(super) const HOTBAR_KEYS: usize = EFFECT_TIMERS + 1;

pub(super) const ARMOR_MOVE: usize = HOTBAR_KEYS + 1;
pub(super) const ARMOR_SIZE: usize = ARMOR_MOVE + 1;
pub(super) const ARMOR_SMALLER: usize = ARMOR_SIZE + 1;
pub(super) const ARMOR_BIGGER: usize = ARMOR_SMALLER + 1;
pub(super) const ARMOR_RESET: usize = ARMOR_BIGGER + 1;

/// The FPS counter page's coordinates switch, added after the armor controls so no earlier index
/// moves.
pub(super) const FPS_COORDS: usize = ARMOR_RESET + 1;

/// The Hive Utilities page's Auto queue switch, added last so no earlier index moves, and the
/// start page's button for that page.
pub(super) const AUTO_QUEUE: usize = FPS_COORDS + 1;

pub(super) const OPEN_HIVE: usize = AUTO_QUEUE + 1;

/// The last control index.
#[cfg(test)]
pub(super) const LAST_CONTROL: usize = OPEN_HIVE;
