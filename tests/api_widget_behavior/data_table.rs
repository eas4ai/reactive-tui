use super::{click, key, run, Control};
use reactive_tui::{
    builder,
    component::Element,
    event::types::KeyCode,
    widgets::display::{
        table::{TableColumn, TableRow},
        DataTable, DataTableProps, DisplaySize, FilterType,
    },
};
use std::sync::{Arc, Mutex};

fn config() -> DataTableProps {
    let mut column = TableColumn::new("Name", "name").with_width(DisplaySize::Flex(1.0));
    column.min_width = 4;
    let rows = ["Zed", "Ada", "Bea", "Cam"]
        .iter()
        .enumerate()
        .map(|(i, name)| TableRow::new(&i.to_string()).with_cell("name", name))
        .collect();
    let mut props = DataTableProps::new(vec![column], rows)
        .with_features(false, false, false)
        .with_pagination(true, 2);
    props.column_visibility_control = false;
    props.table_props.border.enabled = false;
    props.table_props.sortable = true;
    props
}

#[test]
fn data_table_pages_sort_before_slicing_and_report_source_indices() {
    for size in [(24, 8), (48, 14)] {
        let selected = Arc::new(Mutex::new(Vec::new()));
        let sink = selected.clone();
        let pages = Arc::new(Mutex::new(Vec::new()));
        let mut props = config();
        props.pagination.total_rows = 0; // The view derives the count from current data.
        props.table_props.on_select = Some(Arc::new(move |row| sink.lock().unwrap().push(row)));
        let sink = pages.clone();
        props.on_page_change = Some(Arc::new(move |page| sink.lock().unwrap().push(page)));
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![
                (2, click(1, 0)),
                (3, click(1, 1)),
                (4, click(6, 3)),
                (5, click(1, 1)),
                (6, None),
            ],
        );
        assert_eq!(
            *selected.lock().unwrap(),
            vec![Some(1), Some(3)],
            "{}",
            frames.last().unwrap().text
        );
        assert_eq!(*pages.lock().unwrap(), vec![1]);
        assert!(frames.last().unwrap().text.contains("Cam"));
        assert!(frames.last().unwrap().text.contains("Zed"));
    }
}

#[test]
fn data_table_builder_preserves_filters_and_hidden_columns() {
    for size in [(24, 8), (48, 14)] {
        let mut name = TableColumn::new("Name", "name").with_width(DisplaySize::Flex(1.0));
        name.min_width = 4;
        let mut secret = TableColumn::new("Secret", "secret");
        secret.min_width = 4;
        let table = builder::data_table()
            .columns(vec![name, secret])
            .simple_row(vec![("name", "Ada"), ("secret", "hidden")])
            .simple_row(vec![("name", "Zed"), ("secret", "hidden")])
            .filter("name", FilterType::Equals("Ada".into()))
            .hide_columns(vec!["secret".into()])
            .features(false, false, false)
            .pagination(false, 2)
            .build();
        let frames = run(Control(table), size, vec![(2, None)]);
        let text = &frames.last().unwrap().text;
        assert!(text.contains("Ada"), "{text}");
        assert!(
            !text.contains("Zed") && !text.contains("Secret") && !text.contains("hidden"),
            "{text}"
        );
    }
}

#[test]
fn data_table_search_edits_filter_rows_and_calls_back_once_per_change() {
    for size in [(24, 8), (48, 14)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let sink = changes.clone();
        let mut props = config();
        props.searchable = true;
        props.on_search_change = Some(Arc::new(move |value| sink.lock().unwrap().push(value)));
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![
                (2, click(3, 0)),
                (2, key(KeyCode::Char('A'))),
                (3, key(KeyCode::Char('d'))),
                (4, None),
            ],
        );
        assert_eq!(
            *changes.lock().unwrap(),
            vec!["A", "Ad"],
            "{}",
            frames.last().unwrap().text
        );
        let text = &frames.last().unwrap().text;
        assert!(
            text.contains("Ada") && !text.contains("Zed") && !text.contains("Cam"),
            "{text}"
        );
    }
}

