//! Terminal FFI functions

use super::*;
use crate::core::terminal::Terminal;
use std::boxed::Box;

/// Create a new terminal instance
#[no_mangle]
pub extern "C" fn rtui_terminal_create(out_terminal: *mut *mut ReactiveTerminal) -> ReactiveError {
    if out_terminal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| match Terminal::new() {
        Ok(terminal) => {
            let boxed = Box::new(terminal);
            unsafe {
                *out_terminal = Box::into_raw(boxed) as *mut ReactiveTerminal;
            }
            Ok(())
        }
        Err(e) => Err(e.into()),
    }))
}

/// Destroy a terminal instance
#[no_mangle]
pub extern "C" fn rtui_terminal_destroy(terminal: *mut ReactiveTerminal) {
    if !terminal.is_null() {
        unsafe {
            let _ = Box::from_raw(terminal as *mut Terminal);
        }
    }
}

/// Get terminal dimensions
#[no_mangle]
pub extern "C" fn rtui_terminal_get_dimensions(
    terminal: *const ReactiveTerminal,
    out_dimensions: *mut RTuiDimensions,
) -> ReactiveError {
    if terminal.is_null() || out_dimensions.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Validate pointer alignment and basic sanity
        if (terminal as usize) % std::mem::align_of::<Terminal>() != 0 {
            return Err(ReactiveError::InvalidPointer);
        }
        let term = &*(terminal as *const Terminal);
        match term.size() {
            Ok((width, height)) => {
                *out_dimensions = RTuiDimensions { width, height };
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }))
}

/// Enter modern mode (raw mode + alternate screen + mouse)
#[no_mangle]
pub extern "C" fn rtui_terminal_enter_raw_mode(terminal: *mut ReactiveTerminal) -> ReactiveError {
    if terminal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Validate pointer alignment and basic sanity
        if (terminal as usize) % std::mem::align_of::<Terminal>() != 0 {
            return Err(ReactiveError::InvalidPointer);
        }
        let term = &mut *(terminal as *mut Terminal);
        match term.enter_modern_mode() {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }))
}

/// Exit modern mode
#[no_mangle]
pub extern "C" fn rtui_terminal_exit_raw_mode(terminal: *mut ReactiveTerminal) -> ReactiveError {
    if terminal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Validate pointer alignment and basic sanity
        if (terminal as usize) % std::mem::align_of::<Terminal>() != 0 {
            return Err(ReactiveError::InvalidPointer);
        }
        let term = &mut *(terminal as *mut Terminal);
        match term.exit_modern_mode() {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }))
}

/// Control synchronized updates
/// @param begin: true to begin sync, false to end sync
#[no_mangle]
pub extern "C" fn rtui_terminal_sync(
    terminal: *mut ReactiveTerminal,
    begin: bool,
) -> ReactiveError {
    if terminal.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Validate pointer alignment and basic sanity
        if (terminal as usize) % std::mem::align_of::<Terminal>() != 0 {
            return Err(ReactiveError::InvalidPointer);
        }
        let term = &mut *(terminal as *mut Terminal);
        if begin {
            match term.begin_sync() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.into()),
            }
        } else {
            match term.end_sync() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }
    }))
}

/// Poll for events (static function)
#[no_mangle]
pub extern "C" fn rtui_terminal_poll_event(
    timeout_ms: u32,
    out_event: *mut RTuiEvent,
) -> ReactiveError {
    if out_event.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let timeout = if timeout_ms > 0 {
            Some(timeout_ms as u64)
        } else {
            None
        };

        match Terminal::poll_event(timeout) {
            Ok(Some(event)) => {
                unsafe {
                    *out_event = convert_event_to_ffi(event);
                }
                Ok(())
            }
            Ok(None) => Err(ReactiveError::NotFound),
            Err(e) => Err(e.into()),
        }
    }))
}

// Helper to convert crossterm events to FFI events
fn convert_event_to_ffi(event: crossterm::event::Event) -> RTuiEvent {
    use crossterm::event::{Event, KeyCode, KeyModifiers};

    match event {
        Event::Key(key_event) => {
            let key_code = match key_event.code {
                KeyCode::Char(c) => c as u32,
                KeyCode::Enter => 0x0D,
                KeyCode::Esc => 0x1B,
                KeyCode::Backspace => 0x08,
                KeyCode::Tab => 0x09,
                KeyCode::Left => 0x25,
                KeyCode::Right => 0x27,
                KeyCode::Up => 0x26,
                KeyCode::Down => 0x28,
                _ => 0,
            };

            let mut modifiers = 0u8;
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                modifiers |= 1;
            }
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                modifiers |= 2;
            }
            if key_event.modifiers.contains(KeyModifiers::ALT) {
                modifiers |= 4;
            }

            RTuiEvent {
                event_type: RTuiEventType::Key,
                data: RTuiEventData {
                    key: RTuiKeyEvent {
                        key_code,
                        modifiers,
                    },
                },
            }
        }
        Event::Resize(width, height) => RTuiEvent {
            event_type: RTuiEventType::Resize,
            data: RTuiEventData {
                resize: RTuiResizeEvent { width, height },
            },
        },
        _ => RTuiEvent {
            event_type: RTuiEventType::Key,
            data: RTuiEventData {
                key: RTuiKeyEvent {
                    key_code: 0,
                    modifiers: 0,
                },
            },
        },
    }
}
