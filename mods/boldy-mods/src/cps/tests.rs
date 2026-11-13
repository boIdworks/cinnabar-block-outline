use super::*;

#[test]
fn counts_the_clicks_of_the_last_second() {
    let mut clicks = Clicks::new();
    for time in [0.0, 0.2, 0.4, 0.6, 0.8] {
        clicks.click(time);
    }
    assert_eq!(clicks.per_second(0.9), 5);
    // The first click is a full second old at 1.0 and still counts; at 1.05 it is gone.
    assert_eq!(clicks.per_second(1.0), 5);
    assert_eq!(clicks.per_second(1.05), 4);
    assert_eq!(clicks.per_second(1.45), 2);
    assert_eq!(clicks.per_second(5.0), 0);
    // A click after a long pause starts from one.
    clicks.click(9.0);
    assert_eq!(clicks.per_second(9.0), 1);
}

fn narrow() -> Widths {
    Widths {
        caption: 10.0,
        fps_caption: 10.0,
        digit: 2.0,
        space: 1.0,
        bar: 1.0,
    }
}

#[test]
fn the_probe_starts_on_the_guess_and_measures_at_once() {
    let probe = WidthProbe::new();
    assert_eq!(probe.widths(), Widths::GUESS);
    assert!(probe.due(0.0));
}

#[test]
fn a_failed_measurement_keeps_the_guess_and_is_retried_not_cached() {
    let mut probe = WidthProbe::new();
    assert!(!probe.observe(0.0, None), "nothing changed");
    assert_eq!(probe.widths(), Widths::GUESS);
    assert!(!probe.due(0.5), "not again within the recheck time");
    assert!(probe.due(RECHECK_SECONDS), "but it does try again");
}

#[test]
fn a_measurement_is_confirmed_by_a_second_before_the_probe_goes_quiet() {
    let mut probe = WidthProbe::new();
    // The first measurement replaces the guess, which the caller must lay out again for.
    assert!(probe.observe(0.0, Some(narrow())));
    assert_eq!(probe.widths(), narrow());
    assert!(probe.due(1.0), "one reading is not trusted yet");
    // The same again: nothing changes and it settles, so it stops measuring.
    assert!(!probe.observe(1.0, Some(narrow())));
    assert!(!probe.due(100.0));
}

#[test]
fn an_early_wrong_measurement_is_corrected_by_a_later_one() {
    let mut probe = WidthProbe::new();
    assert!(probe.observe(0.0, Some(narrow())));
    // The host's fonts became ready and the same text is wider now.
    assert!(probe.observe(1.0, Some(Widths::GUESS)));
    assert_eq!(probe.widths(), Widths::GUESS);
    assert!(probe.due(2.0), "the new reading has to be confirmed too");
    assert!(!probe.observe(2.0, Some(Widths::GUESS)));
    assert!(!probe.due(3.0));
}

#[test]
fn a_failure_between_two_equal_readings_does_not_settle_them() {
    let mut probe = WidthProbe::new();
    probe.observe(0.0, Some(narrow()));
    probe.observe(1.0, None);
    probe.observe(2.0, Some(narrow()));
    assert!(probe.due(3.0), "the readings were not consecutive");
}

#[test]
fn reset_measures_again_but_keeps_the_widths_meanwhile() {
    let mut probe = WidthProbe::new();
    probe.observe(0.0, Some(narrow()));
    probe.observe(1.0, Some(narrow()));
    assert!(!probe.due(50.0));
    probe.reset();
    assert!(probe.due(50.0));
    assert_eq!(probe.widths(), narrow(), "no flash back to the guess");
}

#[test]
fn a_settled_probe_stays_quiet_whatever_the_clock_says() {
    let mut probe = WidthProbe::new();
    probe.observe(0.0, Some(narrow()));
    probe.observe(1.0, Some(narrow()));
    assert!(!probe.due(f32::NAN));
}

#[test]
fn a_flood_of_clicks_is_capped_not_overrun() {
    let mut clicks = Clicks::new();
    for step in 0..500 {
        clicks.click(step as f32 * 0.001);
    }
    assert_eq!(clicks.per_second(0.5), CAP as u32);
    // Time standing still or going backwards cannot panic or grow it.
    clicks.click(0.0);
    assert_eq!(clicks.per_second(0.0), CAP as u32);
    assert!(clicks.per_second(f32::NAN) <= CAP as u32);
}

