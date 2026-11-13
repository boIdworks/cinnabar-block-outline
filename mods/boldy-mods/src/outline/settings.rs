//! Player-editable settings and how they are read from and written to the settings file.

use super::*;

/// Player-editable settings, persisted as the mod's companion settings JSON.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// sRGB colour and opacity, written as `#RRGGBBAA`.
    pub rgba: [u8; 4],
    /// Cycles the hue over time instead of using `rgba`'s colour; opacity still applies.
    pub rainbow: bool,
    /// Line width in physical pixels.
    pub width: f32,
    /// Opacity, 0–100, of a fill in the outline colour over the block's visible faces; 0 is off.
    pub fill: u8,
    /// Maximum targeting distance from the eye, in blocks.
    pub reach: f32,
    /// Physical key that opens and closes the F8 menu.
    pub menu_key: KeyName,
    /// sRGB colour of the face fill; `None` follows the line colour (rainbow included).
    pub fill_rgb: Option<[u8; 3]>,
    /// Gently fades the outline in and out.
    pub pulse: bool,
    /// How strongly to cancel Cinnabar's light face highlight (its Outline Selection off mode)
    /// on the block it selects, in percent (0 = off, steps of `HIGHLIGHT_CANCEL_STEP`): those
    /// faces are darkened first by up to the factor the highlight brightens them.
    pub highlight_cancel: u8,
    /// How mob hitboxes are drawn, set on the menu's Mob hitbox page. The fields above with
    /// the same names are the block outline's.
    pub mob: Style,
    /// Cinnabar's local fullbright lighting: everything lit as in daylight, on this screen only.
    pub fullbright: bool,
    /// A key that switches fullbright on and off in game, if one is set.
    pub fullbright_key: Option<KeyName>,
    /// Queue the next game by itself when a Hive game ends (needs the commands permission).
    pub auto_queue: bool,
    /// Outline every nearby mob, not only the one aimed at.
    pub show_all_mobs: bool,
    /// Line colour of the mob aimed at while every nearby mob is outlined.
    pub aim_rgb: [u8; 3],
    /// Independent colour for arrow projectiles.
    pub arrow_rgb: [u8; 3],
    /// Saved colours, filled by the menu's Save button.
    pub favourites: [Option<[u8; 4]>; FAVOURITES],
    /// The slot the next Save writes, oldest first.
    pub next_favourite: u8,
    /// The HUD arrow counter, set on the menu's Arrow counter page.
    pub counter: crate::counter::Counter,
    pub armor: crate::armor::Armor,
    /// Zoom, set on the menu's Zoom page.
    pub zoom: crate::zoom::Zoom,
    /// The clicks-per-second readout, set on the menu's Arrow counter page.
    pub cps: crate::cps::Cps,
    pub fps: crate::fps::Fps,
    /// The jump reset timing readout, set on the menu's Jump reset page.
    pub jump_reset: crate::jumpreset::JumpReset,
    /// The custom crosshair, set on the menu's Crosshair page.
    pub crosshair: crate::crosshair::Crosshair,
    /// Whether the block outline is drawn; switched on the menu's start page.
    pub block_on: bool,
    /// Whether mob hitboxes are drawn; switched on the menu's start page.
    pub mob_on: bool,
    /// Hide Cinnabar's hunger row.
    pub hide_hunger: bool,
    /// Show the remaining time under each status-effect icon.
    pub effect_timers: bool,
    /// Show the key bound to each hotbar slot.
    pub hotbar_keys: bool,
}

/// The aimed mob's default colour: a clear red.
pub const DEFAULT_AIM: [u8; 3] = [255, 60, 60];

/// The look of one kind of outline: blocks use the top-level settings, mobs `Settings::mob`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub rgba: [u8; 4],
    pub rainbow: bool,
    pub width: f32,
    pub fill: u8,
    pub fill_rgb: Option<[u8; 3]>,
    pub pulse: bool,
}

impl Style {
    pub const DEFAULT: Self = Self {
        rgba: [255, 255, 255, 255],
        rainbow: false,
        width: 3.0,
        fill: 0,
        fill_rgb: None,
        pulse: false,
    };
}

