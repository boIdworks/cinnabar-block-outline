//! How the menu's control events, typed keys and text change the settings and menu state.

use super::*;

/// Applies one control event to the settings or the menu state. Returns whether the menu should
/// be redrawn; the caller saves the settings when they changed.
pub fn apply(settings: &mut Settings, state: &mut MenuState, id: &str, value: f32) -> bool {
    if !value.is_finite() {
        return false;
    }
    match id {
        "open_block" | "open_mob" => {
            if state.page != Page::Home {
                return false;
            }
            state.page = if id == "open_block" {
                Page::Block
            } else {
                Page::Mob
            };
            state.colour_open = false;
            state.editing_fill = false;
            state.editing_aim = false;
            return true;
        }
        "back" => {
            if state.page == Page::Home {
                return false;
            }
            commit_typing(settings, state);
            // One step back: from the colour tools to the page, from the page to the start.
            if state.colour_open {
                state.colour_open = false;
            } else {
                state.page = Page::Home;
            }
            return true;
        }
        // Clicking another box applies what was typed in the previous one. Done here, on the
        // real settings: committing from inside a page's edit would be overwritten by it.
        _ if id.starts_with("type_") => commit_typing(settings, state),
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
