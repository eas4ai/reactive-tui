//! Buffer drawing: blending, scissor rectangles, wide-cell writes, plain
//! text drawing, ANSI cell emission, and framebuffer compositing.
//! Reference `buffer.zig` BUF-008 … BUF-013.
//!
//! Image-pixel transfer stays with the media commitment: compositing
//! copies placement geometry (clipped exactly like the reference) while
//! decoded pixels remain source-owned.

use super::{
    BufferError, Cell, ClipRect, DEFAULT_SPACE_CHAR, ImagePlacement, OptimizedBuffer, make_cell,
};
use crate::ansi::{self, Rgba, TextAttributes};
use crate::uni::segments::{
    GRAPHEME_ID_MASK, IMAGE_ID_MASK, char_left_extent, char_right_extent, encoded_char_width,
    grapheme_id_from_char, image_fallback_from_char, image_id_from_char, is_cluster_char,
    is_continuation_char, is_grapheme_char, is_image_char, pack_continuation, pack_grapheme_start,
    pack_image_cell,
};
use std::collections::HashMap;
use std::rc::Rc;

/// Quadrant block fallback per image-fallback nibble. Reference
/// `quadrantChars`.
pub const QUADRANT_CHARS: [u32; 16] = [
    32,     // 0000
    0x2591, // 0001 BR
    0x2596, // 0010 BL
    0x2584, // 0011 Lower Half Block
    0x259D, // 0100 TR
    0x2590, // 0101 Right Half Block
    0x259E, // 0110 TR+BL
    0x259F, // 0111 TR+BL+BR
    0x2598, // 1000 TL
    0x259A, // 1001 TL+BR
    0x258C, // 1010 Left Half Block
    0x2599, // 1011 TL+BL+BR
    0x2580, // 1100 Upper Half Block
    0x259C, // 1101 TL+TR+BR
    0x259B, // 1110 TL+TR+BL
    0x2588, // 1111 Full Block
];

fn mul_div_255(a: u32, b: u32) -> u32 {
    (a * b + 127) / 255
}

fn round_div(n: u32, d: u32) -> u8 {
    ((n + d / 2) / d) as u8
}

fn opacity_to_u8(opacity: f32) -> u8 {
    ansi::component_to_u8(opacity)
}

fn apply_opacity(color: Rgba, opacity: u8) -> Rgba {
    ansi::pack_rgba8(
        ansi::red(color),
        ansi::green(color),
        ansi::blue(color),
        mul_div_255(u32::from(ansi::alpha(color)), u32::from(opacity)) as u8,
        ansi::get_meta(color),
    )
}

/// Reference `blendColors`: integer-exact alpha compositing with the
/// opaque fast path and backdrop substitution for transparent
/// destinations.
pub fn blend_colors(src: Rgba, dst0: Rgba, backdrop: Option<Rgba>) -> Rgba {
    let sa = u32::from(ansi::alpha(src));
    if sa == 0 {
        return dst0;
    }
    let dst = if ansi::alpha(dst0) == 0 {
        backdrop.unwrap_or(dst0)
    } else {
        dst0
    };
    if sa == 255 {
        return ansi::rgb_color(ansi::red(src), ansi::green(src), ansi::blue(src), 255);
    }
    let da = u32::from(ansi::alpha(dst));
    let inv = 255 - sa;
    let out_a = sa + mul_div_255(da, inv);
    if out_a == 0 {
        return ansi::rgb_color(0, 0, 0, 0);
    }
    if da == 255 {
        return ansi::rgb_color(
            ((u32::from(ansi::red(src)) * sa + u32::from(ansi::red(dst)) * inv + 127) / 255) as u8,
            ((u32::from(ansi::green(src)) * sa + u32::from(ansi::green(dst)) * inv + 127) / 255)
                as u8,
            ((u32::from(ansi::blue(src)) * sa + u32::from(ansi::blue(dst)) * inv + 127) / 255)
                as u8,
            255,
        );
    }
    ansi::rgb_color(
        round_div(
            u32::from(ansi::red(src)) * sa + mul_div_255(u32::from(ansi::red(dst)) * da, inv),
            out_a,
        ),
        round_div(
            u32::from(ansi::green(src)) * sa + mul_div_255(u32::from(ansi::green(dst)) * da, inv),
            out_a,
        ),
        round_div(
            u32::from(ansi::blue(src)) * sa + mul_div_255(u32::from(ansi::blue(dst)) * da, inv),
            out_a,
        ),
        out_a as u8,
    )
}

fn is_rgba_with_alpha(color: Rgba) -> bool {
    ansi::alpha(color) < 255
}

fn is_fully_opaque(opacity: f32, fg: Rgba, bg: Rgba) -> bool {
    opacity == 1.0 && !is_rgba_with_alpha(fg) && !is_rgba_with_alpha(bg)
}

fn is_fully_transparent(opacity: f32, fg: Rgba, bg: Rgba) -> bool {
    opacity == 0.0 || (ansi::alpha(fg) == 0 && ansi::alpha(bg) == 0)
}

fn opaque_cell(cell: Cell) -> Cell {
    make_cell(
        cell.char,
        ansi::pack_rgba8(
            ansi::red(cell.fg),
            ansi::green(cell.fg),
            ansi::blue(cell.fg),
            255,
            ansi::get_meta(cell.fg),
        ),
        ansi::pack_rgba8(
            ansi::red(cell.bg),
            ansi::green(cell.bg),
            ansi::blue(cell.bg),
            255,
            ansi::get_meta(cell.bg),
        ),
        cell.attributes,
    )
}

impl<'a> OptimizedBuffer<'a> {
    // ---- opacity stack ----

    /// Product of stacked opacities, 1.0 when empty. Reference
    /// `getCurrentOpacity`.
    pub fn current_opacity(&self) -> f32 {
        self.opacity_stack.last().copied().unwrap_or(1.0)
    }

    /// Push opacity multiplied with the current one. Reference
    /// `pushOpacity`.
    pub fn push_opacity(&mut self, opacity: f32) {
        let effective = self.current_opacity() * opacity.clamp(0.0, 1.0);
        self.opacity_stack.push(effective);
    }

    pub fn pop_opacity(&mut self) {
        self.opacity_stack.pop();
    }

    pub fn clear_opacity(&mut self) {
        self.opacity_stack.clear();
    }

    // ---- scissor rectangles ----

    /// Whether a rectangle intersects the active scissor. Reference
    /// `isRectInScissor`.
    pub fn rect_in_scissor(&self, x: i32, y: i32, width: u32, height: u32) -> bool {
        let scissor = match self.current_scissor() {
            None => return true,
            Some(r) => r,
        };
        let rect_end_x = x + width as i32;
        let rect_end_y = y + height as i32;
        let scissor_end_x = scissor.x + scissor.width as i32;
        let scissor_end_y = scissor.y + scissor.height as i32;
        !(x >= scissor_end_x
            || rect_end_x <= scissor.x
            || y >= scissor_end_y
            || rect_end_y <= scissor.y)
    }

