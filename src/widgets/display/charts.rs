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

    /// Create a scatter plot
    pub fn scatter() -> Self {
        Self::new().chart_type(ChartType::Scatter)
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
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("Charts").with_props(self.build())
    }
}

impl Default for ChartsBuilder {
    fn default() -> Self {
        Self {
            chart_type: ChartType::BarVertical,
            series: Vec::new(),
            title: None,
            width: 80,
            height: 20,
            x_axis: ChartAxis::default(),
            y_axis: ChartAxis::default(),
            legend: ChartLegend::default(),
            color_palette: vec![
                "#3b82f6".to_string(),
                "#ef4444".to_string(),
                "#10b981".to_string(),
                "#f59e0b".to_string(),
                "#8b5cf6".to_string(),
                "#06b6d4".to_string(),
                "#f97316".to_string(),
                "#84cc16".to_string(),
            ],
            animated: false,
            animation_duration: 1000,
            show_tooltips: true,
            class: None,
        }
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
}

impl DataPoint {
    /// Create a new data point with just a value
    pub fn new(value: f64) -> Self {
        Self {
            value,
            label: None,
            color: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a data point with value and label
    pub fn with_label(value: f64, label: impl Into<String>) -> Self {
        Self {
            value,
            label: Some(label.into()),
            color: None,
            metadata: HashMap::new(),
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
}

/// Props for the Chart component
#[derive(Clone, PartialEq)]
pub struct ChartProps {
    /// Type of chart to render
    pub chart_type: ChartType,
    /// Data series to display
    pub series: Vec<DataSeries>,
    /// Chart title
    pub title: Option<String>,
    /// Chart width in characters
    pub width: u16,
    /// Chart height in characters
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
            width: 80,
            height: 20,
            x_axis: ChartAxis::default(),
            y_axis: ChartAxis::default(),
            legend: ChartLegend::default(),
            color_palette: vec![
                "#3b82f6".to_string(), // blue
                "#ef4444".to_string(), // red
                "#10b981".to_string(), // green
                "#f59e0b".to_string(), // yellow
                "#8b5cf6".to_string(), // purple
                "#06b6d4".to_string(), // cyan
                "#f97316".to_string(), // orange
                "#84cc16".to_string(), // lime
            ],
            animated: false,
            animation_duration: 1000,
            show_tooltips: true,
            class: None,
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
pub struct Chart;

mod live;

impl Component for Chart {
    type Props = ChartProps;
    type State = ChartState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveChart>(live::LiveProps {
            config: props.clone(),
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
