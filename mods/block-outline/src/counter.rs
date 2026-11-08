//! The arrow counter: an arrow icon and the number of arrows carried, drawn on the HUD where the
//! player put it. Host-independent so `cargo test` can check the counting, layout and patches.

use crate::panel::fixed;

/// Where and how the arrow counter is shown; saved with the other settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Counter {
    pub on: bool,
    /// Shown only while a bow or crossbow is held (main hand or offhand).
    pub bow_only: bool,
    /// Where the arrow icon's centre is, measured from the screen edge or middle it is nearest,
    /// so it keeps its place beside the hotbar when the window or GUI scale changes.
    pub pos: Pos,
    /// Size multiplier: 1 is a hotbar-sized icon.
    pub scale: f32,
}

/// Just right of the hotbar, level with it: 105 units right of the middle, 11 above the bottom.
pub const DEFAULT_POS: Pos = Pos {
    anchor: [Anchor::Centre, Anchor::End],
    off: [105.0, -11.0],
};
pub const MIN_SCALE: f32 = 0.5;
pub const MAX_SCALE: f32 = 4.0;
/// One click of - / +, one wheel notch or one key press while resizing.
pub const SCALE_STEP: f32 = 0.1;

impl Default for Counter {
    fn default() -> Self {
        Self {
            on: true,
            bow_only: false,
            pos: DEFAULT_POS,
            scale: 1.0,
        }
    }
}

impl Counter {
    /// `scale` moved by `steps` steps, rounded to whole steps and kept in bounds.
    pub fn stepped(scale: f32, steps: f32) -> f32 {
        clamp_scale(((scale + steps * SCALE_STEP) / SCALE_STEP).round() * SCALE_STEP)
    }
}

pub fn clamp_scale(scale: f32) -> f32 {
    if scale.is_finite() {
        scale.clamp(MIN_SCALE, MAX_SCALE)
    } else {
        1.0
    }
}

/// Which line of the HUD an axis of the counter's position is measured from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    /// The left or top edge.
    Start,
    /// The middle: where the hotbar is centred.
    Centre,
    /// The right or bottom edge.
    End,
    /// Older settings: a fraction of the width or height. Turned into one of the above as
    /// soon as the HUD's size is known.
    Fraction,
}

impl Anchor {
    pub fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Centre => "centre",
            Self::End => "end",
            Self::Fraction => "fraction",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "start" => Self::Start,
            "centre" => Self::Centre,
            "end" => Self::End,
            "fraction" => Self::Fraction,
            _ => return None,
        })
    }
}

/// The counter's position: per axis, a line to measure from and the distance from it in GUI
/// units (or a fraction of the length for [`Anchor::Fraction`]). Measuring from the nearest
/// edge or the middle, not as a share of the screen, keeps a counter placed beside the
/// hotbar beside it when the window or GUI scale changes, as the hotbar is anchored the same
/// way.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pos {
    pub anchor: [Anchor; 2],
    pub off: [f32; 2],
}

/// Furthest an offset may be from its line, in GUI units.
const MAX_OFFSET: f32 = 20_000.0;

impl Pos {
    /// A position as fractions of the HUD's width and height.
    pub fn fraction(x: f32, y: f32) -> Self {
        Self {
            anchor: [Anchor::Fraction; 2],
            off: [x, y].map(|p| {
                if p.is_finite() {
                    p.clamp(0.0, 1.0)
                } else {
                    0.5
                }
            }),
        }
    }

    /// The same position with its offsets made finite and in range.
    pub fn sanitised(self) -> Self {
        let mut out = self;
        for axis in 0..2 {
            out.off[axis] = match (self.anchor[axis], self.off[axis]) {
                (_, o) if !o.is_finite() => 0.0,
                (Anchor::Fraction, o) => o.clamp(0.0, 1.0),
                (_, o) => o.clamp(-MAX_OFFSET, MAX_OFFSET),
            };
        }
        out
    }

    /// Where the arrow icon's centre is in a HUD of size `viewport`.
    pub fn at(self, viewport: [f32; 2]) -> [f32; 2] {
        std::array::from_fn(|axis| {
            let length = viewport[axis];
            let off = self.off[axis];
            match self.anchor[axis] {
                Anchor::Start => off,
                Anchor::Centre => length / 2.0 + off,
                Anchor::End => length + off,
                Anchor::Fraction => off * length,
            }
        })
    }

