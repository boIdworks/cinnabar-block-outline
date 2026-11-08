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
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            on: true,
            pos: DEFAULT_POS,
            scale: 1.0,
        }
    }
}

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
    /// Width of the text at text scale 1.
    pub width: f32,
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
        offset: [0.0; 2],
        edit: None,
    };
}

/// The size of the counter at text scale 1: the text's width, a little spare so the shadow is
/// not clipped, and one line's height.
pub fn base(width: f32) -> [f32; 2] {
    [width + 2.0, LINE]
}

/// The size of the counter at `scale`.
pub fn size(scale: f32, width: f32) -> [f32; 2] {
    base(width).map(|side| side * scale)
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
    let size = size(s, view.width);
    out.push_str(r##""#fps_visible":"##);
    out.push_str(if view.visible { "true" } else { "false" });
    out.push_str(r##","#fps_text":""##);
    text(&mut out, view.fps);
    out.push_str(r##"","#fps_font":"##);
    fixed(&mut out, s);
    out.push_str(r##","#fps_offset":"##);
    pair(&mut out, view.offset);
    out.push_str(r##","#fps_w":"##);
    fixed(&mut out, size[0]);
    out.push_str(r##","#fps_h":"##);
    fixed(&mut out, size[1]);
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
