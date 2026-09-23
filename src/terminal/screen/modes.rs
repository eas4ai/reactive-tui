use super::*;

impl VirtualScreen {
    pub(super) fn reverse_index(&mut self) {
        self.cursor.pending_wrap = false;
        if self.cursor.row != self.scrolling_region.top {
            self.cursor.row = self.cursor.row.saturating_sub(1);
            return;
        }
        let region = self.scrolling_region;
        self.scroll_lines(region.top, region.bottom, 1, false, false);
    }

    pub(super) fn address_cursor(&mut self, col: u16, row: u16) {
        let row = if self.modes.origin_mode {
            row.saturating_add(self.scrolling_region.top)
                .min(self.scrolling_region.bottom)
        } else {
            row
        };
        self.set_cursor_position(col, row);
    }

    pub(super) fn set_scroll_margins(&mut self, params: &[u16]) {
        let top = params.first().copied().unwrap_or(1).max(1) - 1;
        let bottom = params
            .get(1)
            .copied()
            .filter(|&n| n != 0)
            .unwrap_or(self.height)
            .saturating_sub(1);
        if top < bottom && bottom < self.height {
            self.scrolling_region.top = top;
            self.scrolling_region.bottom = bottom;
            self.address_cursor(0, 0);
        }
    }

    pub(super) fn set_private_modes(&mut self, params: &[u16], enabled: bool) {
        for &mode in params {
            match mode {
                1 => self.modes.application_cursor_keys = enabled,
                6 => {
                    self.modes.origin_mode = enabled;
                    self.address_cursor(0, 0);
                }
                7 => {
                    self.modes.auto_wrap = enabled;
                    self.cursor.pending_wrap = false;
                }
                25 => self.modes.cursor_visible = enabled,
                47 | 1047 | 1049 => self.switch_screen(mode, enabled),
                1048 if enabled => self.save_cursor(),
                1048 => self.restore_cursor(),
                2004 => self.modes.bracketed_paste = enabled,
                _ => {}
            }
        }
    }

    fn switch_screen(&mut self, mode: u16, enabled: bool) {
        if self.using_alt_screen == enabled {
            return;
        }
        if enabled {
            if mode == 1049 {
                self.saved_cursor = Some(self.cursor.clone());
                self.alt_buffer = Self::create_buffer(self.width, self.height);
                self.cursor.move_to(0, 0);
            }
        } else {
            if mode == 1047 {
                self.alt_buffer = Self::create_buffer(self.width, self.height);
            }
            if mode == 1049 {
                if let Some(cursor) = self.saved_cursor.take() {
                    self.cursor = cursor;
                    self.cursor.col = self.cursor.col.min(self.width.saturating_sub(1));
                    self.cursor.row = self.cursor.row.min(self.height.saturating_sub(1));
                }
            }
        }
        self.cursor.pending_wrap = false;
        self.using_alt_screen = enabled;
        self.modes.alternate_screen = enabled;
    }
}