    /// The position of an icon centred on `at`, measured from whichever of the two edges and
    /// the middle is nearest on each axis.
    pub fn anchored(at: [f32; 2], viewport: [f32; 2]) -> Self {
        let mut anchor = [Anchor::Start; 2];
        let mut off = [0.0; 2];
        for axis in 0..2 {
            let length = viewport[axis];
            let at = if at[axis].is_finite() {
                at[axis]
            } else {
                length / 2.0
            };
            let lines = [
                (Anchor::Start, 0.0),
                (Anchor::Centre, length / 2.0),
                (Anchor::End, length),
            ];
            let (line, base) = lines
                .into_iter()
                .min_by(|a, b| (at - a.1).abs().total_cmp(&(at - b.1).abs()))
                .unwrap_or(lines[1]);
            anchor[axis] = line;
            off[axis] = at - base;
        }
        Self { anchor, off }.sanitised()
    }

    /// Whether an older fraction is still waiting to be turned into an anchored position.
    pub fn is_fraction(self) -> bool {
        self.anchor.contains(&Anchor::Fraction)
    }
}

/// The item counted. Tipped arrows are this item with a potion in their metadata.
pub const ARROW: &str = "minecraft:arrow";

/// What the inventory says about arrows: how many, and which hotbar slots (bits 0-8) and
/// whether the offhand hold something that shoots them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Quiver {
    pub arrows: u32,
    pub bow_slots: u16,
    pub bow_offhand: bool,
}

fn shoots_arrows(identifier: &str) -> bool {
    matches!(identifier, "minecraft:bow" | "minecraft:crossbow")
}

impl Quiver {
    /// Counts `inventory` (hotbar first) and the `offhand` as `(identifier, count)` per cell.
    pub fn count<'a>(
        inventory: impl IntoIterator<Item = Option<(&'a str, u16)>>,
        offhand: Option<(&'a str, u16)>,
    ) -> Self {
        let mut quiver = Self::default();
        for (index, cell) in inventory.into_iter().enumerate() {
            let Some((identifier, count)) = cell else {
                continue;
            };
            if identifier == ARROW {
                quiver.arrows += u32::from(count);
            } else if index < 9 && shoots_arrows(identifier) {
                quiver.bow_slots |= 1 << index;
            }
        }
        if let Some((identifier, count)) = offhand {
            if identifier == ARROW {
                quiver.arrows += u32::from(count);
            } else {
                quiver.bow_offhand = shoots_arrows(identifier);
            }
        }
        quiver
    }

    /// Whether a bow or crossbow is in hand with hotbar slot `selected` chosen.
    pub fn holding_bow(&self, selected: Option<u8>) -> bool {
        self.bow_offhand
            || selected.is_some_and(|slot| slot < 9 && self.bow_slots & (1 << slot) != 0)
    }
}

/// What the HUD shows this frame. Patches are sent only when it changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub visible: bool,
    /// None while the count is unknown (no Player status permission): shown as a yellow ?.
    pub arrows: Option<u32>,
    /// Top-left corner and size of the whole counter, in HUD GUI units.
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub scale: f32,
    /// Width of the number at text scale 1.
    pub text_width: f32,
    /// While the counter is being moved: the edit frame, handles and guides.
    pub edit: Option<Edit>,
}

/// How the edit frame looks while the counter is moved.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edit {
    /// The part under the pointer.
    pub hover: Option<Part>,
    /// The part being dragged.
    pub dragging: Option<Part>,
    /// The vertical (x) and horizontal (y) line the counter snapped to, drawn as a guide.
    pub guides: [Option<f32>; 2],
    /// Too close to the bottom for the size readout below: it goes above.
    pub label_above: bool,
}

/// What the pointer can grab while moving: the counter itself, or one of its corner handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Body,
    /// A resize handle: 0 top left, 1 top right, 2 bottom left, 3 bottom right.
    Handle(u8),
}

/// A drag in progress, from the press that started it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Drag {
    /// Moving: where the pointer holds the counter, from its top-left corner.
    Move { grab: [f32; 2] },
    /// Resizing from a corner handle: the pointer's offset from that corner, the opposite
    /// corner (which stays put) and which way the dragged corner grows (+1 or -1 per axis).
    Resize {
        grab: [f32; 2],
        anchor: [f32; 2],
        dir: [f32; 2],
    },
}

