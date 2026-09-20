//! Segmentation and navigation (`uni` domain, UNI-004, UNI-005,
//! UNI-006, UNI-007, UNI-009).
//!
//! Ports the navigation layer of `utf8.zig` (wrap and position
//! search, previous-grapheme lookup, line breaks, tab stops, checked
//! decoding, the stateful width cursor, chunk layout) and the pool-free
//! cell-character packing of `grapheme.zig`. Measurement primitives
//! (`walk`, `WidthState`, property tables) live in the parent module;
//! the ported vectors in `tests/uni_segments.rs` decide every
//! disagreement.

use super::{WidthMethod, WidthState, char_width, cluster_ranges, walk};

/// Wrap/position search result. Mirrors `WrapByWidthResult` and
/// `PosByWidthResult`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidthSpan {
    pub byte_offset: u32,
    pub grapheme_count: u32,
    pub columns_used: u32,
}

/// Previous-grapheme lookup result. Mirrors `PrevGraphemeResult`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrevGrapheme {
    pub start_offset: usize,
    pub width: u32,
}

/// Line-break kind. Mirrors `LineBreakKind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineBreakKind {
    Lf,
    Cr,
    Crlf,
}

/// One line break: byte position and kind. CRLF is reported once, at
/// the `\n`. Mirrors `LineBreak`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineBreak {
    pub pos: usize,
    pub kind: LineBreakKind,
}

/// Cluster byte bounds under the method's boundary rules.
pub fn grapheme_breaks(text: &str, method: WidthMethod) -> Vec<(usize, usize)> {
    cluster_ranges(text, method)
}

/// Reference `findWrapPosByWidth`.
pub fn wrap_pos_by_width(
    text: &str,
    max_columns: u32,
    tab_width: u8,
    ascii_only: bool,
    method: WidthMethod,
) -> WidthSpan {
    if text.is_empty() || max_columns == 0 {
        return WidthSpan {
            byte_offset: 0,
            grapheme_count: 0,
            columns_used: 0,
        };
    }
    if ascii_only {
        let n = text.len() as u32;
        if max_columns >= n {
            return WidthSpan {
                byte_offset: n,
                grapheme_count: n,
                columns_used: n,
            };
        }
        return WidthSpan {
            byte_offset: max_columns,
            grapheme_count: max_columns,
            columns_used: max_columns,
        };
    }
    if method == WidthMethod::Wcwidth {
        return wrap_pos_wcwidth(text, max_columns, tab_width);
    }
    let (clusters, _) = walk(text, tab_width, method);
    wrap_over_clusters(&clusters, text.len(), max_columns)
}

fn wrap_pos_wcwidth(text: &str, max_columns: u32, tab_width: u8) -> WidthSpan {
    let mut pos: usize = 0;
    let mut columns: u32 = 0;
    let mut count: u32 = 0;
    for (start, ch) in text.char_indices() {
        let _ = start;
        let mut buf = [0u8; 4];
        let enc = ch.encode_utf8(&mut buf);
        let w = char_width(enc.as_bytes()[0], ch as u32, tab_width);
        if columns >= max_columns {
            return WidthSpan {
                byte_offset: pos as u32,
                grapheme_count: count,
                columns_used: columns,
            };
        }
        if columns + w > max_columns {
            return WidthSpan {
                byte_offset: pos as u32,
                grapheme_count: count,
                columns_used: columns,
            };
        }
        columns += w;
        count += 1;
        pos += enc.len();
    }
    WidthSpan {
        byte_offset: text.len() as u32,
        grapheme_count: count,
        columns_used: columns,
    }
}

/// Reference `findWrapPosByWidthGraphemeSafe`: cluster boundaries for
/// every method, wcwidth accumulation inside each cluster.
pub fn wrap_pos_grapheme_safe(
    text: &str,
    max_columns: u32,
    tab_width: u8,
    ascii_only: bool,
    method: WidthMethod,
) -> WidthSpan {
    if method == WidthMethod::Wcwidth {
        if text.is_empty() || max_columns == 0 {
            return WidthSpan {
                byte_offset: 0,
                grapheme_count: 0,
                columns_used: 0,
            };
        }
        if ascii_only {
            let n = text.len() as u32;
            if max_columns >= n {
                return WidthSpan {
                    byte_offset: n,
                    grapheme_count: n,
                    columns_used: n,
                };
            }
            return WidthSpan {
                byte_offset: max_columns,
                grapheme_count: max_columns,
                columns_used: max_columns,
            };
        }
        let (clusters, _) = walk(text, tab_width, WidthMethod::Wcwidth);
        return wrap_over_clusters(&clusters, text.len(), max_columns);
    }
    wrap_pos_by_width(text, max_columns, tab_width, ascii_only, method)
}

