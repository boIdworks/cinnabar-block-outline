//! The HUD editor's logic: the arrow counter and the CPS readout both show their frames; one
//! drag moves or resizes one of them. Host-independent, so `cargo test` can check grabbing,
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
    /// The element grabbed last (0 the arrow counter, 1 the CPS readout): on top when they
    /// overlap, and the one the keys act on when the pointer is over neither.
    pub selected: usize,
    /// The drag in progress: the element, the part grabbed and how.
    pub drag: Option<(usize, Part, Drag)>,
}

/// What one frame of editing shows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Output {
    /// How each element's frame looks: the arrow counter's, then the CPS readout's.
    pub edits: [Edit; 2],
    /// Something is in the way of the guide box at the top of the screen.
    pub guide_blocked: bool,
}

impl Editor {
    /// One frame. `items` are moved and resized in place; `bases` are the elements' sizes at
    /// scale 1; `guide` is the width and height the guide box takes at the top of the screen.
    pub fn step(
        &mut self,
        items: &mut [Item; 2],
        bases: [[f32; 2]; 2],
        viewport: [f32; 2],
        input: Input,
        guide: [f32; 2],
    ) -> Output {
        let scaled = |index: usize, scale: f32| bases[index].map(|side| side * scale);
        let rect = |items: &[Item; 2], index: usize| {
            let size = scaled(index, items[index].scale);
            (counter::place(items[index].pos, size, viewport), size)
        };
        let mut edits = [Edit::default(); 2];
        let mut guides = [None; 2];
        let mut under = None;
        if !input.held {
            self.drag = None;
        }
        if let Some(pointer) = input.pointer {
            // What is under the pointer, the selected element first.
            for index in [self.selected, 1 - self.selected] {
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
mod tests {
    use super::*;
    use crate::counter::{Anchor, MAX_SCALE};

    const VIEWPORT: [f32; 2] = [480.0, 270.0];
    const GUIDE: [f32; 2] = [240.0, 84.0];
    // The arrow counter (34 by 16 at scale 1) and the CPS readout (60 by 10).
    const BASES: [[f32; 2]; 2] = [[34.0, 16.0], [60.0, 10.0]];

    /// The counter beside the hotbar and the readout in the top left corner.
    fn items() -> [Item; 2] {
        [
            Item {
                pos: Pos {
                    anchor: [Anchor::Centre, Anchor::End],
                    off: [103.0, -11.0],
                },
                scale: 1.0,
            },
            Item {
                pos: crate::cps::DEFAULT_POS,
                scale: 1.0,
            },
        ]
    }

    fn corner(items: &[Item; 2], index: usize) -> [f32; 2] {
        let size = BASES[index].map(|side| side * items[index].scale);
        counter::place(items[index].pos, size, VIEWPORT)
    }

    fn mouse(at: [f32; 2], pressed: bool, held: bool) -> Input {
        Input {
            pointer: Some(at),
            pressed,
            held,
            steps: 0.0,
        }
    }

    fn step(editor: &mut Editor, items: &mut [Item; 2], input: Input) -> Output {
        editor.step(items, BASES, VIEWPORT, input, GUIDE)
    }

    #[test]
    fn dragging_the_readout_moves_only_the_readout() {
        let (mut editor, mut items) = (Editor::default(), items());
        let counter_before = items[0];
        let from = corner(&items, 1);
        let grab = [from[0] + 20.0, from[1] + 5.0];
        // The press picks the readout up and the frame lights as dragged.
        let out = step(&mut editor, &mut items, mouse(grab, true, true));
        assert_eq!(editor.selected, 1);
        assert!(out.edits[1].dragging.is_some() && out.edits[0].dragging.is_none());
        // Moving the pointer carries it along; the other element stays where it was.
        let out = step(
            &mut editor,
            &mut items,
            mouse([grab[0] + 100.0, grab[1] + 60.0], false, true),
        );
        let to = corner(&items, 1);
        assert!((to[0] - (from[0] + 100.0)).abs() < SNAP_SLACK, "{to:?}");
        assert!((to[1] - (from[1] + 60.0)).abs() < SNAP_SLACK, "{to:?}");
        assert_eq!(items[0], counter_before);
        assert!(out.edits[1].dragging.is_some());
        // Letting go ends the drag; the frame stays lit only while hovered.
        let out = step(
            &mut editor,
            &mut items,
            mouse([grab[0] + 100.0, grab[1] + 60.0], false, false),
        );
        assert_eq!(editor.drag, None);
        assert!(out.edits[1].dragging.is_none() && out.edits[1].hover.is_some());
        assert_eq!(out.edits[0].hover, None);
    }

    /// Snapping may pull a drag by up to this many units.
    const SNAP_SLACK: f32 = counter::SNAP + 0.01;

    #[test]
    fn a_corner_handle_resizes_with_the_opposite_corner_still() {
        let (mut editor, mut items) = (Editor::default(), items());
        let from = corner(&items, 1);
        let size = BASES[1];
        // Grab the bottom right handle (3) and pull out to double the size.
        let handle = [from[0] + size[0] + 3.0, from[1] + size[1] + 3.0];
        let out = step(&mut editor, &mut items, mouse(handle, true, true));
        assert_eq!(out.edits[1].dragging, Some(Part::Handle(3)));
        step(
            &mut editor,
            &mut items,
            mouse([handle[0] + size[0], handle[1] + size[1]], false, true),
        );
        assert!(items[1].scale > 1.5, "{}", items[1].scale);
        let after = corner(&items, 1);
        assert!((after[0] - from[0]).abs() < 0.6 && (after[1] - from[1]).abs() < 0.6);
    }

    #[test]
    fn the_element_grabbed_last_wins_where_they_overlap() {
        let (mut editor, mut items) = (Editor::default(), items());
        // Put the readout right on top of the counter.
        items[1].pos = items[0].pos;
        let at = corner(&items, 0);
        let inside = [at[0] + 10.0, at[1] + 8.0];
        // Nothing grabbed yet: the arrow counter (index 0) is on top.
        step(&mut editor, &mut items, mouse(inside, true, true));
        assert_eq!(editor.drag.map(|(index, ..)| index), Some(0));
        step(&mut editor, &mut items, mouse(inside, false, false));
        // Once the readout is the selected one, it takes the grab instead.
        editor.selected = 1;
        step(&mut editor, &mut items, mouse(inside, true, true));
        assert_eq!(editor.drag.map(|(index, ..)| index), Some(1));
    }

    #[test]
    fn keys_resize_what_the_pointer_is_over_else_the_selected_one() {
        let (mut editor, mut items) = (Editor::default(), items());
        let over_readout = {
            let at = corner(&items, 1);
            [at[0] + 20.0, at[1] + 5.0]
        };
        let bigger = |pointer| Input {
            pointer,
            steps: 1.0,
            ..Input::default()
        };
        step(&mut editor, &mut items, bigger(Some(over_readout)));
        assert!((items[1].scale - 1.1).abs() < 1e-4 && items[0].scale == 1.0);
        // Hovering does not select: with the pointer over neither, the one grabbed last (the
        // arrow counter, by default) takes the key...
        step(&mut editor, &mut items, bigger(Some([300.0, 100.0])));
        assert!((items[0].scale - 1.1).abs() < 1e-4 && (items[1].scale - 1.1).abs() < 1e-4);
        // ...until the readout is the one grabbed last.
        editor.selected = 1;
        step(&mut editor, &mut items, bigger(None));
        assert!((items[1].scale - 1.2).abs() < 1e-4 && (items[0].scale - 1.1).abs() < 1e-4);
        editor.selected = 0;
        // Sizes stop at the limits, and nothing resizes during a drag.
        for _ in 0..80 {
            step(&mut editor, &mut items, bigger(None));
        }
        assert_eq!(items[0].scale, MAX_SCALE);
        let at = corner(&items, 1);
        let grab = [at[0] + 20.0, at[1] + 5.0];
        step(&mut editor, &mut items, mouse(grab, true, true));
        let before = items[1].scale;
        let mut input = mouse(grab, false, true);
        input.steps = 1.0;
        step(&mut editor, &mut items, input);
        assert_eq!(items[1].scale, before);
    }

    #[test]
    fn the_wheel_and_keys_make_steps() {
        let keys = |names: &[&str]| names.iter().map(|k| (*k).to_owned()).collect::<Vec<_>>();
        assert_eq!(resize_steps(0.0, &[]), 0.0);
        assert_eq!(resize_steps(3.0, &[]), 1.0);
        assert_eq!(resize_steps(-0.5, &[]), -1.0);
        assert_eq!(resize_steps(f32::NAN, &[]), 0.0);
        assert_eq!(resize_steps(0.0, &keys(&["Equal", "NumpadAdd"])), 2.0);
        assert_eq!(resize_steps(1.0, &keys(&["Minus", "KeyA"])), 0.0);
    }

    #[test]
    fn the_guide_box_moves_down_only_when_something_is_in_its_way() {
        let (mut editor, mut items) = (Editor::default(), items());
        // The readout in the top left and the counter at the bottom leave the top clear.
        let out = step(&mut editor, &mut items, Input::default());
        assert!(!out.guide_blocked);
        // The readout moved to the top middle is in the way.
        items[1].pos = Pos {
            anchor: [Anchor::Centre, Anchor::Start],
            off: [0.0, 9.0],
        };
        let out = step(&mut editor, &mut items, Input::default());
        assert!(out.guide_blocked);
    }

    #[test]
    fn dragged_guide_lines_show_on_both_frames() {
        let (mut editor, mut items) = (Editor::default(), items());
        // Drag the counter until its middle is within a few units of the screen's middle.
        let at = corner(&items, 0);
        let grab = [at[0] + 17.0, at[1] + 8.0];
        step(&mut editor, &mut items, mouse(grab, true, true));
        let out = step(
            &mut editor,
            &mut items,
            mouse([VIEWPORT[0] / 2.0 + 1.0, grab[1]], false, true),
        );
        assert_eq!(out.edits[0].guides[0], Some(VIEWPORT[0] / 2.0));
        assert_eq!(out.edits[1].guides, out.edits[0].guides);
    }
}
