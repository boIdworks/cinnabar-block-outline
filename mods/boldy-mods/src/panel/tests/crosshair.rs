//! The crosshair page and the drawing view.

use super::*;

#[test]
fn the_crosshair_page_shows_values_that_step_and_reset() {
    use crate::crosshair::{self, DEFAULT};
    let mut s = Settings::default();
    let mut m = MenuState::default();
    assert!(
        !s.crosshair.on,
        "the game's own crosshair until switched on"
    );
    assert!(press(&mut s, &mut m, "open_crosshair", 1.0));
    assert_eq!(m.page, Page::Crosshair);
    let shown = update(&s, 0.0, m);
    assert!(
        shown.contains(r##""#title_text":"Crosshair""##)
            && shown.contains(r##""#crosshair_page_visible":true"##)
            && shown.contains(r##""#crosshair_top_visible":true"##)
            && shown.contains(r##""#colour_visible":false"##)
            && shown.contains(r##""#settings_visible":false"##),
        "{shown}"
    );
    // The HUD stays visible behind the page, so the crosshair can be seen while it changes.
    assert!(shown.contains(r##""show_gameplay_hud":true"##));
    assert!(!switch_on(&shown, "crosshair_on"));
    assert!(switch_on(&shown, "crosshair_outline"));
    for text in [
        "Style: Cross",
        "Length: 4",
        "Thickness: 1",
        "Gap: 2",
        "Opacity: 100%%",
    ] {
        assert!(shown.contains(text), "{text} in {shown}");
    }
    assert!(shown.contains(r##""#crosshair_length_reset":"""##));
    // Each value steps, stays in bounds and resets from its own button.
    assert!(press(&mut s, &mut m, "crosshair_length_more", 1.0));
    assert_eq!(s.crosshair.length, 4.5);
    let shown = update(&s, 0.0, m);
    assert!(
        shown.contains("Length: 4.5") && shown.contains(r##""#crosshair_length_reset":"reset""##)
    );
    assert!(press(&mut s, &mut m, "crosshair_length", 1.0));
    assert_eq!(s.crosshair.length, DEFAULT.length);
    for _ in 0..50 {
        press(&mut s, &mut m, "crosshair_thickness_more", 1.0);
        press(&mut s, &mut m, "crosshair_gap_less", 1.0);
        press(&mut s, &mut m, "crosshair_opacity_less", 1.0);
    }
    assert_eq!(s.crosshair.thickness, crosshair::THICKNESS.max);
    assert_eq!(s.crosshair.gap, crosshair::GAP.min);
    assert_eq!(s.crosshair.opacity, crosshair::MIN_OPACITY);
    assert!(
        !press(&mut s, &mut m, "crosshair_gap_less", 1.0),
        "at the bottom"
    );
    // Styles go round in a circle.
    assert!(press(&mut s, &mut m, "crosshair_style_less", 1.0));
    assert_eq!(crosshair::style_name(s.crosshair.style), "Custom");
    let shown = update(&s, 0.0, m);
    assert!(shown.contains("Style: Custom") && shown.contains("Pixel size: 2 px"));
    assert!(shown.contains("Gap: not used by a drawing"), "{shown}");
    assert!(press(&mut s, &mut m, "crosshair_style_more", 1.0));
    assert_eq!(s.crosshair.style, 0);
    // The switches, and the start page's one, are the same setting.
    assert!(press(&mut s, &mut m, "crosshair_on", 1.0));
    assert!(s.crosshair.on);
    assert!(press(&mut s, &mut m, "crosshair_outline", 0.0));
    assert!(!s.crosshair.outline);
    // Reset asks first, puts the look back and leaves it switched on.
    s.crosshair.style = 4;
    assert!(confirmed(&mut s, &mut m, "crosshair_reset"));
    assert_eq!(
        s.crosshair,
        crosshair::Crosshair {
            on: true,
            ..DEFAULT
        }
    );
    // Its controls do nothing from other pages, and an outline page's Reset leaves it alone.
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert!(!press(&mut s, &mut m, "crosshair_length_more", 1.0));
    assert!(press(&mut s, &mut m, "home_crosshair", 0.0));
    assert!(!s.crosshair.on);
    assert!(!switch_on(&update(&s, 0.0, m), "home_crosshair"));
    s.crosshair.gap = 7.0;
    s.reset();
    assert_eq!(s.crosshair.gap, 7.0);
}

#[test]
fn the_colour_tools_edit_the_crosshair_colour_alone() {
    let mut s = Settings::default();
    let mut m = MenuState::default();
    let line = s.rgba;
    press(&mut s, &mut m, "open_crosshair", 1.0);
    assert!(press(&mut s, &mut m, "crosshair_colour", 1.0));
    assert!(m.colour_open);
    let shown = update(&s, 0.0, m);
    assert!(
        shown.contains(r##""#title_text":"Crosshair > Colour""##)
            && shown.contains(r##""#colour_visible":true"##)
            && shown.contains(r##""#crosshair_page_visible":false"##)
            && shown.contains(r##""#crosshair_top_visible":true"##),
        "{shown}"
    );
    assert_eq!(card_height(m), CARD_H);
    // Quick colours, the bars, typed hex codes and saved colours all set the crosshair's.
    assert!(press(&mut s, &mut m, "swatch_lime", 1.0));
    assert_eq!(s.crosshair.rgb, [80, 255, 60]);
    assert!(press(&mut s, &mut m, "red", 10.0));
    assert_eq!(s.crosshair.rgb, [10, 255, 60]);
    assert_eq!(apply_text(&mut s, m, "hex", "#FF000080"), Some(true));
    assert_eq!(s.crosshair.rgb, [255, 0, 0]);
    assert!(update(&s, 0.0, m).contains(r##""#hex_text":"#FF0000""##));
    assert!(press(&mut s, &mut m, "save_favourite", 1.0));
    assert_eq!(s.favourites[0], Some([255, 0, 0, 255]));
    assert_eq!(s.rgba, line, "the block outline keeps its colour");
    assert_eq!(
        s.crosshair.opacity, 100,
        "a hex code's alpha is not the opacity"
    );
    // The swatch on its button shows it.
    assert!(update(&s, 0.0, m).contains(r##""#crosshair_swatch":[1,0,0,1]"##));
    // Back, or the button again, hides the tools.
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert!(!m.colour_open && m.page == Page::Crosshair);
    press(&mut s, &mut m, "crosshair_colour", 1.0);
    assert!(press(&mut s, &mut m, "crosshair_colour", 1.0));
    assert!(!m.colour_open);
}

#[test]
fn the_keyboard_reaches_every_control_of_the_crosshair_page() {
    let s = Settings::default();
    let mut m = MenuState {
        page: Page::Crosshair,
        ..MenuState::default()
    };
    focus_on(&s, &mut m, &Action::Toggle("crosshair_outline"));
    assert_eq!(focused(&m), "A thin dark edge, seen on any ground");
    // Left moves to its neighbour, the Show switch.
    keyed(&s, &mut m, &["ArrowLeft"]);
    assert_eq!(focused(&m), "Use it in place of the game's crosshair");
    focus_on(
        &s,
        &mut m,
        &Action::Stepper {
            value: "crosshair_gap",
            less: "crosshair_gap_less",
            more: "crosshair_gap_more",
        },
    );
    assert_eq!(
        keyed(&s, &mut m, &["ArrowRight"]).events,
        vec![("crosshair_gap_more".to_owned(), 1.0)]
    );
    assert_eq!(enter(&s, &mut m), vec![("crosshair_gap".to_owned(), 1.0)]);
    let state = m;
    let tip = |point| tip_at(point, state, false).map(|index| controls()[index].tip);
    assert_eq!(
        tip([15.0, CROSS_STYLE_Y + 5.0]),
        Some("The shape; click to go back to a cross")
    );
    assert_eq!(
        tip([SETTINGS_W - 15.0, CROSS_OPACITY_Y + 5.0]),
        Some("10%% less or more")
    );
    assert_eq!(
        tip([15.0, EDIT_Y + 5.0]),
        Some("Shows or hides the colour tools")
    );
    assert!(card_height(m) <= CARD_H && card_height(MenuState::default()) <= CARD_H);
    assert_eq!(
        tip_at([15.0, HOME_CROSS_Y + 5.0], MenuState::default(), false)
            .map(|index| controls()[index].tip),
        Some("Shape, size and colour of your crosshair")
    );
}

#[test]
fn the_drawing_view_opens_from_the_crosshair_page_and_clears_on_confirm() {
    use crate::crosshair::{CUSTOM, DEFAULT_DRAWING, Drawing};
    let mut s = Settings::default();
    let mut m = MenuState::default();
    press(&mut s, &mut m, "open_crosshair", 1.0);
    assert!(press(&mut s, &mut m, "crosshair_draw", 1.0));
    assert!(m.drawing && !m.colour_open);
    assert!(
        s.crosshair.on && s.crosshair.style == CUSTOM,
        "drawing shows the drawing"
    );
    let shown = update(&s, 0.0, m);
    assert!(
        shown.contains(r##""#title_text":"Crosshair > Draw""##)
            && shown.contains(r##""#draw_page_visible":true"##)
            && shown.contains(r##""#crosshair_page_visible":false"##)
            && shown.contains(r##""#crosshair_top_visible":true"##)
            && shown.contains("Left click draws"),
        "{shown}"
    );
    // The help line is sent only while the grid shows. The grid itself is the HUD's
    // `drawgrid`, which the menu embeds: a binding cannot hold its pixels.
    assert!(!update(&s, 0.0, MenuState::default()).contains("#draw_note_text"));
    assert!(!shown.contains("paths"), "no points in a menu update");
    let document = document();
    assert!(document.contains(r#""renderer":"cinnabar_hud_control","control":"drawgrid""#));
    assert!(card_height(m) <= CARD_H);
    // The grid's squares map to rows and columns; outside it there is none.
    let [x, y, side] = DRAW_GRID;
    for size in crate::crosshair::RESOLUTIONS.map(usize::from) {
        let cell = side / size as f32;
        assert_eq!(draw_cell([x + 0.5, y + 0.5], size), Some((0, 0)));
        assert_eq!(
            draw_cell([x + cell * 7.5, y + cell * 2.5], size),
            Some((2, 7))
        );
        let last = size - 1;
        assert_eq!(
            draw_cell([x + side - 0.1, y + side - 0.1], size),
            Some((last, last))
        );
        assert_eq!(draw_cell([x - 1.0, y + 5.0], size), None);
        assert_eq!(draw_cell([x + 5.0, y + side], size), None);
    }
    // The resolution steps through 16, 32 and 64, keeping the drawing's shape and its size on
    // screen; its own button goes back to 16.
    assert!(update(&s, 0.0, m).contains(r##""#crosshair_res_text":"16x16""##));
    assert!(press(&mut s, &mut m, "crosshair_res_more", 1.0));
    assert_eq!((s.crosshair.pixels.size(), s.crosshair.pixel_size), (32, 1));
    assert_eq!(s.crosshair.pixels, DEFAULT_DRAWING.resized(32));
    let shown = update(&s, 0.0, m);
    assert!(shown.contains(r##""#crosshair_res_text":"32x32""##));
    assert!(press(&mut s, &mut m, "crosshair_res_more", 1.0));
    assert_eq!(s.crosshair.pixels.size(), 64);
    assert!(
        !press(&mut s, &mut m, "crosshair_res_more", 1.0),
        "64 is the most"
    );
    assert!(press(&mut s, &mut m, "crosshair_res", 1.0));
    assert_eq!(s.crosshair.pixels, DEFAULT_DRAWING);
    // On a drawing, the thickness value is its pixel size in screen pixels.
    // Back from 64 to 16 pixels, each pixel is four times as wide, so it stays as large.
    assert!(update(&s, 0.0, m).contains("Pixel size: 4 px"));
    assert!(press(&mut s, &mut m, "crosshair_thickness_more", 1.0));
    assert_eq!(s.crosshair.pixel_size, 5);
    assert_eq!(
        s.crosshair.thickness,
        crate::crosshair::DEFAULT.thickness,
        "lines unchanged"
    );
    // Mirror is a switch; Clear asks first and empties the grid.
    assert!(press(&mut s, &mut m, "crosshair_mirror", 0.0));
    assert!(!s.crosshair.mirror);
    assert!(confirmed(&mut s, &mut m, "crosshair_clear"));
    assert_eq!(s.crosshair.pixels, Drawing::empty(16));
    let shown = draw_update(&s, m);
    assert!(shown.contains("Left click draws"), "{shown}");
    // Too detailed says so.
    m.draw_full = true;
    assert!(draw_update(&s, m).contains("Too detailed"));
    // Reset keeps the drawing; Back goes to the page, then to the start.
    s.crosshair.pixels = DEFAULT_DRAWING;
    s.crosshair.gap = 5.0;
    assert!(confirmed(&mut s, &mut m, "crosshair_reset"));
    assert_eq!(s.crosshair.pixels, DEFAULT_DRAWING);
    assert_eq!(s.crosshair.gap, 2.0);
    press(&mut s, &mut m, "crosshair_draw", 1.0);
    assert!(press(&mut s, &mut m, "back", 1.0));
    assert!(!m.drawing && m.page == Page::Crosshair);
    // The colour button leaves the drawing for the colour tools.
    press(&mut s, &mut m, "crosshair_draw", 1.0);
    press(&mut s, &mut m, "crosshair_colour", 1.0);
    assert!(m.colour_open && !m.drawing);
    let tip =
        |point| tip_at(point, MenuState { drawing: true, ..m }, false).map(|i| controls()[i].tip);
    let _ = tip;
    let drawing = MenuState {
        page: Page::Crosshair,
        drawing: true,
        ..MenuState::default()
    };
    assert_eq!(
        tip_at([x + 20.0, y + 20.0], drawing, false).map(|i| controls()[i].tip),
        Some("Left click draws, right click erases")
    );
}
