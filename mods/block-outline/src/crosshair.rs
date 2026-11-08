//! The custom crosshair: drawn on the HUD layer in place of the game's own, in the player's own
//! colour, shape and size. Host-independent so `cargo test` can check the shapes and patches.
//!
//! The crosshair is worked out in physical pixels, on whole pixels so thin lines stay sharp, as
//! rectangles (arms, square dots, a drawing's pixels) and circles (a ring, a round dot).
//! Rectangles are drawn as rounded rectangles with no rounding, in the crosshair's colour over
//! a black one a little larger for the outline. Circles are vector paths: filled even-odd in
//! colour, and stroked in black under that, twice the outline's width, so the outline shows just
//! outside the fill. Cinnabar draws nothing of a vector path wider than about 128 physical
//! pixels, so circles are kept within [`MAX_PATH`] and everything else is a rectangle.
//!
//! A binding can only hold a flag, a number, short text or two to four numbers, so shapes cannot
//! be bound. The crosshair is instead a top-level control definition of the HUD document
//! (`xhair`), replaced by `controls` patches; its pieces are small instances of templates (`xf`,
//! `xe`) whose colours are bound. Writing a piece costs a frame several thousand fuel, so a
//! crosshair is sent in steps, one a frame: a header (`xhair`: its box, rings and colours) and
//! [`PARTS`] parts of at most [`PART_RECTS`] rectangles each (`xpart0`...), each only when it
//! changed. The menu's drawing grid is the HUD's `drawgrid`, which the menu embeds
//! (`cinnabar_hud_control`), sent the same way: its lines for the resolution (`gridlines`) and
//! its drawn cells in parts (`gcells0`...).
//!
//! The Custom style is the player's own [`Drawing`], 16, 32 or 64 pixels square, made on the
//! menu. It is drawn as rectangles (runs of pixels in a row, joined with the same run in the
//! rows below), at most [`MAX_RECTS`] of them.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use crate::panel::{digits, fixed};

/// The shapes the Style value steps through, in order.
pub const STYLES: [&str; 8] = [
    "Cross",
    "Cross + dot",
    "T shape",
    "X",
    "Circle",
    "Circle + dot",
    "Dot",
    "Custom",
];
/// The same shapes as the settings file names them.
const STYLE_KEYS: [&str; 8] = [
    "cross",
    "cross_dot",
    "t",
    "x",
    "circle",
    "circle_dot",
    "dot",
    "custom",
];

const CROSS_DOT: u8 = 1;
const T_SHAPE: u8 = 2;
const X: u8 = 3;
const CIRCLE: u8 = 4;
const CIRCLE_DOT: u8 = 5;
const DOT: u8 = 6;
/// The player's own drawing.
pub const CUSTOM: u8 = 7;

/// The grid sizes a drawing can have, in pixels a side.
pub const RESOLUTIONS: [u8; 3] = [16, 32, 64];
/// The largest of them: a row of the drawing fits a `u64`.
pub const MAX_GRID: usize = 64;
/// How many parts a crosshair or the drawing grid is sent in, one a frame, and the most
/// rectangles in each: a frame cannot afford to write many more.
pub const PARTS: usize = 9;
pub const PART_RECTS: usize = 16;
/// The most rectangles a drawing may take.
pub const MAX_RECTS: usize = PARTS * PART_RECTS;
/// The most runs of pixels a drawing may have in rows unlike the row above: finding its
/// rectangles costs a frame for each ([`rects_cost`]).
pub const MAX_RUNS: u64 = 320;

/// The player's own drawing: `size` pixels a side, a row each from the top; bit `n` of a row is
/// its pixel `n` from the left. Bits and rows past `size` are always clear.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drawing {
    size: u8,
    rows: [u64; MAX_GRID],
}

/// The default drawing: a cross two pixels thick, centred on the grid's middle.
pub const DEFAULT_DRAWING: Drawing = {
    let mut rows = [0u64; MAX_GRID];
    let mut row = 1;
    while row < 15 {
        rows[row] = match row {
            7 | 8 => 0x7C3E,
            1..=5 | 10..=14 => 0x180,
            _ => 0,
        };
        row += 1;
    }
    Drawing { size: 16, rows }
};

/// All of a row's pixels at grid `size`.
const fn row_mask(size: usize) -> u64 {
    if size >= 64 {
        u64::MAX
    } else {
        (1 << size) - 1
    }
}

impl Drawing {
    /// An empty drawing `size` pixels a side, or 16 for a size not in [`RESOLUTIONS`].
    pub const fn empty(size: u8) -> Self {
        let size = match size {
            32 | 64 => size,
            _ => 16,
        };
        Drawing {
            size,
            rows: [0; MAX_GRID],
        }
    }

    /// Pixels a side.
    pub fn size(&self) -> usize {
        usize::from(self.size)
    }

    /// Whether it has no pixels drawn.
    pub fn is_empty(&self) -> bool {
        self.rows.iter().all(|row| *row == 0)
    }

    /// Whether pixel (`row`, `col`) is drawn.
    pub fn pixel(&self, row: usize, col: usize) -> bool {
        row < self.size() && col < self.size() && self.rows[row] & (1 << col) != 0
    }

    /// The drawing with pixel (`row`, `col`) set to `on`, and with `mirror` its reflections in
    /// the grid's middle row and column too.
    pub fn painted(&self, row: usize, col: usize, on: bool, mirror: bool) -> Self {
        let mut out = *self;
        let n = self.size();
        if row >= n || col >= n {
            return out;
        }
        let last = n - 1;
        let cells = if mirror {
            [
                (row, col),
                (row, last - col),
                (last - row, col),
                (last - row, last - col),
            ]
        } else {
            [(row, col); 4]
        };
        for (r, c) in cells {
            if on {
                out.rows[r] |= 1 << c;
            } else {
                out.rows[r] &= !(1 << c);
            }
        }
        out
    }

    /// The drawing with every pixel on the straight line from `from` to `to` (row, column) set
    /// to `on`, mirrored as [`Drawing::painted`] is: a stroke drawn faster than a pixel a frame
    /// leaves no gaps.
    pub fn painted_line(
        &self,
        from: (usize, usize),
        to: (usize, usize),
        on: bool,
        mirror: bool,
    ) -> Self {
        let mut out = *self;
        // Bresenham's line, over whole pixels.
        let (mut row, mut col) = (from.0 as i64, from.1 as i64);
        let (end_row, end_col) = (to.0 as i64, to.1 as i64);
        let (d_col, d_row) = ((end_col - col).abs(), -(end_row - row).abs());
        let (step_col, step_row) = ((end_col - col).signum(), (end_row - row).signum());
        let mut error = d_col + d_row;
        loop {
            out = out.painted(row as usize, col as usize, on, mirror);
            if row == end_row && col == end_col {
                return out;
            }
            let twice = 2 * error;
            if twice >= d_row {
                error += d_row;
                col += step_col;
            }
            if twice <= d_col {
                error += d_col;
                row += step_row;
            }
        }
    }

    /// The same drawing at grid `size`: each pixel becomes a square of pixels at a larger size,
    /// and at a smaller one a pixel is drawn where any pixel of its square was. Its shape stays.
    ///
    /// Whole rows at a time, visiting only drawn pixels: pixel by pixel, a 64 by 64 grid cost
    /// over half a frame's fuel, and with the save and the menu's redraw the frame ran out.
    pub fn resized(&self, size: u8) -> Self {
        let mut out = Drawing::empty(size);
        let (from, to) = (self.size(), out.size());
        if to >= from {
            let f = to / from;
            let square = row_mask(f);
            for row in 0..from {
                let (mut bits, mut wide) = (self.rows[row], 0u64);
                while bits != 0 {
                    wide |= square << (bits.trailing_zeros() as usize * f);
                    bits &= bits - 1;
                }
                out.rows[row * f..(row + 1) * f].fill(wide);
            }
        } else {
            let f = from / to;
            for row in 0..to {
                let mut bits = self.rows[row * f..(row + 1) * f]
                    .iter()
                    .fold(0, |all, line| all | line);
                let mut narrow = 0u64;
                while bits != 0 {
                    narrow |= 1 << (bits.trailing_zeros() as usize / f);
                    bits &= bits - 1;
                }
                out.rows[row] = narrow;
            }
        }
        out
    }

    /// The drawing as the settings file keeps it: each row in hexadecimal, `size / 4` digits
    /// with pixel 0 the lowest bit, rows joined by `/`. Written into one buffer: a 64 by 64
    /// drawing is a thousand digits, and the save shares its frame with the menu's redraw.
    pub fn text(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let n = self.size();
        let digits_per_row = n / 4;
        let mut bytes = vec![b'/'; n * (digits_per_row + 1) - 1];
        for row in 0..n {
            let line = self.rows[row];
            let start = row * (digits_per_row + 1);
            for digit in 0..digits_per_row {
                let nibble = (line >> ((digits_per_row - 1 - digit) * 4)) & 0xF;
                bytes[start + digit] = HEX[nibble as usize];
            }
        }
        String::from_utf8(bytes).unwrap_or_default()
    }

    /// A drawing of grid `size` in the settings file's form ([`Drawing::text`]). The older
    /// form, rows of `.` and `#`, is read too. Missing or unreadable rows are empty.
    pub fn parse(text: &str, size: u8) -> Self {
        let mut out = Drawing::empty(size);
        let n = out.size();
        let mask = row_mask(n);
        if out.read_hex_rows(text.as_bytes(), n, mask) {
            return out;
        }
        Self::parse_general(text, size)
    }

    /// [`Drawing::parse`] for any text, row by row.
    fn parse_general(text: &str, size: u8) -> Self {
        let mut out = Drawing::empty(size);
        let n = out.size();
        let mask = row_mask(n);
        for (row, line) in text.trim().split('/').take(n).enumerate() {
            let line = line.trim();
            out.rows[row] = if let Some(bits) = hex_bits(line) {
                // The usual form: only hex digits. Read by hand, as a 64 pixel drawing has 64
                // rows and this runs at start, where fuel is short.
                bits & mask
            } else if line.contains(['#', '.']) {
                line.chars()
                    .take(n)
                    .enumerate()
                    .filter(|(_, c)| matches!(c, '#' | 'X' | 'x' | '1'))
                    .fold(0u64, |bits, (col, _)| bits | 1 << col)
            } else {
                u64::from_str_radix(line, 16).unwrap_or(0) & mask
            };
        }
        out
    }
}

