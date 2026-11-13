//! Jump reset: when you are hit, a jump pressed at the right moment cuts the knockback. This
//! module tells you whether you pressed too early or too late, in milliseconds, and whether the
//! hit actually moved you (a successful reset takes little or no knockback), in a line of text
//! under the crosshair. Host-independent so `cargo test` can check the timing, the movement
//! watch, the combat feed and the patches.
//!
//! The reference point is the moment the game shows you being hurt (the host's `hurt` combat
//! event for the local player), measured in frames, so it is accurate to about a frame. A jump
//! pressed before it is early, one pressed after it is late. Knockback is read from your own
//! velocity: a hit shoves it in one tick by a good deal more than walking or jumping ever does.

use crate::panel::digits;

/// The window around a hit inside which a jump counts as perfect, in milliseconds: the default
/// and the bounds of the menu's - / + buttons.
pub const DEFAULT_WINDOW: u16 = 75;
pub const MIN_WINDOW: u16 = 20;
pub const MAX_WINDOW: u16 = 200;
/// One click of - / +.
pub const WINDOW_STEP: u16 = 5;

/// A jump and a hit further apart than this (seconds) have nothing to do with each other.
pub const RELEVANT: f32 = 0.4;
/// How long a verdict stays on screen, in seconds.
pub const SHOW: f32 = 1.6;

/// How long after a hit your movement is watched for knockback, in seconds.
pub const OBSERVE: f32 = 0.4;
/// A sudden change of your horizontal velocity of at least this much (blocks per tick) counts as
/// knockback, in the file only. A real hit pushes about 0.4; sprint-jumping adds 0.2 by itself.
pub const DEFAULT_KNOCKBACK: f32 = 0.08;
pub const MIN_KNOCKBACK: f32 = 0.01;
pub const MAX_KNOCKBACK: f32 = 0.4;

pub fn clamp_knockback(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(MIN_KNOCKBACK, MAX_KNOCKBACK)
    } else {
        DEFAULT_KNOCKBACK
    }
}

/// Jump reset settings, saved with the other settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JumpReset {
    pub on: bool,
    /// A jump within this many milliseconds of the hit is perfect.
    pub window: u16,
    /// The smallest sudden change of horizontal velocity, in blocks per tick, that counts as
    /// knockback.
    pub knockback: f32,
    /// The last hits and how they went, kept in the settings file so the numbers can be read
    /// back afterwards (to tune the threshold from real fights).
    pub log: Log,
}

impl Default for JumpReset {
    fn default() -> Self {
        Self {
            on: false,
            window: DEFAULT_WINDOW,
            knockback: DEFAULT_KNOCKBACK,
            log: Log::new(),
        }
    }
}

/// How many hits the log keeps.
pub const LOG_LEN: usize = 16;

/// How many jump resets are kept in full, in a list of their own that survives restarts: each is
/// a full recording of the hit (about 250 bytes), and the settings file may hold 16 KiB.
pub const RESETS_KEPT: usize = 24;

/// Adds `row` to `rows`, forgetting the oldest beyond `cap`.
pub fn push_capped(rows: &mut Vec<String>, row: String, cap: usize) {
    rows.push(row);
    let extra = rows.len().saturating_sub(cap);
    rows.drain(..extra);
}

/// `rows` joined by `/`, with the oldest left out until the text is at most `max_bytes` long.
pub fn joined_within(rows: &[String], max_bytes: usize) -> String {
    (0..=rows.len())
        .map(|skip| rows[skip..].join("/"))
        .find(|text| text.len() <= max_bytes)
        .unwrap_or_default()
}

/// One hit, as written to the log.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Row {
    /// When you jumped, in milliseconds from the hit (negative before it); none for no jump.
    pub offset_ms: Option<i16>,
    /// The biggest sudden change of horizontal velocity after the hit, in thousandths of a block
    /// per tick.
    pub knockback: u16,
    /// Whether you were on the ground when hit, if known.
    pub grounded: Option<bool>,
    /// Your horizontal speed just before the hit, in thousandths of a block per tick.
    pub speed: u16,
}

