//! Reactive system FFI functions

use super::*;
use crate::reactive::signal::{Signal, ThreadSafeSignal};
// Note: hooks module doesn't exist yet, implementing basic reactive primitives
use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;

/// Opaque handle to a signal
#[repr(C)]
pub struct RTuiSignal {
    _private: [u8; 0],
}

/// Opaque handle to a thread-safe signal
#[repr(C)]
pub struct RTuiThreadSafeSignal {
    _private: [u8; 0],
}

/// Opaque handle to an effect
#[repr(C)]
pub struct RTuiEffect {
    _private: [u8; 0],
}

/// Opaque handle to a memo
#[repr(C)]
pub struct RTuiMemo {
    _private: [u8; 0],
}

/// Signal value types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiSignalValueType {
    String = 0,
    Integer = 1,
    Float = 2,
    Boolean = 3,
    Custom = 4,
}

/// Signal value union
#[repr(C)]
pub union RTuiSignalValue {
    pub string_value: *const c_char,
    pub int_value: i64,
    pub float_value: f64,
    pub bool_value: bool,
    pub custom_ptr: *mut std::ffi::c_void,
}

/// Signal change callback function type
pub type RTuiSignalChangeCallback = extern "C" fn(
    old_value: RTuiSignalValue,
    new_value: RTuiSignalValue,
    value_type: RTuiSignalValueType,
    user_data: *mut std::ffi::c_void,
);

/// Effect callback function type
pub type RTuiEffectCallback = extern "C" fn(user_data: *mut std::ffi::c_void);

/// Effect cleanup callback function type
pub type RTuiEffectCleanupCallback = extern "C" fn(user_data: *mut std::ffi::c_void);

/// Memo compute callback function type
pub type RTuiMemoComputeCallback = extern "C" fn(user_data: *mut std::ffi::c_void) -> RTuiSignalValue;

/// Create a string signal
#[no_mangle]
pub extern "C" fn rtui_signal_string_create(
    initial_value: *const c_char,
    out_signal: *mut *mut RTuiSignal,
) -> ReactiveError {
    if out_signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let initial_str = if initial_value.is_null() {
            String::new()
        } else {
            CStr::from_ptr(initial_value)
                .to_str()
                .map_err(|_| ReactiveError::InvalidUtf8)?
                .to_string()
        };

        let signal = Signal::new(initial_str);
        *out_signal = Box::into_raw(Box::new(signal)) as *mut RTuiSignal;
        Ok(())
    }))
}

/// Create an integer signal
#[no_mangle]
pub extern "C" fn rtui_signal_int_create(
    initial_value: i64,
    out_signal: *mut *mut RTuiSignal,
) -> ReactiveError {
    if out_signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let signal = Signal::new(initial_value);
        unsafe {
            *out_signal = Box::into_raw(Box::new(signal)) as *mut RTuiSignal;
        }
        Ok(())
    }))
}

/// Create a float signal
#[no_mangle]
pub extern "C" fn rtui_signal_float_create(
    initial_value: f64,
    out_signal: *mut *mut RTuiSignal,
) -> ReactiveError {
    if out_signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let signal = Signal::new(initial_value);
        unsafe {
            *out_signal = Box::into_raw(Box::new(signal)) as *mut RTuiSignal;
        }
        Ok(())
    }))
}

/// Create a boolean signal
#[no_mangle]
pub extern "C" fn rtui_signal_bool_create(
    initial_value: bool,
    out_signal: *mut *mut RTuiSignal,
) -> ReactiveError {
    if out_signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let signal = Signal::new(initial_value);
        unsafe {
            *out_signal = Box::into_raw(Box::new(signal)) as *mut RTuiSignal;
        }
        Ok(())
    }))
}

/// Destroy a signal
#[no_mangle]
pub extern "C" fn rtui_signal_destroy(signal: *mut RTuiSignal) {
    if !signal.is_null() {
        unsafe {
            // Note: This is simplified - in practice you'd need to handle different signal types
            let _ = Box::from_raw(signal as *mut Signal<String>);
        }
    }
}

/// Get string signal value
#[no_mangle]
pub extern "C" fn rtui_signal_string_get(
    signal: *const RTuiSignal,
    buffer: *mut c_char,
    buffer_size: usize,
) -> ReactiveError {
    if signal.is_null() || buffer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &*(signal as *const Signal<String>);
        let value = signal_ref.get();
        
        if value.len() >= buffer_size {
            return Err(ReactiveError::BufferTooSmall);
        }

        let c_string = CString::new(value).map_err(|_| ReactiveError::InvalidUtf8)?;
        let bytes = c_string.as_bytes_with_nul();
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
        Ok(())
    }))
}

/// Set string signal value
#[no_mangle]
pub extern "C" fn rtui_signal_string_set(
    signal: *mut RTuiSignal,
    value: *const c_char,
) -> ReactiveError {
    if signal.is_null() || value.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let value_str = CStr::from_ptr(value)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?
            .to_string();

        let signal_ref = &mut *(signal as *mut Signal<String>);
        signal_ref.set(value_str);
        Ok(())
    }))
}

/// Get integer signal value
#[no_mangle]
pub extern "C" fn rtui_signal_int_get(
    signal: *const RTuiSignal,
    out_value: *mut i64,
) -> ReactiveError {
    if signal.is_null() || out_value.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &*(signal as *const Signal<i64>);
        *out_value = signal_ref.get();
        Ok(())
    }))
}