impl Drawing {
    /// Reads the usual form, rows of hex digits joined by `/`, eight digits at a time: the
    /// general reader below costs about sixty fuel a byte (a 64 pixel drawing is over a thousand
    /// bytes), and this runs in `init`, where the whole allowance is 200,000. Returns `false`
    /// with `self` half filled for anything else (spaces, the older `.` and `#` rows, a row of
    /// more than 16 digits), which the caller reads the slow way.
    fn read_hex_rows(&mut self, bytes: &[u8], n: usize, mask: u64) -> bool {
        let (mut at, mut row) = (0usize, 0usize);
        loop {
            let (mut bits, mut digits) = (0u64, 0u32);
            // Eight digits at a time while there are that many.
            while let Some(chunk) = bytes.get(at..).and_then(<[u8]>::first_chunk::<8>) {
                let Some(eight) = hex8(u64::from_le_bytes(*chunk)) else {
                    break;
                };
                if digits > 8 {
                    return false;
                }
                bits = bits << 32 | u64::from(eight);
                digits += 8;
                at += 8;
            }
            // The rest of the row, a digit at a time.
            while let Some(&byte) = bytes.get(at) {
                let digit = match byte {
                    b'0'..=b'9' => byte - b'0',
                    b'a'..=b'f' => byte - b'a' + 10,
                    b'A'..=b'F' => byte - b'A' + 10,
                    b'/' => break,
                    _ => return false,
                };
                if digits == 16 {
                    return false;
                }
                bits = bits << 4 | u64::from(digit);
                digits += 1;
                at += 1;
            }
            if row < n {
                self.rows[row] = bits & mask;
            }
            row += 1;
            if at >= bytes.len() {
                return true;
            }
            // Past the `/`.
            at += 1;
        }
    }
}

/// Eight hex digits (their bytes as a number, the first digit in the lowest byte) as the number
/// they spell, or `None` if any byte is not a hex digit. All eight at once: each byte becomes a
/// nibble (`c & 15`, plus 9 for a letter), which is checked by writing the nibbles back as digits
/// and comparing with the input.
fn hex8(word: u64) -> Option<u32> {
    const ONES: u64 = 0x0101_0101_0101_0101;
    const LOW: u64 = 0x0F0F_0F0F_0F0F_0F0F;
    let nibbles = (word & LOW) + 9 * ((word >> 6) & ONES);
    let written = nibbles + 0x30 * ONES + 39 * (((nibbles + 6 * ONES) >> 4) & ONES);
    // A letter in either case is compared as lower case, a digit as it is.
    let folded = word | ((word >> 1) & (0x20 * ONES));
    if nibbles & !LOW != 0 || written != folded {
        return None;
    }
    // Pairs of nibbles into bytes, bytes into 16 bit numbers, those into one 32 bit number.
    let bytes = ((nibbles << 4) | (nibbles >> 8)) & 0x00FF_00FF_00FF_00FF;
    let shorts = ((bytes << 8) | (bytes >> 16)) & 0x0000_FFFF_0000_FFFF;
    Some((((shorts << 16) | (shorts >> 32)) & 0xFFFF_FFFF) as u32)
}

/// A row of 1 to 16 hex digits as its bits, `None` for anything else (the older `.` and `#`
/// rows, signs, spaces), which the caller reads the slow way.
fn hex_bits(line: &str) -> Option<u64> {
    let bytes = line.as_bytes();
    if bytes.is_empty() || bytes.len() > 16 {
        return None;
    }
    let mut bits = 0u64;
    for &byte in bytes {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        bits = bits << 4 | u64::from(digit);
    }
    Some(bits)
}

/// Crosshair settings, saved with the other settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crosshair {
    /// Draw this crosshair instead of the game's own.
    pub on: bool,
    /// An index into [`STYLES`].
    pub style: u8,
    pub rgb: [u8; 3],
    /// Percent, from [`MIN_OPACITY`] to 100.
    pub opacity: u8,
    /// Arm length in GUI units; for the circles, how wide the ring is drawn.
    pub length: f32,
    /// Line thickness in GUI units; dots are twice as wide.
    pub thickness: f32,
    /// Room left empty in the middle, in GUI units.
    pub gap: f32,
    /// A thin dark edge round the shape, so it shows on bright and dark ground alike.
    pub outline: bool,
    /// The Custom style's drawing.
    pub pixels: Drawing,
    /// How wide each of the drawing's pixels is, in screen pixels.
    pub pixel_size: u8,
    /// Drawing paints the same pixel in all four quarters of the grid.
    pub mirror: bool,
}

/// A drawing's pixel size, in screen pixels.
pub const MIN_PIXEL_SIZE: u8 = 1;
pub const MAX_PIXEL_SIZE: u8 = 8;
pub const DEFAULT_PIXEL_SIZE: u8 = 2;

pub const DEFAULT: Crosshair = Crosshair {
    on: false,
    style: 0,
    rgb: [255, 255, 255],
    opacity: 100,
    length: 4.0,
    thickness: 1.0,
    gap: 2.0,
    outline: true,
    pixels: DEFAULT_DRAWING,
    pixel_size: DEFAULT_PIXEL_SIZE,
    mirror: true,
};

impl Default for Crosshair {
    fn default() -> Self {
        DEFAULT
    }
}

/// A pixel size within bounds.
pub fn clamp_pixel_size(size: f32) -> u8 {
    if !size.is_finite() {
        return DEFAULT_PIXEL_SIZE;
    }
    size.round()
        .clamp(f32::from(MIN_PIXEL_SIZE), f32::from(MAX_PIXEL_SIZE)) as u8
}

/// The next larger or smaller resolution, staying at the ends.
pub fn step_resolution(size: u8, up: bool) -> u8 {
    let at = RESOLUTIONS.iter().position(|r| *r == size).unwrap_or(0);
    let at = if up {
        (at + 1).min(RESOLUTIONS.len() - 1)
    } else {
        at.saturating_sub(1)
    };
    RESOLUTIONS[at]
}

/// `look` with its drawing at grid `size`, its shape kept and its pixel size changed so it stays
/// about as large on screen (each pixel at least one screen pixel).
pub fn with_resolution(look: &Crosshair, size: u8) -> Crosshair {
    let from = look.pixels.size() as f32;
    let pixels = look.pixels.resized(size);
    let to = pixels.size() as f32;
    Crosshair {
        pixels,
        pixel_size: clamp_pixel_size(f32::from(look.pixel_size) * from / to),
        ..*look
    }
}

/// A value's bounds and the step its - and + buttons take, in GUI units.
#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl Range {
    /// `value` on the nearest step and within bounds; anything unusable is the minimum.
    pub fn clamp(self, value: f32) -> f32 {
        if !value.is_finite() {
            return self.min;
        }
        ((value / self.step).round() * self.step).clamp(self.min, self.max)
    }

    /// `value` moved by `steps` steps (negative is smaller).
    pub fn stepped(self, value: f32, steps: f32) -> f32 {
        self.clamp(value + steps * self.step)
    }
}

pub const LENGTH: Range = Range {
    min: 1.0,
    max: 20.0,
    step: 0.5,
};
pub const THICKNESS: Range = Range {
    min: 0.5,
    max: 5.0,
    step: 0.5,
};
pub const GAP: Range = Range {
    min: 0.0,
    max: 10.0,
    step: 0.5,
};
/// Below this the crosshair would be too faint to aim with.
pub const MIN_OPACITY: u8 = 10;
pub const OPACITY_STEP: u8 = 10;

/// `percent` on a step of [`OPACITY_STEP`], within bounds.
pub fn clamp_opacity(percent: f32) -> u8 {
    if !percent.is_finite() {
        return 100;
    }
    let step = f32::from(OPACITY_STEP);
    ((percent / step).round() * step).clamp(f32::from(MIN_OPACITY), 100.0) as u8
}

/// The opacity one step down or up.
pub fn step_opacity(opacity: u8, up: bool) -> u8 {
    let step = f32::from(OPACITY_STEP);
    clamp_opacity(f32::from(opacity) + if up { step } else { -step })
}

/// The next or previous style, wrapping round at either end.
pub fn step_style(style: u8, up: bool) -> u8 {
    let count = STYLES.len() as u8;
    let style = style.min(count - 1);
    if up {
        (style + 1) % count
    } else {
        (style + count - 1) % count
    }
}

/// The style's name on the menu.
pub fn style_name(style: u8) -> &'static str {
    STYLES[usize::from(style).min(STYLES.len() - 1)]
}

/// The style's name in the settings file.
pub fn style_key(style: u8) -> &'static str {
    STYLE_KEYS[usize::from(style).min(STYLE_KEYS.len() - 1)]
}

/// The style the settings file names, if it names one.
pub fn style_from_key(key: &str) -> Option<u8> {
    let key = key.trim();
    STYLE_KEYS
        .iter()
        .position(|known| known.eq_ignore_ascii_case(key))
        .map(|index| index as u8)
}

/// The drawing as rectangles `[col0, row0, col1, row1]`, ends exclusive: each row's runs of
/// pixels, joined with the same run in the rows below. None overlaps another. Stops once there
/// are more than [`MAX_RECTS`]: such a drawing is too detailed, and the rest is never drawn.
///
/// A row the same as the one above only lengthens the rectangles already open, so it costs
/// nothing; every other row costs for each run of pixels in it ([`rects_cost`]).
fn find_rects(drawing: &Drawing) -> Vec<[u8; 4]> {
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
const RUN_COST: u64 = 140;
const NEW_RECT_COST: u64 = 200;

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
type Found = Option<(Drawing, Rc<Vec<[u8; 4]>>)>;

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
const CIRCLE_POINTS: usize = 24;

/// The circle's points round a unit circle, worked out once.
fn unit_circle() -> &'static [[f32; 2]; CIRCLE_POINTS] {
    static TABLE: OnceLock<[[f32; 2]; CIRCLE_POINTS]> = OnceLock::new();
    TABLE.get_or_init(|| {
        std::array::from_fn(|index| {
            let angle = std::f32::consts::TAU * index as f32 / CIRCLE_POINTS as f32;
            [angle.cos(), angle.sin()]
        })
    })
}

/// A circle's points; a small one (a dot) looks as round with half of them, which saves fuel.
fn circle(centre: f32, radius: f32) -> Vec<[f32; 2]> {
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

/// What the crosshair shows: its look and the HUD's size (width, height and GUI scale).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub look: Crosshair,
    pub viewport: [f32; 3],
}

/// The crosshair's instance in the HUD document's root, as a string literal: its definition,
/// `xhair`, is replaced whenever its shape changes.
macro_rules! document {
    () => {
        r##"{"crosshair@boldy_hud.xhair":{}}"##
    };
}
pub(crate) use document;

