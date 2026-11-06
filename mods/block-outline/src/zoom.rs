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
            on: true,
            toggle: false,
            scroll: true,
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
mod tests {
    use super::*;

    #[test]
    fn backspace_unbinds_and_escape_cancels_the_key_being_chosen() {
        let keys = |names: &[&str]| names.iter().map(|k| (*k).to_owned()).collect::<Vec<_>>();
        let z = KeyName::new("KeyZ");
        let menu = KeyName::new("KeyL").unwrap();
        // Nothing pressed yet, or only a mouse click: keep waiting.
        assert_eq!(captured_key(&keys(&[]), z, menu), None);
        assert_eq!(captured_key(&keys(&["MouseLeft"]), z, menu), None);
        // Backspace and Delete unbind, even when a click came first.
        assert_eq!(captured_key(&keys(&["Backspace"]), z, menu), Some(None));
        assert_eq!(
            captured_key(&keys(&["MouseLeft", "Delete"]), z, menu),
            Some(None)
        );
        // Escape cancels: the key stays as it was, bound or not.
        assert_eq!(captured_key(&keys(&["Escape"]), z, menu), Some(z));
        assert_eq!(
            captured_key(&keys(&["MouseLeft", "Escape"]), z, menu),
            Some(z)
        );
        assert_eq!(captured_key(&keys(&["Escape"]), None, menu), Some(None));
        // Unbinding an unbound key stays unbound.
        assert_eq!(captured_key(&keys(&["Delete"]), None, menu), Some(None));
        // Any other key becomes the key; the menu key is refused and nothing changes.
        assert_eq!(
            captured_key(&keys(&["KeyC"]), z, menu),
            Some(KeyName::new("KeyC"))
        );
        assert_eq!(captured_key(&keys(&["KeyL"]), z, menu), Some(z));
        assert_eq!(captured_key(&keys(&["KeyL"]), None, menu), Some(None));
    }

    #[test]
    fn scroll_direction_follows_the_hotbar() {
        // Wheel up selects the previous slot: zoom in. Wheel down the next: zoom out.
        assert_eq!(scroll_steps(4, 3), 1);
        assert_eq!(scroll_steps(4, 5), -1);
        assert_eq!(scroll_steps(4, 4), 0);
        // Wrapping round the ends is still one notch.
        assert_eq!(scroll_steps(0, 8), 1);
        assert_eq!(scroll_steps(8, 0), -1);
        // Several notches between two frames.
        assert_eq!(scroll_steps(6, 3), 3);
        assert_eq!(scroll_steps(2, 5), -3);
        assert_eq!(scroll_steps(0, 9), 0, "not a hotbar slot");
    }

    #[test]
    fn eases_in_and_out_and_stops_applying() {
        let zoom = Zoom::default();
        let mut engine = Engine::default();
        assert!(!engine.tick(&zoom, false, 0, 0.016).applying);
        let mut last = 1.0;
        for _ in 0..8 {
            let out = engine.tick(&zoom, true, 0, 0.016);
            assert!(out.applying && engine.divisor > last, "zooms in steadily");
            last = engine.divisor;
        }
        // After well over the zoom time it has arrived at the level and looks slower to match.
        for _ in 0..60 {
            engine.tick(&zoom, true, 0, 0.016);
        }
        let out = engine.tick(&zoom, true, 0, 0.016);
        assert!((engine.divisor - 4.0).abs() < 0.01, "{}", engine.divisor);
        assert!((out.fov_scale - 0.25).abs() < 0.01 && (out.look_scale - 0.25).abs() < 0.01);
        // Letting go eases back to exactly 1 and then stops sending anything.
        for _ in 0..120 {
            engine.tick(&zoom, false, 0, 0.016);
        }
        let out = engine.tick(&zoom, false, 0, 0.016);
        assert_eq!(engine.divisor, 1.0);
        assert!(!out.applying);
        assert_eq!((out.fov_scale, out.look_scale), (1.0, 1.0));
    }

    #[test]
    fn steps_scale_the_level_and_reset_each_zoom() {
        let zoom = Zoom {
            time: 0.0,
            ..Zoom::default()
        };
        let mut engine = Engine::default();
        engine.tick(&zoom, true, 0, 0.016);
        assert!(
            (engine.divisor - 4.0).abs() < 1e-4,
            "instant with zero time"
        );
        // Two notches in at 20% each, one out.
        engine.tick(&zoom, true, 2, 0.016);
        engine.tick(&zoom, true, -1, 0.016);
        assert!(
            (engine.divisor - 4.0 * 1.2).abs() < 1e-4,
            "{}",
            engine.divisor
        );
        // The limits hold: nowhere near past 10x, nor below a sliver of zoom.
        engine.tick(&zoom, true, 8, 0.016);
        engine.tick(&zoom, true, 8, 0.016);
        assert!((engine.divisor - MAX_LEVEL).abs() < 1e-3);
        assert!((engine.tick(&zoom, true, 0, 0.016).fov_scale - 0.1).abs() < 1e-4);
        engine.tick(&zoom, true, -8, 0.016);
        engine.tick(&zoom, true, -8, 0.016);
        engine.tick(&zoom, true, -8, 0.016);
        assert!((engine.divisor - 1.1).abs() < 1e-4);
        // A new zoom starts again from the setting.
        engine.tick(&zoom, false, 0, 0.016);
        engine.tick(&zoom, true, 0, 0.016);
        assert!((engine.divisor - 4.0).abs() < 1e-4);
        // Scrolling does nothing while not zooming.
        let mut idle = Engine::default();
        assert!(!idle.tick(&zoom, false, 5, 0.016).applying);
    }

    #[test]
    fn sensitivity_option_and_clamps() {
        let zoom = Zoom {
            time: 0.0,
            sensitivity: false,
            level: 8.0,
            ..Zoom::default()
        };
        let mut engine = Engine::default();
        let out = engine.tick(&zoom, true, 0, 0.016);
        assert!((out.fov_scale - 0.125).abs() < 1e-5 && out.look_scale == 1.0);
        let max = Zoom {
            time: 0.0,
            level: MAX_LEVEL,
            ..Zoom::default()
        };
        assert!((Engine::default().tick(&max, true, 0, 0.016).look_scale - 0.1).abs() < 1e-4);
        assert_eq!(clamp_level(99.0), MAX_LEVEL);
        assert_eq!(clamp_level(0.0), MIN_LEVEL);
        assert_eq!(clamp_level(f32::NAN), 4.0);
        assert_eq!(clamp_level(3.3), 3.5);
        assert_eq!(clamp_time(-1.0), 0.0);
        assert_eq!(clamp_time(7.0), MAX_TIME);
        assert_eq!(clamp_step(1.0), MIN_STEP);
        assert_eq!(clamp_step(99.0), MAX_STEP);
        assert_eq!(clamp_step(22.0), 20);
    }
}
