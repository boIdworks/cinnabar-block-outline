//! The static JSON-UI document: the card, its pages and their controls.

use super::help::TIP_WIDTH;
use super::*;

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
    format!(
        r#"{{"{name}":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":{size},"offset":[{},{}],"anchor_from":"top_left","anchor_to":"top_left","radius":3,"color":{colour}{extra}}}}}"#,
        offset[0], offset[1],
    )
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

/// A button whose background colour and caption come from `#<name>_bg` and `#<name>_text`.
pub(super) fn toggle_button(name: &str, area: [f32; 4], control: usize) -> String {
    button(
        name,
        area,
        &format!("mod.control:{control}"),
        &[
            rect(
                "bg",
                r#"["100%","100%"]"#,
                [0.0, 0.0],
                TRACK,
                &format!(
                    r##","layer":1,"bindings":[{{"binding_name":"#{name}_bg","binding_name_override":"#color"}}]"##
                ),
            ),
            label(
                "text",
                &format!("#{name}_text"),
                [area[2] - 10.0, 10.0],
                [6.0, 3.0],
                TEXT,
            ),
        ],
    )
}

/// A Line / Fill colour button: a toggle with a small swatch of that colour on its right.
pub(super) fn edit_button(name: &str, area: [f32; 4], control: usize, swatch: &str) -> String {
    let toggle = toggle_button(name, area, control);
    let swatch = format!(
        r##",{{"swatch_edge":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[22,11],"anchor_from":"right_middle","anchor_to":"right_middle","offset":[-3,0],"radius":2.5,"color":[0.9,0.9,0.92,0.9],"layer":2,"controls":[{{"swatch":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,9],"radius":2,"color":[1,1,1,1],"layer":1,"bindings":[{{"binding_name":"{swatch}","binding_name_override":"#color"}}]}}}}]}}}}"##
    );
    // Insert the swatch as the button's last child.
    let close = toggle.rfind("]}}").expect("button closes its controls");
    format!("{}{}{}", &toggle[..close], swatch, &toggle[close..])
}