/// The definitions the crosshair and the menu's drawing grid use, as string literal entries of
/// the HUD document: an empty crosshair, hidden, and its empty parts; their fill (`xf`) and
/// outline (`xe`) pieces; the grid (`drawgrid`, 224 units a side) with its cells' parts
/// (`gcells0`... of `gp` pieces), a highlight on the pixel under the pointer and its mirrored
/// pixels (`gc`), and its lines (`gridlines`, one of `lines16`, `lines32` and `lines64`: faint
/// `gl` lines, stronger `gj` lines every 4 or 8 pixels, and `gm` lines through the middle).
macro_rules! definitions {
    () => {
        concat!(
            r##""xhair":{"type":"panel","size":[1,1],"offset":[0,0],"anchor_from":"center","anchor_to":"center","layer":2,"bindings":[{"binding_name":"#crosshair_visible","binding_name_override":"#visible"}]},"##,
            r##""xpart0":{"type":"panel","size":["100%","100%"]},"xpart1":{"type":"panel","size":["100%","100%"]},"xpart2":{"type":"panel","size":["100%","100%"]},"##,
            r##""xpart3":{"type":"panel","size":["100%","100%"]},"xpart4":{"type":"panel","size":["100%","100%"]},"xpart5":{"type":"panel","size":["100%","100%"]},"##,
            r##""xpart6":{"type":"panel","size":["100%","100%"]},"xpart7":{"type":"panel","size":["100%","100%"]},"xpart8":{"type":"panel","size":["100%","100%"]},"##,
            r##""xf":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,1],"layer":2,"bindings":[{"binding_name":"#crosshair_color","binding_name_override":"#color"}]},"##,
            r##""xe":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0,0,0,1],"layer":1,"bindings":[{"binding_name":"#crosshair_edge_color","binding_name_override":"#color"}]},"##,
            r##""drawgrid":{"type":"panel","size":[224,224],"controls":["##,
            r##"{"back":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["100%","100%"],"radius":0,"color":[0.04,0.045,0.06,1],"layer":1}},"##,
            r##"{"c0@boldy_hud.gcells0":{}},{"c1@boldy_hud.gcells1":{}},{"c2@boldy_hud.gcells2":{}},{"c3@boldy_hud.gcells3":{}},{"c4@boldy_hud.gcells4":{}},"##,
            r##"{"c5@boldy_hud.gcells5":{}},{"c6@boldy_hud.gcells6":{}},{"c7@boldy_hud.gcells7":{}},{"c8@boldy_hud.gcells8":{}},"##,
            r##"{"cursor0@boldy_hud.gc":{"bindings":[{"binding_name":"#drawgrid_cursor0_visible","binding_name_override":"#visible"},{"binding_name":"#drawgrid_cursor0_offset","binding_name_override":"#offset"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"cursor1@boldy_hud.gc":{"bindings":[{"binding_name":"#drawgrid_cursor1_visible","binding_name_override":"#visible"},{"binding_name":"#drawgrid_cursor1_offset","binding_name_override":"#offset"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"cursor2@boldy_hud.gc":{"bindings":[{"binding_name":"#drawgrid_cursor2_visible","binding_name_override":"#visible"},{"binding_name":"#drawgrid_cursor2_offset","binding_name_override":"#offset"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"cursor3@boldy_hud.gc":{"bindings":[{"binding_name":"#drawgrid_cursor3_visible","binding_name_override":"#visible"},{"binding_name":"#drawgrid_cursor3_offset","binding_name_override":"#offset"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"lines@boldy_hud.gridlines":{}}]},"##,
            r##""gcells0":{"type":"panel","size":["100%","100%"],"layer":3},"gcells1":{"type":"panel","size":["100%","100%"],"layer":3},"gcells2":{"type":"panel","size":["100%","100%"],"layer":3},"##,
            r##""gcells3":{"type":"panel","size":["100%","100%"],"layer":3},"gcells4":{"type":"panel","size":["100%","100%"],"layer":3},"gcells5":{"type":"panel","size":["100%","100%"],"layer":3},"##,
            r##""gcells6":{"type":"panel","size":["100%","100%"],"layer":3},"gcells7":{"type":"panel","size":["100%","100%"],"layer":3},"gcells8":{"type":"panel","size":["100%","100%"],"layer":3},"##,
            r##""gp":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,1],"layer":0,"bindings":[{"binding_name":"#drawgrid_color","binding_name_override":"#color"}]},"##,
            r##""gl":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,0.09],"layer":0},"##,
            r##""gj":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,0.26],"layer":0},"##,
            r##""gc":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[14,14],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,0.32],"layer":4},"##,
            r##""gm":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.66,0.5,0.97,0.55],"layer":1},"##,
            r##""gridlines":{"type":"panel","size":["100%","100%"],"layer":5,"controls":[{"l@boldy_hud.lines16":{}}]},"##,
            r##""lines16":{"type":"panel","size":["100%","100%"],"controls":[{"v0@boldy_hud.gj":{"offset":[-0.25,0],"size":[0.5,224]}},{"h0@boldy_hud.gj":{"offset":[0,-0.25],"size":[224,0.5]}},{"v1@boldy_hud.gl":{"offset":[13.75,0],"size":[0.5,224]}},{"h1@boldy_hud.gl":{"offset":[0,13.75],"size":[224,0.5]}},{"v2@boldy_hud.gl":{"offset":[27.75,0],"size":[0.5,224]}},{"h2@boldy_hud.gl":{"offset":[0,27.75],"size":[224,0.5]}},{"v3@boldy_hud.gl":{"offset":[41.75,0],"size":[0.5,224]}},{"h3@boldy_hud.gl":{"offset":[0,41.75],"size":[224,0.5]}},{"v4@boldy_hud.gj":{"offset":[55.75,0],"size":[0.5,224]}},{"h4@boldy_hud.gj":{"offset":[0,55.75],"size":[224,0.5]}},{"v5@boldy_hud.gl":{"offset":[69.75,0],"size":[0.5,224]}},{"h5@boldy_hud.gl":{"offset":[0,69.75],"size":[224,0.5]}},{"v6@boldy_hud.gl":{"offset":[83.75,0],"size":[0.5,224]}},{"h6@boldy_hud.gl":{"offset":[0,83.75],"size":[224,0.5]}},{"v7@boldy_hud.gl":{"offset":[97.75,0],"size":[0.5,224]}},{"h7@boldy_hud.gl":{"offset":[0,97.75],"size":[224,0.5]}},{"v9@boldy_hud.gl":{"offset":[125.75,0],"size":[0.5,224]}},{"h9@boldy_hud.gl":{"offset":[0,125.75],"size":[224,0.5]}},{"v10@boldy_hud.gl":{"offset":[139.75,0],"size":[0.5,224]}},{"h10@boldy_hud.gl":{"offset":[0,139.75],"size":[224,0.5]}},{"v11@boldy_hud.gl":{"offset":[153.75,0],"size":[0.5,224]}},{"h11@boldy_hud.gl":{"offset":[0,153.75],"size":[224,0.5]}},{"v12@boldy_hud.gj":{"offset":[167.75,0],"size":[0.5,224]}},{"h12@boldy_hud.gj":{"offset":[0,167.75],"size":[224,0.5]}},{"v13@boldy_hud.gl":{"offset":[181.75,0],"size":[0.5,224]}},{"h13@boldy_hud.gl":{"offset":[0,181.75],"size":[224,0.5]}},{"v14@boldy_hud.gl":{"offset":[195.75,0],"size":[0.5,224]}},{"h14@boldy_hud.gl":{"offset":[0,195.75],"size":[224,0.5]}},{"v15@boldy_hud.gl":{"offset":[209.75,0],"size":[0.5,224]}},{"h15@boldy_hud.gl":{"offset":[0,209.75],"size":[224,0.5]}},{"v16@boldy_hud.gj":{"offset":[223.75,0],"size":[0.5,224]}},{"h16@boldy_hud.gj":{"offset":[0,223.75],"size":[224,0.5]}},{"mv@boldy_hud.gm":{"offset":[111.5,0],"size":[1,224]}},{"mh@boldy_hud.gm":{"offset":[0,111.5],"size":[224,1]}}]},"##,
            r##""lines32":{"type":"panel","size":["100%","100%"],"controls":[{"v0@boldy_hud.gj":{"offset":[-0.25,0],"size":[0.5,224]}},{"h0@boldy_hud.gj":{"offset":[0,-0.25],"size":[224,0.5]}},{"v1@boldy_hud.gl":{"offset":[6.75,0],"size":[0.5,224]}},{"h1@boldy_hud.gl":{"offset":[0,6.75],"size":[224,0.5]}},{"v2@boldy_hud.gl":{"offset":[13.75,0],"size":[0.5,224]}},{"h2@boldy_hud.gl":{"offset":[0,13.75],"size":[224,0.5]}},{"v3@boldy_hud.gl":{"offset":[20.75,0],"size":[0.5,224]}},{"h3@boldy_hud.gl":{"offset":[0,20.75],"size":[224,0.5]}},{"v4@boldy_hud.gj":{"offset":[27.75,0],"size":[0.5,224]}},{"h4@boldy_hud.gj":{"offset":[0,27.75],"size":[224,0.5]}},{"v5@boldy_hud.gl":{"offset":[34.75,0],"size":[0.5,224]}},{"h5@boldy_hud.gl":{"offset":[0,34.75],"size":[224,0.5]}},{"v6@boldy_hud.gl":{"offset":[41.75,0],"size":[0.5,224]}},{"h6@boldy_hud.gl":{"offset":[0,41.75],"size":[224,0.5]}},{"v7@boldy_hud.gl":{"offset":[48.75,0],"size":[0.5,224]}},{"h7@boldy_hud.gl":{"offset":[0,48.75],"size":[224,0.5]}},{"v8@boldy_hud.gj":{"offset":[55.75,0],"size":[0.5,224]}},{"h8@boldy_hud.gj":{"offset":[0,55.75],"size":[224,0.5]}},{"v9@boldy_hud.gl":{"offset":[62.75,0],"size":[0.5,224]}},{"h9@boldy_hud.gl":{"offset":[0,62.75],"size":[224,0.5]}},{"v10@boldy_hud.gl":{"offset":[69.75,0],"size":[0.5,224]}},{"h10@boldy_hud.gl":{"offset":[0,69.75],"size":[224,0.5]}},{"v11@boldy_hud.gl":{"offset":[76.75,0],"size":[0.5,224]}},{"h11@boldy_hud.gl":{"offset":[0,76.75],"size":[224,0.5]}},{"v12@boldy_hud.gj":{"offset":[83.75,0],"size":[0.5,224]}},{"h12@boldy_hud.gj":{"offset":[0,83.75],"size":[224,0.5]}},{"v13@boldy_hud.gl":{"offset":[90.75,0],"size":[0.5,224]}},{"h13@boldy_hud.gl":{"offset":[0,90.75],"size":[224,0.5]}},{"v14@boldy_hud.gl":{"offset":[97.75,0],"size":[0.5,224]}},{"h14@boldy_hud.gl":{"offset":[0,97.75],"size":[224,0.5]}},{"v15@boldy_hud.gl":{"offset":[104.75,0],"size":[0.5,224]}},{"h15@boldy_hud.gl":{"offset":[0,104.75],"size":[224,0.5]}},{"v17@boldy_hud.gl":{"offset":[118.75,0],"size":[0.5,224]}},{"h17@boldy_hud.gl":{"offset":[0,118.75],"size":[224,0.5]}},{"v18@boldy_hud.gl":{"offset":[125.75,0],"size":[0.5,224]}},{"h18@boldy_hud.gl":{"offset":[0,125.75],"size":[224,0.5]}},{"v19@boldy_hud.gl":{"offset":[132.75,0],"size":[0.5,224]}},{"h19@boldy_hud.gl":{"offset":[0,132.75],"size":[224,0.5]}},{"v20@boldy_hud.gj":{"offset":[139.75,0],"size":[0.5,224]}},{"h20@boldy_hud.gj":{"offset":[0,139.75],"size":[224,0.5]}},{"v21@boldy_hud.gl":{"offset":[146.75,0],"size":[0.5,224]}},{"h21@boldy_hud.gl":{"offset":[0,146.75],"size":[224,0.5]}},{"v22@boldy_hud.gl":{"offset":[153.75,0],"size":[0.5,224]}},{"h22@boldy_hud.gl":{"offset":[0,153.75],"size":[224,0.5]}},{"v23@boldy_hud.gl":{"offset":[160.75,0],"size":[0.5,224]}},{"h23@boldy_hud.gl":{"offset":[0,160.75],"size":[224,0.5]}},{"v24@boldy_hud.gj":{"offset":[167.75,0],"size":[0.5,224]}},{"h24@boldy_hud.gj":{"offset":[0,167.75],"size":[224,0.5]}},{"v25@boldy_hud.gl":{"offset":[174.75,0],"size":[0.5,224]}},{"h25@boldy_hud.gl":{"offset":[0,174.75],"size":[224,0.5]}},{"v26@boldy_hud.gl":{"offset":[181.75,0],"size":[0.5,224]}},{"h26@boldy_hud.gl":{"offset":[0,181.75],"size":[224,0.5]}},{"v27@boldy_hud.gl":{"offset":[188.75,0],"size":[0.5,224]}},{"h27@boldy_hud.gl":{"offset":[0,188.75],"size":[224,0.5]}},{"v28@boldy_hud.gj":{"offset":[195.75,0],"size":[0.5,224]}},{"h28@boldy_hud.gj":{"offset":[0,195.75],"size":[224,0.5]}},{"v29@boldy_hud.gl":{"offset":[202.75,0],"size":[0.5,224]}},{"h29@boldy_hud.gl":{"offset":[0,202.75],"size":[224,0.5]}},{"v30@boldy_hud.gl":{"offset":[209.75,0],"size":[0.5,224]}},{"h30@boldy_hud.gl":{"offset":[0,209.75],"size":[224,0.5]}},{"v31@boldy_hud.gl":{"offset":[216.75,0],"size":[0.5,224]}},{"h31@boldy_hud.gl":{"offset":[0,216.75],"size":[224,0.5]}},{"v32@boldy_hud.gj":{"offset":[223.75,0],"size":[0.5,224]}},{"h32@boldy_hud.gj":{"offset":[0,223.75],"size":[224,0.5]}},{"mv@boldy_hud.gm":{"offset":[111.5,0],"size":[1,224]}},{"mh@boldy_hud.gm":{"offset":[0,111.5],"size":[224,1]}}]},"##,
            r##""lines64":{"type":"panel","size":["100%","100%"],"controls":[{"v0@boldy_hud.gj":{"offset":[-0.25,0],"size":[0.5,224]}},{"h0@boldy_hud.gj":{"offset":[0,-0.25],"size":[224,0.5]}},{"v2@boldy_hud.gl":{"offset":[6.75,0],"size":[0.5,224]}},{"h2@boldy_hud.gl":{"offset":[0,6.75],"size":[224,0.5]}},{"v4@boldy_hud.gl":{"offset":[13.75,0],"size":[0.5,224]}},{"h4@boldy_hud.gl":{"offset":[0,13.75],"size":[224,0.5]}},{"v6@boldy_hud.gl":{"offset":[20.75,0],"size":[0.5,224]}},{"h6@boldy_hud.gl":{"offset":[0,20.75],"size":[224,0.5]}},{"v8@boldy_hud.gj":{"offset":[27.75,0],"size":[0.5,224]}},{"h8@boldy_hud.gj":{"offset":[0,27.75],"size":[224,0.5]}},{"v10@boldy_hud.gl":{"offset":[34.75,0],"size":[0.5,224]}},{"h10@boldy_hud.gl":{"offset":[0,34.75],"size":[224,0.5]}},{"v12@boldy_hud.gl":{"offset":[41.75,0],"size":[0.5,224]}},{"h12@boldy_hud.gl":{"offset":[0,41.75],"size":[224,0.5]}},{"v14@boldy_hud.gl":{"offset":[48.75,0],"size":[0.5,224]}},{"h14@boldy_hud.gl":{"offset":[0,48.75],"size":[224,0.5]}},{"v16@boldy_hud.gj":{"offset":[55.75,0],"size":[0.5,224]}},{"h16@boldy_hud.gj":{"offset":[0,55.75],"size":[224,0.5]}},{"v18@boldy_hud.gl":{"offset":[62.75,0],"size":[0.5,224]}},{"h18@boldy_hud.gl":{"offset":[0,62.75],"size":[224,0.5]}},{"v20@boldy_hud.gl":{"offset":[69.75,0],"size":[0.5,224]}},{"h20@boldy_hud.gl":{"offset":[0,69.75],"size":[224,0.5]}},{"v22@boldy_hud.gl":{"offset":[76.75,0],"size":[0.5,224]}},{"h22@boldy_hud.gl":{"offset":[0,76.75],"size":[224,0.5]}},{"v24@boldy_hud.gj":{"offset":[83.75,0],"size":[0.5,224]}},{"h24@boldy_hud.gj":{"offset":[0,83.75],"size":[224,0.5]}},{"v26@boldy_hud.gl":{"offset":[90.75,0],"size":[0.5,224]}},{"h26@boldy_hud.gl":{"offset":[0,90.75],"size":[224,0.5]}},{"v28@boldy_hud.gl":{"offset":[97.75,0],"size":[0.5,224]}},{"h28@boldy_hud.gl":{"offset":[0,97.75],"size":[224,0.5]}},{"v30@boldy_hud.gl":{"offset":[104.75,0],"size":[0.5,224]}},{"h30@boldy_hud.gl":{"offset":[0,104.75],"size":[224,0.5]}},{"v34@boldy_hud.gl":{"offset":[118.75,0],"size":[0.5,224]}},{"h34@boldy_hud.gl":{"offset":[0,118.75],"size":[224,0.5]}},{"v36@boldy_hud.gl":{"offset":[125.75,0],"size":[0.5,224]}},{"h36@boldy_hud.gl":{"offset":[0,125.75],"size":[224,0.5]}},{"v38@boldy_hud.gl":{"offset":[132.75,0],"size":[0.5,224]}},{"h38@boldy_hud.gl":{"offset":[0,132.75],"size":[224,0.5]}},{"v40@boldy_hud.gj":{"offset":[139.75,0],"size":[0.5,224]}},{"h40@boldy_hud.gj":{"offset":[0,139.75],"size":[224,0.5]}},{"v42@boldy_hud.gl":{"offset":[146.75,0],"size":[0.5,224]}},{"h42@boldy_hud.gl":{"offset":[0,146.75],"size":[224,0.5]}},{"v44@boldy_hud.gl":{"offset":[153.75,0],"size":[0.5,224]}},{"h44@boldy_hud.gl":{"offset":[0,153.75],"size":[224,0.5]}},{"v46@boldy_hud.gl":{"offset":[160.75,0],"size":[0.5,224]}},{"h46@boldy_hud.gl":{"offset":[0,160.75],"size":[224,0.5]}},{"v48@boldy_hud.gj":{"offset":[167.75,0],"size":[0.5,224]}},{"h48@boldy_hud.gj":{"offset":[0,167.75],"size":[224,0.5]}},{"v50@boldy_hud.gl":{"offset":[174.75,0],"size":[0.5,224]}},{"h50@boldy_hud.gl":{"offset":[0,174.75],"size":[224,0.5]}},{"v52@boldy_hud.gl":{"offset":[181.75,0],"size":[0.5,224]}},{"h52@boldy_hud.gl":{"offset":[0,181.75],"size":[224,0.5]}},{"v54@boldy_hud.gl":{"offset":[188.75,0],"size":[0.5,224]}},{"h54@boldy_hud.gl":{"offset":[0,188.75],"size":[224,0.5]}},{"v56@boldy_hud.gj":{"offset":[195.75,0],"size":[0.5,224]}},{"h56@boldy_hud.gj":{"offset":[0,195.75],"size":[224,0.5]}},{"v58@boldy_hud.gl":{"offset":[202.75,0],"size":[0.5,224]}},{"h58@boldy_hud.gl":{"offset":[0,202.75],"size":[224,0.5]}},{"v60@boldy_hud.gl":{"offset":[209.75,0],"size":[0.5,224]}},{"h60@boldy_hud.gl":{"offset":[0,209.75],"size":[224,0.5]}},{"v62@boldy_hud.gl":{"offset":[216.75,0],"size":[0.5,224]}},{"h62@boldy_hud.gl":{"offset":[0,216.75],"size":[224,0.5]}},{"v64@boldy_hud.gj":{"offset":[223.75,0],"size":[0.5,224]}},{"h64@boldy_hud.gj":{"offset":[0,223.75],"size":[224,0.5]}},{"mv@boldy_hud.gm":{"offset":[111.5,0],"size":[1,224]}},{"mh@boldy_hud.gm":{"offset":[0,111.5],"size":[224,1]}}]}"##
        )
    };
}
pub(crate) use definitions;

