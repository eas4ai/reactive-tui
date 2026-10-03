//! Typed chart builders over `Vec<T>` with accessor closures, mirroring the
//! reference's method names (CHT-020): shared `name`, `interactive`,
//! `tooltip_title`, `tooltip_value`, `tooltip_value_color`,
//! `tooltip_content`, `tick_margin`, `grid`, `grid_dashed` and `x_axis`; for
//! lines and areas `x`, `y`, `stroke`, `natural`, `linear`, `step_after`,
//! `dot`, `y_domain`, `point_count`, `y_axis`, `y_axis_label_placement`,
//! `y_tick_count`, `y_tick_format`, `x_tick_count`, `grid_columns`,
//! `reference_line` and `y_padding`, areas also `fill` and `stacked`; for
//! scatters a numeric `x` (CHT-033) with the line's axis methods; for bars
//! `band`, `value`, `fill`, `fill_gradient`, `label`, `label_color`,
//! `label_axis`, `value_axis`, `value_tick_count`,
//! `value_axis_label_placement`, `value_tick_format`, `band_count`,
//! `band_tick_count`, `alignment`, `padding_inner`, `padding_outer`,
//! `max_band_width`, `min_length`, `corner_radius` and `stacked`; and for candlesticks `x`,
//! `open`, `high`, `low`, `close`, `body_width_ratio`, `max_band_width`,
//! `bullish` and `bearish`. The pie, donut and radar builders take the
//! reference's own names (CHT-029): `value`, `label`, `color`,
//! `inner_radius`, `outer_radius`, `pad_angle` and `label_gap`, for radar
//! `stroke`, `fill`, `dot`, `grid`, `grid_levels` and `max_value`, and for
//! Sankey `new(nodes, links)`, `value_scale`, `node_align`, `iterations`,
//! `node_width`, `node_padding`, `node_color`, `node_label`, `value_label`,
//! `labels`, `link_opacity`, `min_link_width` and `label_gap`. The closures
//! run once at `build()`; the resulting [`ChartProps`] hold plain data points
//! and stay comparable.

use super::{
    AxisLabelPlacement, BarGrowth, ChartProps, ChartType, ChartsBuilder, Curve, DataPoint,
    DataSeries, RadialOptions, SankeyAlign, SankeyLabel, SankeyLink, SankeyOptions,
    SankeyValueScale, SizeClass,
};
use crate::component::Element;

type Label<T> = Box<dyn Fn(&T) -> String>;
type Value<T> = Box<dyn Fn(&T) -> f64>;
/// A Sankey node's text from the node and its throughput, or a point's
/// tooltip text from the datum and its value.
type ValueText<T> = Box<dyn Fn(&T, f64) -> String>;
/// A Sankey node's label lines from the node and its throughput.
type LabelLines<T> = Box<dyn Fn(&T, f64) -> Vec<SankeyLabel>>;
/// A point's whole tooltip content as lines, from the datum.
type Lines<T> = Box<dyn Fn(&T) -> Vec<String>>;
/// A bar's gradient stops from the datum, the chart's value range and a
/// mapping from a chart value to a position along the bar (0 at its base,
/// 1 at its tip).
type Stops<T> = Box<dyn Fn(&T, (f64, f64), &dyn Fn(f64) -> f32) -> Vec<(f32, String)>>;

/// Options every typed builder shares. Grid and axis choices go straight
/// into the generic builder, so `build()` settles them by orientation with
/// `label_axis` and `value_axis` and nothing overwrites them afterwards
/// (CHT-020, CHT-035).
struct Common {
    base: ChartsBuilder,
    tick_margin: usize,
}

impl Common {
    fn new(kind: ChartType) -> Self {
        Self {
            base: ChartsBuilder::new().chart_type(kind),
            tick_margin: 0,
        }
    }

    fn finish(self, series: Vec<DataSeries>) -> ChartProps {
        self.base
            .with_series(series)
            .tick_margin(self.tick_margin)
            .build()
    }
}

/// Per-point tooltip text from the datum, run once at `build()` (CHT-018).
struct TooltipSpec<T> {
    title: Option<Label<T>>,
    value: Option<ValueText<T>>,
    color: Option<ValueText<T>>,
    content: Option<Lines<T>>,
}

impl<T> Default for TooltipSpec<T> {
    fn default() -> Self {
        Self {
            title: None,
            value: None,
            color: None,
            content: None,
        }
    }
}

impl<T> TooltipSpec<T> {
    fn apply(&self, point: &mut DataPoint, datum: &T) {
        let value = point.value;
        if let Some(title) = &self.title {
            point.tooltip.title = Some(title(datum));
        }
        if let Some(text) = &self.value {
            point.tooltip.value = Some(text(datum, value));
        }
        if let Some(color) = &self.color {
            point.tooltip.color = Some(color(datum, value));
        }
        if let Some(content) = &self.content {
            point.tooltip.lines = content(datum);
        }
    }
}

macro_rules! common_methods {
    () => {
        /// Chart title.
        pub fn title(mut self, title: impl Into<String>) -> Self {
            self.common.base = self.common.base.title(title);
            self
        }

        /// The name the screen reader is told, instead of the title
        /// (CHT-036).
        pub fn aria_label(mut self, label: impl Into<String>) -> Self {
            self.common.base = self.common.base.aria_label(label);
            self
        }

        /// Fixed size in cells; unset fills the allotted rectangle.
        pub fn size(mut self, width: u16, height: u16) -> Self {
            self.common.base = self.common.base.size(width, height);
            self
        }

        /// Force a size class.
        pub fn size_class(mut self, class: SizeClass) -> Self {
            self.common.base = self.common.base.size_class(class);
            self
        }

        /// Draw the drawing's horizontal axis line and its labels.
        pub fn x_axis(mut self, show: bool) -> Self {
            self.common.base = self.common.base.x_axis_labels(show);
            self
        }

        /// Hover, keyboard selection and the tooltip, on or off (on by
        /// default).
        pub fn interactive(mut self, interactive: bool) -> Self {
            self.common.base = self.common.base.interactive(interactive);
            self
        }

        /// Render with ASCII glyphs only.
        pub fn ascii(mut self, ascii: bool) -> Self {
            self.common.base = self.common.base.ascii(ascii);
            self
        }

        /// Add CSS classes such as `reduced-motion`.
        pub fn class(mut self, class: impl Into<String>) -> Self {
            self.common.base = self.common.base.class(class);
            self
        }

        /// Build and render as an element.
        pub fn render(self) -> Element {
            Element::typed::<super::Chart>(self.build())
        }
    };
}

