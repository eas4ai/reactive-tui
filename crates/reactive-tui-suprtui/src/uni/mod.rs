//! Unicode width layer (`uni` domain, UNI-001, UNI-002, UNI-003).
//!
//! Ports the width behavior of `utf8.zig`: `eastAsianWidth` /
//! `eawToWidth` for code points, `GraphemeWidthState` for cluster
//! accumulation, and `calculateTextWidth` / `findRenderClusterInfo`
//! for text measurement. Property data comes from UCD 17.0.0 tables
//! in `uni_tables.rs` (extracted verbatim) and the
//! `unicode-properties` / `unicode-segmentation` crates; the ported
//! reference vectors in `tests` decide every disagreement.

#[path = "../uni_tables.rs"]
mod tables;

pub mod pool;
pub mod segments;

use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
use unicode_segmentation::UnicodeSegmentation;

/// Width method, matching the reference discriminants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WidthMethod {
    Wcwidth = 0,
    Unicode = 1,
    NoZwj = 2,
    UnicodeWide = 3,
}

/// One non-ASCII or tab render cluster: byte span, cell width, and
/// starting column. Mirrors `RenderClusterInfo`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCluster {
    pub byte_start: u32,
    pub byte_len: u32,
    pub width_cols: u32,
    pub col_start: u32,
}

fn general_category(cp: u32) -> Option<GeneralCategory> {
    char::from_u32(cp).map(|ch| ch.general_category())
}

fn is_mark(cp: u32) -> bool {
    matches!(
        general_category(cp),
        Some(
            GeneralCategory::NonspacingMark
                | GeneralCategory::SpacingMark
                | GeneralCategory::EnclosingMark
        )
    )
}

/// Reference `eawToWidth`, returning -1 for controls that never
/// advance the cursor.
fn codepoint_width(cp: u32) -> i32 {
    if cp > 0x10FFFF {
        return 0;
    }
    if cp == 0 {
        return 0;
    }
    if cp < 32 || (0x7F..0xA0).contains(&cp) {
        return -1;
    }
    if is_mark(cp) {
        return 0;
    }
    if tables::ref_explicit_zero(cp) {
        return 0;
    }
    if tables::eaw_wide_or_full(cp) || default_wide_block(cp) {
        return 2;
    }
    if tables::ref_explicit_wide(cp) {
        return 2;
    }
    1
}

/// UCD rule: undesignated code points in these blocks default to
/// Wide. Verified exact for UCD 17.0.0: no listed entry inside them
/// says otherwise.
fn default_wide_block(cp: u32) -> bool {
    (0x3400..=0x4DBF).contains(&cp)
        || (0x4E00..=0x9FFF).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0x20000..=0x2FFFD).contains(&cp)
        || (0x30000..=0x3FFFD).contains(&cp)
}

/// Width of one code point in cells. The method is accepted for
/// dispatch symmetry; code-point width itself is method-independent
/// in the reference, and divergence happens at cluster level.
pub fn cell_width(cp: u32, _method: WidthMethod) -> u32 {
    codepoint_width(cp).max(0) as u32
}

pub(crate) fn char_width(byte: u8, cp: u32, tab_width: u8) -> u32 {
    if byte == b'\t' {
        return tab_width as u32;
    }
    if byte < 0x80 && (32..=126).contains(&byte) {
        return 1;
    }
    if byte >= 0x80 {
        return codepoint_width(cp).max(0) as u32;
    }
    0
}

/// uucode standalone width of zero, used for `unicode_wide`.
fn standalone_zero(cp: u32) -> bool {
    if cp == 0x00AD {
        return false;
    }
    if matches!(
        general_category(cp),
        Some(
            GeneralCategory::Control
                | GeneralCategory::Surrogate
                | GeneralCategory::LineSeparator
                | GeneralCategory::ParagraphSeparator
        )
    ) {
        return true;
    }
    tables::default_ignorable(cp)
}

fn is_emoji_modifier(cp: u32) -> bool {
    (0x1F3FB..=0x1F3FF).contains(&cp)
}

fn is_jamo_v(cp: u32) -> bool {
    (0x1160..=0x11A7).contains(&cp) || (0xD7B0..=0xD7C6).contains(&cp)
}

fn is_jamo_t(cp: u32) -> bool {
    (0x11A8..=0x11FF).contains(&cp) || (0xD7CB..=0xD7FB).contains(&cp)
}

