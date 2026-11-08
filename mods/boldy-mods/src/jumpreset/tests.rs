use super::*;

fn run(events: &[(&str, f32)], window: u16) -> Vec<Timing> {
    // Each event happens after the given seconds since the one before.
    let mut tracker = Tracker::new();
    let mut results = Vec::new();
    for &(what, after) in events {
        tracker.step(after);
        let timing = match what {
            "press" => tracker.press(window),
            _ => tracker.hit(window),
        };
        results.extend(timing);
    }
    results
}

#[test]
fn a_jump_before_the_hit_is_early_and_after_it_is_late() {
    let early = run(&[("press", 0.0), ("hit", 0.2)], 75);
    assert_eq!(early.len(), 1);
    assert_eq!(
        (early[0].verdict, early[0].before, early[0].ms),
        (Verdict::Early, true, 200)
    );
    let late = run(&[("hit", 0.0), ("press", 0.12)], 75);
    assert_eq!(late.len(), 1);
    assert_eq!(
        (late[0].verdict, late[0].before, late[0].ms),
        (Verdict::Late, false, 120)
    );
}

#[test]
fn a_jump_inside_the_window_is_perfect_on_either_side() {
    for events in [
        [("press", 0.0), ("hit", 0.05)],
        [("hit", 0.0), ("press", 0.05)],
        [("press", 0.0), ("hit", 0.0)],
    ] {
        let got = run(&events, 75);
        assert_eq!(got.len(), 1, "{events:?}");
        assert_eq!(got[0].verdict, Verdict::Perfect, "{events:?}");
    }
    // The edge of the window is still perfect, one millisecond past it is not.
    assert_eq!(judge(-0.075, 75).verdict, Verdict::Perfect);
    assert_eq!(judge(0.075, 75).verdict, Verdict::Perfect);
    assert_eq!(judge(0.076, 75).verdict, Verdict::Late);
    assert_eq!(judge(-0.076, 75).verdict, Verdict::Early);
    // A wider window forgives more.
    assert_eq!(judge(0.15, 150).verdict, Verdict::Perfect);
}

#[test]
fn a_jump_and_a_hit_far_apart_say_nothing() {
    // A jump long before a hit is an ordinary jump.
    assert!(run(&[("press", 0.0), ("hit", 0.41)], 75).is_empty());
    // A hit long before a jump is an ordinary jump too.
    assert!(run(&[("hit", 0.0), ("press", 0.41)], 75).is_empty());
    // Just inside is still about that hit.
    assert_eq!(run(&[("press", 0.0), ("hit", 0.4)], 75).len(), 1);
    // Time in small steps adds up the same.
    let mut tracker = Tracker::new();
    tracker.press(75);
    for _ in 0..10 {
        tracker.step(0.01);
    }
    assert_eq!(tracker.hit(75).map(|t| t.ms), Some(100));
}

#[test]
fn the_latest_jump_before_a_hit_is_the_one_judged() {
    // Two jumps, then the hit: the second, nearer one counts.
    let got = run(&[("press", 0.0), ("press", 0.25), ("hit", 0.05)], 75);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].ms, 50);
    assert_eq!(got[0].verdict, Verdict::Perfect);
}

#[test]
fn one_hit_is_judged_once() {
    // The first jump after a hit is judged; a second jump has no hit left to be late for.
    let got = run(&[("hit", 0.0), ("press", 0.1), ("press", 0.05)], 75);
    assert_eq!(got.len(), 1);
    // And a jump is not judged twice against two hits.
    let got = run(&[("press", 0.0), ("hit", 0.1), ("hit", 0.05)], 75);
    assert_eq!(got.len(), 1);
}

#[test]
fn bad_time_cannot_break_the_tracker() {
    let mut tracker = Tracker::new();
    tracker.press(75);
    tracker.step(f32::NAN);
    tracker.step(-3.0);
    tracker.step(f32::INFINITY);
    // Infinite time is not a time: nothing aged, so the jump still pairs with the hit.
    assert_eq!(tracker.hit(75).map(|t| t.ms), Some(0));
    assert_eq!(judge(f32::NAN, 75).ms, 999);
    assert_eq!(judge(100.0, 75).ms, 999);
}