fn thousandths(value: f32) -> u16 {
    let milli = (value * 1000.0).round();
    if milli.is_finite() {
        milli.clamp(0.0, f32::from(u16::MAX)) as u16
    } else {
        0
    }
}

impl Row {
    /// The row for a hit judged `timing` (if you jumped) with this `outcome` and `trace`.
    pub fn new(timing: Option<Timing>, outcome: Outcome, trace: Trace) -> Self {
        Self {
            offset_ms: timing.map(|t| {
                let ms = t.ms as i16;
                if t.before { -ms } else { ms }
            }),
            knockback: thousandths(outcome.knockback),
            grounded: trace.grounded,
            speed: thousandths(trace.speed),
        }
    }
}

/// The last [`LOG_LEN`] hits, oldest first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Log {
    len: u8,
    rows: [Row; LOG_LEN],
}

impl Log {
    pub const fn new() -> Self {
        Self {
            len: 0,
            rows: [Row {
                offset_ms: None,
                knockback: 0,
                grounded: None,
                speed: 0,
            }; LOG_LEN],
        }
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows[..usize::from(self.len)]
    }

    /// Adds a hit, forgetting the oldest when full.
    pub fn push(&mut self, row: Row) {
        if usize::from(self.len) == LOG_LEN {
            self.rows.rotate_left(1);
            self.rows[LOG_LEN - 1] = row;
        } else {
            self.rows[usize::from(self.len)] = row;
            self.len += 1;
        }
    }

    /// The log as text for the settings file: `offset,knockback,grounded,speed` per hit, hits
    /// separated by `;`. The offset is `x` for no jump and grounded `x` when unknown; knockback
    /// and speed are thousandths of a block per tick, grounded is 1 or 0.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for (index, row) in self.rows().iter().enumerate() {
            if index > 0 {
                out.push(';');
            }
            match row.offset_ms {
                Some(ms) => out.push_str(&ms.to_string()),
                None => out.push('x'),
            }
            out.push(',');
            out.push_str(&row.knockback.to_string());
            out.push(',');
            out.push(match row.grounded {
                Some(true) => '1',
                Some(false) => '0',
                None => 'x',
            });
            out.push(',');
            out.push_str(&row.speed.to_string());
        }
        out
    }

    /// Reads [`Log::to_text`] back, skipping what does not parse. In one pass over the bytes
    /// when the text is in the usual form: the general reader below costs about 1,500 fuel a
    /// row, and this runs in `init`, where the whole allowance is 200,000.
    pub fn parse(text: &str) -> Self {
        Self::parse_fast(text).unwrap_or_else(|| Self::parse_general(text))
    }

    /// [`Log::parse`] for text made only of what [`Log::to_text`] writes (digits, `-`, `x`, `,`
    /// and `;`) with every row complete; `None` for anything else.
    fn parse_fast(text: &str) -> Option<Self> {
        let mut log = Self::new();
        let mut at = 0;
        let bytes = text.as_bytes();
        while at < bytes.len() {
            let offset = Self::number(bytes, &mut at, b',')?;
            let knockback = Self::number(bytes, &mut at, b',')?;
            let grounded = match bytes.get(at)? {
                b'1' => Some(true),
                b'0' => Some(false),
                b'x' => None,
                _ => return None,
            };
            at += 1;
            if bytes.get(at) != Some(&b',') {
                return None;
            }
            at += 1;
            let speed = Self::number(bytes, &mut at, b';')?;
            log.push(Row {
                offset_ms: offset.and_then(|ms| i16::try_from(ms).ok()),
                knockback: u16::try_from(knockback?).ok()?,
                grounded,
                speed: u16::try_from(speed?).ok()?,
            });
        }
        Some(log)
    }

    /// The number at `*at` up to `end` (or the end of the text), moving `*at` past `end`. The
    /// outer `None` is text this reader does not take; an inner `None` is the `x` of no jump.
    fn number(bytes: &[u8], at: &mut usize, end: u8) -> Option<Option<i32>> {
        let start = *at;
        let negative = bytes.get(start) == Some(&b'-');
        let mut at_digit = start + usize::from(negative);
        if bytes.get(at_digit) == Some(&b'x') && !negative {
            at_digit += 1;
            if bytes.get(at_digit).is_some_and(|&byte| byte != end) {
                return None;
            }
            *at = at_digit + 1;
            return Some(None);
        }
        let first = at_digit;
        let mut value = 0i32;
        while let Some(&byte) = bytes.get(at_digit) {
            if byte == end {
                break;
            }
            if !byte.is_ascii_digit() {
                return None;
            }
            value = value.checked_mul(10)?.checked_add(i32::from(byte - b'0'))?;
            at_digit += 1;
        }
        if at_digit == first {
            return None;
        }
        *at = at_digit + 1;
        Some(Some(if negative { -value } else { value }))
    }

    /// [`Log::parse`] for any text, skipping what does not parse.
    fn parse_general(text: &str) -> Self {
        let mut log = Self::new();
        for part in text.split(';') {
            let mut fields = part.split(',');
            let (Some(offset), Some(knockback), Some(grounded), Some(speed), None) = (
                fields.next(),
                fields.next(),
                fields.next(),
                fields.next(),
                fields.next(),
            ) else {
                continue;
            };
            let (Ok(knockback), Ok(speed)) = (knockback.parse(), speed.parse()) else {
                continue;
            };
            log.push(Row {
                offset_ms: offset.parse().ok(),
                knockback,
                grounded: match grounded {
                    "1" => Some(true),
                    "0" => Some(false),
                    _ => None,
                },
                speed,
            });
        }
        log
    }
}

