//! Chart widget builder
//!
//! `builder::chart()` builds a Chart element from series made by hand. It
//! wraps [`ChartsBuilder`], so every chart type and every option the typed
//! builders offer is reachable here too (CHT-035).

use crate::component::Element;
use crate::widgets::display::{
    AxisLabelPlacement, BarGrowth, ChartAxis, ChartLegend, ChartProps, ChartType, ChartsBuilder,
    Curve, DataPoint, DataSeries, LegendPosition, RadialOptions, SankeyLink, SankeyOptions,
    SizeClass,
};

/// Create a Chart with fluent configuration
pub fn chart() -> ChartBuilder {
    ChartBuilder::new()
}

/// Builder for Chart components with fluent API
#[derive(Clone, Debug)]
pub struct ChartBuilder {
    inner: ChartsBuilder,
    class: Option<String>,
}

/// A method that hands its arguments to the same method of the wrapped
/// [`ChartsBuilder`].
macro_rules! forward {
    ($($(#[$doc:meta])* $name:ident($($arg:ident: $ty:ty),*);)*) => {
        $(
            $(#[$doc])*
            pub fn $name(mut self, $($arg: $ty),*) -> Self {
                self.inner = self.inner.$name($($arg),*);
                self
            }
        )*
    };
}

impl ChartBuilder {
    fn new() -> Self {
        Self {
            inner: ChartsBuilder::new(),
            class: None,
        }
    }

    /// Set chart type
    pub fn chart_type(mut self, chart_type: ChartType) -> Self {
        self.inner = self.inner.chart_type(chart_type);
        self
    }

    /// Create a bar chart
    pub fn bar_chart(self) -> Self {
        self.chart_type(ChartType::BarVertical)
    }

    /// Create a horizontal bar chart
    pub fn horizontal_bar_chart(self) -> Self {
        self.chart_type(ChartType::BarHorizontal)
    }

    /// Create a line chart
    pub fn line_chart(self) -> Self {
        self.chart_type(ChartType::Line)
    }

    /// Create a pie chart
    pub fn pie_chart(self) -> Self {
        self.chart_type(ChartType::Pie)
    }

    /// Create a donut chart
    pub fn donut_chart(self) -> Self {
        self.chart_type(ChartType::Donut)
    }

    /// Create an area chart
    pub fn area_chart(self) -> Self {
        self.chart_type(ChartType::Area)
    }

    /// Create a scatter plot
    pub fn scatter_plot(self) -> Self {
        self.chart_type(ChartType::Scatter)
    }

    /// Create a scatter chart (the same as [`Self::scatter_plot`])
    pub fn scatter_chart(self) -> Self {
        self.chart_type(ChartType::Scatter)
    }

    /// Create a candlestick chart; add candles with [`Self::candle_series`]
    pub fn candlestick_chart(self) -> Self {
        self.chart_type(ChartType::Candlestick)
    }

    /// Create a radar chart
    pub fn radar_chart(self) -> Self {
        self.chart_type(ChartType::Radar)
    }

    /// Create a Sankey chart; add its nodes and links with
    /// [`Self::sankey_nodes`]
    pub fn sankey_chart(self) -> Self {
        self.chart_type(ChartType::Sankey)
    }

    /// Add a data series
    pub fn series(mut self, series: DataSeries) -> Self {
        self.inner = self.inner.series(series);
        self
    }

    /// Add multiple data series
    pub fn series_list(mut self, series: Vec<DataSeries>) -> Self {
        self.inner = self.inner.with_series(series);
        self
    }

    /// Add a simple data series from values
    pub fn simple_series(self, name: &str, values: Vec<f64>) -> Self {
        let data_points: Vec<DataPoint> = values.into_iter().map(DataPoint::new).collect();
        self.series(DataSeries::new(name, data_points))
    }

    /// Add a labeled data series
    pub fn labeled_series(self, name: &str, data: Vec<(f64, &str)>) -> Self {
        let data_points: Vec<DataPoint> = data
            .into_iter()
            .map(|(value, label)| DataPoint::with_label(value, label))
            .collect();
        self.series(DataSeries::new(name, data_points))
    }

    /// Add a series of candles as (open, high, low, close)
    pub fn candle_series(self, name: &str, candles: Vec<(f64, f64, f64, f64)>) -> Self {
        let data_points: Vec<DataPoint> = candles
            .into_iter()
            .map(|(open, high, low, close)| DataPoint::candle(open, high, low, close))
            .collect();
        self.series(DataSeries::new(name, data_points))
    }

    /// A Sankey chart's nodes by name and its links as (from, to, value)
    /// by node index
    pub fn sankey_nodes<N: Into<String>>(
        mut self,
        nodes: impl IntoIterator<Item = N>,
        links: Vec<(usize, usize, f64)>,
    ) -> Self {
        let points: Vec<DataPoint> = nodes
            .into_iter()
            .map(|name| DataPoint::with_label(0.0, name))
            .collect();
        self.inner = self
            .inner
            .series(DataSeries::new("nodes", points))
            .sankey_options(SankeyOptions {
                links: links
                    .into_iter()
                    .map(|(from, to, value)| SankeyLink::new(from, to, value))
                    .collect(),
                ..SankeyOptions::default()
            });
        self
    }

    /// Set chart title
    pub fn title(mut self, title: &str) -> Self {
        self.inner = self.inner.title(title);
        self
    }

    /// Set chart dimensions
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.inner = self.inner.size(width, height);
        self
    }

    /// Configure axes
    pub fn axes(mut self, x_title: Option<String>, y_title: Option<String>) -> Self {
        let mut props = self.inner.clone().build();
        props.x_axis.title = x_title;
        props.y_axis.title = y_title;
        self.inner = self.inner.x_axis(props.x_axis).y_axis(props.y_axis);
        self
    }

    /// Set custom color palette
    pub fn colors(mut self, colors: Vec<String>) -> Self {
        self.inner = self.inner.color_palette(colors);
        self
    }

    /// Enable animation
    pub fn animated(mut self, duration_ms: u64) -> Self {
        self.inner = self.inner.animated(true).animation_duration(duration_ms);
        self
    }

    /// Configure legend
    pub fn legend(mut self, visible: bool, position: LegendPosition) -> Self {
        self.inner = self.inner.legend(ChartLegend {
            visible,
            position,
            ..ChartLegend::default()
        });
        self
    }

    /// The tooltip title of every point of the series added last, from the
    /// point (CHT-018, CHT-035).
    pub fn tooltip_title<S: Into<String>>(mut self, title: impl Fn(&DataPoint) -> S) -> Self {
        self.inner = self.inner.tooltip_title(title);
        self
    }

    /// The tooltip value text of every point of the series added last, from
    /// the point and its value (CHT-018).
    pub fn tooltip_value<S: Into<String>>(mut self, value: impl Fn(&DataPoint, f64) -> S) -> Self {
        self.inner = self.inner.tooltip_value(value);
        self
    }

    /// The color token of the tooltip value text of every point of the
    /// series added last (CHT-018).
    pub fn tooltip_value_color<S: Into<String>>(
        mut self,
        color: impl Fn(&DataPoint, f64) -> S,
    ) -> Self {
        self.inner = self.inner.tooltip_value_color(color);
        self
    }

    /// The whole tooltip content of every point of the series added last,
    /// as lines (CHT-018).
    pub fn tooltip_content<S: Into<String>>(
        mut self,
        content: impl Fn(&DataPoint) -> Vec<S>,
    ) -> Self {
        self.inner = self.inner.tooltip_content(content);
        self
    }

    /// The value-label color token of every point of the series added last
    /// (CHT-013).
    pub fn label_color<S: Into<String>>(mut self, color: impl Fn(&DataPoint) -> S) -> Self {
        self.inner = self.inner.label_color(color);
        self
    }

    /// Each bar's own fill color token, from the point (CHT-013).
    pub fn fill_with<S: Into<String>>(mut self, fill: impl Fn(&DataPoint) -> S) -> Self {
        self.inner = self.inner.fill_with(fill);
        self
    }

    /// Each bar's fill gradient from base to tip, run once at `build()`
    /// over the chart's value range (CHT-013).
    pub fn fill_gradient<S: Into<String>>(
        mut self,
        stops: impl Fn(&DataPoint, (f64, f64), &dyn Fn(f64) -> f32) -> Vec<(f32, S)>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.inner = self.inner.fill_gradient(stops);
        self
    }

    /// Set CSS classes
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self.inner = self.inner.class(class);
        self
    }

    forward! {
        /// The name the screen reader is told, instead of the title (CHT-036).
        aria_label(label: impl Into<String>);
        /// Chart width in cells; 0 fills the allotted rectangle.
        width(width: u16);
        /// Chart height in cells; 0 fills the allotted rectangle.
        height(height: u16);
        /// Configure the drawing's horizontal axis.
        x_axis(axis: ChartAxis);
        /// Configure the drawing's vertical axis.
        y_axis(axis: ChartAxis);
        /// Show or hide the drawing's horizontal axis line and labels.
        x_axis_labels(show: bool);
        /// Show or hide the drawing's vertical axis line and labels.
        y_axis_labels(show: bool);
        /// Hide the legend.
        no_legend();
        /// Set color palette.
        color_palette(colors: Vec<String>);
        /// Animation duration in milliseconds.
        animation_duration(duration: u64);
        /// Enable or disable tooltips.
        show_tooltips(show: bool);
        /// Hover, keyboard selection and the tooltip, on or off.
        interactive(interactive: bool);
        /// Which edge bars grow from.
        growth(growth: BarGrowth);
        /// Which edge bars grow from (the reference's name).
        alignment(alignment: BarGrowth);
        /// Stack series end to end or on top of each other.
        stacked(stacked: bool);
        /// Curve style for line and area strokes.
        curve(curve: Curve);
        /// Smooth spline strokes.
        natural();
        /// Straight strokes.
        linear();
        /// Step strokes holding each value until the next point.
        step_after();
        /// Draw a dot at every line-chart point, or not.
        dots(dots: bool);
        /// Draw a dot at every line-chart point.
        dot();
        /// Name of the series added last.
        name(name: impl Into<String>);
        /// Stroke color token of the series added last.
        stroke(token: impl Into<String>);
        /// Fill color token of the series added last.
        fill(token: impl Into<String>);
        /// Color token for candles that close above their open.
        bullish(token: impl Into<String>);
        /// Color token for candles that close below their open.
        bearish(token: impl Into<String>);
        /// Force a size class.
        size_class(class: SizeClass);
        /// Render with ASCII glyphs only.
        ascii(ascii: bool);
        /// Show every n-th category label; 0 avoids overlap automatically.
        tick_margin(margin: usize);
        /// Duration of the animation from old values to new ones, in ms.
        transition_duration(ms: u64);
        /// Pie, donut and radar geometry.
        radial(radial: RadialOptions);
        /// Sankey links and layout.
        sankey_options(sankey: SankeyOptions);
        /// Turn value labels on or off.
        value_labels(on: bool);
        /// Draw grid lines on both axes.
        grid(grid: bool);
        /// Draw the grid dashed or solid.
        grid_dashed(dashed: bool);
        /// Divide the plot into `count` columns with vertical grid lines.
        grid_columns(count: usize);
        /// Pin the value axis to `min..=max`.
        y_domain(min: f64, max: f64);
        /// Lay the category axis out for `count` points.
        point_count(count: usize);
        /// Where the vertical axis's tick labels sit.
        y_axis_label_placement(placement: AxisLabelPlacement);
        /// How many ticks the vertical axis carries.
        y_tick_count(count: usize);
        /// The text of each vertical-axis tick label from its value.
        y_tick_format(format: impl Fn(f64) -> String + Send + Sync + 'static);
        /// Label `count` of the category values, spread evenly.
        x_tick_count(count: usize);
        /// Draw a dashed reference line across the plot at `value`.
        reference_line(value: f64);
        /// Rows kept clear above the highest value and below the lowest.
        y_padding(top: u16, bottom: u16);
        /// Show or hide a bar chart's band axis.
        label_axis(show: bool);
        /// Show or hide a bar chart's value axis.
        value_axis(show: bool);
        /// How many ticks a bar chart's value axis carries.
        value_tick_count(count: usize);
        /// Where a bar chart's value tick labels sit.
        value_axis_label_placement(placement: AxisLabelPlacement);
        /// The text of each value tick label of a bar chart from its value.
        value_tick_format(format: impl Fn(f64) -> String + Send + Sync + 'static);
        /// Lay a bar chart's band axis out for `count` bands.
        band_count(count: usize);
        /// Label `count` of a bar chart's bands, spread evenly.
        band_tick_count(count: usize);
        /// Space between bands as a fraction of a band.
        padding_inner(padding: f64);
        /// Space outside the first and last band as a fraction of a band.
        padding_outer(padding: f64);
        /// The widest a band is drawn, in cells.
        max_band_width(width: u16);
        /// The shortest a bar is drawn, in cells.
        min_length(length: f64);
        /// A candle body's width as a fraction of its band.
        body_width_ratio(ratio: f32);
    }

    /// The props as they stand, for callers that compose further.
    pub fn props(&self) -> ChartProps {
        self.inner.clone().build()
    }

    /// Build the Chart element
    pub fn build(self) -> Element {
        self.build_with_name("Chart")
    }

    /// Build the Chart element with a custom component name
    pub fn build_with_name(self, component_name: &str) -> Element {
        let props = self.inner.build();
        let mut element = Element::component_with_props(component_name, props);
        if let Some(class) = self.class {
            element = element.with_class(&class);
        }
        element
    }
}

impl From<ChartBuilder> for Element {
    fn from(builder: ChartBuilder) -> Self {
        builder.build()
    }
}