/// The methods every cartesian builder has besides [`common_methods!`]:
/// label stride, grid, the tooltip closures and headroom.
macro_rules! cartesian_methods {
    () => {
        /// Show every n-th category label; 0 avoids overlap automatically.
        pub fn tick_margin(mut self, margin: usize) -> Self {
            self.common.tick_margin = margin;
            self
        }

        /// Draw grid lines.
        pub fn grid(mut self, grid: bool) -> Self {
            self.common.base = self.common.base.grid(grid);
            self
        }

        /// Draw the grid dashed (dotted cells, the default) or solid
        /// (CHT-034).
        pub fn grid_dashed(mut self, dashed: bool) -> Self {
            self.common.base = self.common.base.grid_dashed(dashed);
            self
        }

        /// Rows kept clear above the highest value and below the lowest
        /// (CHT-034).
        pub fn y_padding(mut self, top: u16, bottom: u16) -> Self {
            self.common.base = self.common.base.y_padding(top, bottom);
            self
        }

        /// The tooltip's title row from the datum, instead of its category
        /// (CHT-018).
        pub fn tooltip_title<S: Into<String>>(mut self, title: impl Fn(&T) -> S + 'static) -> Self {
            self.tooltip.title = Some(Box::new(move |d| title(d).into()));
            self
        }

        /// A point's value text from the datum and its value, instead of
        /// the formatted value (CHT-018).
        pub fn tooltip_value<S: Into<String>>(
            mut self,
            value: impl Fn(&T, f64) -> S + 'static,
        ) -> Self {
            self.tooltip.value = Some(Box::new(move |d, v| value(d, v).into()));
            self
        }

        /// The color token of a point's value text, from the datum and its
        /// value (CHT-018).
        pub fn tooltip_value_color<S: Into<String>>(
            mut self,
            color: impl Fn(&T, f64) -> S + 'static,
        ) -> Self {
            self.tooltip.color = Some(Box::new(move |d, v| color(d, v).into()));
            self
        }

        /// The tooltip's whole content as lines from the datum, instead of
        /// the series rows (CHT-018).
        pub fn tooltip_content<S: Into<String>>(
            mut self,
            content: impl Fn(&T) -> Vec<S> + 'static,
        ) -> Self {
            self.tooltip.content = Some(Box::new(move |d| {
                content(d).into_iter().map(Into::into).collect()
            }));
            self
        }
    };
}

/// The value-axis options of lines, areas and scatters (CHT-034).
macro_rules! value_axis_methods {
    () => {
        /// Pin the vertical axis to `min..=max` instead of fitting the data;
        /// a value outside stops at the plot's edge.
        pub fn y_domain(mut self, min: f64, max: f64) -> Self {
            self.common.base = self.common.base.y_domain(min, max);
            self
        }

        /// Lay the category axis out for `count` evenly spaced points, the
        /// data taking the leading ones.
        pub fn point_count(mut self, count: usize) -> Self {
            self.common.base = self.common.base.point_count(count);
            self
        }

        /// Show the vertical axis's tick labels, or hide them.
        pub fn y_axis(mut self, show: bool) -> Self {
            self.common.base = self.common.base.y_axis_labels(show);
            self
        }

        /// Where the vertical axis's tick labels sit: in a gutter left of
        /// the plot, or inside it beside their grid lines.
        pub fn y_axis_label_placement(mut self, placement: AxisLabelPlacement) -> Self {
            self.common.base = self.common.base.y_axis_label_placement(placement);
            self
        }

        /// How many ticks the vertical axis carries, at least two; they
        /// place the grid rows and the tick labels.
        pub fn y_tick_count(mut self, count: usize) -> Self {
            self.common.base = self.common.base.y_tick_count(count);
            self
        }

        /// The text of each vertical-axis tick label from its value, run
        /// once at `build()`.
        pub fn y_tick_format<S: Into<String>>(
            mut self,
            format: impl Fn(f64) -> S + Send + Sync + 'static,
        ) -> Self {
            self.common.base = self.common.base.y_tick_format(move |v| format(v).into());
            self
        }

        /// Label `count` of the category values, spread from the first to
        /// the last, instead of every `tick_margin`-th.
        pub fn x_tick_count(mut self, count: usize) -> Self {
            self.common.base = self.common.base.x_tick_count(count);
            self
        }

        /// Divide the plot into `count` columns with vertical grid lines.
        pub fn grid_columns(mut self, count: usize) -> Self {
            self.common.base = self.common.base.grid_columns(count);
            self
        }

        /// Draw a dashed line across the plot at `value`; call again for
        /// more.
        pub fn reference_line(mut self, value: f64) -> Self {
            self.common.base = self.common.base.reference_line(value);
            self
        }
    };
}

/// One series described by accessors.
struct SeriesSpec<T> {
    name: Option<String>,
    value: Value<T>,
    color: Option<String>,
    /// An area's fill color token; `None` fills with the stroke (CHT-012).
    fill: Option<String>,
    /// The series' own curve (CHT-012).
    curve: Option<Curve>,
    /// Dots at every point of this series (CHT-012).
    dots: Option<bool>,
}

impl<T> SeriesSpec<T> {
    fn new(value: Value<T>) -> Self {
        Self {
            name: None,
            value,
            color: None,
            fill: None,
            curve: None,
            dots: None,
        }
    }
}

fn points<T>(data: &[T], label: Option<&Label<T>>, value: &Value<T>) -> Vec<DataPoint> {
    data.iter()
        .map(|d| {
            let mut point = DataPoint::new(value(d));
            if let Some(label) = label {
                point.label = Some(label(d));
            }
            point
        })
        .collect()
}

fn series<T>(data: &[T], label: Option<&Label<T>>, specs: &[SeriesSpec<T>]) -> Vec<DataSeries> {
    cartesian_series(data, label, None, specs, &TooltipSpec::default())
}

/// The series of a cartesian builder: labelled by `label` or placed at the
/// numeric `x`, with each spec's colors, curve and dots, and the tooltip
/// closures run into every point.
fn cartesian_series<T>(
    data: &[T],
    label: Option<&Label<T>>,
    x: Option<&Value<T>>,
    specs: &[SeriesSpec<T>],
    tooltip: &TooltipSpec<T>,
) -> Vec<DataSeries> {
    specs
        .iter()
        .enumerate()
        .map(|(i, spec)| {
            let mut points = points(data, label, &spec.value);
            for (point, datum) in points.iter_mut().zip(data) {
                if let Some(x) = x {
                    point.x = Some(x(datum));
                }
                tooltip.apply(point, datum);
            }
            let mut series = DataSeries::new(
                spec.name
                    .clone()
                    .unwrap_or_else(|| format!("series {}", i + 1)),
                points,
            );
            series.color = spec.color.clone();
            series.fill = spec.fill.clone();
            series.curve = spec.curve;
            series.dots = spec.dots;
            series
        })
        .collect()
}

/// A line chart over `Vec<T>`.
pub struct LineChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    x: Option<Label<T>>,
    series: Vec<SeriesSpec<T>>,
    tooltip: TooltipSpec<T>,
}

/// The per-series methods of lines and areas: name, stroke, curve and dots
/// apply to the series added last, or to the whole chart before any series.
macro_rules! stroke_methods {
    () => {
        /// Name of the series added last.
        pub fn name(mut self, name: impl Into<String>) -> Self {
            if let Some(last) = self.series.last_mut() {
                last.name = Some(name.into());
            }
            self
        }

        /// Stroke color token (palette name, theme variable or hex) of the
        /// series added last.
        pub fn stroke(mut self, token: impl Into<String>) -> Self {
            if let Some(last) = self.series.last_mut() {
                last.color = Some(token.into());
            }
            self
        }

        /// Smooth spline strokes for the series added last, or for the
        /// chart before any series.
        pub fn natural(self) -> Self {
            self.curved(Curve::Natural)
        }

        /// Straight strokes for the series added last, or for the chart
        /// before any series.
        pub fn linear(self) -> Self {
            self.curved(Curve::Linear)
        }

        /// Step strokes holding each value until the next point, for the
        /// series added last or for the chart before any series.
        pub fn step_after(self) -> Self {
            self.curved(Curve::StepAfter)
        }

        fn curved(mut self, curve: Curve) -> Self {
            match self.series.last_mut() {
                Some(last) => last.curve = Some(curve),
                None => self.common.base = self.common.base.curve(curve),
            }
            self
        }

        /// Draw a dot at every point of the series added last, or of every
        /// series before any is added; dots are off until asked for
        /// (CHT-012).
        pub fn dot(mut self) -> Self {
            match self.series.last_mut() {
                Some(last) => last.dots = Some(true),
                None => self.common.base = self.common.base.dots(true),
            }
            self
        }
    };
}

