//! The per-frame fuel budget: how much is left, the optional per-feature profile and its report.

use super::*;

/// Whether to measure where each frame's fuel goes, by feature. Off: every `timed` call is just
/// the work, with no extra host calls and no periodic save. Switch it on to profile: the numbers
/// are then written to the settings file as `fuel_report` (not read back) about every
/// `FUEL_SAVE_EVERY` seconds of play.
pub(super) const PROFILE: bool = false;

pub(super) const SECTIONS: [&str; 21] = [
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

/// Notes the fuel the frame that started with `start` left over: the start page shows the last
/// and the busiest frame, and a frame near the limit is reported once. A frame has little fuel,
/// so this watches it rather than letting a trap be the first sign.
pub(super) fn note_fuel(start: u64) {
    let used = start.saturating_sub(budget::remaining_fuel());
    record(S_TOTAL, used);
    let peak = FUEL_PEAK.with(|peak| {
        peak.set(peak.get().max(used));
        peak.get()
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
    // Every so often, with fuel to spare and the menu closed, write the numbers down.
    let now = SECONDS.with(StdCell::get);
    if PROFILE
        && now - FUEL_SAVED_AT.with(StdCell::get) >= FUEL_SAVE_EVERY
        && !MENU_OPEN.with(StdCell::get)
        && headroom() >= REPORT_RESERVE
    {
        FUEL_SAVED_AT.with(|at| at.set(now));
        save_settings(&settings());
    }
    let seconds = SECONDS.with(StdCell::get);
    let mut state = MENU.with(StdCell::get);
    if state.page != panel::Page::Home
        || !MENU_OPEN.with(StdCell::get)
        || seconds - FUEL_SHOWN_AT.with(StdCell::get) < FUEL_SHOWN_EVERY
    {
        return;
    }
    FUEL_SHOWN_AT.with(|at| at.set(seconds));
    let thousands = |fuel: u64| u16::try_from(fuel / 1000).unwrap_or(u16::MAX);
    state.fuel_k = [thousands(used), thousands(peak)];
    MENU.with(|menu| menu.set(state));
    let _ = host_panel::update_content(&panel::fuel_update(state));
}

#[cfg(test)]
mod tests;
