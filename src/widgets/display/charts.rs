//! Chart Widgets for Data Visualization
//!
//! Provides a comprehensive set of chart components for terminal-based data visualization:
//! - Bar Charts (horizontal and vertical)
//! - Line Charts with multiple series
//! - Pie Charts with customizable segments
//! - Area Charts and Scatter Plots
//! - Real-time data support with animations

use crate::component::{Component, Element, Props};
use std::collections::HashMap;

pub use mask::GlyphSet;
pub use plot::{Curve, SizeClass};

/// Whether the terminal draws braille and block glyphs, as reported by the
/// backend's capability query; charts fall back to ASCII when it is false
/// (CHT-028). Unreported means available.
static GLYPH_SUPPORT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Record the terminal capability report for braille and block glyphs.
/// Backends call this once their query has answered; `false` makes every
/// chart resolve cells with `#`, `|`, `-` and `.` instead.
pub fn report_glyph_support(available: bool) {
    GLYPH_SUPPORT.store(available, std::sync::atomic::Ordering::Relaxed);
}

/// The last reported glyph support (see [`report_glyph_support`]).
pub fn glyph_support() -> bool {
    GLYPH_SUPPORT.load(std::sync::atomic::Ordering::Relaxed)
}

/// Tests that change or depend on a process-wide terminal choice hold this
/// lock, so they cannot race one another under parallel test threads: the
/// glyph report, the application's image blitter, and the terminal
/// environment variables the capability and host-identity probes read.
#[cfg(test)]
pub(crate) static GLYPH_REPORT_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Builder for creating Chart components with a fluent API. It holds the
/// props it builds, so every option of a typed builder has a method here
/// (CHT-035); the options that depend on the chart's orientation or on its
/// data (the value axis of a bar chart, a tick format) apply at `build()`.
#[derive(Clone, Default)]
pub struct ChartsBuilder {
    props: ChartProps,
    /// Deferred options, applied at `build()` once the type and data are known.
    deferred: Deferred,
}

/// A tick label formatter, run once at `build()` (CHT-020, CHT-034).
type TickFormat = std::sync::Arc<dyn Fn(f64) -> String + Send + Sync>;

/// A bar's gradient stops from the point, the chart's value range and a
/// mapping from a chart value to a position along the bar (0 at its base, 1
/// at its tip); run once at `build()`, when the range is known (CHT-013).
type GradientStops = std::sync::Arc<
    dyn Fn(&DataPoint, (f64, f64), &dyn Fn(f64) -> f32) -> Vec<(f32, String)> + Send + Sync,
>;

#[derive(Clone, Default)]
struct Deferred {
    /// Fill gradients per series index, run at `build()` (CHT-013).
    gradients: Vec<(usize, GradientStops)>,
    /// Show the band (category) axis of a bar chart.
    label_axis: Option<bool>,
    /// Show the value axis of a bar chart.
    value_axis: Option<bool>,
    value_tick_count: Option<usize>,
    value_placement: Option<AxisLabelPlacement>,
    value_format: Option<TickFormat>,
    y_format: Option<TickFormat>,
    /// Label count on the band axis of a bar chart.
    band_tick_count: Option<usize>,
    /// Color tokens for candles by direction.
    bullish: Option<String>,
    bearish: Option<String>,
}

impl std::fmt::Debug for ChartsBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChartsBuilder")
            .field("props", &self.props)
            .finish_non_exhaustive()
    }
}

impl ChartsBuilder {
    /// Create a new ChartsBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a bar chart (vertical)
    pub fn bar() -> Self {
        Self::new().chart_type(ChartType::BarVertical)
    }

    /// Create a horizontal bar chart
    pub fn bar_horizontal() -> Self {
        Self::new().chart_type(ChartType::BarHorizontal)
    }

    /// Create a line chart
    pub fn line() -> Self {
        Self::new().chart_type(ChartType::Line)
    }

    /// Create an area chart
    pub fn area() -> Self {
        Self::new().chart_type(ChartType::Area)
    }

    /// Create a pie chart
    pub fn pie() -> Self {
        Self::new().chart_type(ChartType::Pie)
    }

    /// Create a donut chart
    pub fn donut() -> Self {
        Self::new().chart_type(ChartType::Donut)
    }

    /// Create a radar chart: one spoke per point index, one polygon per
    /// series.
    pub fn radar() -> Self {
        Self::new().chart_type(ChartType::Radar)
    }

    /// Create a Sankey chart: the first series' points are the nodes (their
    /// labels and color tokens; a node's throughput comes from the links)
    /// and [`SankeyOptions::links`] the flows between them.
    pub fn sankey() -> Self {
        Self::new().chart_type(ChartType::Sankey)
    }

    /// Create a scatter plot
    pub fn scatter() -> Self {
        Self::new().chart_type(ChartType::Scatter)
    }

    /// Create a candlestick chart; each point carries open, high, low and
    /// close through [`DataPoint::candle`].
    pub fn candlestick() -> Self {
        Self::new().chart_type(ChartType::Candlestick)
    }

    /// Set the chart type
    pub fn chart_type(mut self, chart_type: ChartType) -> Self {
        self.props.chart_type = chart_type;
        self
    }

    /// Add a data series
    pub fn series(mut self, series: DataSeries) -> Self {
        self.props.series.push(series);
        self
    }

    /// Add multiple data series
    pub fn with_series(mut self, series: Vec<DataSeries>) -> Self {
        self.props.series.extend(series);
        self
    }

