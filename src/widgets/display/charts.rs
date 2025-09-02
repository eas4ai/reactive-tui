//! Chart Widgets for Data Visualization
//!
//! Provides a comprehensive set of chart components for terminal-based data visualization:
//! - Bar Charts (horizontal and vertical)
//! - Line Charts with multiple series
//! - Pie Charts with customizable segments
//! - Area Charts and Scatter Plots
//! - Real-time data support with animations

use crate::component::{Component, Element, Props};
use crate::prelude::LayoutType;
use std::collections::HashMap;

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
    /// Fill style for area charts
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
            fill_style: FillStyle::None,
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
    Floating(u16, u16), // x, y coordinates
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

impl Component for Chart {
    type Props = ChartProps;
    type State = ChartState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let mut container = Element::layout(LayoutType::Flex);

        // Add title if present
        if let Some(title) = &props.title {
            container = container.child(
                Element::text(title)
                    .with_class("chart-title text-center font-bold mb-2")
            );
        }

        // Create the main chart area
        let chart_content = match props.chart_type {
            ChartType::BarVertical => self.render_bar_chart_vertical(props, state),
            ChartType::BarHorizontal => self.render_bar_chart_horizontal(props, state),
            ChartType::Line => self.render_line_chart(props, state),
            ChartType::Area => self.render_area_chart(props, state),
            ChartType::Pie => self.render_pie_chart(props, state),
            ChartType::Donut => self.render_donut_chart(props, state),
            ChartType::Scatter => self.render_scatter_plot(props, state),
        };

        container = container.child(chart_content);

        // Add legend if enabled
        if props.legend.visible && !props.series.is_empty() {
            container = container.child(self.render_legend(props, state));
        }

        // Apply CSS classes
        if let Some(class) = &props.class {
            container = container.with_class(class);
        }

        container
    }
}

impl Chart {
    /// Render a vertical bar chart
    fn render_bar_chart_vertical(&self, props: &ChartProps, _state: &ChartState) -> Element {
        let chart_data = self.prepare_chart_data(props);

        // Create ASCII bar chart representation
        let mut chart_lines = Vec::new();

        // Calculate bar dimensions
        let max_value = chart_data.max_value;
        let bar_height = props.height.saturating_sub(4); // Leave space for axes
        let bar_width = props.width / props.series.len().max(1) as u16;

        // Generate chart lines from top to bottom
        for row in 0..bar_height {
            let mut line = String::new();

            for (series_idx, series) in props.series.iter().enumerate() {
                if !series.visible {
                    continue;
                }

                for (_point_idx, point) in series.data.iter().enumerate() {
                    let normalized_height = if max_value > 0.0 {
                        (point.value / max_value * bar_height as f64) as u16
                    } else {
                        0
                    };

                    let current_row_from_bottom = bar_height - row - 1;

                    if current_row_from_bottom < normalized_height {
                        line.push('█'); // Full block character
                    } else {
                        line.push(' ');
                    }
                }

                // Add spacing between series
                if series_idx < props.series.len() - 1 {
                    line.push(' ');
                }
            }

            chart_lines.push(line);
        }

        // Create the chart element
        let mut chart_element = Element::layout(LayoutType::Flex);

        for line in chart_lines {
            chart_element = chart_element.child(Element::text(line));
        }

        // Add x-axis labels if enabled
        if props.x_axis.show_labels {
            let mut x_labels = String::new();
            for (idx, series) in props.series.iter().enumerate() {
                if !series.visible {
                    continue;
                }

                let label = if idx < props.x_axis.custom_labels.len() {
                    &props.x_axis.custom_labels[idx]
                } else {
                    &series.name
                };

                x_labels.push_str(&format!("{:^width$}", label, width = bar_width as usize));
                if idx < props.series.len() - 1 {
                    x_labels.push(' ');
                }
            }
            chart_element = chart_element.child(Element::text(x_labels));
        }

        chart_element
    }

