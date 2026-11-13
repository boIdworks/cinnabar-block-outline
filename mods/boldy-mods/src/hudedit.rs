//! The HUD editor's logic: the arrow counter, CPS, FPS and armor HUD all show their frames; one drag moves or resizes one of them. Host-independent, so `cargo test` can check grabbing,
//! dragging, snapping, resizing and which element takes the keys.

use crate::counter::{self, Counter, Drag, Edit, Part, Pos};

/// One element being edited: where it is and how big (a multiple of its size at scale 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Item {
    pub pos: Pos,
    pub scale: f32,
}

/// What the pointer and keys did this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    /// The pointer in HUD GUI units, while the menu owns it.
    pub pointer: Option<[f32; 2]>,
    /// The left button went down this frame.
    pub pressed: bool,
    /// The left button is down.
    pub held: bool,
    /// Resize steps asked for with the wheel and the + and - keys, see [`resize_steps`].
    pub steps: f32,
}

/// Resize steps from a wheel turn (`wheel` notches, positive up) and the keys pressed.
pub fn resize_steps(wheel: f32, keys: &[String]) -> f32 {
    let mut steps = 0.0;
    if wheel.is_finite() && wheel != 0.0 {
        steps += wheel.signum();
    }
    for key in keys {
        match key.as_str() {
            "Equal" | "NumpadAdd" => steps += 1.0,
            "Minus" | "NumpadSubtract" => steps -= 1.0,
            _ => {}
        }
    }
    steps
}

/// The editor's state between frames.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Editor {
    /// The element grabbed last (0 arrows, 1 CPS, 2 FPS, 3 armor): on
    /// top when they overlap, and the one the keys act on when the pointer is over none.
    pub selected: usize,
    /// The drag in progress: the element, the part grabbed and how.
    pub drag: Option<(usize, Part, Drag)>,
}

/// What one frame of editing shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Output<const N: usize> {
    /// How each element's frame looks, in the order of the items.
    pub edits: [Edit; N],
    /// Something is in the way of the guide box at the top of the screen.
    pub guide_blocked: bool,
}

impl Editor {
    /// One frame. `items` are moved and resized in place; `bases` are the elements' sizes at
    /// scale 1; `guide` is the width and height the guide box takes at the top of the screen.
    pub fn step<const N: usize>(
        &mut self,
        items: &mut [Item; N],
        bases: [[f32; 2]; N],
        viewport: [f32; 2],
        input: Input,
        guide: [f32; 2],
    ) -> Output<N> {
        if self.selected >= N {
            self.selected = 0;
        }
        let scaled = |index: usize, scale: f32| bases[index].map(|side| side * scale);
        let rect = |items: &[Item; N], index: usize| {
            let size = scaled(index, items[index].scale);
            (counter::place(items[index].pos, size, viewport), size)
        };
        let mut edits = [Edit::default(); N];
        let mut guides = [None; 2];
        let mut under = None;
        if !input.held {
            self.drag = None;
        }
        if let Some(pointer) = input.pointer {
            // What is under the pointer, the selected element first.
            let selected = self.selected;
            for index in core::iter::once(selected).chain((0..N).filter(|&i| i != selected)) {
                let (corner, size) = rect(items, index);
                if let Some(part) = counter::hit(pointer, corner, size) {
                    under = Some((index, part));
                    break;
                }
            }
            if input.pressed
                && self.drag.is_none()
                && let Some((index, part)) = under
            {
                let (corner, size) = rect(items, index);
                let drag = counter::start_drag(part, pointer, corner, size);
                self.drag = Some((index, part, drag));
                self.selected = index;
            }
            match self.drag {
                Some((index, part, drag)) => {
                    let (scale, at, lines) =
                        counter::drag_to(drag, pointer, items[index].scale, bases[index], viewport);
                    items[index] = Item {
                        pos: counter::centre(at, scaled(index, scale), viewport),
                        scale,
                    };
                    guides = lines;
                    edits[index].dragging = Some(part);
                    self.selected = index;
                }
                None => {
                    if let Some((index, part)) = under {
                        edits[index].hover = Some(part);
                    }
                }
            }
        }
        if self.drag.is_none() && input.steps != 0.0 {
            // The wheel and keys resize what the pointer is over, else the one grabbed last.
            let target = under.map_or(self.selected, |(index, _)| index);
            items[target].scale = Counter::stepped(items[target].scale, input.steps);
        }
        // The guide box sits at the top of the screen unless an element is in its way there.
        let mut guide_blocked = false;
        for (index, edit) in edits.iter_mut().enumerate() {
            let (corner, size) = rect(items, index);
            edit.label_above = counter::label_above(corner, size, viewport);
            edit.guides = guides;
            guide_blocked |= corner[0] < (viewport[0] + guide[0]) / 2.0
                && corner[0] + size[0] > (viewport[0] - guide[0]) / 2.0
                && corner[1] < guide[1];
        }
        Output {
            edits,
            guide_blocked,
        }
    }
}

#[cfg(test)]
mod tests;
