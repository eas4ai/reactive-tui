//! Windows Console API implementation for direct terminal access
//!
//! Native Windows Console API integration for terminal control

use super::PlatformTty;
use crate::error::Result;
use std::os::windows::io::{AsRawHandle, RawHandle};
use std::time::Duration;

#[cfg(windows)]
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*, System::Console::*};

#[cfg(windows)]
const FROM_LEFT_1ST_BUTTON_PRESSED: DWORD = 0x0001;
#[cfg(windows)]
const RIGHTMOST_BUTTON_PRESSED: DWORD = 0x0002;
#[cfg(windows)]
const MOUSE_MOVED: DWORD = 0x0001;
#[cfg(windows)]
const MOUSE_WHEELED: DWORD = 0x0004;

/// Windows TTY implementation using Console API
#[cfg(windows)]
pub struct WindowsTty {
    /// Handle to console input
    stdin_handle: HANDLE,
    /// Handle to console output  
    stdout_handle: HANDLE,
    /// Original input mode to restore
    original_input_mode: DWORD,
    /// Original output mode to restore
    original_output_mode: DWORD,
}

#[cfg(not(windows))]
pub struct WindowsTty;

#[cfg(windows)]
impl PlatformTty for WindowsTty {
    fn write(&self, data: &[u8]) -> Result<usize> {
        self.write_bytes(data)
    }

    fn size(&self) -> Result<(u16, u16)> {
        self.get_size()
    }

    fn restore(&self) -> Result<()> {
        self.restore_modes()
    }
}

