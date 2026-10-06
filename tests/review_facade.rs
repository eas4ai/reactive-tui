//! The code review's C facade findings (docs/spec/roadmap.md,
//! review-facade-findings), each through its requirement's falsifier:
//! docs/spec/ffi.md FFI-002 to FFI-007, docs/spec/text.md TXT-004 and
//! docs/spec/animation.md ANI-010. A test's name starts with its
//! requirement, which is how scripts/cairn/review_facade.py picks it. The C
//! functions need the `ffi` feature; the gap buffer and animation debugger
//! tests run with any features.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

/// Counts the heap allocations of the thread that asked for it (TXT-004).
struct Counting;

thread_local! {
    static TRACKED: Cell<Option<usize>> = const { Cell::new(None) };
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = TRACKED.try_with(|tracked| {
            if let Some(count) = tracked.get() {
                tracked.set(Some(count + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let _ = TRACKED.try_with(|tracked| {
            if let Some(count) = tracked.get() {
                tracked.set(Some(count + 1));
            }
        });
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// The heap allocations `body` makes on this thread.
#[allow(dead_code)]
pub fn allocations_during(body: impl FnOnce()) -> usize {
    TRACKED.with(|tracked| tracked.set(Some(0)));
    body();
    TRACKED.with(|tracked| tracked.replace(None)).unwrap_or(0)
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
