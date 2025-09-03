//! Foreign Function Interface for reactive-tui
//!
//! Provides a stable C ABI for using reactive-tui from other languages.
//! Zero overhead for Rust users - this module is only for FFI bindings.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::{self, AssertUnwindSafe};

mod animation;
mod app;
mod builder;
mod component;
mod dialog;
mod error;
// mod pointer; // Disabled due to thread safety issues
// mod render; // Disabled due to pointer dependency
// mod surface; // Disabled due to pointer dependency
mod terminal;
mod types;
mod widgets;

pub use animation::*;
pub use app::*;
pub use builder::*;
pub use component::*;
pub use dialog::*;
pub use error::*;
// pub use pointer::*; // Disabled due to thread safety issues
// pub use render::*; // Disabled due to pointer dependency
// pub use surface::*; // Disabled due to pointer dependency
pub use terminal::*;
pub use types::*;
pub use widgets::*;

/// Version information for ABI compatibility
#[repr(C)]
pub struct RTuiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub abi_version: u32,
}

/// Get the library version
#[no_mangle]
pub extern "C" fn rtui_version() -> RTuiVersion {
    RTuiVersion {
        major: 0,
        minor: 1,
        patch: 0,
        abi_version: 1,
    }
}

/// Helper function to catch panics and convert them to error codes
pub(crate) fn catch_panic<F, T>(f: F) -> ReactiveError
where
    F: FnOnce() -> Result<T, ReactiveError> + std::panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(Ok(_)) => ReactiveError::Success,
        Ok(Err(e)) => e,
        Err(_) => ReactiveError::Panic,
    }
}

/// Initialize the library (must be called before any other functions)
#[no_mangle]
pub extern "C" fn rtui_init() -> ReactiveError {
    // Initialize any global state if needed
    ReactiveError::Success
}

/// Cleanup the library
#[no_mangle]
pub extern "C" fn rtui_cleanup() {
    // Clear all pointer trackers to prevent stale references
    // Note: pointer tracking cleanup would go here if implemented
    // pointer::clear_all();
}

/// Helper to convert C string to Rust string
unsafe fn c_str_to_string(s: *const c_char) -> Result<String, ReactiveError> {
    if s.is_null() {
        return Err(ReactiveError::NullPointer);
    }

    unsafe {
        CStr::from_ptr(s)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| ReactiveError::InvalidUtf8)
    }
}
