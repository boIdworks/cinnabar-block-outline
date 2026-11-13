//! The terrain lookup and targeting glue, tested natively: none of it calls the host.

use super::*;

const UNIT: [f64; 6] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
const SLAB: [f64; 6] = [0.0, 0.0, 0.0, 1.0, 0.5, 1.0];

fn stamp(dimension: i32) -> Stamp {
    Stamp {
        session: 1,
        dimension,
        registry: 1,
        revision: 1,
    }
}

fn entry(selection: Selection, empty: bool) -> PaletteEntry {
    PaletteEntry { selection, empty }
}

/// Shape 0 is the unit cube, shape 1 a slab and a post, shape 2 is empty.
fn shapes() -> (Vec<u32>, Vec<f64>) {
    let post = [0.4, 0.0, 0.4, 0.6, 1.0, 0.6];
    let boxes = [UNIT, SLAB, post].concat();
    (vec![0, 1, 3, 3], boxes)
}

fn blocks(states: Vec<u16>, palette: Vec<PaletteEntry>) -> Blocks {
    let (shape_offsets, shape_boxes) = shapes();
    let words = states.len().div_ceil(64);
    Blocks {
        loaded: vec![u64::MAX; words],
        states,
        palette,
        overrides: Vec::new(),
        shape_offsets,
        shape_boxes,
    }
}

fn sample(min: [i32; 3], size: [i32; 3], blocks: Blocks) -> Sample {
    Sample {
        frame_session: 7,
        min,
        size,
        stamp: stamp(0),
        blocks,
    }
}

fn boxes(cell: Cell) -> Option<Vec<[f64; 6]>> {
    match cell {
        Cell::Boxes(list) => Some(list),
        Cell::Unknown => None,
    }
}

#[test]
fn only_air_liquids_and_nameless_blocks_are_empty() {
    for id in [
        "",
        "minecraft:air",
        "minecraft:cave_air",
        "minecraft:void_air",
        "minecraft:structure_void",
        "minecraft:water",
        "minecraft:flowing_water",
        "minecraft:lava",
        "minecraft:flowing_lava",
        "minecraft:light_block",
        "minecraft:light_block_7",
    ] {
        assert!(empty_block(id), "{id} has no outline");
    }
    for id in [
        "minecraft:stone",
        "minecraft:oak_stairs",
        "minecraft:waterlogged",
        "minecraft:water_cauldron",
        "hive:custom_block",
    ] {
        assert!(!empty_block(id), "{id} is a solid block");
    }
}

#[test]
fn bounds_cover_every_box_and_nothing_gives_none() {
    assert_eq!(bounds_of(&[]), None);
    assert_eq!(bounds_of(&[UNIT]), Some(([0.0; 3], [1.0; 3])));
    let far = [3.0, -1.0, 2.0, 4.0, 0.5, 5.0];
    assert_eq!(
        bounds_of(&[UNIT, far]),
        Some(([0.0, -1.0, 0.0], [4.0, 1.0, 5.0]))
    );
}

#[test]
fn a_shape_is_the_boxes_between_its_offsets() {
    let all = blocks(Vec::new(), Vec::new());
    assert_eq!(shape_boxes(&all, 0), vec![UNIT]);
    assert_eq!(
        shape_boxes(&all, 1),
        vec![SLAB, [0.4, 0.0, 0.4, 0.6, 1.0, 0.6]]
    );
    assert!(shape_boxes(&all, 2).is_empty(), "shape 2 is known-empty");
    assert!(shape_boxes(&all, 3).is_empty(), "past the last offset");
    assert!(shape_boxes(&all, u32::MAX).is_empty());
}

#[test]
fn a_shape_whose_data_is_cut_short_loses_only_the_missing_boxes() {
    let mut all = blocks(Vec::new(), Vec::new());
    all.shape_boxes.truncate(9); // the unit cube and half of the slab
    assert_eq!(shape_boxes(&all, 0), vec![UNIT]);
    assert!(shape_boxes(&all, 1).is_empty());
}

