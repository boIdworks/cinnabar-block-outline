//! How the menu's control events, typed keys and text change the settings and menu state.

use super::*;

/// Applies one control event to the settings or the menu state. Returns whether the menu should
/// be redrawn; the caller saves the settings when they changed.
///
/// A reset button first asks "Sure? Click again"; the second click, with nothing else clicked in
/// between, resets. The Reset in the counter's moving guide has no such step: it is right
/// beside what it resets.
pub fn apply(settings: &mut Settings, state: &mut MenuState, id: &str, value: f32) -> bool {
    if !value.is_finite() {
        return false;
    }
    let asked = match id {
        "reset" => Reset::Page,
        "counter_reset" if !state.moving => Reset::Counter,
        "cps_reset" if !state.moving => Reset::Cps,
        "fps_reset" if !state.moving => Reset::Fps,
        "zoom_reset" => Reset::Zoom,
        "crosshair_reset" => Reset::Crosshair,
        "crosshair_clear" => Reset::Drawing,
        _ => Reset::None,
    };
    let was = state.confirm;
    if asked != Reset::None && was != asked {
        state.confirm = asked;
        return true;
    }
    state.confirm = Reset::None;
    let changed = apply_event(settings, state, id, value) || was != Reset::None;
    // A page or view change leaves the focused control hidden: focus the new page's first.
    settle_focus(settings, state);
    changed
}

fn apply_event(settings: &mut Settings, state: &mut MenuState, id: &str, value: f32) -> bool {
    match id {
        "open_block" | "open_mob" | "open_counter" | "open_cps" | "open_fps" | "open_zoom"
        | "open_light" | "open_jump" | "open_crosshair" => {
            if state.page != Page::Home {
                return false;
            }
            state.page = match id {
                "open_block" => Page::Block,
                "open_mob" => Page::Mob,
                "open_zoom" => Page::Zoom,
                "open_cps" => Page::Cps,
                "open_fps" => Page::Fps,
                "open_light" => Page::Lighting,
                "open_jump" => Page::JumpReset,
                "open_crosshair" => Page::Crosshair,
                _ => Page::Counter,
            };
            state.colour_open = false;
            state.editing_fill = false;
            state.editing_aim = false;
            state.drawing = false;
            return true;
        }
        // The start page's switches: each turns a mod on or off from the list.
        "home_block" | "home_mob" | "home_counter" | "home_cps" | "home_fps" | "home_zoom"
        | "home_light" | "home_jump" | "home_crosshair" => {
            if state.page != Page::Home {
                return false;
            }
            let on = value > 0.5;
            match id {
                "home_block" => settings.block_on = on,
                "home_mob" => settings.mob_on = on,
                "home_counter" => settings.counter.on = on,
                "home_cps" => settings.cps.on = on,
                "home_fps" => settings.fps.on = on,
                "home_zoom" => settings.zoom.on = on,
                "home_jump" => settings.jump_reset.on = on,
                "home_crosshair" => settings.crosshair.on = on,
                _ => settings.fullbright = on,
            }
            return true;
        }
        "back" => {
            if state.page == Page::Home {
                return false;
            }
            state.moving = false;
            commit_typing(settings, state);
            // One step back: from the colour tools or the drawing to the page, from the page to
            // the start.
            if state.colour_open {
                state.colour_open = false;
            } else if state.drawing {
                state.drawing = false;
                state.draw_full = false;
            } else {
                state.page = Page::Home;
            }
            return true;
        }
        // Clicking another box applies what was typed in the previous one. Done here, on the
        // real settings: committing from inside a page's edit would be overwritten by it.
        _ if id.starts_with("type_") => commit_typing(settings, state),
        _ if id.starts_with("counter") => return apply_counter(settings, state, id, value),
        _ if id.starts_with("cps") => return apply_cps(settings, state, id, value),
        _ if id.starts_with("fps") => return apply_fps(settings, state, id, value),
        _ if id.starts_with("zoom") => return apply_zoom(settings, id, value),
        _ if id.starts_with("jump") => return apply_jump(settings, id, value),
        _ if id.starts_with("crosshair") => return apply_crosshair(settings, state, id, value),
        _ => {}
    }
    if state.page == Page::Mob {
        let mut mobs = settings.for_mobs();
        let changed = apply_page(&mut mobs, state, id, value);
        settings.store_mob(&mobs);
        return changed;
    }
    apply_page(settings, state, id, value)
}

