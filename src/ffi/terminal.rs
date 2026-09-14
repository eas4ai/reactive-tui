//! Terminal FFI functions - Modern API
//!
//! Provides terminal control and capabilities detection with modern patterns

use crate::core::capabilities::{TerminalCapabilities, TerminalQuery};
use crate::core::terminal::Terminal;

/// FFI Terminal handle (opaque pointer to Terminal)
#[repr(C)]
pub struct RTuiTerminal {
    _private: [u8; 0],
}

/// Terminal capabilities structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    /// RGB color support
    pub rgb: bool,
    /// 256 color support  
    pub color_256: bool,
    /// Unicode support level (0=basic, 1=extended, 2=full)
    pub unicode_level: u8,
    /// Kitty keyboard protocol support
    pub kitty_keyboard: bool,
    /// Mouse support
    pub mouse: bool,
    /// Pixel mouse support
    pub pixel_mouse: bool,
    /// Hyperlinks support
    pub hyperlinks: bool,
    /// Image support
    pub images: bool,
    /// Synchronized output support
    pub synchronized_output: bool,
    /// Bracketed paste support
    pub bracketed_paste: bool,
}

/// Cursor style enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum CursorStyle {
    /// Block cursor (solid rectangle)
    Block = 0,
    /// Underline cursor (line under character)
    Underline = 1,
    /// Bar cursor (vertical line)
    Bar = 2,
}

//
// TERMINAL MANAGEMENT
//

