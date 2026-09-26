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

/// Builder for creating Chart components with a fluent API
#[derive(Clone, Debug)]
pub struct ChartsBuilder {
    chart_type: ChartType,
    series: Vec<DataSeries>,
    title: Option<String>,
    width: u16,
    height: u16,
    x_axis: ChartAxis,
    y_axis: ChartAxis,
    legend: ChartLegend,
    color_palette: Vec<String>,
    animated: bool,
    animation_duration: u64,
    show_tooltips: bool,
    class: Option<String>,
    growth: BarGrowth,
    stacked: bool,
    curve: Curve,
    dots: bool,
    size_class: Option<SizeClass>,
    ascii: bool,
    tick_margin: usize,
    transition_duration: u64,
    value_labels: Option<bool>,
    radial: RadialOptions,
    sankey: SankeyOptions,
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
        self.chart_type = chart_type;
        self
    }

    /// Add a data series
    pub fn series(mut self, series: DataSeries) -> Self {
        self.series.push(series);
        self
    }

    /// Add multiple data series
    pub fn with_series(mut self, series: Vec<DataSeries>) -> Self {
        self.series.extend(series);
        self
    }

    /// Set the chart title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the chart width
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Set the chart height
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Set the chart size
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Configure the X-axis
    pub fn x_axis(mut self, axis: ChartAxis) -> Self {
        self.x_axis = axis;
        self
    }

    /// Configure the Y-axis
    pub fn y_axis(mut self, axis: ChartAxis) -> Self {
        self.y_axis = axis;
        self
    }

    /// Configure the legend
    pub fn legend(mut self, legend: ChartLegend) -> Self {
        self.legend = legend;
        self
    }

    /// Hide the legend
    pub fn no_legend(mut self) -> Self {
        self.legend.visible = false;
        self
    }

    /// Set color palette
    pub fn color_palette(mut self, colors: Vec<String>) -> Self {
        self.color_palette = colors;
        self
    }

    /// Enable animation
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Set animation duration in milliseconds
    pub fn animation_duration(mut self, duration: u64) -> Self {
        self.animation_duration = duration;
        self
    }

    /// Enable or disable tooltips
    pub fn show_tooltips(mut self, show: bool) -> Self {
        self.show_tooltips = show;
        self
    }

    /// Add CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Which edge bars grow from: Bottom or Top for vertical bars, Left or
    /// Right for horizontal ones.
    pub fn growth(mut self, growth: BarGrowth) -> Self {
        self.growth = growth;
        self
    }

    /// Stack series instead of grouping them side by side (bars) or
    /// overlaying them (areas).
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }

    /// Curve style for line and area strokes.
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    /// Draw a dot at every line-chart point.
    pub fn dots(mut self, dots: bool) -> Self {
        self.dots = dots;
        self
    }

    /// Force a size class regardless of the rectangle the chart gets.
    pub fn size_class(mut self, class: SizeClass) -> Self {
        self.size_class = Some(class);
        self
    }

    /// Render with ASCII glyphs only.
    pub fn ascii(mut self, ascii: bool) -> Self {
        self.ascii = ascii;
        self
    }

    /// Show every n-th axis label; 0 chooses a stride that avoids overlap.
    pub fn tick_margin(mut self, margin: usize) -> Self {
        self.tick_margin = margin;
        self
    }

    /// Duration of the animation from old values to new ones, in ms.
    pub fn transition_duration(mut self, ms: u64) -> Self {
        self.transition_duration = ms;
        self
    }

    /// Pie, donut and radar geometry (CHT-015, CHT-016).
    pub fn radial(mut self, radial: RadialOptions) -> Self {
        self.radial = radial;
        self
    }

    /// Sankey links and layout (CHT-030).
    pub fn sankey_options(mut self, sankey: SankeyOptions) -> Self {
        self.sankey = sankey;
        self
    }

    /// Turn value labels on or off; unset follows the size class.
    pub fn value_labels(mut self, on: bool) -> Self {
        self.value_labels = Some(on);
        self
    }

    /// Build the ChartProps
    pub fn build(self) -> ChartProps {
        ChartProps {
            chart_type: self.chart_type,
            series: self.series,
            title: self.title,
            width: self.width,
            height: self.height,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
            legend: self.legend,
            color_palette: self.color_palette,
            animated: self.animated,
            animation_duration: self.animation_duration,
            show_tooltips: self.show_tooltips,
            class: self.class,
            growth: self.growth,
            stacked: self.stacked,
            curve: self.curve,
            dots: self.dots,
            size_class: self.size_class,
            ascii: self.ascii,
            tick_margin: self.tick_margin,
            transition_duration: self.transition_duration,
            value_labels: self.value_labels,
            radial: self.radial,
            sankey: self.sankey,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("Charts").with_props(self.build())
    }
}

impl Default for ChartsBuilder {
    fn default() -> Self {
        let props = ChartProps::default();
        Self {
            chart_type: props.chart_type,
            series: props.series,
            title: props.title,
            width: props.width,
            height: props.height,
            x_axis: props.x_axis,
            y_axis: props.y_axis,
            legend: props.legend,
            color_palette: props.color_palette,
            animated: props.animated,
            animation_duration: props.animation_duration,
            show_tooltips: props.show_tooltips,
            class: props.class,
            growth: props.growth,
            stacked: props.stacked,
            curve: props.curve,
            dots: props.dots,
            size_class: props.size_class,
            ascii: props.ascii,
            tick_margin: props.tick_margin,
            transition_duration: props.transition_duration,
            value_labels: props.value_labels,
            radial: props.radial,
            sankey: props.sankey,
        }
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
        }
    }

    /// Create a data point with value and label
    pub fn with_label(value: f64, label: impl Into<String>) -> Self {
        Self {
            value,
            label: Some(label.into()),
            color: None,
            metadata: HashMap::new(),
            candle: None,
        }
    }

    /// Create a candlestick point from open, high, low and close.
    pub fn candle(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            value: close,
            label: None,
            color: None,
            metadata: HashMap::new(),
            candle: Some(Candle {
                open,
                high,
                low,
                close,
            }),
        }
    }

    /// Set color for this data point
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
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
        }
    }

    /// Set color for this series
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
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
            dots: true,
            size_class: None,
            ascii: false,
            tick_margin: 0,
            transition_duration: 200,
            value_labels: None,
            radial: RadialOptions::default(),
            sankey: SankeyOptions::default(),
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
