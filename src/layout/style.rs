use taffy::style::{Style, Display, FlexDirection, JustifyContent as TJustify, AlignItems as TAlign, AlignContent as TAlignContent, Dimension, LengthPercentage, LengthPercentageAuto, Overflow, GridAutoFlow as TGridAutoFlow, GridPlacement, TrackSizingFunction, GridTemplateComponent};
use taffy::geometry::{Size, Rect, Point, Line};
use taffy::prelude::{FromFr as _, TaffyGridLine, TaffyGridSpan};

/// RGBA color tuple (r, g, b, a)
pub type RgbaColor = (f32, f32, f32, f32);

/// Text decoration flags (bold, italic, underline, reverse)
pub type TextDecorations = (bool, bool, bool, bool);

/// Visual style data containing foreground color, background color, and text decorations
pub type VisualStyle = (RgbaColor, RgbaColor, TextDecorations);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction { Row, Column }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent { Start, Center, End, SpaceBetween, SpaceAround, SpaceEvenly }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems { Start, Center, End, Stretch }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceItems { Start, Center, End, Stretch }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignSelf { Auto, Start, Center, End, Stretch }


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridAutoFlow { Row, Column, RowDense, ColumnDense }

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
#[derive(Clone, Debug)]
#[derive(Default)]
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
    fg_rgba: Option<(f32,f32,f32,f32)>,
    bg_rgba: Option<(f32,f32,f32,f32)>,
    bold: bool,
    italic: bool,
    underline: bool,
    reverse: bool,
    strike: bool,
    // Cached padding/margin for paint-time (px only)
    pad_l: Option<f32>, pad_r: Option<f32>, pad_t: Option<f32>, pad_b: Option<f32>,
    mar_l: Option<f32>, mar_r: Option<f32>, mar_t: Option<f32>, mar_b: Option<f32>,
}


