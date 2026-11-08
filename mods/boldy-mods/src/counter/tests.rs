use super::*;

#[test]
fn counts_arrows_and_finds_bows() {
    let mut inventory = vec![None; 36];
    inventory[0] = Some(("minecraft:bow", 1));
    inventory[4] = Some((ARROW, 64));
    inventory[20] = Some((ARROW, 3));
    inventory[12] = Some(("minecraft:crossbow", 1));
    let quiver = Quiver::count(inventory.iter().copied(), Some((ARROW, 10)));
    assert_eq!(quiver.arrows, 77);
    assert_eq!(quiver.bow_slots, 1, "only hotbar bows can be held");
    assert!(!quiver.bow_offhand);
    assert!(quiver.holding_bow(Some(0)));
    assert!(!quiver.holding_bow(Some(4)));
    assert!(!quiver.holding_bow(None));
    let offhand = Quiver::count([None; 36], Some(("minecraft:crossbow", 1)));
    assert_eq!(offhand.arrows, 0);
    assert!(offhand.holding_bow(Some(3)));
}

#[test]
fn a_resized_window_remeasures_now_and_republishes_once_settled() {
    let mut watch = ViewportWatch::new();
    let first = [480.0, 270.0, 4.0];
    assert_eq!(watch.observe(first, 0.0), Reaction::default());
    assert_eq!(watch.observe(first, 1.0), Reaction::default());
    // Dragging a window edge: re-measured every step, republished only after it stops.
    let steps = [
        [500.0, 270.0, 4.0],
        [520.0, 280.0, 4.0],
        [540.0, 280.0, 3.0],
    ];
    for (index, viewport) in steps.into_iter().enumerate() {
        let reaction = watch.observe(viewport, 2.0 + index as f32 * 0.1);
        assert_eq!(
            reaction,
            Reaction {
                remeasure: true,
                republish: false
            }
        );
    }
    let still = steps[2];
    assert_eq!(watch.observe(still, 2.4), Reaction::default());
    let settled = watch.observe(still, 2.6);
    assert_eq!(
        settled,
        Reaction {
            remeasure: true,
            republish: true
        }
    );
    assert_eq!(watch.observe(still, 3.0), Reaction::default());
}

#[test]
fn a_counter_beside_the_hotbar_stays_beside_it_when_the_window_changes() {
    // Placed right of the hotbar (91 units from the middle) and just above the bottom
    // on a 480x270 HUD, then the HUD changes size.
    let small = [480.0, 270.0];
    let size = size(1.0, 12.0);
    let beside = [240.0 + 91.0 + 2.0, 270.0 - 11.0 - size[1] / 2.0];
    let pos = centre(beside, size, small);
    assert_eq!(pos.anchor, [Anchor::Centre, Anchor::End]);
    assert_eq!(place(pos, size, small), beside);
    for big in [[960.0, 540.0], [1280.0, 540.0], [427.0, 240.0]] {
        let corner = place(pos, size, big);
        // The same distance from the middle and from the bottom, as the hotbar is.
        assert_eq!(corner[0] - big[0] / 2.0, beside[0] - small[0] / 2.0);
        assert_eq!(big[1] - corner[1], small[1] - beside[1]);
    }
    // Near the left or top edge it follows that edge instead.
    let near_corner = centre([10.0, 12.0], size, small);
    assert_eq!(near_corner.anchor, [Anchor::Start, Anchor::Start]);
    assert_eq!(place(near_corner, size, [960.0, 540.0]), [10.0, 12.0]);
}

#[test]
fn an_older_fraction_becomes_an_anchored_position() {
    let pos = Pos::fraction(0.607_291_64, 0.977_777_8);
    assert!(pos.is_fraction());
    let viewport = [839.0, 749.0];
    let at = pos.at(viewport);
    let anchored = Pos::anchored(at, viewport);
    assert!(!anchored.is_fraction());
    assert_eq!(anchored.anchor, [Anchor::Centre, Anchor::End]);
    let again = anchored.at(viewport);
    assert!((again[0] - at[0]).abs() < 1e-3 && (again[1] - at[1]).abs() < 1e-3);
    assert_eq!(Pos::fraction(f32::NAN, 9.0).off, [0.5, 1.0]);
    assert_eq!(Anchor::from_name(Anchor::End.name()), Some(Anchor::End));
}

