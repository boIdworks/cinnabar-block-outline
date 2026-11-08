//! Targeting: the block and mob raycast, bridging support, shape edges and line widths.

/// Whether any part of the box `min`..`max` can be in front of the viewer at `eye` looking along
/// `dir` (a unit vector): the box's furthest corner along `dir` is past the plane through the eye.
/// A box wholly behind that plane cannot be on screen, so it need not be drawn. Boxes that straddle
/// the plane, or come out as nonsense, are kept.
pub fn in_front_of(min: [f64; 3], max: [f64; 3], eye: [f64; 3], dir: [f64; 3]) -> bool {
    let furthest: f64 = (0..3)
        .map(|axis| {
            let corner = if dir[axis] >= 0.0 {
                max[axis]
            } else {
                min[axis]
            };
            (corner - eye[axis]) * dir[axis]
        })
        .sum();
    // A little slack, so a box right at the plane is never dropped for rounding.
    !furthest.is_finite() || furthest > -0.05
}

/// View direction for actor YXZ radians: yaw turns left, pitch up, forward is -Z.
pub fn look_direction(yaw: f32, pitch: f32) -> [f64; 3] {
    let (yaw, pitch) = (f64::from(yaw), f64::from(pitch));
    [
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    ]
}

/// What the terrain reports at one block position.
pub enum Cell {
    /// Not loaded or outside the sampled region; the ray stops without a target.
    Unknown,
    /// Block-local selection boxes as `[min x, min y, min z, max x, max y, max z]`.
    Boxes(Vec<[f64; 6]>),
}

/// The targeted block and the world-space bounds of its selection shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub block: [i32; 3],
    pub min: [f64; 3],
    pub max: [f64; 3],
    /// Distance along the look ray to where it enters the shape.
    pub distance: f64,
    /// Whether this is Cinnabar's own selection, which it outlines or highlights itself: a block
    /// hit by the look ray. The bridging support block and actors are not.
    pub direct: bool,
    /// The shape's boxes in world space (just the bounds when there are too many).
    pub boxes: Boxes,
}

/// Shapes with more boxes than this are outlined by their bounds: merging their outlines would
/// cost too much of a frame's fuel.
pub const MAX_SHAPE_BOXES: usize = 4;

/// A block's selection boxes in world space, `[min x, min y, min z, max x, max y, max z]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boxes {
    boxes: [[f64; 6]; MAX_SHAPE_BOXES],
    len: u8,
}

impl Boxes {
    pub fn single(min: [f64; 3], max: [f64; 3]) -> Self {
        let mut boxes = [[0.0; 6]; MAX_SHAPE_BOXES];
        boxes[0] = [min[0], min[1], min[2], max[0], max[1], max[2]];
        Self { boxes, len: 1 }
    }

    /// The boxes, or just `bounds` when there are more than `MAX_SHAPE_BOXES`.
    pub fn new(list: &[[f64; 6]], min: [f64; 3], max: [f64; 3]) -> Self {
        if list.is_empty() || list.len() > MAX_SHAPE_BOXES {
            return Self::single(min, max);
        }
        let mut boxes = [[0.0; 6]; MAX_SHAPE_BOXES];
        boxes[..list.len()].copy_from_slice(list);
        Self {
            boxes,
            len: list.len() as u8,
        }
    }

    pub fn as_slice(&self) -> &[[f64; 6]] {
        &self.boxes[..usize::from(self.len)]
    }

    /// Every box grown by `by` on all sides.
    pub fn inflated(&self, by: f64) -> Self {
        let mut grown = *self;
        for b in &mut grown.boxes[..usize::from(self.len)] {
            for axis in 0..3 {
                b[axis] -= by;
                b[axis + 3] += by;
            }
        }
        grown
    }
}

/// One straight piece of a shape's outline, along `axis` from `start`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub start: [f64; 3],
    pub axis: usize,
    pub length: f64,
    /// Which of the four quarters around the line are inside the shape: bit `a + 2 * b`, where
    /// `a` and `b` are 1 on the high side of the first and second other axis.
    quarters: u8,
}

