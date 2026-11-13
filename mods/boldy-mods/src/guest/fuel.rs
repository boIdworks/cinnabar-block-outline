//! The per-frame fuel budget: how much is left, the optional per-feature profile and its report.

use super::*;

/// Whether to measure where each frame's fuel goes, by feature. Off (the shipped build): every
/// `timed` call is just the work, with no extra host calls and no periodic save. Build with the
/// `fuel-profile` feature (`python scripts\build-mod.py --fuel-profile`) to profile: the numbers
/// are then written to the settings file as `fuel_report` (not read back) about every
/// `FUEL_SAVE_EVERY` seconds of play.
pub(super) const PROFILE: bool = cfg!(feature = "fuel-profile");

/// The profile's sections. Each number in the report is the average **per call**, not per frame:
/// a section called once every few frames averages high on its own calls and low per frame (the
/// third number, the calls, says which). Some sections are inside others and are not extra to
/// them: `armor` lies inside `arrow`, `hotbar` inside `cps` and `xh_grid` inside `crosshair`, so
/// adding every average double-counts. `total` is the whole frame, including `fuel_save` and `fuel_show`.
pub(super) const SECTIONS: [&str; 32] = [
    "reads",
    "menu",
    "arrow",
    "cps",
    "jump",
    "zoom",
    "terrain",
    "actors",
    "draw",
    "total",
    "fullbright",
    "keys",
    "viewport",
    "aim",
    "camera",
    "hidden",
    "mining",
    "boxes",
    "keep",
    "shapes",
    "crosshair",
    "xh_shown",
    "xh_hide",
    "xh_plan",
    "xh_grid",
    "effects",
    "hotbar",
    "armor",
    "fuel_save",
    "fuel_show",
    "coords",
    "queue",
];

pub(super) const S_READS: usize = 0;

pub(super) const S_MENU: usize = 1;

pub(super) const S_ARROW: usize = 2;

pub(super) const S_CPS: usize = 3;

pub(super) const S_JUMP: usize = 4;

pub(super) const S_ZOOM: usize = 5;

pub(super) const S_TERRAIN: usize = 6;

pub(super) const S_ACTORS: usize = 7;

pub(super) const S_DRAW: usize = 8;

pub(super) const S_TOTAL: usize = 9;

pub(super) const S_FULLBRIGHT: usize = 10;

pub(super) const S_KEYS: usize = 11;

pub(super) const S_VIEWPORT: usize = 12;

pub(super) const S_AIM: usize = 13;

pub(super) const S_CAMERA: usize = 14;

pub(super) const S_HIDDEN: usize = 15;

pub(super) const S_MINING: usize = 16;

pub(super) const S_BOXES: usize = 17;

pub(super) const S_KEEP: usize = 18;

pub(super) const S_SHAPES: usize = 19;

pub(super) const S_CROSSHAIR: usize = 20;

/// Parts of the crosshair's frame: the check that the game's own crosshair would show, hiding it,
/// the plan comparison, and the drawing grid. Each is part of `crosshair`, not extra to it.
pub(super) const S_XH_SHOWN: usize = 21;

pub(super) const S_XH_HIDE: usize = 22;

pub(super) const S_XH_PLAN: usize = 23;

pub(super) const S_XH_GRID: usize = 24;
pub(super) const S_EFFECTS: usize = 25;

/// The hotbar key labels, and the armor HUD (part of `arrow`, not extra to it).
pub(super) const S_HOTBAR: usize = 26;

pub(super) const S_ARMOR: usize = 27;

/// The fuel bookkeeping itself, run before the frame total is taken so the total includes it: the
/// profile's periodic settings save (profile builds only) and the start page's fuel line.
pub(super) const S_FUEL_SAVE: usize = 28;

pub(super) const S_FUEL_SHOW: usize = 29;

/// The coordinates line's position read (part of `cps`, not extra to it; a call every frame, of
/// which about one in six reads).
pub(super) const S_COORDS: usize = 30;

