//! Text storage, ported from the reference `rope.zig`,
//! `text-buffer.zig`, and `edit-buffer.zig` (TXT-001 … TXT-004).
//!
//! The reference entangles a balanced rope with views and memory
//! buffers; the spec observes only edit and read behavior (rope
//! balancing is an internal detail), so storage here is a plain
//! `String` addressed by byte offsets. Style spans shift and stretch
//! with edits, undo/redo restore content plus spans step by step, and
//! registered views go dirty on every content edit under an advancing
//! epoch. Wrapping, selection, cursor motion, highlights, syntax
//! styles, and bounds errors arrive with the `text-view` commitment,
//! which owns the view domain.

/// Style span over `[start, end)` byte offsets with a caller-defined
/// style id. Syntax styles attach meaning to ids in text-view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub style: u32,
}

/// Opaque handle to a registered view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ViewId(usize);

#[derive(Clone, Debug)]
struct Snapshot {
    text: String,
    spans: Vec<Span>,
}

/// Text entry-point failures. Out-of-range offsets and split points
/// inside a character return errors; nothing here panics on input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextError {
    OutOfRange,
    NotCharBoundary,
    LineOutOfRange,
}

/// Editable text with spans, history, and view dirt tracking.
/// Reference `UnifiedTextBuffer` plus `EditBuffer` history, behavior
/// only.
pub struct TextBuffer {
    text: String,
    spans: Vec<Span>,
    history: Vec<Snapshot>,
    future: Vec<Snapshot>,
    views: Vec<bool>,
    free_views: Vec<usize>,
    epoch: u64,
}

impl TextBuffer {
    pub fn new() -> Self {
        TextBuffer {
            text: String::new(),
            spans: Vec::new(),
            history: Vec::new(),
            future: Vec::new(),
            views: Vec::new(),
            free_views: Vec::new(),
            epoch: 0,
        }
    }

    pub fn from_text(text: &str) -> Self {
        let mut buffer = TextBuffer::new();
        buffer.text.push_str(text);
        buffer
    }

    /// Full content as plain text.
    pub fn content(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    fn check_offset(&self, offset: usize) -> Result<(), TextError> {
        if offset > self.text.len() {
            return Err(TextError::OutOfRange);
        }
        if !self.text.is_char_boundary(offset) {
            return Err(TextError::NotCharBoundary);
        }
        Ok(())
    }

    fn begin_edit(&mut self) {
        self.history.push(Snapshot {
            text: self.text.clone(),
            spans: self.spans.clone(),
        });
        self.future.clear();
    }

    fn finish_edit(&mut self) {
        self.epoch += 1;
        for dirty in self.views.iter_mut() {
            *dirty = true;
        }
    }

    /// Insert text at a byte offset, shifting spans at or after it and
    /// stretching spans that cover it.
    pub fn insert(&mut self, offset: usize, text: &str) -> Result<(), TextError> {
        self.check_offset(offset)?;
        if text.is_empty() {
            return Ok(());
        }
        self.begin_edit();
        self.text.insert_str(offset, text);
        let delta = text.len();
        for span in self.spans.iter_mut() {
            if offset <= span.start {
                span.start += delta;
                span.end += delta;
            } else if offset <= span.end {
                span.end += delta;
            }
        }
        self.finish_edit();
        Ok(())
    }

    /// Delete `[offset, offset + len)`, dropping spans it fully covers
    /// and clamping spans it overlaps.
    pub fn delete(&mut self, offset: usize, len: usize) -> Result<(), TextError> {
        self.check_offset(offset)?;
        let end = offset.checked_add(len).ok_or(TextError::OutOfRange)?;
        self.check_offset(end)?;
        if len == 0 {
            return Ok(());
        }
        self.begin_edit();
        self.text.drain(offset..end);
        let mut kept = Vec::with_capacity(self.spans.len());
        for span in self.spans.drain(..) {
            if span.end <= offset {
                kept.push(span);
            } else if span.start >= end {
                kept.push(Span {
                    start: span.start - len,
                    end: span.end - len,
                    style: span.style,
                });
            } else if span.start >= offset && span.end <= end {
                // Fully covered: the span's text is gone with it.
            } else {
                let overlap = span.end.min(end) - span.start.max(offset);
                let start = span.start.min(offset);
                let end = start + (span.end - span.start) - overlap;
                kept.push(Span {
                    start,
                    end,
                    style: span.style,
                });
            }
        }
        self.spans = kept;
        self.finish_edit();
        Ok(())
    }

    /// Split the buffer at an offset, returning the tail as a new
    /// buffer. Spans crossing the split are clamped to their side;
    /// history stays with the head.
    pub fn split_off(&mut self, offset: usize) -> Result<TextBuffer, TextError> {
        self.check_offset(offset)?;
        self.begin_edit();
        let tail_text = self.text.split_off(offset);
        let mut tail_spans = Vec::new();
        let mut head_spans = Vec::with_capacity(self.spans.len());
        for span in self.spans.drain(..) {
            if span.end <= offset {
                head_spans.push(span);
            } else if span.start >= offset {
                tail_spans.push(Span {
                    start: span.start - offset,
                    end: span.end - offset,
                    style: span.style,
                });
            } else {
                head_spans.push(Span {
                    start: span.start,
                    end: offset,
                    style: span.style,
                });
                tail_spans.push(Span {
                    start: 0,
                    end: span.end - offset,
                    style: span.style,
                });
            }
        }
        self.spans = head_spans;
        let mut tail = TextBuffer::new();
        tail.text = tail_text;
        tail.spans = tail_spans;
        self.finish_edit();
        Ok(tail)
    }

    /// Cover `[start, end)` with a style id. Bounds are validated like
    /// edits; spans are rendered content, so this pushes history, marks
    /// views dirty, and advances the epoch like any content edit.
    pub fn add_span(&mut self, start: usize, end: usize, style: u32) -> Result<(), TextError> {
        self.check_offset(start)?;
        self.check_offset(end)?;
        if start > end {
            return Err(TextError::OutOfRange);
        }
        self.begin_edit();
        self.spans.push(Span { start, end, style });
        self.finish_edit();
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    fn restore(&mut self, snapshot: Snapshot) {
        self.text = snapshot.text;
        self.spans = snapshot.spans;
        self.epoch += 1;
        for dirty in self.views.iter_mut() {
            *dirty = true;
        }
    }

    /// Restore content and styles one step back.
    pub fn undo(&mut self) -> bool {
        let Some(snapshot) = self.history.pop() else {
            return false;
        };
        self.future.push(Snapshot {
            text: std::mem::take(&mut self.text),
            spans: std::mem::take(&mut self.spans),
        });
        self.restore(snapshot);
        true
    }

    /// Replay one undone step. Any new edit clears the redo stack.
    pub fn redo(&mut self) -> bool {
        let Some(snapshot) = self.future.pop() else {
            return false;
        };
        self.history.push(Snapshot {
            text: std::mem::take(&mut self.text),
            spans: std::mem::take(&mut self.spans),
        });
        self.restore(snapshot);
        true
    }

    /// Register a view; it reads dirty until cleared.
    pub fn register_view(&mut self) -> ViewId {
        if let Some(index) = self.free_views.pop() {
            self.views[index] = true;
            ViewId(index)
        } else {
            self.views.push(true);
            ViewId(self.views.len() - 1)
        }
    }

    pub fn unregister_view(&mut self, view: ViewId) {
        if view.0 < self.views.len() {
            self.views[view.0] = false;
            self.free_views.push(view.0);
        }
    }

    pub fn view_dirty(&self, view: ViewId) -> bool {
        self.views.get(view.0).copied().unwrap_or(false)
    }

    pub fn clear_view_dirty(&mut self, view: ViewId) {
        if let Some(dirty) = self.views.get_mut(view.0) {
            *dirty = false;
        }
    }

    /// Content epoch, advanced by every content edit, undo, and redo.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        TextBuffer::new()
    }
}

// ---- iterators (TXT-013) ----

/// One grapheme cluster with its byte range. Clusters never split:
/// combining sequences and zero-width joiners travel together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cluster<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

impl TextBuffer {
    /// Logical line ranges without terminators, in order.
    pub fn line_ranges(&self) -> Vec<(usize, usize)> {
        logical_lines(self.content())
    }

