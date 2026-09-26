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

use crate::ansi::{self, CellDecoration, Rgba, TextAttributes, UnderlineStyle};
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

/// Space codepoint that fills cleared cells.
pub const DEFAULT_SPACE_CHAR: u32 = 32;
/// Largest Unicode codepoint. Emitters write a space for a plain char above it.
pub const MAX_UNICODE_CODEPOINT: u32 = 0x10FFFF;

/// A `CellDecoration` packed into one word so the column array compares
/// as bytes (RAS-008): bits 0-2 underline style, bit 3 overline, bit 4
/// whether an underline color is set, bits 8-31 that color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct PackedDecoration(u32);

impl From<CellDecoration> for PackedDecoration {
    fn from(d: CellDecoration) -> Self {
        let mut word = u32::from(d.underline as u8) & 0x7;
        if d.overline {
            word |= 1 << 3;
        }
        if let Some([r, g, b]) = d.underline_color {
            word |= 1 << 4;
            word |= (u32::from(r) << 8) | (u32::from(g) << 16) | (u32::from(b) << 24);
        }
        PackedDecoration(word)
    }
}

impl From<PackedDecoration> for CellDecoration {
    fn from(p: PackedDecoration) -> Self {
        let word = p.0;
        let underline = match word & 0x7 {
            1 => UnderlineStyle::Single,
            2 => UnderlineStyle::Double,
            3 => UnderlineStyle::Curly,
            4 => UnderlineStyle::Dotted,
            5 => UnderlineStyle::Dashed,
            _ => UnderlineStyle::None,
        };
        CellDecoration {
            underline,
            underline_color: (word & (1 << 4) != 0).then_some([
                ((word >> 8) & 0xFF) as u8,
                ((word >> 16) & 0xFF) as u8,
                ((word >> 24) & 0xFF) as u8,
            ]),
            overline: word & (1 << 3) != 0,
        }
    }
}

/// One grid cell: packed char, colors, attribute word. Reference `Cell`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    /// Packed character: a plain codepoint, or a grapheme, continuation, or
    /// image code flagged in bits 31-30.
    pub char: u32,
    /// Foreground color.
    pub fg: Rgba,
    /// Background color.
    pub bg: Rgba,
    /// Attribute word: style flags in bits 0-7, link id in bits 8-31.
    pub attributes: u32,
    /// Extended underline and overline decoration.
    pub decoration: CellDecoration,
}

impl Cell {
    /// Return the cell with `decoration` in place of its own.
    pub fn with_decoration(mut self, decoration: CellDecoration) -> Self {
        self.decoration = decoration;
        self
    }
}

/// Build a cell with no extended decoration.
pub fn make_cell(char: u32, fg: Rgba, bg: Rgba, attributes: u32) -> Cell {
    #[cfg(test)]
    ras_008_tests::count_cell();
    Cell {
        char,
        fg,
        bg,
        attributes,
        decoration: CellDecoration::default(),
    }
}

/// Columns the row compare checks per step (RAS-008): a fixed-size block the
/// compiler compares with vector instructions, with no call per block, so an
/// unchanged row costs no more than whole-row comparisons (RAS-005).
const COMPARE_BLOCK: usize = 32;

/// Whether two `COMPARE_BLOCK`-long slices hold equal values.
#[inline(always)]
fn same_block<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    match (
        <&[T; COMPARE_BLOCK]>::try_from(a),
        <&[T; COMPARE_BLOCK]>::try_from(b),
    ) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// Clipping rectangle for the scissor stack. Reference `ClipRect`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipRect {
    /// Left column; may be negative.
    pub x: i32,
    /// Top row; may be negative.
    pub y: i32,
    /// Width in cells.
    pub width: u32,
    /// Height in cells.
    pub height: u32,
}

/// Image render protocol, mirroring the reference discriminants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ImageProtocol {
    /// Choose from the terminal's detected capabilities.
    Auto = 0,
    /// Kitty graphics protocol.
    Kitty = 1,
    /// Sixel graphics.
    Sixel = 2,
    /// Unicode block characters drawn in cells.
    Blocks = 3,
}

