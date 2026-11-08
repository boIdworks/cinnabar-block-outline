//! Putting the controls of the current page together into the menu document.

use super::*;

/// The settings view, below the Line / Fill colour buttons while the colour tools are hidden.
pub(in crate::panel) fn settings_column() -> Vec<String> {
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
pub(in crate::panel) fn colour_column() -> Vec<String> {
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
pub(in crate::panel) fn section(name: &str, top: f32, bottom: f32) -> String {
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
pub(in crate::panel) fn focus_edge(name: &str, side: &str, vertical: bool) -> String {
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
pub(in crate::panel) fn behind(panel: String) -> String {
    panel.replacen(r#""layer":2"#, r#""layer":0"#, 1)
}

/// The panels behind each page's settings. They sit below the hover highlight and the focus
/// frame, which a panel on top of them would hide.
pub(in crate::panel) fn sections() -> String {
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
    format!("{block},{mob},{counter},{cps},{fps},{zoom},{light},{jump},{crosshair},{colour},{home}")
}

/// The static JSON-UI document; everything that changes comes from bindings. The card is pinned
/// to the right edge of the screen; below its top controls it shows either the settings or the
/// colour tools.
pub(in crate::panel) fn document() -> String {
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