/// [`apply`] for the Arrow counter page and the Done button shown while moving the counter.
fn apply_counter(settings: &mut Settings, state: &mut MenuState, id: &str, value: f32) -> bool {
    use crate::counter::{Counter, DEFAULT_POS};
    let before = (*settings, *state);
    let counter = &mut settings.counter;
    match id {
        "counter" => counter.on = value > 0.5,
        "counter_bow" => counter.bow_only = value > 0.5,
        "counter_move" => {
            // Something to drag: moving shows the counter even while it is off.
            counter.on = true;
            state.moving = true;
            state.moving_cps = false;
            state.moving_fps = false;
        }
        "counter_done" => state.moving = false,
        "counter_size" => counter.scale = 1.0,
        "counter_smaller" => counter.scale = Counter::stepped(counter.scale, -1.0),
        "counter_bigger" => counter.scale = Counter::stepped(counter.scale, 1.0),
        "counter_reset" if state.moving_fps => {
            // The Reset in the moving guide resets whatever is being moved.
            settings.fps.pos = crate::fps::DEFAULT_POS;
            settings.fps.scale = 1.0;
        }
        "counter_reset" if state.moving_cps => {
            // The Reset in the moving guide resets whatever is being moved.
            settings.cps.pos = crate::cps::DEFAULT_POS;
            settings.cps.scale = 1.0;
        }
        "counter_reset" => {
            counter.pos = DEFAULT_POS;
            counter.scale = 1.0;
        }
        _ => return false,
    }
    (*settings, *state) != before
}

/// [`apply`] for the CPS page: switches, moving, the size and reset.
fn apply_cps(settings: &mut Settings, state: &mut MenuState, id: &str, value: f32) -> bool {
    use crate::cps::{Cps, DEFAULT_POS};
    let before = (*settings, *state);
    let cps = &mut settings.cps;
    match id {
        "cps_on" => cps.on = value > 0.5,
        "cps_right" => cps.right = value > 0.5,
        "cps_zeros" => cps.zeros = value > 0.5,
        "cps_move" => {
            // Something to drag: moving shows the readout even while it is off.
            cps.on = true;
            state.moving = true;
            state.moving_cps = true;
            state.moving_fps = false;
        }
        "cps_size" => cps.scale = 1.0,
        "cps_size_less" => cps.scale = Cps::stepped(cps.scale, -1.0),
        "cps_size_more" => cps.scale = Cps::stepped(cps.scale, 1.0),
        "cps_reset" => {
            cps.pos = DEFAULT_POS;
            cps.scale = 1.0;
        }
        _ => return false,
    }
    (*settings, *state) != before
}

/// [`apply`] for the FPS counter page: the switch, moving, the size and reset.
fn apply_fps(settings: &mut Settings, state: &mut MenuState, id: &str, value: f32) -> bool {
    use crate::cps::Cps;
    let before = (*settings, *state);
    let fps = &mut settings.fps;
    match id {
        "fps_on" => fps.on = value > 0.5,
        "fps_move" => {
            // Something to drag: moving shows the counter even while it is off.
            fps.on = true;
            state.moving = true;
            state.moving_cps = false;
            state.moving_fps = true;
        }
        "fps_size" => fps.scale = 1.0,
        "fps_size_less" => fps.scale = Cps::stepped(fps.scale, -1.0),
        "fps_size_more" => fps.scale = Cps::stepped(fps.scale, 1.0),
        "fps_reset" => {
            fps.pos = crate::fps::DEFAULT_POS;
            fps.scale = 1.0;
        }
        _ => return false,
    }
    (*settings, *state) != before
}

/// [`apply`] for the Jump reset page: the switch, and the perfect window that steps or resets.
fn apply_jump(settings: &mut Settings, id: &str, value: f32) -> bool {
    use crate::jumpreset::{DEFAULT_WINDOW, JumpReset};
    let before = *settings;
    let jump = &mut settings.jump_reset;
    match id {
        "jump_on" => jump.on = value > 0.5,
        "jump_window" => jump.window = DEFAULT_WINDOW,
        "jump_window_less" => jump.window = JumpReset::stepped(jump.window, -1),
        "jump_window_more" => jump.window = JumpReset::stepped(jump.window, 1),
        _ => return false,
    }
    *settings != before
}

