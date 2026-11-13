//! Arrow, Home and End navigation, reset offers and the start page's switches.

use super::*;

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
    assert_eq!(
        controls()[usize::from(m.focus.unwrap())].action,
        Action::Toggle("fill_match")
    );
    keyed(&s, &mut m, &["ArrowUp"]);
    keyed(&s, &mut m, &["ArrowRight"]);
    assert_eq!(
        controls()[usize::from(m.focus.unwrap())].action,
        Action::Toggle("pulse")
    );
    keyed(&s, &mut m, &["ArrowDown"]);
    assert_eq!(
        controls()[usize::from(m.focus.unwrap())].action,
        Action::Press("reset".into())
    );
    // Left on the right-hand button goes to its neighbour; Left at the row's start goes nowhere.
    keyed(&s, &mut m, &["ArrowLeft"]);
    assert_eq!(
        controls()[usize::from(m.focus.unwrap())].action,
        Action::Toggle("fill_match")
    );
    let there = m.focus;
    let sent = keyed(&s, &mut m, &["ArrowLeft"]);
    assert_eq!(
        (m.focus, sent.events.is_empty()),
        (there, true),
        "nothing is left of it"
    );
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
    assert_eq!(
        keyed(&s, &mut m, &["ArrowRight"]).events,
        vec![("opacity".to_owned(), 55.0)]
    );
    assert_eq!(m.focus, at);
    // Shift makes the step five times as big.
    assert_eq!(
        keyed_shift(&s, &mut m, &["ArrowRight"]).events,
        vec![("opacity".to_owned(), 75.0)]
    );
    assert_eq!(
        keyed_shift(&s, &mut m, &["ArrowLeft"]).events,
        vec![("opacity".to_owned(), 25.0)]
    );
    // On a full-width switch with nothing beside it they turn it off and on.
    let mut z = MenuState {
        page: Page::Zoom,
        ..MenuState::default()
    };
    focus_on(&s, &mut z, &Action::Toggle("zoom"));
    assert_eq!(
        keyed(&s, &mut z, &["ArrowLeft"]).events,
        vec![("zoom".to_owned(), 0.0)]
    );
    assert_eq!(
        keyed(&s, &mut z, &["ArrowRight"]).events,
        vec![("zoom".to_owned(), 1.0)]
    );
    // On a value with - and + they press those.
    focus_on(
        &s,
        &mut z,
        &Action::Stepper {
            value: "zoom_level",
            less: "zoom_level_less",
            more: "zoom_level_more",
        },
    );
    assert_eq!(
        keyed(&s, &mut z, &["ArrowLeft"]).events,
        vec![("zoom_level_less".to_owned(), 1.0)]
    );
    assert_eq!(
        keyed(&s, &mut z, &["ArrowRight"]).events,
        vec![("zoom_level_more".to_owned(), 1.0)]
    );
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
    assert!(
        matches!(&controls()[usize::from(m.focus.unwrap())].action, Action::Press(id) if id.starts_with("favourite_") || id == "save_favourite")
    );
    keyed(&s, &mut m, &["ArrowUp"]);
    assert!(
        matches!(&controls()[usize::from(m.focus.unwrap())].action, Action::Press(id) if id.starts_with("swatch_"))
    );
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
    assert_eq!(
        focused(&m),
        "Closes the menu (Esc does too)",
        "back past the first wraps to the last"
    );
    // Any arrow on a fresh menu focuses the first control without pressing anything.
    for arrow in ["ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight"] {
        let mut fresh = MenuState::default();
        let sent = keyed(&s, &mut fresh, &[arrow]);
        assert!(sent.events.is_empty() && fresh.focus.is_some(), "{arrow}");
        assert_eq!(
            focused(&fresh),
            "Colours and size of the block outline",
            "{arrow}"
        );
    }
}

#[test]
fn every_page_can_be_walked_with_arrows_and_reaches_every_stop() {
    let s = Settings::default();
    for state in [
        MenuState::default(),
        on(false),
        on(true),
        MenuState {
            page: Page::Mob,
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
            let mut m = MenuState {
                focus: Some(*start),
                ..state
            };
            reached.insert(*start);
            for arrow in ["ArrowDown", "ArrowRight", "ArrowUp", "ArrowLeft"] {
                let mut at = m;
                keyed(&s, &mut at, &[arrow]);
                reached.extend(at.focus);
            }
            let _ = &mut m;
        }
        assert_eq!(
            reached, stops,
            "page {:?}: some stop cannot be reached by arrows",
            state.page
        );
    }
}

