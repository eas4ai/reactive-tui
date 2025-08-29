use super::geometry::{Point, Rect, Size};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    /// Create a transparent color
    pub fn transparent() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }

    /// Create white color
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    /// Create black color
    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

use crate::error::{RTuiError, Result};
use bitflags::bitflags;
bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Attr: u8 { const BOLD=1<<0; const ITALIC=1<<1; const UNDERLINE=1<<2; const REVERSE=1<<3; const STRIKE=1<<4; }
}

impl Attr {
    #[inline]
    pub fn from_flags(
        bold: bool,
        italic: bool,
        underline: bool,
        reverse: bool,
        strike: bool,
    ) -> Self {
        let mut a = Attr::empty();
        if bold {
            a |= Attr::BOLD;
        }
        if italic {
            a |= Attr::ITALIC;
        }
        if underline {
            a |= Attr::UNDERLINE;
        }
        if reverse {
            a |= Attr::REVERSE;
        }
        if strike {
            a |= Attr::STRIKE;
        }
        a
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Rgba,
    pub bg: Rgba,
    pub attr: Attr,
}

pub struct Surface {
    w: usize,
    h: usize,
    buf: Vec<Cell>,
}

impl Surface {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            buf: vec![Cell::default(); w * h],
        }
    }

    /// Create a new surface with validation
    pub fn new_validated(w: usize, h: usize) -> Result<Self> {
        if w == 0 || h == 0 {
            return Err(RTuiError::invalid_parameter(
                "Surface dimensions must be greater than 0",
            ));
        }
        if w > 10000 || h > 10000 {
            return Err(RTuiError::invalid_parameter(
                "Surface dimensions too large (max 10000x10000)",
            ));
        }
        Ok(Self::new(w, h))
    }
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }
    pub fn dims(&self) -> (usize, usize) {
        (self.w, self.h)
    }
    /// Reinitialize the surface to the given dimensions, reusing allocation when possible.
    pub fn reinit(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        let needed = w * h;
        if self.buf.capacity() >= needed {
            self.buf.resize(needed, Cell::default());
        } else {
            self.buf = vec![Cell::default(); needed];
        }
    }
    pub fn clear(&mut self, bg: Rgba) {
        for c in &mut self.buf {
            *c = Cell {
                ch: ' ',
                fg: Rgba {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
                bg,
                attr: Attr::empty(),
            };
        }
    }
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.w && y < self.h {
            let i = self.idx(x, y);
            self.buf[i] = cell;
        }
    }
    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.buf[self.idx(x, y)]
    }

    pub fn write_str(&mut self, mut x: usize, y: usize, s: &str, fg: Rgba, bg: Rgba, attr: Attr) {
        if y >= self.h {
            return;
        }
        for ch in s.chars() {
            if x >= self.w {
                break;
            }
            let cell = Cell { ch, fg, bg, attr };
            self.set(x, y, cell);
            x += 1;
            if unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1) == 2 && x < self.w {
                self.set(
                    x,
                    y,
                    Cell {
                        ch: ' ',
                        fg,
                        bg,
                        attr,
                    },
                );
                x += 1;
            }
        }
    }

    /// Write a string at the given point
    pub fn write_str_at(&mut self, point: Point, s: &str, fg: Rgba, bg: Rgba, attr: Attr) {
        self.write_str(point.x, point.y, s, fg, bg, attr)
    }

    /// Set a cell at the given point
    pub fn set_at(&mut self, point: Point, cell: Cell) {
        if point.x < self.w && point.y < self.h {
            self.set(point.x, point.y, cell);
        }
    }

    /// Get a cell at the given point
    pub fn get_at(&self, point: Point) -> Cell {
        self.get(point.x, point.y)
    }

    /// Fill a rectangle with a character
    pub fn fill_rect(&mut self, rect: Rect, ch: char, fg: Rgba, bg: Rgba, attr: Attr) {
        let bounds = Rect::from_coords(0, 0, self.w, self.h);
        let rect = rect.clamp(&bounds);

        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                self.set(x, y, Cell { ch, fg, bg, attr });
            }
        }
    }

    /// Clear a rectangular area
    pub fn clear_rect(&mut self, rect: Rect) {
        self.fill_rect(rect, ' ', Rgba::white(), Rgba::black(), Attr::empty());
    }

    /// Get the size of this surface
    pub fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }

    /// Get the bounds of this surface as a Rect
    pub fn bounds(&self) -> Rect {
        Rect::from_coords(0, 0, self.w, self.h)
    }

    /// Write a string with width clipping
    #[allow(clippy::too_many_arguments)]
    pub fn write_str_clipped(
        &mut self,
        x: usize,
        y: usize,
        s: &str,
        max_width: usize,
        fg: Rgba,
        bg: Rgba,
        attr: Attr,
    ) {
        if y >= self.h {
            return;
        }

        // If max_width is 0, use surface width as limit
        let limit = if max_width == 0 {
            self.w.saturating_sub(x)
        } else {
            max_width
        };
        let mut used = 0usize;

        for ch in s.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
            if used + char_width > limit || x + used >= self.w {
                break;
            }

            let cell = Cell { ch, fg, bg, attr };
            self.set(x + used, y, cell);
            used += char_width;

            // For wide characters, add continuation cell
            if char_width == 2 && x + used < self.w {
                self.set(
                    x + used,
                    y,
                    Cell {
                        ch: ' ',
                        fg,
                        bg,
                        attr,
                    },
                );
                used += 1;
            }
        }
    }

    /// Write text with a PaintStyle
    pub fn write_text_styled(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        style: &crate::ui::paint::PaintStyle,
    ) {
        self.write_str(x, y, text, style.fg, style.bg, style.attr);
    }

    /// Write clipped text with a PaintStyle
    pub fn write_text_styled_clipped(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        max_width: usize,
        style: &crate::ui::paint::PaintStyle,
    ) {
        self.write_str_clipped(x, y, text, max_width, style.fg, style.bg, style.attr);
    }

    pub fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize, bg: Option<Rgba>) {
        let (ww, hh) = self.dims();
        for yy in y..y.saturating_add(h).min(hh) {
            for xx in x..x.saturating_add(w).min(ww) {
                let mut c = self.get(xx, yy);
                if let Some(bg) = bg {
                    c.bg = bg;
                }
                self.set(xx, yy, c);
            }
        }
    }
}

