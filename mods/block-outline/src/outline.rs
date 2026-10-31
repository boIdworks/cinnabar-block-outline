//! Host-independent block outline logic: settings, colours, the block raycast and box edges.
//! Kept free of Cinnabar bindings so `cargo test` exercises it natively.

/// Player-editable settings, persisted as the mod's companion settings JSON.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// sRGB colour and opacity, written as `#RRGGBBAA`.
    pub rgba: [u8; 4],
    /// Cycles the hue over time instead of using `rgba`'s colour; opacity still applies.
    pub rainbow: bool,
    /// Line width in physical pixels.
    pub width: f32,
    /// Opacity, 0–100, of a fill in the outline colour over the block's visible faces; 0 is off.
    pub fill: u8,
    /// Maximum targeting distance from the eye, in blocks.
    pub reach: f32,
}

pub const MIN_WIDTH: f32 = 1.0;
pub const MAX_WIDTH: f32 = 8.0;
pub const MIN_REACH: f32 = 1.0;
/// Cinnabar's keyboard-and-mouse block reach, Survival and Creative alike
/// (`CREATIVE_MOUSE_REACH_BLOCKS`; controller 5.6, touch Survival 6.7).
pub const PLAYER_REACH: f32 = 5.7;
/// A 15-block terrain query (radius 7) stays inside the 16-block query extent limit.
pub const MAX_REACH: f32 = 7.0;

impl Default for Settings {
    fn default() -> Self {
        Self {
            rgba: [255, 255, 255, 255],
            rainbow: false,
            width: 3.0,
            fill: 0,
            reach: PLAYER_REACH,
        }
    }
}

/// One-click colours offered by the menu.
pub const SWATCHES: [(&str, [u8; 3]); 10] = [
    ("White", [255, 255, 255]),
    ("Black", [0, 0, 0]),
    ("Red", [255, 40, 40]),
    ("Orange", [255, 140, 0]),
    ("Yellow", [255, 230, 0]),
    ("Lime", [80, 255, 60]),
    ("Cyan", [0, 230, 255]),
    ("Blue", [40, 90, 255]),
    ("Purple", [170, 60, 255]),
    ("Pink", [255, 100, 200]),
];

impl Settings {
    /// Reads settings JSON, keeping defaults for missing or invalid fields. A legacy
    /// `"color": "rainbow"` turns rainbow on.
    pub fn parse(json: &str) -> Self {
        let mut settings = Self::default();
        match string_field(json, "color") {
            Some(text) if text.trim().eq_ignore_ascii_case("rainbow") => settings.rainbow = true,
            Some(text) => {
                if let Some(rgba) = parse_color(text) {
                    settings.rgba = rgba;
                }
            }
            None => {}
        }
        if let Some(rainbow) = bool_field(json, "rainbow") {
            settings.rainbow = rainbow;
        }
        if let Some(width) = number_field(json, "width") {
            settings.width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        }
        if let Some(fill) = number_field(json, "fill") {
            settings.fill = fill.round().clamp(0.0, 100.0) as u8;
        }
        if let Some(reach) = number_field(json, "reach") {
            settings.reach = reach.clamp(MIN_REACH, MAX_REACH);
        }
        settings
    }

    pub fn to_json(&self) -> String {
        let [r, g, b, a] = self.rgba;
        format!(
            "{{
  \"color\": \"#{r:02X}{g:02X}{b:02X}{a:02X}\",
  \"rainbow\": {},
  \"width\": {},
  \"fill\": {},
  \"reach\": {}
}}
",
            self.rainbow, self.width, self.fill, self.reach
        )
    }

    /// sRGB 0–1 colour shown at `seconds`: the rainbow hue or the fixed colour, with opacity.
    pub fn srgb(&self, seconds: f32) -> [f32; 4] {
        let alpha = f32::from(self.rgba[3]) / 255.0;
        if self.rainbow {
            let [r, g, b] = hue_to_rgb((seconds * 0.2).rem_euclid(1.0));
            [r, g, b, alpha]
        } else {
            let [r, g, b, _] = self.rgba.map(|c| f32::from(c) / 255.0);
            [r, g, b, alpha]
        }
    }

    /// Linear RGBA for the renderer at `seconds` of accumulated frame time.
    pub fn linear_rgba(&self, seconds: f32) -> [f32; 4] {
        let [r, g, b, a] = self.srgb(seconds);
        [srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), a]
    }
}