/// Placement geometry plus handle. Decoded pixels live with the media
/// commitment; the buffer only tracks and drops the placement entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImagePlacement {
    /// Placement id; compositing renumbers merged placements from 1.
    pub placement_id: u32,
    /// Handle of the decoded image to show.
    pub image_handle: u32,
    /// Left column, in cells.
    pub x: i32,
    /// Top row, in cells.
    pub y: i32,
    /// Width in cells.
    pub width: u32,
    /// Height in cells.
    pub height: u32,
    /// Width in pixels. Compositing scales it to the visible part and keeps 0 as 0.
    pub pixel_width: u32,
    /// Height in pixels. Compositing scales it to the visible part and keeps 0 as 0.
    pub pixel_height: u32,
    /// Left edge of the shown source-image region, in image pixels.
    pub source_x: u32,
    /// Top edge of the shown source-image region, in image pixels.
    pub source_y: u32,
    /// Width of the shown source-image region, in image pixels.
    pub source_width: u32,
    /// Height of the shown source-image region, in image pixels.
    pub source_height: u32,
    /// Opacity from 0 (transparent) to 255 (opaque).
    pub opacity: u8,
    /// Image protocol requested for this placement.
    pub protocol: ImageProtocol,
}

/// Reference `BufferError`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferError {
    /// The grapheme pool refused an allocation.
    OutOfMemory,
    /// A width or height was zero.
    InvalidDimensions,
}

/// Construction options. Reference `OptimizedBuffer.InitOptions`. Pools
/// are caller-owned handles; a missing link pool becomes a fresh owned
/// pool (never a process-global one).
pub struct InitOptions<'a> {
    /// Starting value for `OptimizedBuffer::respect_alpha`.
    pub respect_alpha: bool,
    /// Starting value for `OptimizedBuffer::blend_backdrop`.
    pub blend_backdrop: Option<Rgba>,
    /// Grapheme pool for cluster bytes.
    pub pool: Rc<RefCell<GraphemePool<'a>>>,
    /// Link pool to share; `None` creates a fresh one.
    pub link_pool: Option<Rc<RefCell<LinkPool>>>,
    /// How the buffer measures character widths.
    pub width_method: WidthMethod,
    /// Buffer name, returned by `OptimizedBuffer::id`.
    pub id: String,
}

impl<'a> InitOptions<'a> {
    /// Options over `pool` with defaults. Alpha blending is off, there is no
    /// backdrop, the link pool is fresh, widths use `WidthMethod::Unicode`, and
    /// the id is `"unnamed buffer"`.
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
    decorations: Vec<PackedDecoration>,
    width: u32,
    height: u32,
    respect_alpha: bool,
    blend_backdrop: Option<Rgba>,
    /// Grapheme pool that holds this grid's cluster bytes.
    pub pool: Rc<RefCell<GraphemePool<'a>>>,
    /// Link pool that holds this grid's link URLs.
    pub link_pool: Rc<RefCell<LinkPool>>,
    /// Counts this grid's cell references to each grapheme id.
    pub grapheme_tracker: GraphemeTracker<'a>,
    /// Counts this grid's cell references to each link id.
    pub link_tracker: LinkTracker,
    /// How this grid measures character widths.
    pub width_method: WidthMethod,
    id: String,
    scissor_stack: Vec<ClipRect>,
    opacity_stack: Vec<f32>,
    placements: Vec<ImagePlacement>,
    /// Times `clear` has run on this grid (RAS-007).
    clears: u64,
}

