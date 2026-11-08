//! Colour parsing and conversion: hex codes, HSV, the colour wheel and sRGB to linear.

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
pub(super) fn hex_digits(bytes: &[u8]) -> String {
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
pub(super) fn hue_to_rgb(hue: f32) -> [f32; 3] {
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
