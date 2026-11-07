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
            on: true,
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

    /// Reads [`Log::to_text`] back, skipping what does not parse.
    pub fn parse(text: &str) -> Self {
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
mod tests {
    use super::*;

    fn run(events: &[(&str, f32)], window: u16) -> Vec<Timing> {
        // Each event happens after the given seconds since the one before.
        let mut tracker = Tracker::new();
        let mut results = Vec::new();
        for &(what, after) in events {
            tracker.step(after);
            let timing = match what {
                "press" => tracker.press(window),
                _ => tracker.hit(window),
            };
            results.extend(timing);
        }
        results
    }

    #[test]
    fn a_jump_before_the_hit_is_early_and_after_it_is_late() {
        let early = run(&[("press", 0.0), ("hit", 0.2)], 75);
        assert_eq!(early.len(), 1);
        assert_eq!(
            (early[0].verdict, early[0].before, early[0].ms),
            (Verdict::Early, true, 200)
        );
        let late = run(&[("hit", 0.0), ("press", 0.12)], 75);
        assert_eq!(late.len(), 1);
        assert_eq!(
            (late[0].verdict, late[0].before, late[0].ms),
            (Verdict::Late, false, 120)
        );
    }

    #[test]
    fn a_jump_inside_the_window_is_perfect_on_either_side() {
        for events in [
            [("press", 0.0), ("hit", 0.05)],
            [("hit", 0.0), ("press", 0.05)],
            [("press", 0.0), ("hit", 0.0)],
        ] {
            let got = run(&events, 75);
            assert_eq!(got.len(), 1, "{events:?}");
            assert_eq!(got[0].verdict, Verdict::Perfect, "{events:?}");
        }
        // The edge of the window is still perfect, one millisecond past it is not.
        assert_eq!(judge(-0.075, 75).verdict, Verdict::Perfect);
        assert_eq!(judge(0.075, 75).verdict, Verdict::Perfect);
        assert_eq!(judge(0.076, 75).verdict, Verdict::Late);
        assert_eq!(judge(-0.076, 75).verdict, Verdict::Early);
        // A wider window forgives more.
        assert_eq!(judge(0.15, 150).verdict, Verdict::Perfect);
    }

    #[test]
    fn a_jump_and_a_hit_far_apart_say_nothing() {
        // A jump long before a hit is an ordinary jump.
        assert!(run(&[("press", 0.0), ("hit", 0.41)], 75).is_empty());
        // A hit long before a jump is an ordinary jump too.
        assert!(run(&[("hit", 0.0), ("press", 0.41)], 75).is_empty());
        // Just inside is still about that hit.
        assert_eq!(run(&[("press", 0.0), ("hit", 0.4)], 75).len(), 1);
        // Time in small steps adds up the same.
        let mut tracker = Tracker::new();
        tracker.press(75);
        for _ in 0..10 {
            tracker.step(0.01);
        }
        assert_eq!(tracker.hit(75).map(|t| t.ms), Some(100));
    }

    #[test]
    fn the_latest_jump_before_a_hit_is_the_one_judged() {
        // Two jumps, then the hit: the second, nearer one counts.
        let got = run(&[("press", 0.0), ("press", 0.25), ("hit", 0.05)], 75);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].ms, 50);
        assert_eq!(got[0].verdict, Verdict::Perfect);
    }

    #[test]
    fn one_hit_is_judged_once() {
        // The first jump after a hit is judged; a second jump has no hit left to be late for.
        let got = run(&[("hit", 0.0), ("press", 0.1), ("press", 0.05)], 75);
        assert_eq!(got.len(), 1);
        // And a jump is not judged twice against two hits.
        let got = run(&[("press", 0.0), ("hit", 0.1), ("hit", 0.05)], 75);
        assert_eq!(got.len(), 1);
    }

    #[test]
    fn bad_time_cannot_break_the_tracker() {
        let mut tracker = Tracker::new();
        tracker.press(75);
        tracker.step(f32::NAN);
        tracker.step(-3.0);
        tracker.step(f32::INFINITY);
        // Infinite time is not a time: nothing aged, so the jump still pairs with the hit.
        assert_eq!(tracker.hit(75).map(|t| t.ms), Some(0));
        assert_eq!(judge(f32::NAN, 75).ms, 999);
        assert_eq!(judge(100.0, 75).ms, 999);
    }

    #[test]
    fn the_window_steps_and_stays_in_bounds() {
        assert_eq!(JumpReset::stepped(75, 1), 80);
        assert_eq!(JumpReset::stepped(75, -1), 70);
        assert_eq!(JumpReset::stepped(MAX_WINDOW, 1), MAX_WINDOW);
        assert_eq!(JumpReset::stepped(MIN_WINDOW, -1), MIN_WINDOW);
        // A value from a hand-edited file moves onto a step.
        assert_eq!(JumpReset::stepped(77, 1), 80);
        assert_eq!(JumpReset::stepped(0, 0), MIN_WINDOW);
        assert_eq!(clamp_window(5000), MAX_WINDOW);
    }

    #[test]
    fn the_feed_skips_history_and_counts_new_hits() {
        let mut feed = Feed::new();
        // History: two full reads of other events, then a short one with an old hit of mine.
        assert_eq!(
            feed.take(&[(1, false), (2, true), (3, false), (4, false)], 4),
            0
        );
        assert_eq!(feed.after, 4);
        assert!(!feed.live);
        assert_eq!(feed.take(&[(5, false), (6, true)], 4), 0);
        assert_eq!(feed.after, 6);
        // Now at the end: a new hit counts, someone else's does not.
        assert_eq!(feed.take(&[(7, true), (8, false)], 4), 1);
        assert_eq!(feed.take(&[], 4), 0);
        assert_eq!(feed.after, 8);
        // A burst larger than a read is followed up, and every hit in it counts.
        assert_eq!(
            feed.take(&[(9, true), (10, true), (11, true), (12, false)], 4),
            3
        );
        assert_eq!(feed.take(&[(13, true)], 4), 1);
    }

    #[test]
    fn an_empty_history_is_live_straight_away() {
        let mut feed = Feed::new();
        assert_eq!(feed.take(&[], 4), 0);
        assert_eq!(feed.take(&[(1, true)], 4), 1);
    }

    #[test]
    fn the_text_says_what_happened() {
        let say = |timing: Timing| {
            let mut out = String::new();
            timing_text(&mut out, timing);
            out
        };
        assert_eq!(say(judge(-0.085, 50)), "Too early: 85 ms");
        assert_eq!(say(judge(0.12, 50)), "Too late: 120 ms");
        assert_eq!(say(judge(-0.012, 75)), "Perfect (12 ms early)");
        assert_eq!(say(judge(0.03, 75)), "Perfect (30 ms late)");
        assert_eq!(say(judge(0.0, 75)), "Perfect");
        assert_eq!(say(judge(-5.0, 75)), "Too early: 999 ms");
    }

    #[test]
    fn the_colour_tells_close_from_far() {
        assert_eq!(timing_colour(judge(0.01, 75))[1], 1.0);
        let close = timing_colour(judge(0.12, 75));
        let far = timing_colour(judge(0.3, 75));
        assert!(close[0] == 1.0 && close[1] > far[1]);
        assert_ne!(close, far);
        assert!(judge(0.15, 75).near && !judge(0.151, 75).near);
    }

    #[test]
    fn the_patch_carries_the_text_and_colour() {
        let view = View {
            timing: Some(judge(-0.085, 50)),
            outcome: None,
        };
        let patch = patch(&view);
        assert!(patch.starts_with(r##"{"binding_changes":{"#jumpreset_visible":true,"##));
        assert!(
            patch.contains(r##""#jumpreset_text":"Too early: 85 ms""##),
            "{patch}"
        );
        assert!(patch.contains(r##""#jumpreset_color":[1,"##), "{patch}");
        assert!(patch.ends_with("}}"));
        let hidden = initial_bindings();
        assert!(hidden.contains(r##""#jumpreset_visible":false"##));
        assert!(hidden.contains(r##""#jumpreset_text":"""##));
        let json = format!("{{{hidden}}}");
        assert!(!json.contains(",}"), "no trailing comma: {json}");
    }

    #[test]
    fn the_document_is_balanced_json() {
        let doc = document!();
        for (open, close) in [('{', '}'), ('[', ']')] {
            assert_eq!(doc.matches(open).count(), doc.matches(close).count());
        }
        assert!(doc.contains("#jumpreset_color"));
    }

    /// A velocity trace: each frame lasts `dt` seconds and has this (x, y, z) velocity.
    fn watched(frames: &[[f32; 3]], dt: f32, hit_at: usize, threshold: f32) -> Option<Outcome> {
        let mut watch = Watch::new();
        let mut result = None;
        let mut step = |watch: &mut Watch, frame: Option<[f32; 3]>| {
            watch.record(frame, None);
            result = result.or(watch.sample(dt, frame, threshold));
        };
        for (index, frame) in frames.iter().enumerate() {
            if index == hit_at {
                watch.hit();
            }
            step(&mut watch, Some(*frame));
        }
        // Let the watching time run out with the last velocity held.
        for _ in 0..8 {
            step(&mut watch, frames.last().copied());
        }
        result
    }

    fn still(count: usize, v: [f32; 3]) -> Vec<[f32; 3]> {
        vec![v; count]
    }

    #[test]
    fn a_shove_that_carries_on_is_not_a_reset_and_one_that_dies_is() {
        let dt = 0.05;
        let hit = |after: &[[f32; 3]], threshold: f32| {
            let mut frames = still(4, [0.0; 3]);
            frames.extend_from_slice(after);
            frames.extend(still(6, *after.last().unwrap()));
            watched(&frames, dt, 4, threshold).expect("watched to the end")
        };
        // The shove carries on: 0.4, then 0.4 again.
        let out = hit(&[[0.4, 0.4, 0.0], [0.4, 0.3, 0.0]], DEFAULT_KNOCKBACK);
        assert!(!out.reset && (out.knockback - 0.4).abs() < 1e-3, "{out:?}");
        // Ground friction takes a normal hit down to about 0.15 a tick later: not a reset.
        let out = hit(&[[0.4, 0.4, 0.0], [0.15, 0.3, 0.0]], DEFAULT_KNOCKBACK);
        assert!(!out.reset && (out.knockback - 0.15).abs() < 1e-3, "{out:?}");
        // A jump that ate it leaves hardly anything.
        let out = hit(&[[0.4, 0.4, 0.0], [0.04, 0.3, 0.0]], DEFAULT_KNOCKBACK);
        assert!(out.reset && (out.knockback - 0.04).abs() < 1e-3, "{out:?}");
        // The shove can come diagonally and a tick late: what counts is the tick after it.
        let out = hit(
            &[[0.1, 0.0, 0.0], [0.28, 0.4, 0.28], [0.03, 0.3, 0.03]],
            DEFAULT_KNOCKBACK,
        );
        assert!(
            out.reset && (out.knockback - 0.0424).abs() < 1e-3,
            "{out:?}"
        );
        // The threshold decides: 0.1 left is a reset for a loose one, not a strict one.
        let shove = [[0.4, 0.4, 0.0], [0.1, 0.3, 0.0]];
        assert!(!hit(&shove, 0.05).reset);
        assert!(hit(&shove, 0.2).reset);
    }

    #[test]
    fn a_hurt_without_a_shove_is_no_hit_and_says_nothing() {
        let dt = 0.05;
        // Running on as before, or burning: no shove anywhere, so no verdict (a Reset! would be
        // a lie) and the watch says it dropped it.
        for v in [[0.0, 0.0, -0.13], [0.0, 0.0, 0.0], [0.0, 0.42, 0.0]] {
            let mut watch = Watch::new();
            watch.sample(dt, Some(v), DEFAULT_KNOCKBACK);
            watch.hit();
            let mut out = None;
            for _ in 0..12 {
                watch.record(Some(v), None);
                out = out.or(watch.sample(dt, Some(v), DEFAULT_KNOCKBACK));
            }
            assert_eq!(out, None, "{v:?}");
            assert!(!watch.shoved());
            assert!(watch.take_dropped());
            assert!(!watch.take_dropped(), "reading it clears it");
        }
        // A shove that only shows in the last slot cannot be followed: the full shove counts.
        let mut watch = Watch::new();
        watch.hit();
        for index in 0..SLOTS {
            let v = if index == SLOTS - 1 {
                [0.4, 0.4, 0.0]
            } else {
                [0.0; 3]
            };
            watch.record(Some(v), None);
            watch.sample(SLOT_SECONDS, Some(v), DEFAULT_KNOCKBACK);
        }
        assert!(watch.shoved());
        assert!((watch.carried() - 0.4).abs() < 1e-3);
        assert!(!watch.take_dropped());
    }

    #[test]
    fn the_shove_in_the_hit_frame_itself_counts() {
        let dt = 0.05;
        // The shove arrives in the very frame of the hit, and is gone a tick later.
        let mut now = still(3, [0.0; 3]);
        now.push([0.0, 0.4, 0.4]);
        now.extend(still(8, [0.0, 0.3, 0.02]));
        assert!(watched(&now, dt, 3, DEFAULT_KNOCKBACK).unwrap().reset);
        // The same frames with the hit a tick later: the shove came before it and is no part of
        // it, so this is a hurt without a shove and says nothing.
        assert_eq!(watched(&now, dt, 4, DEFAULT_KNOCKBACK), None);
    }

    #[test]
    fn the_watch_waits_out_its_time_and_needs_a_measurement() {
        let mut watch = Watch::new();
        let frame = |watch: &mut Watch, seconds: f32, v: Option<[f32; 3]>| {
            watch.record(v, None);
            watch.sample(seconds, v, 0.08)
        };
        let shove = Some([0.4, 0.4, 0.0]);
        let calm = Some([0.02, 0.0, 0.0]);
        // Nothing to watch: no outcome, however long.
        assert_eq!(frame(&mut watch, 1.0, calm), None);
        watch.hit();
        assert!(watch.watching());
        assert_eq!(frame(&mut watch, 0.1, shove), None);
        assert_eq!(frame(&mut watch, 0.1, calm), None);
        assert!(frame(&mut watch, 0.25, calm).is_some());
        assert!(!watch.watching());
        assert_eq!(frame(&mut watch, 0.5, calm), None);
        // Never being able to read the velocity gives no outcome rather than a false reset.
        watch.hit();
        assert_eq!(frame(&mut watch, 0.5, None), None);
        assert!(watch.take_dropped());
        // Nonsense velocities are skipped, not recorded.
        watch.hit();
        assert_eq!(frame(&mut watch, 0.1, Some([f32::NAN, 0.0, 0.0])), None);
        assert_eq!(frame(&mut watch, 0.5, calm), None);
        assert!(watch.take_dropped());
        // A real shove then ends in an outcome, and nothing was dropped.
        watch.hit();
        assert_eq!(frame(&mut watch, 0.1, shove), None);
        let out = frame(&mut watch, 0.5, calm).unwrap();
        assert!(out.reset && out.knockback < 0.05, "{out:?}");
        assert!(!watch.take_dropped());
    }

    #[test]
    fn a_second_hit_starts_the_watch_again() {
        let mut watch = Watch::new();
        let frame = |watch: &mut Watch, seconds: f32, v: [f32; 3]| {
            watch.record(Some(v), None);
            watch.sample(seconds, Some(v), 0.08)
        };
        frame(&mut watch, 0.05, [0.0; 3]);
        watch.hit();
        frame(&mut watch, 0.05, [0.4, 0.4, 0.0]);
        frame(&mut watch, 0.05, [0.4, 0.3, 0.0]);
        // Hit again before the first was finished: only the new one is judged.
        watch.hit();
        assert_eq!(frame(&mut watch, 0.05, [0.4, 0.4, 0.0]), None);
        assert_eq!(frame(&mut watch, 0.05, [0.02, 0.3, 0.0]), None);
        let out = frame(&mut watch, 0.4, [0.02, 0.3, 0.0]).unwrap();
        assert!(out.reset, "the second shove died at once: {out:?}");
    }

    #[test]
    fn the_readout_joins_timing_and_knockback() {
        let say = |view: View| {
            let mut out = String::new();
            text(&mut out, &view);
            out
        };
        let timing = Some(judge(-0.085, 50));
        let reset = Some(Outcome {
            knockback: 0.04,
            reset: true,
            airborne: false,
            small: false,
        });
        let pushed = Some(Outcome {
            knockback: 0.4,
            reset: false,
            airborne: false,
            small: false,
        });
        assert_eq!(
            say(View {
                timing,
                outcome: pushed
            }),
            "Too early: 85 ms | Knock left 0.40"
        );
        assert_eq!(
            say(View {
                timing: Some(judge(0.01, 75)),
                outcome: reset
            }),
            "Perfect (10 ms late) | Reset! knock left 0.04"
        );
        // Either part alone, and a knockback of 1.5 or more digits long still reads.
        assert_eq!(
            say(View {
                timing: None,
                outcome: reset
            }),
            "Reset! knock left 0.04"
        );
        assert_eq!(
            say(View {
                timing,
                outcome: None
            }),
            "Too early: 85 ms"
        );
        let big = Some(Outcome {
            knockback: 12.345,
            reset: false,
            airborne: false,
            small: false,
        });
        assert_eq!(
            say(View {
                timing: None,
                outcome: big
            }),
            "Knock left 12.35"
        );
        assert_eq!(say(View::HIDDEN), "");
        assert!(View::HIDDEN.is_hidden());
        assert!(!SAMPLE.is_hidden());
    }

    #[test]
    fn a_small_knock_is_a_reset_only_with_a_jump_near_the_hit() {
        let small = Outcome {
            knockback: 0.01,
            reset: true,
            airborne: false,
            small: true,
        };
        let big = Outcome {
            knockback: 0.3,
            reset: false,
            airborne: false,
            small: false,
        };
        let near = Some(judge(-0.05, 75));
        let edge = Some(judge(0.15, 75));
        let far = Some(judge(-0.232, 75));
        // Jump near the hit and the shove gone: a reset, early or late alike.
        assert!(small.judged(near).reset);
        assert!(small.judged(Some(judge(0.1, 75))).reset);
        assert!(small.judged(edge).reset, "twice the window is still near");
        // A jump too far off, or none at all (a wall or corner killed it): no reset.
        assert!(!small.judged(far).reset);
        assert!(!small.judged(None).reset);
        assert!(small.judged(None).small, "but it is still a small knock");
        // A shove that carried on is never a reset, whatever the jump.
        assert!(!big.judged(near).reset);
        // A wider window widens what is near.
        assert!(small.judged(Some(judge(0.2, 150))).reset);
    }

    #[test]
    fn the_readout_says_why_a_small_knock_is_not_a_reset() {
        let say = |view: View| {
            let mut out = String::new();
            text(&mut out, &view);
            out
        };
        let small = Outcome {
            knockback: 0.009,
            reset: false,
            airborne: false,
            small: true,
        };
        assert_eq!(
            say(View {
                timing: None,
                outcome: Some(small)
            }),
            "Knock left 0.01 (no jump)"
        );
        assert_eq!(
            say(View {
                timing: Some(judge(-0.232, 75)),
                outcome: Some(small)
            }),
            "Too early: 232 ms | Knock left 0.01 (jump off)"
        );
        let reset = Outcome {
            reset: true,
            ..small
        };
        assert_eq!(
            say(View {
                timing: Some(judge(0.0, 75)),
                outcome: Some(reset)
            }),
            "Perfect | Reset! knock left 0.01"
        );
        // In the air outranks the rest.
        let air = Outcome {
            airborne: true,
            ..small
        };
        assert_eq!(
            say(View {
                timing: None,
                outcome: Some(air)
            }),
            "Knock left 0.01 (in air)"
        );
    }

    #[test]
    fn a_hit_taken_in_the_air_says_so() {
        let mut watch = Watch::new();
        watch.ground(Some(false));
        watch.sample(0.05, Some([0.0, -0.3, 0.0]), 0.08);
        watch.hit();
        let mut out = None;
        for frame in 0..12 {
            // The shove, then it carries on at 0.34 a tick, as it does in the air.
            let v = if frame == 0 {
                [0.4, 0.4, 0.0]
            } else {
                [0.34, 0.3, 0.0]
            };
            watch.record(Some(v), Some(false));
            out = out.or(watch.sample(0.05, Some(v), 0.08));
            watch.ground(Some(false));
        }
        let out = out.unwrap();
        assert!(out.airborne && !out.reset, "{out:?}");
        let mut text = String::new();
        super::text(
            &mut text,
            &View {
                timing: None,
                outcome: Some(out),
            },
        );
        assert_eq!(text, "Knock left 0.34 (in air)");
        // On the ground it does not.
        let mut grounded = Watch::new();
        grounded.ground(Some(true));
        grounded.sample(0.05, Some([0.0, -0.078, 0.0]), 0.08);
        grounded.hit();
        let mut out = None;
        for frame in 0..12 {
            let v = if frame == 0 {
                [0.4, 0.4, 0.0]
            } else {
                [0.15, 0.3, 0.0]
            };
            grounded.record(Some(v), Some(true));
            out = out.or(grounded.sample(0.05, Some(v), 0.08));
        }
        assert!(!out.unwrap().airborne);
    }

    #[test]
    fn the_colour_follows_the_outcome_first() {
        let green = [0.4, 1.0, 0.5, 1.0];
        let reset = Outcome {
            knockback: 0.0,
            reset: true,
            airborne: false,
            small: false,
        };
        let pushed = Outcome {
            knockback: 0.4,
            reset: false,
            airborne: false,
            small: false,
        };
        // A working reset is green even when the jump was well off.
        let late = Some(judge(0.3, 75));
        assert_eq!(
            colour(&View {
                timing: late,
                outcome: Some(reset)
            }),
            green
        );
        // Pushed after a late jump: the timing's colour; after a perfect one or none: orange.
        assert_eq!(
            colour(&View {
                timing: late,
                outcome: Some(pushed)
            }),
            timing_colour(judge(0.3, 75))
        );
        let orange = [1.0, 0.6, 0.3, 1.0];
        assert_eq!(
            colour(&View {
                timing: Some(judge(0.0, 75)),
                outcome: Some(pushed)
            }),
            orange
        );
        assert_eq!(
            colour(&View {
                timing: None,
                outcome: Some(pushed)
            }),
            orange
        );
        // Timing alone, before the knockback is known.
        assert_eq!(
            colour(&View {
                timing: late,
                outcome: None
            }),
            timing_colour(judge(0.3, 75))
        );
    }

    #[test]
    fn the_watch_remembers_how_things_stood_before_the_hit() {
        let mut watch = Watch::new();
        watch.ground(Some(true));
        watch.sample(0.016, Some([0.3, 0.0, 0.4]), 0.25);
        watch.hit();
        // The frame of the hit moves the velocity, but the trace is from before.
        watch.ground(Some(false));
        watch.sample(0.016, Some([0.0, 0.4, 0.0]), 0.25);
        let trace = watch.trace();
        assert_eq!(trace.grounded, Some(true));
        assert!((trace.speed - 0.5).abs() < 1e-6, "{trace:?}");
        // With nothing known before, it says so.
        let mut fresh = Watch::new();
        fresh.hit();
        assert_eq!(fresh.trace(), Trace::default());
    }

    #[test]
    fn the_log_keeps_the_last_hits_and_reads_back() {
        let timing = Some(judge(-0.085, 50));
        let outcome = Outcome {
            knockback: 0.4123,
            reset: false,
            airborne: false,
            small: false,
        };
        let trace = Trace {
            grounded: Some(true),
            speed: 0.13,
        };
        let row = Row::new(timing, outcome, trace);
        assert_eq!(
            row,
            Row {
                offset_ms: Some(-85),
                knockback: 412,
                grounded: Some(true),
                speed: 130,
            }
        );
        let late = Row::new(Some(judge(0.12, 50)), outcome, Trace::default());
        assert_eq!(late.offset_ms, Some(120));
        assert_eq!(Row::new(None, outcome, Trace::default()).offset_ms, None);
        let mut log = Log::new();
        assert_eq!(log.to_text(), "");
        log.push(row);
        log.push(Row::new(None, outcome, Trace::default()));
        assert_eq!(log.to_text(), "-85,412,1,130;x,412,x,0");
        assert_eq!(Log::parse(&log.to_text()), log);
        // Full: the oldest goes first.
        for step in 0..30u16 {
            log.push(Row {
                knockback: step,
                ..Row::default()
            });
        }
        assert_eq!(log.rows().len(), LOG_LEN);
        assert_eq!(log.rows()[0].knockback, 14);
        assert_eq!(log.rows()[LOG_LEN - 1].knockback, 29);
        assert_eq!(Log::parse(&log.to_text()), log);
        // Bad text is skipped, not trusted.
        let odd = Log::parse("junk;1,2;x,5,1,7;1,2,3,4,5;-9,notanumber,0,1;;2000000,3,0,4");
        assert_eq!(odd.rows().len(), 2);
        assert_eq!(odd.rows()[0].knockback, 5);
        assert_eq!(odd.rows()[1].offset_ms, None);
        assert_eq!(Log::parse("").rows().len(), 0);
    }

    #[test]
    fn a_hit_records_the_velocity_before_and_every_50_ms_after() {
        let mut watch = Watch::new();
        watch.ground(Some(true));
        watch.sample(0.016, Some([0.1, 0.0, -0.2]), 0.25);
        watch.hit();
        // 60 frames of 10 ms: the first is the hit's own frame, then one slot per 50 ms.
        for frame in 0..60 {
            let t = frame as f32 * 0.01;
            let v = Some([t, 0.5, -0.25]);
            let ground = Some(frame < 20);
            watch.record(v, ground);
            watch.sample(0.01, v, 0.25);
            watch.ground(ground);
        }
        let text = watch.recording();
        let (before, slots) = text.split_once('|').unwrap();
        assert_eq!(before, "100,0,-200");
        let slots: Vec<&str> = slots.split(';').collect();
        assert_eq!(slots.len(), SLOTS, "{text}");
        assert!(slots[0].starts_with("0,500,-250,1"), "{text}");
        assert!(slots[7].ends_with(",0"), "{text}: in the air by 350 ms");
        // A new hit starts from nothing, and unreadable velocities fill nothing.
        watch.hit();
        watch.record(None, None);
        assert!(watch.recording().ends_with('|'));
        assert_eq!(milli(f32::NAN), 0);
        assert_eq!(milli(1000.0), i16::MAX);
    }

    #[test]
    fn a_hurt_just_after_a_fast_fall_is_fall_damage() {
        let mut watch = Watch::new();
        assert!(!watch.falling());
        // Falling at 0.7 a tick, then landing: the landing's hurt comes a tick or two later.
        watch.sample(0.05, Some([0.0, -0.7, 0.0]), 0.35);
        assert!(watch.falling());
        watch.sample(0.05, Some([0.0, -0.078, 0.0]), 0.35);
        assert!(watch.falling(), "just landed");
        for _ in 0..8 {
            watch.sample(0.05, Some([0.0, -0.078, 0.0]), 0.35);
        }
        assert!(!watch.falling(), "0.4 s later it is an ordinary hit again");
        // An ordinary jump's landing speed is not a fall that hurts, nor is an unreadable or
        // nonsense velocity.
        watch.sample(0.05, Some([0.0, -0.5, 0.0]), 0.35);
        watch.sample(0.05, Some([0.0, f32::NAN, 0.0]), 0.35);
        watch.sample(0.05, None, 0.35);
        assert!(!watch.falling());
        // A falling start from a knockback's own climb and drop is only fast later: fall speed
        // exactly at the limit counts.
        watch.sample(0.05, Some([0.0, -FALL_SPEED, 0.0]), 0.35);
        assert!(watch.falling());
    }

    #[test]
    fn resets_are_kept_up_to_a_cap_and_fit_the_file() {
        let mut rows = Vec::new();
        for n in 0..30 {
            push_capped(&mut rows, format!("row{n}"), RESETS_KEPT);
        }
        assert_eq!(rows.len(), RESETS_KEPT);
        assert_eq!(rows[0], "row6");
        assert_eq!(rows.last().unwrap(), "row29");
        // Everything fits: all of it, joined.
        assert_eq!(joined_within(&rows[22..], 100), "row28/row29");
        // Short of room the oldest go first, never the newest.
        assert_eq!(joined_within(&rows[22..], 8), "row29");
        assert_eq!(joined_within(&rows[22..], 5), "row29");
        // No room for even one: nothing, not half a row.
        assert_eq!(joined_within(&rows, 3), "");
        assert_eq!(joined_within(&[], 10), "");
        // 24 full recordings of about 250 bytes are well inside what the file allows.
        const { assert!(RESETS_KEPT * 300 < 8 * 1024) };
    }

    #[test]
    fn the_knockback_threshold_stays_in_bounds() {
        assert_eq!(clamp_knockback(f32::NAN), DEFAULT_KNOCKBACK);
        assert_eq!(clamp_knockback(0.0), MIN_KNOCKBACK);
        assert_eq!(clamp_knockback(50.0), MAX_KNOCKBACK);
        assert_eq!(clamp_knockback(0.3), 0.3);
    }
}
