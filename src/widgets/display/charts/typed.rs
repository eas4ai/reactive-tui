//! Typed chart builders over `Vec<T>` with accessor closures, mirroring the
//! reference's method names (CHT-020): `x`, `y`, `band`, `value`, `stroke`,
//! `fill`, `natural`, `linear`, `step_after`, `dot`, `tick_margin`,
//! `alignment`, `label`, `grid`, and for candlesticks `open`, `high`, `low`
//! and `close`. The pie, donut and radar builders take the reference's own
//! names (CHT-029): `value`, `label`, `color`, `inner_radius`,
//! `outer_radius`, `pad_angle` and `label_gap`, for radar `stroke`,
//! `fill`, `dot`, `grid`, `grid_levels` and `max_value`, and for Sankey
//! `new(nodes, links)`, `value_scale`, `node_align`, `iterations`,
//! `node_width`, `node_padding`, `node_color`, `node_label`, `value_label`,
//! `labels`, `link_opacity`, `min_link_width` and `label_gap`. The closures run
//! once at `build()`; the resulting [`ChartProps`] hold plain data points and
//! stay comparable.

use super::{
    BarGrowth, ChartAxis, ChartProps, ChartType, ChartsBuilder, Curve, DataPoint, DataSeries,
    RadialOptions, SankeyAlign, SankeyLabel, SankeyLink, SankeyOptions, SankeyValueScale,
    SizeClass,
};
use crate::component::Element;

type Label<T> = Box<dyn Fn(&T) -> String>;
type Value<T> = Box<dyn Fn(&T) -> f64>;
/// A Sankey node's text from the node and its throughput.
type ValueText<T> = Box<dyn Fn(&T, f64) -> String>;
/// A Sankey node's label lines from the node and its throughput.
type LabelLines<T> = Box<dyn Fn(&T, f64) -> Vec<SankeyLabel>>;

/// Options every typed builder shares.
struct Common {
    base: ChartsBuilder,
    tick_margin: usize,
    grid: bool,
    x_axis: bool,
}

impl Common {
    fn new(kind: ChartType) -> Self {
        Self {
            base: ChartsBuilder::new().chart_type(kind),
            tick_margin: 0,
            grid: true,
            x_axis: true,
        }
    }

    fn finish(self, series: Vec<DataSeries>) -> ChartProps {
        let mut props = self
            .base
            .with_series(series)
            .tick_margin(self.tick_margin)
            .build();
        props.x_axis = ChartAxis {
            show_grid: self.grid,
            show_labels: self.x_axis,
            ..props.x_axis
        };
        props.y_axis = ChartAxis {
            show_grid: self.grid,
            ..props.y_axis
        };
        props
    }
}

