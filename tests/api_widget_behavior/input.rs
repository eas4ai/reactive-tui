use super::{ctrl, key, run, Control};
use crate::app_input::click;
use reactive_tui::{
    component::{Component, Element},
    event::types::{Event, KeyCode, PasteEvent, ResizeEvent},
    widgets::{input::InputMode, TextInput, TextInputProps},
};
use std::sync::{Arc, Mutex};

fn input(props: TextInputProps) -> (Element, Arc<Mutex<Vec<String>>>) {
    let changes = Arc::new(Mutex::new(Vec::new()));
    let callback = changes.clone();
    let element = Element::typed_with::<TextInput>(props, move |props| {
        let callback = callback.clone();
        TextInput::new(props).with_on_change(move |value| callback.lock().unwrap().push(value))
    })
    .auto_focus();
    (element, changes)
}

#[test]
fn autocomplete_accepts_with_tab_and_mouse_without_moving_focus() {
    use reactive_tui::widgets::input::Suggestion;
    for size in [(24, 8), (48, 12)] {
        for mouse in [false, true] {
            let changes = Arc::new(Mutex::new(Vec::new()));
            let callback = changes.clone();
            let element =
                Element::typed_with::<TextInput>(TextInputProps::default(), move |props| {
                    let callback = callback.clone();
                    TextInput::new(props)
                        .with_on_change(move |value| callback.lock().unwrap().push(value))
                        .with_suggestions(|_, _| {
                            vec![
                                Suggestion {
                                    text: "alpha".into(),
                                    insert_text: "alpha".into(),
                                    description: None,
                                },
                                Suggestion {
                                    text: "atom".into(),
                                    insert_text: "atom".into(),
                                    description: None,
                                },
                            ]
                        })
                })
                .auto_focus();
            let mut steps = vec![(1, key(KeyCode::Char('a')))];
            if mouse {
                steps.push((2, click(3, 2)));
            } else {
                steps.push((2, key(KeyCode::Down)));
                steps.push((2, key(KeyCode::Tab)));
            }
            steps.push((3, None));
            let frames = run(Control(element), size, steps);
            assert_eq!(*changes.lock().unwrap(), vec!["a", "atom"]);
            assert!(frames.last().unwrap().text.contains("atom"));
            assert!(!frames.last().unwrap().text.contains("alpha"));
        }
    }
}

