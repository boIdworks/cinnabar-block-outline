//! What the menu document says: reset questions, buttons, the controls table and text fit.

use super::*;

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
    assert!(
        asking.contains(r##""#reset_bg":[0.62,0.2,0.18,1]"##),
        "{asking}"
    );
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
    let zoom = MenuState {
        capturing_zoom: true,
        ..MenuState::default()
    };
    assert!(update(&s, 0.0, zoom).contains("Esc cancels, Del clears"));
    let menu = MenuState {
        capturing: true,
        ..MenuState::default()
    };
    assert!(update(&s, 0.0, menu).contains("Press key, Esc cancels"));
}

/// The buttons of the real document, as `[x, y, w, h]` on the card (the moving guide, which
/// has its own coordinates, comes after the card and is left out).
pub(super) fn document_buttons() -> Vec<[f32; 4]> {
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

pub(super) fn contains(outer: [f32; 4], inner: [f32; 4]) -> bool {
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
            buttons
                .iter()
                .any(|b| (0..4).all(|i| (b[i] - area[i]).abs() < 0.01))
        };
        let (plain, lifted) = (control.area, shifted(control.area));
        match &control.action {
            Action::Press(_) | Action::Key(_) | Action::Toggle(_) | Action::Hex => {
                // Key rows hold a label and a button; the others are exactly their button.
                let key_row = matches!(control.action, Action::Key(_));
                let ok = if key_row {
                    inside(plain) || inside(lifted)
                } else {
                    exact(plain) || exact(lifted)
                };
                assert!(
                    ok,
                    "the control \"{}\" at {:?} is not a button",
                    control.tip, control.area
                );
            }
            Action::Slider(_) | Action::Stepper { .. } => {
                assert!(
                    inside(plain) || inside(lifted),
                    "no button in the row at {x},{y},{w},{h}"
                );
            }
            Action::Help => {
                assert!(
                    inside(plain)
                        || inside(lifted)
                        || exact(plain)
                        || control.tip.contains("click")
                        || control.tip.contains("less or more")
                        || control.tip.contains("smaller or bigger")
                        || control.tip.contains("weaker or stronger")
                        || control.tip.contains("Click, type")
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
    assert!(
        !fits("It snaps to the hotbar and screen middle", 200.0),
        "the check must catch it"
    );
    for line in GUIDE_LINES {
        assert!(
            fits(line, GUIDE_W - GUIDE_TEXT_X - 10.0),
            "guide line too long: {line}"
        );
    }
    for control in controls() {
        // `%%` shows as one `%`.
        assert!(
            fits(&control.tip.replace("%%", "%"), TIP_WIDTH - 8.0),
            "tip too long: {}",
            control.tip
        );
        assert!(
            fits(&control.tip.replace("%%", "%"), SETTINGS_W - 20.0),
            "hint too long: {}",
            control.tip
        );
    }
    assert!(fits(HINT_DEFAULT, SETTINGS_W - 20.0));
    // What the start page buttons say, beside the badge and before the arrow.
    for about in HOME_ABOUT {
        assert!(
            fits(about, SETTINGS_W - 70.0),
            "home description too long: {about}"
        );
    }
    // The key buttons while they wait, in the narrowest button that shows them.
    for text in [CAPTURE_MENU, CAPTURE_OPTIONAL] {
        assert!(fits(text, SETTINGS_W - 10.0 - KEY_X - 10.0), "{text}");
    }
    // A reset button's question in the narrowest button (half the card).
    assert!(fits("Sure? Click again", HALF - 10.0));
    // The title, the longest trail in the room left of Back.
    for state in [
        MenuState {
            page: Page::Block,
            colour_open: true,
            editing_fill: true,
            ..MenuState::default()
        },
        MenuState {
            page: Page::Mob,
            colour_open: true,
            editing_aim: true,
            ..MenuState::default()
        },
        MenuState {
            page: Page::Counter,
            ..MenuState::default()
        },
    ] {
        let patch = update(&Settings::default(), 0.0, state);
        let title = patch
            .split(r##""#title_text":""##)
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        // The title is drawn larger.
        assert!(
            title.len() as f32 * 5.0 * TITLE_SCALE + 6.0 <= BACK_AREA[0] - 14.0 - TITLE_X,
            "title too long: {title}"
        );
    }
}
