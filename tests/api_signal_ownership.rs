#![cfg(feature = "ffi")]

use reactive_tui::ffi::*;
use std::ffi::{CStr, CString};
use std::ptr;

#[test]
fn legacy_typed_round_trips_and_common_destruction() {
    for _ in 0..32 {
        let mut signal = ptr::null_mut();
        assert_eq!(
            rtui_signal_int_create(i64::MIN, &mut signal),
            ReactiveError::Success
        );
        let mut value = 0;
        assert_eq!(
            rtui_signal_int_get(signal, &mut value),
            ReactiveError::Success
        );
        assert_eq!(value, i64::MIN);
        assert_eq!(
            rtui_signal_int_set(signal, i64::MAX),
            ReactiveError::Success
        );
        assert_eq!(
            rtui_signal_int_get(signal, &mut value),
            ReactiveError::Success
        );
        assert_eq!(value, i64::MAX);
        assert_eq!(
            rtui_signal_set_int(signal, 3),
            ReactiveError::InvalidParameter
        );
        rtui_signal_destroy(signal);

        assert_eq!(
            rtui_signal_float_create(-1.25, &mut signal),
            ReactiveError::Success
        );
        let mut float = 0.0;
        assert_eq!(rtui_signal_float_set(signal, 4.5), ReactiveError::Success);
        assert_eq!(
            rtui_signal_float_get(signal, &mut float),
            ReactiveError::Success
        );
        assert_eq!(float, 4.5);
        assert_eq!(rtui_signal_get_float_new(signal), 4.5);
        rtui_signal_destroy_new(signal);

        assert_eq!(
            rtui_signal_bool_create(false, &mut signal),
            ReactiveError::Success
        );
        let mut boolean = false;
        assert_eq!(rtui_signal_bool_set(signal, true), ReactiveError::Success);
        assert_eq!(
            rtui_signal_bool_get(signal, &mut boolean),
            ReactiveError::Success
        );
        assert!(boolean);
        rtui_signal_destroy(signal);

        let initial = CString::new("界🙂".repeat(64)).unwrap();
        assert_eq!(
            rtui_signal_string_create(initial.as_ptr(), &mut signal),
            ReactiveError::Success
        );
        let owned = rtui_signal_get_string_owned(signal);
        assert!(!owned.is_null());
        assert_eq!(unsafe { CStr::from_ptr(owned) }, initial.as_c_str());
        rtui_string_free(owned);
        assert_eq!(
            rtui_signal_string_set(signal, c"updated".as_ptr()),
            ReactiveError::Success
        );
        let mut buf = [0; 32];
        assert_eq!(
            rtui_signal_string_get(signal, buf.as_mut_ptr(), buf.len()),
            ReactiveError::Success
        );
        assert_eq!(unsafe { CStr::from_ptr(buf.as_ptr()) }, c"updated");
        rtui_signal_destroy(signal);
    }
}

#[test]
fn tagged_types_reject_mismatch_and_both_destructors_release_handles() {
    let int = rtui_signal_new_int(i32::MIN);
    assert!(!int.is_null());
    assert_eq!(rtui_signal_get_int(int), i32::MIN);
    assert_eq!(rtui_signal_set_int(int, i32::MAX), ReactiveError::Success);
    assert_eq!(rtui_signal_get_int(int), i32::MAX);
    let mut wide = 99;
    assert_eq!(
        rtui_signal_int_get(int, &mut wide),
        ReactiveError::InvalidParameter
    );
    assert_eq!(wide, 99);
    assert_eq!(
        rtui_signal_bool_set(int, true),
        ReactiveError::InvalidParameter
    );
    assert!(rtui_signal_get_string_owned(int).is_null());
    rtui_signal_destroy(int);
    let float = rtui_signal_new_float(-2.0);
    assert_eq!(
        rtui_signal_set_float_new(float, 8.25),
        ReactiveError::Success
    );
    assert_eq!(rtui_signal_get_float_new(float), 8.25);
    rtui_signal_destroy_new(float);
    let boolean = rtui_signal_new_bool(true);
    assert!(rtui_signal_get_bool_new(boolean));
    assert_eq!(
        rtui_signal_set_bool_new(boolean, false),
        ReactiveError::Success
    );
    assert!(!rtui_signal_get_bool_new(boolean));
    rtui_signal_destroy(boolean);
    let string = rtui_signal_new_string(c"start".as_ptr());
    assert_eq!(
        rtui_signal_set_string_new(string, c"finish".as_ptr()),
        ReactiveError::Success
    );
    let mut buf = [0; 32];
    assert_eq!(
        rtui_signal_string_get(string, buf.as_mut_ptr(), 1),
        ReactiveError::BufferTooSmall
    );
    assert_eq!(
        rtui_signal_string_get(string, buf.as_mut_ptr(), buf.len()),
        ReactiveError::Success
    );
    assert_eq!(unsafe { CStr::from_ptr(buf.as_ptr()) }, c"finish");
    rtui_signal_destroy_new(string);
    rtui_signal_destroy(ptr::null_mut());
    rtui_signal_destroy_new(ptr::null_mut());
    assert_eq!(
        rtui_signal_int_create(1, ptr::null_mut()),
        ReactiveError::NullPointer
    );
    assert_eq!(
        rtui_signal_int_get(ptr::null(), &mut wide),
        ReactiveError::NullPointer
    );
}

#[test]
fn hook_handles_own_shared_values_past_context_destruction() {
    let hooks = rtui_hooks_new();
    let a = rtui_use_signal_int(hooks, c"count".as_ptr(), 1);
    let b = rtui_use_signal_int(hooks, c"count".as_ptr(), 2);
    assert!(!a.is_null() && !b.is_null());
    assert_eq!(rtui_signal_set_int(a, 42), ReactiveError::Success);
    assert_eq!(rtui_signal_get_int(b), 42);
    rtui_signal_destroy(a);
    rtui_hooks_destroy(hooks);
    assert_eq!(rtui_signal_get_int(b), 42);
    rtui_signal_destroy_new(b);
}

#[test]
fn thread_safe_signal_has_its_own_matching_lifecycle() {
    let mut signal = ptr::null_mut();
    assert_eq!(
        rtui_thread_safe_signal_string_create(c"initial".as_ptr(), &mut signal),
        ReactiveError::Success
    );
    assert_eq!(
        rtui_thread_safe_signal_string_set(signal, c"changed".as_ptr()),
        ReactiveError::Success
    );
    let mut buf = [0; 32];
    assert_eq!(
        rtui_thread_safe_signal_string_get(signal, buf.as_mut_ptr(), buf.len()),
        ReactiveError::Success
    );
    assert_eq!(unsafe { CStr::from_ptr(buf.as_ptr()) }, c"changed");
    rtui_thread_safe_signal_destroy(signal);
}