#[test]
fn supplied_text_replacement_resets_undo_without_corrupting_value() {
    use std::sync::atomic::{AtomicBool, Ordering};
    struct ReplaceText(AtomicBool);
    impl reactive_tui::app::RootComponent for ReplaceText {
        fn render(&self) -> Element {
            Element::typed::<TextInput>(TextInputProps {
                value: if self.0.load(Ordering::SeqCst) {
                    "ab"
                } else {
                    ""
                }
                .into(),
                ..Default::default()
            })
            .auto_focus()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn handle_event(&self, event: &Event) -> reactive_tui::event::router::EventResult {
            if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
                self.0.store(true, Ordering::SeqCst);
                reactive_tui::event::router::EventResult::Consumed
            } else {
                reactive_tui::event::router::EventResult::Ignored
            }
        }
    }
    for size in [(24, 8), (48, 12)] {
        let frames = run(
            ReplaceText(AtomicBool::new(false)),
            size,
            vec![
                (1, key(KeyCode::Char('a'))),
                (1, ctrl('a')),
                (1, key(KeyCode::F(2))),
                (2, ctrl('z')),
                (2, key(KeyCode::End)),
                (2, key(KeyCode::Char('X'))),
                (3, None),
            ],
        );
        assert!(
            frames.last().unwrap().text.contains("abX"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn readonly_input_allows_tab_to_the_next_control() {
    for size in [(24, 8), (48, 12)] {
        let calls = Arc::new(Mutex::new(0));
        let callback = calls.clone();
        let control = reactive_tui::builder::div()
            .class("flex flex-col")
            .child(
                reactive_tui::builder::text_input()
                    .value("fixed")
                    .readonly(true)
                    .build()
                    .auto_focus(),
            )
            .child(
                reactive_tui::builder::button()
                    .text("Apply")
                    .on_click(move || *callback.lock().unwrap() += 1)
                    .build(),
            )
            .build();
        run(
            Control(control),
            size,
            vec![(1, key(KeyCode::Tab)), (1, key(KeyCode::Enter)), (2, None)],
        );
        assert_eq!(*calls.lock().unwrap(), 1);
    }
}

#[test]
fn input_builders_and_macros_preserve_editable_controls() {
    use reactive_tui::widgets::input::TextInputBuilder;
    for size in [(24, 8), (48, 12)] {
        let controls = vec![
            TextInputBuilder::new().value("seed").render(),
            Element::typed::<TextInput>(TextInputBuilder::new().value("seed").build()),
            reactive_tui::text_input![value: "seed"],
            reactive_tui::builder::text_input().value("seed").into(),
        ];
        for control in controls {
            let frames = run(
                Control(control.auto_focus()),
                size,
                vec![
                    (1, key(KeyCode::End)),
                    (1, key(KeyCode::Char('X'))),
                    (2, None),
                ],
            );
            assert!(frames.last().unwrap().text.contains("seedX"));
        }
    }
}

#[test]
fn numeric_and_email_builder_modes_affect_input_and_validation() {
    for size in [(24, 8), (48, 12)] {
        let numeric = reactive_tui::builder::text_input()
            .input_type("number")
            .build()
            .auto_focus();
        let frames = run(
            Control(numeric),
            size,
            vec![
                (1, key(KeyCode::Char('1'))),
                (1, key(KeyCode::Char('x'))),
                (1, key(KeyCode::Char('.'))),
                (1, key(KeyCode::Char('2'))),
                (2, None),
            ],
        );
        assert!(frames.last().unwrap().text.contains("1.2"));
        assert!(!frames.last().unwrap().text.contains('x'));
        let email = reactive_tui::builder::text_input()
            .input_type("email")
            .value("bad")
            .build()
            .auto_focus();
        let frames = run(
            Control(email),
            size,
            vec![
                (1, ctrl('a')),
                (1, Some(Event::Paste(PasteEvent::new("a@b.test".into())))),
                (2, None),
            ],
        );
        assert!(frames[0].text.contains('❌'));
        assert!(!frames.last().unwrap().text.contains('❌'));
        assert!(frames.last().unwrap().text.contains("a@b.test"));
    }
}

#[test]
fn multiline_builder_renders_line_numbers_tabs_and_auto_indent() {
    use reactive_tui::widgets::input::TextInputBuilder;
    for size in [(24, 8), (48, 12)] {
        let (element, changes) = input(
            TextInputBuilder::new()
                .value("  a")
                .multi_line(3)
                .show_line_numbers(true)
                .tab_size(2)
                .auto_indent(true)
                .build(),
        );
        let frames = run(
            Control(element),
            size,
            vec![
                (1, key(KeyCode::End)),
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Tab)),
                (1, key(KeyCode::Char('b'))),
                (2, None),
            ],
        );
        assert_eq!(changes.lock().unwrap().last().unwrap(), "  a\n    b");
        let screen = &frames.last().unwrap().text;
        assert!(screen.contains("1 [  a"), "{screen}");
        assert!(screen.contains("2 [    b"), "{screen}");
    }
}

#[test]
fn input_selection_copy_cut_paste_redo_and_submit_reach_app() {
    for size in [(24, 8), (48, 12)] {
        let submitted = Arc::new(Mutex::new(Vec::new()));
        let callback = submitted.clone();
        let control = Element::typed_with::<TextInput>(
            TextInputProps {
                value: "界🙂".into(),
                ..Default::default()
            },
            move |props| {
                let callback = callback.clone();
                TextInput::new(props)
                    .with_on_submit(move |value| callback.lock().unwrap().push(value))
            },
        )
        .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, ctrl('a')),
                (1, ctrl('c')),
                (1, ctrl('x')),
                (1, ctrl('v')),
                (1, ctrl('z')),
                (1, ctrl('y')),
                (1, key(KeyCode::Enter)),
                (2, None),
            ],
        );
        assert_eq!(*submitted.lock().unwrap(), vec!["界🙂"]);
        assert!(frames.last().unwrap().text.contains("界🙂"));
    }
}

#[test]
fn unhandled_default_quit_keys_exit_before_further_typing() {
    for quit in [key(KeyCode::Escape), ctrl('c')] {
        let frames = run(
            Control(
                reactive_tui::builder::text_input()
                    .value("seed")
                    .build()
                    .auto_focus(),
            ),
            (24, 8),
            vec![(1, quit), (1, key(KeyCode::Char('X'))), (2, None)],
        );
        assert!(!frames.last().unwrap().text.contains('X'));
        assert!(frames.last().unwrap().text.contains("seed"));
    }
}