#[test]
fn data_table_export_buttons_and_column_panel_are_live() {
    for size in [(24, 10), (48, 16)] {
        let exports = Arc::new(Mutex::new(Vec::new()));
        let sink = exports.clone();
        let mut props = config();
        props.exportable = true;
        props.on_export = Some(Arc::new(move |format| {
            sink.lock().unwrap().push(format.to_string())
        }));
        run(
            Control(Element::typed::<DataTable>(props.clone())),
            size,
            vec![(2, click(1, 0)), (2, click(5, 0)), (2, None)],
        );
        assert_eq!(*exports.lock().unwrap(), vec!["csv", "json"]);

        let changes = Arc::new(Mutex::new(Vec::new()));
        let sink = changes.clone();
        props.exportable = false;
        props.column_visibility_control = true;
        let mut column = TableColumn::new("Value", "value").with_width(DisplaySize::Fixed(6));
        column.min_width = 2;
        props.table_props.columns.push(column);
        for row in &mut props.table_props.rows {
            row.cells.insert(
                "value".into(),
                reactive_tui::widgets::display::table::TableCell::new("9"),
            );
        }
        props.on_column_visibility_change =
            Some(Arc::new(move |hidden| sink.lock().unwrap().push(hidden)));
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![
                (2, click(1, 0)),
                (3, click(2, 2)),
                (4, click(1, 0)),
                (5, None),
            ],
        );
        assert_eq!(
            *changes.lock().unwrap(),
            vec![vec!["value".to_string()]],
            "{}",
            frames.last().unwrap().text
        );
        assert!(!frames.last().unwrap().text.contains("Value"));
        assert!(frames.last().unwrap().text.contains("Name"));
    }
}

#[test]
fn data_table_applies_each_filter_type_to_actual_rows() {
    for size in [(24, 10), (48, 16)] {
        for (key, filter, expected) in [
            ("name", FilterType::Contains("aD".into()), vec!["Ada"]),
            ("name", FilterType::Equals("BEA".into()), vec!["Bea"]),
            ("age", FilterType::Range(20.0, 30.0), vec!["Ada", "Cam"]),
            (
                "date",
                FilterType::DateRange("2024-02-01".into(), "2024-03-01".into()),
                vec!["Ada", "Cam"],
            ),
            ("active", FilterType::Boolean(false), vec!["Bea"]),
        ] {
            let mut props = config().with_pagination(false, 2);
            props.table_props.rows = vec![
                TableRow::new("a")
                    .with_cell("name", "Ada")
                    .with_cell("age", "20")
                    .with_cell("date", "2024-02-29")
                    .with_cell("active", "yes"),
                TableRow::new("b")
                    .with_cell("name", "Bea")
                    .with_cell("age", "31")
                    .with_cell("date", "2024-02-30")
                    .with_cell("active", "no"),
                TableRow::new("c")
                    .with_cell("name", "Cam")
                    .with_cell("age", "30")
                    .with_cell("date", "2024-03-01")
                    .with_cell("active", "unknown"),
            ];
            props
                .filters
                .push(reactive_tui::widgets::display::ColumnFilter {
                    column_key: key.into(),
                    filter_type: filter,
                    active: true,
                });
            let frames = run(
                Control(Element::typed::<DataTable>(props)),
                size,
                vec![(2, None)],
            );
            let text = &frames.last().unwrap().text;
            for name in ["Ada", "Bea", "Cam"] {
                assert_eq!(text.contains(name), expected.contains(&name), "{text}");
            }
        }
    }
}

#[test]
fn data_table_filter_panel_edits_emit_changes_and_survive_closing() {
    for size in [(24, 10), (48, 16)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let sink = changes.clone();
        let mut props = config();
        props.show_filters = true;
        props.on_filter_change = Some(Arc::new(move |filters| sink.lock().unwrap().push(filters)));
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![
                (2, click(1, 0)),
                (3, click(2, 1)),
                (4, click(3, 2)),
                (4, key(KeyCode::Char('A'))),
                (5, key(KeyCode::Char('d'))),
                (6, key(KeyCode::Char('a'))),
                (7, click(1, 0)),
                (8, None),
            ],
        );
        let changes = changes.lock().unwrap();
        assert_eq!(
            changes.last().unwrap()[0].filter_type,
            FilterType::Equals("Ada".into())
        );
        let text = &frames.last().unwrap().text;
        assert!(
            text.contains("Ada") && !text.contains("Zed") && !text.contains("Bea"),
            "{text}"
        );
    }
}

