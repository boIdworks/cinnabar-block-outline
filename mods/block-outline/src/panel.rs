//! The F8 settings menu: the panel JSON, its live bindings and how its events change settings.
//! Host-independent so `cargo test` can check the documents it produces.

use std::fmt::Write as _;

use crate::outline::{MAX_WIDTH, MIN_WIDTH, SWATCHES, Settings};

/// The host opens and closes the panel with this physical key; Escape also closes it.
pub const TOGGLE_KEY: &str = "F8";

/// Card size in panel units.
const CARD: [f32; 2] = [244.0, 248.0];
/// Slider rows: control id, its label, first row y.
const SLIDERS: [(&str, [f32; 4]); 6] = [
    ("red", [0.86, 0.24, 0.24, 1.0]),
    ("green", [0.28, 0.78, 0.32, 1.0]),
    ("blue", [0.26, 0.46, 0.92, 1.0]),
    ("opacity", [0.75, 0.75, 0.78, 1.0]),
    ("width", [0.95, 0.66, 0.2, 1.0]),
    ("fill", [0.55, 0.75, 0.95, 1.0]),
];
const RAINBOW: usize = SLIDERS.len();
const FIRST_SWATCH: usize = RAINBOW + 1;
const ROW_Y: f32 = 34.0;
const ROW_STEP: f32 = 22.0;
const TRACK_X: f32 = 108.0;
const TRACK_WIDTH: f32 = 126.0;
const ROW_HEIGHT: f32 = 13.0;

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
    let mut controls = String::new();
    for (index, (id, _)) in SLIDERS.iter().enumerate() {
        let (min, max, step) = slider_range(index);
        let value = slider_value(settings, index);
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
    format!(
        r#"{{"toggle_key":"{TOGGLE_KEY}","controls":[{controls}],"surface":{{"screen":"block_outline.screen","document":{},"bindings":{}}}}}"#,
        json_string(&document()),
        bindings(settings, seconds)
    )
}

