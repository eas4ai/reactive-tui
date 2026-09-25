//! Terminal cursor management

use super::{ScrollingRegion, TerminalStyle};

/// Terminal cursor shape styles
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CursorShape {
    /// Default terminal cursor
    #[default]
    Default,
    /// Solid block cursor
    Block,
    /// Underline cursor
    Underline,
    /// Vertical bar cursor
    Bar,
    /// Blinking block cursor
    BlinkingBlock,
    /// Blinking underline cursor
    BlinkingUnderline,
    /// Blinking vertical bar cursor
    BlinkingBar,
}

/// Terminal cursor state and properties
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalCursor {
    /// Current column position (0-based)
    pub col: u16,
    /// Current row position (0-based)
    pub row: u16,
    /// Whether the cursor is visible
    pub visible: bool,
    /// Shape of the cursor
    pub shape: CursorShape,
    /// Style applied to the cursor
    pub style: TerminalStyle,
    /// Whether the cursor is pending a wrap to next line
    pub pending_wrap: bool,
    /// Saved cursor position for restore operations
    pub(super) saved_position: Option<(u16, u16)>,
    /// Saved cursor style for restore operations
    saved_style: Option<TerminalStyle>,
    /// URL for hyperlink at cursor position
    pub hyperlink_url: Option<String>,
    /// ID for hyperlink at cursor position
    pub hyperlink_id: Option<String>,
}

impl TerminalCursor {
    /// Create a new terminal cursor at origin
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

    /// Move cursor to specific position
    pub fn move_to(&mut self, col: u16, row: u16) {
        self.col = col;
        self.row = row;
        self.pending_wrap = false;
    }

    /// Move cursor to specific column
    pub fn move_to_col(&mut self, col: u16) {
        self.col = col;
        self.pending_wrap = false;
    }

    /// Move cursor to specific row
    pub fn move_to_row(&mut self, row: u16) {
        self.row = row;
        self.pending_wrap = false;
    }

    /// Move cursor up by n rows
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

    /// Move cursor down by n rows
    pub fn move_down(
        &mut self,
        n: u16,
        scrolling_region: Option<&ScrollingRegion>,
        screen_height: u16,
    ) {
        self.pending_wrap = false;
        match scrolling_region {
            Some(region) if self.is_within_region(region) => {
                self.row = self.row.saturating_add(n).min(region.bottom);
            }
            _ => {
                self.row = self
                    .row
                    .saturating_add(n)
                    .min(screen_height.saturating_sub(1));
            }
        }
    }

    /// Move cursor left by n columns
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

    /// Move cursor right by n columns
    pub fn move_right(
        &mut self,
        n: u16,
        scrolling_region: Option<&ScrollingRegion>,
        screen_width: u16,
    ) {
        self.pending_wrap = false;
        match scrolling_region {
            Some(region) if self.is_within_region(region) => {
                self.col = self.col.saturating_add(n).min(region.right);
            }
            _ => {
                self.col = self
                    .col
                    .saturating_add(n)
                    .min(screen_width.saturating_sub(1));
            }
        }
    }

    /// Move cursor to beginning of current line
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

    /// Move cursor down one line
    pub fn line_feed(&mut self, scrolling_region: Option<&ScrollingRegion>, screen_height: u16) {
        self.pending_wrap = false;
        self.move_down(1, scrolling_region, screen_height);
    }

    /// Move cursor to beginning of next line
    pub fn new_line(&mut self, scrolling_region: Option<&ScrollingRegion>, screen_height: u16) {
        self.line_feed(scrolling_region, screen_height);
        self.carriage_return(scrolling_region);
    }

    /// Advance cursor by character width
    pub fn advance(&mut self, char_width: u8, screen_width: u16, auto_wrap: bool) {
        let next = self.col.saturating_add(u16::from(char_width));
        self.pending_wrap = auto_wrap && next >= screen_width;
        self.col = next.min(screen_width.saturating_sub(1));
    }

    /// Handle pending line wrap
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

    /// Save current cursor position and style
    pub fn save(&mut self) {
        self.saved_position = Some((self.col, self.row));
        self.saved_style = Some(self.style);
    }

    /// Restore saved cursor position and style
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

    /// Set cursor visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Set the cursor shape
    ///
    /// # Arguments
    /// * `shape` - The new cursor shape to use
    pub fn set_shape(&mut self, shape: CursorShape) {
        self.shape = shape;
    }

    /// Check if the cursor is within a scrolling region
    ///
    /// # Arguments
    /// * `region` - The scrolling region to check against
    ///
    /// # Returns
    /// true if the cursor is within the region bounds
    pub fn is_within_region(&self, region: &ScrollingRegion) -> bool {
        region.contains(self.col, self.row)
    }

    /// Constrain the cursor position to screen bounds
    ///
    /// # Arguments
    /// * `width` - Screen width in columns
    /// * `height` - Screen height in rows
    pub fn constrain_to_screen(&mut self, width: u16, height: u16) {
        self.col = self.col.min(width.saturating_sub(1));
        self.row = self.row.min(height.saturating_sub(1));
        self.pending_wrap = false;
    }

    /// Constrain the cursor position to a scrolling region
    ///
    /// # Arguments
    /// * `region` - The scrolling region to constrain to
    pub fn constrain_to_region(&mut self, region: &ScrollingRegion) {
        self.col = self.col.clamp(region.left, region.right);
        self.row = self.row.clamp(region.top, region.bottom);
        self.pending_wrap = false;
    }

    /// Set hyperlink information for the cursor
    ///
    /// # Arguments
    /// * `url` - Optional URL for the hyperlink
    /// * `id` - Optional ID for the hyperlink
    pub fn set_hyperlink(&mut self, url: Option<String>, id: Option<String>) {
        self.hyperlink_url = url;
        self.hyperlink_id = id;
    }

    /// Clear hyperlink information from the cursor
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

    #[test]
    fn movement_saturates_and_non_wrapping_output_stays_on_screen() {
        let mut cursor = TerminalCursor::new();
        cursor.move_to(79, 23);
        cursor.advance(2, 80, false);
        assert_eq!(cursor.col, 79);
        assert!(!cursor.pending_wrap);
        cursor.move_right(u16::MAX, None, 80);
        cursor.move_down(u16::MAX, None, 24);
        assert_eq!((cursor.col, cursor.row), (79, 23));
        cursor.col = u16::MAX;
        cursor.advance(2, 80, true);
        assert_eq!(cursor.col, 79);
        assert!(cursor.pending_wrap);
    }
}
