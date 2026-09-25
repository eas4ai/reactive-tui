use taffy::geometry::{Line, Point, Rect, Size};
mod inline;
use taffy::prelude::{FromFr as _, TaffyGridLine, TaffyGridSpan};
use taffy::style::{
    AlignContent as TAlignContent, AlignItems as TAlign, Dimension, Display, FlexDirection,
    GridAutoFlow as TGridAutoFlow, GridPlacement, GridTemplateComponent,
    JustifyContent as TJustify, LengthPercentage, LengthPercentageAuto, Overflow, Style,
    TrackSizingFunction,
};

/// RGBA color using the standard Surface Rgba type
pub type RgbaColor = crate::core::surface::Rgba;

/// Text decoration flags
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextDecorations {
    /// Whether text should be bold
    pub bold: bool,
    /// Whether text should be italic
    pub italic: bool,
    /// Whether text should be underlined
    pub underline: bool,
    /// Whether foreground/background colors should be reversed
    pub reverse: bool,
}

/// Visual style data containing foreground color, background color, and text decorations
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VisualStyle {
    /// Foreground (text) color
    pub fg: RgbaColor,
    /// Background color
    pub bg: RgbaColor,
    /// Text decoration flags (bold, italic, etc.)
    pub decorations: TextDecorations,
}

/// Box model spacing (left, right, top, bottom)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoxSpacing {
    /// Left spacing/padding
    pub left: f32,
    /// Right spacing/padding
    pub right: f32,
    /// Top spacing/padding
    pub top: f32,
    /// Bottom spacing/padding
    pub bottom: f32,
}

impl BoxSpacing {
    /// Create a new box spacing with individual values
    ///
    /// # Arguments
    /// * `left` - Left spacing value
    /// * `right` - Right spacing value
    /// * `top` - Top spacing value
    /// * `bottom` - Bottom spacing value
    ///
    /// # Returns
    /// A new `BoxSpacing` instance
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Create uniform spacing on all sides
    ///
    /// # Arguments
    /// * `value` - The spacing value to apply to all sides
    ///
    /// # Returns
    /// A new `BoxSpacing` with equal spacing on all sides
    pub fn uniform(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    /// Create symmetric spacing (horizontal and vertical)
    ///
    /// # Arguments
    /// * `horizontal` - Spacing for left and right sides
    /// * `vertical` - Spacing for top and bottom sides
    ///
    /// # Returns
    /// A new `BoxSpacing` with symmetric spacing
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }

    /// Get the total horizontal spacing (left + right)
    ///
    /// # Returns
    /// The sum of left and right spacing values
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Get the total vertical spacing (top + bottom)
    ///
    /// # Returns
    /// The sum of top and bottom spacing values
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Flex direction for layout
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Horizontal layout, left to right
    Row,
    /// Vertical layout, top to bottom
    Column,
    /// Horizontal layout, right to left
    RowReverse,
    /// Vertical layout, bottom to top
    ColumnReverse,
}

/// Content justification along the main axis
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    /// Align items to the start of the container
    Start,
    /// Center items in the container
    Center,
    /// Align items to the end of the container
    End,
    /// Distribute items with space between them
    SpaceBetween,
    /// Distribute items with space around them
    SpaceAround,
    /// Distribute items with equal space around them
    SpaceEvenly,
}

/// Cross-axis alignment for flex items
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems {
    /// Align items to the start of the cross axis
    Start,
    /// Center items on the cross axis
    Center,
    /// Align items to the end of the cross axis
    End,
    /// Stretch items to fill the cross axis
    Stretch,
    /// Align items to their baseline
    Baseline,
}
/// Grid item placement alignment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceItems {
    /// Place items at the start of their grid area
    Start,
    /// Center items in their grid area
    Center,
    /// Place items at the end of their grid area
    End,
    /// Stretch items to fill their grid area
    Stretch,
}

/// Individual item alignment override
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignSelf {
    /// Use the parent's align-items value
    Auto,
    /// Align this item to the start
    Start,
    /// Center this item
    Center,
    /// Align this item to the end
    End,
    /// Stretch this item
    Stretch,
}

/// CSS Grid auto flow direction
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GridAutoFlow {
    /// Fill rows first
    Row,
    /// Fill columns first
    Column,
    /// Fill rows densely
    RowDense,
    /// Fill columns densely
    ColumnDense,
}

impl GridAutoFlow {
    fn to_taffy(self) -> TGridAutoFlow {
        match self {
            GridAutoFlow::Row => TGridAutoFlow::Row,
            GridAutoFlow::Column => TGridAutoFlow::Column,
            GridAutoFlow::RowDense => TGridAutoFlow::RowDense,
            GridAutoFlow::ColumnDense => TGridAutoFlow::ColumnDense,
        }
    }
}
/// Builder for creating and configuring styles
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StyleBuilder {
    #[serde(default)]
    pub(crate) responsive_profiles: Vec<(u16, StyleBuilder)>,
    #[serde(default)]
    pub(crate) accessibility: std::collections::BTreeMap<String, Option<String>>,
    #[serde(default)]
    pub(crate) unconstrained_width: Option<bool>,
    pub(crate) motion: super::motion::MotionStyle,
    pub(crate) gradient: Option<super::css::gradients::Gradient>,
    pub(crate) gradient_border: Option<super::css::gradients::GradientBorder>,
    pub(crate) text: super::text::TextStyle,
    style: Style,
    // Grid extras
    grid_cols: Option<u16>,
    grid_rows: Option<u16>,
    grid_auto_flow: Option<GridAutoFlow>,
    col_span: Option<u16>,
    row_span: Option<u16>,
    col_start: Option<i16>,
    col_end: Option<i16>,
    row_start: Option<i16>,
    row_end: Option<i16>,
    // Visual extras (Utility CSS)
    pub(crate) fg_rgba: Option<(f32, f32, f32, f32)>,
    pub(crate) bg_rgba: Option<(f32, f32, f32, f32)>,
    pub(crate) opacity: Option<f32>,
    z_index: Option<i32>,
    bold: bool,
    italic: bool,
    underline: bool,
    reverse: bool,
    strike: bool,
    // Cached padding/margin for paint-time (px only)
    pad_l: Option<f32>,
    pad_r: Option<f32>,
    pad_t: Option<f32>,
    pad_b: Option<f32>,
    mar_l: Option<f32>,
    mar_r: Option<f32>,
    mar_t: Option<f32>,
    mar_b: Option<f32>,
    // CSS Animation integration
    css_animation: Option<String>,
}

impl StyleBuilder {
    /// Resolve responsive macro styles for a terminal width in columns.
    /// App and ScreenManager do this before applying utility classes. Standalone
    /// callers can resolve explicitly before building a Taffy style.
    pub fn at_width(mut self, width: u16) -> Self {
        while let Some((minimum, style)) = self.responsive_profiles.pop() {
            if width >= minimum {
                return style;
            }
        }
        self
    }

