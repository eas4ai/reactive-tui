use super::{app_input, Control};
use reactive_tui::{
    core::geometry::Rect,
    event::types::{Event, KeyCode, ResizeEvent},
    widgets::dialog::{
        AutocompleteConfig, AutocompleteDialog, AutocompleteDialogOptions, DialogComponent,
        DialogId, DialogResult, DialogTheme,
    },
};
use std::sync::{Arc, Mutex};

fn dialog(options: AutocompleteDialogOptions) -> reactive_tui::component::Element {
    AutocompleteDialog::new(DialogId::from_u32(31), options)
        .render(Rect::default(), &DialogTheme::default())
}

#[test]
fn autocomplete_highlight_and_region_styles_reach_rendered_cells() {
    use std::collections::HashMap;
    use unicode_width::UnicodeWidthStr;
    for size in [(32, 12), (60, 20)] {
        for highlight in [false, true] {
            let frames = app_input::run_when(
                Control(dialog(AutocompleteDialogOptions {
                    title: "STYLED".into(),
                    prompt: "QUERY".into(),
                    css_classes: HashMap::from([
                        ("dialog".into(), "bg-#123456 reduced-motion".into()),
                        ("content".into(), "bg-#345678".into()),
                        ("input".into(), "w-full bg-#102030".into()),
                    ]),
                    autocomplete: AutocompleteConfig {
                        default_value: Some("ph".into()),
                        static_suggestions: vec!["Alpha".into()],
                        highlight_matches: highlight,
                        ..Default::default()
                    },
                    ..Default::default()
                })),
                size,
                vec![("Alpha", None)],
            );
            let frame = frames.last().unwrap();
            let cell_at = |text: &str, offset: usize| {
                let (row, column) = frame
                    .text
                    .lines()
                    .enumerate()
                    .find_map(|(row, line)| {
                        line.find(text)
                            .map(|column| (row, line[..column].width() + offset))
                    })
                    .unwrap_or_else(|| panic!("Missing {text}: {}", frame.text));
                frame.screen.cell(row as u16, column as u16).unwrap()
            };
            for offset in 0..5 {
                let cell = cell_at("Alpha", offset);
                let matched = highlight && (2..4).contains(&offset);
                assert_eq!(cell.bold(), matched, "offset {offset}: {}", frame.text);
                assert_eq!(cell.underline(), matched, "offset {offset}: {}", frame.text);
            }
            assert_eq!(
                cell_at("QUERY", 0).bgcolor(),
                vt100::Color::Rgb(52, 86, 120)
            );
            assert_eq!(cell_at("[ph", 1).bgcolor(), vt100::Color::Rgb(16, 32, 48));
        }
    }
}

#[test]
fn autocomplete_minimum_counts_graphemes_and_shows_the_empty_placeholder() {
    use reactive_tui::event::types::PasteEvent;
    for size in [(32, 12), (60, 20)] {
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let frames = app_input::run_visibility(
            Control(dialog(AutocompleteDialogOptions {
                title: "MINIMUM".into(),
                prompt: "QUERY".into(),
                css_classes: [("dialog".into(), "reduced-motion".into())].into(),
                autocomplete: AutocompleteConfig {
                    placeholder: Some("START HERE".into()),
                    min_chars: 2,
                    static_suggestions: vec!["e\u{301}Xray".into()],
                    ..Default::default()
                },
                on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
                ..Default::default()
            })),
            size,
            vec![
                (
                    "START HERE",
                    Some("e\u{301}Xray"),
                    Some(Event::Paste(PasteEvent::new("e\u{301}".into()))),
                ),
                (
                    "[e\u{301}",
                    Some("e\u{301}Xray"),
                    app_input::key(KeyCode::Char('X')),
                ),
                (
                    "e\u{301}Xray",
                    Some("START HERE"),
                    app_input::key(KeyCode::Enter),
                ),
            ],
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("e\u{301}Xray")));
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Selected(value)] if value == "e\u{301}Xray")
        );
    }
}

#[test]
fn autocomplete_filters_navigates_and_delivers_selection_once() {
    for size in [(32, 12), (60, 20)] {
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let changes = Arc::new(Mutex::new(Vec::new()));
        let changed = changes.clone();
        let element = dialog(AutocompleteDialogOptions {
            title: "AUTOCOMPLETE".into(),
            prompt: "QUERY".into(),
            autocomplete: AutocompleteConfig {
                static_suggestions: vec!["Alpha".into(), "Alpine".into(), "Beta".into()],
                ..Default::default()
            },
            on_change: Some(Arc::new(move |value| {
                changed.lock().unwrap().push(value.to_string())
            })),
            on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
            ..Default::default()
        });
        app_input::run_until_hidden(
            Control(element),
            size,
            vec![
                ("QUERY", app_input::key(KeyCode::Char('a'))),
                ("Alpha", app_input::key(KeyCode::Char('l'))),
                ("Alpine", app_input::key(KeyCode::Down)),
                ("Alpine", app_input::key(KeyCode::Enter)),
            ],
            "AUTOCOMPLETE",
        );
        assert_eq!(*changes.lock().unwrap(), ["a", "al"]);
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Selected(value)] if value == "Alpine")
        );
    }
}