#[test]
fn data_table_shift_sort_adds_secondary_column_before_paging() {
    use reactive_tui::event::types::{
        Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind, Position,
    };
    for size in [(24, 10), (48, 16)] {
        let mut props = config();
        props.table_props.columns[0].width = DisplaySize::Fixed(10);
        let mut column = TableColumn::new("Group", "group").with_width(DisplaySize::Fixed(8));
        column.min_width = 2;
        props.table_props.columns.push(column);
        props.table_props.rows = vec![
            TableRow::new("z")
                .with_cell("name", "Zed")
                .with_cell("group", "A"),
            TableRow::new("a")
                .with_cell("name", "Ada")
                .with_cell("group", "B"),
            TableRow::new("b")
                .with_cell("name", "Bea")
                .with_cell("group", "A"),
        ];
        let mut shift = MouseEvent::new(MouseEventKind::Down, Position::cell(1, 0))
            .with_button(MouseButton::Left);
        shift.modifiers = KeyModifiers {
            shift: true,
            ..KeyModifiers::empty()
        };
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![(2, click(12, 0)), (3, Some(Event::Mouse(shift))), (4, None)],
        );
        assert_eq!(
            frames.last().unwrap().screen.cell(1, 0).unwrap().contents(),
            "B",
            "{}",
            frames.last().unwrap().text
        );
        assert_eq!(
            frames.last().unwrap().screen.cell(2, 0).unwrap().contents(),
            "Z"
        );
        assert!(!frames.last().unwrap().text.contains("Ada"));
    }
}

#[test]
fn data_table_virtual_scroll_uses_measured_rows_and_preserves_header() {
    use reactive_tui::event::types::{
        Event, MouseEvent, MouseEventKind, Position, WheelDelta, WheelEvent, WheelPhase,
    };
    for size in [(24, 8), (48, 14)] {
        let mut props = config()
            .with_pagination(false, 2)
            .with_virtual_scroll(true, 32, 96);
        props.table_props.border.enabled = true;
        props.table_props.border.style = reactive_tui::widgets::display::BorderStyle::None;
        props.table_props.rows = (0..100)
            .map(|i| TableRow::new(&i.to_string()).with_cell("name", &format!("Row{i:02}")))
            .collect();
        props.virtual_scroll.total_rows = 100;
        props.virtual_scroll.scroll_offset = 10;
        props.virtual_scroll.overscan = 2;
        let mut wheel = MouseEvent::new(MouseEventKind::Wheel, Position::cell(2, 2));
        wheel.wheel = Some(WheelEvent {
            delta: WheelDelta::Lines { x: 0.0, y: 3.0 },
            phase: WheelPhase::Changed,
        });
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![(2, Some(Event::Mouse(wheel))), (3, None)],
        );
        let text = &frames.last().unwrap().text;
        assert!(
            text.contains("Name") && text.contains("Row13") && text.contains("Row15"),
            "{text}"
        );
        assert!(!text.contains("Row12") && !text.contains("Row16"), "{text}");
        assert!(
            frames.last().unwrap().geometry.len() < 40,
            "virtual scroll expanded {} nodes",
            frames.last().unwrap().geometry.len()
        );
    }
}

#[test]
fn data_table_query_and_page_changes_reset_the_visible_row_window() {
    use reactive_tui::event::types::{
        Event, MouseEvent, MouseEventKind, Position, WheelDelta, WheelEvent, WheelPhase,
    };
    for size in [(24, 10), (48, 16)] {
        let mut props = config()
            .with_pagination(false, 5)
            .with_virtual_scroll(true, 32, 96);
        props.searchable = true;
        props.table_props.rows = (0..30)
            .map(|i| TableRow::new(&i.to_string()).with_cell("name", &format!("Row{i:02}")))
            .collect();
        props.virtual_scroll.scroll_offset = 10;
        let frames = run(
            Control(Element::typed::<DataTable>(props.clone())),
            size,
            vec![(2, click(3, 0)), (3, key(KeyCode::Char('R'))), (4, None)],
        );
        assert!(
            frames.last().unwrap().text.contains("Row00"),
            "{}",
            frames.last().unwrap().text
        );
        props.searchable = false;
        props.pagination.enabled = true;
        props.show_pagination = true;
        props.virtual_scroll.scroll_offset = 0;
        let mut wheel = MouseEvent::new(MouseEventKind::Wheel, Position::cell(2, 2));
        wheel.wheel = Some(WheelEvent {
            delta: WheelDelta::Lines { x: 0.0, y: 3.0 },
            phase: WheelPhase::Changed,
        });
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![(2, Some(Event::Mouse(wheel))), (3, click(6, 4)), (4, None)],
        );
        assert!(
            frames.last().unwrap().text.contains("Row05"),
            "{}",
            frames.last().unwrap().text
        );
        assert!(!frames.last().unwrap().text.contains("Row08"));
    }
}

