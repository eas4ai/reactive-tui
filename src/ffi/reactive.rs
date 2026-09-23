//! Reactive system FFI functions - seamless reactive bindings for C
//!
//! This module provides a complete reactive system for C integration,
//! including signals, effects, hooks, and component support.

#![allow(unused_imports)]
#![allow(dead_code)]

use super::*;
use crate::component::Element;
use crate::reactive::signal::Signal;
use crate::reactive::ThreadSafeSignal;
use crate::reactive::{use_effect, use_signal, Hooks};
use std::any::{Any, TypeId};
use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::rc::Rc;
use std::sync::Arc;

/// Owning, thread-confined handle to a typed reactive signal.
///
/// Both destructor names consume this handle exactly once. Live handles from
/// either constructor family share the same allocation representation; typed
/// accessors reject mismatched values. Legacy integers are i64, improved integers
/// are c_int. Arbitrary pointers and handles used after destruction are invalid.
#[repr(C)]
pub struct RTuiSignal {
    _private: [u8; 0],
}

/// Opaque handle to a thread-safe signal
#[repr(C)]
pub struct RTuiThreadSafeSignal {
    _private: [u8; 0],
}

/// Opaque handle to a reactive effect
#[repr(C)]
pub struct RTuiEffect {
    _private: [u8; 0],
}

/// Opaque handle to a memo
#[repr(C)]
pub struct RTuiMemo {
    _private: [u8; 0],
}

/// Opaque handle to hooks context
#[repr(C)]
pub struct RTuiHooks {
    _private: [u8; 0],
}

/// Opaque handle to component props
#[repr(C)]
pub struct RTuiProps {
    _private: [u8; 0],
}

/// Signal value types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiSignalValueType {
    /// String signal type
    String = 0,
    /// Integer signal type
    Integer = 1,
    /// Float signal type
    Float = 2,
    /// Boolean signal type
    Boolean = 3,
    /// Custom signal type
    Custom = 4,
}

/// Signal value union
#[repr(C)]
pub union RTuiSignalValue {
    /// String value pointer
    pub string_value: *const c_char,
    /// Integer value
    pub int_value: i64,
    /// Float value
    pub float_value: f64,
    /// Boolean value
    pub bool_value: bool,
    /// Custom type pointer
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

/// Effect cleanup callback function type.
///
/// Pass `Some(callback)` to register cleanup or `None` when no cleanup is needed.
/// A bare function pointer is not a nullable callback:
///
/// ```compile_fail
/// use std::ffi::c_void;
///
/// type BareCleanupCallback = extern "C" fn(*mut c_void);
///
/// fn nullable(
///     callback: BareCleanupCallback,
/// ) -> Option<extern "C" fn(*mut c_void)> {
///     callback
/// }
/// ```
pub type RTuiEffectCleanupCallback = Option<extern "C" fn(user_data: *mut std::ffi::c_void)>;

/// Memo compute callback function type
pub type RTuiMemoComputeCallback =
    extern "C" fn(user_data: *mut std::ffi::c_void) -> RTuiSignalValue;

/// Component function type - takes hooks, props, and user data, returns element
pub type RTuiComponentFn = extern "C" fn(
    hooks: *mut RTuiHooks,
    props: *const RTuiProps,
    user_data: *mut c_void,
) -> *mut super::builder::RTuiElement;

/// Helper function to catch panics and return values with defaults
fn catch_panic_with_default<F, T>(f: F, default: T) -> T
where
    F: FnOnce() -> Result<T, ReactiveError> + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(Ok(value)) => value,
        Ok(Err(_)) => default,
        Err(_) => default,
    }
}

/// Effect function type for hooks
pub type RTuiEffectFn = extern "C" fn(user_data: *mut c_void);

/// Type information for runtime type checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RTuiSignalType {
    /// Integer signal type
    Int,
    /// Legacy 64-bit integer signal type
    Int64,
    /// String signal type
    String,
    /// Boolean signal type
    Bool,
    /// Float signal type
    Float,
}

/// Internal signal wrapper with type information and reference counting
#[derive(Clone)]
struct FFISignal {
    signal: Rc<dyn Any>,
    signal_type: RTuiSignalType,
}