/// The bindings of a hidden crosshair and an empty grid, to publish with the rest of the layer.
pub fn initial_bindings() -> String {
    String::from(
        r##""#crosshair_visible":false,"#crosshair_color":[1,1,1,1],"#crosshair_edge_color":[0,0,0,1],"#drawgrid_color":[1,1,1,1],"#drawgrid_cursor0_visible":false,"#drawgrid_cursor1_visible":false,"#drawgrid_cursor2_visible":false,"#drawgrid_cursor3_visible":false"##,
    )
}

/// The GUI scale `view` is drawn at; an unusable one is 1.
fn view_scale(view: &View) -> f32 {
    let scale = view.viewport[2];
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

/// The shape `view` draws.
pub fn view_shape(view: &View) -> Shape {
    shape(&view.look, view_scale(view))
}

/// FNV-1a over 32-bit words: cheap, and enough to tell two versions of a part apart.
fn hash(words: impl Iterator<Item = u32>) -> u64 {
    words.fold(0xcbf2_9ce4_8422_2325, |h, word| {
        (h ^ u64::from(word)).wrapping_mul(0x100_0000_01b3)
    })
}

/// What sending a crosshair (or the grid) takes: a header, then each part, each told apart by
/// a hash so only what changed is sent again.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub header: u64,
    pub parts: [u64; PARTS],
}

/// The hash of an empty part, as the HUD document starts with.
pub fn empty_part() -> u64 {
    hash(std::iter::empty())
}

/// The rectangles of part `index`.
fn part<T>(rects: &[T], index: usize) -> &[T] {
    let start = (index * PART_RECTS).min(rects.len());
    &rects[start..(start + PART_RECTS).min(rects.len())]
}

