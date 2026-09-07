//! Cell buffer core, ported from the reference `buffer.zig` (BUF-001 … BUF-007).
//!
//! Storage, bounds, resize, and clear plus the cell-write paths (`set`,
//! `set_raw`, `sync_cell`) with grapheme-span cleanup and link tracking.
//! Drawing, blending, scissor rectangles beyond point checks, ANSI
//! emission, compositing, and image materialization arrive with the
//! `buffer-draw` commitment that tests them.
//!
//! Image placements keep their geometry and handle here; decoded image
//! pixels stay with the media commitment. `clear` drops placement
//! geometry, so no placement survives it.

use crate::ansi::{self, Rgba, TextAttributes};
use crate::link::{LinkPool, LinkTracker};
use crate::uni::WidthMethod;
use crate::uni::pool::{GraphemePool, GraphemeTracker};
use crate::uni::segments::{
    char_left_extent, char_right_extent, grapheme_id_from_char, is_continuation_char,
    is_grapheme_char, pack_continuation,
};
use std::cell::RefCell;
use std::rc::Rc;

pub mod draw;

pub const DEFAULT_SPACE_CHAR: u32 = 32;
pub const MAX_UNICODE_CODEPOINT: u32 = 0x10FFFF;

/// One grid cell: packed char, colors, attribute word. Reference `Cell`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub char: u32,
    pub fg: Rgba,
    pub bg: Rgba,
    pub attributes: u32,
}

pub fn make_cell(char: u32, fg: Rgba, bg: Rgba, attributes: u32) -> Cell {
    Cell {
        char,
        fg,
        bg,
        attributes,
    }
}

/// Clipping rectangle for the scissor stack. Reference `ClipRect`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Image render protocol, mirroring the reference discriminants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ImageProtocol {
    Auto = 0,
    Kitty = 1,
    Sixel = 2,
    Blocks = 3,
}

/// Placement geometry plus handle. Decoded pixels live with the media
/// commitment; the buffer only tracks and drops the placement entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImagePlacement {
    pub placement_id: u32,
    pub image_handle: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub opacity: u8,
    pub protocol: ImageProtocol,
}

/// Reference `BufferError`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferError {
    OutOfMemory,
    InvalidDimensions,
    InvalidUnicode,
    BufferTooSmall,
}

/// Construction options. Reference `OptimizedBuffer.InitOptions`. Pools
/// are caller-owned handles; a missing link pool becomes a fresh owned
/// pool (never a process-global one).
pub struct InitOptions<'a> {
    pub respect_alpha: bool,
    pub blend_backdrop: Option<Rgba>,
    pub pool: Rc<RefCell<GraphemePool<'a>>>,
    pub link_pool: Option<Rc<RefCell<LinkPool>>>,
    pub width_method: WidthMethod,
    pub id: String,
}

impl<'a> InitOptions<'a> {
    pub fn new(pool: Rc<RefCell<GraphemePool<'a>>>) -> Self {
        InitOptions {
            respect_alpha: false,
            blend_backdrop: None,
            pool,
            link_pool: None,
            width_method: WidthMethod::Unicode,
            id: "unnamed buffer".to_string(),
        }
    }
}

/// Terminal cell grid. Reference `OptimizedBuffer`.
pub struct OptimizedBuffer<'a> {
    chars: Vec<u32>,
    fgs: Vec<Rgba>,
    bgs: Vec<Rgba>,
    attributes: Vec<u32>,
    width: u32,
    height: u32,
    respect_alpha: bool,
    blend_backdrop: Option<Rgba>,
    pub pool: Rc<RefCell<GraphemePool<'a>>>,
    pub link_pool: Rc<RefCell<LinkPool>>,
    pub grapheme_tracker: GraphemeTracker<'a>,
    pub link_tracker: LinkTracker,
    pub width_method: WidthMethod,
    id: String,
    scissor_stack: Vec<ClipRect>,
    opacity_stack: Vec<f32>,
    placements: Vec<ImagePlacement>,
}