    /// Set the chart title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.props.title = Some(title.into());
        self
    }

    /// The name the screen reader is told, instead of the title (CHT-036).
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.props.aria_label = Some(label.into());
        self
    }

    /// Set the chart width
    pub fn width(mut self, width: u16) -> Self {
        self.props.width = width;
        self
    }

    /// Set the chart height
    pub fn height(mut self, height: u16) -> Self {
        self.props.height = height;
        self
    }

    /// Set the chart size
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.props.width = width;
        self.props.height = height;
        self
    }

    /// Configure the X-axis
    pub fn x_axis(mut self, axis: ChartAxis) -> Self {
        self.props.x_axis = axis;
        self
    }

    /// Configure the Y-axis
    pub fn y_axis(mut self, axis: ChartAxis) -> Self {
        self.props.y_axis = axis;
        self
    }

    /// Show or hide the drawing's horizontal axis line and labels.
    pub fn x_axis_labels(mut self, show: bool) -> Self {
        self.props.x_axis.show_labels = show;
        self
    }

    /// Show or hide the drawing's vertical axis line and labels.
    pub fn y_axis_labels(mut self, show: bool) -> Self {
        self.props.y_axis.show_labels = show;
        self
    }

    /// Configure the legend
    pub fn legend(mut self, legend: ChartLegend) -> Self {
        self.props.legend = legend;
        self
    }

    /// Hide the legend
    pub fn no_legend(mut self) -> Self {
        self.props.legend.visible = false;
        self
    }

    /// Set color palette
    pub fn color_palette(mut self, colors: Vec<String>) -> Self {
        self.props.color_palette = colors;
        self
    }

    /// Enable animation
    pub fn animated(mut self, animated: bool) -> Self {
        self.props.animated = animated;
        self
    }

    /// Set animation duration in milliseconds
    pub fn animation_duration(mut self, duration: u64) -> Self {
        self.props.animation_duration = duration;
        self
    }

    /// Enable or disable tooltips
    pub fn show_tooltips(mut self, show: bool) -> Self {
        self.props.show_tooltips = show;
        self
    }

    /// The reference's name for [`Self::show_tooltips`]: hover, keyboard
    /// selection and the tooltip.
    pub fn interactive(self, interactive: bool) -> Self {
        self.show_tooltips(interactive)
    }

    /// Add CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.props.class = Some(class.into());
        self
    }

    /// Which edge bars grow from: Bottom or Top for vertical bars, Left or
    /// Right for horizontal ones.
    pub fn growth(mut self, growth: BarGrowth) -> Self {
        self.props.growth = growth;
        self
    }

    /// The reference's name for [`Self::growth`].
    pub fn alignment(self, alignment: BarGrowth) -> Self {
        self.growth(alignment)
    }

    /// Stack series instead of grouping them side by side (bars) or
    /// overlaying them (areas).
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.props.stacked = stacked;
        self
    }

    /// Curve style for line and area strokes.
    pub fn curve(mut self, curve: Curve) -> Self {
        self.props.curve = curve;
        self
    }

    /// Smooth spline strokes.
    pub fn natural(self) -> Self {
        self.curve(Curve::Natural)
    }

    /// Straight strokes.
    pub fn linear(self) -> Self {
        self.curve(Curve::Linear)
    }

    /// Step strokes holding each value until the next point.
    pub fn step_after(self) -> Self {
        self.curve(Curve::StepAfter)
    }

    /// Draw a dot at every line-chart point.
    pub fn dots(mut self, dots: bool) -> Self {
        self.props.dots = dots;
        self
    }

    /// Turn the dots on (the reference's name).
    pub fn dot(self) -> Self {
        self.dots(true)
    }

    /// Name of the series added last.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        if let Some(last) = self.props.series.last_mut() {
            last.name = name.into();
        }
        self
    }

    /// Stroke color token of the series added last.
    pub fn stroke(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.props.series.last_mut() {
            last.color = Some(token.into());
        }
        self
    }

    /// Fill color token of the series added last: an area's fill, which
    /// unset takes the stroke color (CHT-012), or the color of a bar
    /// series' bars (CHT-013).
    pub fn fill(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.props.series.last_mut() {
            last.fill = Some(token.into());
        }
        self
    }

    /// Run `apply` over every point of the series added last.
    fn last_points(&mut self, mut apply: impl FnMut(&mut DataPoint)) {
        if let Some(last) = self.props.series.last_mut() {
            for point in &mut last.data {
                apply(point);
            }
        }
    }

    /// The tooltip title of every point of the series added last, from the
    /// point, in place of its category label (CHT-018): the generic route's
    /// form of the typed `tooltip_title`, evaluated into the points so the
    /// routes build the same props (CHT-035).
    pub fn tooltip_title<S: Into<String>>(mut self, title: impl Fn(&DataPoint) -> S) -> Self {
        self.last_points(|point| point.tooltip.title = Some(title(point).into()));
        self
    }

    /// The tooltip value text of every point of the series added last, from
    /// the point and its value, in place of the formatted value (CHT-018).
    pub fn tooltip_value<S: Into<String>>(mut self, value: impl Fn(&DataPoint, f64) -> S) -> Self {
        self.last_points(|point| point.tooltip.value = Some(value(point, point.value).into()));
        self
    }

    /// The color token of the tooltip value text of every point of the
    /// series added last, from the point and its value (CHT-018).
    pub fn tooltip_value_color<S: Into<String>>(
        mut self,
        color: impl Fn(&DataPoint, f64) -> S,
    ) -> Self {
        self.last_points(|point| point.tooltip.color = Some(color(point, point.value).into()));
        self
    }

    /// The whole tooltip content of every point of the series added last,
    /// as lines from the point, in place of the series rows (CHT-018).
    pub fn tooltip_content<S: Into<String>>(
        mut self,
        content: impl Fn(&DataPoint) -> Vec<S>,
    ) -> Self {
        self.last_points(|point| {
            point.tooltip.lines = content(point).into_iter().map(Into::into).collect()
        });
        self
    }

    /// The value-label color token of every point of the series added last,
    /// from the point (CHT-013).
    pub fn label_color<S: Into<String>>(mut self, color: impl Fn(&DataPoint) -> S) -> Self {
        self.last_points(|point| point.label_color = Some(color(point).into()));
        self
    }

    /// Each bar's own fill color token, from the point, over the series fill
    /// (CHT-013).
    pub fn fill_with<S: Into<String>>(mut self, fill: impl Fn(&DataPoint) -> S) -> Self {
        self.last_points(|point| point.color = Some(fill(point).into()));
        self
    }

    /// Each bar's fill gradient from base to tip, as (offset, color token)
    /// stops from the point, the chart's value range and a mapping from a
    /// chart value to a position along the bar; run once at `build()`, when
    /// every series' values are known (CHT-013).
    pub fn fill_gradient<S: Into<String>>(
        mut self,
        stops: impl Fn(&DataPoint, (f64, f64), &dyn Fn(f64) -> f32) -> Vec<(f32, S)>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        if let Some(index) = self.props.series.len().checked_sub(1) {
            self.deferred.gradients.push((
                index,
                std::sync::Arc::new(move |point, range, to_bar| {
                    stops(point, range, to_bar)
                        .into_iter()
                        .map(|(offset, token)| (offset, token.into()))
                        .collect()
                }),
            ));
        }
        self
    }

    /// Color token for candles that close above their open.
    pub fn bullish(mut self, token: impl Into<String>) -> Self {
        self.deferred.bullish = Some(token.into());
        self
    }

    /// Color token for candles that close below their open.
    pub fn bearish(mut self, token: impl Into<String>) -> Self {
        self.deferred.bearish = Some(token.into());
        self
    }

    /// Force a size class regardless of the rectangle the chart gets.
    pub fn size_class(mut self, class: SizeClass) -> Self {
        self.props.size_class = Some(class);
        self
    }

    /// Render with ASCII glyphs only.
    pub fn ascii(mut self, ascii: bool) -> Self {
        self.props.ascii = ascii;
        self
    }

    /// Show every n-th category label; 0 chooses a stride that avoids
    /// overlap.
    pub fn tick_margin(mut self, margin: usize) -> Self {
        self.props.tick_margin = margin;
        self
    }

    /// Duration of the animation from old values to new ones, in ms.
    pub fn transition_duration(mut self, ms: u64) -> Self {
        self.props.transition_duration = ms;
        self
    }

    /// Pie, donut and radar geometry (CHT-015, CHT-016).
    pub fn radial(mut self, radial: RadialOptions) -> Self {
        self.props.radial = radial;
        self
    }

    /// Sankey links and layout (CHT-030).
    pub fn sankey_options(mut self, sankey: SankeyOptions) -> Self {
        self.props.sankey = sankey;
        self
    }

    /// Turn value labels on or off; unset follows the size class.
    pub fn value_labels(mut self, on: bool) -> Self {
        self.props.value_labels = Some(on);
        self
    }

    /// Draw grid lines on both axes.
    pub fn grid(mut self, grid: bool) -> Self {
        self.props.x_axis.show_grid = grid;
        self.props.y_axis.show_grid = grid;
        self
    }

    /// Draw the grid dashed (dotted cells) or solid (CHT-034).
    pub fn grid_dashed(mut self, dashed: bool) -> Self {
        self.props.x_axis.dashed = dashed;
        self.props.y_axis.dashed = dashed;
        self
    }

    /// Divide the plot into `count` columns with vertical grid lines, the
    /// first on its left edge (CHT-034).
    pub fn grid_columns(mut self, count: usize) -> Self {
        self.props.x_axis.grid_columns = Some(count);
        self
    }

    /// Pin the value axis to `min..=max` instead of fitting the data
    /// (CHT-034); shapes outside stop at the plot's edge.
    pub fn y_domain(mut self, min: f64, max: f64) -> Self {
        self.props.y_axis.min = Some(min);
        self.props.y_axis.max = Some(max);
        self
    }

    /// Lay the category axis out for `count` evenly spaced points, the data
    /// taking the leading ones (CHT-034).
    pub fn point_count(mut self, count: usize) -> Self {
        self.props.point_count = Some(count);
        self
    }

    /// Where the vertical axis's tick labels sit: in a gutter left of the
    /// plot, or inside it beside their grid lines (CHT-034).
    pub fn y_axis_label_placement(mut self, placement: AxisLabelPlacement) -> Self {
        self.props.y_axis.placement = placement;
        self
    }

    /// How many ticks the vertical axis carries, at least two; they place
    /// the grid rows and the tick labels (CHT-034).
    pub fn y_tick_count(mut self, count: usize) -> Self {
        self.props.y_axis.tick_count = count.max(2);
        self
    }

    /// The text of each vertical-axis tick label from its value, run once
    /// at `build()` (CHT-034).
    pub fn y_tick_format(mut self, format: impl Fn(f64) -> String + Send + Sync + 'static) -> Self {
        self.deferred.y_format = Some(std::sync::Arc::new(format));
        self
    }

    /// Label `count` of the category values, spread from the first to the
    /// last, instead of every `tick_margin`-th (CHT-034).
    pub fn x_tick_count(mut self, count: usize) -> Self {
        self.props.x_axis.label_count = Some(count);
        self
    }

    /// Draw a dashed line across the plot at `value`; call again for more
    /// (CHT-034).
    pub fn reference_line(mut self, value: f64) -> Self {
        self.props.reference_lines.push(value);
        self
    }

    /// Rows kept clear above the highest value and below the lowest
    /// (CHT-034).
    pub fn y_padding(mut self, top: u16, bottom: u16) -> Self {
        self.props.headroom = (top, bottom);
        self
    }

    /// Show or hide a bar chart's band axis (its categories).
    pub fn label_axis(mut self, show: bool) -> Self {
        self.deferred.label_axis = Some(show);
        self
    }

    /// Show or hide a bar chart's value axis.
    pub fn value_axis(mut self, show: bool) -> Self {
        self.deferred.value_axis = Some(show);
        self
    }

    /// How many ticks a bar chart's value axis carries, at least two.
    pub fn value_tick_count(mut self, count: usize) -> Self {
        self.deferred.value_tick_count = Some(count.max(2));
        self
    }

    /// Where a bar chart's value tick labels sit.
    pub fn value_axis_label_placement(mut self, placement: AxisLabelPlacement) -> Self {
        self.deferred.value_placement = Some(placement);
        self
    }

    /// The text of each value tick label of a bar chart, run once at
    /// `build()`.
    pub fn value_tick_format(
        mut self,
        format: impl Fn(f64) -> String + Send + Sync + 'static,
    ) -> Self {
        self.deferred.value_format = Some(std::sync::Arc::new(format));
        self
    }

    /// Lay a bar chart's band axis out for `count` bands, the data taking
    /// the leading ones.
    pub fn band_count(mut self, count: usize) -> Self {
        self.props.band_count = Some(count);
        self
    }

    /// Label `count` of a bar chart's bands, spread from the first to the
    /// last.
    pub fn band_tick_count(mut self, count: usize) -> Self {
        self.deferred.band_tick_count = Some(count);
        self
    }

    /// Space between bands as a fraction of a band, 0 to 1 (default 0.4).
    pub fn padding_inner(mut self, padding: f64) -> Self {
        self.props.padding_inner = Some(padding);
        self
    }

    /// Space before the first band and after the last, as a fraction of a
    /// band (default 0.2).
    pub fn padding_outer(mut self, padding: f64) -> Self {
        self.props.padding_outer = Some(padding);
        self
    }

    /// Keep every band at most `width` cells wide, so a few bars across a
    /// wide chart stay narrow.
    pub fn max_band_width(mut self, width: u16) -> Self {
        self.props.max_band_width = Some(width);
        self
    }

    /// The shortest a bar is drawn, in cells, so a tiny value still shows.
    pub fn min_length(mut self, length: f64) -> Self {
        self.props.min_bar_length = length;
        self
    }

    /// A candle body's width as a fraction of its band (default 0.8).
    pub fn body_width_ratio(mut self, ratio: f32) -> Self {
        self.props.body_width_ratio = ratio;
        self
    }

    /// Whether the chart's bars run horizontally.
    fn horizontal(&self) -> bool {
        self.props.chart_type == ChartType::BarHorizontal
            || (self.props.chart_type == ChartType::BarVertical
                && self.props.growth.is_horizontal())
    }

    /// Build the ChartProps: the deferred options settle by the chart's
    /// orientation and data.
    pub fn build(self) -> ChartProps {
        let horizontal = self.horizontal();
        let Self {
            mut props,
            deferred,
        } = self;
        {
            let (value_axis, category_axis) = if horizontal {
                (&mut props.x_axis, &mut props.y_axis)
            } else {
                (&mut props.y_axis, &mut props.x_axis)
            };
            if let Some(show) = deferred.value_axis {
                value_axis.show_labels = show;
            }
            if let Some(show) = deferred.label_axis {
                category_axis.show_labels = show;
            }
            if let Some(count) = deferred.value_tick_count {
                value_axis.tick_count = count;
            }
            if let Some(placement) = deferred.value_placement {
                value_axis.placement = placement;
            }
            if let Some(count) = deferred.band_tick_count {
                category_axis.label_count = Some(count);
            }
        }
        if let Some((bullish, bearish)) = (deferred.bullish.is_some() || deferred.bearish.is_some())
            .then_some((deferred.bullish, deferred.bearish))
        {
            for series in &mut props.series {
                for point in &mut series.data {
                    if let Some(candle) = point.candle {
                        let token = if candle.is_bullish() {
                            &bullish
                        } else {
                            &bearish
                        };
                        if point.color.is_none() {
                            point.color = token.clone();
                        }
                    }
                }
            }
        }
        // Fill gradients run once, over the chart's value range: every
        // value and zero, as the typed bar builder's (CHT-013, CHT-035).
        if !deferred.gradients.is_empty() {
            let range = props
                .series
                .iter()
                .flat_map(|s| s.data.iter().map(|p| p.value))
                .filter(|v| v.is_finite())
                .fold((0.0f64, 0.0f64), |(low, high), v| (low.min(v), high.max(v)));
            for (index, stops) in &deferred.gradients {
                if let Some(series) = props.series.get_mut(*index) {
                    for point in &mut series.data {
                        let value = point.value;
                        let to_bar = move |v: f64| -> f32 {
                            if value == 0.0 {
                                0.0
                            } else {
                                (v / value).clamp(0.0, 1.0) as f32
                            }
                        };
                        point.gradient = stops(point, range, &to_bar);
                    }
                }
            }
        }
        // Tick formats run once, over the ticks the value axis will carry.
        let format = if horizontal {
            deferred.value_format
        } else {
            deferred.value_format.or(deferred.y_format)
        };
        if let Some(format) = format {
            let value_axis = if horizontal {
                &props.x_axis
            } else {
                &props.y_axis
            };
            let ticks = plot::value_ticks(&props, value_axis);
            let labelled: Vec<(f64, String)> = ticks
                .into_iter()
                .map(|value| (value, format(value)))
                .collect();
            if horizontal {
                props.x_axis.ticks = labelled;
            } else {
                props.y_axis.ticks = labelled;
            }
        }
        props
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("Charts").with_props(self.build())
    }
}