#[test]
fn cells_outside_the_sampled_region_are_unknown() {
    let region = sample(
        [10, 60, -5],
        [2, 3, 4],
        blocks(vec![0; 24], vec![entry(Selection::Fixed(Some(0)), false)]),
    );
    for outside in [
        [9, 60, -5],
        [12, 60, -5],
        [10, 59, -5],
        [10, 63, -5],
        [10, 60, -6],
        [10, 60, -1],
    ] {
        assert!(boxes(cell_at(&region, outside)).is_none(), "{outside:?}");
    }
    for inside in [[10, 60, -5], [11, 62, -2]] {
        assert!(boxes(cell_at(&region, inside)).is_some(), "{inside:?}");
    }
}

/// The region's cells run x, then z, then y: the order the host sends them in.
#[test]
fn each_block_reads_its_own_cell_in_x_then_z_then_y_order() {
    let (size_x, size_y, size_z) = (3, 4, 5);
    let mut states = Vec::new();
    for x in 0..size_x {
        for z in 0..size_z {
            for y in 0..size_y {
                // A palette entry per block, so a block reading the wrong cell is caught.
                states.push((x * 100 + y * 10 + z) as u16);
            }
        }
    }
    let palette = (0..400)
        .map(|n| {
            // Shape 0 for even entries, shape 1 for odd ones.
            entry(Selection::Fixed(Some(n % 2)), false)
        })
        .collect();
    let region = sample(
        [-2, 64, 30],
        [size_x, size_y, size_z],
        blocks(states, palette),
    );
    for x in 0..size_x {
        for y in 0..size_y {
            for z in 0..size_z {
                let entry = (x * 100 + y * 10 + z) as u32;
                let want = if entry.is_multiple_of(2) {
                    vec![UNIT]
                } else {
                    shape_boxes(&region.blocks, 1)
                };
                let got = boxes(cell_at(&region, [-2 + x, 64 + y, 30 + z]));
                assert_eq!(got, Some(want), "block {x},{y},{z}");
            }
        }
    }
}

#[test]
fn a_cell_the_host_has_not_loaded_is_unknown() {
    let mut all = blocks(vec![0; 70], vec![entry(Selection::Fixed(Some(0)), false)]);
    all.loaded = vec![u64::MAX, !(1u64 << 3)]; // cell 67 is not loaded
    let region = sample([0, 0, 0], [70, 1, 1], all);
    assert!(boxes(cell_at(&region, [66, 0, 0])).is_some());
    assert!(
        boxes(cell_at(&region, [67, 0, 0])).is_none(),
        "its bit is clear"
    );
    assert!(boxes(cell_at(&region, [68, 0, 0])).is_some());
    let mut short = blocks(vec![0; 70], vec![entry(Selection::Fixed(Some(0)), false)]);
    short.loaded = vec![u64::MAX];
    let region = sample([0, 0, 0], [70, 1, 1], short);
    assert!(
        boxes(cell_at(&region, [64, 0, 0])).is_none(),
        "no word for it at all"
    );
}

#[test]
fn a_cell_with_no_layer_or_no_palette_entry_is_handled() {
    let region = sample(
        [0, 0, 0],
        [1, 1, 3],
        blocks(
            vec![NO_LAYER, 0, 9],
            vec![entry(Selection::Fixed(Some(0)), false)],
        ),
    );
    assert_eq!(
        boxes(cell_at(&region, [0, 0, 0])),
        Some(Vec::new()),
        "no block layer"
    );
    assert_eq!(boxes(cell_at(&region, [0, 0, 1])), Some(vec![UNIT]));
    assert!(
        boxes(cell_at(&region, [0, 0, 2])).is_none(),
        "palette entry 9 does not exist"
    );
}

