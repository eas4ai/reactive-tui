use crate::error::Result;
use ::suprtui::render::{Backend as ByteBackend, WriteStatus};
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
            // ST instead of CAN here too: CAN paints a glyph on some terminals.
            writer.write_all(
                b"\x1b\\\x1b[?2026l\x1b[0m\x1b[?1004l\x1b[0 q\x1b]112\x07\x1b[?25h\x1b[?1049l",
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
        let mut terminal = TerminalOutput::new(Rc::clone(&writer), true);
        terminal.enter().unwrap();
        writer.borrow_mut().clear();
        terminal.restore().unwrap();
        assert!(!writer.borrow().contains(&0x18));
    }
}
