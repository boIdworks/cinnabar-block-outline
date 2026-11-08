use super::*;

#[test]
fn fps_is_frames_over_the_time_they_took() {
    let mut meter = Meter::new();
    assert_eq!(meter.fps(), 0);
    // 60 frames a second: known after half a second.
    for _ in 0..30 {
        meter.tick(1.0 / 60.0);
    }
    assert_eq!(meter.fps(), 60);
    // Frames of unknown length change nothing.
    meter.tick(0.0);
    meter.tick(f32::NAN);
    for _ in 0..72 {
        meter.tick(1.0 / 144.0);
    }
    assert_eq!(meter.fps(), 144);
    // A long stall shows as a low number.
    meter.tick(2.0);
    assert_eq!(meter.fps(), 1);
}

#[test]
fn the_default_sits_below_the_cps_readout() {
    let viewport = [480.0, 270.0];
    let cps = crate::counter::place(
        crate::cps::DEFAULT_POS,
        crate::cps::size(1.0, 40.0),
        viewport,
    );
    let fps = crate::counter::place(DEFAULT_POS, size(1.0, 40.0), viewport);
    assert_eq!(fps, [cps[0], cps[1] + 26.0]);
    let widths = crate::cps::Widths::GUESS;
    assert_eq!(widths.fps_width(60), widths.fps_width(144));
    assert!(widths.fps_width(1000) > widths.fps_width(999));
}

#[test]
fn the_patch_carries_the_text_and_place() {
    let view = View {
        visible: true,
        fps: 144,
        scale: 1.5,
        width: 40.0,
        offset: [4.0, 16.0],
        edit: None,
    };
    let patch = patch(&view);
    assert!(patch.starts_with(r##"{"binding_changes":{"#fps_visible":true,"##));
    assert!(patch.contains(r##""#fps_text":"FPS: 144","##));
    assert!(patch.contains(r##""#fps_offset":[4,16]"##), "{patch}");
    assert!(patch.contains(r##""#fps_w":63,"##), "{patch}");
    assert!(patch.contains(r##""#fpsedit_visible":false"##));
    assert!(patch.ends_with("}}"));
    let hidden = initial_bindings();
    assert!(hidden.contains(r##""#fps_visible":false"##));
    let json = format!("{{{hidden}}}");
    assert!(!json.contains(",}"), "no trailing comma: {json}");
}