/// How much Cinnabar's selection highlight brightens a face: it draws a fixed grey of 0.65
/// (`HIGHLIGHT_COLOR`) with a `Dst * Src + Src * Dst` blend, which multiplies by 2 × 0.65.
pub const GAME_HIGHLIGHT_GAIN: f32 = 1.3;

/// Steps of the menu's highlight fix - / + buttons, in percent.
pub const HIGHLIGHT_CANCEL_STEP: u8 = 5;
/// The highlight fix strength that matched an untouched block best in game.
pub const DEFAULT_HIGHLIGHT_CANCEL: u8 = 50;

/// Opacity of a black layer that undoes `percent` of the game highlight's gain: at 100,
/// `(1 - a) × 1.3 = 1`.
pub fn highlight_cancel_alpha(percent: u8) -> f32 {
    let gain = 1.0 + (GAME_HIGHLIGHT_GAIN - 1.0) * f32::from(percent.min(100)) / 100.0;
    1.0 - 1.0 / gain
}

/// The highlight fix strength one step stronger (`up`) or weaker, within 0–100 and on a step.
pub fn step_highlight_cancel(percent: u8, up: bool) -> u8 {
    let step = HIGHLIGHT_CANCEL_STEP;
    let snapped = percent.min(100) / step * step;
    if up {
        (snapped + step).min(100)
    } else if snapped < percent {
        snapped
    } else {
        snapped.saturating_sub(step)
    }
}

/// Number of favourite colour slots.
pub const FAVOURITES: usize = 4;

/// A physical key name as Cinnabar's controls report it (`F8`, `KeyE`, `Digit1`), stored
/// inline so [`Settings`] stays `Copy`. At most 32 ASCII letters and digits.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeyName {
    bytes: [u8; 32],
    len: u8,
}

impl KeyName {
    pub const DEFAULT: Self = Self::literal("F8");

    const fn literal(name: &str) -> Self {
        let mut bytes = [0; 32];
        let mut index = 0;
        while index < name.len() {
            bytes[index] = name.as_bytes()[index];
            index += 1;
        }
        Self {
            bytes,
            len: name.len() as u8,
        }
    }

    /// A valid key name: 1 to 32 ASCII letters and digits.
    pub fn new(name: &str) -> Option<Self> {
        let valid =
            (1..=32).contains(&name.len()) && name.bytes().all(|b| b.is_ascii_alphanumeric());
        valid.then(|| Self::literal(name))
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..usize::from(self.len)]).unwrap_or("F8")
    }

    /// A short label: `KeyE` shows as `E`, `Digit1` as `1`, others as named.
    pub fn display(&self) -> &str {
        let name = self.as_str();
        name.strip_prefix("Key")
            .or_else(|| name.strip_prefix("Digit"))
            .filter(|rest| rest.len() == 1)
            .unwrap_or(name)
    }
}

impl std::fmt::Debug for KeyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub const MIN_WIDTH: f32 = 1.0;
pub const MAX_WIDTH: f32 = 8.0;
pub const MIN_REACH: f32 = 1.0;
/// Cinnabar's keyboard-and-mouse block reach, Survival and Creative alike
/// (`CREATIVE_MOUSE_REACH_BLOCKS`; controller 5.6, touch Survival 6.7).
pub const PLAYER_REACH: f32 = 5.7;
/// A 15-block terrain query (radius 7) stays inside the 16-block query extent limit.
pub const MAX_REACH: f32 = 7.0;