impl<T> LineChartBuilder<T> {
    /// A line chart over `data`.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::Line),
            data: data.into_iter().collect(),
            x: None,
            series: Vec::new(),
            tooltip: TooltipSpec::default(),
        }
    }

    common_methods!();
    cartesian_methods!();
    value_axis_methods!();
    stroke_methods!();

    /// Category label accessor.
    pub fn x<S: Into<String>>(mut self, x: impl Fn(&T) -> S + 'static) -> Self {
        self.x = Some(Box::new(move |d| x(d).into()));
        self
    }

    /// Value accessor; each call adds a series.
    pub fn y(mut self, y: impl Fn(&T) -> f64 + 'static) -> Self {
        self.series.push(SeriesSpec::new(Box::new(y)));
        self
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        let series = cartesian_series(
            &self.data,
            self.x.as_ref(),
            None,
            &self.series,
            &self.tooltip,
        );
        self.common.finish(series)
    }
}

/// An area chart over `Vec<T>`.
pub struct AreaChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    x: Option<Label<T>>,
    series: Vec<SeriesSpec<T>>,
    tooltip: TooltipSpec<T>,
}

impl<T> AreaChartBuilder<T> {
    /// An area chart over `data`.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::Area),
            data: data.into_iter().collect(),
            x: None,
            series: Vec::new(),
            tooltip: TooltipSpec::default(),
        }
    }

    common_methods!();
    cartesian_methods!();
    value_axis_methods!();
    stroke_methods!();

    /// Category label accessor.
    pub fn x<S: Into<String>>(mut self, x: impl Fn(&T) -> S + 'static) -> Self {
        self.x = Some(Box::new(move |d| x(d).into()));
        self
    }

    /// Value accessor; each call adds a series.
    pub fn y(mut self, y: impl Fn(&T) -> f64 + 'static) -> Self {
        self.series.push(SeriesSpec::new(Box::new(y)));
        self
    }

    /// Fill color token of the series added last; unset, the fill takes
    /// the stroke color at 0.4 opacity (CHT-012).
    pub fn fill(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.fill = Some(token.into());
        }
        self
    }

    /// Stack the series on top of each other.
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.common.base = self.common.base.stacked(stacked);
        self
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        let series = cartesian_series(
            &self.data,
            self.x.as_ref(),
            None,
            &self.series,
            &self.tooltip,
        );
        self.common.finish(series)
    }
}

/// A scatter plot over `Vec<T>`: points placed by a numeric x (CHT-033).
pub struct ScatterChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    x: Option<Value<T>>,
    label: Option<Label<T>>,
    series: Vec<SeriesSpec<T>>,
    tooltip: TooltipSpec<T>,
}

impl<T> ScatterChartBuilder<T> {
    /// A scatter plot over `data`.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::Scatter),
            data: data.into_iter().collect(),
            x: None,
            label: None,
            series: Vec::new(),
            tooltip: TooltipSpec::default(),
        }
    }

    common_methods!();
    cartesian_methods!();
    value_axis_methods!();

    /// Numeric x accessor: points sit where their x values fall on a
    /// linear axis (CHT-033).
    pub fn x(mut self, x: impl Fn(&T) -> f64 + 'static) -> Self {
        self.x = Some(Box::new(x));
        self
    }

    /// A point's label, for its tooltip; unset, the tooltip shows its x.
    pub fn label<S: Into<String>>(mut self, label: impl Fn(&T) -> S + 'static) -> Self {
        self.label = Some(Box::new(move |d| label(d).into()));
        self
    }

    /// Value accessor; each call adds a series.
    pub fn y(mut self, y: impl Fn(&T) -> f64 + 'static) -> Self {
        self.series.push(SeriesSpec::new(Box::new(y)));
        self
    }

    /// Name of the series added last.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.name = Some(name.into());
        }
        self
    }

    /// Marker color token of the series added last.
    pub fn stroke(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.color = Some(token.into());
        }
        self
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        let series = cartesian_series(
            &self.data,
            self.label.as_ref(),
            self.x.as_ref(),
            &self.series,
            &self.tooltip,
        );
        self.common.finish(series)
    }
}

/// A bar chart over `Vec<T>`.
pub struct BarChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    band: Option<Label<T>>,
    label: Option<Label<T>>,
    label_color: Option<Label<T>>,
    fill_with: Option<Label<T>>,
    gradient: Option<Stops<T>>,
    series: Vec<SeriesSpec<T>>,
    tooltip: TooltipSpec<T>,
}