#[test]
fn the_window_steps_and_stays_in_bounds() {
    assert_eq!(JumpReset::stepped(75, 1), 80);
    assert_eq!(JumpReset::stepped(75, -1), 70);
    assert_eq!(JumpReset::stepped(MAX_WINDOW, 1), MAX_WINDOW);
    assert_eq!(JumpReset::stepped(MIN_WINDOW, -1), MIN_WINDOW);
    // A value from a hand-edited file moves onto a step.
    assert_eq!(JumpReset::stepped(77, 1), 80);
    assert_eq!(JumpReset::stepped(0, 0), MIN_WINDOW);
    assert_eq!(clamp_window(5000), MAX_WINDOW);
}

#[test]
fn the_feed_skips_history_and_counts_new_hits() {
    let mut feed = Feed::new();
    // History: two full reads of other events, then a short one with an old hit of mine.
    assert_eq!(
        feed.take(&[(1, false), (2, true), (3, false), (4, false)], 4),
        0
    );
    assert_eq!(feed.after, 4);
    assert!(!feed.live);
    assert_eq!(feed.take(&[(5, false), (6, true)], 4), 0);
    assert_eq!(feed.after, 6);
    // Now at the end: a new hit counts, someone else's does not.
    assert_eq!(feed.take(&[(7, true), (8, false)], 4), 1);
    assert_eq!(feed.take(&[], 4), 0);
    assert_eq!(feed.after, 8);
    // A burst larger than a read is followed up, and every hit in it counts.
    assert_eq!(
        feed.take(&[(9, true), (10, true), (11, true), (12, false)], 4),
        3
    );
    assert_eq!(feed.take(&[(13, true)], 4), 1);
}

#[test]
fn an_empty_history_is_live_straight_away() {
    let mut feed = Feed::new();
    assert_eq!(feed.take(&[], 4), 0);
    assert_eq!(feed.take(&[(1, true)], 4), 1);
}

#[test]
fn the_text_says_what_happened() {
    let say = |timing: Timing| {
        let mut out = String::new();
        timing_text(&mut out, timing);
        out
    };
    assert_eq!(say(judge(-0.085, 50)), "Too early: 85 ms");
    assert_eq!(say(judge(0.12, 50)), "Too late: 120 ms");
    assert_eq!(say(judge(-0.012, 75)), "Perfect (12 ms early)");
    assert_eq!(say(judge(0.03, 75)), "Perfect (30 ms late)");
    assert_eq!(say(judge(0.0, 75)), "Perfect");
    assert_eq!(say(judge(-5.0, 75)), "Too early: 999 ms");
}

#[test]
fn the_colour_tells_close_from_far() {
    assert_eq!(timing_colour(judge(0.01, 75))[1], 1.0);
    let close = timing_colour(judge(0.12, 75));
    let far = timing_colour(judge(0.3, 75));
    assert!(close[0] == 1.0 && close[1] > far[1]);
    assert_ne!(close, far);
    assert!(judge(0.15, 75).near && !judge(0.151, 75).near);
}