/// Internal hooks wrapper for component integration
struct FFIHooks {
    /// Signal storage for hooks
    signals: RefCell<HashMap<String, Rc<FFISignal>>>,
    /// Effect storage for hooks
    effects: RefCell<Vec<Box<dyn Any>>>,
}

/// Internal props wrapper
struct FFIProps {
    /// Property storage
    props: HashMap<String, String>, // Simplified - would support multiple types
}

impl FFISignal {
    fn new_int(value: c_int) -> Self {
        Self {
            signal: Rc::new(Signal::new(value)),
            signal_type: RTuiSignalType::Int,
        }
    }

    fn new_int64(value: i64) -> Self {
        Self {
            signal: Rc::new(Signal::new(value)),
            signal_type: RTuiSignalType::Int64,
        }
    }

    fn get_int64(&self) -> Result<i64, ReactiveError> {
        self.signal
            .downcast_ref::<Signal<i64>>()
            .map(Signal::get)
            .ok_or(ReactiveError::InvalidParameter)
    }

    fn set_int64(&self, value: i64) -> Result<(), ReactiveError> {
        let signal = self
            .signal
            .downcast_ref::<Signal<i64>>()
            .ok_or(ReactiveError::InvalidParameter)?;
        signal.set(value);
        Ok(())
    }

    fn new_string(value: String) -> Self {
        Self {
            signal: Rc::new(Signal::new(value)),
            signal_type: RTuiSignalType::String,
        }
    }

    fn new_bool(value: bool) -> Self {
        Self {
            signal: Rc::new(Signal::new(value)),
            signal_type: RTuiSignalType::Bool,
        }
    }

    fn new_float(value: f64) -> Self {
        Self {
            signal: Rc::new(Signal::new(value)),
            signal_type: RTuiSignalType::Float,
        }
    }

    fn get_int(&self) -> Result<c_int, ReactiveError> {
        if self.signal_type != RTuiSignalType::Int {
            return Err(ReactiveError::InvalidParameter);
        }

        let signal = self
            .signal
            .downcast_ref::<Signal<c_int>>()
            .ok_or(ReactiveError::InvalidParameter)?;

        Ok(signal.get())
    }

    fn set_int(&self, value: c_int) -> Result<(), ReactiveError> {
        if self.signal_type != RTuiSignalType::Int {
            return Err(ReactiveError::InvalidParameter);
        }

        let signal = self
            .signal
            .downcast_ref::<Signal<c_int>>()
            .ok_or(ReactiveError::InvalidParameter)?;

        signal.set(value);
        Ok(())
    }

    fn get_string(&self) -> Result<String, ReactiveError> {
        if self.signal_type != RTuiSignalType::String {
            return Err(ReactiveError::InvalidParameter);
        }

        let signal = self
            .signal
            .downcast_ref::<Signal<String>>()
            .ok_or(ReactiveError::InvalidParameter)?;

        Ok(signal.get())
    }

    fn set_string(&self, value: &str) -> Result<(), ReactiveError> {
        if self.signal_type != RTuiSignalType::String {
            return Err(ReactiveError::InvalidParameter);
        }

        let signal = self
            .signal
            .downcast_ref::<Signal<String>>()
            .ok_or(ReactiveError::InvalidParameter)?;

        signal.set(value.to_string());
        Ok(())
    }

    fn get_bool(&self) -> Result<bool, ReactiveError> {
        if self.signal_type != RTuiSignalType::Bool {
            return Err(ReactiveError::InvalidParameter);
        }

        let signal = self
            .signal
            .downcast_ref::<Signal<bool>>()
            .ok_or(ReactiveError::InvalidParameter)?;

        Ok(signal.get())
    }

    fn set_bool(&self, value: bool) -> Result<(), ReactiveError> {
        if self.signal_type != RTuiSignalType::Bool {
            return Err(ReactiveError::InvalidParameter);
        }

        let signal = self
            .signal
            .downcast_ref::<Signal<bool>>()
            .ok_or(ReactiveError::InvalidParameter)?;

        signal.set(value);
        Ok(())
    }

    fn get_float(&self) -> Result<f64, ReactiveError> {
        if self.signal_type != RTuiSignalType::Float {
            return Err(ReactiveError::InvalidParameter);
        }

        let signal = self
            .signal
            .downcast_ref::<Signal<f64>>()
            .ok_or(ReactiveError::InvalidParameter)?;

        Ok(signal.get())
    }

