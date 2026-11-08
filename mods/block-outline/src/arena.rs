//! A scratch arena for the large host replies (terrain regions, actor lists).
//!
//! Every frame callback has a small fuel budget. A changed terrain region arrives with a name,
//! a canonical state and a list of tags for each block state, so lifting it makes hundreds of
//! small allocations, and dropping the previous one as many frees. Through the general allocator
//! that alone can use up a frame's fuel on servers with many custom blocks. While the arena is
//! on, allocations are bump-allocated from a fixed buffer instead (a few instructions each),
//! frees of arena memory do nothing, and [`Arena::scratch`] reclaims everything at once. Callers
//! copy the little they keep into normal memory before resetting.
//!
//! The arena logic lives in [`Arena`], which owns its buffer and is tested natively; the
//! `#[global_allocator]` that routes the guest's allocations through one is wasm-only.

// Only the wasm guest installs the allocator; natively the arena is exercised by the tests.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#![warn(clippy::undocumented_unsafe_blocks)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::{Cell, UnsafeCell};

/// The buffer, aligned for the largest alignment the guest asks for in practice (a larger
/// request still works: it is aligned within the buffer, or goes to the system allocator).
#[repr(C, align(16))]
struct Space<const N: usize>(UnsafeCell<[u8; N]>);

/// A bump allocator over `N` bytes of its own memory.
///
/// It is off until [`Arena::scratch`] turns it on. All state is in [`Cell`]s, so no reference
/// into it is ever held across an allocation (which re-enters it).
pub struct Arena<const N: usize> {
    space: Space<N>,
    /// Offset of the first free byte.
    next: Cell<usize>,
    /// Whether new allocations are taken from the buffer.
    on: Cell<bool>,
    /// Whether an allocation did not fit while the arena was on (and went to the system
    /// allocator instead), so that some of what a `scratch` read made lives outside the buffer.
    spilled: Cell<bool>,
}

// SAFETY: the guest runs on a single thread, so the cells and the buffer are never accessed
// concurrently.
unsafe impl<const N: usize> Sync for Arena<N> {}

impl<const N: usize> Arena<N> {
    pub const fn new() -> Self {
        Arena {
            space: Space(UnsafeCell::new([0; N])),
            next: Cell::new(0),
            on: Cell::new(false),
            spilled: Cell::new(false),
        }
    }

    fn start(&self) -> *mut u8 {
        self.space.0.get().cast()
    }

    /// Whether `ptr` points into the buffer.
    pub fn contains(&self, ptr: *mut u8) -> bool {
        (ptr as usize).wrapping_sub(self.start() as usize) < N
    }

    /// Takes `layout` from the buffer, or `None` when the arena is off or has no room left (in
    /// which case the caller uses the system allocator; a miss while on is remembered).
    pub fn bump(&self, layout: Layout) -> Option<*mut u8> {
        if !self.on.get() {
            return None;
        }
        let base = self.start() as usize;
        let mask = layout.align() - 1;
        let first = base.checked_add(self.next.get())?.checked_add(mask)? & !mask;
        let offset = first - base;
        match offset.checked_add(layout.size()) {
            Some(end) if end <= N => {
                self.next.set(end);
                // SAFETY: `offset + size <= N`, so the result stays inside the buffer (or one
                // past its end for an empty allocation).
                Some(unsafe { self.start().add(offset) })
            }
            _ => {
                self.spilled.set(true);
                None
            }
        }
    }

    /// Runs `read` with allocations in the arena, then `keep` on its result with normal
    /// allocations, and frees the whole arena. Only what `keep` returns survives. `T` must be
    /// plain data (no handles or other `Drop` side effects): when it all lies in the arena it is
    /// not dropped piece by piece, which would visit every string of a terrain reply, but freed
    /// with the arena.
    pub fn scratch<T, R>(&self, read: impl FnOnce() -> T, keep: impl FnOnce(&T) -> R) -> R {
        if self.on.get() {
            // Nested use would reset memory the outer call still holds.
            let value = read();
            return keep(&value);
        }
        self.on.set(true);
        self.spilled.set(false);
        let value = read();
        self.on.set(false);
        let kept = keep(&value);
        if self.spilled.get() {
            // Some of it lives outside the arena and must be freed normally.
            drop(value);
        } else {
            std::mem::forget(value);
        }
        // Nothing allocated in the arena is alive any more.
        self.next.set(0);
        kept
    }
}

impl<const N: usize> Default for Arena<N> {
    fn default() -> Self {
        Self::new()
    }
}

const SIZE: usize = 2 << 20;

static ARENA: Arena<SIZE> = Arena::new();

/// Routes allocations to the arena while it is on, otherwise to the system allocator.
pub struct ScratchAllocator;

