pub mod image;
pub mod modal;
pub mod popover;
pub mod progress_bar;
pub mod table;
pub mod tree;

pub use image::{
    Image, ImageCapabilities, ImageDisplayMode, ImageFormat, ImageQuality, ImageSource,
};
pub use modal::Modal;
pub use popover::Popover;
pub use progress_bar::ProgressBar;
pub use table::Table;
pub use tree::Tree;

/// Common display component utilities
#[derive(Debug, Clone, PartialEq)]
pub enum DisplaySize {
    Auto,
    Fixed(u16),
    Percent(f32),
    Flex(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    pub enabled: bool,
    pub style: BorderStyle,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BorderStyle {
    Single,
    Double,
    Rounded,
    Thick,
    None,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            enabled: true,
            style: BorderStyle::Single,
            color: None,
        }
    }
}

/// Common scrolling behavior
#[derive(Debug, Clone)]
pub struct ScrollState {
    pub offset_x: u16,
    pub offset_y: u16,
    pub viewport_width: u16,
    pub viewport_height: u16,
    pub content_width: u16,
    pub content_height: u16,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            viewport_width: 0,
            viewport_height: 0,
            content_width: 0,
            content_height: 0,
        }
    }

    pub fn can_scroll_up(&self) -> bool {
        self.offset_y > 0
    }

    pub fn can_scroll_down(&self) -> bool {
        self.offset_y + self.viewport_height < self.content_height
    }

    pub fn can_scroll_left(&self) -> bool {
        self.offset_x > 0
    }

    pub fn can_scroll_right(&self) -> bool {
        self.offset_x + self.viewport_width < self.content_width
    }

    pub fn scroll_up(&mut self, amount: u16) {
        self.offset_y = self.offset_y.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: u16) {
        let max_offset = self.content_height.saturating_sub(self.viewport_height);
        self.offset_y = (self.offset_y + amount).min(max_offset);
    }

    pub fn scroll_left(&mut self, amount: u16) {
        self.offset_x = self.offset_x.saturating_sub(amount);
    }

    pub fn scroll_right(&mut self, amount: u16) {
        let max_offset = self.content_width.saturating_sub(self.viewport_width);
        self.offset_x = (self.offset_x + amount).min(max_offset);
    }

    pub fn scroll_to_top(&mut self) {
        self.offset_y = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.offset_y = self.content_height.saturating_sub(self.viewport_height);
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}
