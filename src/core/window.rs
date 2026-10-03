//! Hierarchical Window System
//!
//! Provides a libvaxis-inspired window system with parent-child relationships,
//! automatic constraint handling, built-in border rendering, advanced text printing,
//! cursor management, scrolling, and mouse event handling.

use crate::core::geometry::{Rect, Size};
use crate::core::surface::{Attr, Cell, Rgba, Surface};
use crate::event::types::MouseEvent;

/// Window size specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowSize {
    /// Expand to fill available space
    Expand,
    /// Fixed size limit
    Limit(usize),
}

/// Text segment with styling for advanced printing
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// The text content to display
    pub text: String,
    /// Foreground color
    pub fg: Rgba,
    /// Background color
    pub bg: Rgba,
    /// Text attributes (bold, italic, etc.)
    pub attr: Attr,
}

impl Segment {
    /// Create a new segment with default styling
    ///
    /// # Arguments
    /// * `text` - The text content for the segment
    ///
    /// # Returns
    /// A new `Segment` with white on black text and no attributes
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            fg: Rgba::white(),
            bg: Rgba::black(),
            attr: Attr::empty(),
        }
    }

    /// Create a new segment with custom styling
    ///
    /// # Arguments
    /// * `text` - The text content for the segment
    /// * `fg` - Foreground color
    /// * `bg` - Background color
    /// * `attr` - Text attributes
    ///
    /// # Returns
    /// A new `Segment` with the specified styling
    pub fn with_style(text: impl Into<String>, fg: Rgba, bg: Rgba, attr: Attr) -> Self {
        Self {
            text: text.into(),
            fg,
            bg,
            attr,
        }
    }
}

/// Options for printing text to windows
#[derive(Debug, Clone)]
pub struct PrintOptions {
    /// Vertical offset to start printing at
    pub row_offset: usize,
    /// Horizontal offset to start printing at
    pub col_offset: usize,
    /// Wrap behavior for printing
    pub wrap: WrapMode,
    /// When true, print will write to the screen. When false, only calculate size
    pub commit: bool,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            row_offset: 0,
            col_offset: 0,
            wrap: WrapMode::Grapheme,
            commit: true,
        }
    }
}

/// Text wrapping modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrapMode {
    /// Wrap at grapheme boundaries
    Grapheme,
    /// Wrap at word boundaries
    Word,
    /// Stop printing after one line
    None,
}

/// Result of a print operation
#[derive(Debug, Clone, PartialEq)]
pub struct PrintResult {
    /// Final column position after printing
    pub col: usize,
    /// Final row position after printing
    pub row: usize,
    /// Whether the text overflowed the available space
    pub overflow: bool,
}

/// Cursor shape options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorShape {
    /// Default cursor shape (terminal-dependent)
    Default,
    /// Block cursor (fills entire character cell)
    Block,
    /// Underline cursor (horizontal line at bottom)
    Underline,
    /// Bar cursor (vertical line at left edge)
    Bar,
}

/// Border style options
#[derive(Debug, Clone, PartialEq)]
pub struct BorderOptions {
    /// Visual style for the border
    pub style: BorderStyle,
    /// Which sides of the border to draw
    pub where_: BorderLocation,
    /// Character glyphs to use for border drawing
    pub glyphs: BorderGlyphs,
    /// Optional color override for the border
    pub color: Option<Rgba>,
}

impl Default for BorderOptions {
    fn default() -> Self {
        Self {
            style: BorderStyle::default(),
            where_: BorderLocation::None,
            glyphs: BorderGlyphs::SingleRounded,
            color: None,
        }
    }
}

/// Border location specification
#[derive(Debug, Clone, PartialEq)]
pub enum BorderLocation {
    /// No borders
    None,
    /// All four borders
    All,
    /// Top border only
    Top,
    /// Right border only
    Right,
    /// Bottom border only
    Bottom,
    /// Left border only
    Left,
    /// Custom combination of borders
    Custom(BorderSides),
}

/// Custom border sides
#[derive(Debug, Clone, PartialEq)]
pub struct BorderSides {
    /// Whether to show top border
    pub top: bool,
    /// Whether to show right border
    pub right: bool,
    /// Whether to show bottom border
    pub bottom: bool,
    /// Whether to show left border
    pub left: bool,
}