#[test]
fn autocomplete_click_uses_suggestion_bounds_after_resize_and_honors_veto() {
    use app_input::Action;
    for (initial, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let selections = Arc::new(Mutex::new(Vec::new()));
        let selected = selections.clone();
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let element = dialog(AutocompleteDialogOptions {
            title: "POINTER".into(),
            prompt: "QUERY".into(),
            autocomplete: AutocompleteConfig {
                default_value: Some("a".into()),
                static_suggestions: vec!["Alpha".into(), "Beta".into()],
                ..Default::default()
            },
            on_select: Some(Arc::new(move |value| {
                selected.lock().unwrap().push(value.to_string());
                value == "Beta"
            })),
            on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
            ..Default::default()
        });
        let frames = app_input::run_actions_until_hidden(
            Control(element),
            initial,
            vec![
                (
                    "Alpha",
                    Action::Event(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                ),
                ("Alpha", Action::ClickText("Alpha", 1)),
                ("Beta", Action::ClickText("Beta", 1)),
            ],
            "POINTER",
        );
        assert_eq!(frames.last().unwrap().screen.size(), (resized.1, resized.0));
        assert_eq!(*selections.lock().unwrap(), ["Alpha", "Beta"]);
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Selected(value)] if value == "Beta")
        );
    }
}

#[test]
fn autocomplete_unicode_empty_results_submit_typed_value() {
    for size in [(32, 12), (60, 20)] {
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let element = dialog(AutocompleteDialogOptions {
            title: "UNICODE".into(),
            prompt: "QUERY".into(),
            autocomplete: AutocompleteConfig {
                default_value: Some("Ae\u{301}界👩‍💻Z".into()),
                ..Default::default()
            },
            on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
            ..Default::default()
        });
        app_input::run_until_hidden(
            Control(element),
            size,
            vec![
                ("No suggestions", app_input::key(KeyCode::End)),
                ("No suggestions", app_input::key(KeyCode::Left)),
                ("No suggestions", app_input::key(KeyCode::Backspace)),
                ("Ae\u{301}界Z", app_input::key(KeyCode::Enter)),
            ],
            "UNICODE",
        );
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Confirmed(Some(value))] if value == "Ae\u{301}界Z")
        );
    }
}

#[test]
fn autocomplete_escape_dismisses_suggestions_before_canceling() {
    for size in [(32, 12), (60, 20)] {
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let element = dialog(AutocompleteDialogOptions {
            title: "ESCAPE".into(),
            prompt: "QUERY".into(),
            autocomplete: AutocompleteConfig {
                default_value: Some("a".into()),
                static_suggestions: vec!["Alpha".into()],
                ..Default::default()
            },
            on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
            ..Default::default()
        });
        let frames = app_input::run_until_hidden(
            Control(element),
            size,
            vec![
                ("Alpha", app_input::key(KeyCode::Escape)),
                ("QUERY", app_input::key(KeyCode::Escape)),
            ],
            "ESCAPE",
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("QUERY") && !frame.text.contains("Alpha")));
        assert!(matches!(
            results.lock().unwrap().as_slice(),
            [DialogResult::Cancelled]
        ));
    }
}

#[test]
fn autocomplete_custom_filter_renderer_and_limit_are_applied() {
    for size in [(32, 12), (60, 20)] {
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let element = dialog(AutocompleteDialogOptions {
            title: "CUSTOM".into(),
            prompt: "QUERY".into(),
            autocomplete: AutocompleteConfig {
                default_value: Some("seed".into()),
                static_suggestions: vec!["original".into()],
                max_suggestions: 1,
                filter_function: Some(Arc::new(|query, values| {
                    assert_eq!(query, "seed");
                    assert_eq!(values, ["original"]);
                    vec!["first".into(), "second".into()]
                })),
                suggestion_renderer: Some(Arc::new(|suggestion| {
                    reactive_tui::component::Element::text(format!("Rendered {}", suggestion.value))
                })),
                ..Default::default()
            },
            on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
            ..Default::default()
        });
        let frames = app_input::run_until_hidden(
            Control(element),
            size,
            vec![
                ("Rendered first", app_input::key(KeyCode::Down)),
                ("Rendered first", app_input::key(KeyCode::Tab)),
            ],
            "CUSTOM",
        );
        assert!(frames.iter().all(|frame| !frame.text.contains("second")));
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Selected(value)] if value == "first")
        );
    }
}

