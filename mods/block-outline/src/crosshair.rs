//! The custom crosshair: drawn on the HUD layer in place of the game's own, in the player's own
//! colour, shape and size. Host-independent so `cargo test` can check the shapes and patches.
//!
//! The crosshair is worked out in physical pixels, on whole pixels so thin lines stay sharp, as
//! rectangles (arms, square dots, a drawing's pixels) and circles (a ring, a round dot).
//! Rectangles are drawn as rounded rectangles with no rounding, in the crosshair's colour over
//! a black one a little larger for the outline. Circles are vector paths: filled even-odd in
//! colour, and stroked in black under that, twice the outline's width, so the outline shows just
//! outside the fill. Cinnabar draws nothing of a vector path wider than about 128 physical
//! pixels, so circles are kept within [`MAX_PATH`] and everything else is a rectangle.
//!
//! A binding can only hold a flag, a number, short text or two to four numbers, so shapes cannot
//! be bound. The crosshair is instead a top-level control definition of the HUD document
//! (`xhair`), replaced by `controls` patches; its pieces are small instances of templates (`xf`,
//! `xe`) whose colours are bound. Writing a piece costs a frame several thousand fuel, so a
//! crosshair is sent in steps, one a frame: a header (`xhair`: its box, rings and colours) and
//! [`PARTS`] parts of at most [`PART_RECTS`] rectangles each (`xpart0`...), each only when it
//! changed. The menu's drawing grid is the HUD's `drawgrid`, which the menu embeds
//! (`cinnabar_hud_control`), sent the same way: its lines for the resolution (`gridlines`) and
//! its drawn cells in parts (`gcells0`...).
//!
//! The Custom style is the player's own [`Drawing`], 16, 32 or 64 pixels square, made on the
//! menu. It is drawn as rectangles (runs of pixels in a row, joined with the same run in the
//! rows below), at most [`MAX_RECTS`] of them.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use crate::panel::{digits, fixed};

mod hud;
mod model;
mod shape;

pub use hud::*;
pub use model::*;
pub use shape::*;

#[cfg(test)]
mod tests;