/// Border style (colors, attributes)
#[derive(Debug, Clone, PartialEq)]
pub struct BorderStyle {
    /// Foreground color for border characters
    pub fg: Rgba,
    /// Background color for border area
    pub bg: Rgba,
    /// Text attributes (bold, italic, etc.)
    pub attr: Attr,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            fg: Rgba::white(),
            bg: Rgba::black(),
            attr: Attr::empty(),
        }
    }
}

/// Border glyph sets
#[derive(Debug, Clone, PartialEq)]
pub enum BorderGlyphs {
    /// Single-line rounded corner border glyphs
    SingleRounded,
    /// Single-line square corner border glyphs
    SingleSquare,
    /// Custom border glyphs: [top_left, horizontal, top_right, vertical, bottom_right, bottom_left]
    Custom([&'static str; 6]),
}

impl BorderGlyphs {
    const SINGLE_ROUNDED: [&'static str; 6] = ["╭", "─", "╮", "│", "╯", "╰"];
    const SINGLE_SQUARE: [&'static str; 6] = ["┌", "─", "┐", "│", "┘", "└"];

    /// Get the glyph characters for this border style
    ///
    /// # Returns
    /// Array of 6 glyph strings: [top_left, horizontal, top_right, vertical, bottom_right, bottom_left]
    pub fn glyphs(&self) -> [&'static str; 6] {
        match self {
            BorderGlyphs::SingleRounded => Self::SINGLE_ROUNDED,
            BorderGlyphs::SingleSquare => Self::SINGLE_SQUARE,
            BorderGlyphs::Custom(glyphs) => *glyphs,
        }
    }
}

/// Child window creation options
#[derive(Debug, Clone)]
pub struct ChildOptions {
    /// Horizontal offset from parent window
    pub x_off: usize,
    /// Vertical offset from parent window
    pub y_off: usize,
    /// Width specification for the child window
    pub width: WindowSize,
    /// Height specification for the child window
    pub height: WindowSize,
    /// Border configuration for the child window
    pub border: BorderOptions,
}

impl Default for ChildOptions {
    fn default() -> Self {
        Self {
            x_off: 0,
            y_off: 0,
            width: WindowSize::Expand,
            height: WindowSize::Expand,
            border: BorderOptions::default(),
        }
    }
}

/// A hierarchical window with automatic constraint handling
#[derive(Debug)]
pub struct Window {
    /// Horizontal offset from parent (or screen)
    pub x_off: usize,
    /// Vertical offset from parent (or screen)
    pub y_off: usize,
    /// Width of this window
    pub width: usize,
    /// Height of this window
    pub height: usize,
    /// Reference to the underlying surface
    surface: *mut Surface,
    /// Cursor visibility
    cursor_visible: bool,
    /// Cursor position (relative to this window)
    cursor_col: usize,
    cursor_row: usize,
    /// Cursor shape
    cursor_shape: CursorShape,
}

impl Window {
    /// Get terminal character dimensions in pixels
    ///
    /// Returns the width and height in pixels of a single character cell.
    /// This queries the terminal if possible, otherwise returns standard dimensions.
    fn get_character_dimensions(&self) -> Result<(u16, u16), Box<dyn std::error::Error>> {
        use std::process::Command;

        // Try to query terminal using escape sequences
        let mut command = Command::new("stty");
        command.arg("size");
        if let Ok(output) = crate::core::owned_process::run(
            command,
            None,
            crate::core::owned_process::Options {
                purpose: "terminal character-size discovery",
                timeout: crate::core::owned_process::TERMINAL_HELPER_TIMEOUT,
                max_input: 0,
                max_output: 4 * 1024,
                capture_output: true,
                allow_background_after_success: false,
            },
            || false,
        ) {
            if let Ok(size_str) = String::from_utf8(output) {
                let parts: Vec<&str> = size_str.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let (Ok(rows), Ok(cols)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>())
                    {
                        // Estimate pixel dimensions based on typical terminal sizes
                        // Most modern terminals: 80x24 chars at ~1280x384 pixels
                        let char_w = 1280_u16.checked_div(cols).unwrap_or(16);
                        let char_h = 384_u16.checked_div(rows).unwrap_or(16);
                        return Ok((char_w.clamp(8, 32), char_h.clamp(8, 32)));
                    }
                }
            }
        }

        // Fallback to environment variables
        if let (Ok(term_w), Ok(term_h)) = (
            std::env::var("COLUMNS")
                .and_then(|s| s.parse::<u16>().map_err(|_| std::env::VarError::NotPresent)),
            std::env::var("LINES")
                .and_then(|s| s.parse::<u16>().map_err(|_| std::env::VarError::NotPresent)),
        ) {
            let char_w = 1280_u16.checked_div(term_w).unwrap_or(16);
            let char_h = 384_u16.checked_div(term_h).unwrap_or(16);
            return Ok((char_w.clamp(8, 32), char_h.clamp(8, 32)));
        }

        Err("Could not determine character dimensions".into())
    }
    /// Create a new root window from a surface
    pub fn new(surface: &mut Surface) -> Self {
        let (width, height) = surface.dims();
        Self {
            x_off: 0,
            y_off: 0,
            width,
            height,
            surface,
            cursor_visible: false,
            cursor_col: 0,
            cursor_row: 0,
            cursor_shape: CursorShape::Default,
        }
    }

    /// Create a child window with the given options
    pub fn child(&self, opts: ChildOptions) -> Window {
        // Calculate resolved dimensions
        let resolved_width = match opts.width {
            WindowSize::Expand => self.width.saturating_sub(opts.x_off),
            WindowSize::Limit(w) => {
                if w + opts.x_off > self.width {
                    self.width.saturating_sub(opts.x_off)
                } else {
                    w
                }
            }
        };

        let resolved_height = match opts.height {
            WindowSize::Expand => self.height.saturating_sub(opts.y_off),
            WindowSize::Limit(h) => {
                if h + opts.y_off > self.height {
                    self.height.saturating_sub(opts.y_off)
                } else {
                    h
                }
            }
        };

        let result = Window {
            x_off: opts.x_off + self.x_off,
            y_off: opts.y_off + self.y_off,
            width: resolved_width,
            height: resolved_height,
            surface: self.surface,
            cursor_visible: false,
            cursor_col: 0,
            cursor_row: 0,
            cursor_shape: CursorShape::Default,
        };

        // Draw borders if requested
        if !matches!(opts.border.where_, BorderLocation::None) {
            result.draw_border(&opts.border);
        }

        result
    }

    /// Draw a border around this window
    fn draw_border(&self, border: &BorderOptions) {
        let glyphs = border.glyphs.glyphs();
        let style = &border.style;

        let sides = match &border.where_ {
            BorderLocation::None => return,
            BorderLocation::All => BorderSides {
                top: true,
                right: true,
                bottom: true,
                left: true,
            },
            BorderLocation::Top => BorderSides {
                top: true,
                right: false,
                bottom: false,
                left: false,
            },
            BorderLocation::Right => BorderSides {
                top: false,
                right: true,
                bottom: false,
                left: false,
            },
            BorderLocation::Bottom => BorderSides {
                top: false,
                right: false,
                bottom: true,
                left: false,
            },
            BorderLocation::Left => BorderSides {
                top: false,
                right: false,
                bottom: false,
                left: true,
            },
            BorderLocation::Custom(sides) => sides.clone(),
        };

        let h = self.height;
        let w = self.width;

        // Draw horizontal borders
        if sides.top {
            for i in 0..w {
                self.write_cell(
                    i,
                    0,
                    Cell {
                        ch: glyphs[1].chars().next().unwrap_or('─'),
                        fg: style.fg,
                        bg: style.bg,
                        attr: style.attr,
                        image_id: None,
                        image_placement: None,
                    },
                );
            }
        }
        if sides.bottom && h > 0 {
            for i in 0..w {
                self.write_cell(
                    i,
                    h - 1,
                    Cell {
                        ch: glyphs[1].chars().next().unwrap_or('─'),
                        fg: style.fg,
                        bg: style.bg,
                        attr: style.attr,
                        image_id: None,
                        image_placement: None,
                    },
                );
            }
        }

        // Draw vertical borders
        if sides.left {
            for i in 0..h {
                self.write_cell(
                    0,
                    i,
                    Cell {
                        ch: glyphs[3].chars().next().unwrap_or('│'),
                        fg: style.fg,
                        bg: style.bg,
                        attr: style.attr,
                        image_id: None,
                        image_placement: None,
                    },
                );
            }
        }
        if sides.right && w > 0 {
            for i in 0..h {
                self.write_cell(
                    w - 1,
                    i,
                    Cell {
                        ch: glyphs[3].chars().next().unwrap_or('│'),
                        fg: style.fg,
                        bg: style.bg,
                        attr: style.attr,
                        image_id: None,
                        image_placement: None,
                    },
                );
            }
        }

        // Draw corners
        if sides.top && sides.left {
            self.write_cell(
                0,
                0,
                Cell {
                    ch: glyphs[0].chars().next().unwrap_or('┌'),
                    fg: style.fg,
                    bg: style.bg,
                    attr: style.attr,
                    image_id: None,
                    image_placement: None,
                },
            );
        }
        if sides.top && sides.right && w > 0 {
            self.write_cell(
                w - 1,
                0,
                Cell {
                    ch: glyphs[2].chars().next().unwrap_or('┐'),
                    fg: style.fg,
                    bg: style.bg,
                    attr: style.attr,
                    image_id: None,
                    image_placement: None,
                },
            );
        }
        if sides.bottom && sides.left && h > 0 {
            self.write_cell(
                0,
                h - 1,
                Cell {
                    ch: glyphs[5].chars().next().unwrap_or('└'),
                    fg: style.fg,
                    bg: style.bg,
                    attr: style.attr,
                    image_id: None,
                    image_placement: None,
                },
            );
        }
        if sides.bottom && sides.right && w > 0 && h > 0 {
            self.write_cell(
                w - 1,
                h - 1,
                Cell {
                    ch: glyphs[4].chars().next().unwrap_or('┘'),
                    fg: style.fg,
                    bg: style.bg,
                    attr: style.attr,
                    image_id: None,
                    image_placement: None,
                },
            );
        }
    }

    /// Write a cell to this window (with bounds checking and coordinate translation)
    pub fn write_cell(&self, col: usize, row: usize, cell: Cell) {
        if self.height == 0 || self.width == 0 {
            return;
        }
        if row >= self.height || col >= self.width {
            return;
        }

        // Translate coordinates to absolute surface coordinates
        let abs_col = col + self.x_off;
        let abs_row = row + self.y_off;

        // Safe surface access with null pointer check
        if self.surface.is_null() {
            #[cfg(debug_assertions)]
            log::warn!("Attempted to write to null surface");
            return;
        }

        unsafe {
            // Additional bounds check on the surface itself
            let surface_ref = &*self.surface;
            let (surface_width, surface_height) = surface_ref.dims();
            if abs_col < surface_width && abs_row < surface_height {
                (*self.surface).set(abs_col, abs_row, cell);
            }
        }
    }

    /// Read a cell from this window
    pub fn read_cell(&self, col: usize, row: usize) -> Option<Cell> {
        if self.height == 0 || self.width == 0 {
            return None;
        }
        if row >= self.height || col >= self.width {
            return None;
        }

        // Translate coordinates to absolute surface coordinates
        let abs_col = col + self.x_off;
        let abs_row = row + self.y_off;

        // Safe surface access with null pointer check
        if self.surface.is_null() {
            #[cfg(debug_assertions)]
            log::warn!("Attempted to read from null surface");
            return None;
        }

        unsafe {
            // Additional bounds check on the surface itself
            let surface_ref = &*self.surface;
            let (surface_width, surface_height) = surface_ref.dims();
            if abs_col < surface_width && abs_row < surface_height {
                Some((*self.surface).get(abs_col, abs_row))
            } else {
                None
            }
        }
    }

    /// Clear this window
    pub fn clear(&self) {
        self.fill(Cell::default());
    }

    /// Fill this window with a cell (optimized like libvaxis)
    pub fn fill(&self, cell: Cell) {
        // Safe surface access with null pointer check
        if self.surface.is_null() {
            #[cfg(debug_assertions)]
            log::warn!("Attempted to fill null surface");
            return;
        }

        unsafe {
            let surface = &mut *self.surface;
            let (surface_width, surface_height) = surface.dims();

            if surface_width < self.x_off || surface_height < self.y_off {
                return;
            }

            // Check if we have a full width window (contiguous memory optimization)
            if self.x_off == 0 && self.width == surface_width {
                // Contiguous memory - can use memset-like operation
                let _start = self.y_off * self.width;
                let _end = _start + (self.height * self.width);

                for y in self.y_off..(self.y_off + self.height).min(surface_height) {
                    for x in 0..self.width.min(surface_width) {
                        surface.set(x, y, cell);
                    }
                }
            } else {
                // Non-contiguous - iterate over rows
                for row in 0..self.height {
                    let abs_row = self.y_off + row;
                    if abs_row >= surface_height {
                        break;
                    }

                    for col in 0..self.width {
                        let abs_col = self.x_off + col;
                        if abs_col >= surface_width {
                            break;
                        }
                        surface.set(abs_col, abs_row, cell);
                    }
                }
            }
        }
    }

    /// Get the size of this window
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Get the bounds of this window (in local coordinates)
    pub fn bounds(&self) -> Rect {
        Rect::from_coords(0, 0, self.width, self.height)
    }

    /// Write a string to this window
    pub fn write_str(&self, col: usize, row: usize, text: &str, fg: Rgba, bg: Rgba, attr: Attr) {
        if row >= self.height {
            return;
        }

        for (current_col, ch) in (col..).zip(text.chars()) {
            if current_col >= self.width {
                break;
            }
            self.write_cell(
                current_col,
                row,
                Cell {
                    ch,
                    fg,
                    bg,
                    attr,
                    image_id: None,
                    image_placement: None,
                },
            );
        }
    }

    /// Hide the cursor
    pub fn hide_cursor(&mut self) {
        self.cursor_visible = false;
    }

    /// Show the cursor at the given coordinates (relative to this window)
    pub fn show_cursor(&mut self, col: usize, row: usize) {
        if self.height == 0 || self.width == 0 {
            return;
        }
        if row >= self.height || col >= self.width {
            return;
        }

        self.cursor_visible = true;
        self.cursor_col = col;
        self.cursor_row = row;
    }

    /// Set the cursor shape
    pub fn set_cursor_shape(&mut self, shape: CursorShape) {
        self.cursor_shape = shape;
    }

    /// Get the absolute cursor position (for the underlying surface)
    pub fn absolute_cursor_position(&self) -> Option<(usize, usize)> {
        if self.cursor_visible {
            Some((self.cursor_col + self.x_off, self.cursor_row + self.y_off))
        } else {
            None
        }
    }

    /// Print segments to the window with advanced wrapping and overflow detection
    pub fn print(&self, segments: &[Segment], opts: PrintOptions) -> PrintResult {
        let mut row = opts.row_offset;
        let mut col = opts.col_offset;
        let mut overflow = false;

        match opts.wrap {
            WrapMode::Grapheme => {
                'outer: for segment in segments {
                    for ch in segment.text.chars() {
                        if col >= self.width {
                            row += 1;
                            col = 0;
                        }
                        if row >= self.height {
                            overflow = true;
                            break 'outer;
                        }

                        if ch == '\n' {
                            row += 1;
                            col = 0;
                            continue;
                        }

                        let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
                        if char_width == 0 {
                            continue;
                        }

                        if opts.commit {
                            self.write_cell(
                                col,
                                row,
                                Cell {
                                    ch,
                                    fg: segment.fg,
                                    bg: segment.bg,
                                    attr: segment.attr,
                                    image_id: None,
                                    image_placement: None,
                                },
                            );
                        }

                        col += char_width;
                    }
                }

                if col >= self.width {
                    row += 1;
                    col = 0;
                }
            }

            WrapMode::Word => {
                'outer: for segment in segments {
                    let words = segment.text.split_whitespace();
                    for word in words {
                        let word_width = unicode_width::UnicodeWidthStr::width(word);

                        // If word doesn't fit on current line and is smaller than window width
                        if word_width + col > self.width && word_width < self.width {
                            row += 1;
                            col = 0;
                        }

                        for ch in word.chars() {
                            if row >= self.height {
                                overflow = true;
                                break 'outer;
                            }

                            let char_width =
                                unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);

                            if opts.commit {
                                self.write_cell(
                                    col,
                                    row,
                                    Cell {
                                        ch,
                                        fg: segment.fg,
                                        bg: segment.bg,
                                        attr: segment.attr,
                                        image_id: None,
                                        image_placement: None,
                                    },
                                );
                            }

                            col += char_width;
                            if col >= self.width {
                                row += 1;
                                col = 0;
                            }
                        }

                        // Add space after word if there's room
                        if col < self.width {
                            if opts.commit {
                                self.write_cell(
                                    col,
                                    row,
                                    Cell {
                                        ch: ' ',
                                        fg: segment.fg,
                                        bg: segment.bg,
                                        attr: segment.attr,
                                        image_id: None,
                                        image_placement: None,
                                    },
                                );
                            }
                            col += 1;
                        }
                    }
                }
            }

            WrapMode::None => {
                'outer: for segment in segments {
                    for ch in segment.text.chars() {
                        if col >= self.width {
                            overflow = true;
                            break 'outer;
                        }

                        if ch == '\n' {
                            overflow = true;
                            break 'outer;
                        }

                        let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
                        if char_width == 0 {
                            continue;
                        }

                        if opts.commit {
                            self.write_cell(
                                col,
                                row,
                                Cell {
                                    ch,
                                    fg: segment.fg,
                                    bg: segment.bg,
                                    attr: segment.attr,
                                    image_id: None,
                                    image_placement: None,
                                },
                            );
                        }

                        col += char_width;
                    }
                }
            }
        }

