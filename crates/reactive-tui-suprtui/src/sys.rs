//! System foundations (SYS-001, SYS-002, SYS-004 … SYS-008).
//!
//! Ports `event-bus.zig`, `logger.zig`, `split-scrollback.zig`, and
//! the `native-span-feed.zig` delivery contract. Everything is
//! caller-owned: sinks, loggers, feeds, and pools live in the
//! caller's structs, never in process globals (SYS-001). SYS-003
//! (clipboard lifecycle) lives in the follow-on `sys-clipboard`
//! commitment; SYS-004's pool itself is [`crate::link`].

use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// SYS-002: ordered event bus.
// ---------------------------------------------------------------------------

/// Event delivery callback: name plus payload bytes.
pub type EventCallback = dyn FnMut(&str, &[u8]);

/// A destroyable event sink. Dropping the callback stops delivery;
/// the struct itself stays usable (emits become no-ops).
pub struct EventSink {
    callback: Option<Box<EventCallback>>,
}

impl EventSink {
    /// Create a live sink that passes each event to `callback`.
    pub fn new(callback: impl FnMut(&str, &[u8]) + 'static) -> Self {
        Self {
            callback: Some(Box::new(callback)),
        }
    }

    /// Destroy delivery. Later emits to this sink do nothing.
    pub fn destroy(&mut self) {
        self.callback = None;
    }

    /// Return whether the sink still has a callback.
    pub fn is_alive(&self) -> bool {
        self.callback.is_some()
    }
}

/// Deliver one event in call order. A missing sink or callback is a
/// silent no-op, never an error.
pub fn emit(sink: Option<&mut EventSink>, name: &str, data: &[u8]) {
    if let Some(sink) = sink
        && let Some(callback) = sink.callback.as_mut()
    {
        callback(name, data);
    }
}

// ---------------------------------------------------------------------------
// SYS-007: level-gated logging through a caller-supplied sink.
// ---------------------------------------------------------------------------

/// Severity gate. Higher variants are more verbose; a message is
/// delivered only when its level is at or below the logger's level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LogLevel {
    /// Errors only. The default level.
    #[default]
    Err = 0,
    /// Warnings and errors.
    Warn = 1,
    /// Informational messages, warnings, and errors.
    Info = 2,
    /// Every message, including debug output.
    Debug = 3,
}

/// Log delivery callback: level plus message text.
pub type LogCallback = dyn FnMut(LogLevel, &str);

/// Caller-owned logger. Without a sink every message drops silently
/// without panicking.
pub struct Logger {
    level: LogLevel,
    sink: Option<Box<LogCallback>>,
}

impl Logger {
    /// Create a logger at `level` with no sink.
    pub fn new(level: LogLevel) -> Self {
        Self { level, sink: None }
    }

    /// Set the most verbose level the logger delivers.
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    /// Return the current level.
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Install the callback that receives delivered messages. It replaces any earlier sink.
    pub fn set_sink(&mut self, sink: impl FnMut(LogLevel, &str) + 'static) {
        self.sink = Some(Box::new(sink));
    }

    /// Remove the sink. Later messages drop silently.
    pub fn clear_sink(&mut self) {
        self.sink = None;
    }

    /// Send `message` to the sink when `level` is at or below the logger's level.
    pub fn log(&mut self, level: LogLevel, message: &str) {
        if level > self.level {
            return;
        }
        if let Some(sink) = self.sink.as_mut() {
            sink(level, message);
        }
    }

    /// Log `message` at [`LogLevel::Err`].
    pub fn err(&mut self, message: &str) {
        self.log(LogLevel::Err, message);
    }

    /// Log `message` at [`LogLevel::Warn`].
    pub fn warn(&mut self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    /// Log `message` at [`LogLevel::Info`].
    pub fn info(&mut self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Log `message` at [`LogLevel::Debug`].
    pub fn debug(&mut self, message: &str) {
        self.log(LogLevel::Debug, message);
    }
}

// ---------------------------------------------------------------------------
// SYS-006: split-scrollback accounting.
// ---------------------------------------------------------------------------

/// Render-offset accounting for split scrollback. Direct port of the
/// reference formulas: the offset clamps to published rows, viewport
/// scrolls consume published rows, and newlines/columns grow them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SplitScrollback {
    /// Rows published to scrollback so far.
    pub published_rows: u32,
    /// Columns used on the last published row.
    pub tail_column: u32,
}

impl SplitScrollback {
    /// Start over with `seed_rows` published rows and the tail at column 0.
    pub fn reset(&mut self, seed_rows: u32) {
        self.published_rows = seed_rows;
        self.tail_column = 0;
    }

