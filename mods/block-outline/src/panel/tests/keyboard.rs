//! Keyboard walking of the start page, the document limits and text contrast.

use super::*;

pub(super) fn keyed(s: &Settings, m: &mut MenuState, names: &[&str]) -> Keyed {
    let pressed: Vec<String> = names.iter().map(|name| (*name).to_owned()).collect();
    keys(s, m, &pressed, false)
}

/// [`keyed`] with Shift held.
pub(super) fn keyed_shift(s: &Settings, m: &mut MenuState, names: &[&str]) -> Keyed {
    let pressed: Vec<String> = names.iter().map(|name| (*name).to_owned()).collect();
    keys(s, m, &pressed, true)
}

/// The control the keyboard is on, as the text it shows for help.
pub(super) fn focused(m: &MenuState) -> &'static str {
    controls()[usize::from(m.focus.expect("something is focused"))].tip
}

/// Moves the focus to the control whose action is `action` (by Tab, as a person would).
pub(super) fn focus_on(s: &Settings, m: &mut MenuState, action: &Action) {
    for _ in 0..40 {
        keyed(s, m, &["Tab"]);
        if &controls()[usize::from(m.focus.unwrap())].action == action {
            return;
        }
    }
    panic!("never reached {action:?}");
}

/// What a press of Enter on the focused control sends.
pub(super) fn enter(s: &Settings, m: &mut MenuState) -> Vec<(String, f32)> {
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
    // Down visits the page buttons, then the menu key; Up goes back.
    let mut seen = Vec::new();
    for _ in 0..10 {
        keyed(&s, &mut m, &["ArrowDown"]);
        seen.push(enter(&s, &mut m).remove(0).0);
    }
    assert_eq!(
        seen,
        [
            "open_mob",
            "open_counter",
            "open_cps",
            "open_fps",
            "open_zoom",
            "open_light",
            "open_jump",
            "open_crosshair",
            "menu_key",
            "close"
        ]
    );
    // Down from the close button reaches the switch right below it, and Up comes back.
    keyed(&s, &mut m, &["ArrowDown"]);
    assert_eq!(enter(&s, &mut m), vec![("home_block".to_owned(), 0.0)]);
    keyed(&s, &mut m, &["ArrowUp"]);
    assert_eq!(enter(&s, &mut m)[0].0, "close");
    // Right from a page button reaches its switch, which Enter flips, and Left goes back.
    keyed(&s, &mut m, &["Tab"]);
    assert_eq!(enter(&s, &mut m)[0].0, "open_block");
    keyed(&s, &mut m, &["ArrowRight"]);
    assert_eq!(enter(&s, &mut m), vec![("home_block".to_owned(), 0.0)]);
    keyed(&s, &mut m, &["ArrowLeft"]);
    assert_eq!(enter(&s, &mut m)[0].0, "open_block");
    // Shift-less Tab also goes forward, and the focus ring and bottom line follow it.
    let patch = focus_update(m, false);
    assert!(
        patch.contains(r##""#focus_visible":true"##)
            && patch.contains("Colours and size of the block"),
        "{patch}"
    );
}

#[test]
fn enter_and_arrows_change_the_focused_control() {
    let mut s = Settings::default();
    // Fullbright is a switch: Enter flips it, Left and Right set it off and on.
    let mut m = MenuState {
        page: Page::Lighting,
        ..MenuState::default()
    };
    keyed(&s, &mut m, &["Tab"]);
    assert_eq!(enter(&s, &mut m), vec![("fullbright".to_owned(), 1.0)]);
    s.fullbright = true;
    assert_eq!(enter(&s, &mut m), vec![("fullbright".to_owned(), 0.0)]);
    assert_eq!(
        keyed(&s, &mut m, &["ArrowLeft"]).events,
        vec![("fullbright".to_owned(), 0.0)]
    );
    assert_eq!(
        keyed(&s, &mut m, &["ArrowRight"]).events,
        vec![("fullbright".to_owned(), 1.0)]
    );

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
    assert_eq!(
        left.iter().map(|event| event.1).collect::<Vec<_>>(),
        vec![50.0, 50.0]
    );
    assert_eq!(enter(&s, &mut m), vec![("type_opacity".to_owned(), 1.0)]);
}

#[test]
fn keys_do_nothing_while_typing_or_waiting_and_a_click_is_not_needed_to_leave() {
    let s = Settings::default();
    let mut m = MenuState::default();
    keyed(&s, &mut m, &["Tab"]);
    let focus = m.focus;
    for blocked in [
        MenuState {
            typing: Some(Typing::new(0)),
            ..m
        },
        MenuState {
            capturing: true,
            ..m
        },
        MenuState {
            capturing_zoom: true,
            ..m
        },
        MenuState { moving: true, ..m },
    ] {
        let mut blocked = blocked;
        assert_eq!(
            keyed(&s, &mut blocked, &["Tab", "Enter", "ArrowDown"]),
            Keyed::default()
        );
        assert_eq!(blocked.focus, focus);
    }
    // Backspace goes back a page, but does nothing on the start page.
    assert!(keyed(&s, &mut m, &["Backspace"]).events.is_empty());
    m.page = Page::Zoom;
    assert_eq!(
        keyed(&s, &mut m, &["Backspace"]).events,
        vec![("back".to_owned(), 1.0)]
    );
}

#[test]
fn moving_the_focus_cancels_a_waiting_reset_and_pages_refocus() {
    let mut s = Settings::default();
    let mut m = MenuState {
        page: Page::Zoom,
        ..MenuState::default()
    };
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
    let mut m = MenuState {
        page: Page::Zoom,
        focus: Some(0),
        ..MenuState::default()
    };
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
        MenuState {
            page: Page::Block,
            colour_open: true,
            editing_fill: true,
            ..MenuState::default()
        },
        MenuState {
            page: Page::Mob,
            ..MenuState::default()
        },
        MenuState {
            page: Page::Mob,
            colour_open: true,
            ..MenuState::default()
        },
        MenuState {
            page: Page::Counter,
            ..MenuState::default()
        },
        MenuState {
            page: Page::Zoom,
            ..MenuState::default()
        },
        MenuState {
            page: Page::Crosshair,
            ..MenuState::default()
        },
        MenuState {
            page: Page::Crosshair,
            colour_open: true,
            ..MenuState::default()
        },
        MenuState {
            page: Page::Crosshair,
            drawing: true,
            ..MenuState::default()
        },
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
            assert_eq!(
                events.len(),
                1,
                "Enter on {:?} sent {events:?}",
                controls()[usize::from(focus)].tip
            );
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
    let nodes = document.matches(r#""type":"#).count() + document.matches("@bo.").count();
    assert!(
        nodes <= 3000,
        "the menu document has about {nodes} nodes; the host's limit is 4096"
    );
    assert!(nodes > 500, "the count looks wrong: {nodes}");
}

/// WCAG relative luminance and contrast ratio of two sRGB colours.
pub(super) fn contrast(a: [f32; 3], b: [f32; 3]) -> f32 {
    let luminance = |c: [f32; 3]| {
        let lin = |v: f32| {
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
    };
    let (hi, lo) = (
        luminance(a).max(luminance(b)),
        luminance(a).min(luminance(b)),
    );
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
