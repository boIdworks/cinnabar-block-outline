//! The CPS counter: clicks per second of the left and right mouse buttons, drawn as a line of
//! text in a corner of the HUD. Host-independent so `cargo test` can check the counting, the
//! placement and the patches.

use crate::counter::{Anchor, Edit, Pos};
use crate::panel::{digits, fixed};

/// Clicks are counted over this many seconds, so the number is clicks in the last second.
pub const WINDOW: f32 = 1.0;
/// The most clicks remembered per button, far above what a hand can do in a second.
const CAP: usize = 64;

/// CPS settings, saved with the other settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cps {
    pub on: bool,
    /// Also show the right button: `CPS: 7 | 3` instead of `CPS: 7`.
    pub right: bool,
    /// Write counts below 10 with a leading zero (`07`) rather than plain (`7`).
    pub zeros: bool,
    /// Where the text starts: its top-left corner is half a line height (the position's unit
    /// is the icon's, as for the arrow counter) from `pos`, measured from the nearest screen
    /// edge or the middle, so it keeps its place when the window or GUI scale changes.
    pub pos: Pos,
    /// Size multiplier: 1 is the normal HUD text size.
    pub scale: f32,
}

impl Default for Cps {
    fn default() -> Self {
        Self {
            on: false,
            right: true,
            zeros: true,
            pos: DEFAULT_POS,
            scale: 1.0,
        }
    }
}

/// The top left corner of the screen, a small margin in.
pub const DEFAULT_POS: Pos = Pos {
    anchor: [Anchor::Start, Anchor::Start],
    off: [9.0, 9.0],
};

pub const MIN_SCALE: f32 = 0.5;
pub const MAX_SCALE: f32 = 4.0;
/// One click of - / +.
pub const SCALE_STEP: f32 = 0.1;

pub fn clamp_scale(scale: f32) -> f32 {
    if scale.is_finite() {
        scale.clamp(MIN_SCALE, MAX_SCALE)
    } else {
        1.0
    }
}

impl Cps {
    /// `scale` moved by `steps` steps, rounded to whole steps and kept in bounds.
    pub fn stepped(scale: f32, steps: f32) -> f32 {
        clamp_scale(((scale + steps * SCALE_STEP) / SCALE_STEP).round() * SCALE_STEP)
    }
}

/// When one button was clicked, newest last.
#[derive(Clone, Copy, Debug)]
pub struct Clicks {
    times: [f32; CAP],
    len: usize,
}

impl Clicks {
    pub const fn new() -> Self {
        Self {
            times: [0.0; CAP],
            len: 0,
        }
    }

    /// Forgets the clicks older than [`WINDOW`] before `now`.
    fn expire(&mut self, now: f32) {
        let old = self.times[..self.len]
            .iter()
            .take_while(|&&time| {
                // Older than the window, or not a number: forgotten.
                let age = now - time;
                age > WINDOW || age.is_nan()
            })
            .count();
        if old > 0 {
            self.times.copy_within(old..self.len, 0);
            self.len -= old;
        }
    }

    /// Notes a click at `now` seconds.
    pub fn click(&mut self, now: f32) {
        self.expire(now);
        if self.len == CAP {
            self.times.copy_within(1..CAP, 0);
            self.len -= 1;
        }
        self.times[self.len] = now;
        self.len += 1;
    }

    /// Clicks in the last [`WINDOW`] seconds up to `now`.
    pub fn per_second(&mut self, now: f32) -> u32 {
        self.expire(now);
        self.len as u32
    }
}

impl Default for Clicks {
    fn default() -> Self {
        Self::new()
    }
}

/// Height of a line of HUD text at text scale 1.
const LINE: f32 = 10.0;