    /// Render a horizontal bar chart
    fn render_bar_chart_horizontal(&self, props: &ChartProps, _state: &ChartState) -> Element {
        let chart_data = self.prepare_chart_data(props);
        let max_value = chart_data.max_value;
        let bar_width = props.width.saturating_sub(20); // Leave space for labels

        let mut chart_element = Element::layout(LayoutType::Flex);

        for series in &props.series {
            if !series.visible {
                continue;
            }

            for point in &series.data {
                let normalized_width = if max_value > 0.0 {
                    (point.value / max_value * bar_width as f64) as u16
                } else {
                    0
                };

                let mut bar_line = String::new();

                // Add label
                let label = point.label.as_ref().unwrap_or(&series.name);
                bar_line.push_str(&format!("{:>15} ", label));

                // Add bar
                for _ in 0..normalized_width {
                    bar_line.push('█');
                }

                // Add value
                bar_line.push_str(&format!(" {:.1}", point.value));

                chart_element = chart_element.child(Element::text(bar_line));
            }
        }

        chart_element
    }

    /// Render a line chart
    fn render_line_chart(&self, props: &ChartProps, _state: &ChartState) -> Element {
        let chart_data = self.prepare_chart_data(props);

        // Create a simple ASCII line chart
        let mut chart_lines = vec![vec![' '; props.width as usize]; props.height as usize];

        for series in &props.series {
            if !series.visible || series.data.is_empty() {
                continue;
            }

            // Plot points and connect with lines
            for (i, point) in series.data.iter().enumerate() {
                let x = (i as f64 / (series.data.len() - 1).max(1) as f64 * (props.width - 1) as f64) as usize;
                let y = if chart_data.max_value > chart_data.min_value {
                    ((chart_data.max_value - point.value) / (chart_data.max_value - chart_data.min_value)
                     * (props.height - 1) as f64) as usize
                } else {
                    props.height as usize / 2
                };

                if x < props.width as usize && y < props.height as usize {
                    chart_lines[y][x] = '●'; // Bullet point for data points
                }

                // Connect to previous point with line
                if i > 0 && series.line_style != LineStyle::None {
                    let prev_point = &series.data[i - 1];
                    let prev_x = ((i - 1) as f64 / (series.data.len() - 1).max(1) as f64 * (props.width - 1) as f64) as usize;
                    let prev_y = if chart_data.max_value > chart_data.min_value {
                        ((chart_data.max_value - prev_point.value) / (chart_data.max_value - chart_data.min_value)
                         * (props.height - 1) as f64) as usize
                    } else {
                        props.height as usize / 2
                    };

                    // Simple line drawing between points
                    self.draw_line(&mut chart_lines, prev_x, prev_y, x, y, &series.line_style);
                }
            }
        }

        // Convert chart lines to elements
        let mut chart_element = Element::layout(LayoutType::Flex);
        for line in chart_lines {
            chart_element = chart_element.child(Element::text(line.into_iter().collect::<String>()));
        }

        chart_element
    }

    /// Render an area chart (filled line chart)
    fn render_area_chart(&self, props: &ChartProps, state: &ChartState) -> Element {
        // For now, render as line chart with fill indication
        // In a full implementation, this would fill the area under the line
        let mut line_chart = self.render_line_chart(props, state);

        // Add area fill indication in the title or styling
        if let Some(title) = &props.title {
            line_chart = Element::layout(LayoutType::Flex)
                .child(Element::text(format!("{} (Area)", title)))
                .child(line_chart);
        }

        line_chart
    }

    /// Render a pie chart
    fn render_pie_chart(&self, props: &ChartProps, _state: &ChartState) -> Element {
        let chart_data = self.prepare_chart_data(props);
        let total_value = chart_data.total_value;

        if total_value == 0.0 {
            return Element::text("No data to display");
        }

        let mut chart_element = Element::layout(LayoutType::Flex);

        // Simple text-based pie chart representation
        chart_element = chart_element.child(Element::text("┌─ Pie Chart ─┐"));

        for series in &props.series {
            if !series.visible {
                continue;
            }

            for point in &series.data {
                let percentage = (point.value / total_value * 100.0) as u16;
                let bar_length = (percentage as f64 / 100.0 * 20.0) as usize; // 20 char width

                let mut line = String::new();
                line.push_str(&format!("│ {:>12} ",
                    point.label.as_ref().unwrap_or(&series.name)));

                // Visual representation
                for _ in 0..bar_length {
                    line.push('█');
                }
                for _ in bar_length..20 {
                    line.push('░');
                }

                line.push_str(&format!(" {:>3}% │", percentage));

                chart_element = chart_element.child(Element::text(line));
            }
        }

        chart_element = chart_element.child(Element::text("└──────────────┘"));
        chart_element
    }

