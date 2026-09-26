use reactive_tui::component::{
    registry::{global_cleanup_all, register_component},
    Element,
};
use reactive_tui::render::tree::element_to_render_node;
use reactive_tui::widgets::display::{
    Chart, ChartAxis, ChartLegend, ChartProps, ChartType, DataPoint, DataSeries, FillStyle,
    LegendPosition, LineStyle,
};

fn create_sample_data() -> Vec<DataSeries> {
    vec![
        DataSeries::new(
            "Sales",
            vec![
                DataPoint::with_label(100.0, "Q1"),
                DataPoint::with_label(150.0, "Q2"),
                DataPoint::with_label(120.0, "Q3"),
                DataPoint::with_label(180.0, "Q4"),
            ],
        )
        .with_color("#3b82f6"),
        DataSeries::new(
            "Revenue",
            vec![
                DataPoint::with_label(80.0, "Q1"),
                DataPoint::with_label(120.0, "Q2"),
                DataPoint::with_label(140.0, "Q3"),
                DataPoint::with_label(200.0, "Q4"),
            ],
        )
        .with_color("#ef4444"),
        DataSeries::new(
            "Profit",
            vec![
                DataPoint::with_label(20.0, "Q1"),
                DataPoint::with_label(30.0, "Q2"),
                DataPoint::with_label(25.0, "Q3"),
                DataPoint::with_label(45.0, "Q4"),
            ],
        )
        .with_color("#10b981"),
    ]
}

#[test]
fn test_chart_creation() {
    global_cleanup_all().unwrap();
    register_component::<Chart>("Chart").unwrap();

    println!("🧪 Testing Chart Creation");

    let data = create_sample_data();

    // Create bar chart
    let bar_props = ChartProps::bar_chart(data.clone())
        .with_title("Quarterly Performance")
        .with_size(80, 20);

    assert_eq!(bar_props.chart_type, ChartType::BarVertical);
    assert_eq!(bar_props.title.as_deref(), Some("Quarterly Performance"));
    assert_eq!((bar_props.width, bar_props.height), (80, 20));

    let element = Element::component_with_props("Chart", bar_props);
    let render_node = element_to_render_node(element);
    assert_eq!(render_node.node_type(), "Chart");
}

#[test]
fn test_data_point_creation() {
    println!("🧪 Testing Data Point Creation");

    // Test basic data point
    let point1 = DataPoint::new(42.5);
    assert_eq!(point1.value, 42.5);
    assert_eq!(point1.label, None);
    assert_eq!(point1.color, None);

    // Test data point with label
    let point2 = DataPoint::with_label(100.0, "January");
    assert_eq!(point2.value, 100.0);
    assert_eq!(point2.label, Some("January".to_string()));

    // Test data point with color and metadata
    let point3 = DataPoint::with_label(75.0, "February")
        .with_color("#ff0000")
        .with_metadata("category", "sales")
        .with_metadata("region", "north");

    assert_eq!(point3.color, Some("#ff0000".to_string()));
    assert_eq!(point3.metadata.get("category"), Some(&"sales".to_string()));
    assert_eq!(point3.metadata.get("region"), Some(&"north".to_string()));

    println!("✅ Data point creation test passed");
}

#[test]
fn test_data_series_creation() {
    println!("🧪 Testing Data Series Creation");

    let points = vec![
        DataPoint::new(10.0),
        DataPoint::new(20.0),
        DataPoint::new(15.0),
    ];

    let series = DataSeries::new("Test Series", points.clone())
        .with_color("#00ff00")
        .with_line_style(LineStyle::Dashed)
        .with_fill_style(FillStyle::Gradient);

    assert_eq!(series.name, "Test Series");
    assert_eq!(series.data.len(), 3);
    assert_eq!(series.color, Some("#00ff00".to_string()));
    assert_eq!(series.line_style, LineStyle::Dashed);
    assert_eq!(series.fill_style, FillStyle::Gradient);
    assert!(series.visible);

    println!("✅ Data series creation test passed");
}

#[test]
fn test_chart_types() {
    global_cleanup_all().unwrap();
    register_component::<Chart>("ChartTypes").unwrap();

    println!("🧪 Testing Different Chart Types");

    let data = create_sample_data();

    // Test bar chart (vertical)
    let bar_vertical = ChartProps {
        chart_type: ChartType::BarVertical,
        series: data.clone(),
        ..Default::default()
    };

    // Test bar chart (horizontal)
    let bar_horizontal = ChartProps {
        chart_type: ChartType::BarHorizontal,
        series: data.clone(),
        ..Default::default()
    };

    // Test line chart
    let line_chart = ChartProps::line_chart(data.clone()).with_title("Line Chart Example");

    // Test pie chart
    let pie_chart = ChartProps::pie_chart(data.clone()).with_title("Pie Chart Example");

    // Test area chart
    let area_chart = ChartProps {
        chart_type: ChartType::Area,
        series: data.clone(),
        title: Some("Area Chart".to_string()),
        ..Default::default()
    };

    // Test scatter plot
    let scatter_plot = ChartProps {
        chart_type: ChartType::Scatter,
        series: data,
        title: Some("Scatter Plot".to_string()),
        ..Default::default()
    };

    // Create elements for each chart type
    let charts = [
        (bar_vertical, ChartType::BarVertical),
        (bar_horizontal, ChartType::BarHorizontal),
        (line_chart, ChartType::Line),
        (pie_chart, ChartType::Pie),
        (area_chart, ChartType::Area),
        (scatter_plot, ChartType::Scatter),
    ];
    for (props, expected) in charts {
        assert_eq!(props.chart_type, expected);
        assert!(!props.series.is_empty(), "sample series are retained");
        let element = Element::component_with_props("ChartTypes", props);
        assert_eq!(element_to_render_node(element).node_type(), "ChartTypes");
    }
}

