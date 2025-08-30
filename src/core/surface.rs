use super::geometry::{Point, Rect, Size};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    /// Default epsilon for color comparisons (1/255 for 8-bit precision)
    pub const DEFAULT_EPSILON: f32 = 1.0 / 255.0;

    /// Create a new RGBA color
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

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

    /// Compare colors with epsilon tolerance for floating-point precision
    pub fn equals_epsilon(self, other: Self, epsilon: f32) -> bool {
        (self.r - other.r).abs() < epsilon
            && (self.g - other.g).abs() < epsilon
            && (self.b - other.b).abs() < epsilon
            && (self.a - other.a).abs() < epsilon
    }

    /// Compare colors with default epsilon
    pub fn approx_eq(self, other: Self) -> bool {
        self.equals_epsilon(other, Self::DEFAULT_EPSILON)
    }

    /// Blend two colors using alpha blending
    pub fn blend(self, other: Self, alpha: f32) -> Self {
        let alpha = alpha.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * alpha,
            g: self.g + (other.g - self.g) * alpha,
            b: self.b + (other.b - self.b) * alpha,
            a: self.a + (other.a - self.a) * alpha,
        }
    }

    /// Multiply color by alpha (for transparency effects)
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a * alpha.clamp(0.0, 1.0),
        }
    }

    /// Premultiply alpha (for efficient blending)
    pub fn premultiply_alpha(self) -> Self {
        Self {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }

    /// Convert to linear color space (gamma correction)
    pub fn to_linear(self) -> Self {
        fn srgb_to_linear(c: f32) -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        Self {
            r: srgb_to_linear(self.r),
            g: srgb_to_linear(self.g),
            b: srgb_to_linear(self.b),
            a: self.a,
        }
    }

    /// Convert from linear to sRGB color space
    pub fn to_srgb(self) -> Self {
        fn linear_to_srgb(c: f32) -> f32 {
            if c <= 0.0031308 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        }

        Self {
            r: linear_to_srgb(self.r),
            g: linear_to_srgb(self.g),
            b: linear_to_srgb(self.b),
            a: self.a,
        }
    }

    /// Get luminance (perceived brightness) using Rec. 709 coefficients
    pub fn luminance(self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Calculate contrast ratio between two colors (WCAG standard)
    pub fn contrast_ratio(self, other: Self) -> f32 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }
}

// SIMD-optimized color operations (when portable_simd is available)
#[cfg(feature = "simd")]
impl Rgba {
    /// SIMD-optimized epsilon comparison
    pub fn equals_epsilon_simd(self, other: Self, epsilon: f32) -> bool {
        use std::simd::{SimdFloat, SimdPartialOrd, f32x4};

        let a = f32x4::from_array([self.r, self.g, self.b, self.a]);
        let b = f32x4::from_array([other.r, other.g, other.b, other.a]);
        let diff = (a - b).abs();
        let eps = f32x4::splat(epsilon);

        diff.simd_lt(eps).all()
    }