#[test]
fn static_suggestions_are_available_after_typing() {
    use reactive_tui::widgets::input::Suggestion;
    for size in [(24, 8), (48, 12)] {
        let (element, changes) = input(TextInputProps {
            suggestions: vec![Suggestion {
                text: "alpha".into(),
                insert_text: "alpha".into(),
                description: None,
            }],
            ..Default::default()
        });
        let frames = run(
            Control(element),
            size,
            vec![
                (1, key(KeyCode::Char('a'))),
                (2, key(KeyCode::Tab)),
                (3, None),
            ],
        );
        assert_eq!(*changes.lock().unwrap(), vec!["a", "alpha"]);
        assert!(frames.last().unwrap().text.contains("alpha"));
    }
}

#[test]
fn password_masks_graphemes_without_revealing_original_text() {
    for size in [(12, 6), (48, 12)] {
        let (element, _) = input(TextInputProps {
            value: "界🙂e\u{301}".into(),
            mode: InputMode::Password,
            ..Default::default()
        });
        let frames = run(
            Control(element),
            size,
            vec![(1, key(KeyCode::End)), (2, None)],
        );
        for frame in &frames {
            assert!(frame.text.contains("***"), "{}", frame.text);
            assert!(!frame.text.contains("****"));
            assert!(!frame.text.contains('界') && !frame.text.contains('🙂'));
        }
    }
}

#[test]
fn max_length_counts_graphemes_and_accepts_combining_input() {
    for size in [(12, 6), (48, 12)] {
        let (element, changes) = input(TextInputProps {
            max_length: Some(2),
            ..Default::default()
        });
        run(
            Control(element),
            size,
            vec![
                (1, key(KeyCode::Char('界'))),
                (1, key(KeyCode::Char('e'))),
                (1, key(KeyCode::Char('\u{301}'))),
                (1, key(KeyCode::Char('X'))),
                (2, None),
            ],
        );
        assert_eq!(*changes.lock().unwrap(), vec!["界", "界e", "界e\u{301}"]);
    }
}

#[test]
fn bracketed_paste_and_undo_preserve_multiline_trailing_newlines() {
    for size in [(24, 8), (48, 12)] {
        let (element, changes) = input(TextInputProps {
            value: "one\n".into(),
            mode: InputMode::MultiLine { height: 4 },
            ..Default::default()
        });
        let frames = run(
            Control(element),
            size,
            vec![
                (
                    1,
                    Some(Event::Key(
                        reactive_tui::event::types::KeyEvent::new(KeyCode::End)
                            .with_modifiers(reactive_tui::event::types::KeyModifiers::ctrl()),
                    )),
                ),
                (1, key(KeyCode::Char('界'))),
                (1, Some(Event::Paste(PasteEvent::new("\n🙂\n".into())))),
                (1, ctrl('a')),
                (1, key(KeyCode::Delete)),
                (1, ctrl('z')),
                (2, None),
            ],
        );
        assert_eq!(
            *changes.lock().unwrap(),
            vec!["one\n界", "one\n界\n🙂\n", "", "one\n界\n🙂\n"]
        );
        let last = frames.last().unwrap();
        assert!(
            last.text.contains("one") && last.text.contains('界') && last.text.contains('🙂'),
            "{}",
            last.text
        );
    }
}

#[test]
fn narrow_multiline_fields_wrap_wide_graphemes_into_visible_rows() {
    let (element, _) = input(TextInputProps {
        value: "界界界界界".into(),
        mode: InputMode::MultiLine { height: 3 },
        width: Some(6),
        wrap_text: true,
        ..Default::default()
    });
    let frames = run(
        Control(element.class("w-8")),
        (24, 8),
        vec![(1, key(KeyCode::End)), (2, None)],
    );
    let screen = &frames.last().unwrap().screen;
    for (row, column) in [(0, 3), (0, 5), (1, 3), (1, 5), (2, 3)] {
        assert_eq!(
            screen.cell(row, column).unwrap().contents(),
            "界",
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn text_input_mouse_uses_display_columns_before_and_after_resize() {
    let (element, changes) = input(TextInputProps {
        value: "界🙂".into(),
        ..Default::default()
    });
    run(
        Control(element.class("w-12")),
        (24, 6),
        vec![
            (1, click(5, 0)),
            (1, key(KeyCode::Char('X'))),
            (2, Some(Event::Resize(ResizeEvent::new(16, 8)))),
            (3, click(3, 0)),
            (3, key(KeyCode::Char('Y'))),
            (4, None),
        ],
    );
    assert_eq!(*changes.lock().unwrap(), vec!["界X🙂", "Y界X🙂"]);
}

#[test]
fn regex_validation_changes_visible_error_and_recovers() {
    let (element, _) = input(TextInputProps {
        value: "AB".into(),
        validator_pattern: Some("^[A-Z]{2}$".into()),
        error_message: Some("bad value".into()),
        ..Default::default()
    });
    let frames = run(
        Control(element),
        (24, 6),
        vec![
            (1, key(KeyCode::End)),
            (1, key(KeyCode::Char('c'))),
            (2, key(KeyCode::Backspace)),
            (3, None),
        ],
    );
    assert!(frames[1].text.contains("bad value"), "{}", frames[1].text);
    assert!(!frames.last().unwrap().text.contains("bad value"));
    let (invalid, _) = input(TextInputProps {
        validator_pattern: Some("[".into()),
        error_message: Some("bad pattern".into()),
        ..Default::default()
    });
    assert!(run(Control(invalid), (24, 6), vec![])[0]
        .text
        .contains("bad pattern"));
}

#[test]
fn empty_and_zero_width_selects_handle_navigation_without_panics() {
    use reactive_tui::widgets::{Select, SelectProps};
    for size in [(12, 6), (48, 12)] {
        let control = Element::typed::<Select<u8>>(SelectProps {
            width: Some(0),
            max_visible_items: 0,
            ..Default::default()
        })
        .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::End)),
                (1, key(KeyCode::PageDown)),
                (1, key(KeyCode::Escape)),
                (2, None),
            ],
        );
        assert!(
            frames.len() >= 2,
            "empty select keeps painting after navigation keys at {size:?}: {} frames",
            frames.len()
        );
    }
}

