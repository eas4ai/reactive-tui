//! Embedded virtual terminal (TRM-006, TRM-007).
//!
//! Ports `embedded-terminal/main.zig` and `compositor.zig`. The Ghostty
//! engine is replaced by a pure-Rust core: a `vte` parser drives a
//! caller-owned screen grid with scrollback, selection, cursor state,
//! and a PTY response queue (see the engine decision in
//! `docs/decisions/`). TRM-006 observes composed grids, which any
//! conforming engine produces, so the ported vectors apply directly.
//!
//! Input-side encoders (key, mouse, paste, focus) are not a TRM
//! requirement and are out of scope.

use crate::ansi::{TextAttributes, rgb_color};
use crate::buffer::{OptimizedBuffer, make_cell};
use crate::uni::{WidthMethod, width_at};
use vte::{Params, Parser, Perform};

/// Maximum queued PTY response bytes before the sticky overflow error.
pub const RESPONSE_LIMIT: usize = 1024 * 1024;

/// Embedded terminal failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedError {
    /// Zero dimensions, or selection outside the viewport.
    InvalidValue,
    /// Response queue exceeded [`RESPONSE_LIMIT`].
    ResponseOverflow,
    /// Target buffer refused an allocation.
    OutOfMemory,
}

/// Viewport coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

/// Reported cursor state. Mirrors `main.zig`'s `Cursor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorState {
    pub x: u16,
    pub y: u16,
    pub has_value: bool,
    pub visible: bool,
    pub blinking: bool,
    pub wide_tail: bool,
    /// 0 = bar, 1 = block, 2 = underline, 3 = hollow.
    pub style: u8,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

const DEFAULT_FG: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 255,
};
const DEFAULT_BG: Rgb = Rgb { r: 0, g: 0, b: 0 };

fn palette_entry(index: u8) -> Rgb {
    const BASE: [[u8; 3]; 16] = [
        [0, 0, 0],
        [170, 0, 0],
        [0, 170, 0],
        [170, 85, 0],
        [0, 0, 170],
        [170, 0, 170],
        [0, 170, 170],
        [170, 170, 170],
        [85, 85, 85],
        [255, 85, 85],
        [85, 255, 85],
        [255, 255, 85],
        [85, 85, 255],
        [255, 85, 255],
        [85, 255, 255],
        [255, 255, 255],
    ];
    let c = BASE[usize::from(index & 15)];
    Rgb {
        r: c[0],
        g: c[1],
        b: c[2],
    }
}

fn indexed_color(index: u8) -> Rgb {
    if index < 16 {
        return palette_entry(index);
    }
    if index < 232 {
        const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
        let v = index - 16;
        return Rgb {
            r: LEVELS[usize::from(v / 36)],
            g: LEVELS[usize::from((v % 36) / 6)],
            b: LEVELS[usize::from(v % 6)],
        };
    }
    let gray = 8 + 10 * (index - 232);
    Rgb {
        r: gray,
        g: gray,
        b: gray,
    }
}

#[derive(Debug, Clone, Copy)]
struct Pen {
    fg: Rgb,
    bg: Rgb,
    attrs: u32,
    inverse: bool,
}

impl Default for Pen {
    fn default() -> Self {
        Self {
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            attrs: 0,
            inverse: false,
        }
    }
}

/// One grid cell. `width == 0` marks an untouched cell or a wide tail;
/// both render as the cleared background.
#[derive(Debug, Clone)]
struct VCell {
    text: String,
    width: u8,
    fg: Rgb,
    bg: Rgb,
    attrs: u32,
    inverse: bool,
}

impl VCell {
    fn empty() -> Self {
        Self {
            text: String::new(),
            width: 0,
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            attrs: 0,
            inverse: false,
        }
    }

    fn blank() -> Self {
        Self {
            width: 1,
            ..Self::empty()
        }
    }

    fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

#[derive(Debug, Clone)]
struct Line {
    cells: Vec<VCell>,
    /// The row continues on the next line (autowrap, not a newline).
    wrapped: bool,
}

impl Line {
    fn blank(cols: usize) -> Self {
        Self {
            cells: (0..cols).map(|_| VCell::empty()).collect(),
            wrapped: false,
        }
    }

