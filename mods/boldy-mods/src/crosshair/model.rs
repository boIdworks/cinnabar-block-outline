//! The crosshair's styles, drawing grid and settings, and the steps that change them.

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
pub(super) const STYLE_KEYS: [&str; 8] = [
    "cross",
    "cross_dot",
    "t",
    "x",
    "circle",
    "circle_dot",
    "dot",
    "custom",
];

pub(super) const CROSS_DOT: u8 = 1;

pub(super) const T_SHAPE: u8 = 2;

pub(super) const X: u8 = 3;

pub(super) const CIRCLE: u8 = 4;

pub(super) const CIRCLE_DOT: u8 = 5;

pub(super) const DOT: u8 = 6;

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
#[derive(Clone, Copy, Debug, Eq)]
pub struct Drawing {
    size: u8,
    pub(super) rows: [u64; MAX_GRID],
}

impl PartialEq for Drawing {
    /// Compared row by row, and only the rows in use: the derived comparison of the whole array is
    /// a byte-by-byte `bcmp` in the wasm build, which cost about 12,000 fuel every frame the
    /// custom crosshair was up (rows past `size` are always clear, so they cannot differ).
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
            && self
                .rows
                .iter()
                .zip(&other.rows)
                .take(usize::from(self.size))
                .all(|(a, b)| a == b)
    }
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
pub(super) const fn row_mask(size: usize) -> u64 {
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
    pub(super) fn parse_general(text: &str, size: u8) -> Self {
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
pub(super) fn hex8(word: u64) -> Option<u32> {
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
pub(super) fn hex_bits(line: &str) -> Option<u64> {
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