    /// Capture styles as owned data that can cross the renderer worker boundary.
    pub fn snapshot(&self) -> StyleSnapshot {
        let responsive = !self.responsive_profiles.is_empty();
        if !self.finite_numbers() {
            return StyleSnapshot(Err("Style numbers must be finite".into()), responsive);
        }
        StyleSnapshot(
            serde_json::to_vec(self).map_err(|error| error.to_string()),
            responsive,
        )
    }

    fn finite_numbers(&self) -> bool {
        if self
            .responsive_profiles
            .iter()
            .any(|(_, style)| !style.finite_numbers())
        {
            return false;
        }
        let s = &self.style;
        let lengths = [
            s.size.width.into_raw(),
            s.size.height.into_raw(),
            s.min_size.width.into_raw(),
            s.min_size.height.into_raw(),
            s.max_size.width.into_raw(),
            s.max_size.height.into_raw(),
            s.flex_basis.into_raw(),
            s.inset.left.into_raw(),
            s.inset.right.into_raw(),
            s.inset.top.into_raw(),
            s.inset.bottom.into_raw(),
            s.margin.left.into_raw(),
            s.margin.right.into_raw(),
            s.margin.top.into_raw(),
            s.margin.bottom.into_raw(),
            s.padding.left.into_raw(),
            s.padding.right.into_raw(),
            s.padding.top.into_raw(),
            s.padding.bottom.into_raw(),
            s.border.left.into_raw(),
            s.border.right.into_raw(),
            s.border.top.into_raw(),
            s.border.bottom.into_raw(),
            s.gap.width.into_raw(),
            s.gap.height.into_raw(),
        ];
        let m = self.motion.transform;
        let numbers = [
            s.flex_grow,
            s.flex_shrink,
            s.scrollbar_width,
            s.aspect_ratio.unwrap_or(1.0),
            self.opacity.unwrap_or(1.0),
            m.x,
            m.y,
            m.x_percent,
            m.y_percent,
            m.scale_x,
            m.scale_y,
            m.rotation,
            m.skew_x,
            m.skew_y,
        ];
        let colors = [self.fg_rgba, self.bg_rgba];
        let gradients = [
            self.gradient.as_ref(),
            self.gradient_border.as_ref().map(|border| &border.gradient),
        ];
        lengths.iter().all(|length| length.value().is_finite())
            && numbers
                .iter()
                .chain(m.matrix.iter())
                .all(|value| value.is_finite())
            && colors
                .iter()
                .flatten()
                .all(|&(r, g, b, a)| [r, g, b, a].iter().all(|value| value.is_finite()))
            && gradients.iter().flatten().all(|gradient| {
                [gradient.stops.from, gradient.stops.via, gradient.stops.to]
                    .iter()
                    .flatten()
                    .all(|stop| stop.3.is_finite())
            })
    }
    /// Create a new StyleBuilder with default values
    ///
    /// # Returns
    /// A new `StyleBuilder` instance with default styling
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserve cell space for an outline painted by a widget, independently of padding.
    pub(crate) fn cell_border(mut self, width: f32) -> Self {
        self.style.border = taffy::geometry::Rect::length(width);
        self
    }

    /// Set display mode to flex layout
    ///
    /// # Returns
    /// Self for method chaining
    pub fn display_flex(mut self) -> Self {
        self.style.display = Display::Flex;
        self
    }

    /// Set display mode to grid layout
    ///
    /// # Returns
    /// Self for method chaining
    pub fn display_grid(mut self) -> Self {
        self.style.display = Display::Grid;
        self
    }