/// What the HUD shows this frame. Patches are sent only when it changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub visible: bool,
    pub left: u32,
    /// The right button's clicks, when it is shown.
    pub right: Option<u32>,
    /// Whether counts below 10 get a leading zero.
    pub zeros: bool,
    pub scale: f32,
    /// Where the two parts of the text sit, at text scale 1.
    pub layout: Layout,
    /// Top-left corner of the text, in HUD GUI units.
    pub offset: [f32; 2],
    /// While the HUD is being edited: how the readout's frame and handles look.
    pub edit: Option<Edit>,
}

impl View {
    pub const HIDDEN: View = View {
        visible: false,
        left: 0,
        right: None,
        zeros: true,
        scale: 1.0,
        layout: Layout::EMPTY,
        offset: [0.0; 2],
        edit: None,
    };
}

/// Where the parts of the readout sit, at text scale 1, as three pieces of text so that nothing
/// shifts when a number gets another digit: `CPS:`, the left number right-aligned in a slot of
/// two digits (it grows leftwards), and `| 3` after it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// Width of `CPS:`.
    pub caption_w: f32,
    /// Where the left number's slot starts, from the readout's left edge, and its width.
    pub num_x: f32,
    pub num_w: f32,
    /// Where `| 3` starts and the width kept for it.
    pub right_x: f32,
    pub right_w: f32,
    /// Width of the whole readout.
    pub total: f32,
}

impl Layout {
    pub const EMPTY: Layout = Layout {
        caption_w: 0.0,
        num_x: 0.0,
        num_w: 0.0,
        right_x: 0.0,
        right_w: 0.0,
        total: 0.0,
    };
}

/// The pieces of the text, measured once at text scale 1: `CPS:`, `FPS:` (for the FPS counter,
/// see [`crate::fps`]), one digit, a space and the bar. Every digit is as wide as the others in
/// Cinnabar's font.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Widths {
    pub caption: f32,
    pub fps_caption: f32,
    pub digit: f32,
    pub space: f32,
    pub bar: f32,
}

impl Widths {
    /// A guess for when the text cannot be measured.
    pub const GUESS: Widths = Widths {
        caption: 24.0,
        fps_caption: 24.0,
        digit: 6.0,
        space: 4.0,
        bar: 2.0,
    };

    /// Room for `left` and `right`: two digits each, or more once a number needs them. A
    /// number that is still short leaves its blank on the left, after `CPS:`, so the bar and
    /// the right number stay where they are when the count reaches 10.
    pub fn layout(&self, left: u32, right: Option<u32>) -> Layout {
        let digits = |n: u32| crate::counter::digits(n).max(2) as f32;
        let num_x = self.caption + self.space;
        let num_w = self.digit * digits(left);
        match right {
            None => Layout {
                caption_w: self.caption,
                num_x,
                num_w,
                right_x: 0.0,
                right_w: 0.0,
                total: num_x + num_w,
            },
            Some(right) => {
                let right_x = num_x + num_w + self.space;
                let right_w = self.bar + self.space + self.digit * digits(right);
                Layout {
                    caption_w: self.caption,
                    num_x,
                    num_w,
                    right_x,
                    right_w,
                    total: right_x + right_w,
                }
            }
        }
    }

    /// Width of the FPS counter's text, `FPS: 144`: room for three digits, or more once needed,
    /// so it keeps its size from 100 down to 0.
    pub fn fps_width(&self, fps: u32) -> f32 {
        self.fps_caption + self.space + self.digit * crate::counter::digits(fps).max(3) as f32
    }
}

/// How long to wait between measurements of the text while they are not settled, in seconds.
pub const RECHECK_SECONDS: f32 = 1.0;

/// Keeps the measured [`Widths`] right. The host may not be able to measure yet when the mod
/// starts, or may answer before its fonts are ready, and a box cut to those widths clips the
/// text to `...`. So it measures again, at most once per [`RECHECK_SECONDS`], until two
/// successful measurements in a row agree; after that it costs nothing (no host calls) until
/// [`WidthProbe::reset`] (a new GUI scale). Until a measurement succeeds the guess is used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidthProbe {
    widths: Widths,
    last: Option<Widths>,
    settled: bool,
    next_at: f32,
}

