//! FFI type definitions

use crate::ffi::error::ReactiveError;
use std::os::raw::c_void;

/// Opaque handle to a terminal
#[repr(C)]
pub struct ReactiveTerminal {
    _private: [u8; 0],
}

/// Opaque handle to a surface
#[repr(C)]
pub struct RTuiSurface {
    _private: [u8; 0],
}

/// Opaque handle to a renderer
#[repr(C)]
pub struct RTuiRenderer {
    _private: [u8; 0],
}

/// RGB color
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiColor {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
}

/// Terminal dimensions
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiDimensions {
    /// Width in characters
    pub width: u16,
    /// Height in characters
    pub height: u16,
}

/// Position in terminal
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiPosition {
    /// X coordinate (column)
    pub x: u16,
    /// Y coordinate (row)
    pub y: u16,
}

/// Rectangle area
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiRect {
    /// X coordinate of top-left corner
    pub x: u16,
    /// Y coordinate of top-left corner
    pub y: u16,
    /// Width of the rectangle
    pub width: u16,
    /// Height of the rectangle
    pub height: u16,
}

/// Text attributes flags
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiTextAttributes {
    /// Bold text
    pub bold: bool,
    /// Italic text
    pub italic: bool,
    /// Underlined text
    pub underline: bool,
    /// Strikethrough text
    pub strikethrough: bool,
    /// Reverse video
    pub reverse: bool,
    /// Blinking text
    pub blink: bool,
    /// Hidden text
    pub hidden: bool,
}

/// Cell in the terminal
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiCell {
    /// Unicode codepoint
    pub ch: u32,
    /// Foreground color
    pub fg: RTuiColor,
    /// Background color
    pub bg: RTuiColor,
    /// Text attributes
    pub attrs: RTuiTextAttributes,
}

/// Event type
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiEventType {
    /// Keyboard event
    Key = 0,
    /// Mouse event
    Mouse = 1,
    /// Terminal resize event
    Resize = 2,
    /// Focus change event
    Focus = 3,
    /// Paste event
    Paste = 4,
}

/// Key event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiKeyEvent {
    /// Key code
    pub key_code: u32,
    /// Modifier keys (bit flags: 1=Shift, 2=Ctrl, 4=Alt, 8=Meta)
    pub modifiers: u8,
}

/// Mouse event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiMouseEvent {
    /// X coordinate
    pub x: u16,
    /// Y coordinate
    pub y: u16,
    /// Mouse button
    pub button: u8,
    /// Modifier keys
    pub modifiers: u8,
    /// Event type (0=Press, 1=Release, 2=Move, 3=ScrollUp, 4=ScrollDown)
    pub event_type: u8,
}

/// Resize event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiResizeEvent {
    /// New width
    pub width: u16,
    /// New height
    pub height: u16,
}

/// Event union
#[repr(C)]
pub union RTuiEventData {
    /// Key event data
    pub key: RTuiKeyEvent,
    /// Mouse event data
    pub mouse: RTuiMouseEvent,
    /// Resize event data
    pub resize: RTuiResizeEvent,
}

/// Event structure
#[repr(C)]
pub struct RTuiEvent {
    /// Type of event
    pub event_type: RTuiEventType,
    /// Event data
    pub data: RTuiEventData,
}

/// Callback function types
pub type RTuiEventCallback = extern "C" fn(event: *const RTuiEvent, user_data: *mut c_void);
/// Render callback function type
pub type RTuiRenderCallback = extern "C" fn(surface: *mut RTuiSurface, user_data: *mut c_void);

/// Validate that a pointer is not null and properly aligned
pub(crate) fn validate_pointer<T>(ptr: *const T) -> bool {
    !ptr.is_null() && (ptr as usize) % std::mem::align_of::<T>() == 0
}

/// Validate that a mutable pointer is not null and properly aligned
pub(crate) fn validate_mut_pointer<T>(ptr: *mut T) -> bool {
    !ptr.is_null() && (ptr as usize) % std::mem::align_of::<T>() == 0
}

/// Safe cast helper for opaque pointers
pub(crate) unsafe fn safe_cast<T, U>(ptr: *mut T) -> Result<*mut U, ReactiveError> {
    if ptr.is_null() {
        return Err(ReactiveError::NullPointer);
    }

    // Basic alignment check
    if (ptr as usize) % std::mem::align_of::<U>() != 0 {
        return Err(ReactiveError::InvalidPointer);
    }

    Ok(ptr as *mut U)
}

/// Safe const cast helper for opaque pointers
pub(crate) unsafe fn safe_cast_const<T, U>(ptr: *const T) -> Result<*const U, ReactiveError> {
    if ptr.is_null() {
        return Err(ReactiveError::NullPointer);
    }

    // Basic alignment check
    if (ptr as usize) % std::mem::align_of::<U>() != 0 {
        return Err(ReactiveError::InvalidPointer);
    }

    Ok(ptr as *const U)
}

// Cleanup function is in mod.rs to avoid duplication