#[test]
fn placement_stays_on_screen_and_round_trips() {
    let viewport = [480.0, 270.0];
    let size = size(2.0, 12.0);
    assert_eq!(size, [60.0, 32.0]);
    // The icon (32 wide at this size) is centred on the position.
    let corner = place(Pos::fraction(0.5, 0.5), size, viewport);
    assert_eq!(corner, [224.0, 119.0]);
    let back = centre(corner, size, viewport).at(viewport);
    assert!((back[0] - 240.0).abs() < 1e-4 && (back[1] - 135.0).abs() < 1e-4);
    // A wider number leaves the icon where it was: only the number moves.
    let wider = self::size(2.0, 30.0);
    assert_eq!(place(Pos::fraction(0.5, 0.5), wider, viewport), corner);
    // Dragged past the corner, it stops at the edge.
    assert_eq!(
        place(Pos::fraction(1.0, 1.0), size, viewport),
        [420.0, 238.0]
    );
    assert_eq!(place(Pos::fraction(0.0, 0.0), size, viewport), [0.0, 0.0]);
}

#[test]
fn scale_steps_stay_in_bounds() {
    assert!((Counter::stepped(1.0, 1.0) - 1.1).abs() < 1e-5);
    assert!((Counter::stepped(1.0, -3.0) - 0.7).abs() < 1e-5);
    assert_eq!(Counter::stepped(3.95, 5.0), MAX_SCALE);
    assert_eq!(Counter::stepped(0.5, -1.0), MIN_SCALE);
    assert_eq!(clamp_scale(f32::NAN), 1.0);
    assert_eq!(digits(0), 1);
    assert_eq!(digits(9), 1);
    assert_eq!(digits(10), 2);
    assert_eq!(digits(2368), 4);
}