    /// Grid end of the last non-empty cell (mirrors `text_end`).
    fn text_end(&self) -> usize {
        for (x, cell) in self.cells.iter().enumerate().rev() {
            if !cell.is_empty() {
                return x + usize::from(cell.width.max(1));
            }
        }
        0
    }
}

fn grid_width(c: char) -> u8 {
    if (c as u32) < 32 || (c as u32) == 127 {
        return 0;
    }
    let mut buf = [0u8; 4];
    let text = c.encode_utf8(&mut buf);
    width_at(text, 0, 8, WidthMethod::Unicode).min(2) as u8
}

struct Engine {
    cols: usize,
    rows: usize,
    scrollback: Vec<Line>,
    screen: Vec<Line>,
    scrollback_bytes: usize,
    max_scrollback_bytes: usize,
    view_offset: usize,
    cursor_x: usize,
    cursor_y: usize,
    cursor_visible: bool,
    cursor_style: u8,
    cursor_blink: bool,
    pen: Pen,
    saved_cursor: Option<(usize, usize, Pen)>,
    responses: Vec<u8>,
    response_error: Option<EmbeddedError>,
    selection: Option<(Point, Point)>,
    last_selection: Vec<Option<(u16, u16)>>,
    dirty: Vec<bool>,
    full_dirty: bool,
    composed_once: bool,
}

impl Engine {
    fn new(cols: usize, rows: usize, max_scrollback_bytes: usize) -> Self {
        Self {
            cols,
            rows,
            scrollback: Vec::new(),
            screen: (0..rows).map(|_| Line::blank(cols)).collect(),
            scrollback_bytes: 0,
            max_scrollback_bytes,
            view_offset: 0,
            cursor_x: 0,
            cursor_y: 0,
            cursor_visible: true,
            cursor_style: 1,
            cursor_blink: false,
            pen: Pen::default(),
            saved_cursor: None,
            responses: Vec::new(),
            response_error: None,
            selection: None,
            last_selection: vec![None; rows],
            dirty: vec![true; rows],
            full_dirty: true,
            composed_once: false,
        }
    }

    fn mark_row(&mut self, y: usize) {
        if y < self.rows {
            self.dirty[y] = true;
        }
    }

    fn mark_all(&mut self) {
        self.dirty.iter_mut().for_each(|d| *d = true);
    }

    fn push_response(&mut self, bytes: &[u8]) {
        if self.response_error.is_some() {
            return;
        }
        if bytes.len() > RESPONSE_LIMIT.saturating_sub(self.responses.len().min(RESPONSE_LIMIT)) {
            self.response_error = Some(EmbeddedError::ResponseOverflow);
            return;
        }
        self.responses.extend_from_slice(bytes);
    }

    fn scroll_up(&mut self) {
        let mut top = Line::blank(self.cols);
        std::mem::swap(&mut self.screen[0], &mut top);
        self.screen.remove(0);
        self.screen.push(Line::blank(self.cols));
        let bytes: usize = top.cells.iter().map(|c| c.text.len()).sum();
        self.scrollback_bytes += bytes;
        self.scrollback.push(top);
        while self.scrollback_bytes > self.max_scrollback_bytes {
            let Some(old) = self.scrollback.first() else {
                break;
            };
            let old_bytes: usize = old.cells.iter().map(|c| c.text.len()).sum();
            self.scrollback.remove(0);
            self.scrollback_bytes = self.scrollback_bytes.saturating_sub(old_bytes);
            if self.view_offset > 0 {
                self.view_offset -= 1;
            }
        }
        self.view_offset = 0;
        self.mark_all();
    }

    fn scroll_down(&mut self) {
        self.screen.pop();
        self.screen.insert(0, Line::blank(self.cols));
        self.mark_all();
    }

    fn line_feed(&mut self) {
        if self.cursor_y + 1 < self.rows {
            self.cursor_y += 1;
        } else {
            self.scroll_up();
        }
        self.view_offset = 0;
    }