fn wrap_over_clusters(clusters: &[super::Walked], text_len: usize, max_columns: u32) -> WidthSpan {
    let mut columns: u32 = 0;
    let mut count: u32 = 0;
    for (i, _) in clusters.iter().enumerate() {
        if i > 0 {
            let p = &clusters[i - 1];
            if columns + p.width > max_columns {
                return WidthSpan {
                    byte_offset: p.start as u32,
                    grapheme_count: count,
                    columns_used: columns,
                };
            }
            columns += p.width;
            count += 1;
        }
    }
    if let Some(last) = clusters.last()
        && last.width > 0
    {
        if columns + last.width > max_columns {
            return WidthSpan {
                byte_offset: last.start as u32,
                grapheme_count: count,
                columns_used: columns,
            };
        }
        columns += last.width;
        count += 1;
    }
    WidthSpan {
        byte_offset: text_len as u32,
        grapheme_count: count,
        columns_used: columns,
    }
}

/// Reference `findPosByWidth`.
pub fn pos_by_width(
    text: &str,
    max_columns: u32,
    tab_width: u8,
    ascii_only: bool,
    include_start_before: bool,
    method: WidthMethod,
) -> WidthSpan {
    if text.is_empty() || max_columns == 0 {
        return WidthSpan {
            byte_offset: 0,
            grapheme_count: 0,
            columns_used: 0,
        };
    }
    if ascii_only {
        let n = text.len() as u32;
        if max_columns >= n {
            return WidthSpan {
                byte_offset: n,
                grapheme_count: n,
                columns_used: n,
            };
        }
        return WidthSpan {
            byte_offset: max_columns,
            grapheme_count: max_columns,
            columns_used: max_columns,
        };
    }
    if method == WidthMethod::Wcwidth {
        return pos_wcwidth(text, max_columns, tab_width, include_start_before);
    }
    let (clusters, _) = walk(text, tab_width, method);
    pos_over_clusters(&clusters, text.len(), max_columns, include_start_before)
}

/// Reference `findGraphemePosByWidth`: cluster walk for every method.
pub fn grapheme_pos_by_width(
    text: &str,
    max_columns: u32,
    tab_width: u8,
    ascii_only: bool,
    include_start_before: bool,
    method: WidthMethod,
) -> WidthSpan {
    if text.is_empty() || max_columns == 0 {
        return WidthSpan {
            byte_offset: 0,
            grapheme_count: 0,
            columns_used: 0,
        };
    }
    if ascii_only {
        let n = text.len() as u32;
        if max_columns >= n {
            return WidthSpan {
                byte_offset: n,
                grapheme_count: n,
                columns_used: n,
            };
        }
        return WidthSpan {
            byte_offset: max_columns,
            grapheme_count: max_columns,
            columns_used: max_columns,
        };
    }
    let (clusters, _) = walk(text, tab_width, method);
    pos_over_clusters(&clusters, text.len(), max_columns, include_start_before)
}

fn pos_over_clusters(
    clusters: &[super::Walked],
    text_len: usize,
    max_columns: u32,
    include_start_before: bool,
) -> WidthSpan {
    let mut columns: u32 = 0;
    let mut count: u32 = 0;
    // Byte offset of the cluster currently being committed; the
    // reference reports the committed cluster's own start on stop.
    for (i, _) in clusters.iter().enumerate() {
        if i > 0 {
            let p = &clusters[i - 1];
            if include_start_before {
                if columns >= max_columns {
                    return WidthSpan {
                        byte_offset: p.start as u32,
                        grapheme_count: count,
                        columns_used: columns,
                    };
                }
                columns += p.width;
                count += 1;
            } else {
                if columns + p.width > max_columns {
                    return WidthSpan {
                        byte_offset: p.start as u32,
                        grapheme_count: count,
                        columns_used: columns,
                    };
                }
                columns += p.width;
            }
        }
    }
    if let Some(last) = clusters.last()
        && last.width > 0
    {
        if columns >= max_columns {
            return WidthSpan {
                byte_offset: last.start as u32,
                grapheme_count: count,
                columns_used: columns,
            };
        }
        if !include_start_before && columns + last.width > max_columns {
            return WidthSpan {
                byte_offset: last.start as u32,
                grapheme_count: count,
                columns_used: columns,
            };
        }
        columns += last.width;
        if include_start_before {
            count += 1;
        }
    }
    WidthSpan {
        byte_offset: text_len as u32,
        grapheme_count: count,
        columns_used: columns,
    }
}