/// [`apply`] for the Crosshair page: its switches, the colour button that shows or hides the
/// colour tools, values that step or reset, and Reset, which keeps it on or off. (The colour
/// tools' own controls are the outline pages', aimed at the crosshair's colour.)
fn apply_crosshair(settings: &mut Settings, state: &mut MenuState, id: &str, value: f32) -> bool {
    use crate::crosshair::{self, CUSTOM, DEFAULT, Drawing, GAP, LENGTH, RESOLUTIONS, THICKNESS};
    if state.page != Page::Crosshair {
        return false;
    }
    let before = (*settings, *state);
    let c = &mut settings.crosshair;
    // A value's own button resets it; its - and + step it.
    let (id, step) = match (id.strip_suffix("_less"), id.strip_suffix("_more")) {
        (Some(id), _) => (id, Some(false)),
        (_, Some(id)) => (id, Some(true)),
        _ => (id, None),
    };
    let steps = |up: bool| if up { 1.0 } else { -1.0 };
    match (id, step) {
        ("crosshair_on", None) => c.on = value > 0.5,
        ("crosshair_outline", None) => c.outline = value > 0.5,
        ("crosshair_colour", None) => {
            state.colour_open = !state.colour_open;
            state.drawing = false;
        }
        // Drawing shows the drawing: the Custom style, switched on.
        ("crosshair_draw", None) => {
            state.drawing = true;
            state.colour_open = false;
            state.draw_full = false;
            c.style = CUSTOM;
            c.on = true;
        }
        ("crosshair_mirror", None) => c.mirror = value > 0.5,
        ("crosshair_clear", None) => {
            c.pixels = Drawing::empty(c.pixels.size() as u8);
            state.draw_full = false;
        }
        // A new resolution keeps the drawing's shape, and about its size on screen.
        ("crosshair_res", None) => {
            *c = crosshair::with_resolution(c, RESOLUTIONS[0]);
            state.draw_full = false;
        }
        ("crosshair_res", Some(up)) => {
            let size = crosshair::step_resolution(c.pixels.size() as u8, up);
            *c = crosshair::with_resolution(c, size);
            state.draw_full = false;
        }
        // On a drawing, the thickness value is its pixel size, in screen pixels.
        ("crosshair_thickness", None) if c.style == CUSTOM => {
            c.pixel_size = DEFAULT.pixel_size;
        }
        ("crosshair_thickness", Some(up)) if c.style == CUSTOM => {
            c.pixel_size = crosshair::clamp_pixel_size(f32::from(c.pixel_size) + steps(up));
        }
        ("crosshair_style", None) => c.style = DEFAULT.style,
        ("crosshair_style", Some(up)) => c.style = crosshair::step_style(c.style, up),
        ("crosshair_length", None) => c.length = DEFAULT.length,
        ("crosshair_length", Some(up)) => c.length = LENGTH.stepped(c.length, steps(up)),
        ("crosshair_thickness", None) => c.thickness = DEFAULT.thickness,
        ("crosshair_thickness", Some(up)) => {
            c.thickness = THICKNESS.stepped(c.thickness, steps(up));
        }
        ("crosshair_gap", None) => c.gap = DEFAULT.gap,
        ("crosshair_gap", Some(up)) => c.gap = GAP.stepped(c.gap, steps(up)),
        ("crosshair_opacity", None) => c.opacity = DEFAULT.opacity,
        ("crosshair_opacity", Some(up)) => c.opacity = crosshair::step_opacity(c.opacity, up),
        ("crosshair_reset", None) => {
            // The drawing is kept: Clear is for that.
            *c = crate::crosshair::Crosshair {
                on: c.on,
                pixels: c.pixels,
                pixel_size: c.pixel_size,
                mirror: c.mirror,
                ..DEFAULT
            };
            state.colour_open = false;
            state.drawing = false;
        }
        _ => return false,
    }
    (*settings, *state) != before
}

