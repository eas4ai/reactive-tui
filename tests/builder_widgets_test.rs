use reactive_tui::component::registry::{register_component, global_cleanup_all};
use reactive_tui::builder::{data_table, chart};
use reactive_tui::widgets::display::{
    DataTable, Chart, ChartType, DataSeries, DataPoint, FilterType
};
use reactive_tui::{data_table, chart};

#[test]
fn test_data_table_builder() {
    global_cleanup_all().unwrap();
    register_component::<DataTable>("DataTableBuilder").unwrap();
    
    println!("🧪 Testing DataTable Builder");
    
    // Test basic data table builder
    let table = data_table()
        .column("Name", "name")
        .column("Age", "age")
        .column("City", "city")
        .simple_row(vec![("name", "Alice"), ("age", "25"), ("city", "New York")])
        .simple_row(vec![("name", "Bob"), ("age", "30"), ("city", "San Francisco")])
        .simple_row(vec![("name", "Carol"), ("age", "28"), ("city", "Chicago")])
        .pagination(true, 10)
        .features(true, true, true)
        .class("custom-table")
        .build_with_name("DataTableBuilder");
    
    // Verify the element was created
    assert!(matches!(table.element_type, reactive_tui::component::ElementType::Component(_)));
    assert_eq!(table.class, Some("custom-table".to_string()));
    
    println!("✅ DataTable builder test passed");
}

#[test]
fn test_data_table_macro() {
    global_cleanup_all().unwrap();
    register_component::<DataTable>("DataTableMacro").unwrap();
    
    println!("🧪 Testing DataTable Macro");
    
    // Test data table macro
    let table = data_table![
        columns: [
            "Product" => "product",
            "Sales" => "sales",
            "Revenue" => "revenue"
        ],
        rows: [
            vec![("product", "Widget A"), ("sales", "100"), ("revenue", "$1000")],
            vec![("product", "Widget B"), ("sales", "150"), ("revenue", "$1500")],
            vec![("product", "Widget C"), ("sales", "80"), ("revenue", "$800")]
        ]
    ];
    
    // Verify the element was created
    assert!(matches!(table.element_type, reactive_tui::component::ElementType::Component(_)));
    
    // Test with pagination
    let paginated_table = data_table![
        columns: [
            "Name" => "name",
            "Score" => "score"
        ],
        rows: [
            vec![("name", "Alice"), ("score", "95")],
            vec![("name", "Bob"), ("score", "87")],
            vec![("name", "Carol"), ("score", "92")]
        ],
        pagination: 5
    ];
    
    assert!(matches!(paginated_table.element_type, reactive_tui::component::ElementType::Component(_)));
    
    println!("✅ DataTable macro test passed");
}

#[test]
fn test_chart_builder() {
    global_cleanup_all().unwrap();
    register_component::<Chart>("ChartBuilder").unwrap();
    
    println!("🧪 Testing Chart Builder");
    
    // Test basic chart builder
    let bar_chart = chart()
        .bar_chart()
        .title("Sales Performance")
        .size(80, 20)
        .simple_series("Q1 Sales", vec![100.0, 150.0, 120.0, 180.0])
        .simple_series("Q2 Sales", vec![110.0, 160.0, 130.0, 190.0])
        .colors(vec!["#3b82f6".to_string(), "#ef4444".to_string()])
        .animated(2000)
        .class("performance-chart")
        .build();

    // Verify the element was created
    assert!(matches!(bar_chart.element_type, reactive_tui::component::ElementType::Component(_)));
    assert_eq!(bar_chart.class, Some("performance-chart".to_string()));

    // Test different chart types
    let line_chart = chart()
        .line_chart()
        .title("Trend Analysis")
        .labeled_series("Revenue", vec![(100.0, "Jan"), (120.0, "Feb"), (110.0, "Mar")])
        .build();

    assert!(matches!(line_chart.element_type, reactive_tui::component::ElementType::Component(_)));

    let pie_chart = chart()
        .pie_chart()
        .title("Market Share")
        .simple_series("Companies", vec![35.0, 25.0, 20.0, 15.0, 5.0])
        .build();

    assert!(matches!(pie_chart.element_type, reactive_tui::component::ElementType::Component(_)));
    
    println!("✅ Chart builder test passed");
}