/// The plan for `shape`, which is [`view_shape`] of `view`.
pub fn plan(view: &View, shape: &Shape) -> Plan {
    let look = &view.look;
    let header = hash(
        [
            shape.side.to_bits(),
            shape.stroke.to_bits(),
            u32::from(shape.turned),
            u32::from(look.outline),
            u32::from(look.opacity),
            u32::from(look.rgb[0]) << 16 | u32::from(look.rgb[1]) << 8 | u32::from(look.rgb[2]),
        ]
        .into_iter()
        .chain(view.viewport.map(f32::to_bits))
        .chain(
            shape
                .circles
                .iter()
                .flatten()
                .flatten()
                .map(|v| v.to_bits()),
        ),
    );
    let parts = std::array::from_fn(|index| {
        let rects = part(&shape.rects, index);
        if rects.is_empty() {
            return empty_part();
        }
        hash(
            [
                view_scale(view).to_bits(),
                u32::from(look.outline),
                shape.stroke.to_bits(),
            ]
            .into_iter()
            .chain(rects.iter().flatten().map(|v| v.to_bits())),
        )
    });
    Plan { header, parts }
}

/// About how much fuel a header takes to write: some, and more for each circle point.
pub fn header_cost(shape: &Shape) -> u64 {
    let points: usize = shape.circles.iter().map(Vec::len).sum();
    20_000 + 550 * points as u64
}

/// About how much fuel part `index` of `shape` takes to write, measured under wasmtime's fuel
/// count with some to spare.
pub fn part_cost(shape: &Shape, index: usize) -> u64 {
    26_000 + 3_500 * part(&shape.rects, index).len() as u64
}

/// About how much fuel working out a shape and its plan takes, before it is known: finding the
/// rectangles of a large drawing is most of it.
pub fn plan_cost(look: &Crosshair) -> u64 {
    if look.style == CUSTOM {
        20_000 + rects_cost(&look.pixels)
    } else {
        20_000
    }
}

/// A patch showing `view`, or hiding the crosshair for `None`, whole: every part, then the
/// header. For checks and tests; the HUD is sent the steps one a frame, in this order.
pub fn patch(view: Option<&View>) -> String {
    let Some(view) = view else {
        return visibility(false);
    };
    // In the order the HUD is sent them: the parts, then the header that places them.
    let shape = view_shape(view);
    let mut patches: Vec<String> = (0..PARTS)
        .map(|index| part_patch(view, &shape, index))
        .collect();
    patches.push(header_patch(view, &shape));
    patches.join("\n")
}

