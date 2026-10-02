//! The plot layer every chart type draws through (CHT-010): scales, ticks,
//! axes, grid, label fitting, legend, tooltip, curve interpolation, min/max
//! decimation, size classes and the Sankey layout. Renderers obtain every cell or dot position
//! from a scale here and never map a data value to a position themselves.
//!
//! Positions are plain `f64` range units chosen by the caller; the chart
//! renderers use dot units (two per column, four per row) so the mask canvas
//! can place rectangle edges at eighths of a cell.

pub mod axis;
pub mod curve;
pub mod decimate;
pub mod domain;
pub mod layout;
pub mod legend;
pub mod polar;
pub mod ramp;
pub mod sankey;
pub mod scale;
pub mod tick;
pub mod tooltip;

pub use axis::{fit_label, text_width, Axis, Grid, Orientation};
pub use curve::{polyline, Curve};
pub use decimate::{decimate_by_column, decimate_min_max, Sample};
pub use domain::{value_domain, value_ticks};
pub use layout::{Rect, SizeClass};
pub use legend::{Legend, LegendEntry, SWATCH};
pub use polar::{spoke_angles, PolarGrid};
pub use ramp::{mix, Ramp};
pub use sankey::{
    Sankey, SankeyAlign, SankeyError, SankeyGraph, SankeyLink, SankeyNode, SankeyRibbon,
    SankeyValueScale,
};
pub use scale::{ScaleBand, ScaleLinear, ScaleOrdinal, ScalePoint};
pub use tick::{
    band_ticks, format_tick, label_skip, labeled_ticks, linear_ticks, nice_domain, nice_step,
    point_ticks, spread_ticks, tick_step, Tick,
};
pub use tooltip::{Tooltip, TooltipBox, TooltipLine, TooltipRow, TooltipStyle, MAX_ROWS};

/// A resolved color: red, green, blue and alpha in `0.0..=1.0`.
pub type Rgba = (f32, f32, f32, f32);

/// Where the plot layer writes text. Charts implement it on their text
/// layer; `text` writes over the shapes, `under` writes beneath them.
pub trait TextSink {
    /// Write `text` starting at (`x`, `y`), at most `width` cells, over the
    /// shapes.
    fn text(&mut self, x: usize, y: usize, width: usize, text: &str, color: Option<Rgba>);
    /// Write one glyph beneath the shapes, for grid and axis lines.
    fn under(&mut self, x: usize, y: usize, glyph: &str, color: Option<Rgba>);
    /// Paint `background` behind the `width` by `height` cells from (`x`,
    /// `y`), keeping their glyphs: a tooltip box is opaque (CHT-018) and a
    /// highlight band tints its row. A sink without backgrounds ignores it.
    fn fill(&mut self, x: usize, y: usize, width: usize, height: usize, background: Option<Rgba>) {
        let _ = (x, y, width, height, background);
    }
}
