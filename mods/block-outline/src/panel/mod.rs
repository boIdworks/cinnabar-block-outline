//! The settings menu: the panel JSON, its live bindings and how its events change settings.
//! Host-independent so `cargo test` can check the documents it produces.

use std::fmt::Write as _;

use crate::outline::{
    FAVOURITES, MAX_WIDTH, MIN_WIDTH, SWATCHES, Settings, hex_code, hsv_to_rgb, parse_color,
    rgb_to_hsv, wheel_colour, wheel_position,
};

mod bindings;
mod content;
mod controls;
mod events;
mod help;
mod ids;
mod json;
mod layout;
mod metrics;

pub use content::*;
use ids::*;
pub use metrics::*;

#[cfg(test)]
mod tests;

use bindings::{bindings, bindings_into};
pub use controls::{Keyed, keys, tip_at};
use controls::{focus_area, settle_focus, tip_area};
pub use events::{apply, apply_text, commit_typing, type_key};
use help::focus_bindings;
pub use help::{card_point, focus_update, highlight_update, tip_update};
use json::vector;
pub(crate) use json::{digits, fixed, json_string};
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
    /// The HUD (the arrow counter and the CPS readout) is being dragged and resized: the card hides and a short guide shows.
    pub moving: bool,
    /// Which element the keys and Reset act on while moving: the CPS readout (true) or the
    /// arrow counter (false), whichever was grabbed last.
    pub moving_cps: bool,
    /// The same for the FPS counter; it wins over `moving_cps` when both are set.
    pub moving_fps: bool,
    /// The moving guide sits at the bottom of the screen, because the counter is near the top.
    pub guide_bottom: bool,
    /// Fuel the last frame used and the busiest frame used, in thousands (of 200): shown at the
    /// foot of the start page.
    pub fuel_k: [u16; 2],
    /// The Crosshair page shows its drawing grid in place of its settings.
    pub drawing: bool,
    /// The last pixel drawn was refused because the drawing would be too detailed to send.
    pub draw_full: bool,
}

/// The reset buttons, which ask for a second click before they change anything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reset {
    #[default]
    None,
    /// The Block outline or Mob hitbox page's Reset.
    Page,
    Counter,
    Cps,
    Fps,
    Zoom,
    Crosshair,
    /// The drawing grid's Clear.
    Drawing,
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
    Cps,
    Fps,
    Zoom,
    Lighting,
    JumpReset,
    Crosshair,
}

impl Page {
    /// The Block outline or Mob hitbox page, which share the outline controls.
    pub fn outline(self) -> bool {
        matches!(self, Page::Block | Page::Mob)
    }

    /// A page with a colour to edit, which the colour tools can replace: the outline pages and
    /// the Crosshair page.
    pub fn has_colour(self) -> bool {
        self.outline() || self == Page::Crosshair
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