    /// Grapheme clusters over `[start, end)`. Offsets must be char
    /// boundaries; every yielded cluster is non-empty so iteration
    /// always terminates, even over zero-width prefixes.
    pub fn clusters(&self, start: usize, end: usize) -> Result<Vec<Cluster<'_>>, TextError> {
        self.check_offset(start)?;
        self.check_offset(end)?;
        if start > end {
            return Err(TextError::OutOfRange);
        }
        let text = self.content();
        let mut out = Vec::new();
        let mut pos = start;
        for (s, e) in grapheme_breaks(&text[start..end], WidthMethod::Unicode) {
            let (cs, ce) = (start + s, start + e);
            if ce <= pos {
                // Defensive: never emit an empty or backward step,
                // so a surprising break table cannot hang the caller.
                continue;
            }
            out.push(Cluster {
                text: &text[cs.max(pos)..ce],
                start: cs.max(pos),
                end: ce,
            });
            pos = ce;
        }
        if pos < end {
            // The break table covered nothing past `pos` (for
            // example a lone zero-width prefix the table skipped):
            // yield the remainder whole rather than dropping text.
            out.push(Cluster {
                text: &text[pos..end],
                start: pos,
                end,
            });
        }
        Ok(out)
    }

    /// Logical `(row, byte col)` holding an offset.
    pub fn offset_to_coords(&self, offset: usize) -> Result<(usize, usize), TextError> {
        self.check_offset(offset)?;
        for (row, (start, end)) in self.line_ranges().iter().enumerate() {
            if offset <= *end {
                return Ok((row, offset - start));
            }
        }
        let last = self.line_ranges().len().saturating_sub(1);
        Ok((last, 0))
    }

    /// Offset of a `(row, byte col)` pair; columns past the line end
    /// clamp to it.
    pub fn coords_to_offset(&self, row: usize, col: usize) -> Result<usize, TextError> {
        let ranges = self.line_ranges();
        let (start, end) = ranges.get(row).copied().ok_or(TextError::LineOutOfRange)?;
        Ok((start + col).min(end))
    }
}

// ---- view domain (text-view commitment) ----

use crate::uni::segments::{
    grapheme_breaks, is_word_codepoint, line_breaks, wrap_pos_grapheme_safe,
};
use crate::uni::{WidthMethod, width_at};

/// Line wrap mode. Only `uni` decides break positions; the view caches
/// the resulting virtual lines.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WrapMode {
    /// One virtual line per logical line, however wide.
    #[default]
    None,
    /// Break after the last whitespace fitting the width; overlong
    /// words break at the width.
    Word,
    /// Break at the width on cluster boundaries.
    Char,
}

/// One laid-out row: byte range into the buffer without the line
/// terminator, plus its logical line index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtualLine {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

/// Highlight overlay owned by the view: spans grouped under one
/// reference id for exact removal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Highlight {
    pub id: u32,
    pub spans: Vec<Span>,
}

/// Attached syntax style: a substring pattern whose occurrences take
/// the style id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxStyle {
    pub name: String,
    pub pattern: String,
    pub style: u32,
}

fn logical_lines(text: &str) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let mut start = 0;
    for br in line_breaks(text) {
        let (end, next) = match br.kind {
            crate::uni::segments::LineBreakKind::Crlf => (br.pos - 1, br.pos + 1),
            _ => (br.pos, br.pos + 1),
        };
        lines.push((start, end));
        start = next;
    }
    lines.push((start, text.len()));
    lines
}

/// Editing view over an owned buffer: wrapping, selection, a
/// cluster-safe cursor, highlight overlays, and syntax styles.
pub struct TextView {
    buffer: TextBuffer,
    width: u32,
    wrap: WrapMode,
    cursor: usize,
    selection: Option<(usize, usize)>,
    highlights: Vec<Highlight>,
    syntax_styles: Vec<SyntaxStyle>,
    cached_lines: Vec<VirtualLine>,
    cache_epoch: u64,
    cache_width: u32,
    cache_wrap: WrapMode,
    recomputes: u64,
    occupancy: SelectionOccupancy,
    gesture: Option<GestureState>,
}

impl TextView {
    pub fn new(buffer: TextBuffer) -> Self {
        TextView {
            buffer,
            width: 80,
            wrap: WrapMode::None,
            cursor: 0,
            selection: None,
            highlights: Vec::new(),
            syntax_styles: Vec::new(),
            cached_lines: Vec::new(),
            cache_epoch: u64::MAX,
            cache_width: 0,
            cache_wrap: WrapMode::Char,
            recomputes: 0,
            occupancy: SelectionOccupancy::Boundary,
            gesture: None,
        }
    }

    pub fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut TextBuffer {
        &mut self.buffer
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
    }

    pub fn set_wrap_mode(&mut self, wrap: WrapMode) {
        self.wrap = wrap;
    }

    fn wrap_width(&self) -> u32 {
        self.width.max(1)
    }

    /// Layouts recomputed since construction (TXT-014): repeated
    /// queries share one layout, so the count moves only when
    /// content, width, or wrap mode moves.
    pub fn recompute_count(&self) -> u64 {
        self.recomputes
    }