impl<'a> OptimizedBuffer<'a> {
    /// Create a `width` by `height` grid of zeroed cells: char 0 and transparent
    /// black colors. Fails with `InvalidDimensions` when either side is zero.
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
            decorations: vec![PackedDecoration::default(); size],
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
            clears: 0,
        })
    }

    /// Times [`OptimizedBuffer::clear`] has run on this grid. A painter
    /// clears the next frame's grid once per frame and the renderer never
    /// does (RAS-007).
    pub fn clear_count(&self) -> u64 {
        self.clears
    }

    /// Grid width in cells.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Grid height in cells.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Buffer name from `InitOptions::id`.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Image placements recorded in this grid.
    pub fn placements(&self) -> &[ImagePlacement] {
        &self.placements
    }

    /// Record an image placement; `clear` drops it.
    pub fn push_placement(&mut self, placement: ImagePlacement) {
        self.placements.push(placement);
    }

    fn coords_to_index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    // ---- scissor point checks (rect ops arrive with buffer-draw) ----

    /// Top of the scissor stack, or `None` when the stack is empty.
    pub fn current_scissor(&self) -> Option<ClipRect> {
        self.scissor_stack.last().copied()
    }

    /// Whether a point lies inside the top scissor; true when no scissor is set.
    pub fn point_in_scissor(&self, x: i32, y: i32) -> bool {
        match self.current_scissor() {
            None => true,
            Some(r) => {
                x >= r.x && x < r.x + r.width as i32 && y >= r.y && y < r.y + r.height as i32
            }
        }
    }

    /// Push a scissor as given, without intersecting it with the current one.
    pub fn push_scissor(&mut self, rect: ClipRect) {
        self.scissor_stack.push(rect);
    }

    /// Remove the top scissor.
    pub fn pop_scissor(&mut self) {
        self.scissor_stack.pop();
    }

    /// Remove every scissor.
    pub fn clear_scissors(&mut self) {
        self.scissor_stack.clear();
    }

    // ---- core entry points ----

    /// Resize the grid and clear it to spaces on opaque black. A same-size call
    /// does nothing; a zero side fails with `InvalidDimensions`.
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
        self.decorations.resize(size, PackedDecoration::default());
        self.width = width;
        self.height = height;
        // Always clear after resize: new cells would be garbage and
        // shrunken-away grapheme refs must be released.
        self.clear(ansi::rgb_color(0, 0, 0, 255), None);
        Ok(())
    }

    /// Reset every cell to `char` (space when `None`) with a white foreground,
    /// `bg`, and no attributes. Also drops links, grapheme references, and image
    /// placements.
    pub fn clear(&mut self, bg: Rgba, char: Option<u32>) {
        self.clears += 1;
        let cell_char = char.unwrap_or(DEFAULT_SPACE_CHAR);
        self.link_tracker.clear();
        self.grapheme_tracker.clear();
        self.placements.clear();
        self.chars.fill(cell_char);
        self.attributes.fill(0);
        self.decorations.fill(PackedDecoration::default());
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
                    self.decorations[span_i] = PackedDecoration::default();
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
                self.decorations[index..end_of_line].fill(cell.decoration.into());
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
            self.decorations[index] = cell.decoration.into();

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
                    self.decorations[index + 1..index + 1 + max_right].fill(cell.decoration.into());
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
        self.decorations[index] = cell.decoration.into();
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
        #[cfg(test)]
        ras_008_tests::count_cell();
        Some(Cell {
            char: self.chars[index],
            fg: self.fgs[index],
            bg: self.bgs[index],
            attributes: self.attributes[index],
            decoration: self.decorations[index].into(),
        })
    }

    // ---- index-based access for the rasterizer (RAS-003, RAS-008) ----

    /// Cell index of a coordinate pair; the caller keeps it in range.
    #[inline]
    pub fn index_of(&self, x: u32, y: u32) -> usize {
        self.coords_to_index(x, y)
    }

    /// Packed character at a cell index; panics when out of range.
    #[inline]
    pub fn char_at(&self, index: usize) -> u32 {
        self.chars[index]
    }

    /// Foreground color at a cell index; panics when out of range.
    #[inline]
    pub fn fg_at(&self, index: usize) -> Rgba {
        self.fgs[index]
    }

    /// Background color at a cell index; panics when out of range.
    #[inline]
    pub fn bg_at(&self, index: usize) -> Rgba {
        self.bgs[index]
    }

    /// Attribute word at a cell index; panics when out of range.
    #[inline]
    pub fn attributes_at(&self, index: usize) -> u32 {
        self.attributes[index]
    }

    /// Decoration at a cell index; panics when out of range.
    #[inline]
    pub fn decoration_at(&self, index: usize) -> CellDecoration {
        self.decorations[index].into()
    }

    /// Whether the cell at `index` holds the same value in both buffers,
    /// read from the column arrays without building a `Cell`.
    #[inline]
    pub fn cell_eq_at(&self, other: &Self, index: usize) -> bool {
        self.chars[index] == other.chars[index]
            && self.fgs[index] == other.fgs[index]
            && self.bgs[index] == other.bgs[index]
            && self.attributes[index] == other.attributes[index]
            && self.decorations[index] == other.decorations[index]
    }

    /// The first column of row `y` whose cell differs between the two
    /// buffers, or `None` when the row is identical. One pass over the row,
    /// left to right: the five column arrays are compared a block of
    /// columns at a time, and the first block that differs is searched
    /// column by column, so no column before the change is read twice. No
    /// `Cell` is built (RAS-008).
    pub fn row_first_change(&self, other: &Self, y: u32) -> Option<u32> {
        let width = self.width as usize;
        let start = y as usize * width;
        let end = start + width;
        let mut from = start;
        while from + COMPARE_BLOCK <= end {
            let to = from + COMPARE_BLOCK;
            let same = same_block(&self.chars[from..to], &other.chars[from..to])
                && same_block(&self.fgs[from..to], &other.fgs[from..to])
                && same_block(&self.bgs[from..to], &other.bgs[from..to])
                && same_block(&self.attributes[from..to], &other.attributes[from..to])
                && same_block(&self.decorations[from..to], &other.decorations[from..to]);
            if !same {
                break;
            }
            from = to;
        }
        (from..end)
            .find(|&i| !self.cell_eq_at(other, i))
            .map(|i| (i - start) as u32)
    }

    // ---- tiny reference accessors ----

    /// Whether compositing this buffer always alpha-blends its cells instead of
    /// copying them.
    pub fn respect_alpha(&self) -> bool {
        self.respect_alpha
    }

    /// Set whether compositing this buffer always alpha-blends its cells.
    pub fn set_respect_alpha(&mut self, respect_alpha: bool) {
        self.respect_alpha = respect_alpha;
    }

    /// Color that blending uses in place of a fully transparent destination.
    pub fn blend_backdrop(&self) -> Option<Rgba> {
        self.blend_backdrop
    }

    /// Set the blend backdrop; `None` blends over a transparent destination as is.
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