    /// Render a donut chart (pie chart with hole)
    fn render_donut_chart(&self, props: &ChartProps, state: &ChartState) -> Element {
        // For now, render as pie chart with donut indication
        let mut pie_chart = self.render_pie_chart(props, state);

        // Modify the title to indicate it's a donut chart
        pie_chart = Element::layout(LayoutType::Flex)
            .child(Element::text("┌─ Donut Chart ─┐"))
            .child(pie_chart);

        pie_chart
    }

    /// Render a scatter plot
    fn render_scatter_plot(&self, props: &ChartProps, _state: &ChartState) -> Element {
        let chart_data = self.prepare_chart_data(props);

        // Create scatter plot grid
        let mut chart_lines = vec![vec![' '; props.width as usize]; props.height as usize];

        for series in &props.series {
            if !series.visible {
                continue;
            }

            for (i, point) in series.data.iter().enumerate() {
                let x = (i as f64 / series.data.len().max(1) as f64 * (props.width - 1) as f64) as usize;
                let y = if chart_data.max_value > chart_data.min_value {
                    ((chart_data.max_value - point.value) / (chart_data.max_value - chart_data.min_value)
                     * (props.height - 1) as f64) as usize
                } else {
                    props.height as usize / 2
                };

                if x < props.width as usize && y < props.height as usize {
                    chart_lines[y][x] = '•'; // Small bullet for scatter points
                }
            }
        }

        // Convert to elements
        let mut chart_element = Element::layout(LayoutType::Flex);
        for line in chart_lines {
            chart_element = chart_element.child(Element::text(line.into_iter().collect::<String>()));
        }

        chart_element
    }

    /// Render chart legend
    fn render_legend(&self, props: &ChartProps, _state: &ChartState) -> Element {
        let mut legend_element = Element::layout(LayoutType::Flex);

        legend_element = legend_element.child(Element::text("Legend:"));

        for (idx, series) in props.series.iter().enumerate() {
            if !series.visible {
                continue;
            }

            let color_indicator = if idx < props.color_palette.len() {
                "█" // Use color from palette
            } else {
                "■" // Default indicator
            };

            let legend_line = format!("{} {}", color_indicator, series.name);
            legend_element = legend_element.child(Element::text(legend_line));
        }

        legend_element
    }

    /// Prepare chart data for rendering
    fn prepare_chart_data(&self, props: &ChartProps) -> ChartData {
        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;
        let mut total_value = 0.0;
        let mut point_count = 0;

        for series in &props.series {
            if !series.visible {
                continue;
            }

            for point in &series.data {
                min_value = min_value.min(point.value);
                max_value = max_value.max(point.value);
                total_value += point.value;
                point_count += 1;
            }
        }

        // Handle edge cases
        if point_count == 0 {
            min_value = 0.0;
            max_value = 1.0;
        } else if min_value == max_value {
            min_value -= 1.0;
            max_value += 1.0;
        }

        // Apply axis overrides
        if let Some(axis_min) = props.y_axis.min {
            min_value = axis_min;
        }
        if let Some(axis_max) = props.y_axis.max {
            max_value = axis_max;
        }

        ChartData {
            min_value,
            max_value,
            total_value,
            _point_count: point_count,
        }
    }

    /// Draw a line between two points in the chart grid
    fn draw_line(&self, grid: &mut [Vec<char>], x1: usize, y1: usize, x2: usize, y2: usize, style: &LineStyle) {
        let line_char = match style {
            LineStyle::Solid => '─',
            LineStyle::Dashed => '┄',
            LineStyle::Dotted => '·',
            LineStyle::None => return,
        };

        // Simple line drawing using Bresenham-like algorithm
        let dx = (x2 as i32 - x1 as i32).abs();
        let dy = (y2 as i32 - y1 as i32).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x1 as i32;
        let mut y = y1 as i32;

        loop {
            if x >= 0 && x < grid[0].len() as i32 && y >= 0 && y < grid.len() as i32 {
                let ux = x as usize;
                let uy = y as usize;
                if grid[uy][ux] == ' ' {
                    grid[uy][ux] = line_char;
                }
            }

            if x == x2 as i32 && y == y2 as i32 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }
}

/// Prepared chart data for rendering
#[derive(Debug, Clone)]
struct ChartData {
    min_value: f64,
    max_value: f64,
    total_value: f64,
    _point_count: usize,
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
