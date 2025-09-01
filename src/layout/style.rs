use taffy::geometry::{Line, Point, Rect, Size};
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
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
}

/// Visual style data containing foreground color, background color, and text decorations
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VisualStyle {
    pub fg: RgbaColor,
    pub bg: RgbaColor,
    pub decorations: TextDecorations,
}

/// Box model spacing (left, right, top, bottom)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoxSpacing {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl BoxSpacing {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn uniform(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceItems {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignSelf {
    Auto,
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridAutoFlow {
    Row,
    Column,
    RowDense,
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
#[derive(Clone, Debug, Default)]
pub struct StyleBuilder {
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
    fg_rgba: Option<(f32, f32, f32, f32)>,
    bg_rgba: Option<(f32, f32, f32, f32)>,
    opacity: Option<f32>,
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
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display_flex(mut self) -> Self {
        self.style.display = Display::Flex;
        self
    }
    pub fn display_grid(mut self) -> Self {
        self.style.display = Display::Grid;
        self
    }

    pub fn direction(mut self, dir: Direction) -> Self {
        self.style.flex_direction = match dir {
            Direction::Row => FlexDirection::Row,
            Direction::Column => FlexDirection::Column,
            Direction::RowReverse => FlexDirection::RowReverse,
            Direction::ColumnReverse => FlexDirection::ColumnReverse,
        };
        self
    }

    pub fn size_px(mut self, w: Option<f32>, h: Option<f32>) -> Self {
        if let Some(w) = w {
            self.style.size.width = Dimension::length(w);
        }
        if let Some(h) = h {
            self.style.size.height = Dimension::length(h);
        }
        self
    }

    pub fn width_pct(mut self, pct: f32) -> Self {
        self.style.size.width = Dimension::percent(pct / 100.0);
        self
    }
    pub fn height_pct(mut self, pct: f32) -> Self {
        self.style.size.height = Dimension::percent(pct / 100.0);
        self
    }

    pub fn width_px(mut self, px: f32) -> Self {
        self.style.size.width = Dimension::length(px);
        self
    }
    pub fn height_px(mut self, px: f32) -> Self {
        self.style.size.height = Dimension::length(px);
        self
    }

    pub fn width_percent(mut self, pct: f32) -> Self {
        self.style.size.width = Dimension::percent(pct / 100.0);
        self
    }
    pub fn height_percent(mut self, pct: f32) -> Self {
        self.style.size.height = Dimension::percent(pct / 100.0);
        self
    }

    pub fn width_auto(mut self) -> Self {
        self.style.size.width = Dimension::auto();
        self
    }
    pub fn height_auto(mut self) -> Self {
        self.style.size.height = Dimension::auto();
        self
    }

    pub fn min_size_px(mut self, w: Option<f32>, h: Option<f32>) -> Self {
        if let Some(w) = w {
            self.style.min_size.width = Dimension::length(w);
        }
        if let Some(h) = h {
            self.style.min_size.height = Dimension::length(h);
        }
        self
    }

    pub fn max_size_px(mut self, w: Option<f32>, h: Option<f32>) -> Self {
        if let Some(w) = w {
            self.style.max_size.width = Dimension::length(w);
        }
        if let Some(h) = h {
            self.style.max_size.height = Dimension::length(h);
        }
        self
    }

    pub fn min_width_px(mut self, px: f32) -> Self {
        self.style.min_size.width = Dimension::length(px);
        self
    }
    pub fn min_height_px(mut self, px: f32) -> Self {
        self.style.min_size.height = Dimension::length(px);
        self
    }
    pub fn max_width_px(mut self, px: f32) -> Self {
        self.style.max_size.width = Dimension::length(px);
        self
    }
    pub fn max_height_px(mut self, px: f32) -> Self {
        self.style.max_size.height = Dimension::length(px);
        self
    }

    pub fn min_width_percent(mut self, pct: f32) -> Self {
        self.style.min_size.width = Dimension::percent(pct / 100.0);
        self
    }
    pub fn min_height_percent(mut self, pct: f32) -> Self {
        self.style.min_size.height = Dimension::percent(pct / 100.0);
        self
    }
    pub fn max_width_percent(mut self, pct: f32) -> Self {
        self.style.max_size.width = Dimension::percent(pct / 100.0);
        self
    }
    pub fn max_height_percent(mut self, pct: f32) -> Self {
        self.style.max_size.height = Dimension::percent(pct / 100.0);
        self
    }

    pub fn min_width_auto(mut self) -> Self {
        self.style.min_size.width = Dimension::auto();
        self
    }
    pub fn min_height_auto(mut self) -> Self {
        self.style.min_size.height = Dimension::auto();
        self
    }
    pub fn max_width_auto(mut self) -> Self {
        self.style.max_size.width = Dimension::auto();
        self
    }
    pub fn max_height_auto(mut self) -> Self {
        self.style.max_size.height = Dimension::auto();
        self
    }

    pub fn flex_grow(mut self, v: f32) -> Self {
        self.style.flex_grow = v;
        self
    }
    pub fn flex_shrink(mut self, v: f32) -> Self {
        self.style.flex_shrink = v;
        self
    }

    pub fn flex_wrap(mut self, wrap: bool) -> Self {
        use taffy::FlexWrap;
        self.style.flex_wrap = if wrap {
            FlexWrap::Wrap
        } else {
            FlexWrap::NoWrap
        };
        self
    }

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
    pub fn padding_x_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.left = lp;
        self.style.padding.right = lp;
        self.pad_l = Some(v);
        self.pad_r = Some(v);
        self
    }
    pub fn padding_y_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.top = lp;
        self.style.padding.bottom = lp;
        self.pad_t = Some(v);
        self.pad_b = Some(v);
        self
    }
    pub fn padding_l_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.left = lp;
        self.pad_l = Some(v);
        self
    }
    pub fn padding_r_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.right = lp;
        self.pad_r = Some(v);
        self
    }
    pub fn padding_t_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.top = lp;
        self.pad_t = Some(v);
        self
    }
    pub fn padding_b_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.bottom = lp;
        self.pad_b = Some(v);
        self
    }

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
    pub fn margin_x_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.left = lp;
        self.style.margin.right = lp;
        self.mar_l = Some(v);
        self.mar_r = Some(v);
        self
    }
    pub fn margin_y_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.top = lp;
        self.style.margin.bottom = lp;
        self.mar_t = Some(v);
        self.mar_b = Some(v);
        self
    }
    pub fn margin_l_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.left = lp;
        self.mar_l = Some(v);
        self
    }
    pub fn margin_r_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.right = lp;
        self.mar_r = Some(v);
        self
    }
    pub fn margin_t_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.top = lp;
        self.mar_t = Some(v);
        self
    }
    pub fn margin_b_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin.bottom = lp;
        self.mar_b = Some(v);
        self
    }

    // Auto margin utilities for centering
    pub fn margin_auto(mut self) -> Self {
        self.style.margin.left = LengthPercentageAuto::auto();
        self.style.margin.right = LengthPercentageAuto::auto();
        self.style.margin.top = LengthPercentageAuto::auto();
        self.style.margin.bottom = LengthPercentageAuto::auto();
        self
    }

    pub fn margin_x_auto(mut self) -> Self {
        self.style.margin.left = LengthPercentageAuto::auto();
        self.style.margin.right = LengthPercentageAuto::auto();
        self
    }

    pub fn margin_y_auto(mut self) -> Self {
        self.style.margin.top = LengthPercentageAuto::auto();
        self.style.margin.bottom = LengthPercentageAuto::auto();
        self
    }

    pub fn gap_px(mut self, x: f32, y: f32) -> Self {
        self.style.gap = Size {
            width: LengthPercentage::length(x),
            height: LengthPercentage::length(y),
        };
        self
    }

    pub fn text_rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.fg_rgba = Some((r, g, b, a));
        self
    }

    // Alias for text_rgba for CSS utility compatibility
    pub fn fg_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.text_rgba(r, g, b, a)
    }
    pub fn bg_rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.bg_rgba = Some((r, g, b, a));
        self
    }
    pub fn bold(mut self, v: bool) -> Self {
        self.bold = v;
        self
    }
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = v;
        self
    }
    pub fn underline(mut self, v: bool) -> Self {
        self.underline = v;
        self
    }
    pub fn reverse(mut self, v: bool) -> Self {
        self.reverse = v;
        self
    }
    pub fn strike(mut self, v: bool) -> Self {
        self.strike = v;
        self
    }

    // Font weight methods for CSS utility compatibility
    pub fn font_weight_light(mut self) -> Self {
        // Light weight is typically represented by making text less bold
        self.bold = false;
        self
    }
    pub fn font_weight_normal(mut self) -> Self {
        self.bold = false;
        self
    }
    pub fn font_weight_medium(mut self) -> Self {
        self.bold = false; // Medium is closer to normal in terminal
        self
    }
    pub fn font_weight_bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn font_weight_black(mut self) -> Self {
        self.bold = true; // Black weight is still just bold in terminal
        self
    }

    pub fn opacity(mut self, alpha: f32) -> Self {
        self.opacity = Some(alpha.clamp(0.0, 1.0));
        self
    }

    pub fn z_index(mut self, z: i32) -> Self {
        self.z_index = Some(z);
        self
    }

    pub fn has_bg_color(&self) -> bool {
        self.bg_rgba.is_some()
    }

    pub fn get_z_index(&self) -> Option<i32> {
        self.z_index
    }

    pub fn get_overflow_x(&self) -> Overflow {
        self.style.overflow.x
    }

    pub fn get_overflow_y(&self) -> Overflow {
        self.style.overflow.y
    }

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
            || self.reverse;
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

    pub fn pad_cache(&self) -> BoxSpacing {
        BoxSpacing {
            left: self.pad_l.unwrap_or(0.0),
            right: self.pad_r.unwrap_or(0.0),
            top: self.pad_t.unwrap_or(0.0),
            bottom: self.pad_b.unwrap_or(0.0),
        }
    }

    pub fn mar_cache(&self) -> BoxSpacing {
        BoxSpacing {
            left: self.mar_l.unwrap_or(0.0),
            right: self.mar_r.unwrap_or(0.0),
            top: self.mar_t.unwrap_or(0.0),
            bottom: self.mar_b.unwrap_or(0.0),
        }
    }

    pub fn grid_cols(mut self, n: u16) -> Self {
        self.grid_cols = Some(n);
        self
    }
    pub fn grid_rows(mut self, n: u16) -> Self {
        self.grid_rows = Some(n);
        self
    }
    pub fn grid_auto_flow(mut self, f: GridAutoFlow) -> Self {
        self.grid_auto_flow = Some(f);
        self
    }

    pub fn col_span(mut self, n: u16) -> Self {
        self.col_span = Some(n);
        self
    }
    pub fn row_span(mut self, n: u16) -> Self {
        self.row_span = Some(n);
        self
    }
    pub fn col_start(mut self, n: i16) -> Self {
        self.col_start = Some(n);
        self
    }
    pub fn col_end(mut self, n: i16) -> Self {
        self.col_end = Some(n);
        self
    }
    pub fn row_start(mut self, n: i16) -> Self {
        self.row_start = Some(n);
        self
    }
    pub fn row_end(mut self, n: i16) -> Self {
        self.row_end = Some(n);
        self
    }

    // Grid template methods for CSS utility compatibility
    pub fn grid_template_columns(self, cols: u16) -> Self {
        self.grid_cols(cols)
    }

    pub fn grid_template_rows(self, rows: u16) -> Self {
        self.grid_rows(rows)
    }

    pub fn grid_column_span(self, span: u16) -> Self {
        self.col_span(span)
    }

    pub fn grid_row_span(self, span: u16) -> Self {
        self.row_span(span)
    }

    // Grid auto methods
    pub fn grid_column_auto(mut self) -> Self {
        self.style.grid_column = Line {
            start: GridPlacement::Auto,
            end: GridPlacement::Auto,
        };
        self
    }

    pub fn grid_row_auto(mut self) -> Self {
        self.style.grid_row = Line {
            start: GridPlacement::Auto,
            end: GridPlacement::Auto,
        };
        self
    }

    pub fn grid_column_start(self, start: i16) -> Self {
        self.col_start(start)
    }

    pub fn grid_column_end(self, end: i16) -> Self {
        self.col_end(end)
    }

    pub fn grid_row_start(self, start: i16) -> Self {
        self.row_start(start)
    }

    // Advanced grid utilities
    pub fn grid_auto_fit_columns(mut self, min_size: u16) -> Self {
        // In TUI, we simulate auto-fit by setting a flexible grid
        // This would need special handling in the layout system
        self.grid_cols = Some(min_size.max(1));
        self
    }

    pub fn grid_auto_fill_columns(mut self, min_size: u16) -> Self {
        // Similar to auto-fit but fills available space
        self.grid_cols = Some(min_size.max(1));
        self
    }

    pub fn grid_auto_fit_rows(mut self, min_size: u16) -> Self {
        self.grid_rows = Some(min_size.max(1));
        self
    }

    pub fn grid_auto_fill_rows(mut self, min_size: u16) -> Self {
        self.grid_rows = Some(min_size.max(1));
        self
    }

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

    pub fn grid_row_end(self, end: i16) -> Self {
        self.row_end(end)
    }

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
    pub fn justify_content(self, j: JustifyContent) -> Self {
        self.justify(j)
    }

    pub fn align_items(self, a: AlignItems) -> Self {
        self.align(a)
    }

    // Overflow utilities (critical for TUI)
    pub fn overflow_hidden(mut self) -> Self {
        self.style.overflow.x = Overflow::Hidden;
        self.style.overflow.y = Overflow::Hidden;
        self
    }

    pub fn overflow_scroll(mut self) -> Self {
        self.style.overflow.x = Overflow::Scroll;
        self.style.overflow.y = Overflow::Scroll;
        self
    }

    pub fn overflow_auto(mut self) -> Self {
        self.style.overflow.x = Overflow::Scroll; // TUI doesn't have "auto", use scroll
        self.style.overflow.y = Overflow::Scroll;
        self
    }

    pub fn overflow_visible(mut self) -> Self {
        self.style.overflow.x = Overflow::Visible;
        self.style.overflow.y = Overflow::Visible;
        self
    }

    // X-axis overflow
    pub fn overflow_x_hidden(mut self) -> Self {
        self.style.overflow.x = Overflow::Hidden;
        self
    }

    pub fn overflow_x_scroll(mut self) -> Self {
        self.style.overflow.x = Overflow::Scroll;
        self
    }

    pub fn overflow_x_auto(mut self) -> Self {
        self.style.overflow.x = Overflow::Scroll;
        self
    }

    pub fn overflow_x_visible(mut self) -> Self {
        self.style.overflow.x = Overflow::Visible;
        self
    }

    // Y-axis overflow
    pub fn overflow_y_hidden(mut self) -> Self {
        self.style.overflow.y = Overflow::Hidden;
        self
    }

    pub fn overflow_y_scroll(mut self) -> Self {
        self.style.overflow.y = Overflow::Scroll;
        self
    }

    pub fn overflow_y_auto(mut self) -> Self {
        self.style.overflow.y = Overflow::Scroll;
        self
    }

    pub fn overflow_y_visible(mut self) -> Self {
        self.style.overflow.y = Overflow::Visible;
        self
    }

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
    pub fn position_static(mut self) -> Self {
        self.style.position = taffy::style::Position::Relative; // TUI equivalent
        self.z_index(0)
    }

    pub fn position_relative(mut self) -> Self {
        self.style.position = taffy::style::Position::Relative;
        self.z_index(1)
    }

    pub fn position_absolute(mut self) -> Self {
        self.style.position = taffy::style::Position::Absolute;
        self.z_index(10)
    }

    pub fn position_fixed(mut self) -> Self {
        self.style.position = taffy::style::Position::Absolute; // TUI equivalent
        self.z_index(50)
    }

    pub fn position_sticky(mut self) -> Self {
        self.style.position = taffy::style::Position::Relative; // TUI equivalent
        self.z_index(20)
    }

    // Inset utilities
    pub fn inset_all(mut self, value: f32) -> Self {
        self.style.inset.left = LengthPercentageAuto::length(value);
        self.style.inset.right = LengthPercentageAuto::length(value);
        self.style.inset.top = LengthPercentageAuto::length(value);
        self.style.inset.bottom = LengthPercentageAuto::length(value);
        self
    }

    pub fn inset_all_auto(mut self) -> Self {
        self.style.inset.left = LengthPercentageAuto::auto();
        self.style.inset.right = LengthPercentageAuto::auto();
        self.style.inset.top = LengthPercentageAuto::auto();
        self.style.inset.bottom = LengthPercentageAuto::auto();
        self
    }

    pub fn inset_top(mut self, value: f32) -> Self {
        self.style.inset.top = LengthPercentageAuto::length(value);
        self
    }

    pub fn inset_right(mut self, value: f32) -> Self {
        self.style.inset.right = LengthPercentageAuto::length(value);
        self
    }

    pub fn inset_bottom(mut self, value: f32) -> Self {
        self.style.inset.bottom = LengthPercentageAuto::length(value);
        self
    }

    pub fn inset_left(mut self, value: f32) -> Self {
        self.style.inset.left = LengthPercentageAuto::length(value);
        self
    }

    pub fn build(mut self) -> Style {
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
        let fg = RgbaColor { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        let bg = RgbaColor { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
        let decorations = TextDecorations { bold: true, italic: true, underline: false, reverse: false };

        let style = VisualStyle { fg, bg, decorations };
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
        assert_eq!(spacing.vertical(), 7.0);   // top + bottom
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
        assert_eq!(GridAutoFlow::ColumnDense.to_taffy(), TGridAutoFlow::ColumnDense);
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

        let style = StyleBuilder::new().direction(Direction::ColumnReverse).build();
        assert_eq!(style.flex_direction, FlexDirection::ColumnReverse);
    }

    #[test]
    fn test_style_builder_size_px() {
        let style = StyleBuilder::new().size_px(Some(100.0), Some(200.0)).build();
        assert_eq!(style.size.width, Dimension::length(100.0));
        assert_eq!(style.size.height, Dimension::length(200.0));

        // Test partial sizing
        let style = StyleBuilder::new().size_px(Some(50.0), None).build();
        assert_eq!(style.size.width, Dimension::length(50.0));
        assert_eq!(style.size.height, Dimension::auto()); // Should remain default
    }

    #[test]
    fn test_style_builder_grid_setup() {
        let style = StyleBuilder::new()
            .grid_cols(3)
            .grid_rows(2)
            .build();

        assert_eq!(style.display, Display::Grid);
        assert_eq!(style.grid_template_columns.len(), 3);
        assert_eq!(style.grid_template_rows.len(), 2);
    }

    #[test]
    fn test_style_builder_grid_placement() {
        let style = StyleBuilder::new()
            .col_span(2)
            .row_span(3)
            .build();

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
            fg: RgbaColor { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
            bg: RgbaColor { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
            decorations: TextDecorations { bold: true, italic: false, underline: true, reverse: false },
        };

        let style2 = VisualStyle {
            fg: RgbaColor { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
            bg: RgbaColor { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
            decorations: TextDecorations { bold: true, italic: false, underline: true, reverse: false },
        };

        let style3 = VisualStyle {
            fg: RgbaColor { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }, // Different color
            bg: RgbaColor { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
            decorations: TextDecorations { bold: true, italic: false, underline: true, reverse: false },
        };

        assert_eq!(style1, style2);
        assert_ne!(style1, style3);
    }
}
