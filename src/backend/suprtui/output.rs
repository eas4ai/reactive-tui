use crate::error::Result;
use ::suprtui::render::{Backend as ByteBackend, WriteStatus};
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

/// Byte sink that retains the original I/O error and commits with a flush.
pub(super) struct CheckedOutput<W: Write> {
    writer: Rc<RefCell<W>>,
    frame: Vec<u8>,
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
        Ok(())
    }

    pub(super) fn take_error(&mut self) -> Option<io::Error> {
        self.error.take()
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
        let result = (|| {
            let mut writer = self.writer.borrow_mut();
            // Establish a frame boundary even after a partial write followed
            // by resize (which replaces the renderer and its byte sink).
            writer.write_all(b"\x18\x1b[?2026l\x1b[0m")?;
            if self.before_cells.is_empty() && self.after_cells.is_empty() {
                writer.write_all(&self.frame)?;
            } else {
                // Keep image deletion, cells and new placements in the engine's
                // single synchronized update. Never append after its closing marker.
                let body = self
                    .frame
                    .strip_prefix(b"\x1b[?2026h")
                    .and_then(|bytes| bytes.strip_suffix(b"\x1b[?2026l"))
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "SuprTUI frame has no synchronized-update envelope",
                        )
                    })?;
                writer.write_all(b"\x1b[?2026h")?;
                writer.write_all(&self.before_cells)?;
                writer.write_all(body)?;
                writer.write_all(&self.after_cells)?;
                writer.write_all(b"\x1b[?2026l")?;
            }
            writer.flush()
        })();
        match result {
            Ok(()) => WriteStatus::Ok,
            Err(error) => {
                self.error = Some(error);
                WriteStatus::Failed
            }
        }
    }
}

/// Session ownership stays separate from renderer buffers so resize cannot
/// lose restoration state. Drop also covers worker unwinding and disconnects.
pub(super) struct TerminalOutput<W: Write> {
    writer: Rc<RefCell<W>>,
    terminal: bool,
    active: bool,
}

impl<W: Write> TerminalOutput<W> {
    pub(super) fn new(writer: Rc<RefCell<W>>, terminal: bool) -> Self {
        Self {
            writer,
            terminal,
            active: false,
        }
    }

    pub(super) fn enter(&mut self) -> Result<()> {
        if self.terminal {
            self.active = true; // Also restore if setup only partially writes.
            let mut writer = self.writer.borrow_mut();
            writer.write_all(b"\x1b[?1049h\x1b[?25l\x1b[?1004h")?;
            writer.flush()?;
        }
        Ok(())
    }

    pub(super) fn restore(&mut self) -> Result<()> {
        if self.active {
            let mut writer = self.writer.borrow_mut();
            writer.write_all(
                b"\x18\x1b[?2026l\x1b[0m\x1b[?1004l\x1b[0 q\x1b]112\x07\x1b[?25h\x1b[?1049l",
            )?;
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
