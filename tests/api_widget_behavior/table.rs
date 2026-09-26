use super::{click, key, run, Control};
use reactive_tui::{
    component::Element,
    event::types::{Event, KeyCode, ResizeEvent},
    widgets::display::{
        table::{Table, TableColumn, TableProps, TableRow},
        DisplaySize,
    },
};
use std::sync::{Arc, Mutex};

fn props() -> TableProps {
    let mut name = TableColumn::new("Name", "name").with_width(DisplaySize::Flex(1.0));
    name.min_width = 5;
    let mut value = TableColumn::new("Value", "value").with_width(DisplaySize::Fixed(8));
    value.min_width = 4;
    let mut props = TableProps {
        columns: vec![name, value],
        rows: vec![
            TableRow::new("z")
                .with_cell("name", "Zed")
                .with_cell("value", "9"),
            TableRow::new("a")
                .with_cell("name", "Ada")
                .with_cell("value", "1"),
            TableRow::new("b")
                .with_cell("name", "Bea")
                .with_cell("value", "2"),
        ],
        sortable: true,
        ..Default::default()
    };
    props.border.enabled = false;
    props
}

#[test]
fn table_named_route_sorts_selects_and_activates_source_rows() {
    for size in [(24, 7), (48, 12)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.on_select = Some(Arc::new(move |row| {
            sink.lock().unwrap().push(format!("select:{row:?}"))
        }));
        let sink = calls.clone();
        config.on_sort = Some(Arc::new(move |column, ascending| {
            sink.lock()
                .unwrap()
                .push(format!("sort:{column}:{ascending}"))
        }));
        let sink = calls.clone();
        config.on_row_action = Some(Arc::new(move |row, action| {
            sink.lock().unwrap().push(format!("action:{row}:{action}"))
        }));
        let frames = run(
            Control(Table::with_props(config).auto_focus()),
            size,
            vec![
                (2, click(1, 0)),
                (3, key(KeyCode::Down)),
                (4, key(KeyCode::Enter)),
                (4, click(1, 3)),
                (5, None),
            ],
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("Zed") && frame.text.contains("Ada")));
        assert_eq!(
            *calls.lock().unwrap(),
            vec![
                "sort:0:true",
                "select:Some(1)",
                "action:1:select",
                "select:Some(0)"
            ],
            "{}",
            frames.last().unwrap().text
        );
        assert!(
            frames.last().unwrap().screen.cell(3, 1).unwrap().bgcolor() != vt100::Color::Default
        );
    }
}

#[test]
fn table_column_targets_follow_resize() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let sink = calls.clone();
    let mut config = props();
    config.on_sort = Some(Arc::new(move |column, _| sink.lock().unwrap().push(column)));
    let frames = run(
        Control(Element::typed::<Table>(config)),
        (24, 7),
        vec![
            (2, click(18, 0)),
            (3, Some(Event::Resize(ResizeEvent::new(48, 12)))),
            (5, click(18, 0)),
            (6, None),
        ],
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec![1, 0],
        "{}",
        frames.last().unwrap().text
    );
}