    /// Return `surface_offset` clamped to the published row count.
    pub fn render_offset(&self, surface_offset: u32) -> u32 {
        if surface_offset == 0 {
            return 0;
        }
        self.published_rows.min(surface_offset)
    }

    /// Record a viewport scroll of `lines` rows. Each scrolled row consumes one published row;
    /// the tail column resets when none remain.
    pub fn note_viewport_scroll(&mut self, lines: u32) {
        self.published_rows = self
            .published_rows
            .saturating_sub(lines.min(self.published_rows));
        if self.published_rows == 0 {
            self.tail_column = 0;
        }
    }

    /// Record a newline: add one row and move the tail to column 0. On empty scrollback the
    /// first newline also counts the row it ends.
    pub fn note_newline(&mut self) {
        if self.published_rows == 0 {
            self.published_rows = 1;
        }
        self.published_rows += 1;
        self.tail_column = 0;
    }

    /// Publish `row_count` rows of `row_columns` columns each, wrapping at `terminal_width`.
    /// Every row but the last ends with a newline; the last does only when `trailing_newline`
    /// is set.
    pub fn publish_snapshot_rows(
        &mut self,
        row_count: u32,
        row_columns: u32,
        terminal_width: u32,
        trailing_newline: bool,
    ) {
        for row in 0..row_count {
            self.publish_row(
                row_columns,
                terminal_width,
                row + 1 < row_count || trailing_newline,
            );
        }
    }

    /// Publish one row of `columns` columns, wrapping at `width`, then a newline when
    /// `trailing_newline` is set. A zero `width` counts as 1.
    pub fn publish_row(&mut self, columns: u32, width: u32, trailing_newline: bool) {
        self.publish_columns(columns, width);
        if trailing_newline {
            self.note_newline();
        }
    }

    fn publish_columns(&mut self, columns: u32, width: u32) {
        if columns == 0 {
            return;
        }
        let safe_width = width.max(1);
        let mut remaining = columns;
        while remaining > 0 {
            if self.published_rows == 0 {
                self.published_rows = 1;
            }
            if self.tail_column >= safe_width {
                self.published_rows += 1;
                self.tail_column = 0;
            }
            let available = safe_width - self.tail_column;
            let step = remaining.min(available);
            self.tail_column += step;
            remaining -= step;
            if remaining > 0 && self.tail_column >= safe_width {
                self.published_rows += 1;
                self.tail_column = 0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SYS-005: ordered span feed.
// ---------------------------------------------------------------------------

/// Span feed failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanError {
    /// Non-auto-commit write exceeds the remaining chunk.
    NoSpace,
    /// Write while a reservation is active.
    Busy,
    /// Write, commit, or reserve on a closed feed.
    Closed,
    /// Zero chunk size or other malformed option.
    InvalidArgument,
}

/// One drained span: contiguous bytes in publish order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The span's bytes.
    pub data: Vec<u8>,
}

/// Feed counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpanStats {
    /// Total bytes accepted by writes.
    pub bytes_written: u64,
    /// Spans published so far.
    pub spans_published: u64,
    /// Spans removed by [`SpanFeed::drain`] so far.
    pub spans_drained: u64,
    /// Bytes written but not yet published.
    pub pending_bytes: usize,
    /// Published spans waiting to be drained.
    pub pending_spans: usize,
}

/// Feed construction options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanFeedOptions {
    /// Maximum bytes in one pending chunk. Must be nonzero. Defaults to 4096.
    pub chunk_size: usize,
    /// Publish each chunk as it fills, so writes never fail with
    /// [`SpanError::NoSpace`]. Defaults to `false`.
    pub auto_commit: bool,
}

impl Default for SpanFeedOptions {
    fn default() -> Self {
        Self {
            chunk_size: 4096,
            auto_commit: false,
        }
    }
}

/// Ordered byte-span feed. Spans publish in write order, atomic
/// writes stay contiguous, and each drained span is delivered exactly
/// once.
#[derive(Debug)]
pub struct SpanFeed {
    chunk_size: usize,
    auto_commit: bool,
    pending: Vec<u8>,
    spans: VecDeque<Span>,
    reserved: bool,
    closed: bool,
    bytes_written: u64,
    spans_published: u64,
    spans_drained: u64,
}

impl SpanFeed {
    /// Create an empty feed. Fails with [`SpanError::InvalidArgument`] when `chunk_size` is 0.
    pub fn new(options: SpanFeedOptions) -> Result<Self, SpanError> {
        if options.chunk_size == 0 {
            return Err(SpanError::InvalidArgument);
        }
        Ok(Self {
            chunk_size: options.chunk_size,
            auto_commit: options.auto_commit,
            pending: Vec::new(),
            spans: VecDeque::new(),
            reserved: false,
            closed: false,
            bytes_written: 0,
            spans_published: 0,
            spans_drained: 0,
        })
    }