/// The edge a bar grows from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarGrowth {
    /// Vertical bars rising from the bottom.
    #[default]
    Bottom,
    /// Vertical bars hanging from the top.
    Top,
    /// Horizontal bars growing rightward from the left.
    Left,
    /// Horizontal bars growing leftward from the right.
    Right,
}

impl BarGrowth {
    /// Whether bars run horizontally.
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// Open, high, low and close values of one candlestick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    /// Opening value.
    pub open: f64,
    /// Highest value.
    pub high: f64,
    /// Lowest value.
    pub low: f64,
    /// Closing value.
    pub close: f64,
}

impl Candle {
    /// Whether the candle closed above its open.
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

/// Where a value axis draws its tick labels (CHT-034).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AxisLabelPlacement {
    /// In a gutter beside the plot, which the plot shrinks to make room for.
    #[default]
    Outside,
    /// Inside the plot beside their grid lines, so the plot keeps its full
    /// size.
    Inside,
}

/// Chart axis configuration
#[derive(Debug, Clone, PartialEq)]
pub struct ChartAxis {
    /// Axis title
    pub title: Option<String>,
    /// Minimum value (auto if None)
    pub min: Option<f64>,
    /// Maximum value (auto if None)
    pub max: Option<f64>,
    /// Whether to show grid lines
    pub show_grid: bool,
    /// Whether to show axis labels
    pub show_labels: bool,
    /// Number of tick marks
    pub tick_count: usize,
    /// Custom tick labels
    pub custom_labels: Vec<String>,
    /// Explicit ticks as (value, label), from a tick format run at
    /// `build()`; empty lets the axis choose round values (CHT-034).
    pub ticks: Vec<(f64, String)>,
    /// Where a value axis draws its tick labels (CHT-034).
    pub placement: AxisLabelPlacement,
    /// How many category labels to show, spread from the first to the
    /// last; `None` shows every `tick_margin`-th (CHT-034).
    pub label_count: Option<usize>,
    /// Draw this axis's grid lines dashed (dotted cells) rather than solid
    /// (CHT-034).
    pub dashed: bool,
    /// Divide the plot into this many columns with vertical grid lines, the
    /// first on its left edge (CHT-034).
    pub grid_columns: Option<usize>,
}