    fn clamp_cursor(&mut self) {
        self.cursor_x = self.cursor_x.min(self.cols - 1);
        self.cursor_y = self.cursor_y.min(self.rows - 1);
    }

    /// Visible line for a viewport row, following the scroll offset.
    fn visible_line(&self, vy: usize) -> &Line {
        if self.view_offset == 0 {
            return &self.screen[vy];
        }
        let total = self.scrollback.len() + self.rows;
        let index = total - self.rows - self.view_offset + vy;
        if index < self.scrollback.len() {
            &self.scrollback[index]
        } else {
            &self.screen[index - self.scrollback.len()]
        }
    }

    fn selection_range_for_row(&self, vy: usize) -> Option<(usize, usize)> {
        let (mut a, mut b) = self.selection?;
        if (a.y, a.x) > (b.y, b.x) {
            std::mem::swap(&mut a, &mut b);
        }
        let vy = vy as u16;
        if vy < a.y || vy > b.y {
            return None;
        }
        let start = if vy == a.y { usize::from(a.x) } else { 0 };
        let end = if vy == b.y {
            usize::from(b.x)
        } else {
            self.cols.saturating_sub(1)
        };
        Some((start, end))
    }

    fn print_char(&mut self, c: char) {
        let w = grid_width(c);
        if w == 0 {
            // Combining mark: attach to the previously printed cell.
            if self.cursor_x > 0 {
                let line = &mut self.screen[self.cursor_y];
                let base = self.cursor_x - 1;
                if base < line.cells.len() {
                    line.cells[base].text.push(c);
                    self.mark_row(self.cursor_y);
                }
            }
            return;
        }
        if self.cursor_x + usize::from(w) > self.cols {
            self.screen[self.cursor_y].wrapped = true;
            self.line_feed();
            self.cursor_x = 0;
        }
        let y = self.cursor_y;
        let x = self.cursor_x;
        let line = &mut self.screen[y];
        line.cells[x] = VCell {
            text: c.to_string(),
            width: w,
            fg: self.pen.fg,
            bg: self.pen.bg,
            attrs: self.pen.attrs,
            inverse: self.pen.inverse,
        };
        if w == 2 && x + 1 < self.cols {
            line.cells[x + 1] = VCell::empty();
        }
        self.cursor_x = (x + usize::from(w)).min(self.cols);
        self.view_offset = 0;
        self.mark_row(y);
    }

    fn erase_cells(&mut self, y: usize, from: usize, to: usize) {
        let line = &mut self.screen[y];
        for x in from..to.min(line.cells.len()) {
            line.cells[x] = VCell::empty();
        }
        self.mark_row(y);
    }

    fn sgr(&mut self, groups: &[Vec<u16>]) {
        if groups.is_empty() {
            self.pen = Pen::default();
            return;
        }
        for group in groups {
            let first = group.first().copied().unwrap_or(0);
            match first {
                0 => self.pen = Pen::default(),
                1 => self.pen.attrs |= u32::from(TextAttributes::BOLD),
                2 => self.pen.attrs |= u32::from(TextAttributes::DIM),
                3 => self.pen.attrs |= u32::from(TextAttributes::ITALIC),
                4 => {
                    if group.get(1).copied().unwrap_or(1) == 0 {
                        self.pen.attrs &= !u32::from(TextAttributes::UNDERLINE);
                    } else {
                        self.pen.attrs |= u32::from(TextAttributes::UNDERLINE);
                    }
                }
                5 => self.pen.attrs |= u32::from(TextAttributes::BLINK),
                7 => self.pen.inverse = true,
                8 => self.pen.attrs |= u32::from(TextAttributes::HIDDEN),
                9 => self.pen.attrs |= u32::from(TextAttributes::STRIKETHROUGH),
                22 => {
                    self.pen.attrs &=
                        !(u32::from(TextAttributes::BOLD) | u32::from(TextAttributes::DIM));
                }
                23 => self.pen.attrs &= !u32::from(TextAttributes::ITALIC),
                24 => self.pen.attrs &= !u32::from(TextAttributes::UNDERLINE),
                25 => self.pen.attrs &= !u32::from(TextAttributes::BLINK),
                27 => self.pen.inverse = false,
                28 => self.pen.attrs &= !u32::from(TextAttributes::HIDDEN),
                29 => self.pen.attrs &= !u32::from(TextAttributes::STRIKETHROUGH),
                30..=37 => self.pen.fg = palette_entry((first - 30) as u8),
                39 => self.pen.fg = DEFAULT_FG,
                40..=47 => self.pen.bg = palette_entry((first - 40) as u8),
                49 => self.pen.bg = DEFAULT_BG,
                90..=97 => self.pen.fg = palette_entry((first - 90 + 8) as u8),
                100..=107 => self.pen.bg = palette_entry((first - 100 + 8) as u8),
                38 | 48 => {
                    let target = if first == 38 {
                        &mut self.pen.fg
                    } else {
                        &mut self.pen.bg
                    };
                    let flat: Vec<u16> = group[1..].to_vec();
                    if flat.first() == Some(&5) {
                        if let Some(index) = flat.get(1) {
                            *target = indexed_color(*index as u8);
                        }
                    } else if flat.first() == Some(&2) {
                        if flat.len() >= 4 {
                            *target = Rgb {
                                r: flat[1] as u8,
                                g: flat[2] as u8,
                                b: flat[3] as u8,
                            };
                        }
                    } else if first == 38 {
                        self.pen.fg = DEFAULT_FG;
                    } else {
                        self.pen.bg = DEFAULT_BG;
                    }
                }
                _ => {}
            }
        }
    }