impl Default for Settings {
    fn default() -> Self {
        Self {
            rgba: [255, 255, 255, 255],
            rainbow: false,
            width: 3.0,
            fill: 0,
            reach: PLAYER_REACH,
            menu_key: KeyName::DEFAULT,
            fill_rgb: None,
            pulse: false,
            highlight_cancel: DEFAULT_HIGHLIGHT_CANCEL,
            mob: Style::DEFAULT,
            show_all_mobs: true,
            aim_rgb: DEFAULT_AIM,
            arrow_rgb: [255, 255, 255],
            fullbright: false,
            fullbright_key: None,
            auto_queue: false,
            favourites: [None; FAVOURITES],
            next_favourite: 0,
            counter: crate::counter::Counter::default(),
            armor: crate::armor::Armor::default(),
            zoom: crate::zoom::Zoom::default(),
            cps: crate::cps::Cps::default(),
            fps: crate::fps::Fps::default(),
            jump_reset: crate::jumpreset::JumpReset::default(),
            crosshair: crate::crosshair::DEFAULT,
            block_on: false,
            mob_on: false,
            hide_hunger: false,
            effect_timers: false,
            hotbar_keys: false,
        }
    }
}

/// The crosshair settings in settings JSON, with defaults for anything missing or unusable.
pub(super) fn parse_crosshair(json: &Fields) -> crate::crosshair::Crosshair {
    use crate::crosshair::{self, GAP, LENGTH, THICKNESS};
    let mut out = crosshair::DEFAULT;
    if let Some(on) = bool_field(json, "crosshair") {
        out.on = on;
    }
    if let Some(style) = string_field(json, "crosshair_style").and_then(crosshair::style_from_key) {
        out.style = style;
    }
    if let Some([r, g, b, _]) = string_field(json, "crosshair_color").and_then(parse_color) {
        out.rgb = [r, g, b];
    }
    if let Some(opacity) = number_field(json, "crosshair_opacity") {
        out.opacity = crosshair::clamp_opacity(opacity);
    }
    if let Some(length) = number_field(json, "crosshair_length") {
        out.length = LENGTH.clamp(length);
    }
    if let Some(thickness) = number_field(json, "crosshair_thickness") {
        out.thickness = THICKNESS.clamp(thickness);
    }
    if let Some(gap) = number_field(json, "crosshair_gap") {
        out.gap = GAP.clamp(gap);
    }
    if let Some(outline) = bool_field(json, "crosshair_outline") {
        out.outline = outline;
    }
    // The resolution first: the drawing is read at it. Older files had none, and a 15 by 15
    // drawing, which is read into the smallest grid.
    let size = number_field(json, "crosshair_resolution")
        .map_or(crosshair::RESOLUTIONS[0], |size| {
            size.round().clamp(0.0, 255.0) as u8
        });
    if let Some(text) = string_field(json, "crosshair_pixels") {
        out.pixels = crosshair::Drawing::parse(text, size);
    }
    if let Some(pixel) = number_field(json, "crosshair_pixel_size") {
        out.pixel_size = crosshair::clamp_pixel_size(pixel);
    }
    if let Some(mirror) = bool_field(json, "crosshair_mirror") {
        out.mirror = mirror;
    }
    out
}

/// One-click colours offered by the menu.
pub const SWATCHES: [(&str, [u8; 3]); 10] = [
    ("White", [255, 255, 255]),
    ("Black", [0, 0, 0]),
    ("Red", [255, 40, 40]),
    ("Orange", [255, 140, 0]),
    ("Yellow", [255, 230, 0]),
    ("Lime", [80, 255, 60]),
    ("Cyan", [0, 230, 255]),
    ("Blue", [40, 90, 255]),
    ("Purple", [170, 60, 255]),
    ("Pink", [255, 100, 200]),
];

impl Settings {
    /// Reads settings JSON, keeping defaults for missing or invalid fields. A legacy
    /// `"color": "rainbow"` turns rainbow on.
    pub fn parse(json: &str) -> Self {
        Self::from_fields(&Fields::new(json))
    }

