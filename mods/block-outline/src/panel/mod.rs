//! The settings menu: the panel JSON, its live bindings and how its events change settings.
//! Host-independent so `cargo test` can check the documents it produces.

use std::fmt::Write as _;

use crate::outline::{
    FAVOURITES, MAX_WIDTH, MIN_WIDTH, SWATCHES, Settings, hex_code, hsv_to_rgb, parse_color,
    rgb_to_hsv, wheel_colour, wheel_position,
};

mod bindings;
mod controls;
mod events;
mod help;
mod json;
mod layout;
#[cfg(test)]
mod tests;

use bindings::{bindings, bindings_into};
pub use events::{apply, apply_text, commit_typing, type_key};
pub use controls::{Keyed, keys, tip_at};
use controls::{focus_area, settle_focus, tip_area};
use help::focus_bindings;
pub use help::{card_point, focus_update, highlight_update, tip_update};
pub(crate) use json::{digits, fixed, json_string};
use json::vector;
use layout::document;

/// Menu state that is not saved: whether the "Menu key" button is waiting for a key, and
/// whether the colour tools edit the fill instead of the lines.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MenuState {
    pub capturing: bool,
    /// Whether the "Fullbright key" button waits for a key.
    pub capturing_fullbright: bool,
    /// Whether the "Zoom key" button waits for a key.
    pub capturing_zoom: bool,
    pub editing_fill: bool,
    /// Whether the colour tools edit the aimed mob's colour (Mob hitbox page).
    pub editing_aim: bool,
    /// The page shown: the start page, or the Block outline or Mob hitbox settings.
    pub page: Page,
    /// Whether the colour tools are shown, after clicking Line colour or Fill colour.
    pub colour_open: bool,
    /// The box being typed into, if any. The mod reads key presses itself, as for the menu key,
    /// because Cinnabar's native text editor does not open for this menu in game.
    pub typing: Option<Typing>,
    /// The reset button waiting for a second click, if any.
    pub confirm: Reset,
    /// The control the keyboard is on (an index into `controls()`), once Tab or an arrow key
    /// was pressed; a click on the menu clears it.
    pub focus: Option<u8>,
    /// The arrow counter is being dragged and resized: the card hides and a short guide shows.
    pub moving: bool,
    /// The moving guide sits at the bottom of the screen, because the counter is near the top.
    pub guide_bottom: bool,
}

/// The reset buttons, which ask for a second click before they change anything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reset {
    #[default]
    None,
    /// The Block outline or Mob hitbox page's Reset.
    Page,
    Counter,
    Zoom,
}

/// How long a reset button waits for its second click, in seconds.
pub const CONFIRM_SECONDS: f32 = 4.0;

/// The menu's pages. The Block outline and Mob hitbox pages share their controls, which edit
/// the block look or the mob look (`Settings::mob`); the Arrow counter page has its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Home,
    Block,
    Mob,
    Counter,
    Zoom,
}

impl Page {
    /// The Block outline or Mob hitbox page, which share the outline controls.
    pub fn outline(self) -> bool {
        matches!(self, Page::Block | Page::Mob)
    }
}

/// The settings a page's controls show and edit: the mob look in the outline fields on the Mob
/// hitbox page.
fn view(settings: &Settings, state: MenuState) -> Settings {
    if state.page == Page::Mob {
        settings.for_mobs()
    } else {
        *settings
    }
}

/// Whether the shown page's fill matches its line, which decides its Line / Fill buttons.
pub fn fill_matches(settings: &Settings, state: MenuState) -> bool {
    view(settings, state).fill_rgb.is_none()
}

/// Text typed into one box: a slider's number (`field` 0–5) or the hex code (`HEX_FIELD`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Typing {
    pub field: u8,
    text: [u8; 8],
    len: u8,
}

/// The `Typing::field` of the hex code box.
pub const HEX_FIELD: u8 = SLIDERS.len() as u8;

impl Typing {
    fn new(field: u8) -> Self {
        Self {
            field,
            text: [0; 8],
            len: 0,
        }
    }

    pub fn text(&self) -> &str {
        std::str::from_utf8(&self.text[..usize::from(self.len)]).unwrap_or("")
    }

    fn push(&mut self, c: u8) {
        if usize::from(self.len) < self.text.len() {
            self.text[usize::from(self.len)] = c;
            self.len += 1;
        }
    }
}