    fn csi_params(params: &Params) -> Vec<Vec<u16>> {
        params.iter().map(|group| group.to_vec()).collect()
    }

    /// Flat `;`-separated values across groups (positionals like CUP).
    fn flat_params(params: &Params) -> Vec<u16> {
        params.iter().flatten().copied().collect()
    }

    fn num(flat: &[u16], index: usize, default: u16) -> u16 {
        flat.get(index)
            .copied()
            .filter(|v| *v != 0)
            .unwrap_or(default)
    }
}

impl Perform for Engine {
    fn print(&mut self, c: char) {
        self.print_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => self.cursor_x = self.cursor_x.saturating_sub(1),
            0x09 => {
                let next = (self.cursor_x / 8 + 1) * 8;
                if next >= self.cols {
                    self.line_feed();
                    self.cursor_x = 0;
                } else {
                    self.cursor_x = next;
                }
            }
            0x0a..=0x0c => self.line_feed(),
            0x0d => self.cursor_x = 0,
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        let private = intermediates.contains(&b'?');
        let groups = Self::csi_params(params);
        let flat = Self::flat_params(params);
        let num = |index: usize, default: u16| Self::num(&flat, index, default);
        let n = num(0, 1).max(1) as usize;
        match action {
            'A' if !private => self.cursor_y = self.cursor_y.saturating_sub(n),
            'B' if !private => {
                self.cursor_y = (self.cursor_y + n).min(self.rows - 1);
            }
            'C' if !private => {
                self.cursor_x = (self.cursor_x + n).min(self.cols - 1);
            }
            'D' if !private => self.cursor_x = self.cursor_x.saturating_sub(n),
            'E' if !private => {
                self.cursor_y = (self.cursor_y + n).min(self.rows - 1);
                self.cursor_x = 0;
            }
            'F' if !private => {
                self.cursor_y = self.cursor_y.saturating_sub(n);
                self.cursor_x = 0;
            }
            'G' if !private => {
                self.cursor_x = n.saturating_sub(1).min(self.cols - 1);
            }
            'H' | 'f' if !private => {
                let row = num(0, 1).max(1) as usize;
                let col = num(1, 1).max(1) as usize;
                self.cursor_y = row.saturating_sub(1).min(self.rows - 1);
                self.cursor_x = col.saturating_sub(1).min(self.cols - 1);
            }
            'J' if !private => match num(0, 0) {
                0 => {
                    let (x, y) = (self.cursor_x, self.cursor_y);
                    let cols = self.cols;
                    self.erase_cells(y, x, cols);
                    for row in y + 1..self.rows {
                        self.erase_cells(row, 0, cols);
                    }
                }
                1 => {
                    let (x, y) = (self.cursor_x, self.cursor_y);
                    for row in 0..y {
                        let cols = self.cols;
                        self.erase_cells(row, 0, cols);
                    }
                    self.erase_cells(y, 0, x + 1);
                }
                _ => {
                    for row in 0..self.rows {
                        let cols = self.cols;
                        self.erase_cells(row, 0, cols);
                    }
                }
            },
            'K' if !private => {
                let cols = self.cols;
                match num(0, 0) {
                    0 => self.erase_cells(self.cursor_y, self.cursor_x, cols),
                    1 => {
                        let x = self.cursor_x + 1;
                        self.erase_cells(self.cursor_y, 0, x);
                    }
                    _ => self.erase_cells(self.cursor_y, 0, cols),
                }
            }
            'L' if !private => {
                for _ in 0..n.min(self.rows) {
                    self.screen.pop();
                    self.screen.insert(self.cursor_y, Line::blank(self.cols));
                }
                self.mark_all();
            }
            'M' if !private => {
                for _ in 0..n.min(self.rows) {
                    self.screen.remove(self.cursor_y.min(self.rows - 1));
                    self.screen.push(Line::blank(self.cols));
                }
                self.mark_all();
            }
            'P' if !private => {
                let y = self.cursor_y;
                let line = &mut self.screen[y];
                let from = self.cursor_x.min(line.cells.len());
                let count = n.min(line.cells.len() - from);
                line.cells.drain(from..from + count);
                line.cells.extend((0..count).map(|_| VCell::empty()));
                self.mark_row(y);
            }
            'S' if !private => {
                for _ in 0..n {
                    self.scroll_up();
                }
            }
            'T' if !private => {
                for _ in 0..n {
                    self.scroll_down();
                }
            }
            'X' if !private => {
                let x = self.cursor_x;
                self.erase_cells(self.cursor_y, x, x + n);
            }
            'd' if !private => {
                let row = num(0, 1).max(1) as usize;
                self.cursor_y = row.saturating_sub(1).min(self.rows - 1);
            }
            'e' if !private => {
                self.cursor_y = (self.cursor_y + n).min(self.rows - 1);
            }
            'h' | 'l' if private => {
                // Mode sets the TRM scope does not observe; tracked
                // nowhere because no requirement reads them back.
            }
            'm' if !private => self.sgr(&groups),
            'n' if !private => match num(0, 0) {
                5 => self.push_response(b"\x1b[0n"),
                6 => {
                    let report = format!("\x1b[{};{}R", self.cursor_y + 1, self.cursor_x + 1);
                    self.push_response(report.as_bytes());
                }
                _ => {}
            },
            'q' if intermediates.contains(&b' ') => match num(0, 1) {
                0..=2 => {
                    self.cursor_style = 1;
                    self.cursor_blink = num(0, 1) != 2;
                }
                3 | 4 => {
                    self.cursor_style = 2;
                    self.cursor_blink = num(0, 1) == 3;
                }
                _ => {
                    self.cursor_style = 0;
                    self.cursor_blink = num(0, 1) == 5;
                }
            },
            's' if !private => {
                self.saved_cursor = Some((self.cursor_x, self.cursor_y, self.pen));
            }
            'u' if !private => {
                if let Some((x, y, pen)) = self.saved_cursor {
                    self.cursor_x = x.min(self.cols - 1);
                    self.cursor_y = y.min(self.rows - 1);
                    self.pen = pen;
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        if intermediates.is_empty() {
            match byte {
                b'7' => {
                    self.saved_cursor = Some((self.cursor_x, self.cursor_y, self.pen));
                }
                b'8' => {
                    if let Some((x, y, pen)) = self.saved_cursor {
                        self.cursor_x = x.min(self.cols - 1);
                        self.cursor_y = y.min(self.rows - 1);
                        self.pen = pen;
                    }
                }
                b'D' => self.line_feed(),
                b'E' => {
                    self.cursor_x = 0;
                    self.line_feed();
                }
                b'M' => {
                    if self.cursor_y == 0 {
                        self.scroll_down();
                    } else {
                        self.cursor_y -= 1;
                    }
                }
                b'c' => {
                    let cols = self.cols;
                    let rows = self.rows;
                    let max = self.max_scrollback_bytes;
                    *self = Self::new(cols, rows, max);
                }
                _ => {}
            }
        }
    }
}

/// Construction options. Mirrors `main.zig`'s `Options`.
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedOptions {
    pub cols: u16,
    pub rows: u16,
    pub max_scrollback: usize,
}

impl EmbeddedOptions {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols,
            rows,
            max_scrollback: 10_000,
        }
    }
}

/// Caller-owned embedded virtual terminal (TRM-006, TRM-007).
pub struct EmbeddedTerminal {
    parser: Parser,
    inner: Engine,
}

impl EmbeddedTerminal {
    pub fn new(options: EmbeddedOptions) -> Result<Self, EmbeddedError> {
        if options.cols == 0 || options.rows == 0 {
            return Err(EmbeddedError::InvalidValue);
        }
        Ok(Self {
            parser: Parser::new(),
            inner: Engine::new(
                usize::from(options.cols),
                usize::from(options.rows),
                options.max_scrollback,
            ),
        })
    }

