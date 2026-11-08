//! The static JSON-UI document: the card, its pages and their controls.

use super::help::TIP_WIDTH;
use super::*;

/// `value` with at most three decimals and no trailing zeros: `0.5`, `1`, `0.984`.
fn short(value: f32) -> String {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() {
        "0".into()
    } else {
        text.into()
    }
}

/// Offset that centres a line of text in a control `height` tall.
fn text_y(height: f32) -> f32 {
    ((height - 10.0) / 2.0).floor()
}

/// A filled rounded rectangle.
pub(super) fn rect(
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
fn bg_template() -> String {
    let mut colour = String::new();
    vector(&mut colour, TRACK);
    format!(
        r#""bg":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["100%","100%"],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":3,"color":{colour},"layer":1}}"#
    )
}

/// A button's background, from [`bg_template`]; its colour follows the `binding` if one is given.
pub(super) fn track_bg(binding: Option<&str>) -> String {
    match binding {
        Some(binding) => format!(
            r##"{{"bg@bo.bg":{{"bindings":[{{"binding_name":"{binding}","binding_name_override":"#color"}}]}}}}"##
        ),
        None => String::from(r#"{"bg@bo.bg":{}}"#),
    }
}

/// A label reading its text from `#<binding>`, or literal text.
pub(super) fn label(
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
pub(super) fn button(name: &str, area: [f32; 4], action: &str, children: &[String]) -> String {
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
fn centred(label: String) -> String {
    label.replacen(
        r#""type":"label","#,
        r#""type":"label","text_alignment":"center","#,
        1,
    )
}

/// `button` (as [`button`] builds it) with `child` added after its other children.
fn with_last_child(button: &str, child: &str) -> String {
    let close = button
        .rfind("]}}")
        .expect("a button ends with its list of controls");
    format!("{},{child}{}", &button[..close], &button[close..])
}

/// A button whose background colour and caption come from `#<name>_bg` and `#<name>_text`.
pub(super) fn toggle_button(name: &str, area: [f32; 4], control: usize) -> String {
    bound_button(name, area, control, false)
}

/// [`toggle_button`] with the caption in the middle: for buttons that do something, such as Reset.
pub(super) fn action_button(name: &str, area: [f32; 4], control: usize) -> String {
    bound_button(name, area, control, true)
}

fn bound_button(name: &str, area: [f32; 4], control: usize, middle: bool) -> String {
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
pub(super) fn switch_button(name: &str, area: [f32; 4], control: usize, caption: &str) -> String {
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
pub(super) fn key_button(name: &str, area: [f32; 4], control: usize) -> String {
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
pub(super) fn value_button(name: &str, area: [f32; 4], control: usize) -> String {
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
pub(super) fn accent_button(name: &str, area: [f32; 4], control: usize, caption: &str) -> String {
    let (mut plain, mut accent) = (String::new(), String::new());
    vector(&mut plain, TRACK);
    vector(&mut accent, ACTIVE);
    plain_button(name, area, control, caption).replacen(&plain, &accent, 1)
}

/// A Line / Fill colour button: a toggle with a small swatch of that colour on its right.
pub(super) fn edit_button(name: &str, area: [f32; 4], control: usize, swatch: &str) -> String {
    let toggle = toggle_button(name, area, control);
    let swatch = format!(
        r##"{{"swatch_edge":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[22,11],"anchor_from":"right_middle","anchor_to":"right_middle","offset":[-3,0],"radius":2.5,"color":[0.9,0.9,0.92,0.9],"layer":2,"controls":[{{"swatch":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,9],"radius":2,"color":[1,1,1,1],"layer":1,"bindings":[{{"binding_name":"{swatch}","binding_name_override":"#color"}}]}}}}]}}}}"##
    );
    with_last_child(&toggle, &swatch)
}

/// A button showing a fixed caption, centred, on the plain track colour.
pub(super) fn plain_button(name: &str, area: [f32; 4], control: usize, caption: &str) -> String {
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
pub(super) fn colour_button(
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
pub(super) fn slider_row(index: usize, y: f32) -> Vec<String> {
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
pub(super) fn wheel() -> String {
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
pub(super) fn shown_when(name: &str, children: &[String]) -> String {
    shown_by(name, name, children)
}

/// [`shown_when`] for a panel named `name` that follows the `#<binding>_visible` binding.
pub(super) fn shown_by(name: &str, binding: &str, children: &[String]) -> String {
    format!(
        r##"{{"{name}":{{"type":"panel","size":["100%","100%"],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"bindings":[{{"binding_name":"#{binding}_visible","binding_name_override":"#visible"}}],"controls":[{}]}}}}"##,
        children.join(",")
    )
}

/// A small section caption with a thin rule running to the card's right edge.
pub(super) fn caption(name: &str, text: &str, y: f32) -> String {
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

/// The title, preview and Line / Fill colour buttons, shown above either view.
pub(super) fn top_controls() -> Vec<String> {
    let right = 10.0 + HALF + 4.0;
    vec![
        // A small accent beside the title gives the card its colour.
        rect("accent", "[3,12]", [10.0, 7.0], ACCENT, r#","layer":2"#),
        label(
            "title",
            "#title_text",
            [BACK_AREA[0] - 14.0 - TITLE_X, 10.0],
            [10.0 + TITLE_X, 7.0],
            TEXT,
        )
        .replacen(
            r#""type":"label","#,
            &format!(r#""type":"label","font_scale_factor":{TITLE_SCALE},"#),
            1,
        ),
        plain_button("close", CLOSE_AREA, CLOSE, "X"),
        rect(
            "title_rule",
            &format!("[{},1]", SETTINGS_W - 20.0),
            [10.0, 25.0],
            RULE,
            r#","layer":2"#,
        ),
        shown_when(
            "page",
            &[
                // One step back: colour tools to the page, the page to the start.
                plain_button("back", BACK_AREA, BACK, "< Back"),
                shown_when(
                    "fill_own",
                    &[
                        edit_button(
                            "edit_line",
                            [10.0, EDIT_Y, HALF, BUTTON_H],
                            EDIT_LINE,
                            "#line_swatch",
                        ),
                        edit_button(
                            "edit_fill",
                            [right, EDIT_Y, HALF, BUTTON_H],
                            EDIT_FILL,
                            "#fill_swatch",
                        ),
                    ],
                ),
                shown_when(
                    "fill_match",
                    &[edit_button(
                        "edit_line",
                        [10.0, EDIT_Y, SETTINGS_W - 20.0, BUTTON_H],
                        EDIT_LINE,
                        "#line_swatch",
                    )],
                ),
                shown_when(
                    "crosshair_top",
                    &[edit_button(
                        "crosshair_colour",
                        [10.0, EDIT_Y, HALF, BUTTON_H],
                        CROSS_COLOUR,
                        "#crosshair_swatch",
                    )],
                ),
                shown_when(
                    "crosshair_draw_button",
                    &[plain_button(
                        "crosshair_draw",
                        [TOP_RIGHT_X, EDIT_Y, HALF, BUTTON_H],
                        CROSS_DRAW,
                        "Draw your own",
                    )],
                ),
                shown_when(
                    "mob_top",
                    &[
                        edit_button(
                            "edit_line",
                            [10.0, EDIT_Y, HALF, BUTTON_H],
                            EDIT_LINE,
                            "#line_swatch",
                        ),
                        edit_button(
                            "edit_aim",
                            [right, EDIT_Y, HALF, BUTTON_H],
                            EDIT_AIM,
                            "#aim_swatch",
                        ),
                    ],
                ),
            ],
        ),
        shown_when("home", &home_controls()),
        // At the foot of the card, which follows the page's height.
        format!(
            r##"{{"hint_row":{{"type":"panel","size":[{SETTINGS_W},12],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"bindings":[{{"binding_name":"#hint_offset","binding_name_override":"#offset"}}],"controls":[{}]}}}}"##,
            label(
                "hint",
                "#hint_text",
                [SETTINGS_W - 20.0, 10.0],
                [10.0, 0.0],
                DIM
            )
        ),
    ]
}

/// The start page: a button for each page and the menu key.
pub(super) fn home_controls() -> Vec<String> {
    let page_button =
        |name: &str, y: f32, control: usize, icon: usize, title: &str, about: &str| {
            let badge = (HOME_BUTTON_H - BADGE) / 2.0;
            let mut children = vec![
                track_bg(None),
                // A dark tile with the page's icon on it, drawn in the page's current colour.
                rect(
                    "badge",
                    &format!("[{BADGE},{BADGE}]"),
                    [badge, badge],
                    ICON_BG,
                    r#","layer":2"#,
                )
                .replace(r#""radius":3"#, r#""radius":6"#),
            ];
            children.extend(home_icon(name, icon, badge));
            // The name and the line under it, centred together on the button.
            let name_y = ((HOME_BUTTON_H - 21.5) / 2.0).floor();
            children.push(label(
                "name",
                title,
                [HOME_BUTTON_W - 46.0, 10.0],
                [BADGE + 8.0, name_y],
                TEXT,
            ));
            children.push(label(
                "about",
                about,
                [HOME_BUTTON_W - 46.0, 10.0],
                [BADGE + 8.0, name_y + 11.5],
                DIM,
            ));
            children.push(label(
                "arrow",
                ">",
                [10.0, 10.0],
                [HOME_BUTTON_W - 14.0, text_y(HOME_BUTTON_H)],
                TEXT,
            ));
            button(
                name,
                [10.0, y, HOME_BUTTON_W, HOME_BUTTON_H],
                &format!("mod.control:{control}"),
                &children,
            )
        };
    vec![
        page_button(
            "open_block",
            HOME_BLOCK_Y,
            OPEN_BLOCK,
            0,
            "Block outline",
            HOME_ABOUT[0],
        ),
        page_button(
            "open_mob",
            HOME_MOB_Y,
            OPEN_MOB,
            1,
            "Mob hitbox",
            HOME_ABOUT[1],
        ),
        page_button(
            "open_counter",
            HOME_COUNTER_Y,
            OPEN_COUNTER,
            2,
            "Arrow counter",
            HOME_ABOUT[2],
        ),
        page_button(
            "open_cps",
            HOME_CPS_Y,
            OPEN_CPS,
            4,
            "CPS counter",
            HOME_ABOUT[4],
        ),
        page_button(
            "open_fps",
            HOME_FPS_Y,
            OPEN_FPS,
            8,
            "FPS counter",
            HOME_ABOUT[8],
        ),
        page_button(
            "open_zoom",
            HOME_ZOOM_Y,
            OPEN_ZOOM,
            3,
            "Zoom",
            HOME_ABOUT[3],
        ),
        page_button(
            "open_light",
            HOME_LIGHT_Y,
            OPEN_LIGHT,
            5,
            "Lighting",
            HOME_ABOUT[5],
        ),
        page_button(
            "open_jump",
            HOME_JUMP_Y,
            OPEN_JUMP,
            6,
            "Jump reset",
            HOME_ABOUT[6],
        ),
        home_switch("home_block", HOME_BLOCK_Y, HOME_BLOCK_ON),
        home_switch("home_mob", HOME_MOB_Y, HOME_MOB_ON),
        home_switch("home_counter", HOME_COUNTER_Y, HOME_COUNTER_ON),
        home_switch("home_cps", HOME_CPS_Y, HOME_CPS_ON),
        home_switch("home_fps", HOME_FPS_Y, HOME_FPS_ON),
        home_switch("home_zoom", HOME_ZOOM_Y, HOME_ZOOM_ON),
        home_switch("home_light", HOME_LIGHT_Y, HOME_LIGHT_ON),
        page_button(
            "open_crosshair",
            HOME_CROSS_Y,
            OPEN_CROSS,
            7,
            "Crosshair",
            HOME_ABOUT[7],
        ),
        home_switch("home_jump", HOME_JUMP_Y, HOME_JUMP_ON),
        home_switch("home_crosshair", HOME_CROSS_Y, HOME_CROSS_ON),
        caption("menu_caption", "Menu", HOME_MENU_CAPTION_Y),
        label(
            "key_title",
            "Menu key",
            [KEY_X - 12.0, 10.0],
            [10.0, HOME_KEY_Y + text_y(BUTTON_H)],
            TEXT,
        ),
        key_button(
            "menu_key",
            [KEY_X, HOME_KEY_Y, SETTINGS_W - 10.0 - KEY_X, BUTTON_H],
            MENU_KEY,
        ),
        // How much of a frame's fuel the mod uses, to see how close it runs to the limit.
        label(
            "fuel",
            "#fuel_text",
            [SETTINGS_W - 20.0, 10.0],
            [10.0, HOME_KEY_Y + BUTTON_H + 4.0],
            DIM,
        ),
    ]
}

/// A start page's on/off switch for the page button at `y`: a small switch on a button of its
/// own, right of the page button. Bound by `#<name>_bg` and `#<name>_knob` like the others.
fn home_switch(name: &str, y: f32, control: usize) -> String {
    let mut off = String::new();
    vector(&mut off, SWITCH_OFF);
    let at = [
        (HOME_SWITCH_W - PILL_W) / 2.0,
        (HOME_BUTTON_H - PILL_H) / 2.0,
    ];
    let pill = format!(
        r##"{{"pill":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{PILL_W},{PILL_H}],"anchor_from":"top_left","anchor_to":"top_left","offset":[{},{}],"radius":{},"color":{off},"layer":2,"bindings":[{{"binding_name":"#{name}_bg","binding_name_override":"#color"}}],"controls":[{{"knob":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{KNOB},{KNOB}],"anchor_from":"left_middle","anchor_to":"left_middle","offset":[{KNOB_OFF},0],"radius":{},"color":[1,1,1,1],"layer":1,"bindings":[{{"binding_name":"#{name}_knob","binding_name_override":"#offset"}}]}}}}]}}}}"##,
        at[0],
        at[1],
        PILL_H / 2.0,
        KNOB / 2.0,
    );
    button(
        name,
        [
            10.0 + HOME_BUTTON_W + HOME_GAP,
            y,
            HOME_SWITCH_W,
            HOME_BUTTON_H,
        ],
        &format!("mod.control:{control}"),
        &[track_bg(None), pill],
    )
}

/// The icon of a start page button, drawn from a few rectangles on the `BADGE` tile at `(at, at)`:
/// a block (two squares), a mob in its hitbox, an arrow and a magnifier. The strokes take the
/// colour of `#<name>_color`; cut-outs are the tile's colour.
fn home_icon(name: &str, kind: usize, at: f32) -> Vec<String> {
    let piece = |id: &str, x: f32, y: f32, w: f32, h: f32, radius: f32, stroke: bool, layer: u8| {
        let extra = if stroke {
            format!(
                r##","layer":{layer},"bindings":[{{"binding_name":"#{name}_color","binding_name_override":"#color"}}]"##
            )
        } else {
            format!(r#","layer":{layer}"#)
        };
        rect(
            id,
            &format!("[{w},{h}]"),
            [at + x, at + y],
            if stroke { [1.0; 4] } else { ICON_BG },
            &extra,
        )
        .replace(r#""radius":3"#, &format!(r#""radius":{radius}"#))
    };
    match kind {
        // Two outlined squares, the front one over the back one: a block.
        0 => vec![
            piece("i0", 8.0, 3.0, 11.0, 11.0, 1.5, true, 3),
            piece("i1", 10.0, 5.0, 7.0, 7.0, 0.5, false, 4),
            piece("i2", 3.0, 8.0, 11.0, 11.0, 1.5, true, 5),
            piece("i3", 5.0, 10.0, 7.0, 7.0, 0.5, false, 6),
        ],
        // A figure inside its hitbox.
        1 => vec![
            piece("i0", 6.0, 3.0, 10.0, 16.0, 1.5, true, 3),
            piece("i1", 8.0, 5.0, 6.0, 12.0, 0.5, false, 4),
            piece("i2", 9.0, 6.0, 4.0, 4.0, 2.0, true, 5),
            piece("i3", 8.0, 11.0, 6.0, 4.0, 1.0, true, 5),
        ],
        // An arrow pointing right, in steps of two units.
        2 => vec![
            piece("i0", 4.0, 10.0, 14.0, 2.0, 0.5, true, 3),
            piece("i1", 13.0, 6.0, 2.0, 2.0, 0.0, true, 3),
            piece("i2", 15.0, 8.0, 2.0, 2.0, 0.0, true, 3),
            piece("i3", 13.0, 14.0, 2.0, 2.0, 0.0, true, 3),
            piece("i4", 15.0, 12.0, 2.0, 2.0, 0.0, true, 3),
        ],
        // A sun: a ring with four rays.
        5 => vec![
            piece("i0", 6.0, 6.0, 10.0, 10.0, 5.0, true, 3),
            piece("i1", 8.0, 8.0, 6.0, 6.0, 3.0, false, 4),
            piece("i2", 10.5, 2.0, 1.0, 3.0, 0.0, true, 3),
            piece("i3", 10.5, 17.0, 1.0, 3.0, 0.0, true, 3),
            piece("i4", 2.0, 10.5, 3.0, 1.0, 0.0, true, 3),
            piece("i5", 17.0, 10.5, 3.0, 1.0, 0.0, true, 3),
        ],
        // An arrow pointing up over a ground line: a jump.
        6 => vec![
            piece("i0", 10.0, 4.0, 2.0, 2.0, 0.0, true, 3),
            piece("i1", 8.0, 6.0, 2.0, 2.0, 0.0, true, 3),
            piece("i2", 12.0, 6.0, 2.0, 2.0, 0.0, true, 3),
            piece("i3", 6.0, 8.0, 2.0, 2.0, 0.0, true, 3),
            piece("i4", 14.0, 8.0, 2.0, 2.0, 0.0, true, 3),
            piece("i5", 10.0, 6.0, 2.0, 10.0, 0.5, true, 3),
            piece("i6", 5.0, 18.0, 12.0, 1.5, 0.5, true, 3),
        ],
        // A crosshair: four short lines round a dot.
        7 => vec![
            piece("i0", 10.0, 3.0, 2.0, 5.0, 0.5, true, 3),
            piece("i1", 10.0, 14.0, 2.0, 5.0, 0.5, true, 3),
            piece("i2", 3.0, 10.0, 5.0, 2.0, 0.5, true, 3),
            piece("i3", 14.0, 10.0, 5.0, 2.0, 0.5, true, 3),
            piece("i4", 10.0, 10.0, 2.0, 2.0, 0.5, true, 3),
        ],
        // A mouse: an outlined body with the button split.
        4 => vec![
            piece("i0", 6.0, 3.0, 10.0, 16.0, 5.0, true, 3),
            piece("i1", 7.5, 4.5, 7.0, 13.0, 3.5, false, 4),
            piece("i2", 10.5, 4.5, 1.0, 5.0, 0.0, true, 5),
            piece("i3", 7.5, 9.5, 7.0, 1.0, 0.0, true, 5),
        ],
        // Three bars rising to the right: frames per second.
        8 => vec![
            piece("i0", 4.0, 12.0, 3.0, 6.0, 0.5, true, 3),
            piece("i1", 9.5, 8.0, 3.0, 10.0, 0.5, true, 3),
            piece("i2", 15.0, 4.0, 3.0, 14.0, 0.5, true, 3),
        ],
        // A magnifier: a ring and a handle.
        _ => vec![
            piece("i0", 4.0, 4.0, 12.0, 12.0, 6.0, true, 3),
            piece("i1", 6.0, 6.0, 8.0, 8.0, 4.0, false, 4),
            piece("i2", 14.0, 14.0, 2.0, 2.0, 0.0, true, 3),
            piece("i3", 16.0, 16.0, 2.0, 2.0, 0.0, true, 3),
        ],
    }
}

/// The Arrow counter page: on/off, bow only, moving, size and reset.
pub(super) fn counter_controls() -> Vec<String> {
    let full = SETTINGS_W - 20.0;
    vec![
        switch_button(
            "counter",
            [10.0, COUNTER_ON_Y, full, BUTTON_H],
            COUNTER,
            "Show counter",
        ),
        switch_button(
            "counter_bow",
            [10.0, COUNTER_BOW_Y, full, BUTTON_H],
            COUNTER_BOW,
            "Only with a bow",
        ),
        caption("counter_caption", "Place", COUNTER_CAPTION_Y),
        plain_button(
            "counter_move",
            [10.0, COUNTER_MOVE_Y, full, BUTTON_H],
            COUNTER_MOVE,
            "Move and resize on screen",
        ),
        value_button(
            "counter_size",
            [10.0, COUNTER_SIZE_Y, HIGHLIGHT_W, BUTTON_H],
            COUNTER_SIZE,
        ),
        plain_button(
            "counter_smaller",
            [
                SETTINGS_W - 10.0 - 2.0 * STEP_W - 4.0,
                COUNTER_SIZE_Y,
                STEP_W,
                BUTTON_H,
            ],
            COUNTER_SMALLER,
            "-",
        ),
        plain_button(
            "counter_bigger",
            [SETTINGS_W - 10.0 - STEP_W, COUNTER_SIZE_Y, STEP_W, BUTTON_H],
            COUNTER_BIGGER,
            "+",
        ),
        action_button(
            "counter_reset",
            [10.0, COUNTER_RESET_Y, full, BUTTON_H],
            COUNTER_RESET,
        ),
    ]
}

/// The Lighting page: Fullbright and its key.
pub(super) fn lighting_controls() -> Vec<String> {
    vec![
        switch_button(
            "fullbright",
            [10.0, LIGHT_ON_Y, SETTINGS_W - 20.0, BUTTON_H],
            FULLBRIGHT,
            "Fullbright",
        ),
        label(
            "fullbright_key_title",
            "Fullbright key",
            [KEY_X - 12.0, 10.0],
            [10.0, LIGHT_KEY_Y + text_y(BUTTON_H)],
            TEXT,
        ),
        key_button(
            "fullbright_key",
            [KEY_X, LIGHT_KEY_Y, SETTINGS_W - 10.0 - KEY_X, BUTTON_H],
            FULLBRIGHT_KEY,
        ),
    ]
}

/// The CPS page: on/off, right clicks, leading zeros, moving, size and reset.
pub(super) fn cps_controls() -> Vec<String> {
    let full = SETTINGS_W - 20.0;
    let mut page = vec![
        switch_button(
            "cps_on",
            [10.0, CPS_ON_Y, full, BUTTON_H],
            CPS_ON,
            "Show CPS",
        ),
        switch_button(
            "cps_right",
            [10.0, CPS_RIGHT_Y, full, BUTTON_H],
            CPS_RIGHT,
            "Show right clicks",
        ),
        switch_button(
            "cps_zeros",
            [10.0, CPS_ZEROS_Y, full, BUTTON_H],
            CPS_ZEROS,
            "Leading zeros",
        ),
        caption("cps_caption", "Place", CPS_CAPTION_Y),
        plain_button(
            "cps_move",
            [10.0, CPS_MOVE_Y, full, BUTTON_H],
            CPS_MOVE,
            "Move and resize on screen",
        ),
    ];
    page.extend(stepper(
        "cps_size",
        CPS_SIZE_Y,
        CPS_SIZE,
        CPS_SMALLER,
        CPS_BIGGER,
    ));
    page.push(action_button(
        "cps_reset",
        [10.0, CPS_RESET_Y, full, BUTTON_H],
        CPS_RESET,
    ));
    page
}

/// The FPS counter page: on/off, moving, size and reset.
pub(super) fn fps_controls() -> Vec<String> {
    let full = SETTINGS_W - 20.0;
    let mut page = vec![
        switch_button(
            "fps_on",
            [10.0, FPS_ON_Y, full, BUTTON_H],
            FPS_ON,
            "Show FPS",
        ),
        caption("fps_caption", "Place", FPS_CAPTION_Y),
        plain_button(
            "fps_move",
            [10.0, FPS_MOVE_Y, full, BUTTON_H],
            FPS_MOVE,
            "Move and resize on screen",
        ),
    ];
    page.extend(stepper(
        "fps_size",
        FPS_SIZE_Y,
        FPS_SIZE,
        FPS_SMALLER,
        FPS_BIGGER,
    ));
    page.push(action_button(
        "fps_reset",
        [10.0, FPS_RESET_Y, full, BUTTON_H],
        FPS_RESET,
    ));
    page
}

/// The Jump reset page: on/off, the perfect window and a short note.
pub(super) fn jump_controls() -> Vec<String> {
    let mut page = vec![
        switch_button(
            "jump_on",
            [10.0, JUMP_ON_Y, SETTINGS_W - 20.0, BUTTON_H],
            JUMP_ON,
            "Show timing",
        ),
        caption("jump_caption", "Timing", JUMP_CAPTION_Y),
    ];
    page.extend(stepper(
        "jump_window",
        JUMP_WINDOW_Y,
        JUMP_WINDOW,
        JUMP_WINDOW_LESS,
        JUMP_WINDOW_MORE,
    ));
    for (index, line) in JUMP_NOTE.iter().enumerate() {
        page.push(label(
            &format!("jump_note_{index}"),
            line,
            [SETTINGS_W - 20.0, 10.0],
            [10.0, JUMP_NOTE_Y + 12.0 * index as f32],
            DIM,
        ));
    }
    page
}

/// The Crosshair page's settings, below its colour button: the shape's values, then opacity,
/// the switches and reset.
pub(super) fn crosshair_controls() -> Vec<String> {
    let right = 10.0 + HALF + 4.0;
    let mut page = vec![
        caption("crosshair_shape_caption", "Shape", LOOK_CAPTION_Y),
        caption(
            "crosshair_options_caption",
            "Options",
            CROSS_OPTIONS_CAPTION_Y,
        ),
    ];
    for (name, _, control, y) in CROSS_VALUES {
        page.extend(stepper(name, y, control, control + 1, control + 2));
    }
    page.push(switch_button(
        "crosshair_on",
        [10.0, CROSS_TOGGLES_Y, HALF, BUTTON_H],
        CROSS_ON,
        "Show",
    ));
    page.push(switch_button(
        "crosshair_outline",
        [right, CROSS_TOGGLES_Y, HALF, BUTTON_H],
        CROSS_OUTLINE,
        "Outline",
    ));
    page.push(action_button(
        "crosshair_reset",
        [10.0, CROSS_RESET_Y, SETTINGS_W - 20.0, BUTTON_H],
        CROSS_RESET,
    ));
    page
}

/// The drawing view: the grid (the HUD's `drawgrid`, embedded), a line of help, and Mirror
/// beside Clear.
pub(super) fn draw_controls() -> Vec<String> {
    let [x, y, side] = DRAW_GRID;
    let right = 10.0 + HALF + 4.0;
    let mut page = vec![
        // The HUD's `drawgrid`, at its own size: the mod replaces it as the player draws.
        format!(
            r##"{{"draw_grid":{{"type":"custom","renderer":"cinnabar_hud_control","control":"drawgrid","size":[{side},{side}],"offset":[{x},{y}],"anchor_from":"top_left","anchor_to":"top_left","layer":1}}}}"##
        ),
        label(
            "draw_note",
            "#draw_note_text",
            [SETTINGS_W - 20.0, 10.0],
            [10.0, DRAW_NOTE_Y],
            DIM,
        ),
        switch_button(
            "crosshair_mirror",
            [10.0, DRAW_BUTTONS_Y, HALF, BUTTON_H],
            DRAW_MIRROR,
            "Mirror",
        ),
        action_button(
            "crosshair_clear",
            [right, DRAW_BUTTONS_Y, HALF, BUTTON_H],
            DRAW_CLEAR,
        ),
    ];
    // The resolution, beside the Colour button.
    let less_x = TOP_RIGHT_X + RES_VALUE_W + 4.0;
    page.push(value_button(
        "crosshair_res",
        [TOP_RIGHT_X, EDIT_Y, RES_VALUE_W, BUTTON_H],
        DRAW_RES,
    ));
    page.push(plain_button(
        "crosshair_res_less",
        [less_x, EDIT_Y, STEP_W, BUTTON_H],
        DRAW_RES + 1,
        "-",
    ));
    page.push(plain_button(
        "crosshair_res_more",
        [less_x + STEP_W + 4.0, EDIT_Y, STEP_W, BUTTON_H],
        DRAW_RES + 2,
        "+",
    ));
    page
}

/// A value row: the value on a button that resets it, beside - and + buttons.
fn stepper(name: &str, y: f32, value: usize, less: usize, more: usize) -> Vec<String> {
    vec![
        value_button(name, [10.0, y, HIGHLIGHT_W, BUTTON_H], value),
        plain_button(
            &format!("{name}_less"),
            [SETTINGS_W - 10.0 - 2.0 * STEP_W - 4.0, y, STEP_W, BUTTON_H],
            less,
            "-",
        ),
        plain_button(
            &format!("{name}_more"),
            [SETTINGS_W - 10.0 - STEP_W, y, STEP_W, BUTTON_H],
            more,
            "+",
        ),
    ]
}

/// The Zoom page: on/off, the zoom key, how it works and three values.
pub(super) fn zoom_controls() -> Vec<String> {
    let full = SETTINGS_W - 20.0;
    let mut page = vec![
        switch_button(
            "zoom",
            [10.0, ZOOM_ON_Y, full, BUTTON_H],
            ZOOM_ON,
            "Enable zoom",
        ),
        label(
            "zoom_key_title",
            "Zoom key",
            [KEY_X - 12.0, 10.0],
            [10.0, ZOOM_KEY_Y + text_y(BUTTON_H)],
            TEXT,
        ),
        key_button(
            "zoom_key",
            [KEY_X, ZOOM_KEY_Y, SETTINGS_W - 10.0 - KEY_X, BUTTON_H],
            ZOOM_KEY,
        ),
        switch_button(
            "zoom_mode",
            [10.0, ZOOM_MODE_Y, full, BUTTON_H],
            ZOOM_MODE,
            "Press to toggle",
        ),
        switch_button(
            "zoom_scroll",
            [10.0, ZOOM_SCROLL_Y, full, BUTTON_H],
            ZOOM_SCROLL,
            "Scroll zoom",
        ),
        switch_button(
            "zoom_sens",
            [10.0, ZOOM_SENS_Y, full, BUTTON_H],
            ZOOM_SENS,
            "Relative sensitivity",
        ),
        caption("zoom_caption", "Feel", ZOOM_CAPTION_Y),
    ];
    page.extend(stepper(
        "zoom_level",
        ZOOM_LEVEL_Y,
        ZOOM_LEVEL,
        ZOOM_LEVEL_LESS,
        ZOOM_LEVEL_MORE,
    ));
    page.extend(stepper(
        "zoom_time",
        ZOOM_TIME_Y,
        ZOOM_TIME,
        ZOOM_TIME_LESS,
        ZOOM_TIME_MORE,
    ));
    page.extend(stepper(
        "zoom_step",
        ZOOM_STEP_Y,
        ZOOM_STEP,
        ZOOM_STEP_LESS,
        ZOOM_STEP_MORE,
    ));
    page.push(action_button(
        "zoom_reset",
        [10.0, ZOOM_RESET_Y, full, BUTTON_H],
        ZOOM_RESET,
    ));
    page
}

/// The guide shown while the counter is moved, with Reset and Done buttons: one copy at the
/// top of the screen and one at the bottom, so it never covers the counter.
pub(super) fn moving_guides() -> String {
    format!(
        "{},{}",
        moving_guide("guide_top", "top_middle", 10.0),
        moving_guide("guide_bottom", "bottom_middle", -10.0)
    )
}

fn moving_guide(name: &str, anchor: &str, offset_y: f32) -> String {
    let lines = GUIDE_LINES;
    // The card's look: a light edge, the dark body and the accent beside the text.
    let mut children = vec![
        rect(
            "guide_border",
            &format!("[{},{}]", GUIDE_W + 2.0, GUIDE_H + 2.0),
            [-1.0, -1.0],
            BORDER,
            r#","layer":1"#,
        )
        .replace(r#""radius":3"#, r#""radius":8"#),
        rect(
            "guide_bg",
            r#"["100%","100%"]"#,
            [0.0, 0.0],
            [0.07, 0.08, 0.1, 0.96],
            r#","layer":1"#,
        )
        .replace(r#""radius":3"#, r#""radius":7"#),
        rect(
            "guide_accent",
            "[3,34]",
            [10.0, 8.0],
            ACCENT,
            r#","layer":2"#,
        ),
    ];
    for (index, line) in lines.iter().enumerate() {
        children.push(label(
            &format!("guide_{index}"),
            line,
            [GUIDE_W - GUIDE_TEXT_X - 10.0, 10.0],
            [GUIDE_TEXT_X, 8.0 + 12.0 * index as f32],
            if index == 0 { TEXT } else { DIM },
        ));
    }
    children.push(plain_button(
        "guide_reset",
        [GUIDE_W - 116.0, GUIDE_H - 24.0, 50.0, BUTTON_H],
        COUNTER_RESET,
        "Reset",
    ));
    // Done is the main action here.
    children.push(accent_button(
        "counter_done",
        [GUIDE_W - 60.0, GUIDE_H - 24.0, 50.0, BUTTON_H],
        COUNTER_DONE,
        "Done",
    ));
    format!(
        r##"{{"{name}":{{"type":"panel","size":[{GUIDE_W},{GUIDE_H}],"offset":[0,{offset_y}],"anchor_from":"{anchor}","anchor_to":"{anchor}","layer":2,"bindings":[{{"binding_name":"#{name}_visible","binding_name_override":"#visible"}}],"controls":[{}]}}}}"##,
        children.join(",")
    )
}

/// The settings view, below the Line / Fill colour buttons while the colour tools are hidden.
pub(super) fn settings_column() -> Vec<String> {
    let right = 10.0 + HALF + 4.0;
    let mut column = Vec::new();
    for index in 3..SLIDERS.len() - 1 {
        column.extend(slider_row(index, slider_place(index).1));
    }
    // Mob hitboxes have no fill.
    let fill = SLIDERS.len() - 1;
    column.push(shown_by(
        "fill_row",
        "block_only",
        &slider_row(fill, slider_place(fill).1),
    ));
    column.push(caption("look_caption", "Look", LOOK_CAPTION_Y));
    let mut options = vec![caption("options_caption", "Options", OPTIONS_CAPTION_Y)];
    // Mob hitboxes have no pulse: their Rainbow switch takes the whole row.
    options.push(shown_by(
        "block_effects",
        "block_only",
        &[
            switch_button(
                "rainbow",
                [10.0, TOGGLES_Y, HALF, BUTTON_H],
                RAINBOW,
                "Rainbow",
            ),
            switch_button("pulse", [right, TOGGLES_Y, HALF, BUTTON_H], PULSE, "Pulse"),
        ],
    ));
    options.push(shown_by(
        "mob_effects",
        "mob_only",
        &[switch_button(
            "rainbow",
            [10.0, TOGGLES_Y, SETTINGS_W - 20.0, BUTTON_H],
            RAINBOW,
            "Rainbow",
        )],
    ));
    // Blocks: Fill match beside Reset. Mobs: All mobs beside Reset.
    options.push(shown_by(
        "block_bottom",
        "block_only",
        &[switch_button(
            "fill_match",
            [10.0, BOTTOM_Y, HALF, BUTTON_H],
            FILL_MATCH,
            "Match fill",
        )],
    ));
    options.push(shown_by(
        "mob_bottom",
        "mob_only",
        &[switch_button(
            "show_all_mobs",
            [10.0, BOTTOM_Y, HALF, BUTTON_H],
            SHOW_ALL,
            "All mobs",
        )],
    ));
    options.push(action_button(
        "reset",
        [right, BOTTOM_Y, HALF, BUTTON_H],
        RESET,
    ));
    // Only blocks have the game's highlight.
    options.push(shown_when(
        "block_only",
        &[
            toggle_button(
                "hide_game_highlight",
                [10.0, HIGHLIGHT_Y, HIGHLIGHT_W, BUTTON_H],
                HIDE_HIGHLIGHT,
            ),
            plain_button(
                "highlight_less",
                [
                    SETTINGS_W - 10.0 - 2.0 * STEP_W - 4.0,
                    HIGHLIGHT_Y,
                    STEP_W,
                    BUTTON_H,
                ],
                HIGHLIGHT_LESS,
                "-",
            ),
            plain_button(
                "highlight_more",
                [SETTINGS_W - 10.0 - STEP_W, HIGHLIGHT_Y, STEP_W, BUTTON_H],
                HIGHLIGHT_MORE,
                "+",
            ),
        ],
    ));
    // Moved up together (`#options_offset`) where a page has fewer bars.
    column.push(format!(
        r##"{{"options":{{"type":"panel","size":["100%","100%"],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"bindings":[{{"binding_name":"#options_offset","binding_name_override":"#offset"}}],"controls":[{}]}}}}"##,
        options.join(",")
    ));
    column
}

/// The colour column: RGB, wheel, hex code, quick and saved colours for the edited colour.
pub(super) fn colour_column() -> Vec<String> {
    let mut column: Vec<String> = Vec::new();
    for index in 0..3 {
        column.extend(slider_row(index, slider_place(index).1));
    }
    column.push(wheel());
    let hex_w = SETTINGS_W - 10.0 - HEX_X;
    column.push(label(
        "hex_title",
        "HEX CODE",
        [hex_w, 10.0],
        [HEX_X, HEX_Y],
        CAPTION_TEXT,
    ));
    column.push(button(
        "hex_field",
        [HEX_X, HEX_Y + 12.0, hex_w, BUTTON_H],
        &format!("mod.control:{}", FIRST_TYPE + SLIDERS.len()),
        &[
            rect(
                "field_bg",
                r#"["100%","100%"]"#,
                [0.0, 0.0],
                TRACK,
                r##","layer":1,"bindings":[{"binding_name":"#hex_box","binding_name_override":"#color"}]"##,
            ),
            label("field_text", "#hex_text", [hex_w - 8.0, 10.0], [5.0, text_y(BUTTON_H)], TEXT),
        ],
    ));
    column.push(label(
        "type_hint",
        "Type 0-9 and A-F,",
        [hex_w, 10.0],
        [HEX_X, HEX_Y + 35.0],
        DIM,
    ));
    column.push(label(
        "enter_hint",
        "then press Enter",
        [hex_w, 10.0],
        [HEX_X, HEX_Y + 47.0],
        DIM,
    ));

    column.push(label(
        "swatch_title",
        "QUICK COLOURS",
        [120.0, 10.0],
        [10.0, SWATCH_Y],
        CAPTION_TEXT,
    ));
    let swatch_width = (SETTINGS_W - 20.0) / SWATCHES.len() as f32;
    for (index, (name, rgb)) in SWATCHES.iter().enumerate() {
        let [r, g, b] = rgb.map(|c| f32::from(c) / 255.0);
        column.push(colour_button(
            &format!("swatch_{}", name.to_lowercase()),
            [
                10.0 + swatch_width * index as f32,
                SWATCH_Y + 12.0,
                swatch_width - 3.0,
                BUTTON_H,
            ],
            FIRST_SWATCH + index,
            Ok([r, g, b, 1.0]),
        ));
    }
    column.push(label(
        "favourite_title",
        "SAVED COLOURS",
        [120.0, 10.0],
        [10.0, FAVOURITE_Y],
        CAPTION_TEXT,
    ));
    for slot in 0..FAVOURITES {
        column.push(colour_button(
            &format!("favourite_{slot}"),
            [
                10.0 + 34.0 * slot as f32,
                FAVOURITE_Y + 12.0,
                30.0,
                BUTTON_H,
            ],
            FIRST_FAVOURITE + slot,
            Err(&format!("#favourite_{slot}_color")),
        ));
    }
    let save_x = 10.0 + 34.0 * FAVOURITES as f32;
    column.push(plain_button(
        "save_favourite",
        [
            save_x,
            FAVOURITE_Y + 12.0,
            SETTINGS_W - 10.0 - save_x,
            BUTTON_H,
        ],
        SAVE_FAVOURITE,
        "Save colour",
    ));
    column
}

/// A lighter inset panel behind a group of rows, from `top` to `bottom` on the card.
fn section(name: &str, top: f32, bottom: f32) -> String {
    rect(
        name,
        &format!("[{},{}]", SETTINGS_W - 2.0 * SECTION_PAD, bottom - top),
        [SECTION_PAD, top],
        SECTION,
        r#","layer":1"#,
    )
    .replace(r#""radius":3"#, r#""radius":6"#)
}

/// One side of the focus frame, moved by `#focus_<side>` and sized along its length by
/// `#focus_wide` (top and bottom) or `#focus_tall` (left and right).
fn focus_edge(name: &str, side: &str, vertical: bool) -> String {
    let size = if vertical {
        format!("[{FOCUS_LINE},10]")
    } else {
        format!("[10,{FOCUS_LINE}]")
    };
    let (length, axis) = if vertical {
        ("#focus_tall", "#size_binding_y_absolute")
    } else {
        ("#focus_wide", "#size_binding_x_absolute")
    };
    rect(
        name,
        &size,
        [0.0, 0.0],
        FOCUS,
        &format!(
            r##","layer":2,"bindings":[{{"binding_name":"#focus_visible","binding_name_override":"#visible"}},{{"binding_name":"#focus_{side}","binding_name_override":"#offset"}},{{"binding_name":"{length}","binding_name_override":"{axis}"}}]"##
        ),
    )
    .replace(r#""radius":3"#, r#""radius":0.75"#)
}

/// `panel` drawn right above the card's background. Layers add up through parents, so a panel
/// of layer 2 holding a rectangle of layer 1 would be drawn at 3, above the focus frame and
/// the hover highlight (layer 2), and hide them.
fn behind(panel: String) -> String {
    panel.replacen(r#""layer":2"#, r#""layer":0"#, 1)
}

/// The panels behind each page's settings. They sit below the hover highlight and the focus
/// frame, which a panel on top of them would hide.
fn sections() -> String {
    let pad = 4.0;
    let first = LOOK_CAPTION_Y - pad;
    let options = OPTIONS_CAPTION_Y - pad;
    let row_end = |y: f32| y + BUTTON_H + pad;
    let fill_row = ROW_Y + ROW_STEP * 2.0;
    let block = behind(shown_by(
        "sections_block",
        "block_settings",
        &[
            section("look", first, row_end(fill_row)),
            section("options", options, row_end(HIGHLIGHT_Y)),
        ],
    ));
    // The mob page has no Fill row and no highlight fix, and its options sit one row higher.
    let mob = behind(shown_by(
        "sections_mob",
        "mob_settings",
        &[
            section("look", first, row_end(ROW_Y + ROW_STEP)),
            section(
                "options",
                options - MOB_OPTIONS_SHIFT,
                row_end(BOTTOM_Y - MOB_OPTIONS_SHIFT),
            ),
        ],
    ));
    let counter = behind(shown_when(
        "counter_page",
        &[
            section("display", EDIT_Y - pad, row_end(COUNTER_BOW_Y)),
            section("place", COUNTER_CAPTION_Y - pad, row_end(COUNTER_RESET_Y)),
        ],
    ));
    let cps = behind(shown_when(
        "cps_page",
        &[
            section("display", EDIT_Y - pad, row_end(CPS_ZEROS_Y)),
            section("place", CPS_CAPTION_Y - pad, row_end(CPS_RESET_Y)),
        ],
    ));
    let fps = behind(shown_when(
        "fps_page",
        &[
            section("display", EDIT_Y - pad, row_end(FPS_ON_Y)),
            section("place", FPS_CAPTION_Y - pad, row_end(FPS_RESET_Y)),
        ],
    ));
    let light = behind(shown_when(
        "light_page",
        &[section("lighting", EDIT_Y - pad, row_end(LIGHT_KEY_Y))],
    ));
    let jump = behind(shown_when(
        "jump_page",
        &[
            section("display", EDIT_Y - pad, row_end(JUMP_ON_Y)),
            section("timing", JUMP_CAPTION_Y - pad, row_end(JUMP_WINDOW_Y)),
        ],
    ));
    let crosshair = behind(shown_when(
        "crosshair_page",
        &[
            section("shape", LOOK_CAPTION_Y - pad, row_end(CROSS_GAP_Y)),
            section(
                "options",
                CROSS_OPTIONS_CAPTION_Y - pad,
                row_end(CROSS_RESET_Y),
            ),
        ],
    ));
    let zoom = behind(shown_when(
        "zoom_page",
        &[
            section("controls", EDIT_Y - pad, row_end(ZOOM_SENS_Y)),
            section("feel", ZOOM_CAPTION_Y - pad, row_end(ZOOM_RESET_Y)),
        ],
    ));
    // The colour tools: the picker (bars, wheel, hex code) and the colours to pick from.
    let colour = behind(shown_by(
        "sections_colour",
        "colour",
        &[
            section("picker", RGB_Y - pad, WHEEL[1] + WHEEL[2] + pad),
            section(
                "palette",
                SWATCH_Y - pad,
                FAVOURITE_Y + 12.0 + BUTTON_H + pad,
            ),
        ],
    ));
    let home = behind(shown_when(
        "home",
        &[section(
            "menu",
            HOME_MENU_CAPTION_Y - pad,
            row_end(HOME_KEY_Y),
        )],
    ));
    format!(
        "{block},{mob},{counter},{cps},{fps},{zoom},{light},{jump},{crosshair},{colour},{home}"
    )
}

/// The static JSON-UI document; everything that changes comes from bindings. The card is pinned
/// to the right edge of the screen; below its top controls it shows either the settings or the
/// colour tools.
pub(super) fn document() -> String {
    let card = [
        // A light edge round the card, so it stands out against any scenery.
        rect(
            "border",
            &format!("[{},{}]", SETTINGS_W + 2.0, CARD_H + 2.0),
            [-1.0, -1.0],
            BORDER,
            r##","layer":1,"bindings":[{"binding_name":"#card_border_height","binding_name_override":"#size_binding_y_absolute"}]"##,
        )
        .replace(r#""radius":3"#, r#""radius":8"#),
        // As tall as the page shown (`#card_height`), from the top of the card.
        rect(
            "background",
            &format!("[{SETTINGS_W},{CARD_H}]"),
            [0.0, 0.0],
            [0.07, 0.08, 0.1, 0.96],
            r##","layer":1,"bindings":[{"binding_name":"#card_height","binding_name_override":"#size_binding_y_absolute"}]"##,
        )
        .replace(r#""radius":3"#, r#""radius":7"#),
        sections(),
        rect(
            "hover",
            "[10,10]",
            [0.0, 0.0],
            [1.0, 1.0, 1.0, 0.08],
            r##","layer":2,"bindings":[{"binding_name":"#hover_visible","binding_name_override":"#visible"},{"binding_name":"#hover_offset","binding_name_override":"#offset"},{"binding_name":"#hover_width","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#hover_height","binding_name_override":"#size_binding_y_absolute"}]"##,
        ),
        // The keyboard focus frame: a faint tint behind the focused control and a thin outline
        // round it, so a slider or anything else stays in plain sight.
        rect(
            "focus",
            "[10,10]",
            [0.0, 0.0],
            FOCUS_TINT,
            r##","layer":2,"bindings":[{"binding_name":"#focus_visible","binding_name_override":"#visible"},{"binding_name":"#focus_offset","binding_name_override":"#offset"},{"binding_name":"#focus_width","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#focus_height","binding_name_override":"#size_binding_y_absolute"}]"##,
        ),
        focus_edge("focus_top", "top", false),
        focus_edge("focus_bottom", "bottom", false),
        focus_edge("focus_left", "left", true),
        focus_edge("focus_right", "right", true),
        format!(
            r#"{{"top":{{"type":"panel","size":["100%","100%"],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"controls":[{}]}}}}"#,
            top_controls().join(",")
        ),
        format!(
            r##"{{"settings":{{"type":"panel","size":["100%","100%"],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"bindings":[{{"binding_name":"#settings_visible","binding_name_override":"#visible"}}],"controls":[{}]}}}}"##,
            settings_column().join(",")
        ),
        format!(
            r##"{{"colour":{{"type":"panel","size":["100%","100%"],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"bindings":[{{"binding_name":"#colour_visible","binding_name_override":"#visible"}}],"controls":[{}]}}}}"##,
            colour_column().join(",")
        ),
        shown_when("counter_page", &counter_controls()),
        shown_when("cps_page", &cps_controls()),
        shown_when("fps_page", &fps_controls()),
        shown_when("zoom_page", &zoom_controls()),
        shown_when("light_page", &lighting_controls()),
        shown_when("jump_page", &jump_controls()),
        shown_when("crosshair_page", &crosshair_controls()),
        shown_when("draw_page", &draw_controls()),
        // Hover help: hidden until the mod shows it next to the pointer.
        format!(
            r##"{{"tip":{{"type":"panel","size":[{TIP_WIDTH},14],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","layer":20,"bindings":[{{"binding_name":"#tip_visible","binding_name_override":"#visible"}},{{"binding_name":"#tip_offset","binding_name_override":"#offset"}}],"controls":[{},{}]}}}}"##,
            rect("tip_bg", r#"["100%","100%"]"#, [0.0, 0.0], [0.02, 0.02, 0.03, 0.96], r#","layer":21"#),
            label("tip_text", "#tip_text", [TIP_WIDTH - 8.0, 10.0], [4.0, 2.5], TEXT).replace(r#""layer":4"#, r#""layer":22"#),
        ),
    ];
    format!(
        r##"{{"namespace":"bo","screen":{{"type":"screen","size":["100%","100%"],"controls":[{{"card":{{"type":"panel","size":[{SETTINGS_W},{CARD_H}],"anchor_from":"right_middle","anchor_to":"right_middle","offset":[-{SCREEN_MARGIN},0],"bindings":[{{"binding_name":"#card_visible","binding_name_override":"#visible"}}],"controls":[{}]}}}},{}]}},{},"r":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","anchor_from":"top_left","anchor_to":"top_left"}},"cell":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{WHEEL_CELL_SIZE},{WHEEL_CELL_SIZE}],"radius":0,"layer":1}},"cell_q":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{WHEEL_QUARTER_SIZE},{WHEEL_QUARTER_SIZE}],"radius":0,"layer":1}}}}"##,
        card.join(","),
        moving_guides(),
        bg_template()
    )
}