#[test]
fn table_scrolls_sorted_rows_and_skips_disabled_rows() {
    use reactive_tui::event::types::{
        MouseEvent, MouseEventKind, Position, WheelDelta, WheelEvent, WheelPhase,
    };
    for size in [(24, 7), (48, 12)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.rows = (0..12)
            .map(|i| TableRow::new(&i.to_string()).with_cell("name", &format!("Row{i:02}")))
            .collect();
        config.rows[1].selectable = false;
        config.max_height = Some(4);
        config.on_select = Some(Arc::new(move |row| sink.lock().unwrap().push(row)));
        let mut mouse = MouseEvent::new(MouseEventKind::Wheel, Position::cell(2, 2));
        mouse.wheel = Some(WheelEvent {
            delta: WheelDelta::Lines { x: 0.0, y: 3.0 },
            phase: WheelPhase::Changed,
        });
        let frames = run(
            Control(Table::with_props(config).auto_focus()),
            size,
            vec![
                (2, key(KeyCode::Down)),
                (3, key(KeyCode::Down)),
                (4, Some(Event::Mouse(mouse))),
                (5, click(1, 1)),
                (6, key(KeyCode::End)),
                (7, None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec![Some(0), Some(2), Some(3), Some(11)]
        );
        assert!(
            frames.last().unwrap().text.contains("Row11"),
            "{}",
            frames.last().unwrap().text
        );
        assert!(!frames.last().unwrap().text.contains("Row00"));
    }
}

#[test]
fn table_resizing_moves_the_column_boundary_and_stops_on_release() {
    use reactive_tui::event::types::{MouseButton, MouseEvent, MouseEventKind, Position};
    for size in [(24, 7), (48, 12)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.columns[0].width = DisplaySize::Fixed(10);
        config.resizable_columns = true;
        config.on_sort = Some(Arc::new(move |column, _| sink.lock().unwrap().push(column)));
        let mouse = |kind, x| {
            Some(Event::Mouse(
                MouseEvent::new(kind, Position::cell(x, 0)).with_button(MouseButton::Left),
            ))
        };
        let frames = run(
            Control(Table::with_props(config)),
            size,
            vec![
                (2, click(9, 0)),
                (2, mouse(MouseEventKind::Drag, 14)),
                (3, mouse(MouseEventKind::Up, 14)),
                (3, mouse(MouseEventKind::Move, 20)),
                (3, click(12, 0)),
                (4, None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec![0],
            "{}",
            frames.last().unwrap().text
        );
        assert!(frames
            .last()
            .unwrap()
            .screen
            .cell(0, 15)
            .unwrap()
            .contents()
            .contains('V'));
    }
}

#[test]
fn table_paints_configured_borders_and_handles_padded_cell_targets() {
    use reactive_tui::widgets::display::BorderStyle;
    for size in [(24, 7), (48, 12)] {
        for (style, corner) in [
            (BorderStyle::Single, '┌'),
            (BorderStyle::Double, '╔'),
            (BorderStyle::Rounded, '╭'),
            (BorderStyle::Thick, '┏'),
        ] {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let sink = calls.clone();
            let mut config = props();
            config.border.enabled = true;
            config.border.style = style;
            config.border.color = Some("red-500".into());
            config.on_select = Some(Arc::new(move |row| sink.lock().unwrap().push(row)));
            let frames = run(
                Control(Table::with_props(config)),
                size,
                vec![(2, click(2, 2)), (3, None)],
            );
            assert!(
                frames.last().unwrap().text.contains(corner),
                "{}",
                frames.last().unwrap().text
            );
            assert_eq!(*calls.lock().unwrap(), vec![Some(0)]);
            assert_eq!(
                frames.last().unwrap().screen.cell(0, 0).unwrap().fgcolor(),
                vt100::Color::Rgb(239, 68, 68)
            );
        }
    }
}

#[test]
fn table_clips_overflow_before_the_border_hit_targets() {
    for size in [(24, 7), (48, 12)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.border.enabled = true;
        config.columns[0].width = DisplaySize::Fixed(80);
        config.on_select = Some(Arc::new(move |row| sink.lock().unwrap().push(row)));
        let frames = run(
            Control(Table::with_props(config)),
            size,
            vec![
                (2, click(size.0 - 1, 2)),
                (2, click(0, 2)),
                (2, click(2, 2)),
                (3, None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec![Some(0)],
            "{}",
            frames.last().unwrap().text
        );
        assert_eq!(
            frames
                .last()
                .unwrap()
                .screen
                .cell(2, size.0 - 1)
                .unwrap()
                .contents(),
            "│"
        );
    }
}

#[test]
fn table_without_explicit_height_scrolls_within_the_terminal() {
    for size in [(24, 7), (48, 12)] {
        let mut config = props();
        config.rows = (0..40)
            .map(|i| TableRow::new(&i.to_string()).with_cell("name", &format!("Row{i:02}")))
            .collect();
        let frames = run(
            Control(Table::with_props(config).auto_focus()),
            size,
            vec![(2, key(KeyCode::End)), (3, None)],
        );
        assert!(
            frames.last().unwrap().text.contains("Row39"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn table_column_width_alignment_and_cell_styles_reach_the_frame() {
    use reactive_tui::widgets::display::{table::TableCell, Alignment};
    for size in [(24, 7), (48, 12)] {
        let mut auto = TableColumn::new("A", "a");
        auto.min_width = 1;
        auto.max_width = Some(8);
        auto.alignment = Alignment::End;
        let mut percent = TableColumn::new("B", "b").with_width(DisplaySize::Percent(50.0));
        percent.min_width = 1;
        percent.max_width = Some(10);
        percent.alignment = Alignment::Center;
        let mut flex = TableColumn::new("C", "c").with_width(DisplaySize::Flex(1.0));
        flex.min_width = 3;
        flex.alignment = Alignment::End;
        let mut first = TableRow::new("first")
            .with_cell("a", "界e\u{301}")
            .with_cell("b", "X")
            .with_cell("c", "Z");
        first.cells.insert(
            "b".into(),
            TableCell::new("X").with_style("text-yellow-500"),
        );
        let mut config = props();
        config.columns = vec![auto, percent, flex];
        config.rows = vec![
            first.clone(),
            TableRow {
                id: "second".into(),
                ..first
            },
        ];
        config.row_style = Some("text-red-500".into());
        config.zebra_striping = true;
        config.alternate_row_style = Some("bg-green-500".into());
        config.selected_style = Some("bg-blue-500".into());
        let frames = run(
            Control(Table::with_props(config)),
            size,
            vec![(2, click(9, 2)), (3, None)],
        );
        let screen = &frames.last().unwrap().screen;
        assert_eq!(screen.cell(1, 2).unwrap().contents(), "界");
        assert_eq!(screen.cell(1, 4).unwrap().contents(), "e\u{301}");
        assert_eq!(screen.cell(1, 9).unwrap().contents(), "X");
        assert_eq!(screen.cell(1, size.0 - 1).unwrap().contents(), "Z");
        assert_eq!(
            screen.cell(1, 9).unwrap().fgcolor(),
            vt100::Color::Rgb(234, 179, 8)
        );
        assert_eq!(
            screen.cell(1, 2).unwrap().fgcolor(),
            vt100::Color::Rgb(239, 68, 68)
        );
        assert_eq!(
            screen.cell(2, 9).unwrap().bgcolor(),
            vt100::Color::Rgb(59, 130, 246)
        );
        assert!(frames
            .iter()
            .any(|frame| frame.screen.cell(2, 9).unwrap().bgcolor()
                == vt100::Color::Rgb(34, 197, 94)));
    }
}

#[test]
fn table_rejects_ambiguous_ids_and_contradictory_widths_without_callbacks() {
    for size in [(24, 7), (48, 12)] {
        for issue in 0..5 {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let sink = calls.clone();
            let mut config = props();
            config.on_select = Some(Arc::new(move |row| sink.lock().unwrap().push(row)));
            let expected = match issue {
                0 => {
                    config.rows[1].id = config.rows[0].id.clone();
                    "Duplicate table row"
                }
                1 => {
                    config.columns[1].key = config.columns[0].key.clone();
                    "Duplicate table column"
                }
                2 => {
                    config.columns[0].max_width = Some(1);
                    "minimum"
                }
                3 => {
                    config.sort_column = Some(50);
                    "out of range"
                }
                _ => {
                    config.columns[0].width = DisplaySize::Flex(f32::NAN);
                    "finite"
                }
            };
            let frames = run(
                Control(Table::with_props(config).auto_focus()),
                size,
                vec![(2, key(KeyCode::Down)), (2, click(1, 1)), (2, None)],
            );
            assert!(
                frames
                    .last()
                    .unwrap()
                    .text
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .contains(expected),
                "{}",
                frames.last().unwrap().text
            );
            assert!(calls.lock().unwrap().is_empty());
        }
    }
}

#[test]
fn table_initial_sort_selection_and_hidden_header_follow_props() {
    for size in [(24, 7), (48, 12)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.show_header = false;
        config.sort_column = Some(0);
        config.sort_ascending = false;
        config.selected_row = Some(1);
        config.on_select = Some(Arc::new(move |row| sink.lock().unwrap().push(row)));
        let frames = run(
            Control(Table::with_props(config).auto_focus()),
            size,
            vec![(2, key(KeyCode::Up)), (3, None)],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec![Some(2)],
            "{}",
            frames.last().unwrap().text
        );
        assert!(!frames.last().unwrap().text.contains("Name"));
        assert_eq!(
            frames.last().unwrap().screen.cell(0, 0).unwrap().contents(),
            "Z"
        );
        assert!(frames
            .iter()
            .any(|frame| frame.screen.cell(2, 1).unwrap().bgcolor() != vt100::Color::Default));
    }
}

#[test]
fn table_multi_selection_cell_actions_and_inert_inputs() {
    use reactive_tui::event::types::{KeyEvent, KeyModifiers};
    for size in [(24, 7), (48, 12)] {
        let multi = Arc::new(Mutex::new(Vec::new()));
        let actions = Arc::new(Mutex::new(Vec::new()));
        let sink = multi.clone();
        let mut config = props();
        config.multi_select = true;
        config.rows[1].selectable = false;
        config.rows[2].cells.get_mut("value").unwrap().clickable = true;
        config.rows[2].cells.get_mut("value").unwrap().action = Some("open".into());
        config.on_multi_select = Some(Arc::new(move |rows| sink.lock().unwrap().push(rows)));
        let sink = actions.clone();
        config.on_row_action = Some(Arc::new(move |row, action| {
            sink.lock().unwrap().push((row, action.to_string()))
        }));
        let mut shift = KeyEvent::new(KeyCode::Down);
        shift.modifiers = KeyModifiers {
            shift: true,
            ..KeyModifiers::empty()
        };
        let frames = run(
            Control(Table::with_props(config.clone()).auto_focus()),
            size,
            vec![
                (2, key(KeyCode::Down)),
                (3, Some(Event::Key(shift))),
                (4, click(1, 2)),
                (4, click(size.0 - 2, 3)),
                (5, None),
            ],
        );
        assert_eq!(
            *multi.lock().unwrap(),
            vec![vec![0], vec![0, 2], vec![2]],
            "{}",
            frames.last().unwrap().text
        );
        assert_eq!(*actions.lock().unwrap(), vec![(2, "open".into())]);
        multi.lock().unwrap().clear();
        actions.lock().unwrap().clear();
        run(
            Control(
                Table::with_props(config.clone())
                    .disabled(true)
                    .auto_focus(),
            ),
            size,
            vec![
                (2, key(KeyCode::Down)),
                (2, click(size.0 - 2, 3)),
                (2, None),
            ],
        );
        assert!(multi.lock().unwrap().is_empty());
        assert!(actions.lock().unwrap().is_empty());
        config.rows.clear();
        let frames = run(
            Control(Table::with_props(config).auto_focus()),
            size,
            vec![(2, key(KeyCode::End)), (2, key(KeyCode::Enter)), (2, None)],
        );
        assert!(frames.last().unwrap().text.contains("Name"));
        assert!(multi.lock().unwrap().is_empty());
        assert!(actions.lock().unwrap().is_empty());
    }
}

#[test]
fn table_keeps_selected_row_identity_when_props_reorder() {
    use reactive_tui::{app::RootComponent, event::router::EventResult};
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Reorder {
        changed: AtomicBool,
        actions: Arc<Mutex<Vec<usize>>>,
    }
    impl RootComponent for Reorder {
        fn render(&self) -> Element {
            let mut config = props();
            if self.changed.load(Ordering::SeqCst) {
                config.rows.swap(0, 2);
            }
            let sink = self.actions.clone();
            config.on_row_action = Some(Arc::new(move |row, _| sink.lock().unwrap().push(row)));
            Table::with_props(config).auto_focus()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if matches!(event,Event::Key(key) if key.code == KeyCode::F(2)) {
                self.changed.store(true, Ordering::SeqCst);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }
    for size in [(24, 7), (48, 12)] {
        let actions = Arc::new(Mutex::new(Vec::new()));
        let frames = run(
            Reorder {
                changed: AtomicBool::new(false),
                actions: actions.clone(),
            },
            size,
            vec![
                (2, key(KeyCode::Down)),
                (3, key(KeyCode::F(2))),
                (4, key(KeyCode::Enter)),
                (4, None),
            ],
        );
        assert_eq!(
            *actions.lock().unwrap(),
            vec![2],
            "{}",
            frames.last().unwrap().text
        );
        assert!(
            frames.last().unwrap().screen.cell(3, 1).unwrap().bgcolor() != vt100::Color::Default
        );
    }
}

#[test]
fn table_contributes_intrinsic_width_inside_an_auto_sized_padded_parent() {
    for size in [(24, 12), (48, 16)] {
        let frames = run(
            Control(
                Element::layout(reactive_tui::component::LayoutType::Flex)
                    .with_class("flex flex-col p-1")
                    .with_child(Table::with_props(props())),
            ),
            size,
            vec![(2, None)],
        );
        assert!(
            frames.last().unwrap().text.contains("Name"),
            "{}",
            frames.last().unwrap().text
        );
        assert!(frames.last().unwrap().text.contains("Zed"));
    }
}