#[test]
fn the_text_shows_one_or_both_buttons() {
    let mut out = String::new();
    left_text(&mut out, 12, true);
    assert_eq!(out, "12");
    out.clear();
    left_text(&mut out, 7, true);
    assert_eq!(out, "07");
    out.clear();
    right_text(&mut out, 3, true);
    assert_eq!(out, "| 03");
    out.clear();
    right_text(&mut out, 0, true);
    assert_eq!(out, "| 00");
    out.clear();
    left_text(&mut out, 100, true);
    assert_eq!(out, "100");
    // With the zeros off the numbers are plain, and the layout keeps the same slots.
    out.clear();
    left_text(&mut out, 7, false);
    assert_eq!(out, "7");
    out.clear();
    right_text(&mut out, 0, false);
    assert_eq!(out, "| 0");
}

#[test]
fn the_caption_bar_and_right_number_stay_put_as_the_left_number_grows() {
    let widths = Widths::GUESS;
    let at = |left, right| widths.layout(left, Some(right));
    // From 0 to 99 clicks nothing about the layout changes, whatever the right number.
    let kept = at(0, 0);
    for (left, right) in [(5, 0), (9, 9), (10, 0), (13, 7), (99, 99)] {
        assert_eq!(at(left, right), kept, "{left} | {right}");
    }
    // `CPS:` and a space, then a slot of two digits, then the bar and the right number.
    assert_eq!(kept.num_x, 24.0 + 4.0);
    assert_eq!(kept.num_w, 12.0);
    assert_eq!(kept.right_x, kept.num_x + 12.0 + 4.0);
    assert_eq!(kept.total, kept.right_x + 2.0 + 4.0 + 12.0);
    // Only a third digit makes room.
    assert!(at(100, 0).right_x > kept.right_x);
    // Without the right button there is only the caption and the number.
    let alone = widths.layout(7, None);
    assert_eq!(
        (alone.right_x, alone.right_w, alone.total),
        (0.0, 0.0, alone.num_x + alone.num_w)
    );
    assert_eq!(size(2.0, 36.0), [76.0, 20.0]);
}

#[test]
fn the_default_sits_in_the_top_left_at_any_window_size() {
    let size = size(1.0, 40.0);
    for viewport in [[480.0, 270.0], [960.0, 540.0], [427.0, 240.0]] {
        assert_eq!(
            crate::counter::place(DEFAULT_POS, size, viewport),
            [4.0, 4.0]
        );
    }
    // Placed in the bottom right, it follows that corner instead.
    let viewport = [480.0, 270.0];
    let corner = [viewport[0] - size[0] - 4.0, viewport[1] - size[1] - 4.0];
    let pos = crate::counter::centre(corner, size, viewport);
    let bigger = [960.0, 540.0];
    assert_eq!(
        crate::counter::place(pos, size, bigger),
        [bigger[0] - size[0] - 4.0, bigger[1] - size[1] - 4.0]
    );
}

#[test]
fn sizes_step_and_stay_in_bounds() {
    assert_eq!(Cps::stepped(1.0, 1.0), 1.1);
    assert_eq!(Cps::stepped(MAX_SCALE, 1.0), MAX_SCALE);
    assert_eq!(Cps::stepped(MIN_SCALE, -1.0), MIN_SCALE);
    assert_eq!(clamp_scale(f32::NAN), 1.0);
}

#[test]
fn the_patch_carries_the_text_and_place() {
    let view = View {
        visible: true,
        left: 9,
        right: Some(4),
        zeros: true,
        scale: 1.5,
        layout: Widths::GUESS.layout(9, Some(4)),
        offset: [4.0, 4.0],
        edit: None,
    };
    let patch = patch(&view);
    assert!(patch.starts_with(r##"{"binding_changes":{"#cps_visible":true,"##));
    assert!(patch.contains(r##""#cps_text":"09","##));
    assert!(patch.contains(r##""#cps_right_text":"| 04","##));
    assert!(patch.contains(r##""#cps_right_visible":true"##));
    assert!(
        patch.contains(r##""#cps_offset":[4,4]"##) || patch.contains(r##""#cps_offset":[4.0"##)
    );
    assert!(patch.ends_with("}}"));
    let hidden = initial_bindings();
    assert!(hidden.contains(r##""#cps_visible":false"##));
    let json = format!("{{{hidden}}}");
    assert!(!json.contains(",}"), "no trailing comma: {json}");
}