    /// Intersect a rectangle with the active scissor. Reference
    /// `clipRectToScissor`.
    pub fn clip_rect_to_scissor(
        &self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Option<ClipRect> {
        let scissor = match self.current_scissor() {
            None => {
                return Some(ClipRect {
                    x,
                    y,
                    width,
                    height,
                });
            }
            Some(r) => r,
        };
        let rect_end_x = x + width as i32;
        let rect_end_y = y + height as i32;
        let scissor_end_x = scissor.x + scissor.width as i32;
        let scissor_end_y = scissor.y + scissor.height as i32;
        let ix = x.max(scissor.x);
        let iy = y.max(scissor.y);
        let iex = rect_end_x.min(scissor_end_x);
        let iey = rect_end_y.min(scissor_end_y);
        if ix >= iex || iy >= iey {
            return None;
        }
        Some(ClipRect {
            x: ix,
            y: iy,
            width: (iex - ix) as u32,
            height: (iey - iy) as u32,
        })
    }

    /// Push intersected with the current rect so nested scissors always
    /// clip to parents; a fully outside rect pushes degenerate.
    /// Reference `pushScissorRect`.
    pub fn push_scissor_rect(&mut self, x: i32, y: i32, width: u32, height: u32) {
        let mut rect = ClipRect {
            x,
            y,
            width,
            height,
        };
        if self.current_scissor().is_some() {
            match self.clip_rect_to_scissor(rect.x, rect.y, rect.width, rect.height) {
                Some(clipped) => rect = clipped,
                None => {
                    rect = ClipRect {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                    };
                }
            }
        }
        self.push_scissor(rect);
    }

    // ---- blending ----

    /// Blend one overlay cell over one destination cell. Reference
    /// `blendCells`. Links always come from the overlay, even when zero.
    pub fn blend_cells(&self, overlay: Cell, dest: Cell) -> Cell {
        let has_bg_alpha = is_rgba_with_alpha(overlay.bg);
        let has_fg_alpha = is_rgba_with_alpha(overlay.fg);
        if !(has_bg_alpha || has_fg_alpha) {
            return overlay;
        }
        let blended_bg = if has_bg_alpha {
            blend_colors(overlay.bg, dest.bg, self.blend_backdrop)
        } else {
            overlay.bg
        };
        let preserve_char = overlay.char == DEFAULT_SPACE_CHAR
            && dest.char != 0
            && dest.char != DEFAULT_SPACE_CHAR
            && encoded_char_width(dest.char) == 1;
        let final_char = if preserve_char {
            dest.char
        } else {
            overlay.char
        };
        let final_fg = if preserve_char {
            blend_colors(overlay.bg, dest.fg, self.blend_backdrop)
        } else if has_fg_alpha {
            blend_colors(overlay.fg, blended_bg, self.blend_backdrop)
        } else {
            overlay.fg
        };
        let base_attrs = if preserve_char {
            TextAttributes::base_attributes(dest.attributes)
        } else {
            TextAttributes::base_attributes(overlay.attributes)
        };
        let overlay_link = TextAttributes::link_id(overlay.attributes);
        make_cell(
            final_char,
            final_fg,
            blended_bg,
            TextAttributes::set_link_id(u32::from(base_attrs), overlay_link),
        )
    }

    /// Reference `setCellWithAlphaBlending` (char form).
    pub fn set_cell_with_alpha_blending(
        &mut self,
        x: u32,
        y: u32,
        char: u32,
        fg: Rgba,
        bg: Rgba,
        attributes: u32,
    ) {
        self.set_cell_with_alpha_blending_cell(x, y, make_cell(char, fg, bg, attributes));
    }

    fn blend_cell_with_opacity(
        &mut self,
        x: u32,
        y: u32,
        cell: Cell,
        opacity: f32,
        dest_cell: Option<Cell>,
    ) {
        let opacity_u8 = opacity_to_u8(opacity);
        let effective = make_cell(
            cell.char,
            apply_opacity(cell.fg, opacity_u8),
            apply_opacity(cell.bg, opacity_u8),
            cell.attributes,
        );
        match dest_cell {
            Some(dest) => {
                let blended = self.blend_cells(effective, dest);
                if !self.grapheme_tracker.has_any()
                    && !self.link_tracker.has_any()
                    && !is_cluster_char(blended.char)
                {
                    self.set_raw(x, y, blended);
                } else {
                    self.set(x, y, blended);
                }
            }
            None => self.set(x, y, effective),
        }
    }

    fn set_cell_with_alpha_blending_cell_without_images(&mut self, x: u32, y: u32, cell: Cell) {
        if !self.point_in_scissor(x as i32, y as i32) {
            return;
        }
        let opacity = self.current_opacity();
        if is_fully_transparent(opacity, cell.fg, cell.bg) {
            return;
        }
        if is_fully_opaque(opacity, cell.fg, cell.bg) {
            self.set(x, y, cell);
            return;
        }
        let dest = self.get(x, y);
        self.blend_cell_with_opacity(x, y, cell, opacity, dest);
    }

    fn skip_transparent_cell_draw(&self, opacity: f32, fully_transparent: bool) -> bool {
        if fully_transparent {
            if self.placements.is_empty() {
                return true;
            }
            if opacity == 0.0 {
                return true;
            }
        }
        false
    }

    fn set_visible_cell_with_alpha_blending(
        &mut self,
        x: u32,
        y: u32,
        cell: Cell,
        opacity: f32,
        fully_transparent: bool,
    ) {
        if !self.point_in_scissor(x as i32, y as i32) {
            return;
        }
        if is_fully_opaque(opacity, cell.fg, cell.bg) {
            self.set(x, y, cell);
            return;
        }
        let dest = self.get(x, y);
        let first_overlaps_image = dest.map(|d| is_image_char(d.char)).unwrap_or(false);
        if first_overlaps_image || self.cell_span_tail_overlaps_image(x, y, cell.char) {
            self.set(x, y, opaque_cell(cell));
            return;
        }
        if fully_transparent {
            return;
        }
        self.blend_cell_with_opacity(x, y, cell, opacity, dest);
    }

    fn set_cell_with_alpha_blending_cell(&mut self, x: u32, y: u32, cell: Cell) {
        let opacity = self.current_opacity();
        let fully_transparent = is_fully_transparent(opacity, cell.fg, cell.bg);
        if self.skip_transparent_cell_draw(opacity, fully_transparent) {
            return;
        }
        self.set_visible_cell_with_alpha_blending(x, y, cell, opacity, fully_transparent);
    }

    /// Reference `setCellWithAlphaBlendingRaw` (char form).
    pub fn set_cell_with_alpha_blending_raw(
        &mut self,
        x: u32,
        y: u32,
        char: u32,
        fg: Rgba,
        bg: Rgba,
        attributes: u32,
    ) {
        self.set_cell_with_alpha_blending_raw_cell(x, y, make_cell(char, fg, bg, attributes));
    }

    fn set_cell_with_alpha_blending_raw_cell(&mut self, x: u32, y: u32, cell: Cell) {
        if !self.point_in_scissor(x as i32, y as i32) {
            return;
        }
        let opacity = self.current_opacity();
        if opacity == 0.0 {
            return;
        }
        if is_fully_opaque(opacity, cell.fg, cell.bg) {
            debug_assert!(!is_grapheme_char(cell.char));
            debug_assert!(!is_continuation_char(cell.char));
            self.set_raw(x, y, cell);
            return;
        }
        if is_fully_transparent(opacity, cell.fg, cell.bg) {
            return;
        }
        let opacity_u8 = opacity_to_u8(opacity);
        let effective = make_cell(
            cell.char,
            apply_opacity(cell.fg, opacity_u8),
            apply_opacity(cell.bg, opacity_u8),
            cell.attributes,
        );
        match self.get(x, y) {
            Some(dest) => {
                let blended = self.blend_cells(effective, dest);
                debug_assert!(!is_grapheme_char(blended.char));
                debug_assert!(!is_continuation_char(blended.char));
                self.set_raw(x, y, blended);
            }
            None => {
                debug_assert!(!is_grapheme_char(effective.char));
                debug_assert!(!is_continuation_char(effective.char));
                self.set_raw(x, y, effective);
            }
        }
    }

    fn set_cell_with_alpha_blending_raw_image_aware(&mut self, x: u32, y: u32, cell: Cell) {
        if let Some(dest) = self.get(x, y)
            && is_image_char(dest.char)
            && self.current_opacity() > 0.0
        {
            self.set_raw(x, y, opaque_cell(cell));
            return;
        }
        self.set_cell_with_alpha_blending_raw_cell(x, y, cell);
    }

    fn cell_span_overlaps_image(&self, x: u32, y: u32, char: u32) -> bool {
        if self.placements.is_empty() || y >= self.height {
            return false;
        }
        let width = if is_grapheme_char(char) {
            char_right_extent(char) + 1
        } else {
            1
        };
        let mut offset = 0;
        while offset < width && x + offset < self.width {
            if is_image_char(self.chars[self.coords_to_index(x + offset, y)]) {
                return true;
            }
            offset += 1;
        }
        false
    }

    fn cell_span_tail_overlaps_image(&self, x: u32, y: u32, char: u32) -> bool {
        if !is_grapheme_char(char) || y >= self.height {
            return false;
        }
        let width = char_right_extent(char) + 1;
        let mut offset = 1;
        while offset < width && x + offset < self.width {
            if is_image_char(self.chars[self.coords_to_index(x + offset, y)]) {
                return true;
            }
            offset += 1;
        }
        false
    }

    fn set_text_cell(&mut self, x: u32, y: u32, cell: Cell) {
        if !self.placements.is_empty() && self.cell_span_overlaps_image(x, y, cell.char) {
            self.set(x, y, opaque_cell(cell));
            return;
        }
        if is_rgba_with_alpha(cell.bg) {
            self.set_cell_with_alpha_blending_cell_without_images(x, y, cell);
            return;
        }
        self.set(x, y, cell);
    }

    /// Single opaque-aware character draw. Reference `drawChar`.
    pub fn draw_char(&mut self, char: u32, x: u32, y: u32, fg: Rgba, bg: Rgba, attributes: u32) {
        let cell = make_cell(char, fg, bg, attributes);
        let opacity = self.current_opacity();
        let fully_transparent = is_fully_transparent(opacity, fg, bg);
        if self.skip_transparent_cell_draw(opacity, fully_transparent) {
            return;
        }
        self.set_visible_cell_with_alpha_blending(x, y, cell, opacity, fully_transparent);
    }

    // ---- fills ----

    /// Fill a rectangle with a background color. Reference `fillRect`.
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, bg: Rgba) {
        if self.width == 0 || self.height == 0 || width == 0 || height == 0 {
            return;
        }
        if x >= self.width || y >= self.height {
            return;
        }
        if !self.rect_in_scissor(x as i32, y as i32, width, height) {
            return;
        }
        let opacity = self.current_opacity();
        let fully_transparent = is_fully_transparent(opacity, ansi::rgb_color(0, 0, 0, 0), bg);
        if fully_transparent && (opacity == 0.0 || self.placements.is_empty()) {
            return;
        }
        let max_end_x = if x < self.width { self.width - 1 } else { 0 };
        let max_end_y = if y < self.height { self.height - 1 } else { 0 };
        let end_x = max_end_x.min(x + width - 1);
        let end_y = max_end_y.min(y + height - 1);
        if x > end_x || y > end_y {
            return;
        }
        let clipped =
            match self.clip_rect_to_scissor(x as i32, y as i32, end_x - x + 1, end_y - y + 1) {
                Some(r) => r,
                None => return,
            };
        let csx = x.max(clipped.x as u32);
        let csy = y.max(clipped.y as u32);
        let cex = end_x.min((clipped.x + clipped.width as i32 - 1) as u32);
        let cey = end_y.min((clipped.y + clipped.height as i32 - 1) as u32);

        if fully_transparent {
            let cell = make_cell(
                DEFAULT_SPACE_CHAR,
                ansi::rgb_color(255, 255, 255, 255),
                bg,
                0,
            );
            let clipped_area = (cex - csx + 1) as u64 * (cey - csy + 1) as u64;
            let mut intersection_area = 0u64;
            for placement in &self.placements {
                let isx = (csx as i32).max(placement.x);
                let isy = (csy as i32).max(placement.y);
                let iex = (cex as i32 + 1).min(placement.x + placement.width as i32);
                let iey = (cey as i32 + 1).min(placement.y + placement.height as i32);
                if isx >= iex || isy >= iey {
                    continue;
                }
                let w = (iex - isx) as u64;
                let h = (iey - isy) as u64;
                intersection_area = clipped_area.min(intersection_area + w * h);
            }
            if intersection_area == 0 {
                return;
            }
            if intersection_area >= clipped_area / 2 + clipped_area % 2 {
                for fy in csy..=cey {
                    for fx in csx..=cex {
                        let idx = self.coords_to_index(fx, fy);
                        if is_image_char(self.chars[idx]) {
                            self.set_raw(fx, fy, opaque_cell(cell));
                        }
                    }
                }
                return;
            }
            for placement in self.placements.clone() {
                let isx = (csx as i32).max(placement.x);
                let isy = (csy as i32).max(placement.y);
                let iex = (cex as i32 + 1).min(placement.x + placement.width as i32);
                let iey = (cey as i32 + 1).min(placement.y + placement.height as i32);
                if isx >= iex || isy >= iey {
                    continue;
                }
                for fy in (isy as u32)..(iey as u32) {
                    for fx in (isx as u32)..(iex as u32) {
                        let idx = self.coords_to_index(fx, fy);
                        if is_image_char(self.chars[idx]) {
                            self.set_raw(fx, fy, opaque_cell(cell));
                        }
                    }
                }
            }
            return;
        }

        let has_alpha = is_rgba_with_alpha(bg) || opacity < 1.0;
        let grapheme_aware = self.grapheme_tracker.has_any();
        let link_aware = self.link_tracker.has_any();
        if grapheme_aware || link_aware {
            for fy in csy..=cey {
                for fx in csx..=cex {
                    self.set_cell_with_alpha_blending_cell(
                        fx,
                        fy,
                        make_cell(
                            DEFAULT_SPACE_CHAR,
                            ansi::rgb_color(255, 255, 255, 255),
                            bg,
                            0,
                        ),
                    );
                }
            }
        } else if has_alpha {
            let image_aware = !self.placements.is_empty();
            for fy in csy..=cey {
                for fx in csx..=cex {
                    let cell = make_cell(
                        DEFAULT_SPACE_CHAR,
                        ansi::rgb_color(255, 255, 255, 255),
                        bg,
                        0,
                    );
                    if image_aware {
                        self.set_cell_with_alpha_blending_raw_image_aware(fx, fy, cell);
                    } else {
                        self.set_cell_with_alpha_blending_raw_cell(fx, fy, cell);
                    }
                }
            }
        } else {
            for fy in csy..=cey {
                let row_start = self.coords_to_index(csx, fy);
                let row_width = (cex - csx + 1) as usize;
                self.chars[row_start..row_start + row_width].fill(DEFAULT_SPACE_CHAR);
                self.fgs[row_start..row_start + row_width]
                    .fill(ansi::rgb_color(255, 255, 255, 255));
                self.bgs[row_start..row_start + row_width].fill(bg);
                self.attributes[row_start..row_start + row_width].fill(0);
            }
        }
    }