/// Icon side at scale 1, a hotbar slot's item size.
const ICON: f32 = 16.0;
/// Gap between the icon and the number, at scale 1.
const GAP: f32 = 2.0;
/// Text line height at text scale 1.
const LINE: f32 = 10.0;
/// How far the edit frame reaches beyond the counter on each side.
const FRAME: f32 = 3.0;
/// Side of a square resize handle, centred on a corner of the frame.
const HANDLE: f32 = 7.0;
/// Extra reach around the handles and the counter for grabbing them.
const GRAB_MARGIN: f32 = 4.0;
/// Width of the size readout.
const READOUT_W: f32 = 40.0;
/// Edges and centres snap to a guide line within this distance, in GUI units.
pub const SNAP: f32 = 4.0;

/// Cinnabar's hotbar: 182 wide and 22 tall, centred and flush with the bottom of the HUD.
const HOTBAR: [f32; 2] = [182.0, 22.0];
/// Space left between the counter and the hotbar or the screen edge when snapped beside them.
const SNAP_GAP: f32 = 2.0;

/// Which parts of the counter may line up with a guide line.
const START: u8 = 1;
const MIDDLE: u8 = 2;
const END: u8 = 4;

/// The counter's size at `scale` with a number `text_width` wide at text scale 1.
pub fn size(scale: f32, text_width: f32) -> [f32; 2] {
    [(ICON + GAP + text_width) * scale, ICON * scale]
}

/// How long the window and GUI scale must hold still before the HUD layer is published again.
pub const SETTLE_SECONDS: f32 = 0.3;

/// Notices the HUD root changing size or GUI scale (a resized window, a new GUI scale, fullscreen).
/// Text widths measured at the old scale are then wrong and the host may have laid the layer
/// out again, so the counter is re-measured and re-patched at once, and published afresh once
/// the size has stopped changing (not on every step of a window drag).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewportWatch {
    seen: Option<[f32; 3]>,
    settle_at: Option<f32>,
}

/// What a changed HUD root asks for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reaction {
    /// Measure the number again and patch every binding.
    pub remeasure: bool,
    /// Publish the layer again from scratch.
    pub republish: bool,
}

impl ViewportWatch {
    pub const fn new() -> Self {
        Self {
            seen: None,
            settle_at: None,
        }
    }

    /// Feeds the HUD root `[width, height, gui scale]` seen at `now` seconds.
    pub fn observe(&mut self, viewport: [f32; 3], now: f32) -> Reaction {
        let mut reaction = Reaction::default();
        if self.seen != Some(viewport) {
            // The first sighting only records the size: the layer was just published for it.
            if self.seen.is_some() {
                reaction.remeasure = true;
                self.settle_at = Some(now + SETTLE_SECONDS);
            }
            self.seen = Some(viewport);
        }
        if self.settle_at.is_some_and(|at| now >= at) {
            self.settle_at = None;
            reaction.remeasure = true;
            reaction.republish = true;
        }
        reaction
    }
}

/// Width of `digits` digits in Cinnabar's native font when it cannot be measured.
pub fn guess_width(digits: usize) -> f32 {
    6.0 * digits as f32
}

/// Number of decimal digits in `n`.
pub fn digits(n: u32) -> usize {
    n.checked_ilog10().map_or(1, |log| log as usize + 1)
}

/// The top-left corner of a counter of `size` whose arrow icon is centred on `pos`, kept fully
/// on screen. The position is the icon's, not the whole counter's, so the icon stays still
/// when the number gets wider and only the number moves.
pub fn place(pos: Pos, size: [f32; 2], viewport: [f32; 2]) -> [f32; 2] {
    let icon = size[1];
    let centre = pos.at(viewport);
    std::array::from_fn(|axis| {
        let corner = centre[axis] - icon / 2.0;
        corner.min(viewport[axis] - size[axis]).max(0.0)
    })
}

/// The position of the arrow icon of a counter whose top-left corner is at `corner`: the
/// inverse of [`place`].
pub fn centre(corner: [f32; 2], size: [f32; 2], viewport: [f32; 2]) -> Pos {
    let icon = size[1];
    Pos::anchored(corner.map(|c| c + icon / 2.0), viewport)
}