/// A value that is not its default says `reset` beside it, since clicking it resets it.
#[test]
fn changed_values_offer_a_reset() {
    let mut s = Settings::default();
    let plain = update(&s, 0.0, MenuState::default());
    for name in ["zoom_level", "zoom_time", "zoom_step", "counter_size"] {
        assert!(
            plain.contains(&format!(r##""#{name}_reset":"","##)),
            "{name} is at its default"
        );
    }
    s.zoom.level = 6.0;
    s.counter.scale = 1.4;
    let changed = update(&s, 0.0, MenuState::default());
    assert!(changed.contains(r##""#zoom_level_reset":"reset""##));
    assert!(changed.contains(r##""#counter_size_reset":"reset""##));
    assert!(
        changed.contains(r##""#zoom_time_reset":"","##),
        "the others are still at their defaults"
    );
}

/// Every switch on the pages must do something: a switch with a control but no handler looks
/// fine and silently ignores clicks.
#[test]
fn every_switch_changes_a_setting() {
    for name in SWITCHES {
        let page = match name {
            n if n.starts_with("home_") => Page::Home,
            "rainbow" | "pulse" | "fill_match" => Page::Block,
            "show_all_mobs" => Page::Mob,
            "fullbright" => Page::Lighting,
            "auto_queue" => Page::HiveUtilities,
            "show_hunger" | "effect_timers" | "hotbar_keys" => Page::VanillaHud,
            n if n.starts_with("counter") => Page::Counter,
            n if n.starts_with("armor") => Page::Armor,
            n if n.starts_with("zoom") => Page::Zoom,
            n if n.starts_with("jump") => Page::JumpReset,
            n if n.starts_with("crosshair") => Page::Crosshair,
            n if n.starts_with("fps") => Page::Fps,
            _ => Page::Cps,
        };
        let changed = [1.0, 0.0].into_iter().any(|value| {
            let mut settings = Settings::default();
            let mut state = on(false);
            state.page = page;
            let before = settings;
            press(&mut settings, &mut state, name, value);
            settings != before
        });
        assert!(changed, "the {name} switch changes nothing");
    }
}

/// The start page's switches turn each mod on and off, and a mod that is off draws nothing.
#[test]
fn the_start_page_switches_turn_mods_on_and_off() {
    let mut s = Settings::default();
    let mut m = MenuState::default();
    for id in [
        "home_block",
        "home_mob",
        "home_counter",
        "home_cps",
        "home_zoom",
        "home_light",
        "home_jump",
    ] {
        assert!(press(&mut s, &mut m, id, 0.0) || id == "home_light", "{id}");
    }
    assert!(!s.block_on && !s.mob_on && !s.counter.on && !s.cps.on && !s.zoom.on);
    assert!(!s.jump_reset.on);
    assert!(!s.fullbright);
    assert!(press(&mut s, &mut m, "home_light", 1.0));
    assert!(s.fullbright);
    for id in [
        "home_block",
        "home_mob",
        "home_counter",
        "home_cps",
        "home_zoom",
        "home_jump",
    ] {
        assert!(press(&mut s, &mut m, id, 1.0), "{id}");
    }
    assert!(s.block_on && s.mob_on && s.counter.on && s.cps.on && s.zoom.on && s.jump_reset.on);
    // Their switches show it, and the page's own switches agree.
    let update = update(&s, 0.0, m);
    for id in [
        "home_block",
        "home_mob",
        "home_counter",
        "home_cps",
        "home_zoom",
        "home_jump",
    ] {
        assert!(switch_on(&update, id), "{id}");
    }
    assert!(switch_on(&update, "home_light"));
    // Only on the start page.
    m.page = Page::Zoom;
    assert!(!press(&mut s, &mut m, "home_block", 0.0));
    assert!(s.block_on);
    // A reset keeps them as they are.
    s.block_on = false;
    s.reset();
    assert!(!s.block_on);
}