impl<T> BarChartBuilder<T> {
    /// A bar chart over `data`.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::BarVertical),
            data: data.into_iter().collect(),
            band: None,
            label: None,
            label_color: None,
            fill_with: None,
            gradient: None,
            series: Vec::new(),
            tooltip: TooltipSpec::default(),
        }
    }

    common_methods!();
    cartesian_methods!();

    /// Category (band) label accessor.
    pub fn band<S: Into<String>>(mut self, band: impl Fn(&T) -> S + 'static) -> Self {
        self.band = Some(Box::new(move |d| band(d).into()));
        self
    }

    /// Value accessor; each call adds a series.
    pub fn value(mut self, value: impl Fn(&T) -> f64 + 'static) -> Self {
        self.series.push(SeriesSpec::new(Box::new(value)));
        self
    }

    /// Name of the series added last.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.name = Some(name.into());
        }
        self
    }

    /// Fill color token of the series added last: the color of its bars,
    /// stored as the series fill like the generic route's `.fill()` so both
    /// routes build the same props (CHT-013, CHT-035).
    pub fn fill(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.fill = Some(token.into());
        }
        self
    }

    /// Each bar's fill color token from its datum, over the series fill
    /// (CHT-013).
    pub fn fill_with<S: Into<String>>(mut self, fill: impl Fn(&T) -> S + 'static) -> Self {
        self.fill_with = Some(Box::new(move |d| fill(d).into()));
        self
    }

    /// Each bar's fill gradient from base to tip, as (offset, color token)
    /// stops from the datum, the chart's value range and a mapping from a
    /// chart value to a position along the bar (CHT-013).
    pub fn fill_gradient<S: Into<String>>(
        mut self,
        stops: impl Fn(&T, (f64, f64), &dyn Fn(f64) -> f32) -> Vec<(f32, S)> + 'static,
    ) -> Self {
        self.gradient = Some(Box::new(move |d, range, to_bar| {
            stops(d, range, to_bar)
                .into_iter()
                .map(|(offset, token)| (offset, token.into()))
                .collect()
        }));
        self
    }

    /// The edge bars grow from.
    pub fn alignment(mut self, alignment: BarGrowth) -> Self {
        self.common.base = self.common.base.growth(alignment);
        self
    }

    /// Value label accessor, shown beside each bar at the large size class.
    pub fn label<S: Into<String>>(mut self, label: impl Fn(&T) -> S + 'static) -> Self {
        self.label = Some(Box::new(move |d| label(d).into()));
        self.common.base = self.common.base.value_labels(true);
        self
    }

    /// Each bar's value-label color token from its datum (CHT-013).
    pub fn label_color<S: Into<String>>(mut self, color: impl Fn(&T) -> S + 'static) -> Self {
        self.label_color = Some(Box::new(move |d| color(d).into()));
        self
    }

    /// Show or hide the band axis line and labels.
    pub fn label_axis(mut self, show: bool) -> Self {
        self.common.base = self.common.base.label_axis(show);
        self
    }

    /// Show or hide the value axis's tick labels.
    pub fn value_axis(mut self, show: bool) -> Self {
        self.common.base = self.common.base.value_axis(show);
        self
    }

    /// How many ticks the value axis carries, at least two; they place the
    /// grid lines and the tick labels.
    pub fn value_tick_count(mut self, count: usize) -> Self {
        self.common.base = self.common.base.value_tick_count(count);
        self
    }

    /// Where the value tick labels sit: in a gutter beside the bars, or
    /// inside the plot beside their grid lines.
    pub fn value_axis_label_placement(mut self, placement: AxisLabelPlacement) -> Self {
        self.common.base = self.common.base.value_axis_label_placement(placement);
        self
    }

    /// The text of each value tick label from its value, run once at
    /// `build()`.
    pub fn value_tick_format<S: Into<String>>(
        mut self,
        format: impl Fn(f64) -> S + Send + Sync + 'static,
    ) -> Self {
        self.common.base = self
            .common
            .base
            .value_tick_format(move |v| format(v).into());
        self
    }

    /// Lay the band axis out for `count` bands, the data taking the
    /// leading ones.
    pub fn band_count(mut self, count: usize) -> Self {
        self.common.base = self.common.base.band_count(count);
        self
    }

    /// Label `count` of the bands, spread from the first to the last.
    pub fn band_tick_count(mut self, count: usize) -> Self {
        self.common.base = self.common.base.band_tick_count(count);
        self
    }

    /// Space between bands as a fraction of a band (default 0.4).
    pub fn padding_inner(mut self, padding: f64) -> Self {
        self.common.base = self.common.base.padding_inner(padding);
        self
    }

    /// Space outside the first and last band as a fraction of a band
    /// (default 0.2).
    pub fn padding_outer(mut self, padding: f64) -> Self {
        self.common.base = self.common.base.padding_outer(padding);
        self
    }

    /// Keep every band at most `width` cells wide.
    pub fn max_band_width(mut self, width: u16) -> Self {
        self.common.base = self.common.base.max_band_width(width);
        self
    }

    /// The shortest a bar is drawn, in cells, so a tiny value still shows.
    pub fn min_length(mut self, length: f64) -> Self {
        self.common.base = self.common.base.min_length(length);
        self
    }

    /// Round each bar's corners by `cells` where the plot is a picture
    /// (CHT-013); the cell fallback cannot show it. Default none.
    pub fn corner_radius(mut self, cells: f32) -> Self {
        self.common.base = self.common.base.corner_radius(cells);
        self
    }

    /// Stack the series end to end.
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.common.base = self.common.base.stacked(stacked);
        self
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        let mut series = cartesian_series(
            &self.data,
            self.band.as_ref(),
            None,
            &self.series,
            &self.tooltip,
        );
        // The chart's value range for the gradient closure: every bar's
        // value and zero.
        let range = series
            .iter()
            .flat_map(|s| s.data.iter().map(|p| p.value))
            .filter(|v| v.is_finite())
            .fold((0.0f64, 0.0f64), |(low, high), v| (low.min(v), high.max(v)));
        for s in &mut series {
            for (point, datum) in s.data.iter_mut().zip(&self.data) {
                if let Some(label) = &self.label {
                    point.value_label = Some(label(datum));
                }
                if let Some(color) = &self.label_color {
                    point.label_color = Some(color(datum));
                }
                if let Some(fill) = &self.fill_with {
                    point.color = Some(fill(datum));
                }
                if let Some(gradient) = &self.gradient {
                    let value = point.value;
                    let to_bar = move |v: f64| -> f32 {
                        if value == 0.0 {
                            0.0
                        } else {
                            (v / value).clamp(0.0, 1.0) as f32
                        }
                    };
                    point.gradient = gradient(datum, range, &to_bar);
                }
            }
        }
        self.common.finish(series)
    }
}

/// A candlestick chart over `Vec<T>`.
pub struct CandlestickChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    x: Option<Label<T>>,
    open: Option<Value<T>>,
    high: Option<Value<T>>,
    low: Option<Value<T>>,
    close: Option<Value<T>>,
    bullish: Option<String>,
    bearish: Option<String>,
    name: Option<String>,
    tooltip: TooltipSpec<T>,
}

impl<T> CandlestickChartBuilder<T> {
    /// A candlestick chart over `data`.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::Candlestick),
            data: data.into_iter().collect(),
            x: None,
            open: None,
            high: None,
            low: None,
            close: None,
            bullish: None,
            bearish: None,
            name: None,
            tooltip: TooltipSpec::default(),
        }
    }

    common_methods!();
    cartesian_methods!();

    /// Name of the candle series, for the legend and the tooltip.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Category label accessor.
    pub fn x<S: Into<String>>(mut self, x: impl Fn(&T) -> S + 'static) -> Self {
        self.x = Some(Box::new(move |d| x(d).into()));
        self
    }

    /// Opening value accessor.
    pub fn open(mut self, open: impl Fn(&T) -> f64 + 'static) -> Self {
        self.open = Some(Box::new(open));
        self
    }

    /// High value accessor.
    pub fn high(mut self, high: impl Fn(&T) -> f64 + 'static) -> Self {
        self.high = Some(Box::new(high));
        self
    }

    /// Low value accessor.
    pub fn low(mut self, low: impl Fn(&T) -> f64 + 'static) -> Self {
        self.low = Some(Box::new(low));
        self
    }

    /// Closing value accessor.
    pub fn close(mut self, close: impl Fn(&T) -> f64 + 'static) -> Self {
        self.close = Some(Box::new(close));
        self
    }

    /// Color token for candles that close above their open.
    pub fn bullish(mut self, token: impl Into<String>) -> Self {
        self.bullish = Some(token.into());
        self
    }

    /// Color token for candles that close below their open.
    pub fn bearish(mut self, token: impl Into<String>) -> Self {
        self.bearish = Some(token.into());
        self
    }

    /// A candle body's width as a fraction of its band (default 0.8).
    pub fn body_width_ratio(mut self, ratio: f32) -> Self {
        self.common.base = self.common.base.body_width_ratio(ratio);
        self
    }

    /// Keep every candle's band at most `width` cells wide.
    pub fn max_band_width(mut self, width: u16) -> Self {
        self.common.base = self.common.base.max_band_width(width);
        self
    }

    /// Evaluate the accessors into props. A missing accessor falls back to
    /// the close, so a partial description still draws.
    pub fn build(self) -> ChartProps {
        let close = self.close.as_ref();
        let get = |f: &Option<Value<T>>, d: &T| f.as_ref().or(close).map_or(0.0, |f| f(d));
        let mut points = Vec::with_capacity(self.data.len());
        for d in &self.data {
            let (o, h, l, c) = (
                get(&self.open, d),
                get(&self.high, d),
                get(&self.low, d),
                get(&self.close, d),
            );
            let mut point = DataPoint::candle(o, h.max(o).max(c), l.min(o).min(c), c);
            if let Some(x) = &self.x {
                point.label = Some(x(d));
            }
            if let Some(token) = if c > o { &self.bullish } else { &self.bearish } {
                point.color = Some(token.clone());
            }
            self.tooltip.apply(&mut point, d);
            points.push(point);
        }
        let name = self.name.clone().unwrap_or_else(|| "candles".into());
        let series = vec![DataSeries::new(name, points)];
        self.common.finish(series)
    }
}