/// The guide lines along `axis` (0: vertical lines at x, 1: horizontal lines at y) and which
/// parts of the counter may line up with each: the hotbar's edges and middle, just beside the
/// hotbar, the screen's middle and its edges.
fn guide_lines(axis: usize, viewport: [f32; 2]) -> [(f32, u8); 7] {
    let [w, h] = viewport;
    if axis == 0 {
        let left = (w - HOTBAR[0]) / 2.0;
        let right = left + HOTBAR[0];
        [
            (w / 2.0, START | MIDDLE | END),
            (left, START | END),
            (right, START | END),
            // Beside the hotbar, a small gap away.
            (left - SNAP_GAP, END),
            (right + SNAP_GAP, START),
            (SNAP_GAP, START),
            (w - SNAP_GAP, END),
        ]
    } else {
        let top = h - HOTBAR[1];
        [
            (h / 2.0, MIDDLE),
            (top, START | END),
            (h, END),
            // Level with the middle of the hotbar.
            (h - HOTBAR[1] / 2.0, MIDDLE),
            // Just above the hotbar.
            (top - SNAP_GAP, END),
            (SNAP_GAP, START),
            (h - SNAP_GAP, END),
        ]
    }
}

/// How far to shift a span starting at `start`, `len` long, to line one of its allowed parts up
/// with the nearest guide line within `SNAP`, and that line.
fn snap_span(start: f32, len: f32, axis: usize, viewport: [f32; 2]) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None;
    for (line, parts) in guide_lines(axis, viewport) {
        for (part, at) in [(START, 0.0), (MIDDLE, len / 2.0), (END, len)] {
            if parts & part == 0 {
                continue;
            }
            let shift = line - (start + at);
            if shift.abs() <= SNAP && best.is_none_or(|(b, _)| shift.abs() < b.abs()) {
                best = Some((shift, line));
            }
        }
    }
    best
}

/// `corner` moved so the counter lines up with the nearest guide on each axis, and those lines.
pub fn snap_move(
    corner: [f32; 2],
    size: [f32; 2],
    viewport: [f32; 2],
) -> ([f32; 2], [Option<f32>; 2]) {
    let mut out = corner;
    let mut lines = [None; 2];
    for axis in 0..2 {
        if let Some((shift, line)) = snap_span(corner[axis], size[axis], axis, viewport) {
            out[axis] += shift;
            lines[axis] = Some(line);
        }
    }
    (out, lines)
}

/// The corner of a counter at `corner` of `size` that handle `index` sits on.
fn handle_corner(index: u8, corner: [f32; 2], size: [f32; 2]) -> [f32; 2] {
    [
        corner[0] + if index & 1 == 1 { size[0] } else { 0.0 },
        corner[1] + if index & 2 == 2 { size[1] } else { 0.0 },
    ]
}

/// The centre of handle `index`: on the frame, just outside the counter's corner.
fn handle_centre(index: u8, corner: [f32; 2], size: [f32; 2]) -> [f32; 2] {
    let at = handle_corner(index, corner, size);
    [
        at[0] + if index & 1 == 1 { FRAME } else { -FRAME },
        at[1] + if index & 2 == 2 { FRAME } else { -FRAME },
    ]
}

/// What `pointer` is over: a corner handle first (they overlap the counter), then the counter.
pub fn hit(pointer: [f32; 2], corner: [f32; 2], size: [f32; 2]) -> Option<Part> {
    let reach = HANDLE / 2.0 + GRAB_MARGIN;
    // A tiny counter's handles overlap: the nearest wins.
    let handle = (0..4)
        .map(|index| {
            let at = handle_centre(index, corner, size);
            let distance = (pointer[0] - at[0]).abs().max((pointer[1] - at[1]).abs());
            (index, distance)
        })
        .filter(|(_, distance)| *distance <= reach)
        .min_by(|a, b| a.1.total_cmp(&b.1));
    if let Some((index, _)) = handle {
        return Some(Part::Handle(index));
    }
    let inside = (0..2).all(|axis| {
        pointer[axis] >= corner[axis] - FRAME - GRAB_MARGIN
            && pointer[axis] <= corner[axis] + size[axis] + FRAME + GRAB_MARGIN
    });
    inside.then_some(Part::Body)
}