#[test]
fn a_block_without_shape_data_is_a_full_block_unless_it_is_empty() {
    let palette = vec![
        entry(Selection::Fixed(None), false),
        entry(Selection::Fixed(None), true),
        entry(Selection::NotRequested, false),
        entry(Selection::NotRequested, true),
    ];
    let region = sample([0, 0, 0], [1, 1, 4], blocks(vec![0, 1, 2, 3], palette));
    let full = vec![[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]];
    assert_eq!(boxes(cell_at(&region, [0, 0, 0])), Some(full.clone()));
    assert_eq!(
        boxes(cell_at(&region, [0, 0, 1])),
        Some(Vec::new()),
        "air or liquid"
    );
    assert_eq!(boxes(cell_at(&region, [0, 0, 2])), Some(full));
    assert_eq!(boxes(cell_at(&region, [0, 0, 3])), Some(Vec::new()));
}

#[test]
fn per_cell_shapes_come_from_the_overrides_sorted_by_slot() {
    let mut all = blocks(vec![0, 0, 0], vec![entry(Selection::PerCell, false)]);
    all.overrides = vec![(0, Some(1)), (2, None)];
    let region = sample([0, 0, 0], [1, 1, 3], all);
    assert_eq!(
        boxes(cell_at(&region, [0, 0, 0])),
        Some(shape_boxes(&region.blocks, 1)),
        "slot 0 has its own shape"
    );
    assert_eq!(
        boxes(cell_at(&region, [0, 0, 1])),
        Some(vec![[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]),
        "slot 1 has no override: a full block"
    );
    assert_eq!(
        boxes(cell_at(&region, [0, 0, 2])),
        Some(vec![[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]),
        "slot 2's override says unavailable"
    );
}

#[test]
fn a_resolved_shape_is_known_only_with_an_index() {
    assert_eq!(known(&ResolvedShape::Known(4)), Some(4));
    assert_eq!(known(&ResolvedShape::Unavailable), None);
}

/// A world of air with one block at (0, 0, 3), sampled around the eye at (0.5, 0.5, 0.5).
fn world_with_a_block_ahead() -> Sample {
    let mut states = vec![NO_LAYER; 512];
    let cell = ((4 * 8) + 7) * 8 + 4; // block (0, 0, 3) of the region at (-4, -4, -4)
    states[cell] = 0;
    sample(
        [-4, -4, -4],
        [8, 8, 8],
        blocks(states, vec![entry(Selection::Fixed(Some(0)), false)]),
    )
}

fn set_sample(sample: Sample) {
    SAMPLE.with(|slot| *slot.borrow_mut() = Some(sample));
}

const LOOK_AHEAD: [[f64; 3]; 2] = [[0.5, 0.5, 0.5], [0.0, 0.0, 1.0]];

#[test]
fn the_crosshair_targets_the_block_ahead() {
    set_sample(world_with_a_block_ahead());
    let (hit, actor) = aimed_target(7, 0, LOOK_AHEAD, 5.0, None, &[]).expect("a block is ahead");
    assert!(!actor);
    assert_eq!(hit.block, [0, 0, 3]);
    assert!((hit.distance - 2.5).abs() < 1e-9, "{}", hit.distance);
    assert!(
        hit.direct,
        "a block under the look ray is Cinnabar's own selection"
    );
}

#[test]
fn nothing_is_targeted_past_the_reach() {
    set_sample(world_with_a_block_ahead());
    assert!(aimed_target(7, 0, LOOK_AHEAD, 2.0, None, &[]).is_none());
}

#[test]
fn a_sample_of_another_session_or_dimension_is_never_used() {
    set_sample(world_with_a_block_ahead());
    assert!(
        aimed_target(8, 0, LOOK_AHEAD, 5.0, None, &[]).is_none(),
        "after a reconnect the old sample describes another world"
    );
    assert!(
        aimed_target(7, 1, LOOK_AHEAD, 5.0, None, &[]).is_none(),
        "after a dimension change too"
    );
}

#[test]
fn with_no_sample_there_is_no_target() {
    SAMPLE.with(|slot| *slot.borrow_mut() = None);
    assert!(aimed_target(7, 0, LOOK_AHEAD, 5.0, None, &[]).is_none());
}

#[test]
fn a_mob_in_front_of_the_block_is_targeted_instead() {
    set_sample(world_with_a_block_ahead());
    let mob: outline::ActorBox = [[0.0, 0.0, 1.5, 1.0, 2.0, 2.0]];
    let (hit, actor) = aimed_target(7, 0, LOOK_AHEAD, 5.0, None, &[mob]).expect("a target");
    assert!(actor, "the mob is nearer than the block");
    assert_eq!((hit.min, hit.max), ([0.0, 0.0, 1.5], [1.0, 2.0, 2.0]));
}

#[test]
fn when_cinnabar_names_the_aimed_mob_that_one_is_the_target_wherever_it_is() {
    set_sample(world_with_a_block_ahead());
    let aimed = [
        [2.0, 0.0, 2.0, 3.0, 2.0, 3.0],
        [2.5, 2.0, 2.0, 2.8, 2.5, 3.0],
    ];
    let (hit, actor) =
        aimed_target(7, 0, LOOK_AHEAD, 5.0, Some(Some(&aimed)), &[]).expect("a target");
    assert!(actor);
    assert_eq!((hit.min, hit.max), ([2.0, 0.0, 2.0], [3.0, 2.5, 3.0]));
}

#[test]
fn when_cinnabar_says_no_mob_is_aimed_only_blocks_count() {
    set_sample(world_with_a_block_ahead());
    let mob: outline::ActorBox = [[0.0, 0.0, 1.5, 1.0, 2.0, 2.0]];
    let (hit, actor) = aimed_target(7, 0, LOOK_AHEAD, 5.0, Some(None), &[mob]).expect("a block");
    assert!(
        !actor,
        "a nearby mob is ignored when the game says it would not be hit"
    );
    assert_eq!(hit.block, [0, 0, 3]);
}

#[test]
fn a_single_box_outline_is_just_its_edges() {
    let one = outline::Boxes::single([0.0; 3], [1.0; 3]);
    assert_eq!(segments_for(&one), outline::shape_edges(&[UNIT]));
}

#[test]
fn a_merged_outline_is_the_same_every_time_it_is_asked_for() {
    let stair = outline::Boxes::new(&[SLAB, [0.0, 0.5, 0.0, 1.0, 1.0, 0.5]], [0.0; 3], [1.0; 3]);
    let first = segments_for(&stair);
    assert_eq!(first, outline::shape_edges(stair.as_slice()));
    assert_eq!(segments_for(&stair), first, "served from the cache");
    let post = outline::Boxes::new(&[[0.4, 0.0, 0.4, 0.6, 1.0, 0.6], UNIT], [0.0; 3], [1.0; 3]);
    assert_eq!(
        segments_for(&post),
        outline::shape_edges(post.as_slice()),
        "a different shape replaces the cached one"
    );
}

#[test]
fn actors_in_range_counts_the_nearest_run_within_range() {
    let eye = [0.0, 0.0, 0.0];
    let at = |x: f32| (x, 0.0, 0.0);
    // Nearest first: the first three are within 10 blocks, the fourth is not.
    let list = [at(1.0), at(5.0), at(10.0), at(11.0), at(2.0)];
    assert_eq!(actors_in_range(list.into_iter(), eye, 10.0), 3);
    assert_eq!(actors_in_range([].into_iter(), eye, 10.0), 0);
    // Distance counts in all three axes.
    assert_eq!(actors_in_range([(6.0, 6.0, 6.0)].into_iter(), eye, 10.0), 0);
    assert_eq!(actors_in_range([(5.0, 5.0, 5.0)].into_iter(), eye, 10.0), 1);
}

#[test]
fn the_next_actor_ask_follows_what_was_in_range_with_a_few_to_spare() {
    // Two of twelve in range: those, and the spare.
    assert_eq!(next_actor_ask(2, 12, 12), 2 + ACTOR_SPARE as u32);
    // None in range still asks for a few, so one coming near is seen.
    assert_eq!(next_actor_ask(0, 12, 12), ACTOR_SPARE as u32);
    // Every one read was in range: there may be more, so the most.
    assert_eq!(next_actor_ask(12, 12, 12), 12);
    assert_eq!(next_actor_ask(3, 3, 12), 12);
    assert_eq!(next_actor_ask(0, 0, 12), 12);
    // Never above the most, never below one.
    assert_eq!(next_actor_ask(11, 12, 12), 12);
    assert_eq!(next_actor_ask(usize::MAX - 1, usize::MAX, 12), 12);
    assert!(next_actor_ask(0, 5, 1) >= 1);
}

#[test]
fn arrows_are_separate_from_melee_targets_even_when_not_pickable() {
    let mut arrow = player_state::Actor {
        session: 1,
        dimension: 0,
        runtime_id: 42,
        player: false,
        position: (1.0, 2.0, 3.0),
        name: String::new(),
        identifier: "minecraft:arrow".into(),
        yaw: 0.0,
        pitch: 0.0,
        pickable: false,
        hittable: Some(false),
        look_origin: None,
        look_direction: None,
        invisible: false,
        sneaking: false,
        hurt: false,
        health: None,
        boxes: vec![player_state::Bounds {
            min: (1.0, 2.0, 3.0),
            max: (1.5, 2.5, 3.5),
        }],
    };
    assert_eq!(
        projectile_boxes([&arrow].into_iter()),
        vec![[[1.0, 2.0, 3.0, 1.5, 2.5, 3.5]]]
    );
    assert!(boxes_of([&arrow].into_iter(), &|_| false).0.is_empty());
    arrow.pickable = true;
    assert!(boxes_of([&arrow].into_iter(), &|_| false).0.is_empty());
    arrow.invisible = true;
    assert!(projectile_boxes([&arrow].into_iter()).is_empty());
}

#[test]
fn arrow_shapes_use_their_own_colour_and_extended_range() {
    let settings = Settings {
        arrow_rgb: [12, 34, 56],
        mob_on: true,
        ..Settings::default()
    };
    let target = Target {
        hit: None,
        mobs: Vec::new(),
        arrows: vec![[[30.0, 0.0, 0.0], [31.0, 1.0, 1.0]]],
        dimension: 0,
        camera: [0.0; 3],
    };
    let result = super::shapes(&target, &settings);
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].settings.rgba[..3], &[12, 34, 56]);
    let actor = [[[30.0, 0.0, 0.0, 31.0, 1.0, 1.0]]];
    assert_eq!(
        outline::nearest_mob_boxes(&actor, [0.0; 3], None, ARROW_RANGE, MAX_MOB_BOXES).len(),
        1
    );
    assert!(
        outline::nearest_mob_boxes(&actor, [0.0; 3], None, MOB_RANGE, MAX_MOB_BOXES).is_empty()
    );
}

#[test]
fn bow_extends_only_players_and_returns_to_normal_when_put_away() {
    let nearby_mob = [[4.0, 0.0, 0.0, 5.0, 1.0, 1.0]];
    let distant_player = [[30.0, 0.0, 0.0, 31.0, 1.0, 1.0]];
    let distant_mob = [[32.0, 0.0, 0.0, 33.0, 1.0, 1.0]];
    let out_of_range_player = [[60.0, 0.0, 0.0, 61.0, 1.0, 1.0]];
    let actors = [nearby_mob, distant_player, distant_mob, out_of_range_player];
    let players = [false, true, false, true];
    let normal = nearest_boxes(&actors, &players, [0.0; 3], None, false);
    assert_eq!(normal.len(), 1);
    let bow = nearest_boxes(&actors, &players, [0.0; 3], None, true);
    assert_eq!(bow.len(), 2);
    assert_eq!(bow[1][0], [30.0, 0.0, 0.0]);
    assert_eq!(
        nearest_boxes(&actors, &players, [0.0; 3], None, false),
        normal
    );
}
