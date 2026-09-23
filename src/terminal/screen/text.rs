use super::*;
use unicode_segmentation::UnicodeSegmentation;

impl VirtualScreen {
    pub(super) fn print_char(&mut self, ch: char) {
        if self.extend_grapheme(ch) {
            return;
        }
        let mut cell = TerminalCell::with_char(ch);
        if cell.width == 0 || self.width == 0 || self.height == 0 {
            return;
        }
        cell.style = self.cursor.style;
        cell.hyperlink = self.cursor.hyperlink_url.clone();
        cell.hyperlink_id = self.cursor.hyperlink_id.clone();
        self.place_grapheme(cell, 0);
    }

    fn extend_grapheme(&mut self, ch: char) -> bool {
        let Some((col, row)) = self.last_print else {
            return false;
        };
        let Some(mut cell) = self.cell_at(col, row).cloned() else {
            return false;
        };
        let mut joined = cell.character.clone();
        joined.push(ch);
        if joined.graphemes(true).count() != 1 {
            return false;
        }
        // A child can emit an unlimited run of combining marks. Bound each cell.
        if joined.len() > 4096 {
            return true;
        }
        self.clear_glyph_at(col, row);
        let old_width = cell.width;
        cell.set_string(joined);
        self.cursor.move_to(col, row);
        self.place_grapheme(cell, old_width);
        true
    }

    fn place_grapheme(&mut self, cell: TerminalCell, old_width: u8) {
        if u16::from(cell.width) > self.width {
            self.last_print = None;
            return;
        }
        let over_edge = self.cursor.col.saturating_add(u16::from(cell.width)) > self.width;
        if self.cursor.pending_wrap || (over_edge && self.modes.auto_wrap) {
            let end = if self.cursor.pending_wrap {
                self.width
            } else {
                self.cursor.col
            };
            self.advance_line(Some(end));
            self.carriage_return();
        } else if over_edge {
            self.last_print = None;
            return;
        }
        let (col, row) = self.cursor_position();
        if col >= self.width || row >= self.height {
            return;
        }
        let width = cell.width;
        if self.modes.insert_mode && width != old_width {
            self.shift_characters(u16::from(width.abs_diff(old_width)), width > old_width);
        }
        for x in col..col + u16::from(width) {
            self.clear_glyph_at(x, row);
        }
        let line = &mut self.current_buffer()[usize::from(row)];
        line[usize::from(col)] = cell.clone();
        for x in 1..width {
            let mut continuation = cell.clone();
            continuation.character.clear();
            continuation.width = 0;
            line[usize::from(col) + usize::from(x)] = continuation;
        }
        self.last_print = Some((col, row));
        self.cursor.advance(width, self.width, self.modes.auto_wrap);
    }

    pub(super) fn clear_glyph_at(&mut self, col: u16, row: u16) {
        let bg = self.cursor.style.background;
        let Some(line) = self.current_buffer().get_mut(usize::from(row)) else {
            return;
        };
        let mut start = usize::from(col);
        if start >= line.len() {
            return;
        }
        while start > 0 && line[start].is_wide_continuation() {
            start -= 1;
        }
        let end = (start + usize::from(line[start].width.max(1))).min(line.len());
        for cell in &mut line[start..end] {
            cell.clear(bg);
        }
    }
}