    /// Set flex direction for layout
    ///
    /// # Arguments
    /// * `dir` - The direction for flex layout (Row, Column, RowReverse, ColumnReverse)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn direction(mut self, dir: Direction) -> Self {
        self.style.flex_direction = match dir {
            Direction::Row => FlexDirection::Row,
            Direction::Column => FlexDirection::Column,
            Direction::RowReverse => FlexDirection::RowReverse,
            Direction::ColumnReverse => FlexDirection::ColumnReverse,
        };
        self
    }

    /// Set width and height in pixels
    ///
    /// # Arguments
    /// * `w` - Optional width in pixels
    /// * `h` - Optional height in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn size_px(mut self, w: Option<f32>, h: Option<f32>) -> Self {
        if let Some(w) = w {
            self.style.size.width = Dimension::length(w);
        }
        if let Some(h) = h {
            self.style.size.height = Dimension::length(h);
        }
        self
    }

    /// Set width as a percentage of parent
    ///
    /// # Arguments
    /// * `pct` - Width percentage (0-100)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn width_pct(mut self, pct: f32) -> Self {
        self.style.size.width = Dimension::percent(pct / 100.0);
        self
    }

    /// Set height as a percentage of parent
    ///
    /// # Arguments
    /// * `pct` - Height percentage (0-100)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn height_pct(mut self, pct: f32) -> Self {
        self.style.size.height = Dimension::percent(pct / 100.0);
        self
    }

    /// Set width in pixels
    ///
    /// # Arguments
    /// * `px` - Width in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn width_px(mut self, px: f32) -> Self {
        self.style.size.width = Dimension::length(px);
        self
    }

    /// Set height in pixels
    ///
    /// # Arguments
    /// * `px` - Height in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn height_px(mut self, px: f32) -> Self {
        self.style.size.height = Dimension::length(px);
        self
    }

    /// Set width as a percentage of parent
    pub fn width_percent(mut self, pct: f32) -> Self {
        self.style.size.width = Dimension::percent(pct / 100.0);
        self
    }
    /// Set height as a percentage of parent
    pub fn height_percent(mut self, pct: f32) -> Self {
        self.style.size.height = Dimension::percent(pct / 100.0);
        self
    }

    /// Set width to auto (content-based sizing)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn width_auto(mut self) -> Self {
        self.style.size.width = Dimension::auto();
        self
    }

    /// Set height to auto (content-based sizing)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn height_auto(mut self) -> Self {
        self.style.size.height = Dimension::auto();
        self
    }

    /// Set minimum width and height in pixels
    ///
    /// # Arguments
    /// * `w` - Optional minimum width in pixels
    /// * `h` - Optional minimum height in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn min_size_px(mut self, w: Option<f32>, h: Option<f32>) -> Self {
        if let Some(w) = w {
            self.style.min_size.width = Dimension::length(w);
        }
        if let Some(h) = h {
            self.style.min_size.height = Dimension::length(h);
        }
        self
    }

    /// Set maximum width and height in pixels
    ///
    /// # Arguments
    /// * `w` - Optional maximum width in pixels
    /// * `h` - Optional maximum height in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn max_size_px(mut self, w: Option<f32>, h: Option<f32>) -> Self {
        if let Some(w) = w {
            self.style.max_size.width = Dimension::length(w);
        }
        if let Some(h) = h {
            self.style.max_size.height = Dimension::length(h);
        }
        self
    }

    /// Set minimum width in pixels
    pub fn min_width_px(mut self, px: f32) -> Self {
        self.style.min_size.width = Dimension::length(px);
        self
    }
    /// Set minimum height in pixels
    pub fn min_height_px(mut self, px: f32) -> Self {
        self.style.min_size.height = Dimension::length(px);
        self
    }
    /// Set maximum width in pixels
    pub fn max_width_px(mut self, px: f32) -> Self {
        self.style.max_size.width = Dimension::length(px);
        self
    }
    /// Set maximum height in pixels
    pub fn max_height_px(mut self, px: f32) -> Self {
        self.style.max_size.height = Dimension::length(px);
        self
    }

    /// Set minimum width as a percentage of parent
    pub fn min_width_percent(mut self, pct: f32) -> Self {
        self.style.min_size.width = Dimension::percent(pct / 100.0);
        self
    }
    /// Set minimum height as a percentage of parent
    pub fn min_height_percent(mut self, pct: f32) -> Self {
        self.style.min_size.height = Dimension::percent(pct / 100.0);
        self
    }
    /// Set maximum width as a percentage of parent
    pub fn max_width_percent(mut self, pct: f32) -> Self {
        self.style.max_size.width = Dimension::percent(pct / 100.0);
        self
    }
    /// Set maximum height as a percentage of parent
    pub fn max_height_percent(mut self, pct: f32) -> Self {
        self.style.max_size.height = Dimension::percent(pct / 100.0);
        self
    }

    /// Set minimum width to auto
    pub fn min_width_auto(mut self) -> Self {
        self.style.min_size.width = Dimension::auto();
        self
    }
    /// Set minimum height to auto
    pub fn min_height_auto(mut self) -> Self {
        self.style.min_size.height = Dimension::auto();
        self
    }
    /// Set maximum width to auto
    pub fn max_width_auto(mut self) -> Self {
        self.style.max_size.width = Dimension::auto();
        self
    }
    /// Set maximum height to auto
    pub fn max_height_auto(mut self) -> Self {
        self.style.max_size.height = Dimension::auto();
        self
    }

    /// Set flex grow factor
    ///
    /// Controls how much the item should grow relative to other flex items.
    ///
    /// # Arguments
    /// * `v` - Flex grow factor (0.0 = no growth, 1.0 = normal growth)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn flex_grow(mut self, v: f32) -> Self {
        self.style.flex_grow = v;
        self
    }

    /// Set flex shrink factor
    ///
    /// Controls how much the item should shrink relative to other flex items.
    ///
    /// # Arguments
    /// * `v` - Flex shrink factor (0.0 = no shrinking, 1.0 = normal shrinking)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn flex_shrink(mut self, v: f32) -> Self {
        self.style.flex_shrink = v;
        self
    }

    /// Set whether flex items should wrap to new lines
    ///
    /// # Arguments
    /// * `wrap` - true to allow wrapping, false to keep items on one line
    ///
    /// # Returns
    /// Self for method chaining
    pub fn flex_wrap(mut self, wrap: bool) -> Self {
        use taffy::FlexWrap;
        self.style.flex_wrap = if wrap {
            FlexWrap::Wrap
        } else {
            FlexWrap::NoWrap
        };
        self
    }

    /// Set padding on all sides in pixels
    ///
    /// # Arguments
    /// * `v` - Padding value in pixels for all sides
    ///
    /// # Returns
    /// Self for method chaining
    pub fn padding_all_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding = Rect {
            left: lp,
            right: lp,
            top: lp,
            bottom: lp,
        };
        self.pad_l = Some(v);
        self.pad_r = Some(v);
        self.pad_t = Some(v);
        self.pad_b = Some(v);
        self
    }

    /// Set horizontal padding (left and right) in pixels
    ///
    /// # Arguments
    /// * `v` - Padding value in pixels for left and right sides
    ///
    /// # Returns
    /// Self for method chaining
    pub fn padding_x_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.left = lp;
        self.style.padding.right = lp;
        self.pad_l = Some(v);
        self.pad_r = Some(v);
        self
    }

    /// Set vertical padding (top and bottom) in pixels
    ///
    /// # Arguments
    /// * `v` - Padding value in pixels for top and bottom sides
    ///
    /// # Returns
    /// Self for method chaining
    pub fn padding_y_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.top = lp;
        self.style.padding.bottom = lp;
        self.pad_t = Some(v);
        self.pad_b = Some(v);
        self
    }
    /// Set left padding in pixels
    ///
    /// # Arguments
    /// * `v` - Left padding value in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn padding_l_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.left = lp;
        self.pad_l = Some(v);
        self
    }

    /// Set right padding in pixels
    ///
    /// # Arguments
    /// * `v` - Right padding value in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn padding_r_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.right = lp;
        self.pad_r = Some(v);
        self
    }

    /// Set top padding in pixels
    ///
    /// # Arguments
    /// * `v` - Top padding value in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn padding_t_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.top = lp;
        self.pad_t = Some(v);
        self
    }

    /// Set bottom padding in pixels
    ///
    /// # Arguments
    /// * `v` - Bottom padding value in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn padding_b_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.bottom = lp;
        self.pad_b = Some(v);
        self
    }

    /// Set margin on all sides in pixels
    ///
    /// # Arguments
    /// * `v` - Margin value in pixels for all sides
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_all_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin = Rect {
            left: lp,
            right: lp,
            top: lp,
            bottom: lp,
        };
        self.mar_l = Some(v);
        self.mar_r = Some(v);
        self.mar_t = Some(v);
        self.mar_b = Some(v);
        self
    }

    /// Set horizontal margin (left and right) in pixels
    ///
    /// # Arguments
    /// * `v` - Margin value in pixels for left and right sides
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_x_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.left = lp;
        self.style.margin.right = lp;
        self.mar_l = Some(v);
        self.mar_r = Some(v);
        self
    }

    /// Set vertical margin (top and bottom) in pixels
    ///
    /// # Arguments
    /// * `v` - Margin value in pixels for top and bottom sides
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_y_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.top = lp;
        self.style.margin.bottom = lp;
        self.mar_t = Some(v);
        self.mar_b = Some(v);
        self
    }
    /// Set left margin in pixels
    ///
    /// # Arguments
    /// * `v` - Left margin value in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_l_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.left = lp;
        self.mar_l = Some(v);
        self
    }

    /// Set right margin in pixels
    ///
    /// # Arguments
    /// * `v` - Right margin value in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_r_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.right = lp;
        self.mar_r = Some(v);
        self
    }

    /// Set top margin in pixels
    ///
    /// # Arguments
    /// * `v` - Top margin value in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_t_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.top = lp;
        self.mar_t = Some(v);
        self
    }

    /// Set bottom margin in pixels
    ///
    /// # Arguments
    /// * `v` - Bottom margin value in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_b_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.bottom = lp;
        self.mar_b = Some(v);
        self
    }

    /// Set all margins to auto for centering
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_auto(mut self) -> Self {
        self.style.margin.left = LengthPercentageAuto::auto();
        self.style.margin.right = LengthPercentageAuto::auto();
        self.style.margin.top = LengthPercentageAuto::auto();
        self.style.margin.bottom = LengthPercentageAuto::auto();
        self
    }

    /// Set horizontal margins to auto for horizontal centering
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_x_auto(mut self) -> Self {
        self.style.margin.left = LengthPercentageAuto::auto();
        self.style.margin.right = LengthPercentageAuto::auto();
        self
    }

    /// Set vertical margins to auto for vertical centering
    ///
    /// # Returns
    /// Self for method chaining
    pub fn margin_y_auto(mut self) -> Self {
        self.style.margin.top = LengthPercentageAuto::auto();
        self.style.margin.bottom = LengthPercentageAuto::auto();
        self
    }

    /// Set gap between flex/grid items in pixels
    ///
    /// # Arguments
    /// * `x` - Horizontal gap in pixels
    /// * `y` - Vertical gap in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn gap_px(mut self, x: f32, y: f32) -> Self {
        self.style.gap = Size {
            width: LengthPercentage::length(x),
            height: LengthPercentage::length(y),
        };
        self
    }

    /// Set text color with RGBA values
    pub fn text_rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.fg_rgba = Some((r, g, b, a));
        self
    }

    /// Set foreground color with RGBA values (alias for text_rgba)
    pub fn fg_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.text_rgba(r, g, b, a)
    }

    /// Set background color with RGBA values
    pub fn bg_rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.bg_rgba = Some((r, g, b, a));
        self
    }
    /// Set bold text style
    pub fn bold(mut self, v: bool) -> Self {
        self.bold = v;
        self.text.bold = Some(v);
        self
    }

    /// Set italic text style
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = v;
        self.text.italic = Some(v);
        self
    }

    /// Set underline text style
    pub fn underline(mut self, v: bool) -> Self {
        self.underline = v;
        self.text.underline = Some(v);
        self
    }

    /// Set reverse text style
    pub fn reverse(mut self, v: bool) -> Self {
        self.reverse = v;
        self
    }

    /// Set strikethrough text style
    pub fn strike(mut self, v: bool) -> Self {
        self.strike = v;
        self.text.strike = Some(v);
        self
    }

    /// Whether the text uses strikethrough, independently of legacy decorations.
    pub fn get_strike(&self) -> bool {
        self.strike
    }

    // Font weight methods for CSS utility compatibility
    /// Set font weight to light (removes bold)
    pub fn font_weight_light(self) -> Self {
        self.bold(false)
    }
    /// Set font weight to normal (removes bold)
    pub fn font_weight_normal(self) -> Self {
        self.bold(false)
    }
    /// Set font weight to medium (normal in terminals)
    pub fn font_weight_medium(self) -> Self {
        self.bold(false)
    }
    /// Set font weight to bold
    pub fn font_weight_bold(self) -> Self {
        self.bold(true)
    }
    /// Set font weight to black (bold in terminals)
    pub fn font_weight_black(self) -> Self {
        self.bold(true)
    }

    /// Set opacity/alpha value (0.0-1.0)
    pub fn opacity(mut self, alpha: f32) -> Self {
        self.opacity = Some(alpha.clamp(0.0, 1.0));
        self
    }

    /// Set z-index for layering
    pub fn z_index(mut self, z: i32) -> Self {
        self.z_index = Some(z);
        self
    }

    /// Check if background color is set
    pub fn has_bg_color(&self) -> bool {
        self.bg_rgba.is_some()
    }

    /// Get z-index value if set
    pub fn get_z_index(&self) -> Option<i32> {
        self.z_index
    }

    /// Get horizontal overflow setting
    pub fn get_overflow_x(&self) -> Overflow {
        self.style.overflow.x
    }

    /// Get vertical overflow setting
    pub fn get_overflow_y(&self) -> Overflow {
        self.style.overflow.y
    }

    /// Take visual style properties, leaving None
    pub fn take_visuals(&mut self) -> Option<VisualStyle> {
        let had_fg = self.fg_rgba.is_some();
        let had_bg = self.bg_rgba.is_some();
        let has_opacity = self.opacity.is_some();
        let any = had_fg
            || had_bg
            || has_opacity
            || self.bold
            || self.italic
            || self.underline
            || self.reverse
            || self.strike;
        if !any {
            return None;
        }
        let mut fg = self.fg_rgba.take().unwrap_or((1.0, 1.0, 1.0, 1.0));
        let mut bg = self.bg_rgba.take().unwrap_or((0.0, 0.0, 0.0, 1.0));

        // Apply opacity if set
        if let Some(opacity) = self.opacity.take() {
            fg.3 *= opacity;
            bg.3 *= opacity;
        }

        let decorations = TextDecorations {
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            reverse: self.reverse,
        };

        Some(VisualStyle {
            fg: crate::core::surface::Rgba {
                r: fg.0,
                g: fg.1,
                b: fg.2,
                a: fg.3,
            },
            bg: crate::core::surface::Rgba {
                r: bg.0,
                g: bg.1,
                b: bg.2,
                a: bg.3,
            },
            decorations,
        })
    }

    /// Get cached padding values as BoxSpacing
    pub fn pad_cache(&self) -> BoxSpacing {
        BoxSpacing {
            left: self.pad_l.unwrap_or(0.0),
            right: self.pad_r.unwrap_or(0.0),
            top: self.pad_t.unwrap_or(0.0),
            bottom: self.pad_b.unwrap_or(0.0),
        }
    }

    /// Get cached margin values as BoxSpacing
    pub fn mar_cache(&self) -> BoxSpacing {
        BoxSpacing {
            left: self.mar_l.unwrap_or(0.0),
            right: self.mar_r.unwrap_or(0.0),
            top: self.mar_t.unwrap_or(0.0),
            bottom: self.mar_b.unwrap_or(0.0),
        }
    }

    /// Set number of grid columns
    pub fn grid_cols(mut self, n: u16) -> Self {
        self.grid_cols = Some(n);
        self
    }

    /// Set number of grid rows
    pub fn grid_rows(mut self, n: u16) -> Self {
        self.grid_rows = Some(n);
        self
    }

    /// Set grid auto flow direction
    pub fn grid_auto_flow(mut self, f: GridAutoFlow) -> Self {
        self.grid_auto_flow = Some(f);
        self
    }

    /// Set column span for grid items
    pub fn col_span(mut self, n: u16) -> Self {
        self.col_span = Some(n);
        self
    }

    /// Set row span for grid items
    pub fn row_span(mut self, n: u16) -> Self {
        self.row_span = Some(n);
        self
    }

    /// Set grid column start position
    pub fn col_start(mut self, n: i16) -> Self {
        self.col_start = Some(n);
        self
    }

    /// Set grid column end position
    pub fn col_end(mut self, n: i16) -> Self {
        self.col_end = Some(n);
        self
    }

    /// Set grid row start position
    pub fn row_start(mut self, n: i16) -> Self {
        self.row_start = Some(n);
        self
    }

    /// Set grid row end position
    pub fn row_end(mut self, n: i16) -> Self {
        self.row_end = Some(n);
        self
    }

    // Grid template methods for CSS utility compatibility
    /// Set grid template columns (alias for grid_cols)
    pub fn grid_template_columns(self, cols: u16) -> Self {
        self.grid_cols(cols)
    }

    /// Set grid template rows (alias for grid_rows)
    pub fn grid_template_rows(self, rows: u16) -> Self {
        self.grid_rows(rows)
    }

    /// Set grid column span (alias for col_span)
    pub fn grid_column_span(self, span: u16) -> Self {
        self.col_span(span)
    }

    /// Set grid row span (alias for row_span)
    pub fn grid_row_span(self, span: u16) -> Self {
        self.row_span(span)
    }

    // Grid auto methods
    /// Set grid column to auto placement
    pub fn grid_column_auto(mut self) -> Self {
        self.style.grid_column = Line {
            start: GridPlacement::Auto,
            end: GridPlacement::Auto,
        };
        self
    }

    /// Set grid row to auto placement
    pub fn grid_row_auto(mut self) -> Self {
        self.style.grid_row = Line {
            start: GridPlacement::Auto,
            end: GridPlacement::Auto,
        };
        self
    }

    /// Set grid column start position (alias for col_start)
    pub fn grid_column_start(self, start: i16) -> Self {
        self.col_start(start)
    }

    /// Set grid column end position (alias for col_end)
    pub fn grid_column_end(self, end: i16) -> Self {
        self.col_end(end)
    }

    /// Set grid row start position (alias for row_start)
    pub fn grid_row_start(self, start: i16) -> Self {
        self.row_start(start)
    }

    // Advanced grid utilities
    /// Set grid columns to auto-fit with minimum size
    pub fn grid_auto_fit_columns(mut self, min_size: u16) -> Self {
        // In TUI, we simulate auto-fit by setting a flexible grid
        // This would need special handling in the layout system
        self.grid_cols = Some(min_size.max(1));
        self
    }

    /// Set grid columns to auto-fill with minimum size
    pub fn grid_auto_fill_columns(mut self, min_size: u16) -> Self {
        // Similar to auto-fit but fills available space
        self.grid_cols = Some(min_size.max(1));
        self
    }

    /// Set grid rows to auto-fit with minimum size
    pub fn grid_auto_fit_rows(mut self, min_size: u16) -> Self {
        self.grid_rows = Some(min_size.max(1));
        self
    }

    /// Set grid rows to auto-fill with minimum size
    pub fn grid_auto_fill_rows(mut self, min_size: u16) -> Self {
        self.grid_rows = Some(min_size.max(1));
        self
    }

    /// Set grid area by name (header, sidebar, main, footer, or custom)
    /// Set grid area name for template-based layouts
    pub fn grid_area(self, area_name: &str) -> Self {
        // Store grid area name for template areas in reactive-tui's advanced grid system
        // This integrates with the hierarchical window system for named grid layouts
        match area_name {
            "header" => self.grid_row_start(1).grid_column_span(12),
            "sidebar" => self.grid_column_start(1).grid_row_span(3),
            "main" => self.grid_column_start(2).grid_column_span(10),
            "footer" => self.grid_row_start(-1).grid_column_span(12),
            _ => {
                // Custom area - use as CSS grid area identifier
                // In a full implementation, this would be stored for layout resolution
                self
            }
        }
    }

    /// Set grid template areas for complex layouts
    pub fn grid_template_areas(mut self, areas: &[&str]) -> Self {
        // Store template areas for named grid layouts in reactive-tui's advanced grid system
        // This integrates with the hierarchical window system for complex layouts

        // Set up grid based on template areas
        let rows = areas.len() as u16;
        let cols = areas
            .first()
            .map(|row| row.split_whitespace().count() as u16)
            .unwrap_or(1);

        self.grid_rows = Some(rows);
        self.grid_cols = Some(cols);

        // In a full implementation, this would store the area names for layout resolution
        // For now, we set up the basic grid structure
        self.display_grid()
    }

    /// Set grid row end position
    pub fn grid_row_end(self, end: i16) -> Self {
        self.row_end(end)
    }

    /// Set alignment of this item within its container
    pub fn align_self(mut self, a: AlignSelf) -> Self {
        self.style.align_self = Some(match a {
            AlignSelf::Auto => TAlign::Stretch, // Taffy lacks Auto in 0.9; use Stretch as default
            AlignSelf::Start => TAlign::FlexStart,
            AlignSelf::Center => TAlign::Center,
            AlignSelf::End => TAlign::FlexEnd,
            AlignSelf::Stretch => TAlign::Stretch,
        });
        self
    }

    /// Set justification of content along main axis
    pub fn justify(mut self, j: JustifyContent) -> Self {
        self.style.justify_content = Some(match j {
            JustifyContent::Start => TJustify::FlexStart,
            JustifyContent::Center => TJustify::Center,
            JustifyContent::End => TJustify::FlexEnd,
            JustifyContent::SpaceBetween => TJustify::SpaceBetween,
            JustifyContent::SpaceAround => TJustify::SpaceAround,
            JustifyContent::SpaceEvenly => TJustify::SpaceEvenly,
        });
        self
    }

    // Alias methods for CSS utility compatibility
    /// Set justification of content (alias for justify)
    pub fn justify_content(self, j: JustifyContent) -> Self {
        self.justify(j)
    }

    /// Set alignment of items (alias for align)
    pub fn align_items(self, a: AlignItems) -> Self {
        self.align(a)
    }

    // Overflow utilities (critical for TUI)
    /// Set overflow to hidden for both axes
    pub fn overflow_hidden(mut self) -> Self {
        self.style.overflow.x = Overflow::Hidden;
        self.style.overflow.y = Overflow::Hidden;
        self
    }

    /// Set overflow to scroll for both axes
    pub fn overflow_scroll(mut self) -> Self {
        self.style.overflow.x = Overflow::Scroll;
        self.style.overflow.y = Overflow::Scroll;
        self
    }

    /// Set overflow to auto (scroll when needed)
    pub fn overflow_auto(mut self) -> Self {
        self.style.overflow.x = Overflow::Scroll; // TUI doesn't have "auto", use scroll
        self.style.overflow.y = Overflow::Scroll;
        self
    }

    /// Set overflow to visible for both axes
    pub fn overflow_visible(mut self) -> Self {
        self.style.overflow.x = Overflow::Visible;
        self.style.overflow.y = Overflow::Visible;
        self
    }

    // X-axis overflow
    /// Set horizontal overflow to hidden
    pub fn overflow_x_hidden(mut self) -> Self {
        self.style.overflow.x = Overflow::Hidden;
        self
    }

    /// Set horizontal overflow to scroll
    pub fn overflow_x_scroll(mut self) -> Self {
        self.style.overflow.x = Overflow::Scroll;
        self
    }

    /// Set horizontal overflow to auto (scroll when needed)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn overflow_x_auto(mut self) -> Self {
        self.style.overflow.x = Overflow::Scroll;
        self
    }

    /// Set horizontal overflow to visible (content can overflow)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn overflow_x_visible(mut self) -> Self {
        self.style.overflow.x = Overflow::Visible;
        self
    }

    /// Set vertical overflow to hidden (clip overflowing content)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn overflow_y_hidden(mut self) -> Self {
        self.style.overflow.y = Overflow::Hidden;
        self
    }

    /// Set vertical overflow to scroll (always show scrollbar)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn overflow_y_scroll(mut self) -> Self {
        self.style.overflow.y = Overflow::Scroll;
        self
    }

    /// Set vertical overflow to auto (scroll when needed)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn overflow_y_auto(mut self) -> Self {
        self.style.overflow.y = Overflow::Scroll;
        self
    }

    /// Set vertical overflow to visible (content can overflow)
    pub fn overflow_y_visible(mut self) -> Self {
        self.style.overflow.y = Overflow::Visible;
        self
    }

    /// Set alignment of items along the cross axis
    pub fn align(mut self, a: AlignItems) -> Self {
        self.style.align_items = Some(match a {
            AlignItems::Start => TAlign::FlexStart,
            AlignItems::Center => TAlign::Center,
            AlignItems::End => TAlign::FlexEnd,
            AlignItems::Stretch => TAlign::Stretch,
            AlignItems::Baseline => TAlign::Baseline,
        });
        self
    }

    /// Set alignment of content along the cross axis
    pub fn align_content(mut self, a: JustifyContent) -> Self {
        // Map subset to AlignContent. This is a simplification (we reuse JustifyContent enum for brevity)
        self.style.align_content = Some(match a {
            JustifyContent::Start => TAlignContent::FlexStart,
            JustifyContent::Center => TAlignContent::Center,
            JustifyContent::End => TAlignContent::FlexEnd,
            JustifyContent::SpaceBetween => TAlignContent::SpaceBetween,
            JustifyContent::SpaceAround => TAlignContent::SpaceAround,
            JustifyContent::SpaceEvenly => TAlignContent::SpaceEvenly,
        });
        self
    }

    /// Set overflow to clip (hidden) for both axes
    pub fn overflow_clip(mut self) -> Self {
        self.style.overflow = Point {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        };
        self
    }

    /// Set aspect ratio constraint
    pub fn aspect_ratio(mut self, ratio: f32) -> Self {
        self.style.aspect_ratio = Some(ratio);
        self
    }

    // Advanced positioning utilities
    /// Set position to static (relative in TUI)
    pub fn position_static(mut self) -> Self {
        self.style.position = taffy::style::Position::Relative; // TUI equivalent
        self.z_index(0)
    }

    /// Set position to relative
    pub fn position_relative(mut self) -> Self {
        self.style.position = taffy::style::Position::Relative;
        self.z_index(1)
    }

    /// Set position to absolute
    pub fn position_absolute(mut self) -> Self {
        self.style.position = taffy::style::Position::Absolute;
        self.z_index(10)
    }

    /// Set position to fixed (absolute in TUI)
    pub fn position_fixed(mut self) -> Self {
        self.style.position = taffy::style::Position::Absolute; // TUI equivalent
        self.z_index(50)
    }

    /// Set position to sticky (positioned based on scroll position)
    pub fn position_sticky(mut self) -> Self {
        self.style.position = taffy::style::Position::Relative; // TUI equivalent
        self.z_index(20)
    }

    // Inset utilities
    /// Set all inset values to the same length
    pub fn inset_all(mut self, value: f32) -> Self {
        self.style.inset.left = LengthPercentageAuto::length(value);
        self.style.inset.right = LengthPercentageAuto::length(value);
        self.style.inset.top = LengthPercentageAuto::length(value);
        self.style.inset.bottom = LengthPercentageAuto::length(value);
        self
    }

    /// Set all inset values to auto
    pub fn inset_all_auto(mut self) -> Self {
        self.style.inset.left = LengthPercentageAuto::auto();
        self.style.inset.right = LengthPercentageAuto::auto();
        self.style.inset.top = LengthPercentageAuto::auto();
        self.style.inset.bottom = LengthPercentageAuto::auto();
        self
    }

    /// Set top inset value
    pub fn inset_top(mut self, value: f32) -> Self {
        self.style.inset.top = LengthPercentageAuto::length(value);
        self
    }

    /// Set right inset value
    pub fn inset_right(mut self, value: f32) -> Self {
        self.style.inset.right = LengthPercentageAuto::length(value);
        self
    }

    /// Set bottom inset value
    pub fn inset_bottom(mut self, value: f32) -> Self {
        self.style.inset.bottom = LengthPercentageAuto::length(value);
        self
    }

    /// Set left inset value
    pub fn inset_left(mut self, value: f32) -> Self {
        self.style.inset.left = LengthPercentageAuto::length(value);
        self
    }

    /// Build the final Style object
    pub fn build(mut self) -> Style {
        if self
            .accessibility
            .get("sr-only")
            .is_some_and(|value| value.as_deref() == Some("true"))
        {
            self.style.size.width = Dimension::length(0.0);
            self.style.size.height = Dimension::length(0.0);
        }
        // Wire grid templates to equal-fr tracks when counts are set
        if let Some(cols) = self.grid_cols {
            self.style.display = Display::Grid;
            self.style.grid_template_columns = (0..cols)
                .map(|_| GridTemplateComponent::from(TrackSizingFunction::from_fr(1.0)))
                .collect();
        }
        if let Some(rows) = self.grid_rows {
            self.style.display = Display::Grid;
            self.style.grid_template_rows = (0..rows)
                .map(|_| GridTemplateComponent::from(TrackSizingFunction::from_fr(1.0)))
                .collect();
        }
        if let Some(flow) = self.grid_auto_flow {
            self.style.grid_auto_flow = flow.to_taffy();
        }
        // Map placement to grid_row/grid_column where possible
        if let Some(span) = self.col_span {
            self.style.grid_column = Line {
                start: GridPlacement::from_span(span),
                end: GridPlacement::Auto,
            };
        }
        if let Some(span) = self.row_span {
            self.style.grid_row = Line {
                start: GridPlacement::from_span(span),
                end: GridPlacement::Auto,
            };
        }
        if let Some(s) = self.col_start {
            self.style.grid_column = Line {
                start: GridPlacement::from_line_index(s),
                end: self.style.grid_column.end,
            };
        }
        if let Some(e) = self.col_end {
            self.style.grid_column = Line {
                start: self.style.grid_column.start,
                end: GridPlacement::from_line_index(e),
            };
        }
        if let Some(s) = self.row_start {
            self.style.grid_row = Line {
                start: GridPlacement::from_line_index(s),
                end: self.style.grid_row.end,
            };
        }
        if let Some(e) = self.row_end {
            self.style.grid_row = Line {
                start: self.style.grid_row.start,
                end: GridPlacement::from_line_index(e),
            };
        }
        self.style
    }

    /// Set CSS animation for this element
    ///
    /// This marks the element as having a CSS animation that should be
    /// applied when the element is rendered.
    ///
    /// # Arguments
    /// * `animation_name` - Name of the CSS animation (e.g., "pulse", "bounce")
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_css_animation(mut self, animation_name: &str) -> Self {
        self.css_animation = Some(animation_name.to_string());
        self
    }

    /// Get the CSS animation name if set
    pub fn get_css_animation(&self) -> Option<&String> {
        self.css_animation.as_ref()
    }

    /// Take the CSS animation name, leaving None
    pub fn take_css_animation(&mut self) -> Option<String> {
        self.css_animation.take()
    }
}