pub fn clamp_window(window: u16) -> u16 {
    window.clamp(MIN_WINDOW, MAX_WINDOW)
}

impl JumpReset {
    /// `window` moved by `steps` steps of [`WINDOW_STEP`], on a step and in bounds.
    pub fn stepped(window: u16, steps: i32) -> u16 {
        let step = i32::from(WINDOW_STEP);
        let snapped = (i32::from(window) + step / 2) / step * step;
        let moved = (snapped + steps * step).clamp(i32::from(MIN_WINDOW), i32::from(MAX_WINDOW));
        moved as u16
    }
}

/// How a jump compared with the hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Perfect,
    Early,
    Late,
}

/// The verdict for one jump and how far from the hit it was, in whole milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    pub verdict: Verdict,
    /// Whether the jump came before the hit (also for a perfect one).
    pub before: bool,
    pub ms: u16,
    /// Inside twice the perfect window: close enough to be worth a small change, not a big one.
    pub near: bool,
}

/// Judges a jump `offset` seconds from the hit: negative before it, positive after it.
pub fn judge(offset: f32, window: u16) -> Timing {
    let ms = (offset.abs() * 1000.0).round();
    let ms = if ms.is_finite() {
        ms.min(999.0) as u16
    } else {
        999
    };
    let before = offset < 0.0;
    let window = clamp_window(window);
    let verdict = if ms <= window {
        Verdict::Perfect
    } else if before {
        Verdict::Early
    } else {
        Verdict::Late
    };
    Timing {
        verdict,
        before,
        ms,
        near: ms <= window.saturating_mul(2),
    }
}

/// Pairs jump presses with hits. Each side waits at most [`RELEVANT`] seconds for the other, so
/// time only has to be known as the age of what is waiting: no clock that grows for hours.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tracker {
    /// Seconds since the last jump press that no hit has used yet.
    press: Option<f32>,
    /// Seconds since the last hit that no jump has used yet.
    hit: Option<f32>,
}

impl Tracker {
    pub const fn new() -> Self {
        Self {
            press: None,
            hit: None,
        }
    }

    /// Ages what is waiting by `seconds` and drops what no longer matters. Call once a frame,
    /// before the frame's presses and hits.
    pub fn step(&mut self, seconds: f32) {
        let seconds = if seconds.is_finite() {
            seconds.max(0.0)
        } else {
            0.0
        };
        for waiting in [&mut self.press, &mut self.hit] {
            if let Some(age) = waiting {
                *age += seconds;
                if *age > RELEVANT {
                    *waiting = None;
                }
            }
        }
    }

    /// A jump was pressed now. Returns the verdict when it follows a hit (late, or perfect).
    pub fn press(&mut self, window: u16) -> Option<Timing> {
        if let Some(age) = self.hit.take() {
            self.press = None;
            return Some(judge(age, window));
        }
        self.press = Some(0.0);
        None
    }