/// The menu's width: one narrow column at the right edge of the screen, so the middle of the
/// screen, where the outline is, stays clear.
const SETTINGS_W: f32 = 264.0;
/// Card height, the same for the settings and the colour tools so nothing moves when switching.
const CARD_H: f32 = 320.0;
/// Height of every button and small control row; tall enough to hit comfortably.
const BUTTON_H: f32 = 18.0;
/// Vertical distance between two rows of buttons.
const ROW_PITCH: f32 = 24.0;
/// Width of one half of the settings column, for rows holding two buttons.
const HALF: f32 = (SETTINGS_W - 24.0) / 2.0;
/// Slider rows: control id and bar colour.
const SLIDERS: [(&str, [f32; 4]); 6] = [
    ("red", [0.86, 0.24, 0.24, 1.0]),
    ("green", [0.28, 0.78, 0.32, 1.0]),
    ("blue", [0.26, 0.46, 0.92, 1.0]),
    ("opacity", ACCENT),
    ("width", ACCENT),
    ("fill", ACCENT),
];

// Control indices. New controls are appended so existing indices never move.
const RAINBOW: usize = SLIDERS.len();
const FIRST_SWATCH: usize = RAINBOW + 1;
/// Hidden sliders the wheel's `mod.vector` route drives.
const WHEEL_X: usize = FIRST_SWATCH + SWATCHES.len();
const WHEEL_Y: usize = WHEEL_X + 1;
const HEX: usize = WHEEL_Y + 1;
const MENU_KEY: usize = HEX + 1;
const PULSE: usize = MENU_KEY + 1;
const RESET: usize = PULSE + 1;
const EDIT_LINE: usize = RESET + 1;
const EDIT_FILL: usize = EDIT_LINE + 1;
const FILL_MATCH: usize = EDIT_FILL + 1;
const FIRST_FAVOURITE: usize = FILL_MATCH + 1;
const SAVE_FAVOURITE: usize = FIRST_FAVOURITE + FAVOURITES;
/// Buttons that start typing into a slider's number box, then the hex code box.
const FIRST_TYPE: usize = SAVE_FAVOURITE + 1;
const HIDE_HIGHLIGHT: usize = FIRST_TYPE + SLIDERS.len() + 1;
/// Leaves the colour tools for the settings.
const BACK: usize = HIDE_HIGHLIGHT + 1;
/// The start page's buttons.
const OPEN_BLOCK: usize = BACK + 1;
const OPEN_MOB: usize = OPEN_BLOCK + 1;
/// The Mob hitbox page's All mobs switch and Aimed colour button.
const SHOW_ALL: usize = OPEN_MOB + 1;
const EDIT_AIM: usize = SHOW_ALL + 1;
/// The game highlight fix's - and + buttons.
const HIGHLIGHT_LESS: usize = EDIT_AIM + 1;
const HIGHLIGHT_MORE: usize = HIGHLIGHT_LESS + 1;
/// The start page's Fullbright switch.
const FULLBRIGHT: usize = HIGHLIGHT_MORE + 1;
const FULLBRIGHT_KEY: usize = FULLBRIGHT + 1;
/// Closes the menu.
const CLOSE: usize = FULLBRIGHT_KEY + 1;
/// The start page's Arrow counter button and that page's controls.
const OPEN_COUNTER: usize = CLOSE + 1;
const COUNTER: usize = OPEN_COUNTER + 1;
const COUNTER_BOW: usize = COUNTER + 1;
const COUNTER_MOVE: usize = COUNTER_BOW + 1;
/// The size button (back to 100%) and its - and + buttons.
const COUNTER_SIZE: usize = COUNTER_MOVE + 1;
const COUNTER_SMALLER: usize = COUNTER_SIZE + 1;
const COUNTER_BIGGER: usize = COUNTER_SMALLER + 1;
const COUNTER_RESET: usize = COUNTER_BIGGER + 1;
/// Ends moving the counter.
const COUNTER_DONE: usize = COUNTER_RESET + 1;
/// The start page's Zoom button and that page's controls: switches, the zoom key and three
/// values, each a button that resets it beside its - and + buttons.
const OPEN_ZOOM: usize = COUNTER_DONE + 1;
const ZOOM_ON: usize = OPEN_ZOOM + 1;
const ZOOM_MODE: usize = ZOOM_ON + 1;
const ZOOM_SCROLL: usize = ZOOM_MODE + 1;
const ZOOM_SENS: usize = ZOOM_SCROLL + 1;
const ZOOM_KEY: usize = ZOOM_SENS + 1;
const ZOOM_LEVEL: usize = ZOOM_KEY + 1;
const ZOOM_LEVEL_LESS: usize = ZOOM_LEVEL + 1;
const ZOOM_LEVEL_MORE: usize = ZOOM_LEVEL_LESS + 1;
const ZOOM_TIME: usize = ZOOM_LEVEL_MORE + 1;
const ZOOM_TIME_LESS: usize = ZOOM_TIME + 1;
const ZOOM_TIME_MORE: usize = ZOOM_TIME_LESS + 1;
const ZOOM_STEP: usize = ZOOM_TIME_MORE + 1;
const ZOOM_STEP_LESS: usize = ZOOM_STEP + 1;
const ZOOM_STEP_MORE: usize = ZOOM_STEP_LESS + 1;
const ZOOM_RESET: usize = ZOOM_STEP_MORE + 1;
/// The last control index.
#[cfg(test)]
const LAST_CONTROL: usize = ZOOM_RESET;
/// Width of the small - and + buttons.
const STEP_W: f32 = 26.0;
/// The highlight fix's Off / On button, beside its - and + buttons.
const HIGHLIGHT_W: f32 = SETTINGS_W - 20.0 - 2.0 * (STEP_W + 4.0);