/// The drag a press at `pointer` starts on `part` of a counter at `corner` of `size`.
pub fn start_drag(part: Part, pointer: [f32; 2], corner: [f32; 2], size: [f32; 2]) -> Drag {
    match part {
        Part::Body => Drag::Move {
            grab: [pointer[0] - corner[0], pointer[1] - corner[1]],
        },
        Part::Handle(index) => {
            let moving = handle_corner(index, corner, size);
            Drag::Resize {
                grab: [pointer[0] - moving[0], pointer[1] - moving[1]],
                anchor: handle_corner(3 - index, corner, size),
                dir: [
                    if index & 1 == 1 { 1.0 } else { -1.0 },
                    if index & 2 == 2 { 1.0 } else { -1.0 },
                ],
            }
        }
    }
}

/// Where a drag with the pointer at `pointer` puts the counter: its new scale (unchanged when
/// moving), top-left corner and the guide lines it snapped to. A resize keeps the opposite
/// corner still and follows whichever axis the pointer pulls further, then snaps the moving
/// edges to the nearest guide line, so the counter can match the hotbar's height or edges.
pub fn drag_to(
    drag: Drag,
    pointer: [f32; 2],
    scale: f32,
    base: [f32; 2],
    viewport: [f32; 2],
) -> (f32, [f32; 2], [Option<f32>; 2]) {
    match drag {
        Drag::Move { grab } => {
            let size = base.map(|side| side * scale);
            let wanted = [pointer[0] - grab[0], pointer[1] - grab[1]];
            let (corner, lines) = snap_move(wanted, size, viewport);
            (scale, corner, lines)
        }
        Drag::Resize { grab, anchor, dir } => {
            let wanted = (0..2)
                .map(|axis| (pointer[axis] - grab[axis] - anchor[axis]) * dir[axis] / base[axis])
                .fold(f32::NEG_INFINITY, f32::max);
            let mut scale = (clamp_scale(wanted) * 100.0).round() / 100.0;
            let mut lines = [None; 2];
            // Snap whichever moving edge is nearest a guide line it may line up with.
            let mut best: Option<(f32, f32, usize, f32)> = None;
            for axis in 0..2 {
                let edge = anchor[axis] + dir[axis] * base[axis] * scale;
                let part = if dir[axis] > 0.0 { END } else { START };
                for (line, parts) in guide_lines(axis, viewport) {
                    let distance = (line - edge).abs();
                    let snapped = (line - anchor[axis]) * dir[axis] / base[axis];
                    if parts & part != 0
                        && distance <= SNAP
                        && (MIN_SCALE..=MAX_SCALE).contains(&snapped)
                        && best.is_none_or(|(d, ..)| distance < d)
                    {
                        best = Some((distance, snapped, axis, line));
                    }
                }
            }
            if let Some((_, snapped, axis, line)) = best {
                scale = snapped;
                lines[axis] = Some(line);
            }
            let size = base.map(|side| side * scale);
            let corner = std::array::from_fn(|axis| {
                if dir[axis] > 0.0 {
                    anchor[axis]
                } else {
                    anchor[axis] - size[axis]
                }
            });
            (scale, corner, lines)
        }
    }
}

/// Whether the size readout must go above a counter at `corner` of `size`: no room below it.
pub fn label_above(corner: [f32; 2], size: [f32; 2], viewport: [f32; 2]) -> bool {
    corner[1] + size[1] + FRAME + 16.0 > viewport[1]
}

/// One resize handle of the edit overlay: a dark edge around a light square.
macro_rules! overlay_handle {
    ($p:literal, $i:literal) => {
        concat!(
            r##"{"handle_"##, $i, r##"":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[7,7],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":1.5,"color":[0.05,0.05,0.07,1],"layer":4,"bindings":["##,
            r##"{"binding_name":"#"##, $p, r##"_frame_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#"##, $p, r##"_handle_"##, $i, r##"_offset","binding_name_override":"#offset"}],"controls":["##,
            r##"{"dot":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[5,5],"anchor_from":"center","anchor_to":"center","radius":1,"color":[1,1,1,1],"layer":5,"bindings":["##,
            r##"{"binding_name":"#"##, $p, r##"_handle_"##, $i, r##"_color","binding_name_override":"#color"}]}}]}},"##
        )
    };
}

pub(crate) use overlay_handle;

