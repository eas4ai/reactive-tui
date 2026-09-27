use crate::error::Result;
use ::suprtui::render::{Backend as ByteBackend, WriteStatus};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

/// Byte sink that retains the original I/O error and commits with a flush.
pub(super) struct CheckedOutput<W: Write> {
    writer: Rc<RefCell<W>>,
    frame: Vec<u8>,
    /// The assembled frame waiting for `flush_pending` (PIP-001).
    pending: Vec<u8>,
    error: Option<io::Error>,
    failed: bool,
    before_cells: Vec<u8>,
    after_cells: Vec<u8>,
}

impl<W: Write> CheckedOutput<W> {
    pub(super) fn new(writer: Rc<RefCell<W>>) -> Self {
        Self {
            writer,
            frame: Vec::new(),
            pending: Vec::new(),
            error: None,
            failed: false,
            before_cells: Vec::new(),
            after_cells: Vec::new(),
        }
    }

    /// Attach trusted graphics commands to the next complete cell frame.
    /// The caller forces cell output when only these commands have changed.
    pub(super) fn set_graphics(&mut self, before_cells: Vec<u8>, after_cells: Vec<u8>) {
        self.before_cells = before_cells;
        self.after_cells = after_cells;
    }

    pub(super) fn finish_graphics(&mut self, cleanup: Vec<u8>) -> Result<()> {
        if cleanup.is_empty() {
            return Ok(());
        }
        self.error = None;
        self.set_graphics(cleanup, Vec::new());
        self.begin_frame();
        self.write_bytes(b"\x1b[?2026h\x1b[?2026l");
        if self.end_frame() != WriteStatus::Ok {
            return Err(self
                .take_error()
                .unwrap_or_else(|| io::Error::other("image cleanup failed"))
                .into());
        }
        self.flush_pending().map_err(Into::into)
    }

    pub(super) fn take_error(&mut self) -> Option<io::Error> {
        self.error.take()
    }

    /// Whether a rendered frame waits for `flush_pending`.
    pub(super) fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Write and flush the frame `end_frame` assembled. Called by the
    /// worker after it has replied with the frame's geometry (PIP-001); a
    /// failure is returned to the worker, which reports it with the next
    /// present, sync or shutdown (PIP-002).
    pub(super) fn flush_pending(&mut self) -> io::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let result = {
            let mut writer = self.writer.borrow_mut();
            writer
                .write_all(&self.pending)
                .and_then(|()| writer.flush())
        };
        self.pending.clear();
        result
    }
}

impl<W: Write> ByteBackend for CheckedOutput<W> {
    fn prepare_frame(&mut self) -> WriteStatus {
        WriteStatus::Ok
    }
    fn begin_frame(&mut self) {
        self.frame.clear();
        self.failed = false;
    }
    fn write_bytes(&mut self, data: &[u8]) {
        self.frame.extend_from_slice(data);
    }
    fn write_out(&mut self, data: &[u8]) {
        let mut writer = self.writer.borrow_mut();
        if let Err(error) = writer.write_all(data).and_then(|()| writer.flush()) {
            self.error = Some(error);
        }
    }
    fn fail_frame(&mut self) {
        self.failed = true;
    }
    fn end_frame(&mut self) -> WriteStatus {
        if self.failed || self.error.is_some() {
            return WriteStatus::Failed;
        }
        if self.frame.is_empty() {
            return WriteStatus::Ok;
        }
        // Assemble the bytes now; the write and flush happen in
        // `flush_pending` after the worker has replied (PIP-001).
        self.pending.clear();
        // Establish a frame boundary even after a partial write followed
        // by resize (which replaces the renderer and its byte sink).
        // ESC \ (ST) aborts a partial sequence like CAN would, but CAN
        // prints a visible glyph on Konsole-lineage terminals while a
        // bare ST is a no-op everywhere; ESC [?2026l ends a synchronized
        // update a partial write left open. The frame resets the style
        // itself right after its sync-set, so no reset is added here:
        // RAS-002 allows that one and the one before the sync-reset.
        self.pending.extend_from_slice(b"\x1b\\\x1b[?2026l");
        if self.before_cells.is_empty() && self.after_cells.is_empty() {
            self.pending.extend_from_slice(&self.frame);
            return WriteStatus::Ok;
        }
        // Keep image deletion, cells and new placements in the engine's
        // single synchronized update. Never append after its closing marker.
        let Some(body) = self
            .frame
            .strip_prefix(b"\x1b[?2026h")
            .and_then(|bytes| bytes.strip_suffix(b"\x1b[?2026l"))
        else {
            self.pending.clear();
            self.error = Some(io::Error::new(
                io::ErrorKind::InvalidData,
                "SuprTUI frame has no synchronized-update envelope",
            ));
            return WriteStatus::Failed;
        };
        self.pending.extend_from_slice(b"\x1b[?2026h");
        self.pending.extend_from_slice(&self.before_cells);
        self.pending.extend_from_slice(body);
        self.pending.extend_from_slice(&self.after_cells);
        self.pending.extend_from_slice(b"\x1b[?2026l");
        WriteStatus::Ok
    }
}