impl StyleBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn display_flex(mut self) -> Self { self.style.display = Display::Flex; self }
    pub fn display_grid(mut self) -> Self { self.style.display = Display::Grid; self }

    pub fn direction(mut self, dir: Direction) -> Self {
        self.style.flex_direction = match dir { Direction::Row => FlexDirection::Row, Direction::Column => FlexDirection::Column };
        self
    }

    pub fn size_px(mut self, w: Option<f32>, h: Option<f32>) -> Self {
        if let Some(w) = w { self.style.size.width = Dimension::length(w); }
        if let Some(h) = h { self.style.size.height = Dimension::length(h); }
        self
    }

    pub fn width_pct(mut self, pct: f32) -> Self { self.style.size.width = Dimension::percent(pct/100.0); self }
    pub fn height_pct(mut self, pct: f32) -> Self { self.style.size.height = Dimension::percent(pct/100.0); self }

    pub fn min_size_px(mut self, w: Option<f32>, h: Option<f32>) -> Self {
        if let Some(w) = w { self.style.min_size.width = Dimension::length(w); }
        if let Some(h) = h { self.style.min_size.height = Dimension::length(h); }
        self
    }

    pub fn max_size_px(mut self, w: Option<f32>, h: Option<f32>) -> Self {
        if let Some(w) = w { self.style.max_size.width = Dimension::length(w); }
        if let Some(h) = h { self.style.max_size.height = Dimension::length(h); }
        self
    }

    pub fn flex_grow(mut self, v: f32) -> Self { self.style.flex_grow = v; self }
    pub fn flex_shrink(mut self, v: f32) -> Self { self.style.flex_shrink = v; self }

    pub fn padding_all_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding = Rect { left: lp, right: lp, top: lp, bottom: lp };
        self.pad_l = Some(v); self.pad_r = Some(v); self.pad_t = Some(v); self.pad_b = Some(v);
        self
    }
    pub fn padding_x_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.left = lp; self.style.padding.right = lp;
        self.pad_l = Some(v); self.pad_r = Some(v); self
    }
    pub fn padding_y_px(mut self, v: f32) -> Self {
        let lp = LengthPercentage::length(v);
        self.style.padding.top = lp; self.style.padding.bottom = lp;
        self.pad_t = Some(v); self.pad_b = Some(v); self
    }
    pub fn padding_l_px(mut self, v: f32) -> Self { let lp=LengthPercentage::length(v); self.style.padding.left=lp; self.pad_l=Some(v); self }
    pub fn padding_r_px(mut self, v: f32) -> Self { let lp=LengthPercentage::length(v); self.style.padding.right=lp; self.pad_r=Some(v); self }
    pub fn padding_t_px(mut self, v: f32) -> Self { let lp=LengthPercentage::length(v); self.style.padding.top=lp; self.pad_t=Some(v); self }
    pub fn padding_b_px(mut self, v: f32) -> Self { let lp=LengthPercentage::length(v); self.style.padding.bottom=lp; self.pad_b=Some(v); self }

    pub fn margin_all_px(mut self, v: f32) -> Self {
        let lp = LengthPercentageAuto::length(v);
        self.style.margin = Rect { left: lp, right: lp, top: lp, bottom: lp };
        self.mar_l = Some(v); self.mar_r = Some(v); self.mar_t = Some(v); self.mar_b = Some(v);
        self
    }
    pub fn margin_x_px(mut self, v: f32) -> Self { let lp=LengthPercentageAuto::length(v); self.style.margin.left=lp; self.style.margin.right=lp; self.mar_l=Some(v); self.mar_r=Some(v); self }
    pub fn margin_y_px(mut self, v: f32) -> Self { let lp=LengthPercentageAuto::length(v); self.style.margin.top=lp; self.style.margin.bottom=lp; self.mar_t=Some(v); self.mar_b=Some(v); self }
    pub fn margin_l_px(mut self, v: f32) -> Self { let lp=LengthPercentageAuto::length(v); self.style.margin.left=lp; self.mar_l=Some(v); self }
    pub fn margin_r_px(mut self, v: f32) -> Self { let lp=LengthPercentageAuto::length(v); self.style.margin.right=lp; self.mar_r=Some(v); self }
    pub fn margin_t_px(mut self, v: f32) -> Self { let lp=LengthPercentageAuto::length(v); self.style.margin.top=lp; self.mar_t=Some(v); self }
    pub fn margin_b_px(mut self, v: f32) -> Self { let lp=LengthPercentageAuto::length(v); self.style.margin.bottom=lp; self.mar_b=Some(v); self }

    pub fn gap_px(mut self, x: f32, y: f32) -> Self {
        self.style.gap = Size { width: LengthPercentage::length(x), height: LengthPercentage::length(y) };
        self
    }

    pub fn text_rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self { self.fg_rgba = Some((r,g,b,a)); self }
    pub fn bg_rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self { self.bg_rgba = Some((r,g,b,a)); self }
    pub fn bold(mut self, v: bool) -> Self { self.bold = v; self }
    pub fn italic(mut self, v: bool) -> Self { self.italic = v; self }
    pub fn underline(mut self, v: bool) -> Self { self.underline = v; self }
    pub fn reverse(mut self, v: bool) -> Self { self.reverse = v; self }
    pub fn strike(mut self, v: bool) -> Self { self.strike = v; self }

    pub fn take_visuals(&mut self) -> Option<VisualStyle> {
        let had_fg = self.fg_rgba.is_some();
        let had_bg = self.bg_rgba.is_some();
        let any = had_fg || had_bg || self.bold || self.italic || self.underline || self.reverse;
        if !any { return None; }
        let fg = self.fg_rgba.take().unwrap_or((1.0,1.0,1.0,1.0));
        let bg = self.bg_rgba.take().unwrap_or((0.0,0.0,0.0,1.0));

        let attrs = (self.bold, self.italic, self.underline, self.reverse);
        Some((fg,bg,attrs))
    }

    pub fn pad_cache(&self) -> (f32,f32,f32,f32) {
        (self.pad_l.unwrap_or(0.0), self.pad_r.unwrap_or(0.0), self.pad_t.unwrap_or(0.0), self.pad_b.unwrap_or(0.0))
    }
    pub fn mar_cache(&self) -> (f32,f32,f32,f32) {
        (self.mar_l.unwrap_or(0.0), self.mar_r.unwrap_or(0.0), self.mar_t.unwrap_or(0.0), self.mar_b.unwrap_or(0.0))
    }

    pub fn grid_cols(mut self, n: u16) -> Self { self.grid_cols = Some(n); self }
    pub fn grid_rows(mut self, n: u16) -> Self { self.grid_rows = Some(n); self }
    pub fn grid_auto_flow(mut self, f: GridAutoFlow) -> Self { self.grid_auto_flow = Some(f); self }

    pub fn col_span(mut self, n: u16) -> Self { self.col_span = Some(n); self }
    pub fn row_span(mut self, n: u16) -> Self { self.row_span = Some(n); self }
    pub fn col_start(mut self, n: i16) -> Self { self.col_start = Some(n); self }
    pub fn col_end(mut self, n: i16) -> Self { self.col_end = Some(n); self }
    pub fn row_start(mut self, n: i16) -> Self { self.row_start = Some(n); self }
    pub fn row_end(mut self, n: i16) -> Self { self.row_end = Some(n); self }

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

    pub fn align(mut self, a: AlignItems) -> Self {
        self.style.align_items = Some(match a {
            AlignItems::Start => TAlign::FlexStart,
            AlignItems::Center => TAlign::Center,
            AlignItems::End => TAlign::FlexEnd,
            AlignItems::Stretch => TAlign::Stretch,
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


    pub fn overflow_clip(mut self) -> Self { self.style.overflow = Point { x: Overflow::Hidden, y: Overflow::Hidden }; self }

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
        if let Some(flow) = self.grid_auto_flow { self.style.grid_auto_flow = flow.to_taffy(); }
        // Map placement to grid_row/grid_column where possible
        if let Some(span) = self.col_span { self.style.grid_column = Line { start: GridPlacement::from_span(span), end: GridPlacement::Auto }; }
        if let Some(span) = self.row_span { self.style.grid_row = Line { start: GridPlacement::from_span(span), end: GridPlacement::Auto }; }
        if let Some(s) = self.col_start { self.style.grid_column = Line { start: GridPlacement::from_line_index(s), end: self.style.grid_column.end }; }
        if let Some(e) = self.col_end { self.style.grid_column = Line { start: self.style.grid_column.start, end: GridPlacement::from_line_index(e) }; }
        if let Some(s) = self.row_start { self.style.grid_row = Line { start: GridPlacement::from_line_index(s), end: self.style.grid_row.end }; }
        if let Some(e) = self.row_end { self.style.grid_row = Line { start: self.style.grid_row.start, end: GridPlacement::from_line_index(e) }; }
        self.style
    }
}