#[test]
fn test_chart_macro() {
    global_cleanup_all().unwrap();
    register_component::<Chart>("ChartMacro").unwrap();
    
    println!("🧪 Testing Chart Macro");
    
    // Test simple bar chart macro
    let bar_chart = chart![bar: "Sales" => [100.0, 150.0, 120.0, 180.0]];
    assert!(matches!(bar_chart.element_type, reactive_tui::component::ElementType::Component(_)));
    
    // Test line chart macro
    let line_chart = chart![line: "Revenue" => [1000.0, 1200.0, 1100.0, 1400.0]];
    assert!(matches!(line_chart.element_type, reactive_tui::component::ElementType::Component(_)));
    
    // Test pie chart macro
    let pie_chart = chart![pie: "Market Share" => [35.0, 25.0, 20.0, 15.0, 5.0]];
    assert!(matches!(pie_chart.element_type, reactive_tui::component::ElementType::Component(_)));
    
    // Test with title
    let titled_chart = chart![bar: "Q1 Performance" => [80.0, 95.0, 110.0], title: "Quarterly Results"];
    assert!(matches!(titled_chart.element_type, reactive_tui::component::ElementType::Component(_)));
    
    println!("✅ Chart macro test passed");
}

#[test]
fn test_advanced_data_table_features() {
    global_cleanup_all().unwrap();
    register_component::<DataTable>("AdvancedDataTable").unwrap();
    
    println!("🧪 Testing Advanced DataTable Features");
    
    // Test with virtual scrolling and filters
    let advanced_table = data_table()
        .column("ID", "id")
        .column("Name", "name")
        .column("Department", "department")
        .column("Salary", "salary")
        .simple_row(vec![("id", "1"), ("name", "Alice"), ("department", "Engineering"), ("salary", "75000")])
        .simple_row(vec![("id", "2"), ("name", "Bob"), ("department", "Marketing"), ("salary", "65000")])
        .simple_row(vec![("id", "3"), ("name", "Carol"), ("department", "Engineering"), ("salary", "78000")])
        .virtual_scroll(true, 32, 400)
        .filter("department", FilterType::Contains("Engineering".to_string()))
        .hide_columns(vec!["id".to_string()])
        .features(true, true, true)
        .build();
    
    assert!(matches!(advanced_table.element_type, reactive_tui::component::ElementType::Component(_)));
    
    println!("✅ Advanced DataTable features test passed");
}

#[test]
fn test_advanced_chart_features() {
    global_cleanup_all().unwrap();
    register_component::<Chart>("AdvancedChart").unwrap();
    
    println!("🧪 Testing Advanced Chart Features");
    
    // Test with custom data series
    let sales_data = DataSeries::new("Sales", vec![
        DataPoint::with_label(100.0, "Q1"),
        DataPoint::with_label(150.0, "Q2"),
        DataPoint::with_label(120.0, "Q3"),
        DataPoint::with_label(180.0, "Q4"),
    ]);
    
    let revenue_data = DataSeries::new("Revenue", vec![
        DataPoint::with_label(1000.0, "Q1"),
        DataPoint::with_label(1500.0, "Q2"),
        DataPoint::with_label(1200.0, "Q3"),
        DataPoint::with_label(1800.0, "Q4"),
    ]);
    
    let advanced_chart = chart()
        .chart_type(ChartType::Line)
        .title("Performance Dashboard")
        .size(100, 25)
        .series(sales_data)
        .series(revenue_data)
        .axes(Some("Quarter".to_string()), Some("Amount".to_string()))
        .colors(vec![
            "#3b82f6".to_string(),
            "#ef4444".to_string(),
            "#10b981".to_string(),
        ])
        .animated(1500)
        .build();
    
    assert!(matches!(advanced_chart.element_type, reactive_tui::component::ElementType::Component(_)));
    
    println!("✅ Advanced Chart features test passed");
}

#[test]
fn test_builder_integration() {
    global_cleanup_all().unwrap();
    register_component::<DataTable>("IntegrationDataTable").unwrap();
    register_component::<Chart>("IntegrationChart").unwrap();
    
    println!("🧪 Testing Builder Integration");
    
    // Test that builders integrate well with the existing builder system
    use reactive_tui::builder::{div, h1, p};
    
    let dashboard = div()
        .class("dashboard p-4")
        .children(vec![
            h1().text("Analytics Dashboard").build(),
            p().text("Performance metrics and data visualization").build(),
            
            // Add a data table
            data_table()
                .column("Metric", "metric")
                .column("Value", "value")
                .simple_row(vec![("metric", "Users"), ("value", "1,234")])
                .simple_row(vec![("metric", "Revenue"), ("value", "$12,345")])
                .class("metrics-table")
                .build(),
            
            // Add a chart
            chart()
                .line_chart()
                .title("Monthly Trends")
                .simple_series("Growth", vec![10.0, 15.0, 12.0, 18.0, 22.0])
                .size(60, 15)
                .class("trends-chart")
                .build(),
        ])
        .build();
    
    assert!(matches!(dashboard.element_type, reactive_tui::component::ElementType::Layout(_)));
    assert_eq!(dashboard.children.len(), 4); // h1, p, table, chart
    
    println!("✅ Builder integration test passed");
}