/// A patch moving every control and label to `settings`. Runs in frame callbacks, so it
/// formats numbers with [`fixed`] rather than the much costlier float formatting.
pub fn update(settings: &Settings, seconds: f32) -> String {
    let mut out = String::with_capacity(1024);
    out.push_str(r#"{"control_values":["#);
    for index in 0..SLIDERS.len() {
        out.push('[');
        fixed(&mut out, index as f32);
        out.push(',');
        fixed(&mut out, slider_value(settings, index));
        out.push_str("],");
    }
    out.push('[');
    fixed(&mut out, RAINBOW as f32);
    out.push_str(if settings.rainbow { ",true]" } else { ",false]" });
    out.push_str(r#"],"binding_changes":"#);
    bindings_into(&mut out, settings, seconds);
    out.push('}');
    out
}

/// A patch of only the preview colour, for animating the rainbow while the menu is open.
pub fn preview_update(settings: &Settings, seconds: f32) -> String {
    let mut out = String::with_capacity(96);
    out.push_str(r##"{"binding_changes":{"#preview_color":"##);
    vector(&mut out, settings.srgb(seconds));
    out.push_str("}}");
    out
}

/// Applies one control event; returns whether the settings changed.
pub fn apply(settings: &mut Settings, id: &str, value: f32) -> bool {
    if !value.is_finite() {
        return false;
    }
    let before = *settings;
    let byte = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    match id {
        "red" => settings.rgba[0] = byte(value),
        "green" => settings.rgba[1] = byte(value),
        "blue" => settings.rgba[2] = byte(value),
        "opacity" => settings.rgba[3] = byte(value.clamp(0.0, 100.0) * 2.55),
        "width" => settings.width = ((value * 2.0).round() / 2.0).clamp(MIN_WIDTH, MAX_WIDTH),
        "fill" => settings.fill = value.round().clamp(0.0, 100.0) as u8,
        "rainbow" => settings.rainbow = value > 0.5,
        _ => {
            let Some(name) = id.strip_prefix("swatch_") else {
                return false;
            };
            let Some((_, [r, g, b])) = SWATCHES
                .iter()
                .find(|(swatch, _)| swatch.eq_ignore_ascii_case(name))
            else {
                return false;
            };
            settings.rgba = [*r, *g, *b, settings.rgba[3]];
            settings.rainbow = false;
        }
    }
    *settings != before
}

fn slider_range(index: usize) -> (f32, f32, f32) {
    match index {
        0..=2 => (0.0, 255.0, 1.0),
        3 => (0.0, 100.0, 1.0),
        4 => (MIN_WIDTH, MAX_WIDTH, 0.5),
        _ => (0.0, 100.0, 1.0),
    }
}

fn slider_value(settings: &Settings, index: usize) -> f32 {
    match index {
        0..=2 => f32::from(settings.rgba[index]),
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

/// Every surface binding: row captions, fill fractions, the rainbow toggle and the preview.
fn bindings(settings: &Settings, seconds: f32) -> String {
    let mut out = String::with_capacity(768);
    bindings_into(&mut out, settings, seconds);
    out
}

fn bindings_into(out: &mut String, settings: &Settings, seconds: f32) {
    out.push('{');
    for (index, (id, _)) in SLIDERS.iter().enumerate() {
        let (min, max, _) = slider_range(index);
        let value = slider_value(settings, index);
        out.push_str("\"#");
        out.push_str(id);
        out.push_str("_text\":\"");
        out.push_str(title(id));
        out.push(' ');
        fixed(out, value);
        if index == 4 {
            out.push_str("px");
        }
        out.push_str("\",\"#");
        out.push_str(id);
        out.push_str("_fill\":");
        fixed(out, ((value - min) / (max - min)).clamp(0.0, 1.0));
        out.push(',');
    }
    let (text, background) = if settings.rainbow {
        ("Rainbow: On", [0.55, 0.3, 0.85, 1.0])
    } else {
        ("Rainbow: Off", [0.2, 0.21, 0.24, 1.0])
    };
    out.push_str(r##""#rainbow_text":""##);
    out.push_str(text);
    out.push_str(r##"","#rainbow_bg":"##);
    vector(out, background);
    out.push_str(r##","#preview_color":"##);
    vector(out, settings.srgb(seconds));
    out.push('}');
}

fn vector(out: &mut String, values: [f32; 4]) {
    out.push('[');
    for (index, value) in values.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        fixed(out, value);
    }
    out.push(']');
}

/// Appends `value` with at most three decimals, using only integer arithmetic.
fn fixed(out: &mut String, value: f32) {
    let milli = (value * 1000.0).round();
    if !milli.is_finite() {
        out.push('0');
        return;
    }
    let milli = milli as i64;
    if milli < 0 {
        out.push('-');
    }
    let milli = milli.unsigned_abs();
    digits(out, milli / 1000);
    let mut fraction = milli % 1000;
    if fraction != 0 {
        out.push('.');
        let mut scale = 100;
        while fraction != 0 {
            out.push(char::from(b'0' + (fraction / scale) as u8));
            fraction %= scale;
            scale /= 10;
        }
    }
}

fn digits(out: &mut String, mut value: u64) {
    let mut buffer = [0u8; 20];
    let mut at = buffer.len();
    loop {
        at -= 1;
        buffer[at] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    for &digit in &buffer[at..] {
        out.push(char::from(digit));
    }
}

/// Escapes `text` as a JSON string literal.
fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A filled rounded rectangle.
fn rect(name: &str, size: &str, offset: [f32; 2], color: [f32; 4], extra: &str) -> String {
    format!(
        r#"{{"{name}":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":{size},"offset":[{},{}],"anchor_from":"top_left","anchor_to":"top_left","radius":3,"color":{}{extra}}}}}"#,
        offset[0],
        offset[1],
        {
            let mut text = String::new();
            vector(&mut text, color);
            text
        }
    )
}

/// A label reading its text from `#<binding>`, or literal text when `binding` is empty.
fn label(name: &str, text: &str, size: [f32; 2], offset: [f32; 2], color: [f32; 3]) -> String {
    let (text, bindings) = match text.strip_prefix('#') {
        Some(_) => (text, format!(r#","bindings":[{{"binding_name":"{text}"}}]"#)),
        None => (text, String::new()),
    };
    format!(
        r#"{{"{name}":{{"type":"label","text":"{text}","size":[{},{}],"offset":[{},{}],"anchor_from":"top_left","anchor_to":"top_left","color":[{},{},{}],"layer":4{bindings}}}}}"#,
        size[0], size[1], offset[0], offset[1], color[0], color[1], color[2]
    )
}

/// A button over `[x, y, w, h]` routed to `action`, drawing `children`.
fn button(name: &str, area: [f32; 4], action: &str, children: &[String]) -> String {
    format!(
        r#"{{"{name}":{{"type":"button","size":[{},{}],"offset":[{},{}],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"button_mappings":[{{"from_button_id":"button.menu_select","to_button_id":"{action}","mapping_type":"pressed"}}],"controls":[{}]}}}}"#,
        area[2],
        area[3],
        area[0],
        area[1],
        children.join(",")
    )
}

/// The static JSON-UI document; everything that changes comes from bindings.
fn document() -> String {
    const TEXT: [f32; 3] = [0.92, 0.93, 0.95];
    const DIM: [f32; 3] = [0.62, 0.64, 0.68];
    const TRACK: [f32; 4] = [0.2, 0.21, 0.24, 1.0];
    let [card_w, card_h] = CARD;
    let mut card: Vec<String> = vec![
        rect(
            "background",
            r#"["100%","100%"]"#,
            [0.0, 0.0],
            [0.07, 0.08, 0.1, 0.94],
            r#","layer":1"#,
        ),
        label("title", "Block Outline", [120.0, 10.0], [10.0, 9.0], TEXT),
        rect(
            "preview_frame",
            "[52,16]",
            [card_w - 62.0, 6.0],
            [0.32, 0.33, 0.37, 1.0],
            r#","layer":2"#,
        ),
        rect(
            "preview",
            "[48,12]",
            [card_w - 60.0, 8.0],
            [1.0, 1.0, 1.0, 1.0],
            r##","layer":3,"bindings":[{"binding_name":"#preview_color","binding_name_override":"#color"}]"##,
        ),
    ];
    for (index, (id, fill)) in SLIDERS.iter().enumerate() {
        let y = ROW_Y + ROW_STEP * index as f32;
        card.push(label(
            &format!("{id}_label"),
            &format!("#{id}_text"),
            [TRACK_X - 14.0, 10.0],
            [10.0, y + 2.0],
            TEXT,
        ));
        card.push(button(
            &format!("{id}_slider"),
            [TRACK_X, y, TRACK_WIDTH, ROW_HEIGHT],
            &format!("mod.control:{index}"),
            &[
                rect("track", r#"["100%","100%"]"#, [0.0, 0.0], TRACK, r#","layer":1"#),
                rect(
                    "fill",
                    r#"["50%","100%"]"#,
                    [0.0, 0.0],
                    *fill,
                    &format!(
                        r##","layer":2,"bindings":[{{"binding_name":"#{id}_fill","binding_name_override":"#size_binding_x"}}]"##
                    ),
                ),
            ],
        ));
    }
    let rainbow_y = ROW_Y + ROW_STEP * SLIDERS.len() as f32;
    card.push(button(
        "rainbow_toggle",
        [10.0, rainbow_y, card_w - 20.0, 15.0],
        &format!("mod.control:{RAINBOW}"),
        &[
            rect(
                "toggle_bg",
                r#"["100%","100%"]"#,
                [0.0, 0.0],
                TRACK,
                r##","layer":1,"bindings":[{"binding_name":"#rainbow_bg","binding_name_override":"#color"}]"##,
            ),
            label(
                "toggle_text",
                "#rainbow_text",
                [100.0, 10.0],
                [8.0, 3.0],
                TEXT,
            ),
        ],
    ));
    let swatch_y = rainbow_y + 22.0;
    card.push(label("swatch_title", "Quick colours", [120.0, 10.0], [10.0, swatch_y], DIM));
    let swatch_width = (card_w - 20.0) / SWATCHES.len() as f32;
    for (index, (name, rgb)) in SWATCHES.iter().enumerate() {
        let [r, g, b] = rgb.map(|c| f32::from(c) / 255.0);
        card.push(button(
            &format!("swatch_{}", name.to_lowercase()),
            [10.0 + swatch_width * index as f32, swatch_y + 12.0, swatch_width - 3.0, 16.0],
            &format!("mod.control:{}", FIRST_SWATCH + index),
            &[
                rect("edge", r#"["100%","100%"]"#, [0.0, 0.0], [0.4, 0.41, 0.45, 1.0], r#","layer":1"#),
                rect(
                    "colour",
                    r#"["100% - 2px","100% - 2px"]"#,
                    [1.0, 1.0],
                    [r, g, b, 1.0],
                    r#","layer":2"#,
                ),
            ],
        ));
    }
    card.push(label(
        "hint",
        "Click or drag the bars. F8 or Esc closes.",
        [card_w - 20.0, 10.0],
        [10.0, card_h - 16.0],
        DIM,
    ));
    format!(
        r#"{{"namespace":"block_outline","screen":{{"type":"screen","size":["100%","100%"],"controls":[{{"card":{{"type":"panel","size":[{card_w},{card_h}],"controls":[{}]}}}}]}}}}"#,
        card.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_change_settings_within_bounds() {
        let mut s = Settings::default();
        assert!(apply(&mut s, "red", 12.4));
        assert_eq!(s.rgba[0], 12);
        assert!(apply(&mut s, "opacity", 50.0));
        assert_eq!(s.rgba[3], 128);
        assert!(apply(&mut s, "width", 4.3));
        assert_eq!(s.width, 4.5);
        assert!(apply(&mut s, "rainbow", 1.0));
        assert!(s.rainbow);
        assert!(apply(&mut s, "swatch_cyan", 1.0));
        assert_eq!(s.rgba, [0, 230, 255, 128]);
        assert!(!s.rainbow);
        assert!(!apply(&mut s, "swatch_cyan", 1.0));
        assert!(!apply(&mut s, "unknown", 1.0));
        assert!(!apply(&mut s, "red", f32::NAN));
    }

    #[test]
    fn slider_values_round_trip_through_events() {
        let s = Settings {
            rgba: [10, 20, 30, 255],
            rainbow: false,
            width: 2.5,
            fill: 40,
            reach: crate::outline::PLAYER_REACH,
        };
        let mut copy = Settings::default();
        for (index, (id, _)) in SLIDERS.iter().enumerate() {
            apply(&mut copy, id, slider_value(&s, index));
        }
        assert_eq!(copy, s);
    }

    #[test]
    fn fixed_formats_like_json_numbers() {
        let cases = [(0.0, "0"), (3.0, "3"), (2.5, "2.5"), (0.314, "0.314"),
            (-0.25, "-0.25"), (255.0, "255"), (0.05, "0.05"), (f32::NAN, "0")];
        for (value, text) in cases {
            let mut out = String::new();
            fixed(&mut out, value);
            assert_eq!(out, text, "{value}");
        }
        let update = update(&Settings::default(), 0.0);
        assert!(update.starts_with(r#"{"control_values":[[0,255],[1,255],[2,255],[3,100],[4,3],[5,0],[6,false]],"binding_changes":{"#));
        assert!(update.contains(r##""#width_text":"Thickness 3px""##));
        assert!(update.ends_with("}}"));
    }

    #[test]
    fn swatch_controls_line_up_with_their_routes() {
        let json = content(&Settings::default(), 0.0);
        assert!(json.contains(r#""id":"swatch_pink""#));
        assert!(json.contains(&format!("mod.control:{}", FIRST_SWATCH + SWATCHES.len() - 1)));
        assert!(!json.contains(&format!("mod.control:{}", FIRST_SWATCH + SWATCHES.len())));
    }
}
