use super::*;
use super::{help::*, json::*};
use crate::outline::KeyName;

const FILL: MenuState = MenuState {
    capturing: false,
    capturing_fullbright: false,
    editing_fill: true,
    editing_aim: false,
    page: Page::Block,
    colour_open: true,
    typing: None,
};

/// A page's state, with the colour tools open or not.
fn on(colour_open: bool) -> MenuState {
    MenuState {
        page: Page::Block,
        colour_open,
        ..MenuState::default()
    }
}

fn press(settings: &mut Settings, state: &mut MenuState, id: &str, value: f32) -> bool {
    apply(settings, state, id, value)
}

#[test]
fn events_change_settings_within_bounds() {
    let (mut s, mut m) = (Settings::default(), MenuState::default());
    assert!(press(&mut s, &mut m, "red", 12.4));
    assert_eq!(s.rgba[0], 12);
    assert!(press(&mut s, &mut m, "opacity", 50.0));
    assert_eq!(s.rgba[3], 128);
    assert!(press(&mut s, &mut m, "width", 4.3));
    assert_eq!(s.width, 4.5);
    assert!(press(&mut s, &mut m, "rainbow", 1.0));
    assert!(press(&mut s, &mut m, "swatch_cyan", 1.0));
    assert_eq!(s.rgba, [0, 230, 255, 128]);
    assert!(!s.rainbow, "a fixed line colour turns rainbow off");
    assert!(!press(&mut s, &mut m, "swatch_cyan", 1.0));
    assert!(!press(&mut s, &mut m, "unknown", 1.0));
    assert!(!press(&mut s, &mut m, "red", f32::NAN));
    assert!(press(&mut s, &mut m, "pulse", 1.0));
    assert!(s.pulse);
}

#[test]
fn slider_values_round_trip_through_events() {
    let s = Settings {
        rgba: [10, 20, 30, 255],
        width: 2.5,
        fill: 40,
        ..Settings::default()
    };
    let (mut copy, mut m) = (Settings::default(), MenuState::default());
    for (index, (id, _)) in SLIDERS.iter().enumerate() {
        press(
            &mut copy,
            &mut m,
            id,
            slider_value(&s, MenuState::default(), index),
        );
    }
    assert_eq!(copy, s);
}