/// [`apply`] for the Zoom page: switches, and values that step by one notch or reset.
fn apply_zoom(settings: &mut Settings, id: &str, value: f32) -> bool {
    use crate::zoom::{self, Zoom};
    let before = *settings;
    let defaults = Zoom::default();
    let z = &mut settings.zoom;
    match id {
        "zoom" => z.on = value > 0.5,
        "zoom_mode" => z.toggle = value > 0.5,
        "zoom_scroll" => z.scroll = value > 0.5,
        "zoom_sens" => z.sensitivity = value > 0.5,
        "zoom_level" => z.level = defaults.level,
        "zoom_level_less" => z.level = zoom::clamp_level(z.level - zoom::LEVEL_STEP),
        "zoom_level_more" => z.level = zoom::clamp_level(z.level + zoom::LEVEL_STEP),
        "zoom_time" => z.time = defaults.time,
        "zoom_time_less" => z.time = zoom::clamp_time(z.time - zoom::TIME_STEP),
        "zoom_time_more" => z.time = zoom::clamp_time(z.time + zoom::TIME_STEP),
        "zoom_step" => z.step = defaults.step,
        "zoom_step_less" => {
            z.step = zoom::clamp_step(f32::from(z.step) - f32::from(zoom::STEP_STEP));
        }
        "zoom_step_more" => {
            z.step = zoom::clamp_step(f32::from(z.step) + f32::from(zoom::STEP_STEP));
        }
        // Everything but the key the player chose.
        "zoom_reset" => {
            *z = Zoom {
                key: z.key,
                ..defaults
            }
        }
        _ => return false,
    }
    *settings != before
}

/// [`apply`] for the controls of a page, editing `settings` as the page shows them.
pub(super) fn apply_page(
    settings: &mut Settings,
    state: &mut MenuState,
    id: &str,
    value: f32,
) -> bool {
    let before = (*settings, *state);
    let byte = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    let mut rgb = target_rgb(settings, *state);
    match id {
        "red" | "green" | "blue" => {
            rgb[match id {
                "red" => 0,
                "green" => 1,
                _ => 2,
            }] = byte(value);
            set_target_rgb(settings, *state, rgb);
        }
        "opacity" => settings.rgba[3] = byte(value.clamp(0.0, 100.0) * 2.55),
        "width" => settings.width = ((value * 2.0).round() / 2.0).clamp(MIN_WIDTH, MAX_WIDTH),
        // Mob hitboxes have no fill.
        "fill" if state.page != Page::Mob => settings.fill = value.round().clamp(0.0, 100.0) as u8,
        "rainbow" => settings.rainbow = value > 0.5,
        // Mob hitboxes have no pulse.
        "pulse" if state.page != Page::Mob => settings.pulse = value > 0.5,
        "hide_game_highlight" => {
            // Off, or back on at the strength that matched best.
            settings.highlight_cancel = if settings.highlight_cancel > 0 {
                0
            } else {
                crate::outline::DEFAULT_HIGHLIGHT_CANCEL
            };
        }
        "highlight_less" | "highlight_more" => {
            settings.highlight_cancel = crate::outline::step_highlight_cancel(
                settings.highlight_cancel,
                id == "highlight_more",
            );
        }
        // Mob hitbox fill always follows the line colour.
        "fill_match" | "edit_fill" if state.page == Page::Mob => return false,
        "fill_match" => {
            if value > 0.5 {
                settings.fill_rgb = None;
                // A matching fill has no colour of its own to edit.
                state.editing_fill = false;
            } else {
                settings.fill_rgb = Some([settings.rgba[0], settings.rgba[1], settings.rgba[2]]);
            }
        }
        "edit_line" | "edit_fill" | "edit_aim" => {
            let fill = id == "edit_fill";
            let aim = id == "edit_aim";
            if fill && settings.fill_rgb.is_none() {
                // Hidden while the fill matches the line.
                return false;
            }
            if state.colour_open && state.editing_fill == fill && state.editing_aim == aim {
                // Clicking the open one again hides the colour tools.
                state.colour_open = false;
            } else {
                state.colour_open = true;
                state.editing_fill = fill;
                state.editing_aim = aim;
            }
        }
        "show_all_mobs" => settings.show_all_mobs = value > 0.5,
        "fullbright" => settings.fullbright = value > 0.5,
        "reset" => {
            // On the mob page this resets the mob look only (see `apply`).
            settings.reset();
            if state.page == Page::Mob {
                let defaults = Settings::default();
                settings.show_all_mobs = defaults.show_all_mobs;
                settings.aim_rgb = defaults.aim_rgb;
            }
            state.editing_fill = false;
            state.editing_aim = false;
        }
        _ if id.starts_with("type_") => {
            let field = if id == "type_hex" {
                Some(HEX_FIELD)
            } else {
                SLIDERS
                    .iter()
                    .position(|(slider, _)| id[5..] == **slider)
                    .map(|i| i as u8)
            };
            let Some(field) = field else { return false };
            state.typing = Some(Typing::new(field));
            return true;
        }
        "save_favourite" => {
            let [r, g, b] = rgb;
            let alpha = if edits_alpha(*state) {
                settings.rgba[3]
            } else {
                255
            };
            settings.save_favourite([r, g, b, alpha]);
        }
        "wheel_x" | "wheel_y" => {
            let [mut x, mut y] = wheel_position(rgb);
            if id == "wheel_x" {
                x = value;
            } else {
                y = value;
            }
            // Keep the current brightness; from black, pick at full brightness instead.
            let brightness = match rgb_to_hsv(rgb)[2] {
                v if v > 0.0 => v,
                _ => 1.0,
            };
            set_target_rgb(settings, *state, wheel_colour(x, y, brightness));
        }
        _ => {
            if let Some(slot) = id
                .strip_prefix("favourite_")
                .and_then(|slot| slot.parse::<usize>().ok())
            {
                let Some(Some(colour)) = settings.favourites.get(slot).copied() else {
                    return false;
                };
                set_target_rgb(settings, *state, [colour[0], colour[1], colour[2]]);
                if edits_alpha(*state) {
                    settings.rgba[3] = colour[3];
                }
            } else if let Some(name) = id.strip_prefix("swatch_") {
                let Some((_, swatch)) = SWATCHES
                    .iter()
                    .find(|(swatch, _)| swatch.eq_ignore_ascii_case(name))
                else {
                    return false;
                };
                set_target_rgb(settings, *state, *swatch);
            } else {
                return false;
            }
        }
    }
    (*settings, *state) != before
}