    /// Fill with signed origin, clipped to the grid. Reference
    /// `fillRectClipped`.
    pub fn fill_rect_clipped(&mut self, x: i32, y: i32, width: u32, height: u32, bg: Rgba) {
        if width == 0 || height == 0 {
            return;
        }
        let start_x = 0.max(x);
        let start_y = 0.max(y);
        let end_x = (self.width as i32 - 1).min(x + width as i32 - 1);
        let end_y = (self.height as i32 - 1).min(y + height as i32 - 1);
        if start_x > end_x || start_y > end_y {
            return;
        }
        self.fill_rect(
            start_x as u32,
            start_y as u32,
            (end_x - start_x + 1) as u32,
            (end_y - start_y + 1) as u32,
            bg,
        );
    }

    // ---- text drawing ----

    /// Draw plain text with the current opacity. Reference `drawText`.
    pub fn draw_text(
        &mut self,
        text: &str,
        x: u32,
        y: u32,
        fg: Rgba,
        bg: Option<Rgba>,
        attributes: u32,
    ) -> Result<(), BufferError> {
        let opacity = self.current_opacity();
        let bg_or_transparent = bg.unwrap_or_else(|| ansi::rgb_color(0, 0, 0, 0));
        if is_fully_transparent(opacity, fg, bg_or_transparent)
            && (opacity == 0.0 || self.placements.is_empty())
        {
            return Ok(());
        }
        self.draw_visible_text(text, x, y, fg, bg, attributes);
        Ok(())
    }