    /// The player was hit now. Returns the verdict when a jump came first (early, or perfect).
    pub fn hit(&mut self, window: u16) -> Option<Timing> {
        if let Some(age) = self.press.take() {
            self.hit = None;
            return Some(judge(-age, window));
        }
        self.hit = Some(0.0);
        None
    }
}

/// Follows the host's combat feed, which keeps the last events and hands them out oldest first,
/// a few per read. Everything already in it when the feed is first read is history and is
/// skipped; only events that arrive after that are hits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Feed {
    /// The last event sequence seen; the next read asks for later ones.
    pub after: u64,
    /// Whether the history was read to its end, so new events count.
    live: bool,
}

impl Feed {
    pub const fn new() -> Self {
        Self {
            after: 0,
            live: false,
        }
    }

    /// Takes one read of at most `limit` events, given as their sequence and whether each is
    /// the local player being hurt. Returns how many hits were new.
    pub fn take(&mut self, events: &[(u64, bool)], limit: usize) -> usize {
        let mut hits = 0;
        for &(sequence, hurt) in events {
            self.after = self.after.max(sequence);
            if hurt && self.live {
                hits += 1;
            }
        }
        // A short read reached the end of the history; a full one may have more behind it.
        if events.len() < limit {
            self.live = true;
        }
        hits
    }
}

/// How a hit moved you.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outcome {
    /// The biggest sudden change of horizontal velocity after the hit, in blocks per tick.
    pub knockback: f32,
    /// Little or no knockback: the jump reset worked.
    pub reset: bool,
    /// You were in the air when hit, where a jump cannot reset anything.
    pub airborne: bool,
    /// The shove was almost gone a tick later. A wall or a corner does that too, so this alone
    /// is not a reset: see [`Outcome::judged`].
    pub small: bool,
}

impl Outcome {
    /// A reset needs both a shove that is gone and a jump pressed close to the hit (within
    /// twice the perfect window): being pushed into a wall also kills the shove, and with no
    /// jump near it nothing was reset.
    pub fn judged(mut self, timing: Option<Timing>) -> Self {
        self.reset = self.small && timing.is_some_and(|timing| timing.near);
        self
    }
}

/// What was known just before a hit.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Trace {
    /// Whether you were on the ground, if known.
    pub grounded: Option<bool>,
    /// Your horizontal speed in blocks per tick.
    pub speed: f32,
}

/// Records your movement after a hit and finds out whether the hit pushed you. Each frame it
/// compares your horizontal velocity with the frame before; a hit shows as one big step, while
/// walking, sprinting or jumping change it in small ones. The biggest step in the
/// [`OBSERVE`] seconds after the hit is the knockback.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Watch {
    /// Horizontal velocity (x, z) in the last frame that had one.
    prev: Option<[f32; 2]>,
    /// Seconds since the hit being watched; none when nothing is.
    age: Option<f32>,
    /// Whether you were on the ground in the last frame that said.
    ground: Option<bool>,
    /// How things stood before the hit being watched (or the last one watched).
    trace: Trace,
    /// Your whole velocity (x, y, z) in the last frame that had one.
    prev3: Option<[f32; 3]>,
    /// Seconds since you last fell faster than [`FALL_SPEED`]; none if not within
    /// [`FALL_RELEVANT`].
    fell: Option<f32>,
    /// Your velocity just before the hit, in thousandths of a block per tick.
    before3: [i16; 3],
    /// Your velocity and whether you were on the ground (1, 0, or 2 for unknown) every
    /// [`SLOT_SECONDS`] from the hit on, for finding out what a reset looks like.
    slots: [[i16; 4]; SLOTS],
    /// How many of `slots` are filled.
    filled: u8,
    /// A watch ended without a shove (not a melee hit); cleared by [`Watch::take_dropped`].
    dropped: bool,
}

/// A fall faster than this (blocks per tick downwards) is a fall that hurts: damage starts at
/// three blocks, which is about 0.6 a tick. A hurt event within [`FALL_RELEVANT`] seconds of one
/// is fall damage, not a hit, and is skipped.
pub const FALL_SPEED: f32 = 0.55;
pub const FALL_RELEVANT: f32 = 0.4;