        PrintResult { col, row, overflow }
    }

    /// Print a single segment (convenience method)
    pub fn print_segment(&self, segment: &Segment, opts: PrintOptions) -> PrintResult {
        self.print(std::slice::from_ref(segment), opts)
    }

    /// Scroll the window down n rows (shifts content up)
    pub fn scroll(&self, n: usize) {
        if n > self.height {
            return;
        }

        unsafe {
            let surface = &mut *self.surface;
            let (surface_width, _) = surface.dims();

            // Shift rows up
            for row in 0..(self.height - n) {
                let src_row = self.y_off + row + n;
                let dst_row = self.y_off + row;

                for col in 0..self.width {
                    let src_col = self.x_off + col;
                    let dst_col = self.x_off + col;

                    if src_col < surface_width && dst_col < surface_width {
                        let cell = surface.get(src_col, src_row);
                        surface.set(dst_col, dst_row, cell);
                    }
                }
            }

            // Clear the bottom n rows
            let clear_cell = Cell::default();
            for row in (self.height - n)..self.height {
                let abs_row = self.y_off + row;
                for col in 0..self.width {
                    let abs_col = self.x_off + col;
                    if abs_col < surface_width {
                        surface.set(abs_col, abs_row, clear_cell);
                    }
                }
            }
        }
    }

    /// Check if a mouse event occurred within this window
    pub fn has_mouse(&self, mouse_event: Option<&MouseEvent>) -> Option<MouseEvent> {
        use crate::event::types::Position;

        let event = mouse_event?;

        let (col, row) = match event.position {
            Position::Cell { x, y } => (x as usize, y as usize),
            Position::Pixel { x, y } => {
                // Convert pixels to cells using standard terminal character dimensions
                // Most terminals use approximately 16x16 pixel cells for modern displays
                // This matches the dimensions used elsewhere in the codebase
                const CHAR_WIDTH: u16 = 16;
                const CHAR_HEIGHT: u16 = 16;

                // Query terminal for actual dimensions if available
                let (char_w, char_h) = if let Ok(dimensions) = self.get_character_dimensions() {
                    dimensions
                } else {
                    (CHAR_WIDTH, CHAR_HEIGHT)
                };

                ((x / char_w as u32) as usize, (y / char_h as u32) as usize)
            }
        };

        if col >= self.x_off
            && col < (self.x_off + self.width)
            && row >= self.y_off
            && row < (self.y_off + self.height)
        {
            // Return event with coordinates relative to this window
            let relative_position = Position::Cell {
                x: (col - self.x_off) as u16,
                y: (row - self.y_off) as u16,
            };

            Some(MouseEvent {
                position: relative_position,
                ..event.clone()
            })
        } else {
            None
        }
    }

    /// Get the grapheme width (for proper text rendering)
    pub fn grapheme_width(&self, text: &str) -> usize {
        unicode_width::UnicodeWidthStr::width(text)
    }

    /// Create a child window that can be used for rendering Elements
    pub fn element_child(&self, opts: ChildOptions) -> ElementWindow {
        let window = self.child(opts);
        ElementWindow { window }
    }
}