    /// Draw one already-segmented grapheme with an authoritative
    /// terminal-cell width. Reference `drawGrapheme`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_grapheme(
        &mut self,
        grapheme_bytes: &[u8],
        cell_width: u8,
        x: u32,
        y: u32,
        fg: Rgba,
        bg: Rgba,
        attributes: u32,
    ) -> Result<(), BufferError> {
        if grapheme_bytes.is_empty() || cell_width == 0 || x >= self.width || y >= self.height {
            return Ok(());
        }
        if x + u32::from(cell_width) > self.width {
            return Ok(());
        }
        for offset in 0..cell_width {
            if !self.point_in_scissor((x + u32::from(offset)) as i32, y as i32) {
                return Ok(());
            }
        }
        let encoded_char =
            if grapheme_bytes.len() == 1 && cell_width == 1 && grapheme_bytes[0] >= 32 {
                u32::from(grapheme_bytes[0])
            } else {
                let gid = self
                    .pool
                    .borrow_mut()
                    .alloc(grapheme_bytes)
                    .map_err(|_| BufferError::OutOfMemory)?;
                pack_grapheme_start(gid & GRAPHEME_ID_MASK, u32::from(cell_width))
            };
        self.set(x, y, make_cell(encoded_char, fg, bg, attributes));
        Ok(())
    }

    fn draw_visible_text(
        &mut self,
        text: &str,
        x: u32,
        y: u32,
        fg: Rgba,
        bg: Option<Rgba>,
        attributes: u32,
    ) {
        if x >= self.width || y >= self.height || text.is_empty() {
            return;
        }
        let explicit_colors_opaque = match bg {
            Some(background) => !is_rgba_with_alpha(fg) && !is_rgba_with_alpha(background),
            None => false,
        };
        let ascii_only = crate::uni::is_ascii_only(text.as_bytes());
        if explicit_colors_opaque && ascii_only {
            let printable = text.bytes().all(|b| (32..=126).contains(&b));
            if printable {
                let background = bg.unwrap_or_else(|| ansi::rgb_color(0, 0, 0, 0));
                for (offset, byte) in text.bytes().enumerate() {
                    let char_x = x + offset as u32;
                    if char_x >= self.width {
                        break;
                    }
                    self.set(
                        char_x,
                        y,
                        make_cell(u32::from(byte), fg, background, attributes),
                    );
                }
                return;
            }
        }

        let tab_width: u8 = 2;
        let text_bytes = text.as_bytes();
        let clusters = crate::uni::render_clusters(text, tab_width, ascii_only, self.width_method);
        let mut advance_cells = 0u32;
        let mut byte_offset = 0usize;
        let mut col = 0u32;
        let mut special_idx = 0usize;

        'text_loop: while byte_offset < text.len() {
            let char_x = x + advance_cells;
            if char_x >= self.width {
                break;
            }
            let at_special = special_idx < clusters.len() && clusters[special_idx].col_start == col;
            let (grapheme_bytes, cluster_width_cols): (&[u8], u32) = if at_special {
                let g = &clusters[special_idx];
                let end = (g.byte_start + g.byte_len) as usize;
                let bytes = &text_bytes[g.byte_start as usize..end];
                let width = g.width_cols;
                byte_offset = end;
                special_idx += 1;
                (bytes, width)
            } else {
                if byte_offset >= text.len() {
                    break;
                }
                let bytes = &text_bytes[byte_offset..byte_offset + 1];
                byte_offset += 1;
                (bytes, 1)
            };

            let is_tab = grapheme_bytes.len() == 1 && grapheme_bytes[0] == b'\t';
            if !is_tab && !self.point_in_scissor(char_x as i32, y as i32) {
                advance_cells += cluster_width_cols;
                col += cluster_width_cols;
                continue;
            }

            let bg_color = match bg {
                Some(b) => b,
                None => match self.get(char_x, y) {
                    Some(existing) => existing.bg,
                    None => ansi::rgb_color(0, 0, 0, 255),
                },
            };

            let width_at_byte = if at_special {
                clusters[special_idx - 1].byte_start as usize
            } else {
                byte_offset - 1
            };
            let cell_width =
                crate::uni::width_at(text, width_at_byte, tab_width, self.width_method);
            if cell_width == 0 {
                col += cluster_width_cols;
                continue;
            }
            if cell_width > 1 && !is_tab {
                if char_x + cell_width > self.width {
                    advance_cells += cluster_width_cols;
                    col += cluster_width_cols;
                    continue;
                }
                for span_offset in 1..cell_width {
                    if !self.point_in_scissor((char_x + span_offset) as i32, y as i32) {
                        advance_cells += cluster_width_cols;
                        col += cluster_width_cols;
                        continue 'text_loop;
                    }
                }
            }

            if is_tab {
                let mut tab_col = 0;
                while tab_col < cluster_width_cols {
                    let tab_x = char_x + tab_col;
                    if tab_x >= self.width {
                        break;
                    }
                    if self.point_in_scissor(tab_x as i32, y as i32) {
                        let cell = make_cell(DEFAULT_SPACE_CHAR, fg, bg_color, attributes);
                        if explicit_colors_opaque {
                            self.set(tab_x, y, cell);
                        } else {
                            self.set_text_cell(tab_x, y, cell);
                        }
                    }
                    tab_col += 1;
                }
                advance_cells += cluster_width_cols;
                col += cluster_width_cols;
                continue;
            }

            let encoded_char =
                if grapheme_bytes.len() == 1 && cell_width == 1 && grapheme_bytes[0] >= 32 {
                    u32::from(grapheme_bytes[0])
                } else {
                    match self.pool.borrow_mut().alloc(grapheme_bytes) {
                        Ok(gid) => pack_grapheme_start(gid & GRAPHEME_ID_MASK, cell_width),
                        Err(_) => continue,
                    }
                };
            let cell = make_cell(encoded_char, fg, bg_color, attributes);
            if explicit_colors_opaque {
                self.set(char_x, y, cell);
            } else {
                self.set_text_cell(char_x, y, cell);
            }
            advance_cells += cell_width;
            col += cluster_width_cols;
        }
    }

    // ---- compositing ----

    /// Composite a source grid onto this one with clipping, alpha, and
    /// scissor exactly as direct drawing. Reference `drawFrameBuffer`.
    /// Cluster bytes resolve through the SOURCE grid's pool (BUF-013).
    /// Placement geometry is clipped and copied; decoded image pixels
    /// stay source-owned until the media commitment.
    /// Translate a source packed char for a foreign pool: resolve the
    /// cluster bytes through the source grid's pool and intern them in
    /// this grid's pool, preserving extents. Reference behavior for
    /// same-pool composites (id copied raw); BUF-013 demands the
    /// translation when pools differ.
    fn translate_composite_char(
        &mut self,
        frame_buffer: &OptimizedBuffer<'_>,
        src_char: u32,
        id_map: &mut HashMap<u32, u32>,
    ) -> u32 {
        if is_grapheme_char(src_char) {
            let gid = grapheme_id_from_char(src_char);
            if let Some(&mapped) = id_map.get(&gid) {
                return pack_grapheme_start(mapped, char_right_extent(src_char) + 1);
            }
            let bytes = match frame_buffer.pool.borrow().get(gid) {
                Ok(bytes) => bytes.to_vec(),
                Err(_) => return DEFAULT_SPACE_CHAR,
            };
            let new_id = match self.pool.borrow_mut().alloc(&bytes) {
                Ok(id) => id & GRAPHEME_ID_MASK,
                Err(_) => return DEFAULT_SPACE_CHAR,
            };
            id_map.insert(gid, new_id);
            return pack_grapheme_start(new_id, char_right_extent(src_char) + 1);
        }
        if is_continuation_char(src_char) {
            let gid = grapheme_id_from_char(src_char);
            if let Some(&mapped) = id_map.get(&gid) {
                return pack_continuation(
                    char_left_extent(src_char),
                    char_right_extent(src_char),
                    mapped,
                );
            }
        }
        src_char
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_frame_buffer(
        &mut self,
        dest_x: i32,
        dest_y: i32,
        frame_buffer: &OptimizedBuffer<'_>,
        source_x: Option<u32>,
        source_y: Option<u32>,
        source_width: Option<u32>,
        source_height: Option<u32>,
    ) {
        if self.width == 0
            || self.height == 0
            || frame_buffer.width == 0
            || frame_buffer.height == 0
        {
            return;
        }
        let opacity = self.current_opacity();
        if opacity == 0.0 {
            return;
        }
        let src_x = source_x.unwrap_or(0);
        let src_y = source_y.unwrap_or(0);
        let src_w = source_width.unwrap_or(frame_buffer.width);
        let src_h = source_height.unwrap_or(frame_buffer.height);
        if src_x >= frame_buffer.width || src_y >= frame_buffer.height {
            return;
        }
        if src_w == 0 || src_h == 0 {
            return;
        }
        let clamped_src_w = src_w.min(frame_buffer.width - src_x);
        let clamped_src_h = src_h.min(frame_buffer.height - src_y);
        let start_dest_x = 0.max(dest_x);
        let start_dest_y = 0.max(dest_y);
        let end_dest_x = (self.width as i32 - 1).min(dest_x + clamped_src_w as i32 - 1);
        let end_dest_y = (self.height as i32 - 1).min(dest_y + clamped_src_h as i32 - 1);
        if start_dest_x > end_dest_x || start_dest_y > end_dest_y {
            return;
        }
        let dest_w = (end_dest_x - start_dest_x + 1) as u32;
        let dest_h = (end_dest_y - start_dest_y + 1) as u32;
        if !self.rect_in_scissor(start_dest_x, start_dest_y, dest_w, dest_h) {
            return;
        }
        let grapheme_aware =
            self.grapheme_tracker.has_any() || frame_buffer.grapheme_tracker.has_any();
        let link_aware = self.link_tracker.has_any() || frame_buffer.link_tracker.has_any();
        let image_aware = !self.placements.is_empty() || !frame_buffer.placements.is_empty();
        let clipped = match self.clip_rect_to_scissor(start_dest_x, start_dest_y, dest_w, dest_h) {
            Some(r) => r,
            None => return,
        };
        let csx = start_dest_x.max(clipped.x);
        let csy = start_dest_y.max(clipped.y);
        let cex = end_dest_x.min(clipped.x + clipped.width as i32 - 1);
        let cey = end_dest_y.min(clipped.y + clipped.height as i32 - 1);

        if !grapheme_aware && !frame_buffer.respect_alpha && !link_aware && !image_aware {
            for dy in csy..=cey {
                let rel_y = (dy - dest_y) as u32;
                let sy = src_y + rel_y;
                if sy >= frame_buffer.height {
                    continue;
                }
                let rel_x = (csx - dest_x) as u32;
                let sx = src_x + rel_x;
                if sx >= frame_buffer.width {
                    continue;
                }
                let dest_row = self.coords_to_index(csx as u32, dy as u32);
                let src_row = frame_buffer.coords_to_index(sx, sy);
                let copy_w = ((cex - csx + 1) as u32).min(frame_buffer.width - sx) as usize;
                self.chars[dest_row..dest_row + copy_w]
                    .copy_from_slice(&frame_buffer.chars[src_row..src_row + copy_w]);
                self.fgs[dest_row..dest_row + copy_w]
                    .copy_from_slice(&frame_buffer.fgs[src_row..src_row + copy_w]);
                self.bgs[dest_row..dest_row + copy_w]
                    .copy_from_slice(&frame_buffer.bgs[src_row..src_row + copy_w]);
                self.attributes[dest_row..dest_row + copy_w]
                    .copy_from_slice(&frame_buffer.attributes[src_row..src_row + copy_w]);
            }
            return;
        }

        // Placement pass: clip geometries into this grid, remapping image
        // ids through a per-call table (0 = unmapped, falls back to the
        // quadrant char). Decoded pixels stay source-owned.
        let mut image_id_map = vec![0u32; frame_buffer.placements.len() + 1];
        for (source_id, placement) in frame_buffer.placements.iter().enumerate() {
            let source_id = source_id as u32 + 1;
            if self.placements.len() as u32 >= IMAGE_ID_MASK {
                break;
            }
            let full_x = dest_x + placement.x - src_x as i32;
            let full_y = dest_y + placement.y - src_y as i32;
            let x0 = full_x.max(csx);
            let y0 = full_y.max(csy);
            let x1 = (full_x + placement.width as i32).min(cex + 1);
            let y1 = (full_y + placement.height as i32).min(cey + 1);
            if x0 >= x1 || y0 >= y1 {
                continue;
            }
            let left = (x0 - full_x) as u32;
            let top = (y0 - full_y) as u32;
            let right = (x1 - full_x) as u32;
            let bottom = (y1 - full_y) as u32;
            let source_start_x = placement.source_x
                + ((left as u64 * placement.source_width as u64) / placement.width as u64) as u32;
            let source_start_y = placement.source_y
                + ((top as u64 * placement.source_height as u64) / placement.height as u64) as u32;
            let source_end_x = placement.source_x
                + (right as u64 * placement.source_width as u64).div_ceil(placement.width as u64)
                    as u32;
            let source_end_y = placement.source_y
                + (bottom as u64 * placement.source_height as u64).div_ceil(placement.height as u64)
                    as u32;
            let visible_width = (x1 - x0) as u32;
            let visible_height = (y1 - y0) as u32;
            let opacity_u8 = opacity_to_u8(self.current_opacity());
            self.placements.push(ImagePlacement {
                placement_id: self.placements.len() as u32 + 1,
                image_handle: placement.image_handle,
                x: x0,
                y: y0,
                width: visible_width,
                height: visible_height,
                pixel_width: if placement.pixel_width == 0 {
                    0
                } else {
                    (visible_width as u64 * placement.pixel_width as u64)
                        .div_ceil(placement.width as u64) as u32
                },
                pixel_height: if placement.pixel_height == 0 {
                    0
                } else {
                    (visible_height as u64 * placement.pixel_height as u64)
                        .div_ceil(placement.height as u64) as u32
                },
                source_x: source_start_x,
                source_y: source_start_y,
                source_width: source_end_x - source_start_x,
                source_height: source_end_y - source_start_y,
                opacity: mul_div_255(u32::from(placement.opacity), u32::from(opacity_u8)) as u8,
                protocol: placement.protocol,
            });
            image_id_map[source_id as usize] = self.placements.len() as u32;
        }

        // Foreign pools need id translation (BUF-013); same-pool ids
        // copy raw like the reference. Lifetimes differ across pools,
        // but the layout is identical, so an address comparison is sound.
        let cross_pool =
            Rc::as_ptr(&self.pool) as *const () != Rc::as_ptr(&frame_buffer.pool) as *const ();
        let mut translated_ids: HashMap<u32, u32> = HashMap::new();

        let mut dy = csy;
        while dy <= cey {
            let mut last_drawn_grapheme_id = 0u32;
            let mut dx = csx;
            while dx <= cex {
                let rel_x = (dx - dest_x) as u32;
                let rel_y = (dy - dest_y) as u32;
                let sx = src_x + rel_x;
                let sy = src_y + rel_y;
                if sx >= frame_buffer.width || sy >= frame_buffer.height {
                    dx += 1;
                    continue;
                }
                let src_index = frame_buffer.coords_to_index(sx, sy);
                let mut src_char = frame_buffer.chars[src_index];
                if cross_pool && (is_grapheme_char(src_char) || is_continuation_char(src_char)) {
                    src_char =
                        self.translate_composite_char(frame_buffer, src_char, &mut translated_ids);
                }
                if is_image_char(src_char) {
                    let source_id = image_id_from_char(src_char);
                    let mapped = image_id_map.get(source_id as usize).copied().unwrap_or(0);
                    src_char = if mapped != 0 {
                        pack_image_cell(mapped, image_fallback_from_char(src_char))
                    } else {
                        QUADRANT_CHARS[image_fallback_from_char(src_char) as usize]
                    };
                }
                let src_fg = frame_buffer.fgs[src_index];
                let src_bg = frame_buffer.bgs[src_index];
                let src_attr = frame_buffer.attributes[src_index];
                let transparent_cell = ansi::alpha(src_bg) == 0 && ansi::alpha(src_fg) == 0;
                if transparent_cell
                    && is_image_char(src_char)
                    && let Some(current) = self.get(dx as u32, dy as u32)
                {
                    self.set(
                        dx as u32,
                        dy as u32,
                        make_cell(src_char, current.fg, current.bg, current.attributes),
                    );
                }
                if transparent_cell {
                    dx += 1;
                    continue;
                }
                if grapheme_aware {
                    if is_continuation_char(src_char) {
                        let gid = src_char & GRAPHEME_ID_MASK;
                        if gid != last_drawn_grapheme_id {
                            self.set_cell_with_alpha_blending_cell(
                                dx as u32,
                                dy as u32,
                                make_cell(DEFAULT_SPACE_CHAR, src_fg, src_bg, src_attr),
                            );
                        }
                        dx += 1;
                        continue;
                    }
                    if is_grapheme_char(src_char) {
                        last_drawn_grapheme_id = src_char & GRAPHEME_ID_MASK;
                    }
                    self.set_cell_with_alpha_blending_cell(
                        dx as u32,
                        dy as u32,
                        make_cell(src_char, src_fg, src_bg, src_attr),
                    );
                    dx += 1;
                    continue;
                }
                self.set_cell_with_alpha_blending_raw_cell(
                    dx as u32,
                    dy as u32,
                    make_cell(src_char, src_fg, src_bg, src_attr),
                );
                dx += 1;
            }
            dy += 1;
        }
    }

    // ---- ANSI cell emission ----

    /// Escape sequences for one cell: fg, bg, then style flags in
    /// reference order, then the character bytes. Grapheme bytes resolve
    /// through this grid's pool; image cells use the quadrant fallback;
    /// continuation cells emit nothing. `rgb`/`ansi256` are the terminal
    /// capabilities (quantized fallback arrives with render-terminal).
    pub fn cell_ansi(&self, x: u32, y: u32, rgb: bool, ansi256: bool) -> Option<String> {
        let cell = self.get(x, y)?;
        let mut out = String::new();
        out.push_str(&color_seq(cell.fg, false, rgb, ansi256));
        out.push_str(&color_seq(cell.bg, true, rgb, ansi256));
        let base = TextAttributes::base_attributes(cell.attributes);
        if base & TextAttributes::BOLD != 0 {
            out.push_str("\x1b[1m");
        }
        if base & TextAttributes::DIM != 0 {
            out.push_str("\x1b[2m");
        }
        if base & TextAttributes::ITALIC != 0 {
            out.push_str("\x1b[3m");
        }
        if base & TextAttributes::UNDERLINE != 0 {
            out.push_str("\x1b[4m");
        }
        if base & TextAttributes::BLINK != 0 {
            out.push_str("\x1b[5m");
        }
        if base & TextAttributes::INVERSE != 0 {
            out.push_str("\x1b[7m");
        }
        if base & TextAttributes::HIDDEN != 0 {
            out.push_str("\x1b[8m");
        }
        if base & TextAttributes::STRIKETHROUGH != 0 {
            out.push_str("\x1b[9m");
        }
        if cell.char == 0 {
            out.push(' ');
        } else if is_image_char(cell.char) {
            let fallback = QUADRANT_CHARS[image_fallback_from_char(cell.char) as usize];
            push_codepoint(&mut out, fallback);
        } else if is_grapheme_char(cell.char) {
            let gid = grapheme_id_from_char(cell.char);
            match self.pool.borrow().get(gid) {
                Ok(bytes) if !bytes.is_empty() => {
                    out.push_str(core::str::from_utf8(bytes).unwrap_or(" "));
                }
                _ => out.push(' '),
            }
        } else if is_continuation_char(cell.char) {
            // Continuations serialize as nothing (reference frame output).
        } else if cell.char > super::MAX_UNICODE_CODEPOINT {
            out.push(' ');
        } else {
            push_codepoint(&mut out, cell.char);
        }
        Some(out)
    }
}

