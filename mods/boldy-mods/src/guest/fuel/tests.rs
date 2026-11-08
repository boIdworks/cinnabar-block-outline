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
    assert_eq!(SECTIONS.len(), 21);
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
fn the_frame_cap_is_what_the_budget_comments_say() {
    assert_eq!(FRAME_CAP, 200_000);
    const {
        assert!(
            FUEL_WARN < FRAME_CAP,
            "a warning has to come before the cap"
        )
    };
}