fn pos_wcwidth(
    text: &str,
    max_columns: u32,
    tab_width: u8,
    include_start_before: bool,
) -> WidthSpan {
    let mut pos: usize = 0;
    let mut columns: u32 = 0;
    let mut count: u32 = 0;
    for ch in text.chars() {
        let mut buf = [0u8; 4];
        let enc = ch.encode_utf8(&mut buf);
        let w = char_width(enc.as_bytes()[0], ch as u32, tab_width);
        let start_col = columns;
        let end_col = columns + w;
        if include_start_before {
            if start_col >= max_columns {
                return WidthSpan {
                    byte_offset: pos as u32,
                    grapheme_count: count,
                    columns_used: columns,
                };
            }
        } else if end_col > max_columns {
            return WidthSpan {
                byte_offset: pos as u32,
                grapheme_count: count,
                columns_used: columns,
            };
        }
        columns = end_col;
        count += 1;
        pos += enc.len();
    }
    WidthSpan {
        byte_offset: text.len() as u32,
        grapheme_count: count,
        columns_used: columns,
    }
}

/// Reference `getPrevGraphemeStart`.
pub fn prev_grapheme_start(
    text: &str,
    byte_offset: usize,
    tab_width: u8,
    method: WidthMethod,
) -> Option<PrevGrapheme> {
    if byte_offset == 0 || text.is_empty() || byte_offset > text.len() {
        return None;
    }
    if method == WidthMethod::Wcwidth {
        let mut pos: usize = 0;
        let mut last: Option<PrevGrapheme> = None;
        for ch in text.chars() {
            if pos >= byte_offset {
                break;
            }
            let mut buf = [0u8; 4];
            let enc = ch.encode_utf8(&mut buf);
            let w = char_width(enc.as_bytes()[0], ch as u32, tab_width);
            if w > 0 {
                last = Some(PrevGrapheme {
                    start_offset: pos,
                    width: w,
                });
            }
            pos += enc.len();
        }
        return last;
    }
    prev_grapheme_start_unicode(text, byte_offset, tab_width, method)
}

const REPLACEMENT: u32 = 0xFFFD;

/// Unicode previous-grapheme walk. U+FFFD is transparent to break
/// decisions (the reference skips break-state updates for it), so the
/// walk segments the text with U+FFFD removed and maps positions
/// back; break evolution over the valid characters is identical.
fn prev_grapheme_start_unicode(
    text: &str,
    byte_offset: usize,
    tab_width: u8,
    method: WidthMethod,
) -> Option<PrevGrapheme> {
    // Filtered coordinates: every valid char keeps its original byte
    // offset, so break evolution matches the reference exactly.
    let mut filtered = String::new();
    let mut orig_of: Vec<usize> = Vec::new();
    for (op, ch) in text.char_indices() {
        if ch as u32 == REPLACEMENT {
            continue;
        }
        for _ in 0..ch.len_utf8() {
            orig_of.push(op);
        }
        filtered.push(ch);
    }
    orig_of.push(text.len());
    if filtered.is_empty() {
        // No valid character ever breaks; the start stays at zero.
        let width = super::width_at(text, 0, tab_width, method);
        return Some(PrevGrapheme {
            start_offset: 0,
            width,
        });
    }
    // Raw standard boundaries for every method: the reference calls
    // uucode.isBreak directly here, with no method dispatch.
    let starts: std::collections::HashSet<usize> = cluster_ranges(&filtered, WidthMethod::Unicode)
        .iter()
        .map(|(s, _)| *s)
        .collect();
    let mut prev_start: usize = 0;
    let mut second_to_last: usize = 0;
    let mut first = true;
    for (fi, _) in filtered.char_indices() {
        if orig_of[fi] >= byte_offset {
            break;
        }
        if first || starts.contains(&fi) {
            second_to_last = prev_start;
            prev_start = fi;
            first = false;
        }
    }
    let prev_orig = orig_of[prev_start];
    let second_orig = orig_of[second_to_last];
    let start_offset = if prev_orig < byte_offset {
        prev_orig
    } else {
        second_orig
    };
    let width = super::width_at(text, start_offset, tab_width, method);
    Some(PrevGrapheme {
        start_offset,
        width,
    })
}

/// Reference `findLineBreaks`: byte scan where CRLF is one break
/// recorded at the `\n`. The vector-chunk edge handling collapses to
/// this scalar form with identical output.
pub fn line_breaks(text: &str) -> Vec<LineBreak> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i: usize = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    out.push(LineBreak {
                        pos: i + 1,
                        kind: LineBreakKind::Crlf,
                    });
                    i += 2;
                } else {
                    out.push(LineBreak {
                        pos: i,
                        kind: LineBreakKind::Cr,
                    });
                    i += 1;
                }
            }
            b'\n' => {
                out.push(LineBreak {
                    pos: i,
                    kind: LineBreakKind::Lf,
                });
                i += 1;
            }
            _ => i += 1,
        }
    }
    out
}

