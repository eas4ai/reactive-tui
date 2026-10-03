//! Foreign Function Interface for reactive-tui
//!
//! Modern FFI providing backend APIs for state and rendering.
//! Follows modern patterns for function naming, parameter passing, and memory management.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::{self, AssertUnwindSafe};

pub(crate) trait FfiPanicDefault {
    fn ffi_panic_default() -> Self;
}

pub(crate) fn ffi_panic_default<T: FfiPanicDefault>() -> T {
    T::ffi_panic_default()
}

macro_rules! impl_ffi_panic_default {
    ($($type:ty => $value:expr),+ $(,)?) => {
        $(
            impl FfiPanicDefault for $type {
                fn ffi_panic_default() -> Self {
                    $value
                }
            }
        )+
    };
}

impl_ffi_panic_default!(
    () => (),
    bool => false,
    u8 => 0,
    u16 => 0,
    u32 => 0,
    u64 => 0,
    usize => 0,
    i8 => 0,
    i16 => 0,
    i32 => 0,
    i64 => 0,
    isize => 0,
    f32 => 0.0,
    f64 => 0.0,
);

impl<T> FfiPanicDefault for *const T {
    fn ffi_panic_default() -> Self {
        std::ptr::null()
    }
}

impl<T> FfiPanicDefault for *mut T {
    fn ffi_panic_default() -> Self {
        std::ptr::null_mut()
    }
}

// Core modern FFI modules
mod lib; // Main FFI interface
mod stats;
mod terminal; // Terminal control
mod terminal_legacy;
mod text; // Text buffer operations // Performance monitoring and debugging

mod animation;
mod app;
mod builder;
mod component;
mod controller;
mod dialog;
mod editor;
mod error;
mod foreign;
mod layout;
mod pointer;
mod reactive;
mod render;
mod surface;
mod types;
mod widgets;

// Export modern FFI API as primary interface
pub use lib::{
    bufferClear,
    bufferDrawText,
    bufferFillRect,
    bufferGetAttributesPtr,
    bufferGetBgPtr,
    bufferGetCharPtr,
    bufferGetFgPtr,
    bufferGetRespectAlpha,
    bufferReleaseAttrPtr,
    bufferReleaseBgPtr,
    bufferReleaseCharPtr,
    bufferReleaseFgPtr,
    bufferResize,
    bufferSetCellWithAlphaBlending,
    bufferSetRespectAlpha,
    createOptimizedBuffer,
    createRenderer,
    destroyOptimizedBuffer,
    destroyRenderer,
    getBufferHeight,
    getBufferWidth,
    render,
    // System integration functions
    renderSurfaceToTerminal,
    renderTextToTerminal,
    renderWithStats,
    resizeRenderer,
    setBackgroundColor,
};
pub use stats::{
    addToHitGrid, checkHit, dumpBuffers, dumpHitGrid, dumpStdoutBuffer, getFrameStats,
    resetPerformanceCounters, setDebugOverlay, setLogCallback, setRenderOffset, startProfiling,
    stopProfiling, updateMemoryStats, updateStats, DebugOverlayCorner, LogCallback, LogLevel,
};
pub use terminal::{
    clearTerminal, createTerminal, destroyTerminal, disableKittyKeyboard, disableMouse,
    enableKittyKeyboard, enableMouse, getTerminalCapabilities, processCapabilityResponse,
    setCursorColor, setCursorPosition, setCursorStyle, setTerminalTitle, setupTerminal,
    Capabilities, CursorStyle,
};
pub use text::{
    createTextBuffer, destroyTextBuffer, renderTextBufferDirect, renderTextBufferToRenderer,
    renderTextBufferToSurface, textBufferGetCapacity, textBufferGetCharPtr, textBufferGetLength,
    textBufferGetSelectionInfo, textBufferReset, textBufferResetDefaults, textBufferResetSelection,
    textBufferResize, textBufferSetDefaultAttributes, textBufferSetDefaultBg,
    textBufferSetDefaultFg, textBufferSetSelection, textBufferWriteChunk, LineInfo, RTuiTextBuffer,
};

pub use self::reactive::*;
pub use animation::*;
pub use app::*;
pub use builder::*;
pub use component::*;
pub use dialog::*;
pub use editor::*;
pub use error::*;
pub use foreign::*;
pub use layout::*;
pub use pointer::*;
pub use render::*;
pub use surface::*;
pub use terminal::*;
pub use terminal_legacy::*;
pub use types::*;
pub use widgets::*;

/// Version information for ABI compatibility
#[repr(C)]
pub struct RTuiVersion {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Patch version number
    pub patch: u32,
    /// ABI version for compatibility checking
    pub abi_version: u32,
}

impl FfiPanicDefault for RTuiVersion {
    fn ffi_panic_default() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            abi_version: 0,
        }
    }
}

impl FfiPanicDefault for ReactiveError {
    fn ffi_panic_default() -> Self {
        Self::Panic
    }
}

/// Get the library version
#[reactive_tui_macros::ffi_export]
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
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_init() -> ReactiveError {
    // Initialize any global state if needed
    ReactiveError::Success
}

/// Cleanup the library
#[reactive_tui_macros::ffi_export]
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
