//! The FPS counter: frames per second, drawn as a line of text on the HUD with its own place and
//! size, apart from the CPS readout. Host-independent so `cargo test` can check the counting, the
//! placement and the patches.

use crate::counter::{Anchor, Edit, Pos};
use crate::panel::{digits, fixed};

/// FPS settings, saved with the other settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fps {
    pub on: bool,
    /// Where the text sits, as for the CPS readout (see [`crate::cps::Cps::pos`]).
    pub pos: Pos,
    /// Size multiplier: 1 is the normal HUD text size.
    pub scale: f32,
    /// A second line under the frames per second with where the player stands, `XYZ: 12 64 -30`.
    pub coords: bool,
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            on: false,
            pos: DEFAULT_POS,
            scale: 1.0,
            coords: false,
        }
    }
}

/// The block the player stands in: `[x, y, z]`, the feet's position rounded down.
pub type Coords = [i32; 3];

/// The block of a feet position, or `None` when the host gave something that is not a position.
pub fn block_of(position: (f32, f32, f32)) -> Option<Coords> {
    let (x, y, z) = position;
    [x, y, z]
        .iter()
        .all(|n| n.is_finite())
        .then(|| [x, y, z].map(|n| n.floor() as i32))
}

/// How often the position is read while the coordinates show, in seconds. Ten times a second
/// follows a run closely, and keeps the one host call it takes rare.
pub const COORDS_EVERY: f32 = 0.1;

/// The top left corner of the screen, below the CPS readout with room between them, so that
/// while the HUD is edited each frame can be grabbed without catching the other.
pub const DEFAULT_POS: Pos = Pos {
    anchor: [Anchor::Start, Anchor::Start],
    off: [9.0, 35.0],
};

/// How often the number changes, in seconds: often enough to follow, slow enough to read (and to
/// keep HUD patches rare).
pub const EVERY: f32 = 0.5;

/// Counts frames and works out the frames per second every [`EVERY`] seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Meter {
    frames: u32,
    time: f32,
    fps: u32,
}

impl Meter {
    pub const fn new() -> Self {
        Self {
            frames: 0,
            time: 0.0,
            fps: 0,
        }
    }

    /// Notes one frame that took `delta` seconds. A frame of unknown length counts for nothing.
    pub fn tick(&mut self, delta: f32) {
        if !(delta.is_finite() && delta > 0.0) {
            return;
        }
        self.frames += 1;
        self.time += delta;
        if self.time >= EVERY {
            self.fps = (self.frames as f32 / self.time).round() as u32;
            self.frames = 0;
            self.time = 0.0;
        }
    }

    /// The frames per second over the last stretch, 0 until one has been measured.
    pub fn fps(&self) -> u32 {
        self.fps
    }
}

impl Default for Meter {
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
    pub fps: u32,
    pub scale: f32,
    /// Width of the FPS text at text scale 1; 0 when only the coordinates show.
    pub width: f32,
    /// The coordinates line under it, with its width at text scale 1; `None` when it is off.
    pub coords: Option<(Coords, f32)>,
    /// Top-left corner of the text, in HUD GUI units.
    pub offset: [f32; 2],
    /// While the HUD is being edited: how the counter's frame and handles look.
    pub edit: Option<Edit>,
}

impl View {
    pub const HIDDEN: View = View {
        visible: false,
        fps: 0,
        scale: 1.0,
        width: 0.0,
        coords: None,
        offset: [0.0; 2],
        edit: None,
    };
}

/// The size of the counter at text scale 1: the wider text's width, a little spare so the shadow
/// is not clipped, and a line's height for each of the two lines that show. A width of 0 is a
/// line that is not shown (the FPS text with only the coordinates on, or the coordinates off).
pub fn base(width: f32, coords_width: f32) -> [f32; 2] {
    let lines = usize::from(width > 0.0) + usize::from(coords_width > 0.0);
    [width.max(coords_width) + 2.0, LINE * lines.max(1) as f32]
}

/// The size of the counter at `scale`.
pub fn size(scale: f32, width: f32, coords_width: f32) -> [f32; 2] {
    base(width, coords_width).map(|side| side * scale)
}

/// Writes the coordinates line: `XYZ: 12 64 -30`.
pub fn coords_text(out: &mut String, coords: Coords) {
    out.push_str("XYZ:");
    for n in coords {
        out.push(' ');
        if n < 0 {
            out.push('-');
        }
        digits(out, u64::from(n.unsigned_abs()));
    }
}

/// Width of the coordinates line at text scale 1, worked out from the measured widths: the
/// caption (a little wider than `FPS:`), a space between numbers and a digit for each character.
pub fn coords_width(widths: &crate::cps::Widths, coords: Coords) -> f32 {
    let characters: usize = coords
        .iter()
        .map(|n| usize::from(*n < 0) + crate::counter::digits(n.unsigned_abs()))
        .sum();
    widths.fps_caption * 1.2 + 3.0 * widths.space + widths.digit * characters as f32
}

/// Writes the counter's text: `FPS: 144`.
pub fn text(out: &mut String, fps: u32) {
    out.push_str("FPS: ");
    digits(out, u64::from(fps));
}