    fn refresh_cache(&mut self) {
        if self.buffer.epoch() == self.cache_epoch
            && self.width == self.cache_width
            && self.wrap == self.cache_wrap
        {
            return;
        }
        self.recomputes += 1;
        let mut lines = Vec::new();
        let text = self.buffer.content();
        for (index, (start, end)) in logical_lines(text).iter().enumerate() {
            let line = &text[*start..*end];
            if self.wrap == WrapMode::None || line.is_empty() {
                lines.push(VirtualLine {
                    line: index,
                    start: *start,
                    end: *end,
                });
                continue;
            }
            let width = self.wrap_width();
            let starts: Vec<usize> = grapheme_breaks(line, WidthMethod::Unicode)
                .iter()
                .map(|(s, _)| *s)
                .collect();
            let snap = |cut: usize| -> usize {
                let mut at = 0;
                for s in starts.iter() {
                    if *s <= cut {
                        at = *s;
                    } else {
                        break;
                    }
                }
                at
            };
            let mut pos = 0;
            while pos < line.len() {
                let rest = &line[pos..];
                let span = wrap_pos_grapheme_safe(rest, width, 8, false, WidthMethod::Unicode);
                let mut cut = span.byte_offset as usize;
                if cut >= rest.len() {
                    lines.push(VirtualLine {
                        line: index,
                        start: *start + pos,
                        end: *end,
                    });
                    break;
                }
                if self.wrap == WrapMode::Word {
                    let prefix = &rest[..cut];
                    if let Some((k, ch)) = prefix
                        .char_indices()
                        .rev()
                        .find(|(_, ch)| ch.is_whitespace())
                    {
                        let after = k + ch.len_utf8();
                        if after < rest.len() {
                            cut = after;
                        }
                    }
                }
                cut = snap(pos + cut);
                if cut <= pos {
                    // No forward break: consume one cluster so the loop
                    // always progresses.
                    cut = starts
                        .iter()
                        .copied()
                        .find(|s| *s > pos)
                        .unwrap_or(line.len());
                }
                lines.push(VirtualLine {
                    line: index,
                    start: *start + pos,
                    end: *start + cut,
                });
                pos = cut;
            }
        }
        self.cache_epoch = self.buffer.epoch();
        self.cache_width = self.width;
        self.cache_wrap = self.wrap;
        self.cached_lines = lines;
    }

    /// Laid-out rows, rebuilt when content, width, or wrap mode moves.
    pub fn virtual_lines(&mut self) -> &[VirtualLine] {
        self.refresh_cache();
        &self.cached_lines
    }

    pub fn virtual_line_count(&mut self) -> usize {
        self.refresh_cache();
        self.cached_lines.len()
    }

    fn check_offset(&self, offset: usize) -> Result<(), TextError> {
        if offset > self.buffer.len() {
            return Err(TextError::OutOfRange);
        }
        if !self.buffer.content().is_char_boundary(offset) {
            return Err(TextError::NotCharBoundary);
        }
        Ok(())
    }

    fn check_line(&self, line: usize) -> Result<(usize, usize), TextError> {
        logical_lines(self.buffer.content())
            .get(line)
            .copied()
            .ok_or(TextError::LineOutOfRange)
    }

    /// Number of logical lines; a trailing break opens a final empty
    /// line.
    pub fn line_count(&self) -> usize {
        logical_lines(self.buffer.content()).len()
    }

    /// Byte range of one logical line without its terminator.
    pub fn line_range(&self, line: usize) -> Result<(usize, usize), TextError> {
        self.check_line(line)
    }

    /// Logical line and byte column holding an offset.
    pub fn offset_to_line_col(&self, offset: usize) -> Result<(usize, usize), TextError> {
        self.check_offset(offset)?;
        let mut line = 0;
        for (start, end) in logical_lines(self.buffer.content()) {
            if offset <= end {
                return Ok((line, offset - start));
            }
            line += 1;
        }
        Ok((line.saturating_sub(1), 0))
    }

    // ---- selection (TXT-006) ----