/// A horizontal speed of at least this (blocks per tick) in a recorded slot is the hit's shove,
/// which sets it to 0.4.
pub const SHOVE_SPEED: f32 = 0.3;

/// How many velocity slots a hit records, and how far apart they are.
pub const SLOTS: usize = 8;
pub const SLOT_SECONDS: f32 = 0.05;

fn milli(value: f32) -> i16 {
    let milli = (value * 1000.0).round();
    if milli.is_finite() {
        milli.clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16
    } else {
        0
    }
}

impl Watch {
    pub const fn new() -> Self {
        Self {
            prev: None,
            age: None,
            ground: None,
            trace: Trace {
                grounded: None,
                speed: 0.0,
            },
            prev3: None,
            fell: None,
            before3: [0; 3],
            slots: [[0; 4]; SLOTS],
            filled: 0,
            dropped: false,
        }
    }

    /// A hit just happened: start watching (a new hit replaces one still being watched). Call
    /// before the frame's [`Watch::sample`] and [`Watch::ground`], which belong to the frame
    /// after the hit's moment; what they held until now is how things stood before it.
    pub fn hit(&mut self) {
        self.age = Some(0.0);
        self.trace = Trace {
            grounded: self.ground,
            speed: self.prev.map_or(0.0, |v| v[0].hypot(v[1])),
        };
        self.before3 = self.prev3.map_or([0; 3], |v| v.map(milli));
        self.slots = [[0; 4]; SLOTS];
        self.filled = 0;
        self.dropped = false;
    }

    /// Records this frame's velocity and ground state in the next slot once it is due. Call
    /// after [`Watch::hit`] and before [`Watch::sample`], which ages the watch.
    pub fn record(&mut self, v: Option<[f32; 3]>, on_ground: Option<bool>) {
        let Some(age) = self.age else { return };
        let slot = usize::from(self.filled);
        if slot >= SLOTS || age < slot as f32 * SLOT_SECONDS {
            return;
        }
        let Some(v) = v.filter(|v| v.iter().all(|part| part.is_finite())) else {
            return;
        };
        let ground = match on_ground {
            Some(true) => 1,
            Some(false) => 0,
            None => 2,
        };
        self.slots[slot] = [milli(v[0]), milli(v[1]), milli(v[2]), ground];
        self.filled += 1;
    }

    /// The last hit's recorded movement as text: your velocity just before it, then the slots,
    /// `x,y,z|x,y,z,g;x,y,z,g;...` (thousandths of a block per tick; g is 1 on the ground, 0 in
    /// the air, 2 unknown).
    pub fn recording(&self) -> String {
        let [x, y, z] = self.before3;
        let mut out = format!("{x},{y},{z}|");
        for (index, slot) in self.slots[..usize::from(self.filled)].iter().enumerate() {
            if index > 0 {
                out.push(';');
            }
            out.push_str(&format!("{},{},{},{}", slot[0], slot[1], slot[2], slot[3]));
        }
        out
    }

    /// Notes whether the player is on the ground this frame, when known.
    pub fn ground(&mut self, on_ground: Option<bool>) {
        self.ground = on_ground;
    }

    /// How things stood before the hit last watched.
    pub fn trace(&self) -> Trace {
        self.trace
    }

    /// Whether you fell fast enough to be hurt by it a moment ago: a hurt event now is fall
    /// damage, not a hit, and nothing about it says anything on a jump reset.
    pub fn falling(&self) -> bool {
        self.fell.is_some()
    }

    /// Whether a hit is being watched.
    pub fn watching(&self) -> bool {
        self.age.is_some()
    }