/// The edit overlay of an element of the HUD document, bound by `#<prefix>_*` names: a frame
/// around it (brighter under the pointer, purple while dragged), a resize handle on each corner
/// and the size readout. Its parent panel is the element's rectangle. A string literal, so the
/// document is escaped once when the mod is built, not in a frame: a frame has little fuel.
macro_rules! overlay_doc {
    ($p:literal) => {
        concat!(
            r##"{"frame":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["100% + 6px","100% + 6px"],"anchor_from":"center","anchor_to":"center","radius":3,"color":[1,1,1,0.18],"layer":1,"bindings":["##,
            r##"{"binding_name":"#"##, $p, r##"_frame_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#"##, $p, r##"_frame_color","binding_name_override":"#color"}]}},"##,
            $crate::counter::overlay_handle!($p, "0"),
            $crate::counter::overlay_handle!($p, "1"),
            $crate::counter::overlay_handle!($p, "2"),
            $crate::counter::overlay_handle!($p, "3"),
            r##"{"readout":{"type":"label","text":"#"##, $p, r##"_size_text","size":[40,10],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","color":[1,1,1,0.9],"shadow":true,"localize":false,"text_alignment":"center","layer":4,"bindings":["##,
            r##"{"binding_name":"#"##, $p, r##"_size_text"},"##,
            r##"{"binding_name":"#"##, $p, r##"_frame_visible","binding_name_override":"#visible"},"##,
            r##"{"binding_name":"#"##, $p, r##"_size_offset","binding_name_override":"#offset"}]}}"##
        )
    };
}
pub(crate) use overlay_doc;

/// Length of `text` as a JSON string literal (quotes and backslashes escaped), for [`escape`].
pub const fn escaped_len(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut len = 2;
    let mut i = 0;
    while i < bytes.len() {
        len += match bytes[i] {
            b'"' | b'\\' => 2,
            0..=0x1f => panic!("a control character in a document"),
            _ => 1,
        };
        i += 1;
    }
    len
}

/// `text` as a JSON string literal, at compile time; `N` is [`escaped_len`] of it.
pub const fn escape<const N: usize>(text: &str) -> [u8; N] {
    let bytes = text.as_bytes();
    let mut out = [0u8; N];
    out[0] = b'"';
    let mut at = 1;
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte == b'"' || byte == b'\\' {
            out[at] = b'\\';
            at += 1;
        }
        out[at] = byte;
        at += 1;
        i += 1;
    }
    out[at] = b'"';
    out
}

/// A document, as raw JSON and as a JSON string literal made at compile time.
macro_rules! escaped_const {
    ($raw:ident, $escaped:ident) => {
        const _: () = assert!($crate::counter::escaped_len($raw) > 2);
        const $escaped: &str = {
            const LEN: usize = $crate::counter::escaped_len($raw);
            const BYTES: [u8; LEN] = $crate::counter::escape::<LEN>($raw);
            match core::str::from_utf8(&BYTES) {
                Ok(text) => text,
                Err(_) => panic!("the escaped document is not UTF-8"),
            }
        };
    };
}

/// The HUD document: the crosshair, the CPS readout behind the arrow counter, then the guide
/// lines and the arrow counter.
pub(crate) const DOCUMENT: &str = concat!(
    r##"{"namespace":"boldy_hud","root":{"type":"panel","size":["100%","100%"],"controls":["##,
    crate::crosshair::document!(),
    ",",
    crate::cps::document!(),
    ",",
    crate::fps::document!(),
    ",",
    crate::jumpreset::document!(),
    ",",
    // While moving: the guide lines the element snapped to.
    r##"{"guide_x":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,"100%"],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.7,0.45,1,0.85],"layer":1,"bindings":["##,
    r##"{"binding_name":"#guide_x_visible","binding_name_override":"#visible"},"##,
    r##"{"binding_name":"#guide_x_offset","binding_name_override":"#offset"}]}},"##,
    r##"{"guide_y":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["100%",1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.7,0.45,1,0.85],"layer":1,"bindings":["##,
    r##"{"binding_name":"#guide_y_visible","binding_name_override":"#visible"},"##,
    r##"{"binding_name":"#guide_y_offset","binding_name_override":"#offset"}]}},"##,
    r##"{"arrows":{"type":"panel","size":[34,16],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"bindings":["##,
    r##"{"binding_name":"#arrows_visible","binding_name_override":"#visible"},"##,
    r##"{"binding_name":"#arrows_offset","binding_name_override":"#offset"},"##,
    r##"{"binding_name":"#arrows_w","binding_name_override":"#size_binding_x_absolute"},"##,
    r##"{"binding_name":"#arrows_h","binding_name_override":"#size_binding_y_absolute"}],"controls":["##,
    overlay_doc!("arrows"),
    ",",
    r##"{"icon":{"type":"custom","renderer":"inventory_item_renderer","size":[16,16],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","layer":2,"bindings":["##,
    r##"{"binding_name":"#arrow_item","binding_name_override":"#item_renderer_data"},"##,
    r##"{"binding_name":"#arrows_icon","binding_name_override":"#size_binding_x_absolute"},"##,
    r##"{"binding_name":"#arrows_icon","binding_name_override":"#size_binding_y_absolute"}]}},"##,
    r##"{"count":{"type":"label","text":"#arrows_text","size":[18,10],"offset":[18,4],"anchor_from":"top_left","anchor_to":"top_left","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"left","font_scale_factor":1,"layer":3,"bindings":["##,
    r##"{"binding_name":"#arrows_text"},"##,
    r##"{"binding_name":"#arrows_color","binding_name_override":"#color"},"##,
    r##"{"binding_name":"#arrows_font","binding_name_override":"#font_scale_factor"},"##,
    r##"{"binding_name":"#arrows_text_offset","binding_name_override":"#offset"},"##,
    r##"{"binding_name":"#arrows_text_w","binding_name_override":"#size_binding_x_absolute"},"##,
    r##"{"binding_name":"#arrows_text_h","binding_name_override":"#size_binding_y_absolute"}]}}"##,
    r##"]}}]},"##,
    // Definitions the root and the menu draw: the crosshair, and the menu's drawing grid.
    crate::crosshair::definitions!(),
    "}",
);
escaped_const!(DOCUMENT, DOCUMENT_ESCAPED);