impl Default for ChartAxis {
    fn default() -> Self {
        Self {
            title: None,
            min: None,
            max: None,
            show_grid: true,
            show_labels: true,
            tick_count: 5,
            custom_labels: Vec::new(),
            ticks: Vec::new(),
            placement: AxisLabelPlacement::Outside,
            label_count: None,
            dashed: true,
            grid_columns: None,
        }
    }
}

/// Chart legend configuration
#[derive(Debug, Clone, PartialEq)]
pub struct ChartLegend {
    /// Whether to show the legend
    pub visible: bool,
    /// Legend position
    pub position: LegendPosition,
    /// Maximum width for legend
    pub max_width: Option<u16>,
}

impl Default for ChartLegend {
    fn default() -> Self {
        Self {
            visible: true,
            position: LegendPosition::Right,
            max_width: None,
        }
    }
}

/// Legend position options
#[derive(Debug, Clone, PartialEq)]
pub enum LegendPosition {
    /// Top of chart
    Top,
    /// Bottom of chart
    Bottom,
    /// Left of chart
    Left,
    /// Right of chart
    Right,
    /// Floating inside chart
    Floating(u16, u16),
}

/// Tooltip text a point carries in place of the defaults (CHT-018): the
/// typed builders' `tooltip_title`, `tooltip_value`, `tooltip_value_color`
/// and `tooltip_content` closures are run once at `build()` into these.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PointTooltip {
    /// The title row, instead of the category label.
    pub title: Option<String>,
    /// The value text, instead of the formatted value.
    pub value: Option<String>,
    /// The color token of the value text.
    pub color: Option<String>,
    /// Whole content lines, instead of the series rows.
    pub lines: Vec<String>,
}

