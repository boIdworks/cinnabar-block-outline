//! Host-independent block outline logic: settings, colours, the block raycast and box edges.
//! Kept free of Cinnabar bindings so `cargo test` exercises it natively.

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
    /// Outline every nearby mob, not only the one aimed at.
    pub show_all_mobs: bool,
    /// Line colour of the mob aimed at while every nearby mob is outlined.
    pub aim_rgb: [u8; 3],
    /// Saved colours, filled by the menu's Save button.
    pub favourites: [Option<[u8; 4]>; FAVOURITES],
    /// The slot the next Save writes, oldest first.
    pub next_favourite: u8,
    /// The HUD arrow counter, set on the menu's Arrow counter page.
    pub counter: crate::counter::Counter,
    /// Zoom, set on the menu's Zoom page.
    pub zoom: crate::zoom::Zoom,
    /// The clicks-per-second readout, set on the menu's Arrow counter page.
    pub cps: crate::cps::Cps,
    /// The jump reset timing readout, set on the menu's Jump reset page.
    pub jump_reset: crate::jumpreset::JumpReset,
    /// Whether the block outline is drawn; switched on the menu's start page.
    pub block_on: bool,
    /// Whether mob hitboxes are drawn; switched on the menu's start page.
    pub mob_on: bool,
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
            fullbright: false,
            fullbright_key: None,
            favourites: [None; FAVOURITES],
            next_favourite: 0,
            counter: crate::counter::Counter::default(),
            zoom: crate::zoom::Zoom::default(),
            cps: crate::cps::Cps::default(),
            jump_reset: crate::jumpreset::JumpReset::default(),
            block_on: true,
            mob_on: true,
        }
    }
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
        if let Some(on) = bool_field(json, "block_on") {
            settings.block_on = on;
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
        field("show_all_mobs", self.show_all_mobs.to_string());
        field("fullbright", self.fullbright.to_string());
        field(
            "fullbright_key",
            format!(
                "\"{}\"",
                self.fullbright_key.as_ref().map_or("", KeyName::as_str)
            ),
        );
        let [r, g, b] = self.aim_rgb;
        field("aim_color", format!("\"{}\"", hex_code([r, g, b, 255])));
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
        field("jump_reset", self.jump_reset.on.to_string());
        field("jump_reset_window", self.jump_reset.window.to_string());
        field("jump_reset_carry", self.jump_reset.knockback.to_string());
        field(
            "jump_reset_log",
            format!("\"{}\"", self.jump_reset.log.to_text()),
        );
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
            fullbright: self.fullbright,
            fullbright_key: self.fullbright_key,
            menu_key: self.menu_key,
            highlight_cancel: self.highlight_cancel,
            favourites: self.favourites,
            next_favourite: self.next_favourite,
            counter: self.counter,
            zoom: self.zoom,
            cps: self.cps,
            jump_reset: self.jump_reset,
            block_on: self.block_on,
            mob_on: self.mob_on,
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

/// Accepts `#RGB`, `#RRGGBB` or `#RRGGBBAA`; the `#` is optional.
pub fn parse_color(text: &str) -> Option<[u8; 4]> {
    let text = text.trim();
    let hex = text.strip_prefix('#').unwrap_or(text);
    if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    match hex.len() {
        3 => {
            let nibble = |i: usize| u8::from_str_radix(&hex[i..=i], 16).ok().map(|v| v * 17);
            Some([nibble(0)?, nibble(1)?, nibble(2)?, 255])
        }
        6 => Some([byte(0)?, byte(2)?, byte(4)?, 255]),
        8 => Some([byte(0)?, byte(2)?, byte(4)?, byte(6)?]),
        _ => None,
    }
}

/// Always `#RRGGBBAA`, for the settings file.
pub fn hex_code_full(rgba: [u8; 4]) -> String {
    hex_digits(&rgba)
}

/// `#RRGGBB`, or `#RRGGBBAA` when the colour is not fully opaque.
pub fn hex_code(rgba: [u8; 4]) -> String {
    hex_digits(if rgba[3] == 255 {
        &rgba[..3]
    } else {
        &rgba[..]
    })
}

/// `#` followed by two upper-case hex digits per byte, without float or `fmt` machinery.
fn hex_digits(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(1 + 2 * bytes.len());
    out.push('#');
    for byte in bytes {
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    out
}

/// Hue (0–1), saturation (0–1) and value (0–1) of an sRGB colour.
pub fn rgb_to_hsv([r, g, b]: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = [r, g, b].map(|c| f32::from(c) / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta <= 0.0 {
        0.0
    } else if max == r {
        ((g - b) / delta).rem_euclid(6.0) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };
    let saturation = if max <= 0.0 { 0.0 } else { delta / max };
    [hue, saturation, max]
}

/// sRGB bytes for a hue, saturation and value, each 0–1.
pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> [u8; 3] {
    let [r, g, b] = hue_to_rgb(hue.rem_euclid(1.0));
    let (saturation, value) = (saturation.clamp(0.0, 1.0), value.clamp(0.0, 1.0));
    [r, g, b].map(|c| ((1.0 - saturation + saturation * c) * value * 255.0).round() as u8)
}

/// Colour picked at a point of the wheel: `x`, `y` from 0 to 1 across its square, `y` down.
/// Angle is hue with red to the right, distance from the centre is saturation, and `value`
/// keeps the current brightness.
pub fn wheel_colour(x: f32, y: f32, value: f32) -> [u8; 3] {
    let (dx, dy) = (x - 0.5, 0.5 - y);
    let saturation = (2.0 * (dx * dx + dy * dy).sqrt()).min(1.0);
    let hue = dy.atan2(dx) / std::f32::consts::TAU;
    hsv_to_rgb(hue, saturation, value)
}

/// Where a colour sits on the wheel, as the `x`, `y` that [`wheel_colour`] reads.
pub fn wheel_position(rgb: [u8; 3]) -> [f32; 2] {
    let [hue, saturation, _] = rgb_to_hsv(rgb);
    let angle = hue * std::f32::consts::TAU;
    [
        0.5 + saturation * angle.cos() / 2.0,
        0.5 - saturation * angle.sin() / 2.0,
    ]
}

pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Fully saturated sRGB colour for a hue in 0–1.
fn hue_to_rgb(hue: f32) -> [f32; 3] {
    let h = hue * 6.0;
    let x = 1.0 - (h.rem_euclid(2.0) - 1.0).abs();
    match h as u32 {
        0 => [1.0, x, 0.0],
        1 => [x, 1.0, 0.0],
        2 => [0.0, 1.0, x],
        3 => [0.0, x, 1.0],
        4 => [x, 0.0, 1.0],
        _ => [1.0, 0.0, x],
    }
}

/// The text after `"key"` and its colon, for a flat JSON object.
fn field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let quoted = format!("\"{key}\"");
    let mut rest = json;
    while let Some(at) = rest.find(&quoted) {
        let after = rest[at + quoted.len()..].trim_start();
        if let Some(value) = after.strip_prefix(':') {
            return Some(value.trim_start());
        }
        rest = &rest[at + quoted.len()..];
    }
    None
}

/// The text value of `key` in a settings file, for extras the settings do not hold.
pub fn text_field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    string_field(json, key)
}

fn string_field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let value = field(json, key)?.strip_prefix('"')?;
    Some(&value[..value.find('"')?])
}

fn bool_field(json: &str, key: &str) -> Option<bool> {
    let value = field(json, key)?;
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn number_field(json: &str, key: &str) -> Option<f32> {
    let value = field(json, key)?;
    let end = value
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')))
        .unwrap_or(value.len());
    value[..end].parse::<f32>().ok().filter(|v| v.is_finite())
}

/// Whether any part of the box `min`..`max` can be in front of the viewer at `eye` looking along
/// `dir` (a unit vector): the box's furthest corner along `dir` is past the plane through the eye.
/// A box wholly behind that plane cannot be on screen, so it need not be drawn. Boxes that straddle
/// the plane, or come out as nonsense, are kept.
pub fn in_front_of(min: [f64; 3], max: [f64; 3], eye: [f64; 3], dir: [f64; 3]) -> bool {
    let furthest: f64 = (0..3)
        .map(|axis| {
            let corner = if dir[axis] >= 0.0 {
                max[axis]
            } else {
                min[axis]
            };
            (corner - eye[axis]) * dir[axis]
        })
        .sum();
    // A little slack, so a box right at the plane is never dropped for rounding.
    !furthest.is_finite() || furthest > -0.05
}

/// View direction for actor YXZ radians: yaw turns left, pitch up, forward is -Z.
pub fn look_direction(yaw: f32, pitch: f32) -> [f64; 3] {
    let (yaw, pitch) = (f64::from(yaw), f64::from(pitch));
    [
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    ]
}

/// What the terrain reports at one block position.
pub enum Cell {
    /// Not loaded or outside the sampled region; the ray stops without a target.
    Unknown,
    /// Block-local selection boxes as `[min x, min y, min z, max x, max y, max z]`.
    Boxes(Vec<[f64; 6]>),
}

/// The targeted block and the world-space bounds of its selection shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub block: [i32; 3],
    pub min: [f64; 3],
    pub max: [f64; 3],
    /// Distance along the look ray to where it enters the shape.
    pub distance: f64,
    /// Whether this is Cinnabar's own selection, which it outlines or highlights itself: a block
    /// hit by the look ray. The bridging support block and actors are not.
    pub direct: bool,
    /// The shape's boxes in world space (just the bounds when there are too many).
    pub boxes: Boxes,
}