/// Reference `findTabStops`: byte offsets of every tab.
pub fn tab_stops(text: &str) -> Vec<usize> {
    text.bytes()
        .enumerate()
        .filter_map(|(i, b)| (b == b'\t').then_some(i))
        .collect()
}

/// Strict UTF-8 decoding failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    OutOfBounds,
    UnexpectedEnd,
    InvalidStart,
    InvalidContinuation,
    Overlong,
    Surrogate,
    TooLarge,
}

/// Checked UTF-8 decode at a byte offset. Valid sequences decode
/// exactly like the reference unchecked decoder; anything else
/// (truncated, overlong, surrogate, out of range, bad continuation)
/// returns an error instead of reading on regardless.
pub fn decode_utf8_at(bytes: &[u8], pos: usize) -> Result<(char, u8), DecodeError> {
    let b0 = *bytes.get(pos).ok_or(DecodeError::OutOfBounds)?;
    if b0 < 0x80 {
        return Ok((b0 as char, 1));
    }
    let (min, len) = if b0 & 0xE0 == 0xC0 {
        (0x80u32, 2u8)
    } else if b0 & 0xF0 == 0xE0 {
        (0x800u32, 3u8)
    } else if b0 & 0xF8 == 0xF0 {
        (0x10000u32, 4u8)
    } else {
        return Err(DecodeError::InvalidStart);
    };
    if pos + len as usize > bytes.len() {
        return Err(DecodeError::UnexpectedEnd);
    }
    let mut cp: u32 = (b0 & (0xFF >> (len + 1))) as u32;
    for k in 1..len {
        let b = bytes[pos + k as usize];
        if b & 0xC0 != 0x80 {
            return Err(DecodeError::InvalidContinuation);
        }
        cp = (cp << 6) | (b & 0x3F) as u32;
    }
    if cp < min {
        return Err(DecodeError::Overlong);
    }
    if (0xD800..0xE000).contains(&cp) {
        return Err(DecodeError::Surrogate);
    }
    if cp > 0x10FFFF {
        return Err(DecodeError::TooLarge);
    }
    char::from_u32(cp)
        .map(|ch| (ch, len))
        .ok_or(DecodeError::TooLarge)
}

/// Reference `TextWidthCursor`: a stateful column cursor advanced in
/// pieces. Break decisions match whole-text segmentation: the
/// reference threads break state across calls, which evolves over
/// the consumed prefix exactly as the segmenter did, so resuming
/// inside the containing cluster and accumulating forward from the
/// offset is observably identical.
pub struct TextWidthCursor<'a> {
    text: &'a str,
    tab_width: u8,
    method: WidthMethod,
    byte_offset: usize,
    columns: u32,
}

impl<'a> TextWidthCursor<'a> {
    pub fn new(text: &'a str, tab_width: u8, method: WidthMethod) -> TextWidthCursor<'a> {
        TextWidthCursor {
            text,
            tab_width,
            method,
            byte_offset: 0,
            columns: 0,
        }
    }

    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    pub fn columns(&self) -> u32 {
        self.columns
    }

    /// Advance to `byte_end`, returning total columns. Offsets past
    /// the end clamp; a non-boundary offset snaps forward to the next
    /// boundary (offsets from whole-character advances always are).
    pub fn advance_to(&mut self, byte_end: usize) -> u32 {
        let end = byte_end.min(self.text.len());
        if end <= self.byte_offset {
            return self.columns;
        }
        let mut off = self.byte_offset;
        while off < self.text.len() && !self.text.is_char_boundary(off) {
            off += 1;
        }
        let ranges = cluster_ranges(self.text, self.method);
        let mut idx = 0;
        while idx < ranges.len() && ranges[idx].1 <= off {
            idx += 1;
        }
        while off < end {
            if idx < ranges.len() && ranges[idx].0 < off {
                // Resuming inside a cluster: accumulate forward from
                // the offset exactly as the reference slow path does.
                let (start, stop) = ranges[idx];
                let _ = start;
                let slice = &self.text[off..stop.min(self.text.len())];
                let mut chars = slice.chars();
                let first = chars.next().unwrap_or('\0');
                let mut buf = [0u8; 4];
                let enc = first.encode_utf8(&mut buf);
                let mut state = WidthState::init(
                    first as u32,
                    char_width(enc.as_bytes()[0], first as u32, self.tab_width),
                    self.method,
                );
                let mut adv = enc.len();
                for ch in chars {
                    let cp = ch as u32;
                    let mut buf = [0u8; 4];
                    let enc = ch.encode_utf8(&mut buf);
                    state.add(cp, char_width(enc.as_bytes()[0], cp, self.tab_width));
                    adv += enc.len();
                }
                self.columns += state.width;
                off += adv;
                idx += 1;
                continue;
            }
            if idx < ranges.len() {
                let (start, stop) = ranges[idx];
                if start >= end {
                    break;
                }
                let (_, w) = cluster_width_at(&ranges, idx, self.text, self.tab_width, self.method);
                let _ = stop;
                self.columns += w;
                off = stop;
                idx += 1;
            } else {
                break;
            }
        }
        self.byte_offset = off.min(self.text.len());
        self.columns
    }
}

/// Word class for wrap and word-motion decisions. Mirrors `WordClass`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordClass {
    AsciiWord,
    CjkWord,
    Other,
}