/// The counter's controls of the HUD document, as a string literal: its label, then the edit
/// overlay on a panel that follows its rectangle.
macro_rules! document {
    () => {
        concat!(
            r##"{"fps":{"type":"label","text":"#fps_text","size":[48,10],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"left","font_scale_factor":1,"layer":2,"bindings":["##,
            r##"{"binding_name":"#fps_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#fps_text"},"##,
            r##"{"binding_name":"#fps_font","binding_name_override":"#font_scale_factor"},"##,
            r##"{"binding_name":"#fps_offset","binding_name_override":"#offset"},"##,
            r##"{"binding_name":"#fps_w","binding_name_override":"#size_binding_x_absolute"},"##,
            r##"{"binding_name":"#fps_h","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"fpsc":{"type":"label","text":"#fpsc_text","size":[48,10],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"left","font_scale_factor":1,"layer":2,"bindings":["##,
            r##"{"binding_name":"#fpsc_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#fpsc_text"},"##,
            r##"{"binding_name":"#fps_font","binding_name_override":"#font_scale_factor"},"##,
            r##"{"binding_name":"#fpsc_offset","binding_name_override":"#offset"},"##,
            r##"{"binding_name":"#fpsc_w","binding_name_override":"#size_binding_x_absolute"},"##,
            r##"{"binding_name":"#fps_h","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"fpsedit":{"type":"panel","size":[48,10],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","layer":3,"bindings":["##,
            r##"{"binding_name":"#fpsedit_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#fpsedit_offset","binding_name_override":"#offset"},"##,
            r##"{"binding_name":"#fpsedit_w","binding_name_override":"#size_binding_x_absolute"},"##,
            r##"{"binding_name":"#fpsedit_h","binding_name_override":"#size_binding_y_absolute"}],"controls":["##,
            $crate::counter::overlay_doc!("fpsedit"),
            r##"]}}"##
        )
    };
}
pub(crate) use document;

/// The bindings of a hidden counter, to publish with the rest of the layer.
pub fn initial_bindings() -> String {
    bindings(&View::HIDDEN)
}

/// A `hud.update-surface` patch showing `view`.
pub fn patch(view: &View) -> String {
    let mut out = String::with_capacity(160);
    out.push_str(r#"{"binding_changes":{"#);
    out.push_str(&bindings(view));
    out.push_str("}}");
    out
}

/// The counter's bindings as `"name":value` pairs, without braces.
fn bindings(view: &View) -> String {
    let mut out = String::with_capacity(320);
    let s = view.scale;
    let pair = |out: &mut String, [x, y]: [f32; 2]| {
        out.push('[');
        fixed(out, x);
        out.push(',');
        fixed(out, y);
        out.push(']');
    };
    let coords_width = view.coords.map_or(0.0, |(_, width)| width);
    let size = size(s, view.width, coords_width);
    // The FPS text has no width when only the coordinates show.
    let fps_shown = view.width > 0.0;
    out.push_str(r##""#fps_visible":"##);
    out.push_str(if view.visible && fps_shown {
        "true"
    } else {
        "false"
    });
    out.push_str(r##","#fps_text":""##);
    text(&mut out, view.fps);
    out.push_str(r##"","#fps_font":"##);
    fixed(&mut out, s);
    out.push_str(r##","#fps_offset":"##);
    pair(&mut out, view.offset);
    out.push_str(r##","#fps_w":"##);
    fixed(&mut out, size[0]);
    // The label is one line tall; the edit frame below is as tall as both lines.
    out.push_str(r##","#fps_h":"##);
    fixed(&mut out, LINE * s);
    // The coordinates line: under the first, as wide as its own text.
    out.push_str(r##","#fpsc_visible":"##);
    out.push_str(if view.coords.is_some() && view.visible {
        "true"
    } else {
        "false"
    });
    out.push_str(r##","#fpsc_text":""##);
    if let Some((coords, _)) = view.coords {
        coords_text(&mut out, coords);
    }
    // Under the FPS text, or at the top when that is not shown.
    out.push_str(r##"","#fpsc_offset":"##);
    let line = if fps_shown { LINE * s } else { 0.0 };
    pair(&mut out, [view.offset[0], view.offset[1] + line]);
    out.push_str(r##","#fpsc_w":"##);
    fixed(&mut out, (coords_width + 2.0) * s);
    // The edit overlay follows the counter's rectangle.
    out.push_str(r##","#fpsedit_visible":"##);
    out.push_str(if view.edit.is_some() { "true" } else { "false" });
    out.push_str(r##","#fpsedit_offset":"##);
    pair(&mut out, view.offset);
    out.push_str(r##","#fpsedit_w":"##);
    fixed(&mut out, size[0]);
    out.push_str(r##","#fpsedit_h":"##);
    fixed(&mut out, size[1]);
    out.push(',');
    out.push_str(&crate::counter::overlay_bindings(
        "fpsedit", size, s, view.edit,
    ));
    out
}

#[cfg(test)]
mod tests;