/// Accepts `#RGB`, `#RRGGBB` or `#RRGGBBAA`; the `#` is optional.
pub fn parse_color(text: &str) -> Option<[u8; 4]> {
    let text = text.trim();
    let hex = text.strip_prefix('#').unwrap_or(text);
    if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    match hex.len() {
        3 => {
            let nibble = |i: usize| u8::from_str_radix(&hex[i..=i], 16).ok().map(|v| v * 17);
            Some([nibble(0)?, nibble(1)?, nibble(2)?, 255])
        }
        6 => Some([byte(0)?, byte(2)?, byte(4)?, 255]),
        8 => Some([byte(0)?, byte(2)?, byte(4)?, byte(6)?]),
        _ => None,
    }
}

pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Fully saturated sRGB colour for a hue in 0–1.
fn hue_to_rgb(hue: f32) -> [f32; 3] {
    let h = hue * 6.0;
    let x = 1.0 - (h.rem_euclid(2.0) - 1.0).abs();
    match h as u32 {
        0 => [1.0, x, 0.0],
        1 => [x, 1.0, 0.0],
        2 => [0.0, 1.0, x],
        3 => [0.0, x, 1.0],
        4 => [x, 0.0, 1.0],
        _ => [1.0, 0.0, x],
    }
}

/// The text after `"key"` and its colon, for a flat JSON object.
fn field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let quoted = format!("\"{key}\"");
    let mut rest = json;
    while let Some(at) = rest.find(&quoted) {
        let after = rest[at + quoted.len()..].trim_start();
        if let Some(value) = after.strip_prefix(':') {
            return Some(value.trim_start());
        }
        rest = &rest[at + quoted.len()..];
    }
    None
}

fn string_field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let value = field(json, key)?.strip_prefix('"')?;
    Some(&value[..value.find('"')?])
}