impl Surface {
    pub fn clone_into_new(&self) -> Surface {
        let mut s = Surface::new(self.w, self.h);
        s.buf.copy_from_slice(&self.buf);
        s
    }
    pub fn copy_from(&mut self, other: &Surface) {
        assert_eq!(self.dims(), other.dims());
        self.buf.copy_from_slice(&other.buf);
    }
}

pub struct DiffWriter {
    out: Vec<u8>,
    cur_fg: Option<Rgba>,
    cur_bg: Option<Rgba>,
    cur_attr: Attr,
    last_rows_changed: usize,
    last_spans_written: usize,
}

impl Default for DiffWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffWriter {
    pub fn new() -> Self {
        Self {
            out: Vec::with_capacity(1 << 20),
            cur_fg: None,
            cur_bg: None,
            cur_attr: Attr::empty(),
            last_rows_changed: 0,
            last_spans_written: 0,
        }
    }
    #[inline]
    fn push(&mut self, s: &str) {
        self.out.extend_from_slice(s.as_bytes());
    }

    fn sgr_color(&mut self, r: u8, g: u8, b: u8, is_fg: bool) {
        if is_fg {
            self.push(&format!("\x1b[38;2;{r};{g};{b}m"));
        } else {
            self.push(&format!("\x1b[48;2;{r};{g};{b}m"));
        }
    }

    fn apply_attr_delta(&mut self, want: Attr) {
        let cur = self.cur_attr;
        // Turn off bits no longer needed
        if cur.contains(Attr::BOLD) && !want.contains(Attr::BOLD) {
            self.push("\x1b[22m");
        }
        if cur.contains(Attr::ITALIC) && !want.contains(Attr::ITALIC) {
            self.push("\x1b[23m");
        }
        if cur.contains(Attr::UNDERLINE) && !want.contains(Attr::UNDERLINE) {
            self.push("\x1b[24m");
        }
        if cur.contains(Attr::REVERSE) && !want.contains(Attr::REVERSE) {
            self.push("\x1b[27m");
        }
        if cur.contains(Attr::STRIKE) && !want.contains(Attr::STRIKE) {
            self.push("\x1b[29m");
        }
        // Turn on bits newly required
        if !cur.contains(Attr::BOLD) && want.contains(Attr::BOLD) {
            self.push("\x1b[1m");
        }
        if !cur.contains(Attr::ITALIC) && want.contains(Attr::ITALIC) {
            self.push("\x1b[3m");
        }
        if !cur.contains(Attr::UNDERLINE) && want.contains(Attr::UNDERLINE) {
            self.push("\x1b[4m");
        }
        if !cur.contains(Attr::REVERSE) && want.contains(Attr::REVERSE) {
            self.push("\x1b[7m");
        }
        if !cur.contains(Attr::STRIKE) && want.contains(Attr::STRIKE) {
            self.push("\x1b[9m");
        }
        self.cur_attr = want;
    }

