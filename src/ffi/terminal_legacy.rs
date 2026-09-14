//! Compatibility entry points declared by the C terminal and event headers.
//!
//! Callers must supply valid writable output pointers and serialize access to
//! each terminal handle, including destruction. The input stream has one reader.

use super::*;
use crate::core::terminal::Terminal;
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};

/// Create a terminal handle without entering raw mode.
#[no_mangle]
pub extern "C" fn rtui_terminal_create(out_terminal: *mut *mut ReactiveTerminal) -> ReactiveError {
    if out_terminal.is_null() {
        return ReactiveError::NullPointer;
    }
    catch_panic(AssertUnwindSafe(|| {
        unsafe {
            *out_terminal = std::ptr::null_mut();
        }
        let terminal = Terminal::new().map_err(|_| ReactiveError::TerminalNotAvailable)?;
        let raw = Box::into_raw(Box::new(terminal));
        if !pointer::trackers::terminal_tracker().register(raw) {
            unsafe {
                drop(Box::from_raw(raw));
            }
            return Err(ReactiveError::InternalError);
        }
        unsafe {
            *out_terminal = raw.cast();
        }
        Ok(())
    }))
}

/// Release a terminal handle. Null and unregistered handles are ignored.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_terminal_destroy(terminal: *mut ReactiveTerminal) {
    destroyTerminal(terminal.cast());
}

/// Read the host terminal size. No synthetic dimensions are returned on error.
#[no_mangle]
pub extern "C" fn rtui_terminal_get_dimensions(
    terminal: *const ReactiveTerminal,
    out_dimensions: *mut RTuiDimensions,
) -> ReactiveError {
    if terminal.is_null() || out_dimensions.is_null() {
        return ReactiveError::NullPointer;
    }
    catch_panic(AssertUnwindSafe(|| {
        let raw = terminal.cast::<Terminal>();
        if !pointer::trackers::terminal_tracker().is_valid(raw) {
            return Err(ReactiveError::InvalidPointer);
        }
        let (width, height) = unsafe { &*raw }
            .size()
            .map_err(|_| ReactiveError::TerminalNotAvailable)?;
        unsafe {
            *out_dimensions = RTuiDimensions { width, height };
        }
        Ok(())
    }))
}

/// Begin or end a synchronized terminal update.
#[no_mangle]
pub extern "C" fn rtui_terminal_sync(
    terminal: *mut ReactiveTerminal,
    begin: bool,
) -> ReactiveError {
    if terminal.is_null() {
        return ReactiveError::NullPointer;
    }
    catch_panic(AssertUnwindSafe(|| {
        let raw = terminal.cast::<Terminal>();
        if !pointer::trackers::terminal_tracker().is_valid(raw) {
            return Err(ReactiveError::InvalidPointer);
        }
        let terminal = unsafe { &mut *raw };
        let result = if begin {
            terminal.begin_sync()
        } else {
            terminal.end_sync()
        };
        result.map_err(|_| ReactiveError::TerminalNotAvailable)
    }))
}

fn modifiers(keys: KeyModifiers) -> u8 {
    u8::from(keys.contains(KeyModifiers::SHIFT))
        | (u8::from(keys.contains(KeyModifiers::CONTROL)) << 1)
        | (u8::from(keys.contains(KeyModifiers::ALT)) << 2)
        | (u8::from(keys.contains(KeyModifiers::SUPER)) << 3)
}