/// The bindings of the edit overlay for an element of `size` at `scale`, as `"name":value`
/// pairs without braces or a trailing comma. The frame is shown while `edit` is some.
pub fn overlay_bindings(p: &str, size: [f32; 2], scale: f32, edit: Option<Edit>) -> String {
    let e = edit.unwrap_or_default();
    let [w, h] = size;
    let mut out = String::with_capacity(620);
    let pair = |out: &mut String, [x, y]: [f32; 2]| {
        out.push('[');
        fixed(out, x);
        out.push(',');
        fixed(out, y);
        out.push(']');
    };
    // `"#<prefix><tail>`, without formatting: a frame has little fuel.
    let name = |out: &mut String, tail: &str| {
        out.push_str("\"#");
        out.push_str(p);
        out.push_str(tail);
    };
    name(&mut out, "_frame_visible\":");
    out.push_str(if edit.is_some() { "true," } else { "false," });
    name(&mut out, "_frame_color\":");
    out.push_str(match (e.dragging, e.hover) {
        (Some(_), _) => "[0.55,0.3,0.85,0.45]",
        (None, Some(_)) => "[1,1,1,0.3]",
        (None, None) => "[1,1,1,0.16]",
    });
    for index in 0..4u8 {
        let digit = char::from(b'0' + index);
        let handle = Some(Part::Handle(index));
        out.push(',');
        name(&mut out, "_handle_");
        out.push(digit);
        out.push_str("_color\":");
        out.push_str(if e.dragging == handle {
            "[0.75,0.5,1,1]"
        } else if e.dragging.is_none() && e.hover == handle {
            "[1,1,1,1]"
        } else {
            "[0.8,0.8,0.85,0.9]"
        });
        out.push(',');
        name(&mut out, "_handle_");
        out.push(digit);
        out.push_str("_offset\":");
        // Relative to the element's top-left corner.
        let at = handle_centre(index, [0.0, 0.0], size);
        pair(&mut out, [at[0] - HANDLE / 2.0, at[1] - HANDLE / 2.0]);
    }
    out.push(',');
    name(&mut out, "_size_text\":\"");
    crate::panel::digits(&mut out, (scale * 100.0).round().max(0.0) as u64);
    out.push_str("%\",");
    name(&mut out, "_size_offset\":");
    pair(
        &mut out,
        [
            (w - READOUT_W) / 2.0,
            if e.label_above {
                -FRAME - HANDLE / 2.0 - 12.0
            } else {
                h + FRAME + HANDLE / 2.0 + 2.0
            },
        ],
    );
    out
}

