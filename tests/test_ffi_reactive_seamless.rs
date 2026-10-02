//! Tests for the seamless reactive FFI bindings
//!
//! These tests verify that the new reactive API provides a seamless
//! experience for C developers working with reactive state.

#![allow(unused_unsafe)]

#[cfg(feature = "ffi")]
mod ffi_reactive_tests {
    use reactive_tui::ffi::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_seamless_signal_creation() {
        // Test integer signals
        let int_signal = rtui_signal_new_int(42);
        assert!(!int_signal.is_null());

        let value = rtui_signal_get_int(int_signal);
        assert_eq!(value, 42);

        let result = rtui_signal_set_int(int_signal, 100);
        assert_eq!(result, ReactiveError::Success);

        let new_value = rtui_signal_get_int(int_signal);
        assert_eq!(new_value, 100);

        rtui_signal_destroy_new(int_signal);
    }

    #[test]
    fn test_seamless_string_signals() {
        let initial = CString::new("Hello, World!").unwrap();
        let signal = rtui_signal_new_string(initial.as_ptr());
        assert!(!signal.is_null());

        // Get the string value
        let value_ptr = rtui_signal_get_string_owned(signal);
        assert!(!value_ptr.is_null());

        let value_cstr = unsafe { std::ffi::CStr::from_ptr(value_ptr) };
        let value_str = value_cstr.to_str().unwrap();
        assert_eq!(value_str, "Hello, World!");

        // Free the returned string
        rtui_string_free(value_ptr);

        // Set a new value
        let new_value = CString::new("Updated!").unwrap();
        let result = rtui_signal_set_string_new(signal, new_value.as_ptr());
        assert_eq!(result, ReactiveError::Success);

        // Verify the update
        let updated_ptr = rtui_signal_get_string_owned(signal);
        let updated_cstr = unsafe { std::ffi::CStr::from_ptr(updated_ptr) };
        let updated_str = updated_cstr.to_str().unwrap();
        assert_eq!(updated_str, "Updated!");

        rtui_string_free(updated_ptr);
        rtui_signal_destroy_new(signal);
    }

    #[test]
    fn test_seamless_boolean_signals() {
        let signal = rtui_signal_new_bool(true);
        assert!(!signal.is_null());

        let value = rtui_signal_get_bool_new(signal);
        assert_eq!(value, true);

        let result = rtui_signal_set_bool_new(signal, false);
        assert_eq!(result, ReactiveError::Success);

        let new_value = rtui_signal_get_bool_new(signal);
        assert_eq!(new_value, false);

        rtui_signal_destroy_new(signal);
    }

    #[test]
    fn test_seamless_float_signals() {
        let signal = rtui_signal_new_float(3.14159);
        assert!(!signal.is_null());

        let value = rtui_signal_get_float_new(signal);
        assert!((value - 3.14159).abs() < 0.0001);

        let result = rtui_signal_set_float_new(signal, 2.71828);
        assert_eq!(result, ReactiveError::Success);

        let new_value = rtui_signal_get_float_new(signal);
        assert!((new_value - 2.71828).abs() < 0.0001);

        rtui_signal_destroy_new(signal);
    }

    #[test]
    fn test_hooks_system() {
        let hooks = rtui_hooks_new();
        assert!(!hooks.is_null());

        // Use integer signal with hooks
        let key = CString::new("counter").unwrap();
        let signal = rtui_use_signal_int(hooks, key.as_ptr(), 0);
        assert!(!signal.is_null());

        let value = rtui_signal_get_int(signal);
        assert_eq!(value, 0);

        // Update the signal
        let result = rtui_signal_set_int(signal, 5);
        assert_eq!(result, ReactiveError::Success);

        let new_value = rtui_signal_get_int(signal);
        assert_eq!(new_value, 5);

        // Use the same signal again - should return the same instance
        let signal2 = rtui_use_signal_int(hooks, key.as_ptr(), 999);
        assert!(!signal2.is_null());

        // Should have the updated value, not the initial value
        let value2 = rtui_signal_get_int(signal2);
        assert_eq!(value2, 5); // Not 999, because it's the same signal

        rtui_signal_destroy_new(signal);
        rtui_signal_destroy_new(signal2);
        rtui_hooks_destroy(hooks);
    }