/// One fg/bg sequence by intent. Reference `emitColor` with truecolor
/// capabilities (quantized path arrives with render-terminal).
fn color_seq(color: Rgba, is_background: bool, rgb: bool, ansi256: bool) -> String {
    if ansi::intent(color) == ansi::ColorIntent::Default {
        return if is_background {
            "\x1b[49m".to_string()
        } else {
            "\x1b[39m".to_string()
        };
    }
    if is_background && ansi::alpha(color) == 0 {
        return "\x1b[49m".to_string();
    }
    if ansi::intent(color) == ansi::ColorIntent::Indexed && ansi256 {
        let index = ansi::slot(color);
        return if is_background {
            format!("\x1b[48;5;{index}m")
        } else {
            format!("\x1b[38;5;{index}m")
        };
    }
    if rgb {
        let (r, g, b) = (ansi::red(color), ansi::green(color), ansi::blue(color));
        return if is_background {
            format!("\x1b[48;2;{r};{g};{b}m")
        } else {
            format!("\x1b[38;2;{r};{g};{b}m")
        };
    }
    // No color capability: emit nothing (quantized path is render's).
    String::new()
}

fn push_codepoint(out: &mut String, cp: u32) {
    match char::from_u32(cp) {
        Some(ch) => out.push(ch),
        None => out.push(' '),
    }
}