impl Segment {
    pub fn end(&self) -> [f64; 3] {
        let mut end = self.start;
        end[self.axis] += self.length;
        end
    }

    /// Whether a face meeting at this line faces `camera`, so the line is on the visible side
    /// of the shape.
    pub fn visible(&self, camera: [f64; 3]) -> bool {
        let [u, v] = Edge {
            axis: self.axis,
            sides: [false, false],
        }
        .others();
        let inside = |a: u8, b: u8| self.quarters >> (a + 2 * b) & 1 == 1;
        let ahead = |axis: usize, high: bool| {
            let offset = camera[axis] - self.start[axis];
            if high { offset > 1e-9 } else { offset < -1e-9 }
        };
        // A face between two quarters faces away from the inside one.
        (0..2).any(|b| inside(0, b) != inside(1, b) && ahead(u, inside(0, b)))
            || (0..2).any(|a| inside(a, 0) != inside(a, 1) && ahead(v, inside(a, 0)))
    }
}

/// The outline of the union of `boxes`: every box edge, split where other boxes start or end,
/// keeping only the pieces that are real corners of the merged shape (not lines across a flat
/// face or inside it). A single box gives its 12 edges.
pub fn shape_edges(boxes: &[[f64; 6]]) -> Vec<Segment> {
    const EPS: f64 = 1e-4;
    // One box (most blocks, every mob): its own 12 edges, each with the box in one quarter.
    if let [b] = boxes {
        return EDGES
            .iter()
            .filter(|edge| b[edge.axis + 3] - b[edge.axis] >= 0.01)
            .map(|edge| {
                let [u, v] = edge.others();
                let mut start = [0.0; 3];
                start[edge.axis] = b[edge.axis];
                start[u] = if edge.sides[0] { b[u + 3] } else { b[u] };
                start[v] = if edge.sides[1] { b[v + 3] } else { b[v] };
                // The box lies on the low side of an edge on its high side, and the other way.
                let quarter = u8::from(!edge.sides[0]) + 2 * u8::from(!edge.sides[1]);
                Segment {
                    start,
                    axis: edge.axis,
                    length: b[edge.axis + 3] - b[edge.axis],
                    quarters: 1 << quarter,
                }
            })
            .collect();
    }
    // Reused for every edge rather than allocated for each.
    let mut cuts: Vec<f64> = Vec::with_capacity(2 * boxes.len() + 2);
    let inside = |p: [f64; 3]| {
        boxes
            .iter()
            .any(|b| (0..3).all(|axis| b[axis] < p[axis] && p[axis] < b[axis + 3]))
    };
    let mut segments: Vec<Segment> = Vec::new();
    for b in boxes {
        for edge in EDGES {
            let axis = edge.axis;
            let [u, v] = edge.others();
            let mut start = [0.0; 3];
            start[u] = if edge.sides[0] { b[u + 3] } else { b[u] };
            start[v] = if edge.sides[1] { b[v + 3] } else { b[v] };
            let (lo, hi) = (b[axis], b[axis + 3]);
            // Where other boxes begin or end along this edge.
            cuts.clear();
            cuts.extend([lo, hi]);
            for other in boxes {
                for cut in [other[axis], other[axis + 3]] {
                    if cut > lo + EPS && cut < hi - EPS {
                        cuts.push(cut);
                    }
                }
            }
            cuts.sort_unstable_by(f64::total_cmp);
            cuts.dedup_by(|a, b| (*a - *b).abs() < EPS);
            for pair in cuts.windows(2) {
                let (from, to) = (pair[0], pair[1]);
                if to - from < 0.01 {
                    continue;
                }
                let mut probe = start;
                probe[axis] = (from + to) / 2.0;
                let mut quarters = 0u8;
                for (bit, (du, dv)) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
                    .into_iter()
                    .enumerate()
                {
                    let mut p = probe;
                    p[u] += du * EPS * 10.0;
                    p[v] += dv * EPS * 10.0;
                    if inside(p) {
                        quarters |= 1 << bit;
                    }
                }
                // A corner of the merged shape: one or three quarters inside, or two diagonal.
                let real = matches!(quarters.count_ones(), 1 | 3)
                    || quarters == 0b1001
                    || quarters == 0b0110;
                if !real {
                    continue;
                }
                let mut from_point = start;
                from_point[axis] = from;
                let segment = Segment {
                    start: from_point,
                    axis,
                    length: to - from,
                    quarters,
                };
                let same = |s: &Segment| {
                    s.axis == axis
                        && (0..3).all(|i| (s.start[i] - segment.start[i]).abs() < EPS)
                        && (s.length - segment.length).abs() < EPS
                };
                if !segments.iter().any(same) {
                    segments.push(segment);
                }
            }
        }
    }
    segments
}

