//! The calling thread's allocation count, kept where reading it or adding to
//! it allocates nothing, since `add_one` runs inside a test binary's global
//! allocator (tests/suprtui_renderer.rs for PNT-003, tests/review_facade.rs
//! for TXT-004). Where the target has native thread-local storage, a
//! `thread_local!` is such a place. The GNU Windows target the push workflow
//! builds (BAR-012) has none: there `thread_local!` boxes a thread's first
//! value, and that box would call the allocator again before the slot
//! exists, and so on until the stack is gone, before the harness prints its
//! first line. On Windows the count is therefore the value of a raw TLS slot
//! of the process.
//!
//! Each test binary includes this file with `#[path]`; it is not a test
//! target of its own.

#[cfg(not(windows))]
mod platform {
    use std::cell::Cell;

    thread_local! {
        static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    }

    pub fn count() -> usize {
        ALLOCATIONS.with(Cell::get)
    }

    pub fn add_one() {
        // `try_with` fails only while the thread is being torn down.
        let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicU32, Ordering};
    use windows_sys::Win32::System::Threading::{
        TlsAlloc, TlsFree, TlsGetValue, TlsSetValue, TLS_OUT_OF_INDEXES,
    };

    static SLOT: AtomicU32 = AtomicU32::new(TLS_OUT_OF_INDEXES);

    /// The process's slot for the count, allocated by the first thread that
    /// asks; a thread that loses that race frees the slot it allocated.
    fn slot() -> u32 {
        let slot = SLOT.load(Ordering::Acquire);
        if slot != TLS_OUT_OF_INDEXES {
            return slot;
        }
        // SAFETY: TlsAlloc takes nothing and touches no Rust memory.
        let fresh = unsafe { TlsAlloc() };
        if fresh == TLS_OUT_OF_INDEXES {
            // A message would allocate, inside the allocator.
            std::process::abort();
        }
        match SLOT.compare_exchange(
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

    pub fn count() -> usize {
        // SAFETY: the slot comes from TlsAlloc; a thread that never added
        // reads it as null, which is the count 0.
        unsafe { TlsGetValue(slot()) as usize }
    }

    pub fn add_one() {
        let slot = slot();
        // SAFETY: as above; the stored value is the count itself, never
        // dereferenced as a pointer.
        unsafe {
            let count = TlsGetValue(slot) as usize + 1;
            let _ = TlsSetValue(slot, count as *const c_void);
        }
    }
}

pub use platform::{add_one, count};