    /// [`Settings::parse`] for a file already split into its fields.
    pub fn from_fields(json: &Fields) -> Self {
        let mut settings = Self::default();
        match string_field(json, "color") {
            Some(text) if text.trim().eq_ignore_ascii_case("rainbow") => settings.rainbow = true,
            Some(text) => {
                if let Some(rgba) = parse_color(text) {
                    settings.rgba = rgba;
                }
            }
            None => {}
        }
        if let Some(rainbow) = bool_field(json, "rainbow") {
            settings.rainbow = rainbow;
        }
        if let Some(width) = number_field(json, "width") {
            settings.width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        }
        if let Some(fill) = number_field(json, "fill") {
            settings.fill = fill.round().clamp(0.0, 100.0) as u8;
        }
        match string_field(json, "fill_color") {
            Some(text) if text.trim().eq_ignore_ascii_case("line") => settings.fill_rgb = None,
            Some(text) => {
                if let Some([r, g, b, _]) = parse_color(text) {
                    settings.fill_rgb = Some([r, g, b]);
                }
            }
            None => {}
        }
        // Older files had an on/off switch; full strength turned out too dark, so on means half.
        if let Some(hide) = bool_field(json, "hide_game_highlight") {
            settings.highlight_cancel = if hide { 50 } else { 0 };
        }
        if let Some(percent) = number_field(json, "highlight_cancel") {
            settings.highlight_cancel = percent.round().clamp(0.0, 100.0) as u8;
        }
        if let Some(pulse) = bool_field(json, "pulse") {
            settings.pulse = pulse;
        }
        // Mobs looked like blocks before they had their own page: start from the block look.
        settings.mob = settings.style();
        if let Some(rgba) = string_field(json, "mob_color").and_then(parse_color) {
            settings.mob.rgba = rgba;
        }
        if let Some(rainbow) = bool_field(json, "mob_rainbow") {
            settings.mob.rainbow = rainbow;
        }
        if let Some(width) = number_field(json, "mob_width") {
            settings.mob.width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        }
        // Mob hitboxes never pulse and have no fill.
        settings.mob.pulse = false;
        settings.mob.fill_rgb = None;
        settings.mob.fill = 0;
        if let Some(fullbright) = bool_field(json, "fullbright") {
            settings.fullbright = fullbright;
        }
        if let Some(key) = string_field(json, "fullbright_key") {
            settings.fullbright_key = KeyName::new(key);
        }
        if let Some(on) = bool_field(json, "auto_queue") {
            settings.auto_queue = on;
        }
        if let Some(on) = bool_field(json, "block_on") {
            settings.block_on = on;
        }
        if let Some(on) = bool_field(json, "hide_hunger") {
            settings.hide_hunger = on;
        }
        if let Some(on) = bool_field(json, "effect_timers") {
            settings.effect_timers = on;
        }
        if let Some(on) = bool_field(json, "hotbar_keys") {
            settings.hotbar_keys = on;
        }
        if let Some(on) = bool_field(json, "mob_on") {
            settings.mob_on = on;
        }
        if let Some(all) = bool_field(json, "show_all_mobs") {
            settings.show_all_mobs = all;
        }
        if let Some([r, g, b, _]) = string_field(json, "aim_color").and_then(parse_color) {
            settings.aim_rgb = [r, g, b];
        }
        if let Some([r, g, b, _]) = string_field(json, "arrow_color").and_then(parse_color) {
            settings.arrow_rgb = [r, g, b];
        }
        if let Some(list) = string_field(json, "favourites") {
            for (slot, code) in settings.favourites.iter_mut().zip(list.split(',')) {
                *slot = parse_color(code);
            }
        }
        if let Some(next) = number_field(json, "next_favourite") {
            settings.next_favourite = (next.max(0.0) as usize % FAVOURITES) as u8;
        }
        if let Some(key) = string_field(json, "menu_key").and_then(KeyName::new) {
            settings.menu_key = key;
        }
        if let Some(reach) = number_field(json, "reach") {
            settings.reach = reach.clamp(MIN_REACH, MAX_REACH);
        }
        if let Some(on) = bool_field(json, "zoom") {
            settings.zoom.on = on;
        }
        if let Some(toggle) = bool_field(json, "zoom_toggle") {
            settings.zoom.toggle = toggle;
        }
        if let Some(scroll) = bool_field(json, "zoom_scroll") {
            settings.zoom.scroll = scroll;
        }
        if let Some(sensitivity) = bool_field(json, "zoom_sensitivity") {
            settings.zoom.sensitivity = sensitivity;
        }
        if let Some(level) = number_field(json, "zoom_level") {
            settings.zoom.level = crate::zoom::clamp_level(level);
        }
        if let Some(time) = number_field(json, "zoom_time") {
            settings.zoom.time = crate::zoom::clamp_time(time);
        }
        if let Some(step) = number_field(json, "zoom_step") {
            settings.zoom.step = crate::zoom::clamp_step(step);
        }
        // An empty (or unusable) name is an unbound key, as for the light key.
        if let Some(key) = string_field(json, "zoom_key") {
            settings.zoom.key = KeyName::new(key);
        }
        if let (Some(x), Some(y), Some(ax), Some(ay)) = (
            number_field(json, "armor_x"),
            number_field(json, "armor_y"),
            string_field(json, "armor_anchor_x").and_then(crate::counter::Anchor::from_name),
            string_field(json, "armor_anchor_y").and_then(crate::counter::Anchor::from_name),
        ) {
            settings.armor.pos = crate::counter::Pos {
                anchor: [ax, ay],
                off: [x, y],
            }
            .sanitised();
        }
        if let Some(scale) = number_field(json, "armor_scale") {
            settings.armor.scale = crate::counter::clamp_scale(scale);
        }
        if let Some(on) = bool_field(json, "armor_on") {
            settings.armor.on = on;
        }
        if let Some(on) = bool_field(json, "armor_horizontal") {
            settings.armor.horizontal = on;
        }
        if let Some(on) = bool_field(json, "armor_percent") {
            settings.armor.percent = on;
        }
        if let Some(on) = bool_field(json, "armor_background") {
            settings.armor.background = on;
        }
        if let Some(on) = bool_field(json, "arrow_counter") {
            settings.counter.on = on;
        }
        if let Some(bow_only) = bool_field(json, "arrow_counter_bow_only") {
            settings.counter.bow_only = bow_only;
        }
        if let (Some(x), Some(y)) = (
            number_field(json, "arrow_counter_x"),
            number_field(json, "arrow_counter_y"),
        ) {
            use crate::counter::{Anchor, Pos};
            let anchor = |key| string_field(json, key).and_then(Anchor::from_name);
            // Without anchors (older files) the numbers are fractions of the screen.
            settings.counter.pos = match (
                anchor("arrow_counter_anchor_x"),
                anchor("arrow_counter_anchor_y"),
            ) {
                (Some(ax), Some(ay)) => Pos {
                    anchor: [ax, ay],
                    off: [x, y],
                }
                .sanitised(),
                _ => Pos::fraction(x, y),
            };
        }
        if let Some(scale) = number_field(json, "arrow_counter_scale") {
            settings.counter.scale = crate::counter::clamp_scale(scale);
        }
        if let Some(on) = bool_field(json, "cps") {
            settings.cps.on = on;
        }
        if let Some(right) = bool_field(json, "cps_right") {
            settings.cps.right = right;
        }
        if let Some(zeros) = bool_field(json, "cps_zeros") {
            settings.cps.zeros = zeros;
        }
        if let (Some(x), Some(y)) = (number_field(json, "cps_x"), number_field(json, "cps_y")) {
            use crate::counter::{Anchor, Pos};
            let anchor = |key| string_field(json, key).and_then(Anchor::from_name);
            if let (Some(ax), Some(ay)) = (anchor("cps_anchor_x"), anchor("cps_anchor_y")) {
                settings.cps.pos = Pos {
                    anchor: [ax, ay],
                    off: [x, y],
                }
                .sanitised();
            }
        }
        if let Some(scale) = number_field(json, "cps_scale") {
            settings.cps.scale = crate::cps::clamp_scale(scale);
        }
        if let Some(on) = bool_field(json, "fps") {
            settings.fps.on = on;
        }
        if let (Some(x), Some(y)) = (number_field(json, "fps_x"), number_field(json, "fps_y")) {
            use crate::counter::{Anchor, Pos};
            let anchor = |key| string_field(json, key).and_then(Anchor::from_name);
            if let (Some(ax), Some(ay)) = (anchor("fps_anchor_x"), anchor("fps_anchor_y")) {
                settings.fps.pos = Pos {
                    anchor: [ax, ay],
                    off: [x, y],
                }
                .sanitised();
            }
        }
        if let Some(scale) = number_field(json, "fps_scale") {
            settings.fps.scale = crate::cps::clamp_scale(scale);
        }
        if let Some(on) = bool_field(json, "fps_coords") {
            settings.fps.coords = on;
        }
        if let Some(on) = bool_field(json, "jump_reset") {
            settings.jump_reset.on = on;
        }
        if let Some(window) = number_field(json, "jump_reset_window") {
            // Whole milliseconds, on a step like the menu's buttons keep it.
            settings.jump_reset.window = crate::jumpreset::JumpReset::stepped(
                window.round().clamp(0.0, f32::from(u16::MAX)) as u16,
                0,
            );
        }
        if let Some(knockback) = number_field(json, "jump_reset_carry") {
            settings.jump_reset.knockback = crate::jumpreset::clamp_knockback(knockback);
        }
        if let Some(log) = string_field(json, "jump_reset_log") {
            settings.jump_reset.log = crate::jumpreset::Log::parse(log);
        }
        settings.crosshair = parse_crosshair(json);
        settings
    }