macro_rules! common_methods {
    () => {
        /// Chart title.
        pub fn title(mut self, title: impl Into<String>) -> Self {
            self.common.base = self.common.base.title(title);
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

        /// Draw the category axis and its labels.
        pub fn x_axis(mut self, show: bool) -> Self {
            self.common.x_axis = show;
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

/// One series described by accessors.
struct SeriesSpec<T> {
    name: Option<String>,
    value: Value<T>,
    color: Option<String>,
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
    specs
        .iter()
        .enumerate()
        .map(|(i, spec)| {
            let mut series = DataSeries::new(
                spec.name
                    .clone()
                    .unwrap_or_else(|| format!("series {}", i + 1)),
                points(data, label, &spec.value),
            );
            series.color = spec.color.clone();
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
}

impl<T> LineChartBuilder<T> {
    /// A line chart over `data`.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::Line),
            data: data.into_iter().collect(),
            x: None,
            series: Vec::new(),
        }
    }

    common_methods!();

    /// Show every n-th axis label; 0 avoids overlap automatically.
    pub fn tick_margin(mut self, margin: usize) -> Self {
        self.common.tick_margin = margin;
        self
    }

    /// Draw grid lines.
    pub fn grid(mut self, grid: bool) -> Self {
        self.common.grid = grid;
        self
    }

    /// Category label accessor.
    pub fn x<S: Into<String>>(mut self, x: impl Fn(&T) -> S + 'static) -> Self {
        self.x = Some(Box::new(move |d| x(d).into()));
        self
    }

    /// Value accessor; each call adds a series.
    pub fn y(mut self, y: impl Fn(&T) -> f64 + 'static) -> Self {
        self.series.push(SeriesSpec {
            name: None,
            value: Box::new(y),
            color: None,
        });
        self
    }

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

    /// Smooth spline strokes.
    pub fn natural(mut self) -> Self {
        self.common.base = self.common.base.curve(Curve::Natural);
        self
    }

    /// Straight strokes.
    pub fn linear(mut self) -> Self {
        self.common.base = self.common.base.curve(Curve::Linear);
        self
    }

    /// Step strokes holding each value until the next point.
    pub fn step_after(mut self) -> Self {
        self.common.base = self.common.base.curve(Curve::StepAfter);
        self
    }

    /// Draw a dot at every point.
    pub fn dot(mut self) -> Self {
        self.common.base = self.common.base.dots(true);
        self
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        let series = series(&self.data, self.x.as_ref(), &self.series);
        self.common.finish(series)
    }
}

/// An area chart over `Vec<T>`.
pub struct AreaChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    x: Option<Label<T>>,
    series: Vec<SeriesSpec<T>>,
    fills: Vec<Option<String>>,
}

impl<T> AreaChartBuilder<T> {
    /// An area chart over `data`.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::Area),
            data: data.into_iter().collect(),
            x: None,
            series: Vec::new(),
            fills: Vec::new(),
        }
    }

    common_methods!();

    /// Show every n-th axis label; 0 avoids overlap automatically.
    pub fn tick_margin(mut self, margin: usize) -> Self {
        self.common.tick_margin = margin;
        self
    }

    /// Draw grid lines.
    pub fn grid(mut self, grid: bool) -> Self {
        self.common.grid = grid;
        self
    }

    /// Category label accessor.
    pub fn x<S: Into<String>>(mut self, x: impl Fn(&T) -> S + 'static) -> Self {
        self.x = Some(Box::new(move |d| x(d).into()));
        self
    }

    /// Value accessor; each call adds a series.
    pub fn y(mut self, y: impl Fn(&T) -> f64 + 'static) -> Self {
        self.series.push(SeriesSpec {
            name: None,
            value: Box::new(y),
            color: None,
        });
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

    /// Stroke color token of the series added last.
    pub fn stroke(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.color = Some(token.into());
        }
        self
    }

    /// Fill color token of the series added last.
    pub fn fill(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.fills.last_mut() {
            *last = Some(token.into());
        }
        self
    }

    /// Stack the series on top of each other.
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.common.base = self.common.base.stacked(stacked);
        self
    }

    /// Smooth spline strokes.
    pub fn natural(mut self) -> Self {
        self.common.base = self.common.base.curve(Curve::Natural);
        self
    }

    /// Straight strokes.
    pub fn linear(mut self) -> Self {
        self.common.base = self.common.base.curve(Curve::Linear);
        self
    }

    /// Step strokes.
    pub fn step_after(mut self) -> Self {
        self.common.base = self.common.base.curve(Curve::StepAfter);
        self
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        let mut series = series(&self.data, self.x.as_ref(), &self.series);
        for (s, fill) in series.iter_mut().zip(&self.fills) {
            if let Some(fill) = fill {
                s.color = Some(fill.clone());
            }
        }
        self.common.finish(series)
    }
}

/// A scatter plot over `Vec<T>`.
pub struct ScatterChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    x: Option<Label<T>>,
    series: Vec<SeriesSpec<T>>,
}

impl<T> ScatterChartBuilder<T> {
    /// A scatter plot over `data`.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::Scatter),
            data: data.into_iter().collect(),
            x: None,
            series: Vec::new(),
        }
    }

    common_methods!();

    /// Show every n-th axis label; 0 avoids overlap automatically.
    pub fn tick_margin(mut self, margin: usize) -> Self {
        self.common.tick_margin = margin;
        self
    }

    /// Draw grid lines.
    pub fn grid(mut self, grid: bool) -> Self {
        self.common.grid = grid;
        self
    }

    /// Category label accessor.
    pub fn x<S: Into<String>>(mut self, x: impl Fn(&T) -> S + 'static) -> Self {
        self.x = Some(Box::new(move |d| x(d).into()));
        self
    }

    /// Value accessor; each call adds a series.
    pub fn y(mut self, y: impl Fn(&T) -> f64 + 'static) -> Self {
        self.series.push(SeriesSpec {
            name: None,
            value: Box::new(y),
            color: None,
        });
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
        let series = series(&self.data, self.x.as_ref(), &self.series);
        self.common.finish(series)
    }
}