impl<'a> OptimizedBuffer<'a> {
    pub fn new(width: u32, height: u32, options: InitOptions<'a>) -> Result<Self, BufferError> {
        if width == 0 || height == 0 {
            return Err(BufferError::InvalidDimensions);
        }
        let size = width as usize * height as usize;
        let link_pool = options
            .link_pool
            .unwrap_or_else(|| Rc::new(RefCell::new(LinkPool::new())));
        Ok(OptimizedBuffer {
            chars: vec![0; size],
            fgs: vec![ansi::rgb_color(0, 0, 0, 0); size],
            bgs: vec![ansi::rgb_color(0, 0, 0, 0); size],
            attributes: vec![0; size],
            width,
            height,
            respect_alpha: options.respect_alpha,
            blend_backdrop: options.blend_backdrop,
            pool: Rc::clone(&options.pool),
            link_pool: Rc::clone(&link_pool),
            grapheme_tracker: GraphemeTracker::new(options.pool),
            link_tracker: LinkTracker::new(link_pool),
            width_method: options.width_method,
            id: options.id,
            scissor_stack: Vec::new(),
            opacity_stack: Vec::new(),
            placements: Vec::new(),
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn placements(&self) -> &[ImagePlacement] {
        &self.placements
    }

    pub fn push_placement(&mut self, placement: ImagePlacement) {
        self.placements.push(placement);
    }

    fn coords_to_index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    // ---- scissor point checks (rect ops arrive with buffer-draw) ----

    pub fn current_scissor(&self) -> Option<ClipRect> {
        self.scissor_stack.last().copied()
    }

    pub fn point_in_scissor(&self, x: i32, y: i32) -> bool {
        match self.current_scissor() {
            None => true,
            Some(r) => {
                x >= r.x && x < r.x + r.width as i32 && y >= r.y && y < r.y + r.height as i32
            }
        }
    }

    pub fn push_scissor(&mut self, rect: ClipRect) {
        self.scissor_stack.push(rect);
    }

    pub fn pop_scissor(&mut self) {
        self.scissor_stack.pop();
    }

    pub fn clear_scissors(&mut self) {
        self.scissor_stack.clear();
    }

    // ---- core entry points ----

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), BufferError> {
        if self.width == width && self.height == height {
            return Ok(());
        }
        if width == 0 || height == 0 {
            return Err(BufferError::InvalidDimensions);
        }
        let size = width as usize * height as usize;
        self.chars.resize(size, 0);
        self.fgs.resize(size, ansi::rgb_color(0, 0, 0, 0));
        self.bgs.resize(size, ansi::rgb_color(0, 0, 0, 0));
        self.attributes.resize(size, 0);
        self.width = width;
        self.height = height;
        // Always clear after resize: new cells would be garbage and
        // shrunken-away grapheme refs must be released.
        self.clear(ansi::rgb_color(0, 0, 0, 255), None);
        Ok(())
    }

    pub fn clear(&mut self, bg: Rgba, char: Option<u32>) {
        let cell_char = char.unwrap_or(DEFAULT_SPACE_CHAR);
        self.link_tracker.clear();
        self.grapheme_tracker.clear();
        self.placements.clear();
        self.chars.fill(cell_char);
        self.attributes.fill(0);
        self.fgs.fill(ansi::rgb_color(255, 255, 255, 255));
        self.bgs.fill(bg);
    }

    /// Validate coordinates and return the cell index, or `None` when out
    /// of bounds or clipped by the scissor. Reference `validateAndIndex`.
    fn validate_and_index(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        if !self.point_in_scissor(x as i32, y as i32) {
            return None;
        }
        Some(self.coords_to_index(x, y))
    }

    /// Write cell data and update the link tracker. No grapheme tracking,
    /// span cleanup, or continuation propagation. Reference `setRaw`.
    pub fn set_raw(&mut self, x: u32, y: u32, cell: Cell) {
        if let Some(index) = self.validate_and_index(x, y) {
            self.write_cell_and_links(index, cell);
        }
    }

    /// Like `set`, but without span cleanup. Reference `syncCell`.
    pub fn sync_cell(&mut self, x: u32, y: u32, cell: Cell) {
        self.set_internal(false, x, y, cell);
    }

    /// Full cell write with span cleanup. Reference `set`.
    pub fn set(&mut self, x: u32, y: u32, cell: Cell) {
        self.set_internal(true, x, y, cell);
    }

    fn set_internal(&mut self, span_cleanup: bool, x: u32, y: u32, cell: Cell) {
        let index = match self.validate_and_index(x, y) {
            Some(index) => index,
            None => return,
        };
        let prev_char = self.chars[index];
        let prev_link_id = TextAttributes::link_id(self.attributes[index]);
        let mut tracker_replaced = false;

        if !span_cleanup {
            let old_start_id = if is_grapheme_char(prev_char) {
                Some(grapheme_id_from_char(prev_char))
            } else {
                None
            };
            let new_start_id = if !is_grapheme_char(cell.char) {
                None
            } else {
                let new_width = char_right_extent(cell.char) + 1;
                if x + new_width > self.width {
                    None
                } else {
                    Some(grapheme_id_from_char(cell.char))
                }
            };
            if old_start_id.is_some() || new_start_id.is_some() {
                self.grapheme_tracker.replace(old_start_id, new_start_id);
                tracker_replaced = true;
            }
        }

        // Overwriting a grapheme span with a different char clears it first.
        if span_cleanup
            && (is_grapheme_char(prev_char) || is_continuation_char(prev_char))
            && prev_char != cell.char
        {
            let row_start = (y * self.width) as usize;
            let row_end = row_start + self.width as usize - 1;
            let left = char_left_extent(prev_char) as usize;
            let right = char_right_extent(prev_char) as usize;
            let id = grapheme_id_from_char(prev_char);

            let new_grapheme_id = if !is_grapheme_char(cell.char) {
                None
            } else {
                let new_width = char_right_extent(cell.char) + 1;
                if x + new_width > self.width {
                    None
                } else {
                    Some(grapheme_id_from_char(cell.char))
                }
            };
            self.grapheme_tracker.replace(Some(id), new_grapheme_id);
            tracker_replaced = true;

            let span_start = index - left.min(index - row_start);
            let span_end = index + right.min(row_end - index);
            let mut span_i = span_start;
            while span_i <= span_end {
                let span_char = self.chars[span_i];
                if (is_grapheme_char(span_char) || is_continuation_char(span_char))
                    && grapheme_id_from_char(span_char) == id
                {
                    let span_link_id = TextAttributes::link_id(self.attributes[span_i]);
                    if span_link_id != 0 {
                        self.link_tracker.remove_cell_ref(span_link_id);
                    }
                    self.chars[span_i] = DEFAULT_SPACE_CHAR;
                    self.attributes[span_i] = 0;
                }
                span_i += 1;
            }
        }

        if is_grapheme_char(cell.char) {
            let right = char_right_extent(cell.char);
            let width = 1 + right;

            if x + width > self.width {
                let end_of_line = ((y + 1) * self.width) as usize;
                let mut eol_i = index;
                while eol_i < end_of_line {
                    let eol_link_id = TextAttributes::link_id(self.attributes[eol_i]);
                    if eol_link_id != 0 {
                        self.link_tracker.remove_cell_ref(eol_link_id);
                    }
                    eol_i += 1;
                }
                self.chars[index..end_of_line].fill(DEFAULT_SPACE_CHAR);
                self.attributes[index..end_of_line].fill(cell.attributes);
                self.fgs[index..end_of_line].fill(cell.fg);
                self.bgs[index..end_of_line].fill(cell.bg);
                let new_link_id = TextAttributes::link_id(cell.attributes);
                if new_link_id != 0 {
                    for _ in index..end_of_line {
                        self.link_tracker.add_cell_ref(new_link_id);
                    }
                }
                return;
            }

            self.chars[index] = cell.char;
            self.fgs[index] = cell.fg;
            self.bgs[index] = cell.bg;
            self.attributes[index] = cell.attributes;

            let id = grapheme_id_from_char(cell.char);
            let is_same_grapheme_start = is_grapheme_char(prev_char) && prev_char == cell.char;
            if !tracker_replaced && !is_same_grapheme_start {
                self.grapheme_tracker.add(id);
            }

            let new_link_id = TextAttributes::link_id(cell.attributes);
            if prev_link_id != 0 && prev_link_id != new_link_id {
                self.link_tracker.remove_cell_ref(prev_link_id);
            }
            if new_link_id != 0 && new_link_id != prev_link_id {
                self.link_tracker.add_cell_ref(new_link_id);
            }

            if width > 1 {
                let row_end_index = (y * self.width + self.width - 1) as usize;
                let max_right = (right as usize).min(row_end_index - index);
                if max_right > 0 {
                    let mut cont_i = 1;
                    while cont_i <= max_right {
                        let cont_link_id = TextAttributes::link_id(self.attributes[index + cont_i]);
                        if cont_link_id != 0 {
                            self.link_tracker.remove_cell_ref(cont_link_id);
                        }
                        cont_i += 1;
                    }
                    self.fgs[index + 1..index + 1 + max_right].fill(cell.fg);
                    self.bgs[index + 1..index + 1 + max_right].fill(cell.bg);
                    self.attributes[index + 1..index + 1 + max_right].fill(cell.attributes);
                    let mut k = 1;
                    while k <= max_right {
                        let cont = pack_continuation(k as u32, (max_right - k) as u32, id);
                        self.chars[index + k] = cont;
                        if new_link_id != 0 {
                            self.link_tracker.add_cell_ref(new_link_id);
                        }
                        k += 1;
                    }
                }
            }
        } else {
            self.write_cell_and_links(index, cell);
        }
    }

    /// Write cell data at an index and update the link tracker.
    fn write_cell_and_links(&mut self, index: usize, cell: Cell) {
        let prev_link_id = TextAttributes::link_id(self.attributes[index]);
        let new_link_id = TextAttributes::link_id(cell.attributes);
        self.chars[index] = cell.char;
        self.fgs[index] = cell.fg;
        self.bgs[index] = cell.bg;
        self.attributes[index] = cell.attributes;
        if prev_link_id != 0 && prev_link_id != new_link_id {
            self.link_tracker.remove_cell_ref(prev_link_id);
        }
        if new_link_id != 0 && new_link_id != prev_link_id {
            self.link_tracker.add_cell_ref(new_link_id);
        }
    }

    /// Cell read. Out-of-grid coordinates return `None`; never panics.
    /// Reference `get` (bounds only, no scissor check).
    pub fn get(&self, x: u32, y: u32) -> Option<Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = self.coords_to_index(x, y);
        Some(Cell {
            char: self.chars[index],
            fg: self.fgs[index],
            bg: self.bgs[index],
            attributes: self.attributes[index],
        })
    }

    // ---- tiny reference accessors ----

    pub fn respect_alpha(&self) -> bool {
        self.respect_alpha
    }

    pub fn set_respect_alpha(&mut self, respect_alpha: bool) {
        self.respect_alpha = respect_alpha;
    }

    pub fn blend_backdrop(&self) -> Option<Rgba> {
        self.blend_backdrop
    }

    pub fn set_blend_backdrop(&mut self, color: Option<Rgba>) {
        self.blend_backdrop = color;
    }
}

#[cfg(test)]
use crate::ansi::rgb_color;

/// BUF-001 falsifier: a pack, blend, and unpack round trip preserves the
/// intent and the slot.
#[cfg(test)]
#[test]
fn color_model() {
    use crate::ansi::{ColorIntent, default_color, indexed_color, intent, rgb_color, slot};

    let rgb = rgb_color(10, 20, 30, 40);
    assert_eq!(ColorIntent::Rgb, intent(rgb));
    assert_eq!(
        (10, 20, 30, 40),
        (
            ansi::red(rgb),
            ansi::green(rgb),
            ansi::blue(rgb),
            ansi::alpha(rgb)
        )
    );

    let indexed = indexed_color(9, 255, 0, 0);
    assert_eq!(ColorIntent::Indexed, intent(indexed));
    assert_eq!(9, slot(indexed));
    assert_eq!(
        (255, 0, 0, 255),
        (
            ansi::red(indexed),
            ansi::green(indexed),
            ansi::blue(indexed),
            ansi::alpha(indexed)
        )
    );

    let default = default_color(1, 2, 3, 4);
    assert_eq!(ColorIntent::Default, intent(default));
    assert_eq!(
        (1, 2, 3, 4),
        (
            ansi::red(default),
            ansi::green(default),
            ansi::blue(default),
            ansi::alpha(default)
        )
    );

    // with_meta swaps intent/slot without touching channels.
    let swapped = ansi::with_meta(rgb, ansi::pack_meta(ColorIntent::Indexed, 7));
    assert_eq!(ColorIntent::Indexed, intent(swapped));
    assert_eq!(7, slot(swapped));
    assert_eq!(
        (10, 20, 30),
        (
            ansi::red(swapped),
            ansi::green(swapped),
            ansi::blue(swapped)
        )
    );
}

/// BUF-002 falsifier: palette index 9 resolves to pure red (255, 0, 0).
#[cfg(test)]
#[test]
fn palette() {
    use crate::ansi::{ANSI_256_CUBE_LEVELS, ANSI16_RGB, fallback_ansi256_color};

    let red9 = fallback_ansi256_color(9);
    assert_eq!(
        (255, 0, 0),
        (ansi::red(red9), ansi::green(red9), ansi::blue(red9))
    );
    for (i, base) in ANSI16_RGB.iter().enumerate() {
        let c = fallback_ansi256_color(i);
        assert_eq!(
            (base[0], base[1], base[2]),
            (ansi::red(c), ansi::green(c), ansi::blue(c)),
            "index {i}"
        );
    }
    let cube0 = fallback_ansi256_color(16);
    assert_eq!(
        (0, 0, 0),
        (ansi::red(cube0), ansi::green(cube0), ansi::blue(cube0))
    );
    let cube_last = fallback_ansi256_color(231);
    assert_eq!(
        (255, 255, 255),
        (
            ansi::red(cube_last),
            ansi::green(cube_last),
            ansi::blue(cube_last)
        )
    );
    let ramp = ANSI_256_CUBE_LEVELS;
    // Cube (r=1, g=2, b=3): index 67.
    let cube_mid = fallback_ansi256_color(16 + 36 + 2 * 6 + 3);
    assert_eq!(
        (ramp[1], ramp[2], ramp[3]),
        (
            ansi::red(cube_mid),
            ansi::green(cube_mid),
            ansi::blue(cube_mid)
        )
    );
    let gray0 = fallback_ansi256_color(232);
    assert_eq!(
        (8, 8, 8),
        (ansi::red(gray0), ansi::green(gray0), ansi::blue(gray0))
    );
    let gray_last = fallback_ansi256_color(255);
    assert_eq!(
        (238, 238, 238),
        (
            ansi::red(gray_last),
            ansi::green(gray_last),
            ansi::blue(gray_last)
        )
    );
}

/// BUF-003 falsifier: a set-then-get round trip preserves every field,
/// and a link-id write leaves style flags untouched.
#[cfg(test)]
#[test]
fn cell_roundtrip() {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut buf = OptimizedBuffer::new(8, 4, InitOptions::new(Rc::clone(&pool))).unwrap();
    let fg = rgb_color(1, 2, 3, 255);
    let bg = rgb_color(4, 5, 6, 255);
    let attrs = TextAttributes::BOLD | TextAttributes::ITALIC;
    buf.set(2, 1, make_cell(0x41, fg, bg, u32::from(attrs)));
    let cell = buf.get(2, 1).unwrap();
    assert_eq!(0x41, cell.char);
    assert_eq!(fg, cell.fg);
    assert_eq!(bg, cell.bg);
    assert_eq!(u32::from(attrs), cell.attributes);

    let linked = TextAttributes::set_link_id(u32::from(attrs), 0xABCDEu32);
    buf.set(3, 1, make_cell(0x42, fg, bg, linked));
    let back = buf.get(3, 1).unwrap();
    assert_eq!(attrs, TextAttributes::base_attributes(back.attributes));
    assert_eq!(0xABCDE, TextAttributes::link_id(back.attributes));
    assert!(TextAttributes::has_link(back.attributes));
}

/// BUF-004 falsifier: out-of-bounds access never panics, and `get`
/// returns `None` outside the grid.
#[cfg(test)]
#[test]
fn bounds() {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut buf = OptimizedBuffer::new(4, 3, InitOptions::new(Rc::clone(&pool))).unwrap();
    assert!(buf.get(4, 0).is_none());
    assert!(buf.get(0, 3).is_none());
    assert!(buf.get(u32::MAX, u32::MAX).is_none());
    assert!(buf.get(3, 2).is_some());
    let cell = make_cell(0x41, rgb_color(0, 0, 0, 255), rgb_color(0, 0, 0, 255), 0);
    buf.set(4, 0, cell);
    buf.set(0, 3, cell);
    buf.set(u32::MAX, u32::MAX, cell);
    buf.set_raw(9, 9, cell);
    // Inside still untouched default zeros.
    assert_eq!(0, buf.get(3, 2).unwrap().char);
}

/// BUF-005 falsifier: a zero-size resize fails.
#[cfg(test)]
#[test]
fn resize_errors() {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut buf = OptimizedBuffer::new(4, 4, InitOptions::new(Rc::clone(&pool))).unwrap();
    assert_eq!(Err(BufferError::InvalidDimensions), buf.resize(0, 4));
    assert_eq!(Err(BufferError::InvalidDimensions), buf.resize(4, 0));
    assert_eq!(Err(BufferError::InvalidDimensions), buf.resize(0, 0));
    assert_eq!((4, 4), (buf.width(), buf.height()));
}

/// BUF-006 falsifier: no pre-resize value survives a resize.
#[cfg(test)]
#[test]
fn resize_clears() {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut buf = OptimizedBuffer::new(4, 4, InitOptions::new(Rc::clone(&pool))).unwrap();
    let fg = rgb_color(9, 9, 9, 255);
    let bg = rgb_color(0, 0, 0, 255);
    buf.set(0, 0, make_cell(0x5A, fg, bg, 0xFF));
    buf.resize(6, 6).unwrap();
    for y in 0..6 {
        for x in 0..6 {
            let cell = buf.get(x, y).unwrap();
            assert_eq!(DEFAULT_SPACE_CHAR, cell.char, "cell {x},{y}");
            assert_eq!(0, cell.attributes, "cell {x},{y}");
        }
    }
}

/// BUF-007 falsifier: `clear` resets cells and drops link, grapheme, and
/// placement state.
#[cfg(test)]
#[test]
fn clear() {
    use crate::link::LinkPool;

    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut options = InitOptions::new(Rc::clone(&pool));
    options.link_pool = Some(Rc::clone(&link_pool));
    let mut buf = OptimizedBuffer::new(6, 2, options).unwrap();

    let link_id = link_pool
        .borrow_mut()
        .alloc(b"https://example.com")
        .unwrap();
    let gid = pool.borrow_mut().alloc("你".as_bytes()).unwrap();
    let start = crate::uni::segments::pack_grapheme_start(gid, 2);
    // Plant the link with raw layout shifts, not the packing helper: this
    // test isolates clear(), so its setup must not depend on BUF-003's
    // writer. The layout (link id in bits 8-31) is spec-pinned by BUF-003.
    let linked = ((link_id & TextAttributes::LINK_ID_PAYLOAD_MASK)
        << TextAttributes::LINK_ID_SHIFT)
        | u32::from(TextAttributes::BOLD);
    let fg = rgb_color(1, 2, 3, 255);
    let bg = rgb_color(0, 0, 0, 255);
    buf.set(0, 0, make_cell(start, fg, bg, linked));
    buf.push_placement(ImagePlacement {
        placement_id: 1,
        image_handle: 7,
        x: 0,
        y: 0,
        width: 2,
        height: 2,
        pixel_width: 20,
        pixel_height: 20,
        source_x: 0,
        source_y: 0,
        source_width: 20,
        source_height: 20,
        opacity: 255,
        protocol: ImageProtocol::Kitty,
    });
    assert!(buf.link_tracker.has_any());
    assert!(buf.grapheme_tracker.has_any());
    assert_eq!(1, buf.placements().len());

    buf.clear(bg, None);
    let cell = buf.get(0, 0).unwrap();
    assert_eq!(DEFAULT_SPACE_CHAR, cell.char);
    assert_eq!(rgb_color(255, 255, 255, 255), cell.fg);
    assert_eq!(0, cell.attributes);
    assert_eq!(bg, cell.bg);
    assert!(!buf.link_tracker.has_any());
    assert!(!buf.grapheme_tracker.has_any());
    assert_eq!(0, link_pool.borrow().get_refcount(link_id).unwrap());
    assert!(buf.placements().is_empty());

    // Caller-supplied clear char.
    buf.clear(bg, Some(0x2E));
    assert_eq!(0x2E, buf.get(1, 1).unwrap().char);
}
