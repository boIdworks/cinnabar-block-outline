use super::*;

const FULL: [f64; 6] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

#[test]
fn parses_colors_and_clamps_numbers() {
    let s = Settings::parse(r##"{"color": "#ff8800", "width": 20, "reach": 3.5}"##);
    assert_eq!(s.rgba, [255, 136, 0, 255]);
    assert!(!s.rainbow);
    assert_eq!(s.width, MAX_WIDTH);
    assert_eq!(s.reach, 3.5);
    assert_eq!(parse_color("#0F08"), None);
    assert_eq!(parse_color("0f0"), Some([0, 255, 0, 255]));
    assert_eq!(parse_color("#11223344"), Some([0x11, 0x22, 0x33, 0x44]));
    assert_eq!(Settings::parse(""), Settings::default());
    assert_eq!(
        Settings::parse("{\"color\": \"nope\"}"),
        Settings::default()
    );
}

#[test]
fn settings_round_trip_and_legacy_rainbow() {
    let s = Settings {
        rgba: [1, 2, 3, 4],
        rainbow: true,
        width: 3.5,
        fill: 35,
        reach: 5.0,
        menu_key: KeyName::new("KeyE").unwrap(),
        fill_rgb: Some([1, 2, 3]),
        pulse: true,
        highlight_cancel: 40,
        mob: Style {
            rgba: [200, 10, 20, 128],
            rainbow: false,
            width: 6.5,
            fill: 0,
            fill_rgb: None,
            pulse: false,
        },
        show_all_mobs: false,
        aim_rgb: [9, 9, 9],
        fullbright: true,
        fullbright_key: KeyName::new("KeyB"),
        favourites: [Some([9, 8, 7, 6]), None, Some([255, 0, 0, 255]), None],
        next_favourite: 3,
        counter: crate::counter::Counter {
            on: false,
            bow_only: true,
            pos: crate::counter::Pos {
                anchor: [crate::counter::Anchor::Start, crate::counter::Anchor::End],
                off: [30.5, -12.25],
            },
            scale: 2.5,
        },
        zoom: crate::zoom::Zoom {
            on: false,
            toggle: true,
            scroll: false,
            sensitivity: false,
            level: 6.5,
            time: 0.4,
            step: 35,
            // Unbound: written as an empty name and read back as none.
            key: None,
        },
        cps: crate::cps::Cps {
            on: false,
            right: false,
            zeros: false,
            pos: crate::counter::Pos {
                anchor: [crate::counter::Anchor::End, crate::counter::Anchor::Centre],
                off: [-14.5, 3.25],
            },
            scale: 1.7,
        },
        fps: crate::fps::Fps {
            on: false,
            pos: crate::counter::Pos {
                anchor: [crate::counter::Anchor::Centre, crate::counter::Anchor::End],
                off: [20.5, -30.0],
            },
            scale: 2.3,
        },
        jump_reset: crate::jumpreset::JumpReset {
            on: false,
            window: 120,
            knockback: 0.3,
            log: {
                let mut log = crate::jumpreset::Log::new();
                log.push(crate::jumpreset::Row {
                    offset_ms: Some(-40),
                    knockback: 55,
                    grounded: Some(true),
                    speed: 130,
                });
                log
            },
        },
        crosshair: crate::crosshair::Crosshair {
            on: true,
            style: 5,
            rgb: [12, 200, 99],
            opacity: 70,
            length: 7.5,
            thickness: 1.5,
            gap: 0.0,
            outline: false,
            pixels: crate::crosshair::Drawing::parse("1/0/ffffffff/0/8001/0/100", 32),
            pixel_size: 3,
            mirror: false,
        },
        block_on: false,
        mob_on: false,
    };
    assert_eq!(Settings::parse(&s.to_json()), s);
    // Crosshair values typed by hand are put on a step and kept in bounds.
    let crosshair = Settings::parse(
        r#"{"crosshair_length": 99, "crosshair_gap": 1.3, "crosshair_opacity": 3, "crosshair_style": "dot"}"#,
    )
    .crosshair;
    assert_eq!(crosshair.length, crate::crosshair::LENGTH.max);
    assert_eq!(crosshair.gap, 1.5);
    assert_eq!(crosshair.opacity, crate::crosshair::MIN_OPACITY);
    assert_eq!(crate::crosshair::style_name(crosshair.style), "Dot");
    assert!(!crosshair.on, "off unless switched on");
    // A window typed by hand is rounded onto a step and kept in bounds.
    let typed = |window: &str| {
        Settings::parse(&format!(r#"{{"jump_reset_window": {window}}}"#))
            .jump_reset
            .window
    };
    assert_eq!(typed("77"), 75);
    assert_eq!(typed("1"), crate::jumpreset::MIN_WINDOW);
    assert_eq!(typed("99999"), crate::jumpreset::MAX_WINDOW);
    let kb = |value: &str| {
        Settings::parse(&format!(r#"{{"jump_reset_carry": {value}}}"#))
            .jump_reset
            .knockback
    };
    assert_eq!(kb("0.4"), 0.4);
    assert_eq!(kb("0"), crate::jumpreset::MIN_KNOCKBACK);
    assert_eq!(kb("9"), crate::jumpreset::MAX_KNOCKBACK);
    // Older files have no jump reset: it is on, at the default window.
    assert_eq!(
        Settings::parse("{}").jump_reset,
        crate::jumpreset::JumpReset::default()
    );
    let legacy = Settings::parse(r#"{"color": "rainbow", "width": 3, "reach": 6}"#);
    assert!(legacy.rainbow);
    assert_eq!(legacy.rgba, Settings::default().rgba);
    let saved = Settings::parse(r##"{"color": "#50FF3CFF", "width": 3, "reach": 6}"##);
    assert_eq!(saved.rgba, [0x50, 0xFF, 0x3C, 0xFF]);
}

#[test]
fn the_nearest_mob_boxes_come_nearest_first_within_range_and_the_cap() {
    let eye = [0.0, 1.6, 0.0];
    let at = |x: f64, z: f64| -> ActorBox { [[x - 0.3, 0.0, z - 0.3, x + 0.3, 1.8, z + 0.3]] };
    let actors = [
        at(5.0, 0.0),
        at(2.0, 0.0),
        at(30.0, 0.0),
        at(3.0, 4.0),
        at(-1.0, 0.0),
    ];
    let nearest = |view, range, limit| nearest_mob_boxes(&actors, eye, view, range, limit);
    let xs = |boxes: &[[[f64; 3]; 2]]| -> Vec<i32> {
        boxes
            .iter()
            .map(|b| ((b[0][0] + b[1][0]) / 2.0).round() as i32)
            .collect()
    };
    // Nearest first, the one 30 blocks away outside a range of 12.
    assert_eq!(xs(&nearest(None, 12.0, 8)), [-1, 2, 3, 5]);
    // The cap keeps the nearest.
    assert_eq!(xs(&nearest(None, 12.0, 2)), [-1, 2]);
    assert_eq!(xs(&nearest(None, 12.0, 1)), [-1]);
    assert!(nearest(None, 12.0, 0).is_empty());
    // Nothing in range, nothing returned.
    assert!(nearest(None, 0.5, 8).is_empty());
    // Looking along +X, what is at -X is behind and is left out; the cap then goes to the rest.
    let along_x = Some([1.0, 0.0, 0.0]);
    assert_eq!(xs(&nearest(along_x, 12.0, 8)), [2, 3, 5]);
    assert_eq!(xs(&nearest(along_x, 12.0, 2)), [2, 3]);
    // Equal distances keep the order they came in.
    let twins = [at(3.0, 0.0), at(-3.0, 0.0), at(0.0, 3.0)];
    let order = nearest_mob_boxes(&twins, eye, None, 12.0, 8);
    assert_eq!(order.len(), 3);
    assert!(
        order[0][0][0] > 2.0 && order[1][1][0] < -2.0,
        "first come, first kept"
    );
    // Several boxes in one actor all count; an empty actor adds nothing.
    let many = [
        vec![
            [0.0, 0.0, 2.0, 1.0, 1.0, 3.0],
            [0.0, 0.0, 6.0, 1.0, 1.0, 7.0],
        ],
        vec![],
    ];
    assert_eq!(nearest_mob_boxes(&many, eye, None, 12.0, 8).len(), 2);
    // Far more than the cap in, never more than the cap out.
    let crowd: Vec<ActorBox> = (0..40).map(|n| at(f64::from(n) * 0.2, 1.0)).collect();
    assert_eq!(
        nearest_mob_boxes(&crowd, eye, None, 12.0, 100).len(),
        MOB_BOX_CAP
    );
    let nearest16 = nearest_mob_boxes(&crowd, eye, None, 12.0, 100);
    assert!(
        nearest16
            .windows(2)
            .all(|pair| pair[0][0][0] <= pair[1][0][0])
    );
}

#[test]
fn boxes_wholly_behind_the_viewer_are_skipped() {
    let eye = [0.0, 1.6, 0.0];
    let ahead = look_direction(0.0, 0.0);
    let one = |centre: [f64; 3]| {
        (
            [centre[0] - 0.3, centre[1] - 0.9, centre[2] - 0.3],
            [centre[0] + 0.3, centre[1] + 0.9, centre[2] + 0.3],
        )
    };
    let seen = |centre: [f64; 3], dir: [f64; 3]| {
        let (min, max) = one(centre);
        in_front_of(min, max, eye, dir)
    };
    // Looking along -Z: a mob ahead is in front, one behind is not, to the side it is.
    assert!(seen([0.0, 1.0, -5.0], ahead));
    assert!(!seen([0.0, 1.0, 5.0], ahead));
    assert!(
        seen([6.0, 1.0, -0.2], ahead),
        "off to the side, slightly ahead"
    );
    // Straight to the side and a little behind still straddles with its width.
    assert!(seen([3.0, 1.0, 0.2], ahead));
    // Right on top of you, or around you, is kept.
    assert!(seen([0.0, 1.0, 0.0], ahead));
    // Turning round flips it.
    let back = look_direction(std::f32::consts::PI, 0.0);
    assert!(!seen([0.0, 1.0, -5.0], back));
    assert!(seen([0.0, 1.0, 5.0], back));
    // Looking up or down: a mob below you while looking up is behind the plane.
    let up = look_direction(0.0, 1.4);
    assert!(!seen([0.0, -6.0, -0.5], up));
    assert!(seen([0.0, 6.0, -0.5], up));
    // Every corner counts: a long box reaching in front of the plane is in front.
    assert!(in_front_of([-0.3, 0.7, -4.0], [0.3, 2.5, 4.0], eye, ahead));
    // Nonsense in, kept: nothing is hidden on a guess.
    assert!(in_front_of([f64::NAN; 3], [1.0; 3], eye, ahead));
    assert!(in_front_of([0.0; 3], [1.0; 3], eye, [f64::NAN, 0.0, 0.0]));
}

#[test]
fn look_direction_matches_actor_convention() {
    let forward = look_direction(0.0, 0.0);
    assert!((forward[2] + 1.0).abs() < 1e-9);
    let left = look_direction(std::f32::consts::FRAC_PI_2, 0.0);
    assert!((left[0] + 1.0).abs() < 1e-6);
    let up = look_direction(0.0, std::f32::consts::FRAC_PI_2);
    assert!((up[1] - 1.0).abs() < 1e-6);
}

#[test]
fn raycast_hits_first_solid_block_within_reach() {
    let solid = |p: [i32; 3]| {
        if p == [0, 64, -3] {
            Cell::Boxes(vec![FULL])
        } else {
            Cell::Boxes(vec![])
        }
    };
    let hit = raycast([0.5, 64.5, 0.5], [0.0, 0.0, -1.0], 6.0, solid).unwrap();
    assert_eq!(hit.block, [0, 64, -3]);
    assert_eq!(hit.min, [0.0, 64.0, -3.0]);
    assert_eq!(hit.max, [1.0, 65.0, -2.0]);
    assert_eq!(
        raycast([0.5, 64.5, 0.5], [0.0, 0.0, -1.0], 2.0, solid),
        None
    );
}

#[test]
fn raycast_respects_partial_shapes_and_unknown_cells() {
    // A bottom slab under the ray is missed; the block behind it is hit.
    let slab = [0.0, 0.0, 0.0, 1.0, 0.5, 1.0];
    let world = |p: [i32; 3]| match p {
        [0, 64, -1] => Cell::Boxes(vec![slab]),
        [0, 64, -2] => Cell::Boxes(vec![FULL]),
        _ => Cell::Boxes(vec![]),
    };
    let hit = raycast([0.5, 64.75, 0.5], [0.0, 0.0, -1.0], 6.0, world).unwrap();
    assert_eq!(hit.block, [0, 64, -2]);
    let unknown = |p: [i32; 3]| {
        if p == [0, 64, -1] {
            Cell::Unknown
        } else {
            Cell::Boxes(vec![FULL])
        }
    };
    assert_eq!(
        raycast([0.5, 64.5, 0.5], [0.0, 0.0, -1.0], 6.0, unknown),
        None
    );
}

#[test]
fn diagonal_ray_finds_block() {
    let dir = look_direction(0.6, -0.7);
    let target = |p: [i32; 3]| {
        if p[1] <= 62 {
            Cell::Boxes(vec![FULL])
        } else {
            Cell::Boxes(vec![])
        }
    };
    let hit = raycast([0.3, 64.62, 0.7], dir, 6.0, target).unwrap();
    assert_eq!(hit.block[1], 62);
}

#[test]
fn bridging_outlines_the_block_underfoot_only_when_looking_steeply_down() {
    // Standing at the edge of a one-block-wide bridge along -Z; the void lies ahead.
    let bridge = |p: [i32; 3]| {
        if p[0] == 0 && p[1] == 63 && p[2] >= 0 {
            Cell::Boxes(vec![FULL])
        } else {
            Cell::Boxes(vec![])
        }
    };
    let eye = [0.5, 65.62, 0.3];
    // Aiming into the empty cell in front of the bridge's end, [0, 63, -1].
    let steep = look_direction(0.0, -1.35);
    assert_eq!(
        raycast(eye, steep, 6.0, bridge),
        None,
        "the look ray misses into the void"
    );
    assert_eq!(placement_cell([0, 63, 0], steep), [0, 63, -1]);
    assert_eq!(
        support_hit(eye, steep, 6.0, bridge).unwrap().block,
        [0, 63, 0]
    );
    // Steep enough, but aiming two blocks ahead: the ray passes over the placement cell.
    let ahead = look_direction(0.0, -0.8);
    assert!(ahead[1] < SUPPORT_MAX_Y);
    assert_eq!(support_hit(eye, ahead, 6.0, bridge), None);
    let shallow = look_direction(0.0, -0.6);
    assert_eq!(
        support_hit(eye, shallow, 6.0, bridge),
        None,
        "y -0.56 is not steep enough"
    );
    // Facing diagonally (yaw 45°, towards -x -z), aiming into the corner gap.
    let diagonal = look_direction(std::f32::consts::FRAC_PI_4, -1.3);
    assert!(diagonal[0] < 0.0 && diagonal[2] < 0.0);
    assert_eq!(
        support_hit([0.5, 65.62, 0.5], diagonal, 6.0, bridge)
            .unwrap()
            .block,
        [0, 63, 0]
    );
    // Bridging straight along -z but aiming into the side gap at -x, and the far corner.
    let side = [-0.62, -0.78, 0.0];
    assert!(side[1] < SUPPORT_MAX_Y);
    let unit = |v: [f64; 3]| {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        v.map(|c| c / l)
    };
    assert!(
        support_hit([0.5, 65.62, 0.5], unit(side), 6.0, bridge).is_some(),
        "side gap"
    );
    assert!(
        support_hit([0.5, 65.62, 0.5], unit([-0.45, -0.78, -0.45]), 6.0, bridge).is_some(),
        "corner"
    );
    // Diagonally, a look steep enough to bridge (y below -0.7) lands at most ~1.6 blocks out,
    // still inside the corner gap, so the outline shows across that whole range.
    let shallowest = look_direction(std::f32::consts::FRAC_PI_4, -0.8);
    assert!(shallowest[1] < SUPPORT_MAX_Y);
    assert!(support_hit([0.5, 65.62, 0.5], shallowest, 6.0, bridge).is_some());
    // Too far above the support for the 2-block downward ray.
    assert_eq!(support_hit([0.5, 66.62, 0.3], steep, 6.0, bridge), None);
}

#[test]
fn width_shrinks_with_distance_but_not_up_close() {
    let (min, max) = ([0.0; 3], [1.0; 3]);
    assert_eq!(
        width_at(4.0, min, max, [0.5, 0.5, 2.5]),
        4.0,
        "2 blocks away"
    );
    assert_eq!(
        width_at(4.0, min, max, [0.5, 0.5, 5.5]),
        2.0,
        "5 blocks away: half"
    );
    assert_eq!(
        width_at(1.5, min, max, [0.5, 0.5, 20.5]),
        1.0,
        "never below one pixel"
    );
    assert_eq!(width_at(1.0, min, max, [0.5, 0.5, 20.5]), 1.0);
}

#[test]
fn region_follows_the_look_ray() {
    let eye = [0.5, 65.62, 0.5];
    let volume =
        |(min, max): ([i32; 3], [i32; 3])| (0..3).map(|a| max[a] - min[a]).product::<i32>();
    // Every direction at the longest reach fits the host's extent and holds the ray.
    for yaw in 0..16 {
        for pitch in -4..=4 {
            let dir = look_direction(yaw as f32 * 0.4, pitch as f32 * 0.38);
            let region = region_for(None, eye, dir, f64::from(MAX_REACH), REGION_SLACK);
            assert!(
                (0..3).all(|a| region.1[a] - region.0[a] <= REGION_EXTENT),
                "{region:?}"
            );
            assert!(region_covers(region, eye, dir, f64::from(MAX_REACH)));
            let end = (0..3)
                .map(|a| eye[a] + dir[a] * f64::from(MAX_REACH))
                .collect::<Vec<_>>();
            assert!(
                (0..3).all(|a| f64::from(region.0[a]) <= end[a] && end[a] < f64::from(region.1[a]))
            );
        }
    }
    // Looking straight ahead reads far fewer blocks than the old 16-block cube.
    let ahead = region_for(None, eye, [0.0, 0.0, -1.0], 5.7, REGION_SLACK);
    assert!(volume(ahead) * 4 < 16 * 16 * 16, "{}", volume(ahead));
    // Small turns and steps keep the region; a big turn asks for a new one.
    let turned = look_direction(0.15, 0.1);
    assert_eq!(region_for(Some(ahead), eye, turned, 5.7, 0), ahead);
    assert_eq!(
        region_for(Some(ahead), [0.9, 65.62, 0.2], [0.0, 0.0, -1.0], 5.7, 0),
        ahead
    );
    assert_ne!(region_for(Some(ahead), eye, [1.0, 0.0, 0.0], 5.7, 0), ahead);
    // Looking steeply down holds the bridging support's depth and its neighbours.
    let down = look_direction(0.0, -1.2);
    let (min, max) = region_for(None, eye, down, 5.7, 0);
    assert!(min[1] <= (eye[1] - SUPPORT_DEPTH).floor() as i32 && min[0] <= -1 && 1 < max[0]);
    assert_eq!(
        volume(region_for(None, eye, [0.0, 0.0, -1.0], 5.7, 0)) as usize,
        region_cells(ray_region(eye, [0.0, 0.0, -1.0], 5.7, 1))
    );
}

#[test]
fn region_slack_follows_what_reads_cost() {
    let cells = |slack: i32| {
        region_cells(ray_region(
            [0.5, 65.62, 0.5],
            [0.0, 0.0, -1.0],
            5.7,
            1 + slack,
        ))
    };
    // Unknown cost: the smallest region first.
    assert_eq!(region_slack(None, cells), 0);
    // Cheap blocks: the full slack; dear ones: less; very dear: none.
    assert_eq!(region_slack(Some(10.0), cells), REGION_SLACK);
    let between = READ_FUEL_TARGET / cells(1) as f64;
    assert_eq!(region_slack(Some(between), cells), 1);
    assert_eq!(region_slack(Some(10_000.0), cells), 0);
}

#[test]
fn far_blocks_need_their_centre_within_reach() {
    // A wall along z = -6 in front of the eye: the ray reaches its face at 5.5, inside 5.7,
    // but the block's centre is 6.0 away, so Cinnabar targets nothing.
    let wall = |p: [i32; 3]| {
        if p[2] == -6 {
            Cell::Boxes(vec![FULL])
        } else {
            Cell::Boxes(vec![])
        }
    };
    let eye = [0.5, 64.5, 0.5];
    let ahead = [0.0, 0.0, -1.0];
    assert!(raycast(eye, ahead, 5.7, wall).is_some());
    assert_eq!(target(eye, ahead, 5.7, wall), None);
    // One block closer, the centre is 5.0 away and it is targeted.
    let near = |p: [i32; 3]| {
        if p[2] == -5 {
            Cell::Boxes(vec![FULL])
        } else {
            Cell::Boxes(vec![])
        }
    };
    assert_eq!(target(eye, ahead, 5.7, near).unwrap().block, [0, 64, -5]);
}

#[test]
fn hex_codes_and_the_wheel_round_trip() {
    assert_eq!(hex_code([255, 136, 0, 255]), "#FF8800");
    assert_eq!(hex_code([1, 2, 3, 4]), "#01020304");
    for rgba in [[255, 136, 0, 255], [1, 2, 3, 4], [80, 255, 60, 255]] {
        assert_eq!(parse_color(&hex_code(rgba)), Some(rgba));
    }
    // Red is at the right edge, cyan at the left, white in the middle.
    assert_eq!(wheel_colour(1.0, 0.5, 1.0), [255, 0, 0]);
    assert_eq!(wheel_colour(0.0, 0.5, 1.0), [0, 255, 255]);
    assert_eq!(wheel_colour(0.5, 0.5, 1.0), [255, 255, 255]);
    // Outside the disc clamps to full saturation; brightness is kept.
    assert_eq!(wheel_colour(2.0, 0.5, 0.5), [128, 0, 0]);
    for rgb in [
        [255, 136, 0],
        [40, 90, 255],
        [170, 60, 255],
        [255, 255, 255],
    ] {
        let [x, y] = wheel_position(rgb);
        let [_, _, value] = rgb_to_hsv(rgb);
        let back = wheel_colour(x, y, value);
        for c in 0..3 {
            assert!(back[c].abs_diff(rgb[c]) <= 2, "{rgb:?} -> {back:?}");
        }
    }
}

#[test]
fn mobs_in_front_win_over_blocks_like_cinnabar() {
    let wall = |p: [i32; 3]| {
        if p[2] == -4 {
            Cell::Boxes(vec![FULL])
        } else {
            Cell::Boxes(vec![])
        }
    };
    let eye = [0.5, 64.5, 0.5];
    let ahead = [0.0, 0.0, -1.0];
    let mob_at = |z: f64| vec![[0.2, 64.0, z - 0.3, 0.8, 65.0, z + 0.3]];
    // A mob 2 blocks ahead, in front of the wall at 3.5: its hitbox is outlined.
    match target_with_actors(eye, ahead, 5.7, wall, &[mob_at(-1.5)]) {
        Some(Target::Actor(hit)) => assert_eq!(hit.max, [0.8, 65.0, -1.2]),
        other => panic!("{other:?}"),
    }
    // Behind the wall, the wall wins.
    assert!(matches!(
        target_with_actors(eye, ahead, 5.7, wall, &[mob_at(-5.5)]),
        Some(Target::Block(_))
    ));
    // Off to the side, the ray misses it.
    let beside = vec![[3.0, 64.0, -2.0, 3.6, 65.0, -1.4]];
    assert!(matches!(
        target_with_actors(eye, ahead, 5.7, wall, &[beside]),
        Some(Target::Block(_))
    ));
    // Your own body, around the eye, never counts.
    let me = vec![[0.2, 63.0, 0.2, 0.8, 64.8, 0.8]];
    assert!(matches!(
        target_with_actors(eye, ahead, 5.7, wall, &[me]),
        Some(Target::Block(_))
    ));
    // A mob in front of the void blocks the bridging fallback too.
    let steep = look_direction(0.0, -1.35);
    let floor = |p: [i32; 3]| {
        if p == [0, 63, 0] {
            Cell::Boxes(vec![FULL])
        } else {
            Cell::Boxes(vec![])
        }
    };
    let eye = [0.5, 65.62, 0.3];
    assert!(target_with_actors(eye, steep, 5.7, floor, NO_ACTORS).is_some());
    let mob_below = vec![[0.0, 63.0, -1.0, 1.0, 64.0, 0.0]];
    assert!(matches!(
        target_with_actors(eye, steep, 5.7, floor, &[mob_below]),
        Some(Target::Actor(_))
    ));
}

#[test]
fn favourites_reset_and_pulse() {
    let mut s = Settings {
        menu_key: KeyName::new("KeyG").unwrap(),
        ..Settings::default()
    };
    for index in 0..5u8 {
        s.save_favourite([index, 0, 0, 255]);
    }
    // Five saves into four slots: the oldest was replaced.
    assert_eq!(s.favourites[0], Some([4, 0, 0, 255]));
    assert_eq!(s.favourites[1], Some([1, 0, 0, 255]));
    assert_eq!(Settings::parse(&s.to_json()), s);
    s.width = 7.0;
    s.pulse = true;
    s.fill_rgb = Some([1, 2, 3]);
    let favourites = s.favourites;
    s.reset();
    assert_eq!(s.width, Settings::default().width);
    assert_eq!(s.fill_rgb, None);
    assert_eq!(s.favourites, favourites, "favourites survive a reset");
    assert_eq!(s.menu_key.as_str(), "KeyG", "the menu key survives a reset");
    assert_eq!(s.pulse_factor(0.3), 1.0, "pulse off");
    s.pulse = true;
    let range = (0..60).map(|i| s.pulse_factor(i as f32 * 0.05));
    let (low, high) = range.fold((1.0f32, 0.0f32), |(lo, hi), v| (lo.min(v), hi.max(v)));
    assert!(
        (0.45 - 1e-3..0.5).contains(&low) && high > 0.99,
        "{low} {high}"
    );
}

#[test]
fn shape_edges_follow_the_merged_shape() {
    // One box: its 12 edges.
    assert_eq!(shape_edges(&[[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]).len(), 12);
    // Two halves of a cube: still 12 edges, no line across the faces where they meet.
    let halves = [
        [0.0, 0.0, 0.0, 1.0, 0.5, 1.0],
        [0.0, 0.5, 0.0, 1.0, 1.0, 1.0],
    ];
    let edges = shape_edges(&halves);
    let total: f64 = edges.iter().map(|s| s.length).sum();
    assert!((total - 12.0).abs() < 1e-6, "{edges:?}");
    assert!(
        edges
            .iter()
            .all(|s| s.start[1] < 0.25 || s.start[1] > 0.75 || s.axis == 1)
    );
    // A stair: an L-shaped section along x has 6 corners, so 6 lines along x and 6 at each
    // end: 18 edges in all.
    let stair = [
        [0.0, 0.0, 0.0, 1.0, 0.5, 1.0],
        [0.0, 0.5, 0.0, 1.0, 1.0, 0.5],
    ];
    let lines = shape_edges(&stair);
    let along_x = lines.iter().filter(|s| s.axis == 0).count();
    assert_eq!(along_x, 6, "{lines:?}");
    let corners: f64 = lines.iter().filter(|s| s.axis != 0).map(|s| s.length).sum();
    assert!(
        (corners - 2.0 * (1.0 + 1.0 + 0.5 + 0.5 + 0.5 + 0.5)).abs() < 1e-6,
        "{lines:?}"
    );
    // The step's inner corner is seen from above in front, not from below behind.
    let step = lines
        .iter()
        .find(|s| s.axis == 0 && (s.start[1] - 0.5).abs() < 1e-9 && (s.start[2] - 0.5).abs() < 1e-9)
        .expect("the step line");
    assert!(step.visible([0.5, 2.0, 2.0]));
    assert!(!step.visible([0.5, -1.0, -1.0]));
    // The single-box path matches the general one, piece for piece.
    for b in [
        [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        [0.25, 0.0, 0.1, 0.75, 0.5, 0.9],
    ] {
        let fast = shape_edges(&[b]);
        // The same box twice takes the general path and gives the same outline.
        let general = shape_edges(&[b, b]);
        assert_eq!(fast.len(), general.len());
        for segment in &general {
            assert!(fast.contains(segment), "{segment:?} not in {fast:?}");
        }
    }
    // A single box's edge is visible exactly as the old cube test said.
    for camera in [[3.0, 0.5, 0.5], [0.5, 3.0, -2.0], [-2.0, -2.0, -2.0]] {
        let lines = shape_edges(&[[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]);
        let seen = lines.iter().filter(|s| s.visible(camera)).count();
        let expected = EDGES
            .iter()
            .filter(|e| e.visible([0.0; 3], [1.0; 3], camera))
            .count();
        assert_eq!(seen, expected, "{camera:?}");
    }
}

#[test]
fn only_cinnabars_own_selection_counts_as_direct() {
    // Undoing the highlight: brightened by 1.3 after darkening by the cancel layer is 1.
    assert!(((1.0 - highlight_cancel_alpha(100)) * GAME_HIGHLIGHT_GAIN - 1.0).abs() < 1e-6);
    assert_eq!(highlight_cancel_alpha(0), 0.0);
    assert!(highlight_cancel_alpha(50) < highlight_cancel_alpha(100));
    assert_eq!(step_highlight_cancel(50, true), 55);
    assert_eq!(step_highlight_cancel(50, false), 45);
    assert_eq!(step_highlight_cancel(100, true), 100);
    assert_eq!(step_highlight_cancel(0, false), 0);
    assert_eq!(step_highlight_cancel(42, false), 40);
    assert_eq!(step_highlight_cancel(42, true), 45);
    assert_eq!(
        Settings::parse(r#"{"hide_game_highlight": true}"#).highlight_cancel,
        50
    );
    assert_eq!(
        Settings::parse(r#"{"highlight_cancel": 70}"#).highlight_cancel,
        70
    );
    // Mobs start from the block look in older files, then keep their own.
    let old = Settings::parse(r##"{"color": "#010203", "width": 5, "fill_color": "#040506"}"##);
    assert_eq!(
        old.mob,
        crate::outline::Style {
            fill_rgb: None,
            ..old.style()
        },
        "mob fill follows its line"
    );
    let mut s = old;
    let mut mobs = s.for_mobs();
    mobs.rgba = [200, 0, 0, 255];
    mobs.width = 2.0;
    mobs.menu_key = KeyName::new("KeyK").unwrap();
    s.store_mob(&mobs);
    assert_eq!(s.style(), old.style(), "the block look is untouched");
    assert_eq!((s.mob.rgba, s.mob.width), ([200, 0, 0, 255], 2.0));
    assert_eq!(
        s.menu_key.as_str(),
        "KeyK",
        "shared settings edited on the mob page stay"
    );
    s.reset();
    assert_eq!(
        s.mob.rgba,
        [200, 0, 0, 255],
        "resetting blocks keeps the mob look"
    );
    let pulsing = Settings::parse(r#"{"pulse": true, "mob_pulse": true}"#);
    assert!(
        pulsing.pulse && !pulsing.mob.pulse,
        "mob hitboxes never pulse"
    );
    let aimed = s.aimed();
    assert_eq!((aimed.rgba, aimed.width), ([255, 60, 60, 255], 2.0));
    assert_eq!(
        Settings::parse(&Settings::default().to_json()),
        Settings::default()
    );
    let wall = |p: [i32; 3]| {
        if p[2] == -3 {
            Cell::Boxes(vec![FULL])
        } else {
            Cell::Boxes(vec![])
        }
    };
    let eye = [0.5, 64.5, 0.5];
    match target_with_actors(eye, [0.0, 0.0, -1.0], 5.7, wall, NO_ACTORS) {
        Some(Target::Block(hit)) => assert!(hit.direct),
        other => panic!("{other:?}"),
    }
    let mob = vec![[0.2, 64.0, -1.8, 0.8, 65.0, -1.2]];
    match target_with_actors(eye, [0.0, 0.0, -1.0], 5.7, wall, &[mob]) {
        Some(Target::Actor(hit)) => assert!(!hit.direct),
        other => panic!("{other:?}"),
    }
    // The bridging support is the mod's own addition; Cinnabar highlights nothing there.
    let floor = |p: [i32; 3]| {
        if p == [0, 63, 0] {
            Cell::Boxes(vec![FULL])
        } else {
            Cell::Boxes(vec![])
        }
    };
    let steep = look_direction(0.0, -1.35);
    assert!(
        !support_hit([0.5, 65.62, 0.3], steep, 5.7, floor)
            .unwrap()
            .direct
    );
}

#[test]
fn faces_match_their_edges() {
    let (min, max) = ([0.0; 3], [1.0; 3]);
    for camera in [
        [0.5, 3.0, 0.5],
        [3.0, 3.0, 3.0],
        [-2.0, 0.5, 0.5],
        [0.5, 0.5, 0.5],
    ] {
        let faces = FACES.iter().filter(|f| f.visible(min, max, camera)).count();
        let edges = EDGES.iter().filter(|e| e.visible(min, max, camera)).count();
        // Visible faces share edges pairwise: 1 face 4 edges, 2 faces 7, 3 faces 9.
        assert_eq!(edges, [0, 4, 7, 9][faces], "{camera:?}");
    }
    for face in FACES {
        let corners = face.unit_corners();
        assert!(
            corners
                .iter()
                .all(|c| c[face.axis] == if face.max_side { 1.0 } else { 0.0 })
        );
    }
}

#[test]
fn edges_are_distinct_and_cover_the_cube() {
    let mut seen = Vec::new();
    for edge in EDGES {
        let [start, end] = edge.unit_points();
        let length: f32 = (0..3).map(|a| (end[a] - start[a]).abs()).sum();
        assert_eq!(length, 1.0);
        assert!(!seen.contains(&(start, end)));
        seen.push((start, end));
    }
}

#[test]
fn only_edges_of_faces_toward_the_camera_are_visible() {
    let (min, max) = ([0.0; 3], [1.0; 3]);
    let count = |camera| EDGES.iter().filter(|e| e.visible(min, max, camera)).count();
    // Straight above: only the top face is visible, so its four edges remain.
    assert_eq!(count([0.5, 3.0, 0.5]), 4);
    // Above and in front (+z): top and front faces, sharing one edge: 4 + 4 - 1.
    assert_eq!(count([0.5, 3.0, 3.0]), 7);
    // Above, in front and to the side: three faces, the usual 3D view: 9 edges, 3 hidden.
    assert_eq!(count([3.0, 3.0, 3.0]), 9);
    // Inside the block, nothing faces the camera.
    assert_eq!(count([0.5, 0.5, 0.5]), 0);
    // The hidden edges are exactly those meeting the far corner (0, 0, 0).
    for edge in EDGES {
        let [start, _] = edge.unit_points();
        assert_eq!(edge.visible(min, max, [3.0; 3]), start != [0.0; 3]);
    }
}

#[test]
fn fields_find_keys_in_any_order_and_ignore_text_values() {
    let json = "{
  \"a\": 1,
 \"b\" :	\"two\",
\"c\":true, \"d\":-0.5e1 ,\"text\": \"\\\"a\\\": x\"}";
    let fields = Fields::new(json);
    assert_eq!(number_field(&fields, "a"), Some(1.0));
    assert_eq!(string_field(&fields, "b"), Some("two"));
    assert_eq!(bool_field(&fields, "c"), Some(true));
    // Out of the order they are written in, and more than once.
    assert_eq!(number_field(&fields, "d"), Some(-5.0));
    assert_eq!(number_field(&fields, "a"), Some(1.0));
    assert_eq!(bool_field(&fields, "c"), Some(true));
    // A missing key stays missing and leaves the others findable.
    assert_eq!(number_field(&fields, "missing"), None);
    assert_eq!(string_field(&fields, "b"), Some("two"));
    // A key's name inside a text value is not a key.
    let fields = Fields::new("{\"note\": \"color\", \"color\": \"#102030\"}");
    assert_eq!(string_field(&fields, "color"), Some("#102030"));
    // Wrong kinds and damaged text give nothing, never a panic.
    assert_eq!(bool_field(&fields, "color"), None);
    for broken in [
        "",
        "{",
        "{\"a",
        "{\"a\"",
        "{\"a\":",
        "\"",
        "{\"a\": \"x",
        "{\"é\": 1, \"b\": é}",
    ] {
        let fields = Fields::new(broken);
        assert_eq!(number_field(&fields, "a"), None);
        let _ = Settings::parse(broken);
    }
}

#[test]
fn a_big_settings_file_reads_every_field() {
    // Every field after a long log, in reverse order of writing too: the lookup cursor must
    // not lose any of them.
    let mut s = Settings {
        width: 7.0,
        reach: 6.1,
        ..Settings::default()
    };
    s.zoom.level = 6.0;
    let json = s.to_json();
    assert_eq!(Settings::parse(&json), s);
    let mut lines: Vec<&str> = json.lines().collect();
    lines.reverse();
    let reversed = format!(
        "{{
{}
}}",
        lines
            .join(
                "
"
            )
            .replace("{", "")
            .replace("}", "")
    );
    assert_eq!(Settings::parse(&reversed), s);
    let padded = format!(
        "{{\"jump_reset_resets\": \"{}\",
{}",
        "9|1/".repeat(4000),
        &json[1..]
    );
    assert_eq!(Settings::parse(&padded), s);
    assert_eq!(
        text_field(&Fields::new(&padded), "jump_reset_resets").map(str::len),
        Some(16000)
    );
}

#[test]
pub(super) fn find_quote_agrees_with_a_plain_search() {
    let text: Vec<u8> = (0..300u32)
        .map(|n| match n % 11 {
            3 | 7 => b'"',
            _ => b'a' + (n % 26) as u8,
        })
        .collect();
    for from in 0..=text.len() + 1 {
        let plain = text
            .get(from..)
            .and_then(|rest| rest.iter().position(|&byte| byte == b'"'))
            .map(|offset| from + offset);
        assert_eq!(find_quote(&text, from), plain, "from {from}");
    }
    // Bytes that look like a quote's neighbours in the bit trick.
    let tricky = [
        0x22u8 ^ 1,
        0x22 ^ 0x80,
        0x23,
        0x21,
        0x22,
        0xA2,
        0x22,
        0x22,
        0x22,
    ];
    for from in 0..tricky.len() {
        let plain = tricky[from..]
            .iter()
            .position(|&byte| byte == b'"')
            .map(|o| from + o);
        assert_eq!(find_quote(&tricky, from), plain);
    }
}