fn bool_field(json: &str, key: &str) -> Option<bool> {
    let value = field(json, key)?;
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn number_field(json: &str, key: &str) -> Option<f32> {
    let value = field(json, key)?;
    let end = value
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')))
        .unwrap_or(value.len());
    value[..end].parse::<f32>().ok().filter(|v| v.is_finite())
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
#[derive(Debug, PartialEq)]
pub struct Hit {
    pub block: [i32; 3],
    pub min: [f64; 3],
    pub max: [f64; 3],
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
                let hit = boxes.iter().any(|b| {
                    let (min, max) = world(b);
                    ray_box(origin, dir, min, max).is_some_and(|t| t <= reach)
                });
                if hit {
                    let mut min = [f64::INFINITY; 3];
                    let mut max = [f64::NEG_INFINITY; 3];
                    for b in &boxes {
                        let (lo, hi) = world(b);
                        for axis in 0..3 {
                            min[axis] = min[axis].min(lo[axis]);
                            max[axis] = max[axis].max(hi[axis]);
                        }
                    }
                    return Some(Hit { block, min, max });
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
    let hit = match raycast(origin, dir, reach, &mut lookup) {
        Some(hit) => hit,
        None => support_hit(origin, dir, reach, &mut lookup)?,
    };
    centre_within(hit.block, origin, reach).then_some(hit)
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
    aimed.then_some(hit)
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

/// Side of the sampled terrain cube: the host's largest query extent.
pub const REGION_EXTENT: i32 = 16;

/// The terrain cube to sample, as inclusive minimum and exclusive maximum corners. Every block
/// within `radius` of `eye_block` must be inside it. The current cube is kept while that holds
/// with a one-block margin, so a new sample is requested before the player reaches the edge and
/// the old one still covers the ray while the new one loads; otherwise the cube is recentred.
pub fn region_for(
    current: Option<([i32; 3], [i32; 3])>,
    eye_block: [i32; 3],
    radius: i32,
) -> ([i32; 3], [i32; 3]) {
    let slack = (REGION_EXTENT - (2 * radius + 1)).max(0);
    let margin = (slack / 2).min(1);
    if let Some((min, max)) = current {
        let covers = (0..3).all(|axis| {
            min[axis] <= eye_block[axis] - radius - margin
                && eye_block[axis] + radius + margin < max[axis]
        });
        if covers {
            return (min, max);
        }
    }
    let min = eye_block.map(|v| v - radius - slack / 2);
    let max = min.map(|v| v + 2 * radius + 1 + slack);
    (min, max)
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
    Face { axis: 0, max_side: false },
    Face { axis: 0, max_side: true },
    Face { axis: 1, max_side: false },
    Face { axis: 1, max_side: true },
    Face { axis: 2, max_side: false },
    Face { axis: 2, max_side: true },
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
    let mut edges = [Edge { axis: 0, sides: [false, false] }; 12];
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
            .any(|(axis, side)| if side { camera[axis] > max[axis] } else { camera[axis] < min[axis] })
    }

    /// World-space start and end points on the box.
    pub fn points(self, min: [f64; 3], max: [f64; 3]) -> [[f32; 3]; 2] {
        self.unit_points().map(|unit| {
            std::array::from_fn(|axis| (min[axis] + f64::from(unit[axis]) * (max[axis] - min[axis])) as f32)
        })
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(Settings::parse("{\"color\": \"nope\"}"), Settings::default());
    }

    #[test]
    fn settings_round_trip_and_legacy_rainbow() {
        let s = Settings {
            rgba: [1, 2, 3, 4],
            rainbow: true,
            width: 3.5,
            fill: 35,
            reach: 5.0,
        };
        assert_eq!(Settings::parse(&s.to_json()), s);
        let legacy = Settings::parse(r#"{"color": "rainbow", "width": 3, "reach": 6}"#);
        assert!(legacy.rainbow);
        assert_eq!(legacy.rgba, Settings::default().rgba);
        let saved = Settings::parse(r##"{"color": "#50FF3CFF", "width": 3, "reach": 6}"##);
        assert_eq!(saved.rgba, [0x50, 0xFF, 0x3C, 0xFF]);
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
            if p == [0, 64, -3] { Cell::Boxes(vec![FULL]) } else { Cell::Boxes(vec![]) }
        };
        let hit = raycast([0.5, 64.5, 0.5], [0.0, 0.0, -1.0], 6.0, solid).unwrap();
        assert_eq!(hit.block, [0, 64, -3]);
        assert_eq!(hit.min, [0.0, 64.0, -3.0]);
        assert_eq!(hit.max, [1.0, 65.0, -2.0]);
        assert_eq!(raycast([0.5, 64.5, 0.5], [0.0, 0.0, -1.0], 2.0, solid), None);
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
            if p == [0, 64, -1] { Cell::Unknown } else { Cell::Boxes(vec![FULL]) }
        };
        assert_eq!(raycast([0.5, 64.5, 0.5], [0.0, 0.0, -1.0], 6.0, unknown), None);
    }

    #[test]
    fn diagonal_ray_finds_block() {
        let dir = look_direction(0.6, -0.7);
        let target = |p: [i32; 3]| {
            if p[1] <= 62 { Cell::Boxes(vec![FULL]) } else { Cell::Boxes(vec![]) }
        };
        let hit = raycast([0.3, 64.62, 0.7], dir, 6.0, target).unwrap();
        assert_eq!(hit.block[1], 62);
    }

    #[test]
    fn bridging_outlines_the_block_underfoot_only_when_looking_steeply_down() {
        // Standing at the edge of a one-block-wide bridge along -Z; the void lies ahead.
        let bridge = |p: [i32; 3]| {
            if p[0] == 0 && p[1] == 63 && p[2] >= 0 { Cell::Boxes(vec![FULL]) } else { Cell::Boxes(vec![]) }
        };
        let eye = [0.5, 65.62, 0.3];
        // Aiming into the empty cell in front of the bridge's end, [0, 63, -1].
        let steep = look_direction(0.0, -1.35);
        assert_eq!(raycast(eye, steep, 6.0, bridge), None, "the look ray misses into the void");
        assert_eq!(placement_cell([0, 63, 0], steep), [0, 63, -1]);
        assert_eq!(support_hit(eye, steep, 6.0, bridge).unwrap().block, [0, 63, 0]);
        // Steep enough, but aiming two blocks ahead: the ray passes over the placement cell.
        let ahead = look_direction(0.0, -0.8);
        assert!(ahead[1] < SUPPORT_MAX_Y);
        assert_eq!(support_hit(eye, ahead, 6.0, bridge), None);
        let shallow = look_direction(0.0, -0.6);
        assert_eq!(support_hit(eye, shallow, 6.0, bridge), None, "y -0.56 is not steep enough");
        // Facing diagonally (yaw 45°, towards -x -z), aiming into the corner gap.
        let diagonal = look_direction(std::f32::consts::FRAC_PI_4, -1.3);
        assert!(diagonal[0] < 0.0 && diagonal[2] < 0.0);
        assert_eq!(support_hit([0.5, 65.62, 0.5], diagonal, 6.0, bridge).unwrap().block, [0, 63, 0]);
        // Bridging straight along -z but aiming into the side gap at -x, and the far corner.
        let side = [-0.62, -0.78, 0.0];
        assert!(side[1] < SUPPORT_MAX_Y);
        let unit = |v: [f64; 3]| { let l = (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt(); v.map(|c| c / l) };
        assert!(support_hit([0.5, 65.62, 0.5], unit(side), 6.0, bridge).is_some(), "side gap");
        assert!(support_hit([0.5, 65.62, 0.5], unit([-0.45, -0.78, -0.45]), 6.0, bridge).is_some(), "corner");
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
        assert_eq!(width_at(4.0, min, max, [0.5, 0.5, 2.5]), 4.0, "2 blocks away");
        assert_eq!(width_at(4.0, min, max, [0.5, 0.5, 5.5]), 2.0, "5 blocks away: half");
        assert_eq!(width_at(1.5, min, max, [0.5, 0.5, 20.5]), 1.0, "never below one pixel");
        assert_eq!(width_at(1.0, min, max, [0.5, 0.5, 20.5]), 1.0);
    }

    #[test]
    fn region_is_kept_until_the_player_nears_its_edge() {
        let (min, max) = region_for(None, [0, 64, 0], 6);
        assert_eq!((min, max), ([-7, 57, -7], [9, 73, 9]), "16 blocks, 3 spare split 1/2");
        let mut current = (min, max);
        let mut requeries = 0;
        for x in 0..12 {
            let next = region_for(Some(current), [x, 64, 0], 6);
            // Whatever happens, the reach around the player is always covered.
            assert!(next.0[0] <= x - 6 && x + 6 < next.1[0]);
            if next != current {
                requeries += 1;
                // The old cube still covered the reach when it was replaced.
                assert!(current.0[0] <= x - 6 && x + 6 < current.1[0]);
            }
            current = next;
        }
        assert!(requeries <= 6, "{requeries} requests for 12 blocks walked");
        // Maximum reach leaves one spare block and no margin: recentre on every move, as before.
        let (min, max) = region_for(None, [0, 64, 0], 7);
        assert_eq!(max[0] - min[0], 16);
        // The cube is always 16 blocks, so a shorter reach keeps the current one.
        assert_eq!(region_for(Some((min, max)), [0, 64, 0], 3), (min, max));
    }

    #[test]
    fn far_blocks_need_their_centre_within_reach() {
        // A wall along z = -6 in front of the eye: the ray reaches its face at 5.5, inside 5.7,
        // but the block's centre is 6.0 away, so Cinnabar targets nothing.
        let wall = |p: [i32; 3]| if p[2] == -6 { Cell::Boxes(vec![FULL]) } else { Cell::Boxes(vec![]) };
        let eye = [0.5, 64.5, 0.5];
        let ahead = [0.0, 0.0, -1.0];
        assert!(raycast(eye, ahead, 5.7, wall).is_some());
        assert_eq!(target(eye, ahead, 5.7, wall), None);
        // One block closer, the centre is 5.0 away and it is targeted.
        let near = |p: [i32; 3]| if p[2] == -5 { Cell::Boxes(vec![FULL]) } else { Cell::Boxes(vec![]) };
        assert_eq!(target(eye, ahead, 5.7, near).unwrap().block, [0, 64, -5]);
    }

    #[test]
    fn faces_match_their_edges() {
        let (min, max) = ([0.0; 3], [1.0; 3]);
        for camera in [[0.5, 3.0, 0.5], [3.0, 3.0, 3.0], [-2.0, 0.5, 0.5], [0.5, 0.5, 0.5]] {
            let faces = FACES.iter().filter(|f| f.visible(min, max, camera)).count();
            let edges = EDGES.iter().filter(|e| e.visible(min, max, camera)).count();
            // Visible faces share edges pairwise: 1 face 4 edges, 2 faces 7, 3 faces 9.
            assert_eq!(edges, [0, 4, 7, 9][faces], "{camera:?}");
        }
        for face in FACES {
            let corners = face.unit_corners();
            assert!(corners.iter().all(|c| c[face.axis] == if face.max_side { 1.0 } else { 0.0 }));
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
}