impl WidthProbe {
    pub const fn new() -> Self {
        Self {
            widths: Widths::GUESS,
            last: None,
            settled: false,
            next_at: 0.0,
        }
    }

    /// The widths to lay out with now.
    pub fn widths(&self) -> Widths {
        self.widths
    }

    /// Whether to measure now.
    pub fn due(&self, now: f32) -> bool {
        !self.settled && now >= self.next_at
    }

    /// Takes the result of a measurement (`None` if the host could not measure). Returns whether
    /// the widths in use changed, so what was laid out with the old ones must be redone.
    pub fn observe(&mut self, now: f32, measured: Option<Widths>) -> bool {
        self.next_at = now + RECHECK_SECONDS;
        let Some(measured) = measured else {
            // Keep what is in use (the guess, or the last good measurement) and try again.
            self.last = None;
            return false;
        };
        let changed = measured != self.widths;
        self.widths = measured;
        self.settled = self.last == Some(measured);
        self.last = Some(measured);
        changed
    }

    /// Measure again from the next call, keeping the widths in use meanwhile (the GUI scale
    /// changed).
    pub fn reset(&mut self) {
        self.last = None;
        self.settled = false;
        self.next_at = 0.0;
    }
}

impl Default for WidthProbe {
    fn default() -> Self {
        Self::new()
    }
}

/// The size of the readout at text scale 1: the text's width, a little spare so the shadow is
/// not clipped, and one line's height.
pub fn base(width: f32) -> [f32; 2] {
    [width + 2.0, LINE]
}

/// The size of the readout at `scale`.
pub fn size(scale: f32, width: f32) -> [f32; 2] {
    base(width).map(|side| side * scale)
}

/// Writes a click count, with a leading zero below 10 (`07`) when `zeros` is on.
fn padded(out: &mut String, n: u32, zeros: bool) {
    if zeros && n < 10 {
        out.push('0');
    }
    digits(out, u64::from(n));
}

/// Writes the left number's text.
pub fn left_text(out: &mut String, left: u32, zeros: bool) {
    padded(out, left, zeros);
}

/// Writes the right part of the text: `| 3`.
pub fn right_text(out: &mut String, right: u32, zeros: bool) {
    out.push_str("| ");
    padded(out, right, zeros);
}

/// The readout's controls of the HUD document, as a string literal: its three labels (`CPS:`
/// fixed, the left number aligned to the right of its slot, and `| 3`), then the edit overlay on
/// a panel that follows the readout's rectangle.
macro_rules! document {
    () => {
        concat!(
            r##"{"cps":{"type":"label","text":"CPS:","size":[24,10],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"left","font_scale_factor":1,"layer":2,"bindings":["##,
            r##"{"binding_name":"#cps_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#cps_font","binding_name_override":"#font_scale_factor"},"##,
            r##"{"binding_name":"#cps_offset","binding_name_override":"#offset"},"##,
            r##"{"binding_name":"#cps_caption_w","binding_name_override":"#size_binding_x_absolute"},"##,
            r##"{"binding_name":"#cps_h","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"cps_num":{"type":"label","text":"#cps_text","size":[12,10],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"right","font_scale_factor":1,"layer":2,"bindings":["##,
            r##"{"binding_name":"#cps_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#cps_text"},"##,
            r##"{"binding_name":"#cps_font","binding_name_override":"#font_scale_factor"},"##,
            r##"{"binding_name":"#cps_num_offset","binding_name_override":"#offset"},"##,
            r##"{"binding_name":"#cps_num_w","binding_name_override":"#size_binding_x_absolute"},"##,
            r##"{"binding_name":"#cps_h","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"cps_right":{"type":"label","text":"#cps_right_text","size":[30,10],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"left","font_scale_factor":1,"layer":2,"bindings":["##,
            r##"{"binding_name":"#cps_right_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#cps_right_text"},"##,
            r##"{"binding_name":"#cps_font","binding_name_override":"#font_scale_factor"},"##,
            r##"{"binding_name":"#cps_right_offset","binding_name_override":"#offset"},"##,
            r##"{"binding_name":"#cps_right_w","binding_name_override":"#size_binding_x_absolute"},"##,
            r##"{"binding_name":"#cps_h","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"cpsedit":{"type":"panel","size":[34,16],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","layer":3,"bindings":["##,
            r##"{"binding_name":"#cpsedit_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#cpsedit_offset","binding_name_override":"#offset"},"##,
            r##"{"binding_name":"#cpsedit_w","binding_name_override":"#size_binding_x_absolute"},"##,
            r##"{"binding_name":"#cpsedit_h","binding_name_override":"#size_binding_y_absolute"}],"controls":["##,
            $crate::counter::overlay_doc!("cpsedit"),
            r##"]}}"##
        )
    };
}
pub(crate) use document;

