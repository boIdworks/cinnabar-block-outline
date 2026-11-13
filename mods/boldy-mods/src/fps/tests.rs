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
    let fps = crate::counter::place(DEFAULT_POS, size(1.0, 40.0, 0.0), viewport);
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
        coords: None,
        offset: [4.0, 16.0],
        edit: None,
    };
    let patch = patch(&view);
    assert!(patch.starts_with(r##"{"binding_changes":{"#fps_visible":true,"##));
    assert!(patch.contains(r##""#fps_text":"FPS: 144","##));
    assert!(patch.contains(r##""#fps_offset":[4,16]"##), "{patch}");
    assert!(patch.contains(r##""#fps_w":63,"##), "{patch}");
    assert!(patch.contains(r##""#fpsedit_visible":false"##));
    assert!(patch.contains(r##""#fpsc_visible":false"##), "{patch}");
    assert!(patch.ends_with("}}"));
    let hidden = initial_bindings();
    assert!(hidden.contains(r##""#fps_visible":false"##));
    let json = format!("{{{hidden}}}");
    assert!(!json.contains(",}"), "no trailing comma: {json}");
}

#[test]
fn the_block_is_the_feet_position_rounded_down() {
    assert_eq!(block_of((12.7, 64.0, -30.2)), Some([12, 64, -31]));
    assert_eq!(block_of((-0.5, 0.0, 0.5)), Some([-1, 0, 0]));
    // Not a position: nothing shown, not a wrong number.
    assert_eq!(block_of((f32::NAN, 64.0, 0.0)), None);
    assert_eq!(block_of((0.0, f32::INFINITY, 0.0)), None);
}

#[test]
fn the_coordinates_line_reads_xyz_with_signs() {
    let mut text = String::new();
    coords_text(&mut text, [12, 64, -30]);
    assert_eq!(text, "XYZ: 12 64 -30");
    text.clear();
    coords_text(&mut text, [0, -1, 1_000_000]);
    assert_eq!(text, "XYZ: 0 -1 1000000");
    text.clear();
    // The most negative number does not overflow.
    coords_text(&mut text, [i32::MIN, 0, 0]);
    assert_eq!(text, "XYZ: -2147483648 0 0");
}

#[test]
fn the_coordinates_widen_the_counter_and_add_a_line() {
    let widths = crate::cps::Widths::GUESS;
    let short = coords_width(&widths, [1, 2, 3]);
    let long = coords_width(&widths, [-1234, 64, -5678]);
    assert!(long > short, "more characters, wider");
    // Off: the counter is one line of the FPS text. On: two lines, as wide as the wider text.
    assert_eq!(base(40.0, 0.0), [42.0, 10.0]);
    assert_eq!(base(40.0, long), [long + 2.0, 20.0]);
    assert_eq!(
        base(100.0, short),
        [102.0, 20.0],
        "the FPS text can be the wider"
    );
    assert_eq!(size(2.0, 40.0, long), [(long + 2.0) * 2.0, 40.0]);
}

#[test]
fn the_patch_shows_the_coordinates_under_the_fps_when_on() {
    let widths = crate::cps::Widths::GUESS;
    let coords = [12, 64, -30];
    let width = coords_width(&widths, coords);
    let view = View {
        visible: true,
        fps: 60,
        scale: 2.0,
        width: 40.0,
        coords: Some((coords, width)),
        offset: [4.0, 16.0],
        edit: None,
    };
    let patch = patch(&view);
    assert!(patch.contains(r##""#fpsc_visible":true"##), "{patch}");
    assert!(
        patch.contains(r##""#fpsc_text":"XYZ: 12 64 -30""##),
        "{patch}"
    );
    // One line down at scale 2, and the first label stays one line tall.
    assert!(patch.contains(r##""#fpsc_offset":[4,36]"##), "{patch}");
    assert!(patch.contains(r##""#fps_h":20,"##), "{patch}");
    // The edit frame covers both lines.
    assert!(patch.contains(r##""#fpsedit_h":40"##), "{patch}");
    // Hidden, the line hides with it.
    let mut hidden = view;
    hidden.visible = false;
    assert!(self::patch(&hidden).contains(r##""#fpsc_visible":false"##));
    let initial = format!("{{{}}}", initial_bindings());
    assert!(initial.contains(r##""#fpsc_visible":false"##));
}

#[test]
fn the_coordinates_are_off_by_default() {
    assert!(!Fps::default().coords);
}

#[test]
fn the_coordinates_alone_take_the_top_line() {
    let widths = crate::cps::Widths::GUESS;
    let coords = [1, 2, 3];
    let width = coords_width(&widths, coords);
    assert_eq!(base(0.0, width), [width + 2.0, 10.0], "one line");
    let view = View {
        visible: true,
        fps: 60,
        scale: 1.0,
        width: 0.0,
        coords: Some((coords, width)),
        offset: [4.0, 16.0],
        edit: None,
    };
    let patch = patch(&view);
    assert!(patch.contains(r##""#fps_visible":false"##), "{patch}");
    assert!(patch.contains(r##""#fpsc_visible":true"##), "{patch}");
    assert!(patch.contains(r##""#fpsc_offset":[4,16]"##), "{patch}");
}
