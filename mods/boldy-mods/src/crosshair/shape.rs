//! Turning a crosshair into rectangles: runs of pixels, circles and the style shapes.

use super::*;

/// The drawing as rectangles `[col0, row0, col1, row1]`, ends exclusive: each row's runs of
/// pixels, joined with the same run in the rows below. None overlaps another. Stops once there
/// are more than [`MAX_RECTS`]: such a drawing is too detailed, and the rest is never drawn.
///
/// A row the same as the one above only lengthens the rectangles already open, so it costs
/// nothing; every other row costs for each run of pixels in it ([`rects_cost`]).
pub(super) fn find_rects(drawing: &Drawing) -> Vec<[u8; 4]> {
    // A row holds at most this many runs. Fixed arrays: this is the costly part of a drawing.
    const RUNS: usize = MAX_GRID / 2;
    let mut done: Vec<[u8; 4]> = Vec::with_capacity(MAX_RECTS + 1);
    // Rectangles open in the row before, sorted by their first column: `[col0, row0, col1, _]`.
    let (mut open, mut open_len) = ([[0u8; 4]; RUNS], 0);
    let mut previous = 0u64;
    let n = drawing.size();
    // Where the rectangles still open end: the grid's foot, or the row reached when stopping.
    let mut last = n as u8;
    let mut runs = 0u64;
    for (row, &line) in drawing.rows[..n].iter().enumerate() {
        if line == previous {
            continue;
        }
        previous = line;
        let row = row as u8;
        let (mut next, mut next_len) = ([[0u8; 4]; RUNS], 0);
        let mut bits = line;
        let mut j = 0;
        runs += u64::from((line & !(line << 1)).count_ones());
        while bits != 0 {
            // The lowest run of set bits, then the same bits cleared.
            let start = bits.trailing_zeros() as u8;
            let end = start + (bits >> start).trailing_ones() as u8;
            bits &= bits.wrapping_add(bits.isolate_lowest_one());
            // Rectangles starting further left did not go on into this row.
            while j < open_len && open[j][0] < start {
                done.push([open[j][0], open[j][1], open[j][2], row]);
                j += 1;
            }
            next[next_len] = if j < open_len && open[j][0] == start && open[j][2] == end {
                j += 1;
                open[j - 1]
            } else {
                [start, row, end, 0]
            };
            next_len += 1;
        }
        for rect in &open[j..open_len] {
            done.push([rect[0], rect[1], rect[2], row]);
        }
        (open, open_len) = (next, next_len);
        if done.len() + open_len > MAX_RECTS || runs > MAX_RUNS {
            last = row + 1;
            break;
        }
    }
    done.extend(open[..open_len].iter().map(|r| [r[0], r[1], r[2], last]));
    done
}

/// About how much fuel [`rects`] takes for `drawing`, worked out cheaply beforehand: nothing
/// when it was the last asked for, else some, and more for each run of pixels in a row that
/// differs from the one above. Measured under wasmtime's fuel count, with some to spare.
pub fn rects_cost(drawing: &Drawing) -> u64 {
    let known = RECTS.with(|memo| {
        memo.borrow()
            .as_ref()
            .is_some_and(|(known, _)| known == drawing)
    });
    if known {
        return 0;
    }
    // Finding them stops past the limit; each run may also start a rectangle, which costs more.
    let runs = runs(drawing).min(MAX_RUNS + 64);
    22_000 + RUN_COST * runs + NEW_RECT_COST * runs.min(MAX_RECTS as u64 + 1)
}

pub(super) const RUN_COST: u64 = 140;

pub(super) const NEW_RECT_COST: u64 = 200;

/// The runs of pixels in rows unlike the row above: what finding the rectangles costs for.
/// Cheap: a few operations a row.
pub fn runs(drawing: &Drawing) -> u64 {
    let mut previous = 0u64;
    let mut runs = 0u64;
    for &line in &drawing.rows[..drawing.size()] {
        if line != previous {
            // A run starts at each pixel whose left neighbour is empty.
            runs += u64::from((line & !(line << 1)).count_ones());
            previous = line;
        }
    }
    runs
}