/// Shapes with more boxes than this are outlined by their bounds: merging their outlines would
/// cost too much of a frame's fuel.
pub const MAX_SHAPE_BOXES: usize = 4;

/// A block's selection boxes in world space, `[min x, min y, min z, max x, max y, max z]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boxes {
    boxes: [[f64; 6]; MAX_SHAPE_BOXES],
    len: u8,
}

impl Boxes {
    pub fn single(min: [f64; 3], max: [f64; 3]) -> Self {
        let mut boxes = [[0.0; 6]; MAX_SHAPE_BOXES];
        boxes[0] = [min[0], min[1], min[2], max[0], max[1], max[2]];
        Self { boxes, len: 1 }
    }

    /// The boxes, or just `bounds` when there are more than `MAX_SHAPE_BOXES`.
    pub fn new(list: &[[f64; 6]], min: [f64; 3], max: [f64; 3]) -> Self {
        if list.is_empty() || list.len() > MAX_SHAPE_BOXES {
            return Self::single(min, max);
        }
        let mut boxes = [[0.0; 6]; MAX_SHAPE_BOXES];
        boxes[..list.len()].copy_from_slice(list);
        Self {
            boxes,
            len: list.len() as u8,
        }
    }

    pub fn as_slice(&self) -> &[[f64; 6]] {
        &self.boxes[..usize::from(self.len)]
    }

    /// Every box grown by `by` on all sides.
    pub fn inflated(&self, by: f64) -> Self {
        let mut grown = *self;
        for b in &mut grown.boxes[..usize::from(self.len)] {
            for axis in 0..3 {
                b[axis] -= by;
                b[axis + 3] += by;
            }
        }
        grown
    }
}

/// One straight piece of a shape's outline, along `axis` from `start`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub start: [f64; 3],
    pub axis: usize,
    pub length: f64,
    /// Which of the four quarters around the line are inside the shape: bit `a + 2 * b`, where
    /// `a` and `b` are 1 on the high side of the first and second other axis.
    quarters: u8,
}

impl Segment {
    pub fn end(&self) -> [f64; 3] {
        let mut end = self.start;
        end[self.axis] += self.length;
        end
    }

    /// Whether a face meeting at this line faces `camera`, so the line is on the visible side
    /// of the shape.
    pub fn visible(&self, camera: [f64; 3]) -> bool {
        let [u, v] = Edge {
            axis: self.axis,
            sides: [false, false],
        }
        .others();
        let inside = |a: u8, b: u8| self.quarters >> (a + 2 * b) & 1 == 1;
        let ahead = |axis: usize, high: bool| {
            let offset = camera[axis] - self.start[axis];
            if high { offset > 1e-9 } else { offset < -1e-9 }
        };
        // A face between two quarters faces away from the inside one.
        (0..2).any(|b| inside(0, b) != inside(1, b) && ahead(u, inside(0, b)))
            || (0..2).any(|a| inside(a, 0) != inside(a, 1) && ahead(v, inside(a, 0)))
    }
}

/// The outline of the union of `boxes`: every box edge, split where other boxes start or end,
/// keeping only the pieces that are real corners of the merged shape (not lines across a flat
/// face or inside it). A single box gives its 12 edges.
pub fn shape_edges(boxes: &[[f64; 6]]) -> Vec<Segment> {
    const EPS: f64 = 1e-4;
    // One box (most blocks, every mob): its own 12 edges, each with the box in one quarter.
    if let [b] = boxes {
        return EDGES
            .iter()
            .filter(|edge| b[edge.axis + 3] - b[edge.axis] >= 0.01)
            .map(|edge| {
                let [u, v] = edge.others();
                let mut start = [0.0; 3];
                start[edge.axis] = b[edge.axis];
                start[u] = if edge.sides[0] { b[u + 3] } else { b[u] };
                start[v] = if edge.sides[1] { b[v + 3] } else { b[v] };
                // The box lies on the low side of an edge on its high side, and the other way.
                let quarter = u8::from(!edge.sides[0]) + 2 * u8::from(!edge.sides[1]);
                Segment {
                    start,
                    axis: edge.axis,
                    length: b[edge.axis + 3] - b[edge.axis],
                    quarters: 1 << quarter,
                }
            })
            .collect();
    }
    // Reused for every edge rather than allocated for each.
    let mut cuts: Vec<f64> = Vec::with_capacity(2 * boxes.len() + 2);
    let inside = |p: [f64; 3]| {
        boxes
            .iter()
            .any(|b| (0..3).all(|axis| b[axis] < p[axis] && p[axis] < b[axis + 3]))
    };
    let mut segments: Vec<Segment> = Vec::new();
    for b in boxes {
        for edge in EDGES {
            let axis = edge.axis;
            let [u, v] = edge.others();
            let mut start = [0.0; 3];
            start[u] = if edge.sides[0] { b[u + 3] } else { b[u] };
            start[v] = if edge.sides[1] { b[v + 3] } else { b[v] };
            let (lo, hi) = (b[axis], b[axis + 3]);
            // Where other boxes begin or end along this edge.
            cuts.clear();
            cuts.extend([lo, hi]);
            for other in boxes {
                for cut in [other[axis], other[axis + 3]] {
                    if cut > lo + EPS && cut < hi - EPS {
                        cuts.push(cut);
                    }
                }
            }
            cuts.sort_unstable_by(f64::total_cmp);
            cuts.dedup_by(|a, b| (*a - *b).abs() < EPS);
            for pair in cuts.windows(2) {
                let (from, to) = (pair[0], pair[1]);
                if to - from < 0.01 {
                    continue;
                }
                let mut probe = start;
                probe[axis] = (from + to) / 2.0;
                let mut quarters = 0u8;
                for (bit, (du, dv)) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
                    .into_iter()
                    .enumerate()
                {
                    let mut p = probe;
                    p[u] += du * EPS * 10.0;
                    p[v] += dv * EPS * 10.0;
                    if inside(p) {
                        quarters |= 1 << bit;
                    }
                }
                // A corner of the merged shape: one or three quarters inside, or two diagonal.
                let real = matches!(quarters.count_ones(), 1 | 3)
                    || quarters == 0b1001
                    || quarters == 0b0110;
                if !real {
                    continue;
                }
                let mut from_point = start;
                from_point[axis] = from;
                let segment = Segment {
                    start: from_point,
                    axis,
                    length: to - from,
                    quarters,
                };
                let same = |s: &Segment| {
                    s.axis == axis
                        && (0..3).all(|i| (s.start[i] - segment.start[i]).abs() < EPS)
                        && (s.length - segment.length).abs() < EPS
                };
                if !segments.iter().any(same) {
                    segments.push(segment);
                }
            }
        }
    }
    segments
}

