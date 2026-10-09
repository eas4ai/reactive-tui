//! The calling thread's allocation count and the bytes it holds, kept where
//! reading or changing them allocates nothing, since `add_one`, `hold` and
//! `release` run inside a test binary's global allocator
//! (tests/suprtui_renderer.rs for PNT-003, tests/review_facade.rs for TXT-004
//! and FFI-006). Where the target has native thread-local storage, a
//! `thread_local!` is such a place. The GNU Windows target the push workflow
//! builds (BAR-012) has none: there `thread_local!` boxes a thread's first
//! value, and that box would call the allocator again before the slot
//! exists, and so on until the stack is gone, before the harness prints its
//! first line. On Windows each value is therefore a raw TLS slot of the
//! process.
//!
//! Each test binary includes this file with `#[path]`; it is not a test
//! target of its own, and a binary uses the functions it needs.
#![allow(dead_code)]

#[cfg(not(windows))]
mod platform {
    use std::cell::Cell;

    thread_local! {
        static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
        static HELD: Cell<usize> = const { Cell::new(0) };
    }

    pub fn count() -> usize {
        ALLOCATIONS.with(Cell::get)
    }

    pub fn add_one() {
        // `try_with` fails only while the thread is being torn down.
        let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
    }

    pub fn held() -> usize {
        HELD.with(Cell::get)
    }

    pub fn hold(bytes: usize) {
        let _ = HELD.try_with(|n| n.set(n.get() + bytes));
    }

    pub fn release(bytes: usize) {
        // A block freed on another thread than the one that took it would
        // go below zero here; it is not this thread's to count.
        let _ = HELD.try_with(|n| n.set(n.get().saturating_sub(bytes)));
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicU32, Ordering};
    use windows_sys::Win32::System::Threading::{
        TlsAlloc, TlsFree, TlsGetValue, TlsSetValue, TLS_OUT_OF_INDEXES,
    };

    static COUNT_SLOT: AtomicU32 = AtomicU32::new(TLS_OUT_OF_INDEXES);
    static HELD_SLOT: AtomicU32 = AtomicU32::new(TLS_OUT_OF_INDEXES);

    /// The process's slot behind `cell`, allocated by the first thread that
    /// asks; a thread that loses that race frees the slot it allocated.
    fn slot(cell: &AtomicU32) -> u32 {
        let slot = cell.load(Ordering::Acquire);
        if slot != TLS_OUT_OF_INDEXES {
            return slot;
        }
        // SAFETY: TlsAlloc takes nothing and touches no Rust memory.
        let fresh = unsafe { TlsAlloc() };
        if fresh == TLS_OUT_OF_INDEXES {
            // A message would allocate, inside the allocator.
            std::process::abort();
        }
        match cell.compare_exchange(
            TLS_OUT_OF_INDEXES,
            fresh,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => fresh,
            Err(existing) => {
                // SAFETY: `fresh` is a slot this thread allocated and never
                // stored anything in.
                let _ = unsafe { TlsFree(fresh) };
                existing
            }
        }
    }

    fn get(cell: &AtomicU32) -> usize {
        // SAFETY: the slot comes from TlsAlloc; a thread that never stored
        // reads it as null, which is the value 0.
        unsafe { TlsGetValue(slot(cell)) as usize }
    }

    fn set(cell: &AtomicU32, value: usize) {
        // SAFETY: as above; the stored value is the number itself, never
        // dereferenced as a pointer.
        unsafe {
            let _ = TlsSetValue(slot(cell), value as *const c_void);
        }
    }

    pub fn count() -> usize {
        get(&COUNT_SLOT)
    }

    pub fn add_one() {
        set(&COUNT_SLOT, get(&COUNT_SLOT) + 1);
    }

    pub fn held() -> usize {
        get(&HELD_SLOT)
    }

    pub fn hold(bytes: usize) {
        set(&HELD_SLOT, get(&HELD_SLOT) + bytes);
    }

    pub fn release(bytes: usize) {
        set(&HELD_SLOT, get(&HELD_SLOT).saturating_sub(bytes));
    }
}

#[allow(unused_imports)]
pub use platform::{add_one, count, held, hold, release};