    fn set_float(&self, value: f64) -> Result<(), ReactiveError> {
        if self.signal_type != RTuiSignalType::Float {
            return Err(ReactiveError::InvalidParameter);
        }

        let signal = self
            .signal
            .downcast_ref::<Signal<f64>>()
            .ok_or(ReactiveError::InvalidParameter)?;

        signal.set(value);
        Ok(())
    }
}

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

        let signal = FFISignal::new_string(initial_str);
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
        let signal = FFISignal::new_int64(initial_value);
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
        let signal = FFISignal::new_float(initial_value);
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
        let signal = FFISignal::new_bool(initial_value);
        unsafe {
            *out_signal = Box::into_raw(Box::new(signal)) as *mut RTuiSignal;
        }
        Ok(())
    }))
}

/// Destroy a signal
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_signal_destroy(signal: *mut RTuiSignal) {
    rtui_signal_destroy_new(signal);
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
        let signal_ref = &*(signal as *const FFISignal);
        let value = signal_ref.get_string()?;

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

        let signal_ref = &*(signal as *const FFISignal);
        signal_ref.set_string(&value_str)?;
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
        let signal_ref = &*(signal as *const FFISignal);
        *out_value = signal_ref.get_int64()?;
        Ok(())
    }))
}

/// Set integer signal value
#[no_mangle]
pub extern "C" fn rtui_signal_int_set(signal: *mut RTuiSignal, value: i64) -> ReactiveError {
    if signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &*(signal as *const FFISignal);
        signal_ref.set_int64(value)?;
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
        let signal_ref = &*(signal as *const FFISignal);
        *out_value = signal_ref.get_float()?;
        Ok(())
    }))
}

/// Set float signal value
#[no_mangle]
pub extern "C" fn rtui_signal_float_set(signal: *mut RTuiSignal, value: f64) -> ReactiveError {
    if signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &*(signal as *const FFISignal);
        signal_ref.set_float(value)?;
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
        let signal_ref = &*(signal as *const FFISignal);
        *out_value = signal_ref.get_bool()?;
        Ok(())
    }))
}

/// Set boolean signal value
#[no_mangle]
pub extern "C" fn rtui_signal_bool_set(signal: *mut RTuiSignal, value: bool) -> ReactiveError {
    if signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let signal_ref = &*(signal as *const FFISignal);
        signal_ref.set_bool(value)?;
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
#[reactive_tui_macros::ffi_export]
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
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_effect_destroy(effect: *mut RTuiEffect) {
    if !effect.is_null() {
        unsafe {
            let effect_data = Box::from_raw(
                effect
                    as *mut (
                        RTuiEffectCallback,
                        RTuiEffectCleanupCallback,
                        *mut std::ffi::c_void,
                    ),
            );
            if let Some(cleanup) = effect_data.1 {
                cleanup(effect_data.2);
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
        let effect_data = &*(effect
            as *const (
                RTuiEffectCallback,
                RTuiEffectCleanupCallback,
                *mut std::ffi::c_void,
            ));
        (effect_data.0)(effect_data.2);
        Ok(())
    }))
}

// ============================================================================
// SEAMLESS REACTIVE API - New improved functions
// ============================================================================

/// Create a new integer signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_new_int(initial_value: c_int) -> *mut RTuiSignal {
    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let signal = FFISignal::new_int(initial_value);
            let boxed = Box::new(signal);
            Ok(Box::into_raw(boxed) as *mut RTuiSignal)
        }),
        std::ptr::null_mut(),
    )
}

/// Create a new string signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_new_string(initial_value: *const c_char) -> *mut RTuiSignal {
    if initial_value.is_null() {
        return std::ptr::null_mut();
    }

    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let c_str = unsafe { CStr::from_ptr(initial_value) };
            let value = c_str.to_str().map_err(|_| ReactiveError::InvalidUtf8)?;

            let signal = FFISignal::new_string(value.to_string());
            let boxed = Box::new(signal);
            Ok(Box::into_raw(boxed) as *mut RTuiSignal)
        }),
        std::ptr::null_mut(),
    )
}