// Layout. Settings positions are relative to the settings column, colour positions to the
// colour column.
const EDIT_Y: f32 = 31.0;
/// The start page: the two page buttons and the menu key.
const HOME_BLOCK_Y: f32 = 34.0;
const HOME_MOB_Y: f32 = 71.0;
const HOME_COUNTER_Y: f32 = 108.0;
const HOME_ZOOM_Y: f32 = 145.0;
const HOME_BUTTON_H: f32 = 34.0;
/// Below the page buttons: a Lighting section (Fullbright and its key) and a Menu section.
const HOME_LIGHT_CAPTION_Y: f32 = HOME_ZOOM_Y + HOME_BUTTON_H + 8.0;
const HOME_FULLBRIGHT_Y: f32 = HOME_LIGHT_CAPTION_Y + 14.0;
const HOME_FULLBRIGHT_KEY_Y: f32 = HOME_FULLBRIGHT_Y + ROW_PITCH;
const HOME_MENU_CAPTION_Y: f32 = HOME_FULLBRIGHT_KEY_Y + BUTTON_H + 12.0;
const HOME_KEY_Y: f32 = HOME_MENU_CAPTION_Y + 14.0;
/// The mob page has no Fill row, so its options sit one row higher.
const MOB_OPTIONS_SHIFT: f32 = ROW_STEP;
/// Room below a page's last control for the hint line.
const CARD_FOOT: f32 = 28.0;
/// The Arrow counter page's rows.
const COUNTER_ON_Y: f32 = 34.0;
const COUNTER_BOW_Y: f32 = COUNTER_ON_Y + ROW_PITCH;
const COUNTER_CAPTION_Y: f32 = COUNTER_BOW_Y + ROW_PITCH + SECTION_GAP;
const COUNTER_MOVE_Y: f32 = COUNTER_CAPTION_Y + 14.0;
const COUNTER_SIZE_Y: f32 = COUNTER_MOVE_Y + ROW_PITCH;
const COUNTER_RESET_Y: f32 = COUNTER_SIZE_Y + ROW_PITCH;
/// The Zoom page's rows.
const ZOOM_ON_Y: f32 = 34.0;
const ZOOM_KEY_Y: f32 = ZOOM_ON_Y + ROW_PITCH;
const ZOOM_MODE_Y: f32 = ZOOM_KEY_Y + ROW_PITCH;
const ZOOM_SCROLL_Y: f32 = ZOOM_MODE_Y + ROW_PITCH;
const ZOOM_SENS_Y: f32 = ZOOM_SCROLL_Y + ROW_PITCH;
const ZOOM_CAPTION_Y: f32 = ZOOM_SENS_Y + ROW_PITCH + SECTION_GAP;
const ZOOM_LEVEL_Y: f32 = ZOOM_CAPTION_Y + 14.0;
const ZOOM_TIME_Y: f32 = ZOOM_LEVEL_Y + ROW_PITCH;
const ZOOM_STEP_Y: f32 = ZOOM_TIME_Y + ROW_PITCH;
const ZOOM_RESET_Y: f32 = ZOOM_STEP_Y + ROW_PITCH;
/// The guide shown at the top of the screen while the counter is moved.
const GUIDE_W: f32 = 240.0;
/// Where the guide's text starts, after its accent.
const GUIDE_TEXT_X: f32 = 20.0;
/// What the guide says; each line has to fit the guide's width (about 5.9 units a letter).
const GUIDE_LINES: [&str; 3] = [
    "Drag the counter to move it",
    "Drag any corner square to resize it",
    "Snaps to the hotbar and screen middle",
];
const GUIDE_H: f32 = 68.0;

