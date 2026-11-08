use super::*;

#[test]
fn backspace_unbinds_and_escape_cancels_the_key_being_chosen() {
    let keys = |names: &[&str]| names.iter().map(|k| (*k).to_owned()).collect::<Vec<_>>();
    let z = KeyName::new("KeyZ");
    let menu = KeyName::new("KeyL").unwrap();
    // Nothing pressed yet, or only a mouse click: keep waiting.
    assert_eq!(captured_key(&keys(&[]), z, menu), None);
    assert_eq!(captured_key(&keys(&["MouseLeft"]), z, menu), None);
    // Backspace and Delete unbind, even when a click came first.
    assert_eq!(captured_key(&keys(&["Backspace"]), z, menu), Some(None));
    assert_eq!(
        captured_key(&keys(&["MouseLeft", "Delete"]), z, menu),
        Some(None)
    );
    // Escape cancels: the key stays as it was, bound or not.
    assert_eq!(captured_key(&keys(&["Escape"]), z, menu), Some(z));
    assert_eq!(
        captured_key(&keys(&["MouseLeft", "Escape"]), z, menu),
        Some(z)
    );
    assert_eq!(captured_key(&keys(&["Escape"]), None, menu), Some(None));
    // Unbinding an unbound key stays unbound.
    assert_eq!(captured_key(&keys(&["Delete"]), None, menu), Some(None));
    // Any other key becomes the key; the menu key is refused and nothing changes.
    assert_eq!(
        captured_key(&keys(&["KeyC"]), z, menu),
        Some(KeyName::new("KeyC"))
    );
    assert_eq!(captured_key(&keys(&["KeyL"]), z, menu), Some(z));
    assert_eq!(captured_key(&keys(&["KeyL"]), None, menu), Some(None));
}

#[test]
fn scroll_direction_follows_the_hotbar() {
    // Wheel up selects the previous slot: zoom in. Wheel down the next: zoom out.
    assert_eq!(scroll_steps(4, 3), 1);
    assert_eq!(scroll_steps(4, 5), -1);
    assert_eq!(scroll_steps(4, 4), 0);
    // Wrapping round the ends is still one notch.
    assert_eq!(scroll_steps(0, 8), 1);
    assert_eq!(scroll_steps(8, 0), -1);
    // Several notches between two frames.
    assert_eq!(scroll_steps(6, 3), 3);
    assert_eq!(scroll_steps(2, 5), -3);
    assert_eq!(scroll_steps(0, 9), 0, "not a hotbar slot");
}

#[test]
fn eases_in_and_out_and_stops_applying() {
    let zoom = Zoom::default();
    let mut engine = Engine::default();
    assert!(!engine.tick(&zoom, false, 0, 0.016).applying);
    let mut last = 1.0;
    for _ in 0..8 {
        let out = engine.tick(&zoom, true, 0, 0.016);
        assert!(out.applying && engine.divisor > last, "zooms in steadily");
        last = engine.divisor;
    }
    // After well over the zoom time it has arrived at the level and looks slower to match.
    for _ in 0..60 {
        engine.tick(&zoom, true, 0, 0.016);
    }
    let out = engine.tick(&zoom, true, 0, 0.016);
    assert!((engine.divisor - 4.0).abs() < 0.01, "{}", engine.divisor);
    assert!((out.fov_scale - 0.25).abs() < 0.01 && (out.look_scale - 0.25).abs() < 0.01);
    // Letting go eases back to exactly 1 and then stops sending anything.
    for _ in 0..120 {
        engine.tick(&zoom, false, 0, 0.016);
    }
    let out = engine.tick(&zoom, false, 0, 0.016);
    assert_eq!(engine.divisor, 1.0);
    assert!(!out.applying);
    assert_eq!((out.fov_scale, out.look_scale), (1.0, 1.0));
}

#[test]
fn steps_scale_the_level_and_reset_each_zoom() {
    let zoom = Zoom {
        time: 0.0,
        ..Zoom::default()
    };
    let mut engine = Engine::default();
    engine.tick(&zoom, true, 0, 0.016);
    assert!(
        (engine.divisor - 4.0).abs() < 1e-4,
        "instant with zero time"
    );
    // Two notches in at 20% each, one out.
    engine.tick(&zoom, true, 2, 0.016);
    engine.tick(&zoom, true, -1, 0.016);
    assert!(
        (engine.divisor - 4.0 * 1.2).abs() < 1e-4,
        "{}",
        engine.divisor
    );
    // The limits hold: nowhere near past 10x, nor below a sliver of zoom.
    engine.tick(&zoom, true, 8, 0.016);
    engine.tick(&zoom, true, 8, 0.016);
    assert!((engine.divisor - MAX_LEVEL).abs() < 1e-3);
    assert!((engine.tick(&zoom, true, 0, 0.016).fov_scale - 0.1).abs() < 1e-4);
    engine.tick(&zoom, true, -8, 0.016);
    engine.tick(&zoom, true, -8, 0.016);
    engine.tick(&zoom, true, -8, 0.016);
    assert!((engine.divisor - 1.1).abs() < 1e-4);
    // A new zoom starts again from the setting.
    engine.tick(&zoom, false, 0, 0.016);
    engine.tick(&zoom, true, 0, 0.016);
    assert!((engine.divisor - 4.0).abs() < 1e-4);
    // Scrolling does nothing while not zooming.
    let mut idle = Engine::default();
    assert!(!idle.tick(&zoom, false, 5, 0.016).applying);
}

#[test]
fn sensitivity_option_and_clamps() {
    let zoom = Zoom {
        time: 0.0,
        sensitivity: false,
        level: 8.0,
        ..Zoom::default()
    };
    let mut engine = Engine::default();
    let out = engine.tick(&zoom, true, 0, 0.016);
    assert!((out.fov_scale - 0.125).abs() < 1e-5 && out.look_scale == 1.0);
    let max = Zoom {
        time: 0.0,
        level: MAX_LEVEL,
        ..Zoom::default()
    };
    assert!((Engine::default().tick(&max, true, 0, 0.016).look_scale - 0.1).abs() < 1e-4);
    assert_eq!(clamp_level(99.0), MAX_LEVEL);
    assert_eq!(clamp_level(0.0), MIN_LEVEL);
    assert_eq!(clamp_level(f32::NAN), 4.0);
    assert_eq!(clamp_level(3.3), 3.5);
    assert_eq!(clamp_time(-1.0), 0.0);
    assert_eq!(clamp_time(7.0), MAX_TIME);
    assert_eq!(clamp_step(1.0), MIN_STEP);
    assert_eq!(clamp_step(99.0), MAX_STEP);
    assert_eq!(clamp_step(22.0), 20);
}
