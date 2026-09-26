use reactive_tui::component::{
    registry::{global_cleanup_all, register_component},
    Element,
};
use reactive_tui::render::tree::element_to_render_node;
use reactive_tui::widgets::display::table::{TableCell, TableColumn, TableRow};
use reactive_tui::widgets::display::{
    Alignment, DataTable, DataTableProps, DisplaySize, FilterType, PaginationConfig,
    VirtualScrollConfig,
};
use std::collections::HashMap;

fn create_sample_data() -> (Vec<TableColumn>, Vec<TableRow>) {
    // Create sample columns
    let columns = vec![
        TableColumn {
            title: "ID".to_string(),
            key: "id".to_string(),
            width: DisplaySize::Fixed(60),
            alignment: Alignment::Center,
            sortable: true,
            resizable: false,
            min_width: 50,
            max_width: Some(100),
        },
        TableColumn {
            title: "Name".to_string(),
            key: "name".to_string(),
            width: DisplaySize::Flex(2.0),
            alignment: Alignment::Start,
            sortable: true,
            resizable: true,
            min_width: 100,
            max_width: None,
        },
        TableColumn {
            title: "Age".to_string(),
            key: "age".to_string(),
            width: DisplaySize::Fixed(80),
            alignment: Alignment::Center,
            sortable: true,
            resizable: false,
            min_width: 60,
            max_width: Some(120),
        },
        TableColumn {
            title: "Department".to_string(),
            key: "department".to_string(),
            width: DisplaySize::Flex(1.5),
            alignment: Alignment::Start,
            sortable: true,
            resizable: true,
            min_width: 120,
            max_width: None,
        },
        TableColumn {
            title: "Salary".to_string(),
            key: "salary".to_string(),
            width: DisplaySize::Fixed(100),
            alignment: Alignment::End,
            sortable: true,
            resizable: false,
            min_width: 80,
            max_width: Some(150),
        },
    ];

    // Create sample rows
    let sample_data = vec![
        ("1", "Alice Johnson", "28", "Engineering", "75000"),
        ("2", "Bob Smith", "34", "Marketing", "65000"),
        ("3", "Carol Davis", "29", "Engineering", "78000"),
        ("4", "David Wilson", "42", "Sales", "55000"),
        ("5", "Eve Brown", "31", "Engineering", "82000"),
        ("6", "Frank Miller", "38", "Marketing", "68000"),
        ("7", "Grace Lee", "26", "Design", "62000"),
        ("8", "Henry Taylor", "45", "Sales", "58000"),
        ("9", "Ivy Chen", "33", "Engineering", "85000"),
        ("10", "Jack Anderson", "29", "Design", "64000"),
    ];

    let rows = sample_data
        .into_iter()
        .map(|(id, name, age, dept, salary)| {
            let mut cells = HashMap::new();
            cells.insert(
                "id".to_string(),
                TableCell {
                    content: id.to_string(),
                    style: None,
                    alignment: None,
                    clickable: false,
                    action: None,
                },
            );
            cells.insert(
                "name".to_string(),
                TableCell {
                    content: name.to_string(),
                    style: None,
                    alignment: None,
                    clickable: true,
                    action: Some("view_profile".to_string()),
                },
            );
            cells.insert(
                "age".to_string(),
                TableCell {
                    content: age.to_string(),
                    style: None,
                    alignment: None,
                    clickable: false,
                    action: None,
                },
            );
            cells.insert(
                "department".to_string(),
                TableCell {
                    content: dept.to_string(),
                    style: Some("text-blue-600".to_string()),
                    alignment: None,
                    clickable: false,
                    action: None,
                },
            );
            cells.insert(
                "salary".to_string(),
                TableCell {
                    content: format!("${}", salary),
                    style: Some("font-bold text-green-600".to_string()),
                    alignment: None,
                    clickable: false,
                    action: None,
                },
            );

            TableRow {
                id: id.to_string(),
                cells,
                selectable: true,
                style: None,
                data: HashMap::new(),
            }
        })
        .collect();

    (columns, rows)
}

#[test]
fn test_advanced_table_creation() {
    global_cleanup_all().unwrap();
    register_component::<DataTable>("DataTable").unwrap();

    println!("🧪 Testing Advanced Table Creation");

    let (columns, rows) = create_sample_data();

    // Create advanced table props
    let props = DataTableProps::new(columns, rows)
        .with_pagination(true, 5)
        .with_virtual_scroll(false, 32, 400)
        .with_features(true, true, true);

    assert!(props.pagination.enabled, "pagination is enabled");
    assert_eq!(props.pagination.page_size, 5, "page size is retained");
    assert!(props.show_pagination, "pagination controls are shown");

    // Create the element
    let element = Element::component_with_props("DataTable", props);
    let render_node = element_to_render_node(element);
    assert_eq!(
        render_node.node_type(),
        "DataTable",
        "render node keeps the component name"
    );
}

