//! The settings menu: the panel JSON, its live bindings and how its events change settings.
//! Host-independent so `cargo test` can check the documents it produces.

use std::fmt::Write as _;

use crate::outline::{
    FAVOURITES, MAX_WIDTH, MIN_WIDTH, SWATCHES, Settings, hex_code, hsv_to_rgb, parse_color,
    rgb_to_hsv, wheel_colour, wheel_position,
};

mod bindings;
mod events;
mod help;
mod json;
mod layout;
#[cfg(test)]
mod tests;

use bindings::{bindings, bindings_into};
pub use events::{apply, apply_text, commit_typing, type_key};
pub use help::{card_point, card_width, highlight_update, tip_at, tip_update};
use json::{digits, fixed, json_string, vector};
use layout::document;

/// The panel's own toggle key, which cannot change after load. A key no keyboard has, so the
/// mod opens and closes the menu itself with the player's chosen key (`Settings::menu_key`).
/// Cinnabar's fallback F10 and Escape still work as usual.
pub const TOGGLE_KEY: &str = "F24";

/// Menu state that is not saved: whether the "Menu key" button is waiting for a key, and
/// whether the colour tools edit the fill instead of the lines.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MenuState {
    pub capturing: bool,
    /// Whether the "Fullbright key" button waits for a key.
    pub capturing_fullbright: bool,
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
}