    pub fn cols(&self) -> u16 {
        self.inner.cols as u16
    }

    pub fn rows(&self) -> u16 {
        self.inner.rows as u16
    }

    /// Feed program output bytes through the parser (TRM-006).
    pub fn write(&mut self, bytes: &[u8]) {
        let inner = &mut self.inner;
        self.parser.advance(inner, bytes);
    }

    /// Resize with content reflow; clears deduplication-era state such as
    /// the scroll offset (TRM-006).
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), EmbeddedError> {
        if cols == 0 || rows == 0 {
            return Err(EmbeddedError::InvalidValue);
        }
        let (cols, rows) = (usize::from(cols), usize::from(rows));
        // Reflow every retained line into the new width.
        let mut stream: Vec<(VCell, bool)> = Vec::new();
        for line in self.inner.scrollback.iter().chain(self.inner.screen.iter()) {
            let mut end = line.cells.len();
            while end > 0 && line.cells[end - 1].is_empty() {
                end -= 1;
            }
            for cell in &line.cells[..end] {
                if cell.width == 0 && cell.is_empty() {
                    stream.push((VCell::blank(), false));
                } else if cell.width > 0 {
                    stream.push((cell.clone(), false));
                }
            }
            if !line.wrapped {
                stream.push((VCell::empty(), true));
            }
        }
        let mut lines: Vec<Line> = Vec::new();
        let mut current = Line::blank(cols);
        let mut x = 0;
        for (cell, hard_break) in stream {
            if hard_break {
                lines.push(std::mem::replace(&mut current, Line::blank(cols)));
                x = 0;
                continue;
            }
            let w = usize::from(cell.width.max(1));
            if x + w > cols {
                current.wrapped = true;
                lines.push(std::mem::replace(&mut current, Line::blank(cols)));
                x = 0;
            }
            current.cells[x] = cell;
            if current.cells[x].width == 2 && x + 1 < cols {
                current.cells[x + 1] = VCell::empty();
            }
            x += w;
        }
        if x > 0 || lines.is_empty() {
            lines.push(current);
        }
        while lines.len() < rows {
            lines.push(Line::blank(cols));
        }
        let split = lines.len().saturating_sub(rows);
        let mut scrollback: Vec<Line> = lines.drain(..split).collect();
        let mut scrollback_bytes: usize = scrollback
            .iter()
            .flat_map(|l| &l.cells)
            .map(|c| c.text.len())
            .sum();
        while scrollback_bytes > self.inner.max_scrollback_bytes {
            let Some(old) = scrollback.first() else {
                break;
            };
            let old_bytes: usize = old.cells.iter().map(|c| c.text.len()).sum();
            scrollback.remove(0);
            scrollback_bytes = scrollback_bytes.saturating_sub(old_bytes);
        }
        self.inner.cols = cols;
        self.inner.rows = rows;
        self.inner.scrollback = scrollback;
        self.inner.scrollback_bytes = scrollback_bytes;
        self.inner.screen = lines;
        self.inner.view_offset = 0;
        self.inner.clamp_cursor();
        self.inner.selection = None;
        self.inner.last_selection = vec![None; rows];
        self.inner.dirty = vec![true; rows];
        self.inner.full_dirty = true;
        Ok(())
    }