    /// SIMD-optimized color blending
    pub fn blend_simd(self, other: Self, alpha: f32) -> Self {
        use std::simd::f32x4;

        let alpha = alpha.clamp(0.0, 1.0);
        let a = f32x4::from_array([self.r, self.g, self.b, self.a]);
        let b = f32x4::from_array([other.r, other.g, other.b, other.a]);
        let alpha_vec = f32x4::splat(alpha);

        // result = a + (b - a) * alpha
        let result = a + (b - a) * alpha_vec;
        let arr = result.to_array();

        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    /// SIMD-optimized alpha multiplication
    pub fn with_alpha_simd(self, alpha: f32) -> Self {
        use std::simd::f32x4;

        let alpha = alpha.clamp(0.0, 1.0);
        let color = f32x4::from_array([self.r, self.g, self.b, self.a]);
        let alpha_vec = f32x4::from_array([1.0, 1.0, 1.0, alpha]);
        let result = color * alpha_vec;
        let arr = result.to_array();

        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    /// SIMD-optimized premultiply alpha
    pub fn premultiply_alpha_simd(self) -> Self {
        use std::simd::f32x4;

        let color = f32x4::from_array([self.r, self.g, self.b, self.a]);
        let alpha_vec = f32x4::splat(self.a);
        let alpha_mask = f32x4::from_array([1.0, 1.0, 1.0, 0.0]); // Don't multiply alpha by itself
        let result = color * (alpha_vec * alpha_mask + f32x4::from_array([0.0, 0.0, 0.0, 1.0]));
        let arr = result.to_array();

        Self::new(arr[0], arr[1], arr[2], self.a)
    }
}

// Fallback implementations that use SIMD when available
impl Rgba {
    /// Optimized epsilon comparison (uses SIMD when available)
    #[inline]
    pub fn equals_epsilon_fast(self, other: Self, epsilon: f32) -> bool {
        #[cfg(feature = "simd")]
        {
            self.equals_epsilon_simd(other, epsilon)
        }
        #[cfg(not(feature = "simd"))]
        {
            self.equals_epsilon(other, epsilon)
        }
    }

    /// Optimized color blending (uses SIMD when available)
    #[inline]
    pub fn blend_fast(self, other: Self, alpha: f32) -> Self {
        #[cfg(feature = "simd")]
        {
            self.blend_simd(other, alpha)
        }
        #[cfg(not(feature = "simd"))]
        {
            self.blend(other, alpha)
        }
    }

    /// Optimized alpha multiplication (uses SIMD when available)
    #[inline]
    pub fn with_alpha_fast(self, alpha: f32) -> Self {
        #[cfg(feature = "simd")]
        {
            self.with_alpha_simd(alpha)
        }
        #[cfg(not(feature = "simd"))]
        {
            self.with_alpha(alpha)
        }
    }
}

use crate::error::{ReactiveError, Result};
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
            return Err(ReactiveError::invalid_parameter(
                "Surface dimensions must be greater than 0",
            ));
        }
        if w > 10000 || h > 10000 {
            return Err(ReactiveError::invalid_parameter(
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

/// Enhanced diff statistics for performance monitoring
#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    pub rows_changed: usize,
    pub spans_written: usize,
    pub cells_changed: usize,
    pub cells_total: usize,
    pub bytes_written: usize,
    pub color_changes: usize,
    pub attr_changes: usize,
    pub cursor_moves: usize,
    pub efficiency_ratio: f32,
}

pub struct DiffWriter {
    out: Vec<u8>,
    cur_fg: Option<Rgba>,
    cur_bg: Option<Rgba>,
    cur_attr: Attr,
    last_rows_changed: usize,
    last_spans_written: usize,
    // Enhanced statistics
    stats: DiffStats,
    // Optimization settings
    use_epsilon_comparison: bool,
    skip_identical_colors: bool,
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
            stats: DiffStats::default(),
            use_epsilon_comparison: true,
            skip_identical_colors: true,
        }
    }

    /// Create a new DiffWriter with optimization settings
    pub fn with_options(use_epsilon_comparison: bool, skip_identical_colors: bool) -> Self {
        Self {
            out: Vec::with_capacity(1 << 20),
            cur_fg: None,
            cur_bg: None,
            cur_attr: Attr::empty(),
            last_rows_changed: 0,
            last_spans_written: 0,
            stats: DiffStats::default(),
            use_epsilon_comparison,
            skip_identical_colors,
        }
    }

    /// Get the latest diff statistics
    pub fn stats(&self) -> &DiffStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = DiffStats::default();
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

        // Reset statistics
        self.stats = DiffStats::default();
        self.stats.cells_total = w * h;
        for y in 0..h {
            let mut run_buf = String::with_capacity(w);
            let mut run_start_col: Option<usize> = None;
            let mut wrote_row = false;
            for x in 0..w {
                let a = cur.get(x, y);
                let b = next.get(x, y);

                // Enhanced cell comparison with epsilon-based color comparison
                let cells_equal = if !force {
                    if self.use_epsilon_comparison {
                        a.ch == b.ch
                            && a.fg.approx_eq(b.fg)
                            && a.bg.approx_eq(b.bg)
                            && a.attr == b.attr
                    } else {
                        a == b
                    }
                } else {
                    false
                };

                if cells_equal {
                    if let Some(start) = run_start_col {
                        self.push(&format!("\x1b[{y1};{x1}H", y1 = y + 1, x1 = start + 1));
                        self.push(&run_buf);
                        self.last_spans_written += 1;
                        self.stats.cursor_moves += 1;
                        wrote_row = true;
                        run_buf.clear();
                        run_start_col = None;
                    }
                    continue;
                }
                if run_start_col.is_none() {
                    run_start_col = Some(x);
                }
                // Track that this cell changed
                self.stats.cells_changed += 1;

                // Check for color/attribute changes regardless of run buffer state
                let fg_changed = if self.use_epsilon_comparison {
                    self.cur_fg.is_none_or(|cur| !cur.approx_eq(b.fg))
                } else {
                    self.cur_fg != Some(b.fg)
                };

                let bg_changed = if self.use_epsilon_comparison {
                    self.cur_bg.is_none_or(|cur| !cur.approx_eq(b.bg))
                } else {
                    self.cur_bg != Some(b.bg)
                };

                let attr_changed = self.cur_attr != b.attr;

                // If any style changed, flush the current run and start a new one
                if fg_changed || bg_changed || attr_changed {
                    // Flush current run if it has content
                    if !run_buf.is_empty() {
                        if let Some(start) = run_start_col {
                            self.push(&format!("\x1b[{y1};{x1}H", y1 = y + 1, x1 = start + 1));
                            self.push(&run_buf);
                            self.last_spans_written += 1;
                            self.stats.cursor_moves += 1;
                        }
                        run_buf.clear();
                        run_start_col = None;
                    }

                    // Apply new colors/attributes
                    if fg_changed {
                        let (r, g, bv) = (
                            (b.fg.r * 255.0) as u8,
                            (b.fg.g * 255.0) as u8,
                            (b.fg.b * 255.0) as u8,
                        );
                        self.sgr_color(r, g, bv, true);
                        self.cur_fg = Some(b.fg);
                        self.stats.color_changes += 1;
                    }

                    if bg_changed {
                        let (r, g, bv) = (
                            (b.bg.r * 255.0) as u8,
                            (b.bg.g * 255.0) as u8,
                            (b.bg.b * 255.0) as u8,
                        );
                        self.sgr_color(r, g, bv, false);
                        self.cur_bg = Some(b.bg);
                        self.stats.color_changes += 1;
                    }

                    if attr_changed {
                        self.apply_attr_delta(b.attr);
                        self.stats.attr_changes += 1;
                    }
                }

                // Start new run if needed
                if run_start_col.is_none() {
                    run_start_col = Some(x);
                }

                // Add character to current run
                run_buf.push(b.ch);
                if unicode_width::UnicodeWidthChar::width(b.ch).unwrap_or(1) == 2 {
                    run_buf.push(' ');
                }
            }

            // Flush any remaining run at end of line
            if !run_buf.is_empty()
                && let Some(start) = run_start_col
            {
                self.push(&format!("\x1b[{y1};{x1}H", y1 = y + 1, x1 = start + 1));
                self.push(&run_buf);
                self.last_spans_written += 1;
                self.stats.cursor_moves += 1;
                wrote_row = true;
            }
            if wrote_row {
                self.last_rows_changed += 1;
            }
        }
        self.push("\x1b[?25h");

        // Finalize statistics
        self.stats.rows_changed = self.last_rows_changed;
        self.stats.spans_written = self.last_spans_written;
        self.stats.bytes_written = self.out.len();
        self.stats.efficiency_ratio = if self.stats.cells_total > 0 {
            self.stats.cells_changed as f32 / self.stats.cells_total as f32
        } else {
            0.0
        };
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

    /// Enable or disable epsilon-based color comparison
    pub fn set_epsilon_comparison(&mut self, enabled: bool) {
        self.use_epsilon_comparison = enabled;
    }

    /// Enable or disable skipping identical colors
    pub fn set_skip_identical_colors(&mut self, enabled: bool) {
        self.skip_identical_colors = enabled;
    }

    /// Get detailed diff statistics
    pub fn detailed_stats(&self) -> DiffStats {
        self.stats.clone()
    }

    /// Check if the last diff was efficient (low change ratio)
    pub fn is_efficient(&self) -> bool {
        self.stats.efficiency_ratio < 0.3 // Less than 30% of cells changed
    }

    /// Get a performance assessment of the last diff
    pub fn performance_assessment(&self) -> &'static str {
        match self.stats.efficiency_ratio {
            r if r < 0.1 => "Excellent - Very few changes",
            r if r < 0.3 => "Good - Moderate changes",
            r if r < 0.6 => "Fair - Many changes",
            _ => "Poor - Most cells changed",
        }
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
