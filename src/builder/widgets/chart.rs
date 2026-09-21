//! Chart widget builder
//!
//! This module provides the ChartBuilder for creating various types of charts
//! including bar charts, line charts, pie charts, and more.

use crate::component::Element;
use crate::widgets::display::{
    ChartAxis, ChartLegend, ChartProps, ChartType, DataPoint, DataSeries, LegendPosition,
};

/// Create a Chart with fluent configuration
pub fn chart() -> ChartBuilder {
    ChartBuilder::new()
}

/// Builder for Chart components with fluent API
pub struct ChartBuilder {
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
    class: Option<String>,
}

impl ChartBuilder {
    fn new() -> Self {
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
            ],
            animated: false,
            animation_duration: 1000,
            class: None,
        }
    }

    /// Set chart type
    pub fn chart_type(mut self, chart_type: ChartType) -> Self {
        self.chart_type = chart_type;
        self
    }

    /// Create a bar chart
    pub fn bar_chart(mut self) -> Self {
        self.chart_type = ChartType::BarVertical;
        self
    }

    /// Create a horizontal bar chart
    pub fn horizontal_bar_chart(mut self) -> Self {
        self.chart_type = ChartType::BarHorizontal;
        self
    }

    /// Create a line chart
    pub fn line_chart(mut self) -> Self {
        self.chart_type = ChartType::Line;
        self
    }

    /// Create a pie chart
    pub fn pie_chart(mut self) -> Self {
        self.chart_type = ChartType::Pie;
        self
    }

    /// Create an area chart
    pub fn area_chart(mut self) -> Self {
        self.chart_type = ChartType::Area;
        self
    }

    /// Create a scatter plot
    pub fn scatter_plot(mut self) -> Self {
        self.chart_type = ChartType::Scatter;
        self
    }

    /// Add a data series
    pub fn series(mut self, series: DataSeries) -> Self {
        self.series.push(series);
        self
    }

    /// Add multiple data series
    pub fn series_list(mut self, series: Vec<DataSeries>) -> Self {
        self.series.extend(series);
        self
    }

    /// Add a simple data series from values
    pub fn simple_series(mut self, name: &str, values: Vec<f64>) -> Self {
        let data_points: Vec<DataPoint> = values.into_iter().map(DataPoint::new).collect();
        self.series.push(DataSeries::new(name, data_points));
        self
    }

    /// Add a labeled data series
    pub fn labeled_series(mut self, name: &str, data: Vec<(f64, &str)>) -> Self {
        let data_points: Vec<DataPoint> = data
            .into_iter()
            .map(|(value, label)| DataPoint::with_label(value, label))
            .collect();
        self.series.push(DataSeries::new(name, data_points));
        self
    }

    /// Set chart title
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set chart dimensions
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Configure axes
    pub fn axes(mut self, x_title: Option<String>, y_title: Option<String>) -> Self {
        self.x_axis.title = x_title;
        self.y_axis.title = y_title;
        self
    }

    /// Set custom color palette
    pub fn colors(mut self, colors: Vec<String>) -> Self {
        self.color_palette = colors;
        self
    }

    /// Enable animation
    pub fn animated(mut self, duration_ms: u64) -> Self {
        self.animated = true;
        self.animation_duration = duration_ms;
        self
    }

    /// Configure legend
    pub fn legend(mut self, visible: bool, position: LegendPosition) -> Self {
        self.legend.visible = visible;
        self.legend.position = position;
        self
    }

    /// Set CSS classes
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Chart element
    pub fn build(self) -> Element {
        self.build_with_name("Chart")
    }

    /// Build the Chart element with a custom component name
    pub fn build_with_name(self, component_name: &str) -> Element {
        let props = ChartProps {
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
            show_tooltips: true,
            class: self.class.clone(),
        };

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