    /// Anchor and focus as an ordered byte range, if any.
    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection.map(|(a, f)| (a.min(f), a.max(f)))
    }

    pub fn set_selection(&mut self, anchor: usize, focus: usize) -> Result<(), TextError> {
        self.check_offset(anchor)?;
        self.check_offset(focus)?;
        self.selection = Some((anchor, focus));
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Exactly the selected bytes, or `None` without a selection.
    /// Endpoints clamp to the content after direct buffer edits.
    pub fn selected_text(&self) -> Option<&str> {
        let (a, f) = self.selection?;
        let len = self.buffer.len();
        let (start, end) = (a.min(f).min(len), a.max(f).min(len));
        self.buffer.content().get(start..end)
    }

    /// Cover the whole word around an offset; words are runs of word
    /// codepoints under `uni` classification.
    pub fn select_word_at(&mut self, offset: usize) -> Result<(), TextError> {
        self.check_offset(offset)?;
        let text = self.buffer.content();
        let mut start = offset;
        while start > 0 {
            let prev = text[..start].chars().next_back();
            match prev {
                Some(ch) if is_word_codepoint(ch as u32) => start -= ch.len_utf8(),
                _ => break,
            }
        }
        let mut end = offset;
        while end < text.len() {
            let next = text[end..].chars().next();
            match next {
                Some(ch) if is_word_codepoint(ch as u32) => end += ch.len_utf8(),
                _ => break,
            }
        }
        self.selection = Some((start, end));
        Ok(())
    }

    /// Cover the whole logical line around an offset, terminator
    /// excluded.
    pub fn select_line_at(&mut self, offset: usize) -> Result<(), TextError> {
        self.check_offset(offset)?;
        let (line, _) = self.offset_to_line_col(offset)?;
        let (start, end) = self.check_line(line)?;
        self.selection = Some((start, end));
        Ok(())
    }

    // ---- gesture and viewport selection (TXT-011, TXT-012) ----

    /// How a pressed cell expands. `Cell` keeps raw offsets, `Word`
    /// and `Line` expand each endpoint through the word unit or the
    /// logical line, as in the reference selection behaviors.
    pub fn set_gesture_occupancy(&mut self, occupancy: SelectionOccupancy) {
        self.occupancy = occupancy;
    }

    pub fn gesture_occupancy(&self) -> SelectionOccupancy {
        self.occupancy
    }

    /// Display-column cell to content offset on a virtual line.
    /// Columns past the line end land on the line end (padding
    /// clicks); a virtual line past the layout is `None`.
    pub fn cell_to_offset(&mut self, vline: usize, col: u32) -> Option<usize> {
        self.refresh_cache();
        let line = self.cached_lines.get(vline).copied()?;
        let text = self.buffer.content();
        let mut acc = 0u32;
        for s in self.cluster_starts(line.start, line.end) {
            let w = width_at(text, s, 8, WidthMethod::Unicode).max(1);
            if col < acc + w {
                return Some(s);
            }
            acc += w;
        }
        Some(line.end)
    }

    /// Word unit around an offset: maximal runs of one class, where
    /// the reference boundary set (spaces, tabs, quotes, brackets,
    /// and other separators) groups instead of splitting, so space
    /// runs are selectable. `/` is not a boundary, unlike in
    /// wrapping; CJK and ASCII group together.
    fn word_unit_at(&self, offset: usize, line_start: usize, line_end: usize) -> (usize, usize) {
        let text = self.buffer.content();
        let at = offset.clamp(line_start, line_end);
        let class = |o: usize| {
            text[o..]
                .chars()
                .next()
                .map(is_selection_separator)
                .unwrap_or(true)
        };
        let anchor_class = class(at.min(line_end.saturating_sub(1).max(line_start)));
        let mut start = at;
        while start > line_start {
            let prev_len = text[..start]
                .chars()
                .next_back()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            if prev_len == 0 || class(start - prev_len) != anchor_class {
                break;
            }
            start -= prev_len;
        }
        let mut end = at;
        while end < line_end {
            if class(end) != anchor_class {
                break;
            }
            end += text[end..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
        }
        (start, end)
    }

    /// Display width of a virtual line in columns.
    fn vline_width(&mut self, vline: usize) -> Option<u32> {
        self.refresh_cache();
        let line = self.cached_lines.get(vline).copied()?;
        let text = self.buffer.content();
        let mut acc = 0u32;
        for s in self.cluster_starts(line.start, line.end) {
            acc += width_at(text, s, 8, WidthMethod::Unicode).max(1);
        }
        Some(acc)
    }

    fn unit_at(
        &mut self,
        vline: usize,
        col: u32,
        behavior: GestureBehavior,
    ) -> Result<(usize, usize), TextError> {
        let offset = self
            .cell_to_offset(vline, col)
            .ok_or(TextError::LineOutOfRange)?;
        let (line, _) = self.offset_to_line_col(offset)?;
        let (line_start, line_end) = self.check_line(line)?;
        // Padding clicks (past the last cell) select nothing: a
        // zero-width range at the line end, except line behavior
        // which still takes the line.
        let padding = self.vline_width(vline).is_some_and(|w| col >= w);
        let unit = match behavior {
            GestureBehavior::Cell => (offset, offset),
            GestureBehavior::Word if padding => (line_end, line_end),
            GestureBehavior::Word => self.word_unit_at(offset, line_start, line_end),
            GestureBehavior::Line => (line_start, line_end),
        };
        Ok(self.apply_occupancy(unit.0, unit.1))
    }

    fn apply_occupancy(&self, start: usize, end: usize) -> (usize, usize) {
        let (lo, hi) = (start.min(end), start.max(end));
        if self.occupancy == SelectionOccupancy::Cell && hi > lo {
            // Block occupancy includes the grapheme under the max
            // endpoint: snap the end to the next cluster boundary.
            let text = self.buffer.content();
            let (line, _) = self.offset_to_line_col(hi).unwrap_or((0, 0));
            if let Ok((line_start, line_end)) = self.check_line(line) {
                for s in self.cluster_starts(line_start, line_end) {
                    if s > hi {
                        return (lo, s);
                    }
                }
                return (lo, text.len().min(line_end.max(hi)));
            }
        }
        (lo, hi)
    }

    /// Press: select the unit under a cell and open a gesture.
    pub fn gesture_press(
        &mut self,
        vline: usize,
        col: u32,
        behavior: GestureBehavior,
    ) -> Result<(), TextError> {
        let unit = self.unit_at(vline, col, behavior)?;
        self.selection = Some(unit);
        self.gesture = Some(GestureState {
            anchor_unit: unit,
            behavior,
        });
        Ok(())
    }

    /// Drag: extend the press unit through the focus unit. Forward
    /// and backward drags over the same cells agree, since both
    /// endpoints expand to unit edges before the union.
    pub fn gesture_move(&mut self, vline: usize, col: u32) -> Result<(), TextError> {
        let anchor_unit = self.gesture.as_ref().map(|g| g.anchor_unit);
        let behavior = self.gesture.as_ref().map(|g| g.behavior);
        let (anchor_unit, behavior) = match (anchor_unit, behavior) {
            (Some(unit), Some(behavior)) => (unit, behavior),
            _ => return Err(TextError::OutOfRange),
        };
        let focus_unit = self.unit_at(vline, col, behavior)?;
        let united = (
            anchor_unit.0.min(focus_unit.0),
            anchor_unit.1.max(focus_unit.1),
        );
        self.selection = Some(self.apply_occupancy(united.0, united.1));
        Ok(())
    }

    /// Release: commit the drag and close the gesture, returning the
    /// ordered range. Without an open gesture, reports the current
    /// selection unchanged.
    pub fn gesture_release(&mut self) -> Option<(usize, usize)> {
        self.gesture = None;
        self.selection()
    }

    pub fn gesture_active(&self) -> bool {
        self.gesture.is_some()
    }

    /// Re-express the current selection with cell occupancy so a
    /// word or line range keeps its text under block semantics.
    pub fn gesture_convert_to_cell(&mut self) {
        if let Some((start, end)) = self.selection() {
            self.occupancy = SelectionOccupancy::Cell;
            self.selection = Some(self.apply_occupancy(start, end));
        }
    }

    /// Resolve a viewport cell to a content offset. The row counts
    /// virtual lines below `viewport.first_row`; in wrapping modes
    /// the horizontal offset is ignored, otherwise the column adds
    /// `viewport.left_col`. Out-of-window rows are `None`.
    pub fn viewport_to_offset(
        &mut self,
        viewport: Viewport,
        row: usize,
        col: u32,
    ) -> Option<usize> {
        let wrapping = self.wrap != WrapMode::None;
        let effective = if wrapping {
            col
        } else {
            col.saturating_add(viewport.left_col)
        };
        self.cell_to_offset(viewport.first_row.saturating_add(row), effective)
    }

    /// Select across viewport coordinates with boundary occupancy.
    pub fn select_viewport(
        &mut self,
        viewport: Viewport,
        anchor_row: usize,
        anchor_col: u32,
        focus_row: usize,
        focus_col: u32,
    ) -> Result<(), TextError> {
        let anchor = self
            .viewport_to_offset(viewport, anchor_row, anchor_col)
            .ok_or(TextError::LineOutOfRange)?;
        let focus = self
            .viewport_to_offset(viewport, focus_row, focus_col)
            .ok_or(TextError::LineOutOfRange)?;
        self.selection = Some((anchor.min(focus), anchor.max(focus)));
        Ok(())
    }

    // ---- cluster-safe cursor (TXT-007) ----

    /// Cursor offset, clamped to the content after direct buffer edits.
    pub fn cursor(&self) -> usize {
        self.cursor.min(self.buffer.len())
    }

    pub fn set_cursor(&mut self, offset: usize) -> Result<(), TextError> {
        self.check_offset(offset)?;
        self.cursor = offset;
        Ok(())
    }

    fn cluster_starts(&self, line_start: usize, line_end: usize) -> Vec<usize> {
        let text = self.buffer.content();
        grapheme_breaks(&text[line_start..line_end], WidthMethod::Unicode)
            .iter()
            .map(|(s, _)| line_start + s)
            .collect()
    }

    fn snap_to_cluster(&self, offset: usize) -> usize {
        let text = self.buffer.content();
        let (line, _) = self.offset_to_line_col(offset).unwrap_or((0, 0));
        let Ok((start, end)) = self.check_line(line) else {
            return text.len();
        };
        let mut at = start;
        for s in self.cluster_starts(start, end) {
            if s <= offset {
                at = s;
            } else {
                break;
            }
        }
        at
    }

    /// Step to the previous cluster start, crossing to the previous
    /// line end at a line start; stays at zero.
    pub fn move_left(&mut self) -> usize {
        let cursor = self.cursor();
        if cursor == 0 {
            return 0;
        }
        let (line, _) = self.offset_to_line_col(cursor).unwrap_or((0, 0));
        let (start, _) = self.check_line(line).unwrap_or((0, 0));
        if cursor == start && line > 0 {
            let (_, prev_end) = self.check_line(line - 1).unwrap_or((0, 0));
            self.cursor = prev_end;
            return prev_end;
        }
        let mut at = start;
        for s in self.cluster_starts(start, cursor) {
            if s < cursor {
                at = s;
            }
        }
        self.cursor = at;
        at
    }

    /// Step to the next cluster start, crossing to the next line start
    /// at a line end; stays at the content end.
    pub fn move_right(&mut self) -> usize {
        let cursor = self.cursor();
        let text = self.buffer.content();
        if cursor >= text.len() {
            return text.len();
        }
        let (line, _) = self.offset_to_line_col(cursor).unwrap_or((0, 0));
        let (_, end) = self.check_line(line).unwrap_or((0, text.len()));
        if cursor == end {
            let (next_start, _) = self
                .check_line(line + 1)
                .unwrap_or((text.len(), text.len()));
            self.cursor = next_start.min(text.len());
            return self.cursor;
        }
        let mut at = end.min(text.len());
        for s in self.cluster_starts(cursor, end) {
            if s > cursor {
                at = s;
                break;
            }
        }
        self.cursor = at.min(text.len());
        self.cursor
    }

    /// Move to the same character column in the neighboring logical
    /// line, clamped and snapped to a cluster start.
    pub fn move_up(&mut self) -> usize {
        self.move_vertical(-1)
    }

    pub fn move_down(&mut self) -> usize {
        self.move_vertical(1)
    }

    fn move_vertical(&mut self, delta: i32) -> usize {
        let cursor = self.cursor();
        let text = self.buffer.content();
        let (line, _) = self.offset_to_line_col(cursor).unwrap_or((0, 0));
        let (origin_start, _) = self.check_line(line).unwrap_or((0, 0));
        let col_chars = text[origin_start..cursor.min(text.len())].chars().count();
        let target = line as i32 + delta;
        if target < 0 {
            self.cursor = 0;
            return 0;
        }
        let Ok((start, end)) = self.check_line(target as usize) else {
            self.cursor = text.len();
            return text.len();
        };
        let line_text = &text[start..end];
        let mut at = end;
        for (i, (k, _)) in line_text.char_indices().enumerate() {
            if i >= col_chars {
                at = start + k;
                break;
            }
        }
        self.cursor = self.snap_to_cluster(at.min(end));
        self.cursor
    }

    pub fn move_home(&mut self) -> usize {
        let cursor = self.cursor();
        let (line, _) = self.offset_to_line_col(cursor).unwrap_or((0, 0));
        let (start, _) = self.check_line(line).unwrap_or((0, 0));
        self.cursor = start;
        start
    }

    pub fn move_end(&mut self) -> usize {
        let cursor = self.cursor();
        let (line, _) = self.offset_to_line_col(cursor).unwrap_or((0, 0));
        let (_, end) = self.check_line(line).unwrap_or((0, self.buffer.len()));
        self.cursor = end;
        end
    }

    /// Insert text at the cursor and advance past it.
    pub fn insert_text(&mut self, text: &str) -> Result<(), TextError> {
        let cursor = self.cursor();
        self.buffer.insert(cursor, text)?;
        self.cursor = cursor + text.len();
        self.clear_selection();
        Ok(())
    }

    /// Delete the cluster before the cursor, joining lines at a line
    /// start. Never strands half a cluster.
    pub fn backspace(&mut self) -> Result<(), TextError> {
        let cursor = self.cursor();
        if cursor == 0 {
            return Ok(());
        }
        let (line, _) = self.offset_to_line_col(cursor).unwrap_or((0, 0));
        let (start, _) = self.check_line(line).unwrap_or((0, 0));
        if cursor == start && line > 0 {
            let (_, prev_end) = self.check_line(line - 1)?;
            self.buffer.delete(prev_end, cursor - prev_end)?;
            self.cursor = prev_end;
        } else {
            let prev = self.move_left();
            self.buffer.delete(prev, cursor - prev)?;
            self.cursor = prev;
        }
        self.clear_selection();
        Ok(())
    }

    /// Delete the cluster at the cursor, joining lines at a line end.
    pub fn delete_at_cursor(&mut self) -> Result<(), TextError> {
        let cursor = self.cursor();
        let text = self.buffer.content();
        if cursor >= text.len() {
            return Ok(());
        }
        let (line, _) = self.offset_to_line_col(cursor).unwrap_or((0, 0));
        let (_, end) = self.check_line(line).unwrap_or((0, text.len()));
        if cursor == end {
            let (next_start, _) = self.check_line(line + 1)?;
            self.buffer.delete(cursor, next_start - cursor)?;
        } else {
            let next = self.move_right();
            self.buffer.delete(cursor, next - cursor)?;
            self.cursor = cursor;
        }
        self.clear_selection();
        Ok(())
    }

    // ---- highlights by reference (TXT-008) ----

    /// Stage an overlay span under a highlight id. Overlays render on
    /// top of buffer spans; removing by id drops exactly its spans.
    pub fn add_highlight(
        &mut self,
        id: u32,
        start: usize,
        end: usize,
        style: u32,
    ) -> Result<(), TextError> {
        self.check_offset(start)?;
        self.check_offset(end)?;
        if start > end {
            return Err(TextError::OutOfRange);
        }
        match self.highlights.iter_mut().find(|h| h.id == id) {
            Some(entry) => entry.spans.push(Span { start, end, style }),
            None => self.highlights.push(Highlight {
                id,
                spans: vec![Span { start, end, style }],
            }),
        }
        Ok(())
    }

    pub fn highlights(&self) -> &[Highlight] {
        &self.highlights
    }

    /// Drop exactly one reference's spans; foreign spans are untouched.
    /// Returns false when the id was never added.
    pub fn remove_highlight(&mut self, id: u32) -> bool {
        match self.highlights.iter().position(|h| h.id == id) {
            Some(index) => {
                self.highlights.remove(index);
                true
            }
            None => false,
        }
    }

    /// Buffer spans with highlight overlays appended, the render input.
    pub fn resolved_spans(&self) -> Vec<Span> {
        let mut out = self.buffer.spans().to_vec();
        for highlight in self.highlights.iter() {
            out.extend(highlight.spans.iter().copied());
        }
        out
    }

    // ---- syntax styles (TXT-009) ----

    /// Attach a style to every non-overlapping occurrence of a
    /// substring pattern, returning the span count. An empty pattern
    /// matches nothing.
    pub fn attach_syntax_style(
        &mut self,
        name: &str,
        pattern: &str,
        style: u32,
    ) -> Result<usize, TextError> {
        if pattern.is_empty() {
            return Ok(0);
        }
        let mut count = 0;
        let mut search = 0;
        let text = self.buffer.content();
        let mut hits = Vec::new();
        while let Some(found) = text[search..].find(pattern) {
            let start = search + found;
            hits.push((start, start + pattern.len()));
            search = start + pattern.len();
            count += 1;
        }
        for (start, end) in hits {
            self.buffer.add_span(start, end, style)?;
        }
        self.syntax_styles.push(SyntaxStyle {
            name: name.to_string(),
            pattern: pattern.to_string(),
            style,
        });
        Ok(count)
    }

    pub fn syntax_styles(&self) -> &[SyntaxStyle] {
        &self.syntax_styles
    }
}

#[cfg(test)]
fn test_view(text: &str) -> TextView {
    TextView::new(TextBuffer::from_text(text))
}

/// TXT-005 falsifier: a virtual line exceeds the set width, or a wrap
/// splits a grapheme cluster.
#[cfg(test)]
#[test]
fn wrapping() {
    let mut view = test_view("aa bb cc dd ee");
    view.set_wrap_mode(WrapMode::Word);
    view.set_width(6);
    let rows = view.virtual_lines().to_vec();
    let lines: Vec<String> = rows
        .iter()
        .map(|v| view.buffer().content()[v.start..v.end].to_string())
        .collect();
    assert_eq!(vec!["aa bb ", "cc dd ", "ee"], lines);
    // No virtual line exceeds the width.
    let widths: Vec<usize> = rows
        .iter()
        .map(|v| view.buffer().content()[v.start..v.end].chars().count())
        .collect();
    assert!(widths.iter().all(|w| *w <= 6));

    // Char wrap never splits a cluster: e + acute stays whole.
    let mut wide = test_view("ae\u{301}b ce\u{301}d");
    wide.set_wrap_mode(WrapMode::Char);
    wide.set_width(2);
    let wrows = wide.virtual_lines().to_vec();
    let rows: Vec<String> = wrows
        .iter()
        .map(|v| wide.buffer().content()[v.start..v.end].to_string())
        .collect();
    assert!(rows.iter().all(|r| r.chars().count() <= 3));
    assert_eq!("ae\u{301}", rows[0]);
    assert_eq!(4, rows.len());
    assert_eq!(4, wide.virtual_line_count());

    // NoWrap keeps logical lines however wide.
    let mut plain = test_view("aa bb cc dd ee");
    plain.set_width(6);
    assert_eq!(1, plain.virtual_line_count());
}

/// TXT-006 falsifier: a word selection stops mid-word, or extracted
/// text differs from the selected range.
#[cfg(test)]
#[test]
fn selection() {
    let mut view = test_view("hello brave world");
    view.select_word_at(8).unwrap();
    assert_eq!(Some((6, 11)), view.selection());
    assert_eq!(Some("brave"), view.selected_text());
    view.select_line_at(0).unwrap();
    assert_eq!(Some((0, 17)), view.selection());
    assert_eq!(Some("hello brave world"), view.selected_text());
    view.set_selection(11, 6).unwrap();
    assert_eq!(Some((6, 11)), view.selection());
    assert_eq!(Some("brave"), view.selected_text());
    view.clear_selection();
    assert_eq!(None, view.selection());
    assert_eq!(None, view.selected_text());
}

/// TXT-007 falsifier: a backspace on a combining sequence removes only
/// the mark or only the base.
#[cfg(test)]
#[test]
fn cursor_cluster_safety() {
    // e + combining acute is one cluster over two chars.
    let mut view = test_view("ae\u{301}b");
    view.set_cursor(4).unwrap();
    view.backspace().unwrap();
    assert_eq!("ab", view.buffer().content());
    assert_eq!(1, view.cursor());
    // Delete takes the whole cluster forward too.
    let mut forward = test_view("ae\u{301}b");
    forward.set_cursor(1).unwrap();
    forward.delete_at_cursor().unwrap();
    assert_eq!("ab", forward.buffer().content());
    // Arrows step over the cluster, never into it.
    let mut walk = test_view("ae\u{301}b");
    assert_eq!(0, walk.cursor());
    assert_eq!(1, walk.move_right());
    assert_eq!(4, walk.move_right());
    assert_eq!(5, walk.move_right());
    assert_eq!(5, walk.move_right());
    assert_eq!(4, walk.move_left());
    // Insert then backspace round-trips a cluster.
    walk.set_cursor(0).unwrap();
    walk.insert_text("e\u{301}").unwrap();
    assert_eq!(3, walk.cursor());
    walk.backspace().unwrap();
    assert_eq!("ae\u{301}b", walk.buffer().content());
}

/// TXT-008 falsifier: a foreign span disappears, or a removed span
/// still renders.
#[cfg(test)]
#[test]
fn highlight_refs() {
    let mut view = test_view("hello world");
    view.add_highlight(1, 0, 5, 10).unwrap();
    view.add_highlight(2, 6, 11, 20).unwrap();
    view.add_highlight(1, 0, 2, 10).unwrap();
    assert!(view.remove_highlight(1));
    assert!(!view.remove_highlight(1));
    assert!(!view.remove_highlight(99));
    assert_eq!(1, view.highlights().len());
    assert_eq!(2, view.highlights()[0].id);
    // The foreign span renders on; the removed ones do not.
    let rendered: Vec<Span> = view.resolved_spans();
    assert!(rendered.contains(&Span {
        start: 6,
        end: 11,
        style: 20
    }));
    assert!(!rendered.iter().any(|s| s.style == 10));
}

/// TXT-009 falsifier: no span changes after attaching a style to text
/// it covers.
#[cfg(test)]
#[test]
fn syntax_style() {
    let mut view = test_view("fn main() { fn helper() {} }");
    let count = view.attach_syntax_style("keyword", "fn", 3).unwrap();
    assert_eq!(2, count);
    let styled: Vec<Span> = view
        .buffer()
        .spans()
        .iter()
        .copied()
        .filter(|s| s.style == 3)
        .collect();
    assert_eq!(
        vec![
            Span {
                start: 0,
                end: 2,
                style: 3
            },
            Span {
                start: 12,
                end: 14,
                style: 3
            }
        ],
        styled
    );
    assert_eq!(0, view.attach_syntax_style("empty", "", 4).unwrap());
}

/// TXT-010 falsifier: any offset beyond the content end panics or
/// corrupts content.
#[cfg(test)]
#[test]
fn offset_bounds() {
    let mut view = test_view("ab\né́d");
    assert_eq!(Err(TextError::OutOfRange), view.set_cursor(99));
    assert_eq!(Err(TextError::OutOfRange), view.set_selection(0, 99));
    assert_eq!(Err(TextError::OutOfRange), view.select_word_at(99));
    assert_eq!(Err(TextError::OutOfRange), view.select_line_at(99));
    assert_eq!(Err(TextError::OutOfRange), view.add_highlight(1, 0, 99, 1));
    assert_eq!(Err(TextError::NotCharBoundary), view.set_cursor(4));
    assert_eq!(Err(TextError::LineOutOfRange), view.line_range(7));
    assert_eq!(2, view.line_count());
    assert_eq!(Ok((1, 0)), view.offset_to_line_col(3));
    // Content survives every rejected call above.
    assert_eq!("ab\né́d", view.buffer().content());
    // Direct buffer edits cannot strand the cursor past the end.
    view.set_cursor(5).unwrap();
    view.buffer_mut().delete(0, 5).unwrap();
    assert!(view.cursor() <= view.buffer().len());
}

/// TXT-001 falsifier: plain-text extraction differs from the reference
/// fixture after a ported edit sequence.
#[cfg(test)]
#[test]
fn edits() {
    let mut buf = TextBuffer::from_text("hello world");
    buf.insert(5, ", brave").unwrap();
    assert_eq!("hello, brave world", buf.content());
    buf.delete(5, 7).unwrap();
    assert_eq!("hello world", buf.content());
    buf.insert(11, "!").unwrap();
    let tail = buf.split_off(6).unwrap();
    assert_eq!("hello ", buf.content());
    assert_eq!("world!", tail.content());
    // Multibyte text: offsets are bytes, splits never strand.
    let mut wide = TextBuffer::from_text("héllo→world");
    wide.insert(1, "X").unwrap();
    assert_eq!("hXéllo→world", wide.content());
    wide.delete(2, "é".len()).unwrap();
    assert_eq!("hXllo→world", wide.content());
    assert_eq!(TextError::OutOfRange, wide.insert(99, "?").unwrap_err());
    assert_eq!(TextError::OutOfRange, wide.delete(0, 99).unwrap_err());
    assert_eq!(TextError::NotCharBoundary, wide.insert(6, "?").unwrap_err());
}

/// TXT-002 falsifier: inserting text before a span leaves the span on
/// its old columns instead of shifting with its text.
#[cfg(test)]
#[test]
fn span_tracking() {
    let mut buf = TextBuffer::from_text("hello world");
    buf.add_span(6, 11, 1).unwrap();
    buf.insert(0, "well, ").unwrap();
    assert_eq!(
        [Span {
            start: 12,
            end: 17,
            style: 1
        }],
        buf.spans()
    );
    // At the span start the insert shifts it with its text.
    buf.insert(12, "wide ").unwrap();
    assert_eq!(
        [Span {
            start: 17,
            end: 22,
            style: 1
        }],
        buf.spans()
    );
    assert_eq!("well, hello wide world", buf.content());
    // Inside the span stretches it.
    buf.insert(19, "XX").unwrap();
    assert_eq!(
        [Span {
            start: 17,
            end: 24,
            style: 1
        }],
        buf.spans()
    );
    // Deleting across the span clamps it to the surviving text.
    buf.delete(10, 8).unwrap();
    assert_eq!("well, helloXXrld", buf.content());
    assert_eq!(
        [Span {
            start: 10,
            end: 16,
            style: 1
        }],
        buf.spans()
    );
    // Deleting the whole span drops it.
    buf.delete(10, 6).unwrap();
    assert!(buf.spans().is_empty());
}

/// TXT-003 falsifier: redo still applies after an intervening edit, or
/// undo restores content but drops styles.
#[cfg(test)]
#[test]
fn undo_redo() {
    let mut buf = TextBuffer::from_text("ab");
    buf.insert(2, "c").unwrap();
    buf.add_span(0, 3, 7).unwrap();
    buf.delete(0, 1).unwrap();
    assert_eq!("bc", buf.content());
    assert!(buf.can_undo());
    // Step back: delete undone first, content and styles together.
    assert!(buf.undo());
    assert_eq!("abc", buf.content());
    assert_eq!(
        [Span {
            start: 0,
            end: 3,
            style: 7
        }],
        buf.spans()
    );
    assert!(buf.undo());
    assert_eq!("abc", buf.content());
    assert!(buf.spans().is_empty());
    assert!(buf.undo());
    assert_eq!("ab", buf.content());
    assert!(!buf.can_undo());
    assert!(!buf.undo());
    // Replay in order.
    assert!(buf.redo());
    assert_eq!("abc", buf.content());
    assert!(buf.redo());
    assert_eq!(
        [Span {
            start: 0,
            end: 3,
            style: 7
        }],
        buf.spans()
    );
    assert!(buf.redo());
    assert_eq!("bc", buf.content());
    assert!(!buf.can_redo());
    // A new edit clears the redo stack.
    assert!(buf.undo());
    buf.insert(0, "z").unwrap();
    assert_eq!("zabc", buf.content());
    assert!(!buf.can_redo());
    assert!(!buf.redo());
}

/// TXT-004 falsifier: a registered view reads clean after an edit, or
/// the epoch is unchanged.
#[cfg(test)]
#[test]
fn view_dirty_tracking() {
    let mut buf = TextBuffer::from_text("ab");
    let first = buf.register_view();
    let second = buf.register_view();
    buf.clear_view_dirty(first);
    buf.clear_view_dirty(second);
    let epoch = buf.epoch();
    buf.insert(2, "c").unwrap();
    assert!(buf.epoch() > epoch);
    assert!(buf.view_dirty(first));
    assert!(buf.view_dirty(second));
    buf.clear_view_dirty(first);
    assert!(!buf.view_dirty(first));
    assert!(buf.view_dirty(second));
    // Undo and redo are content edits too.
    buf.clear_view_dirty(second);
    assert!(buf.undo());
    assert!(buf.view_dirty(first));
    assert!(buf.view_dirty(second));
    // Unregistered views read clean and never panic.
    buf.unregister_view(first);
    assert!(!buf.view_dirty(first));
    assert!(!buf.view_dirty(ViewId(999)));
    buf.clear_view_dirty(ViewId(999));
}

// ---- selection types and editor view (TXT-011, TXT-012, TXT-015) ----

/// How a pressed cell expands into a range. Mirrors the reference
/// `SelectionBehavior`: `cell` keeps the inclusive press/drag,
/// `word` and `line` expand each endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureBehavior {
    Cell,
    Word,
    Line,
}