fn convert_event(event: Event) -> Result<RTuiEvent, ReactiveError> {
    let (event_type, data) = match event {
        Event::Key(key) => {
            // The legacy ABI has no key-kind field or named special-key table.
            if key.kind == crossterm::event::KeyEventKind::Release {
                return Err(ReactiveError::NotSupported);
            }
            let key_code = match key.code {
                KeyCode::Char(ch) => ch as u32,
                KeyCode::Enter => 13,
                KeyCode::Tab => 9,
                KeyCode::Esc => 27,
                KeyCode::Backspace => 8,
                _ => return Err(ReactiveError::NotSupported),
            };
            (
                RTuiEventType::Key,
                RTuiEventData {
                    key: RTuiKeyEvent {
                        key_code,
                        modifiers: modifiers(key.modifiers),
                    },
                },
            )
        }
        Event::Resize(width, height) => (
            RTuiEventType::Resize,
            RTuiEventData {
                resize: RTuiResizeEvent { width, height },
            },
        ),
        Event::Mouse(mouse) => {
            let (event_type, button) = match mouse.kind {
                MouseEventKind::Down(button) => (0, Some(button)),
                MouseEventKind::Up(button) => (1, Some(button)),
                MouseEventKind::Drag(button) => (2, Some(button)),
                MouseEventKind::Moved => (2, None),
                MouseEventKind::ScrollUp => (3, None),
                MouseEventKind::ScrollDown => (4, None),
                _ => return Err(ReactiveError::NotSupported),
            };
            let button = match button {
                Some(MouseButton::Left) => 1,
                Some(MouseButton::Middle) => 2,
                Some(MouseButton::Right) => 3,
                None => 0,
            };
            (
                RTuiEventType::Mouse,
                RTuiEventData {
                    mouse: RTuiMouseEvent {
                        x: mouse.column,
                        y: mouse.row,
                        button,
                        modifiers: modifiers(mouse.modifiers),
                        event_type,
                    },
                },
            )
        }
        // The legacy union cannot store paste text or a focus-state flag.
        Event::FocusGained | Event::FocusLost | Event::Paste(_) => {
            return Err(ReactiveError::NotSupported)
        }
    };
    Ok(RTuiEvent { event_type, data })
}

/// Poll once; zero is nonblocking. Unsupported payloads leave output unchanged.
#[no_mangle]
pub extern "C" fn rtui_terminal_poll_event(
    timeout_ms: u32,
    out_event: *mut RTuiEvent,
) -> ReactiveError {
    if out_event.is_null() {
        return ReactiveError::NullPointer;
    }
    catch_panic(AssertUnwindSafe(|| {
        let event = Terminal::poll_event(Some(u64::from(timeout_ms)))
            .map_err(|_| ReactiveError::TerminalNotAvailable)?
            .ok_or(ReactiveError::NotFound)?;
        let event = convert_event(event)?;
        unsafe {
            *out_event = event;
        }
        Ok(())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_translation_preserves_unicode_and_modifier_bits() {
        let event = convert_event(Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char('界'),
            KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
        )))
        .unwrap();
        assert!(matches!(event.event_type, RTuiEventType::Key));
        let key = unsafe { event.data.key };
        assert_eq!(key.key_code, '界' as u32);
        assert_eq!(key.modifiers, 15);
    }

    #[test]
    fn event_translation_preserves_resize_and_mouse_data() {
        let event = convert_event(Event::Resize(120, 40)).unwrap();
        assert!(matches!(event.event_type, RTuiEventType::Resize));
        let size = unsafe { event.data.resize };
        assert_eq!((size.width, size.height), (120, 40));
        let event = convert_event(Event::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: 7,
            row: 9,
            modifiers: KeyModifiers::CONTROL,
        }))
        .unwrap();
        assert!(matches!(event.event_type, RTuiEventType::Mouse));
        let mouse = unsafe { event.data.mouse };
        assert_eq!(
            (
                mouse.x,
                mouse.y,
                mouse.button,
                mouse.modifiers,
                mouse.event_type
            ),
            (7, 9, 3, 2, 0)
        );
    }

    #[test]
    fn unsupported_event_payloads_return_an_error() {
        for event in [
            Event::FocusGained,
            Event::Paste("text".into()),
            Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::F(1),
                KeyModifiers::empty(),
            )),
        ] {
            assert!(matches!(
                convert_event(event),
                Err(ReactiveError::NotSupported)
            ));
        }
    }
}
