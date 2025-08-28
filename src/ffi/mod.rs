//! Foreign Function Interface for reactive-tui
//!
//! Provides a stable C ABI for using reactive-tui from other languages.
//! Zero overhead for Rust users - this module is only for FFI bindings.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::{self, AssertUnwindSafe};

mod error;
mod render;
mod surface;
mod terminal;
mod types;

pub use error::*;
pub use render::*;
pub use surface::*;
pub use terminal::*;
pub use types::*;

/// Version information for ABI compatibility
#[repr(C)]
pub struct RTuiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub abi_version: u32,
}

/// Get the library version
#[unsafe(no_mangle)]
pub extern "C" fn rtui_version() -> RTuiVersion {
    RTuiVersion {
        major: 0,
        minor: 1,
        patch: 0,
        abi_version: 1,
    }
}

/// Initialize the library (must be called before any other functions)
#[unsafe(no_mangle)]
pub extern "C" fn rtui_init() -> RTuiError {
    // Initialize any global state if needed
    RTuiError::Success
}

/// Cleanup the library
#[unsafe(no_mangle)]
pub extern "C" fn rtui_cleanup() {
    // Cleanup any global state
}

/// Helper to convert C string to Rust string
unsafe fn c_str_to_string(s: *const c_char) -> Result<String, RTuiError> {
    if s.is_null() {
        return Err(RTuiError::NullPointer);
    }

    unsafe {
        CStr::from_ptr(s)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| RTuiError::InvalidUtf8)
    }
}

/// Helper to catch panics and convert to error codes
fn catch_panic<F, T>(f: F) -> RTuiError
where
    F: FnOnce() -> Result<T, RTuiError> + panic::UnwindSafe,
    T: Default,
{
    match panic::catch_unwind(f) {
        Ok(Ok(_)) => RTuiError::Success,
        Ok(Err(e)) => e,
        Err(_) => RTuiError::Panic,
    }
}