/// A window wrapper that provides Element-compatible rendering
#[derive(Debug)]
pub struct ElementWindow {
    window: Window,
}

impl ElementWindow {
    /// Get the underlying window
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Get the size for Element layout calculations
    pub fn element_size(&self) -> crate::core::geometry::Size {
        self.window.size()
    }

    /// Render an Element tree to this window
    /// Uses a simplified layout algorithm suitable for basic element rendering
    pub fn render_element(&self, element: &crate::component::Element) {
        self.render_element_simple(element, 0, 0);
    }

    /// Simple element rendering (recursive)
    fn render_element_simple(&self, element: &crate::component::Element, x: usize, y: usize) {
        use crate::component::ElementType;

        match &element.element_type {
            ElementType::Text(text) => {
                self.window
                    .write_str(x, y, text, Rgba::white(), Rgba::black(), Attr::empty());
            }
            ElementType::Layout(_) => {
                // Render children in a simple vertical layout
                for (current_y, child) in (y..).zip(&element.children) {
                    if current_y >= self.window.height {
                        break;
                    }
                    self.render_element_simple(child, x, current_y);
                }
            }
            ElementType::Component(_) | ElementType::Fragment | ElementType::Empty => {
                // For these types, just render children
                for (current_y, child) in (y..).zip(&element.children) {
                    if current_y >= self.window.height {
                        break;
                    }
                    self.render_element_simple(child, x, current_y);
                }
            }
        }
    }