/// Chart data point with value and optional label
#[derive(Debug, Clone, PartialEq)]
pub struct DataPoint {
    /// The numeric value
    pub value: f64,
    /// Optional label for this data point
    pub label: Option<String>,
    /// Optional color override for this point
    pub color: Option<String>,
    /// Optional metadata for tooltips/interactions
    pub metadata: HashMap<String, String>,
    /// Open, high, low and close for candlestick charts; `value` holds the
    /// close so the point still works in every other chart type.
    pub candle: Option<Candle>,
    /// The point's position on a numeric x axis (scatter charts, CHT-033);
    /// `None` places the point by its index.
    pub x: Option<f64>,
    /// Color token of the point's value label; `None` takes the bar's color
    /// (CHT-013).
    pub label_color: Option<String>,
    /// A bar's fill gradient as (offset, color token) stops from its base
    /// (0) to its tip (1); empty fills the bar with its color (CHT-013).
    pub gradient: Vec<(f32, String)>,
    /// Tooltip text that replaces the defaults for this point (CHT-018).
    pub tooltip: PointTooltip,
    /// The text of a bar's value label, instead of the formatted value
    /// (CHT-013); it is not tooltip metadata.
    pub value_label: Option<String>,
}

impl DataPoint {
    /// Create a new data point with just a value
    pub fn new(value: f64) -> Self {
        Self {
            value,
            label: None,
            color: None,
            metadata: HashMap::new(),
            candle: None,
            x: None,
            label_color: None,
            gradient: Vec::new(),
            tooltip: PointTooltip::default(),
            value_label: None,
        }
    }

