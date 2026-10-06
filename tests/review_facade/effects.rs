//! Part of tests/review_facade.rs: FFI-006, a C effect runs on the signals it
//! reads, and the hooks hand out one signal per key.

use reactive_tui::ffi::*;
use std::cell::Cell;
use std::ffi::c_void;
use std::ptr;

/// What the effect's callbacks see: the signal to read and their counts.
struct Counter {
    signal: *mut RTuiSignal,
    runs: Cell<usize>,
    cleanups: Cell<usize>,
    seen: Cell<i64>,
}

extern "C" fn effect_body(user_data: *mut c_void) {
    let counter = unsafe { &*(user_data as *const Counter) };
    let mut value = 0i64;
    unsafe { rtui_signal_int_get(counter.signal, &mut value) };
    counter.seen.set(value);
    counter.runs.set(counter.runs.get() + 1);
}

extern "C" fn effect_cleanup(user_data: *mut c_void) {
    let counter = unsafe { &*(user_data as *const Counter) };
    counter.cleanups.set(counter.cleanups.get() + 1);
}

/// FFI-006: an effect runs when made, again after a signal it read changes,
/// with its cleanup before the rerun and at destroy, and by hand.
#[test]
fn ffi_006_an_effect_runs_when_made_and_when_its_signal_changes() {
    unsafe {
        let mut signal = ptr::null_mut();
        assert_eq!(
            rtui_signal_int_create(1, &mut signal),
            ReactiveError::Success,
            "rtui_signal_int_create failed"
        );
        let counter = Box::new(Counter {
            signal,
            runs: Cell::new(0),
            cleanups: Cell::new(0),
            seen: Cell::new(0),
        });
        let user_data = &*counter as *const Counter as *mut c_void;
        let mut effect = ptr::null_mut();
        assert_eq!(
            rtui_effect_create(effect_body, Some(effect_cleanup), user_data, &mut effect),
            ReactiveError::Success,
            "rtui_effect_create failed"
        );
        assert_eq!(
            counter.runs.get(),
            1,
            "FFI-006: the effect ran {} times when made, not once",
            counter.runs.get()
        );
        assert_eq!(
            rtui_signal_int_set(signal, 2),
            ReactiveError::Success,
            "rtui_signal_int_set failed"
        );
        assert_eq!(
            (counter.runs.get(), counter.seen.get(), counter.cleanups.get()),
            (2, 2, 1),
            "FFI-006: after the signal changed to 2 the effect had run {} times, last saw {}, with {} cleanups (expected 2, 2, 1)",
            counter.runs.get(),
            counter.seen.get(),
            counter.cleanups.get()
        );
        assert_eq!(
            rtui_effect_run(effect),
            ReactiveError::Success,
            "rtui_effect_run failed"
        );
        assert_eq!(
            (counter.runs.get(), counter.cleanups.get()),
            (3, 2),
            "FFI-006: running the effect by hand gave {} runs and {} cleanups (expected 3 and 2)",
            counter.runs.get(),
            counter.cleanups.get()
        );
        rtui_effect_destroy(effect);
        assert_eq!(
            counter.cleanups.get(),
            3,
            "FFI-006: destroying the effect left {} cleanups, not 3",
            counter.cleanups.get()
        );
        rtui_signal_destroy(signal);
    }
}

/// FFI-006: the hooks hand out the same signal for the same key.
#[test]
fn ffi_006_hooks_hand_out_one_signal_per_key() {
    unsafe {
        let hooks = rtui_hooks_new();
        assert!(!hooks.is_null(), "rtui_hooks_new gave null");
        let first = rtui_use_signal_int(hooks, c"count".as_ptr(), 1);
        let second = rtui_use_signal_int(hooks, c"count".as_ptr(), 1);
        assert!(
            !first.is_null() && !second.is_null(),
            "rtui_use_signal_int gave null"
        );
        assert_eq!(rtui_signal_set_int(first, 5), ReactiveError::Success);
        let read = rtui_signal_get_int(second);
        rtui_signal_destroy(first);
        rtui_signal_destroy(second);
        rtui_hooks_destroy(hooks);
        assert_eq!(
            read, 5,
            "FFI-006: two hooks signals for one key did not share a value: set 5, read {read}"
        );
    }
}

/// FFI-006: the header describes the effects and the hooks as they are.
#[test]
fn ffi_006_the_header_describes_effects_and_hooks() {
    let effect = crate::header::doc_of("rtui_effect_create");
    let hooks = crate::header::doc_of("rtui_use_signal_int");
    assert!(
        effect.contains("signal") && effect.contains("run"),
        "FFI-006: the header's documentation of rtui_effect_create ({effect:?}) does not say when an effect runs"
    );
    assert!(
        hooks.contains("key"),
        "FFI-006: the header's documentation of rtui_use_signal_int ({hooks:?}) does not describe the keyed signal storage"
    );
}