/// What a renderer session changes on its writer's terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Session {
    /// An owned writer: no terminal mode changes.
    Writer,
    /// The terminal's screen, while another owner reads its input.
    Screen,
    /// The terminal's screen and its input: mouse reports and bracketed
    /// paste as well (INP-001), and the Kitty keyboard flags when the
    /// terminal reported the protocol (INP-007).
    ScreenAndInput { kitty: bool },
}

/// The Kitty keyboard flags INP-007 names: disambiguate, report all keys as
/// escape codes, report associated text.
fn kitty_flags() -> PushKeyboardEnhancementFlags {
    PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_ASSOCIATED_TEXT,
    )
}

/// Session ownership stays separate from renderer buffers so resize cannot
/// lose restoration state. Drop also covers worker unwinding and disconnects.
pub(super) struct TerminalOutput<W: Write> {
    writer: Rc<RefCell<W>>,
    session: Session,
    active: bool,
}

impl<W: Write> TerminalOutput<W> {
    pub(super) fn new(writer: Rc<RefCell<W>>, session: Session) -> Self {
        Self {
            writer,
            session,
            active: false,
        }
    }

    pub(super) fn enter(&mut self) -> Result<()> {
        if self.session != Session::Writer {
            self.active = true; // Also restore if setup only partially writes.
            let mut writer = self.writer.borrow_mut();
            writer.write_all(b"\x1b[?1049h\x1b[?25l\x1b[?1004h")?;
            if let Session::ScreenAndInput { kitty } = self.session {
                // On Windows the mouse command sets the console input mode instead.
                crossterm::queue!(&mut *writer, EnableMouseCapture, EnableBracketedPaste)?;
                if kitty {
                    crossterm::queue!(&mut *writer, kitty_flags())?;
                }
            }
            writer.flush()?;
        }
        Ok(())
    }

    pub(super) fn restore(&mut self) -> Result<()> {
        if self.active {
            let mut writer = self.writer.borrow_mut();
            // ST instead of CAN here too: CAN paints a glyph on some terminals.
            writer.write_all(b"\x1b\\\x1b[?2026l\x1b[0m\x1b[?1004l")?;
            if let Session::ScreenAndInput { kitty } = self.session {
                if kitty {
                    crossterm::queue!(&mut *writer, PopKeyboardEnhancementFlags)?;
                }
                crossterm::queue!(&mut *writer, DisableBracketedPaste, DisableMouseCapture)?;
            }
            writer.write_all(b"\x1b[0 q\x1b]112\x07\x1b[?25h\x1b[?1049l")?;
            writer.flush()?;
            self.active = false;
        }
        Ok(())
    }