/// BUF-008 falsifier: blending is integer-exact, links come from the
/// overlay, and a space over a narrow char preserves it.
#[cfg(test)]
#[test]
fn blending() {
    use super::{InitOptions, OptimizedBuffer, make_cell};
    use crate::ansi::{ColorIntent, intent, rgb_color, rgba_from_floats};
    use crate::uni::pool::GraphemePool;
    use std::cell::RefCell;
    use std::rc::Rc;

    // Opaque source wins byte-exact.
    let src = rgb_color(10, 20, 30, 255);
    let dst = rgb_color(200, 210, 220, 255);
    assert_eq!(src, blend_colors(src, dst, None));
    // Transparent source keeps the destination untouched (no backdrop).
    let clear = rgb_color(0, 0, 0, 0);
    assert_eq!(dst, blend_colors(clear, dst, None));
    // Backdrop substitutes only for a transparent destination; an
    // opaque source still wins byte-exact.
    let back = rgb_color(255, 255, 255, 255);
    assert_eq!(src, blend_colors(src, clear, Some(back)));
    // Fully transparent source keeps the destination as-is (no backdrop).
    assert_eq!(clear, blend_colors(clear, clear, Some(back)));

    // Half white over opaque black flattens to 127 gray.
    let semi = rgba_from_floats(0.0, 0.0, 0.0, 0.5);
    let flat = blend_colors(semi, rgb_color(0, 0, 0, 0), Some(back));
    assert_eq!(
        (127, 127, 127, 255),
        (
            ansi::red(flat),
            ansi::green(flat),
            ansi::blue(flat),
            ansi::alpha(flat)
        )
    );

    // Blended results downgrade metadata to rgb.
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut buf = OptimizedBuffer::new(2, 1, InitOptions::new(Rc::clone(&pool))).unwrap();
    let base = rgb_color(0, 0, 0, 255);
    buf.clear(base, None);
    buf.set_cell_with_alpha_blending(
        0,
        0,
        u32::from(b'B'),
        ansi::pack_rgba8(255, 0, 0, 128, ansi::pack_meta(ColorIntent::Indexed, 3)),
        base,
        0,
    );
    let cell = buf.get(0, 0).unwrap();
    assert_eq!(ColorIntent::Rgb, intent(cell.fg));

    // Links always come from the overlay, even when zero.
    let linked = crate::ansi::TextAttributes::set_link_id(0, 0);
    let overlay = make_cell(0x58, src, src, linked);
    let dest_cell = make_cell(
        0x59,
        dst,
        dst,
        crate::ansi::TextAttributes::set_link_id(0, 77),
    );
    let blended = buf.blend_cells(overlay, dest_cell);
    assert_eq!(0x58, blended.char);
    assert_eq!(0, crate::ansi::TextAttributes::link_id(blended.attributes));
}