/// A pie chart over `Vec<T>`: one slice per item.
pub struct PieChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    name: Option<String>,
    value: Option<Value<T>>,
    label: Option<Label<T>>,
    color: Option<Label<T>>,
    radial: RadialOptions,
}

impl<T> PieChartBuilder<T> {
    /// A pie chart over `data`, one slice per item.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::Pie),
            data: data.into_iter().collect(),
            name: None,
            value: None,
            label: None,
            color: None,
            radial: RadialOptions::default(),
        }
    }

    common_methods!();

    /// Slice value accessor.
    pub fn value(mut self, value: impl Fn(&T) -> f64 + 'static) -> Self {
        self.value = Some(Box::new(value));
        self
    }

    /// Slice label accessor, shown beside the chart and in the legend.
    pub fn label<S: Into<String>>(mut self, label: impl Fn(&T) -> S + 'static) -> Self {
        self.label = Some(Box::new(move |d| label(d).into()));
        self
    }

    /// Slice color token accessor.
    pub fn color<S: Into<String>>(mut self, color: impl Fn(&T) -> S + 'static) -> Self {
        self.color = Some(Box::new(move |d| color(d).into()));
        self
    }

    /// Name of the series, shown in the tooltip.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Inner radius as a fraction of the outer one: 0 draws a pie.
    pub fn inner_radius(mut self, fraction: f64) -> Self {
        self.radial.inner_radius = Some(fraction);
        self
    }

    /// Outer radius as a fraction of the largest circle that fits.
    pub fn outer_radius(mut self, fraction: f64) -> Self {
        self.radial.outer_radius = fraction;
        self
    }

    /// Gap between adjacent slices, in radians.
    pub fn pad_angle(mut self, radians: f64) -> Self {
        self.radial.pad_angle = radians;
        self
    }

    /// Columns between the circle and its side labels. At 0 a label touches
    /// the circle, and one whose leader finds no room there sits one column
    /// further out.
    pub fn label_gap(mut self, columns: u16) -> Self {
        self.radial.label_gap = columns;
        self
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        let value = self.value.as_ref();
        let data = self
            .data
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let mut point = DataPoint::new(value.map_or(0.0, |f| f(d)));
                point.label = Some(self.label.as_ref().map_or_else(|| i.to_string(), |f| f(d)));
                point.color = self.color.as_ref().map(|f| f(d));
                point
            })
            .collect();
        let series = DataSeries::new(self.name.unwrap_or_else(|| "series 1".into()), data);
        let mut common = self.common;
        common.base = common.base.x_axis_labels(false);
        let mut props = common.finish(vec![series]);
        props.radial = self.radial;
        props
    }
}

/// A donut chart over `Vec<T>`: a pie with a hole, half the radius unless
/// `inner_radius` says otherwise.
pub struct DonutChartBuilder<T> {
    pie: PieChartBuilder<T>,
}

impl<T> DonutChartBuilder<T> {
    /// A donut chart over `data`, one slice per item.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        let mut pie = PieChartBuilder::new(data);
        pie.common = Common::new(ChartType::Donut);
        Self { pie }
    }

    /// Chart title.
    pub fn title(self, title: impl Into<String>) -> Self {
        Self {
            pie: self.pie.title(title),
        }
    }

    /// Fixed size in cells; unset fills the allotted rectangle.
    pub fn size(self, width: u16, height: u16) -> Self {
        Self {
            pie: self.pie.size(width, height),
        }
    }

    /// Force a size class.
    pub fn size_class(self, class: SizeClass) -> Self {
        Self {
            pie: self.pie.size_class(class),
        }
    }

    /// Render with ASCII glyphs only.
    pub fn ascii(self, ascii: bool) -> Self {
        Self {
            pie: self.pie.ascii(ascii),
        }
    }

    /// Add CSS classes such as `reduced-motion`.
    pub fn class(self, class: impl Into<String>) -> Self {
        Self {
            pie: self.pie.class(class),
        }
    }

    /// Slice value accessor.
    pub fn value(self, value: impl Fn(&T) -> f64 + 'static) -> Self {
        Self {
            pie: self.pie.value(value),
        }
    }

    /// Slice label accessor, shown beside the chart and in the legend.
    pub fn label<S: Into<String>>(self, label: impl Fn(&T) -> S + 'static) -> Self {
        Self {
            pie: self.pie.label(label),
        }
    }

    /// Slice color token accessor.
    pub fn color<S: Into<String>>(self, color: impl Fn(&T) -> S + 'static) -> Self {
        Self {
            pie: self.pie.color(color),
        }
    }

    /// Name of the series, shown in the tooltip.
    pub fn name(self, name: impl Into<String>) -> Self {
        Self {
            pie: self.pie.name(name),
        }
    }

    /// Inner radius as a fraction of the outer one.
    pub fn inner_radius(self, fraction: f64) -> Self {
        Self {
            pie: self.pie.inner_radius(fraction),
        }
    }

    /// Outer radius as a fraction of the largest circle that fits.
    pub fn outer_radius(self, fraction: f64) -> Self {
        Self {
            pie: self.pie.outer_radius(fraction),
        }
    }

    /// Gap between adjacent slices, in radians.
    pub fn pad_angle(self, radians: f64) -> Self {
        Self {
            pie: self.pie.pad_angle(radians),
        }
    }

    /// Columns between the circle and its side labels. At 0 a label touches
    /// the circle, and one whose leader finds no room there sits one column
    /// further out.
    pub fn label_gap(self, columns: u16) -> Self {
        Self {
            pie: self.pie.label_gap(columns),
        }
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        self.pie.build()
    }

    /// Build and render as an element.
    pub fn render(self) -> Element {
        self.pie.render()
    }
}

/// A Sankey chart over `Vec<T>` nodes and the links between them, by node
/// index (CHT-029, CHT-030).
pub struct SankeyChartBuilder<T> {
    common: Common,
    nodes: Vec<T>,
    name: Option<String>,
    node_color: Option<Label<T>>,
    node_label: Option<Label<T>>,
    value_label: Option<ValueText<T>>,
    labels: Option<LabelLines<T>>,
    sankey: SankeyOptions,
}