/// Set integer signal value
#[no_mangle]
pub extern "C" fn rtui_signal_int_set(
    signal: *mut RTuiSignal,
    value: i64,
) -> ReactiveError {
    if signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &mut *(signal as *mut Signal<i64>);
        signal_ref.set(value);
        Ok(())
    }))
}

/// Get float signal value
#[no_mangle]
pub extern "C" fn rtui_signal_float_get(
    signal: *const RTuiSignal,
    out_value: *mut f64,
) -> ReactiveError {
    if signal.is_null() || out_value.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &*(signal as *const Signal<f64>);
        *out_value = signal_ref.get();
        Ok(())
    }))
}

/// Set float signal value
#[no_mangle]
pub extern "C" fn rtui_signal_float_set(
    signal: *mut RTuiSignal,
    value: f64,
) -> ReactiveError {
    if signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &mut *(signal as *mut Signal<f64>);
        signal_ref.set(value);
        Ok(())
    }))
}

/// Get boolean signal value
#[no_mangle]
pub extern "C" fn rtui_signal_bool_get(
    signal: *const RTuiSignal,
    out_value: *mut bool,
) -> ReactiveError {
    if signal.is_null() || out_value.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &*(signal as *const Signal<bool>);
        *out_value = signal_ref.get();
        Ok(())
    }))
}

/// Set boolean signal value
#[no_mangle]
pub extern "C" fn rtui_signal_bool_set(
    signal: *mut RTuiSignal,
    value: bool,
) -> ReactiveError {
    if signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &mut *(signal as *mut Signal<bool>);
        signal_ref.set(value);
        Ok(())
    }))
}

/// Create a thread-safe string signal
#[no_mangle]
pub extern "C" fn rtui_thread_safe_signal_string_create(
    initial_value: *const c_char,
    out_signal: *mut *mut RTuiThreadSafeSignal,
) -> ReactiveError {
    if out_signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let initial_str = if initial_value.is_null() {
            String::new()
        } else {
            CStr::from_ptr(initial_value)
                .to_str()
                .map_err(|_| ReactiveError::InvalidUtf8)?
                .to_string()
        };

        let signal = ThreadSafeSignal::new(initial_str);
        *out_signal = Box::into_raw(Box::new(signal)) as *mut RTuiThreadSafeSignal;
        Ok(())
    }))
}

/// Destroy a thread-safe signal
#[no_mangle]
pub extern "C" fn rtui_thread_safe_signal_destroy(signal: *mut RTuiThreadSafeSignal) {
    if !signal.is_null() {
        unsafe {
            let _ = Box::from_raw(signal as *mut ThreadSafeSignal<String>);
        }
    }
}

/// Get thread-safe string signal value
#[no_mangle]
pub extern "C" fn rtui_thread_safe_signal_string_get(
    signal: *const RTuiThreadSafeSignal,
    buffer: *mut c_char,
    buffer_size: usize,
) -> ReactiveError {
    if signal.is_null() || buffer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &*(signal as *const ThreadSafeSignal<String>);
        let value = signal_ref.get();
        
        if value.len() >= buffer_size {
            return Err(ReactiveError::BufferTooSmall);
        }

        let c_string = CString::new(value).map_err(|_| ReactiveError::InvalidUtf8)?;
        let bytes = c_string.as_bytes_with_nul();
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
        Ok(())
    }))
}

/// Set thread-safe string signal value
#[no_mangle]
pub extern "C" fn rtui_thread_safe_signal_string_set(
    signal: *mut RTuiThreadSafeSignal,
    value: *const c_char,
) -> ReactiveError {
    if signal.is_null() || value.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let value_str = CStr::from_ptr(value)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?
            .to_string();

        let signal_ref = &*(signal as *const ThreadSafeSignal<String>);
        signal_ref.set(value_str);
        Ok(())
    }))
}

/// Create an effect
#[no_mangle]
pub extern "C" fn rtui_effect_create(
    callback: RTuiEffectCallback,
    cleanup: RTuiEffectCleanupCallback,
    user_data: *mut std::ffi::c_void,
    out_effect: *mut *mut RTuiEffect,
) -> ReactiveError {
    if out_effect.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        // Create a simplified effect wrapper
        // In practice, this would integrate with the actual hook system
        let effect_data = Box::new((callback, cleanup, user_data));
        unsafe {
            *out_effect = Box::into_raw(effect_data) as *mut RTuiEffect;
        }
        Ok(())
    }))
}

/// Destroy an effect
#[no_mangle]
pub extern "C" fn rtui_effect_destroy(effect: *mut RTuiEffect) {
    if !effect.is_null() {
        unsafe {
            let effect_data = Box::from_raw(effect as *mut (RTuiEffectCallback, RTuiEffectCleanupCallback, *mut std::ffi::c_void));
            // Call cleanup if provided
            if effect_data.1 as *const () != std::ptr::null() {
                (effect_data.1)(effect_data.2);
            }
        }
    }
}

/// Run an effect
#[no_mangle]
pub extern "C" fn rtui_effect_run(effect: *const RTuiEffect) -> ReactiveError {
    if effect.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let effect_data = &*(effect as *const (RTuiEffectCallback, RTuiEffectCleanupCallback, *mut std::ffi::c_void));
        (effect_data.0)(effect_data.2);
        Ok(())
    }))
}