/// Wrap-break kind. Mirrors `LayoutWrapBreakKind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutWrapBreakKind {
    None,
    Whitespace,
    PreservedWhitespace,
    Punctuation,
    ScriptTransition,
    CjkIntercharacter,
}

impl LayoutWrapBreakKind {
    /// True when the break also ends a word; line-only breaks return
    /// false. Mirrors `isWordBoundary`.
    pub fn is_word_boundary(self) -> bool {
        match self {
            LayoutWrapBreakKind::Whitespace
            | LayoutWrapBreakKind::PreservedWhitespace
            | LayoutWrapBreakKind::Punctuation
            | LayoutWrapBreakKind::ScriptTransition => true,
            LayoutWrapBreakKind::None | LayoutWrapBreakKind::CjkIntercharacter => false,
        }
    }
}

/// One wrap opportunity: the grapheme that creates the break, with
/// byte and column metadata. Mirrors `LayoutWrapBreak`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutWrapBreak {
    pub byte_start: u32,
    pub col_start: u32,
    pub byte_len: u32,
    pub width_cols: u32,
    pub kind: LayoutWrapBreakKind,
}

impl LayoutWrapBreak {
    pub fn col_end(self) -> u32 {
        self.col_start + self.width_cols
    }

    pub fn byte_end(self) -> u32 {
        self.byte_start + self.byte_len
    }
}

/// Chunk endpoint metadata. Mirrors `WordClassEdges`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordClassEdges {
    pub first: WordClass,
    pub last: WordClass,
    pub last_cp: Option<u32>,
    pub has_cjk_breaks: bool,
}

fn is_ascii_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_cjk_word_codepoint(cp: u32) -> bool {
    (0x3400..=0x4DBF).contains(&cp)
        || (0x4E00..=0x9FFF).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0x20000..=0x2A6DF).contains(&cp)
        || (0x2A700..=0x2B73F).contains(&cp)
        || (0x2B740..=0x2B81F).contains(&cp)
        || (0x2B820..=0x2CEAF).contains(&cp)
        || (0x2CEB0..=0x2EBEF).contains(&cp)
        || (0x2EBF0..=0x2EE5D).contains(&cp)
        || (0x2F800..=0x2FA1F).contains(&cp)
        || (0x3040..=0x309F).contains(&cp)
        || (0x30A0..=0x30FF).contains(&cp)
        || (0x31F0..=0x31FF).contains(&cp)
        || (0xFF66..=0xFF9D).contains(&cp)
        || (0x1100..=0x11FF).contains(&cp)
        || (0x3130..=0x318F).contains(&cp)
        || (0xA960..=0xA97F).contains(&cp)
        || (0xAC00..=0xD7AF).contains(&cp)
        || (0xD7B0..=0xD7FF).contains(&cp)
}

fn classify_word_class(cp: u32) -> WordClass {
    if cp <= 0x7F {
        if is_ascii_word_byte(cp as u8) {
            return WordClass::AsciiWord;
        }
        return WordClass::Other;
    }
    if is_cjk_word_codepoint(cp) {
        return WordClass::CjkWord;
    }
    WordClass::Other
}

/// Reference `isWordCodepoint`.
pub fn is_word_codepoint(cp: u32) -> bool {
    classify_word_class(cp) != WordClass::Other
}

/// Reference `isCjkIntercharacterBreak`.
fn is_cjk_intercharacter_break(prev: WordClass, curr: WordClass, curr_cp: u32) -> bool {
    prev == WordClass::CjkWord
        && curr == WordClass::CjkWord
        && curr_cp != 0x3099
        && curr_cp != 0x309A
        && curr_cp != 0x30A0
        && curr_cp != 0x30FB
}

/// Reference `isCjkAsciiTransition`.
fn is_cjk_ascii_transition(prev: WordClass, curr: WordClass) -> bool {
    (prev == WordClass::CjkWord && curr == WordClass::AsciiWord)
        || (prev == WordClass::AsciiWord && curr == WordClass::CjkWord)
}

fn is_ascii_wrap_break(b: u8) -> bool {
    matches!(
        b,
        b' ' | b'\t'
            | b'-'
            | b'/'
            | b'\\'
            | b'.'
            | b','
            | b';'
            | b':'
            | b'!'
            | b'?'
            | b'('
            | b')'
            | b'['
            | b']'
            | b'{'
            | b'}'
    )
}