impl<T> SankeyChartBuilder<T> {
    /// A Sankey chart of `nodes` and `links`; a link names its source and
    /// target by their index in `nodes`.
    pub fn new(
        nodes: impl IntoIterator<Item = T>,
        links: impl IntoIterator<Item = SankeyLink>,
    ) -> Self {
        Self {
            common: Common::new(ChartType::Sankey),
            nodes: nodes.into_iter().collect(),
            name: None,
            node_color: None,
            node_label: None,
            value_label: None,
            labels: None,
            sankey: SankeyOptions {
                links: links.into_iter().collect(),
                ..SankeyOptions::default()
            },
        }
    }

    common_methods!();

    /// How values map to node heights and link widths.
    pub fn value_scale(mut self, scale: SankeyValueScale) -> Self {
        self.sankey.value_scale = scale;
        self
    }

    /// Which column each node takes.
    pub fn node_align(mut self, align: SankeyAlign) -> Self {
        self.sankey.node_align = align;
        self
    }

    /// Relaxation passes that move nodes toward their flows.
    pub fn iterations(mut self, iterations: usize) -> Self {
        self.sankey.iterations = iterations;
        self
    }

    /// Node width in columns.
    pub fn node_width(mut self, columns: u16) -> Self {
        self.sankey.node_width = columns;
        self
    }

    /// Rows between the nodes of a column.
    pub fn node_padding(mut self, rows: u16) -> Self {
        self.sankey.node_padding = rows;
        self
    }

    /// Node color token accessor; unset nodes take the palette in order.
    pub fn node_color<S: Into<String>>(mut self, color: impl Fn(&T) -> S + 'static) -> Self {
        self.node_color = Some(Box::new(move |d| color(d).into()));
        self
    }

    /// Node name accessor, shown beside the node and in the tooltip.
    pub fn node_label<S: Into<String>>(mut self, label: impl Fn(&T) -> S + 'static) -> Self {
        self.node_label = Some(Box::new(move |d| label(d).into()));
        self
    }

    /// Throughput text accessor, given the node and its throughput (the
    /// larger of its incoming and outgoing totals).
    pub fn value_label<S: Into<String>>(mut self, label: impl Fn(&T, f64) -> S + 'static) -> Self {
        self.value_label = Some(Box::new(move |d, v| label(d, v).into()));
        self
    }

    /// The whole label beside a node, as styled lines, given the node and
    /// its throughput; it replaces the name and throughput lines.
    pub fn labels(mut self, labels: impl Fn(&T, f64) -> Vec<SankeyLabel> + 'static) -> Self {
        self.labels = Some(Box::new(labels));
        self
    }

    /// How much of its color a ribbon keeps over the background, 0 to 1.
    pub fn link_opacity(mut self, opacity: f32) -> Self {
        self.sankey.link_opacity = opacity;
        self
    }

    /// The narrowest a ribbon is drawn, in rows.
    pub fn min_link_width(mut self, rows: f32) -> Self {
        self.sankey.min_link_width = rows;
        self
    }

    /// Columns between a first- or last-layer node and its label.
    pub fn label_gap(mut self, columns: u16) -> Self {
        self.sankey.label_gap = columns;
        self
    }

    /// Name of the node series, in the accessibility description.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Evaluate the accessors into props. Each node's throughput is the
    /// larger of its raw incoming and outgoing totals.
    pub fn build(self) -> ChartProps {
        let (mut incoming, mut outgoing) =
            (vec![0.0; self.nodes.len()], vec![0.0; self.nodes.len()]);
        for link in &self.sankey.links {
            if let Some(total) = outgoing.get_mut(link.source) {
                *total += link.value;
            }
            if let Some(total) = incoming.get_mut(link.target) {
                *total += link.value;
            }
        }
        let throughput: Vec<f64> = incoming
            .iter()
            .zip(&outgoing)
            .map(|(i, o): (&f64, &f64)| i.max(*o))
            .collect();
        let data = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let mut point = DataPoint::new(throughput[i]);
                point.label = Some(
                    self.node_label
                        .as_ref()
                        .map_or_else(|| i.to_string(), |f| f(d)),
                );
                point.color = self.node_color.as_ref().map(|f| f(d));
                point
            })
            .collect();
        let mut sankey = self.sankey;
        if let Some(label) = &self.value_label {
            sankey.value_labels = self
                .nodes
                .iter()
                .zip(&throughput)
                .map(|(d, v)| label(d, *v))
                .collect();
        }
        if let Some(labels) = &self.labels {
            sankey.labels = self
                .nodes
                .iter()
                .zip(&throughput)
                .map(|(d, v)| labels(d, *v))
                .collect();
        }
        let series = DataSeries::new(self.name.unwrap_or_else(|| "nodes".into()), data);
        let mut common = self.common;
        common.base = common.base.x_axis_labels(false);
        let mut props = common.finish(vec![series]);
        props.legend.visible = false;
        props.sankey = sankey;
        props
    }
}

/// A radar chart over `Vec<T>`: one spoke per item, one polygon per
/// `value` accessor.
pub struct RadarChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    label: Option<Label<T>>,
    series: Vec<SeriesSpec<T>>,
    fills: Vec<Option<String>>,
    /// Vertex dots: unset leaves a radar's default, which is dots.
    dots: Option<bool>,
    radial: RadialOptions,
}