#[test]
fn data_table_invalid_filter_edits_report_errors_without_replacing_valid_filter() {
    use reactive_tui::event::types::{Event, KeyEvent, KeyModifiers};
    for size in [(24, 10), (48, 16)] {
        let mut props = config();
        props.show_filters = true;
        props.table_props.rows = vec![
            TableRow::new("a").with_cell("name", "20"),
            TableRow::new("b").with_cell("name", "40"),
        ];
        props
            .filters
            .push(reactive_tui::widgets::display::ColumnFilter {
                column_key: "name".into(),
                filter_type: FilterType::Range(0.0, 100.0),
                active: true,
            });
        let mut all = KeyEvent::new(KeyCode::Char('a'));
        all.modifiers = KeyModifiers {
            ctrl: true,
            ..KeyModifiers::empty()
        };
        let mut steps = vec![
            (2, click(1, 0)),
            (3, click(3, 2)),
            (4, Some(Event::Key(all.clone()))),
        ];
        steps.extend(
            "7..2"
                .chars()
                .enumerate()
                .map(|(i, c)| (i + 5, key(KeyCode::Char(c)))),
        );
        steps.push((9, None));
        let frames = run(
            Control(Element::typed::<DataTable>(props.clone())),
            size,
            steps,
        );
        let text = &frames.last().unwrap().text;
        assert!(text.contains("Range needs"), "{text}");
        assert!(text.contains("20") && text.contains("40"), "{text}");
        let mut steps = vec![
            (2, click(1, 0)),
            (3, click(3, 2)),
            (4, Some(Event::Key(all))),
        ];
        steps.extend(
            "20..30"
                .chars()
                .enumerate()
                .map(|(i, c)| (i + 5, key(KeyCode::Char(c)))),
        );
        steps.extend([(11, click(1, 0)), (12, None)]);
        let frames = run(Control(Element::typed::<DataTable>(props)), size, steps);
        let text = &frames.last().unwrap().text;
        assert!(text.contains("20") && !text.contains("40"), "{text}");
    }
}

#[test]
fn data_table_multi_selection_keeps_cursor_and_selections_across_pages() {
    use reactive_tui::event::types::{Event, KeyEvent, KeyModifiers};
    for size in [(24, 10), (48, 16)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let sink = changes.clone();
        let actions = Arc::new(Mutex::new(Vec::new()));
        let mut props = config().with_pagination(false, 2);
        props.table_props.multi_select = true;
        props.table_props.on_multi_select =
            Some(Arc::new(move |rows| sink.lock().unwrap().push(rows)));
        let sink = actions.clone();
        props.table_props.on_row_action =
            Some(Arc::new(move |row, _| sink.lock().unwrap().push(row)));
        let mut shift = KeyEvent::new(KeyCode::Down);
        shift.modifiers = KeyModifiers {
            shift: true,
            ..KeyModifiers::empty()
        };
        let mut all = KeyEvent::new(KeyCode::Char('a'));
        all.modifiers = KeyModifiers {
            ctrl: true,
            ..KeyModifiers::empty()
        };
        run(
            Control(Element::typed::<DataTable>(props.clone())),
            size,
            vec![
                (2, click(1, 1)),
                (3, Some(Event::Key(shift.clone()))),
                (4, Some(Event::Key(shift))),
                (5, Some(Event::Key(all))),
                (6, key(KeyCode::Up)),
                (7, key(KeyCode::Enter)),
                (7, None),
            ],
        );
        assert_eq!(
            *changes.lock().unwrap(),
            vec![
                vec![0],
                vec![0, 1],
                vec![0, 1, 2],
                vec![0, 1, 2, 3],
                vec![1]
            ]
        );
        assert_eq!(*actions.lock().unwrap(), vec![1]);
        changes.lock().unwrap().clear();
        props.pagination.enabled = true;
        props.show_pagination = true;
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![
                (2, click(1, 1)),
                (3, click(6, 3)),
                (4, click(1, 1)),
                (5, click(1, 3)),
                (6, None),
            ],
        );
        assert_eq!(
            *changes.lock().unwrap(),
            vec![vec![0], vec![0, 2]],
            "{}",
            frames.last().unwrap().text
        );
        assert_ne!(
            frames.last().unwrap().screen.cell(1, 1).unwrap().bgcolor(),
            vt100::Color::Default
        );
    }
}