fn ascii_wrap_break_kind(b: u8) -> LayoutWrapBreakKind {
    match b {
        b' ' | b'\t' => LayoutWrapBreakKind::Whitespace,
        b'-' | b'/' | b'\\' | b'.' | b',' | b';' | b':' | b'!' | b'?' | b'(' | b')' | b'['
        | b']' | b'{' | b'}' => LayoutWrapBreakKind::Punctuation,
        _ => LayoutWrapBreakKind::None,
    }
}

fn unicode_wrap_break_kind(cp: u32) -> LayoutWrapBreakKind {
    match cp {
        0x00A0 | 0x1680 | 0x2000..=0x200A | 0x202F | 0x205F | 0x3000 | 0x200B => {
            LayoutWrapBreakKind::Whitespace
        }
        0x00AD | 0x2010 | 0x3001 | 0x3002 | 0xFF01 | 0xFF0C | 0xFF1A | 0xFF1F => {
            LayoutWrapBreakKind::Punctuation
        }
        _ => LayoutWrapBreakKind::None,
    }
}

/// Reference `chunkWordClassEdges`: endpoint classes without scanning
/// contents.
pub fn chunk_word_class_edges(text: &str) -> WordClassEdges {
    if text.is_empty() {
        return WordClassEdges {
            first: WordClass::Other,
            last: WordClass::Other,
            last_cp: None,
            has_cjk_breaks: false,
        };
    }
    let first = text.chars().next().unwrap_or('\0') as u32;
    let last = text.chars().next_back().unwrap_or('\0') as u32;
    WordClassEdges {
        first: classify_word_class(first),
        last: classify_word_class(last),
        last_cp: Some(last),
        has_cjk_breaks: false,
    }
}

struct PendingCluster {
    start: usize,
    end: usize,
    col_offset: u32,
    width: u32,
    kind: LayoutWrapBreakKind,
    class: WordClass,
}

fn commit_cluster(
    prev: &PendingCluster,
    next_class: Option<WordClass>,
    next_cp: u32,
    word_only: bool,
    has_cjk_breaks: &mut bool,
    out: &mut Vec<LayoutWrapBreak>,
) {
    let kind = if prev.kind != LayoutWrapBreakKind::None {
        prev.kind
    } else if let Some(next) = next_class {
        if is_cjk_ascii_transition(prev.class, next) {
            LayoutWrapBreakKind::ScriptTransition
        } else if prev.width > 0 && is_cjk_intercharacter_break(prev.class, next, next_cp) {
            *has_cjk_breaks = true;
            if word_only {
                LayoutWrapBreakKind::None
            } else {
                LayoutWrapBreakKind::CjkIntercharacter
            }
        } else {
            LayoutWrapBreakKind::None
        }
    } else {
        LayoutWrapBreakKind::None
    };
    if kind != LayoutWrapBreakKind::None {
        out.push(LayoutWrapBreak {
            byte_start: prev.start as u32,
            col_start: prev.col_offset,
            byte_len: (prev.end - prev.start) as u32,
            width_cols: prev.width,
            kind,
        });
    }
}

fn merge_kind(current: LayoutWrapBreakKind, next: LayoutWrapBreakKind) -> LayoutWrapBreakKind {
    if current == LayoutWrapBreakKind::Whitespace || next == LayoutWrapBreakKind::Whitespace {
        if current == LayoutWrapBreakKind::Whitespace && next == LayoutWrapBreakKind::Whitespace {
            LayoutWrapBreakKind::Whitespace
        } else {
            LayoutWrapBreakKind::PreservedWhitespace
        }
    } else if current == LayoutWrapBreakKind::None {
        next
    } else {
        current
    }
}

