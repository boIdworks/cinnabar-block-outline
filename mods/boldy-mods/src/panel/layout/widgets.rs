//! Building blocks: buttons, switches, sliders, the colour wheel and captions.

use super::*;

/// `value` with at most three decimals and no trailing zeros: `0.5`, `1`, `0.984`.
pub(in crate::panel) fn short(value: f32) -> String {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() {
        "0".into()
    } else {
        text.into()
    }
}

/// Offset that centres a line of text in a control `height` tall.
pub(in crate::panel) fn text_y(height: f32) -> f32 {
    ((height - 10.0) / 2.0).floor()
}

/// A filled rounded rectangle.
pub(in crate::panel) fn rect(
    name: &str,
    size: &str,
    offset: [f32; 2],
    color: [f32; 4],
    extra: &str,
) -> String {
    let mut colour = String::new();
    vector(&mut colour, color);
    // The type, renderer and anchors come from the `r` template (see `document`): a rectangle
    // written out in full is about 90 bytes longer, and the menu has well over a hundred.
    format!(
        r#"{{"{name}@bo.r":{{"size":{size},"offset":[{},{}],"radius":3,"color":{colour}{extra}}}}}"#,
        offset[0], offset[1],
    )
}

/// The template every button's background inherits: a rounded rectangle filling the button in
/// the track colour. The menu has close to a hundred of them, and writing each out in full would
/// cost about 16 KiB of the host's 192 KiB limit on the document.
pub(in crate::panel) fn bg_template() -> String {
    let mut colour = String::new();
    vector(&mut colour, TRACK);
    format!(
        r#""bg":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["100%","100%"],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":3,"color":{colour},"layer":1}}"#
    )
}