#[test]
fn fill_editing_targets_the_fill_colour() {
    let (mut s, mut m) = (Settings::default(), MenuState::default());
    s.rgba = [255, 0, 0, 255];
    // While the fill matches the line, Fill colour is hidden behind a wide Line colour.
    assert!(!press(&mut s, &mut m, "edit_fill", 1.0));
    assert!(!m.editing_fill);
    let matching = update(&s, 0.0, m);
    assert!(
        matching.contains(r##""#fill_own_visible":false,"#fill_match_visible":true"##),
        "{matching}"
    );
    assert!(matching.contains(r##""#edit_line_text":"Line & fill colour""##));
    // Giving the fill its own colour enables it.
    assert!(press(&mut s, &mut m, "fill_match", 0.0));
    assert_eq!(s.fill_rgb, Some([255, 0, 0]));
    assert!(
        press(&mut s, &mut m, "edit_fill", 1.0),
        "switching tools redraws the menu"
    );
    assert!(m.editing_fill);
    assert!(
        update(&s, 0.0, m).contains(r##""#fill_own_visible":true,"#fill_match_visible":false"##)
    );
    assert!(press(&mut s, &mut m, "swatch_blue", 1.0));
    assert_eq!(s.fill_rgb, Some([40, 90, 255]));
    assert_eq!(s.rgba, [255, 0, 0, 255], "the line keeps its colour");
    assert!(press(&mut s, &mut m, "green", 200.0));
    assert_eq!(s.fill_rgb, Some([40, 200, 255]));
    assert_eq!(apply_text(&mut s, m, "hex", "#112233"), Some(true));
    assert_eq!(s.fill_rgb, Some([0x11, 0x22, 0x33]));
    assert!(update(&s, 0.0, m).contains(r##""#hex_text":"#112233""##));
    // Matching the line again drops the fill's own colour and returns to editing the line.
    assert!(press(&mut s, &mut m, "fill_match", 1.0));
    assert_eq!(s.fill_rgb, None);
    assert!(!m.editing_fill);
    assert!(press(&mut s, &mut m, "fill_match", 0.0));
    assert_eq!(s.fill_rgb, Some([255, 0, 0]));
    // Both buttons show their colour, whichever is being edited.
    let shown = update(&s, 0.0, m);
    assert!(shown.contains(r##""#line_swatch":[1,0,0,1]"##), "{shown}");
    assert!(shown.contains(r##""#fill_swatch":[1,0,0,1]"##));
    s.fill_rgb = Some([0, 0, 255]);
    assert!(preview_update(&s, 0.0, m).contains(r##""#fill_swatch":[0,0,1,1]"##));
}

#[test]
fn favourites_save_and_apply_per_target() {
    let (mut s, mut m) = (Settings::default(), MenuState::default());
    s.rgba = [1, 2, 3, 200];
    assert!(press(&mut s, &mut m, "save_favourite", 1.0));
    assert_eq!(s.favourites[0], Some([1, 2, 3, 200]));
    assert!(
        !press(&mut s, &mut m, "favourite_1", 1.0),
        "an empty slot does nothing"
    );
    s.rgba = [9, 9, 9, 255];
    assert!(press(&mut s, &mut m, "favourite_0", 1.0));
    assert_eq!(
        s.rgba,
        [1, 2, 3, 200],
        "line favourites restore opacity too"
    );
    let mut fill = FILL;
    assert!(press(&mut s, &mut fill, "favourite_0", 1.0));
    assert_eq!(s.fill_rgb, Some([1, 2, 3]));
    assert!(update(&s, 0.0, m).contains(r##""#favourite_0_color":[0.004,0.008,0.012,1]"##));
    assert!(update(&s, 0.0, m).contains(r##""#favourite_1_color":[0.12,0.13,0.15,1]"##));
}

#[test]
fn reset_keeps_key_and_favourites() {
    let (mut s, mut m) = (Settings::default(), FILL);
    s.menu_key = KeyName::new("KeyG").unwrap();
    s.width = 7.0;
    s.favourites[2] = Some([5, 5, 5, 255]);
    assert!(press(&mut s, &mut m, "reset", 1.0));
    assert_eq!(s.width, Settings::default().width);
    assert_eq!(s.menu_key.as_str(), "KeyG");
    assert_eq!(s.favourites[2], Some([5, 5, 5, 255]));
    assert!(!m.editing_fill);
}

#[test]
fn wheel_and_hex_change_the_colour() {
    let (mut s, mut m) = (Settings::default(), MenuState::default());
    s.rgba = [255, 0, 0, 200];
    // Dragging the wheel's x to the left edge at mid height picks cyan, keeping opacity.
    assert!(press(&mut s, &mut m, "wheel_x", 0.0));
    assert_eq!(s.rgba, [0, 255, 255, 200]);
    assert_eq!(apply_text(&mut s, m, "hex", "#FF8800"), Some(true));
    assert_eq!(s.rgba, [255, 136, 0, 255]);
    assert_eq!(
        apply_text(&mut s, m, "hex", "ff8800"),
        Some(false),
        "same colour"
    );
    assert_eq!(apply_text(&mut s, m, "hex", "not a colour"), None);
    let update = update(&s, 0.0, m);
    assert!(
        update.contains(&format!(r##"[{HEX},"#FF8800"]"##)),
        "{update}"
    );
    assert!(update.contains(r##""#wheel_marker":["##));
}

#[test]
fn menu_key_capture_and_hover_help() {
    let mut s = Settings::default();
    s.menu_key = KeyName::new("KeyG").unwrap();
    let capturing = update(
        &s,
        0.0,
        MenuState {
            capturing: true,
            ..MenuState::default()
        },
    );
    assert!(capturing.starts_with(
        r#"{"controls":[{"id":"menu_key","key":"KeyG","capturing":true}],"capture_key":true,"#
    ));
    assert!(capturing.contains(r##""#menu_key_text":"Press a key...""##));
    assert!(update(&s, 0.0, MenuState::default()).contains(r##""#menu_key_text":"G""##));

    // The card sits at the root's right edge, vertically centred, whatever its width.
    let root = [600.0, 400.0];
    for open in [false, true] {
        let to_hud = |card: [f32; 2]| {
            [
                20.0 + 2.0 * (card[0] + root[0] - card_width(open) - SCREEN_MARGIN),
                10.0 + 2.0 * (card[1] + (root[1] - CARD_H) / 2.0),
            ]
        };
        let back = card_point(to_hud([15.0, 40.0]), root, [20.0, 10.0], 2.0, open).unwrap();
        assert!((back[0] - 15.0).abs() < 1e-4 && (back[1] - 40.0).abs() < 1e-4);
    }
    assert_eq!(card_point([0.0; 2], root, [0.0; 2], 0.0, false), None);

    let text = |point, open| tip_at(point, on(open), false).map(|index| tips()[index].2);
    // Settings view.
    assert_eq!(
        text([15.0, ROW_Y + 3.0], false),
        Some("How solid the lines are (0 = invisible)")
    );
    assert_eq!(
        text([15.0, HIGHLIGHT_Y + 5.0], false),
        Some("Cancels the game's tint; click: off/on")
    );
    assert_eq!(
        text([SETTINGS_W - 15.0, HIGHLIGHT_Y + 5.0], false),
        Some("5% weaker or stronger")
    );
    assert_eq!(
        text([SETTINGS_W - 30.0, BOTTOM_Y + 5.0], false),
        Some("This page's defaults; keeps saved")
    );
    // The start page has the page buttons and the menu key; the mob page no highlight fix.
    let home = |point| tip_at(point, MenuState::default(), false).map(|index| tips()[index].2);
    assert_eq!(
        home([15.0, HOME_FULLBRIGHT_Y + 5.0]),
        Some("Lights up caves and nights (only you)")
    );
    let mut lit = Settings::default();
    assert!(apply(
        &mut lit,
        &mut MenuState::default(),
        "fullbright",
        1.0
    ));
    assert!(lit.fullbright && update(&lit, 0.0, MenuState::default()).contains("Fullbright: On"));
    assert!(
        update(&lit, 0.0, MenuState::default()).contains(r##""#fullbright_key_text":"Not set""##)
    );
    let waiting = MenuState {
        capturing_fullbright: true,
        ..MenuState::default()
    };
    assert!(update(&lit, 0.0, waiting).contains(r#""capture_key":true"#));
    lit.fullbright_key = crate::outline::KeyName::new("KeyB");
    assert!(update(&lit, 0.0, MenuState::default()).contains(r##""#fullbright_key_text":"B""##));
    assert_eq!(
        home([15.0, HOME_FULLBRIGHT_KEY_Y + 5.0]),
        Some("Toggles fullbright; Backspace clears")
    );
    assert_eq!(
        home([15.0, HOME_KEY_Y + 5.0]),
        Some("Click, then press your new menu key")
    );
    assert_eq!(
        home([15.0, HOME_MOB_Y + 5.0]),
        Some("Colours and size of mob hitboxes")
    );
    assert_eq!(
        home([15.0, HOME_FULLBRIGHT_KEY_Y + 20.0]),
        None,
        "page rows are hidden on the start page"
    );
    let mob = MenuState {
        page: Page::Mob,
        ..MenuState::default()
    };
    assert_eq!(
        tip_at([15.0, HIGHLIGHT_Y + 5.0], mob, false),
        None,
        "the mob page has no highlight fix"
    );
    let at = |point| tip_at(point, mob, true).map(|index| tips()[index].2);
    assert_eq!(
        at([SETTINGS_W - 30.0, EDIT_Y + 5.0]),
        Some("Colour of the mob you aim at")
    );
    assert_eq!(
        at([15.0, EDIT_Y + 5.0]),
        Some("Hitbox colour; the fill uses it too")
    );
    assert_eq!(
        at([15.0, BOTTOM_Y - MOB_OPTIONS_SHIFT + 5.0]),
        Some("Off: only the mob you aim at")
    );
    // Each page's card is only as tall as its controls; the colour tools use the full height.
    assert!(
        card_height(mob)
            < card_height(MenuState {
                page: Page::Block,
                ..mob
            })
    );
    assert_eq!(
        card_height(MenuState {
            colour_open: true,
            ..mob
        }),
        CARD_H
    );
    let shown_mob = update(&Settings::default(), 0.0, mob);
    assert!(
        shown_mob.contains(&format!(r##""#options_offset":[0,-{MOB_OPTIONS_SHIFT}]"##)),
        "{shown_mob}"
    );
    let hint = |state| {
        tip_at([15.0, card_height(state) - 12.0], state, false).map(|index| tips()[index].2)
    };
    assert_eq!(hint(mob), Some("Menu key or F10 opens; Esc closes"));
    assert_eq!(
        hint(MenuState::default()),
        Some("Menu key or F10 opens; Esc closes")
    );
    let mut mobs = Settings::default();
    let mut on_mob = mob;
    assert!(
        !apply(&mut mobs, &mut on_mob, "edit_fill", 1.0),
        "no mob fill colour"
    );
    assert!(!apply(&mut mobs, &mut on_mob, "fill_match", 0.0));
    let shown = update(&mobs, 0.0, mob);
    assert!(
        shown.contains(
            r##""#fill_own_visible":false,"#fill_match_visible":false,"#mob_top_visible":true"##
        ),
        "{shown}"
    );
    // Colour view, in the same place: the settings below the buttons are hidden.
    assert_eq!(
        text([15.0, RGB_Y + 3.0], true),
        Some("Red part of the colour being edited")
    );
    assert_eq!(
        text([WHEEL[0] + 40.0, WHEEL[1] + 40.0], true),
        Some("Angle picks the hue; centre is paler")
    );
    assert_eq!(
        text([20.0, FAVOURITE_Y + 15.0], true),
        Some("Click a saved colour to use it")
    );
    assert_eq!(
        text([SETTINGS_W - 30.0, FAVOURITE_Y + 15.0], true),
        Some("Saves the colour being edited")
    );
    assert_ne!(
        text([15.0, HIGHLIGHT_Y + 5.0], true),
        Some("Cancels the game's tint; click: off/on"),
        "settings rows are hidden"
    );
    // The buttons on top work in both views.
    assert_eq!(
        text([15.0, EDIT_Y + 5.0], true),
        Some("Shows or hides the line colour tools")
    );
    assert_eq!(
        text([15.0, EDIT_Y + 5.0], false),
        Some("Shows or hides the line colour tools")
    );
    // While the fill matches, the wide Line colour button has its own help.
    let wide = |point| tip_at(point, on(false), true).map(|index| tips()[index].2);
    assert_eq!(
        wide([SETTINGS_W - 30.0, EDIT_Y + 5.0]),
        Some("Colour tools; the fill uses it too")
    );
    assert_eq!(
        text([SETTINGS_W - 30.0, EDIT_Y + 5.0], false),
        Some("Shows or hides the fill colour tools")
    );
    // Hidden colour tools have no help.
    assert_ne!(
        text([20.0, FAVOURITE_Y + 15.0], false),
        Some("Click a saved colour to use it")
    );
    assert_eq!(
        tip_update(None, [0.0; 2], on(false), false),
        r##"{"binding_changes":{"#tip_visible":false}}"##
    );
    let shown = tip_update(Some(13), [SETTINGS_W - 2.0, 40.0], on(false), false);
    assert!(
        shown.contains(&format!("[{},", SETTINGS_W - TIP_WIDTH - 4.0)),
        "{shown}"
    );
    // Every tip fits its box: the native font measured about 5 units per character.
    for (_, _, tip) in tips() {
        assert!(tip.len() as f32 * 5.4 <= TIP_WIDTH - 8.0, "{tip}");
    }
}

#[test]
fn line_and_fill_buttons_show_and_hide_the_colour_tools() {
    let (mut s, mut m) = (Settings::default(), MenuState::default());
    let home = update(&s, 0.0, m);
    assert!(
        home.contains(r##""#home_visible":true,"#page_visible":false"##),
        "{home}"
    );
    assert!(home.contains(r##""#colour_visible":false,"#settings_visible":false"##));
    assert!(press(&mut s, &mut m, "open_block", 1.0));
    assert!(update(&s, 0.0, m).contains(r##""#colour_visible":false,"#settings_visible":true"##));
    assert!(press(&mut s, &mut m, "edit_line", 1.0));
    assert!(m.colour_open && !m.editing_fill);
    let open = update(&s, 0.0, m);
    assert!(open.contains(r##""#colour_visible":true,"#settings_visible":false"##));
    assert!(
        open.contains(r##""#edit_line_bg":[0.55,0.3,0.85,1]"##),
        "the open button is lit"
    );
    // Clicking Line colour again hides them.
    assert!(press(&mut s, &mut m, "edit_line", 1.0));
    assert!(!m.colour_open);
    // Fill colour needs its own colour; then it opens the tools on the fill.
    assert!(!press(&mut s, &mut m, "edit_fill", 1.0));
    press(&mut s, &mut m, "fill_match", 0.0);
    assert!(press(&mut s, &mut m, "edit_fill", 1.0));
    assert!(m.colour_open && m.editing_fill);
    assert!(update(&s, 0.0, m).contains(r##""#edit_fill_bg":[0.55,0.3,0.85,1]"##));
    // Switching straight to the line keeps the tools open.
    assert!(press(&mut s, &mut m, "edit_line", 1.0));
    assert!(m.colour_open && !m.editing_fill);
    // The highlight fix: off / on at 50%, and 5% steps.
    assert!(press(&mut s, &mut m, "highlight_more", 1.0));
    assert_eq!(s.highlight_cancel, 55);
    assert!(press(&mut s, &mut m, "hide_game_highlight", 1.0));
    assert_eq!(s.highlight_cancel, 0);
    assert!(press(&mut s, &mut m, "hide_game_highlight", 1.0));
    assert_eq!(s.highlight_cancel, 50);
    // Back goes one step: to the page, then to the start page, then nowhere.
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert!(!m.colour_open && m.page == Page::Block);
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert_eq!(m.page, Page::Home);
    assert!(!press(&mut s, &mut m, "back", 1.0));
    // The mob page edits the mob look, leaving the block one alone.
    let block = s.style();
    assert!(press(&mut s, &mut m, "open_mob", 1.0));
    assert!(press(&mut s, &mut m, "edit_line", 1.0));
    assert!(press(&mut s, &mut m, "swatch_red", 1.0));
    assert!(press(&mut s, &mut m, "width", 6.0));
    assert_eq!((s.mob.rgba, s.mob.width), ([255, 40, 40, 255], 6.0));
    assert_eq!(s.style(), block);
    assert!(update(&s, 0.0, m).contains(r##""#title_text":"Mob hitbox - line colour""##));
    assert!(update(&s, 0.0, m).contains(r##""#width_value":"6px""##));
    // The aimed colour has its own tools; All mobs switches every hitbox off.
    assert!(press(&mut s, &mut m, "edit_aim", 1.0));
    assert!(m.editing_aim && m.colour_open);
    assert!(press(&mut s, &mut m, "swatch_lime", 1.0));
    assert_eq!(s.aim_rgb, [80, 255, 60]);
    assert_eq!(
        s.mob.rgba,
        [255, 40, 40, 255],
        "the mob line keeps its colour"
    );
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert!(press(&mut s, &mut m, "show_all_mobs", 0.0));
    assert!(!s.show_all_mobs);
    // A number typed on the mob page survives clicking the next box.
    assert!(press(&mut s, &mut m, "type_width", 1.0));
    for key in ["Digit7"] {
        type_key(&mut s, &mut m, key);
    }
    assert!(press(&mut s, &mut m, "type_fill", 1.0));
    assert_eq!(
        s.mob.width, 7.0,
        "typed on the mob page, then another box clicked"
    );
    m.typing = None;
    assert!(press(&mut s, &mut m, "reset", 1.0));
    assert!(s.show_all_mobs && s.aim_rgb == crate::outline::DEFAULT_AIM);
    assert_eq!(s.mob, crate::outline::Style::DEFAULT);
    assert_eq!(s.style(), block, "resetting mobs keeps the block look");
}

#[test]
fn slider_numbers_can_be_typed_and_hover_is_highlighted() {
    let json = content(&Settings::default(), 0.0);
    for index in 0..SLIDERS.len() {
        assert!(
            json.contains(&format!("mod.control:{}", FIRST_TYPE + index)),
            "number box {index}"
        );
    }
    assert!(json.contains(r#""motion":true"#));
    let update = update(&Settings::default(), 0.0, MenuState::default());
    assert!(update.contains(r##""#width_value":"3px""##), "{update}");
    assert!(update.contains(r##""#red_value":"255""##));
    let text = |point| tip_at(point, on(false), false).map(|index| tips()[index].2);
    assert_eq!(
        text([VALUE_X + 5.0, ROW_Y + 3.0]),
        Some("Click, type a number, press Enter")
    );
    assert_eq!(
        text([TRACK_X + 5.0, ROW_Y + 3.0]),
        Some("How solid the lines are (0 = invisible)")
    );
    let opacity = tip_at([15.0, ROW_Y + 3.0], on(false), false);
    assert_eq!(
        highlight_update(opacity, on(false), false),
        format!(
            r##"{{"binding_changes":{{"#hover_visible":true,"#hover_offset":[8,{}],"#hover_width":{},"#hover_height":17}}}}"##,
            ROW_Y - 2.0,
            TRACK_X + track_width() - 6.0
        )
    );
    // With the colour tools open the settings rows are hidden, so they get no highlight.
    assert_eq!(
        highlight_update(opacity, on(true), false),
        r##"{"binding_changes":{"#hover_visible":false}}"##
    );
    assert_eq!(
        highlight_update(None, on(false), false),
        r##"{"binding_changes":{"#hover_visible":false}}"##
    );
}

#[test]
fn typing_into_boxes_with_physical_keys() {
    let (mut s, mut m) = (Settings::default(), MenuState::default());
    let keys = |s: &mut Settings, m: &mut MenuState, keys: &[&str]| {
        for key in keys {
            type_key(s, m, key);
        }
    };
    // Red: click its box, type 42, Enter.
    assert!(press(&mut s, &mut m, "type_red", 1.0));
    assert!(update(&s, 0.0, m).contains(r##""#red_value":"|""##));
    assert!(
        update(&s, 0.0, m).contains(r#""capture_key":true"#),
        "Escape must reach the mod"
    );
    keys(&mut s, &mut m, &["Digit4", "KeyQ", "Numpad2"]);
    assert!(
        update(&s, 0.0, m).contains(r##""#red_value":"42|""##),
        "letters are ignored in numbers"
    );
    keys(&mut s, &mut m, &["Enter"]);
    assert_eq!(s.rgba[0], 42);
    assert_eq!(m.typing, None);
    // Thickness takes a decimal point and is clamped to its range.
    press(&mut s, &mut m, "type_width", 1.0);
    keys(
        &mut s,
        &mut m,
        &["Digit9", "Period", "Digit5", "Period", "NumpadEnter"],
    );
    assert_eq!(s.width, MAX_WIDTH);
    // Escape keeps the old value; Backspace removes the last character.
    press(&mut s, &mut m, "type_green", 1.0);
    keys(&mut s, &mut m, &["Digit1", "Digit2", "Backspace", "Escape"]);
    assert_eq!(s.rgba[1], 255);
    // Hex: letters A to F, the # is added; clicking elsewhere applies.
    press(&mut s, &mut m, "type_hex", 1.0);
    keys(
        &mut s,
        &mut m,
        &[
            "KeyF", "KeyF", "Digit8", "Digit8", "Digit0", "Digit0", "KeyZ",
        ],
    );
    assert!(update(&s, 0.0, m).contains(r##""#hex_text":"#FF8800|""##));
    keys(&mut s, &mut m, &["MouseLeft"]);
    assert_eq!([s.rgba[0], s.rgba[1], s.rgba[2]], [255, 136, 0]);
    // Starting another box applies the one being typed into.
    press(&mut s, &mut m, "type_blue", 1.0);
    keys(&mut s, &mut m, &["Digit7"]);
    press(&mut s, &mut m, "type_red", 1.0);
    assert_eq!(s.rgba[2], 7);
    assert_eq!(m.typing.map(|t| t.field), Some(0));
    // Nonsense is dropped instead of applied.
    press(&mut s, &mut m, "type_hex", 1.0);
    keys(&mut s, &mut m, &["KeyA", "Enter"]);
    assert_eq!(s.rgba[2], 7);
}

#[test]
fn fixed_formats_like_json_numbers() {
    let cases = [
        (0.0, "0"),
        (3.0, "3"),
        (2.5, "2.5"),
        (0.314, "0.314"),
        (-0.25, "-0.25"),
        (255.0, "255"),
        (0.05, "0.05"),
        (f32::NAN, "0"),
    ];
    for (value, text) in cases {
        let mut out = String::new();
        fixed(&mut out, value);
        assert_eq!(out, text, "{value}");
    }
    let update = update(&Settings::default(), 0.0, MenuState::default());
    assert!(update.starts_with(r##"{"controls":[{"id":"menu_key","key":"F8","capturing":false}],"capture_key":false,"control_values":[[0,255],[1,255],[2,255],[3,100],[4,3],[5,0],[6,false],[17,0.5],[18,0.5],[19,"#FFFFFF"],[21,false],[25,true],[42,true],[46,false]],"binding_changes":{"##), "{update}");
    assert!(update.ends_with("}}"));
}

#[test]
fn controls_line_up_with_their_routes() {
    let json = content(&Settings::default(), 0.0);
    assert!(json.contains(r#""id":"swatch_pink""#));
    assert!(json.contains(r#""id":"save_favourite""#));
    assert_eq!(SAVE_FAVOURITE, 30);
    assert!(
        json.contains(&format!("mod.control:{}", FIRST_TYPE + SLIDERS.len())),
        "hex typing route"
    );
    assert!(json.contains(&format!("mod.control:{SAVE_FAVOURITE}")));
    assert!(json.contains(&format!("mod.control:{HIDE_HIGHLIGHT}")));
    assert!(json.contains(&format!("mod.control:{CLOSE}")));
    assert!(!json.contains(&format!("mod.control:{}", CLOSE + 1)));
    // Controls are declared in index order: count them.
    assert_eq!(json.matches(r#""kind":"#).count(), CLOSE + 1);
    let close =
        tip_at([SETTINGS_W - 20.0, 12.0], MenuState::default(), false).map(|index| tips()[index].2);
    assert_eq!(close, Some("Closes the menu (Esc does too)"));
}