    /// Scroll the viewport into scrollback; positive deltas move toward
    /// older lines (TRM-006).
    pub fn scroll(&mut self, delta: i32) {
        let max = self.inner.scrollback.len() as i32;
        let next = (self.inner.view_offset as i32 + delta).clamp(0, max);
        self.inner.view_offset = next as usize;
        self.inner.full_dirty = true;
    }

    /// Select viewport cells; reversed endpoints normalize (TRM-006).
    pub fn set_selection(&mut self, start: Point, end: Point) -> Result<(), EmbeddedError> {
        if usize::from(start.x) >= self.inner.cols
            || usize::from(end.x) >= self.inner.cols
            || usize::from(start.y) >= self.inner.rows
            || usize::from(end.y) >= self.inner.rows
        {
            return Err(EmbeddedError::InvalidValue);
        }
        self.inner.selection = Some((start, end));
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.inner.selection = None;
    }

    /// Extract the selected text: wrapped rows join without a newline,
    /// each line loses trailing blanks, trailing empty lines drop
    /// (TRM-006).
    pub fn selected_text(&self) -> String {
        let Some((mut a, mut b)) = self.inner.selection else {
            return String::new();
        };
        if (a.y, a.x) > (b.y, b.x) {
            std::mem::swap(&mut a, &mut b);
        }
        let mut out = String::new();
        for vy in usize::from(a.y)..=usize::from(b.y) {
            let line = self.inner.visible_line(vy);
            let start = if vy == usize::from(a.y) {
                usize::from(a.x)
            } else {
                0
            };
            let end = if vy == usize::from(b.y) {
                usize::from(b.x)
            } else {
                self.inner.cols.saturating_sub(1)
            };
            let text_end = line.text_end();
            let mut row = String::new();
            // A wide lead starting before the range but overlapping it
            // contributes its whole text (mirrors the highlight rule).
            let mut x = start;
            if x > 0 {
                let mut back = x;
                while back > 0 && line.cells[back].width == 0 && line.cells[back].is_empty() {
                    back -= 1;
                }
                let lead = &line.cells[back];
                if !lead.is_empty() && back + usize::from(lead.width.max(1)) > x {
                    row.push_str(&lead.text);
                    x = back + usize::from(lead.width.max(1));
                }
            }
            let mut skip_until = x;
            while x <= end && x < line.cells.len() {
                if x < skip_until {
                    x += 1;
                    continue;
                }
                if x >= text_end {
                    break;
                }
                let cell = &line.cells[x];
                if cell.is_empty() {
                    row.push(' ');
                    x += 1;
                } else {
                    let w = usize::from(cell.width.max(1));
                    row.push_str(&cell.text);
                    skip_until = x + w;
                    x += 1;
                }
            }
            while row.ends_with(' ') {
                row.pop();
            }
            out.push_str(&row);
            if vy != usize::from(b.y) && !line.wrapped {
                out.push('\n');
            }
        }
        while out.ends_with('\n') {
            out.pop();
        }
        out
    }

