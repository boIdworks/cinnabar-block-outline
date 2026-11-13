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
    Armor,
    VanillaHud,
    /// The Hive Utilities page.
    HiveUtilities,
    /// The CPS page.
    Cps,
    /// The FPS counter page.
    Fps,
    /// The Zoom page.
    Zoom,
    /// The Lighting page.
    Lighting,
    /// The Jump reset page.
    JumpReset,
    /// The Crosshair page's settings view.
    Crosshair,
    /// The Crosshair page's colour button, in every view.
    CrosshairTop,
    /// The Crosshair page's Draw button beside it, but in the drawing view.
    CrosshairDrawButton,
    /// The Crosshair page's drawing view.
    Draw,
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
    let mut table = Table(Vec::with_capacity(112));
    home(&mut table);
    block_and_mob(&mut table);
    counter(&mut table);
    cps(&mut table);
    lighting(&mut table);
    jump(&mut table);
    crosshair(&mut table);
    zoom(&mut table);
    always(&mut table);
    table.0
}

/// The controls being collected, in the order the keyboard walks them.
struct Table(Vec<Control>);

impl Table {
    fn add(&mut self, view: View, area: [f32; 4], tip: &'static str, action: Action) {
        self.0.push(Control {
            view,
            area,
            tip,
            action,
        });
    }
}

fn press(id: &str) -> Action {
    Action::Press(id.to_owned())
}

const FULL: f32 = SETTINGS_W - 20.0;
const RIGHT: f32 = 10.0 + HALF + 4.0;
const HOME_SWITCH_X: f32 = 10.0 + HOME_BUTTON_W + HOME_GAP;
const STEPPER_TIP: &str = "Left/Right change it; Enter resets it";
const TYPE: &str = "Click, type a number, press Enter";

fn row(y: f32) -> [f32; 4] {
    [10.0, y, FULL, BUTTON_H]
}

fn half(y: f32) -> [f32; 4] {
    [10.0, y, HALF, BUTTON_H]
}

fn half_right(y: f32) -> [f32; 4] {
    [RIGHT, y, HALF, BUTTON_H]
}

/// A value with - and + beside it: the row (for the keyboard), the value and the pair.
fn value_area(y: f32) -> [f32; 4] {
    [10.0, y, HIGHLIGHT_W, BUTTON_H]
}

fn pair_area(y: f32) -> [f32; 4] {
    [
        10.0 + HIGHLIGHT_W,
        y,
        SETTINGS_W - 20.0 - HIGHLIGHT_W,
        BUTTON_H,
    ]
}