/// The auto queue: a setting read when it is off; when on, a chat read every quarter second.
pub(super) const S_QUEUE: usize = 31;

/// How often the numbers are saved, and the fuel a frame needs left for that (a save costs about
/// 65,000).
pub(super) const FUEL_SAVE_EVERY: f32 = 30.0;

pub(super) const REPORT_RESERVE: u64 = 120_000;

/// The fuel one feature used over all its calls: the sum, the most in one call and the calls.
#[derive(Clone, Copy, Default)]
pub(super) struct Stat {
    pub(super) sum: u64,
    pub(super) max: u32,
    pub(super) calls: u32,
}

/// Frames in one window of [`FrameFuel::recent`] (about five seconds at 60 fps).
pub(super) const RECENT_WINDOW: u16 = 300;

/// Always-on tracking of what whole frames cost: a running average, the busiest frame of the last
/// few seconds and the busiest ever. It makes no host calls and costs a few integer operations a
/// frame, so it stays on in the shipped build (unlike the per-feature profile).
#[derive(Clone, Copy, Default)]
pub(super) struct FrameFuel {
    frames: u32,
    avg: u64,
    peak: u64,
    window_peak: u64,
    last_window_peak: u64,
    window_frames: u16,
}

impl FrameFuel {
    pub(super) const fn new() -> Self {
        Self {
            frames: 0,
            avg: 0,
            peak: 0,
            window_peak: 0,
            last_window_peak: 0,
            window_frames: 0,
        }
    }

    /// Takes in a frame that used `used` fuel.
    pub(super) fn note(&mut self, used: u64) {
        // A moving average that gives each frame a sixteenth of the weight; the first frame
        // seeds it so it does not climb up from zero.
        self.avg = if self.frames == 0 {
            used
        } else {
            self.avg - (self.avg >> 4) + (used >> 4)
        };
        self.frames = self.frames.saturating_add(1);
        self.peak = self.peak.max(used);
        self.window_peak = self.window_peak.max(used);
        self.window_frames += 1;
        if self.window_frames >= RECENT_WINDOW {
            self.last_window_peak = self.window_peak;
            self.window_peak = 0;
            self.window_frames = 0;
        }
    }

    /// The moving average of a frame's fuel.
    pub(super) fn avg(&self) -> u64 {
        self.avg
    }

    /// The most any frame used in the current window and the one before it: between 300 and 600
    /// frames, so a spike fades from view instead of staying for the whole session.
    pub(super) fn recent(&self) -> u64 {
        self.window_peak.max(self.last_window_peak)
    }

    /// The most any frame has used since the mod started.
    pub(super) fn peak(&self) -> u64 {
        self.peak
    }
}

/// What one frame may spend by this mod's own count. Cinnabar's allowance for a frame has been
/// 200,000 and may be more (a world's first frame gets several times that); the mod's gates below
/// are tuned for 200,000 and keep to it, so the work is spread the same way on any version and no
/// frame takes more than its share of the host's time.
pub(super) const FRAME_CAP: u64 = 200_000;

/// The fuel this frame may still spend: what the host has left, and what is left of
/// [`FRAME_CAP`]. Use it instead of `budget::remaining_fuel` to decide whether a piece of
/// optional work fits.
pub(super) fn headroom() -> u64 {
    let left = budget::remaining_fuel();
    let spent = FRAME_START.with(StdCell::get).saturating_sub(left);
    left.min(FRAME_CAP.saturating_sub(spent))
}

/// Notes that feature `slot` used `used` fuel in one call.
pub(super) fn record(slot: usize, used: u64) {
    if !PROFILE {
        return;
    }
    FUEL_STATS.with(|stats| {
        let stat = &mut stats.borrow_mut()[slot];
        stat.sum += used;
        stat.max = stat.max.max(u32::try_from(used).unwrap_or(u32::MAX));
        stat.calls = stat.calls.saturating_add(1);
    });
}