/// Create terminal instance
#[reactive_tui_macros::ffi_export]
pub extern "C" fn createTerminal() -> *mut RTuiTerminal {
    match Terminal::new() {
        Ok(terminal) => {
            let boxed = Box::new(terminal);
            let raw_ptr = Box::into_raw(boxed);

            // Register the pointer for tracking
            if !super::pointer::trackers::terminal_tracker().register(raw_ptr) {
                unsafe {
                    drop(Box::from_raw(raw_ptr));
                }
                return std::ptr::null_mut();
            }

            raw_ptr as *mut RTuiTerminal
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Destroy terminal instance
#[reactive_tui_macros::ffi_export]
pub extern "C" fn destroyTerminal(terminal: *mut RTuiTerminal) {
    if terminal.is_null() {
        return;
    }

    let terminal_ptr = terminal as *mut Terminal;
    // Remove ownership before releasing the allocation.
    if !super::pointer::trackers::terminal_tracker().unregister(terminal_ptr) {
        return;
    }
    unsafe {
        drop(Box::from_raw(terminal_ptr));
    }
}

/// Setup terminal for TUI mode
#[reactive_tui_macros::ffi_export]
pub extern "C" fn setupTerminal(terminal: *mut RTuiTerminal, use_alternate_screen: bool) {
    if terminal.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Terminal>(terminal as *const u8) {
        return;
    }

    let terminal_ref = unsafe { &mut *(terminal as *mut Terminal) };
    let _ = terminal_ref.init();

    if use_alternate_screen {
        use std::io::{self, Write};
        let _ = io::stdout().write_all(b"\x1b[?1049h");
        let _ = io::stdout().flush();
    }
}

/// Clear terminal
#[reactive_tui_macros::ffi_export]
pub extern "C" fn clearTerminal(terminal: *mut RTuiTerminal) {
    if terminal.is_null() {
        return;
    }

    use std::io::{self, Write};
    let _ = io::stdout().write_all(b"\x1b[2J\x1b[H");
    let _ = io::stdout().flush();
}

/// Get terminal capabilities
#[reactive_tui_macros::ffi_export]
pub extern "C" fn getTerminalCapabilities(
    terminal: *const RTuiTerminal,
    caps_ptr: *mut Capabilities,
) {
    if terminal.is_null() || caps_ptr.is_null() {
        return;
    }

    let terminal_ref = unsafe { &*(terminal as *const Terminal) };
    let caps = terminal_ref.capabilities();

    unsafe {
        *caps_ptr = Capabilities {
            rgb: caps.color_depth.supports(16777216), // 24-bit color
            color_256: caps.color_depth.supports(256),
            unicode_level: if caps.unicode { 2 } else { 1 },
            kitty_keyboard: caps.enhanced_keyboard,
            mouse: true, // Assume mouse support
            pixel_mouse: caps.pixel_mouse,
            hyperlinks: false, // Not exposed in TerminalCapabilities yet
            images: caps.kitty_graphics || caps.sixel || caps.iterm2_graphics,
            synchronized_output: caps.synchronized_output,
            bracketed_paste: false, // Not exposed in TerminalCapabilities yet
        };
    }
}

/// Process capability response
#[reactive_tui_macros::ffi_export]
pub extern "C" fn processCapabilityResponse(
    terminal: *mut RTuiTerminal,
    response_ptr: *const u8,
    response_len: usize,
) {
    if terminal.is_null() || response_ptr.is_null() {
        return;
    }

    let response = unsafe { std::slice::from_raw_parts(response_ptr, response_len) };

    // Use the proper capability parsing from TerminalQuery
    let query = TerminalQuery::new();
    let mut caps = TerminalCapabilities::default();

    // Parse the response buffer using the existing capability parser
    query.parse_response_buffer(response, &mut caps);

    // Apply environment fallbacks to fill in any gaps
    query.apply_env_fallbacks(&mut caps);

    // Store the updated capabilities in the terminal
    // The Terminal struct currently initializes capabilities once during creation
    // Capability updates require extending Terminal with a set_capabilities method
}

//
// CURSOR CONTROL
//

/// Set cursor position
#[reactive_tui_macros::ffi_export]
pub extern "C" fn setCursorPosition(terminal: *mut RTuiTerminal, x: i32, y: i32, visible: bool) {
    if terminal.is_null() {
        return;
    }

    use std::io::{self, Write};

    let cursor_x = std::cmp::max(1, x) as u16;
    let cursor_y = std::cmp::max(1, y) as u16;

    // Set cursor position (1-based coordinates)
    let _ = write!(io::stdout(), "\x1b[{};{}H", cursor_y, cursor_x);

    // Set cursor visibility
    if visible {
        let _ = io::stdout().write_all(b"\x1b[?25h");
    } else {
        let _ = io::stdout().write_all(b"\x1b[?25l");
    }
    let _ = io::stdout().flush();
}

/// Set cursor style
#[reactive_tui_macros::ffi_export]
pub extern "C" fn setCursorStyle(
    terminal: *mut RTuiTerminal,
    style_ptr: *const u8,
    style_len: usize,
    blinking: bool,
) {
    if terminal.is_null() || style_ptr.is_null() {
        return;
    }

    let style_slice = unsafe { std::slice::from_raw_parts(style_ptr, style_len) };
    let style_str = match std::str::from_utf8(style_slice) {
        Ok(s) => s,
        Err(_) => return,
    };

    use std::io::{self, Write};

    let cursor_style = match style_str {
        "block" => CursorStyle::Block,
        "underline" => CursorStyle::Underline,
        "bar" => CursorStyle::Bar,
        _ => CursorStyle::Block,
    };

    // Apply cursor style
    let style_code = match cursor_style {
        CursorStyle::Block => {
            if blinking {
                "1"
            } else {
                "2"
            }
        }
        CursorStyle::Underline => {
            if blinking {
                "3"
            } else {
                "4"
            }
        }
        CursorStyle::Bar => {
            if blinking {
                "5"
            } else {
                "6"
            }
        }
    };

    let _ = write!(io::stdout(), "\x1b[{} q", style_code);
    let _ = io::stdout().flush();
}

/// Set cursor color
#[reactive_tui_macros::ffi_export]
pub extern "C" fn setCursorColor(terminal: *mut RTuiTerminal, color: *const f32) {
    if terminal.is_null() || color.is_null() {
        return;
    }

    use std::io::{self, Write};

    let rgba = super::lib::f32_ptr_to_rgba(color);

    // Set cursor color using OSC 12 sequence
    let r = (rgba.r * 255.0) as u8;
    let g = (rgba.g * 255.0) as u8;
    let b = (rgba.b * 255.0) as u8;

    let _ = write!(io::stdout(), "\x1b]12;#{:02x}{:02x}{:02x}\x1b\\", r, g, b);
    let _ = io::stdout().flush();
}

/// Set terminal title
#[reactive_tui_macros::ffi_export]
pub extern "C" fn setTerminalTitle(
    terminal: *mut RTuiTerminal,
    title_ptr: *const u8,
    title_len: usize,
) {
    if terminal.is_null() || title_ptr.is_null() {
        return;
    }

    let title_slice = unsafe { std::slice::from_raw_parts(title_ptr, title_len) };
    use std::io::{self, Write};

    let title_str = match std::str::from_utf8(title_slice) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Set terminal title using OSC 0 sequence
    let _ = write!(io::stdout(), "\x1b]0;{}\x1b\\", title_str);
    let _ = io::stdout().flush();
}

//
// MOUSE AND KEYBOARD
//

/// Enable mouse support
#[reactive_tui_macros::ffi_export]
pub extern "C" fn enableMouse(terminal: *mut RTuiTerminal, enable_movement: bool) {
    if terminal.is_null() {
        return;
    }

    use std::io::{self, Write};

    // Enable mouse reporting
    let _ = io::stdout().write_all(b"\x1b[?1000h"); // Basic mouse reporting
    if enable_movement {
        let _ = io::stdout().write_all(b"\x1b[?1003h"); // Mouse movement tracking
    }
    let _ = io::stdout().flush();
}

/// Disable mouse support
#[reactive_tui_macros::ffi_export]
pub extern "C" fn disableMouse(terminal: *mut RTuiTerminal) {
    if terminal.is_null() {
        return;
    }

    use std::io::{self, Write};

    // Disable mouse reporting
    let _ = io::stdout().write_all(b"\x1b[?1000l"); // Disable basic mouse
    let _ = io::stdout().write_all(b"\x1b[?1003l"); // Disable movement tracking
    let _ = io::stdout().flush();
}

/// Enable Kitty keyboard protocol
#[reactive_tui_macros::ffi_export]
pub extern "C" fn enableKittyKeyboard(terminal: *mut RTuiTerminal, flags: u8) {
    if terminal.is_null() {
        return;
    }

    use std::io::{self, Write};

    // Enable Kitty keyboard protocol
    let _ = write!(io::stdout(), "\x1b[>{};1u", flags);
    let _ = io::stdout().flush();
}

/// Disable Kitty keyboard protocol
#[reactive_tui_macros::ffi_export]
pub extern "C" fn disableKittyKeyboard(terminal: *mut RTuiTerminal) {
    if terminal.is_null() {
        return;
    }

    use std::io::{self, Write};

    // Disable Kitty keyboard protocol
    let _ = io::stdout().write_all(b"\x1b[<1u");
    let _ = io::stdout().flush();
}