    /// One frame of `seconds`, with the player's velocity `v` in blocks per tick (x, y, z) when it
    /// could be read. Returns the outcome once the watching time is over. Call after
    /// [`Watch::hit`] in a frame with a hit, so the frame's own step counts.
    pub fn sample(&mut self, seconds: f32, v: Option<[f32; 3]>, threshold: f32) -> Option<Outcome> {
        let seconds = if seconds.is_finite() {
            seconds.max(0.0)
        } else {
            0.0
        };
        let now = v
            .filter(|v| v.iter().all(|part| part.is_finite()))
            .map(|v| [v[0], v[2]]);
        self.prev = now.or(self.prev);
        self.prev3 = v
            .filter(|v| v.iter().all(|part| part.is_finite()))
            .or(self.prev3);
        if let Some(age) = self.fell.as_mut() {
            *age += seconds;
            if *age >= FALL_RELEVANT {
                self.fell = None;
            }
        }
        if v.is_some_and(|v| v[1].is_finite() && v[1] <= -FALL_SPEED) {
            self.fell = Some(0.0);
        }
        let age = self.age.as_mut()?;
        *age += seconds;
        if *age < OBSERVE {
            return None;
        }
        self.age = None;
        // Every melee hit sets the shove. A hurt without one (fire, lava, poison, drowning, a
        // velocity that could not be read) is no hit, so there is nothing to judge: saying
        // `Reset!` for it would be a lie.
        if !self.shoved() {
            self.dropped = true;
            return None;
        }
        let carried = self.carried();
        let small = carried < clamp_knockback(threshold);
        Some(Outcome {
            knockback: carried,
            reset: small,
            airborne: self.trace.grounded == Some(false),
            small,
        })
    }

    /// Whether a recorded slot of the hit being watched (or the last one) shows its shove.
    pub fn shoved(&self) -> bool {
        self.slots[..usize::from(self.filled)]
            .iter()
            .any(|slot| f32::from(slot[0]).hypot(f32::from(slot[2])) / 1000.0 >= SHOVE_SPEED)
    }

    /// Whether the last watch ended without a shove (once; reading clears it).
    pub fn take_dropped(&mut self) -> bool {
        std::mem::take(&mut self.dropped)
    }

    /// How much of the hit's shove is left one tick later: the horizontal speed in the slot after
    /// the one where the shove shows (a horizontal speed of at least [`SHOVE_SPEED`]), in blocks
    /// per tick. A normal hit on the ground leaves about 0.15, ground friction having eaten the
    /// rest; a jump pressed as the hit lands leaves almost nothing. No shove in the slots gives 0
    /// (such a hurt is dropped before this is used), and a shove only in the last slot cannot be
    /// followed, so it counts as the full shove.
    pub fn carried(&self) -> f32 {
        let speed = |slot: &[i16; 4]| f32::from(slot[0]).hypot(f32::from(slot[2])) / 1000.0;
        let slots = &self.slots[..usize::from(self.filled)];
        match slots.iter().position(|slot| speed(slot) >= SHOVE_SPEED) {
            None => 0.0,
            Some(shove) => slots.get(shove + 1).map_or(speed(&slots[shove]), speed),
        }
    }
}

/// What the HUD shows this frame. Patches are sent only when it changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub timing: Option<Timing>,
    pub outcome: Option<Outcome>,
}

impl View {
    pub const HIDDEN: View = View {
        timing: None,
        outcome: None,
    };

    pub fn is_hidden(&self) -> bool {
        self.timing.is_none() && self.outcome.is_none()
    }
}

/// The colour of a verdict: green when perfect, yellow when close, red when far off.
pub fn timing_colour(timing: Timing) -> [f32; 4] {
    match (timing.verdict, timing.near) {
        (Verdict::Perfect, _) => [0.4, 1.0, 0.5, 1.0],
        (_, true) => [1.0, 0.85, 0.3, 1.0],
        (_, false) => [1.0, 0.4, 0.35, 1.0],
    }
}

/// The readout's colour: green once the reset worked, whatever the timing was; otherwise the
/// timing's colour, or orange for a hit that pushed you after a perfect jump or without one.
pub fn colour(view: &View) -> [f32; 4] {
    match (view.outcome, view.timing) {
        (Some(outcome), _) if outcome.reset => [0.4, 1.0, 0.5, 1.0],
        (Some(_), Some(timing)) if timing.verdict != Verdict::Perfect => timing_colour(timing),
        (Some(_), _) => [1.0, 0.6, 0.3, 1.0],
        (None, Some(timing)) => timing_colour(timing),
        (None, None) => [1.0; 4],
    }
}