    pub(super) fn restore_with_panic(&mut self, message: Option<&str>) -> Result<()> {
        self.restore()?;
        if let Some(message) = message {
            super::super::write_panic(&mut *self.writer.borrow_mut(), message)?;
        }
        Ok(())
    }
}

impl<W: Write> Drop for TerminalOutput<W> {
    fn drop(&mut self) {
        if let Err(error) = self.restore() {
            log::warn!("Terminal output cleanup failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::suprtui::render::WriteStatus;

    #[test]
    fn frame_boundary_emits_st_not_can() {
        let writer = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut output = CheckedOutput::new(Rc::clone(&writer));
        output.begin_frame();
        output.write_bytes(b"\x1b[?2026hA\x1b[?2026l");
        assert!(matches!(output.end_frame(), WriteStatus::Ok));
        output.flush_pending().unwrap();
        let bytes = writer.borrow();
        assert!(
            !bytes.contains(&0x18),
            "CAN paints a visible glyph on some terminals"
        );
        assert!(bytes.starts_with(b"\x1b\\\x1b[?2026l\x1b[?2026hA"));
    }

    #[test]
    fn restore_emits_no_can_byte() {
        let writer = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut terminal = TerminalOutput::new(Rc::clone(&writer), Session::Screen);
        terminal.enter().unwrap();
        writer.borrow_mut().clear();
        terminal.restore().unwrap();
        assert!(!writer.borrow().contains(&0x18));
    }

    /// INP-001 on Windows, where crossterm's mouse command and raw mode both
    /// set the console input mode, and the mouse command's restore puts back
    /// the raw mode it saved. After a normal shutdown and after a panic
    /// shutdown the backend must leave the mode it found. Needs a console, as
    /// over SSH. The only test in this binary that turns the mouse on: crossterm
    /// keeps the first mode it saves for the whole process.
    #[cfg(windows)]
    #[test]
    fn windows_backend_restores_the_console_input_mode_after_each_exit() {
        use crate::backend::{Backend, SuprTuiBackend};
        use windows_sys::Win32::Foundation::{
            CloseHandle, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE,
        };
        use windows_sys::Win32::Storage::FileSystem::{
            CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
        };
        use windows_sys::Win32::System::Console::{
            GetConsoleMode, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT, ENABLE_QUICK_EDIT_MODE,
        };

        fn console_input_mode() -> u32 {
            let name: Vec<u16> = "CONIN$\0".encode_utf16().collect();
            // SAFETY: `name` is a NUL-terminated UTF-16 string that outlives the call,
            // and the handle is closed before returning.
            unsafe {
                let handle = CreateFileW(
                    name.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    std::ptr::null(),
                    OPEN_EXISTING,
                    0,
                    std::ptr::null_mut(),
                );
                assert_ne!(handle, INVALID_HANDLE_VALUE, "this test needs a console");
                let mut mode = 0;
                let read = GetConsoleMode(handle, &mut mode);
                CloseHandle(handle);
                assert_ne!(read, 0, "the console input mode");
                mode
            }
        }

        let before = console_input_mode();
        for panic in [false, true] {
            let mut backend = SuprTuiBackend::new().expect("the default backend on this console");
            let during = console_input_mode();
            assert!(
                during & ENABLE_MOUSE_INPUT != 0
                    && during & ENABLE_QUICK_EDIT_MODE == 0
                    && during & ENABLE_LINE_INPUT == 0,
                "INP-001: the running backend left the console input mode at {during:#x}"
            );
            if panic {
                backend
                    .shutdown_after_panic("the panic-exit case")
                    .expect("a panic shutdown");
            } else {
                backend.shutdown().expect("a normal shutdown");
            }
            let after = console_input_mode();
            assert_eq!(
                after,
                before,
                "INP-001: after a {} shutdown the console input mode is {after:#x}, not {before:#x}",
                if panic { "panic" } else { "normal" }
            );
        }
    }
}