    pub fn to_json(&self) -> String {
        let favourites: Vec<String> = self
            .favourites
            .iter()
            .map(|slot| slot.map(hex_code).unwrap_or_default())
            .collect();
        let fill_colour = self
            .fill_rgb
            .map_or_else(|| "line".to_owned(), |[r, g, b]| hex_code([r, g, b, 255]));
        let mut out = String::with_capacity(320);
        out.push_str(
            "{
",
        );
        let mut field = |name: &str, value: String| {
            out.push_str("  \"");
            out.push_str(name);
            out.push_str("\": ");
            out.push_str(&value);
            out.push_str(
                ",
",
            );
        };
        field("color", format!("\"{}\"", hex_code_full(self.rgba)));
        field("rainbow", self.rainbow.to_string());
        field("width", self.width.to_string());
        field("fill", self.fill.to_string());
        field("fill_color", format!("\"{fill_colour}\""));
        field("pulse", self.pulse.to_string());
        field("mob_color", format!("\"{}\"", hex_code_full(self.mob.rgba)));
        field("mob_rainbow", self.mob.rainbow.to_string());
        field("mob_width", self.mob.width.to_string());
        field("block_on", self.block_on.to_string());
        field("mob_on", self.mob_on.to_string());
        field("hide_hunger", self.hide_hunger.to_string());
        field("effect_timers", self.effect_timers.to_string());
        field("hotbar_keys", self.hotbar_keys.to_string());
        field("show_all_mobs", self.show_all_mobs.to_string());
        field("fullbright", self.fullbright.to_string());
        field(
            "fullbright_key",
            format!(
                "\"{}\"",
                self.fullbright_key.as_ref().map_or("", KeyName::as_str)
            ),
        );
        field("auto_queue", self.auto_queue.to_string());
        let [r, g, b] = self.aim_rgb;
        field("aim_color", format!("\"{}\"", hex_code([r, g, b, 255])));
        let [r, g, b] = self.arrow_rgb;
        field("arrow_color", format!("\"{}\"", hex_code([r, g, b, 255])));
        field("highlight_cancel", self.highlight_cancel.to_string());
        field("favourites", format!("\"{}\"", favourites.join(",")));
        field("next_favourite", self.next_favourite.to_string());
        field("reach", self.reach.to_string());
        field("zoom", self.zoom.on.to_string());
        field("zoom_toggle", self.zoom.toggle.to_string());
        field("zoom_scroll", self.zoom.scroll.to_string());
        field("zoom_sensitivity", self.zoom.sensitivity.to_string());
        field("zoom_level", self.zoom.level.to_string());
        field("zoom_time", self.zoom.time.to_string());
        field("zoom_step", self.zoom.step.to_string());
        field(
            "zoom_key",
            format!("\"{}\"", self.zoom.key.as_ref().map_or("", KeyName::as_str)),
        );
        field("arrow_counter", self.counter.on.to_string());
        field("arrow_counter_bow_only", self.counter.bow_only.to_string());
        field("armor_x", self.armor.pos.off[0].to_string());
        field("armor_y", self.armor.pos.off[1].to_string());
        field(
            "armor_anchor_x",
            format!("\"{}\"", self.armor.pos.anchor[0].name()),
        );
        field(
            "armor_anchor_y",
            format!("\"{}\"", self.armor.pos.anchor[1].name()),
        );
        field("armor_scale", self.armor.scale.to_string());
        field("armor_on", self.armor.on.to_string());
        field("armor_horizontal", self.armor.horizontal.to_string());
        field("armor_percent", self.armor.percent.to_string());
        field("armor_background", self.armor.background.to_string());
        field("arrow_counter_x", self.counter.pos.off[0].to_string());
        field("arrow_counter_y", self.counter.pos.off[1].to_string());
        field(
            "arrow_counter_anchor_x",
            format!("\"{}\"", self.counter.pos.anchor[0].name()),
        );
        field(
            "arrow_counter_anchor_y",
            format!("\"{}\"", self.counter.pos.anchor[1].name()),
        );
        field("arrow_counter_scale", self.counter.scale.to_string());
        field("cps", self.cps.on.to_string());
        field("cps_right", self.cps.right.to_string());
        field("cps_zeros", self.cps.zeros.to_string());
        field("cps_x", self.cps.pos.off[0].to_string());
        field("cps_y", self.cps.pos.off[1].to_string());
        field(
            "cps_anchor_x",
            format!("\"{}\"", self.cps.pos.anchor[0].name()),
        );
        field(
            "cps_anchor_y",
            format!("\"{}\"", self.cps.pos.anchor[1].name()),
        );
        field("cps_scale", self.cps.scale.to_string());
        field("fps", self.fps.on.to_string());
        field("fps_x", self.fps.pos.off[0].to_string());
        field("fps_y", self.fps.pos.off[1].to_string());
        field(
            "fps_anchor_x",
            format!("\"{}\"", self.fps.pos.anchor[0].name()),
        );
        field(
            "fps_anchor_y",
            format!("\"{}\"", self.fps.pos.anchor[1].name()),
        );
        field("fps_scale", self.fps.scale.to_string());
        field("fps_coords", self.fps.coords.to_string());
        field("jump_reset", self.jump_reset.on.to_string());
        field("jump_reset_window", self.jump_reset.window.to_string());
        field("jump_reset_carry", self.jump_reset.knockback.to_string());
        field(
            "jump_reset_log",
            format!("\"{}\"", self.jump_reset.log.to_text()),
        );
        let crosshair = &self.crosshair;
        field("crosshair", crosshair.on.to_string());
        field(
            "crosshair_style",
            format!("\"{}\"", crate::crosshair::style_key(crosshair.style)),
        );
        let [r, g, b] = crosshair.rgb;
        field(
            "crosshair_color",
            format!("\"{}\"", hex_code([r, g, b, 255])),
        );
        field("crosshair_opacity", crosshair.opacity.to_string());
        field("crosshair_length", crosshair.length.to_string());
        field("crosshair_thickness", crosshair.thickness.to_string());
        field("crosshair_gap", crosshair.gap.to_string());
        field("crosshair_outline", crosshair.outline.to_string());
        field("crosshair_resolution", crosshair.pixels.size().to_string());
        field(
            "crosshair_pixels",
            format!("\"{}\"", crosshair.pixels.text()),
        );
        field("crosshair_pixel_size", crosshair.pixel_size.to_string());
        field("crosshair_mirror", crosshair.mirror.to_string());
        field("menu_key", format!("\"{}\"", self.menu_key.as_str()));
        out.truncate(out.len() - 2);
        out.push_str(
            "
}
",
        );
        out
    }

    /// The block outline back to the defaults, keeping the mob look, menu key and favourites.
    pub fn reset(&mut self) {
        *self = Self {
            mob: self.mob,
            show_all_mobs: self.show_all_mobs,
            aim_rgb: self.aim_rgb,
            arrow_rgb: self.arrow_rgb,
            fullbright: self.fullbright,
            fullbright_key: self.fullbright_key,
            auto_queue: self.auto_queue,
            menu_key: self.menu_key,
            highlight_cancel: self.highlight_cancel,
            favourites: self.favourites,
            next_favourite: self.next_favourite,
            counter: self.counter,
            armor: self.armor,
            zoom: self.zoom,
            cps: self.cps,
            fps: self.fps,
            jump_reset: self.jump_reset,
            crosshair: self.crosshair,
            block_on: self.block_on,
            mob_on: self.mob_on,
            hide_hunger: self.hide_hunger,
            effect_timers: self.effect_timers,
            hotbar_keys: self.hotbar_keys,
            ..Self::default()
        };
    }

    /// The block outline's look.
    pub fn style(&self) -> Style {
        Style {
            rgba: self.rgba,
            rainbow: self.rainbow,
            width: self.width,
            fill: self.fill,
            fill_rgb: self.fill_rgb,
            pulse: self.pulse,
        }
    }

    /// These settings with `style` as the outline look.
    pub fn with_style(mut self, style: Style) -> Self {
        self.rgba = style.rgba;
        self.rainbow = style.rainbow;
        self.width = style.width;
        self.fill = style.fill;
        self.fill_rgb = style.fill_rgb;
        self.pulse = style.pulse;
        self
    }

    /// These settings as mob hitboxes are drawn and edited: the mob look in the outline fields.
    pub fn for_mobs(&self) -> Self {
        self.with_style(self.mob)
    }

    /// How the mob aimed at is drawn while every nearby mob is outlined: the mob look with the
    /// aimed colour for the lines (and a fill that follows them).
    pub fn aimed(&self) -> Self {
        let mut aimed = self.for_mobs();
        let [r, g, b] = self.aim_rgb;
        aimed.rgba = [r, g, b, aimed.rgba[3]];
        aimed.rainbow = false;
        aimed
    }

    /// Takes back settings edited through [`Settings::for_mobs`]: their outline look becomes
    /// the mob look; everything else (menu key, favourites, …) is kept as edited.
    pub fn store_mob(&mut self, edited: &Settings) {
        let block = self.style();
        *self = edited.with_style(block);
        self.mob = edited.style();
    }

    /// Saves `rgba` into the next favourite slot, oldest first.
    pub fn save_favourite(&mut self, rgba: [u8; 4]) {
        let slot = usize::from(self.next_favourite) % FAVOURITES;
        self.favourites[slot] = Some(rgba);
        self.next_favourite = ((slot + 1) % FAVOURITES) as u8;
    }

    /// Opacity multiplier for the pulse effect: a 1.5-second breath between 45% and 100%.
    pub fn pulse_factor(&self, seconds: f32) -> f32 {
        if !self.pulse {
            return 1.0;
        }
        let wave = (seconds * std::f32::consts::TAU / 1.5).sin() * 0.5 + 0.5;
        0.45 + 0.55 * wave
    }

    /// sRGB 0–1 colour of the fill: its own colour, or the line's.
    pub fn fill_srgb(&self, seconds: f32) -> [f32; 3] {
        match self.fill_rgb {
            Some(rgb) => rgb.map(|c| f32::from(c) / 255.0),
            None => {
                let [r, g, b, _] = self.srgb(seconds);
                [r, g, b]
            }
        }
    }

    /// sRGB 0–1 colour shown at `seconds`: the rainbow hue or the fixed colour, with opacity.
    pub fn srgb(&self, seconds: f32) -> [f32; 4] {
        let alpha = f32::from(self.rgba[3]) / 255.0;
        if self.rainbow {
            let [r, g, b] = hue_to_rgb((seconds * 0.2).rem_euclid(1.0));
            [r, g, b, alpha]
        } else {
            let [r, g, b, _] = self.rgba.map(|c| f32::from(c) / 255.0);
            [r, g, b, alpha]
        }
    }

    /// Linear RGBA for the renderer at `seconds` of accumulated frame time.
    pub fn linear_rgba(&self, seconds: f32) -> [f32; 4] {
        let [r, g, b, a] = self.srgb(seconds);
        [srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), a]
    }
}