/// The bindings of a hidden readout, to publish with the rest of the layer.
pub fn initial_bindings() -> String {
    bindings(&View::HIDDEN)
}

/// A `hud.update-surface` patch showing `view`.
pub fn patch(view: &View) -> String {
    let mut out = String::with_capacity(220);
    out.push_str(r#"{"binding_changes":{"#);
    out.push_str(&bindings(view));
    out.push_str("}}");
    out
}

/// The readout's bindings as `"name":value` pairs, without braces.
fn bindings(view: &View) -> String {
    let mut out = String::with_capacity(420);
    let s = view.scale;
    let [x, y] = view.offset;
    let layout = view.layout;
    let pair = |out: &mut String, x: f32, y: f32| {
        out.push('[');
        fixed(out, x);
        out.push(',');
        fixed(out, y);
        out.push(']');
    };
    out.push_str(r##""#cps_visible":"##);
    out.push_str(if view.visible { "true" } else { "false" });
    out.push_str(r##","#cps_right_visible":"##);
    out.push_str(if view.visible && view.right.is_some() {
        "true"
    } else {
        "false"
    });
    out.push_str(r##","#cps_text":""##);
    left_text(&mut out, view.left, view.zeros);
    out.push_str(r##"","#cps_right_text":""##);
    right_text(&mut out, view.right.unwrap_or(0), view.zeros);
    out.push_str(r##"","#cps_font":"##);
    fixed(&mut out, s);
    out.push_str(r##","#cps_offset":"##);
    pair(&mut out, x, y);
    out.push_str(r##","#cps_num_offset":"##);
    pair(&mut out, x + layout.num_x * s, y);
    out.push_str(r##","#cps_right_offset":"##);
    pair(&mut out, x + layout.right_x * s, y);
    // A little spare on each so the text shadow is not clipped.
    out.push_str(r##","#cps_caption_w":"##);
    fixed(&mut out, (layout.caption_w + 2.0) * s);
    out.push_str(r##","#cps_num_w":"##);
    fixed(&mut out, layout.num_w * s);
    out.push_str(r##","#cps_right_w":"##);
    fixed(&mut out, (layout.right_w + 2.0) * s);
    out.push_str(r##","#cps_h":"##);
    fixed(&mut out, LINE * s);
    // The edit overlay follows the readout's rectangle.
    let size = [(layout.total + 2.0) * s, LINE * s];
    out.push_str(r##","#cpsedit_visible":"##);
    out.push_str(if view.edit.is_some() { "true" } else { "false" });
    out.push_str(r##","#cpsedit_offset":"##);
    pair(&mut out, x, y);
    out.push_str(r##","#cpsedit_w":"##);
    fixed(&mut out, size[0]);
    out.push_str(r##","#cpsedit_h":"##);
    fixed(&mut out, size[1]);
    out.push(',');
    out.push_str(&crate::counter::overlay_bindings(
        "cpsedit", size, s, view.edit,
    ));
    out
}

#[cfg(test)]
mod tests;