/// BUF-009 falsifier: scissor push/nest/pop/clear plus clipped writes.
#[cfg(test)]
#[test]
fn scissor() {
    use super::{ClipRect, InitOptions, OptimizedBuffer, make_cell};
    use crate::ansi::rgb_color;
    use crate::uni::pool::GraphemePool;
    use std::cell::RefCell;
    use std::rc::Rc;

    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut buf = OptimizedBuffer::new(8, 8, InitOptions::new(Rc::clone(&pool))).unwrap();
    assert!(buf.current_scissor().is_none());
    buf.push_scissor_rect(2, 2, 4, 4);
    assert_eq!(
        Some(ClipRect {
            x: 2,
            y: 2,
            width: 4,
            height: 4
        }),
        buf.current_scissor()
    );
    // Nested push intersects with the parent.
    buf.push_scissor_rect(4, 4, 4, 4);
    assert_eq!(
        Some(ClipRect {
            x: 4,
            y: 4,
            width: 2,
            height: 2
        }),
        buf.current_scissor()
    );
    // Fully outside push goes degenerate, never unclipped.
    buf.push_scissor_rect(20, 20, 2, 2);
    assert_eq!(
        Some(ClipRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0
        }),
        buf.current_scissor()
    );
    buf.pop_scissor();
    buf.pop_scissor();
    assert_eq!(
        Some(ClipRect {
            x: 2,
            y: 2,
            width: 4,
            height: 4
        }),
        buf.current_scissor()
    );
    buf.clear_scissors();
    assert!(buf.current_scissor().is_none());

    // Clipped writes land only inside.
    let fg = rgb_color(255, 255, 255, 255);
    let bg = rgb_color(0, 0, 0, 255);
    buf.push_scissor_rect(1, 1, 2, 2);
    buf.set(0, 0, make_cell(0x41, fg, bg, 0));
    buf.set(1, 1, make_cell(0x42, fg, bg, 0));
    assert_eq!(0, buf.get(0, 0).unwrap().char);
    assert_eq!(0x42, buf.get(1, 1).unwrap().char);
    buf.clear_scissors();

    // Rectangle helpers.
    assert!(buf.rect_in_scissor(0, 0, 8, 8));
    buf.push_scissor_rect(2, 2, 2, 2);
    assert!(buf.rect_in_scissor(3, 3, 4, 4));
    assert!(!buf.rect_in_scissor(0, 0, 2, 2));
    assert_eq!(
        Some(ClipRect {
            x: 2,
            y: 2,
            width: 2,
            height: 2
        }),
        buf.clip_rect_to_scissor(0, 0, 8, 8)
    );
    assert!(buf.clip_rect_to_scissor(5, 5, 2, 2).is_none());
}