/// A button showing a fixed caption on the plain track colour.
pub(super) fn plain_button(name: &str, area: [f32; 4], control: usize, caption: &str) -> String {
    button(
        name,
        area,
        &format!("mod.control:{control}"),
        &[
            rect(
                "bg",
                r#"["100%","100%"]"#,
                [0.0, 0.0],
                TRACK,
                r#","layer":1"#,
            ),
            label("text", caption, [area[2] - 10.0, 10.0], [6.0, 3.0], TEXT),
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
    vec![
        label(
            &format!("{id}_label"),
            title(id),
            [VALUE_X - 12.0, 10.0],
            [10.0, y + 2.0],
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
                    [4.0, 2.0],
                    TEXT,
                ),
            ],
        ),
        button(
            &format!("{id}_slider"),
            [TRACK_X, y, track, ROW_HEIGHT],
            &format!("mod.control:{index}"),
            &[
                rect(
                    "track",
                    r#"["100%","100%"]"#,
                    [0.0, 0.0],
                    TRACK,
                    r#","layer":1"#,
                ),
                rect(
                    "fill",
                    r#"["50%","100%"]"#,
                    [0.0, 0.0],
                    fill,
                    &format!(
                        r##","layer":2,"bindings":[{{"binding_name":"#{id}_fill","binding_name_override":"#size_binding_x"}}],"controls":[{{"knob":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[3,"100% + 4px"],"anchor_from":"right_middle","anchor_to":"right_middle","radius":1.5,"color":[1,1,1,0.92],"layer":2}}}}]"##
                    ),
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
    let mut children = Vec::with_capacity(cells * cells);
    for row in 0..cells {
        for column in 0..cells {
            let cx = (column as f32 + 0.5) / cells as f32;
            let cy = (row as f32 + 0.5) / cells as f32;
            let (dx, dy) = (cx - 0.5, 0.5 - cy);
            let distance = 2.0 * (dx * dx + dy * dy).sqrt();
            if distance > 1.0 {
                continue;
            }
            let hue = dy.atan2(dx) / std::f32::consts::TAU;
            let [r, g, b] = hsv_to_rgb(hue, distance, 1.0).map(|c| f32::from(c) / 255.0);
            children.push(format!(
                r#"{{"c{row}_{column}":{{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[{s},{s}],"offset":[{},{}],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[{r},{g},{b},1],"layer":1}}}}"#,
                column as f32 * WHEEL_CELL,
                row as f32 * WHEEL_CELL,
                s = WHEEL_CELL + 0.25,
            ));
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
    let rule_x = 16.0 + 5.4 * text.len() as f32;
    let rule = rect(
        &format!("{name}_rule"),
        &format!("[{},1]", SETTINGS_W - 10.0 - rule_x),
        [rule_x, y + 5.0],
        RULE,
        r#","layer":2"#,
    );
    format!(
        "{},{rule}",
        label(name, text, [120.0, 10.0], [10.0, y], DIM)
    )
}

/// The title, preview and Line / Fill colour buttons, shown above either view.
pub(super) fn top_controls() -> Vec<String> {
    let right = 10.0 + HALF + 4.0;
    vec![
        label(
            "title",
            "#title_text",
            [BACK_AREA[0] - 14.0, 10.0],
            [10.0, 9.0],
            TEXT,
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
                            [10.0, EDIT_Y, HALF, 15.0],
                            EDIT_LINE,
                            "#line_swatch",
                        ),
                        edit_button(
                            "edit_fill",
                            [right, EDIT_Y, HALF, 15.0],
                            EDIT_FILL,
                            "#fill_swatch",
                        ),
                    ],
                ),
                shown_when(
                    "fill_match",
                    &[edit_button(
                        "edit_line",
                        [10.0, EDIT_Y, SETTINGS_W - 20.0, 15.0],
                        EDIT_LINE,
                        "#line_swatch",
                    )],
                ),
                shown_when(
                    "mob_top",
                    &[
                        edit_button(
                            "edit_line",
                            [10.0, EDIT_Y, HALF, 15.0],
                            EDIT_LINE,
                            "#line_swatch",
                        ),
                        edit_button(
                            "edit_aim",
                            [right, EDIT_Y, HALF, 15.0],
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
                "Hover for help. Esc closes.",
                [SETTINGS_W - 20.0, 10.0],
                [10.0, 0.0],
                DIM
            )
        ),
    ]
}

/// The start page: a button for each page and the menu key.
pub(super) fn home_controls() -> Vec<String> {
    let page_button = |name: &str, y: f32, control: usize, title: &str, about: &str| {
        button(
            name,
            [10.0, y, SETTINGS_W - 20.0, HOME_BUTTON_H],
            &format!("mod.control:{control}"),
            &[
                rect(
                    "bg",
                    r#"["100%","100%"]"#,
                    [0.0, 0.0],
                    TRACK,
                    r#","layer":1"#,
                ),
                // The page's current line colour.
                rect(
                    "stripe",
                    &format!("[4,{}]", HOME_BUTTON_H - 10.0),
                    [6.0, 5.0],
                    [1.0, 1.0, 1.0, 1.0],
                    &format!(
                        r##","layer":2,"bindings":[{{"binding_name":"#{name}_color","binding_name_override":"#color"}}]"##
                    ),
                ),
                label("name", title, [SETTINGS_W - 50.0, 10.0], [16.0, 6.0], TEXT),
                label("about", about, [SETTINGS_W - 50.0, 10.0], [16.0, 19.0], DIM),
                label("arrow", ">", [10.0, 10.0], [SETTINGS_W - 34.0, 12.0], TEXT),
            ],
        )
    };
    vec![
        page_button(
            "open_block",
            HOME_BLOCK_Y,
            OPEN_BLOCK,
            "Block outline",
            "The block you are looking at",
        ),
        page_button(
            "open_mob",
            HOME_MOB_Y,
            OPEN_MOB,
            "Mob hitbox",
            "Mobs and players you look at",
        ),
        caption("general_caption", "General", HOME_KEY_Y - 16.0),
        label(
            "key_title",
            "Menu key",
            [VALUE_X - 12.0, 10.0],
            [10.0, HOME_KEY_Y + 3.0],
            TEXT,
        ),
        toggle_button(
            "menu_key",
            [VALUE_X, HOME_KEY_Y, SETTINGS_W - 10.0 - VALUE_X, 15.0],
            MENU_KEY,
        ),
        toggle_button(
            "fullbright",
            [10.0, HOME_FULLBRIGHT_Y, SETTINGS_W - 20.0, 15.0],
            FULLBRIGHT,
        ),
        label(
            "fullbright_key_title",
            "Light key",
            [VALUE_X - 12.0, 10.0],
            [10.0, HOME_FULLBRIGHT_KEY_Y + 3.0],
            TEXT,
        ),
        toggle_button(
            "fullbright_key",
            [
                VALUE_X,
                HOME_FULLBRIGHT_KEY_Y,
                SETTINGS_W - 10.0 - VALUE_X,
                15.0,
            ],
            FULLBRIGHT_KEY,
        ),
    ]
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
            toggle_button("rainbow", [10.0, TOGGLES_Y, HALF, 15.0], RAINBOW),
            toggle_button("pulse", [right, TOGGLES_Y, HALF, 15.0], PULSE),
        ],
    ));
    options.push(shown_by(
        "mob_effects",
        "mob_only",
        &[toggle_button(
            "rainbow",
            [10.0, TOGGLES_Y, SETTINGS_W - 20.0, 15.0],
            RAINBOW,
        )],
    ));
    // Blocks: Fill match beside Reset. Mobs: All mobs beside Reset.
    options.push(shown_by(
        "block_bottom",
        "block_only",
        &[toggle_button(
            "fill_match",
            [10.0, BOTTOM_Y, HALF, 15.0],
            FILL_MATCH,
        )],
    ));
    options.push(shown_by(
        "mob_bottom",
        "mob_only",
        &[toggle_button(
            "show_all_mobs",
            [10.0, BOTTOM_Y, HALF, 15.0],
            SHOW_ALL,
        )],
    ));
    options.push(plain_button(
        "reset",
        [right, BOTTOM_Y, HALF, 15.0],
        RESET,
        "Reset",
    ));
    // Only blocks have the game's highlight.
    options.push(shown_when(
        "block_only",
        &[
            toggle_button(
                "hide_game_highlight",
                [10.0, HIGHLIGHT_Y, HIGHLIGHT_W, 15.0],
                HIDE_HIGHLIGHT,
            ),
            plain_button(
                "highlight_less",
                [
                    SETTINGS_W - 10.0 - 2.0 * STEP_W - 4.0,
                    HIGHLIGHT_Y,
                    STEP_W,
                    15.0,
                ],
                HIGHLIGHT_LESS,
                "-",
            ),
            plain_button(
                "highlight_more",
                [SETTINGS_W - 10.0 - STEP_W, HIGHLIGHT_Y, STEP_W, 15.0],
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
        "Hex code",
        [hex_w, 10.0],
        [HEX_X, HEX_Y],
        DIM,
    ));
    column.push(button(
        "hex_field",
        [HEX_X, HEX_Y + 12.0, hex_w, 15.0],
        &format!("mod.control:{}", FIRST_TYPE + SLIDERS.len()),
        &[
            rect(
                "field_bg",
                r#"["100%","100%"]"#,
                [0.0, 0.0],
                TRACK,
                r##","layer":1,"bindings":[{"binding_name":"#hex_box","binding_name_override":"#color"}]"##,
            ),
            label("field_text", "#hex_text", [hex_w - 8.0, 10.0], [5.0, 3.0], TEXT),
        ],
    ));
    column.push(label(
        "type_hint",
        "Type 0-9 and A-F,",
        [hex_w, 10.0],
        [HEX_X, HEX_Y + 32.0],
        DIM,
    ));
    column.push(label(
        "enter_hint",
        "then press Enter",
        [hex_w, 10.0],
        [HEX_X, HEX_Y + 44.0],
        DIM,
    ));

    column.push(label(
        "swatch_title",
        "Quick colours",
        [120.0, 10.0],
        [10.0, SWATCH_Y],
        DIM,
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
                16.0,
            ],
            FIRST_SWATCH + index,
            Ok([r, g, b, 1.0]),
        ));
    }
    column.push(label(
        "favourite_title",
        "Saved colours",
        [120.0, 10.0],
        [10.0, FAVOURITE_Y],
        DIM,
    ));
    for slot in 0..FAVOURITES {
        column.push(colour_button(
            &format!("favourite_{slot}"),
            [10.0 + 34.0 * slot as f32, FAVOURITE_Y + 12.0, 30.0, 16.0],
            FIRST_FAVOURITE + slot,
            Err(&format!("#favourite_{slot}_color")),
        ));
    }
    let save_x = 10.0 + 34.0 * FAVOURITES as f32;
    column.push(plain_button(
        "save_favourite",
        [save_x, FAVOURITE_Y + 12.0, SETTINGS_W - 10.0 - save_x, 16.0],
        SAVE_FAVOURITE,
        "Save colour",
    ));
    column
}

/// The static JSON-UI document; everything that changes comes from bindings. The card is pinned
/// to the right edge of the screen; below its top controls it shows either the settings or the
/// colour tools.
pub(super) fn document() -> String {
    let card = vec![
        // As tall as the page shown (`#card_height`), from the top of the card.
        rect(
            "background",
            &format!("[{SETTINGS_W},{CARD_H}]"),
            [0.0, 0.0],
            [0.07, 0.08, 0.1, 0.96],
            r##","layer":1,"bindings":[{"binding_name":"#card_height","binding_name_override":"#size_binding_y_absolute"}]"##,
        )
        .replace(r#""radius":3"#, r#""radius":7"#),
        rect(
            "hover",
            "[10,10]",
            [0.0, 0.0],
            [1.0, 1.0, 1.0, 0.08],
            r##","layer":2,"bindings":[{"binding_name":"#hover_visible","binding_name_override":"#visible"},{"binding_name":"#hover_offset","binding_name_override":"#offset"},{"binding_name":"#hover_width","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#hover_height","binding_name_override":"#size_binding_y_absolute"}]"##,
        ),
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
        // Hover help: hidden until the mod shows it next to the pointer.
        format!(
            r##"{{"tip":{{"type":"panel","size":[{TIP_WIDTH},14],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","layer":20,"bindings":[{{"binding_name":"#tip_visible","binding_name_override":"#visible"}},{{"binding_name":"#tip_offset","binding_name_override":"#offset"}}],"controls":[{},{}]}}}}"##,
            rect("tip_bg", r#"["100%","100%"]"#, [0.0, 0.0], [0.02, 0.02, 0.03, 0.96], r#","layer":21"#),
            label("tip_text", "#tip_text", [TIP_WIDTH - 8.0, 10.0], [4.0, 2.5], TEXT).replace(r#""layer":4"#, r#""layer":22"#),
        ),
    ];
    format!(
        r##"{{"namespace":"block_outline","screen":{{"type":"screen","size":["100%","100%"],"controls":[{{"card":{{"type":"panel","size":[{SETTINGS_W},{CARD_H}],"anchor_from":"right_middle","anchor_to":"right_middle","offset":[-{SCREEN_MARGIN},0],"controls":[{}]}}}}]}}}}"##,
        card.join(",")
    )
}
