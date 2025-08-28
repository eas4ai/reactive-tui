#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

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
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }
    pub fn dims(&self) -> (usize, usize) {
        (self.w, self.h)
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
}

pub struct DiffWriter {
    out: Vec<u8>,
    cur_fg: Option<Rgba>,
    cur_bg: Option<Rgba>,
    cur_attr: Attr,
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
        for y in 0..h {
            let mut run_buf = String::with_capacity(w);
            let mut run_start_col: Option<usize> = None;
            for x in 0..w {
                let a = cur.get(x, y);
                let b = next.get(x, y);
                if !force && a == b {
                    if let Some(start) = run_start_col {
                        self.push(&format!("\x1b[{y1};{x1}H", y1 = y + 1, x1 = start + 1));
                        self.push(&run_buf);
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
            }
        }
        self.push("\x1b[?25h");
    }

    pub fn output(&self) -> &[u8] {
        &self.out
    }
}