#[test]
fn data_table_reports_zero_sizes_and_the_macro_renders_real_data() {
    for size in [(24, 10), (48, 16)] {
        for virtual_scroll in [false, true] {
            let mut props = config();
            let expected = if virtual_scroll {
                props.virtual_scroll.enabled = true;
                props.virtual_scroll.row_height = 0;
                "Virtual row height"
            } else {
                props.pagination.page_size = 0;
                "Table page size"
            };
            let frames = run(
                Control(Element::typed::<DataTable>(props)),
                size,
                vec![(2, None)],
            );
            assert!(
                frames.last().unwrap().text.contains(expected),
                "{}",
                frames.last().unwrap().text
            );
        }
        let frames = run(
            Control(
                reactive_tui::data_table![columns:["Name"=>"name"],rows:[vec![("name","Actual row")]],pagination:1],
            ),
            size,
            vec![(2, None)],
        );
        assert!(
            frames.last().unwrap().text.contains("Actual row"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn data_table_replaces_filters_from_new_props_and_keeps_disabled_controls_inert() {
    use reactive_tui::{
        app::RootComponent,
        event::{router::EventResult, types::Event},
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Changed(AtomicUsize);
    impl RootComponent for Changed {
        fn render(&self) -> Element {
            let mut props = config();
            let phase = self.0.load(Ordering::SeqCst);
            if phase > 0 {
                props
                    .filters
                    .push(reactive_tui::widgets::display::ColumnFilter {
                        column_key: "name".into(),
                        filter_type: FilterType::Equals(
                            if phase == 1 { "Ada" } else { "Cam" }.into(),
                        ),
                        active: true,
                    });
            }
            Element::typed::<DataTable>(props)
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            match event {
                Event::Key(key) if key.code == KeyCode::F(2) => {
                    self.0.store(1, Ordering::SeqCst);
                    EventResult::Consumed
                }
                Event::Key(key) if key.code == KeyCode::F(3) => {
                    self.0.store(2, Ordering::SeqCst);
                    EventResult::Consumed
                }
                _ => EventResult::Ignored,
            }
        }
    }
    for size in [(24, 10), (48, 16)] {
        let frames = run(
            Changed(AtomicUsize::new(0)),
            size,
            vec![(2, key(KeyCode::F(2))), (3, key(KeyCode::F(3))), (4, None)],
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("Ada") && !frame.text.contains("Zed")));
        assert!(
            frames.last().unwrap().text.contains("Cam")
                && !frames.last().unwrap().text.contains("Ada"),
            "{}",
            frames.last().unwrap().text
        );
        let changes = Arc::new(Mutex::new(Vec::new()));
        let sink = changes.clone();
        let mut props = config();
        props.searchable = true;
        props.on_search_change = Some(Arc::new(move |value| sink.lock().unwrap().push(value)));
        run(
            Control(Element::typed::<DataTable>(props).disabled(true)),
            size,
            vec![(2, click(3, 0)), (2, key(KeyCode::Char('X'))), (2, None)],
        );
        assert!(changes.lock().unwrap().is_empty());
        let pages = Arc::new(Mutex::new(Vec::new()));
        let sink = pages.clone();
        let mut props = config();
        props.table_props.rows.clear();
        props.on_page_change = Some(Arc::new(move |page| sink.lock().unwrap().push(page)));
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![(2, click(6, 1)), (2, key(KeyCode::End)), (2, None)],
        );
        assert!(pages.lock().unwrap().is_empty());
        assert!(frames.last().unwrap().text.contains("(0)"));
    }
}

#[test]
fn data_table_filter_active_toggle_supports_mouse_and_keyboard() {
    for size in [(24, 10), (48, 16)] {
        let active = Arc::new(Mutex::new(Vec::new()));
        let sink = active.clone();
        let mut props = config();
        props.show_filters = true;
        props
            .filters
            .push(reactive_tui::widgets::display::ColumnFilter {
                column_key: "name".into(),
                filter_type: FilterType::Equals("Ada".into()),
                active: false,
            });
        props.on_filter_change = Some(Arc::new(move |filters| {
            sink.lock().unwrap().push(filters[0].active)
        }));
        let frames = run(
            Control(Element::typed::<DataTable>(props)),
            size,
            vec![
                (2, click(1, 0)),
                (3, click(14, 1)),
                (4, key(KeyCode::Enter)),
                (5, click(1, 0)),
                (6, None),
            ],
        );
        assert_eq!(
            *active.lock().unwrap(),
            vec![true, false],
            "{}",
            frames.last().unwrap().text
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("Ada") && !frame.text.contains("Zed")));
        assert!(frames.last().unwrap().text.contains("Zed"));
    }
}