fn walk_chunk(
    text: &str,
    tab_width: u8,
    ascii_only: bool,
    method: WidthMethod,
    word_only: bool,
) -> (Vec<LayoutWrapBreak>, WordClassEdges) {
    if text.is_empty() {
        return (
            Vec::new(),
            WordClassEdges {
                first: WordClass::Other,
                last: WordClass::Other,
                last_cp: None,
                has_cjk_breaks: false,
            },
        );
    }
    if ascii_only {
        let mut out = Vec::new();
        for (pos, b) in text.bytes().enumerate() {
            if b == b' ' {
                out.push(LayoutWrapBreak {
                    byte_start: pos as u32,
                    col_start: pos as u32,
                    byte_len: 1,
                    width_cols: 1,
                    kind: LayoutWrapBreakKind::Whitespace,
                });
            } else if is_ascii_wrap_break(b) {
                out.push(LayoutWrapBreak {
                    byte_start: pos as u32,
                    col_start: pos as u32,
                    byte_len: 1,
                    width_cols: 1,
                    kind: ascii_wrap_break_kind(b),
                });
            }
        }
        let mut edges = chunk_word_class_edges(text);
        edges.has_cjk_breaks = false;
        return (out, edges);
    }
    let first_cp = text.chars().next().unwrap_or('\0') as u32;
    let first_class = classify_word_class(first_cp);
    let (clusters, _) = walk(text, tab_width, method);
    let mut out = Vec::new();
    let mut has_cjk_breaks = false;
    let mut open: Option<PendingCluster> = None;
    let mut last_class = WordClass::Other;
    for c in &clusters {
        let slice = &text[c.start..c.start + c.len];
        let mut chars = slice.chars();
        let fc = chars.next().unwrap_or('\0') as u32;
        let fkind = if text.as_bytes()[c.start] < 0x80 {
            ascii_wrap_break_kind(text.as_bytes()[c.start])
        } else {
            unicode_wrap_break_kind(fc)
        };
        let fclass = classify_word_class(fc);
        let mut kind = fkind;
        for ch in chars {
            let cp = ch as u32;
            let nk = if cp <= 0x7F {
                ascii_wrap_break_kind(cp as u8)
            } else {
                unicode_wrap_break_kind(cp)
            };
            kind = merge_kind(kind, nk);
        }
        if let Some(prev) = open.take() {
            commit_cluster(
                &prev,
                Some(fclass),
                fc,
                word_only,
                &mut has_cjk_breaks,
                &mut out,
            );
        }
        open = Some(PendingCluster {
            start: c.start,
            end: c.start + c.len,
            col_offset: c.col_start,
            width: c.width,
            kind,
            class: fclass,
        });
        last_class = fclass;
    }
    if let Some(prev) = open.take() {
        commit_cluster(&prev, None, 0, word_only, &mut has_cjk_breaks, &mut out);
    }
    let edges = WordClassEdges {
        first: first_class,
        last: last_class,
        last_cp: text.chars().next_back().map(|c| c as u32),
        has_cjk_breaks,
    };
    (out, edges)
}

/// Reference `findChunkLayoutInfo`.
pub fn chunk_layout_info(
    text: &str,
    tab_width: u8,
    ascii_only: bool,
    method: WidthMethod,
) -> (Vec<LayoutWrapBreak>, WordClassEdges) {
    walk_chunk(text, tab_width, ascii_only, method, false)
}

/// Reference `findWordChunkLayoutInfo`: line-only CJK opportunities
/// are not emitted (but still flagged).
pub fn word_chunk_layout_info(
    text: &str,
    tab_width: u8,
    ascii_only: bool,
    method: WidthMethod,
) -> (Vec<LayoutWrapBreak>, WordClassEdges) {
    walk_chunk(text, tab_width, ascii_only, method, true)
}

/// Reference `findLineAndWordChunkLayoutInfo`: disjoint line-only and
/// word break lists.
pub fn line_and_word_chunk_layout_info(
    text: &str,
    tab_width: u8,
    ascii_only: bool,
    method: WidthMethod,
) -> (Vec<LayoutWrapBreak>, Vec<LayoutWrapBreak>, WordClassEdges) {
    let (all, edges) = walk_chunk(text, tab_width, ascii_only, method, false);
    let mut lines = Vec::new();
    let mut words = Vec::new();
    for b in all {
        if b.kind.is_word_boundary() {
            words.push(b);
        } else {
            lines.push(b);
        }
    }
    (lines, words, edges)
}

// ---- Cell-character packing (UNI-009) ----
// Bit layout of the u32 stored per cell, ported from `grapheme.zig`.
// Bits 31-30 select plain (00), image (01), grapheme-start (10), or
// continuation (11); the low 26 bits carry the image or grapheme id.

/// Image cell flag (01 in bits 31-30).
pub const CHAR_FLAG_IMAGE: u32 = 0x4000_0000;
/// Grapheme-start cell flag (10 in bits 31-30).
pub const CHAR_FLAG_GRAPHEME: u32 = 0x8000_0000;
/// Continuation cell flag (11 in bits 31-30).
pub const CHAR_FLAG_CONTINUATION: u32 = 0xC000_0000;
/// Top-two-bits selector mask.
pub const CHAR_TYPE_MASK: u32 = 0xC000_0000;
/// Low 26 bits: image or grapheme id payload.
pub const IMAGE_ID_MASK: u32 = 0x03FF_FFFF;
/// Grapheme id payload mask (class 3 bits, generation 7, slot 16).
pub const GRAPHEME_ID_MASK: u32 = 0x03FF_FFFF;
/// Right-extent bit shift (bits 29-28).
pub const CHAR_EXT_RIGHT_SHIFT: u32 = 28;
/// Left-extent bit shift (bits 27-26).
pub const CHAR_EXT_LEFT_SHIFT: u32 = 26;
/// Two-bit extent mask.
pub const CHAR_EXT_MASK: u32 = 0x3;