    fn publish_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let data = std::mem::take(&mut self.pending);
        self.spans.push_back(Span { data });
        self.spans_published += 1;
    }

    /// Append bytes. Without auto-commit a write that exceeds the
    /// remaining chunk fails with [`SpanError::NoSpace`] and publishes
    /// nothing; with auto-commit each filled chunk publishes itself.
    pub fn write(&mut self, data: &[u8]) -> Result<(), SpanError> {
        if self.closed {
            return Err(SpanError::Closed);
        }
        if self.reserved {
            return Err(SpanError::Busy);
        }
        if data.is_empty() {
            return Ok(());
        }
        if !self.auto_commit && data.len() > self.chunk_size - self.pending.len() {
            return Err(SpanError::NoSpace);
        }
        self.bytes_written += data.len() as u64;
        if self.auto_commit {
            let mut rest = data;
            while !rest.is_empty() {
                let room = self.chunk_size - self.pending.len();
                let take = rest.len().min(room);
                self.pending.extend_from_slice(&rest[..take]);
                rest = &rest[take..];
                if self.pending.len() == self.chunk_size {
                    self.publish_pending();
                }
            }
        } else {
            self.pending.extend_from_slice(data);
        }
        Ok(())
    }

    /// Publish bytes as one contiguous span, flushing earlier pending
    /// bytes first so order is preserved. A failed atomic write
    /// publishes nothing.
    pub fn write_atomic(&mut self, data: &[u8]) -> Result<(), SpanError> {
        if self.closed {
            return Err(SpanError::Closed);
        }
        if self.reserved {
            return Err(SpanError::Busy);
        }
        if data.is_empty() {
            return Ok(());
        }
        self.publish_pending();
        self.bytes_written += data.len() as u64;
        self.spans.push_back(Span {
            data: data.to_vec(),
        });
        self.spans_published += 1;
        Ok(())
    }

    /// Claim the feed for a zero-copy reservation. Only one claim may
    /// be outstanding and none while bytes are pending.
    pub fn reserve(&mut self) -> Result<(), SpanError> {
        if self.closed {
            return Err(SpanError::Closed);
        }
        if self.reserved || !self.pending.is_empty() {
            return Err(SpanError::Busy);
        }
        self.reserved = true;
        Ok(())
    }

    /// Release a reservation. Payload bytes travel through
    /// [`SpanFeed::write`]; the reservation only gates interleaving.
    pub fn commit_reserved(&mut self) -> Result<(), SpanError> {
        if self.closed {
            return Err(SpanError::Closed);
        }
        if !self.reserved {
            return Err(SpanError::InvalidArgument);
        }
        self.reserved = false;
        Ok(())
    }

    /// Publish pending bytes as one span. Empty commits are a no-op.
    pub fn commit(&mut self) -> Result<(), SpanError> {
        if self.closed {
            return Err(SpanError::Closed);
        }
        self.publish_pending();
        Ok(())
    }

    /// Close the feed. Writes, commits, and reservations fail after
    /// this; closing twice is fine.
    pub fn close(&mut self) -> Result<(), SpanError> {
        self.closed = true;
        Ok(())
    }

    /// Return whether [`SpanFeed::close`] was called.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Drain up to `max` spans in publish order. Drained spans never
    /// reappear.
    pub fn drain(&mut self, max: usize) -> Vec<Span> {
        let count = max.min(self.spans.len());
        self.spans_drained += count as u64;
        self.spans.drain(..count).collect()
    }

    /// Return whether any bytes are pending or any published span waits to be drained.
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty() || !self.spans.is_empty()
    }

    /// Return a snapshot of the feed counters.
    pub fn stats(&self) -> SpanStats {
        SpanStats {
            bytes_written: self.bytes_written,
            spans_published: self.spans_published,
            spans_drained: self.spans_drained,
            pending_bytes: self.pending.len(),
            pending_spans: self.spans.len(),
        }
    }
}

// ---------------------------------------------------------------------------
// SYS-010: multi-listener event emitter.
// ---------------------------------------------------------------------------

/// Listener handle returned by [`EventEmitter::on`]. Slot indices
/// are stable: detaching clears a slot without shifting the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(u64);

struct ListenerSlot {
    event: String,
    callback: Option<Box<dyn FnMut()>>,
}