// SAFETY: `alloc` returns either a block of the arena that nothing else uses until the arena is
// reset, or the system allocator's block. `dealloc` and `realloc` hand pointers outside the
// arena back to the system allocator, which is where they came from, and leave arena blocks
// alone (they are reclaimed together).
unsafe impl GlobalAlloc for ScratchAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match ARENA.bump(layout) {
            Some(block) => block,
            // SAFETY: the caller's `layout` obligations pass straight through.
            None => unsafe { System.alloc(layout) },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !ARENA.contains(ptr) {
            // SAFETY: not an arena block, so the system allocator made it with this layout.
            unsafe { System.dealloc(ptr, layout) }
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if !ARENA.contains(ptr) {
            // SAFETY: not an arena block, so the system allocator made it with this layout.
            return unsafe { System.realloc(ptr, layout, new_size) };
        }
        // SAFETY: `new_size` is non-zero and does not overflow `isize` when rounded up to the
        // alignment, which `GlobalAlloc::realloc`'s contract requires of the caller.
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        // SAFETY: `new_layout` is valid (above).
        let new = unsafe { self.alloc(new_layout) };
        if !new.is_null() {
            // SAFETY: `ptr` holds `layout.size()` bytes, `new` holds `new_size`, and they are
            // different blocks, so the first `min` bytes of each are valid and do not overlap.
            unsafe { std::ptr::copy_nonoverlapping(ptr, new, layout.size().min(new_size)) };
        }
        new
    }
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static GLOBAL: ScratchAllocator = ScratchAllocator;

/// [`Arena::scratch`] on the guest's arena.
pub fn scratch<T, R>(read: impl FnOnce() -> T, keep: impl FnOnce(&T) -> R) -> R {
    ARENA.scratch(read, keep)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(size: usize, align: usize) -> Layout {
        Layout::from_size_align(size, align).expect("a valid layout")
    }

    #[test]
    fn an_arena_that_is_off_hands_out_nothing() {
        let arena = Arena::<256>::new();
        assert!(arena.bump(layout(8, 8)).is_none());
        assert!(!arena.spilled.get(), "a miss while off is not a spill");
    }

    #[test]
    fn blocks_are_aligned_inside_the_buffer_and_do_not_overlap() {
        let arena = Arena::<256>::new();
        arena.scratch(
            || {
                let a = arena.bump(layout(3, 1)).expect("room");
                let b = arena.bump(layout(8, 8)).expect("room");
                let c = arena.bump(layout(16, 16)).expect("room");
                assert!(arena.contains(a) && arena.contains(b) && arena.contains(c));
                assert_eq!(b as usize % 8, 0);
                assert_eq!(c as usize % 16, 0);
                assert!(b as usize >= a as usize + 3, "b starts after a ends");
                assert!(c as usize >= b as usize + 8, "c starts after b ends");
            },
            |_| (),
        );
    }

    #[test]
    fn what_does_not_fit_is_left_to_the_caller_and_remembered() {
        let arena = Arena::<64>::new();
        arena.scratch(
            || {
                assert!(arena.bump(layout(48, 1)).is_some());
                assert!(!arena.spilled.get());
                assert!(
                    arena.bump(layout(32, 1)).is_none(),
                    "only 16 bytes are left"
                );
                assert!(arena.spilled.get(), "the miss is remembered");
                assert!(
                    arena.bump(layout(16, 1)).is_some(),
                    "a smaller one still fits"
                );
                assert!(
                    arena.bump(layout(usize::MAX / 2, 1)).is_none(),
                    "no overflow"
                );
            },
            |_| (),
        );
    }

    #[test]
    fn scratch_reclaims_everything_when_it_ends() {
        let arena = Arena::<64>::new();
        for _ in 0..3 {
            arena.scratch(
                || {
                    assert!(
                        arena.bump(layout(64, 1)).is_some(),
                        "the whole buffer is free"
                    );
                },
                |_| (),
            );
            assert_eq!(arena.next.get(), 0);
            assert!(arena.bump(layout(1, 1)).is_none(), "and it is off again");
        }
    }

    #[test]
    fn nested_scratch_does_not_reset_the_outer_one() {
        let arena = Arena::<64>::new();
        arena.scratch(
            || {
                assert!(arena.bump(layout(32, 1)).is_some());
                arena.scratch(|| (), |_| ());
                assert_eq!(
                    arena.next.get(),
                    32,
                    "the inner call left the outer one's memory"
                );
            },
            |_| (),
        );
    }

    #[test]
    fn keep_sees_the_value_and_only_its_result_survives() {
        let arena = Arena::<64>::new();
        let total = arena.scratch(|| [1u32, 2, 3], |numbers| numbers.iter().sum::<u32>());
        assert_eq!(total, 6);
    }

    #[test]
    fn pointers_outside_the_buffer_are_not_contained() {
        let arena = Arena::<64>::new();
        let outside = 0u8;
        assert!(!arena.contains(&outside as *const u8 as *mut u8));
        assert!(!arena.contains(std::ptr::null_mut()));
    }
}