/// The HUD layer, as `hud.set-surface` takes it: hidden until the first patch places it. The
/// document is escaped at compile time; only the starting bindings are written here.
pub fn surface() -> String {
    let mut out = String::with_capacity(DOCUMENT_ESCAPED.len() + 2600);
    out.push_str(r#"{"screen":"boldy_hud.root","document":"#);
    out.push_str(DOCUMENT_ESCAPED);
    let mut initial = bindings(&View {
        visible: false,
        arrows: Some(0),
        offset: [0.0; 2],
        size: size(1.0, guess_width(1)),
        scale: 1.0,
        text_width: guess_width(1),
        edit: None,
    });
    // The arrow artwork never changes: bound once here, beside the patched bindings.
    initial.pop();
    initial.push_str(r##","#arrow_item":{"item":{"identifier":"minecraft:arrow","metadata":0}},"##);
    initial.push_str(&crate::cps::initial_bindings());
    initial.push(',');
    initial.push_str(&crate::fps::initial_bindings());
    initial.push(',');
    initial.push_str(&crate::jumpreset::initial_bindings());
    initial.push(',');
    initial.push_str(&crate::crosshair::initial_bindings());
    initial.push('}');
    out.push_str(r#","bindings":"#);
    out.push_str(&initial);
    out.push('}');
    out
}

/// A `hud.update-surface` patch showing `view`.
pub fn patch(view: &View) -> String {
    let mut out = String::with_capacity(900);
    out.push_str(r#"{"binding_changes":"#);
    out.push_str(&bindings(view));
    out.push('}');
    out
}

/// Every binding the counter reads, as one JSON object.
fn bindings(view: &View) -> String {
    let s = view.scale;
    let [w, h] = view.size;
    let edit = view.edit.unwrap_or_default();
    let mut out = String::with_capacity(1100);
    let pair = |out: &mut String, [x, y]: [f32; 2]| {
        out.push('[');
        fixed(out, x);
        out.push(',');
        fixed(out, y);
        out.push(']');
    };
    let flag = |out: &mut String, name: &str, on: bool| {
        out.push_str(name);
        out.push_str(if on { "true," } else { "false," });
    };
    out.push('{');
    flag(&mut out, r##""#arrows_visible":"##, view.visible);
    // The guide lines the dragged element snapped to, shared by every element being edited.
    let editing = view.edit.is_some();
    flag(
        &mut out,
        r##""#guide_x_visible":"##,
        editing && edit.guides[0].is_some(),
    );
    flag(
        &mut out,
        r##""#guide_y_visible":"##,
        editing && edit.guides[1].is_some(),
    );
    out.push_str(r##""#guide_x_offset":"##);
    pair(&mut out, [edit.guides[0].unwrap_or(0.0) - 0.5, 0.0]);
    out.push_str(r##","#guide_y_offset":"##);
    pair(&mut out, [0.0, edit.guides[1].unwrap_or(0.0) - 0.5]);
    out.push_str(r##","#arrows_offset":"##);
    pair(&mut out, view.offset);
    out.push_str(r##","#arrows_w":"##);
    fixed(&mut out, w);
    out.push_str(r##","#arrows_h":"##);
    fixed(&mut out, h);
    out.push(',');
    out.push_str(&overlay_bindings("arrows", view.size, s, view.edit));
    out.push_str(r##","#arrows_icon":"##);
    fixed(&mut out, ICON * s);
    out.push_str(r##","#arrows_text":""##);
    match view.arrows {
        Some(arrows) => crate::panel::digits(&mut out, u64::from(arrows)),
        None => out.push('?'),
    }
    out.push_str(r##"","#arrows_color":"##);
    // Out of arrows: the number turns red; unknown: a yellow ?.
    out.push_str(match view.arrows {
        Some(0) => "[1,0.35,0.35,1]",
        Some(_) => "[1,1,1,1]",
        None => "[1,0.85,0.3,1]",
    });
    out.push_str(r##","#arrows_font":"##);
    fixed(&mut out, s);
    out.push_str(r##","#arrows_text_offset":"##);
    // Vertically centred beside the icon.
    pair(
        &mut out,
        [(ICON + GAP) * s, (ICON - LINE) / 2.0 * s + 0.5 * s],
    );
    out.push_str(r##","#arrows_text_w":"##);
    // A little spare so the shadow is not clipped.
    fixed(&mut out, (view.text_width + 2.0) * s);
    out.push_str(r##","#arrows_text_h":"##);
    fixed(&mut out, LINE * s);
    out.push('}');
    out
}

#[cfg(test)]
mod tests;