    /// The text of a bar's value label (CHT-013).
    pub fn with_value_label(mut self, label: impl Into<String>) -> Self {
        self.value_label = Some(label.into());
        self
    }

    /// Create a data point with value and label
    pub fn with_label(value: f64, label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::new(value)
        }
    }

    /// A point at numeric `x` with value `y`, for scatter charts (CHT-033).
    pub fn xy(x: f64, y: f64) -> Self {
        Self {
            x: Some(x),
            ..Self::new(y)
        }
    }

    /// Create a candlestick point from open, high, low and close.
    pub fn candle(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            candle: Some(Candle {
                open,
                high,
                low,
                close,
            }),
            ..Self::new(close)
        }
    }

    /// Set color for this data point
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Color token of the point's value label (CHT-013).
    pub fn with_label_color(mut self, color: impl Into<String>) -> Self {
        self.label_color = Some(color.into());
        self
    }

    /// A bar's fill gradient as (offset, token) stops from base to tip
    /// (CHT-013).
    pub fn with_gradient(mut self, stops: Vec<(f32, String)>) -> Self {
        self.gradient = stops;
        self
    }

    /// Tooltip text that replaces the defaults for this point (CHT-018).
    pub fn with_tooltip(mut self, tooltip: PointTooltip) -> Self {
        self.tooltip = tooltip;
        self
    }

    /// Add metadata to this data point
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Chart data series containing multiple data points
#[derive(Debug, Clone, PartialEq)]
pub struct DataSeries {
    /// Name of this data series
    pub name: String,
    /// Data points in this series
    pub data: Vec<DataPoint>,
    /// Color for this series
    pub color: Option<String>,
    /// Whether this series is visible
    pub visible: bool,
    /// Line style for line charts
    pub line_style: LineStyle,
    /// Fill style for area charts; new series default to solid fill.
    pub fill_style: FillStyle,
    /// Fill color token of an area series, `None` filling with the stroke
    /// color (CHT-012), and of a bar series' bars, `None` taking the series
    /// color (CHT-013).
    pub fill: Option<String>,
    /// How much of the fill color shows over the chart background, 0 to 1
    /// (CHT-012).
    pub fill_opacity: f32,
    /// The series' own curve; `None` takes the chart's (CHT-012).
    pub curve: Option<Curve>,
    /// Whether this series draws a dot at every point; `None` takes the
    /// chart's setting (CHT-012).
    pub dots: Option<bool>,
}

impl DataSeries {
    /// Create a new data series
    pub fn new(name: impl Into<String>, data: Vec<DataPoint>) -> Self {
        Self {
            name: name.into(),
            data,
            color: None,
            visible: true,
            line_style: LineStyle::Solid,
            fill_style: FillStyle::Solid,
            fill: None,
            fill_opacity: 0.4,
            curve: None,
            dots: None,
        }
    }

    /// Set color for this series
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Fill color token of an area series (CHT-012).
    pub fn with_fill(mut self, color: impl Into<String>) -> Self {
        self.fill = Some(color.into());
        self
    }

    /// How much of the fill shows over the background, 0 to 1 (CHT-012).
    pub fn with_fill_opacity(mut self, opacity: f32) -> Self {
        self.fill_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// The series' own curve (CHT-012).
    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.curve = Some(curve);
        self
    }

    /// Draw a dot at every point of this series, or not (CHT-012).
    pub fn with_dots(mut self, dots: bool) -> Self {
        self.dots = Some(dots);
        self
    }

    /// Set line style for this series
    pub fn with_line_style(mut self, style: LineStyle) -> Self {
        self.line_style = style;
        self
    }

    /// Set fill style for this series
    pub fn with_fill_style(mut self, style: FillStyle) -> Self {
        self.fill_style = style;
        self
    }
}

/// Line styles for line charts
#[derive(Debug, Clone, PartialEq)]
pub enum LineStyle {
    /// Solid line
    Solid,
    /// Dashed line
    Dashed,
    /// Dotted line
    Dotted,
    /// No line (points only)
    None,
}

/// Fill styles for area charts
#[derive(Debug, Clone, PartialEq)]
pub enum FillStyle {
    /// No fill
    None,
    /// Solid fill
    Solid,
    /// Gradient fill
    Gradient,
    /// Pattern fill
    Pattern(String),
}

/// Chart types supported by the chart widget
#[derive(Debug, Clone, PartialEq)]
pub enum ChartType {
    /// Vertical bar chart
    BarVertical,
    /// Horizontal bar chart
    BarHorizontal,
    /// Line chart
    Line,
    /// Area chart (filled line chart)
    Area,
    /// Pie chart
    Pie,
    /// Donut chart (pie chart with hole)
    Donut,
    /// Scatter plot
    Scatter,
    /// Sankey diagram: nodes in columns joined by flow ribbons (CHT-030)
    Sankey,
    /// Candlestick chart: wick from low to high, body from open to close
    Candlestick,
    /// Radar chart: one spoke per category, one polygon per series
    Radar,
}