/// The menu's pages. The Block outline and Mob hitbox pages share their controls, which edit
/// the block look or the mob look (`Settings::mob`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Home,
    Block,
    Mob,
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
const SETTINGS_W: f32 = 244.0;
/// Card height, the same for the settings and the colour tools so nothing moves when switching.
const CARD_H: f32 = 304.0;
/// Width of one half of the settings column, for rows holding two buttons.
const HALF: f32 = (SETTINGS_W - 24.0) / 2.0;
/// Slider rows: control id and bar colour.
const SLIDERS: [(&str, [f32; 4]); 6] = [
    ("red", [0.86, 0.24, 0.24, 1.0]),
    ("green", [0.28, 0.78, 0.32, 1.0]),
    ("blue", [0.26, 0.46, 0.92, 1.0]),
    ("opacity", [0.75, 0.75, 0.78, 1.0]),
    ("width", [0.95, 0.66, 0.2, 1.0]),
    ("fill", [0.55, 0.75, 0.95, 1.0]),
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
/// Width of the small - and + buttons.
const STEP_W: f32 = 20.0;
/// The highlight fix's Off / On button, beside its - and + buttons.
const HIGHLIGHT_W: f32 = SETTINGS_W - 20.0 - 2.0 * (STEP_W + 4.0);

// Layout. Settings positions are relative to the settings column, colour positions to the
// colour column.
const EDIT_Y: f32 = 30.0;
/// The start page: the two page buttons and the menu key.
const HOME_BLOCK_Y: f32 = 34.0;
const HOME_MOB_Y: f32 = 76.0;
const HOME_BUTTON_H: f32 = 34.0;
const HOME_KEY_Y: f32 = 136.0;
const HOME_FULLBRIGHT_Y: f32 = HOME_KEY_Y + 22.0;
const HOME_FULLBRIGHT_KEY_Y: f32 = HOME_FULLBRIGHT_Y + 22.0;
/// The mob page has no Fill row, so its options sit one row higher.
const MOB_OPTIONS_SHIFT: f32 = ROW_STEP;
/// Room below a page's last control for the hint line.
const CARD_FOOT: f32 = 32.0;

/// The card's height for what is shown: each page is only as tall as its controls, with the
/// hint line below them; the colour tools use the full height.
pub(super) fn card_height(state: MenuState) -> f32 {
    match state.page {
        _ if state.colour_open && state.page != Page::Home => CARD_H,
        Page::Home => HOME_FULLBRIGHT_KEY_Y + 15.0 + CARD_FOOT,
        Page::Block => HIGHLIGHT_Y + 15.0 + CARD_FOOT,
        Page::Mob => BOTTOM_Y - MOB_OPTIONS_SHIFT + 15.0 + CARD_FOOT,
    }
}
/// The colour view's Back button, top right.
/// The close button, top right on every page, and Back beside it.
const CLOSE_AREA: [f32; 4] = [SETTINGS_W - 28.0, 6.0, 18.0, 16.0];
const BACK_AREA: [f32; 4] = [SETTINGS_W - 80.0, 6.0, 48.0, 16.0];
/// Section captions of the settings view, above the bars and the options.
const LOOK_CAPTION_Y: f32 = 52.0;
const ROW_Y: f32 = 66.0;
const ROW_STEP: f32 = 24.0;
const ROW_HEIGHT: f32 = 13.0;
const OPTIONS_CAPTION_Y: f32 = ROW_Y + ROW_STEP * 3.0;
const TOGGLES_Y: f32 = OPTIONS_CAPTION_Y + 14.0;
const BOTTOM_Y: f32 = TOGGLES_Y + 24.0;
const KEY_Y: f32 = BOTTOM_Y + 24.0;
/// The block page's game highlight fix, where the menu key was before it moved to the start.
const HIGHLIGHT_Y: f32 = KEY_Y;
/// Slider rows: name, typed number box and bar. The colour column's bars are shorter.
const VALUE_X: f32 = 66.0;
const VALUE_WIDTH: f32 = 36.0;
const TRACK_X: f32 = 108.0;
const fn track_width() -> f32 {
    SETTINGS_W - 10.0 - TRACK_X
}
/// The colour tools, below the Line / Fill colour buttons in place of the settings.
const RGB_Y: f32 = 54.0;
/// Wheel square's top-left corner and side, and the size of its colour cells.
const WHEEL: [f32; 3] = [10.0, 122.0, 84.0];
const WHEEL_CELL: f32 = 3.0;
const MARKER: f32 = 7.0;
const HEX_X: f32 = WHEEL[0] + WHEEL[2] + 10.0;
const HEX_Y: f32 = 124.0;
const SWATCH_Y: f32 = 214.0;
const FAVOURITE_Y: f32 = 250.0;

const TEXT: [f32; 3] = [0.92, 0.93, 0.95];
const DIM: [f32; 3] = [0.62, 0.64, 0.68];
const TRACK: [f32; 4] = [0.2, 0.21, 0.24, 1.0];
const ACTIVE: [f32; 4] = [0.55, 0.3, 0.85, 1.0];
const EMPTY_SLOT: [f32; 4] = [0.12, 0.13, 0.15, 1.0];
/// Gap between the card and the right edge of the screen.
const SCREEN_MARGIN: f32 = 12.0;
const NUMBER_BOX: [f32; 4] = [0.13, 0.14, 0.17, 1.0];
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
    let state = MenuState::default();
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
        r#",{{"kind":"toggle","id":"pulse","label":"Pulse","value":{}}},{{"kind":"button","id":"reset","label":"Reset"}},{{"kind":"button","id":"edit_line","label":"Line colour"}},{{"kind":"button","id":"edit_fill","label":"Fill colour"}},{{"kind":"toggle","id":"fill_match","label":"Fill matches line","value":{}}}"#,
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
        r#",{{"kind":"button","id":"hide_game_highlight","label":"Game highlight fix: {}%"}}"#,
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
    controls.push_str(r#",{"kind":"button","id":"highlight_less","label":"Weaker highlight fix"}"#);
    controls
        .push_str(r#",{"kind":"button","id":"highlight_more","label":"Stronger highlight fix"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"fullbright","label":"Fullbright","value":{}}}"#,
        settings.fullbright
    );
    controls.push_str(r#",{"kind":"button","id":"fullbright_key","label":"Fullbright key"}"#);
    controls.push_str(r#",{"kind":"button","id":"close","label":"Close menu"}"#);
    format!(
        r#"{{"toggle_key":"{TOGGLE_KEY}","reference_size":[{},{}],"motion":true,"widget_colors":{{"background":[0.07,0.08,0.1,0.98],"surface":[0.2,0.21,0.24,1],"text":[0.92,0.93,0.95,1],"accent":[0.55,0.3,0.85,1],"border":[0.36,0.37,0.42,1]}},"controls":[{controls}],"surface":{{"screen":"block_outline.screen","document":{},"bindings":{}}}}}"#,
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
        if state.capturing || state.capturing_fullbright || state.typing.is_some() {
            r#""capture_key":true,"#
        } else {
            r#""capture_key":false,"#
        },
    );
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