/// Reference `packImageCell`.
pub fn pack_image_cell(id: u32, fallback: u8) -> u32 {
    debug_assert!(id <= IMAGE_ID_MASK);
    debug_assert!(fallback < 16);
    CHAR_FLAG_IMAGE | (id << 4) | fallback as u32
}

/// Reference `isImageChar`.
pub fn is_image_char(char: u32) -> bool {
    char & CHAR_TYPE_MASK == CHAR_FLAG_IMAGE
}

/// Reference `imageIdFromChar`.
pub fn image_id_from_char(char: u32) -> u32 {
    debug_assert!(is_image_char(char));
    (char >> 4) & IMAGE_ID_MASK
}

/// Reference `imageFallbackFromChar`.
pub fn image_fallback_from_char(char: u32) -> u8 {
    debug_assert!(is_image_char(char));
    (char & 0xF) as u8
}

/// Reference `isGraphemeChar`.
pub fn is_grapheme_char(char: u32) -> bool {
    char & CHAR_TYPE_MASK == CHAR_FLAG_GRAPHEME
}

/// Reference `isContinuationChar`.
pub fn is_continuation_char(char: u32) -> bool {
    char & CHAR_TYPE_MASK == CHAR_FLAG_CONTINUATION
}

/// Reference `isClusterChar`.
pub fn is_cluster_char(char: u32) -> bool {
    char & 0x8000_0000 == 0x8000_0000
}

/// Reference `graphemeIdFromChar`.
pub fn grapheme_id_from_char(char: u32) -> u32 {
    debug_assert!(is_cluster_char(char));
    char & GRAPHEME_ID_MASK
}

/// Reference `charRightExtent`.
pub fn char_right_extent(char: u32) -> u32 {
    debug_assert!(is_cluster_char(char));
    (char >> CHAR_EXT_RIGHT_SHIFT) & CHAR_EXT_MASK
}

/// Reference `charLeftExtent`.
pub fn char_left_extent(char: u32) -> u32 {
    debug_assert!(is_cluster_char(char));
    (char >> CHAR_EXT_LEFT_SHIFT) & CHAR_EXT_MASK
}

/// Reference `packGraphemeStart`.
pub fn pack_grapheme_start(gid: u32, total_width: u32) -> u32 {
    debug_assert!(gid <= GRAPHEME_ID_MASK);
    debug_assert!(total_width > 0);
    let width_minus_one = (total_width - 1).min(CHAR_EXT_MASK);
    CHAR_FLAG_GRAPHEME
        | ((width_minus_one & CHAR_EXT_MASK) << CHAR_EXT_RIGHT_SHIFT)
        | (gid & GRAPHEME_ID_MASK)
}

/// Reference `packContinuation`.
pub fn pack_continuation(left: u32, right: u32, gid: u32) -> u32 {
    debug_assert!(gid <= GRAPHEME_ID_MASK);
    debug_assert!(left <= CHAR_EXT_MASK);
    debug_assert!(right <= CHAR_EXT_MASK);
    CHAR_FLAG_CONTINUATION
        | ((left & CHAR_EXT_MASK) << CHAR_EXT_LEFT_SHIFT)
        | ((right & CHAR_EXT_MASK) << CHAR_EXT_RIGHT_SHIFT)
        | (gid & GRAPHEME_ID_MASK)
}

/// Reference `encodedCharWidth`.
pub fn encoded_char_width(char: u32) -> u32 {
    if is_continuation_char(char) {
        char_left_extent(char) + 1 + char_right_extent(char)
    } else if is_grapheme_char(char) {
        char_right_extent(char) + 1
    } else {
        1
    }
}

/// Width of one precomputed cluster, re-accumulated from its start.
fn cluster_width_at(
    ranges: &[(usize, usize)],
    idx: usize,
    text: &str,
    tab_width: u8,
    method: WidthMethod,
) -> (usize, u32) {
    let (start, stop) = ranges[idx];
    let slice = &text[start..stop];
    let mut chars = slice.chars();
    let first = chars.next().unwrap_or('\0');
    let mut buf = [0u8; 4];
    let enc = first.encode_utf8(&mut buf);
    let mut state = WidthState::init(
        first as u32,
        char_width(enc.as_bytes()[0], first as u32, tab_width),
        method,
    );
    for ch in chars {
        let cp = ch as u32;
        let mut buf = [0u8; 4];
        let enc = ch.encode_utf8(&mut buf);
        state.add(cp, char_width(enc.as_bytes()[0], cp, tab_width));
    }
    (stop, state.width)
}