/// Owned style data. Taffy's tagged layout values remain local to each thread.
/// Invalid numeric styles report an error when App prepares the frame.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleSnapshot(std::result::Result<Vec<u8>, String>, bool);

impl StyleSnapshot {
    pub(crate) fn resolve_viewport(&self, width: u16) -> crate::error::Result<Option<Self>> {
        if self.1 {
            Ok(Some(self.restore()?.at_width(width).snapshot()))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn restore(&self) -> crate::error::Result<StyleBuilder> {
        let bytes = self.0.as_ref().map_err(|error| {
            crate::error::ReactiveError::layout(format!("Cannot capture explicit styles: {error}"))
        })?;
        serde_json::from_slice(bytes).map_err(|error| {
            crate::error::ReactiveError::layout(format!("Invalid explicit styles: {error}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use taffy::style::Display;

    #[test]
    fn test_text_decorations_default() {
        let decorations = TextDecorations::default();
        assert!(!decorations.bold);
        assert!(!decorations.italic);
        assert!(!decorations.underline);
        assert!(!decorations.reverse);
    }

    #[test]
    fn test_text_decorations_creation() {
        let decorations = TextDecorations {
            bold: true,
            italic: false,
            underline: true,
            reverse: false,
        };
        assert!(decorations.bold);
        assert!(!decorations.italic);
        assert!(decorations.underline);
        assert!(!decorations.reverse);
    }

    #[test]
    fn test_visual_style_default() {
        let style = VisualStyle::default();
        assert_eq!(style.fg, RgbaColor::default());
        assert_eq!(style.bg, RgbaColor::default());
        assert_eq!(style.decorations, TextDecorations::default());
    }

    #[test]
    fn test_visual_style_creation() {
        let fg = RgbaColor {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let bg = RgbaColor {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        };
        let decorations = TextDecorations {
            bold: true,
            italic: true,
            underline: false,
            reverse: false,
        };

        let style = VisualStyle {
            fg,
            bg,
            decorations,
        };
        assert_eq!(style.fg, fg);
        assert_eq!(style.bg, bg);
        assert_eq!(style.decorations, decorations);
    }

    #[test]
    fn test_box_spacing_new() {
        let spacing = BoxSpacing::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(spacing.left, 1.0);
        assert_eq!(spacing.right, 2.0);
        assert_eq!(spacing.top, 3.0);
        assert_eq!(spacing.bottom, 4.0);
    }

    #[test]
    fn test_box_spacing_uniform() {
        let spacing = BoxSpacing::uniform(5.0);
        assert_eq!(spacing.left, 5.0);
        assert_eq!(spacing.right, 5.0);
        assert_eq!(spacing.top, 5.0);
        assert_eq!(spacing.bottom, 5.0);
    }

    #[test]
    fn test_box_spacing_symmetric() {
        let spacing = BoxSpacing::symmetric(10.0, 20.0);
        assert_eq!(spacing.left, 10.0);
        assert_eq!(spacing.right, 10.0);
        assert_eq!(spacing.top, 20.0);
        assert_eq!(spacing.bottom, 20.0);
    }

    #[test]
    fn test_box_spacing_calculations() {
        let spacing = BoxSpacing::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(spacing.horizontal(), 3.0); // left + right
        assert_eq!(spacing.vertical(), 7.0); // top + bottom
    }

    #[test]
    fn test_direction_enum() {
        // Test that all direction variants exist and are different
        assert_ne!(Direction::Row, Direction::Column);
        assert_ne!(Direction::Row, Direction::RowReverse);
        assert_ne!(Direction::Column, Direction::ColumnReverse);
    }

    #[test]
    fn test_justify_content_enum() {
        // Test that all justify content variants exist
        let variants = [
            JustifyContent::Start,
            JustifyContent::Center,
            JustifyContent::End,
            JustifyContent::SpaceBetween,
            JustifyContent::SpaceAround,
            JustifyContent::SpaceEvenly,
        ];

        // All should be different
        for (i, variant1) in variants.iter().enumerate() {
            for (j, variant2) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(variant1, variant2);
                }
            }
        }
    }

    #[test]
    fn test_align_items_enum() {
        let variants = [
            AlignItems::Start,
            AlignItems::Center,
            AlignItems::End,
            AlignItems::Stretch,
            AlignItems::Baseline,
        ];

        // All should be different
        for (i, variant1) in variants.iter().enumerate() {
            for (j, variant2) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(variant1, variant2);
                }
            }
        }
    }

    #[test]
    fn test_grid_auto_flow_to_taffy() {
        assert_eq!(GridAutoFlow::Row.to_taffy(), TGridAutoFlow::Row);
        assert_eq!(GridAutoFlow::Column.to_taffy(), TGridAutoFlow::Column);
        assert_eq!(GridAutoFlow::RowDense.to_taffy(), TGridAutoFlow::RowDense);
        assert_eq!(
            GridAutoFlow::ColumnDense.to_taffy(),
            TGridAutoFlow::ColumnDense
        );
    }

    #[test]
    fn test_style_builder_new() {
        let builder = StyleBuilder::new();
        assert_eq!(builder.style.display, Display::Flex); // Default display
        assert!(!builder.bold);
        assert!(!builder.italic);
        assert!(!builder.underline);
        assert!(!builder.reverse);
        assert!(!builder.strike);
    }

    #[test]
    fn test_style_builder_display() {
        let style = StyleBuilder::new().display_flex().build();
        assert_eq!(style.display, Display::Flex);

        let style = StyleBuilder::new().display_grid().build();
        assert_eq!(style.display, Display::Grid);
    }

    #[test]
    fn test_style_builder_direction() {
        let style = StyleBuilder::new().direction(Direction::Row).build();
        assert_eq!(style.flex_direction, FlexDirection::Row);

        let style = StyleBuilder::new().direction(Direction::Column).build();
        assert_eq!(style.flex_direction, FlexDirection::Column);

        let style = StyleBuilder::new().direction(Direction::RowReverse).build();
        assert_eq!(style.flex_direction, FlexDirection::RowReverse);

        let style = StyleBuilder::new()
            .direction(Direction::ColumnReverse)
            .build();
        assert_eq!(style.flex_direction, FlexDirection::ColumnReverse);
    }

    #[test]
    fn test_style_builder_size_px() {
        let style = StyleBuilder::new()
            .size_px(Some(100.0), Some(200.0))
            .build();
        assert_eq!(style.size.width, Dimension::length(100.0));
        assert_eq!(style.size.height, Dimension::length(200.0));

        // Test partial sizing
        let style = StyleBuilder::new().size_px(Some(50.0), None).build();
        assert_eq!(style.size.width, Dimension::length(50.0));
        assert_eq!(style.size.height, Dimension::auto()); // Should remain default
    }

    #[test]
    fn test_style_builder_grid_setup() {
        let style = StyleBuilder::new().grid_cols(3).grid_rows(2).build();

        assert_eq!(style.display, Display::Grid);
        assert_eq!(style.grid_template_columns.len(), 3);
        assert_eq!(style.grid_template_rows.len(), 2);
    }

    #[test]
    fn test_style_builder_grid_placement() {
        let style = StyleBuilder::new().col_span(2).row_span(3).build();

        // Grid placement should be set
        assert!(matches!(style.grid_column.start, GridPlacement::Span(_)));
        assert!(matches!(style.grid_row.start, GridPlacement::Span(_)));
    }

    #[test]
    fn test_style_builder_grid_positioning() {
        let style = StyleBuilder::new()
            .col_start(1)
            .col_end(3)
            .row_start(2)
            .row_end(4)
            .build();

        // Grid positioning should be set
        assert!(matches!(style.grid_column.start, GridPlacement::Line(_)));
        assert!(matches!(style.grid_column.end, GridPlacement::Line(_)));
        assert!(matches!(style.grid_row.start, GridPlacement::Line(_)));
        assert!(matches!(style.grid_row.end, GridPlacement::Line(_)));
    }

    #[test]
    fn test_style_builder_chaining() {
        let style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column)
            .size_px(Some(100.0), Some(200.0))
            .build();

        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.size.width, Dimension::length(100.0));
        assert_eq!(style.size.height, Dimension::length(200.0));
    }

    #[test]
    fn test_style_builder_grid_auto_flow() {
        let mut builder = StyleBuilder::new();
        builder.grid_auto_flow = Some(GridAutoFlow::Column);
        let style = builder.build();

        assert_eq!(style.grid_auto_flow, TGridAutoFlow::Column);
    }

    #[test]
    fn test_place_items_enum() {
        let variants = [
            PlaceItems::Start,
            PlaceItems::Center,
            PlaceItems::End,
            PlaceItems::Stretch,
        ];

        // All should be different
        for (i, variant1) in variants.iter().enumerate() {
            for (j, variant2) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(variant1, variant2);
                }
            }
        }
    }