    /// Compose the visible screen onto the target at the origin with
    /// dirty-row granularity and clipping (TRM-006).
    pub fn compose(
        &mut self,
        target: &mut OptimizedBuffer,
        origin_x: i32,
        origin_y: i32,
    ) -> Result<(), EmbeddedError> {
        let inner = &mut self.inner;
        if inner.full_dirty {
            inner.mark_all();
        }
        if inner.view_offset != 0 {
            inner.mark_all();
        }
        // Selection changes dirty their rows even without new output.
        let current_sel: Vec<Option<(u16, u16)>> = (0..inner.rows)
            .map(|vy| {
                inner
                    .selection_range_for_row(vy)
                    .map(|(s, e)| (s as u16, e as u16))
            })
            .collect();
        for (vy, range) in current_sel.iter().enumerate() {
            if inner.last_selection.get(vy) != Some(range) {
                inner.dirty[vy] = true;
            }
        }
        inner.last_selection = current_sel;

        let (cols, rows) = (inner.cols, inner.rows);
        let (tw, th) = (target.width() as i32, target.height() as i32);
        let default_fg = rgb_color(255, 255, 255, 255);
        let default_bg = rgb_color(0, 0, 0, 255);
        for vy in 0..rows {
            if !inner.dirty[vy] {
                continue;
            }
            inner.dirty[vy] = false;
            let dest_y = origin_y + vy as i32;
            if dest_y < 0 || dest_y >= th {
                continue;
            }
            let line = inner.visible_line(vy).clone();
            for tx in 0..tw {
                let sx = tx - origin_x;
                if sx < 0 || sx >= cols as i32 {
                    continue;
                }
                target.set(
                    tx as u32,
                    dest_y as u32,
                    make_cell(32, default_fg, default_bg, 0),
                );
            }
            let sel = inner.selection_range_for_row(vy);
            let text_end = line.text_end();
            for (x, cell) in line.cells.iter().enumerate() {
                let dest_x = origin_x + x as i32;
                if dest_x < 0 || dest_x >= tw {
                    continue;
                }
                if cell.width == 0
                    && x > 0
                    && line.cells.get(x - 1).is_some_and(|lead| lead.width == 2)
                {
                    // Wide tail: colors propagate from the lead draw.
                    continue;
                }
                let (mut fg, mut bg) = (cell.fg, cell.bg);
                if cell.inverse {
                    std::mem::swap(&mut fg, &mut bg);
                }
                if let Some((s, e)) = sel {
                    let w = usize::from(cell.width.max(1));
                    if x < text_end && x + w > s && x <= e {
                        std::mem::swap(&mut fg, &mut bg);
                    }
                }
                let fg_rgba = rgb_color(fg.r, fg.g, fg.b, 255);
                let bg_rgba = rgb_color(bg.r, bg.g, bg.b, 255);
                if cell.is_empty() || cell.attrs & u32::from(TextAttributes::HIDDEN) != 0 {
                    target.set(
                        dest_x as u32,
                        dest_y as u32,
                        make_cell(32, fg_rgba, bg_rgba, cell.attrs),
                    );
                } else {
                    target
                        .draw_grapheme(
                            cell.text.as_bytes(),
                            cell.width,
                            dest_x as u32,
                            dest_y as u32,
                            fg_rgba,
                            bg_rgba,
                            cell.attrs,
                        )
                        .map_err(|_| EmbeddedError::OutOfMemory)?;
                }
            }
        }
        inner.full_dirty = false;
        inner.composed_once = true;
        Ok(())
    }

