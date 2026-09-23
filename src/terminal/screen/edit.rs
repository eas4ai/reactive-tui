use super::*;

impl VirtualScreen {
    pub(super) fn shift_characters(&mut self, count: u16, insert: bool) {
        self.cursor.pending_wrap = false;
        let (col, row) = self.cursor_position();
        if col >= self.width || row >= self.height {
            return;
        }
        let count = usize::from(count.min(self.width - col));
        if count == 0 {
            return;
        }
        if self
            .cell_at(col, row)
            .is_some_and(TerminalCell::is_wide_continuation)
        {
            self.clear_glyph_at(col, row);
        }
        if !insert {
            let end = col + count as u16;
            if self
                .cell_at(end, row)
                .is_some_and(TerminalCell::is_wide_continuation)
            {
                self.clear_glyph_at(end, row);
            }
        }
        let background = self.cursor.style.background;
        let line = &mut self.current_buffer()[usize::from(row)];
        let start = usize::from(col);
        let end = line.len();
        let blanks = if insert {
            line[start..].rotate_right(count);
            start..start + count
        } else {
            line[start..].rotate_left(count);
            end - count..end
        };
        for cell in &mut line[blanks] {
            cell.clear(background);
        }
        // Remove incomplete glyphs at either edit boundary, including the right edge.
        for x in 0..line.len() {
            let invalid = match line[x].width {
                0 => x == 0 || line[x - 1].width != 2,
                2 => x + 1 == line.len() || line[x + 1].width != 0,
                _ => false,
            };
            if invalid {
                line[x].clear(background);
            }
        }
    }

    pub(super) fn erase_characters(&mut self, count: u16) {
        self.cursor.pending_wrap = false;
        let (col, row) = self.cursor_position();
        for x in col..col.saturating_add(count).min(self.width) {
            self.clear_glyph_at(x, row);
        }
    }

    pub(super) fn shift_lines(&mut self, count: u16, insert: bool) {
        self.cursor.pending_wrap = false;
        let region = self.scrolling_region;
        if self.cursor.row < region.top || self.cursor.row > region.bottom {
            return;
        }
        self.scroll_lines(self.cursor.row, region.bottom, count, !insert, false);
    }

    pub(super) fn scroll_lines(
        &mut self,
        top: u16,
        bottom: u16,
        count: u16,
        up: bool,
        history: bool,
    ) {
        if top > bottom || bottom >= self.height {
            return;
        }
        let count = usize::from(count.min(bottom - top + 1));
        if count == 0 {
            return;
        }
        if history && up && !self.using_alt_screen && top == 0 && bottom == self.height - 1 {
            for y in 0..count {
                if self.max_scrollback == 0 {
                    break;
                }
                let row = self.current_buffer_ref()[y].clone();
                self.scrollback.push_back(row);
                while self.scrollback.len() > self.max_scrollback {
                    self.scrollback.pop_front();
                }
            }
        }
        let background = self.cursor.style.background;
        let rows = &mut self.current_buffer()[usize::from(top)..=usize::from(bottom)];
        let blanks = if up {
            rows.rotate_left(count);
            rows.len() - count..rows.len()
        } else {
            rows.rotate_right(count);
            0..count
        };
        for row in &mut rows[blanks] {
            for cell in row {
                cell.clear(background);
            }
        }
    }

    pub(super) fn tabulate(&mut self, count: u16, backward: bool) {
        self.cursor.pending_wrap = false;
        let mut col = self.cursor.col;
        // At most the screen width can contain useful stops, regardless of the count.
        for _ in 0..count.min(self.width) {
            col = if backward {
                (0..col)
                    .rev()
                    .find(|&x| self.tab_stops[usize::from(x)])
                    .unwrap_or(0)
            } else {
                (col.saturating_add(1)..self.width)
                    .find(|&x| self.tab_stops[usize::from(x)])
                    .unwrap_or(self.width.saturating_sub(1))
            };
        }
        self.cursor.col = col;
    }

    pub(super) fn clear_tab_stops(&mut self, mode: u16) {
        match mode {
            0 => {
                if let Some(stop) = self.tab_stops.get_mut(usize::from(self.cursor.col)) {
                    *stop = false;
                }
            }
            3 => self.tab_stops.fill(false),
            _ => {}
        }
    }
}
