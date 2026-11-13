//! The counter, jump reset and zoom pages, and the document snapshots.

use super::*;

#[test]
fn arrow_counter_page_and_moving() {
    let mut s = Settings::default();
    let mut m = MenuState::default();
    assert!(!s.counter.on, "off until asked for");
    assert!(press(&mut s, &mut m, "home_counter", 1.0));
    assert!(s.counter.on);
    assert!(press(&mut s, &mut m, "open_counter", 1.0));
    assert_eq!(m.page, Page::Counter);
    let shown = update(&s, 0.0, m);
    assert!(
        shown.contains(r#""show_gameplay_hud":true"#),
        "the HUD stays visible: {shown}"
    );
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
    let low = MenuState {
        guide_bottom: true,
        ..m
    };
    assert_eq!(
        guide_update(low),
        r##"{"binding_changes":{"#guide_top_visible":false,"#guide_bottom_visible":true}}"##
    );
    assert_eq!(tip_at([15.0, COUNTER_ON_Y + 5.0], m, false), None);
    assert!(press(&mut s, &mut m, "counter_done", 1.0));
    assert!(!m.moving);

    s.counter.pos = crate::counter::Pos::fraction(0.1, 0.2);
    s.counter.scale = 2.0;
    assert!(confirmed(&mut s, &mut m, "counter_reset"));
    assert_eq!(
        s.counter,
        crate::counter::Counter {
            on: true,
            bow_only: true,
            ..Default::default()
        }
    );

    let tip = |point| tip_at(point, m, false).map(|index| controls()[index].tip);
    assert_eq!(
        tip([15.0, COUNTER_MOVE_Y + 5.0]),
        Some("Move the HUD: drag, scroll to resize")
    );
    assert_eq!(
        tip([SETTINGS_W - 15.0, COUNTER_SIZE_Y + 5.0]),
        Some("10%% smaller or bigger")
    );
    assert_eq!(
        tip_at([15.0, HOME_COUNTER_Y + 5.0], MenuState::default(), false)
            .map(|index| controls()[index].tip),
        Some("Shows how many arrows you carry")
    );
    // Back goes to the start page.
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert_eq!(m.page, Page::Home);
}

#[test]
fn the_fuel_line_shows_only_once_a_frame_was_measured() {
    let mut state = MenuState::default();
    assert_eq!(
        fuel_update(state),
        r##"{"binding_changes":{"#fuel_text":""}}"##
    );
    state.fuel_k = [41, 96, 120];
    assert_eq!(
        fuel_update(state),
        r##"{"binding_changes":{"#fuel_text":"Fuel: 41k avg, 96k recent, 120k peak"}}"##
    );
    assert!(update(&Settings::default(), 0.0, state).contains("120k peak"));
    // The menu clips a line at about 40 characters, so the longest it can be must stay under.
    state.fuel_k = [199, 199, 199];
    let line = fuel_update(state);
    let text = line.split('"').nth(5).expect("the text");
    assert!(text.len() <= 40, "{text} is {} characters", text.len());
}

#[test]
fn counter_settings_round_trip() {
    let s = Settings {
        counter: crate::counter::Counter {
            on: false,
            bow_only: true,
            pos: crate::counter::Pos {
                anchor: [crate::counter::Anchor::Centre, crate::counter::Anchor::End],
                off: [93.5, -12.25],
            },
            scale: 1.5,
        },
        ..Settings::default()
    };
    let back = Settings::parse(&s.to_json());
    assert_eq!(back.counter, s.counter);
    // Out-of-range values are clamped; older files keep the defaults.
    let odd = Settings::parse(
        r#"{"arrow_counter_x": 3, "arrow_counter_y": -1, "arrow_counter_scale": 99}"#,
    );
    assert_eq!(odd.counter.pos, crate::counter::Pos::fraction(1.0, 0.0));
    assert_eq!(odd.counter.scale, crate::counter::MAX_SCALE);
    assert_eq!(
        Settings::parse("{}").counter,
        crate::counter::Counter::default()
    );
    // Resetting an outline page keeps the counter.
    let mut reset = s;
    reset.reset();
    assert_eq!(reset.counter, s.counter);
}

/// The host refuses a menu document over 192 KiB, which would stop the whole mod loading. Keep
/// some room: the colour wheel is most of it, so a new page costs about 1 KiB a control. (The
/// Jump reset page and its start page button took this from 172 to 178 KiB: the limit is 192.
/// Button backgrounds inheriting one template and the namespace shortened to `bo`, which every
/// wheel cell repeats, then saved 24 KiB, which the Crosshair page used most of: 173 KiB. The
/// FPS counter page took it to 185 KiB; rectangles inheriting the `r` template then saved
/// 13 KiB: 172 KiB. The armor page and the Vanilla HUD switches took it to 184 KiB; the reserve
/// went from 184 to 186 KiB for the effect-timer and hotbar-key switches: 184.3 KiB, so 7.7 KiB
/// of the limit is still free. The coordinates switch took it to 186.9 KiB, so the reserve went
/// to 188 KiB: 5.1 KiB of the limit is still free. The Hive Utilities page and its start page
/// button (a button with its icon is about 1.4 KiB, the page 1.6 KiB) took it to 190.2 KiB, so
/// the reserve went to 191 KiB: only 1.8 KiB of the limit is free, so the next page needs the
/// document compacted first.)
#[test]
fn menu_document_stays_within_the_host_limit() {
    let size = document().len();
    assert!(
        size <= 191 * 1024,
        "the menu document is {size} bytes; the host's limit is {} and some room is kept",
        192 * 1024
    );
}

#[test]
fn the_jump_reset_page_has_a_switch_and_a_window_that_steps() {
    let mut s = Settings::default();
    let mut m = MenuState::default();
    assert!(!s.jump_reset.on, "off until asked for");
    assert!(press(&mut s, &mut m, "home_jump", 1.0));
    assert!(s.jump_reset.on);
    assert!(press(&mut s, &mut m, "open_jump", 1.0));
    assert_eq!(m.page, Page::JumpReset);
    let shown = update(&s, 0.0, m);
    assert!(
        shown.contains(r##""#title_text":"Jump reset""##)
            && shown.contains(r##""#jump_page_visible":true"##)
            && shown.contains(r##""#cps_page_visible":false"##),
        "{shown}"
    );
    // The HUD stays visible behind the page, so the sample readout can be seen.
    assert!(shown.contains(r##""show_gameplay_hud":true"##));
    assert!(switch_on(&shown, "jump_on"));
    assert!(shown.contains(r##""#jump_window_text":"Perfect window: 75 ms""##));
    assert!(shown.contains(r##""#jump_window_reset":"""##));
    // The window steps by 5 ms, stays in bounds, and its own button resets it.
    assert!(press(&mut s, &mut m, "jump_window_more", 1.0));
    assert_eq!(s.jump_reset.window, 80);
    let shown = update(&s, 0.0, m);
    assert!(shown.contains(r##""#jump_window_text":"Perfect window: 80 ms""##));
    assert!(shown.contains(r##""#jump_window_reset":"reset""##));
    for _ in 0..60 {
        press(&mut s, &mut m, "jump_window_more", 1.0);
    }
    assert_eq!(s.jump_reset.window, crate::jumpreset::MAX_WINDOW);
    assert!(
        !press(&mut s, &mut m, "jump_window_more", 1.0),
        "at the top"
    );
    for _ in 0..60 {
        press(&mut s, &mut m, "jump_window_less", 1.0);
    }
    assert_eq!(s.jump_reset.window, crate::jumpreset::MIN_WINDOW);
    assert!(press(&mut s, &mut m, "jump_window", 1.0));
    assert_eq!(s.jump_reset.window, crate::jumpreset::DEFAULT_WINDOW);
    // The switch, here and on the start page, is one setting.
    assert!(press(&mut s, &mut m, "jump_on", 0.0));
    assert!(!s.jump_reset.on);
    assert!(!switch_on(&update(&s, 0.0, m), "jump_on"));
    assert!(
        !press(&mut s, &mut m, "home_jump", 1.0),
        "only on the start page"
    );
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert!(press(&mut s, &mut m, "home_jump", 1.0));
    assert!(s.jump_reset.on);
    assert!(switch_on(&update(&s, 0.0, m), "home_jump"));
    // Resetting an outline page keeps it.
    s.jump_reset.window = 120;
    s.reset();
    assert_eq!(s.jump_reset.window, 120);
}

#[test]
fn the_keyboard_reaches_every_control_of_the_jump_reset_page() {
    let s = Settings::default();
    let mut m = MenuState {
        page: Page::JumpReset,
        ..MenuState::default()
    };
    focus_on(&s, &mut m, &Action::Toggle("jump_on"));
    assert_eq!(focused(&m), "Shows early or late when you jump");
    focus_on(
        &s,
        &mut m,
        &Action::Stepper {
            value: "jump_window",
            less: "jump_window_less",
            more: "jump_window_more",
        },
    );
    // Right presses the + button, Enter the value (its reset).
    assert_eq!(
        keyed(&s, &mut m, &["ArrowRight"]).events,
        vec![("jump_window_more".to_owned(), 1.0)]
    );
    assert_eq!(enter(&s, &mut m), vec![("jump_window".to_owned(), 1.0)]);
    // Its help shows where the pointer is.
    let state = m;
    let tip = |point| tip_at(point, state, false).map(|index| controls()[index].tip);
    assert_eq!(
        tip([15.0, JUMP_WINDOW_Y + 5.0]),
        Some("Jumps this close to the hit are perfect")
    );
    assert_eq!(
        tip([SETTINGS_W - 15.0, JUMP_WINDOW_Y + 5.0]),
        Some("5 ms narrower or wider")
    );
}

#[test]
fn zoom_page_values_switches_and_key() {
    let mut s = Settings::default();
    let mut m = MenuState::default();
    assert!(!s.zoom.on, "off until asked for");
    assert!(press(&mut s, &mut m, "home_zoom", 1.0));
    assert!(s.zoom.on);
    assert!(press(&mut s, &mut m, "open_zoom", 1.0));
    assert_eq!(m.page, Page::Zoom);
    let shown = update(&s, 0.0, m);
    assert!(
        shown.contains(r##""#title_text":"Zoom""##)
            && shown.contains(r##""#zoom_page_visible":true"##)
    );
    for hidden in ["settings", "colour", "counter_page"] {
        assert!(
            shown.contains(&format!(r##""#{hidden}_visible":false"##)),
            "{hidden}: {shown}"
        );
    }
    assert!(switch_on(&shown, "zoom"));
    assert!(
        !switch_on(&shown, "zoom_mode"),
        "hold the key is the default"
    );
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
    assert!(
        !press(&mut s, &mut m, "zoom_level_more", 1.0),
        "already at the top"
    );
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
    // Scrolling moves the hotbar slot too, so it starts off and the switch says so.
    assert!(!s.zoom.scroll);
    assert!(!switch_on(&shown, "zoom_scroll"));
    assert!(press(&mut s, &mut m, "zoom_scroll", 1.0) && s.zoom.scroll);
    assert!(switch_on(&update(&s, 0.0, m), "zoom_scroll"));
    assert!(press(&mut s, &mut m, "zoom_scroll", 0.0) && !s.zoom.scroll);
    assert!(press(&mut s, &mut m, "zoom_sens", 0.0) && !s.zoom.sensitivity);
    assert!(press(&mut s, &mut m, "zoom", 0.0) && !s.zoom.on);
    // Reset keeps the chosen key.
    s.zoom.key = KeyName::new("KeyC");
    assert!(confirmed(&mut s, &mut m, "zoom_reset"));
    assert_eq!(
        s.zoom,
        crate::zoom::Zoom {
            key: KeyName::new("KeyC"),
            ..Default::default()
        }
    );
    // An unbound key shows as not set, and the reset keeps it unbound.
    s.zoom.key = None;
    assert!(update(&s, 0.0, m).contains(r##""#zoom_key_text":"Click to set""##));
    s.zoom.scroll = true;
    assert!(confirmed(&mut s, &mut m, "zoom_reset"));
    assert_eq!(s.zoom.key, None);
    assert!(!s.zoom.scroll);
    s.zoom.key = KeyName::new("KeyZ");

    // Waiting for the zoom key keeps Escape for itself, and the button says so.
    m.capturing_zoom = true;
    let waiting = update(&s, 0.0, m);
    assert!(waiting.contains(r#""capture_key":true"#));
    assert!(waiting.contains(r##""#zoom_key_text":"Esc cancels, Del clears""##));
    m.capturing_zoom = false;

    // Hover help, the start page's button, and the card fits the page.
    let tip = |point| tip_at(point, m, false).map(|index| controls()[index].tip);
    assert_eq!(
        tip([15.0, ZOOM_MODE_Y + 5.0]),
        Some("Hold the key, or press to toggle")
    );
    assert_eq!(
        tip([SETTINGS_W - 15.0, ZOOM_STEP_Y + 5.0]),
        Some("5%% smaller or bigger")
    );
    assert_eq!(
        tip_at([15.0, HOME_ZOOM_Y + 5.0], MenuState::default(), false)
            .map(|index| controls()[index].tip),
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
    let asking = MenuState {
        confirm: Reset::Page,
        ..on(false)
    };
    let capture = MenuState {
        capturing_zoom: true,
        ..zoom
    };
    let home_capture = MenuState {
        capturing: true,
        ..MenuState::default()
    };
    let mob = MenuState {
        page: Page::Mob,
        ..MenuState::default()
    };
    let stop = |action: Action| {
        controls()
            .iter()
            .position(|c| c.action == action)
            .map(|i| i as u8)
    };
    let focus_slider = MenuState {
        focus: stop(Action::Slider(3)),
        ..on(false)
    };
    let focus_switch = MenuState {
        focus: stop(Action::Toggle("rainbow")),
        ..on(false)
    };
    for (name, state) in [
        ("home", MenuState::default()),
        ("counter", counter),
        ("moving", moving),
        ("block", on(false)),
        ("colour", on(true)),
        ("zoom", zoom),
        (
            "cps",
            MenuState {
                page: Page::Cps,
                ..MenuState::default()
            },
        ),
        (
            "fps",
            MenuState {
                page: Page::Fps,
                ..MenuState::default()
            },
        ),
        (
            "lighting",
            MenuState {
                page: Page::Lighting,
                ..MenuState::default()
            },
        ),
        (
            "jump",
            MenuState {
                page: Page::JumpReset,
                ..MenuState::default()
            },
        ),
        ("asking", asking),
        ("capture", capture),
        ("home_capture", home_capture),
        ("mob", mob),
        ("focus_slider", focus_slider),
        ("focus_switch", focus_switch),
    ] {
        std::fs::write(format!("{dir}/{name}.json"), content_in(&s, 0.0, state)).unwrap();
    }
    // A page with changed values, which say `reset` on their right.
    let mut changed = s;
    changed.zoom.level = 6.0;
    changed.counter.scale = 1.4;
    std::fs::write(
        format!("{dir}/zoom_changed.json"),
        content_in(&changed, 0.0, zoom),
    )
    .unwrap();
    std::fs::write(
        format!("{dir}/counter_surface.json"),
        crate::counter::surface(),
    )
    .unwrap();
    for (name, arrows, scale, moving) in [
        ("counter_hud", 64, 1.0, false),
        ("counter_big", 0, 3.0, true),
    ] {
        let width = crate::counter::guess_width(crate::counter::digits(arrows));
        let size = crate::counter::size(scale, width);
        let view = crate::counter::View {
            visible: true,
            arrows: Some(arrows),
            offset: crate::counter::place(
                crate::counter::Pos::fraction(0.5, 0.4),
                size,
                [426.0, 240.0],
            ),
            size,
            scale,
            text_width: width,
            edit: moving.then_some(crate::counter::Edit {
                hover: Some(crate::counter::Part::Handle(3)),
                guides: [Some(213.0), Some(218.0)],
                ..Default::default()
            }),
        };
        std::fs::write(
            format!("{dir}/{name}.patch.json"),
            crate::counter::patch(&view),
        )
        .unwrap();
    }
}

#[test]
fn armor_options_are_accessible_and_persist() {
    let mut s = Settings::default();
    let mut menu = MenuState::default();
    assert!(!s.armor.on, "the armor HUD is off until asked for");
    assert!(press(&mut s, &mut menu, "home_armor", 1.0));
    assert!(s.armor.on);
    assert!(press(&mut s, &mut menu, "home_armor", 0.0));
    assert!(!s.armor.on);
    assert!(press(&mut s, &mut menu, "home_armor", 1.0));
    assert!(s.armor.on);
    assert!(press(&mut s, &mut menu, "open_armor", 1.0));
    assert_eq!(menu.page, Page::Armor);
    assert!(press(&mut s, &mut menu, "armor_horizontal", 1.0));
    assert!(press(&mut s, &mut menu, "armor_percent", 0.0));
    assert!(press(&mut s, &mut menu, "armor_background", 1.0));
    assert!(s.armor.horizontal && !s.armor.percent && s.armor.background);
    assert_eq!(Settings::parse(&s.to_json()).armor, s.armor);
}

#[test]
fn vanilla_hunger_switch_is_saved() {
    let mut settings = Settings::default();
    let mut menu = MenuState::default();
    assert!(press(&mut settings, &mut menu, "open_vanilla", 1.0));
    assert_eq!(menu.page, Page::VanillaHud);
    assert!(press(&mut settings, &mut menu, "show_hunger", 0.0));
    assert!(settings.hide_hunger);
    assert!(Settings::parse(&settings.to_json()).hide_hunger);
    assert!(press(&mut settings, &mut menu, "show_hunger", 1.0));
    assert!(!settings.hide_hunger);
}

#[test]
fn effect_timer_and_hotbar_key_switches_are_saved_and_default_off() {
    // A settings file from before these existed leaves both off.
    let old = Settings::parse(r#"{"block_on": true}"#);
    assert!(!old.effect_timers && !old.hotbar_keys);
    let mut settings = Settings::default();
    let mut menu = MenuState::default();
    assert!(press(&mut settings, &mut menu, "open_vanilla", 1.0));
    assert!(press(&mut settings, &mut menu, "effect_timers", 1.0));
    assert!(press(&mut settings, &mut menu, "hotbar_keys", 1.0));
    assert!(settings.effect_timers && settings.hotbar_keys);
    let saved = Settings::parse(&settings.to_json());
    assert!(saved.effect_timers && saved.hotbar_keys);
    assert!(press(&mut settings, &mut menu, "effect_timers", 0.0));
    assert!(press(&mut settings, &mut menu, "hotbar_keys", 0.0));
    assert!(!settings.effect_timers && !settings.hotbar_keys);
    let saved = Settings::parse(&settings.to_json());
    assert!(!saved.effect_timers && !saved.hotbar_keys);
    assert!(press(&mut settings, &mut menu, "effect_timers", 1.0));
    assert!(settings.effect_timers && !settings.hotbar_keys);
    // The block outline reset leaves them as they were.
    settings.reset();
    assert!(settings.effect_timers && !settings.hotbar_keys);
}

#[test]
fn armor_move_opens_editor_and_reset_restores_position() {
    let mut settings = Settings::default();
    let mut menu = MenuState {
        page: Page::Armor,
        ..MenuState::default()
    };
    assert!(press(&mut settings, &mut menu, "armor_move", 1.0));
    assert!(menu.moving && menu.moving_armor && settings.armor.on);
    settings.armor.pos = crate::counter::Pos::fraction(0.4, 0.3);
    settings.armor.scale = 2.0;
    assert!(press(&mut settings, &mut menu, "counter_reset", 1.0));
    assert_eq!(settings.armor.pos, crate::armor::DEFAULT_POS);
    assert_eq!(settings.armor.scale, 1.0);
    assert!(press(&mut settings, &mut menu, "counter_done", 1.0));
    assert!(!menu.moving && !menu.moving_armor);
}

#[test]
fn armor_size_controls_match_other_hud_counters() {
    let mut settings = Settings::default();
    let mut menu = MenuState {
        page: Page::Armor,
        ..MenuState::default()
    };
    assert!(press(&mut settings, &mut menu, "armor_size_more", 1.0));
    assert!((settings.armor.scale - 1.1).abs() < 0.001);
    assert_eq!(
        Settings::parse(&settings.to_json()).armor.scale,
        settings.armor.scale
    );
    assert!(confirmed(&mut settings, &mut menu, "armor_reset"));
    assert_eq!(settings.armor.scale, 1.0);
    assert_eq!(settings.armor.pos, crate::armor::DEFAULT_POS);
}

#[test]
fn the_fps_page_has_a_coordinates_switch_that_starts_off() {
    let mut s = Settings::default();
    let mut menu = MenuState::default();
    assert!(!s.fps.coords, "off until asked for");
    assert!(press(&mut s, &mut menu, "open_fps", 1.0));
    assert_eq!(menu.page, Page::Fps);
    assert!(!switch_on(&update(&s, 0.0, menu), "fps_coords"));
    assert!(press(&mut s, &mut menu, "fps_coords", 1.0));
    assert!(s.fps.coords && !s.fps.on, "the FPS stays as it was");
    assert!(switch_on(&update(&s, 0.0, menu), "fps_coords"));
    // It is its own switch: turning the FPS off leaves the coordinates on.
    assert!(press(&mut s, &mut menu, "fps_on", 1.0));
    assert!(s.fps.on && s.fps.coords);
    assert!(press(&mut s, &mut menu, "fps_on", 0.0));
    assert!(!s.fps.on && s.fps.coords);
    assert!(press(&mut s, &mut menu, "fps_coords", 0.0));
    assert!(!s.fps.coords);
}

#[test]
fn the_auto_queue_switch_starts_off_changes_the_setting_and_is_saved() {
    let mut s = Settings::default();
    let mut menu = MenuState::default();
    assert!(!s.auto_queue, "off until asked for");
    assert!(press(&mut s, &mut menu, "open_hive", 1.0));
    assert_eq!(menu.page, Page::HiveUtilities);
    assert!(!switch_on(&update(&s, 0.0, menu), "auto_queue"));
    assert!(press(&mut s, &mut menu, "auto_queue", 1.0));
    assert!(s.auto_queue && !s.fullbright, "its own switch");
    assert!(switch_on(&update(&s, 0.0, menu), "auto_queue"));
    // What the menu changed is what the settings file holds, and it reads back.
    let saved = s.to_json();
    assert!(saved.contains("\"auto_queue\": true"), "{saved}");
    assert!(Settings::parse(&saved).auto_queue);
    assert!(press(&mut s, &mut menu, "auto_queue", 0.0));
    assert!(!s.auto_queue);
}

#[test]
fn every_feature_starts_off_so_a_new_user_chooses_what_to_turn_on() {
    let s = Settings::default();
    assert!(!s.block_on && !s.mob_on, "outlines");
    assert!(!s.counter.on && !s.armor.on, "arrow counter and armor HUD");
    assert!(!s.cps.on && !s.fps.on, "CPS and FPS");
    assert!(
        !s.zoom.on && !s.jump_reset.on && !s.crosshair.on,
        "zoom, jump reset, crosshair"
    );
    assert!(!s.fullbright && !s.auto_queue, "fullbright and auto queue");
    assert!(
        !s.effect_timers && !s.hotbar_keys,
        "effect timers and hotbar keys"
    );
    assert!(
        !s.hide_hunger,
        "the game's own hunger icons stay as the game has them"
    );
    // The start page and the pages show it: every switch reads off.
    let shown = update(&s, 0.0, MenuState::default());
    for home in [
        "home_block",
        "home_mob",
        "home_counter",
        "home_armor",
        "home_cps",
        "home_fps",
        "home_zoom",
        "home_light",
        "home_jump",
        "home_crosshair",
    ] {
        assert!(!switch_on(&shown, home), "{home}");
    }
}