/// The card's height for what is shown: each page is only as tall as its controls, with the
/// hint line below them; the colour tools use the full height.
pub(super) fn card_height(state: MenuState) -> f32 {
    match state.page {
        _ if state.colour_open && state.page.outline() => CARD_H,
        Page::Home => HOME_KEY_Y + BUTTON_H + CARD_FOOT,
        Page::Counter => COUNTER_RESET_Y + BUTTON_H + CARD_FOOT,
        Page::Zoom => ZOOM_RESET_Y + BUTTON_H + CARD_FOOT,
        Page::Block => HIGHLIGHT_Y + BUTTON_H + CARD_FOOT,
        Page::Mob => BOTTOM_Y - MOB_OPTIONS_SHIFT + BUTTON_H + CARD_FOOT,
    }
}
/// The colour view's Back button, top right.
/// The close button, top right on every page, and Back beside it.
const CLOSE_AREA: [f32; 4] = [SETTINGS_W - 30.0, 5.0, 20.0, BUTTON_H];
const BACK_AREA: [f32; 4] = [SETTINGS_W - 84.0, 5.0, 50.0, BUTTON_H];
/// Section captions of the settings view, above the bars and the options.
const LOOK_CAPTION_Y: f32 = 55.0;
const ROW_Y: f32 = 69.0;
const ROW_STEP: f32 = 24.0;
const ROW_HEIGHT: f32 = BUTTON_H;
const OPTIONS_CAPTION_Y: f32 = ROW_Y + ROW_STEP * 3.0 + SECTION_GAP;
const TOGGLES_Y: f32 = OPTIONS_CAPTION_Y + 14.0;
const BOTTOM_Y: f32 = TOGGLES_Y + ROW_PITCH;
const KEY_Y: f32 = BOTTOM_Y + ROW_PITCH;
/// The block page's game highlight fix, where the menu key was before it moved to the start.
const HIGHLIGHT_Y: f32 = KEY_Y;
/// Slider rows: name, typed number box and bar. The colour column's bars are shorter.
const VALUE_X: f32 = 66.0;
/// Where the button of a "label, then key button" row starts: room for "Fullbright key".
const KEY_X: f32 = 92.0;
const VALUE_WIDTH: f32 = 36.0;
const TRACK_X: f32 = 108.0;
const fn track_width() -> f32 {
    SETTINGS_W - 10.0 - TRACK_X
}
/// The colour tools, below the Line / Fill colour buttons in place of the settings.
const RGB_Y: f32 = 54.0;
/// Wheel square's top-left corner and side, and the size of its colour cells.
const WHEEL: [f32; 3] = [10.0, 122.0, 90.0];
const WHEEL_CELL: f32 = 3.0;
const MARKER: f32 = 7.0;
const HEX_X: f32 = WHEEL[0] + WHEEL[2] + 10.0;
const HEX_Y: f32 = 124.0;
const SWATCH_Y: f32 = 224.0;
const FAVOURITE_Y: f32 = 264.0;