impl<T> RadarChartBuilder<T> {
    /// A radar chart over `data`, one category per item.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::Radar),
            data: data.into_iter().collect(),
            label: None,
            series: Vec::new(),
            fills: Vec::new(),
            dots: None,
            radial: RadialOptions::default(),
        }
    }

    common_methods!();

    /// Category label accessor, shown at the end of each spoke.
    pub fn label<S: Into<String>>(mut self, label: impl Fn(&T) -> S + 'static) -> Self {
        self.label = Some(Box::new(move |d| label(d).into()));
        self
    }

    /// Value accessor; each call adds a series.
    pub fn value(mut self, value: impl Fn(&T) -> f64 + 'static) -> Self {
        self.series.push(SeriesSpec::new(Box::new(value)));
        self.fills.push(None);
        self
    }

    /// Name of the series added last.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.name = Some(name.into());
        }
        self
    }

    /// Outline color token of the series added last.
    pub fn stroke(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.color = Some(token.into());
        }
        self
    }

    /// Fill color token of the series added last; `"none"` leaves it
    /// unfilled. Unset fills with the outline color.
    pub fn fill(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.fills.last_mut() {
            *last = Some(token.into());
        }
        self
    }

    /// Draw a dot at each vertex (a radar's default); `dots(false)` turns
    /// them off.
    pub fn dot(mut self) -> Self {
        self.dots = Some(true);
        self
    }

    /// Dots at the vertices, on or off.
    pub fn dots(mut self, dots: bool) -> Self {
        self.dots = Some(dots);
        self
    }

    /// Draw the grid levels and spokes.
    pub fn grid(mut self, grid: bool) -> Self {
        self.radial.grid = grid;
        self
    }

    /// Number of grid levels.
    pub fn grid_levels(mut self, levels: usize) -> Self {
        self.radial.grid_levels = levels;
        self
    }

    /// The scale's maximum; unset uses the largest value.
    pub fn max_value(mut self, max: f64) -> Self {
        self.radial.max_value = Some(max);
        self
    }

    /// Outer radius as a fraction of the largest circle that fits.
    pub fn outer_radius(mut self, fraction: f64) -> Self {
        self.radial.outer_radius = fraction;
        self
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        let series = series(&self.data, self.label.as_ref(), &self.series);
        let mut common = self.common;
        common.base = common.base.x_axis_labels(false);
        let mut props = common.finish(series);
        props.dots = self.dots;
        props.radial = RadialOptions {
            fills: self.fills,
            ..self.radial
        };
        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Row {
        day: &'static str,
        open: f64,
        close: f64,
    }

    fn rows() -> Vec<Row> {
        vec![
            Row {
                day: "mon",
                open: 1.0,
                close: 2.0,
            },
            Row {
                day: "tue",
                open: 2.0,
                close: 1.5,
            },
        ]
    }

    #[test]
    fn accessors_are_evaluated_once_into_plain_points() {
        let props = LineChartBuilder::new(rows())
            .x(|r| r.day)
            .y(|r| r.close)
            .name("close")
            .stroke("chart-2")
            .linear()
            .dot()
            .tick_margin(2)
            .grid(false)
            .build();
        assert_eq!(props.chart_type, ChartType::Line);
        assert_eq!(props.series[0].name, "close");
        assert_eq!(props.series[0].color.as_deref(), Some("chart-2"));
        assert_eq!(props.series[0].data[1].label.as_deref(), Some("tue"));
        assert_eq!(props.series[0].data[1].value, 1.5);
        // Curve and dots after `.y()` belong to that series (CHT-012).
        assert_eq!(props.series[0].curve, Some(Curve::Linear));
        assert_eq!(props.series[0].dots, Some(true));
        assert_eq!(
            props.dots, None,
            "dots are off until a series turns them on"
        );
        assert_eq!(props.tick_margin, 2);
        assert!(!props.x_axis.show_grid);
        assert_eq!(props.clone(), props, "props stay comparable");
    }

    /// `label_axis(false)` and `value_axis(false)` settle by orientation at
    /// `build()` and nothing the typed builder does afterwards puts the
    /// labels back (CHT-020); the generic route builds the same axes
    /// (CHT-035).
    #[test]
    fn label_axis_and_value_axis_hold_through_build_on_both_orientations() {
        let vertical = BarChartBuilder::new(rows())
            .band(|r| r.day)
            .value(|r| r.open)
            .label_axis(false)
            .build();
        assert!(
            !vertical.x_axis.show_labels,
            "a vertical bar chart's band axis stays hidden"
        );
        assert!(vertical.y_axis.show_labels);
        let horizontal = BarChartBuilder::new(rows())
            .band(|r| r.day)
            .value(|r| r.open)
            .alignment(BarGrowth::Left)
            .value_axis(false)
            .build();
        assert!(
            !horizontal.x_axis.show_labels,
            "a horizontal bar chart's value axis stays hidden"
        );
        assert!(horizontal.y_axis.show_labels);
        let generic = ChartsBuilder::new()
            .chart_type(ChartType::BarVertical)
            .series(DataSeries::new(
                "s",
                vec![
                    DataPoint::with_label(1.0, "mon"),
                    DataPoint::with_label(2.0, "tue"),
                ],
            ))
            .label_axis(false)
            .build();
        assert_eq!(generic.x_axis, vertical.x_axis);
        assert_eq!(generic.y_axis, vertical.y_axis);
    }

    /// Every per-point option of the typed bar builder has a generic form on
    /// `ChartsBuilder` over the point, and the two routes build the same
    /// series (CHT-035).
    #[test]
    fn per_point_options_build_the_same_series_on_the_generic_route() {
        let typed = BarChartBuilder::new(rows())
            .band(|r| r.day)
            .value(|r| r.open)
            .fill("chart-3")
            .fill_with(|r| if r.open > 1.5 { "chart-4" } else { "chart-5" })
            .fill_gradient(|r, range, to_bar| {
                vec![
                    (0.0, "chart-1".to_string()),
                    (
                        to_bar(range.1 / 2.0),
                        format!("chart-{}", r.open as usize + 1),
                    ),
                ]
            })
            .label(|r| format!("{:.1}", r.open))
            .label_color(|r| if r.open > 1.5 { "success" } else { "error" })
            .tooltip_title(|r| r.day.to_uppercase())
            .tooltip_value(|r, v| format!("{v} on {}", r.day))
            .tooltip_value_color(|_, v| if v > 1.5 { "success" } else { "error" })
            .tooltip_content(|r| vec![r.day.to_string(), "open".to_string()])
            .build();
        let generic = ChartsBuilder::new()
            .chart_type(ChartType::BarVertical)
            .series(DataSeries::new(
                "series 1",
                rows()
                    .iter()
                    .map(|r| {
                        let mut point = DataPoint::with_label(r.open, r.day);
                        point.value_label = Some(format!("{:.1}", r.open));
                        point
                    })
                    .collect(),
            ))
            .value_labels(true)
            .fill("chart-3")
            .fill_with(|p| if p.value > 1.5 { "chart-4" } else { "chart-5" })
            .fill_gradient(|p, range, to_bar| {
                vec![
                    (0.0, "chart-1".to_string()),
                    (
                        to_bar(range.1 / 2.0),
                        format!("chart-{}", p.value as usize + 1),
                    ),
                ]
            })
            .label_color(|p| if p.value > 1.5 { "success" } else { "error" })
            .tooltip_title(|p| p.label.clone().unwrap_or_default().to_uppercase())
            .tooltip_value(|p, v| format!("{v} on {}", p.label.clone().unwrap_or_default()))
            .tooltip_value_color(|_, v| if v > 1.5 { "success" } else { "error" })
            .tooltip_content(|p| vec![p.label.clone().unwrap_or_default(), "open".to_string()])
            .build();
        assert_eq!(generic.series, typed.series);
        assert_eq!(generic.value_labels, typed.value_labels);
        let first = &generic.series[0].data[0];
        assert_eq!(first.tooltip.title.as_deref(), Some("MON"));
        assert_eq!(first.tooltip.value.as_deref(), Some("1 on mon"));
        assert_eq!(first.tooltip.color.as_deref(), Some("error"));
        assert_eq!(first.tooltip.lines, ["mon", "open"]);
        assert_eq!(first.label_color.as_deref(), Some("error"));
        assert_eq!(first.color.as_deref(), Some("chart-5"));
        assert_eq!(first.gradient.len(), 2);
        assert_eq!(first.gradient[1].1, "chart-2");
    }

    /// CHT-035, roadmap: a radar keeps its vertex dots by default on every
    /// route; both leave the setting unset until asked.
    #[test]
    fn a_radar_leaves_its_vertex_dots_to_the_default_on_both_routes() {
        let typed = RadarChartBuilder::new(rows())
            .label(|r| r.day)
            .value(|r| r.open)
            .build();
        let generic = ChartsBuilder::radar()
            .series(typed.series[0].clone())
            .build();
        assert_eq!(typed.dots, None);
        assert_eq!(generic.dots, None);
        assert_eq!(ChartsBuilder::radar().dots(false).build().dots, Some(false));
        assert_eq!(
            RadarChartBuilder::new(rows())
                .label(|r| r.day)
                .value(|r| r.open)
                .dots(false)
                .build()
                .dots,
            Some(false)
        );
    }

    #[test]
    fn bars_areas_scatter_and_candles_build_their_kinds() {
        let bars = BarChartBuilder::new(rows())
            .band(|r| r.day)
            .value(|r| r.open)
            .fill("chart-3")
            .alignment(BarGrowth::Top)
            .label(|r| format!("{:.1}", r.open))
            .build();
        assert_eq!(bars.growth, BarGrowth::Top);
        assert_eq!(bars.value_labels, Some(true));
        // `.fill()` is the series fill on the typed route as on the generic
        // one, so the routes build the same props (CHT-035).
        assert_eq!(bars.series[0].fill.as_deref(), Some("chart-3"));
        assert_eq!(bars.series[0].color, None);
        let generic = ChartsBuilder::new()
            .chart_type(ChartType::BarVertical)
            .series(bars.series[0].clone())
            .fill("chart-3")
            .build();
        assert_eq!(generic.series[0].fill, bars.series[0].fill);
        assert_eq!(generic.series[0].color, bars.series[0].color);
        // The value label is its own field, not tooltip metadata (CHT-013).
        assert_eq!(bars.series[0].data[0].value_label.as_deref(), Some("1.0"));
        assert!(!bars.series[0].data[0].metadata.contains_key("label"));
        let area = AreaChartBuilder::new(rows())
            .x(|r| r.day)
            .y(|r| r.open)
            .fill("chart-1")
            .y(|r| r.close)
            .stacked(true)
            .step_after()
            .build();
        assert_eq!(area.series.len(), 2);
        // The fill is its own color, apart from the stroke (CHT-012).
        assert_eq!(area.series[0].fill.as_deref(), Some("chart-1"));
        assert_eq!(area.series[0].color, None);
        assert!(area.stacked);
        let scatter = ScatterChartBuilder::new(rows()).y(|r| r.close).build();
        assert_eq!(scatter.chart_type, ChartType::Scatter);
        let candles = CandlestickChartBuilder::new(rows())
            .x(|r| r.day)
            .open(|r| r.open)
            .close(|r| r.close)
            .high(|r| r.open.max(r.close) + 0.5)
            .low(|r| r.open.min(r.close) - 0.5)
            .build();
        let candle = candles.series[0].data[0].candle.unwrap();
        assert!(candle.is_bullish());
        assert_eq!(candle.high, 2.5);
        assert!(!candles.series[0].data[1].candle.unwrap().is_bullish());
    }

    /// CHT-029: the Sankey builder evaluates its accessors once at build,
    /// into node points, throughput texts and label lines, and stores its
    /// options; the props hold no closure.
    #[test]
    fn the_sankey_builder_evaluates_its_accessors() {
        let links =
            [(0, 1, 3.0), (0, 2, 1.0), (2, 1, 0.5)].map(|(s, t, v)| SankeyLink::new(s, t, v));
        let props = SankeyChartBuilder::new(["in", "out", "via"], links)
            .node_label(|n: &&str| n.to_uppercase())
            .node_color(|n: &&str| if *n == "in" { "chart-2" } else { "chart-3" })
            .value_label(|n: &&str, v| format!("{n}={v}"))
            .labels(|n: &&str, v| {
                if *n == "via" {
                    vec![SankeyLabel::new(format!("via {v}")).color("chart-4")]
                } else {
                    Vec::new()
                }
            })
            .value_scale(SankeyValueScale::Sqrt)
            .node_align(SankeyAlign::Left)
            .iterations(3)
            .node_width(3)
            .node_padding(2)
            .link_opacity(0.5)
            .min_link_width(0.5)
            .label_gap(2)
            .name("flows")
            .build();
        assert_eq!(props.chart_type, ChartType::Sankey);
        assert_eq!(props.series[0].name, "flows");
        let nodes = &props.series[0].data;
        // Throughput: the larger of the raw incoming and outgoing totals.
        assert_eq!(
            nodes.iter().map(|p| p.value).collect::<Vec<_>>(),
            vec![4.0, 3.5, 1.0]
        );
        assert_eq!(nodes[1].label.as_deref(), Some("OUT"));
        assert_eq!(nodes[0].color.as_deref(), Some("chart-2"));
        assert_eq!(nodes[2].color.as_deref(), Some("chart-3"));
        let sankey = &props.sankey;
        assert_eq!(sankey.value_labels, vec!["in=4", "out=3.5", "via=1"]);
        assert!(sankey.labels[0].is_empty());
        assert_eq!(
            sankey.labels[2],
            vec![SankeyLabel::new("via 1").color("chart-4")]
        );
        assert_eq!(sankey.links.len(), 3);
        assert_eq!(
            (
                sankey.value_scale,
                sankey.node_align,
                sankey.iterations,
                sankey.node_width,
                sankey.node_padding,
                sankey.link_opacity,
                sankey.min_link_width,
                sankey.label_gap,
            ),
            (
                SankeyValueScale::Sqrt,
                SankeyAlign::Left,
                3,
                3,
                2,
                0.5,
                0.5,
                2
            )
        );
        assert!(
            !props.legend.visible,
            "a Sankey chart's labels name its nodes"
        );
    }

    #[test]
    fn pie_donut_and_radar_builders_evaluate_their_accessors() {
        let pie = PieChartBuilder::new(rows())
            .value(|r| r.close)
            .label(|r| r.day)
            .color(|r| {
                if r.close > r.open {
                    "chart-1"
                } else {
                    "chart-2"
                }
            })
            .name("close")
            .inner_radius(0.3)
            .outer_radius(0.9)
            .pad_angle(0.05)
            .label_gap(3)
            .build();
        assert_eq!(pie.chart_type, ChartType::Pie);
        assert_eq!(pie.series[0].name, "close");
        assert_eq!(pie.series[0].data[1].label.as_deref(), Some("tue"));
        assert_eq!(pie.series[0].data[1].value, 1.5);
        assert_eq!(pie.series[0].data[0].color.as_deref(), Some("chart-1"));
        assert_eq!(pie.series[0].data[1].color.as_deref(), Some("chart-2"));
        assert_eq!(
            (
                pie.radial.inner_radius,
                pie.radial.outer_radius,
                pie.radial.pad_angle,
                pie.radial.label_gap
            ),
            (Some(0.3), 0.9, 0.05, 3)
        );
        let donut = DonutChartBuilder::new(rows()).value(|r| r.open).build();
        assert_eq!(donut.chart_type, ChartType::Donut);
        assert_eq!(donut.series[0].data[0].label.as_deref(), Some("0"));
        let radar = RadarChartBuilder::new(rows())
            .label(|r| r.day)
            .value(|r| r.open)
            .name("open")
            .stroke("chart-3")
            .fill("none")
            .value(|r| r.close)
            .dot()
            .grid(false)
            .grid_levels(5)
            .max_value(4.0)
            .outer_radius(0.8)
            .build();
        assert_eq!(radar.chart_type, ChartType::Radar);
        assert_eq!(radar.series.len(), 2);
        assert_eq!(radar.series[0].color.as_deref(), Some("chart-3"));
        assert_eq!(radar.series[1].data[0].label.as_deref(), Some("mon"));
        assert_eq!(radar.radial.fills, vec![Some("none".to_string()), None]);
        assert!(radar.dots == Some(true) && !radar.radial.grid);
        assert_eq!(
            (
                radar.radial.grid_levels,
                radar.radial.max_value,
                radar.radial.outer_radius
            ),
            (5, Some(4.0), 0.8)
        );
        assert_eq!(radar.clone(), radar, "props stay comparable");
    }
}
