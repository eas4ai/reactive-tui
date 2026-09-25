/// Chart widgets for data visualization
pub mod charts;
/// Data table with filtering, pagination, and virtual scrolling
pub mod data_table;
/// File explorer widget for file system navigation
pub mod file_explorer;
/// Image display widgets for terminal graphics
pub mod image;
/// Modal dialog components
pub mod modal;
pub(super) mod overlay;
/// Popover and tooltip components
pub mod popover;
/// Progress bar components
pub mod progress_bar;
/// Table display components
pub mod table;
/// Tree view components
pub mod tree;

pub use charts::plot;
pub use charts::{
    AreaChartBuilder, BarChartBuilder, CandlestickChartBuilder, DonutChartBuilder,
    LineChartBuilder, PieChartBuilder, RadarChartBuilder, SankeyChartBuilder, ScatterChartBuilder,
};
pub use charts::{
    BarGrowth, Candle, Chart, ChartAxis, ChartLegend, ChartProps, ChartState, ChartType,
    ChartsBuilder, Curve, DataPoint, DataSeries, FillStyle, GlyphSet, LegendPosition, LineStyle,
    RadialOptions, SankeyAlign, SankeyLabel, SankeyLink, SankeyOptions, SankeyValueScale,
    SizeClass,
};
pub use data_table::{
    ColumnFilter, DataTable, DataTableProps, DataTableState, FilterType, PaginationConfig,
    VirtualScrollConfig,
};
pub use file_explorer::{
    FileEntry, FileExplorer, FileExplorerBuilder, FileExplorerProps, FileExplorerState, FileType,
    SelectionMode, SortCriteria, SortOrder, ViewMode,
};
pub use image::{
    image_blitter, set_image_blitter, Blitter, Image, ImageCapabilities, ImageDisplayMode,
    ImageFormat, ImageQuality, ImageSource,
};
pub use modal::Modal;
pub use popover::Popover;
pub use progress_bar::{
    ProgressBar, ProgressBarBuilder, ProgressBarOrientation, ProgressBarProps, ProgressBarState,
};
pub use table::Table;
pub use tree::{Tree, TreeBuilder, TreeNode, TreeProps, TreeState};

/// Common display component utilities
#[derive(Debug, Clone, PartialEq)]
pub enum DisplaySize {
    /// Automatic sizing based on content
    Auto,
    /// Fixed size in pixels
    Fixed(u16),
    /// Percentage of parent size
    Percent(f32),
    /// Flexible sizing with weight
    Flex(f32),
}

/// Alignment options for content positioning
#[derive(Debug, Clone, PartialEq)]
pub enum Alignment {
    /// Align to start (left/top)
    Start,
    /// Center alignment
    Center,
    /// Align to end (right/bottom)
    End,
    /// Stretch to fill available space
    Stretch,
}

/// Border configuration for widgets
#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    /// Whether border is enabled
    pub enabled: bool,
    /// Style of the border
    pub style: BorderStyle,
    /// Optional border color
    pub color: Option<String>,
}

/// Border style options
#[derive(Debug, Clone, PartialEq)]
pub enum BorderStyle {
    /// Single line border
    Single,
    /// Double line border
    Double,
    /// Rounded corner border
    Rounded,
    /// Thick line border
    Thick,
    /// No border
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
    /// Horizontal scroll offset
    pub offset_x: u16,
    /// Vertical scroll offset
    pub offset_y: u16,
    /// Width of the viewport
    pub viewport_width: u16,
    /// Height of the viewport
    pub viewport_height: u16,
    /// Total content width
    pub content_width: u16,
    /// Total content height
    pub content_height: u16,
}

impl ScrollState {
    /// Create a new scroll state with default values
    ///
    /// # Returns
    /// A new `ScrollState` instance with zero offsets and dimensions
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

    /// Check if content can be scrolled up
    ///
    /// # Returns
    /// true if there is content above the current viewport
    pub fn can_scroll_up(&self) -> bool {
        self.offset_y > 0
    }

    /// Check if content can be scrolled down
    ///
    /// # Returns
    /// true if there is content below the current viewport
    pub fn can_scroll_down(&self) -> bool {
        self.offset_y + self.viewport_height < self.content_height
    }

    /// Check if content can be scrolled left
    ///
    /// # Returns
    /// true if there is content to the left of the current viewport
    pub fn can_scroll_left(&self) -> bool {
        self.offset_x > 0
    }

    /// Check if content can be scrolled right
    ///
    /// # Returns
    /// true if there is content to the right of the current viewport
    pub fn can_scroll_right(&self) -> bool {
        self.offset_x + self.viewport_width < self.content_width
    }

    /// Scroll content up by the specified amount
    ///
    /// # Arguments
    /// * `amount` - Number of units to scroll up
    pub fn scroll_up(&mut self, amount: u16) {
        self.offset_y = self.offset_y.saturating_sub(amount);
    }

    /// Scroll content down by the specified amount
    ///
    /// # Arguments
    /// * `amount` - Number of units to scroll down
    pub fn scroll_down(&mut self, amount: u16) {
        let max_offset = self.content_height.saturating_sub(self.viewport_height);
        self.offset_y = (self.offset_y + amount).min(max_offset);
    }

    /// Scroll content left by the specified amount
    ///
    /// # Arguments
    /// * `amount` - Number of units to scroll left
    pub fn scroll_left(&mut self, amount: u16) {
        self.offset_x = self.offset_x.saturating_sub(amount);
    }

    /// Scroll content right by the specified amount
    ///
    /// # Arguments
    /// * `amount` - Number of units to scroll right
    pub fn scroll_right(&mut self, amount: u16) {
        let max_offset = self.content_width.saturating_sub(self.viewport_width);
        self.offset_x = (self.offset_x + amount).min(max_offset);
    }

    /// Scroll to the top of the content
    pub fn scroll_to_top(&mut self) {
        self.offset_y = 0;
    }

    /// Scroll to the bottom of the content
    ///
    /// Sets the vertical offset to show the bottom of the content area.
    pub fn scroll_to_bottom(&mut self) {
        self.offset_y = self.content_height.saturating_sub(self.viewport_height);
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}
