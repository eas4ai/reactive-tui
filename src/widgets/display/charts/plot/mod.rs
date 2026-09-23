//! The plot layer every chart type draws through (CHT-010): scales, ticks,
//! axes, grid, label fitting, legend, tooltip, curve interpolation, min/max
//! decimation and size classes. Renderers obtain every cell or dot position
//! from a scale here and never map a data value to a position themselves.
//!
//! Positions are plain `f64` range units chosen by the caller; the chart
//! renderers use dot units (two per column, four per row) so the mask canvas
//! can place rectangle edges at eighths of a cell.

pub mod axis;
pub mod curve;
pub mod decimate;
pub mod layout;
pub mod legend;
pub mod scale;
pub mod tick;
pub mod tooltip;

pub use axis::{fit_label, text_width, Axis, Grid, Orientation};
pub use curve::{polyline, Curve};
pub use decimate::{decimate_min_max, Sample};
pub use layout::{Rect, SizeClass};
pub use legend::{Legend, LegendEntry, SWATCH};
pub use scale::{ScaleBand, ScaleLinear, ScaleOrdinal, ScalePoint};
pub use tick::{
    band_ticks, format_tick, label_skip, labeled_ticks, linear_ticks, nice_step, point_ticks, Tick,
};
pub use tooltip::{Tooltip, TooltipBox, TooltipRow, MAX_ROWS};

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
}