/// Geometry of pie, donut and radar charts. Radii are fractions of the
/// largest circle the plot area holds, so they keep their proportions at
/// every size class.
#[derive(Debug, Clone, PartialEq)]
pub struct RadialOptions {
    /// Inner radius as a fraction of the outer one; `None` is 0 for a pie
    /// and 0.5 for a donut (CHT-015).
    pub inner_radius: Option<f64>,
    /// Outer radius as a fraction of the largest circle that fits.
    pub outer_radius: f64,
    /// Gap between adjacent slices, in radians.
    pub pad_angle: f64,
    /// Columns between the circle and its side labels. At 0 a label touches
    /// the circle, and one whose leader finds no room there sits one column
    /// further out.
    pub label_gap: u16,
    /// Radar grid levels: concentric polygons at equal steps (CHT-016).
    pub grid_levels: usize,
    /// Radar grid drawn at the medium and large size classes.
    pub grid: bool,
    /// The radar scale's maximum; `None` uses the largest value.
    pub max_value: Option<f64>,
    /// Radar fill color token per series (indexed like the series); a
    /// missing entry fills with the series color, `"none"` leaves the
    /// polygon unfilled.
    pub fills: Vec<Option<String>>,
}

impl Default for RadialOptions {
    fn default() -> Self {
        Self {
            inner_radius: None,
            outer_radius: 1.0,
            pad_angle: 0.0,
            label_gap: 2,
            grid_levels: 4,
            grid: true,
            max_value: None,
            fills: Vec::new(),
        }
    }
}

/// A styled line of a Sankey node's label (CHT-029).
#[derive(Debug, Clone, PartialEq)]
pub struct SankeyLabel {
    /// The line's text.
    pub text: String,
    /// Its color token; unset takes the default text color.
    pub color: Option<String>,
}

impl SankeyLabel {
    /// A line of `text` in the default text color.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
        }
    }

    /// Draw the line in the color token `color`.
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// A Sankey chart's links and layout (CHT-030). The nodes are the first
/// series' points: each point's label names its node and its color token
/// colors it. A node's throughput, shown in its tooltip and large label,
/// is the larger of its incoming and outgoing link totals, not its point's
/// value.
#[derive(Debug, Clone, PartialEq)]
pub struct SankeyOptions {
    /// The flows between nodes, by node index.
    pub links: Vec<SankeyLink>,
    /// Which column each node takes.
    pub node_align: SankeyAlign,
    /// Relaxation passes that move nodes toward their flows.
    pub iterations: usize,
    /// How values map to node heights and link widths.
    pub value_scale: SankeyValueScale,
    /// Node width in columns.
    pub node_width: u16,
    /// Rows between the nodes of a column.
    pub node_padding: u16,
    /// How much of its color a ribbon keeps over the chart background, 0
    /// to 1.
    pub link_opacity: f32,
    /// The narrowest a ribbon is drawn, in rows.
    pub min_link_width: f32,
    /// Columns between a first- or last-layer node and its label.
    pub label_gap: u16,
    /// Each node's throughput text; a missing entry shows the number.
    pub value_labels: Vec<String>,
    /// Each node's label lines; a missing or empty entry shows its name.
    pub labels: Vec<Vec<SankeyLabel>>,
}

impl Default for SankeyOptions {
    fn default() -> Self {
        Self {
            links: Vec::new(),
            node_align: SankeyAlign::default(),
            iterations: 6,
            value_scale: SankeyValueScale::default(),
            node_width: 2,
            node_padding: 1,
            link_opacity: 0.3,
            min_link_width: 0.25,
            label_gap: 1,
            value_labels: Vec::new(),
            labels: Vec::new(),
        }
    }
}

/// Props for the Chart component
#[derive(Clone, PartialEq, Debug)]
pub struct ChartProps {
    /// Type of chart to render
    pub chart_type: ChartType,
    /// Data series to display
    pub series: Vec<DataSeries>,
    /// Chart title
    pub title: Option<String>,
    /// Chart width in cells; 0 fills the allotted rectangle (CHT-021)
    pub width: u16,
    /// Chart height in cells; 0 fills the allotted rectangle (CHT-021)
    pub height: u16,
    /// X-axis configuration
    pub x_axis: ChartAxis,
    /// Y-axis configuration
    pub y_axis: ChartAxis,
    /// Legend configuration
    pub legend: ChartLegend,
    /// Color palette for automatic coloring
    pub color_palette: Vec<String>,
    /// Whether to animate chart rendering
    pub animated: bool,
    /// Animation duration in milliseconds
    pub animation_duration: u64,
    /// Whether to show tooltips on hover
    pub show_tooltips: bool,
    /// Custom CSS classes
    pub class: Option<String>,
    /// The edge bars grow from
    pub growth: BarGrowth,
    /// Stack series (bars end to end, areas on top of each other)
    pub stacked: bool,
    /// Curve style for line and area strokes
    pub curve: Curve,
    /// Draw a dot at each line-chart point
    pub dots: bool,
    /// Forced size class; `None` chooses from the rectangle (CHT-024)
    pub size_class: Option<SizeClass>,
    /// Render with ASCII glyphs only (CHT-028)
    pub ascii: bool,
    /// Show every n-th axis label; 0 picks a stride that avoids overlap
    pub tick_margin: usize,
    /// Milliseconds a data change takes to animate (CHT-022)
    pub transition_duration: u64,
    /// Value labels on bars; `None` follows the size class
    pub value_labels: Option<bool>,
    /// Pie, donut and radar geometry
    pub radial: RadialOptions,
    /// Sankey links and layout
    pub sankey: SankeyOptions,
    /// The name the screen reader is told; `None` falls back to the title
    /// (CHT-036)
    pub aria_label: Option<String>,
    /// Lay the category axis out for this many points, the data taking the
    /// leading ones; `None` uses the data's own length (CHT-034)
    pub point_count: Option<usize>,
    /// Lay a bar chart's band axis out for this many bands (CHT-034)
    pub band_count: Option<usize>,
    /// Values at which a dashed reference line crosses the plot (CHT-034)
    pub reference_lines: Vec<f64>,
    /// Rows kept clear above the highest value and below the lowest
    /// (CHT-034)
    pub headroom: (u16, u16),
    /// Space between bands as a fraction of a band; `None` is 0.4
    /// (CHT-013)
    pub padding_inner: Option<f64>,
    /// Space outside the first and last band as a fraction of a band;
    /// `None` is 0.2 (CHT-013)
    pub padding_outer: Option<f64>,
    /// The widest a band is drawn, in cells (CHT-013, CHT-014)
    pub max_band_width: Option<u16>,
    /// The shortest a bar is drawn, in cells (CHT-013)
    pub min_bar_length: f64,
    /// A candle body's width as a fraction of its band (CHT-014)
    pub body_width_ratio: f32,
}