/// Walks blocks along the ray (Amanatides–Woo) and returns the first selection box hit within
/// `reach`. Boxes containing the origin are ignored, so a block around the eye is not targeted.
pub fn raycast(
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    mut lookup: impl FnMut([i32; 3]) -> Cell,
) -> Option<Hit> {
    let mut block = origin.map(|v| v.floor() as i32);
    let mut step = [0i32; 3];
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];
    for axis in 0..3 {
        if dir[axis] > 0.0 {
            step[axis] = 1;
            t_max[axis] = (f64::from(block[axis]) + 1.0 - origin[axis]) / dir[axis];
            t_delta[axis] = 1.0 / dir[axis];
        } else if dir[axis] < 0.0 {
            step[axis] = -1;
            t_max[axis] = (f64::from(block[axis]) - origin[axis]) / dir[axis];
            t_delta[axis] = -1.0 / dir[axis];
        }
    }
    // Enough steps to cross `reach` diagonally, plus slack.
    let max_steps = (reach.ceil() as usize) * 3 + 3;
    for _ in 0..max_steps {
        match lookup(block) {
            Cell::Unknown => return None,
            Cell::Boxes(boxes) if !boxes.is_empty() => {
                let base = block.map(f64::from);
                let world = |b: &[f64; 6]| {
                    (
                        [base[0] + b[0], base[1] + b[1], base[2] + b[2]],
                        [base[0] + b[3], base[1] + b[4], base[2] + b[5]],
                    )
                };
                let hit = boxes
                    .iter()
                    .filter_map(|b| {
                        let (min, max) = world(b);
                        ray_box(origin, dir, min, max).filter(|t| *t <= reach)
                    })
                    .reduce(f64::min);
                if let Some(distance) = hit {
                    let mut min = [f64::INFINITY; 3];
                    let mut max = [f64::NEG_INFINITY; 3];
                    for b in &boxes {
                        let (lo, hi) = world(b);
                        for axis in 0..3 {
                            min[axis] = min[axis].min(lo[axis]);
                            max[axis] = max[axis].max(hi[axis]);
                        }
                    }
                    let world_boxes: Vec<[f64; 6]> = boxes
                        .iter()
                        .map(|b| {
                            let (lo, hi) = world(b);
                            [lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]
                        })
                        .collect();
                    let boxes = Boxes::new(&world_boxes, min, max);
                    return Some(Hit {
                        block,
                        min,
                        max,
                        distance,
                        direct: true,
                        boxes,
                    });
                }
            }
            Cell::Boxes(_) => {}
        }
        let axis = if t_max[0] < t_max[1] {
            if t_max[0] < t_max[2] { 0 } else { 2 }
        } else if t_max[1] < t_max[2] {
            1
        } else {
            2
        };
        if t_max[axis] > reach {
            return None;
        }
        block[axis] += step[axis];
        t_max[axis] += t_delta[axis];
    }
    None
}

/// The block Cinnabar targets: the look ray's first selection hit within `reach`, kept only if
/// the block's centre is also within `reach` of the eye (`PickResult::trimmed`); otherwise
/// nothing is targeted and the bridging fallback is not tried. After a clear miss, the
/// bridging support block, under the same centre rule.
pub fn target(
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    mut lookup: impl FnMut([i32; 3]) -> Cell,
) -> Option<Hit> {
    match target_with_actors(origin, dir, reach, &mut lookup, NO_ACTORS) {
        Some(Target::Block(hit)) => Some(hit),
        _ => None,
    }
}

/// What the crosshair picks: a block, or an actor's hitbox standing in front of it.
#[derive(Debug, PartialEq)]
pub enum Target {
    Block(Hit),
    /// The actor's hitbox union; `block` is unused.
    Actor(Hit),
}

/// One actor's hitbox: the single box around all of it, `[min x, min y, min z, max x, max y,
/// max z]`. A fixed array, because a list per actor per frame costs allocations the frame's fuel
/// cannot spare.
pub type ActorBox = [[f64; 6]; 1];

/// No actors, typed.
pub const NO_ACTORS: &[ActorBox] = &[];

/// The most hitboxes [`nearest_mob_boxes`] can return.
pub const MOB_BOX_CAP: usize = 16;

/// The hitboxes within `range` blocks of `eye`, nearest first (by the distance to the nearest
/// point of the box), at most `limit` (and [`MOB_BOX_CAP`]). With a view direction the ones wholly
/// behind the viewer are left out ([`in_front_of`]). Picks in one pass into a small fixed buffer:
/// no list of every candidate, no sort.
pub fn nearest_mob_boxes<A: AsRef<[[f64; 6]]>>(
    actors: &[A],
    eye: [f64; 3],
    view: Option<[f64; 3]>,
    range: f64,
    limit: usize,
) -> Vec<[[f64; 3]; 2]> {
    let limit = limit.min(MOB_BOX_CAP);
    let mut best = [(0.0f64, [[0.0f64; 3]; 2]); MOB_BOX_CAP];
    let mut count = 0;
    for b in actors.iter().flat_map(|boxes| boxes.as_ref()) {
        let (min, max) = ([b[0], b[1], b[2]], [b[3], b[4], b[5]]);
        // Distance from the eye to the nearest point of the box.
        let gap: f64 = (0..3)
            .map(|axis| {
                (min[axis] - eye[axis])
                    .max(eye[axis] - max[axis])
                    .max(0.0)
                    .powi(2)
            })
            .sum();
        if gap > range * range || !view.is_none_or(|dir| in_front_of(min, max, eye, dir)) {
            continue;
        }
        // Where it goes among the nearest so far: after every one at least as near, so equal
        // distances keep the order they came in.
        let at = best[..count].partition_point(|(other, _)| *other <= gap);
        if at >= limit {
            continue;
        }
        let end = count.min(limit - 1);
        best.copy_within(at..end, at + 1);
        best[at] = (gap, [min, max]);
        count = (count + 1).min(limit);
    }
    best[..count].iter().map(|(_, b)| *b).collect()
}

/// Padding Cinnabar adds to actor hitboxes when picking (`DEFAULT_ACTOR_PICK_RADIUS`).
pub const ACTOR_PICK_RADIUS: f64 = 0.1;

/// Cinnabar's crosshair choice between actors and blocks (`melee::target::classify`): the
/// nearest actor hit within `reach` wins when it is more than `ACTOR_PICK_RADIUS` in front of
/// the block, and then nothing behind it is outlined, bridging support included. Each actor is
/// a list of world-space boxes; boxes around the eye (the player's own body) are skipped.
pub fn target_with_actors<A: AsRef<[[f64; 6]]>>(
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    mut lookup: impl FnMut([i32; 3]) -> Cell,
    actors: &[A],
) -> Option<Target> {
    let actor = actors
        .iter()
        .filter_map(|boxes| {
            let boxes = boxes.as_ref();
            let padded = |b: &[f64; 6]| {
                let r = ACTOR_PICK_RADIUS;
                (
                    [b[0] - r, b[1] - r, b[2] - r],
                    [b[3] + r, b[4] + r, b[5] + r],
                )
            };
            if boxes.iter().any(|b| {
                let (min, max) = padded(b);
                (0..3).all(|axis| min[axis] <= origin[axis] && origin[axis] <= max[axis])
            }) {
                return None;
            }
            let distance = boxes
                .iter()
                .filter_map(|b| {
                    let (min, max) = padded(b);
                    ray_box(origin, dir, min, max).filter(|t| *t <= reach)
                })
                .reduce(f64::min)?;
            let mut min = [f64::INFINITY; 3];
            let mut max = [f64::NEG_INFINITY; 3];
            for b in boxes {
                for axis in 0..3 {
                    min[axis] = min[axis].min(b[axis]);
                    max[axis] = max[axis].max(b[axis + 3]);
                }
            }
            Some(Hit {
                block: [0; 3],
                min,
                max,
                distance,
                direct: false,
                boxes: Boxes::single(min, max),
            })
        })
        .min_by(|a, b| a.distance.total_cmp(&b.distance));
    let block = raycast(origin, dir, reach, &mut lookup);
    let limit = block.as_ref().map_or(f64::INFINITY, |hit| hit.distance);
    if let Some(actor) = actor {
        if actor.distance + ACTOR_PICK_RADIUS < limit.min(reach) {
            return Some(Target::Actor(actor));
        }
        if actor.distance + ACTOR_PICK_RADIUS < limit {
            return None;
        }
    }
    let hit = match block {
        Some(hit) => hit,
        None => support_hit(origin, dir, reach, &mut lookup)?,
    };
    centre_within(hit.block, origin, reach).then_some(Target::Block(hit))
}

/// Whether the centre of `block` is within `reach` of `eye`, as Cinnabar trims its picks.
pub fn centre_within(block: [i32; 3], eye: [f64; 3], reach: f64) -> bool {
    let distance_squared: f64 = (0..3)
        .map(|axis| (f64::from(block[axis]) + 0.5 - eye[axis]).powi(2))
        .sum();
    distance_squared <= reach * reach
}

/// Longest downward support ray after a clear miss, as Cinnabar's `BLOCK_USE_SUPPORT_DEPTH`.
pub const SUPPORT_DEPTH: f64 = 2.0;
/// Look direction Y must be below this for bridging support, as `BLOCK_USE_SUPPORT_MAX_Y`.
pub const SUPPORT_MAX_Y: f64 = -0.7;