#[test]
fn the_document_is_balanced_json_and_escapes_at_compile_time() {
    for (open, close) in [('{', '}'), ('[', ']')] {
        assert_eq!(
            DOCUMENT.matches(open).count(),
            DOCUMENT.matches(close).count()
        );
    }
    assert!(
        !DOCUMENT.contains(",]") && !DOCUMENT.contains(",}"),
        "no trailing commas"
    );
    assert!(DOCUMENT.contains("#jumpreset_text"));
    assert!(DOCUMENT.contains(r#""xhair":{"#) && DOCUMENT.contains(r#""drawgrid":{"#));
    assert!(DOCUMENT.contains("@boldy_hud.xhair"));
    for prefix in ["arrows", "cpsedit", "fpsedit"] {
        assert!(DOCUMENT.contains(&format!("#{prefix}_handle_3_offset")));
        assert!(DOCUMENT.contains(&format!("#{prefix}_size_offset")));
        let bound = overlay_bindings(prefix, [34.0, 16.0], 1.0, Some(Edit::default()));
        assert!(bound.starts_with(&format!(r##""#{prefix}_frame_visible":true,"##)));
        assert!(!bound.ends_with(','));
        assert!(bound.contains(&format!(r##""#{prefix}_size_text":"100%""##)));
    }
    // The compile-time escape is the runtime one.
    assert_eq!(DOCUMENT_ESCAPED, crate::panel::json_string(DOCUMENT));
    assert!(DOCUMENT_ESCAPED.starts_with("\"{\\\"namespace"));
}

#[test]
fn surface_and_patches_are_json() {
    let surface = surface();
    assert!(surface.starts_with(r#"{"screen":"boldy_hud.root","document":""#));
    assert!(
        surface.contains(
            r##","#arrow_item":{"item":{"identifier":"minecraft:arrow","metadata":0}},"#cps_visible":false,"##
        ) && surface.contains(r##""#cpsedit_frame_visible":false"##)
            && surface.ends_with("}}"),
        "{surface}"
    );
    assert!(surface.contains(r##""#arrows_visible":false"##));
    assert!(surface.contains(r##""#crosshair_visible":false,"##));
    for index in 0..4 {
        assert!(surface.contains(&format!("handle_{index}_offset")));
    }
    let view = View {
        visible: true,
        arrows: Some(0),
        offset: [10.0, 20.5],
        size: [34.0, 16.0],
        scale: 1.0,
        text_width: 6.0,
        edit: Some(Edit {
            guides: [Some(240.0), None],
            ..Edit::default()
        }),
    };
    let patch = patch(&view);
    assert!(patch.starts_with(r##"{"binding_changes":{"#arrows_visible":true,"#guide_x_visible":true,"#guide_y_visible":false,"#guide_x_offset":[239.5,0],"##), "{patch}");
    assert!(
        patch.contains(r##""#arrows_text":"0","#arrows_color":[1,0.35,0.35,1]"##),
        "{patch}"
    );
    // Handles sit on the frame's corners, centred 3 units outside the counter.
    assert!(
        patch.contains(r##""#arrows_handle_0_offset":[-6.5,-6.5]"##),
        "{patch}"
    );
    assert!(
        patch.contains(r##""#arrows_handle_3_offset":[33.5,15.5]"##),
        "{patch}"
    );
    assert!(patch.ends_with("}}"));
    assert_eq!(patch.matches('{').count(), patch.matches('}').count());
    let unknown = super::patch(&View {
        arrows: None,
        ..view
    });
    assert!(
        unknown.contains(r##""#arrows_text":"?","#arrows_color":[1,0.85,0.3,1]"##),
        "{unknown}"
    );
    assert_eq!(surface.matches('{').count(), surface.matches('}').count());
}

#[test]
fn every_corner_resizes_with_the_opposite_corner_still() {
    let viewport = [480.0, 270.0];
    let corner = [100.0, 50.0];
    let size = size(1.0, 12.0); // [30, 16]
    assert_eq!(hit([97.0, 47.0], corner, size), Some(Part::Handle(0)));
    assert_eq!(hit([133.0, 47.0], corner, size), Some(Part::Handle(1)));
    assert_eq!(hit([97.0, 69.0], corner, size), Some(Part::Handle(2)));
    assert_eq!(hit([133.0, 69.0], corner, size), Some(Part::Handle(3)));
    assert_eq!(hit([115.0, 58.0], corner, size), Some(Part::Body));
    assert_eq!(hit([60.0, 58.0], corner, size), None);
    // Bottom right: pulled out to double, the top-left corner stays.
    let drag = start_drag(Part::Handle(3), [130.0, 66.0], corner, size);
    let (scale, at, _) = drag_to(drag, [160.0, 82.0], 1.0, size, viewport);
    assert!((scale - 2.0).abs() < 1e-6);
    assert_eq!(at, corner);
    // Top left: pulled up and left to double, the bottom-right corner stays.
    let drag = start_drag(Part::Handle(0), [100.0, 50.0], corner, size);
    let (scale, at, _) = drag_to(drag, [70.0, 34.0], 1.0, size, viewport);
    assert!((scale - 2.0).abs() < 1e-6);
    assert_eq!(at, [70.0, 34.0]);
    // Top right: the bottom-left corner stays.
    let drag = start_drag(Part::Handle(1), [130.0, 50.0], corner, size);
    let (scale, at, _) = drag_to(drag, [160.0, 34.0], 1.0, size, viewport);
    assert!((scale - 2.0).abs() < 1e-6);
    assert_eq!(at, [100.0, 34.0]);
    // Too small or too big stops at the size limits.
    let (scale, ..) = drag_to(drag, [0.0, 400.0], 1.0, size, viewport);
    assert_eq!(scale, MIN_SCALE);
}

#[test]
fn snaps_beside_and_level_with_the_hotbar() {
    let viewport = [480.0, 270.0];
    // Hotbar: x 149..331, y 248..270.
    let size = size(1.0, 12.0); // [30, 16]
    // Dropped near the hotbar's right side and middle height: it lines up beside it.
    let (corner, lines) = snap_move([334.5, 250.0], size, viewport);
    assert_eq!(corner, [333.0, 251.0]);
    assert_eq!(lines, [Some(333.0), Some(259.0)]);
    // Its bottom edge snaps level with the hotbar's bottom (the screen's bottom).
    let (corner, lines) = snap_move([20.0, 253.5], size, viewport);
    assert_eq!(corner[1], 254.0);
    assert_eq!(lines[1], Some(270.0));
    // Its centre snaps to the middle of the screen.
    let (corner, lines) = snap_move([227.0, 100.0], size, viewport);
    assert_eq!(corner[0], 225.0);
    assert_eq!(lines, [Some(240.0), None]);
    // Far from every guide nothing snaps.
    assert_eq!(snap_move([60.0, 100.0], size, viewport).1, [None, None]);
    // Resizing beside the hotbar from its bottom edge: the top edge snaps to the hotbar's
    // top, so the counter is exactly as tall as the hotbar.
    let corner = [333.0, 254.0];
    let drag = start_drag(Part::Handle(1), [363.0, 254.0], corner, size);
    let (scale, at, lines) = drag_to(drag, [375.0, 249.0], 1.0, size, viewport);
    assert!((scale - 22.0 / 16.0).abs() < 1e-5, "{scale}");
    assert_eq!(at, [333.0, 248.0]);
    assert_eq!(lines, [None, Some(248.0)]);
    assert!(label_above([0.0, 250.0], size, viewport));
    assert!(!label_above([0.0, 100.0], size, viewport));
}