/// BUF-010 falsifier: a wide grapheme at the edge fills EOL, and the
/// synced write skips span cleanup.
#[cfg(test)]
#[test]
fn wide_cells() {
    use super::{InitOptions, OptimizedBuffer, make_cell};
    use crate::ansi::rgb_color;
    use crate::uni::pool::GraphemePool;
    use crate::uni::segments::{
        GRAPHEME_ID_MASK, char_right_extent, grapheme_id_from_char, is_continuation_char,
        is_grapheme_char, pack_grapheme_start,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut buf = OptimizedBuffer::new(4, 1, InitOptions::new(Rc::clone(&pool))).unwrap();
    let fg = rgb_color(255, 255, 255, 255);
    let bg = rgb_color(0, 0, 0, 255);
    buf.clear(bg, None);

    // Width-2 grapheme at the last column cannot fit: EOL fill.
    let gid = pool.borrow_mut().alloc("🌟".as_bytes()).unwrap() & GRAPHEME_ID_MASK;
    buf.set(3, 0, make_cell(pack_grapheme_start(gid, 2), fg, bg, 0xAB));
    let edge = buf.get(3, 0).unwrap();
    assert_eq!(0x20, edge.char);
    assert_eq!(0xAB, edge.attributes);

    // A fitting wide write lays start + continuation.
    buf.set(0, 0, make_cell(pack_grapheme_start(gid, 2), fg, bg, 0));
    let start = buf.get(0, 0).unwrap();
    assert!(is_grapheme_char(start.char));
    assert_eq!(1, char_right_extent(start.char));
    let cont = buf.get(1, 0).unwrap();
    assert!(is_continuation_char(cont.char));
    assert_eq!(gid, grapheme_id_from_char(cont.char));

    // sync_cell skips span cleanup but still swaps tracker refs.
    buf.sync_cell(0, 0, make_cell(0x20, fg, bg, 0));
    assert_eq!(0, buf.grapheme_tracker.grapheme_count());
    assert_eq!(0x20, buf.get(0, 0).unwrap().char);
}

/// BUF-011 falsifier: per-cell escape sequences by intent with the
/// reference flag order.
#[cfg(test)]
#[test]
fn ansi_output() {
    use super::{InitOptions, OptimizedBuffer, make_cell};
    use crate::ansi::{TextAttributes, default_color, indexed_color, rgb_color};
    use crate::uni::pool::GraphemePool;
    use std::cell::RefCell;
    use std::rc::Rc;

    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut buf = OptimizedBuffer::new(3, 1, InitOptions::new(Rc::clone(&pool))).unwrap();
    buf.set(
        0,
        0,
        make_cell(
            0x41,
            rgb_color(255, 0, 0, 255),
            rgb_color(0, 0, 255, 255),
            0,
        ),
    );
    assert_eq!(
        "\x1b[38;2;255;0;0m\x1b[48;2;0;0;255mA",
        buf.cell_ansi(0, 0, true, true).unwrap().as_str()
    );
    buf.set(
        1,
        0,
        make_cell(
            0x42,
            indexed_color(9, 255, 0, 0),
            default_color(0, 0, 0, 255),
            0,
        ),
    );
    assert_eq!(
        "\x1b[38;5;9m\x1b[49mB",
        buf.cell_ansi(1, 0, true, true).unwrap().as_str()
    );
    let flags = TextAttributes::BOLD | TextAttributes::ITALIC;
    buf.set(
        2,
        0,
        make_cell(
            0x43,
            rgb_color(0, 0, 0, 255),
            rgb_color(0, 0, 0, 0),
            u32::from(flags),
        ),
    );
    assert_eq!(
        "\x1b[38;2;0;0;0m\x1b[49m\x1b[1m\x1b[3mC",
        buf.cell_ansi(2, 0, true, true).unwrap().as_str()
    );
    // Out of grid is None, never a panic.
    assert!(buf.cell_ansi(9, 9, true, true).is_none());
}

/// BUF-012 falsifier: same-pool region copies with clipping and offsets.
#[cfg(test)]
#[test]
fn composite() {
    use super::{InitOptions, OptimizedBuffer, make_cell};
    use crate::ansi::rgb_color;
    use crate::uni::pool::GraphemePool;
    use std::cell::RefCell;
    use std::rc::Rc;

    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut src = OptimizedBuffer::new(6, 4, InitOptions::new(Rc::clone(&pool))).unwrap();
    let mut dst = OptimizedBuffer::new(6, 4, InitOptions::new(Rc::clone(&pool))).unwrap();
    let fg = rgb_color(255, 255, 255, 255);
    let bg = rgb_color(0, 0, 0, 255);
    src.clear(bg, None);
    dst.clear(bg, None);
    for (i, ch) in ["A", "B", "C"].iter().enumerate() {
        src.set(
            i as u32,
            0,
            make_cell(ch.chars().next().unwrap() as u32, fg, bg, 0),
        );
    }
    dst.draw_frame_buffer(1, 1, &src, None, None, None, None);
    assert_eq!(u32::from(b'A'), dst.get(1, 1).unwrap().char);
    assert_eq!(u32::from(b'C'), dst.get(3, 1).unwrap().char);
    assert_eq!(0x20, dst.get(0, 0).unwrap().char);

    // Negative offsets clip the source region.
    let mut dst2 = OptimizedBuffer::new(6, 4, InitOptions::new(Rc::clone(&pool))).unwrap();
    dst2.clear(bg, None);
    dst2.draw_frame_buffer(-1, 0, &src, None, None, None, None);
    assert_eq!(u32::from(b'B'), dst2.get(0, 0).unwrap().char);
}

/// BUF-013 falsifier: a cross-pool composite resolves a multi-codepoint
/// cluster through the source grid's pool, never a foreign id.
#[cfg(test)]
#[test]
fn cross_pool_composite() {
    use super::{InitOptions, OptimizedBuffer, make_cell};
    use crate::ansi::rgb_color;
    use crate::uni::pool::GraphemePool;
    use crate::uni::segments::{
        GRAPHEME_ID_MASK, char_right_extent, grapheme_id_from_char, is_continuation_char,
        is_grapheme_char, pack_grapheme_start,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

    let src_pool = Rc::new(RefCell::new(GraphemePool::new()));
    let dst_pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut src = OptimizedBuffer::new(6, 2, InitOptions::new(Rc::clone(&src_pool))).unwrap();
    let mut dst = OptimizedBuffer::new(6, 2, InitOptions::new(Rc::clone(&dst_pool))).unwrap();
    let fg = rgb_color(255, 255, 255, 255);
    let bg = rgb_color(0, 0, 0, 255);
    src.clear(bg, None);
    dst.clear(bg, None);

    let src_gid = src_pool.borrow_mut().alloc("é".as_bytes()).unwrap() & GRAPHEME_ID_MASK;
    src.set(1, 0, make_cell(pack_grapheme_start(src_gid, 1), fg, bg, 0));
    let wide_gid = src_pool.borrow_mut().alloc("你".as_bytes()).unwrap() & GRAPHEME_ID_MASK;
    src.set(3, 0, make_cell(pack_grapheme_start(wide_gid, 2), fg, bg, 0));

    dst.draw_frame_buffer(0, 0, &src, None, None, None, None);

    let start = dst.get(1, 0).unwrap();
    assert!(is_grapheme_char(start.char));
    let dst_gid = grapheme_id_from_char(start.char);
    assert_eq!("é".as_bytes(), dst_pool.borrow().get(dst_gid).unwrap());
    let wstart = dst.get(3, 0).unwrap();
    assert!(is_grapheme_char(wstart.char));
    assert_eq!(1, char_right_extent(wstart.char));
    let wcont = dst.get(4, 0).unwrap();
    assert!(is_continuation_char(wcont.char));
    assert_eq!(
        grapheme_id_from_char(wstart.char),
        grapheme_id_from_char(wcont.char)
    );
    assert_eq!(
        "你".as_bytes(),
        dst_pool
            .borrow()
            .get(grapheme_id_from_char(wstart.char))
            .unwrap()
    );
}
