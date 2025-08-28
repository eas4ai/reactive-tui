//! FFI type definitions

use std::os::raw::c_void;

/// Opaque handle to a terminal
#[repr(C)]
pub struct RTuiTerminal {
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
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Terminal dimensions
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiDimensions {
    pub width: u16,
    pub height: u16,
}

/// Position in terminal
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiPosition {
    pub x: u16,
    pub y: u16,
}

/// Rectangle area
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Text attributes flags
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiTextAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub blink: bool,
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