    #[test]
    fn test_hooks_string_signals() {
        let hooks = rtui_hooks_new();
        assert!(!hooks.is_null());

        let key = CString::new("message").unwrap();
        let initial = CString::new("Hello").unwrap();
        let signal = rtui_use_signal_string(hooks, key.as_ptr(), initial.as_ptr());
        assert!(!signal.is_null());

        let value_ptr = rtui_signal_get_string_owned(signal);
        let value_cstr = unsafe { std::ffi::CStr::from_ptr(value_ptr) };
        let value_str = value_cstr.to_str().unwrap();
        assert_eq!(value_str, "Hello");

        rtui_string_free(value_ptr);
        rtui_signal_destroy_new(signal);
        rtui_hooks_destroy(hooks);
    }

    #[test]
    fn test_hooks_boolean_signals() {
        {
            let hooks = rtui_hooks_new();
            assert!(!hooks.is_null());

            let key = CString::new("enabled").unwrap();
            let signal = rtui_use_signal_bool(hooks, key.as_ptr(), true);
            assert!(!signal.is_null());

            let value = rtui_signal_get_bool_new(signal);
            assert_eq!(value, true);

            rtui_signal_destroy_new(signal);
            rtui_hooks_destroy(hooks);
        }
    }

    #[test]
    fn test_type_safety() {
        {
            // Create an integer signal
            let int_signal = rtui_signal_new_int(42);
            assert!(!int_signal.is_null());

            // Try to get it as a string - should return null/error
            let string_ptr = rtui_signal_get_string_owned(int_signal);
            assert!(string_ptr.is_null()); // Type mismatch should return null

            // Try to get it as a bool - should return false (default)
            let bool_value = rtui_signal_get_bool_new(int_signal);
            assert_eq!(bool_value, false); // Type mismatch returns default

            // But getting as int should work
            let int_value = rtui_signal_get_int(int_signal);
            assert_eq!(int_value, 42);

            rtui_signal_destroy_new(int_signal);
        }
    }

    #[test]
    fn test_null_pointer_safety() {
        {
            // Test null pointer handling
            let value = rtui_signal_get_int(ptr::null());
            assert_eq!(value, 0); // Should return default value

            let result = rtui_signal_set_int(ptr::null_mut(), 42);
            assert_eq!(result, ReactiveError::NullPointer);

            let string_ptr = rtui_signal_get_string_owned(ptr::null());
            assert!(string_ptr.is_null());

            // Destroying null pointer should be safe
            rtui_signal_destroy_new(ptr::null_mut()); // Should not crash
        }
    }

    #[test]
    fn test_memory_management() {
        {
            // Create and destroy many signals to test memory management
            for i in 0..100 {
                let signal = rtui_signal_new_int(i);
                assert!(!signal.is_null());

                let value = rtui_signal_get_int(signal);
                assert_eq!(value, i);

                rtui_signal_destroy_new(signal);
            }

            // Test string signals
            for i in 0..50 {
                let text = format!("Test string {}", i);
                let c_string = CString::new(text.clone()).unwrap();
                let signal = rtui_signal_new_string(c_string.as_ptr());
                assert!(!signal.is_null());

                let value_ptr = rtui_signal_get_string_owned(signal);
                let value_cstr = unsafe { std::ffi::CStr::from_ptr(value_ptr) };
                let value_str = value_cstr.to_str().unwrap();
                assert_eq!(value_str, text);

                rtui_string_free(value_ptr);
                rtui_signal_destroy_new(signal);
            }
        }
    }

    #[test]
    fn test_hooks_persistence() {
        {
            let hooks = rtui_hooks_new();
            assert!(!hooks.is_null());

            let key = CString::new("persistent").unwrap();

            // Create signal with initial value
            let signal1 = rtui_use_signal_int(hooks, key.as_ptr(), 100);
            assert!(!signal1.is_null());

            // Update the signal
            let result = rtui_signal_set_int(signal1, 200);
            assert_eq!(result, ReactiveError::Success);

            // Get the same signal again - should have the updated value
            let signal2 = rtui_use_signal_int(hooks, key.as_ptr(), 999);
            let value = rtui_signal_get_int(signal2);
            assert_eq!(value, 200); // Should be 200, not 100 or 999

            rtui_signal_destroy_new(signal1);
            rtui_signal_destroy_new(signal2);
            rtui_hooks_destroy(hooks);
        }
    }
}