/// Walks blocks along the ray (Amanatides–Woo) and returns the first selection box hit within
/// `reach`. Boxes containing the origin are ignored, so a block around the eye is not targeted.
pub fn raycast(
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    mut lookup: impl FnMut([i32; 3]) -> Cell,
) -> Option<Hit> {
    let mut block = origin.map(|v| v.floor() as i32);
    let mut step = [0i32; 3];
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];
    for axis in 0..3 {
        if dir[axis] > 0.0 {
            step[axis] = 1;
            t_max[axis] = (f64::from(block[axis]) + 1.0 - origin[axis]) / dir[axis];
            t_delta[axis] = 1.0 / dir[axis];
        } else if dir[axis] < 0.0 {
            step[axis] = -1;
            t_max[axis] = (f64::from(block[axis]) - origin[axis]) / dir[axis];
            t_delta[axis] = -1.0 / dir[axis];
        }
    }
    // Enough steps to cross `reach` diagonally, plus slack.
    let max_steps = (reach.ceil() as usize) * 3 + 3;
    for _ in 0..max_steps {
        match lookup(block) {
            Cell::Unknown => return None,
            Cell::Boxes(boxes) if !boxes.is_empty() => {
                let base = block.map(f64::from);
                let world = |b: &[f64; 6]| {
                    (
                        [base[0] + b[0], base[1] + b[1], base[2] + b[2]],
                        [base[0] + b[3], base[1] + b[4], base[2] + b[5]],
                    )
                };
                let hit = boxes
                    .iter()
                    .filter_map(|b| {
                        let (min, max) = world(b);
                        ray_box(origin, dir, min, max).filter(|t| *t <= reach)
                    })
                    .reduce(f64::min);
                if let Some(distance) = hit {
                    let mut min = [f64::INFINITY; 3];
                    let mut max = [f64::NEG_INFINITY; 3];
                    for b in &boxes {
                        let (lo, hi) = world(b);
                        for axis in 0..3 {
                            min[axis] = min[axis].min(lo[axis]);
                            max[axis] = max[axis].max(hi[axis]);
                        }
                    }
                    let world_boxes: Vec<[f64; 6]> = boxes
                        .iter()
                        .map(|b| {
                            let (lo, hi) = world(b);
                            [lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]
                        })
                        .collect();
                    let boxes = Boxes::new(&world_boxes, min, max);
                    return Some(Hit {
                        block,
                        min,
                        max,
                        distance,
                        direct: true,
                        boxes,
                    });
                }
            }
            Cell::Boxes(_) => {}
        }
        let axis = if t_max[0] < t_max[1] {
            if t_max[0] < t_max[2] { 0 } else { 2 }
        } else if t_max[1] < t_max[2] {
            1
        } else {
            2
        };
        if t_max[axis] > reach {
            return None;
        }
        block[axis] += step[axis];
        t_max[axis] += t_delta[axis];
    }
    None
}

/// The block Cinnabar targets: the look ray's first selection hit within `reach`, kept only if
/// the block's centre is also within `reach` of the eye (`PickResult::trimmed`); otherwise
/// nothing is targeted and the bridging fallback is not tried. After a clear miss, the
/// bridging support block, under the same centre rule.
pub fn target(
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    mut lookup: impl FnMut([i32; 3]) -> Cell,
) -> Option<Hit> {
    match target_with_actors(origin, dir, reach, &mut lookup, NO_ACTORS) {
        Some(Target::Block(hit)) => Some(hit),
        _ => None,
    }
}