/// Runs `work` and notes the fuel it used under feature `slot`.
pub(super) fn timed<R>(slot: usize, work: impl FnOnce() -> R) -> R {
    if !PROFILE {
        return work();
    }
    let before = budget::remaining_fuel();
    let result = work();
    record(slot, before.saturating_sub(budget::remaining_fuel()));
    result
}

/// The fuel numbers as `name=average/most/calls` per feature, empty before the first frame.
pub(super) fn fuel_report() -> String {
    FUEL_STATS.with(|stats| {
        let stats = stats.borrow();
        if stats[S_TOTAL].calls == 0 {
            return String::new();
        }
        let parts: Vec<String> = SECTIONS
            .iter()
            .zip(stats.iter())
            .map(|(name, stat)| {
                format!(
                    "{name}={}/{}/{}",
                    stat.sum / u64::from(stat.calls.max(1)),
                    stat.max,
                    stat.calls
                )
            })
            .collect();
        parts.join(";")
    })
}

/// Fuel a frame may use before the mod says so: the host allows 200,000 and quarantines the mod
/// after three frames that run out.
pub(super) const FUEL_WARN: u64 = 150_000;

/// How often the menu's fuel line is refreshed, in seconds.
pub(super) const FUEL_SHOWN_EVERY: f32 = 1.0;

/// Notes the fuel the frame that started with `start` left over: the start page shows the
/// average, recent and busiest frames, and a frame near the limit is reported once. A frame has
/// little fuel, so this watches it rather than letting a trap be the first sign.
///
/// The bookkeeping that costs fuel (the profile's save, the menu's fuel line) runs first, on the
/// numbers of the frames before this one, so the total taken afterwards includes it and a frame
/// that did a save is not understated.
pub(super) fn note_fuel(start: u64) {
    maybe_save_report();
    maybe_show_fuel();
    let used = start.saturating_sub(budget::remaining_fuel());
    record(S_TOTAL, used);
    FRAME_FUEL.with(|cell| {
        let mut tracker = cell.get();
        tracker.note(used);
        cell.set(tracker);
    });
    // A frame that took a heavy job to itself is allowed up to the cap; any other is reported
    // over `FUEL_WARN`.
    let limit = if HEAVY_FRAME.with(StdCell::get) {
        FRAME_CAP
    } else {
        FUEL_WARN
    };
    if used > limit {
        report_once("Boldy Mods: a frame used over 150k of its 200k fuel");
    }
}

/// Every so often, with fuel to spare and the menu closed, writes the profile's numbers down.
/// Free (one constant check) in a normal build.
fn maybe_save_report() {
    if !PROFILE {
        return;
    }
    let now = SECONDS.with(StdCell::get);
    if now - FUEL_SAVED_AT.with(StdCell::get) >= FUEL_SAVE_EVERY
        && !MENU_OPEN.with(StdCell::get)
        && headroom() >= REPORT_RESERVE
    {
        FUEL_SAVED_AT.with(|at| at.set(now));
        timed(S_FUEL_SAVE, || save_settings(&settings()));
    }
}

/// Refreshes the start page's fuel line, at most once a second and only while it is showing. The
/// numbers are those of the frames completed before this one.
fn maybe_show_fuel() {
    let seconds = SECONDS.with(StdCell::get);
    let mut state = MENU.with(StdCell::get);
    if state.page != panel::Page::Home
        || !MENU_OPEN.with(StdCell::get)
        || seconds - FUEL_SHOWN_AT.with(StdCell::get) < FUEL_SHOWN_EVERY
    {
        return;
    }
    timed(S_FUEL_SHOW, || {
        FUEL_SHOWN_AT.with(|at| at.set(seconds));
        let tracker = FRAME_FUEL.with(StdCell::get);
        let thousands = |fuel: u64| u16::try_from(fuel / 1000).unwrap_or(u16::MAX);
        state.fuel_k = [
            thousands(tracker.avg()),
            thousands(tracker.recent()),
            thousands(tracker.peak()),
        ];
        MENU.with(|menu| menu.set(state));
        let _ = host_panel::update_content(&panel::fuel_update(state));
    });
}

#[cfg(test)]
mod tests;
