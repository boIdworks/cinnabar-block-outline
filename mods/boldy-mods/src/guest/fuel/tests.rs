use super::*;

fn set_stats(stats: &[(usize, Stat)]) {
    FUEL_STATS.with(|all| {
        let mut all = all.borrow_mut();
        *all = Default::default();
        for (slot, stat) in stats {
            all[*slot] = *stat;
        }
    });
}

#[test]
fn every_section_has_a_name_and_the_total_is_where_the_report_looks() {
    assert_eq!(SECTIONS.len(), 32);
    assert_eq!(SECTIONS[S_COORDS], "coords");
    assert_eq!(SECTIONS[S_QUEUE], "queue");
    assert_eq!(SECTIONS[S_FUEL_SAVE], "fuel_save");
    assert_eq!(SECTIONS[S_FUEL_SHOW], "fuel_show");
    assert_eq!(SECTIONS[S_HOTBAR], "hotbar");
    assert_eq!(SECTIONS[S_ARMOR], "armor");
    assert_eq!(SECTIONS[S_TOTAL], "total");
    assert_eq!(SECTIONS[S_TERRAIN], "terrain");
    assert_eq!(SECTIONS[S_CROSSHAIR], "crosshair");
    let mut names = SECTIONS.to_vec();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), SECTIONS.len(), "no two sections share a name");
}

#[test]
fn there_is_no_report_before_the_first_frame() {
    set_stats(&[]);
    assert_eq!(fuel_report(), "");
    // Other sections alone do not make a report: only a measured frame does.
    set_stats(&[(
        S_MENU,
        Stat {
            sum: 90,
            max: 50,
            calls: 3,
        },
    )]);
    assert_eq!(fuel_report(), "");
}

#[test]
fn the_report_gives_average_most_and_calls_per_section() {
    set_stats(&[
        (
            S_TOTAL,
            Stat {
                sum: 3000,
                max: 1500,
                calls: 3,
            },
        ),
        (
            S_MENU,
            Stat {
                sum: 90,
                max: 50,
                calls: 3,
            },
        ),
    ]);
    let report = fuel_report();
    let parts: Vec<&str> = report.split(';').collect();
    assert_eq!(parts.len(), SECTIONS.len());
    assert_eq!(parts[S_TOTAL], "total=1000/1500/3");
    assert_eq!(parts[S_MENU], "menu=30/50/3");
    assert_eq!(
        parts[S_READS], "reads=0/0/0",
        "unused sections read zero, not divide by zero"
    );
}

#[test]
fn the_first_frame_seeds_the_average_instead_of_climbing_from_zero() {
    let mut tracker = FrameFuel::new();
    tracker.note(80_000);
    assert_eq!(tracker.avg(), 80_000);
    assert_eq!(tracker.recent(), 80_000);
    assert_eq!(tracker.peak(), 80_000);
}

#[test]
fn the_average_follows_the_frames_without_jumping_on_one_spike() {
    let mut tracker = FrameFuel::new();
    for _ in 0..100 {
        tracker.note(40_000);
    }
    assert!(tracker.avg().abs_diff(40_000) < 16);
    tracker.note(160_000);
    // A sixteenth of the spike's weight: well above before, well below the spike.
    assert!(tracker.avg() > 40_000 && tracker.avg() < 60_000);
    for _ in 0..200 {
        tracker.note(10_000);
    }
    assert!(
        tracker.avg().abs_diff(10_000) < 16,
        "it settles on the new level"
    );
}

#[test]
fn a_spike_stays_in_recent_for_one_to_two_windows_then_fades_but_the_peak_stays() {
    let mut tracker = FrameFuel::new();
    tracker.note(190_000);
    for _ in 0..usize::from(RECENT_WINDOW) - 1 {
        tracker.note(20_000);
    }
    // The window holding the spike has just closed: it is the previous window now.
    assert_eq!(tracker.recent(), 190_000);
    for _ in 0..RECENT_WINDOW - 1 {
        tracker.note(20_000);
    }
    assert_eq!(tracker.recent(), 190_000, "still within the window before");
    tracker.note(20_000);
    assert_eq!(
        tracker.recent(),
        20_000,
        "two windows on, the spike has faded"
    );
    assert_eq!(tracker.peak(), 190_000, "the all-time peak never fades");
}

#[test]
fn the_tracker_is_cheap_to_keep() {
    // It sits in a `Cell` and is copied in and out each frame: keep it a few words.
    assert!(std::mem::size_of::<FrameFuel>() <= 48);
}

#[test]
fn the_frame_cap_is_what_the_budget_comments_say() {
    assert_eq!(FRAME_CAP, 200_000);
    const {
        assert!(
            FUEL_WARN < FRAME_CAP,
            "a warning has to come before the cap"
        )
    };
}