/// Create a new boolean signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_new_bool(initial_value: bool) -> *mut RTuiSignal {
    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let signal = FFISignal::new_bool(initial_value);
            let boxed = Box::new(signal);
            Ok(Box::into_raw(boxed) as *mut RTuiSignal)
        }),
        std::ptr::null_mut(),
    )
}

/// Create a new float signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_new_float(initial_value: f64) -> *mut RTuiSignal {
    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let signal = FFISignal::new_float(initial_value);
            let boxed = Box::new(signal);
            Ok(Box::into_raw(boxed) as *mut RTuiSignal)
        }),
        std::ptr::null_mut(),
    )
}

/// Get the current value of an integer signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_get_int(signal: *const RTuiSignal) -> c_int {
    if signal.is_null() {
        return 0;
    }

    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let signal_ref = unsafe { &*(signal as *const FFISignal) };
            signal_ref.get_int()
        }),
        0,
    )
}

/// Set the value of an integer signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_set_int(signal: *mut RTuiSignal, value: c_int) -> ReactiveError {
    if signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let signal_ref = unsafe { &*(signal as *const FFISignal) };
        signal_ref.set_int(value)?;
        Ok(())
    }))
}

/// Get the current value of a string signal (improved API - returns owned string)
#[no_mangle]
pub extern "C" fn rtui_signal_get_string_owned(signal: *const RTuiSignal) -> *mut c_char {
    if signal.is_null() {
        return std::ptr::null_mut();
    }

    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let signal_ref = unsafe { &*(signal as *const FFISignal) };
            let value = signal_ref.get_string()?;
            let c_string = CString::new(value).map_err(|_| ReactiveError::InvalidUtf8)?;
            Ok(c_string.into_raw())
        }),
        std::ptr::null_mut(),
    )
}

/// Set the value of a string signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_set_string_new(
    signal: *mut RTuiSignal,
    value: *const c_char,
) -> ReactiveError {
    if signal.is_null() || value.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let signal_ref = unsafe { &*(signal as *const FFISignal) };
        let c_str = unsafe { CStr::from_ptr(value) };
        let value_str = c_str.to_str().map_err(|_| ReactiveError::InvalidUtf8)?;
        signal_ref.set_string(value_str)?;
        Ok(())
    }))
}

/// Get the current value of a boolean signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_get_bool_new(signal: *const RTuiSignal) -> bool {
    if signal.is_null() {
        return false;
    }

    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let signal_ref = unsafe { &*(signal as *const FFISignal) };
            signal_ref.get_bool()
        }),
        false,
    )
}

/// Set the value of a boolean signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_set_bool_new(signal: *mut RTuiSignal, value: bool) -> ReactiveError {
    if signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let signal_ref = unsafe { &*(signal as *const FFISignal) };
        signal_ref.set_bool(value)?;
        Ok(())
    }))
}

/// Get the current value of a float signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_get_float_new(signal: *const RTuiSignal) -> f64 {
    if signal.is_null() {
        return 0.0;
    }

    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let signal_ref = unsafe { &*(signal as *const FFISignal) };
            signal_ref.get_float()
        }),
        0.0,
    )
}

/// Set the value of a float signal (improved API)
#[no_mangle]
pub extern "C" fn rtui_signal_set_float_new(signal: *mut RTuiSignal, value: f64) -> ReactiveError {
    if signal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let signal_ref = unsafe { &*(signal as *const FFISignal) };
        signal_ref.set_float(value)?;
        Ok(())
    }))
}

/// Destroy a signal (improved API with proper type safety)
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_signal_destroy_new(signal: *mut RTuiSignal) {
    if !signal.is_null() {
        unsafe {
            let _ = Box::from_raw(signal as *mut FFISignal);
        }
    }
}

/// Free a string returned by rtui_signal_get_string_owned
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_string_free(string: *mut c_char) {
    if !string.is_null() {
        unsafe {
            let _ = CString::from_raw(string);
        }
    }
}

// ============================================================================
// HOOKS SYSTEM - React-like hooks for C components
// ============================================================================

impl FFIHooks {
    fn new() -> Self {
        Self {
            signals: RefCell::new(HashMap::new()),
            effects: RefCell::new(Vec::new()),
        }
    }

