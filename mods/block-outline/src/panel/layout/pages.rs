//! The controls of each page.

use super::*;

/// The title, preview and Line / Fill colour buttons, shown above either view.
pub(in crate::panel) fn top_controls() -> Vec<String> {
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
pub(in crate::panel) fn home_controls() -> Vec<String> {
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
pub(in crate::panel) fn home_switch(name: &str, y: f32, control: usize) -> String {
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
pub(in crate::panel) fn home_icon(name: &str, kind: usize, at: f32) -> Vec<String> {
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
pub(in crate::panel) fn counter_controls() -> Vec<String> {
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
pub(in crate::panel) fn lighting_controls() -> Vec<String> {
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
pub(in crate::panel) fn cps_controls() -> Vec<String> {
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
pub(in crate::panel) fn fps_controls() -> Vec<String> {
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
pub(in crate::panel) fn jump_controls() -> Vec<String> {
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
pub(in crate::panel) fn crosshair_controls() -> Vec<String> {
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
pub(in crate::panel) fn draw_controls() -> Vec<String> {
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
pub(in crate::panel) fn stepper(
    name: &str,
    y: f32,
    value: usize,
    less: usize,
    more: usize,
) -> Vec<String> {
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
pub(in crate::panel) fn zoom_controls() -> Vec<String> {
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
pub(in crate::panel) fn moving_guides() -> String {
    format!(
        "{},{}",
        moving_guide("guide_top", "top_middle", 10.0),
        moving_guide("guide_bottom", "bottom_middle", -10.0)
    )
}

pub(in crate::panel) fn moving_guide(name: &str, anchor: &str, offset_y: f32) -> String {
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