/// The block Cinnabar places against while bridging: when the look ray misses and the player
/// looks steeply down, a ray straight down from the eye picks the supporting block, and the new
/// block goes on its side facing the look direction (`block_use_miss_support_current`).
///
/// Only offered when the look ray within `reach` passes through one of the eight cells around the
/// support at its level, sides and corners alike:
///
/// ```text
/// x 0 x
/// 0 1 0    1 = support, 0 = side gaps, x = corner gaps
/// x 0 x
/// ```
///
/// Looking further out into the void shows nothing.
pub fn support_hit(
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    lookup: impl FnMut([i32; 3]) -> Cell,
) -> Option<Hit> {
    if dir[1] >= SUPPORT_MAX_Y {
        return None;
    }
    let hit = raycast(origin, [0.0, -1.0, 0.0], SUPPORT_DEPTH, lookup)?;
    let [x, y, z] = hit.block;
    let aimed = (-1..=1)
        .flat_map(|dx| (-1..=1).map(move |dz| (dx, dz)))
        .filter(|&offset| offset != (0, 0))
        .any(|(dx, dz)| {
            let cell = [x + dx, y, z + dz];
            let min = cell.map(f64::from);
            let max = min.map(|v| v + 1.0);
            ray_box(origin, dir, min, max).is_some_and(|t| t <= reach)
        });
    aimed.then_some(Hit {
        direct: false,
        ..hit
    })
}

/// The neighbour of `support` on the side facing the dominant horizontal look direction.
pub fn placement_cell(support: [i32; 3], dir: [f64; 3]) -> [i32; 3] {
    let mut cell = support;
    if dir[0].abs() > dir[2].abs() {
        cell[0] += if dir[0] < 0.0 { -1 } else { 1 };
    } else {
        cell[2] += if dir[2] < 0.0 { -1 } else { 1 };
    }
    cell
}

/// Entry distance of the ray into the box, if it enters at or ahead of the origin.
fn ray_box(origin: [f64; 3], dir: [f64; 3], min: [f64; 3], max: [f64; 3]) -> Option<f64> {
    let mut near = f64::NEG_INFINITY;
    let mut far = f64::INFINITY;
    for axis in 0..3 {
        if dir[axis] == 0.0 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
            continue;
        }
        let a = (min[axis] - origin[axis]) / dir[axis];
        let b = (max[axis] - origin[axis]) / dir[axis];
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far && near >= 0.0).then_some(near)
}

/// Side of the largest terrain region the host answers.
pub const REGION_EXTENT: i32 = 16;

/// The most blocks read beyond what the look ray needs on every side when a new region is
/// requested, so small turns and steps keep using it.
pub const REGION_SLACK: i32 = 2;

/// Fuel one terrain read should use at most (of a frame's 200,000), judged by the last read's
/// cost per block.
pub const READ_FUEL_TARGET: f64 = 60_000.0;

/// How many spare blocks a new region gets: as many as keep the read within
/// `READ_FUEL_TARGET` at `fuel_per_cell` (what the last read cost per block), none before the
/// first read, whose cost is unknown. `cells(slack)` is the region's size with that slack.
pub fn region_slack(fuel_per_cell: Option<f64>, cells: impl Fn(i32) -> usize) -> i32 {
    let Some(per_cell) = fuel_per_cell else {
        return 0;
    };
    (1..=REGION_SLACK)
        .rev()
        .find(|&slack| per_cell * cells(slack) as f64 <= READ_FUEL_TARGET)
        .unwrap_or(0)
}

/// The number of blocks in a region.
pub fn region_cells((min, max): ([i32; 3], [i32; 3])) -> usize {
    (0..3)
        .map(|axis| (max[axis] - min[axis]).max(0) as usize)
        .product()
}

/// The blocks the look ray can reach from `eye` within `reach`, and `pad` more on every side,
/// as inclusive minimum and exclusive maximum corners. A pad of one covers the bridging
/// support too: it is only looked for when the ray points steeply down, past its depth.
pub fn ray_region(eye: [f64; 3], dir: [f64; 3], reach: f64, pad: i32) -> ([i32; 3], [i32; 3]) {
    let mut min = [0; 3];
    let mut max = [0; 3];
    for axis in 0..3 {
        let (a, b) = (eye[axis], eye[axis] + dir[axis] * reach);
        min[axis] = a.min(b).floor() as i32 - pad;
        max[axis] = a.max(b).floor() as i32 + 1 + pad;
    }
    (min, max)
}

/// Whether `region` holds every block the look ray needs.
pub fn region_covers(
    region: ([i32; 3], [i32; 3]),
    eye: [f64; 3],
    dir: [f64; 3],
    reach: f64,
) -> bool {
    let (need_min, need_max) = ray_region(eye, dir, reach, 1);
    (0..3).all(|axis| region.0[axis] <= need_min[axis] && need_max[axis] <= region.1[axis])
}

/// The terrain to sample: only the blocks along the look ray, not a cube around the player.
/// Cinnabar sends the name and tags of every kind of block in a region, which costs the mod
/// fuel per kind; on servers with many custom blocks a full cube can use up a whole frame. The
/// current region is kept while it still holds the ray; otherwise a new one is requested with
/// `slack` blocks to spare.
pub fn region_for(
    current: Option<([i32; 3], [i32; 3])>,
    eye: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    slack: i32,
) -> ([i32; 3], [i32; 3]) {
    if let Some(region) = current
        && region_covers(region, eye, dir, reach)
    {
        return region;
    }
    ray_region(eye, dir, reach, 1 + slack)
}

/// Within this many blocks of the camera the outline keeps its full thickness.
pub const FULL_WIDTH_DISTANCE: f64 = 2.5;

/// Line width in pixels for a box seen from `camera`: the chosen thickness up close, then
/// shrinking with distance as a real object would, but never below one pixel.
pub fn width_at(width: f32, min: [f64; 3], max: [f64; 3], camera: [f64; 3]) -> f32 {
    let distance = (0..3)
        .map(|axis| {
            let centre = (min[axis] + max[axis]) / 2.0;
            (centre - camera[axis]).powi(2)
        })
        .sum::<f64>()
        .sqrt();
    let scale = (FULL_WIDTH_DISTANCE / distance.max(FULL_WIDTH_DISTANCE)) as f32;
    (width * scale).max(1.0).min(width)
}

/// One box face: the axis it is perpendicular to and whether it is on the maximum side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    pub axis: usize,
    pub max_side: bool,
}

/// The six faces of a box.
pub const FACES: [Face; 6] = [
    Face {
        axis: 0,
        max_side: false,
    },
    Face {
        axis: 0,
        max_side: true,
    },
    Face {
        axis: 1,
        max_side: false,
    },
    Face {
        axis: 1,
        max_side: true,
    },
    Face {
        axis: 2,
        max_side: false,
    },
    Face {
        axis: 2,
        max_side: true,
    },
];

impl Face {
    /// The face's four corners on the unit cube, in order around it.
    pub fn unit_corners(self) -> [[f32; 3]; 4] {
        let side = if self.max_side { 1.0 } else { 0.0 };
        let (u, v) = match self.axis {
            0 => (1, 2),
            1 => (0, 2),
            _ => (0, 1),
        };
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]].map(|[a, b]| {
            let mut corner = [0.0; 3];
            corner[self.axis] = side;
            corner[u] = a;
            corner[v] = b;
            corner
        })
    }

    /// Whether the camera is beyond the face's plane, so the face is turned towards it.
    pub fn visible(self, min: [f64; 3], max: [f64; 3], camera: [f64; 3]) -> bool {
        if self.max_side {
            camera[self.axis] > max[self.axis]
        } else {
            camera[self.axis] < min[self.axis]
        }
    }
}

/// One box edge: the axis it runs along and, for the other two axes in increasing order,
/// whether it lies on their minimum (false) or maximum (true) side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub axis: usize,
    pub sides: [bool; 2],
}

/// The twelve edges of a box, four along each axis.
pub const EDGES: [Edge; 12] = {
    let mut edges = [Edge {
        axis: 0,
        sides: [false, false],
    }; 12];
    let mut index = 0;
    while index < 12 {
        edges[index] = Edge {
            axis: index / 4,
            sides: [index % 2 == 1, (index / 2) % 2 == 1],
        };
        index += 1;
    }
    edges
};

impl Edge {
    /// The two axes other than the edge's own, in increasing order.
    pub fn others(self) -> [usize; 2] {
        match self.axis {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        }
    }

    /// Start and end points of this edge on the unit cube.
    pub fn unit_points(self) -> [[f32; 3]; 2] {
        let mut start = [0.0; 3];
        for (axis, side) in self.others().into_iter().zip(self.sides) {
            start[axis] = if side { 1.0 } else { 0.0 };
        }
        let mut end = start;
        end[self.axis] = 1.0;
        [start, end]
    }

    /// Whether the edge borders a face that faces `camera`: a face is visible when the camera is
    /// beyond its plane. Edges between two faces turned away from the camera are hidden behind
    /// the block, so only the outline of the faces you can see remains.
    pub fn visible(self, min: [f64; 3], max: [f64; 3], camera: [f64; 3]) -> bool {
        self.others()
            .into_iter()
            .zip(self.sides)
            .any(|(axis, side)| {
                if side {
                    camera[axis] > max[axis]
                } else {
                    camera[axis] < min[axis]
                }
            })
    }