/// A drawing and its rectangles.
pub(super) type Found = Option<(Drawing, Rc<Vec<[u8; 4]>>)>;

thread_local! {
    /// The last drawing [`rects`] was asked for and its rectangles: a frame asks several times.
    static RECTS: RefCell<Found> = const { RefCell::new(None) };
}

/// [`find_rects`] of `drawing`, worked out once for the same drawing.
pub fn rects(drawing: &Drawing) -> Rc<Vec<[u8; 4]>> {
    RECTS.with(|memo| {
        let mut memo = memo.borrow_mut();
        if let Some((known, rects)) = memo.as_ref()
            && known == drawing
        {
            return Rc::clone(rects);
        }
        let rects = Rc::new(find_rects(drawing));
        *memo = Some((*drawing, Rc::clone(&rects)));
        rects
    })
}

/// Whether the drawing is more than can be sent: too many runs of pixels to find its
/// rectangles in a frame (checked first, cheaply), or too many rectangles.
pub fn too_detailed(drawing: &Drawing) -> bool {
    runs(drawing) > MAX_RUNS || rects(drawing).len() > MAX_RECTS
}

/// Points round a circle, enough that its edge looks round at crosshair sizes.
pub(super) const CIRCLE_POINTS: usize = 24;

/// The circle's points round a unit circle, worked out once.
pub(super) fn unit_circle() -> &'static [[f32; 2]; CIRCLE_POINTS] {
    static TABLE: OnceLock<[[f32; 2]; CIRCLE_POINTS]> = OnceLock::new();
    TABLE.get_or_init(|| {
        std::array::from_fn(|index| {
            let angle = std::f32::consts::TAU * index as f32 / CIRCLE_POINTS as f32;
            [angle.cos(), angle.sin()]
        })
    })
}

/// A circle's points; a small one (a dot) looks as round with half of them, which saves fuel.
pub(super) fn circle(centre: f32, radius: f32) -> Vec<[f32; 2]> {
    let step = if radius < 6.0 { 2 } else { 1 };
    unit_circle()
        .iter()
        .step_by(step)
        .map(|[x, y]| [centre + radius * x, centre + radius * y])
        .collect()
}

/// The widest a vector path may be, in physical pixels: Cinnabar samples a path every half pixel
/// and draws nothing of one that needs more than 256 samples across.
pub const MAX_PATH: f32 = 120.0;

/// The crosshair as the HUD draws it, in physical pixels from its box's top-left corner.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    /// The side of its square box, centred on the screen: an even number of pixels.
    pub side: f32,
    /// Where the pieces are centred in the box.
    pub centre: f32,
    /// Rectangles, `[x0, y0, x1, y1]`; none overlaps another.
    pub rects: Vec<[f32; 4]>,
    /// Circle contours, filled even-odd: a ring's two, a dot's one.
    pub circles: Vec<Vec<[f32; 2]>>,
    /// Drawn turned by 45 degrees about the box's middle: the X.
    pub turned: bool,
    /// How far the outline reaches out of the pieces.
    pub stroke: f32,
}