/// A bar chart over `Vec<T>`.
pub struct BarChartBuilder<T> {
    common: Common,
    data: Vec<T>,
    band: Option<Label<T>>,
    label: Option<Label<T>>,
    series: Vec<SeriesSpec<T>>,
}

impl<T> BarChartBuilder<T> {
    /// A bar chart over `data`.
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            common: Common::new(ChartType::BarVertical),
            data: data.into_iter().collect(),
            band: None,
            label: None,
            series: Vec::new(),
        }
    }

    common_methods!();

    /// Show every n-th axis label; 0 avoids overlap automatically.
    pub fn tick_margin(mut self, margin: usize) -> Self {
        self.common.tick_margin = margin;
        self
    }

    /// Draw grid lines.
    pub fn grid(mut self, grid: bool) -> Self {
        self.common.grid = grid;
        self
    }

    /// Category (band) label accessor.
    pub fn band<S: Into<String>>(mut self, band: impl Fn(&T) -> S + 'static) -> Self {
        self.band = Some(Box::new(move |d| band(d).into()));
        self
    }

    /// Value accessor; each call adds a series.
    pub fn value(mut self, value: impl Fn(&T) -> f64 + 'static) -> Self {
        self.series.push(SeriesSpec {
            name: None,
            value: Box::new(value),
            color: None,
        });
        self
    }

    /// Name of the series added last.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.name = Some(name.into());
        }
        self
    }

    /// Fill color token of the series added last.
    pub fn fill(mut self, token: impl Into<String>) -> Self {
        if let Some(last) = self.series.last_mut() {
            last.color = Some(token.into());
        }
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

    /// Stack the series end to end.
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.common.base = self.common.base.stacked(stacked);
        self
    }

    /// Evaluate the accessors into props.
    pub fn build(self) -> ChartProps {
        let mut series = series(&self.data, self.band.as_ref(), &self.series);
        if let Some(label) = &self.label {
            for s in &mut series {
                for (point, datum) in s.data.iter_mut().zip(&self.data) {
                    point.metadata.insert("label".into(), label(datum));
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
        }
    }

    common_methods!();

    /// Show every n-th axis label; 0 avoids overlap automatically.
    pub fn tick_margin(mut self, margin: usize) -> Self {
        self.common.tick_margin = margin;
        self
    }

    /// Draw grid lines.
    pub fn grid(mut self, grid: bool) -> Self {
        self.common.grid = grid;
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
            points.push(point);
        }
        let series = vec![DataSeries::new("candles", points)];
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
        common.x_axis = false;
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
        common.x_axis = false;
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
    dots: bool,
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
            dots: false,
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
        self.series.push(SeriesSpec {
            name: None,
            value: Box::new(value),
            color: None,
        });
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

    /// Draw a dot at each vertex.
    pub fn dot(mut self) -> Self {
        self.dots = true;
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
        common.x_axis = false;
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
        assert_eq!(props.curve, Curve::Linear);
        assert_eq!(props.tick_margin, 2);
        assert!(!props.x_axis.show_grid);
        assert_eq!(props.clone(), props, "props stay comparable");
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
        assert_eq!(bars.series[0].data[0].metadata["label"], "1.0");
        let area = AreaChartBuilder::new(rows())
            .x(|r| r.day)
            .y(|r| r.open)
            .fill("chart-1")
            .y(|r| r.close)
            .stacked(true)
            .step_after()
            .build();
        assert_eq!(area.series.len(), 2);
        assert_eq!(area.series[0].color.as_deref(), Some("chart-1"));
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
        assert!(radar.dots && !radar.radial.grid);
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
