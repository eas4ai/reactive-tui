//! The code review's C facade findings (docs/spec/roadmap.md,
//! review-facade-findings), each through its requirement's falsifier:
//! docs/spec/ffi.md FFI-002 to FFI-007, docs/spec/text.md TXT-004 and
//! docs/spec/animation.md ANI-010. A test's name starts with its
//! requirement, which is how scripts/cairn/review_facade.py picks it. The C
//! functions need the `ffi` feature; the gap buffer and animation debugger
//! tests run with any features.

use std::alloc::{GlobalAlloc, Layout, System};

/// The calling thread's allocation count, kept where the allocator can
/// touch it without allocating; shared with tests/suprtui_renderer.rs.
#[path = "common/allocation_count.rs"]
mod per_thread;

/// Counts the heap allocations of the thread that asked for it (TXT-004) and
/// the bytes it holds (FFI-006).
struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        per_thread::add_one();
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            per_thread::hold(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        per_thread::release(layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        per_thread::add_one();
        let moved = unsafe { System.realloc(ptr, layout, new_size) };
        if !moved.is_null() {
            per_thread::release(layout.size());
            per_thread::hold(new_size);
        }
        moved
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// The heap allocations `body` makes on this thread.
#[allow(dead_code)]
pub fn allocations_during(body: impl FnOnce()) -> usize {
    let before = per_thread::count();
    body();
    per_thread::count() - before
}

/// The heap bytes this thread has taken and not freed.
#[allow(dead_code)]
pub fn bytes_held() -> usize {
    per_thread::held()
}

#[cfg(feature = "ffi")]
mod common;
#[cfg(all(unix, feature = "ffi"))]
#[allow(dead_code)]
mod pty_support;

#[cfg(feature = "ffi")]
#[path = "review_facade/header.rs"]
mod header;
#[cfg(all(unix, feature = "ffi"))]
#[path = "review_facade/terminal.rs"]
mod terminal;

#[path = "review_facade/debug.rs"]
mod debug;
#[cfg(feature = "ffi")]
#[path = "review_facade/effects.rs"]
mod effects;
#[path = "review_facade/gap_buffer.rs"]
mod gap_buffer;
#[cfg(all(unix, feature = "ffi"))]
#[path = "review_facade/render.rs"]
mod render;
#[cfg(all(unix, feature = "ffi"))]
#[path = "review_facade/stats.rs"]
mod stats;
#[cfg(feature = "ffi")]
#[path = "review_facade/text.rs"]
mod text;
#[cfg(feature = "ffi")]
#[path = "review_facade/version.rs"]
mod version;
#[cfg(feature = "ffi")]
#[path = "review_facade/widgets.rs"]
mod widgets;
