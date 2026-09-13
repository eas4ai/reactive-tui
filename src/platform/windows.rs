//! Windows Console API implementation for direct terminal access
//!
//! Native Windows Console API integration for terminal control

use super::PlatformTty;
use crate::error::Result;
use std::collections::VecDeque;
use std::os::windows::io::{AsRawHandle, BorrowedHandle, OwnedHandle, RawHandle};
use std::sync::{Mutex, TryLockError};
use std::time::Duration;

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::*,
    Storage::FileSystem::*,
    System::{Console::*, Threading::WaitForSingleObject},
    UI::Input::KeyboardAndMouse::*,
};

#[derive(Default)]
struct InputState {
    bytes: VecDeque<u8>,
    byte_surrogate: Option<u16>,
    press_surrogate: Option<u16>,
    release_surrogate: Option<u16>,
}

// Console character records contain UTF-16 code units, including surrogate pairs.
fn decode_character(unit: u16, pending: &mut Option<u16>) -> Option<char> {
    if (0xD800..=0xDBFF).contains(&unit) {
        *pending = Some(unit);
        return None;
    }
    if (0xDC00..=0xDFFF).contains(&unit) {
        return Some(
            pending
                .take()
                .and_then(|high| {
                    char::from_u32(
                        0x10000 + ((u32::from(high) - 0xD800) << 10) + (u32::from(unit) - 0xDC00),
                    )
                })
                .unwrap_or(char::REPLACEMENT_CHARACTER),
        );
    }
    *pending = None;
    char::from_u32(u32::from(unit))
}

/// Windows TTY implementation using Console API
#[cfg(windows)]
pub struct WindowsTty {
    /// Handle to console input
    stdin_handle: OwnedHandle,
    /// Handle to console output  
    stdout_handle: OwnedHandle,
    /// Original input mode to restore
    original_input_mode: u32,
    /// Original output mode to restore
    original_output_mode: u32,
    input: Mutex<InputState>,
}

#[cfg(not(windows))]
pub struct WindowsTty;

#[cfg(windows)]
impl PlatformTty for WindowsTty {
    fn write(&self, data: &[u8]) -> Result<usize> {
        WindowsTty::write(self, data)
    }

    fn size(&self) -> Result<(u16, u16)> {
        WindowsTty::size(self)
    }

    fn restore(&self) -> Result<()> {
        WindowsTty::restore(self)
    }
}