/// uucode `wcwidth_zero_in_grapheme`, derived rule ported verbatim.
fn zero_in_grapheme(cp: u32) -> bool {
    if standalone_zero(cp) {
        return true;
    }
    if is_emoji_modifier(cp) {
        return true;
    }
    if matches!(
        general_category(cp),
        Some(GeneralCategory::NonspacingMark | GeneralCategory::EnclosingMark)
    ) {
        return true;
    }
    if is_jamo_v(cp) || is_jamo_t(cp) {
        return true;
    }
    tables::gcb_prepend(cp)
}

fn is_regional_indicator(cp: u32) -> bool {
    (0x1F1E6..=0x1F1FF).contains(&cp)
}

fn is_devanagari_base(cp: u32) -> bool {
    (0x0915..=0x0939).contains(&cp) || (0x0958..=0x095F).contains(&cp)
}

/// Reference `GraphemeWidthState`, ported arm for arm.
pub(crate) struct WidthState {
    pub(crate) width: u32,
    has_width: bool,
    ri_pair: bool,
    vs16: bool,
    virama: bool,
    method: WidthMethod,
}

impl WidthState {
    pub(crate) fn init(first_cp: u32, first_width: u32, method: WidthMethod) -> WidthState {
        WidthState {
            width: first_width,
            has_width: first_width > 0,
            ri_pair: is_regional_indicator(first_cp),
            vs16: false,
            virama: false,
            method,
        }
    }

    pub(crate) fn add(&mut self, cp: u32, cp_width: u32) {
        if self.method == WidthMethod::Wcwidth {
            let w = codepoint_width(cp);
            if w > 0 {
                self.width += w as u32;
                self.has_width = true;
            }
            return;
        }
        if cp == 0xFE0F {
            self.vs16 = true;
            if self.has_width && self.width == 1 {
                self.width = 2;
            }
            return;
        }
        if matches!(general_category(cp), Some(GeneralCategory::NonspacingMark)) {
            self.virama = true;
            return;
        }
        if self.ri_pair && is_regional_indicator(cp) {
            self.width += cp_width;
            self.has_width = true;
        } else if !self.has_width && cp_width > 0 {
            self.width = cp_width;
            self.has_width = true;
        } else if self.has_width
            && ((self.method == WidthMethod::UnicodeWide && !zero_in_grapheme(cp))
                || (tables::gcb_spacing_mark(cp) && cp_width > 0))
        {
            // Reference keeps the unicode_wide promotion and the
            // spacing-mark widening as separate arms with one shared,
            // side-effect-free body, so they merge here unchanged.
            self.width = self.width.max(2);
        } else if self.has_width && self.virama && is_devanagari_base(cp) && cp_width > 0 {
            if cp != 0x0930 {
                self.width += cp_width;
            }
            self.virama = false;
        }
    }
}

const ZWJ: u32 = 0x200D;
const REPLACEMENT: u32 = 0xFFFD;

/// Byte ranges of render clusters under the method's boundary rules:
/// standard extended clusters, forced joints around U+FFFD, and
/// `no_zwj` splits after ZWJ.
pub(crate) fn cluster_ranges(text: &str, method: WidthMethod) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for cluster in text.grapheme_indices(true) {
        let (start, s) = cluster;
        let mut cuts: Vec<usize> = vec![start];
        let mut prev: Option<u32> = None;
        for (i, ch) in s.char_indices() {
            let cp = ch as u32;
            let abs = start + i;
            if let Some(p) = prev
                && (cp == REPLACEMENT
                    || p == REPLACEMENT
                    || (method == WidthMethod::NoZwj && p == ZWJ))
            {
                cuts.push(abs);
            }
            prev = Some(cp);
        }
        cuts.push(start + s.len());
        for w in cuts.windows(2) {
            out.push((w[0], w[1]));
        }
    }
    out
}

pub(crate) struct Walked {
    pub(crate) start: usize,
    pub(crate) len: usize,
    pub(crate) width: u32,
    pub(crate) col_start: u32,
    pub(crate) multibyte: bool,
    pub(crate) tab: bool,
}

