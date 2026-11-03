//! A scratch arena for the large host replies (terrain regions, actor lists).
//!
//! Every frame callback has a small fuel budget. A changed terrain region arrives with a name,
//! a canonical state and a list of tags for each block state, so lifting it makes hundreds of
//! small allocations, and dropping the previous one as many frees. Through the general allocator
//! that alone can use up a frame's fuel on servers with many custom blocks. While the arena is
//! on, allocations are bump-allocated from a fixed buffer instead (a few instructions each),
//! frees of arena memory do nothing, and [`reset`] reclaims everything at once. Callers copy
//! the little they keep into normal memory before resetting.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::UnsafeCell;

const SIZE: usize = 2 << 20;

#[repr(C, align(16))]
struct Space(UnsafeCell<[u8; SIZE]>);

/// Bump offset into the space, whether new allocations go there, and whether one did not fit
/// (and went to the system allocator instead) while it was on.
struct State(UnsafeCell<(usize, bool, bool)>);

// The guest runs on one thread.
unsafe impl Sync for Space {}
unsafe impl Sync for State {}

static SPACE: Space = Space(UnsafeCell::new([0; SIZE]));
static STATE: State = State(UnsafeCell::new((0, false, false)));

fn base() -> usize {
    SPACE.0.get() as usize
}

fn in_arena(ptr: *mut u8) -> bool {
    let at = ptr as usize;
    at >= base() && at < base() + SIZE
}

/// Routes allocations to the arena while it is on, otherwise to the system allocator.
pub struct ScratchAllocator;

unsafe impl GlobalAlloc for ScratchAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let state = unsafe { &mut *STATE.0.get() };
        if state.1 {
            let start = (base() + state.0 + layout.align() - 1) & !(layout.align() - 1);
            let end = start + layout.size();
            if end <= base() + SIZE {
                state.0 = end - base();
                return start as *mut u8;
            }
            state.2 = true;
        }
        // Too big for what is left, or the arena is off.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !in_arena(ptr) {
            unsafe { System.dealloc(ptr, layout) }
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if !in_arena(ptr) {
            return unsafe { System.realloc(ptr, layout, new_size) };
        }
        let new =
            unsafe { self.alloc(Layout::from_size_align_unchecked(new_size, layout.align())) };
        if !new.is_null() {
            unsafe { std::ptr::copy_nonoverlapping(ptr, new, layout.size().min(new_size)) };
        }
        new
    }
}

#[global_allocator]
static GLOBAL: ScratchAllocator = ScratchAllocator;

/// Runs `read` with allocations in the arena, then `keep` on its result with normal allocations,
/// and frees the whole arena. Only what `keep` returns survives. `T` must be plain data (no
/// handles or other `Drop` side effects): when it all lies in the arena it is not dropped piece
/// by piece, which would visit every string of a terrain reply, but freed with the arena.
pub fn scratch<T, R>(read: impl FnOnce() -> T, keep: impl FnOnce(&T) -> R) -> R {
    let state = unsafe { &mut *STATE.0.get() };
    let was_on = state.1;
    if was_on {
        // Nested use would reset memory the outer call still holds.
        let value = read();
        return keep(&value);
    }
    state.1 = true;
    state.2 = false;
    let value = read();
    let spilled = unsafe {
        let state = &mut *STATE.0.get();
        state.1 = false;
        state.2
    };
    let kept = keep(&value);
    if spilled {
        // Some of it lives outside the arena and must be freed normally.
        drop(value);
    } else {
        std::mem::forget(value);
    }
    // Nothing allocated in the arena is alive any more.
    unsafe { (*STATE.0.get()).0 = 0 };
    kept
}