/// What the crosshair picks: a block, or an actor's hitbox standing in front of it.
#[derive(Debug, PartialEq)]
pub enum Target {
    Block(Hit),
    /// The actor's hitbox union; `block` is unused.
    Actor(Hit),
}

/// One actor's hitbox: the single box around all of it, `[min x, min y, min z, max x, max y,
/// max z]`. A fixed array, because a list per actor per frame costs allocations the frame's fuel
/// cannot spare.
pub type ActorBox = [[f64; 6]; 1];

/// No actors, typed.
pub const NO_ACTORS: &[ActorBox] = &[];

/// The most hitboxes [`nearest_mob_boxes`] can return.
pub const MOB_BOX_CAP: usize = 16;

/// The hitboxes within `range` blocks of `eye`, nearest first (by the distance to the nearest
/// point of the box), at most `limit` (and [`MOB_BOX_CAP`]). With a view direction the ones wholly
/// behind the viewer are left out ([`in_front_of`]). Picks in one pass into a small fixed buffer:
/// no list of every candidate, no sort.
pub fn nearest_mob_boxes<A: AsRef<[[f64; 6]]>>(
    actors: &[A],
    eye: [f64; 3],
    view: Option<[f64; 3]>,
    range: f64,
    limit: usize,
) -> Vec<[[f64; 3]; 2]> {
    let limit = limit.min(MOB_BOX_CAP);
    let mut best = [(0.0f64, [[0.0f64; 3]; 2]); MOB_BOX_CAP];
    let mut count = 0;
    for b in actors.iter().flat_map(|boxes| boxes.as_ref()) {
        let (min, max) = ([b[0], b[1], b[2]], [b[3], b[4], b[5]]);
        // Distance from the eye to the nearest point of the box.
        let gap: f64 = (0..3)
            .map(|axis| {
                (min[axis] - eye[axis])
                    .max(eye[axis] - max[axis])
                    .max(0.0)
                    .powi(2)
            })
            .sum();
        if gap > range * range || !view.is_none_or(|dir| in_front_of(min, max, eye, dir)) {
            continue;
        }
        // Where it goes among the nearest so far: after every one at least as near, so equal
        // distances keep the order they came in.
        let at = best[..count].partition_point(|(other, _)| *other <= gap);
        if at >= limit {
            continue;
        }
        let end = count.min(limit - 1);
        best.copy_within(at..end, at + 1);
        best[at] = (gap, [min, max]);
        count = (count + 1).min(limit);
    }
    best[..count].iter().map(|(_, b)| *b).collect()
}

/// Padding Cinnabar adds to actor hitboxes when picking (`DEFAULT_ACTOR_PICK_RADIUS`).
pub const ACTOR_PICK_RADIUS: f64 = 0.1;