/// How a selection occupies cells between its offsets. `Cell`
/// includes the grapheme under the max endpoint (block semantics);
/// `Boundary` is the half-open range `[min, max)`. Mirrors the
/// reference `SelectionOccupancy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionOccupancy {
    Cell,
    Boundary,
}

/// Rectangular window into virtual-line space: rows below
/// `first_row`, display columns from `left_col`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport {
    pub first_row: usize,
    pub left_col: u32,
}

impl Viewport {
    pub fn new(first_row: usize, left_col: u32) -> Self {
        Viewport {
            first_row,
            left_col,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct GestureState {
    anchor_unit: (usize, usize),
    behavior: GestureBehavior,
}

/// Reference word-selection separators: these group into selectable
/// runs instead of splitting words. `/` is deliberately absent
/// (unlike in wrapping), and CJK groups with ASCII.
fn is_selection_separator(ch: char) -> bool {
    matches!(
        ch,
        ' ' | '\t'
            | '\''
            | '"'
            | '│'
            | '`'
            | '|'
            | ':'
            | ';'
            | ','
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '<'
            | '>'
            | '$'
    )
}

/// Editor view owning viewport, cursor visibility, and cell-space
/// selection as one unit over a `TextView` (TXT-015). Colors stay
/// with the view highlight layer; this unit owns geometry and
/// behavior only.
pub struct EditorView {
    view: TextView,
    viewport_rows: u32,
    viewport_cols: u32,
    first_row: usize,
    left_col: u32,
    follow_cursor: bool,
    anchor_cell: Option<(usize, u32)>,
    anchor_behavior: GestureBehavior,
}

impl EditorView {
    pub fn new(buffer: TextBuffer) -> Self {
        EditorView {
            view: TextView::new(buffer),
            viewport_rows: 24,
            viewport_cols: 80,
            first_row: 0,
            left_col: 0,
            follow_cursor: false,
            anchor_cell: None,
            anchor_behavior: GestureBehavior::Cell,
        }
    }

    pub fn text_view(&self) -> &TextView {
        &self.view
    }

    pub fn text_view_mut(&mut self) -> &mut TextView {
        &mut self.view
    }

    /// Size the window and the layout width together.
    pub fn set_viewport_size(&mut self, cols: u32, rows: u32) {
        self.viewport_cols = cols.max(1);
        self.viewport_rows = rows.max(1);
        self.view.set_width(cols);
    }

    pub fn viewport_size(&self) -> (u32, u32) {
        (self.viewport_cols, self.viewport_rows)
    }

    pub fn scroll_to(&mut self, first_row: usize, left_col: u32) {
        self.first_row = first_row;
        self.left_col = left_col;
    }

    pub fn viewport(&self) -> Viewport {
        Viewport::new(self.first_row, self.left_col)
    }

    pub fn set_selection_follow_cursor(&mut self, enabled: bool) {
        self.follow_cursor = enabled;
    }

    /// Scroll the window just enough to show the cursor's virtual
    /// line, keeping `margin_rows` of context where room allows.
    pub fn ensure_cursor_visible(&mut self, margin_rows: u32) {
        let cursor = self.view.cursor();
        let lines = self.view.virtual_lines().to_vec();
        let mut cursor_row = 0;
        for (index, vline) in lines.iter().enumerate() {
            if vline.start <= cursor {
                cursor_row = index;
            } else {
                break;
            }
        }
        let rows = self.viewport_rows as usize;
        let margin = margin_rows as usize;
        if cursor_row < self.first_row {
            self.first_row = cursor_row.saturating_sub(margin);
        } else if cursor_row >= self.first_row + rows {
            self.first_row = (cursor_row + 1 + margin).saturating_sub(rows);
        }
        // Horizontal: reveal the cursor's display column.
        let vline = lines.get(cursor_row);
        if let Some(vline) = vline {
            let text = self.view.buffer().content();
            let mut col = 0u32;
            for s in self.view.cluster_starts(vline.start, vline.end) {
                if s >= cursor {
                    break;
                }
                col += width_at(text, s, 8, WidthMethod::Unicode).max(1);
            }
            let cols = self.viewport_cols;
            if col < self.left_col {
                self.left_col = col;
            } else if col >= self.left_col + cols {
                self.left_col = col + 1 - cols;
            }
        }
    }

    /// Cursor's visual position: virtual-line index plus display
    /// column.
    pub fn visual_cursor(&mut self) -> (usize, u32) {
        let cursor = self.view.cursor();
        let lines = self.view.virtual_lines().to_vec();
        let mut row = 0;
        for (index, vline) in lines.iter().enumerate() {
            if vline.start <= cursor {
                row = index;
            } else {
                break;
            }
        }
        let text = self.view.buffer().content();
        let mut col = 0u32;
        if let Some(vline) = lines.get(row) {
            for s in self.view.cluster_starts(vline.start, vline.end) {
                if s >= cursor {
                    break;
                }
                col += width_at(text, s, 8, WidthMethod::Unicode).max(1);
            }
        }
        (row, col)
    }

    /// Visual cell back to a content offset.
    pub fn visual_to_offset(&mut self, vline: usize, col: u32) -> Option<usize> {
        self.view.cell_to_offset(vline, col)
    }

    /// Set a cell-space selection with a behavior; with cursor
    /// follow, the cursor syncs to the focus.
    pub fn set_local_selection(
        &mut self,
        anchor_row: usize,
        anchor_col: u32,
        focus_row: usize,
        focus_col: u32,
        behavior: GestureBehavior,
    ) -> Result<(), TextError> {
        self.view.gesture_press(anchor_row, anchor_col, behavior)?;
        self.view.gesture_move(focus_row, focus_col)?;
        self.anchor_cell = Some((anchor_row, anchor_col));
        self.anchor_behavior = behavior;
        if self.follow_cursor {
            self.sync_cursor_to_focus();
        }
        Ok(())
    }

    /// Move the focus of the stored anchor selection.
    pub fn update_local_selection(
        &mut self,
        focus_row: usize,
        focus_col: u32,
    ) -> Result<(), TextError> {
        let (anchor_row, anchor_col) = self.anchor_cell.ok_or(TextError::OutOfRange)?;
        self.view
            .gesture_press(anchor_row, anchor_col, self.anchor_behavior)?;
        self.view.gesture_move(focus_row, focus_col)?;
        if self.follow_cursor {
            self.sync_cursor_to_focus();
        }
        Ok(())
    }

    pub fn reset_local_selection(&mut self) {
        self.view.clear_selection();
        self.view.gesture_release();
        self.anchor_cell = None;
    }

    pub fn convert_selection_to_cell(&mut self) {
        self.view.gesture_convert_to_cell();
    }

    pub fn set_selection_occupancy(&mut self, occupancy: SelectionOccupancy) {
        self.view.set_gesture_occupancy(occupancy);
    }

    fn sync_cursor_to_focus(&mut self) {
        if let Some((_, end)) = self.view.selection() {
            let _ = self.view.set_cursor(end);
        }
    }
}

// ---- text-gaps unit tests (the runner also drives tests/text_gaps.rs) ----

#[cfg(test)]
mod gap_tests {
    use super::*;

    #[test]
    fn gesture_selection_word_unit() {
        let mut view = TextView::new(TextBuffer::from_text("alpha beta"));
        view.gesture_press(0, 7, GestureBehavior::Word).unwrap();
        assert_eq!(Some((6, 10)), view.gesture_release());
    }

    #[test]
    fn viewport_selection_offsets() {
        let mut view = TextView::new(TextBuffer::from_text("aaa\nbbb\nccc"));
        let viewport = Viewport::new(1, 0);
        view.select_viewport(viewport, 0, 0, 1, 3).unwrap();
        assert_eq!(Some("bbb\nccc"), view.selected_text());
    }

    #[test]
    fn iterators_round_trip() {
        let buf = TextBuffer::from_text("ab\ncd");
        let (row, col) = buf.offset_to_coords(4).unwrap();
        assert_eq!((1, 1), (row, col));
        assert_eq!(4, buf.coords_to_offset(row, col).unwrap());
        let texts: Vec<&str> = buf.clusters(0, 2).unwrap().iter().map(|c| c.text).collect();
        assert_eq!(vec!["a", "b"], texts);
    }

    #[test]
    fn wrap_cache_counts() {
        let mut view = TextView::new(TextBuffer::from_text("aaa bbb"));
        view.set_wrap_mode(WrapMode::Word);
        view.set_width(4);
        view.virtual_lines();
        assert_eq!(1, view.recompute_count());
        view.virtual_lines();
        assert_eq!(1, view.recompute_count());
    }

    #[test]
    fn editor_view_cursor_visible() {
        let text = (0..40)
            .map(|i| format!("line{i:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut editor = EditorView::new(TextBuffer::from_text(&text));
        editor.set_viewport_size(80, 10);
        editor.text_view_mut().set_cursor(200).unwrap();
        editor.ensure_cursor_visible(1);
        let (row, _) = editor.visual_cursor();
        let viewport = editor.viewport();
        assert!(row >= viewport.first_row);
        assert!(row < viewport.first_row + 10);
    }
}