/// The crosshair `look` draws at GUI `scale` (physical pixels per GUI unit).
pub fn shape(look: &Crosshair, scale: f32) -> Shape {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    // Every size on whole pixels, so edges fall on pixel boundaries.
    let px = |units: f32, least: f32| (units * scale).round().max(least);
    let style = look.style.min(STYLES.len() as u8 - 1);
    let custom = style == CUSTOM;
    // A drawing's pixels are sized in screen pixels; the other styles' lines in GUI units.
    let t = if custom {
        f32::from(look.pixel_size.clamp(MIN_PIXEL_SIZE, MAX_PIXEL_SIZE))
    } else {
        px(THICKNESS.clamp(look.thickness), 1.0)
    };
    let g = px(GAP.clamp(look.gap), 0.0);
    let l = px(LENGTH.clamp(look.length), 1.0);
    let stroke = (scale / 2.0).round().max(1.0);
    let h = t / 2.0;
    let turned = style == X;
    // Arms: top, right, bottom, left.
    let arms = match style {
        T_SHAPE => [false, true, true, true],
        CIRCLE | CIRCLE_DOT | DOT | CUSTOM => [false; 4],
        _ => [true; 4],
    };
    let any_arm = arms.contains(&true);
    let ring = matches!(style, CIRCLE | CIRCLE_DOT);
    let round_dot = matches!(style, CIRCLE_DOT | DOT);
    // A path wider than `MAX_PATH` is not drawn at all, outline included.
    let dot = (2.0 * t).max(2.0).min(MAX_PATH - 2.0 * stroke);
    // The gap is counted from the edge of the middle square (as thick as a line each way), so
    // the arms' ends fall on whole pixels too.
    let start = h + g;
    let reach = start + l;
    // The ring's middle line runs through the middle of where the arms would be, and always
    // leaves a pixel free round the dot inside it.
    let mut ring_radius = start + l / 2.0;
    if ring {
        let inside = if round_dot { dot / 2.0 } else { 0.0 };
        ring_radius = ring_radius
            .min(MAX_PATH / 2.0 - stroke - h)
            .max(h + inside + 1.0);
    }
    let half_grid = look.pixels.size() as f32 / 2.0 * t;
    let extent = if custom {
        half_grid
    } else if ring {
        ring_radius + h
    } else if any_arm {
        // Turned, an arm's far corners reach a little further than its end.
        if turned { reach.hypot(h) } else { reach }
    } else {
        dot / 2.0
    };
    let side = 2.0 * (extent + stroke + 2.0).ceil();
    // An odd thickness centres on the middle of a pixel, so the pieces cover whole pixels. A
    // drawing (an even number of pixels a side) and a turned crosshair are centred on the
    // box's middle.
    let c = side / 2.0
        + if t % 2.0 == 1.0 && !turned && !custom {
            0.5
        } else {
            0.0
        };

    let mut pieces: Vec<[f32; 4]> = Vec::new();
    if custom {
        let origin = c - half_grid;
        let at = |cell: u8| origin + f32::from(cell) * t;
        for &[c0, r0, c1, r1] in rects(&look.pixels).iter().take(MAX_RECTS) {
            pieces.push([at(c0), at(r0), at(c1), at(r1)]);
        }
    } else if any_arm {
        if g == 0.0 {
            // No gap: the arms meet in a filled middle. One bar across, the others up to it.
            pieces.push([c - reach, c - h, c + reach, c + h]);
            if arms[0] {
                pieces.push([c - h, c - reach, c + h, c - h]);
            }
            if arms[2] {
                pieces.push([c - h, c + h, c + h, c + reach]);
            }
        } else {
            // Each arm on its own, clear of the middle square (where a dot may go).
            let (r, s) = (reach, start);
            let arm_rects = [
                [c - h, c - r, c + h, c - s],
                [c + s, c - h, c + r, c + h],
                [c - h, c + s, c + h, c + r],
                [c - r, c - h, c - s, c + h],
            ];
            for (piece, on) in arm_rects.into_iter().zip(arms) {
                if on {
                    pieces.push(piece);
                }
            }
            if style == CROSS_DOT {
                pieces.push([c - h, c - h, c + h, c + h]);
            }
        }
    }
    let mut circles: Vec<Vec<[f32; 2]>> = Vec::new();
    if ring {
        // Two circles filled even-odd make a ring.
        circles.push(circle(c, ring_radius + h));
        circles.push(circle(c, ring_radius - h));
    }
    if round_dot {
        circles.push(circle(c, dot / 2.0));
    }
    Shape {
        side,
        centre: c,
        rects: pieces,
        circles,
        turned,
        stroke,
    }
}