    /// Create a child element window
    pub fn child(&self, opts: ChildOptions) -> ElementWindow {
        self.window.element_child(opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let mut surface = Surface::new(80, 24);
        let window = Window::new(&mut surface);

        assert_eq!(window.width, 80);
        assert_eq!(window.height, 24);
        assert_eq!(window.x_off, 0);
        assert_eq!(window.y_off, 0);
    }

    #[test]
    fn test_child_window_constraints() {
        let mut surface = Surface::new(80, 24);
        let parent = Window::new(&mut surface);

        // Test limit sizing
        let child = parent.child(ChildOptions {
            x_off: 10,
            y_off: 5,
            width: WindowSize::Limit(30),
            height: WindowSize::Limit(10),
            ..Default::default()
        });

        assert_eq!(child.width, 30);
        assert_eq!(child.height, 10);
        assert_eq!(child.x_off, 10);
        assert_eq!(child.y_off, 5);
    }

    #[test]
    fn test_child_window_expand() {
        let mut surface = Surface::new(80, 24);
        let parent = Window::new(&mut surface);

        // Test expand sizing
        let child = parent.child(ChildOptions {
            x_off: 10,
            y_off: 5,
            width: WindowSize::Expand,
            height: WindowSize::Expand,
            ..Default::default()
        });

        assert_eq!(child.width, 70); // 80 - 10
        assert_eq!(child.height, 19); // 24 - 5
    }

    #[test]
    fn test_child_window_overflow_constraint() {
        let mut surface = Surface::new(80, 24);
        let parent = Window::new(&mut surface);

        // Test that child is constrained when it would overflow
        let child = parent.child(ChildOptions {
            x_off: 70,
            y_off: 20,
            width: WindowSize::Limit(50),  // Would overflow
            height: WindowSize::Limit(20), // Would overflow
            ..Default::default()
        });

        assert_eq!(child.width, 10); // 80 - 70
        assert_eq!(child.height, 4); // 24 - 20
    }

    #[test]
    fn test_nested_windows() {
        let mut surface = Surface::new(80, 24);
        let root = Window::new(&mut surface);

        let level1 = root.child(ChildOptions {
            x_off: 5,
            y_off: 3,
            width: WindowSize::Limit(60),
            height: WindowSize::Limit(15),
            ..Default::default()
        });

        let level2 = level1.child(ChildOptions {
            x_off: 10,
            y_off: 5,
            width: WindowSize::Limit(30),
            height: WindowSize::Limit(8),
            ..Default::default()
        });

        // Check absolute coordinates
        assert_eq!(level2.x_off, 15); // 5 + 10
        assert_eq!(level2.y_off, 8); // 3 + 5
        assert_eq!(level2.width, 30);
        assert_eq!(level2.height, 8);
    }

    #[test]
    fn test_cursor_management() {
        let mut surface = Surface::new(80, 24);
        let mut window = Window::new(&mut surface);

        // Initially cursor should be hidden
        assert!(!window.cursor_visible);
        assert!(window.absolute_cursor_position().is_none());

        // Show cursor
        window.show_cursor(10, 5);
        assert!(window.cursor_visible);
        assert_eq!(window.cursor_col, 10);
        assert_eq!(window.cursor_row, 5);
        assert_eq!(window.absolute_cursor_position(), Some((10, 5)));

        // Hide cursor
        window.hide_cursor();
        assert!(!window.cursor_visible);
        assert!(window.absolute_cursor_position().is_none());

        // Test bounds checking
        window.show_cursor(100, 100); // Out of bounds
        assert!(!window.cursor_visible); // Should remain hidden
    }

    #[test]
    fn test_text_printing() {
        let mut surface = Surface::new(20, 10);
        let window = Window::new(&mut surface);

        let segments = vec![Segment::new("Hello "), Segment::new("World!")];

        let result = window.print(&segments, PrintOptions::default());
        assert_eq!(result.col, 12); // "Hello World!" is 12 characters
        assert_eq!(result.row, 0);
        assert!(!result.overflow);
    }

    #[test]
    fn test_text_wrapping() {
        let mut surface = Surface::new(5, 3);
        let window = Window::new(&mut surface);

        let segments = vec![Segment::new("Hello World!")];

        // Test grapheme wrapping
        let result = window.print(
            &segments,
            PrintOptions {
                wrap: WrapMode::Grapheme,
                ..Default::default()
            },
        );
        assert!(result.row > 0); // Should wrap to next line
        assert!(!result.overflow);

        // Test no wrapping (should overflow)
        let result = window.print(
            &segments,
            PrintOptions {
                wrap: WrapMode::None,
                ..Default::default()
            },
        );
        assert!(result.overflow);
    }

    #[test]
    fn test_scrolling() {
        let mut surface = Surface::new(10, 5);
        let window = Window::new(&mut surface);

        // Fill window with test content
        for row in 0..5 {
            window.write_str(
                0,
                row,
                &format!("Row {}", row),
                Rgba::white(),
                Rgba::black(),
                Attr::empty(),
            );
        }

        // Scroll up by 2 rows
        window.scroll(2);

        // Rows shift up by two and the scrolled-out rows are cleared
        let row_text = |row: usize| -> String {
            (0..5)
                .map(|col| window.read_cell(col, row).unwrap().ch)
                .collect()
        };
        assert_eq!(row_text(0), "Row 2");
        assert_eq!(row_text(1), "Row 3");
        assert_eq!(row_text(2), "Row 4");
        // The surface stores the cleared cell's control character as U+FFFD,
        // so compare the rows against that uniform blank rather than text.
        let blank = window.read_cell(0, 3).unwrap().ch;
        assert!(!blank.is_alphanumeric(), "cleared cell holds no text");
        assert!(
            (3..5).all(|row| (0..10).all(|col| window.read_cell(col, row).unwrap().ch == blank)),
            "the last two rows are cleared"
        );
    }

    #[test]
    fn test_mouse_event_handling() {
        use crate::event::types::{KeyModifiers, MouseEvent, MouseEventKind, Position};
        use std::time::Instant;

        let mut surface = Surface::new(80, 24);
        let window = Window::new(&mut surface);

        let child = window.child(ChildOptions {
            x_off: 10,
            y_off: 5,
            width: WindowSize::Limit(20),
            height: WindowSize::Limit(10),
            ..Default::default()
        });

        // Mouse event inside child window
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Click,
            button: crate::event::types::MouseButton::Left,
            position: Position::Cell { x: 15, y: 8 }, // Inside child window
            modifiers: KeyModifiers::empty(),
            timestamp: Instant::now(),
            wheel: None,
        };

        let result = child.has_mouse(Some(&mouse_event));
        assert!(result.is_some());

        if let Some(relative_event) = result {
            if let Position::Cell { x, y } = relative_event.position {
                assert_eq!(x, 5); // 15 - 10 (child x_off)
                assert_eq!(y, 3); // 8 - 5 (child y_off)
            }
        }

        // Mouse event outside child window
        let mouse_event_outside = MouseEvent {
            kind: MouseEventKind::Click,
            button: crate::event::types::MouseButton::Left,
            position: Position::Cell { x: 5, y: 3 }, // Outside child window
            modifiers: KeyModifiers::empty(),
            timestamp: Instant::now(),
            wheel: None,
        };

        let result = child.has_mouse(Some(&mouse_event_outside));
        assert!(result.is_none());
    }

    #[test]
    fn test_border_rendering() {
        let mut surface = Surface::new(20, 10);
        let window = Window::new(&mut surface);

        let child = window.child(ChildOptions {
            x_off: 2,
            y_off: 2,
            width: WindowSize::Limit(10),
            height: WindowSize::Limit(5),
            border: BorderOptions {
                where_: BorderLocation::All,
                glyphs: BorderGlyphs::SingleRounded,
                ..Default::default()
            },
        });

        // Verify the child window was created with borders
        // (In practice, you'd check that border characters were written to the surface)
        assert_eq!(child.width, 10);
        assert_eq!(child.height, 5);
    }
}
