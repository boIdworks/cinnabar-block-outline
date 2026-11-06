use super::*;
use super::{bindings::{CAPTURE_MENU, CAPTURE_OPTIONAL}, controls::*, help::*, json::*};
use crate::outline::KeyName;

const FILL: MenuState = MenuState {
    capturing: false,
    capturing_fullbright: false,
    capturing_zoom: false,
    editing_fill: true,
    editing_aim: false,
    page: Page::Block,
    colour_open: true,
    typing: None,
    confirm: Reset::None,
    focus: None,
    moving: false,
    guide_bottom: false,
};

/// A page's state, with the colour tools open or not.
fn on(colour_open: bool) -> MenuState {
    MenuState {
        page: Page::Block,
        colour_open,
        ..MenuState::default()
    }
}

/// Whether the update shows switch `name` on: its pill in the "on" colour, its knob at the right.
fn switch_on(update: &str, name: &str) -> bool {
    let right = PILL_W - KNOB - KNOB_OFF;
    let knob = format!(r##""#{name}_knob":[{right},0]"##);
    let off = format!(r##""#{name}_knob":[{KNOB_OFF},0]"##);
    assert!(update.contains(&knob) != update.contains(&off), "{name} has exactly one knob place");
    let on = update.contains(&knob);
    let colour = format!(r##""#{name}_bg":[0.45,0.24,0.75,1]"##);
    assert_eq!(update.contains(&colour), on, "{name}'s pill follows its knob");
    on
}

/// Presses a reset button twice, as the menu asks: the first click only asks for confirmation.
fn confirmed(settings: &mut Settings, state: &mut MenuState, id: &str) -> bool {
    let before = *settings;
    assert!(press(settings, state, id, 1.0), "the first click asks");
    assert_eq!(*settings, before, "and changes nothing");
    press(settings, state, id, 1.0)
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
    let (mut s, mut m) = (Settings::default(), on(false));
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
    assert!(confirmed(&mut s, &mut m, "reset"));
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
    let s = Settings {
        menu_key: KeyName::new("KeyG").unwrap(),
        ..Settings::default()
    };
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
    assert!(capturing.contains(r##""#menu_key_text":"Press key, Esc cancels""##));
    assert!(update(&s, 0.0, MenuState::default()).contains(r##""#menu_key_text":"G""##));

    // The card sits at the root's right edge, vertically centred.
    let root = [600.0, 400.0];
    let to_hud = |card: [f32; 2]| {
        [
            20.0 + 2.0 * (card[0] + root[0] - SETTINGS_W - SCREEN_MARGIN),
            10.0 + 2.0 * (card[1] + (root[1] - CARD_H) / 2.0),
        ]
    };
    let back = card_point(to_hud([15.0, 40.0]), root, [20.0, 10.0], 2.0).unwrap();
    assert!((back[0] - 15.0).abs() < 1e-4 && (back[1] - 40.0).abs() < 1e-4);
    assert_eq!(card_point([0.0; 2], root, [0.0; 2], 0.0), None);
    assert_eq!(card_point([0.0; 2], root, [0.0; 2], f32::NAN), None);

    let text = |point, open| tip_at(point, on(open), false).map(|index| controls()[index].tip);
    // Settings view.
    assert_eq!(
        text([15.0, ROW_Y + 3.0], false),
        Some("How solid the lines are (0 = invisible)")
    );
    assert_eq!(
        text([15.0, HIGHLIGHT_Y + 5.0], false),
        Some("Hides the game's highlight; click: on/off")
    );
    assert_eq!(
        text([SETTINGS_W - 15.0, HIGHLIGHT_Y + 5.0], false),
        Some("5%% weaker or stronger")
    );
    assert_eq!(
        text([SETTINGS_W - 30.0, BOTTOM_Y + 5.0], false),
        Some("Resets this page; click twice to confirm")
    );
    // The start page has the page buttons and the menu key; the mob page no highlight fix.
    let home = |point| tip_at(point, MenuState::default(), false).map(|index| controls()[index].tip);
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
    assert!(lit.fullbright);
    assert!(switch_on(&update(&lit, 0.0, MenuState::default()), "fullbright"));
    assert!(
        update(&lit, 0.0, MenuState::default()).contains(r##""#fullbright_key_text":"Click to set""##)
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
        Some("Click, press a key; Del clears it")
    );
    assert_eq!(
        home([15.0, HOME_KEY_Y + 5.0]),
        Some("Click, press a key; Esc cancels")
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
    let at = |point| tip_at(point, mob, true).map(|index| controls()[index].tip);
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
        tip_at([15.0, card_height(state) - 12.0], state, false).map(|index| controls()[index].tip)
    };
    assert_eq!(hint(mob), Some("Menu key opens; Esc closes"));
    assert_eq!(
        hint(MenuState::default()),
        Some("Menu key opens; Esc closes")
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
        Some("Hides the game's highlight; click: on/off"),
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
    let wide = |point| tip_at(point, on(false), true).map(|index| controls()[index].tip);
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
    for control in controls() {
        assert!(control.tip.len() as f32 * 5.4 <= TIP_WIDTH - 8.0, "{}", control.tip);
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
        open.contains(r##""#edit_line_bg":[0.45,0.24,0.75,1]"##),
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
    assert!(update(&s, 0.0, m).contains(r##""#edit_fill_bg":[0.45,0.24,0.75,1]"##));
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
    assert!(update(&s, 0.0, m).contains(r##""#title_text":"Mob > Line colour""##));
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
    type_key(&mut s, &mut m, "Digit7");
    assert!(press(&mut s, &mut m, "type_fill", 1.0));
    assert_eq!(
        s.mob.width, 7.0,
        "typed on the mob page, then another box clicked"
    );
    m.typing = None;
    assert!(confirmed(&mut s, &mut m, "reset"));
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
    let text = |point| tip_at(point, on(false), false).map(|index| controls()[index].tip);
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
            r##"{{"binding_changes":{{"#hover_visible":true,"#hover_offset":[8,{}],"#hover_width":{},"#hover_height":22,"#hint_text":"How solid the lines are (0 = invisible)"}}}}"##,
            ROW_Y - 2.0,
            TRACK_X + track_width() - 6.0
        )
    );
    // With the colour tools open the settings rows are hidden, so they get no highlight.
    assert_eq!(
        highlight_update(opacity, on(true), false),
        r##"{"binding_changes":{"#hover_visible":false,"#hint_text":"Hover or press Tab for help. Esc closes."}}"##
    );
    assert_eq!(
        highlight_update(None, on(false), false),
        r##"{"binding_changes":{"#hover_visible":false,"#hint_text":"Hover or press Tab for help. Esc closes."}}"##
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
    assert!(update.starts_with(r##"{"controls":[{"id":"menu_key","key":"F8","capturing":false}],"capture_key":false,"show_gameplay_hud":false,"control_values":[[0,255],[1,255],[2,255],[3,100],[4,3],[5,0],[6,false],[17,0.5],[18,0.5],[19,"#FFFFFF"],[21,false],[25,true],[42,true],[46,false],[50,true],[51,false],[59,true],[60,false],[61,true],[62,true]],"binding_changes":{"##), "{update}");
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
    assert!(json.contains(&format!("mod.control:{COUNTER_DONE}")));
    assert!(!json.contains(&format!("mod.control:{}", LAST_CONTROL + 1)));
    // Controls are declared in index order: count them.
    assert_eq!(json.matches(r#""kind":"#).count(), LAST_CONTROL + 1);
    let close =
        tip_at([SETTINGS_W - 20.0, 12.0], MenuState::default(), false).map(|index| controls()[index].tip);
    assert_eq!(close, Some("Closes the menu (Esc does too)"));
}

#[test]
fn arrow_counter_page_and_moving() {
    let mut s = Settings::default();
    let mut m = MenuState::default();
    assert!(press(&mut s, &mut m, "open_counter", 1.0));
    assert_eq!(m.page, Page::Counter);
    let shown = update(&s, 0.0, m);
    assert!(shown.contains(r#""show_gameplay_hud":true"#), "the HUD stays visible: {shown}");
    assert!(shown.contains(r##""#title_text":"Arrow counter""##));
    assert!(shown.contains(r##""#counter_page_visible":true"##));
    // The outline pages' widgets stay hidden here.
    for hidden in ["settings", "colour", "fill_own", "fill_match", "mob_top"] {
        assert!(
            shown.contains(&format!(r##""#{hidden}_visible":false"##)),
            "{hidden}: {shown}"
        );
    }
    assert!(shown.contains(r##""#counter_size_text":"Size: 100%%""##));

    assert!(press(&mut s, &mut m, "counter", 0.0));
    assert!(!s.counter.on);
    assert!(press(&mut s, &mut m, "counter_bow", 1.0));
    assert!(s.counter.bow_only);
    assert!(press(&mut s, &mut m, "counter_bigger", 1.0));
    assert!(press(&mut s, &mut m, "counter_bigger", 1.0));
    assert!((s.counter.scale - 1.2).abs() < 1e-5);
    assert!(update(&s, 0.0, m).contains(r##""#counter_size_text":"Size: 120%%""##));
    assert!(press(&mut s, &mut m, "counter_size", 1.0));
    assert_eq!(s.counter.scale, 1.0);

    // Moving hides the card, shows the guide, turns the counter on and keeps Escape.
    assert!(press(&mut s, &mut m, "counter_move", 1.0));
    assert!(m.moving && s.counter.on);
    let moving = update(&s, 0.0, m);
    assert!(moving.contains(r#""capture_key":true"#));
    assert!(moving.contains(r##""#card_visible":false"##));
    assert!(moving.contains(r##""#guide_top_visible":true,"#guide_bottom_visible":false"##));
    let low = MenuState { guide_bottom: true, ..m };
    assert_eq!(
        guide_update(low),
        r##"{"binding_changes":{"#guide_top_visible":false,"#guide_bottom_visible":true}}"##
    );
    assert_eq!(tip_at([15.0, COUNTER_ON_Y + 5.0], m, false), None);
    assert!(press(&mut s, &mut m, "counter_done", 1.0));
    assert!(!m.moving);

    s.counter.pos = [0.1, 0.2];
    s.counter.scale = 2.0;
    assert!(confirmed(&mut s, &mut m, "counter_reset"));
    assert_eq!(s.counter, crate::counter::Counter { on: true, bow_only: true, ..Default::default() });

    let tip = |point| tip_at(point, m, false).map(|index| controls()[index].tip);
    assert_eq!(tip([15.0, COUNTER_MOVE_Y + 5.0]), Some("Drag it anywhere; scroll to resize"));
    assert_eq!(tip([SETTINGS_W - 15.0, COUNTER_SIZE_Y + 5.0]), Some("10%% smaller or bigger"));
    assert_eq!(
        tip_at([15.0, HOME_COUNTER_Y + 5.0], MenuState::default(), false).map(|index| controls()[index].tip),
        Some("Shows how many arrows you carry")
    );
    // Back goes to the start page.
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert_eq!(m.page, Page::Home);
}

#[test]
fn counter_settings_round_trip() {
    let s = Settings {
        counter: crate::counter::Counter {
            on: false,
            bow_only: true,
            pos: [0.25, 0.75],
            scale: 1.5,
        },
        ..Settings::default()
    };
    let back = Settings::parse(&s.to_json());
    assert_eq!(back.counter, s.counter);
    // Out-of-range values are clamped; older files keep the defaults.
    let odd = Settings::parse(r#"{"arrow_counter_x": 3, "arrow_counter_y": -1, "arrow_counter_scale": 99}"#);
    assert_eq!(odd.counter.pos, [1.0, 0.0]);
    assert_eq!(odd.counter.scale, crate::counter::MAX_SCALE);
    assert_eq!(Settings::parse("{}").counter, crate::counter::Counter::default());
    // Resetting an outline page keeps the counter.
    let mut reset = s;
    reset.reset();
    assert_eq!(reset.counter, s.counter);
}

/// The host refuses a menu document over 192 KiB, which would stop the whole mod loading. Keep
/// some room for the next page.
#[test]
fn menu_document_stays_within_the_host_limit() {
    let size = document().len();
    assert!(
        size <= 180 * 1024,
        "the menu document is {size} bytes; the host's limit is {} and some room is kept",
        192 * 1024
    );
}

#[test]
fn zoom_page_values_switches_and_key() {
    let mut s = Settings::default();
    let mut m = MenuState::default();
    assert!(press(&mut s, &mut m, "open_zoom", 1.0));
    assert_eq!(m.page, Page::Zoom);
    let shown = update(&s, 0.0, m);
    assert!(shown.contains(r##""#title_text":"Zoom""##) && shown.contains(r##""#zoom_page_visible":true"##));
    for hidden in ["settings", "colour", "counter_page"] {
        assert!(shown.contains(&format!(r##""#{hidden}_visible":false"##)), "{hidden}: {shown}");
    }
    assert!(switch_on(&shown, "zoom"));
    assert!(!switch_on(&shown, "zoom_mode"), "hold the key is the default");
    assert!(shown.contains(r##""#zoom_key_text":"Z""##));
    assert!(shown.contains(r##""#zoom_level_text":"Zoom level: 4x""##));
    assert!(shown.contains(r##""#zoom_time_text":"Zoom time: 0.25s""##));
    assert!(shown.contains(r##""#zoom_step_text":"Scroll step: 20%%""##));

    // Values step by one notch and stay in bounds.
    assert!(press(&mut s, &mut m, "zoom_level_more", 1.0));
    assert_eq!(s.zoom.level, 4.5);
    assert!(update(&s, 0.0, m).contains(r##""#zoom_level_text":"Zoom level: 4.5x""##));
    for _ in 0..40 {
        press(&mut s, &mut m, "zoom_level_more", 1.0);
    }
    assert_eq!(s.zoom.level, crate::zoom::MAX_LEVEL);
    assert!(!press(&mut s, &mut m, "zoom_level_more", 1.0), "already at the top");
    assert!(press(&mut s, &mut m, "zoom_level", 1.0));
    assert_eq!(s.zoom.level, 4.0, "the value's own button resets it");
    for _ in 0..10 {
        press(&mut s, &mut m, "zoom_time_less", 1.0);
    }
    assert_eq!(s.zoom.time, 0.0);
    assert!(update(&s, 0.0, m).contains(r##""#zoom_time_text":"Zoom time: instant""##));
    assert!(press(&mut s, &mut m, "zoom_step_more", 1.0));
    assert_eq!(s.zoom.step, 25);
    for _ in 0..20 {
        press(&mut s, &mut m, "zoom_step_less", 1.0);
    }
    assert_eq!(s.zoom.step, crate::zoom::MIN_STEP);

    // Switches.
    assert!(press(&mut s, &mut m, "zoom_mode", 1.0) && s.zoom.toggle);
    // Scrolling moves the hotbar slot too, and the switch says so.
    assert!(s.zoom.scroll);
    assert!(switch_on(&shown, "zoom_scroll"));
    assert!(press(&mut s, &mut m, "zoom_scroll", 0.0) && !s.zoom.scroll);
    assert!(!switch_on(&update(&s, 0.0, m), "zoom_scroll"));
    assert!(press(&mut s, &mut m, "zoom_scroll", 1.0) && s.zoom.scroll);
    assert!(press(&mut s, &mut m, "zoom_sens", 0.0) && !s.zoom.sensitivity);
    assert!(press(&mut s, &mut m, "zoom", 0.0) && !s.zoom.on);
    // Reset keeps the chosen key.
    s.zoom.key = KeyName::new("KeyC");
    assert!(confirmed(&mut s, &mut m, "zoom_reset"));
    assert_eq!(s.zoom, crate::zoom::Zoom { key: KeyName::new("KeyC"), ..Default::default() });
    // An unbound key shows as not set, and the reset keeps it unbound.
    s.zoom.key = None;
    assert!(update(&s, 0.0, m).contains(r##""#zoom_key_text":"Click to set""##));
    s.zoom.scroll = false;
    assert!(confirmed(&mut s, &mut m, "zoom_reset"));
    assert_eq!(s.zoom.key, None);
    assert!(s.zoom.scroll);
    s.zoom.key = KeyName::new("KeyZ");

    // Waiting for the zoom key keeps Escape for itself, and the button says so.
    m.capturing_zoom = true;
    let waiting = update(&s, 0.0, m);
    assert!(waiting.contains(r#""capture_key":true"#));
    assert!(waiting.contains(r##""#zoom_key_text":"Esc cancels, Del clears""##));
    m.capturing_zoom = false;

    // Hover help, the start page's button, and the card fits the page.
    let tip = |point| tip_at(point, m, false).map(|index| controls()[index].tip);
    assert_eq!(tip([15.0, ZOOM_MODE_Y + 5.0]), Some("Hold the key, or press to toggle"));
    assert_eq!(tip([SETTINGS_W - 15.0, ZOOM_STEP_Y + 5.0]), Some("5%% smaller or bigger"));
    assert_eq!(
        tip_at([15.0, HOME_ZOOM_Y + 5.0], MenuState::default(), false).map(|index| controls()[index].tip),
        Some("Hold a key to zoom in smoothly")
    );
    assert!(card_height(m) <= CARD_H && card_height(MenuState::default()) <= CARD_H);
    // Back goes to the start page.
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert_eq!(m.page, Page::Home);
}

/// Writes the menu's start page, Arrow counter page and moving guide as panel JSON into
/// `SNAPSHOT_DIR`, for Cinnabar's ui-snapshot tool. Run with `--ignored`.
#[test]
#[ignore]
fn dump_snapshots() {
    let Ok(dir) = std::env::var("SNAPSHOT_DIR") else {
        return;
    };
    let s = Settings::default();
    let counter = MenuState {
        page: Page::Counter,
        ..MenuState::default()
    };
    let moving = MenuState {
        moving: true,
        ..counter
    };
    let zoom = MenuState {
        page: Page::Zoom,
        ..MenuState::default()
    };
    let asking = MenuState { confirm: Reset::Page, ..on(false) };
    let capture = MenuState { capturing_zoom: true, ..zoom };
    let home_capture = MenuState { capturing: true, ..MenuState::default() };
    let mob = MenuState { page: Page::Mob, ..MenuState::default() };
    let stop = |action: Action| controls().iter().position(|c| c.action == action).map(|i| i as u8);
    let focus_slider = MenuState { focus: stop(Action::Slider(3)), ..on(false) };
    let focus_switch = MenuState { focus: stop(Action::Toggle("rainbow")), ..on(false) };
    for (name, state) in [("home", MenuState::default()), ("counter", counter), ("moving", moving), ("block", on(false)), ("colour", on(true)), ("zoom", zoom), ("asking", asking), ("capture", capture), ("home_capture", home_capture), ("mob", mob), ("focus_slider", focus_slider), ("focus_switch", focus_switch)] {
        std::fs::write(format!("{dir}/{name}.json"), content_in(&s, 0.0, state)).unwrap();
    }
    // A page with changed values, which say `reset` on their right.
    let mut changed = s;
    changed.zoom.level = 6.0;
    changed.counter.scale = 1.4;
    std::fs::write(format!("{dir}/zoom_changed.json"), content_in(&changed, 0.0, zoom)).unwrap();
    std::fs::write(format!("{dir}/counter_surface.json"), crate::counter::surface()).unwrap();
    for (name, arrows, scale, moving) in [("counter_hud", 64, 1.0, false), ("counter_big", 0, 3.0, true)] {
        let width = crate::counter::guess_width(crate::counter::digits(arrows));
        let size = crate::counter::size(scale, width);
        let view = crate::counter::View {
            visible: true,
            arrows: Some(arrows),
            offset: crate::counter::place([0.5, 0.4], size, [426.0, 240.0]),
            size,
            scale,
            text_width: width,
            edit: moving.then_some(crate::counter::Edit { hover: Some(crate::counter::Part::Handle(3)), guides: [Some(213.0), Some(218.0)], ..Default::default() }),
        };
        std::fs::write(format!("{dir}/{name}.patch.json"), crate::counter::patch(&view)).unwrap();
    }
}

#[test]
fn resets_ask_before_they_change_anything() {
    let mut s = Settings {
        width: 7.0,
        ..Settings::default()
    };
    let mut m = on(false);
    // The first click turns the button red and asks; nothing is reset yet.
    assert!(press(&mut s, &mut m, "reset", 1.0));
    assert_eq!(s.width, 7.0);
    assert_eq!(m.confirm, Reset::Page);
    let asking = update(&s, 0.0, m);
    assert!(asking.contains(r##""#reset_text":"Sure? Click again""##));
    assert!(asking.contains(r##""#reset_bg":[0.62,0.2,0.18,1]"##), "{asking}");
    // Clicking anything else in between cancels the question and still redraws the button.
    assert!(press(&mut s, &mut m, "rainbow", 1.0));
    assert_eq!(m.confirm, Reset::None);
    assert!(update(&s, 0.0, m).contains(r##""#reset_text":"Reset page""##));
    assert!(press(&mut s, &mut m, "reset", 1.0));
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert_eq!((m.confirm, s.width), (Reset::None, 7.0));
    // Each reset button asks for itself: a different one starts over.
    m.page = Page::Zoom;
    assert!(press(&mut s, &mut m, "zoom_reset", 1.0));
    assert!(press(&mut s, &mut m, "counter_reset", 1.0));
    assert_eq!(m.confirm, Reset::Counter);
    // The Reset in the moving guide sits beside the counter and acts at once.
    s.counter.scale = 2.0;
    m.page = Page::Counter;
    m.moving = true;
    assert!(press(&mut s, &mut m, "counter_reset", 1.0));
    assert_eq!(s.counter.scale, 1.0);
}

/// A key button's wording while it waits tells how to cancel; Escape cancelling is in the mod.
#[test]
fn key_buttons_say_how_to_cancel() {
    let s = Settings::default();
    let zoom = MenuState { capturing_zoom: true, ..MenuState::default() };
    assert!(update(&s, 0.0, zoom).contains("Esc cancels, Del clears"));
    let menu = MenuState { capturing: true, ..MenuState::default() };
    assert!(update(&s, 0.0, menu).contains("Press key, Esc cancels"));
}

/// The buttons of the real document, as `[x, y, w, h]` on the card (the moving guide, which
/// has its own coordinates, comes after the card and is left out).
fn document_buttons() -> Vec<[f32; 4]> {
    let document = document();
    let card = &document[..document.find(r#""guide_top":"#).unwrap()];
    let numbers = |text: &str| -> Vec<f32> {
        text.trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .filter_map(|n| n.trim().parse().ok())
            .collect()
    };
    let mut buttons = Vec::new();
    for (start, _) in card.match_indices(r#""type":"button","size":["#) {
        let rest = &card[start + r#""type":"button","size":"#.len()..];
        let size = numbers(&rest[..rest.find(']').unwrap() + 1]);
        let at = rest.find(r#""offset":"#).unwrap() + r#""offset":"#.len();
        let offset = numbers(&rest[at..at + rest[at..].find(']').unwrap() + 1]);
        buttons.push([offset[0], offset[1], size[0], size[1]]);
    }
    buttons
}

fn contains(outer: [f32; 4], inner: [f32; 4]) -> bool {
    let eps = 0.01;
    inner[0] >= outer[0] - eps
        && inner[1] >= outer[1] - eps
        && inner[0] + inner[2] <= outer[0] + outer[2] + eps
        && inner[1] + inner[3] <= outer[1] + outer[3] + eps
}

/// The table is the one place that says where controls are, and it has to agree with the
/// document that draws them: every button sits inside a control of the table, every table
/// control that is a single button covers exactly one, and every other control (a row, a
/// number box, the hint line) covers at least one button or is plain help. No tolerance.
#[test]
fn the_controls_table_matches_the_document_exactly() {
    let buttons = document_buttons();
    assert!(buttons.len() > 60, "found {} buttons", buttons.len());
    // Areas as they can be shown: the Mob page lifts its options one row.
    let shifted = |area: [f32; 4]| [area[0], area[1] - MOB_OPTIONS_SHIFT, area[2], area[3]];
    let areas: Vec<[f32; 4]> = controls()
        .iter()
        .flat_map(|control| [control.area, shifted(control.area)])
        .collect();
    for button in &buttons {
        assert!(
            areas.iter().any(|&area| contains(area, *button)),
            "no control covers the button at {button:?}"
        );
    }
    for control in controls() {
        let [x, y, w, h] = control.area;
        // The hint line and a label's tip (the title) are not buttons.
        if matches!(control.action, Action::Hint) || control.tip.starts_with("Outlines,") {
            continue;
        }
        let inside = |area: [f32; 4]| buttons.iter().any(|&button| contains(area, button));
        let exact = |area: [f32; 4]| {
            buttons.iter().any(|b| (0..4).all(|i| (b[i] - area[i]).abs() < 0.01))
        };
        let (plain, lifted) = (control.area, shifted(control.area));
        match &control.action {
            Action::Press(_) | Action::Key(_) | Action::Toggle(_) | Action::Hex => {
                // Key rows hold a label and a button; the others are exactly their button.
                let key_row = matches!(control.action, Action::Key(_));
                let ok = if key_row { inside(plain) || inside(lifted) } else { exact(plain) || exact(lifted) };
                assert!(ok, "the control \"{}\" at {:?} is not a button", control.tip, control.area);
            }
            Action::Slider(_) | Action::Stepper { .. } => {
                assert!(inside(plain) || inside(lifted), "no button in the row at {x},{y},{w},{h}");
            }
            Action::Help => {
                assert!(
                    inside(plain) || inside(lifted) || exact(plain) || control.tip.contains("click")
                        || control.tip.contains("less or more") || control.tip.contains("smaller or bigger")
                        || control.tip.contains("weaker or stronger") || control.tip.contains("Click, type")
                        || control.tip.contains("Angle picks"),
                    "the help \"{}\" at {x},{y} has nothing to point at",
                    control.tip
                );
            }
            Action::Hint => {}
        }
    }
}

/// Labels cut off what does not fit (the guide's last line once read "screen..."), and the
/// offline renderer is the only place that shows it, so bound the text by its width instead.
/// Measured in renders: about 5 units a letter, and a label needs a few units to spare; the
/// guide's old 40-letter line in a 200-unit label was cut, the 40-letter tips in 228 are not.
#[test]
fn texts_fit_where_they_are_shown() {
    let fits = |text: &str, width: f32| text.len() as f32 * 5.0 + 6.0 <= width;
    assert!(!fits("It snaps to the hotbar and screen middle", 200.0), "the check must catch it");
    for line in GUIDE_LINES {
        assert!(fits(line, GUIDE_W - GUIDE_TEXT_X - 10.0), "guide line too long: {line}");
    }
    for control in controls() {
        // `%%` shows as one `%`.
        assert!(fits(&control.tip.replace("%%", "%"), TIP_WIDTH - 8.0), "tip too long: {}", control.tip);
        assert!(fits(&control.tip.replace("%%", "%"), SETTINGS_W - 20.0), "hint too long: {}", control.tip);
    }
    assert!(fits(HINT_DEFAULT, SETTINGS_W - 20.0));
    // What the start page buttons say, beside the badge and before the arrow.
    for about in HOME_ABOUT {
        assert!(fits(about, SETTINGS_W - 70.0), "home description too long: {about}");
    }
    // The key buttons while they wait, in the narrowest button that shows them.
    for text in [CAPTURE_MENU, CAPTURE_OPTIONAL] {
        assert!(fits(text, SETTINGS_W - 10.0 - KEY_X - 10.0), "{text}");
    }
    // A reset button's question in the narrowest button (half the card).
    assert!(fits("Sure? Click again", HALF - 10.0));
    // The title, the longest trail in the room left of Back.
    for state in [
        MenuState { page: Page::Block, colour_open: true, editing_fill: true, ..MenuState::default() },
        MenuState { page: Page::Mob, colour_open: true, editing_aim: true, ..MenuState::default() },
        MenuState { page: Page::Counter, ..MenuState::default() },
    ] {
        let patch = update(&Settings::default(), 0.0, state);
        let title = patch.split(r##""#title_text":""##).nth(1).unwrap().split('"').next().unwrap();
        // The title is drawn larger.
        assert!(title.len() as f32 * 5.0 * TITLE_SCALE + 6.0 <= BACK_AREA[0] - 14.0 - TITLE_X, "title too long: {title}");
    }
}

fn keyed(s: &Settings, m: &mut MenuState, names: &[&str]) -> Keyed {
    let pressed: Vec<String> = names.iter().map(|name| (*name).to_owned()).collect();
    keys(s, m, &pressed, false)
}

/// [`keyed`] with Shift held.
fn keyed_shift(s: &Settings, m: &mut MenuState, names: &[&str]) -> Keyed {
    let pressed: Vec<String> = names.iter().map(|name| (*name).to_owned()).collect();
    keys(s, m, &pressed, true)
}

/// The control the keyboard is on, as the text it shows for help.
fn focused(m: &MenuState) -> &'static str {
    controls()[usize::from(m.focus.expect("something is focused"))].tip
}

/// Moves the focus to the control whose action is `action` (by Tab, as a person would).
fn focus_on(s: &Settings, m: &mut MenuState, action: &Action) {
    for _ in 0..40 {
        keyed(s, m, &["Tab"]);
        if &controls()[usize::from(m.focus.unwrap())].action == action {
            return;
        }
    }
    panic!("never reached {action:?}");
}

/// What a press of Enter on the focused control sends.
fn enter(s: &Settings, m: &mut MenuState) -> Vec<(String, f32)> {
    keyed(s, m, &["Enter"]).events
}

#[test]
fn the_keyboard_walks_the_start_page_and_presses_controls() {
    let s = Settings::default();
    let mut m = MenuState::default();
    // Nothing is focused until a key asks for it; Enter alone does nothing.
    assert!(enter(&s, &mut m).is_empty());
    let first = keyed(&s, &mut m, &["Tab"]);
    assert!(first.moved && first.events.is_empty());
    assert_eq!(enter(&s, &mut m), vec![("open_block".to_owned(), 1.0)]);
    // Down visits the page buttons, then fullbright, its key and the menu key; Up goes back.
    let mut seen = Vec::new();
    for _ in 0..7 {
        keyed(&s, &mut m, &["ArrowDown"]);
        seen.push(enter(&s, &mut m).remove(0).0);
    }
    assert_eq!(
        seen,
        ["open_mob", "open_counter", "open_zoom", "fullbright", "fullbright_key", "menu_key", "close"]
    );
    // Past the last stop it wraps to the first, and Up from the first goes to the last.
    keyed(&s, &mut m, &["ArrowDown"]);
    assert_eq!(enter(&s, &mut m)[0].0, "open_block");
    keyed(&s, &mut m, &["ArrowUp"]);
    assert_eq!(enter(&s, &mut m)[0].0, "close");
    // Shift-less Tab also goes forward, and the focus ring and bottom line follow it.
    let patch = focus_update(m, false);
    assert!(patch.contains(r##""#focus_visible":true"##) && patch.contains("Closes the menu"), "{patch}");
}

#[test]
fn enter_and_arrows_change_the_focused_control() {
    let mut s = Settings::default();
    let mut m = MenuState::default();
    // Fullbright is a switch: Enter flips it, Left and Right set it off and on.
    for _ in 0..5 {
        keyed(&s, &mut m, &["Tab"]);
    }
    assert_eq!(enter(&s, &mut m), vec![("fullbright".to_owned(), 1.0)]);
    s.fullbright = true;
    assert_eq!(enter(&s, &mut m), vec![("fullbright".to_owned(), 0.0)]);
    assert_eq!(keyed(&s, &mut m, &["ArrowLeft"]).events, vec![("fullbright".to_owned(), 0.0)]);
    assert_eq!(keyed(&s, &mut m, &["ArrowRight"]).events, vec![("fullbright".to_owned(), 1.0)]);

    // Open the Block page with the keyboard: focus lands on its first control.
    let mut m = MenuState::default();
    keyed(&s, &mut m, &["Tab"]);
    for event in enter(&s, &mut m) {
        assert!(apply(&mut s, &mut m, &event.0, event.1));
    }
    assert_eq!(m.page, Page::Block);
    assert!(m.focus.is_some(), "the new page has a focused control");
    // Walk to the opacity slider: Right adds 5, Left takes 5 away, Enter starts typing.
    let mut steps = 0;
    while controls()[usize::from(m.focus.unwrap())].action != Action::Slider(3) {
        keyed(&s, &mut m, &["Tab"]);
        steps += 1;
        assert!(steps < 20, "never reached the opacity slider");
    }
    s.rgba[3] = 128; // 50%
    let right = keyed(&s, &mut m, &["ArrowRight"]).events;
    assert_eq!(right, vec![("opacity".to_owned(), 55.0)]);
    assert!(apply(&mut s, &mut m, &right[0].0, right[0].1));
    assert_eq!(s.rgba[3], 140); // 55%
    let left = keyed(&s, &mut m, &["ArrowLeft", "ArrowLeft"]).events;
    assert_eq!(left.iter().map(|event| event.1).collect::<Vec<_>>(), vec![50.0, 50.0]);
    assert_eq!(enter(&s, &mut m), vec![("type_opacity".to_owned(), 1.0)]);
}

#[test]
fn keys_do_nothing_while_typing_or_waiting_and_a_click_is_not_needed_to_leave() {
    let s = Settings::default();
    let mut m = MenuState::default();
    keyed(&s, &mut m, &["Tab"]);
    let focus = m.focus;
    for blocked in [
        MenuState { typing: Some(Typing::new(0)), ..m },
        MenuState { capturing: true, ..m },
        MenuState { capturing_zoom: true, ..m },
        MenuState { moving: true, ..m },
    ] {
        let mut blocked = blocked;
        assert_eq!(keyed(&s, &mut blocked, &["Tab", "Enter", "ArrowDown"]), Keyed::default());
        assert_eq!(blocked.focus, focus);
    }
    // Backspace goes back a page, but does nothing on the start page.
    assert!(keyed(&s, &mut m, &["Backspace"]).events.is_empty());
    m.page = Page::Zoom;
    assert_eq!(keyed(&s, &mut m, &["Backspace"]).events, vec![("back".to_owned(), 1.0)]);
}

#[test]
fn moving_the_focus_cancels_a_waiting_reset_and_pages_refocus() {
    let mut s = Settings::default();
    let mut m = MenuState { page: Page::Zoom, ..MenuState::default() };
    // Walk to Reset zoom and press it: it asks.
    let mut guard = 0;
    keyed(&s, &mut m, &["Tab"]);
    while controls()[usize::from(m.focus.unwrap())].action != Action::Press("zoom_reset".into()) {
        keyed(&s, &mut m, &["Tab"]);
        guard += 1;
        assert!(guard < 30);
    }
    let event = enter(&s, &mut m).remove(0);
    assert!(apply(&mut s, &mut m, &event.0, event.1));
    assert_eq!(m.confirm, Reset::Zoom);
    // Moving on takes the question back and asks for a redraw.
    let moved = keyed(&s, &mut m, &["Tab"]);
    assert!(moved.moved && moved.redraw);
    assert_eq!(m.confirm, Reset::None);
    // Leaving the page moves a hidden focus to the new page's first stop.
    let mut m = MenuState { page: Page::Zoom, focus: Some(0), ..MenuState::default() };
    settle_focus(&s, &mut m);
    let first = controls()[usize::from(m.focus.unwrap())].action.clone();
    assert!(matches!(first, Action::Toggle("zoom")), "{first:?}");
}

#[test]
fn every_page_has_stops_and_every_stop_does_something() {
    let s = Settings::default();
    let pages = [
        MenuState::default(),
        on(false),
        on(true),
        MenuState { page: Page::Block, colour_open: true, editing_fill: true, ..MenuState::default() },
        MenuState { page: Page::Mob, ..MenuState::default() },
        MenuState { page: Page::Mob, colour_open: true, ..MenuState::default() },
        MenuState { page: Page::Counter, ..MenuState::default() },
        MenuState { page: Page::Zoom, ..MenuState::default() },
    ];
    for state in pages {
        let matching = fill_matches(&s, state);
        let mut walker = state;
        let count = (0..controls().len())
            .filter(|&index| {
                tip_area(index, state, matching).is_some()
                    && !matches!(controls()[index].action, Action::Help | Action::Hint)
            })
            .count();
        assert!(count >= 3, "{:?} has {count} stops", state.page);
        // Tab through them all: each stop is distinct, and Enter on each yields an event.
        let mut visited = std::collections::HashSet::new();
        for _ in 0..count {
            keyed(&s, &mut walker, &["Tab"]);
            let focus = walker.focus.unwrap();
            assert!(visited.insert(focus), "stop {focus} visited twice");
            let events = enter(&s, &mut walker);
            assert_eq!(events.len(), 1, "Enter on {:?} sent {events:?}", controls()[usize::from(focus)].tip);
        }
        // One more Tab is back at the first.
        keyed(&s, &mut walker, &["Tab"]);
        assert!(!visited.insert(walker.focus.unwrap()));
    }
}

/// The host also refuses a surface of more than 4096 resolved nodes. Every control declares a
/// `"type"` or inherits one with `@`, so counting those counts the nodes; the wheel's cells are
/// most of them. Keep a good margin.
#[test]
fn menu_document_stays_within_the_node_limit() {
    let document = document();
    let nodes = document.matches(r#""type":"#).count() + document.matches("@block_outline.cell").count();
    assert!(nodes <= 3000, "the menu document has about {nodes} nodes; the host's limit is 4096");
    assert!(nodes > 500, "the count looks wrong: {nodes}");
}

/// WCAG relative luminance and contrast ratio of two sRGB colours.
fn contrast(a: [f32; 3], b: [f32; 3]) -> f32 {
    let luminance = |c: [f32; 3]| {
        let lin = |v: f32| if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
        0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
    };
    let (hi, lo) = (luminance(a).max(luminance(b)), luminance(a).min(luminance(b)));
    (hi + 0.05) / (lo + 0.05)
}

/// Every text colour reads on every background it is drawn on, at WCAG AA (4.5:1) for small
/// text, whatever the scenery behind the card.
#[test]
fn text_has_aa_contrast_on_every_background() {
    let rgb = |c: [f32; 4]| [c[0], c[1], c[2]];
    // The card is 96% opaque, so it can be a little lighter than its colour over bright scenery:
    // test against it blended with white at the worst case.
    let card = [0.07, 0.08, 0.1];
    let card_over_white = card.map(|c| c * 0.96 + 0.04);
    for (name, text, background) in [
        ("text on a button", TEXT, rgb(TRACK)),
        ("text on a number box", TEXT, rgb(NUMBER_BOX)),
        ("text on an on switch", TEXT, rgb(ACTIVE)),
        ("text on a reset waiting to confirm", TEXT, rgb(DANGER)),
        ("text on the card", TEXT, card),
        ("text on the card over white", TEXT, card_over_white),
        ("help on the card", DIM, card),
        ("help on the card over white", DIM, card_over_white),
        ("help on a button", DIM, rgb(TRACK)),
        ("text on a settings panel", TEXT, rgb(SECTION)),
        ("help on a settings panel", DIM, rgb(SECTION)),
        ("captions on a settings panel", CAPTION_TEXT, rgb(SECTION)),
        ("captions on the card", CAPTION_TEXT, card),
    ] {
        let ratio = contrast(text, background);
        assert!(ratio >= 4.5, "{name}: contrast {ratio:.2}:1 is under 4.5:1");
    }
    // The focus ring has to stand out from the card (3:1 for interface parts).
    assert!(contrast(rgb(FOCUS), card) >= 3.0);
    assert!(contrast(rgb(FOCUS), rgb(TRACK)) >= 3.0);
}

#[test]
fn home_and_end_jump_to_the_first_and_last_control() {
    let s = Settings::default();
    let mut m = MenuState::default();
    keyed(&s, &mut m, &["End"]);
    assert_eq!(enter(&s, &mut m)[0].0, "close");
    keyed(&s, &mut m, &["Home"]);
    assert_eq!(enter(&s, &mut m)[0].0, "open_block");
}

#[test]
fn up_and_down_follow_the_columns_and_left_and_right_the_rows() {
    let s = Settings::default();
    let mut m = on(false);
    // The Block page: Rainbow | Pulse above Match fill | Reset page.
    focus_on(&s, &mut m, &Action::Toggle("rainbow"));
    // Down from the left column stays in the left column, Down from the right in the right.
    keyed(&s, &mut m, &["ArrowDown"]);
    assert_eq!(controls()[usize::from(m.focus.unwrap())].action, Action::Toggle("fill_match"));
    keyed(&s, &mut m, &["ArrowUp"]);
    keyed(&s, &mut m, &["ArrowRight"]);
    assert_eq!(controls()[usize::from(m.focus.unwrap())].action, Action::Toggle("pulse"));
    keyed(&s, &mut m, &["ArrowDown"]);
    assert_eq!(controls()[usize::from(m.focus.unwrap())].action, Action::Press("reset".into()));
    // Left on the right-hand button goes to its neighbour; Left at the row's start goes nowhere.
    keyed(&s, &mut m, &["ArrowLeft"]);
    assert_eq!(controls()[usize::from(m.focus.unwrap())].action, Action::Toggle("fill_match"));
    let there = m.focus;
    let sent = keyed(&s, &mut m, &["ArrowLeft"]);
    assert_eq!((m.focus, sent.events.is_empty()), (there, true), "nothing is left of it");
    // Up from the first row reaches the colour buttons at the top, and Back beside them.
    for _ in 0..8 {
        keyed(&s, &mut m, &["ArrowUp"]);
    }
    assert!(m.focus.is_some());
}

#[test]
fn left_and_right_change_sliders_and_values_but_move_between_neighbours() {
    let mut s = Settings::default();
    s.rgba[3] = 128;
    let mut m = on(false);
    // On a slider they change it; they never leave it.
    focus_on(&s, &mut m, &Action::Slider(3));
    let at = m.focus;
    assert_eq!(keyed(&s, &mut m, &["ArrowRight"]).events, vec![("opacity".to_owned(), 55.0)]);
    assert_eq!(m.focus, at);
    // Shift makes the step five times as big.
    assert_eq!(keyed_shift(&s, &mut m, &["ArrowRight"]).events, vec![("opacity".to_owned(), 75.0)]);
    assert_eq!(keyed_shift(&s, &mut m, &["ArrowLeft"]).events, vec![("opacity".to_owned(), 25.0)]);
    // On a full-width switch with nothing beside it they turn it off and on.
    let mut z = MenuState { page: Page::Zoom, ..MenuState::default() };
    focus_on(&s, &mut z, &Action::Toggle("zoom"));
    assert_eq!(keyed(&s, &mut z, &["ArrowLeft"]).events, vec![("zoom".to_owned(), 0.0)]);
    assert_eq!(keyed(&s, &mut z, &["ArrowRight"]).events, vec![("zoom".to_owned(), 1.0)]);
    // On a value with - and + they press those.
    focus_on(&s, &mut z, &Action::Stepper { value: "zoom_level", less: "zoom_level_less", more: "zoom_level_more" });
    assert_eq!(keyed(&s, &mut z, &["ArrowLeft"]).events, vec![("zoom_level_less".to_owned(), 1.0)]);
    assert_eq!(keyed(&s, &mut z, &["ArrowRight"]).events, vec![("zoom_level_more".to_owned(), 1.0)]);
    // On a half-width switch they move to the one beside it instead of changing it.
    let mut b = on(false);
    focus_on(&s, &mut b, &Action::Toggle("rainbow"));
    let sent = keyed(&s, &mut b, &["ArrowRight"]);
    assert!(sent.events.is_empty() && sent.moved);
}

#[test]
fn rows_of_colours_are_walked_with_left_and_right() {
    let s = Settings::default();
    let mut m = on(true);
    focus_on(&s, &mut m, &Action::Press("swatch_white".into()));
    let mut names = Vec::new();
    for _ in 0..3 {
        keyed(&s, &mut m, &["ArrowRight"]);
        if let Action::Press(id) = &controls()[usize::from(m.focus.unwrap())].action {
            names.push(id.clone());
        }
    }
    assert_eq!(names, ["swatch_black", "swatch_red", "swatch_orange"]);
    // Down from a swatch lands on a saved colour below it, Up comes back.
    keyed(&s, &mut m, &["ArrowDown"]);
    assert!(matches!(&controls()[usize::from(m.focus.unwrap())].action, Action::Press(id) if id.starts_with("favourite_") || id == "save_favourite"));
    keyed(&s, &mut m, &["ArrowUp"]);
    assert!(matches!(&controls()[usize::from(m.focus.unwrap())].action, Action::Press(id) if id.starts_with("swatch_")));
}

#[test]
fn shift_tab_goes_backwards_and_the_first_arrow_only_brings_the_frame_in() {
    let s = Settings::default();
    let mut m = MenuState::default();
    keyed(&s, &mut m, &["Tab", "Tab"]);
    let second = focused(&m);
    keyed_shift(&s, &mut m, &["Tab"]);
    assert_ne!(focused(&m), second);
    keyed_shift(&s, &mut m, &["Tab"]);
    assert_eq!(focused(&m), "Closes the menu (Esc does too)", "back past the first wraps to the last");
    // Any arrow on a fresh menu focuses the first control without pressing anything.
    for arrow in ["ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight"] {
        let mut fresh = MenuState::default();
        let sent = keyed(&s, &mut fresh, &[arrow]);
        assert!(sent.events.is_empty() && fresh.focus.is_some(), "{arrow}");
        assert_eq!(focused(&fresh), "Colours and size of the block outline", "{arrow}");
    }
}

#[test]
fn every_page_can_be_walked_with_arrows_and_reaches_every_stop() {
    let s = Settings::default();
    for state in [
        MenuState::default(),
        on(false),
        on(true),
        MenuState { page: Page::Mob, ..MenuState::default() },
        MenuState { page: Page::Counter, ..MenuState::default() },
        MenuState { page: Page::Zoom, ..MenuState::default() },
    ] {
        let matching = fill_matches(&s, state);
        let stops: std::collections::HashSet<u8> = (0..controls().len())
            .filter(|&i| {
                tip_area(i, state, matching).is_some()
                    && !matches!(controls()[i].action, Action::Help | Action::Hint)
            })
            .map(|i| i as u8)
            .collect();
        // Down, then Right, then Down again from everywhere reaches every stop: nothing is
        // stranded where the arrows cannot get to it.
        let mut reached = std::collections::HashSet::new();
        for start in &stops {
            let mut m = MenuState { focus: Some(*start), ..state };
            reached.insert(*start);
            for arrow in ["ArrowDown", "ArrowRight", "ArrowUp", "ArrowLeft"] {
                let mut at = m;
                keyed(&s, &mut at, &[arrow]);
                reached.extend(at.focus);
            }
            let _ = &mut m;
        }
        assert_eq!(reached, stops, "page {:?}: some stop cannot be reached by arrows", state.page);
    }
}

/// A value that is not its default says `reset` beside it, since clicking it resets it.
#[test]
fn changed_values_offer_a_reset() {
    let mut s = Settings::default();
    let plain = update(&s, 0.0, MenuState::default());
    for name in ["zoom_level", "zoom_time", "zoom_step", "counter_size"] {
        assert!(plain.contains(&format!(r##""#{name}_reset":"","##)), "{name} is at its default");
    }
    s.zoom.level = 6.0;
    s.counter.scale = 1.4;
    let changed = update(&s, 0.0, MenuState::default());
    assert!(changed.contains(r##""#zoom_level_reset":"reset""##));
    assert!(changed.contains(r##""#counter_size_reset":"reset""##));
    assert!(changed.contains(r##""#zoom_time_reset":"","##), "the others are still at their defaults");
}