#[cfg(windows)]
impl WindowsTty {
    /// Initialize Windows console for direct access
    pub fn init() -> Result<Self> {
        unsafe {
            let stdin_handle = GetStdHandle(STD_INPUT_HANDLE);
            let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);

            if stdin_handle.is_null()
                || stdout_handle.is_null()
                || stdin_handle == INVALID_HANDLE_VALUE
                || stdout_handle == INVALID_HANDLE_VALUE
            {
                return Err(std::io::Error::last_os_error().into());
            }

            // Duplicate the borrowed standard handles. OwnedHandle supplies the
            // Send/Sync and close-on-drop guarantees without unsafe trait impls.
            let stdin_owner = BorrowedHandle::borrow_raw(stdin_handle).try_clone_to_owned()?;
            let stdout_owner = BorrowedHandle::borrow_raw(stdout_handle).try_clone_to_owned()?;

            // Get original console modes
            let mut original_input_mode = 0;
            let mut original_output_mode = 0;

            if GetConsoleMode(stdin_handle, &mut original_input_mode) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            if GetConsoleMode(stdout_handle, &mut original_output_mode) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            // Consume INPUT_RECORD values, not the VT stream produced for ReadFile.
            // Extended flags also disable Quick Edit so selection cannot pause input.
            let new_input_mode = ENABLE_WINDOW_INPUT | ENABLE_MOUSE_INPUT | ENABLE_EXTENDED_FLAGS;

            if SetConsoleMode(stdin_handle, new_input_mode) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            // Set virtual terminal output mode
            let new_output_mode = original_output_mode
                | ENABLE_PROCESSED_OUTPUT
                | ENABLE_VIRTUAL_TERMINAL_PROCESSING
                | DISABLE_NEWLINE_AUTO_RETURN;

            if SetConsoleMode(stdout_handle, new_output_mode) == 0 {
                // Restore input mode on failure
                SetConsoleMode(stdin_handle, original_input_mode);
                return Err(std::io::Error::last_os_error().into());
            }

            Ok(WindowsTty {
                stdin_handle: stdin_owner,
                stdout_handle: stdout_owner,
                original_input_mode,
                original_output_mode,
                input: Mutex::new(InputState::default()),
            })
        }
    }

    /// Write raw bytes to console output
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        unsafe {
            let mut bytes_written = 0;
            let result = WriteFile(
                self.stdout_handle.as_raw_handle(),
                data.as_ptr(),
                data.len().min(u32::MAX as usize) as u32,
                &mut bytes_written,
                std::ptr::null_mut(),
            );

            if result == 0 {
                Err(std::io::Error::last_os_error().into())
            } else {
                Ok(bytes_written as usize)
            }
        }
    }

    /// Read console input records
    pub fn read_input_record(&self, timeout: Option<Duration>) -> Result<INPUT_RECORD> {
        unsafe {
            // Check if input is available
            if let Some(timeout) = timeout {
                let milliseconds =
                    timeout.as_millis() + u128::from(timeout.subsec_nanos() % 1_000_000 != 0);
                let result = WaitForSingleObject(
                    self.stdin_handle.as_raw_handle(),
                    milliseconds.min(u128::from(u32::MAX - 1)) as u32,
                );
                match result {
                    WAIT_OBJECT_0 => {} // Input available
                    WAIT_TIMEOUT => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "Read timeout",
                        )
                        .into())
                    }
                    _ => return Err(std::io::Error::last_os_error().into()),
                }
            }

            let mut input_record = std::mem::MaybeUninit::<INPUT_RECORD>::uninit();
            let mut events_read = 0;

            let result = ReadConsoleInputW(
                self.stdin_handle.as_raw_handle(),
                input_record.as_mut_ptr(),
                1,
                &mut events_read,
            );

            if result == 0 {
                Err(std::io::Error::last_os_error().into())
            } else if events_read == 0 {
                Err(
                    std::io::Error::new(std::io::ErrorKind::WouldBlock, "No input available")
                        .into(),
                )
            } else {
                Ok(input_record.assume_init())
            }
        }
    }

    /// Read UTF-8 bytes from console character records, retaining partial output.
    /// Use one input reader for a console; competing byte reads return WouldBlock.
    pub fn read(&self, buffer: &mut [u8], timeout: Option<Duration>) -> Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let mut input = self.input.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "another console input reader is active",
            ),
            TryLockError::Poisoned(_) => std::io::Error::other("console input lock poisoned"),
        })?;
        let start = std::time::Instant::now();
        let mut first = true;
        while input.bytes.is_empty() {
            if !first && timeout.is_some_and(|limit| start.elapsed() >= limit) {
                return Err(
                    std::io::Error::new(std::io::ErrorKind::TimedOut, "Read timeout").into(),
                );
            }
            first = false;
            let record =
                self.read_input_record(timeout.map(|limit| limit.saturating_sub(start.elapsed())))?;
            if u32::from(record.EventType) != KEY_EVENT {
                continue;
            }
            // EventType selects this union member; the OS filled the record.
            let key = unsafe { record.Event.KeyEvent };
            let unit = unsafe { key.uChar.UnicodeChar };
            if key.bKeyDown == 0 || unit == 0 {
                continue;
            }
            if let Some(character) = decode_character(unit, &mut input.byte_surrogate) {
                let mut bytes = [0; 4];
                let text = character.encode_utf8(&mut bytes);
                for _ in 0..key.wRepeatCount.max(1) {
                    input.bytes.extend(text.as_bytes());
                }
            }
        }
        let count = buffer.len().min(input.bytes.len());
        for destination in &mut buffer[..count] {
            *destination = input
                .bytes
                .pop_front()
                .expect("count is bounded by buffered bytes");
        }
        Ok(count)
    }

    /// Read multiple input events with timeout
    pub fn read_input_events(
        &self,
        timeout: Option<Duration>,
    ) -> Result<Vec<crate::platform::TerminalEvent>> {
        let mut events = Vec::new();
        let start_time = std::time::Instant::now();

        loop {
            let remaining_timeout = if let Some(timeout) = timeout {
                let elapsed = start_time.elapsed();
                if elapsed >= timeout {
                    break;
                }
                Some(timeout - elapsed)
            } else {
                None
            };

            match self.read_input_record(remaining_timeout) {
                Ok(record) => {
                    if let Some(event) = self.parse_input_record(&record) {
                        events.push(event);
                    }
                    // Continue reading if there might be more events
                    if events.len() >= 10 {
                        break; // Limit batch size
                    }
                }
                Err(e) => {
                    if matches!(&e, crate::error::ReactiveError::Io(error)
                        if matches!(error.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock))
                    {
                        break; // Normal timeout
                    } else {
                        return Err(e); // Real error
                    }
                }
            }
        }

        Ok(events)
    }

    /// Parse a Windows INPUT_RECORD into a TerminalEvent
    pub fn parse_input_record(
        &self,
        record: &INPUT_RECORD,
    ) -> Option<crate::platform::TerminalEvent> {
        unsafe {
            match u32::from(record.EventType) {
                KEY_EVENT => {
                    let key_event = &record.Event.KeyEvent;
                    self.parse_key_event(key_event)
                }
                MOUSE_EVENT => {
                    let mouse_event = &record.Event.MouseEvent;
                    self.parse_mouse_event(mouse_event)
                }
                WINDOW_BUFFER_SIZE_EVENT => {
                    let resize_event = &record.Event.WindowBufferSizeEvent;
                    Some(crate::platform::TerminalEvent::Resize {
                        width: resize_event.dwSize.X as u16,
                        height: resize_event.dwSize.Y as u16,
                    })
                }
                FOCUS_EVENT => {
                    let focus_event = &record.Event.FocusEvent;
                    if focus_event.bSetFocus != 0 {
                        Some(crate::platform::TerminalEvent::FocusGained)
                    } else {
                        Some(crate::platform::TerminalEvent::FocusLost)
                    }
                }
                _ => None,
            }
        }
    }

    /// Parse Windows key event
    fn parse_key_event(
        &self,
        key_event: &KEY_EVENT_RECORD,
    ) -> Option<crate::platform::TerminalEvent> {
        use crate::platform::{KeyCode, KeyEventKind, KeyModifiers};

        let vk_code = key_event.wVirtualKeyCode;
        let char_code = unsafe { key_event.uChar.UnicodeChar };
        let control_key_state = key_event.dwControlKeyState;

        // Map virtual key code to KeyCode
        let code = match vk_code {
            VK_BACK => KeyCode::Backspace,
            VK_TAB => KeyCode::Tab,
            VK_RETURN => KeyCode::Enter,
            VK_ESCAPE => KeyCode::Escape,
            VK_SPACE => KeyCode::Char(' '),
            VK_PRIOR => KeyCode::PageUp,
            VK_NEXT => KeyCode::PageDown,
            VK_END => KeyCode::End,
            VK_HOME => KeyCode::Home,
            VK_LEFT => KeyCode::Left,
            VK_UP => KeyCode::Up,
            VK_RIGHT => KeyCode::Right,
            VK_DOWN => KeyCode::Down,
            VK_INSERT => KeyCode::Insert,
            VK_DELETE => KeyCode::Delete,
            VK_F1..=VK_F24 => KeyCode::F((vk_code - VK_F1 + 1) as u8),
            _ => {
                if char_code == 0 || char_code == 0xFFFF {
                    KeyCode::Unknown
                } else {
                    let mut input = self.input.lock().expect("console input lock poisoned");
                    let pending = if key_event.bKeyDown == 0 {
                        &mut input.release_surrogate
                    } else {
                        &mut input.press_surrogate
                    };
                    KeyCode::Char(decode_character(char_code, pending)?)
                }
            }
        };

        // Map modifiers
        let modifiers = KeyModifiers {
            shift: (control_key_state & SHIFT_PRESSED) != 0,
            ctrl: (control_key_state & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0,
            alt: (control_key_state & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED)) != 0,
            meta: false, // Windows doesn't have a meta key
        };

        Some(crate::platform::TerminalEvent::Key {
            code,
            modifiers,
            kind: if key_event.bKeyDown == 0 {
                KeyEventKind::Release
            } else if key_event.wRepeatCount > 1 {
                KeyEventKind::Repeat
            } else {
                KeyEventKind::Press
            },
        })
    }

    /// Parse Windows mouse event
    fn parse_mouse_event(
        &self,
        mouse_event: &MOUSE_EVENT_RECORD,
    ) -> Option<crate::platform::TerminalEvent> {
        use crate::platform::{KeyModifiers, MouseButton, MouseEventKind};

        let column = mouse_event.dwMousePosition.X as u16;
        let row = mouse_event.dwMousePosition.Y as u16;
        let button_state = mouse_event.dwButtonState;
        let control_key_state = mouse_event.dwControlKeyState;
        let event_flags = mouse_event.dwEventFlags;

        let modifiers = KeyModifiers {
            shift: (control_key_state & SHIFT_PRESSED) != 0,
            ctrl: (control_key_state & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0,
            alt: (control_key_state & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED)) != 0,
            meta: false,
        };

        let kind = match event_flags {
            MOUSE_MOVED if button_state & 0xFFFF != 0 => MouseEventKind::Drag,
            MOUSE_MOVED => MouseEventKind::Move,
            MOUSE_WHEELED => {
                let wheel_delta = (button_state >> 16) as i16;
                if wheel_delta > 0 {
                    MouseEventKind::ScrollUp
                } else {
                    MouseEventKind::ScrollDown
                }
            }
            MOUSE_HWHEELED => {
                let wheel_delta = (button_state >> 16) as i16;
                if wheel_delta > 0 {
                    MouseEventKind::ScrollRight
                } else {
                    MouseEventKind::ScrollLeft
                }
            }
            DOUBLE_CLICK => MouseEventKind::Down, // Treat as regular click
            _ => {
                // Regular button event
                if button_state & 0xFFFF != 0 {
                    MouseEventKind::Down
                } else {
                    MouseEventKind::Up
                }
            }
        };
        let button = if button_state & 0x0001 != 0 {
            Some(MouseButton::Left)
        } else if button_state & 0x0002 != 0 {
            Some(MouseButton::Right)
        } else if button_state & 0x0004 != 0 {
            Some(MouseButton::Middle)
        } else {
            None
        };

        Some(crate::platform::TerminalEvent::Mouse {
            kind,
            button,
            column,
            row,
            pixel_x: None,
            pixel_y: None,
            modifiers,
        })
    }

    /// Get console screen buffer size
    pub fn size(&self) -> Result<(u16, u16)> {
        unsafe {
            let mut csbi = std::mem::MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFO>::uninit();
            let result =
                GetConsoleScreenBufferInfo(self.stdout_handle.as_raw_handle(), csbi.as_mut_ptr());

            if result == 0 {
                Err(std::io::Error::last_os_error().into())
            } else {
                let csbi = csbi.assume_init();
                let width = (csbi.srWindow.Right - csbi.srWindow.Left + 1) as u16;
                let height = (csbi.srWindow.Bottom - csbi.srWindow.Top + 1) as u16;
                Ok((width, height))
            }
        }
    }

    /// Restore original console modes
    pub fn restore(&self) -> Result<()> {
        unsafe {
            let input_result =
                SetConsoleMode(self.stdin_handle.as_raw_handle(), self.original_input_mode);
            let output_result = SetConsoleMode(
                self.stdout_handle.as_raw_handle(),
                self.original_output_mode,
            );

            if input_result == 0 || output_result == 0 {
                Err(std::io::Error::last_os_error().into())
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(windows)]
impl Drop for WindowsTty {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[cfg(windows)]
impl AsRawHandle for WindowsTty {
    fn as_raw_handle(&self) -> RawHandle {
        self.stdout_handle.as_raw_handle()
    }
}

// Stub implementation for non-Windows platforms
#[cfg(not(windows))]
impl WindowsTty {
    pub fn init() -> Result<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows TTY not supported on this platform",
        )
        .into())
    }

    pub fn write(&self, _data: &[u8]) -> Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows TTY not supported on this platform",
        )
        .into())
    }

    pub fn size(&self) -> Result<(u16, u16)> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows TTY not supported on this platform",
        )
        .into())
    }

    pub fn restore(&self) -> Result<()> {
        Ok(())
    }
}