#[test]
fn test_pagination_config() {
    println!("🧪 Testing Pagination Configuration");

    let pagination = PaginationConfig {
        current_page: 0,
        page_size: 5,
        total_rows: 23,
        enabled: true,
    };

    assert_eq!(pagination.total_pages(), 5); // 23 rows / 5 per page = 5 pages
    assert_eq!(pagination.start_index(), 0);
    assert_eq!(pagination.end_index(), 5);
    assert!(pagination.has_next_page());
    assert!(!pagination.has_previous_page());

    let pagination_page_2 = PaginationConfig {
        current_page: 1,
        page_size: 5,
        total_rows: 23,
        enabled: true,
    };

    assert_eq!(pagination_page_2.start_index(), 5);
    assert_eq!(pagination_page_2.end_index(), 10);
    assert!(pagination_page_2.has_next_page());
    assert!(pagination_page_2.has_previous_page());

    let pagination_last_page = PaginationConfig {
        current_page: 4,
        page_size: 5,
        total_rows: 23,
        enabled: true,
    };

    assert_eq!(pagination_last_page.start_index(), 20);
    assert_eq!(pagination_last_page.end_index(), 23); // Clamped to total_rows
    assert!(!pagination_last_page.has_next_page());
    assert!(pagination_last_page.has_previous_page());

    println!("✅ Pagination configuration test passed");
}

#[test]
fn test_virtual_scroll_config() {
    println!("🧪 Testing Virtual Scroll Configuration");

    let virtual_scroll = VirtualScrollConfig {
        row_height: 32,
        overscan: 5,
        total_rows: 1000,
        scroll_offset: 50,
        viewport_height: 320, // 10 rows visible
        enabled: true,
    };

    let (start, end) = virtual_scroll.visible_range();

    // Should render from (50 - 5) to (50 + 10 + 5) = 45 to 65
    assert_eq!(start, 45);
    assert_eq!(end, 65);

    // Test edge cases
    let virtual_scroll_start = VirtualScrollConfig {
        row_height: 32,
        overscan: 5,
        total_rows: 1000,
        scroll_offset: 0,
        viewport_height: 320,
        enabled: true,
    };

    let (start, end) = virtual_scroll_start.visible_range();
    assert_eq!(start, 0); // Can't go below 0
    assert_eq!(end, 15); // 0 + 10 + 5

    println!("✅ Virtual scroll configuration test passed");
}

#[test]
fn test_filter_types() {
    println!("🧪 Testing Filter Types");

    let _cell = TableCell {
        content: "Engineering".to_string(),
        style: None,
        alignment: None,
        clickable: false,
        action: None,
    };

    // Test contains filter
    let contains_filter = FilterType::Contains("Eng".to_string());
    // Note: We can't test the actual filtering without the DataTable instance
    // but we can verify the filter types are created correctly

    match contains_filter {
        FilterType::Contains(text) => assert_eq!(text, "Eng"),
        _ => panic!("Expected Contains filter"),
    }

    // Test equals filter
    let equals_filter = FilterType::Equals("Engineering".to_string());
    match equals_filter {
        FilterType::Equals(text) => assert_eq!(text, "Engineering"),
        _ => panic!("Expected Equals filter"),
    }

    // Test range filter
    let range_filter = FilterType::Range(25.0, 35.0);
    match range_filter {
        FilterType::Range(min, max) => {
            assert_eq!(min, 25.0);
            assert_eq!(max, 35.0);
        }
        _ => panic!("Expected Range filter"),
    }

    println!("✅ Filter types test passed");
}

#[test]
fn test_advanced_table_features() {
    global_cleanup_all().unwrap();
    register_component::<DataTable>("DataTableFeatures").unwrap();

    println!("🧪 Testing Advanced Table Features");

    let (columns, rows) = create_sample_data();

    // Test with all features enabled
    let props_full = DataTableProps::new(columns.clone(), rows.clone())
        .with_pagination(true, 3)
        .with_virtual_scroll(true, 32, 200)
        .with_features(true, true, true);

    assert!(props_full.searchable);
    assert!(props_full.show_filters);
    assert!(props_full.exportable);
    assert!(props_full.pagination.enabled);
    assert_eq!(props_full.pagination.page_size, 3);
    assert!(props_full.virtual_scroll.enabled);

    // Test with minimal features
    let props_minimal = DataTableProps::new(columns, rows)
        .with_pagination(false, 10)
        .with_virtual_scroll(false, 32, 400)
        .with_features(false, false, false);

    assert!(!props_minimal.searchable);
    assert!(!props_minimal.show_filters);
    assert!(!props_minimal.exportable);
    assert!(!props_minimal.pagination.enabled);
    assert!(!props_minimal.virtual_scroll.enabled);

    println!("✅ Advanced table features test passed");
}