    pub fn diff(&mut self, cur: &Surface, next: &Surface, force: bool) {
        self.out.clear();
        self.push("\x1b[?25l");
        let (w, h) = cur.dims();
        assert_eq!((w, h), next.dims());
        self.cur_fg = None;
        self.cur_bg = None;
        self.cur_attr = Attr::empty();
        self.last_rows_changed = 0;
        self.last_spans_written = 0;
        for y in 0..h {
            let mut run_buf = String::with_capacity(w);
            let mut run_start_col: Option<usize> = None;
            let mut wrote_row = false;
            for x in 0..w {
                let a = cur.get(x, y);
                let b = next.get(x, y);
                if !force && a == b {
                    if let Some(start) = run_start_col {
                        self.push(&format!("\x1b[{y1};{x1}H", y1 = y + 1, x1 = start + 1));
                        self.push(&run_buf);
                        self.last_spans_written += 1;
                        wrote_row = true;
                        run_buf.clear();
                        run_start_col = None;
                    }
                    continue;
                }
                if run_start_col.is_none() {
                    run_start_col = Some(x);
                }
                if run_buf.is_empty() {
                    if self.cur_fg != Some(b.fg) {
                        let (r, g, bv) = (
                            (b.fg.r * 255.0) as u8,
                            (b.fg.g * 255.0) as u8,
                            (b.fg.b * 255.0) as u8,
                        );
                        self.sgr_color(r, g, bv, true);
                        self.cur_fg = Some(b.fg);
                    }
                    if self.cur_bg != Some(b.bg) {
                        let (r, g, bv) = (
                            (b.bg.r * 255.0) as u8,
                            (b.bg.g * 255.0) as u8,
                            (b.bg.b * 255.0) as u8,
                        );
                        self.sgr_color(r, g, bv, false);
                        self.cur_bg = Some(b.bg);
                    }
                    if self.cur_attr != b.attr {
                        self.apply_attr_delta(b.attr);
                    }
                }
                run_buf.push(b.ch);
                if unicode_width::UnicodeWidthChar::width(b.ch).unwrap_or(1) == 2 {
                    run_buf.push(' ');
                }
            }
            if let Some(start) = run_start_col {
                self.push(&format!("\x1b[{y1};{x1}H", y1 = y + 1, x1 = start + 1));
                self.push(&run_buf);
                self.last_spans_written += 1;
                wrote_row = true;
            }
            if wrote_row {
                self.last_rows_changed += 1;
            }
        }
        self.push("\x1b[?25h");
    }

    pub fn output(&self) -> &[u8] {
        &self.out
    }
}

impl DiffWriter {
    pub fn last_rows_changed(&self) -> usize {
        self.last_rows_changed
    }
    pub fn last_spans_written(&self) -> usize {
        self.last_spans_written
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diffwriter_counts_increase_on_changes() {
        let a = Surface::new(10, 3);
        let mut b = Surface::new(10, 3);
        let fg = Rgba::white();
        let bg = Rgba::black();
        b.write_str(0, 0, "hello", fg, bg, Attr::empty());
        b.write_str(0, 1, "世界", fg, bg, Attr::empty());

        let mut diff = DiffWriter::new();
        diff.diff(&a, &b, false);
        assert!(diff.last_rows_changed() > 0);
        assert!(diff.last_spans_written() > 0);
        assert!(!diff.output().is_empty());
    }

    #[test]
    fn surface_reinit_reuses_capacity() {
        let mut s = Surface::new(40, 10);
        let cap_initial = s.buf.capacity();
        s.reinit(40, 10);
        let cap_same = s.buf.capacity();
        assert_eq!(
            cap_same, cap_initial,
            "capacity should not change on same-size reinit"
        );
        // shrink
        s.reinit(10, 5);
        let cap_shrink = s.buf.capacity();
        assert_eq!(
            cap_shrink, cap_initial,
            "capacity should be reused on shrink when sufficient"
        );
        // grow beyond
        s.reinit(100, 50);
        let cap_grow = s.buf.capacity();
        assert!(cap_grow >= 100 * 50);
    }
}