/// Multi-listener emitter over string event names, mirroring the
/// reference `EventEmitter`: listeners attach per event, detach by
/// id, and fire in registration order. Emitting an event nobody
/// registered is a silent no-op, never an error.
#[derive(Default)]
pub struct EventEmitter {
    slots: Vec<ListenerSlot>,
}

impl EventEmitter {
    /// Create an emitter with no listeners.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a listener to an event; returns its detach id.
    pub fn on(&mut self, event: &str, callback: impl FnMut() + 'static) -> ListenerId {
        self.slots.push(ListenerSlot {
            event: event.to_string(),
            callback: Some(Box::new(callback)),
        });
        ListenerId(self.slots.len() as u64 - 1)
    }

    /// Detach a listener. Unknown ids are silent no-ops. Returns
    /// whether a live listener was removed.
    pub fn off(&mut self, id: ListenerId) -> bool {
        match self.slots.get_mut(id.0 as usize) {
            Some(slot) if slot.callback.is_some() => {
                slot.callback = None;
                true
            }
            _ => false,
        }
    }

    /// Fire every live listener on an event, in registration order.
    pub fn emit(&mut self, event: &str) {
        let mut indices = Vec::new();
        for (index, slot) in self.slots.iter().enumerate() {
            if slot.event == event && slot.callback.is_some() {
                indices.push(index);
            }
        }
        for index in indices {
            if let Some(callback) = self.slots[index].callback.as_mut() {
                callback();
            }
        }
    }

    /// Live listener count, for tests.
    pub fn listener_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.callback.is_some())
            .count()
    }
}

// ---------------------------------------------------------------------------
// SYS-011: file logger.
// ---------------------------------------------------------------------------

/// What went wrong appending a log line. Returned, never panicked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileLogError {
    /// The file could not be opened or written. Holds the I/O error text.
    Io(String),
}

/// Logger that appends level-gated lines to a caller-chosen file.
/// Messages below the level never touch the file; I/O failures
/// report instead of panicking.
pub struct FileLogger {
    level: LogLevel,
    path: std::path::PathBuf,
}

impl FileLogger {
    /// Create a logger that appends to `path` at `level`. The file is not opened here; each
    /// write opens it in append mode and creates it when missing.
    pub fn new(path: &std::path::Path, level: LogLevel) -> Self {
        Self {
            level,
            path: path.to_path_buf(),
        }
    }

    /// Set the most verbose level the logger writes.
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    /// Return the current level.
    pub fn level(&self) -> LogLevel {
        self.level
    }

    fn append(&self, level: LogLevel, message: &str) -> Result<(), FileLogError> {
        use std::fmt::Write as _;
        if level > self.level {
            return Ok(());
        }
        let mut line = String::new();
        let _ = writeln!(line, "[{level:?}] {message}");
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .and_then(|mut file| {
                use std::io::Write as _;
                file.write_all(line.as_bytes())
            })
            .map_err(|err| FileLogError::Io(err.to_string()))
    }

    /// Append `message` at [`LogLevel::Err`]. Fails when the file cannot be opened or written.
    pub fn err(&self, message: &str) -> Result<(), FileLogError> {
        self.append(LogLevel::Err, message)
    }

    /// Append `message` at [`LogLevel::Warn`] when the level allows it. Fails on I/O errors.
    pub fn warn(&self, message: &str) -> Result<(), FileLogError> {
        self.append(LogLevel::Warn, message)
    }

    /// Append `message` at [`LogLevel::Info`] when the level allows it. Fails on I/O errors.
    pub fn info(&self, message: &str) -> Result<(), FileLogError> {
        self.append(LogLevel::Info, message)
    }

    /// Append `message` at [`LogLevel::Debug`] when the level allows it. Fails on I/O errors.
    pub fn debug(&self, message: &str) -> Result<(), FileLogError> {
        self.append(LogLevel::Debug, message)
    }
}

// ---- sys-small unit tests (the runner also drives tests/sys_small.rs) ----

#[cfg(test)]
mod small_tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn emitter_fires_in_order() {
        let fired = Rc::new(RefCell::new(Vec::new()));
        let mut emitter = EventEmitter::new();
        for value in [1, 2, 3] {
            let fired = Rc::clone(&fired);
            emitter.on("tick", move || fired.borrow_mut().push(value));
        }
        emitter.emit("tick");
        assert_eq!(*fired.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn file_logger_gates_levels() {
        let path = std::env::temp_dir().join("suprtui-file-logger-unit.log");
        let _ = std::fs::remove_file(&path);
        let logger = FileLogger::new(&path, LogLevel::Warn);
        logger.info("dropped").unwrap();
        logger.warn("kept").unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(!body.contains("dropped"));
        assert!(body.contains("kept"));
        let _ = std::fs::remove_file(&path);
    }
}