    fn use_signal_int(&self, key: &str, initial: c_int) -> Rc<FFISignal> {
        let mut signals = self.signals.borrow_mut();
        if let Some(signal) = signals.get(key) {
            signal.clone()
        } else {
            let signal = Rc::new(FFISignal::new_int(initial));
            signals.insert(key.to_string(), signal.clone());
            signal
        }
    }

    fn use_signal_string(&self, key: &str, initial: &str) -> Rc<FFISignal> {
        let mut signals = self.signals.borrow_mut();
        if let Some(signal) = signals.get(key) {
            signal.clone()
        } else {
            let signal = Rc::new(FFISignal::new_string(initial.to_string()));
            signals.insert(key.to_string(), signal.clone());
            signal
        }
    }

    fn use_signal_bool(&self, key: &str, initial: bool) -> Rc<FFISignal> {
        let mut signals = self.signals.borrow_mut();
        if let Some(signal) = signals.get(key) {
            signal.clone()
        } else {
            let signal = Rc::new(FFISignal::new_bool(initial));
            signals.insert(key.to_string(), signal.clone());
            signal
        }
    }
}

/// Create a new hooks context
#[no_mangle]
pub extern "C" fn rtui_hooks_new() -> *mut RTuiHooks {
    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let hooks = FFIHooks::new();
            let boxed = Box::new(hooks);
            Ok(Box::into_raw(boxed) as *mut RTuiHooks)
        }),
        std::ptr::null_mut(),
    )
}

/// Destroy a hooks context
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_hooks_destroy(hooks: *mut RTuiHooks) {
    if !hooks.is_null() {
        unsafe {
            let _ = Box::from_raw(hooks as *mut FFIHooks);
        }
    }
}

/// Use an integer signal in a component (React-like hook)
#[no_mangle]
pub extern "C" fn rtui_use_signal_int(
    hooks: *mut RTuiHooks,
    key: *const c_char,
    initial: c_int,
) -> *mut RTuiSignal {
    if hooks.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }

    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let hooks_ref = unsafe { &*(hooks as *const FFIHooks) };
            let c_str = unsafe { CStr::from_ptr(key) };
            let key_str = c_str.to_str().map_err(|_| ReactiveError::InvalidUtf8)?;

            let signal = hooks_ref.use_signal_int(key_str, initial);
            // Return a new reference to the signal
            let boxed = Box::new((*signal).clone());
            Ok(Box::into_raw(boxed) as *mut RTuiSignal)
        }),
        std::ptr::null_mut(),
    )
}

/// Use a string signal in a component (React-like hook)
#[no_mangle]
pub extern "C" fn rtui_use_signal_string(
    hooks: *mut RTuiHooks,
    key: *const c_char,
    initial: *const c_char,
) -> *mut RTuiSignal {
    if hooks.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }

    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let hooks_ref = unsafe { &*(hooks as *const FFIHooks) };
            let c_str = unsafe { CStr::from_ptr(key) };
            let key_str = c_str.to_str().map_err(|_| ReactiveError::InvalidUtf8)?;

            let initial_str = if initial.is_null() {
                ""
            } else {
                let c_str = unsafe { CStr::from_ptr(initial) };
                c_str.to_str().map_err(|_| ReactiveError::InvalidUtf8)?
            };

            let signal = hooks_ref.use_signal_string(key_str, initial_str);
            // Return a new reference to the signal
            let boxed = Box::new((*signal).clone());
            Ok(Box::into_raw(boxed) as *mut RTuiSignal)
        }),
        std::ptr::null_mut(),
    )
}

/// Use a boolean signal in a component (React-like hook)
#[no_mangle]
pub extern "C" fn rtui_use_signal_bool(
    hooks: *mut RTuiHooks,
    key: *const c_char,
    initial: bool,
) -> *mut RTuiSignal {
    if hooks.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }

    catch_panic_with_default(
        AssertUnwindSafe(|| {
            let hooks_ref = unsafe { &*(hooks as *const FFIHooks) };
            let c_str = unsafe { CStr::from_ptr(key) };
            let key_str = c_str.to_str().map_err(|_| ReactiveError::InvalidUtf8)?;

            let signal = hooks_ref.use_signal_bool(key_str, initial);
            // Return a new reference to the signal
            let boxed = Box::new((*signal).clone());
            Ok(Box::into_raw(boxed) as *mut RTuiSignal)
        }),
        std::ptr::null_mut(),
    )
}