/// A button's background, from [`bg_template`]; its colour follows the `binding` if one is given.
pub(in crate::panel) fn track_bg(binding: Option<&str>) -> String {
    match binding {
        Some(binding) => format!(
            r##"{{"bg@bo.bg":{{"bindings":[{{"binding_name":"{binding}","binding_name_override":"#color"}}]}}}}"##
        ),
        None => String::from(r#"{"bg@bo.bg":{}}"#),
    }
}

/// A label reading its text from `#<binding>`, or literal text.
pub(in crate::panel) fn label(
    name: &str,
    text: &str,
    size: [f32; 2],
    offset: [f32; 2],
    color: [f32; 3],
) -> String {
    let bindings = if text.starts_with('#') {
        format!(r#","bindings":[{{"binding_name":"{text}"}}]"#)
    } else {
        String::new()
    };
    format!(
        r#"{{"{name}":{{"type":"label","text":"{text}","size":[{},{}],"offset":[{},{}],"anchor_from":"top_left","anchor_to":"top_left","color":[{},{},{}],"layer":4{bindings}}}}}"#,
        size[0], size[1], offset[0], offset[1], color[0], color[1], color[2]
    )
}

/// A button over `[x, y, w, h]` routed to `action`, drawing `children`.
pub(in crate::panel) fn button(
    name: &str,
    area: [f32; 4],
    action: &str,
    children: &[String],
) -> String {
    format!(
        r#"{{"{name}":{{"type":"button","size":[{},{}],"offset":[{},{}],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"button_mappings":[{{"from_button_id":"button.menu_select","to_button_id":"{action}","mapping_type":"pressed"}}],"controls":[{}]}}}}"#,
        area[2],
        area[3],
        area[0],
        area[1],
        children.join(",")
    )
}

/// Centres a label's text within its box.
pub(in crate::panel) fn centred(label: String) -> String {
    label.replacen(
        r#""type":"label","#,
        r#""type":"label","text_alignment":"center","#,
        1,
    )
}

/// `button` (as [`button`] builds it) with `child` added after its other children.
pub(in crate::panel) fn with_last_child(button: &str, child: &str) -> String {
    let close = button
        .rfind("]}}")
        .expect("a button ends with its list of controls");
    format!("{},{child}{}", &button[..close], &button[close..])
}

/// A button whose background colour and caption come from `#<name>_bg` and `#<name>_text`.
pub(in crate::panel) fn toggle_button(name: &str, area: [f32; 4], control: usize) -> String {
    bound_button(name, area, control, false)
}

/// [`toggle_button`] with the caption in the middle: for buttons that do something, such as Reset.
pub(in crate::panel) fn action_button(name: &str, area: [f32; 4], control: usize) -> String {
    bound_button(name, area, control, true)
}

pub(in crate::panel) fn bound_button(
    name: &str,
    area: [f32; 4],
    control: usize,
    middle: bool,
) -> String {
    let text = label(
        "text",
        &format!("#{name}_text"),
        [if middle { area[2] } else { area[2] - 10.0 }, 10.0],
        [if middle { 0.0 } else { 6.0 }, text_y(area[3])],
        TEXT,
    );
    button(
        name,
        area,
        &format!("mod.control:{control}"),
        &[
            track_bg(Some(&format!("#{name}_bg"))),
            if middle { centred(text) } else { text },
        ],
    )
}

/// An on / off row: the caption on the left and a pill switch on the right whose colour
/// (`#<name>_bg`) and knob position (`#<name>_knob`) show the state.
pub(in crate::panel) fn switch_button(
    name: &str,
    area: [f32; 4],
    control: usize,
    caption: &str,
) -> String {
    let mut off = String::new();
    vector(&mut off, SWITCH_OFF);
    let pill = format!(
        r##"{{"pill":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{PILL_W},{PILL_H}],"anchor_from":"right_middle","anchor_to":"right_middle","offset":[-7,0],"radius":{},"color":{off},"layer":2,"bindings":[{{"binding_name":"#{name}_bg","binding_name_override":"#color"}}],"controls":[{{"knob":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{KNOB},{KNOB}],"anchor_from":"left_middle","anchor_to":"left_middle","offset":[{KNOB_OFF},0],"radius":{},"color":[1,1,1,1],"layer":1,"bindings":[{{"binding_name":"#{name}_knob","binding_name_override":"#offset"}}]}}}}]}}}}"##,
        PILL_H / 2.0,
        KNOB / 2.0,
    );
    button(
        name,
        area,
        &format!("mod.control:{control}"),
        &[
            track_bg(None),
            label(
                "text",
                caption,
                [area[2] - PILL_W - 22.0, 10.0],
                [8.0, text_y(area[3])],
                TEXT,
            ),
            pill,
        ],
    )
}

/// A key button, drawn like a key: a lighter cap with a darker lip under it. The caption (the
/// key, or what to do) comes from `#<name>_text` and the cap's colour from `#<name>_bg`.
pub(in crate::panel) fn key_button(name: &str, area: [f32; 4], control: usize) -> String {
    button(
        name,
        area,
        &format!("mod.control:{control}"),
        &[
            rect(
                "lip",
                r#"["100%","100%"]"#,
                [0.0, 0.0],
                KEY_LIP,
                r#","layer":1"#,
            ),
            rect(
                "cap",
                &format!(r#"["100%","100% - {KEY_LIP_H}px"]"#),
                [0.0, 0.0],
                TRACK,
                &format!(
                    r##","layer":2,"bindings":[{{"binding_name":"#{name}_bg","binding_name_override":"#color"}}]"##
                ),
            ),
            centred(label(
                "text",
                &format!("#{name}_text"),
                [area[2], 10.0],
                [0.0, text_y(area[3] - KEY_LIP_H)],
                TEXT,
            )),
        ],
    )
}

/// A value button (`Zoom level: 4x`): a [`toggle_button`] that also says `reset` on its right
/// (`#<name>_reset`) while the value is not its default, since clicking it resets it.
pub(in crate::panel) fn value_button(name: &str, area: [f32; 4], control: usize) -> String {
    let button = toggle_button(name, area, control);
    let hint = label(
        "reset",
        &format!("#{name}_reset"),
        [area[2] - 8.0, 10.0],
        [0.0, text_y(area[3])],
        DIM,
    )
    .replacen(
        r#""type":"label","#,
        r#""type":"label","text_alignment":"right","#,
        1,
    );
    with_last_child(&button, &hint)
}

/// A button for the main action of a card: `plain_button` in the accent colour.
pub(in crate::panel) fn accent_button(
    name: &str,
    area: [f32; 4],
    control: usize,
    caption: &str,
) -> String {
    let (mut plain, mut accent) = (String::new(), String::new());
    vector(&mut plain, TRACK);
    vector(&mut accent, ACTIVE);
    plain_button(name, area, control, caption).replacen(&plain, &accent, 1)
}

/// A Line / Fill colour button: a toggle with a small swatch of that colour on its right.
pub(in crate::panel) fn edit_button(
    name: &str,
    area: [f32; 4],
    control: usize,
    swatch: &str,
) -> String {
    let toggle = toggle_button(name, area, control);
    let swatch = format!(
        r##"{{"swatch_edge":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[22,11],"anchor_from":"right_middle","anchor_to":"right_middle","offset":[-3,0],"radius":2.5,"color":[0.9,0.9,0.92,0.9],"layer":2,"controls":[{{"swatch":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,9],"radius":2,"color":[1,1,1,1],"layer":1,"bindings":[{{"binding_name":"{swatch}","binding_name_override":"#color"}}]}}}}]}}}}"##
    );
    with_last_child(&toggle, &swatch)
}

/// A button showing a fixed caption, centred, on the plain track colour.
pub(in crate::panel) fn plain_button(
    name: &str,
    area: [f32; 4],
    control: usize,
    caption: &str,
) -> String {
    button(
        name,
        area,
        &format!("mod.control:{control}"),
        &[
            track_bg(None),
            centred(label(
                "text",
                caption,
                [area[2], 10.0],
                [0.0, text_y(area[3])],
                TEXT,
            )),
        ],
    )
}

/// A square colour button with a light edge; `colour` is fixed or bound via `#<binding>`.
pub(in crate::panel) fn colour_button(
    name: &str,
    area: [f32; 4],
    control: usize,
    colour: Result<[f32; 4], &str>,
) -> String {
    let fill = match colour {
        Ok(colour) => rect(
            "colour",
            r#"["100% - 2px","100% - 2px"]"#,
            [1.0, 1.0],
            colour,
            r#","layer":2"#,
        ),
        Err(binding) => rect(
            "colour",
            r#"["100% - 2px","100% - 2px"]"#,
            [1.0, 1.0],
            EMPTY_SLOT,
            &format!(
                r##","layer":2,"bindings":[{{"binding_name":"{binding}","binding_name_override":"#color"}}]"##
            ),
        ),
    };
    button(
        name,
        area,
        &format!("mod.control:{control}"),
        &[
            rect(
                "edge",
                r#"["100%","100%"]"#,
                [0.0, 0.0],
                [0.4, 0.41, 0.45, 1.0],
                r#","layer":1"#,
            ),
            fill,
        ],
    )
}

/// One slider row: its name, typed number box and bar, at `y` in a column whose bar ends 10
/// units before `track_end`.
pub(in crate::panel) fn slider_row(index: usize, y: f32) -> Vec<String> {
    let track = track_width();
    let (id, fill) = SLIDERS[index];
    let (mut empty, mut filled) = (String::new(), String::new());
    vector(&mut empty, SLIDER_TRACK);
    vector(&mut filled, fill);
    vec![
        label(
            &format!("{id}_label"),
            title(id),
            [VALUE_X - 12.0, 10.0],
            [10.0, y + text_y(ROW_HEIGHT)],
            TEXT,
        ),
        button(
            &format!("{id}_number"),
            [VALUE_X, y, VALUE_WIDTH, ROW_HEIGHT],
            &format!("mod.control:{}", FIRST_TYPE + index),
            &[
                rect(
                    "box",
                    r#"["100%","100%"]"#,
                    [0.0, 0.0],
                    NUMBER_BOX,
                    &format!(
                        r##","layer":1,"bindings":[{{"binding_name":"#{id}_box","binding_name_override":"#color"}}]"##
                    ),
                ),
                label(
                    "number",
                    &format!("#{id}_value"),
                    [VALUE_WIDTH - 6.0, 10.0],
                    [4.0, text_y(ROW_HEIGHT)],
                    TEXT,
                ),
            ],
        ),
        button(
            &format!("{id}_slider"),
            [TRACK_X, y, track, ROW_HEIGHT],
            &format!("mod.control:{index}"),
            &[
                format!(
                    r##"{{"track":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["100%",{TRACK_H}],"anchor_from":"left_middle","anchor_to":"left_middle","radius":{},"color":{empty},"layer":1}}}}"##,
                    TRACK_H / 2.0,
                ),
                format!(
                    r##"{{"fill":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["50%",{TRACK_H}],"anchor_from":"left_middle","anchor_to":"left_middle","radius":{},"color":{filled},"layer":2,"bindings":[{{"binding_name":"#{id}_fill","binding_name_override":"#size_binding_x"}}],"controls":[{{"knob_edge":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{},{}],"anchor_from":"right_middle","anchor_to":"right_middle","offset":[{},0],"radius":{},"color":[0,0,0,0.4],"layer":2,"controls":[{{"knob":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{SLIDER_KNOB},{SLIDER_KNOB}],"radius":{},"color":[1,1,1,1],"layer":1}}}}]}}}}]}}}}"##,
                    TRACK_H / 2.0,
                    SLIDER_KNOB + 2.0,
                    SLIDER_KNOB + 2.0,
                    SLIDER_KNOB / 2.0 + 1.0,
                    SLIDER_KNOB / 2.0 + 1.0,
                    SLIDER_KNOB / 2.0,
                ),
            ],
        ),
    ]
}

/// The colour wheel and its marker.
pub(in crate::panel) fn wheel() -> String {
    let [x, y, side] = WHEEL;
    // A disc of small cells: hue by angle, saturation by distance from the centre.
    let cells = (side / WHEEL_CELL) as usize;
    let mut children = Vec::with_capacity(cells * cells + 2);
    // A dark rim a little wider than the disc hides the stepped edge of its cells.
    children.push(
        rect(
            "rim",
            &format!("[{},{}]", side + 3.0, side + 3.0),
            [-1.5, -1.5],
            [0.1, 0.11, 0.14, 1.0],
            r#","layer":1"#,
        )
        .replace(
            r#""radius":3"#,
            &format!(r#""radius":{}"#, side / 2.0 + 1.5),
        ),
    );
    // Where a cell's centre falls: inside the disc or not, and how far from its middle.
    let place = |column: f32, row: f32| {
        let (dx, dy) = (column / cells as f32 - 0.5, 0.5 - row / cells as f32);
        (2.0 * (dx * dx + dy * dy).sqrt(), dy, dx)
    };
    let mut cell = |column: f32, row: f32, template: &str| {
        let (distance, dy, dx) = place(column, row);
        if distance > 1.0 {
            return;
        }
        let hue = dy.atan2(dx) / std::f32::consts::TAU;
        // Three decimals are finer than a colour step, and keep the document small: the
        // wheel is most of it, and the host refuses a document over 192 KiB.
        let [r, g, b] = hsv_to_rgb(hue, distance, 1.0).map(|c| short(f32::from(c) / 255.0));
        let name = children.len();
        children.push(format!(
            r#"{{"c{name}@bo.{template}":{{"offset":[{},{}],"color":[{r},{g},{b},1]}}}}"#,
            // Anchored at the centre, as every control is by default, which saves the two
            // anchor fields on each of the wheel's several hundred cells.
            column * WHEEL_CELL - side / 2.0,
            row * WHEEL_CELL - side / 2.0,
        ));
    };
    for row in 0..cells {
        for column in 0..cells {
            let (c, r) = (column as f32 + 0.5, row as f32 + 0.5);
            let inside = |dc: f32, dr: f32| place(c + dc, r + dr).0 <= 1.0;
            let here = inside(0.0, 0.0);
            // Cells on the rim, inside or out, are cut into quarters, so the edge of the disc
            // follows the circle more closely than whole cells can.
            let rim = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)]
                .iter()
                .any(|&(dc, dr)| inside(dc, dr) != here);
            if rim {
                for (qc, qr) in [(-0.25, -0.25), (0.25, -0.25), (-0.25, 0.25), (0.25, 0.25)] {
                    cell(c + qc, r + qr, "cell_q");
                }
            } else if here {
                cell(c, r, "cell");
            }
        }
    }
    // Marker: a dark ring around a light dot, moved by the `#wheel_marker` offset binding.
    children.push(format!(
        r##"{{"marker":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{m},{m}],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":{r},"color":[0.05,0.05,0.07,1],"layer":3,"bindings":[{{"binding_name":"#wheel_marker","binding_name_override":"#offset"}}],"controls":[{{"dot":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{d},{d}],"radius":{dr},"color":[1,1,1,1],"layer":4}}}}]}}}}"##,
        m = MARKER,
        r = MARKER / 2.0,
        d = MARKER - 3.0,
        dr = (MARKER - 3.0) / 2.0,
    ));
    button(
        "wheel",
        [x, y, side, side],
        &format!("mod.vector:{WHEEL_X}:{WHEEL_Y}"),
        &children,
    )
}

/// A full-card panel holding `children`, shown while the `#<name>_visible` binding is true.
pub(in crate::panel) fn shown_when(name: &str, children: &[String]) -> String {
    shown_by(name, name, children)
}

/// [`shown_when`] for a panel named `name` that follows the `#<binding>_visible` binding.
pub(in crate::panel) fn shown_by(name: &str, binding: &str, children: &[String]) -> String {
    format!(
        r##"{{"{name}":{{"type":"panel","size":["100%","100%"],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"bindings":[{{"binding_name":"#{binding}_visible","binding_name_override":"#visible"}}],"controls":[{}]}}}}"##,
        children.join(",")
    )
}

/// A small section caption with a thin rule running to the card's right edge.
pub(in crate::panel) fn caption(name: &str, text: &str, y: f32) -> String {
    let text = text.to_uppercase();
    let rule_x = 16.0 + 6.0 * text.len() as f32;
    let rule = rect(
        &format!("{name}_rule"),
        &format!("[{},1]", SETTINGS_W - 10.0 - rule_x),
        [rule_x, y + 5.0],
        RULE,
        r#","layer":2"#,
    );
    format!(
        "{},{rule}",
        label(name, &text, [120.0, 10.0], [10.0, y], CAPTION_TEXT)
    )
}