const TEXT: [f32; 3] = [0.92, 0.93, 0.95];
const DIM: [f32; 3] = [0.74, 0.76, 0.8];
const TRACK: [f32; 4] = [0.235, 0.245, 0.29, 1.0];
/// The "on" colour: dark enough for the light text on it to pass WCAG AA (4.5:1), see the test.
const ACTIVE: [f32; 4] = [0.45, 0.24, 0.75, 1.0];
/// A reset button waiting for its second click.
const DANGER: [f32; 4] = [0.62, 0.2, 0.18, 1.0];
/// A switch's pill and knob, in panel units; the knob's left edge is `KNOB_OFF` off, and
/// `PILL_W - KNOB - KNOB_OFF` on.
const PILL_W: f32 = 28.0;
const PILL_H: f32 = 14.0;
const KNOB: f32 = 10.0;
const KNOB_OFF: f32 = 2.0;
/// A slider's thin bar, its round knob and the bar's empty colour.
const TRACK_H: f32 = 6.0;
const SLIDER_KNOB: f32 = 12.0;
const SLIDER_TRACK: [f32; 4] = [0.22, 0.23, 0.28, 1.0];
/// The switches: the pill's colour while off (on is `ACTIVE`), and the header accent.
const SWITCH_OFF: [f32; 4] = [0.43, 0.44, 0.52, 1.0];
/// What each start page button says under its name (Block, Mob, Counter, Zoom).
const HOME_ABOUT: [&str; 4] = [
    "The block you are looking at",
    "Mobs and players you look at",
    "Arrows left, shown on your HUD",
    "Hold a key, scroll to zoom in",
];
/// Settings sit in lighter inset panels, `SECTION_PAD` wider than their rows; the gap between two
/// panels is `SECTION_GAP` more than the rows' own spacing.
const SECTION: [f32; 4] = [0.135, 0.145, 0.185, 1.0];
const SECTION_PAD: f32 = 5.0;
const SECTION_GAP: f32 = 6.0;
/// Section captions are small capitals in a soft accent; the title is drawn this much larger.
const CAPTION_TEXT: [f32; 3] = [0.72, 0.66, 0.94];
const TITLE_SCALE: f32 = 1.25;
/// The tile behind a home button's icon, and the bottom lip that makes a key look like a key.
const ICON_BG: [f32; 4] = [0.12, 0.13, 0.17, 1.0];
const KEY_LIP: [f32; 4] = [0.1, 0.11, 0.14, 1.0];
const KEY_LIP_H: f32 = 2.0;
/// The home buttons' badge, and the room the header accent takes before the title.
const BADGE: f32 = 22.0;
const TITLE_X: f32 = 8.0;
const ACCENT: [f32; 4] = [0.66, 0.5, 0.97, 1.0];
/// The switch rows: bound by `#<name>_bg` (pill colour) and `#<name>_knob` (knob position).
const SWITCHES: [&str; 11] = [
    "rainbow", "pulse", "fill_match", "show_all_mobs", "fullbright", "counter", "counter_bow",
    "zoom", "zoom_mode", "zoom_scroll", "zoom_sens",
];
/// The card's outline and the keyboard focus ring.
const BORDER: [f32; 4] = [0.3, 0.31, 0.37, 1.0];
/// The keyboard focus frame: a thin outline `FOCUS_GAP` outside the control, and a faint tint
/// inside it that covers nothing.
const FOCUS: [f32; 4] = [0.72, 0.62, 0.97, 0.85];
const FOCUS_TINT: [f32; 4] = [0.78, 0.66, 1.0, 0.12];
const FOCUS_GAP: f32 = 2.0;
const FOCUS_LINE: f32 = 1.5;
const EMPTY_SLOT: [f32; 4] = [0.12, 0.13, 0.15, 1.0];
/// Gap between the card and the right edge of the screen.
const SCREEN_MARGIN: f32 = 12.0;
const NUMBER_BOX: [f32; 4] = [0.2, 0.21, 0.26, 1.0];
/// Thin dividers: under the title and beside section captions.
const RULE: [f32; 4] = [0.26, 0.27, 0.31, 1.0];

/// The panel bundle the packer embeds in the component, so the host loads the menu itself:
/// building the whole document inside the guest costs more than a callback's fuel.
pub fn bundle() -> String {
    format!(
        r#"{{"panel":{},"surfaces":{{}}}}"#,
        content(&Settings::default(), 0.0)
    )
}

/// The whole panel for `settings`, as `panel.set-content` takes it. Build-time only.
pub fn content(settings: &Settings, seconds: f32) -> String {
    content_in(settings, seconds, MenuState::default())
}