#[test]
fn typed_select_skips_disabled_first_option_and_reports_selection_once() {
    use reactive_tui::widgets::{Select, SelectOption, SelectProps};
    for size in [(12, 6), (48, 12)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let callback = changes.clone();
        let control = Element::typed_with::<Select<u8>>(
            SelectProps {
                options: vec![
                    SelectOption::new(0, "blocked").disabled(true),
                    SelectOption::new(1, "界🙂"),
                ],
                width: Some(2),
                ..Default::default()
            },
            move |props| {
                let callback = callback.clone();
                Select::new(props).with_on_change(move |value| callback.lock().unwrap().push(value))
            },
        )
        .auto_focus();
        run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Enter)),
                (2, None),
            ],
        );
        assert_eq!(*changes.lock().unwrap(), vec![1]);
    }
}

#[test]
fn select_mouse_respects_padding_and_open_close_callbacks_after_resize() {
    use reactive_tui::widgets::{Select, SelectOption, SelectProps};
    for size in [(24, 8), (48, 12)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let callback = calls.clone();
        let control = Element::typed_with::<Select<u8>>(
            SelectProps {
                options: vec![SelectOption::new(0, "Alpha"), SelectOption::new(1, "Beta")],
                ..Default::default()
            },
            move |props| {
                let opened = callback.clone();
                let closed = callback.clone();
                let changed = callback.clone();
                Select::new(props)
                    .with_on_open(move || opened.lock().unwrap().push("open"))
                    .with_on_change(move |value| {
                        assert_eq!(value, 1);
                        changed.lock().unwrap().push("change");
                    })
                    .with_on_close(move || closed.lock().unwrap().push("close"))
            },
        )
        .class("p-0.5 w-16");
        let frames = run(
            Control(control),
            size,
            vec![
                (1, click(4, 2)),
                (2, Some(Event::Resize(ResizeEvent::new(32, 10)))),
                (3, click(5, 4)),
                (4, None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec!["open", "change", "close"],
            "{:?}",
            frames
                .iter()
                .map(|f| (&f.text, &f.geometry))
                .collect::<Vec<_>>()
        );
        assert!(frames.last().unwrap().text.contains("Beta"));
        assert!(!frames.last().unwrap().text.contains("Alpha"));
    }
}

#[test]
fn select_end_skips_disabled_last_option_and_reopen_keeps_it_visible() {
    use reactive_tui::widgets::{Select, SelectOption, SelectProps};
    for size in [(24, 8), (48, 12)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let callback = changes.clone();
        let control = Element::typed_with::<Select<u8>>(
            SelectProps {
                options: vec![
                    SelectOption::new(0, "Alpha"),
                    SelectOption::new(1, "Beta"),
                    SelectOption::new(2, "blocked").disabled(true),
                ],
                max_visible_items: 1,
                ..Default::default()
            },
            move |props| {
                let callback = callback.clone();
                Select::new(props).with_on_change(move |value| callback.lock().unwrap().push(value))
            },
        )
        .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::End)),
                (1, key(KeyCode::Enter)),
                (2, key(KeyCode::Enter)),
                (3, None),
            ],
        );
        assert_eq!(*changes.lock().unwrap(), vec![1]);
        assert!(!frames.last().unwrap().text.contains("Alpha"));
        assert!(frames.last().unwrap().text.contains("Beta"));
    }
}
