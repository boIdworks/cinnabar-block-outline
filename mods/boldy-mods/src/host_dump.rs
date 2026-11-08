//! Writes every HUD and menu document and patch the mod sends, in the order a session sends them,
//! so Cinnabar's own admission code can check them (`scripts/check-host.py`, which runs it). By hand:
//! `BOLDY_DUMP_DIR=<dir> cargo test --lib host_dump -- --ignored`.

use crate::crosshair::{self, CUSTOM, Crosshair, DEFAULT_DRAWING, Drawing, RESOLUTIONS, STYLES};
use crate::outline::Settings;
use crate::panel::{self, MenuState, Page};

struct Dump {
    dir: std::path::PathBuf,
    count: usize,
}

impl Dump {
    /// Writes each of `json`'s lines as a patch of its own: a crosshair is sent in steps.
    fn steps(&mut self, kind: &str, label: &str, json: &str) {
        for (step, line) in json.lines().enumerate() {
            self.write(kind, &format!("{label}-{step}"), line);
        }
    }

    fn write(&mut self, kind: &str, label: &str, json: &str) {
        let name = format!("{:03}-{kind}-{label}.json", self.count);
        self.count += 1;
        std::fs::write(self.dir.join(name), json).expect("write the dump");
    }
}

#[test]
#[ignore = "writes files for scripts/check-host.py"]
fn host_dump() {
    let Ok(dir) = std::env::var("BOLDY_DUMP_DIR") else {
        panic!("set BOLDY_DUMP_DIR");
    };
    let dir = std::path::PathBuf::from(dir);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut dump = Dump { dir, count: 0 };

    // The HUD layer and every crosshair it can show.
    dump.write("hud_set", "surface", &crate::counter::surface());
    let viewport = [427.0, 240.5, 3.0];
    // A checkerboard, far too detailed: it is drawn in part.
    let mut checks = Drawing::empty(64);
    for row in 0..64 {
        for col in (row % 2..64).step_by(2) {
            checks = checks.painted(row, col, true, false);
        }
    }
    let mut ring64 = Drawing::empty(64);
    for step in 0..256 {
        let angle = step as f32 / 256.0 * std::f32::consts::TAU;
        let (row, col) = (32.0 + 28.0 * angle.sin(), 32.0 + 28.0 * angle.cos());
        ring64 = ring64.painted(row as usize, col as usize, true, false);
    }
    let mut looks: Vec<(String, Crosshair)> = (0..STYLES.len() as u8)
        .map(|style| {
            (
                format!("style{style}"),
                Crosshair {
                    on: true,
                    style,
                    ..crosshair::DEFAULT
                },
            )
        })
        .collect();
    let big = |look: Crosshair| Crosshair {
        length: crosshair::LENGTH.max,
        thickness: crosshair::THICKNESS.max,
        gap: crosshair::GAP.max,
        opacity: 30,
        outline: false,
        ..look
    };
    looks.extend((0..STYLES.len() as u8).map(|style| {
        (
            format!("big{style}"),
            big(Crosshair {
                on: true,
                style,
                ..crosshair::DEFAULT
            }),
        )
    }));
    for (label, pixels, pixel_size) in [
        ("empty", Drawing::empty(16), 2),
        ("default32", DEFAULT_DRAWING.resized(32), 1),
        ("ring64", ring64, 8),
        ("checks", checks, 8),
    ] {
        looks.push((
            format!("custom-{label}"),
            Crosshair {
                on: true,
                style: CUSTOM,
                pixels,
                pixel_size,
                ..crosshair::DEFAULT
            },
        ));
    }
    for (label, look) in &looks {
        let view = crosshair::View {
            look: *look,
            viewport,
        };
        dump.steps(
            "hud_patch",
            &format!("crosshair-{label}"),
            &crosshair::patch(Some(&view)),
        );
    }
    dump.write("hud_patch", "crosshair-hidden", &crosshair::patch(None));
    dump.write("hud_patch", "crosshair-shown", &crosshair::visibility(true));
    // The drawing grid the menu embeds, for an empty, the default, a busy and a full drawing.
    let mut grids = vec![
        ("empty".to_owned(), Drawing::empty(16)),
        ("ring64".to_owned(), ring64),
        ("checks".to_owned(), checks),
    ];
    for size in RESOLUTIONS {
        grids.push((format!("default{size}"), DEFAULT_DRAWING.resized(size)));
        grids.push((
            format!("full{size}"),
            Drawing::parse(&"ffffffffffffffff/".repeat(64), size),
        ));
    }
    for (label, pixels) in grids {
        dump.steps(
            "hud_patch",
            &format!("grid-{label}"),
            &crosshair::grid_patch(&pixels, [200, 40, 90]),
        );
    }

    // The grid's highlight: off, on one pixel, and on its reflections.
    for (label, size, cell, mirror) in [
        ("off", 16, None, true),
        ("one", 64, Some((5, 9)), false),
        ("mirrored", 32, Some((0, 31)), true),
    ] {
        dump.write(
            "hud_patch",
            &format!("grid-cursor-{label}"),
            &crosshair::grid_cursor_patch(size, cell, mirror),
        );
    }

    // The menu, and an update for every page and view.
    dump.write("panel_set", "bundle", &panel::bundle());
    let mut settings = Settings::default();
    settings.crosshair.on = true;
    let pages = [
        Page::Home,
        Page::Block,
        Page::Mob,
        Page::Counter,
        Page::Cps,
        Page::Fps,
        Page::Zoom,
        Page::Lighting,
        Page::JumpReset,
        Page::Crosshair,
    ];
    for page in pages {
        for (view, colour_open, drawing) in [
            ("page", false, false),
            ("colour", true, false),
            ("draw", false, true),
        ] {
            let state = MenuState {
                page,
                colour_open,
                drawing,
                focus: Some(3),
                ..MenuState::default()
            };
            dump.write(
                "panel_patch",
                &format!("{page:?}-{view}"),
                &panel::update(&settings, 1.0, state),
            );
        }
    }
    let drawing = MenuState {
        page: Page::Crosshair,
        drawing: true,
        ..MenuState::default()
    };
    settings.crosshair.style = CUSTOM;
    dump.write(
        "panel_patch",
        "crosshair-custom",
        &panel::update(&settings, 1.0, drawing),
    );
    dump.write(
        "panel_patch",
        "draw-update",
        &panel::draw_update(&settings, drawing),
    );
    settings.crosshair.pixels = checks;
    dump.write(
        "panel_patch",
        "draw-update-checks",
        &panel::draw_update(&settings, drawing),
    );
    dump.write(
        "panel_patch",
        "draw-full",
        &panel::draw_update(
            &settings,
            MenuState {
                draw_full: true,
                ..drawing
            },
        ),
    );
}