/// The start page: a button and a switch for each tool.
fn home(t: &mut Table) {
    // The start page.
    t.add(
        View::Home,
        [10.0, HOME_BLOCK_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Colours and size of the block outline",
        press("open_block"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_BLOCK_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Show or hide the block outline",
        Action::Toggle("home_block"),
    );
    t.add(
        View::Home,
        [10.0, HOME_MOB_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Colours and size of mob hitboxes",
        press("open_mob"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_MOB_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Show or hide mob hitboxes",
        Action::Toggle("home_mob"),
    );
    t.add(
        View::Home,
        [10.0, HOME_COUNTER_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Shows how many arrows you carry",
        press("open_counter"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_COUNTER_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Show or hide the arrow counter",
        Action::Toggle("home_counter"),
    );
    t.add(
        View::Home,
        [10.0, HOME_CPS_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Left and right clicks per second",
        press("open_cps"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_CPS_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Show or hide the CPS counter",
        Action::Toggle("home_cps"),
    );
    t.add(
        View::Home,
        [10.0, HOME_FPS_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Frames per second, on your HUD",
        press("open_fps"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_FPS_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Show or hide the FPS counter",
        Action::Toggle("home_fps"),
    );
    t.add(
        View::Home,
        [10.0, HOME_ZOOM_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Hold a key to zoom in smoothly",
        press("open_zoom"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_ZOOM_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Turn zoom on or off",
        Action::Toggle("home_zoom"),
    );
    t.add(
        View::Home,
        [10.0, HOME_LIGHT_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Fullbright and its key",
        press("open_light"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_LIGHT_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Turn fullbright on or off",
        Action::Toggle("home_light"),
    );
    t.add(
        View::Home,
        [10.0, HOME_JUMP_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Tells you if you jump early or late",
        press("open_jump"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_JUMP_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Turn the jump reset timing on or off",
        Action::Toggle("home_jump"),
    );
    t.add(
        View::Home,
        [10.0, HOME_CROSS_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Shape, size and colour of your crosshair",
        press("open_crosshair"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_CROSS_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Use your own crosshair or the game's",
        Action::Toggle("home_crosshair"),
    );
    t.add(
        View::Home,
        [10.0, HOME_ARMOR_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Armor HUD options",
        press("open_armor"),
    );
    t.add(
        View::Home,
        [HOME_SWITCH_X, HOME_ARMOR_Y, HOME_SWITCH_W, HOME_BUTTON_H],
        "Show or hide the armor HUD",
        Action::Toggle("home_armor"),
    );
    t.add(
        View::Home,
        [10.0, HOME_VANILLA_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Show or hide parts of the vanilla HUD",
        press("open_vanilla"),
    );
    t.add(
        View::Home,
        [10.0, HOME_HIVE_Y, HOME_BUTTON_W, HOME_BUTTON_H],
        "Auto queue for Hive games",
        press("open_hive"),
    );
    t.add(
        View::HiveUtilities,
        row(HIVE_QUEUE_Y),
        "Queues the next Hive game when one ends",
        Action::Toggle("auto_queue"),
    );
    t.add(
        View::VanillaHud,
        row(COUNTER_ON_Y),
        "Show or hide the hunger icons",
        Action::Toggle("show_hunger"),
    );
    t.add(
        View::VanillaHud,
        row(VANILLA_EFFECTS_Y),
        "Show or hide the effect timers",
        Action::Toggle("effect_timers"),
    );
    t.add(
        View::VanillaHud,
        row(VANILLA_HOTBAR_Y),
        "Show or hide the hotbar key labels",
        Action::Toggle("hotbar_keys"),
    );
    t.add(
        View::Home,
        row(HOME_KEY_Y),
        "Click, press a key; Esc cancels",
        Action::Key("menu_key"),
    );
}

/// The Block outline and Mob hitbox pages: the colour buttons, the settings view and the colour view.
fn block_and_mob(t: &mut Table) {
    // The Block outline and Mob hitbox pages: the colour buttons at the top.
    t.add(
        View::FillOwn,
        half(EDIT_Y),
        "Shows or hides the line colour tools",
        press("edit_line"),
    );
    t.add(
        View::FillOwn,
        half_right(EDIT_Y),
        "Shows or hides the fill colour tools",
        press("edit_fill"),
    );
    t.add(
        View::FillMatch,
        row(EDIT_Y),
        "Colour tools; the fill uses it too",
        press("edit_line"),
    );
    t.add(
        View::MobTop,
        [10.0, EDIT_Y, (SETTINGS_W - 28.0) / 3.0, BUTTON_H],
        "Hitbox colour; the fill uses it too",
        press("edit_line"),
    );
    t.add(
        View::MobTop,
        [
            10.0 + (SETTINGS_W - 28.0) / 3.0 + 4.0,
            EDIT_Y,
            (SETTINGS_W - 28.0) / 3.0,
            BUTTON_H,
        ],
        "Colour of the mob you aim at",
        press("edit_aim"),
    );
    t.add(
        View::MobTop,
        [
            10.0 + 2.0 * ((SETTINGS_W - 28.0) / 3.0 + 4.0),
            EDIT_Y,
            (SETTINGS_W - 28.0) / 3.0,
            BUTTON_H,
        ],
        "Arrow hitbox colour",
        press("edit_arrow"),
    );

    // Their settings view: opacity, thickness and fill rows, then the options.
    let slider_row = |index: usize| {
        let (_, y) = slider_place(index);
        [10.0, y, TRACK_X + track_width() - 10.0, ROW_HEIGHT]
    };
    let number_box = |index: usize| {
        let (_, y) = slider_place(index);
        [VALUE_X, y, VALUE_WIDTH, ROW_HEIGHT]
    };
    t.add(View::Settings, number_box(3), TYPE, Action::Help);
    t.add(
        View::Settings,
        slider_row(3),
        "How solid the lines are (0 = invisible)",
        Action::Slider(3),
    );
    t.add(View::Settings, number_box(4), TYPE, Action::Help);
    t.add(
        View::Settings,
        slider_row(4),
        "Line thickness; thins out far away",
        Action::Slider(4),
    );
    t.add(View::BlockOnly, number_box(5), TYPE, Action::Help);
    t.add(
        View::BlockOnly,
        slider_row(5),
        "Tints the faces (0 = off)",
        Action::Slider(5),
    );
    t.add(
        View::BlockOnly,
        half(TOGGLES_Y),
        "Lines cycle through every colour",
        Action::Toggle("rainbow"),
    );
    t.add(
        View::BlockOnly,
        half_right(TOGGLES_Y),
        "The outline gently fades in and out",
        Action::Toggle("pulse"),
    );
    // Mob hitboxes have no pulse: their Rainbow switch takes the whole row.
    t.add(
        View::MobOnly,
        row(TOGGLES_Y),
        "Lines cycle through every colour",
        Action::Toggle("rainbow"),
    );
    t.add(
        View::BlockOnly,
        half(BOTTOM_Y),
        "On: the fill uses the line colour",
        Action::Toggle("fill_match"),
    );
    t.add(
        View::MobOnly,
        half(BOTTOM_Y),
        "Off: only the mob you aim at",
        Action::Toggle("show_all_mobs"),
    );
    t.add(
        View::Settings,
        half_right(BOTTOM_Y),
        "Resets this page; click twice to confirm",
        press("reset"),
    );
    t.add(
        View::BlockOnly,
        value_area(HIGHLIGHT_Y),
        "Hides the game's highlight; click: on/off",
        Action::Help,
    );
    t.add(
        View::BlockOnly,
        pair_area(HIGHLIGHT_Y),
        "5%% weaker or stronger",
        Action::Help,
    );
    t.add(
        View::BlockOnly,
        row(HIGHLIGHT_Y),
        "Left/Right change it; Enter: on/off",
        Action::Stepper {
            value: "hide_game_highlight",
            less: "highlight_less",
            more: "highlight_more",
        },
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
        t.add(View::Colour, number_box(index), TYPE, Action::Help);
        t.add(View::Colour, slider_row(index), tip, Action::Slider(index));
    }
    let [wx, wy, side] = WHEEL;
    t.add(
        View::Colour,
        [wx, wy, side, side],
        "Angle picks the hue; centre is paler",
        Action::Help,
    );
    let hex_w = SETTINGS_W - 10.0 - HEX_X;
    t.add(
        View::Colour,
        [HEX_X, HEX_Y, hex_w, 58.0],
        "Click, type e.g. FF8800, press Enter",
        Action::Help,
    );
    t.add(
        View::Colour,
        [HEX_X, HEX_Y + 12.0, hex_w, BUTTON_H],
        "Click, type e.g. FF8800, press Enter",
        Action::Hex,
    );
    let swatch_width = (SETTINGS_W - 20.0) / SWATCHES.len() as f32;
    for (index, (name, _)) in SWATCHES.iter().enumerate() {
        t.add(
            View::Colour,
            [
                10.0 + swatch_width * index as f32,
                SWATCH_Y + 12.0,
                swatch_width - 3.0,
                BUTTON_H,
            ],
            "Click a colour to use it straight away",
            press(&format!("swatch_{}", name.to_lowercase())),
        );
    }
    for slot in 0..FAVOURITES {
        t.add(
            View::Colour,
            [
                10.0 + 34.0 * slot as f32,
                FAVOURITE_Y + 12.0,
                30.0,
                BUTTON_H,
            ],
            "Click a saved colour to use it",
            press(&format!("favourite_{slot}")),
        );
    }
    let save_x = 10.0 + 34.0 * FAVOURITES as f32;
    t.add(
        View::Colour,
        [
            save_x,
            FAVOURITE_Y + 12.0,
            SETTINGS_W - 10.0 - save_x,
            BUTTON_H,
        ],
        "Saves the colour being edited",
        press("save_favourite"),
    );
}

/// The Arrow counter page.
fn counter(t: &mut Table) {
    // Armor HUD options share this page with the arrow counter.
    t.add(
        View::Armor,
        row(ARMOR_HORIZONTAL_Y),
        "Arrange armor side by side",
        Action::Toggle("armor_horizontal"),
    );
    t.add(
        View::Armor,
        row(ARMOR_PERCENT_Y),
        "Show remaining durability",
        Action::Toggle("armor_percent"),
    );
    t.add(
        View::Armor,
        row(ARMOR_BACKGROUND_Y),
        "Show a hotbar-style slot behind each item",
        Action::Toggle("armor_background"),
    );
    t.add(
        View::Armor,
        row(ARMOR_MOVE_Y),
        "Drag armor HUD to move it",
        press("armor_move"),
    );
    t.add(
        View::Armor,
        value_area(ARMOR_SIZE_Y),
        "Click to reset armor size",
        Action::Help,
    );
    t.add(
        View::Armor,
        pair_area(ARMOR_SIZE_Y),
        "10%% smaller or bigger",
        Action::Help,
    );
    t.add(
        View::Armor,
        row(ARMOR_SIZE_Y),
        STEPPER_TIP,
        Action::Stepper {
            value: "armor_size",
            less: "armor_size_less",
            more: "armor_size_more",
        },
    );
    t.add(
        View::Armor,
        row(ARMOR_RESET_Y),
        "Reset armor position and size; click twice",
        press("armor_reset"),
    );
    // The Arrow counter page.
    t.add(
        View::Counter,
        row(COUNTER_ON_Y),
        "Shows or hides the arrow counter",
        Action::Toggle("counter"),
    );
    t.add(
        View::Counter,
        row(COUNTER_BOW_Y),
        "Only while a bow or crossbow is held",
        Action::Toggle("counter_bow"),
    );
    t.add(
        View::Counter,
        row(COUNTER_MOVE_Y),
        "Move the HUD: drag, scroll to resize",
        press("counter_move"),
    );
    t.add(
        View::Counter,
        value_area(COUNTER_SIZE_Y),
        "Click the size to reset it to 100%%",
        Action::Help,
    );
    t.add(
        View::Counter,
        pair_area(COUNTER_SIZE_Y),
        "10%% smaller or bigger",
        Action::Help,
    );
    t.add(
        View::Counter,
        row(COUNTER_SIZE_Y),
        STEPPER_TIP,
        Action::Stepper {
            value: "counter_size",
            less: "counter_smaller",
            more: "counter_bigger",
        },
    );
    t.add(
        View::Counter,
        row(COUNTER_RESET_Y),
        "Beside the hotbar at 100%%; click twice",
        press("counter_reset"),
    );
}

/// The CPS and FPS counter pages.
fn cps(t: &mut Table) {
    // The CPS page.
    t.add(
        View::Cps,
        row(CPS_ON_Y),
        "Shows or hides the clicks per second",
        Action::Toggle("cps_on"),
    );
    t.add(
        View::Cps,
        row(CPS_RIGHT_Y),
        "Also show the right mouse button",
        Action::Toggle("cps_right"),
    );
    t.add(
        View::Cps,
        row(CPS_ZEROS_Y),
        "Show 07 instead of 7",
        Action::Toggle("cps_zeros"),
    );
    t.add(
        View::Cps,
        row(CPS_MOVE_Y),
        "Move the HUD: drag, scroll to resize",
        press("cps_move"),
    );
    t.add(
        View::Cps,
        value_area(CPS_SIZE_Y),
        "Click the size to reset it to 100%%",
        Action::Help,
    );
    t.add(
        View::Cps,
        pair_area(CPS_SIZE_Y),
        "10%% smaller or bigger",
        Action::Help,
    );
    t.add(
        View::Cps,
        row(CPS_SIZE_Y),
        STEPPER_TIP,
        Action::Stepper {
            value: "cps_size",
            less: "cps_size_less",
            more: "cps_size_more",
        },
    );
    t.add(
        View::Cps,
        row(CPS_RESET_Y),
        "Top left at 100%%; click twice",
        press("cps_reset"),
    );
    t.add(
        View::Fps,
        row(FPS_ON_Y),
        "Shows or hides the frames per second",
        Action::Toggle("fps_on"),
    );
    t.add(
        View::Fps,
        row(FPS_COORDS_Y),
        "Shows your block position under the FPS",
        Action::Toggle("fps_coords"),
    );
    t.add(
        View::Fps,
        row(FPS_MOVE_Y),
        "Move the HUD: drag, scroll to resize",
        press("fps_move"),
    );
    t.add(
        View::Fps,
        value_area(FPS_SIZE_Y),
        "Click the size to reset it to 100%%",
        Action::Help,
    );
    t.add(
        View::Fps,
        pair_area(FPS_SIZE_Y),
        "10%% smaller or bigger",
        Action::Help,
    );
    t.add(
        View::Fps,
        row(FPS_SIZE_Y),
        STEPPER_TIP,
        Action::Stepper {
            value: "fps_size",
            less: "fps_size_less",
            more: "fps_size_more",
        },
    );
    t.add(
        View::Fps,
        row(FPS_RESET_Y),
        "Below the CPS at 100%%; click twice",
        press("fps_reset"),
    );
}

/// The Lighting page.
fn lighting(t: &mut Table) {
    // The Lighting page.
    t.add(
        View::Lighting,
        row(LIGHT_ON_Y),
        "Lights up caves and nights (only you)",
        Action::Toggle("fullbright"),
    );
    t.add(
        View::Lighting,
        row(LIGHT_KEY_Y),
        "Click, press a key; Del clears it",
        Action::Key("fullbright_key"),
    );
}

/// The Jump reset page.
fn jump(t: &mut Table) {
    // The Jump reset page.
    t.add(
        View::JumpReset,
        row(JUMP_ON_Y),
        "Shows early or late when you jump",
        Action::Toggle("jump_on"),
    );
    t.add(
        View::JumpReset,
        value_area(JUMP_WINDOW_Y),
        "Jumps this close to the hit are perfect",
        Action::Help,
    );
    t.add(
        View::JumpReset,
        pair_area(JUMP_WINDOW_Y),
        "5 ms narrower or wider",
        Action::Help,
    );
    t.add(
        View::JumpReset,
        row(JUMP_WINDOW_Y),
        STEPPER_TIP,
        Action::Stepper {
            value: "jump_window",
            less: "jump_window_less",
            more: "jump_window_more",
        },
    );
}

/// The Crosshair page and its drawing view.
fn crosshair(t: &mut Table) {
    // The Crosshair page: its colour button, shown above the colour tools and the drawing too,
    // and beside it Draw, or (in the drawing view) the resolution.
    t.add(
        View::CrosshairTop,
        half(EDIT_Y),
        "Shows or hides the colour tools",
        press("crosshair_colour"),
    );
    t.add(
        View::CrosshairDrawButton,
        half_right(EDIT_Y),
        "Paint your own crosshair, pixel by pixel",
        press("crosshair_draw"),
    );
    for ((value, _, _, y), (value_tip, pair_tip, less, more)) in CROSS_VALUES.into_iter().zip([
        (
            "The shape; click to go back to a cross",
            "The previous or next shape",
            "crosshair_style_less",
            "crosshair_style_more",
        ),
        (
            "How long each line is; click to reset",
            "Half a pixel shorter or longer",
            "crosshair_length_less",
            "crosshair_length_more",
        ),
        (
            "How thick the lines are; click to reset",
            "Half a pixel thinner or thicker",
            "crosshair_thickness_less",
            "crosshair_thickness_more",
        ),
        (
            "Room in the middle; click to reset",
            "Half a pixel less or more",
            "crosshair_gap_less",
            "crosshair_gap_more",
        ),
        (
            "How solid it is; click to reset",
            "10%% less or more",
            "crosshair_opacity_less",
            "crosshair_opacity_more",
        ),
    ]) {
        t.add(View::Crosshair, value_area(y), value_tip, Action::Help);
        t.add(View::Crosshair, pair_area(y), pair_tip, Action::Help);
        t.add(
            View::Crosshair,
            row(y),
            STEPPER_TIP,
            Action::Stepper { value, less, more },
        );
    }
    t.add(
        View::Crosshair,
        half(CROSS_TOGGLES_Y),
        "Use it in place of the game's crosshair",
        Action::Toggle("crosshair_on"),
    );
    t.add(
        View::Crosshair,
        half_right(CROSS_TOGGLES_Y),
        "A thin dark edge, seen on any ground",
        Action::Toggle("crosshair_outline"),
    );
    t.add(
        View::Crosshair,
        row(CROSS_RESET_Y),
        "Back to defaults; click twice",
        press("crosshair_reset"),
    );

    // The drawing view: the grid, then Mirror and Clear.
    let [gx, gy, side] = DRAW_GRID;
    t.add(
        View::Draw,
        [gx, gy, side, side],
        "Left click draws, right click erases",
        Action::Help,
    );
    t.add(
        View::Draw,
        [TOP_RIGHT_X, EDIT_Y, RES_VALUE_W, BUTTON_H],
        "Pixels a side; click for 16x16",
        Action::Help,
    );
    t.add(
        View::Draw,
        [
            TOP_RIGHT_X + RES_VALUE_W,
            EDIT_Y,
            HALF - RES_VALUE_W,
            BUTTON_H,
        ],
        "Half or twice as many pixels",
        Action::Help,
    );
    t.add(
        View::Draw,
        half_right(EDIT_Y),
        STEPPER_TIP,
        Action::Stepper {
            value: "crosshair_res",
            less: "crosshair_res_less",
            more: "crosshair_res_more",
        },
    );
    t.add(
        View::Draw,
        half(DRAW_BUTTONS_Y),
        "Paints all four quarters at once",
        Action::Toggle("crosshair_mirror"),
    );
    t.add(
        View::Draw,
        half_right(DRAW_BUTTONS_Y),
        "Empties the grid; click twice",
        press("crosshair_clear"),
    );
}

/// The Zoom page.
fn zoom(t: &mut Table) {
    // The Zoom page.
    t.add(
        View::Zoom,
        row(ZOOM_ON_Y),
        "Turns the zoom key on or off",
        Action::Toggle("zoom"),
    );
    t.add(
        View::Zoom,
        row(ZOOM_KEY_Y),
        "Click, press a key; Del clears it",
        Action::Key("zoom_key"),
    );
    t.add(
        View::Zoom,
        row(ZOOM_MODE_Y),
        "Hold the key, or press to toggle",
        Action::Toggle("zoom_mode"),
    );
    t.add(
        View::Zoom,
        row(ZOOM_SCROLL_Y),
        "Scroll to zoom; moves hotbar slot too",
        Action::Toggle("zoom_scroll"),
    );
    t.add(
        View::Zoom,
        row(ZOOM_SENS_Y),
        "Look more slowly while zoomed in",
        Action::Toggle("zoom_sens"),
    );
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
        t.add(View::Zoom, value_area(y), value_tip, Action::Help);
        t.add(View::Zoom, pair_area(y), pair_tip, Action::Help);
        t.add(
            View::Zoom,
            row(y),
            STEPPER_TIP,
            Action::Stepper { value, less, more },
        );
    }
    t.add(
        View::Zoom,
        row(ZOOM_RESET_Y),
        "Zoom back to defaults; click twice",
        press("zoom_reset"),
    );
}

/// Back, Close, the title and the bottom line: last, so the keyboard reaches them last.
fn always(t: &mut Table) {
    // Back and Close come last for the keyboard; the title and bottom line are help only.
    t.add(View::Page, BACK_AREA, "Goes back one step", press("back"));
    t.add(
        View::Always,
        CLOSE_AREA,
        "Closes the menu (Esc does too)",
        press("close"),
    );
    t.add(
        View::Always,
        [10.0, 4.0, 150.0, 18.0],
        "Outlines, counter, zoom and fullbright",
        Action::Help,
    );
    t.add(
        View::Always,
        [10.0, CARD_H - 18.0, FULL, 14.0],
        HINT_TIP,
        Action::Hint,
    );
}

mod nav;

pub use nav::*;