/// Applies the typed text, if any, and stops typing. Unusable text is dropped.
pub fn commit_typing(settings: &mut Settings, state: &mut MenuState) {
    let Some(typing) = state.typing.take() else {
        return;
    };
    let text = typing.text();
    if usize::from(typing.field) < SLIDERS.len() {
        if let Ok(value) = text.parse::<f32>() {
            let (min, max, _) = slider_range(usize::from(typing.field));
            let id = SLIDERS[usize::from(typing.field)].0;
            apply(settings, state, id, value.clamp(min, max));
        }
    } else if !text.is_empty() {
        let _ = apply_text(settings, *state, "hex", text);
    }
}

/// Feeds one pressed physical key (`Digit4`, `KeyF`, `Backspace`, …) to the box being typed
/// into. Returns whether the menu should be redrawn.
pub fn type_key(settings: &mut Settings, state: &mut MenuState, key: &str) -> bool {
    let Some(mut typing) = state.typing else {
        return false;
    };
    let hex = typing.field == HEX_FIELD;
    match key {
        "Enter" | "NumpadEnter" | "Tab" | "MouseLeft" | "MouseRight" => {
            commit_typing(settings, state);
            return true;
        }
        "Escape" => {
            state.typing = None;
            return true;
        }
        "Backspace" | "Delete" => typing.len = typing.len.saturating_sub(1),
        _ => {
            let digit = key
                .strip_prefix("Digit")
                .or_else(|| key.strip_prefix("Numpad"))
                .filter(|rest| rest.len() == 1 && rest.as_bytes()[0].is_ascii_digit());
            let letter = key.strip_prefix("Key").filter(|rest| {
                hex && rest.len() == 1 && (b'A'..=b'F').contains(&rest.as_bytes()[0])
            });
            let point =
                matches!(key, "Period" | "NumpadDecimal") && !hex && !typing.text().contains('.');
            match (digit.or(letter), point) {
                (Some(c), _) => typing.push(c.as_bytes()[0]),
                (None, true) => typing.push(b'.'),
                _ => return false,
            }
        }
    }
    state.typing = Some(typing);
    true
}

/// Applies a committed text field. `None` means the text was not a colour, so the caller should
/// put the field back to the current code.
pub fn apply_text(
    settings: &mut Settings,
    state: MenuState,
    id: &str,
    value: &str,
) -> Option<bool> {
    if id != "hex" {
        return Some(false);
    }
    if state.page == Page::Mob {
        let mut mobs = settings.for_mobs();
        let changed = apply_hex(&mut mobs, state, value);
        settings.store_mob(&mobs);
        return changed;
    }
    apply_hex(settings, state, value)
}

pub(super) fn apply_hex(settings: &mut Settings, state: MenuState, value: &str) -> Option<bool> {
    let rgba = parse_color(value)?;
    let before = *settings;
    set_target_rgb(settings, state, [rgba[0], rgba[1], rgba[2]]);
    if edits_alpha(state) {
        settings.rgba[3] = rgba[3];
    }
    Some(*settings != before)
}
