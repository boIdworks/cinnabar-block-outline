use super::*;
use super::{
    bindings::{CAPTURE_MENU, CAPTURE_OPTIONAL},
    controls::*,
    help::*,
    json::*,
};
use crate::outline::KeyName;

const FILL: MenuState = MenuState {
    capturing: false,
    capturing_fullbright: false,
    capturing_zoom: false,
    editing_fill: true,
    editing_aim: false,
    editing_arrow: false,
    page: Page::Block,
    colour_open: true,
    typing: None,
    confirm: Reset::None,
    focus: None,
    moving: false,
    moving_cps: false,
    moving_fps: false,
    moving_armor: false,
    guide_bottom: false,
    fuel_k: [0, 0, 0],
    drawing: false,
    draw_full: false,
};

/// A page's state, with the colour tools open or not.
fn on(colour_open: bool) -> MenuState {
    MenuState {
        page: Page::Block,
        colour_open,
        ..MenuState::default()
    }
}

/// Whether the update shows switch `name` on: its pill in the "on" colour, its knob at the right.
fn switch_on(update: &str, name: &str) -> bool {
    let right = PILL_W - KNOB - KNOB_OFF;
    let knob = format!(r##""#{name}_knob":[{right},0]"##);
    let off = format!(r##""#{name}_knob":[{KNOB_OFF},0]"##);
    assert!(
        update.contains(&knob) != update.contains(&off),
        "{name} has exactly one knob place"
    );
    let on = update.contains(&knob);
    let colour = format!(r##""#{name}_bg":[0.45,0.24,0.75,1]"##);
    assert_eq!(
        update.contains(&colour),
        on,
        "{name}'s pill follows its knob"
    );
    on
}

/// Presses a reset button twice, as the menu asks: the first click only asks for confirmation.
fn confirmed(settings: &mut Settings, state: &mut MenuState, id: &str) -> bool {
    let before = *settings;
    assert!(press(settings, state, id, 1.0), "the first click asks");
    assert_eq!(*settings, before, "and changes nothing");
    press(settings, state, id, 1.0)
}

fn press(settings: &mut Settings, state: &mut MenuState, id: &str, value: f32) -> bool {
    apply(settings, state, id, value)
}

mod crosshair;
mod document;
mod keyboard;
mod navigation;
mod pages;
mod settings;

use keyboard::*;