    /// World-space start and end points on the box.
    pub fn points(self, min: [f64; 3], max: [f64; 3]) -> [[f32; 3]; 2] {
        self.unit_points().map(|unit| {
            std::array::from_fn(|axis| {
                (min[axis] + f64::from(unit[axis]) * (max[axis] - min[axis])) as f32
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: [f64; 6] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    #[test]
    fn parses_colors_and_clamps_numbers() {
        let s = Settings::parse(r##"{"color": "#ff8800", "width": 20, "reach": 3.5}"##);
        assert_eq!(s.rgba, [255, 136, 0, 255]);
        assert!(!s.rainbow);
        assert_eq!(s.width, MAX_WIDTH);
        assert_eq!(s.reach, 3.5);
        assert_eq!(parse_color("#0F08"), None);
        assert_eq!(parse_color("0f0"), Some([0, 255, 0, 255]));
        assert_eq!(parse_color("#11223344"), Some([0x11, 0x22, 0x33, 0x44]));
        assert_eq!(Settings::parse(""), Settings::default());
        assert_eq!(
            Settings::parse("{\"color\": \"nope\"}"),
            Settings::default()
        );
    }

    #[test]
    fn settings_round_trip_and_legacy_rainbow() {
        let s = Settings {
            rgba: [1, 2, 3, 4],
            rainbow: true,
            width: 3.5,
            fill: 35,
            reach: 5.0,
            menu_key: KeyName::new("KeyE").unwrap(),
            fill_rgb: Some([1, 2, 3]),
            pulse: true,
            highlight_cancel: 40,
            mob: Style {
                rgba: [200, 10, 20, 128],
                rainbow: false,
                width: 6.5,
                fill: 0,
                fill_rgb: None,
                pulse: false,
            },
            show_all_mobs: false,
            aim_rgb: [9, 9, 9],
            fullbright: true,
            fullbright_key: KeyName::new("KeyB"),
            favourites: [Some([9, 8, 7, 6]), None, Some([255, 0, 0, 255]), None],
            next_favourite: 3,
            counter: crate::counter::Counter {
                on: false,
                bow_only: true,
                pos: crate::counter::Pos {
                    anchor: [crate::counter::Anchor::Start, crate::counter::Anchor::End],
                    off: [30.5, -12.25],
                },
                scale: 2.5,
            },
            zoom: crate::zoom::Zoom {
                on: false,
                toggle: true,
                scroll: false,
                sensitivity: false,
                level: 6.5,
                time: 0.4,
                step: 35,
                // Unbound: written as an empty name and read back as none.
                key: None,
            },
            cps: crate::cps::Cps {
                on: false,
                right: false,
                zeros: false,
                pos: crate::counter::Pos {
                    anchor: [crate::counter::Anchor::End, crate::counter::Anchor::Centre],
                    off: [-14.5, 3.25],
                },
                scale: 1.7,
            },
            jump_reset: crate::jumpreset::JumpReset {
                on: false,
                window: 120,
                knockback: 0.3,
                log: {
                    let mut log = crate::jumpreset::Log::new();
                    log.push(crate::jumpreset::Row {
                        offset_ms: Some(-40),
                        knockback: 55,
                        grounded: Some(true),
                        speed: 130,
                    });
                    log
                },
            },
            block_on: false,
            mob_on: false,
        };
        assert_eq!(Settings::parse(&s.to_json()), s);
        // A window typed by hand is rounded onto a step and kept in bounds.
        let typed = |window: &str| {
            Settings::parse(&format!(r#"{{"jump_reset_window": {window}}}"#))
                .jump_reset
                .window
        };
        assert_eq!(typed("77"), 75);
        assert_eq!(typed("1"), crate::jumpreset::MIN_WINDOW);
        assert_eq!(typed("99999"), crate::jumpreset::MAX_WINDOW);
        let kb = |value: &str| {
            Settings::parse(&format!(r#"{{"jump_reset_carry": {value}}}"#))
                .jump_reset
                .knockback
        };
        assert_eq!(kb("0.4"), 0.4);
        assert_eq!(kb("0"), crate::jumpreset::MIN_KNOCKBACK);
        assert_eq!(kb("9"), crate::jumpreset::MAX_KNOCKBACK);
        // Older files have no jump reset: it is on, at the default window.
        assert_eq!(
            Settings::parse("{}").jump_reset,
            crate::jumpreset::JumpReset::default()
        );
        let legacy = Settings::parse(r#"{"color": "rainbow", "width": 3, "reach": 6}"#);
        assert!(legacy.rainbow);
        assert_eq!(legacy.rgba, Settings::default().rgba);
        let saved = Settings::parse(r##"{"color": "#50FF3CFF", "width": 3, "reach": 6}"##);
        assert_eq!(saved.rgba, [0x50, 0xFF, 0x3C, 0xFF]);
    }

    #[test]
    fn the_nearest_mob_boxes_come_nearest_first_within_range_and_the_cap() {
        let eye = [0.0, 1.6, 0.0];
        let at = |x: f64, z: f64| -> ActorBox { [[x - 0.3, 0.0, z - 0.3, x + 0.3, 1.8, z + 0.3]] };
        let actors = [
            at(5.0, 0.0),
            at(2.0, 0.0),
            at(30.0, 0.0),
            at(3.0, 4.0),
            at(-1.0, 0.0),
        ];
        let nearest = |view, range, limit| nearest_mob_boxes(&actors, eye, view, range, limit);
        let xs = |boxes: &[[[f64; 3]; 2]]| -> Vec<i32> {
            boxes
                .iter()
                .map(|b| ((b[0][0] + b[1][0]) / 2.0).round() as i32)
                .collect()
        };
        // Nearest first, the one 30 blocks away outside a range of 12.
        assert_eq!(xs(&nearest(None, 12.0, 8)), [-1, 2, 3, 5]);
        // The cap keeps the nearest.
        assert_eq!(xs(&nearest(None, 12.0, 2)), [-1, 2]);
        assert_eq!(xs(&nearest(None, 12.0, 1)), [-1]);
        assert!(nearest(None, 12.0, 0).is_empty());
        // Nothing in range, nothing returned.
        assert!(nearest(None, 0.5, 8).is_empty());
        // Looking along +X, what is at -X is behind and is left out; the cap then goes to the rest.
        let along_x = Some([1.0, 0.0, 0.0]);
        assert_eq!(xs(&nearest(along_x, 12.0, 8)), [2, 3, 5]);
        assert_eq!(xs(&nearest(along_x, 12.0, 2)), [2, 3]);
        // Equal distances keep the order they came in.
        let twins = [at(3.0, 0.0), at(-3.0, 0.0), at(0.0, 3.0)];
        let order = nearest_mob_boxes(&twins, eye, None, 12.0, 8);
        assert_eq!(order.len(), 3);
        assert!(
            order[0][0][0] > 2.0 && order[1][1][0] < -2.0,
            "first come, first kept"
        );
        // Several boxes in one actor all count; an empty actor adds nothing.
        let many = [
            vec![
                [0.0, 0.0, 2.0, 1.0, 1.0, 3.0],
                [0.0, 0.0, 6.0, 1.0, 1.0, 7.0],
            ],
            vec![],
        ];
        assert_eq!(nearest_mob_boxes(&many, eye, None, 12.0, 8).len(), 2);
        // Far more than the cap in, never more than the cap out.
        let crowd: Vec<ActorBox> = (0..40).map(|n| at(f64::from(n) * 0.2, 1.0)).collect();
        assert_eq!(
            nearest_mob_boxes(&crowd, eye, None, 12.0, 100).len(),
            MOB_BOX_CAP
        );
        let nearest16 = nearest_mob_boxes(&crowd, eye, None, 12.0, 100);
        assert!(
            nearest16
                .windows(2)
                .all(|pair| pair[0][0][0] <= pair[1][0][0])
        );
    }

    #[test]
    fn boxes_wholly_behind_the_viewer_are_skipped() {
        let eye = [0.0, 1.6, 0.0];
        let ahead = look_direction(0.0, 0.0);
        let one = |centre: [f64; 3]| {
            (
                [centre[0] - 0.3, centre[1] - 0.9, centre[2] - 0.3],
                [centre[0] + 0.3, centre[1] + 0.9, centre[2] + 0.3],
            )
        };
        let seen = |centre: [f64; 3], dir: [f64; 3]| {
            let (min, max) = one(centre);
            in_front_of(min, max, eye, dir)
        };
        // Looking along -Z: a mob ahead is in front, one behind is not, to the side it is.
        assert!(seen([0.0, 1.0, -5.0], ahead));
        assert!(!seen([0.0, 1.0, 5.0], ahead));
        assert!(
            seen([6.0, 1.0, -0.2], ahead),
            "off to the side, slightly ahead"
        );
        // Straight to the side and a little behind still straddles with its width.
        assert!(seen([3.0, 1.0, 0.2], ahead));
        // Right on top of you, or around you, is kept.
        assert!(seen([0.0, 1.0, 0.0], ahead));
        // Turning round flips it.
        let back = look_direction(std::f32::consts::PI, 0.0);
        assert!(!seen([0.0, 1.0, -5.0], back));
        assert!(seen([0.0, 1.0, 5.0], back));
        // Looking up or down: a mob below you while looking up is behind the plane.
        let up = look_direction(0.0, 1.4);
        assert!(!seen([0.0, -6.0, -0.5], up));
        assert!(seen([0.0, 6.0, -0.5], up));
        // Every corner counts: a long box reaching in front of the plane is in front.
        assert!(in_front_of([-0.3, 0.7, -4.0], [0.3, 2.5, 4.0], eye, ahead));
        // Nonsense in, kept: nothing is hidden on a guess.
        assert!(in_front_of([f64::NAN; 3], [1.0; 3], eye, ahead));
        assert!(in_front_of([0.0; 3], [1.0; 3], eye, [f64::NAN, 0.0, 0.0]));
    }

    #[test]
    fn look_direction_matches_actor_convention() {
        let forward = look_direction(0.0, 0.0);
        assert!((forward[2] + 1.0).abs() < 1e-9);
        let left = look_direction(std::f32::consts::FRAC_PI_2, 0.0);
        assert!((left[0] + 1.0).abs() < 1e-6);
        let up = look_direction(0.0, std::f32::consts::FRAC_PI_2);
        assert!((up[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn raycast_hits_first_solid_block_within_reach() {
        let solid = |p: [i32; 3]| {
            if p == [0, 64, -3] {
                Cell::Boxes(vec![FULL])
            } else {
                Cell::Boxes(vec![])
            }
        };
        let hit = raycast([0.5, 64.5, 0.5], [0.0, 0.0, -1.0], 6.0, solid).unwrap();
        assert_eq!(hit.block, [0, 64, -3]);
        assert_eq!(hit.min, [0.0, 64.0, -3.0]);
        assert_eq!(hit.max, [1.0, 65.0, -2.0]);
        assert_eq!(
            raycast([0.5, 64.5, 0.5], [0.0, 0.0, -1.0], 2.0, solid),
            None
        );
    }

    #[test]
    fn raycast_respects_partial_shapes_and_unknown_cells() {
        // A bottom slab under the ray is missed; the block behind it is hit.
        let slab = [0.0, 0.0, 0.0, 1.0, 0.5, 1.0];
        let world = |p: [i32; 3]| match p {
            [0, 64, -1] => Cell::Boxes(vec![slab]),
            [0, 64, -2] => Cell::Boxes(vec![FULL]),
            _ => Cell::Boxes(vec![]),
        };
        let hit = raycast([0.5, 64.75, 0.5], [0.0, 0.0, -1.0], 6.0, world).unwrap();
        assert_eq!(hit.block, [0, 64, -2]);
        let unknown = |p: [i32; 3]| {
            if p == [0, 64, -1] {
                Cell::Unknown
            } else {
                Cell::Boxes(vec![FULL])
            }
        };
        assert_eq!(
            raycast([0.5, 64.5, 0.5], [0.0, 0.0, -1.0], 6.0, unknown),
            None
        );
    }

    #[test]
    fn diagonal_ray_finds_block() {
        let dir = look_direction(0.6, -0.7);
        let target = |p: [i32; 3]| {
            if p[1] <= 62 {
                Cell::Boxes(vec![FULL])
            } else {
                Cell::Boxes(vec![])
            }
        };
        let hit = raycast([0.3, 64.62, 0.7], dir, 6.0, target).unwrap();
        assert_eq!(hit.block[1], 62);
    }

    #[test]
    fn bridging_outlines_the_block_underfoot_only_when_looking_steeply_down() {
        // Standing at the edge of a one-block-wide bridge along -Z; the void lies ahead.
        let bridge = |p: [i32; 3]| {
            if p[0] == 0 && p[1] == 63 && p[2] >= 0 {
                Cell::Boxes(vec![FULL])
            } else {
                Cell::Boxes(vec![])
            }
        };
        let eye = [0.5, 65.62, 0.3];
        // Aiming into the empty cell in front of the bridge's end, [0, 63, -1].
        let steep = look_direction(0.0, -1.35);
        assert_eq!(
            raycast(eye, steep, 6.0, bridge),
            None,
            "the look ray misses into the void"
        );
        assert_eq!(placement_cell([0, 63, 0], steep), [0, 63, -1]);
        assert_eq!(
            support_hit(eye, steep, 6.0, bridge).unwrap().block,
            [0, 63, 0]
        );
        // Steep enough, but aiming two blocks ahead: the ray passes over the placement cell.
        let ahead = look_direction(0.0, -0.8);
        assert!(ahead[1] < SUPPORT_MAX_Y);
        assert_eq!(support_hit(eye, ahead, 6.0, bridge), None);
        let shallow = look_direction(0.0, -0.6);
        assert_eq!(
            support_hit(eye, shallow, 6.0, bridge),
            None,
            "y -0.56 is not steep enough"
        );
        // Facing diagonally (yaw 45°, towards -x -z), aiming into the corner gap.
        let diagonal = look_direction(std::f32::consts::FRAC_PI_4, -1.3);
        assert!(diagonal[0] < 0.0 && diagonal[2] < 0.0);
        assert_eq!(
            support_hit([0.5, 65.62, 0.5], diagonal, 6.0, bridge)
                .unwrap()
                .block,
            [0, 63, 0]
        );
        // Bridging straight along -z but aiming into the side gap at -x, and the far corner.
        let side = [-0.62, -0.78, 0.0];
        assert!(side[1] < SUPPORT_MAX_Y);
        let unit = |v: [f64; 3]| {
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            v.map(|c| c / l)
        };
        assert!(
            support_hit([0.5, 65.62, 0.5], unit(side), 6.0, bridge).is_some(),
            "side gap"
        );
        assert!(
            support_hit([0.5, 65.62, 0.5], unit([-0.45, -0.78, -0.45]), 6.0, bridge).is_some(),
            "corner"
        );
        // Diagonally, a look steep enough to bridge (y below -0.7) lands at most ~1.6 blocks out,
        // still inside the corner gap, so the outline shows across that whole range.
        let shallowest = look_direction(std::f32::consts::FRAC_PI_4, -0.8);
        assert!(shallowest[1] < SUPPORT_MAX_Y);
        assert!(support_hit([0.5, 65.62, 0.5], shallowest, 6.0, bridge).is_some());
        // Too far above the support for the 2-block downward ray.
        assert_eq!(support_hit([0.5, 66.62, 0.3], steep, 6.0, bridge), None);
    }

    #[test]
    fn width_shrinks_with_distance_but_not_up_close() {
        let (min, max) = ([0.0; 3], [1.0; 3]);
        assert_eq!(
            width_at(4.0, min, max, [0.5, 0.5, 2.5]),
            4.0,
            "2 blocks away"
        );
        assert_eq!(
            width_at(4.0, min, max, [0.5, 0.5, 5.5]),
            2.0,
            "5 blocks away: half"
        );
        assert_eq!(
            width_at(1.5, min, max, [0.5, 0.5, 20.5]),
            1.0,
            "never below one pixel"
        );
        assert_eq!(width_at(1.0, min, max, [0.5, 0.5, 20.5]), 1.0);
    }

    #[test]
    fn region_follows_the_look_ray() {
        let eye = [0.5, 65.62, 0.5];
        let volume =
            |(min, max): ([i32; 3], [i32; 3])| (0..3).map(|a| max[a] - min[a]).product::<i32>();
        // Every direction at the longest reach fits the host's extent and holds the ray.
        for yaw in 0..16 {
            for pitch in -4..=4 {
                let dir = look_direction(yaw as f32 * 0.4, pitch as f32 * 0.38);
                let region = region_for(None, eye, dir, f64::from(MAX_REACH), REGION_SLACK);
                assert!(
                    (0..3).all(|a| region.1[a] - region.0[a] <= REGION_EXTENT),
                    "{region:?}"
                );
                assert!(region_covers(region, eye, dir, f64::from(MAX_REACH)));
                let end = (0..3)
                    .map(|a| eye[a] + dir[a] * f64::from(MAX_REACH))
                    .collect::<Vec<_>>();
                assert!(
                    (0..3).all(
                        |a| f64::from(region.0[a]) <= end[a] && end[a] < f64::from(region.1[a])
                    )
                );
            }
        }
        // Looking straight ahead reads far fewer blocks than the old 16-block cube.
        let ahead = region_for(None, eye, [0.0, 0.0, -1.0], 5.7, REGION_SLACK);
        assert!(volume(ahead) * 4 < 16 * 16 * 16, "{}", volume(ahead));
        // Small turns and steps keep the region; a big turn asks for a new one.
        let turned = look_direction(0.15, 0.1);
        assert_eq!(region_for(Some(ahead), eye, turned, 5.7, 0), ahead);
        assert_eq!(
            region_for(Some(ahead), [0.9, 65.62, 0.2], [0.0, 0.0, -1.0], 5.7, 0),
            ahead
        );
        assert_ne!(region_for(Some(ahead), eye, [1.0, 0.0, 0.0], 5.7, 0), ahead);
        // Looking steeply down holds the bridging support's depth and its neighbours.
        let down = look_direction(0.0, -1.2);
        let (min, max) = region_for(None, eye, down, 5.7, 0);
        assert!(min[1] <= (eye[1] - SUPPORT_DEPTH).floor() as i32 && min[0] <= -1 && 1 < max[0]);
        assert_eq!(
            volume(region_for(None, eye, [0.0, 0.0, -1.0], 5.7, 0)) as usize,
            region_cells(ray_region(eye, [0.0, 0.0, -1.0], 5.7, 1))
        );
    }

    #[test]
    fn region_slack_follows_what_reads_cost() {
        let cells = |slack: i32| {
            region_cells(ray_region(
                [0.5, 65.62, 0.5],
                [0.0, 0.0, -1.0],
                5.7,
                1 + slack,
            ))
        };
        // Unknown cost: the smallest region first.
        assert_eq!(region_slack(None, cells), 0);
        // Cheap blocks: the full slack; dear ones: less; very dear: none.
        assert_eq!(region_slack(Some(10.0), cells), REGION_SLACK);
        let between = READ_FUEL_TARGET / cells(1) as f64;
        assert_eq!(region_slack(Some(between), cells), 1);
        assert_eq!(region_slack(Some(10_000.0), cells), 0);
    }

    #[test]
    fn far_blocks_need_their_centre_within_reach() {
        // A wall along z = -6 in front of the eye: the ray reaches its face at 5.5, inside 5.7,
        // but the block's centre is 6.0 away, so Cinnabar targets nothing.
        let wall = |p: [i32; 3]| {
            if p[2] == -6 {
                Cell::Boxes(vec![FULL])
            } else {
                Cell::Boxes(vec![])
            }
        };
        let eye = [0.5, 64.5, 0.5];
        let ahead = [0.0, 0.0, -1.0];
        assert!(raycast(eye, ahead, 5.7, wall).is_some());
        assert_eq!(target(eye, ahead, 5.7, wall), None);
        // One block closer, the centre is 5.0 away and it is targeted.
        let near = |p: [i32; 3]| {
            if p[2] == -5 {
                Cell::Boxes(vec![FULL])
            } else {
                Cell::Boxes(vec![])
            }
        };
        assert_eq!(target(eye, ahead, 5.7, near).unwrap().block, [0, 64, -5]);
    }

    #[test]
    fn hex_codes_and_the_wheel_round_trip() {
        assert_eq!(hex_code([255, 136, 0, 255]), "#FF8800");
        assert_eq!(hex_code([1, 2, 3, 4]), "#01020304");
        for rgba in [[255, 136, 0, 255], [1, 2, 3, 4], [80, 255, 60, 255]] {
            assert_eq!(parse_color(&hex_code(rgba)), Some(rgba));
        }
        // Red is at the right edge, cyan at the left, white in the middle.
        assert_eq!(wheel_colour(1.0, 0.5, 1.0), [255, 0, 0]);
        assert_eq!(wheel_colour(0.0, 0.5, 1.0), [0, 255, 255]);
        assert_eq!(wheel_colour(0.5, 0.5, 1.0), [255, 255, 255]);
        // Outside the disc clamps to full saturation; brightness is kept.
        assert_eq!(wheel_colour(2.0, 0.5, 0.5), [128, 0, 0]);
        for rgb in [
            [255, 136, 0],
            [40, 90, 255],
            [170, 60, 255],
            [255, 255, 255],
        ] {
            let [x, y] = wheel_position(rgb);
            let [_, _, value] = rgb_to_hsv(rgb);
            let back = wheel_colour(x, y, value);
            for c in 0..3 {
                assert!(back[c].abs_diff(rgb[c]) <= 2, "{rgb:?} -> {back:?}");
            }
        }
    }

    #[test]
    fn mobs_in_front_win_over_blocks_like_cinnabar() {
        let wall = |p: [i32; 3]| {
            if p[2] == -4 {
                Cell::Boxes(vec![FULL])
            } else {
                Cell::Boxes(vec![])
            }
        };
        let eye = [0.5, 64.5, 0.5];
        let ahead = [0.0, 0.0, -1.0];
        let mob_at = |z: f64| vec![[0.2, 64.0, z - 0.3, 0.8, 65.0, z + 0.3]];
        // A mob 2 blocks ahead, in front of the wall at 3.5: its hitbox is outlined.
        match target_with_actors(eye, ahead, 5.7, wall, &[mob_at(-1.5)]) {
            Some(Target::Actor(hit)) => assert_eq!(hit.max, [0.8, 65.0, -1.2]),
            other => panic!("{other:?}"),
        }
        // Behind the wall, the wall wins.
        assert!(matches!(
            target_with_actors(eye, ahead, 5.7, wall, &[mob_at(-5.5)]),
            Some(Target::Block(_))
        ));
        // Off to the side, the ray misses it.
        let beside = vec![[3.0, 64.0, -2.0, 3.6, 65.0, -1.4]];
        assert!(matches!(
            target_with_actors(eye, ahead, 5.7, wall, &[beside]),
            Some(Target::Block(_))
        ));
        // Your own body, around the eye, never counts.
        let me = vec![[0.2, 63.0, 0.2, 0.8, 64.8, 0.8]];
        assert!(matches!(
            target_with_actors(eye, ahead, 5.7, wall, &[me]),
            Some(Target::Block(_))
        ));
        // A mob in front of the void blocks the bridging fallback too.
        let steep = look_direction(0.0, -1.35);
        let floor = |p: [i32; 3]| {
            if p == [0, 63, 0] {
                Cell::Boxes(vec![FULL])
            } else {
                Cell::Boxes(vec![])
            }
        };
        let eye = [0.5, 65.62, 0.3];
        assert!(target_with_actors(eye, steep, 5.7, floor, NO_ACTORS).is_some());
        let mob_below = vec![[0.0, 63.0, -1.0, 1.0, 64.0, 0.0]];
        assert!(matches!(
            target_with_actors(eye, steep, 5.7, floor, &[mob_below]),
            Some(Target::Actor(_))
        ));
    }

    #[test]
    fn favourites_reset_and_pulse() {
        let mut s = Settings {
            menu_key: KeyName::new("KeyG").unwrap(),
            ..Settings::default()
        };
        for index in 0..5u8 {
            s.save_favourite([index, 0, 0, 255]);
        }
        // Five saves into four slots: the oldest was replaced.
        assert_eq!(s.favourites[0], Some([4, 0, 0, 255]));
        assert_eq!(s.favourites[1], Some([1, 0, 0, 255]));
        assert_eq!(Settings::parse(&s.to_json()), s);
        s.width = 7.0;
        s.pulse = true;
        s.fill_rgb = Some([1, 2, 3]);
        let favourites = s.favourites;
        s.reset();
        assert_eq!(s.width, Settings::default().width);
        assert_eq!(s.fill_rgb, None);
        assert_eq!(s.favourites, favourites, "favourites survive a reset");
        assert_eq!(s.menu_key.as_str(), "KeyG", "the menu key survives a reset");
        assert_eq!(s.pulse_factor(0.3), 1.0, "pulse off");
        s.pulse = true;
        let range = (0..60).map(|i| s.pulse_factor(i as f32 * 0.05));
        let (low, high) = range.fold((1.0f32, 0.0f32), |(lo, hi), v| (lo.min(v), hi.max(v)));
        assert!(
            (0.45 - 1e-3..0.5).contains(&low) && high > 0.99,
            "{low} {high}"
        );
    }

    #[test]
    fn shape_edges_follow_the_merged_shape() {
        // One box: its 12 edges.
        assert_eq!(shape_edges(&[[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]).len(), 12);
        // Two halves of a cube: still 12 edges, no line across the faces where they meet.
        let halves = [
            [0.0, 0.0, 0.0, 1.0, 0.5, 1.0],
            [0.0, 0.5, 0.0, 1.0, 1.0, 1.0],
        ];
        let edges = shape_edges(&halves);
        let total: f64 = edges.iter().map(|s| s.length).sum();
        assert!((total - 12.0).abs() < 1e-6, "{edges:?}");
        assert!(
            edges
                .iter()
                .all(|s| s.start[1] < 0.25 || s.start[1] > 0.75 || s.axis == 1)
        );
        // A stair: an L-shaped section along x has 6 corners, so 6 lines along x and 6 at each
        // end: 18 edges in all.
        let stair = [
            [0.0, 0.0, 0.0, 1.0, 0.5, 1.0],
            [0.0, 0.5, 0.0, 1.0, 1.0, 0.5],
        ];
        let lines = shape_edges(&stair);
        let along_x = lines.iter().filter(|s| s.axis == 0).count();
        assert_eq!(along_x, 6, "{lines:?}");
        let corners: f64 = lines.iter().filter(|s| s.axis != 0).map(|s| s.length).sum();
        assert!(
            (corners - 2.0 * (1.0 + 1.0 + 0.5 + 0.5 + 0.5 + 0.5)).abs() < 1e-6,
            "{lines:?}"
        );
        // The step's inner corner is seen from above in front, not from below behind.
        let step = lines
            .iter()
            .find(|s| {
                s.axis == 0 && (s.start[1] - 0.5).abs() < 1e-9 && (s.start[2] - 0.5).abs() < 1e-9
            })
            .expect("the step line");
        assert!(step.visible([0.5, 2.0, 2.0]));
        assert!(!step.visible([0.5, -1.0, -1.0]));
        // The single-box path matches the general one, piece for piece.
        for b in [
            [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            [0.25, 0.0, 0.1, 0.75, 0.5, 0.9],
        ] {
            let fast = shape_edges(&[b]);
            // The same box twice takes the general path and gives the same outline.
            let general = shape_edges(&[b, b]);
            assert_eq!(fast.len(), general.len());
            for segment in &general {
                assert!(fast.contains(segment), "{segment:?} not in {fast:?}");
            }
        }
        // A single box's edge is visible exactly as the old cube test said.
        for camera in [[3.0, 0.5, 0.5], [0.5, 3.0, -2.0], [-2.0, -2.0, -2.0]] {
            let lines = shape_edges(&[[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]);
            let seen = lines.iter().filter(|s| s.visible(camera)).count();
            let expected = EDGES
                .iter()
                .filter(|e| e.visible([0.0; 3], [1.0; 3], camera))
                .count();
            assert_eq!(seen, expected, "{camera:?}");
        }
    }

    #[test]
    fn only_cinnabars_own_selection_counts_as_direct() {
        // Undoing the highlight: brightened by 1.3 after darkening by the cancel layer is 1.
        assert!(((1.0 - highlight_cancel_alpha(100)) * GAME_HIGHLIGHT_GAIN - 1.0).abs() < 1e-6);
        assert_eq!(highlight_cancel_alpha(0), 0.0);
        assert!(highlight_cancel_alpha(50) < highlight_cancel_alpha(100));
        assert_eq!(step_highlight_cancel(50, true), 55);
        assert_eq!(step_highlight_cancel(50, false), 45);
        assert_eq!(step_highlight_cancel(100, true), 100);
        assert_eq!(step_highlight_cancel(0, false), 0);
        assert_eq!(step_highlight_cancel(42, false), 40);
        assert_eq!(step_highlight_cancel(42, true), 45);
        assert_eq!(
            Settings::parse(r#"{"hide_game_highlight": true}"#).highlight_cancel,
            50
        );
        assert_eq!(
            Settings::parse(r#"{"highlight_cancel": 70}"#).highlight_cancel,
            70
        );
        // Mobs start from the block look in older files, then keep their own.
        let old = Settings::parse(r##"{"color": "#010203", "width": 5, "fill_color": "#040506"}"##);
        assert_eq!(
            old.mob,
            crate::outline::Style {
                fill_rgb: None,
                ..old.style()
            },
            "mob fill follows its line"
        );
        let mut s = old;
        let mut mobs = s.for_mobs();
        mobs.rgba = [200, 0, 0, 255];
        mobs.width = 2.0;
        mobs.menu_key = KeyName::new("KeyK").unwrap();
        s.store_mob(&mobs);
        assert_eq!(s.style(), old.style(), "the block look is untouched");
        assert_eq!((s.mob.rgba, s.mob.width), ([200, 0, 0, 255], 2.0));
        assert_eq!(
            s.menu_key.as_str(),
            "KeyK",
            "shared settings edited on the mob page stay"
        );
        s.reset();
        assert_eq!(
            s.mob.rgba,
            [200, 0, 0, 255],
            "resetting blocks keeps the mob look"
        );
        let pulsing = Settings::parse(r#"{"pulse": true, "mob_pulse": true}"#);
        assert!(
            pulsing.pulse && !pulsing.mob.pulse,
            "mob hitboxes never pulse"
        );
        let aimed = s.aimed();
        assert_eq!((aimed.rgba, aimed.width), ([255, 60, 60, 255], 2.0));
        assert_eq!(
            Settings::parse(&Settings::default().to_json()),
            Settings::default()
        );
        let wall = |p: [i32; 3]| {
            if p[2] == -3 {
                Cell::Boxes(vec![FULL])
            } else {
                Cell::Boxes(vec![])
            }
        };
        let eye = [0.5, 64.5, 0.5];
        match target_with_actors(eye, [0.0, 0.0, -1.0], 5.7, wall, NO_ACTORS) {
            Some(Target::Block(hit)) => assert!(hit.direct),
            other => panic!("{other:?}"),
        }
        let mob = vec![[0.2, 64.0, -1.8, 0.8, 65.0, -1.2]];
        match target_with_actors(eye, [0.0, 0.0, -1.0], 5.7, wall, &[mob]) {
            Some(Target::Actor(hit)) => assert!(!hit.direct),
            other => panic!("{other:?}"),
        }
        // The bridging support is the mod's own addition; Cinnabar highlights nothing there.
        let floor = |p: [i32; 3]| {
            if p == [0, 63, 0] {
                Cell::Boxes(vec![FULL])
            } else {
                Cell::Boxes(vec![])
            }
        };
        let steep = look_direction(0.0, -1.35);
        assert!(
            !support_hit([0.5, 65.62, 0.3], steep, 5.7, floor)
                .unwrap()
                .direct
        );
    }

    #[test]
    fn faces_match_their_edges() {
        let (min, max) = ([0.0; 3], [1.0; 3]);
        for camera in [
            [0.5, 3.0, 0.5],
            [3.0, 3.0, 3.0],
            [-2.0, 0.5, 0.5],
            [0.5, 0.5, 0.5],
        ] {
            let faces = FACES.iter().filter(|f| f.visible(min, max, camera)).count();
            let edges = EDGES.iter().filter(|e| e.visible(min, max, camera)).count();
            // Visible faces share edges pairwise: 1 face 4 edges, 2 faces 7, 3 faces 9.
            assert_eq!(edges, [0, 4, 7, 9][faces], "{camera:?}");
        }
        for face in FACES {
            let corners = face.unit_corners();
            assert!(
                corners
                    .iter()
                    .all(|c| c[face.axis] == if face.max_side { 1.0 } else { 0.0 })
            );
        }
    }

    #[test]
    fn edges_are_distinct_and_cover_the_cube() {
        let mut seen = Vec::new();
        for edge in EDGES {
            let [start, end] = edge.unit_points();
            let length: f32 = (0..3).map(|a| (end[a] - start[a]).abs()).sum();
            assert_eq!(length, 1.0);
            assert!(!seen.contains(&(start, end)));
            seen.push((start, end));
        }
    }

    #[test]
    fn only_edges_of_faces_toward_the_camera_are_visible() {
        let (min, max) = ([0.0; 3], [1.0; 3]);
        let count = |camera| EDGES.iter().filter(|e| e.visible(min, max, camera)).count();
        // Straight above: only the top face is visible, so its four edges remain.
        assert_eq!(count([0.5, 3.0, 0.5]), 4);
        // Above and in front (+z): top and front faces, sharing one edge: 4 + 4 - 1.
        assert_eq!(count([0.5, 3.0, 3.0]), 7);
        // Above, in front and to the side: three faces, the usual 3D view: 9 edges, 3 hidden.
        assert_eq!(count([3.0, 3.0, 3.0]), 9);
        // Inside the block, nothing faces the camera.
        assert_eq!(count([0.5, 0.5, 0.5]), 0);
        // The hidden edges are exactly those meeting the far corner (0, 0, 0).
        for edge in EDGES {
            let [start, _] = edge.unit_points();
            assert_eq!(edge.visible(min, max, [3.0; 3]), start != [0.0; 3]);
        }
    }
}