/// Cinnabar's crosshair choice between actors and blocks (`melee::target::classify`): the
/// nearest actor hit within `reach` wins when it is more than `ACTOR_PICK_RADIUS` in front of
/// the block, and then nothing behind it is outlined, bridging support included. Each actor is
/// a list of world-space boxes; boxes around the eye (the player's own body) are skipped.
pub fn target_with_actors<A: AsRef<[[f64; 6]]>>(
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    mut lookup: impl FnMut([i32; 3]) -> Cell,
    actors: &[A],
) -> Option<Target> {
    let actor = actors
        .iter()
        .filter_map(|boxes| {
            let boxes = boxes.as_ref();
            let padded = |b: &[f64; 6]| {
                let r = ACTOR_PICK_RADIUS;
                (
                    [b[0] - r, b[1] - r, b[2] - r],
                    [b[3] + r, b[4] + r, b[5] + r],
                )
            };
            if boxes.iter().any(|b| {
                let (min, max) = padded(b);
                (0..3).all(|axis| min[axis] <= origin[axis] && origin[axis] <= max[axis])
            }) {
                return None;
            }
            let distance = boxes
                .iter()
                .filter_map(|b| {
                    let (min, max) = padded(b);
                    ray_box(origin, dir, min, max).filter(|t| *t <= reach)
                })
                .reduce(f64::min)?;
            let mut min = [f64::INFINITY; 3];
            let mut max = [f64::NEG_INFINITY; 3];
            for b in boxes {
                for axis in 0..3 {
                    min[axis] = min[axis].min(b[axis]);
                    max[axis] = max[axis].max(b[axis + 3]);
                }
            }
            Some(Hit {
                block: [0; 3],
                min,
                max,
                distance,
                direct: false,
                boxes: Boxes::single(min, max),
            })
        })
        .min_by(|a, b| a.distance.total_cmp(&b.distance));
    let block = raycast(origin, dir, reach, &mut lookup);
    let limit = block.as_ref().map_or(f64::INFINITY, |hit| hit.distance);
    if let Some(actor) = actor {
        if actor.distance + ACTOR_PICK_RADIUS < limit.min(reach) {
            return Some(Target::Actor(actor));
        }
        if actor.distance + ACTOR_PICK_RADIUS < limit {
            return None;
        }
    }
    let hit = match block {
        Some(hit) => hit,
        None => support_hit(origin, dir, reach, &mut lookup)?,
    };
    centre_within(hit.block, origin, reach).then_some(Target::Block(hit))
}

/// Whether the centre of `block` is within `reach` of `eye`, as Cinnabar trims its picks.
pub fn centre_within(block: [i32; 3], eye: [f64; 3], reach: f64) -> bool {
    let distance_squared: f64 = (0..3)
        .map(|axis| (f64::from(block[axis]) + 0.5 - eye[axis]).powi(2))
        .sum();
    distance_squared <= reach * reach
}

/// Longest downward support ray after a clear miss, as Cinnabar's `BLOCK_USE_SUPPORT_DEPTH`.
pub const SUPPORT_DEPTH: f64 = 2.0;
/// Look direction Y must be below this for bridging support, as `BLOCK_USE_SUPPORT_MAX_Y`.
pub const SUPPORT_MAX_Y: f64 = -0.7;

/// The block Cinnabar places against while bridging: when the look ray misses and the player
/// looks steeply down, a ray straight down from the eye picks the supporting block, and the new
/// block goes on its side facing the look direction (`block_use_miss_support_current`).
///
/// Only offered when the look ray within `reach` passes through one of the eight cells around the
/// support at its level, sides and corners alike:
///
/// ```text
/// x 0 x
/// 0 1 0    1 = support, 0 = side gaps, x = corner gaps
/// x 0 x
/// ```
///
/// Looking further out into the void shows nothing.
pub fn support_hit(
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    lookup: impl FnMut([i32; 3]) -> Cell,
) -> Option<Hit> {
    if dir[1] >= SUPPORT_MAX_Y {
        return None;
    }
    let hit = raycast(origin, [0.0, -1.0, 0.0], SUPPORT_DEPTH, lookup)?;
    let [x, y, z] = hit.block;
    let aimed = (-1..=1)
        .flat_map(|dx| (-1..=1).map(move |dz| (dx, dz)))
        .filter(|&offset| offset != (0, 0))
        .any(|(dx, dz)| {
            let cell = [x + dx, y, z + dz];
            let min = cell.map(f64::from);
            let max = min.map(|v| v + 1.0);
            ray_box(origin, dir, min, max).is_some_and(|t| t <= reach)
        });
    aimed.then_some(Hit {
        direct: false,
        ..hit
    })
}

/// The neighbour of `support` on the side facing the dominant horizontal look direction.
pub fn placement_cell(support: [i32; 3], dir: [f64; 3]) -> [i32; 3] {
    let mut cell = support;
    if dir[0].abs() > dir[2].abs() {
        cell[0] += if dir[0] < 0.0 { -1 } else { 1 };
    } else {
        cell[2] += if dir[2] < 0.0 { -1 } else { 1 };
    }
    cell
}