#[test]
fn test_chart_configuration() {
    println!("🧪 Testing Chart Configuration");

    let data = create_sample_data();

    // Test comprehensive chart configuration
    let chart_props = ChartProps::bar_chart(data)
        .with_title("Advanced Chart Configuration")
        .with_size(100, 30)
        .with_animation(2000)
        .with_axes(
            Some("Time Period".to_string()),
            Some("Value ($)".to_string()),
        )
        .with_colors(vec![
            "#ff6b6b".to_string(),
            "#4ecdc4".to_string(),
            "#45b7d1".to_string(),
            "#96ceb4".to_string(),
        ]);

    assert_eq!(
        chart_props.title,
        Some("Advanced Chart Configuration".to_string())
    );
    assert_eq!(chart_props.width, 100);
    assert_eq!(chart_props.height, 30);
    assert!(chart_props.animated);
    assert_eq!(chart_props.animation_duration, 2000);
    assert_eq!(chart_props.x_axis.title, Some("Time Period".to_string()));
    assert_eq!(chart_props.y_axis.title, Some("Value ($)".to_string()));
    assert_eq!(chart_props.color_palette.len(), 4);

    println!("✅ Chart configuration test passed");
}

#[test]
fn test_chart_axis_configuration() {
    println!("🧪 Testing Chart Axis Configuration");

    let x_axis = ChartAxis {
        title: Some("X Axis".to_string()),
        min: Some(0.0),
        max: Some(100.0),
        show_grid: true,
        show_labels: true,
        tick_count: 10,
        custom_labels: vec!["Start".to_string(), "Middle".to_string(), "End".to_string()],
    };

    assert_eq!(x_axis.title, Some("X Axis".to_string()));
    assert_eq!(x_axis.min, Some(0.0));
    assert_eq!(x_axis.max, Some(100.0));
    assert!(x_axis.show_grid);
    assert!(x_axis.show_labels);
    assert_eq!(x_axis.tick_count, 10);
    assert_eq!(x_axis.custom_labels.len(), 3);

    println!("✅ Chart axis configuration test passed");
}

#[test]
fn test_chart_legend_configuration() {
    println!("🧪 Testing Chart Legend Configuration");

    let legend = ChartLegend {
        visible: true,
        position: LegendPosition::Bottom,
        max_width: Some(200),
    };

    assert!(legend.visible);
    assert_eq!(legend.position, LegendPosition::Bottom);
    assert_eq!(legend.max_width, Some(200));

    // Test floating legend position
    let floating_legend = ChartLegend {
        visible: true,
        position: LegendPosition::Floating(10, 20),
        max_width: None,
    };

    match floating_legend.position {
        LegendPosition::Floating(x, y) => {
            assert_eq!(x, 10);
            assert_eq!(y, 20);
        }
        _ => panic!("Expected floating legend position"),
    }

    println!("✅ Chart legend configuration test passed");
}

#[test]
fn test_line_and_fill_styles() {
    println!("🧪 Testing Line and Fill Styles");

    // Test line styles
    let solid_line = LineStyle::Solid;
    let dashed_line = LineStyle::Dashed;
    let dotted_line = LineStyle::Dotted;
    let no_line = LineStyle::None;

    assert_eq!(solid_line, LineStyle::Solid);
    assert_eq!(dashed_line, LineStyle::Dashed);
    assert_eq!(dotted_line, LineStyle::Dotted);
    assert_eq!(no_line, LineStyle::None);

    // Test fill styles
    let no_fill = FillStyle::None;
    let solid_fill = FillStyle::Solid;
    let gradient_fill = FillStyle::Gradient;
    let pattern_fill = FillStyle::Pattern("diagonal".to_string());

    assert_eq!(no_fill, FillStyle::None);
    assert_eq!(solid_fill, FillStyle::Solid);
    assert_eq!(gradient_fill, FillStyle::Gradient);

    match pattern_fill {
        FillStyle::Pattern(pattern) => assert_eq!(pattern, "diagonal"),
        _ => panic!("Expected pattern fill style"),
    }

    println!("✅ Line and fill styles test passed");
}
