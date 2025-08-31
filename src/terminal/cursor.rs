//! Terminal cursor management

use super::{ScrollingRegion, TerminalStyle};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorShape {
    Default,
    Block,
    Underline,
    Bar,
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}

impl Default for CursorShape {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerminalCursor {
    pub col: u16,
    pub row: u16,
    pub visible: bool,
    pub shape: CursorShape,
    pub style: TerminalStyle,
    pub pending_wrap: bool,
    saved_position: Option<(u16, u16)>,
    saved_style: Option<TerminalStyle>,
    pub hyperlink_url: Option<String>,
    pub hyperlink_id: Option<String>,
}

impl TerminalCursor {
    pub fn new() -> Self {
        Self {
            col: 0,
            row: 0,
            visible: true,
            shape: CursorShape::Default,
            style: TerminalStyle::default(),
            pending_wrap: false,
            saved_position: None,
            saved_style: None,
            hyperlink_url: None,
            hyperlink_id: None,
        }
    }

    pub fn move_to(&mut self, col: u16, row: u16) {
        self.col = col;
        self.row = row;
        self.pending_wrap = false;
    }

    pub fn move_to_col(&mut self, col: u16) {
        self.col = col;
        self.pending_wrap = false;
    }

    pub fn move_to_row(&mut self, row: u16) {
        self.row = row;
        self.pending_wrap = false;
    }

    pub fn move_up(&mut self, n: u16, scrolling_region: Option<&ScrollingRegion>) {
        self.pending_wrap = false;
        match scrolling_region {
            Some(region) if self.is_within_region(region) => {
                self.row = self.row.saturating_sub(n).max(region.top);
            }
            _ => {
                self.row = self.row.saturating_sub(n);
            }
        }
    }

    pub fn move_down(
        &mut self,
        n: u16,
        scrolling_region: Option<&ScrollingRegion>,
        screen_height: u16,
    ) {
        self.pending_wrap = false;
        match scrolling_region {
            Some(region) if self.is_within_region(region) => {
                self.row = (self.row + n).min(region.bottom);
            }
            _ => {
                self.row = (self.row + n).min(screen_height.saturating_sub(1));
            }
        }
    }

    pub fn move_left(&mut self, n: u16, scrolling_region: Option<&ScrollingRegion>) {
        self.pending_wrap = false;
        match scrolling_region {
            Some(region) if self.is_within_region(region) => {
                self.col = self.col.saturating_sub(n).max(region.left);
            }
            _ => {
                self.col = self.col.saturating_sub(n);
            }
        }
    }

    pub fn move_right(
        &mut self,
        n: u16,
        scrolling_region: Option<&ScrollingRegion>,
        screen_width: u16,
    ) {
        self.pending_wrap = false;
        match scrolling_region {
            Some(region) if self.is_within_region(region) => {
                self.col = (self.col + n).min(region.right);
            }
            _ => {
                self.col = (self.col + n).min(screen_width.saturating_sub(1));
            }
        }
    }

    pub fn carriage_return(&mut self, scrolling_region: Option<&ScrollingRegion>) {
        self.pending_wrap = false;
        match scrolling_region {
            Some(region) if self.is_within_region(region) => {
                self.col = region.left;
            }
            _ => {
                self.col = 0;
            }
        }
    }

    pub fn line_feed(&mut self, scrolling_region: Option<&ScrollingRegion>, screen_height: u16) {
        self.pending_wrap = false;
        self.move_down(1, scrolling_region, screen_height);
    }

    pub fn new_line(&mut self, scrolling_region: Option<&ScrollingRegion>, screen_height: u16) {
        self.line_feed(scrolling_region, screen_height);
        self.carriage_return(scrolling_region);
    }

    pub fn advance(&mut self, char_width: u8, screen_width: u16, auto_wrap: bool) {
        self.col += char_width as u16;

        if auto_wrap && self.col >= screen_width {
            self.pending_wrap = true;
            self.col = screen_width.saturating_sub(1);
        }
    }

    pub fn handle_wrap(&mut self, scrolling_region: Option<&ScrollingRegion>, screen_height: u16) {
        if self.pending_wrap {
            self.pending_wrap = false;
            self.line_feed(scrolling_region, screen_height);
            match scrolling_region {
                Some(region) => self.col = region.left,
                None => self.col = 0,
            }
        }
    }

    pub fn save(&mut self) {
        self.saved_position = Some((self.col, self.row));
        self.saved_style = Some(self.style);
    }

    pub fn restore(&mut self) {
        if let Some((col, row)) = self.saved_position {
            self.col = col;
            self.row = row;
            self.pending_wrap = false;
        }
        if let Some(style) = self.saved_style {
            self.style = style;
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_shape(&mut self, shape: CursorShape) {
        self.shape = shape;
    }

    pub fn is_within_region(&self, region: &ScrollingRegion) -> bool {
        region.contains(self.col, self.row)
    }

    pub fn constrain_to_screen(&mut self, width: u16, height: u16) {
        self.col = self.col.min(width.saturating_sub(1));
        self.row = self.row.min(height.saturating_sub(1));
        self.pending_wrap = false;
    }

    pub fn constrain_to_region(&mut self, region: &ScrollingRegion) {
        self.col = self.col.clamp(region.left, region.right);
        self.row = self.row.clamp(region.top, region.bottom);
        self.pending_wrap = false;
    }

    pub fn set_hyperlink(&mut self, url: Option<String>, id: Option<String>) {
        self.hyperlink_url = url;
        self.hyperlink_id = id;
    }

    pub fn clear_hyperlink(&mut self) {
        self.hyperlink_url = None;
        self.hyperlink_id = None;
    }
}

impl Default for TerminalCursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_creation() {
        let cursor = TerminalCursor::new();
        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.row, 0);
        assert!(cursor.visible);
        assert_eq!(cursor.shape, CursorShape::Default);
    }

    #[test]
    fn test_cursor_movement() {
        let mut cursor = TerminalCursor::new();

        cursor.move_to(5, 10);
        assert_eq!(cursor.col, 5);
        assert_eq!(cursor.row, 10);
        assert!(!cursor.pending_wrap);
    }

    #[test]
    fn test_cursor_advance() {
        let mut cursor = TerminalCursor::new();

        cursor.advance(1, 80, true);
        assert_eq!(cursor.col, 1);
        assert!(!cursor.pending_wrap);

        cursor.col = 79;
        cursor.advance(1, 80, true);
        assert_eq!(cursor.col, 79);
        assert!(cursor.pending_wrap);
    }

    #[test]
    fn test_cursor_save_restore() {
        let mut cursor = TerminalCursor::new();
        cursor.move_to(10, 5);
        cursor.style.bold = true;

        cursor.save();
        cursor.move_to(20, 15);
        cursor.style.bold = false;

        cursor.restore();
        assert_eq!(cursor.col, 10);
        assert_eq!(cursor.row, 5);
        assert!(cursor.style.bold);
    }
}