/// RAS-008: the row compare finds the first differing column in any of the
/// five column arrays and builds no `Cell` doing it. `get` and `make_cell`
/// count the cells they build on this thread while unit tests run.
#[cfg(test)]
mod ras_008_tests {
    use super::{InitOptions, OptimizedBuffer, make_cell};
    use crate::ansi;
    use crate::uni::pool::GraphemePool;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    thread_local! {
        static CELLS_BUILT: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) fn count_cell() {
        CELLS_BUILT.with(|n| n.set(n.get() + 1));
    }

    fn cells_built() -> usize {
        CELLS_BUILT.with(Cell::get)
    }

    fn buffer(width: u32, pool: &Rc<RefCell<GraphemePool<'static>>>) -> OptimizedBuffer<'static> {
        let mut buffer = OptimizedBuffer::new(width, 2, InitOptions::new(pool.clone())).unwrap();
        for y in 0..2 {
            for x in 0..width {
                let color = ansi::rgb_color(x as u8, y as u8, 7, 255);
                buffer.set(x, y, make_cell(u32::from(b'a'), color, color, 0));
            }
        }
        buffer
    }

    #[test]
    fn ras_008_row_compare_finds_the_first_change_in_every_array_without_building_cells() {
        let pool = Rc::new(RefCell::new(GraphemePool::new()));
        let width = 200;
        let same = buffer(width, &pool);
        let base = buffer(width, &pool);
        let before = cells_built();
        assert_eq!(same.row_first_change(&base, 0), None);
        assert_eq!(same.row_first_change(&base, 1), None);
        assert_eq!(cells_built(), before, "an unchanged row built cells");

        // A change in each array, at chunk edges and inside a chunk.
        for column in [0, 1, 63, 64, 65, 127, 128, width - 1] {
            for array in 0..5 {
                let mut changed = buffer(width, &pool);
                let mut cell = changed.get(column, 1).unwrap();
                match array {
                    0 => cell.char = u32::from(b'b'),
                    1 => cell.fg = ansi::rgb_color(255, 0, 0, 255),
                    2 => cell.bg = ansi::rgb_color(0, 255, 0, 255),
                    3 => cell.attributes = 1,
                    _ => cell.decoration.overline = true,
                }
                changed.set(column, 1, cell);
                let before = cells_built();
                assert_eq!(
                    changed.row_first_change(&base, 1),
                    Some(column),
                    "array {array}, column {column}"
                );
                assert_eq!(
                    changed.row_first_change(&base, 0),
                    None,
                    "array {array}, column {column}: the other row is unchanged"
                );
                assert_eq!(
                    cells_built(),
                    before,
                    "the compare built cells (array {array}, column {column})"
                );
            }
        }
    }
}