    /// Report cursor state from the last composed frame (TRM-006).
    pub fn cursor(&self) -> CursorState {
        let inner = &self.inner;
        let x = inner.cursor_x as u16;
        let y = inner.cursor_y as u16;
        let wide_tail = inner.cursor_x > 0
            && inner
                .screen
                .get(inner.cursor_y)
                .and_then(|line| line.cells.get(inner.cursor_x - 1))
                .is_some_and(|cell| cell.width == 2);
        CursorState {
            x,
            y,
            has_value: inner.composed_once,
            visible: inner.cursor_visible,
            blinking: inner.cursor_blink,
            wide_tail,
            style: inner.cursor_style,
            color: [DEFAULT_FG.r, DEFAULT_FG.g, DEFAULT_FG.b],
        }
    }

    /// Force a full redraw on the next compose (TRM-006).
    pub fn invalidate(&mut self) {
        self.inner.full_dirty = true;
    }

    /// Drain queued PTY responses: every byte exactly once, in order.
    /// A sticky overflow error reports once before later drains succeed
    /// with the preserved prefix (TRM-007).
    pub fn drain_responses(&mut self, output: &mut [u8]) -> Result<usize, EmbeddedError> {
        if let Some(err) = self.inner.response_error.take() {
            return Err(err);
        }
        let count = output.len().min(self.inner.responses.len());
        output[..count].copy_from_slice(&self.inner.responses[..count]);
        self.inner.responses.drain(..count);
        Ok(count)
    }
}