/// A patch only showing or hiding the crosshair already defined.
pub fn visibility(visible: bool) -> String {
    format!(r##"{{"binding_changes":{{"#crosshair_visible":{visible}}}}}"##)
}

/// Writes `{"name@boldy_hud.template":{"offset":[x,y],"size":[w,h]}},` for `rect`, in physical
/// pixels: the pieces' panel is scaled down by the GUI scale (see [`header_patch`]).
fn piece(out: &mut String, name: char, index: usize, template: &str, rect: [f32; 4]) {
    let [x0, y0, x1, y1] = rect;
    out.push_str("{\"");
    out.push(name);
    digits(out, index as u64);
    out.push_str("@boldy_hud.");
    out.push_str(template);
    out.push_str(r#"":{"offset":["#);
    fixed(out, x0);
    out.push(',');
    fixed(out, y0);
    out.push_str(r#"],"size":["#);
    fixed(out, x1 - x0);
    out.push(',');
    fixed(out, y1 - y0);
    out.push_str("]}},");
}

/// The patch defining the crosshair's box (size, place, turn), its rings and colours, and
/// showing it. Its rectangles are in its parts ([`part_patch`]).
pub fn header_patch(view: &View, shape: &Shape) -> String {
    let [width, height, _] = view.viewport;
    let scale = view_scale(view);
    let mut out = String::with_capacity(1500);
    out.push_str(r##"{"controls":{"xhair":{"type":"panel","size":["##);
    let size = shape.side / scale;
    fixed(&mut out, size);
    out.push(',');
    fixed(&mut out, size);
    // Moves the box's corner onto a whole pixel, wherever the screen's middle falls.
    out.push_str(r##"],"offset":["##);
    for (index, length) in [width, height].into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        let middle = length * scale / 2.0;
        fixed(&mut out, (middle.round() - middle) / scale);
    }
    out.push_str(r##"],"anchor_from":"center","anchor_to":"center","layer":2,"bindings":[{"binding_name":"#crosshair_visible","binding_name_override":"#visible"}],"controls":["##);
    let outline = view.look.outline;
    if !shape.circles.is_empty() {
        let mut paths = String::new();
        contours(&mut paths, &shape.circles);
        let mut view_box = String::from(r##","view_box":[0,0,"##);
        digits(&mut view_box, shape.side as u64);
        view_box.push(',');
        digits(&mut view_box, shape.side as u64);
        view_box.push(']');
        if outline {
            // Centred on the fill's edge, half of the stroke is hidden under it.
            out.push_str(r##"{"ring_edge":{"type":"custom","renderer":"cinnabar_vector_path","size":["100%","100%"],"layer":1,"closed":true,"stroke_width":"##);
            digits(&mut out, 2 * shape.stroke as u64);
            out.push_str(&view_box);
            out.push_str(r##","paths":"##);
            out.push_str(&paths);
            out.push_str(r##","bindings":[{"binding_name":"#crosshair_edge_color","binding_name_override":"#color"}]}},"##);
        }
        out.push_str(r##"{"ring_fill":{"type":"custom","renderer":"cinnabar_vector_path","size":["100%","100%"],"layer":2"##);
        out.push_str(&view_box);
        out.push_str(r##","paths":"##);
        out.push_str(&paths);
        out.push_str(r##","bindings":[{"binding_name":"#crosshair_color","binding_name_override":"#color"}]}},"##);
    }
    // The rectangles are laid out in physical pixels, as whole numbers, on a panel as many GUI
    // units wide as the box is pixels, drawn scaled down by the GUI scale about its middle (the
    // box's middle). Laid out in GUI units instead, each piece's fractional edges were rounded
    // to the screen's pixels on their own, and pieces gained or lost a pixel. The X is turned
    // here too. (The rings stay outside: a path is drawn at its laid-out size before the scale,
    // and a larger one would not be drawn at all.)
    out.push_str(r##"{"pixels":{"type":"panel","size":["##);
    digits(&mut out, shape.side as u64);
    out.push(',');
    digits(&mut out, shape.side as u64);
    out.push_str(
        r##"],"anchor_from":"center","anchor_to":"center","cinnabar_transform":{"scale":"##,
    );
    out.push_str(&(1.0 / f64::from(scale).clamp(0.25, 64.0)).to_string());
    if shape.turned {
        out.push_str(r##","rotation":45"##);
    }
    out.push_str(r##","origin":[0.5,0.5]},"controls":["##);
    for index in 0..PARTS {
        out.push_str("{\"p");
        digits(&mut out, index as u64);
        out.push_str("@boldy_hud.xpart");
        digits(&mut out, index as u64);
        out.push_str("\":{}},");
    }
    out.pop();
    out.push_str("]}}");
    let alpha = f32::from(view.look.opacity.clamp(MIN_OPACITY, 100)) / 100.0;
    let [r, g, b] = view.look.rgb.map(|c| f32::from(c) / 255.0);
    out.push_str(r##"]}},"binding_changes":{"#crosshair_visible":true,"#crosshair_color":"##);
    colour(&mut out, [r, g, b, alpha]);
    out.push_str(r##","#crosshair_edge_color":"##);
    colour(&mut out, [0.0, 0.0, 0.0, alpha]);
    out.push_str("}}");
    out
}

/// The patch defining part `index` of the crosshair's rectangles, each over its outline.
pub fn part_patch(view: &View, shape: &Shape, index: usize) -> String {
    let rects = part(&shape.rects, index);
    let mut out = String::with_capacity(100 + 220 * rects.len());
    out.push_str(r##"{"controls":{"xpart"##);
    digits(&mut out, index as u64);
    out.push_str(r##"":{"type":"panel","size":["100%","100%"],"controls":["##);
    let o = shape.stroke;
    for (at, &[x0, y0, x1, y1]) in rects.iter().enumerate() {
        if view.look.outline {
            piece(&mut out, 'e', at, "xe", [x0 - o, y0 - o, x1 + o, y1 + o]);
        }
        piece(&mut out, 'f', at, "xf", [x0, y0, x1, y1]);
    }
    if out.ends_with(',') {
        out.pop();
    }
    out.push_str("]}}}");
    out
}

/// The drawing grid's side in GUI units, as the menu embeds it.
pub const GRID_VIEW: f32 = 224.0;

/// The plan for showing `drawing` in `rgb` on the menu's drawing grid: a header (its lines and
/// colour) and its rectangles in parts.
pub fn grid_plan(drawing: &Drawing, rgb: [u8; 3]) -> Plan {
    let rects = rects(drawing);
    let size = drawing.size() as u32;
    let header = hash(
        [
            size,
            u32::from(rgb[0]) << 16 | u32::from(rgb[1]) << 8 | u32::from(rgb[2]),
        ]
        .into_iter(),
    );
    let parts = std::array::from_fn(|index| {
        let cells = part(&rects[..rects.len().min(MAX_RECTS)], index);
        if cells.is_empty() {
            return empty_part();
        }
        hash(std::iter::once(size).chain(cells.iter().map(|r| u32::from_le_bytes(*r))))
    });
    Plan { header, parts }
}

/// The patch showing the grid's lines for `drawing`'s resolution, and its colour.
pub fn grid_header_patch(drawing: &Drawing, rgb: [u8; 3]) -> String {
    let mut out = String::with_capacity(220);
    out.push_str(r##"{"controls":{"gridlines":{"type":"panel","size":["100%","100%"],"layer":5,"controls":[{"l@boldy_hud.lines"##);
    digits(&mut out, drawing.size() as u64);
    out.push_str(r##"":{}}]}},"binding_changes":{"#drawgrid_color":"##);
    let [r, g, b] = rgb.map(|c| f32::from(c) / 255.0);
    colour(&mut out, [r, g, b, 1.0]);
    out.push_str("}}");
    out
}

/// About how much fuel the grid's header takes to write: its lines are fixed, so little.
pub fn grid_header_cost() -> u64 {
    20_000
}

/// About how much fuel working out the grid's plan for `drawing` takes, before it is known.
pub fn grid_plan_cost(drawing: &Drawing) -> u64 {
    20_000 + rects_cost(drawing)
}

/// About how much fuel a part of the grid takes to write.
pub fn grid_part_cost(drawing: &Drawing, index: usize) -> u64 {
    let rects = rects(drawing);
    26_000 + 2_500 * part(&rects[..rects.len().min(MAX_RECTS)], index).len() as u64
}

/// The patch showing part `index` of `drawing`'s rectangles on the grid, in GUI units.
pub fn grid_part_patch(drawing: &Drawing, index: usize) -> String {
    let rects = rects(drawing);
    let cells = part(&rects[..rects.len().min(MAX_RECTS)], index);
    // 14, 7 and 3.5 units a pixel at 16, 32 and 64 pixels.
    let cell = GRID_VIEW / drawing.size() as f32;
    let mut out = String::with_capacity(100 + 70 * cells.len());
    out.push_str(r##"{"controls":{"gcells"##);
    digits(&mut out, index as u64);
    out.push_str(r##"":{"type":"panel","size":["100%","100%"],"layer":3,"controls":["##);
    for (at, &[c0, r0, c1, r1]) in cells.iter().enumerate() {
        if at > 0 {
            out.push(',');
        }
        let units = |out: &mut String, n: u8| fixed(out, f32::from(n) * cell);
        out.push_str("{\"p");
        digits(&mut out, at as u64);
        out.push_str(r#"@boldy_hud.gp":{"offset":["#);
        units(&mut out, c0);
        out.push(',');
        units(&mut out, r0);
        out.push_str(r#"],"size":["#);
        units(&mut out, c1 - c0);
        out.push(',');
        units(&mut out, r1 - r0);
        out.push_str("]}}");
    }
    out.push_str("]}}}");
    out
}

/// A patch moving the highlight to pixel `cell` (row, column) of a grid `size` pixels a side,
/// and with `mirror` to its reflections too; `None` hides it.
pub fn grid_cursor_patch(size: usize, cell: Option<(usize, usize)>, mirror: bool) -> String {
    let side = GRID_VIEW / size.max(1) as f32;
    let mut out = String::with_capacity(400);
    out.push_str(r##"{"binding_changes":{"#drawgrid_cursor_size":"##);
    fixed(&mut out, side);
    let last = size.saturating_sub(1);
    let cells = cell.map(|(row, col)| {
        [
            (row, col),
            (row, last - col.min(last)),
            (last - row.min(last), col),
            (last - row.min(last), last - col.min(last)),
        ]
    });
    for index in 0..4 {
        let at = cells
            .filter(|_| index == 0 || mirror)
            .map(|cells| cells[index]);
        out.push_str(r##","#drawgrid_cursor"##);
        digits(&mut out, index as u64);
        out.push_str(r##"_visible":"##);
        out.push_str(if at.is_some() { "true" } else { "false" });
        if let Some((row, col)) = at {
            out.push_str(r##","#drawgrid_cursor"##);
            digits(&mut out, index as u64);
            out.push_str(r##"_offset":["##);
            fixed(&mut out, col as f32 * side);
            out.push(',');
            fixed(&mut out, row as f32 * side);
            out.push(']');
        }
    }
    out.push_str("}}");
    out
}

/// The grid whole, header and every part: for checks and tests.
pub fn grid_patch(drawing: &Drawing, rgb: [u8; 3]) -> String {
    let mut patches = vec![grid_header_patch(drawing, rgb)];
    patches.extend((0..PARTS).map(|index| grid_part_patch(drawing, index)));
    patches.join("\n")
}

fn colour(out: &mut String, values: [f32; 4]) {
    out.push('[');
    for (index, value) in values.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        fixed(out, value);
    }
    out.push(']');
}

/// The longest a point can be written: `[9999.9,9999.9],`.
const POINT_BYTES: usize = 16;

/// Writes `paths` as JSON, each coordinate to a tenth of a pixel. A frame has little fuel and the
/// circles have many points, so everything is written by index into one buffer sized up front,
/// with integer arithmetic only.
fn contours(out: &mut String, paths: &[Vec<[f32; 2]>]) {
    let points: usize = paths.iter().map(Vec::len).sum();
    let mut bytes = vec![0u8; points * POINT_BYTES + paths.len() * 3 + 2];
    let mut at = 0;
    bytes[at] = b'[';
    at += 1;
    for (index, path) in paths.iter().enumerate() {
        if index > 0 {
            bytes[at] = b',';
            at += 1;
        }
        bytes[at] = b'[';
        at += 1;
        for (point, &[x, y]) in path.iter().enumerate() {
            if point > 0 {
                bytes[at] = b',';
                at += 1;
            }
            bytes[at] = b'[';
            at += 1;
            put_tenths(&mut bytes, &mut at, x);
            bytes[at] = b',';
            at += 1;
            put_tenths(&mut bytes, &mut at, y);
            bytes[at] = b']';
            at += 1;
        }
        bytes[at] = b']';
        at += 1;
    }
    bytes[at] = b']';
    bytes.truncate(at + 1);
    out.push_str(std::str::from_utf8(&bytes).unwrap_or("[]"));
}

/// Writes `value` to a tenth at `at`. Points are measured from the box's corner, so never below
/// 0 and well under 10,000; anything else (NaN included) is written as the nearest of those.
fn put_tenths(bytes: &mut [u8], at: &mut usize, value: f32) {
    // A saturating conversion, rounding half up: no call to a rounding function.
    let n = ((value * 10.0 + 0.5) as u32).min(99_999);
    let (whole, tenth) = (n / 10, n % 10);
    for (place, shown) in [
        (1000, whole >= 1000),
        (100, whole >= 100),
        (10, whole >= 10),
    ] {
        if shown {
            bytes[*at] = b'0' + (whole / place % 10) as u8;
            *at += 1;
        }
    }
    bytes[*at] = b'0' + (whole % 10) as u8;
    *at += 1;
    if tenth != 0 {
        bytes[*at] = b'.';
        bytes[*at + 1] = b'0' + tenth as u8;
        *at += 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look(style: u8) -> Crosshair {
        Crosshair {
            on: true,
            style,
            ..DEFAULT
        }
    }

    /// A drawing of grid `size` from rows given as bits.
    fn drawn(size: u8, rows: &[u64]) -> Drawing {
        let mut out = Drawing::empty(size);
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..64 {
                if bits >> col & 1 == 1 {
                    out = out.painted(row, col, true, false);
                }
            }
        }
        out
    }

    /// Whether `point` is filled by the even-odd rule over `paths`.
    fn filled(paths: &[Vec<[f32; 2]>], [px, py]: [f32; 2]) -> bool {
        let mut inside = false;
        for path in paths {
            let n = path.len();
            for i in 0..n {
                let ([x0, y0], [x1, y1]) = (path[i], path[(i + 1) % n]);
                if (y0 > py) != (y1 > py) && px < x0 + (py - y0) / (y1 - y0) * (x1 - x0) {
                    inside = !inside;
                }
            }
        }
        inside
    }

    /// Whether `shape` covers `point`, turned back for an X.
    fn covers(shape: &Shape, point: [f32; 2]) -> bool {
        let [mut x, mut y] = point;
        if shape.turned {
            let (m, turn) = (shape.side / 2.0, std::f32::consts::FRAC_1_SQRT_2);
            let (dx, dy) = (x - m, y - m);
            (x, y) = (m + (dx + dy) * turn, m + (dy - dx) * turn);
        }
        shape
            .rects
            .iter()
            .any(|&[x0, y0, x1, y1]| x >= x0 && x < x1 && y >= y0 && y < y1)
            || filled(&shape.circles, [x, y])
    }

    /// The pixel centres the shape covers, as offsets from the crosshair's middle pixel.
    fn pixels(shape: &Shape) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        let side = shape.side as i32;
        let middle = side / 2;
        for y in 0..side {
            for x in 0..side {
                if covers(shape, [x as f32 + 0.5, y as f32 + 0.5]) {
                    out.push((x - middle, y - middle));
                }
            }
        }
        out
    }

    #[test]
    fn a_cross_covers_whole_pixels_with_a_gap_in_the_middle() {
        // Scale 1: thickness 1, gap 2 and length 4 pixels.
        let shape = shape(&look(0), 1.0);
        let covered = pixels(&shape);
        // Four arms of four pixels each, one pixel thick, after two empty pixels.
        assert_eq!(covered.len(), 16, "{covered:?}");
        for d in 3..7 {
            for p in [(d, 0), (-d, 0), (0, d), (0, -d)] {
                assert!(covered.contains(&p), "{p:?} in {covered:?}");
            }
        }
        assert!(!covered.contains(&(0, 0)) && !covered.contains(&(2, 0)));
        assert_eq!(shape.side as u32 % 2, 0);
        // Every edge is on a whole pixel.
        for rect in &shape.rects {
            assert_eq!(rect.map(f32::fract), [0.0; 4], "{rect:?}");
        }
    }

    #[test]
    fn arms_that_meet_make_one_solid_plus() {
        let solid = Crosshair {
            gap: 0.0,
            thickness: 1.0,
            length: 3.0,
            ..look(0)
        };
        let shape = shape(&solid, 1.0);
        let covered = pixels(&shape);
        assert!(covered.contains(&(0, 0)));
        // Three pixels out on every side, plus the middle: 13.
        assert_eq!(covered.len(), 13, "{covered:?}");
    }

    #[test]
    fn a_t_has_no_top_arm_and_a_dot_sits_in_the_gap() {
        let t = pixels(&shape(&look(T_SHAPE), 1.0));
        assert_eq!(t.len(), 12);
        assert!(t.iter().all(|&(_, y)| y >= 0), "{t:?}");
        let dotted = pixels(&shape(&look(CROSS_DOT), 1.0));
        assert_eq!(dotted.len(), 17);
        assert!(dotted.contains(&(0, 0)));
    }

    #[test]
    fn circles_are_rings_with_room_round_the_dot() {
        // Scale 2: thickness 2, so the ring runs from 8 to 10 pixels out and the dot is 4 wide.
        let ring = pixels(&shape(&look(CIRCLE), 2.0));
        assert!(!ring.contains(&(0, 0)), "a ring is empty in the middle");
        let dotted = pixels(&shape(&look(CIRCLE_DOT), 2.0));
        assert!(dotted.contains(&(0, 0)), "the dot fills the middle");
        for (near, far) in [
            ((4, 0), (8, 0)),
            ((-5, 0), (-9, 0)),
            ((0, 4), (0, 8)),
            ((0, -5), (0, -9)),
        ] {
            assert!(
                !dotted.contains(&near),
                "free between dot and ring at {near:?}"
            );
            assert!(
                dotted.contains(&far) && ring.contains(&far),
                "ring at {far:?}"
            );
        }
        let dot = pixels(&shape(&look(DOT), 2.0));
        assert!(dot.contains(&(0, 0)) && dot.len() >= 4 && !dot.contains(&(4, 0)));
    }

    #[test]
    fn an_x_is_a_cross_turned_and_everything_fits_its_box() {
        let x = shape(&look(X), 2.0);
        let covered = pixels(&x);
        assert!(
            covered.iter().any(|&(a, b)| a > 3 && b > 3),
            "diagonal arms"
        );
        assert!(
            !covered.iter().any(|&(a, b)| b == 0 && a > 4),
            "no straight arms"
        );
        assert!(
            x.turned
                && patch(Some(&View {
                    look: look(X),
                    viewport: [400.0, 300.0, 2.0]
                }))
                .contains(r#","rotation":45,"origin":[0.5,0.5]}"#)
        );
        let full = Drawing::empty(64).resized(64);
        let mut big_drawing = full;
        for row in 0..64 {
            big_drawing = big_drawing.painted(row, row, true, true);
        }
        for style in 0..STYLES.len() as u8 {
            for scale in [1.0, 2.0, 3.0, 4.5] {
                let big = Crosshair {
                    length: LENGTH.max,
                    thickness: THICKNESS.max,
                    gap: GAP.max,
                    pixels: big_drawing,
                    pixel_size: MAX_PIXEL_SIZE,
                    ..look(style)
                };
                for look in [look(style), big] {
                    let shape = shape(&look, scale);
                    // The outline reaches `stroke` further out, and has to fit too.
                    let name = STYLES[usize::from(style)];
                    let inside = |v: f32| v >= shape.stroke && v <= shape.side - shape.stroke;
                    for point in shape.circles.iter().flatten() {
                        assert!(
                            point.iter().all(|v| inside(*v)),
                            "{name} at {scale}: {point:?}"
                        );
                    }
                    for rect in &shape.rects {
                        assert!(
                            rect.iter().all(|v| inside(*v)),
                            "{name} at {scale}: {rect:?}"
                        );
                        assert!(rect[2] > rect[0] && rect[3] > rect[1], "{rect:?}");
                    }
                    // Cinnabar draws nothing of a wider path, outline included.
                    for point in shape.circles.iter().flatten() {
                        for v in point {
                            let from_middle = (v - shape.centre).abs() + shape.stroke;
                            assert!(from_middle * 2.0 <= MAX_PATH, "{name} at {scale}: {v}");
                        }
                    }
                    assert!(shape.circles.iter().all(|path| path.len() >= 3));
                }
            }
        }
    }

    #[test]
    fn values_step_and_clamp() {
        assert_eq!(LENGTH.stepped(4.0, 1.0), 4.5);
        assert_eq!(LENGTH.stepped(LENGTH.max, 1.0), LENGTH.max);
        assert_eq!(GAP.stepped(0.0, -1.0), 0.0);
        assert_eq!(THICKNESS.clamp(f32::NAN), THICKNESS.min);
        assert_eq!(THICKNESS.clamp(1.26), 1.5);
        assert_eq!(step_opacity(100, true), 100);
        assert_eq!(step_opacity(100, false), 90);
        assert_eq!(step_opacity(MIN_OPACITY, false), MIN_OPACITY);
        assert_eq!(clamp_opacity(47.0), 50);
        assert_eq!(step_style(0, false), STYLES.len() as u8 - 1);
        assert_eq!(step_style(STYLES.len() as u8 - 1, true), 0);
        for style in 0..STYLES.len() as u8 {
            assert_eq!(style_from_key(style_key(style)), Some(style));
        }
        assert_eq!(style_from_key(" X "), Some(X));
        assert_eq!(style_from_key("star"), None);
        assert_eq!(step_resolution(16, true), 32);
        assert_eq!(step_resolution(64, true), 64);
        assert_eq!(step_resolution(32, false), 16);
        assert_eq!(step_resolution(16, false), 16);
        assert_eq!(clamp_pixel_size(0.0), MIN_PIXEL_SIZE);
        assert_eq!(clamp_pixel_size(99.0), MAX_PIXEL_SIZE);
        assert_eq!(clamp_pixel_size(f32::NAN), DEFAULT_PIXEL_SIZE);
        assert_eq!(Drawing::empty(7).size(), 16);
    }

    /// The number after `"name":` (or after `"name":[` and `skip` commas) in a patch.
    fn number(patch: &str, name: &str, skip: usize) -> f32 {
        let at = patch.find(&format!("\"{name}\":")).expect(name) + name.len() + 3;
        let rest = patch[at..].trim_start_matches('[');
        let item = rest.split(',').nth(skip).unwrap();
        item.trim_end_matches([']', '}']).parse().expect(item)
    }

    fn balanced(json: &str) {
        for line in json.lines() {
            for (open, close) in [('{', '}'), ('[', ']')] {
                assert_eq!(
                    line.matches(open).count(),
                    line.matches(close).count(),
                    "{line}"
                );
            }
            assert!(!line.contains(",]") && !line.contains(",}") && !line.contains("NaN"));
        }
    }

    /// Every `{"offset":[x,y],"size":[w,h]}` piece of `template` in a patch, as `[x, y, w, h]`.
    fn pieces(patch: &str, template: &str) -> Vec<[f32; 4]> {
        let marker = format!("@boldy_hud.{template}\":{{\"offset\":[");
        patch
            .match_indices(&marker)
            .map(|(at, _)| {
                let rest = &patch[at + marker.len()..];
                let numbers: Vec<f32> = rest[..rest.find("]}").unwrap()]
                    .replace("],\"size\":[", ",")
                    .split(',')
                    .map(|n| n.parse().unwrap())
                    .collect();
                [numbers[0], numbers[1], numbers[2], numbers[3]]
            })
            .collect()
    }

    #[test]
    fn patches_define_the_crosshair_on_whole_pixels_and_colour_it() {
        let view = View {
            look: look(CROSS_DOT),
            viewport: [427.0, 240.5, 3.0],
        };
        let shown = patch(Some(&view));
        balanced(&shown);
        assert!(
            shown.starts_with(r##"{"controls":{"xpart0":"##),
            "parts first"
        );
        let header = shown.lines().last().unwrap();
        assert!(header.starts_with(r##"{"controls":{"xhair":{"type":"panel","size":["##));
        assert!(shown.contains(r##""binding_changes":{"#crosshair_visible":true,"##));
        assert!(
            shown.contains(r#"{"p8@boldy_hud.xpart8":{}}"#),
            "every part is placed"
        );
        assert!(
            shown.contains(
                r#""cinnabar_transform":{"scale":0.3333333333333333,"origin":[0.5,0.5]}"#
            ),
            "{shown}"
        );
        let size = number(header, "size", 0);
        // The box's top-left corner lands on a whole pixel.
        for (axis, length) in [427.0f32, 240.5].into_iter().enumerate() {
            let offset = number(header, "offset", axis);
            let corner = (length - size) / 2.0 * 3.0 + offset * 3.0;
            assert!((corner - corner.round()).abs() < 0.02, "{corner}");
        }
        // Four arms and the dot, each over a black piece larger all round by the outline.
        let fills = pieces(&shown, "xf");
        let edges = pieces(&shown, "xe");
        assert_eq!((fills.len(), edges.len()), (5, 5));
        let pixel = view_shape(&view).stroke;
        for (fill, edge) in fills.iter().zip(&edges) {
            assert!(
                (edge[0] - (fill[0] - pixel)).abs() < 0.01,
                "{fill:?} {edge:?}"
            );
            assert!(
                (edge[2] - (fill[2] + 2.0 * pixel)).abs() < 0.01,
                "{fill:?} {edge:?}"
            );
            // On whole pixels: the pieces' panel is scaled down by the GUI scale, 3.
            for v in fill {
                assert_eq!(v.fract(), 0.0, "{fill:?}");
            }
        }
        assert!(!shown.contains("vector_path"), "no circles in a cross");
        // A ring and its dot are paths, outline under fill, the same points in both.
        let ring = patch(Some(&View {
            look: look(CIRCLE_DOT),
            viewport: [427.0, 240.5, 3.0],
        }));
        balanced(&ring);
        let edge = ring.find(r#""ring_edge":"#).unwrap();
        let fill = ring.find(r#""ring_fill":"#).unwrap();
        assert!(edge < fill);
        let paths = |from: usize| {
            let at = ring[from..].find(r#""paths":"#).unwrap() + from;
            &ring[at..at + ring[at..].find("]]]").unwrap()]
        };
        assert_eq!(paths(edge), paths(fill));
        assert_eq!(paths(fill).matches("[[").count(), 3);
        // Colours are bound; without an outline there are no black pieces.
        let plain = patch(Some(&View {
            look: Crosshair {
                outline: false,
                opacity: 50,
                rgb: [255, 0, 0],
                ..look(0)
            },
            viewport: [400.0, 300.0, 2.0],
        }));
        balanced(&plain);
        assert!(pieces(&plain, "xe").is_empty(), "no outline: {plain}");
        assert!(
            plain.contains(r##""#crosshair_color":[1,0,0,0.5]"##),
            "{plain}"
        );
        assert_eq!(
            patch(None),
            r##"{"binding_changes":{"#crosshair_visible":false}}"##
        );
        assert_eq!(
            visibility(true),
            r##"{"binding_changes":{"#crosshair_visible":true}}"##
        );
        let mut text = String::new();
        contours(
            &mut text,
            &[
                vec![[0.0, 3.0], [12.25, 107.04]],
                vec![[f32::NAN, -1.0], [0.95, 9.96]],
            ],
        );
        assert_eq!(text, "[[[0,3],[12.3,107]],[[0,0],[1,10]]]");
        text.clear();
        contours(&mut text, &[vec![[123456.0, 9999.94]]]);
        assert_eq!(text, "[[[9999.9,9999.9]]]");
    }

    #[test]
    fn the_default_drawing_is_a_centred_cross() {
        let custom = shape(&look(CUSTOM), 2.0);
        // Four arms, four rectangles, each on whole pixels.
        assert_eq!(rects(&DEFAULT_DRAWING).len(), 4);
        for rect in &custom.rects {
            assert_eq!(rect.map(f32::fract), [0.0; 4], "{rect:?}");
        }
        // Centred: the same on both sides of the middle, along both axes.
        let covered = pixels(&custom);
        assert!(!covered.is_empty());
        for &(x, y) in &covered {
            assert!(
                covered.contains(&(-1 - x, y)) && covered.contains(&(x, -1 - y)),
                "{x},{y}"
            );
        }
        assert!(!covered.contains(&(0, 0)), "an empty middle");
    }

    #[test]
    fn drawings_paint_mirror_merge_resize_and_round_trip() {
        for size in RESOLUTIONS {
            let last = usize::from(size) - 1;
            let drawing = Drawing::empty(size).painted(0, 0, true, true);
            for (row, col) in [(0, 0), (0, last), (last, 0), (last, last)] {
                assert!(drawing.pixel(row, col), "{size}: {row},{col}");
            }
            let one = drawing.painted(0, last, false, false);
            assert!(!one.pixel(0, last) && one.pixel(0, 0));
            assert_eq!(one.painted(last + 1, 3, true, true), one, "off the grid");
            assert_eq!(Drawing::parse(&one.text(), size), one);
        }
        // A filled block of rows with the same run is one rectangle; a ring is four.
        let block = drawn(16, &[0, 0, 0, 0b1110, 0b1110, 0b1110, 0b1110]);
        assert_eq!(*rects(&block), vec![[1, 3, 4, 7]]);
        let ring = drawn(
            16,
            &[0, 0, 0b1111100, 0b1000100, 0b1000100, 0b1000100, 0b1111100],
        );
        assert_eq!(rects(&ring).len(), 4);
        // Pixels drawn are exactly the rectangles' cells.
        let covered: usize = rects(&ring)
            .iter()
            .map(|[c0, r0, c1, r1]| usize::from(c1 - c0) * usize::from(r1 - r0))
            .sum();
        assert_eq!(covered, 16);
        // A run at the grid's far edge, at every size.
        let edge = drawn(64, &[u64::MAX, u64::MAX]);
        assert_eq!(*rects(&edge), vec![[0, 0, 64, 2]]);
        // The older form, rows of `.` and `#`, is read into the smallest grid.
        assert_eq!(
            Drawing::parse("#.x/..1/junk", 16),
            drawn(16, &[0b101, 0b100])
        );
        assert_eq!(Drawing::parse("7c3e", 16), drawn(16, &[0x7C3E]));
        // Eight digits at a time: every byte value, alone among digits, and the digits' order.
        for byte in 0..=u8::MAX {
            let mut word = *b"0123abcd";
            for place in 0..8 {
                word[place] = byte;
                let spelled = u64::from_le_bytes(word);
                assert_eq!(hex8(spelled).is_some(), byte.is_ascii_hexdigit(), "{byte:#x}");
                word = *b"0123abcd";
            }
        }
        assert_eq!(hex8(u64::from_le_bytes(*b"89abCDEF")), Some(0x89AB_CDEF));
        assert_eq!(hex8(u64::from_le_bytes(*b"00000001")), Some(1));
        assert_eq!(hex8(u64::from_le_bytes(*b"10000000")), Some(0x1000_0000));
        // The one-pass reader and the general one read the same text the same way.
        for (text, size) in [
            ("", 16),
            ("7c3e/0/FFFF/ffff/1/2", 16),
            ("7c3e/ 7c3e", 16),
            ("fffffffffffffffff/1", 16),
            ("1fffffffffffffff/2", 16),
            ("12345678/1/", 32),
            ("////", 16),
            ("1/2/3/4/5/6/7/8/9/a/b/c/d/e/f/10/11/12", 16),
            ("-1/+2/0x3/4 /g", 16),
            ("0123456789abcdef/FEDCBA9876543210/00000000ffffffff/0123456789abcdeg", 64),
            ("0123456789abcdef0/1", 64),
            ("0123456789abcdef/", 64),
        ] {
            assert_eq!(
                Drawing::parse(text, size),
                Drawing::parse_general(text, size),
                "{text}"
            );
        }
        // The README's example is the default drawing, written as the settings file writes it.
        assert_eq!(
            DEFAULT_DRAWING.text(),
            "0000/0180/0180/0180/0180/0180/0000/7c3e/7c3e/0000/0180/0180/0180/0180/0180/0000"
        );
        // Resizing keeps the shape: up and back down is the same drawing.
        for size in [32, 64] {
            let bigger = DEFAULT_DRAWING.resized(size);
            assert_eq!(bigger.size(), usize::from(size));
            assert_eq!(rects(&bigger).len(), 4);
            assert_eq!(bigger.resized(16), DEFAULT_DRAWING);
        }
        let lone = drawn(64, &[0, 0, 0, 1 << 5]);
        assert_eq!(lone.resized(16), drawn(16, &[1 << 1]));
        // A pixel size follows, so the crosshair keeps its size on screen.
        let finer = with_resolution(&look(CUSTOM), 32);
        assert_eq!((finer.pixels.size(), finer.pixel_size), (32, 1));
        // A checkerboard is far too detailed; a hand-edited one is drawn only in part.
        let checks = drawn(
            64,
            &[0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA].repeat(32),
        );
        assert!(too_detailed(&checks) && !too_detailed(&DEFAULT_DRAWING));
        let busy = Crosshair {
            pixels: checks,
            ..look(CUSTOM)
        };
        assert_eq!(shape(&busy, 2.0).rects.len(), MAX_RECTS);
        // Stripes cost little (a row like the one above is free); stripes changing every row
        // are too many runs to work out in a frame, though few rectangles.
        let stripes = drawn(64, &[0x5555_5555_5555_5555; 64]);
        assert_eq!(runs(&stripes), 32);
        assert!(!too_detailed(&stripes) && rects(&stripes).len() == 32);
        let shifting: Vec<u64> = (0..64)
            .map(|row| 0x5555_5555_5555_5555u64 ^ (1 << (2 * (row % 32))))
            .collect();
        let shifting = drawn(64, &shifting);
        assert!(runs(&shifting) > MAX_RUNS && too_detailed(&shifting));
        // Estimated costs grow with the work, and a known drawing costs nothing more.
        let _ = rects(&DEFAULT_DRAWING);
        assert_eq!(rects_cost(&DEFAULT_DRAWING), 0);
        assert!(rects_cost(&shifting) > rects_cost(&stripes));
    }

    /// Resizing pixel by pixel, as it is meant to work, to check the fast way against.
    fn resized_slowly(drawing: &Drawing, size: u8) -> Drawing {
        let mut out = Drawing::empty(size);
        let (from, to) = (drawing.size(), out.size());
        for row in 0..to {
            for col in 0..to {
                let drawn = if to >= from {
                    let f = to / from;
                    drawing.pixel(row / f, col / f)
                } else {
                    let f = from / to;
                    (0..f).any(|r| (0..f).any(|c| drawing.pixel(row * f + r, col * f + c)))
                };
                if drawn {
                    out = out.painted(row, col, true, false);
                }
            }
        }
        out
    }

    #[test]
    fn resizing_row_by_row_is_resizing_pixel_by_pixel() {
        // Drawings with pixels everywhere, edges and corners included, at every size.
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        for from in RESOLUTIONS {
            for _ in 0..6 {
                let mut drawing = Drawing::empty(from);
                for row in 0..usize::from(from) {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    for col in 0..usize::from(from) {
                        if seed >> col & 1 == 1 {
                            drawing = drawing.painted(row, col, true, false);
                        }
                    }
                }
                for to in RESOLUTIONS {
                    assert_eq!(
                        drawing.resized(to),
                        resized_slowly(&drawing, to),
                        "{from} to {to}"
                    );
                }
                assert_eq!(Drawing::parse(&drawing.text(), from), drawing);
            }
        }
        assert_eq!(Drawing::empty(64).text().len(), 64 * 17 - 1);
    }

    #[test]
    fn a_fast_stroke_leaves_no_gaps() {
        let line = Drawing::empty(64).painted_line((2, 3), (9, 40), true, false);
        // One pixel in every column it crosses, and its ends.
        for col in 3..=40 {
            assert!((0..64).any(|row| line.pixel(row, col)), "column {col}");
        }
        assert!(line.pixel(2, 3) && line.pixel(9, 40));
        let steep = Drawing::empty(16).painted_line((15, 1), (0, 4), true, true);
        for row in 0..16 {
            assert!((0..16).any(|col| steep.pixel(row, col)), "row {row}");
        }
        // Mirrored like a single pixel, and a line of one pixel is that pixel.
        assert!(steep.pixel(15, 14) && steep.pixel(0, 1));
        assert_eq!(
            Drawing::empty(32).painted_line((5, 6), (5, 6), true, false),
            Drawing::empty(32).painted(5, 6, true, false)
        );
    }

    #[test]
    fn large_drawings_are_sent_in_parts_and_only_what_changed() {
        // Forty separate pixels: three parts.
        let mut drawing = Drawing::empty(64);
        for index in 0..40 {
            drawing = drawing.painted(index, (index * 3) % 64, true, false);
        }
        let view = View {
            look: Crosshair {
                pixels: drawing,
                ..look(CUSTOM)
            },
            viewport: [400.0, 300.0, 2.0],
        };
        let shape = view_shape(&view);
        assert_eq!(shape.rects.len(), 40);
        let plan = plan(&view, &shape);
        assert!(plan.parts[..3].iter().all(|part| *part != empty_part()));
        assert!(plan.parts[3..].iter().all(|part| *part == empty_part()));
        for index in 0..PARTS {
            let patch = part_patch(&view, &shape, index);
            balanced(&patch);
            let expected = part(&shape.rects, index).len();
            assert_eq!(pieces(&patch, "xf").len(), expected, "part {index}");
            assert!(part_cost(&shape, index) >= 20_000);
        }
        // A pixel more changes the part it falls in, and not the header.
        let more = View {
            look: Crosshair {
                pixels: drawing.painted(63, 0, true, false),
                ..view.look
            },
            ..view
        };
        let more_plan = plan_of(&more);
        assert_eq!(more_plan.header, plan.header);
        assert_ne!(more_plan.parts, plan.parts);
        // A colour changes the header only.
        let red = View {
            look: Crosshair {
                rgb: [255, 0, 0],
                ..view.look
            },
            ..view
        };
        let red_plan = plan_of(&red);
        assert_ne!(red_plan.header, plan.header);
        assert_eq!(red_plan.parts, plan.parts);
    }

    fn plan_of(view: &View) -> Plan {
        plan(view, &view_shape(view))
    }

    #[test]
    fn an_empty_drawing_is_an_empty_box_and_the_grid_shows_the_pixels() {
        let empty = View {
            look: Crosshair {
                pixels: Drawing::empty(16),
                ..look(CUSTOM)
            },
            viewport: [400.0, 300.0, 2.0],
        };
        let shown = patch(Some(&empty));
        balanced(&shown);
        assert!(pieces(&shown, "xf").is_empty(), "{shown}");
        assert!(
            plan_of(&empty)
                .parts
                .iter()
                .all(|part| *part == empty_part())
        );
        // The grid's cells: four arms, in 12-unit squares at 16, in the drawing's colour.
        let grid = grid_patch(&DEFAULT_DRAWING, [255, 0, 0]);
        balanced(&grid);
        assert!(grid.starts_with(r##"{"controls":{"gridlines":{"##));
        assert!(grid.contains("@boldy_hud.lines16"));
        let cells = pieces(&grid, "gp");
        assert_eq!(cells.len(), 4, "{grid}");
        assert!(
            cells.contains(&[14.0, 98.0, 70.0, 28.0]),
            "left arm: {cells:?}"
        );
        assert!(grid.contains(r##""#drawgrid_color":[1,0,0,1]"##));
        let at64 = grid_patch(&DEFAULT_DRAWING.resized(64), [255, 0, 0]);
        assert!(at64.contains("@boldy_hud.lines64"));
        assert!(
            pieces(&at64, "gp").contains(&[14.0, 98.0, 70.0, 28.0]),
            "same place at 64"
        );
        // The highlight: on the pixel under the pointer, and with Mirror on its reflections.
        let one = grid_cursor_patch(64, Some((1, 2)), false);
        balanced(&one);
        assert!(one.contains(r##""#drawgrid_cursor_size":3.5"##), "{one}");
        assert!(
            one.contains(r##""#drawgrid_cursor0_offset":[7,3.5]"##),
            "{one}"
        );
        assert!(
            one.contains(r##""#drawgrid_cursor1_visible":false"##),
            "{one}"
        );
        let four = grid_cursor_patch(16, Some((0, 0)), true);
        assert!(
            four.contains(r##""#drawgrid_cursor3_offset":[210,210]"##),
            "{four}"
        );
        let none = grid_cursor_patch(16, None, true);
        assert!(!none.contains("true") && !none.contains("offset"), "{none}");
        let blank = grid_patch(&Drawing::empty(32), [255, 0, 0]);
        balanced(&blank);
        assert!(pieces(&blank, "gp").is_empty());
        assert_ne!(
            grid_plan(&DEFAULT_DRAWING, [1, 2, 3]).header,
            grid_plan(&DEFAULT_DRAWING.resized(32), [1, 2, 3]).header
        );
    }
}