pub(crate) fn walk(text: &str, tab_width: u8, method: WidthMethod) -> (Vec<Walked>, u32) {
    let mut clusters = Vec::new();
    let mut col: u32 = 0;
    for (start, end) in cluster_ranges(text, method) {
        let slice = &text[start..end];
        let mut chars = slice.chars();
        let first = chars.next().unwrap_or('\0');
        let mut buf = [0u8; 4];
        let first_enc = first.encode_utf8(&mut buf);
        let mut state = WidthState::init(
            first as u32,
            char_width(first_enc.as_bytes()[0], first as u32, tab_width),
            method,
        );
        let multibyte = slice.bytes().any(|b| b >= 0x80);
        let tab = slice == "\t";
        for ch in chars {
            let cp = ch as u32;
            let mut buf = [0u8; 4];
            let enc = ch.encode_utf8(&mut buf);
            state.add(cp, char_width(enc.as_bytes()[0], cp, tab_width));
        }
        let col_start = col;
        col += state.width;
        clusters.push(Walked {
            start,
            len: end - start,
            width: state.width,
            col_start,
            multibyte,
            tab,
        });
    }
    (clusters, col)
}

/// Reference `calculateTextWidth`.
pub fn calculate_text_width(
    text: &str,
    tab_width: u8,
    ascii_only: bool,
    method: WidthMethod,
) -> u32 {
    if text.is_empty() {
        return 0;
    }
    if ascii_only {
        return text.len() as u32;
    }
    if method == WidthMethod::Wcwidth {
        let mut total: u32 = 0;
        for ch in text.chars() {
            let mut buf = [0u8; 4];
            let enc = ch.encode_utf8(&mut buf);
            total += char_width(enc.as_bytes()[0], ch as u32, tab_width);
        }
        return total;
    }
    walk(text, tab_width, method).1
}

/// Reference `findRenderClusterInfo`: non-ASCII or tab clusters only.
pub fn render_clusters(
    text: &str,
    tab_width: u8,
    ascii_only: bool,
    method: WidthMethod,
) -> Vec<RenderCluster> {
    if method != WidthMethod::Wcwidth && ascii_only {
        return Vec::new();
    }
    if text.is_empty() {
        return Vec::new();
    }
    walk(text, tab_width, method)
        .0
        .into_iter()
        .filter(|c| (c.multibyte || c.tab) && (c.width > 0 || method == WidthMethod::Wcwidth))
        .map(|c| RenderCluster {
            byte_start: c.start as u32,
            byte_len: c.len as u32,
            width_cols: c.width,
            col_start: c.col_start,
        })
        .collect()
}

/// Reference `getWidthAt`: width of the cluster containing the byte
/// offset, measured forward from the offset itself.
pub fn width_at(text: &str, byte_offset: usize, tab_width: u8, method: WidthMethod) -> u32 {
    if byte_offset >= text.len() {
        return 0;
    }
    if method == WidthMethod::Wcwidth {
        let ch = text[byte_offset..].chars().next().unwrap_or('\0');
        let mut buf = [0u8; 4];
        let enc = ch.encode_utf8(&mut buf);
        return char_width(enc.as_bytes()[0], ch as u32, tab_width);
    }
    for (start, end) in cluster_ranges(text, method) {
        if start <= byte_offset && byte_offset < end {
            let slice = &text[byte_offset..end];
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
            return state.width;
        }
    }
    0
}

/// Reference `isAsciiOnly`: nonempty printable ASCII only.
pub fn is_ascii_only(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    bytes.iter().all(|b| (32..=126).contains(b))
}

/// UNI-008 falsifier, spec-literal path `uni::pool_isolation`: two
/// independently created pools must not observe each other's entries,
/// and no process-global pool exists to leak through.
#[cfg(test)]
#[test]
fn pool_isolation() {
    use pool::GraphemePool;

    let mut first = GraphemePool::new();
    let mut second = GraphemePool::new();
    let id_first = first.alloc(b"pool-one").unwrap();
    first.incref(id_first).unwrap();
    let id_second = second.alloc(b"pool-two").unwrap();
    second.incref(id_second).unwrap();

    assert_eq!(b"pool-one", first.get(id_first).unwrap());
    assert_eq!(b"pool-two", second.get(id_second).unwrap());
    // Same slot layout, same generation: a foreign id must still not
    // resolve to the other pool's bytes.
    assert_eq!(id_first, id_second);
    assert_eq!(b"pool-two", second.get(id_first).unwrap());
    assert_ne!(b"pool-one", second.get(id_first).unwrap());

    first.decref(id_first).unwrap();
    second.decref(id_second).unwrap();
}
