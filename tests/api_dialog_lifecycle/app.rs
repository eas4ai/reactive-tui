use super::*;
use crate::app_input::{self, key, Action};
use reactive_tui::event::types::{Event, KeyCode, ResizeEvent};
use reactive_tui::{app::RootComponent, builder, component::Element, event::router::EventResult};

#[test]
fn engine_input_paints_edits_and_delivers_the_resized_pointer_result() {
    for (initial, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let mut engine = DialogEngine::new();
        engine.enable_async();
        let id = engine.show_input(InputDialogOptions {
            title: "ENGINE INPUT".into(),
            input: InputFieldConfig {
                default_value: Some("abcd".into()),
                ..Default::default()
            },
            ..Default::default()
        });
        let completion = engine.completion(id).unwrap();
        let frames = app_input::run_actions_until_hidden(
            engine.clone(),
            initial,
            vec![
                (
                    "abcd",
                    Action::Event(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                ),
                ("abcd", Action::ClickText("abcd", 2)),
                ("abcd", Action::Event(key(KeyCode::Char('X')).unwrap())),
                ("abXcd", Action::ClickText("OK", 0)),
            ],
            "ENGINE INPUT",
        );
        assert!(!frames.last().unwrap().text.contains("ENGINE INPUT"));
        assert!(
            matches!(completion.try_result(), Some(DialogResult::Confirmed(Some(value))) if value == "abXcd")
        );
        assert_eq!(engine.active_count(), 0);
        assert!(matches!(engine.take_event(), Some(DialogEvent::Opened(open)) if open == id));
        assert!(
            matches!(engine.take_event(), Some(DialogEvent::Closed(closed, DialogResult::Confirmed(Some(value)))) if closed == id && value == "abXcd")
        );
        assert!(engine.take_event().is_none());
    }
}

#[test]
fn engine_confirmation_callback_and_close_event_fire_once() {
    for size in [(32, 12), (60, 20)] {
        let mut engine = DialogEngine::new();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let observed = calls.clone();
        let id = engine.show_confirmation(ConfirmationDialogOptions {
            title: "CONFIRM ENGINE".into(),
            on_close: Some(Arc::new(move |result| {
                observed.lock().unwrap().push(result)
            })),
            ..Default::default()
        });
        app_input::run_actions_until_hidden(
            engine.clone(),
            size,
            vec![("CONFIRM ENGINE", Action::ClickText("OK", 0))],
            "CONFIRM ENGINE",
        );
        assert!(matches!(
            calls.lock().unwrap().as_slice(),
            [DialogResult::Confirmed(None)]
        ));
        let events: Vec<_> = std::iter::from_fn(|| engine.take_event()).collect();
        assert!(
            matches!(events.as_slice(), [DialogEvent::Opened(open), DialogEvent::Closed(closed, DialogResult::Confirmed(None))] if *open == id && *closed == id)
        );
    }
}

#[test]
fn engine_toast_expiration_delivers_completion_without_input() {
    struct SlowSibling;
    impl reactive_tui::component::Component for SlowSibling {
        type Props = reactive_tui::component::props::CommonProps;
        type State = ();
        fn new(_: Self::Props) -> Self {
            // Force the first frame to take longer than the toast lifetime.
            std::thread::sleep(std::time::Duration::from_millis(100));
            Self
        }
        fn render(&self, _: &Self::Props, _: &()) -> Element {
            Element::empty()
        }
    }
    struct Idle(DialogEngine);
    impl reactive_tui::app::RootComponent for Idle {
        fn render(&self) -> reactive_tui::component::Element {
            Element::fragment().with_children(vec![
                self.0.render(),
                Element::typed::<SlowSibling>(Default::default()),
            ])
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn accepts_input(&self) -> bool {
            self.0.active_count() == 0
        }
    }
    for animation in [DialogAnimation::None, DialogAnimation::Fade] {
        for duration in [
            std::time::Duration::ZERO,
            std::time::Duration::from_millis(60),
        ] {
            let mut engine = DialogEngine::with_config(DialogEngineConfig {
                default_theme: DialogTheme {
                    animation: animation.clone(),
                    ..Default::default()
                },
                ..Default::default()
            });
            let id = engine.show_toast(ToastOptions {
                message: "OWNED TOAST".into(),
                duration: Some(duration),
                ..Default::default()
            });
            let frames = app_input::run_visibility(
                Idle(engine.clone()),
                (40, 12),
                vec![("", Some("OWNED TOAST"), None)],
            );
            assert!(!frames.last().unwrap().text.contains("OWNED TOAST"));
            assert!(
                frames
                    .iter()
                    .any(|frame| frame.text.contains("OWNED TOAST")),
                "toast expired without being presented: {:?}",
                frames.iter().map(|frame| &frame.text).collect::<Vec<_>>()
            );
            assert!(matches!(engine.take_event(), Some(DialogEvent::Opened(open)) if open == id));
            assert!(
                matches!(engine.take_event(), Some(DialogEvent::Closed(closed, DialogResult::Confirmed(None))) if closed == id)
            );
        }
    }
}

#[test]
fn dropping_the_app_cancels_sessions_even_when_a_controller_survives() {
    let mut engine = DialogEngine::new();
    engine.enable_async();
    let id = engine.show_toast(ToastOptions {
        duration: Some(std::time::Duration::from_secs(60)),
        ..Default::default()
    });
    let completion = engine.completion(id).unwrap();
    app_input::run(engine.clone(), (32, 12), vec![(1, None)]);
    assert_eq!(engine.active_count(), 0);
    assert!(matches!(
        completion.try_result(),
        Some(DialogResult::Cancelled)
    ));
    let events: Vec<_> = std::iter::from_fn(|| engine.take_event()).collect();
    assert!(matches!(
        events.as_slice(),
        [
            DialogEvent::Opened(_),
            DialogEvent::Closed(_, DialogResult::Cancelled)
        ]
    ));
}

#[test]
fn nested_dialogs_restore_the_underlying_control_and_then_the_background() {
    use reactive_tui::reactive::ThreadSafeSignal;
    struct Root {
        engine: DialogEngine,
        opened: ThreadSafeSignal<usize>,
    }
    impl RootComponent for Root {
        fn render(&self) -> Element {
            let engine = self.engine.clone();
            let opened = self.opened.clone();
            let opener = builder::button()
                .text("OPEN")
                .on_click(move || {
                    opened.update(|n| *n += 1);
                    let nested = engine.clone();
                    let mut engine = engine.clone();
                    let mut more = ConfirmationButton::ok();
                    more.id = "more".into();
                    more.text = "More".into();
                    engine
                        .try_show_confirmation(ConfirmationDialogOptions {
                            title: "OUTER".into(),
                            buttons: ConfirmationButtons::Custom(vec![
                                more,
                                ConfirmationButton::cancel(),
                            ]),
                            default_button: Some("more".into()),
                            on_button_click: Some(Arc::new(move |button| {
                                if button == "more" {
                                    let mut nested = nested.clone();
                                    nested
                                        .try_show_confirmation(ConfirmationDialogOptions {
                                            title: "INNER".into(),
                                            ..Default::default()
                                        })
                                        .unwrap();
                                    false
                                } else {
                                    true
                                }
                            })),
                            ..Default::default()
                        })
                        .unwrap();
                })
                .build()
                .auto_focus()
                .with_key("opener");
            builder::div()
                .children(vec![opener, self.engine.render().with_key("dialogs")])
                .build()
        }
        fn wake_driven(&self) -> bool {
            true
        }
    }
    for size in [(32, 12), (60, 20)] {
        let mut engine = DialogEngine::new();
        let opened = ThreadSafeSignal::new(0);
        app_input::run_visibility(
            Root {
                engine: engine.clone(),
                opened: opened.clone(),
            },
            size,
            vec![
                ("OPEN", Some("OUTER"), key(KeyCode::Enter)),
                ("OUTER", Some("INNER"), key(KeyCode::Enter)),
                ("INNER", None, key(KeyCode::Escape)),
                ("OUTER", Some("INNER"), key(KeyCode::Enter)),
                ("INNER", None, key(KeyCode::Enter)),
                ("OUTER", Some("INNER"), key(KeyCode::Tab)),
                ("OUTER", Some("INNER"), key(KeyCode::Enter)),
                ("OPEN", Some("OUTER"), key(KeyCode::Enter)),
                ("OUTER", None, key(KeyCode::Escape)),
                ("OPEN", Some("OUTER"), None),
            ],
        );
        assert_eq!(opened.get(), 2, "focus must return to the original opener");
        let closed: Vec<_> = std::iter::from_fn(|| engine.take_event())
            .filter_map(|event| {
                if let DialogEvent::Closed(id, result) = event {
                    Some((id.as_u32(), result))
                } else {
                    None
                }
            })
            .collect();
        assert!(
            matches!(
                closed.as_slice(),
                [
                    (2, DialogResult::Cancelled),
                    (3, DialogResult::Confirmed(None)),
                    (1, DialogResult::Cancelled),
                    (4, DialogResult::Cancelled)
                ]
            ),
            "{closed:?}"
        );
    }
}

#[test]
fn engine_z_priority_controls_both_paint_and_pointer_target() {
    for size in [(40, 14), (60, 20)] {
        let mut engine = DialogEngine::new();
        let high = engine.show_confirmation(ConfirmationDialogOptions {
            title: "HIGH TOP".into(),
            size: Some(Size::new(30, 8)),
            ..Default::default()
        });
        let low = engine.show_confirmation(ConfirmationDialogOptions {
            title: "LOW BASE".into(),
            size: Some(Size::new(30, 8)),
            ..Default::default()
        });
        engine.update(high, DialogUpdate::ZIndex(9000)).unwrap();
        app_input::run_actions_until_hidden(
            engine.clone(),
            size,
            vec![
                ("HIGH TOP", Action::ClickText("OK", 0)),
                ("LOW BASE", Action::ClickText("OK", 0)),
            ],
            "LOW BASE",
        );
        let closed: Vec<_> = std::iter::from_fn(|| engine.take_event())
            .filter_map(|event| {
                if let DialogEvent::Closed(id, result) = event {
                    assert!(matches!(result, DialogResult::Confirmed(None)));
                    Some(id)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(closed, vec![high, low]);
    }
}

#[test]
fn progress_updates_change_the_presented_control_and_deliver_completion() {
    struct Updating {
        engine: DialogEngine,
        id: DialogId,
    }
    impl RootComponent for Updating {
        fn render(&self) -> Element {
            self.engine.render()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn try_handle_event(&mut self, event: &Event) -> reactive_tui::error::Result<EventResult> {
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::F(2) => self
                        .engine
                        .update(self.id, DialogUpdate::Progress(0.5))
                        .unwrap(),
                    KeyCode::F(3) => self
                        .engine
                        .close_dialog(self.id, DialogResult::Confirmed(Some("completed".into()))),
                    _ => return Ok(EventResult::Ignored),
                }
                return Ok(EventResult::Handled);
            }
            Ok(EventResult::Ignored)
        }
    }
    let mut engine = DialogEngine::new();
    engine.enable_async();
    let id = engine.show_progress(ProgressDialogOptions {
        title: "WORKING".into(),
        ..Default::default()
    });
    let completion = engine.completion(id).unwrap();
    app_input::run_until_hidden(
        Updating {
            engine: engine.clone(),
            id,
        },
        (40, 14),
        vec![("0.0%", key(KeyCode::F(2))), ("50.0%", key(KeyCode::F(3)))],
        "WORKING",
    );
    assert!(
        matches!(completion.try_result(), Some(DialogResult::Confirmed(Some(value))) if value == "completed")
    );
}

#[test]
fn engine_escape_configuration_preserves_keyboard_cancel() {
    for size in [(32, 12), (60, 20)] {
        let mut engine = DialogEngine::with_config(DialogEngineConfig {
            escape_to_close: false,
            ..Default::default()
        });
        let id = engine.show_input(InputDialogOptions {
            title: "NO ESCAPE ENGINE".into(),
            ..Default::default()
        });
        app_input::run_until_hidden(
            engine.clone(),
            size,
            vec![
                ("NO ESCAPE ENGINE", key(KeyCode::Escape)),
                ("NO ESCAPE ENGINE", key(KeyCode::Tab)),
                ("NO ESCAPE ENGINE", key(KeyCode::Enter)),
            ],
            "NO ESCAPE ENGINE",
        );
        assert!(matches!(engine.take_event(), Some(DialogEvent::Opened(open)) if open == id));
        assert!(
            matches!(engine.take_event(), Some(DialogEvent::Closed(closed, DialogResult::Cancelled)) if closed == id)
        );
    }
}

#[test]
fn custom_animation_receives_intermediate_progress_and_changes_app_frames() {
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Animate {
        engine: DialogEngine,
        finished: Arc<AtomicBool>,
    }
    impl RootComponent for Animate {
        fn render(&self) -> Element {
            self.engine.render()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn accepts_input(&self) -> bool {
            self.finished.load(Ordering::Acquire)
        }
    }
    let mut engine = DialogEngine::with_config(DialogEngineConfig {
        // Long enough that a machine-wide stall (memory reclaim can pause
        // every task for more than 120 ms) cannot skip every intermediate
        // frame; the test ends when the animation does.
        animation_duration: std::time::Duration::from_millis(1000),
        default_theme: DialogTheme {
            animation: DialogAnimation::Custom("test-motion".into()),
            ..Default::default()
        },
        ..Default::default()
    });
    let finished = Arc::new(AtomicBool::new(false));
    let done = finished.clone();
    let samples = Arc::new(Mutex::new(Vec::new()));
    let captured = samples.clone();
    engine
        .register_animation("test-motion", move |progress| {
            captured.lock().unwrap().push(progress);
            done.store(progress == 1.0, Ordering::Release);
            DialogAnimationFrame {
                opacity: progress,
                x: (1.0 - progress) * 3.0,
                ..Default::default()
            }
        })
        .unwrap();
    engine.show_confirmation(ConfirmationDialogOptions {
        title: "ANIMATED".into(),
        ..Default::default()
    });
    let frames = app_input::run_when(
        Animate { engine, finished },
        (40, 14),
        vec![("ANIMATED", None)],
    );
    let samples = samples.lock().unwrap();
    assert!(samples.iter().any(|p| *p > 0.0 && *p < 1.0), "{samples:?}");
    assert_eq!(samples.last(), Some(&1.0));
    assert!(frames.windows(2).any(|pair| pair[0].text != pair[1].text));
}

#[test]
fn invalid_custom_animation_completes_with_error_instead_of_silent_success() {
    struct UntilClosed(DialogEngine);
    impl RootComponent for UntilClosed {
        fn render(&self) -> Element {
            self.0.render()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn accepts_input(&self) -> bool {
            self.0.active_count() == 0
        }
    }
    let mut engine = DialogEngine::with_config(DialogEngineConfig {
        default_theme: DialogTheme {
            animation: DialogAnimation::Custom("invalid".into()),
            ..Default::default()
        },
        ..Default::default()
    });
    engine
        .register_animation("invalid", |_| DialogAnimationFrame {
            x: f32::NAN,
            ..Default::default()
        })
        .unwrap();
    engine.enable_async();
    let id = engine.show_confirmation(Default::default());
    let completion = engine.completion(id).unwrap();
    app_input::run_visibility(
        UntilClosed(engine),
        (40, 14),
        vec![("", Some("Confirm"), None)],
    );
    assert!(
        matches!(completion.try_result(), Some(DialogResult::Error(message)) if message.contains("invalid"))
    );
}

#[test]
fn explicit_input_reset_replaces_edits_even_when_the_seed_value_is_unchanged() {
    struct Reset {
        engine: DialogEngine,
        id: DialogId,
    }
    impl RootComponent for Reset {
        fn render(&self) -> Element {
            self.engine.render()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn try_handle_event(&mut self, event: &Event) -> reactive_tui::error::Result<EventResult> {
            if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
                self.engine
                    .update(self.id, DialogUpdate::InputValue("seed".into()))
                    .unwrap();
                Ok(EventResult::Handled)
            } else {
                Ok(EventResult::Ignored)
            }
        }
    }
    let mut engine = DialogEngine::new();
    engine.enable_async();
    let id = engine.show_input(InputDialogOptions {
        title: "RESET INPUT".into(),
        input: InputFieldConfig {
            default_value: Some("seed".into()),
            ..Default::default()
        },
        ..Default::default()
    });
    let completion = engine.completion(id).unwrap();
    app_input::run_visibility(
        Reset { engine, id },
        (40, 14),
        vec![
            ("seed", None, key(KeyCode::End)),
            ("seed", None, key(KeyCode::Char('X'))),
            ("seedX", None, key(KeyCode::F(2))),
            ("seed", Some("seedX"), key(KeyCode::Enter)),
            ("", Some("RESET INPUT"), None),
        ],
    );
    let result = completion.try_result();
    assert!(
        matches!(&result, Some(DialogResult::Confirmed(Some(value))) if value == "seed"),
        "{result:?}"
    );
}

#[test]
fn autocomplete_selection_and_wizard_data_reach_engine_results() {
    for size in [(32, 12), (60, 20)] {
        let mut engine = DialogEngine::new();
        engine.enable_async();
        let id = engine.show_autocomplete(AutocompleteDialogOptions {
            title: "ENGINE SEARCH".into(),
            autocomplete: AutocompleteConfig {
                default_value: Some("Al".into()),
                min_chars: 1,
                static_suggestions: vec!["Alpha".into(), "Alpine".into()],
                debounce_delay: std::time::Duration::ZERO,
                ..Default::default()
            },
            ..Default::default()
        });
        let completion = engine.completion(id).unwrap();
        app_input::run_until_hidden(
            engine.clone(),
            size,
            vec![("Alpha", key(KeyCode::Enter))],
            "ENGINE SEARCH",
        );
        assert!(
            matches!(completion.try_result(), Some(DialogResult::Selected(value)) if value == "Alpha")
        );

        let completed = Arc::new(Mutex::new(Vec::new()));
        let captured = completed.clone();
        let id = engine.show_wizard(WizardDialogOptions {
            title: "ENGINE WIZARD".into(),
            steps: vec![WizardStep {
                id: "one".into(),
                title: "First step".into(),
                content: Element::text("CONTENT"),
                can_skip: false,
                validator: Some(Arc::new(|data| ValidationResult {
                    valid: data.get("name").is_some_and(|value| value == "Ada"),
                    ..Default::default()
                })),
            }],
            on_complete: Some(Arc::new(move |data| {
                captured.lock().unwrap().push(data.clone());
                true
            })),
            ..Default::default()
        });
        engine
            .update(
                id,
                DialogUpdate::WizardData(std::collections::HashMap::from([(
                    "name".into(),
                    "Ada".into(),
                )])),
            )
            .unwrap();
        let completion = engine.completion(id).unwrap();
        app_input::run_until_hidden(
            engine.clone(),
            size,
            vec![("Finish", key(KeyCode::Enter))],
            "ENGINE WIZARD",
        );
        assert!(matches!(
            completion.try_result(),
            Some(DialogResult::Confirmed(None))
        ));
        assert_eq!(
            completed.lock().unwrap().as_slice(),
            &[std::collections::HashMap::from([(
                "name".into(),
                "Ada".into()
            )])]
        );
    }
}