/// Entry distance of the ray into the box, if it enters at or ahead of the origin.
fn ray_box(origin: [f64; 3], dir: [f64; 3], min: [f64; 3], max: [f64; 3]) -> Option<f64> {
    let mut near = f64::NEG_INFINITY;
    let mut far = f64::INFINITY;
    for axis in 0..3 {
        if dir[axis] == 0.0 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
            continue;
        }
        let a = (min[axis] - origin[axis]) / dir[axis];
        let b = (max[axis] - origin[axis]) / dir[axis];
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far && near >= 0.0).then_some(near)
}

/// Side of the largest terrain region the host answers.
pub const REGION_EXTENT: i32 = 16;

/// The most blocks read beyond what the look ray needs on every side when a new region is
/// requested, so small turns and steps keep using it.
pub const REGION_SLACK: i32 = 2;

/// Fuel one terrain read should use at most (of a frame's 200,000), judged by the last read's
/// cost per block.
pub const READ_FUEL_TARGET: f64 = 60_000.0;

/// How many spare blocks a new region gets: as many as keep the read within
/// `READ_FUEL_TARGET` at `fuel_per_cell` (what the last read cost per block), none before the
/// first read, whose cost is unknown. `cells(slack)` is the region's size with that slack.
pub fn region_slack(fuel_per_cell: Option<f64>, cells: impl Fn(i32) -> usize) -> i32 {
    let Some(per_cell) = fuel_per_cell else {
        return 0;
    };
    (1..=REGION_SLACK)
        .rev()
        .find(|&slack| per_cell * cells(slack) as f64 <= READ_FUEL_TARGET)
        .unwrap_or(0)
}

/// The number of blocks in a region.
pub fn region_cells((min, max): ([i32; 3], [i32; 3])) -> usize {
    (0..3)
        .map(|axis| (max[axis] - min[axis]).max(0) as usize)
        .product()
}

/// The blocks the look ray can reach from `eye` within `reach`, and `pad` more on every side,
/// as inclusive minimum and exclusive maximum corners. A pad of one covers the bridging
/// support too: it is only looked for when the ray points steeply down, past its depth.
pub fn ray_region(eye: [f64; 3], dir: [f64; 3], reach: f64, pad: i32) -> ([i32; 3], [i32; 3]) {
    let mut min = [0; 3];
    let mut max = [0; 3];
    for axis in 0..3 {
        let (a, b) = (eye[axis], eye[axis] + dir[axis] * reach);
        min[axis] = a.min(b).floor() as i32 - pad;
        max[axis] = a.max(b).floor() as i32 + 1 + pad;
    }
    (min, max)
}

/// Whether `region` holds every block the look ray needs.
pub fn region_covers(
    region: ([i32; 3], [i32; 3]),
    eye: [f64; 3],
    dir: [f64; 3],
    reach: f64,
) -> bool {
    let (need_min, need_max) = ray_region(eye, dir, reach, 1);
    (0..3).all(|axis| region.0[axis] <= need_min[axis] && need_max[axis] <= region.1[axis])
}

/// The terrain to sample: only the blocks along the look ray, not a cube around the player.
/// Cinnabar sends the name and tags of every kind of block in a region, which costs the mod
/// fuel per kind; on servers with many custom blocks a full cube can use up a whole frame. The
/// current region is kept while it still holds the ray; otherwise a new one is requested with
/// `slack` blocks to spare.
pub fn region_for(
    current: Option<([i32; 3], [i32; 3])>,
    eye: [f64; 3],
    dir: [f64; 3],
    reach: f64,
    slack: i32,
) -> ([i32; 3], [i32; 3]) {
    if let Some(region) = current
        && region_covers(region, eye, dir, reach)
    {
        return region;
    }
    ray_region(eye, dir, reach, 1 + slack)
}

/// Within this many blocks of the camera the outline keeps its full thickness.
pub const FULL_WIDTH_DISTANCE: f64 = 2.5;

