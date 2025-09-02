//! FFI type definitions

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
    pub ch: u32, // Unicode codepoint
    pub fg: RTuiColor,
    pub bg: RTuiColor,
    pub attrs: RTuiTextAttributes,
}

/// Event type
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiEventType {
    Key = 0,
    Mouse = 1,
    Resize = 2,
    Focus = 3,
    Paste = 4,
}

/// Key event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiKeyEvent {
    pub key_code: u32,
    pub modifiers: u8, // Bit flags: 1=Shift, 2=Ctrl, 4=Alt, 8=Meta
}

/// Mouse event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiMouseEvent {
    pub x: u16,
    pub y: u16,
    pub button: u8,
    pub modifiers: u8,
    pub event_type: u8, // 0=Press, 1=Release, 2=Move, 3=ScrollUp, 4=ScrollDown
}

/// Resize event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiResizeEvent {
    pub width: u16,
    pub height: u16,
}

/// Event union
#[repr(C)]
pub union RTuiEventData {
    pub key: RTuiKeyEvent,
    pub mouse: RTuiMouseEvent,
    pub resize: RTuiResizeEvent,
}

/// Event structure
#[repr(C)]
pub struct RTuiEvent {
    pub event_type: RTuiEventType,
    pub data: RTuiEventData,
}

/// Callback function types
pub type RTuiEventCallback = extern "C" fn(event: *const RTuiEvent, user_data: *mut c_void);
pub type RTuiRenderCallback = extern "C" fn(surface: *mut RTuiSurface, user_data: *mut c_void);