impl Props for ChartProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Default for ChartProps {
    fn default() -> Self {
        Self {
            chart_type: ChartType::BarVertical,
            series: Vec::new(),
            title: None,
            width: 0,
            height: 0,
            x_axis: ChartAxis::default(),
            y_axis: ChartAxis::default(),
            legend: ChartLegend::default(),
            color_palette: (1..=5).map(|i| format!("chart-{i}")).collect(),
            animated: false,
            animation_duration: 1000,
            show_tooltips: true,
            class: None,
            growth: BarGrowth::Bottom,
            stacked: false,
            curve: Curve::Natural,
            dots: false,
            size_class: None,
            ascii: false,
            tick_margin: 0,
            transition_duration: 200,
            value_labels: None,
            radial: RadialOptions::default(),
            sankey: SankeyOptions::default(),
            aria_label: None,
            point_count: None,
            band_count: None,
            reference_lines: Vec::new(),
            headroom: (0, 0),
            padding_inner: None,
            padding_outer: None,
            max_band_width: None,
            min_bar_length: 0.0,
            body_width_ratio: 0.8,
        }
    }
}

/// State for the Chart component
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChartState {
    /// Current animation progress (0.0 to 1.0)
    pub animation_progress: f32,
    /// Whether animation is currently running
    pub animating: bool,
    /// Currently hovered data point
    pub hovered_point: Option<(usize, usize)>, // series_index, point_index
    /// Tooltip content and position
    pub tooltip: Option<(String, u16, u16)>, // content, x, y
}

/// Chart component for data visualization
pub struct Chart {
    /// The props as last handed down, shared with the live chart. The
    /// runtime only calls `update` when props change, so a frame that
    /// changes nothing costs neither a comparison nor a copy of every point.
    shared: std::sync::Arc<ChartProps>,
}

mod live;
pub mod mask;
pub mod plot;
pub mod typed;

pub use plot::sankey::{SankeyAlign, SankeyLink, SankeyValueScale};
pub use typed::{
    AreaChartBuilder, BarChartBuilder, CandlestickChartBuilder, DonutChartBuilder,
    LineChartBuilder, PieChartBuilder, RadarChartBuilder, SankeyChartBuilder, ScatterChartBuilder,
};

impl Component for Chart {
    type Props = ChartProps;
    type State = ChartState;

    fn new(props: Self::Props) -> Self {
        Self {
            shared: std::sync::Arc::new(props),
        }
    }

    fn update(&mut self, props: &Self::Props, _state: &mut Self::State) -> bool {
        self.shared = std::sync::Arc::new(props.clone());
        true
    }

    fn render(&self, _props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveChart>(live::LiveProps {
            config: self.shared.clone(),
            seed: state.clone(),
        })
    }
}

/// Helper functions for creating chart props
impl ChartProps {
    /// Create a new bar chart
    pub fn bar_chart(series: Vec<DataSeries>) -> Self {
        Self {
            chart_type: ChartType::BarVertical,
            series,
            ..Default::default()
        }
    }

    /// Create a new line chart
    pub fn line_chart(series: Vec<DataSeries>) -> Self {
        Self {
            chart_type: ChartType::Line,
            series,
            ..Default::default()
        }
    }

    /// Create a new pie chart
    pub fn pie_chart(series: Vec<DataSeries>) -> Self {
        Self {
            chart_type: ChartType::Pie,
            series,
            ..Default::default()
        }
    }

    /// Set chart dimensions
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set chart title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Enable animations
    pub fn with_animation(mut self, duration_ms: u64) -> Self {
        self.animated = true;
        self.animation_duration = duration_ms;
        self
    }

    /// Configure axes
    pub fn with_axes(mut self, x_title: Option<String>, y_title: Option<String>) -> Self {
        self.x_axis.title = x_title;
        self.y_axis.title = y_title;
        self
    }

    /// Set custom color palette
    pub fn with_colors(mut self, colors: Vec<String>) -> Self {
        self.color_palette = colors;
        self
    }
}