/// [`content`] showing `state`, for rendering other pages in tests.
fn content_in(settings: &Settings, seconds: f32, state: MenuState) -> String {
    let mut controls = String::new();
    for (index, (id, _)) in SLIDERS.iter().enumerate() {
        let (min, max, step) = slider_range(index);
        let value = slider_value(settings, state, index);
        let _ = write!(
            controls,
            r#"{{"kind":"slider","id":"{id}","label":"{}","value":{value},"min":{min},"max":{max},"step":{step}}},"#,
            title(id)
        );
    }
    let _ = write!(
        controls,
        r#"{{"kind":"toggle","id":"rainbow","label":"Rainbow","value":{}}}"#,
        settings.rainbow
    );
    for (name, _) in SWATCHES {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"swatch_{}","label":"{name}"}}"#,
            name.to_lowercase()
        );
    }
    let [x, y] = wheel_position(target_rgb(settings, state));
    let _ = write!(
        controls,
        r#",{{"kind":"slider","id":"wheel_x","label":"Wheel","value":{x},"min":0,"max":1,"step":0.001}},{{"kind":"slider","id":"wheel_y","label":"Wheel","value":{y},"min":0,"max":1,"step":0.001}},{{"kind":"text","id":"hex","label":"Hex","value":"{}","min_chars":0,"max_chars":9}}"#,
        target_hex(settings, state)
    );
    let _ = write!(
        controls,
        r#",{{"kind":"keybind","id":"menu_key","label":"Menu key","key":"{}"}}"#,
        settings.menu_key.as_str()
    );
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"pulse","label":"Pulse","value":{}}},{{"kind":"button","id":"reset","label":"Reset page"}},{{"kind":"button","id":"edit_line","label":"Line colour"}},{{"kind":"button","id":"edit_fill","label":"Fill colour"}},{{"kind":"toggle","id":"fill_match","label":"Match fill to line","value":{}}}"#,
        settings.pulse,
        settings.fill_rgb.is_none()
    );
    for slot in 0..FAVOURITES {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"favourite_{slot}","label":"Saved colour {}"}}"#,
            slot + 1
        );
    }
    controls.push_str(r#",{"kind":"button","id":"save_favourite","label":"Save colour"}"#);
    for (id, _) in SLIDERS {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"type_{id}","label":"Type {}"}}"#,
            title(id)
        );
    }
    controls.push_str(r#",{"kind":"button","id":"type_hex","label":"Type hex code"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"button","id":"hide_game_highlight","label":"Hide game highlight: {}%"}}"#,
        settings.highlight_cancel
    );
    controls.push_str(r#",{"kind":"button","id":"back","label":"Back to settings"}"#);
    controls.push_str(r#",{"kind":"button","id":"open_block","label":"Block outline"}"#);
    controls.push_str(r#",{"kind":"button","id":"open_mob","label":"Mob hitbox"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"show_all_mobs","label":"All mobs","value":{}}}"#,
        settings.show_all_mobs
    );
    controls.push_str(r#",{"kind":"button","id":"edit_aim","label":"Aimed colour"}"#);
    controls.push_str(r#",{"kind":"button","id":"highlight_less","label":"Weaker highlight hiding"}"#);
    controls
        .push_str(r#",{"kind":"button","id":"highlight_more","label":"Stronger highlight hiding"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"fullbright","label":"Fullbright","value":{}}}"#,
        settings.fullbright
    );
    controls.push_str(r#",{"kind":"button","id":"fullbright_key","label":"Fullbright key"}"#);
    controls.push_str(r#",{"kind":"button","id":"close","label":"Close menu"}"#);
    controls.push_str(r#",{"kind":"button","id":"open_counter","label":"Arrow counter"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"counter","label":"Arrow counter","value":{}}},{{"kind":"toggle","id":"counter_bow","label":"Only with a bow","value":{}}}"#,
        settings.counter.on, settings.counter.bow_only
    );
    controls.push_str(r#",{"kind":"button","id":"counter_move","label":"Move and resize"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_size","label":"Counter size"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_smaller","label":"Smaller counter"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_bigger","label":"Bigger counter"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_reset","label":"Reset counter place"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_done","label":"Done moving"}"#);
    controls.push_str(r#",{"kind":"button","id":"open_zoom","label":"Zoom"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"zoom","label":"Zoom","value":{}}},{{"kind":"toggle","id":"zoom_mode","label":"Zoom mode","value":{}}},{{"kind":"toggle","id":"zoom_scroll","label":"Scroll zoom","value":{}}},{{"kind":"toggle","id":"zoom_sens","label":"Relative sensitivity","value":{}}}"#,
        settings.zoom.on, settings.zoom.toggle, settings.zoom.scroll, settings.zoom.sensitivity
    );
    for (id, label) in [
        ("zoom_key", "Zoom key"),
        ("zoom_level", "Zoom level"),
        ("zoom_level_less", "Smaller zoom"),
        ("zoom_level_more", "Bigger zoom"),
        ("zoom_time", "Zoom time"),
        ("zoom_time_less", "Faster zoom"),
        ("zoom_time_more", "Slower zoom"),
        ("zoom_step", "Scroll step"),
        ("zoom_step_less", "Smaller scroll step"),
        ("zoom_step_more", "Bigger scroll step"),
        ("zoom_reset", "Reset zoom"),
    ] {
        let _ = write!(controls, r#",{{"kind":"button","id":"{id}","label":"{label}"}}"#);
    }
    format!(
        r#"{{"reference_size":[{},{}],"motion":true,"widget_colors":{{"background":[0.07,0.08,0.1,0.98],"surface":[0.2,0.21,0.24,1],"text":[0.92,0.93,0.95,1],"accent":[0.45,0.24,0.75,1],"border":[0.36,0.37,0.42,1]}},"controls":[{controls}],"surface":{{"screen":"block_outline.screen","document":{},"bindings":{}}}}}"#,
        // Shrinks the whole menu on windows too small for the card, with a small margin.
        SETTINGS_W + 16.0,
        CARD_H + 16.0,
        json_string(&document()),
        bindings(settings, seconds, state)
    )
}

/// A patch moving every control and label to `settings`. Runs in frame callbacks, so it
/// formats numbers with [`fixed`] rather than the much costlier float formatting.
pub fn update(settings: &Settings, seconds: f32, state: MenuState) -> String {
    let shown = view(settings, state);
    let settings = &shown;
    let mut out = String::with_capacity(1536);
    out.push_str(r#"{"controls":[{"id":"menu_key","key":""#);
    out.push_str(settings.menu_key.as_str());
    out.push_str(if state.capturing {
        r#"","capturing":true}],"#
    } else {
        r#"","capturing":false}],"#
    });
    out.push_str(
        // Escape ends moving the counter instead of closing the menu.
        if state.capturing
            || state.capturing_fullbright
            || state.capturing_zoom
            || state.typing.is_some()
            || state.moving
        {
            r#""capture_key":true,"#
        } else {
            r#""capture_key":false,"#
        },
    );
    // The arrow counter lives on the HUD, which the menu hides on its other pages.
    out.push_str(if state.page == Page::Counter || state.moving {
        r#""show_gameplay_hud":true,"#
    } else {
        r#""show_gameplay_hud":false,"#
    });
    out.push_str(r#""control_values":["#);
    let value = |out: &mut String, index: usize, write: &dyn Fn(&mut String)| {
        if !out.ends_with('[') {
            out.push(',');
        }
        out.push('[');
        fixed(out, index as f32);
        out.push(',');
        write(out);
        out.push(']');
    };
    for index in 0..SLIDERS.len() {
        let v = slider_value(settings, state, index);
        value(&mut out, index, &|out| fixed(out, v));
    }
    let boolean =
        |on: bool| move |out: &mut String| out.push_str(if on { "true" } else { "false" });
    value(&mut out, RAINBOW, &boolean(settings.rainbow));
    let wheel = wheel_position(target_rgb(settings, state));
    value(&mut out, WHEEL_X, &|out| fixed(out, wheel[0]));
    value(&mut out, WHEEL_Y, &|out| fixed(out, wheel[1]));
    let hex = target_hex(settings, state);
    value(&mut out, HEX, &|out| {
        out.push('"');
        out.push_str(&hex);
        out.push('"');
    });
    value(&mut out, PULSE, &boolean(settings.pulse));
    value(&mut out, FILL_MATCH, &boolean(settings.fill_rgb.is_none()));
    value(&mut out, SHOW_ALL, &boolean(settings.show_all_mobs));
    value(&mut out, FULLBRIGHT, &boolean(settings.fullbright));
    value(&mut out, COUNTER, &boolean(settings.counter.on));
    value(&mut out, COUNTER_BOW, &boolean(settings.counter.bow_only));
    value(&mut out, ZOOM_ON, &boolean(settings.zoom.on));
    value(&mut out, ZOOM_MODE, &boolean(settings.zoom.toggle));
    value(&mut out, ZOOM_SCROLL, &boolean(settings.zoom.scroll));
    value(&mut out, ZOOM_SENS, &boolean(settings.zoom.sensitivity));
    out.push_str(r#"],"binding_changes":"#);
    bindings_into(&mut out, settings, seconds, state);
    out.push('}');
    out
}

/// A patch of only the preview colour, for animating the rainbow while the menu is open.
pub fn preview_update(settings: &Settings, seconds: f32, state: MenuState) -> String {
    let mut out = String::with_capacity(96);
    let shown = view(settings, state);
    let settings = &shown;
    out.push_str(r##"{"binding_changes":{"##);
    swatches(&mut out, settings, seconds);
    out.push_str("}}");
    out
}

/// Appends the line and fill swatch colours shown inside the Line / Fill colour buttons.
fn swatches(out: &mut String, settings: &Settings, seconds: f32) {
    out.push_str(r##""#line_swatch":"##);
    let [r, g, b, _] = settings.srgb(seconds);
    vector(out, [r, g, b, 1.0]);
    out.push_str(r##","#fill_swatch":"##);
    let [r, g, b] = settings.fill_srgb(seconds);
    vector(out, [r, g, b, 1.0]);
    out.push_str(r##","#open_block_color":"##);
    let [r, g, b, _] = settings.srgb(seconds);
    vector(out, [r, g, b, 1.0]);
    out.push_str(r##","#open_mob_color":"##);
    let [r, g, b, _] = settings.for_mobs().srgb(seconds);
    vector(out, [r, g, b, 1.0]);
    out.push_str(r##","#aim_swatch":"##);
    let [r, g, b] = settings.aim_rgb.map(|c| f32::from(c) / 255.0);
    vector(out, [r, g, b, 1.0]);
}

/// Whether the colour tools' hex code and saved colours carry the line opacity: only for the
/// line, the colour that owns it.
fn edits_alpha(state: MenuState) -> bool {
    !state.editing_fill && !state.editing_aim
}

/// The colour the colour tools are editing: the fill's own colour, or the line colour.
fn target_rgb(settings: &Settings, state: MenuState) -> [u8; 3] {
    if state.editing_aim {
        return settings.aim_rgb;
    }
    let line = [settings.rgba[0], settings.rgba[1], settings.rgba[2]];
    if state.editing_fill {
        settings.fill_rgb.unwrap_or(line)
    } else {
        line
    }
}

fn target_hex(settings: &Settings, state: MenuState) -> String {
    if !edits_alpha(state) {
        let [r, g, b] = target_rgb(settings, state);
        hex_code([r, g, b, 255])
    } else {
        hex_code(settings.rgba)
    }
}

/// Sets the edited colour. Editing the fill gives it its own colour; editing the line turns
/// rainbow off, since a fixed colour was chosen.
fn set_target_rgb(settings: &mut Settings, state: MenuState, rgb: [u8; 3]) {
    if state.editing_aim {
        settings.aim_rgb = rgb;
        return;
    }
    if state.editing_fill {
        settings.fill_rgb = Some(rgb);
    } else {
        settings.rgba = [rgb[0], rgb[1], rgb[2], settings.rgba[3]];
        settings.rainbow = false;
    }
}

/// Whether slider `index` (red, green, blue) belongs to the colour tools, and its row's top.
fn slider_place(index: usize) -> (bool, f32) {
    if index < 3 {
        (true, RGB_Y + (ROW_STEP - 2.0) * index as f32)
    } else {
        (false, ROW_Y + ROW_STEP * (index - 3) as f32)
    }
}

fn slider_range(index: usize) -> (f32, f32, f32) {
    match index {
        0..=2 => (0.0, 255.0, 1.0),
        3 => (0.0, 100.0, 1.0),
        4 => (MIN_WIDTH, MAX_WIDTH, 0.5),
        _ => (0.0, 100.0, 1.0),
    }
}

fn slider_value(settings: &Settings, state: MenuState, index: usize) -> f32 {
    match index {
        0..=2 => f32::from(target_rgb(settings, state)[index]),
        3 => (f32::from(settings.rgba[3]) / 2.55).round(),
        4 => settings.width,
        _ => f32::from(settings.fill),
    }
}

fn title(id: &str) -> &'static str {
    match id {
        "red" => "Red",
        "green" => "Green",
        "blue" => "Blue",
        "opacity" => "Opacity",
        "width" => "Thickness",
        _ => "Fill",
    }
}

/// A patch showing the moving guide at the top or the bottom of the screen.
pub fn guide_update(state: MenuState) -> String {
    let (top, bottom) = (state.moving && !state.guide_bottom, state.moving && state.guide_bottom);
    format!(r##"{{"binding_changes":{{"#guide_top_visible":{top},"#guide_bottom_visible":{bottom}}}}}"##)
}