/// Line width in pixels for a box seen from `camera`: the chosen thickness up close, then
/// shrinking with distance as a real object would, but never below one pixel.
pub fn width_at(width: f32, min: [f64; 3], max: [f64; 3], camera: [f64; 3]) -> f32 {
    let distance = (0..3)
        .map(|axis| {
            let centre = (min[axis] + max[axis]) / 2.0;
            (centre - camera[axis]).powi(2)
        })
        .sum::<f64>()
        .sqrt();
    let scale = (FULL_WIDTH_DISTANCE / distance.max(FULL_WIDTH_DISTANCE)) as f32;
    (width * scale).max(1.0).min(width)
}

/// One box face: the axis it is perpendicular to and whether it is on the maximum side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    pub axis: usize,
    pub max_side: bool,
}

/// The six faces of a box.
pub const FACES: [Face; 6] = [
    Face {
        axis: 0,
        max_side: false,
    },
    Face {
        axis: 0,
        max_side: true,
    },
    Face {
        axis: 1,
        max_side: false,
    },
    Face {
        axis: 1,
        max_side: true,
    },
    Face {
        axis: 2,
        max_side: false,
    },
    Face {
        axis: 2,
        max_side: true,
    },
];

impl Face {
    /// The face's four corners on the unit cube, in order around it.
    pub fn unit_corners(self) -> [[f32; 3]; 4] {
        let side = if self.max_side { 1.0 } else { 0.0 };
        let (u, v) = match self.axis {
            0 => (1, 2),
            1 => (0, 2),
            _ => (0, 1),
        };
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]].map(|[a, b]| {
            let mut corner = [0.0; 3];
            corner[self.axis] = side;
            corner[u] = a;
            corner[v] = b;
            corner
        })
    }

    /// Whether the camera is beyond the face's plane, so the face is turned towards it.
    pub fn visible(self, min: [f64; 3], max: [f64; 3], camera: [f64; 3]) -> bool {
        if self.max_side {
            camera[self.axis] > max[self.axis]
        } else {
            camera[self.axis] < min[self.axis]
        }
    }
}

/// One box edge: the axis it runs along and, for the other two axes in increasing order,
/// whether it lies on their minimum (false) or maximum (true) side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub axis: usize,
    pub sides: [bool; 2],
}

/// The twelve edges of a box, four along each axis.
pub const EDGES: [Edge; 12] = {
    let mut edges = [Edge {
        axis: 0,
        sides: [false, false],
    }; 12];
    let mut index = 0;
    while index < 12 {
        edges[index] = Edge {
            axis: index / 4,
            sides: [index % 2 == 1, (index / 2) % 2 == 1],
        };
        index += 1;
    }
    edges
};

impl Edge {
    /// The two axes other than the edge's own, in increasing order.
    pub fn others(self) -> [usize; 2] {
        match self.axis {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        }
    }

    /// Start and end points of this edge on the unit cube.
    pub fn unit_points(self) -> [[f32; 3]; 2] {
        let mut start = [0.0; 3];
        for (axis, side) in self.others().into_iter().zip(self.sides) {
            start[axis] = if side { 1.0 } else { 0.0 };
        }
        let mut end = start;
        end[self.axis] = 1.0;
        [start, end]
    }

    /// Whether the edge borders a face that faces `camera`: a face is visible when the camera is
    /// beyond its plane. Edges between two faces turned away from the camera are hidden behind
    /// the block, so only the outline of the faces you can see remains.
    pub fn visible(self, min: [f64; 3], max: [f64; 3], camera: [f64; 3]) -> bool {
        self.others()
            .into_iter()
            .zip(self.sides)
            .any(|(axis, side)| {
                if side {
                    camera[axis] > max[axis]
                } else {
                    camera[axis] < min[axis]
                }
            })
    }

    /// World-space start and end points on the box.
    pub fn points(self, min: [f64; 3], max: [f64; 3]) -> [[f32; 3]; 2] {
        self.unit_points().map(|unit| {
            std::array::from_fn(|axis| {
                (min[axis] + f64::from(unit[axis]) * (max[axis] - min[axis])) as f32
            })
        })
    }
}
