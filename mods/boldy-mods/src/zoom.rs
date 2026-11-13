//! Zoom: hold a key to narrow the view smoothly, with step zoom and relative sensitivity.
//! Host-independent so `cargo test` can check the easing and the scroll maths.
//!
//! A mod cannot read the scroll wheel during gameplay. While zooming, a wheel turn still moves
//! the selected hotbar slot, which a mod can read, so [`scroll_steps`] turns that move into zoom
//! steps (the game moves the slot as well; a mod cannot undo that).

use crate::outline::KeyName;

/// Zoom settings, saved with the other settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zoom {
    pub on: bool,
    /// Press the key to zoom in and again to zoom out, instead of holding it.
    pub toggle: bool,
    /// While zooming, scrolling changes the zoom distance. The game moves the selected hotbar
    /// slot on every wheel turn and a mod cannot stop that; this only notices the slot move.
    pub scroll: bool,
    /// Look more slowly while zoomed, in proportion to the zoom.
    pub sensitivity: bool,
    /// Zoom factor when the key is pressed: 2 shows half the view.
    pub level: f32,
    /// Seconds to zoom in or out; 0 is instant.
    pub time: f32,
    /// Percent each scroll notch or step key changes the zoom by.
    pub step: u8,
    /// The zoom key; `None` when the player unbound it (Escape while choosing it).
    pub key: Option<KeyName>,
}

pub const MIN_LEVEL: f32 = 1.5;
/// The camera API cannot narrow the view past a tenth.
pub const MAX_LEVEL: f32 = 10.0;
pub const LEVEL_STEP: f32 = 0.5;
pub const MAX_TIME: f32 = 1.0;
pub const TIME_STEP: f32 = 0.05;
pub const MIN_STEP: u8 = 5;
pub const MAX_STEP: u8 = 50;
pub const STEP_STEP: u8 = 5;

impl Default for Zoom {
    fn default() -> Self {
        Self {
            on: false,
            toggle: false,
            // Off: it moves the hotbar slot too, which a first-time user should opt into.
            scroll: false,
            sensitivity: true,
            level: 4.0,
            time: 0.25,
            step: 20,
            key: KeyName::new("KeyZ"),
        }
    }
}

fn finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

pub fn clamp_level(level: f32) -> f32 {
    ((finite(level, 4.0) / LEVEL_STEP).round() * LEVEL_STEP).clamp(MIN_LEVEL, MAX_LEVEL)
}

pub fn clamp_time(time: f32) -> f32 {
    ((finite(time, 0.25) / TIME_STEP).round() * TIME_STEP).clamp(0.0, MAX_TIME)
}

pub fn clamp_step(step: f32) -> u8 {
    ((finite(step, 20.0) / f32::from(STEP_STEP)).round() * f32::from(STEP_STEP))
        .clamp(f32::from(MIN_STEP), f32::from(MAX_STEP)) as u8
}

/// The scroll notches turned, positive zooming in, when the selected hotbar slot went from
/// `last` to `now`: wheel up selects the previous slot, wheel down the next, wrapping round the
/// nine slots. A jump of more than four slots is read as a wrap the other way.
pub fn scroll_steps(last: u8, now: u8) -> i32 {
    if last > 8 || now > 8 {
        return 0;
    }
    let forward = (i32::from(now) + 9 - i32::from(last)) % 9;
    if forward <= 4 { -forward } else { 9 - forward }
}

/// What pressing keys means while an optional key button waits for its new key: `None` while no
/// key was pressed yet, else the key to end up with. Backspace and Delete unbind it
/// (`Some(None)`); Escape cancels and the menu key is refused, so `current` stays; any other key
/// becomes the key.
pub fn captured_key(
    pressed: &[String],
    current: Option<KeyName>,
    menu_key: KeyName,
) -> Option<Option<KeyName>> {
    let key = pressed.iter().find(|key| !key.starts_with("Mouse"))?;
    Some(match key.as_str() {
        "Backspace" | "Delete" => None,
        "Escape" => current,
        key if key == menu_key.as_str() => current,
        key => KeyName::new(key).or(current),
    })
}

/// What to ask the camera for this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Output {
    /// Zoomed (or still easing back): send the scales. When false, send nothing and the host
    /// restores the normal view.
    pub applying: bool,
    pub fov_scale: f32,
    pub look_scale: f32,
}

/// The easing state: how far the view is zoomed and the level this zoom has been scrolled to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Engine {
    /// The current zoom factor: 1 is no zoom.
    pub divisor: f32,
    /// The level being zoomed to; set from the settings when a zoom starts, then scrolled.
    pub level: f32,
    was_active: bool,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            divisor: 1.0,
            level: 4.0,
            was_active: false,
        }
    }
}

impl Engine {
    /// Moves the zoom `dt` seconds on: toward the level while `active`, back to 1 otherwise.
    /// `steps` are scroll notches (positive zooms in) applied to the level of this zoom.
    pub fn tick(&mut self, zoom: &Zoom, active: bool, steps: i32, dt: f32) -> Output {
        if active && !self.was_active {
            self.level = zoom.level;
        }
        self.was_active = active;
        if active && steps != 0 {
            let factor = 1.0 + f32::from(zoom.step) / 100.0;
            self.level = (self.level * factor.powi(steps.clamp(-8, 8))).clamp(1.1, MAX_LEVEL);
        }
        let target = if active { self.level.ln() } else { 0.0 };
        let mut now = self.divisor.ln();
        let dt = finite(dt, 0.0).clamp(0.0, 0.1);
        if zoom.time <= 0.01 {
            now = target;
        } else {
            // About 95% of the way after `time` seconds, evenly in how big things look.
            now += (target - now) * (1.0 - (-dt * 3.0 / zoom.time).exp());
        }
        if (target - now).abs() < 0.003 {
            now = target;
        }
        self.divisor = now.exp();
        let applying = self.divisor > 1.003;
        let fov_scale = (1.0 / self.divisor).clamp(0.1, 1.0);
        Output {
            applying,
            fov_scale,
            look_scale: if zoom.sensitivity {
                fov_scale.max(0.05)
            } else {
                1.0
            },
        }
    }
}

#[cfg(test)]
mod tests;