/// Writes the timing's text: `Too early: 85 ms`, `Too late: 120 ms` or `Perfect (12 ms early)`.
/// Integer arithmetic only: a frame has little fuel.
pub fn timing_text(out: &mut String, timing: Timing) {
    let ms = u64::from(timing.ms);
    match timing.verdict {
        Verdict::Early => {
            out.push_str("Too early: ");
            digits(out, ms);
            out.push_str(" ms");
        }
        Verdict::Late => {
            out.push_str("Too late: ");
            digits(out, ms);
            out.push_str(" ms");
        }
        Verdict::Perfect if ms == 0 => out.push_str("Perfect"),
        Verdict::Perfect => {
            out.push_str("Perfect (");
            digits(out, ms);
            out.push_str(if timing.before {
                " ms early)"
            } else {
                " ms late)"
            });
        }
    }
}

/// Writes a number of blocks per tick with two decimals: `0.41`.
fn hundredths(out: &mut String, value: f32) {
    let hundredths = (value * 100.0).round();
    let hundredths = if hundredths.is_finite() {
        hundredths.clamp(0.0, 9999.0) as u64
    } else {
        0
    };
    digits(out, hundredths / 100);
    out.push('.');
    if hundredths % 100 < 10 {
        out.push('0');
    }
    digits(out, hundredths % 100);
}

/// Writes the readout: the timing, then `|` and how much of the shove was left a tick later
/// (`Reset! knock left 0.04` or `Knock left 0.15`), whichever of the two are known.
pub fn text(out: &mut String, view: &View) {
    if let Some(timing) = view.timing {
        timing_text(out, timing);
    }
    if let Some(outcome) = view.outcome {
        if view.timing.is_some() {
            out.push_str(" | ");
        }
        out.push_str(if outcome.reset {
            "Reset! knock left "
        } else {
            "Knock left "
        });
        hundredths(out, outcome.knockback);
        if outcome.airborne {
            out.push_str(" (in air)");
        } else if outcome.small && !outcome.reset {
            // Little knock left but no jump near it: a wall or a corner, not a reset.
            out.push_str(if view.timing.is_some() {
                " (jump off)"
            } else {
                " (no jump)"
            });
        }
    }
}

/// A made-up readout for the menu page, so it can be seen without being hit.
pub const SAMPLE: View = View {
    timing: Some(Timing {
        verdict: Verdict::Early,
        before: true,
        ms: 85,
        near: true,
    }),
    outcome: Some(Outcome {
        knockback: 0.41,
        reset: false,
        airborne: false,
        small: false,
    }),
};

/// The readout of the HUD document, as a string literal: one label under the crosshair.
macro_rules! document {
    () => {
        concat!(
            r##"{"jumpreset":{"type":"label","text":"#jumpreset_text","size":[460,14],"offset":[0,28],"anchor_from":"center","anchor_to":"center","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":1.25,"layer":2,"bindings":["##,
            r##"{"binding_name":"#jumpreset_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#jumpreset_text"},"##,
            r##"{"binding_name":"#jumpreset_color","binding_name_override":"#color"}]}}"##
        )
    };
}
pub(crate) use document;

/// The bindings of a hidden readout, to publish with the rest of the layer.
pub fn initial_bindings() -> String {
    bindings(&View::HIDDEN)
}

/// A `hud.update-surface` patch showing `view`.
pub fn patch(view: &View) -> String {
    let mut out = String::with_capacity(140);
    out.push_str(r#"{"binding_changes":{"#);
    out.push_str(&bindings(view));
    out.push_str("}}");
    out
}

/// The readout's bindings as `"name":value` pairs, without braces.
fn bindings(view: &View) -> String {
    let mut out = String::with_capacity(130);
    out.push_str(r##""#jumpreset_visible":"##);
    out.push_str(if view.is_hidden() { "false" } else { "true" });
    out.push_str(r##","#jumpreset_text":""##);
    text(&mut out, view);
    out.push_str(r##"","#jumpreset_color":["##);
    for (index, part) in colour(view).into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        crate::panel::fixed(&mut out, part);
    }
    out.push(']');
    out
}

#[cfg(test)]
mod tests;