#[test]
fn the_patch_carries_the_text_and_colour() {
    let view = View {
        timing: Some(judge(-0.085, 50)),
        outcome: None,
    };
    let patch = patch(&view);
    assert!(patch.starts_with(r##"{"binding_changes":{"#jumpreset_visible":true,"##));
    assert!(
        patch.contains(r##""#jumpreset_text":"Too early: 85 ms""##),
        "{patch}"
    );
    assert!(patch.contains(r##""#jumpreset_color":[1,"##), "{patch}");
    assert!(patch.ends_with("}}"));
    let hidden = initial_bindings();
    assert!(hidden.contains(r##""#jumpreset_visible":false"##));
    assert!(hidden.contains(r##""#jumpreset_text":"""##));
    let json = format!("{{{hidden}}}");
    assert!(!json.contains(",}"), "no trailing comma: {json}");
}

#[test]
fn the_document_is_balanced_json() {
    let doc = document!();
    for (open, close) in [('{', '}'), ('[', ']')] {
        assert_eq!(doc.matches(open).count(), doc.matches(close).count());
    }
    assert!(doc.contains("#jumpreset_color"));
}

/// A velocity trace: each frame lasts `dt` seconds and has this (x, y, z) velocity.
fn watched(frames: &[[f32; 3]], dt: f32, hit_at: usize, threshold: f32) -> Option<Outcome> {
    let mut watch = Watch::new();
    let mut result = None;
    let mut step = |watch: &mut Watch, frame: Option<[f32; 3]>| {
        watch.record(frame, None);
        result = result.or(watch.sample(dt, frame, threshold));
    };
    for (index, frame) in frames.iter().enumerate() {
        if index == hit_at {
            watch.hit();
        }
        step(&mut watch, Some(*frame));
    }
    // Let the watching time run out with the last velocity held.
    for _ in 0..8 {
        step(&mut watch, frames.last().copied());
    }
    result
}

fn still(count: usize, v: [f32; 3]) -> Vec<[f32; 3]> {
    vec![v; count]
}

#[test]
fn a_shove_that_carries_on_is_not_a_reset_and_one_that_dies_is() {
    let dt = 0.05;
    let hit = |after: &[[f32; 3]], threshold: f32| {
        let mut frames = still(4, [0.0; 3]);
        frames.extend_from_slice(after);
        frames.extend(still(6, *after.last().unwrap()));
        watched(&frames, dt, 4, threshold).expect("watched to the end")
    };
    // The shove carries on: 0.4, then 0.4 again.
    let out = hit(&[[0.4, 0.4, 0.0], [0.4, 0.3, 0.0]], DEFAULT_KNOCKBACK);
    assert!(!out.reset && (out.knockback - 0.4).abs() < 1e-3, "{out:?}");
    // Ground friction takes a normal hit down to about 0.15 a tick later: not a reset.
    let out = hit(&[[0.4, 0.4, 0.0], [0.15, 0.3, 0.0]], DEFAULT_KNOCKBACK);
    assert!(!out.reset && (out.knockback - 0.15).abs() < 1e-3, "{out:?}");
    // A jump that ate it leaves hardly anything.
    let out = hit(&[[0.4, 0.4, 0.0], [0.04, 0.3, 0.0]], DEFAULT_KNOCKBACK);
    assert!(out.reset && (out.knockback - 0.04).abs() < 1e-3, "{out:?}");
    // The shove can come diagonally and a tick late: what counts is the tick after it.
    let out = hit(
        &[[0.1, 0.0, 0.0], [0.28, 0.4, 0.28], [0.03, 0.3, 0.03]],
        DEFAULT_KNOCKBACK,
    );
    assert!(
        out.reset && (out.knockback - 0.0424).abs() < 1e-3,
        "{out:?}"
    );
    // The threshold decides: 0.1 left is a reset for a loose one, not a strict one.
    let shove = [[0.4, 0.4, 0.0], [0.1, 0.3, 0.0]];
    assert!(!hit(&shove, 0.05).reset);
    assert!(hit(&shove, 0.2).reset);
}

#[test]
fn a_hurt_without_a_shove_is_no_hit_and_says_nothing() {
    let dt = 0.05;
    // Running on as before, or burning: no shove anywhere, so no verdict (a Reset! would be
    // a lie) and the watch says it dropped it.
    for v in [[0.0, 0.0, -0.13], [0.0, 0.0, 0.0], [0.0, 0.42, 0.0]] {
        let mut watch = Watch::new();
        watch.sample(dt, Some(v), DEFAULT_KNOCKBACK);
        watch.hit();
        let mut out = None;
        for _ in 0..12 {
            watch.record(Some(v), None);
            out = out.or(watch.sample(dt, Some(v), DEFAULT_KNOCKBACK));
        }
        assert_eq!(out, None, "{v:?}");
        assert!(!watch.shoved());
        assert!(watch.take_dropped());
        assert!(!watch.take_dropped(), "reading it clears it");
    }
    // A shove that only shows in the last slot cannot be followed: the full shove counts.
    let mut watch = Watch::new();
    watch.hit();
    for index in 0..SLOTS {
        let v = if index == SLOTS - 1 {
            [0.4, 0.4, 0.0]
        } else {
            [0.0; 3]
        };
        watch.record(Some(v), None);
        watch.sample(SLOT_SECONDS, Some(v), DEFAULT_KNOCKBACK);
    }
    assert!(watch.shoved());
    assert!((watch.carried() - 0.4).abs() < 1e-3);
    assert!(!watch.take_dropped());
}

#[test]
fn the_shove_in_the_hit_frame_itself_counts() {
    let dt = 0.05;
    // The shove arrives in the very frame of the hit, and is gone a tick later.
    let mut now = still(3, [0.0; 3]);
    now.push([0.0, 0.4, 0.4]);
    now.extend(still(8, [0.0, 0.3, 0.02]));
    assert!(watched(&now, dt, 3, DEFAULT_KNOCKBACK).unwrap().reset);
    // The same frames with the hit a tick later: the shove came before it and is no part of
    // it, so this is a hurt without a shove and says nothing.
    assert_eq!(watched(&now, dt, 4, DEFAULT_KNOCKBACK), None);
}

#[test]
fn the_watch_waits_out_its_time_and_needs_a_measurement() {
    let mut watch = Watch::new();
    let frame = |watch: &mut Watch, seconds: f32, v: Option<[f32; 3]>| {
        watch.record(v, None);
        watch.sample(seconds, v, 0.08)
    };
    let shove = Some([0.4, 0.4, 0.0]);
    let calm = Some([0.02, 0.0, 0.0]);
    // Nothing to watch: no outcome, however long.
    assert_eq!(frame(&mut watch, 1.0, calm), None);
    watch.hit();
    assert!(watch.watching());
    assert_eq!(frame(&mut watch, 0.1, shove), None);
    assert_eq!(frame(&mut watch, 0.1, calm), None);
    assert!(frame(&mut watch, 0.25, calm).is_some());
    assert!(!watch.watching());
    assert_eq!(frame(&mut watch, 0.5, calm), None);
    // Never being able to read the velocity gives no outcome rather than a false reset.
    watch.hit();
    assert_eq!(frame(&mut watch, 0.5, None), None);
    assert!(watch.take_dropped());
    // Nonsense velocities are skipped, not recorded.
    watch.hit();
    assert_eq!(frame(&mut watch, 0.1, Some([f32::NAN, 0.0, 0.0])), None);
    assert_eq!(frame(&mut watch, 0.5, calm), None);
    assert!(watch.take_dropped());
    // A real shove then ends in an outcome, and nothing was dropped.
    watch.hit();
    assert_eq!(frame(&mut watch, 0.1, shove), None);
    let out = frame(&mut watch, 0.5, calm).unwrap();
    assert!(out.reset && out.knockback < 0.05, "{out:?}");
    assert!(!watch.take_dropped());
}

#[test]
fn a_second_hit_starts_the_watch_again() {
    let mut watch = Watch::new();
    let frame = |watch: &mut Watch, seconds: f32, v: [f32; 3]| {
        watch.record(Some(v), None);
        watch.sample(seconds, Some(v), 0.08)
    };
    frame(&mut watch, 0.05, [0.0; 3]);
    watch.hit();
    frame(&mut watch, 0.05, [0.4, 0.4, 0.0]);
    frame(&mut watch, 0.05, [0.4, 0.3, 0.0]);
    // Hit again before the first was finished: only the new one is judged.
    watch.hit();
    assert_eq!(frame(&mut watch, 0.05, [0.4, 0.4, 0.0]), None);
    assert_eq!(frame(&mut watch, 0.05, [0.02, 0.3, 0.0]), None);
    let out = frame(&mut watch, 0.4, [0.02, 0.3, 0.0]).unwrap();
    assert!(out.reset, "the second shove died at once: {out:?}");
}

#[test]
fn the_readout_joins_timing_and_knockback() {
    let say = |view: View| {
        let mut out = String::new();
        text(&mut out, &view);
        out
    };
    let timing = Some(judge(-0.085, 50));
    let reset = Some(Outcome {
        knockback: 0.04,
        reset: true,
        airborne: false,
        small: false,
    });
    let pushed = Some(Outcome {
        knockback: 0.4,
        reset: false,
        airborne: false,
        small: false,
    });
    assert_eq!(
        say(View {
            timing,
            outcome: pushed
        }),
        "Too early: 85 ms | Knock left 0.40"
    );
    assert_eq!(
        say(View {
            timing: Some(judge(0.01, 75)),
            outcome: reset
        }),
        "Perfect (10 ms late) | Reset! knock left 0.04"
    );
    // Either part alone, and a knockback of 1.5 or more digits long still reads.
    assert_eq!(
        say(View {
            timing: None,
            outcome: reset
        }),
        "Reset! knock left 0.04"
    );
    assert_eq!(
        say(View {
            timing,
            outcome: None
        }),
        "Too early: 85 ms"
    );
    let big = Some(Outcome {
        knockback: 12.345,
        reset: false,
        airborne: false,
        small: false,
    });
    assert_eq!(
        say(View {
            timing: None,
            outcome: big
        }),
        "Knock left 12.35"
    );
    assert_eq!(say(View::HIDDEN), "");
    assert!(View::HIDDEN.is_hidden());
    assert!(!SAMPLE.is_hidden());
}

#[test]
fn a_small_knock_is_a_reset_only_with_a_jump_near_the_hit() {
    let small = Outcome {
        knockback: 0.01,
        reset: true,
        airborne: false,
        small: true,
    };
    let big = Outcome {
        knockback: 0.3,
        reset: false,
        airborne: false,
        small: false,
    };
    let near = Some(judge(-0.05, 75));
    let edge = Some(judge(0.15, 75));
    let far = Some(judge(-0.232, 75));
    // Jump near the hit and the shove gone: a reset, early or late alike.
    assert!(small.judged(near).reset);
    assert!(small.judged(Some(judge(0.1, 75))).reset);
    assert!(small.judged(edge).reset, "twice the window is still near");
    // A jump too far off, or none at all (a wall or corner killed it): no reset.
    assert!(!small.judged(far).reset);
    assert!(!small.judged(None).reset);
    assert!(small.judged(None).small, "but it is still a small knock");
    // A shove that carried on is never a reset, whatever the jump.
    assert!(!big.judged(near).reset);
    // A wider window widens what is near.
    assert!(small.judged(Some(judge(0.2, 150))).reset);
}

#[test]
fn the_readout_says_why_a_small_knock_is_not_a_reset() {
    let say = |view: View| {
        let mut out = String::new();
        text(&mut out, &view);
        out
    };
    let small = Outcome {
        knockback: 0.009,
        reset: false,
        airborne: false,
        small: true,
    };
    assert_eq!(
        say(View {
            timing: None,
            outcome: Some(small)
        }),
        "Knock left 0.01 (no jump)"
    );
    assert_eq!(
        say(View {
            timing: Some(judge(-0.232, 75)),
            outcome: Some(small)
        }),
        "Too early: 232 ms | Knock left 0.01 (jump off)"
    );
    let reset = Outcome {
        reset: true,
        ..small
    };
    assert_eq!(
        say(View {
            timing: Some(judge(0.0, 75)),
            outcome: Some(reset)
        }),
        "Perfect | Reset! knock left 0.01"
    );
    // In the air outranks the rest.
    let air = Outcome {
        airborne: true,
        ..small
    };
    assert_eq!(
        say(View {
            timing: None,
            outcome: Some(air)
        }),
        "Knock left 0.01 (in air)"
    );
}

#[test]
fn a_hit_taken_in_the_air_says_so() {
    let mut watch = Watch::new();
    watch.ground(Some(false));
    watch.sample(0.05, Some([0.0, -0.3, 0.0]), 0.08);
    watch.hit();
    let mut out = None;
    for frame in 0..12 {
        // The shove, then it carries on at 0.34 a tick, as it does in the air.
        let v = if frame == 0 {
            [0.4, 0.4, 0.0]
        } else {
            [0.34, 0.3, 0.0]
        };
        watch.record(Some(v), Some(false));
        out = out.or(watch.sample(0.05, Some(v), 0.08));
        watch.ground(Some(false));
    }
    let out = out.unwrap();
    assert!(out.airborne && !out.reset, "{out:?}");
    let mut text = String::new();
    super::text(
        &mut text,
        &View {
            timing: None,
            outcome: Some(out),
        },
    );
    assert_eq!(text, "Knock left 0.34 (in air)");
    // On the ground it does not.
    let mut grounded = Watch::new();
    grounded.ground(Some(true));
    grounded.sample(0.05, Some([0.0, -0.078, 0.0]), 0.08);
    grounded.hit();
    let mut out = None;
    for frame in 0..12 {
        let v = if frame == 0 {
            [0.4, 0.4, 0.0]
        } else {
            [0.15, 0.3, 0.0]
        };
        grounded.record(Some(v), Some(true));
        out = out.or(grounded.sample(0.05, Some(v), 0.08));
    }
    assert!(!out.unwrap().airborne);
}

#[test]
fn the_colour_follows_the_outcome_first() {
    let green = [0.4, 1.0, 0.5, 1.0];
    let reset = Outcome {
        knockback: 0.0,
        reset: true,
        airborne: false,
        small: false,
    };
    let pushed = Outcome {
        knockback: 0.4,
        reset: false,
        airborne: false,
        small: false,
    };
    // A working reset is green even when the jump was well off.
    let late = Some(judge(0.3, 75));
    assert_eq!(
        colour(&View {
            timing: late,
            outcome: Some(reset)
        }),
        green
    );
    // Pushed after a late jump: the timing's colour; after a perfect one or none: orange.
    assert_eq!(
        colour(&View {
            timing: late,
            outcome: Some(pushed)
        }),
        timing_colour(judge(0.3, 75))
    );
    let orange = [1.0, 0.6, 0.3, 1.0];
    assert_eq!(
        colour(&View {
            timing: Some(judge(0.0, 75)),
            outcome: Some(pushed)
        }),
        orange
    );
    assert_eq!(
        colour(&View {
            timing: None,
            outcome: Some(pushed)
        }),
        orange
    );
    // Timing alone, before the knockback is known.
    assert_eq!(
        colour(&View {
            timing: late,
            outcome: None
        }),
        timing_colour(judge(0.3, 75))
    );
}

#[test]
fn the_watch_remembers_how_things_stood_before_the_hit() {
    let mut watch = Watch::new();
    watch.ground(Some(true));
    watch.sample(0.016, Some([0.3, 0.0, 0.4]), 0.25);
    watch.hit();
    // The frame of the hit moves the velocity, but the trace is from before.
    watch.ground(Some(false));
    watch.sample(0.016, Some([0.0, 0.4, 0.0]), 0.25);
    let trace = watch.trace();
    assert_eq!(trace.grounded, Some(true));
    assert!((trace.speed - 0.5).abs() < 1e-6, "{trace:?}");
    // With nothing known before, it says so.
    let mut fresh = Watch::new();
    fresh.hit();
    assert_eq!(fresh.trace(), Trace::default());
}

#[test]
fn the_log_keeps_the_last_hits_and_reads_back() {
    let timing = Some(judge(-0.085, 50));
    let outcome = Outcome {
        knockback: 0.4123,
        reset: false,
        airborne: false,
        small: false,
    };
    let trace = Trace {
        grounded: Some(true),
        speed: 0.13,
    };
    let row = Row::new(timing, outcome, trace);
    assert_eq!(
        row,
        Row {
            offset_ms: Some(-85),
            knockback: 412,
            grounded: Some(true),
            speed: 130,
        }
    );
    let late = Row::new(Some(judge(0.12, 50)), outcome, Trace::default());
    assert_eq!(late.offset_ms, Some(120));
    assert_eq!(Row::new(None, outcome, Trace::default()).offset_ms, None);
    let mut log = Log::new();
    assert_eq!(log.to_text(), "");
    log.push(row);
    log.push(Row::new(None, outcome, Trace::default()));
    assert_eq!(log.to_text(), "-85,412,1,130;x,412,x,0");
    assert_eq!(Log::parse(&log.to_text()), log);
    // Full: the oldest goes first.
    for step in 0..30u16 {
        log.push(Row {
            knockback: step,
            ..Row::default()
        });
    }
    assert_eq!(log.rows().len(), LOG_LEN);
    assert_eq!(log.rows()[0].knockback, 14);
    assert_eq!(log.rows()[LOG_LEN - 1].knockback, 29);
    assert_eq!(Log::parse(&log.to_text()), log);
    // Bad text is skipped, not trusted.
    let odd = Log::parse("junk;1,2;x,5,1,7;1,2,3,4,5;-9,notanumber,0,1;;2000000,3,0,4");
    assert_eq!(odd.rows().len(), 2);
    assert_eq!(odd.rows()[0].knockback, 5);
    assert_eq!(odd.rows()[1].offset_ms, None);
    assert_eq!(Log::parse("").rows().len(), 0);
}

#[test]
fn the_fast_log_reader_agrees_with_the_general_one() {
    for text in [
        "",
        "x,0,x,0",
        "210,346,0,124;x,28,1,92;-222,362,0,188;x,0,0,0;",
        "-32768,65535,1,65535;32767,0,0,0",
        "junk;1,2;x,5,1,7",
        "40000,3,0,4;-40000,3,0,4",
        "1,2,3,4",
        "1,-2,0,4",
        "1,x,0,4",
        "1,2,0,70000",
        "1,2,0,4;;5,6,1,7",
        "-,2,0,4",
        "+5,2,0,4",
        "5,2,1",
        "5,2,1,",
        "99999999999,2,0,4",
    ] {
        if let Some(fast) = Log::parse_fast(text) {
            assert_eq!(fast, Log::parse_general(text), "{text}");
        }
        assert_eq!(Log::parse(text), Log::parse_general(text), "{text}");
    }
    assert!(Log::parse_fast("210,346,0,124;x,28,1,92").is_some());
}

#[test]
fn a_hit_records_the_velocity_before_and_every_50_ms_after() {
    let mut watch = Watch::new();
    watch.ground(Some(true));
    watch.sample(0.016, Some([0.1, 0.0, -0.2]), 0.25);
    watch.hit();
    // 60 frames of 10 ms: the first is the hit's own frame, then one slot per 50 ms.
    for frame in 0..60 {
        let t = frame as f32 * 0.01;
        let v = Some([t, 0.5, -0.25]);
        let ground = Some(frame < 20);
        watch.record(v, ground);
        watch.sample(0.01, v, 0.25);
        watch.ground(ground);
    }
    let text = watch.recording();
    let (before, slots) = text.split_once('|').unwrap();
    assert_eq!(before, "100,0,-200");
    let slots: Vec<&str> = slots.split(';').collect();
    assert_eq!(slots.len(), SLOTS, "{text}");
    assert!(slots[0].starts_with("0,500,-250,1"), "{text}");
    assert!(slots[7].ends_with(",0"), "{text}: in the air by 350 ms");
    // A new hit starts from nothing, and unreadable velocities fill nothing.
    watch.hit();
    watch.record(None, None);
    assert!(watch.recording().ends_with('|'));
    assert_eq!(milli(f32::NAN), 0);
    assert_eq!(milli(1000.0), i16::MAX);
}

#[test]
fn a_hurt_just_after_a_fast_fall_is_fall_damage() {
    let mut watch = Watch::new();
    assert!(!watch.falling());
    // Falling at 0.7 a tick, then landing: the landing's hurt comes a tick or two later.
    watch.sample(0.05, Some([0.0, -0.7, 0.0]), 0.35);
    assert!(watch.falling());
    watch.sample(0.05, Some([0.0, -0.078, 0.0]), 0.35);
    assert!(watch.falling(), "just landed");
    for _ in 0..8 {
        watch.sample(0.05, Some([0.0, -0.078, 0.0]), 0.35);
    }
    assert!(!watch.falling(), "0.4 s later it is an ordinary hit again");
    // An ordinary jump's landing speed is not a fall that hurts, nor is an unreadable or
    // nonsense velocity.
    watch.sample(0.05, Some([0.0, -0.5, 0.0]), 0.35);
    watch.sample(0.05, Some([0.0, f32::NAN, 0.0]), 0.35);
    watch.sample(0.05, None, 0.35);
    assert!(!watch.falling());
    // A falling start from a knockback's own climb and drop is only fast later: fall speed
    // exactly at the limit counts.
    watch.sample(0.05, Some([0.0, -FALL_SPEED, 0.0]), 0.35);
    assert!(watch.falling());
}

#[test]
fn resets_are_kept_up_to_a_cap_and_fit_the_file() {
    let mut rows = Vec::new();
    for n in 0..30 {
        push_capped(&mut rows, format!("row{n}"), RESETS_KEPT);
    }
    assert_eq!(rows.len(), RESETS_KEPT);
    assert_eq!(rows[0], "row6");
    assert_eq!(rows.last().unwrap(), "row29");
    // Everything fits: all of it, joined.
    assert_eq!(joined_within(&rows[22..], 100), "row28/row29");
    // Short of room the oldest go first, never the newest.
    assert_eq!(joined_within(&rows[22..], 8), "row29");
    assert_eq!(joined_within(&rows[22..], 5), "row29");
    // No room for even one: nothing, not half a row.
    assert_eq!(joined_within(&rows, 3), "");
    assert_eq!(joined_within(&[], 10), "");
    // 24 full recordings of about 250 bytes are well inside what the file allows.
    const { assert!(RESETS_KEPT * 300 < 8 * 1024) };
}

#[test]
fn the_knockback_threshold_stays_in_bounds() {
    assert_eq!(clamp_knockback(f32::NAN), DEFAULT_KNOCKBACK);
    assert_eq!(clamp_knockback(0.0), MIN_KNOCKBACK);
    assert_eq!(clamp_knockback(50.0), MAX_KNOCKBACK);
    assert_eq!(clamp_knockback(0.3), 0.3);
}