#[test]
fn autocomplete_reveals_the_keyboard_selection_in_a_long_list() {
    for size in [(32, 12), (60, 20)] {
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let element = dialog(AutocompleteDialogOptions {
            title: "LONG LIST".into(),
            prompt: "QUERY".into(),
            autocomplete: AutocompleteConfig {
                default_value: Some("Item".into()),
                static_suggestions: (0..10).map(|index| format!("Item{index}")).collect(),
                ..Default::default()
            },
            on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
            ..Default::default()
        });
        app_input::run_until_hidden(
            Control(element),
            size,
            vec![
                ("Item0", app_input::key(KeyCode::Up)),
                ("Item9", app_input::key(KeyCode::Enter)),
            ],
            "LONG LIST",
        );
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Selected(value)] if value == "Item9")
        );
    }
}

#[test]
fn autocomplete_builder_wheel_and_click_use_current_rows() {
    use app_input::Action;
    for size in [(32, 12), (60, 20)] {
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let mut options = reactive_tui::widgets::dialog::DialogBuilder::autocomplete(
            "WHEEL SEARCH",
            "QUERY",
            (0..10).map(|index| format!("Item{index}")).collect(),
        );
        options.autocomplete.default_value = Some("Item".into());
        options.on_close = Some(Arc::new(move |result| closed.lock().unwrap().push(result)));
        app_input::run_actions_until_hidden(
            Control(dialog(options)),
            size,
            vec![
                ("Item0", Action::WheelText("Item0", 9.0)),
                ("Item9", Action::WheelText("Item9", -9.0)),
                ("Item0", Action::ClickText("Item0", 2)),
            ],
            "WHEEL SEARCH",
        );
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Selected(value)] if value == "Item0")
        );
    }
}

#[test]
fn autocomplete_prop_updates_keep_edits_and_selection_with_fresh_callbacks() {
    use reactive_tui::{app::RootComponent, component::Element, event::router::EventResult};
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Updating {
        phase: AtomicUsize,
        selections: Arc<Mutex<Vec<(usize, String)>>>,
        results: Arc<Mutex<Vec<(usize, DialogResult)>>>,
    }
    impl RootComponent for Updating {
        fn render(&self) -> Element {
            let phase = self.phase.load(Ordering::SeqCst);
            let selected = self.selections.clone();
            let closed = self.results.clone();
            dialog(AutocompleteDialogOptions {
                title: "UPDATING SEARCH".into(),
                prompt: ["BEFORE", "AFTER", "REPLACED"][phase].into(),
                autocomplete: AutocompleteConfig {
                    default_value: Some(if phase == 2 { "z" } else { "a" }.into()),
                    static_suggestions: if phase == 2 {
                        vec!["Zulu".into()]
                    } else {
                        vec!["Alpha".into(), "Alpine".into()]
                    },
                    filter_function: Some(Arc::new(|_, values| values.to_vec())),
                    ..Default::default()
                },
                on_select: Some(Arc::new(move |value| {
                    selected.lock().unwrap().push((phase, value.to_string()));
                    true
                })),
                on_close: Some(Arc::new(move |result| {
                    closed.lock().unwrap().push((phase, result))
                })),
                ..Default::default()
            })
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
                self.phase.store(1, Ordering::SeqCst);
                EventResult::Consumed
            } else if matches!(event, Event::Key(key) if key.code == KeyCode::F(3)) {
                self.phase.store(2, Ordering::SeqCst);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }
    for (size, reset) in [
        ((32, 12), false),
        ((60, 20), false),
        ((32, 12), true),
        ((60, 20), true),
    ] {
        let selections = Arc::new(Mutex::new(Vec::new()));
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut steps = vec![
            ("BEFORE", app_input::key(KeyCode::End)),
            ("BEFORE", app_input::key(KeyCode::Char('l'))),
            ("[al", app_input::key(KeyCode::Down)),
            ("Alpine", app_input::key(KeyCode::F(2))),
        ];
        if reset {
            steps.extend([
                ("AFTER", app_input::key(KeyCode::F(3))),
                ("[z", app_input::key(KeyCode::Enter)),
            ]);
        } else {
            steps.push(("AFTER", app_input::key(KeyCode::Enter)));
        }
        app_input::run_until_hidden(
            Updating {
                phase: AtomicUsize::new(0),
                selections: selections.clone(),
                results: results.clone(),
            },
            size,
            steps,
            "UPDATING SEARCH",
        );
        let (phase, expected) = if reset { (2, "Zulu") } else { (1, "Alpine") };
        assert_eq!(*selections.lock().unwrap(), [(phase, expected.into())]);
        assert!(
            matches!(results.lock().unwrap().as_slice(), [(actual_phase, DialogResult::Selected(value))] if *actual_phase == phase && value == expected)
        );
    }
}