    #[test]
    fn test_align_self_enum() {
        let variants = [
            AlignSelf::Auto,
            AlignSelf::Start,
            AlignSelf::Center,
            AlignSelf::End,
            AlignSelf::Stretch,
        ];

        // All should be different
        for (i, variant1) in variants.iter().enumerate() {
            for (j, variant2) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(variant1, variant2);
                }
            }
        }
    }

    #[test]
    fn test_style_builder_default_values() {
        let builder = StyleBuilder::new();

        // Test that optional fields start as None
        assert!(builder.grid_cols.is_none());
        assert!(builder.grid_rows.is_none());
        assert!(builder.grid_auto_flow.is_none());
        assert!(builder.col_span.is_none());
        assert!(builder.row_span.is_none());
        assert!(builder.fg_rgba.is_none());
        assert!(builder.bg_rgba.is_none());
        assert!(builder.opacity.is_none());
        assert!(builder.z_index.is_none());

        // Test boolean flags
        assert!(!builder.bold);
        assert!(!builder.italic);
        assert!(!builder.underline);
        assert!(!builder.reverse);
        assert!(!builder.strike);
    }

    #[test]
    fn test_box_spacing_edge_cases() {
        // Test with zero values
        let spacing = BoxSpacing::uniform(0.0);
        assert_eq!(spacing.horizontal(), 0.0);
        assert_eq!(spacing.vertical(), 0.0);

        // Test with negative values (should be allowed)
        let spacing = BoxSpacing::new(-1.0, -2.0, -3.0, -4.0);
        assert_eq!(spacing.horizontal(), -3.0);
        assert_eq!(spacing.vertical(), -7.0);

        // Test with large values
        let spacing = BoxSpacing::uniform(1000.0);
        assert_eq!(spacing.horizontal(), 2000.0);
        assert_eq!(spacing.vertical(), 2000.0);
    }

    #[test]
    fn test_visual_style_equality() {
        let style1 = VisualStyle {
            fg: RgbaColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            bg: RgbaColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            decorations: TextDecorations {
                bold: true,
                italic: false,
                underline: true,
                reverse: false,
            },
        };

        let style2 = VisualStyle {
            fg: RgbaColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            bg: RgbaColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            decorations: TextDecorations {
                bold: true,
                italic: false,
                underline: true,
                reverse: false,
            },
        };

        let style3 = VisualStyle {
            fg: RgbaColor {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            }, // Different color
            bg: RgbaColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            decorations: TextDecorations {
                bold: true,
                italic: false,
                underline: true,
                reverse: false,
            },
        };

        assert_eq!(style1, style2);
        assert_ne!(style1, style3);
    }
}