#[cfg(windows)]
impl WindowsTty {
    /// Initialize Windows console for direct access
    pub fn init() -> Result<Self> {
        unsafe {
            let stdin_handle = GetStdHandle(STD_INPUT_HANDLE);
            let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);

            if stdin_handle == INVALID_HANDLE_VALUE || stdout_handle == INVALID_HANDLE_VALUE {
                return Err(std::io::Error::last_os_error().into());
            }

            // Get original console modes
            let mut original_input_mode = 0;
            let mut original_output_mode = 0;

            if GetConsoleMode(stdin_handle, &mut original_input_mode) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            if GetConsoleMode(stdout_handle, &mut original_output_mode) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            // Set raw input mode
            let new_input_mode =
                ENABLE_VIRTUAL_TERMINAL_INPUT | ENABLE_WINDOW_INPUT | ENABLE_MOUSE_INPUT;

            if SetConsoleMode(stdin_handle, new_input_mode) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            // Set virtual terminal output mode
            let new_output_mode = original_output_mode
                | ENABLE_VIRTUAL_TERMINAL_PROCESSING
                | DISABLE_NEWLINE_AUTO_RETURN;

            if SetConsoleMode(stdout_handle, new_output_mode) == 0 {
                // Restore input mode on failure
                SetConsoleMode(stdin_handle, original_input_mode);
                return Err(std::io::Error::last_os_error().into());
            }

            Ok(WindowsTty {
                stdin_handle,
                stdout_handle,
                original_input_mode,
                original_output_mode,
            })
        }
    }

    /// Write raw bytes to console output
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        unsafe {
            let mut bytes_written = 0;
            let result = WriteFile(
                self.stdout_handle,
                data.as_ptr() as *const std::ffi::c_void,
                data.len() as u32,
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
                let result = WaitForSingleObject(self.stdin_handle, timeout.as_millis() as u32);
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
                self.stdin_handle,
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
                    if e.to_string().contains("timeout") || e.to_string().contains("WouldBlock") {
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
    fn parse_input_record(&self, record: &INPUT_RECORD) -> Option<crate::platform::TerminalEvent> {
        unsafe {
            match record.EventType {
                KEY_EVENT => {
                    let key_event = record.Event.KeyEvent();
                    self.parse_key_event(key_event)
                }
                MOUSE_EVENT => {
                    let mouse_event = record.Event.MouseEvent();
                    self.parse_mouse_event(mouse_event)
                }
                WINDOW_BUFFER_SIZE_EVENT => {
                    let resize_event = record.Event.WindowBufferSizeEvent();
                    Some(crate::platform::TerminalEvent::Resize {
                        width: resize_event.dwSize.X as u16,
                        height: resize_event.dwSize.Y as u16,
                    })
                }
                FOCUS_EVENT => {
                    let focus_event = record.Event.FocusEvent();
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

        // Only process key down events for now
        if key_event.bKeyDown == 0 {
            return None;
        }

        let vk_code = key_event.wVirtualKeyCode;
        let char_code = unsafe { key_event.uChar.UnicodeChar() };
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
            VK_F1 => KeyCode::F(1),
            VK_F2 => KeyCode::F(2),
            VK_F3 => KeyCode::F(3),
            VK_F4 => KeyCode::F(4),
            VK_F5 => KeyCode::F(5),
            VK_F6 => KeyCode::F(6),
            VK_F7 => KeyCode::F(7),
            VK_F8 => KeyCode::F(8),
            VK_F9 => KeyCode::F(9),
            VK_F10 => KeyCode::F(10),
            VK_F11 => KeyCode::F(11),
            VK_F12 => KeyCode::F(12),
            _ => {
                // Use character if available
                if char_code != 0 && char_code != 0xFFFF {
                    if let Some(ch) = char::from_u32(char_code as u32) {
                        KeyCode::Char(ch)
                    } else {
                        KeyCode::Unknown
                    }
                } else {
                    KeyCode::Unknown
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
            kind: KeyEventKind::Press,
        })
    }

    /// Parse Windows mouse event
    fn parse_mouse_event(
        &self,
        mouse_event: &MOUSE_EVENT_RECORD,
    ) -> Option<crate::platform::TerminalEvent> {
        use crate::platform::{KeyModifiers, MouseEventKind};

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
                if (button_state & FROM_LEFT_1ST_BUTTON_PRESSED) != 0 {
                    MouseEventKind::Down
                } else {
                    MouseEventKind::Up
                }
            }
        };

        Some(crate::platform::TerminalEvent::Mouse {
            kind,
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
            let result = GetConsoleScreenBufferInfo(self.stdout_handle, csbi.as_mut_ptr());

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
            let input_result = SetConsoleMode(self.stdin_handle, self.original_input_mode);
            let output_result = SetConsoleMode(self.stdout_handle, self.original_output_mode);

            if input_result == 0 || output_result == 0 {
                Err(std::io::Error::last_os_error().into())
            } else {
                Ok(())
            }
        }
    }

    /// Convert Windows input record to our event format
    pub fn parse_input_record(
        &self,
        record: &INPUT_RECORD,
    ) -> Option<crate::platform::TerminalEvent> {
        unsafe {
            match record.EventType {
                KEY_EVENT => {
                    let key_event = record.Event.KeyEvent;
                    if key_event.bKeyDown != 0 {
                        // Key press
                        let code = self.map_virtual_key_code(key_event.wVirtualKeyCode);
                        let modifiers = self.map_control_key_state(key_event.dwControlKeyState);

                        Some(crate::platform::TerminalEvent::Key {
                            code,
                            modifiers,
                            kind: crate::platform::KeyEventKind::Press,
                        })
                    } else {
                        // Key release
                        let code = self.map_virtual_key_code(key_event.wVirtualKeyCode);
                        let modifiers = self.map_control_key_state(key_event.dwControlKeyState);

                        Some(crate::platform::TerminalEvent::Key {
                            code,
                            modifiers,
                            kind: crate::platform::KeyEventKind::Release,
                        })
                    }
                }

                MOUSE_EVENT => {
                    let mouse_event = record.Event.MouseEvent;

                    // Parse mouse event based on button state and event flags
                    let x = mouse_event.dwMousePosition.X as u16;
                    let y = mouse_event.dwMousePosition.Y as u16;

                    // Check for button events
                    if mouse_event.dwEventFlags == 0 {
                        // Button press/release event
                        if mouse_event.dwButtonState & FROM_LEFT_1ST_BUTTON_PRESSED != 0 {
                            Some(crate::event::Event::Mouse(crate::event::MouseEvent::Press {
                                button: crate::event::MouseButton::Left,
                                position: (x, y),
                                modifiers: parse_control_key_state(mouse_event.dwControlKeyState),
                            }))
                        } else if mouse_event.dwButtonState & RIGHTMOST_BUTTON_PRESSED != 0 {
                            Some(crate::event::Event::Mouse(crate::event::MouseEvent::Press {
                                button: crate::event::MouseButton::Right,
                                position: (x, y),
                                modifiers: parse_control_key_state(mouse_event.dwControlKeyState),
                            }))
                        } else {
                            // Button release
                            Some(crate::event::Event::Mouse(crate::event::MouseEvent::Release {
                                button: crate::event::MouseButton::Left, // Default to left
                                position: (x, y),
                                modifiers: parse_control_key_state(mouse_event.dwControlKeyState),
                            }))
                        }
                    } else if mouse_event.dwEventFlags & MOUSE_MOVED != 0 {
                        // Mouse move event
                        Some(crate::event::Event::Mouse(crate::event::MouseEvent::Move {
                            position: (x, y),
                            modifiers: parse_control_key_state(mouse_event.dwControlKeyState),
                        }))
                    } else if mouse_event.dwEventFlags & MOUSE_WHEELED != 0 {
                        // Mouse wheel event
                        let delta = ((mouse_event.dwButtonState >> 16) as i16) as i32;
                        let direction = if delta > 0 {
                            crate::event::ScrollDirection::Up
                        } else {
                            crate::event::ScrollDirection::Down
                        };
                        Some(crate::event::Event::Mouse(crate::event::MouseEvent::Scroll {
                            direction,
                            position: (x, y),
                            modifiers: parse_control_key_state(mouse_event.dwControlKeyState),
                        }))
                    } else {
                        None
                    }
                }

                WINDOW_BUFFER_SIZE_EVENT => {
                    let size_event = record.Event.WindowBufferSizeEvent;
                    Some(crate::platform::TerminalEvent::Resize {
                        width: size_event.dwSize.X as u16,
                        height: size_event.dwSize.Y as u16,
                    })
                }

                FOCUS_EVENT => {
                    let focus_event = record.Event.FocusEvent;
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

    fn map_virtual_key_code(&self, vk: u16) -> crate::platform::KeyCode {
        use crate::platform::KeyCode;

        match vk {
            VK_BACK => KeyCode::Backspace,
            VK_RETURN => KeyCode::Enter,
            VK_LEFT => KeyCode::Left,
            VK_RIGHT => KeyCode::Right,
            VK_UP => KeyCode::Up,
            VK_DOWN => KeyCode::Down,
            VK_HOME => KeyCode::Home,
            VK_END => KeyCode::End,
            VK_PRIOR => KeyCode::PageUp,
            VK_NEXT => KeyCode::PageDown,
            VK_TAB => KeyCode::Tab,
            VK_DELETE => KeyCode::Delete,
            VK_INSERT => KeyCode::Insert,
            VK_ESCAPE => KeyCode::Escape,
            VK_F1..=VK_F24 => KeyCode::F((vk - VK_F1 + 1) as u8),
            _ => {
                // Try to get character representation
                if vk >= 0x20 && vk <= 0x7E {
                    KeyCode::Char(vk as u8 as char)
                } else {
                    KeyCode::Unknown
                }
            }
        }
    }

    fn map_control_key_state(&self, state: u32) -> crate::platform::KeyModifiers {
        crate::platform::KeyModifiers {
            shift: (state & SHIFT_PRESSED) != 0,
            ctrl: (state & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0,
            alt: (state & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED)) != 0,
            meta: false, // Windows doesn't have a meta key
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
        self.stdout_handle as RawHandle
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

#[cfg(windows)]
fn parse_control_key_state(state: DWORD) -> crate::event::KeyModifiers {
    let mut modifiers = crate::event::KeyModifiers::empty();

    if state & LEFT_CTRL_PRESSED != 0 || state & RIGHT_CTRL_PRESSED != 0 {
        modifiers |= crate::event::KeyModifiers::CONTROL;
    }
    if state & LEFT_ALT_PRESSED != 0 || state & RIGHT_ALT_PRESSED != 0 {
        modifiers |= crate::event::KeyModifiers::ALT;
    }
    if state & SHIFT_PRESSED != 0 {
        modifiers |= crate::event::KeyModifiers::SHIFT;
    }

    modifiers
}
