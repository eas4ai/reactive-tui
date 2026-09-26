use super::{app_input, Control};
use reactive_tui::{builder, event::types::KeyCode};

#[test]
fn input_dialogs_keep_cancel_keyboard_access_when_escape_dismissal_is_disabled() {
    use reactive_tui::{
        core::geometry::Rect,
        widgets::dialog::{
            AutocompleteConfig, AutocompleteDialog, AutocompleteDialogOptions, DialogComponent,
            DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
        },
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        for autocomplete in [false, true] {
            let results = Arc::new(Mutex::new(Vec::new()));
            let closed = results.clone();
            let callback: Arc<dyn Fn(DialogResult) + Send + Sync> =
                Arc::new(move |result| closed.lock().unwrap().push(result));
            let element = if autocomplete {
                AutocompleteDialog::new(
                    DialogId::from_u32(81),
                    AutocompleteDialogOptions {
                        title: "NO ESCAPE".into(),
                        prompt: "VALUE".into(),
                        escape_closable: false,
                        autocomplete: AutocompleteConfig {
                            min_chars: 1,
                            static_suggestions: vec!["Alpha".into()],
                            debounce_delay: std::time::Duration::ZERO,
                            ..Default::default()
                        },
                        on_close: Some(callback),
                        ..Default::default()
                    },
                )
                .render(Rect::default(), &DialogTheme::default())
            } else {
                InputDialog::new(
                    DialogId::from_u32(80),
                    InputDialogOptions {
                        title: "NO ESCAPE".into(),
                        prompt: "VALUE".into(),
                        escape_closable: false,
                        on_close: Some(callback),
                        ..Default::default()
                    },
                )
                .render(Rect::default(), &DialogTheme::default())
            };
            app_input::run_visibility(
                Control(element),
                size,
                vec![
                    ("VALUE", None, super::key(KeyCode::Char('a'))),
                    (
                        if autocomplete { "Alpha" } else { "VALUE" },
                        None,
                        super::key(KeyCode::Escape),
                    ),
                    ("VALUE", Some("Alpha"), super::key(KeyCode::Escape)),
                    ("VALUE", Some("Alpha"), super::key(KeyCode::Tab)),
                    ("VALUE", Some("Alpha"), super::key(KeyCode::Enter)),
                    ("", Some("NO ESCAPE"), None),
                ],
            );
            assert!(matches!(
                results.lock().unwrap().as_slice(),
                [DialogResult::Cancelled]
            ));
        }
    }
}

#[test]
fn input_dialog_pointer_edits_and_submits_at_the_resized_painted_coordinates() {
    use app_input::Action;
    use reactive_tui::{
        core::geometry::Rect,
        event::types::{Event, ResizeEvent},
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            InputFieldConfig,
        },
    };
    use std::sync::{Arc, Mutex};
    for (initial, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let result = Arc::new(Mutex::new(None));
        let closed = result.clone();
        let element = InputDialog::new(
            DialogId::from_u32(22),
            InputDialogOptions {
                title: "POINTER".into(),
                prompt: "VALUE".into(),
                input: InputFieldConfig {
                    default_value: Some("abcd".into()),
                    ..Default::default()
                },
                on_close: Some(Arc::new(move |value| {
                    let DialogResult::Confirmed(Some(value)) = value else {
                        panic!("wrong result: {value:?}")
                    };
                    assert!(closed.lock().unwrap().replace(value).is_none());
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        let frames = app_input::run_actions_until_hidden(
            Control(element),
            initial,
            vec![
                (
                    "abcd",
                    Action::Event(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                ),
                ("abcd", Action::ClickText("abcd", 2)),
                (
                    "abcd",
                    Action::Event(super::key(KeyCode::Char('X')).unwrap()),
                ),
                ("abXcd", Action::ClickText("OK", 0)),
            ],
            "POINTER",
        );
        assert_eq!(frames.last().unwrap().screen.size(), (resized.1, resized.0));
        assert_eq!(result.lock().unwrap().as_deref(), Some("abXcd"));
    }
}

#[test]
fn input_dialog_prop_updates_preserve_edits_replace_callbacks_and_apply_new_seed() {
    use reactive_tui::{
        app::RootComponent,
        component::Element,
        core::geometry::Rect,
        event::{router::EventResult, types::Event},
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            InputFieldConfig,
        },
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };
    struct Updating {
        phase: AtomicUsize,
        changes: Arc<Mutex<Vec<(usize, String)>>>,
        submitted: Arc<Mutex<Vec<(usize, String)>>>,
        closed: Arc<Mutex<Vec<usize>>>,
    }
    impl RootComponent for Updating {
        fn render(&self) -> Element {
            let phase = self.phase.load(Ordering::SeqCst);
            let changes = self.changes.clone();
            let submitted = self.submitted.clone();
            let closed = self.closed.clone();
            InputDialog::new(
                DialogId::from_u32(21),
                InputDialogOptions {
                    title: "UPDATING".into(),
                    prompt: ["EDITABLE", "READONLY", "REPLACED"][phase].into(),
                    input: InputFieldConfig {
                        default_value: Some(if phase == 2 { "reset" } else { "draft" }.into()),
                        attributes: [("readonly".into(), (phase == 1).to_string())].into(),
                        ..Default::default()
                    },
                    on_change: Some(Arc::new(move |value| {
                        changes.lock().unwrap().push((phase, value.into()))
                    })),
                    on_submit: Some(Arc::new(move |value| {
                        submitted.lock().unwrap().push((phase, value.into()));
                        true
                    })),
                    on_close: Some(Arc::new(move |result| {
                        assert!(matches!(result, DialogResult::Confirmed(_)));
                        closed.lock().unwrap().push(phase);
                    })),
                    ..Default::default()
                },
            )
            .render(Rect::default(), &DialogTheme::default())
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if let Event::Key(key) = event {
                let phase = match key.code {
                    KeyCode::F(2) => 1,
                    KeyCode::F(3) => 2,
                    _ => return EventResult::Ignored,
                };
                self.phase.store(phase, Ordering::SeqCst);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }
    for size in [(32, 12), (60, 20)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let submitted = Arc::new(Mutex::new(Vec::new()));
        let closed = Arc::new(Mutex::new(Vec::new()));
        app_input::run_until_hidden(
            Updating {
                phase: AtomicUsize::new(0),
                changes: changes.clone(),
                submitted: submitted.clone(),
                closed: closed.clone(),
            },
            size,
            vec![
                ("EDITABLE", super::key(KeyCode::End)),
                ("draft", super::key(KeyCode::Char('X'))),
                ("draftX", super::key(KeyCode::F(2))),
                ("READONLY", super::key(KeyCode::Backspace)),
                ("draftX", super::key(KeyCode::Char('Y'))),
                ("draftX", super::key(KeyCode::F(3))),
                ("reset", super::key(KeyCode::End)),
                ("REPLACED", super::key(KeyCode::Char('Z'))),
                ("resetZ", super::key(KeyCode::Enter)),
            ],
            "UPDATING",
        );
        assert_eq!(
            *changes.lock().unwrap(),
            [(0, "draftX".into()), (2, "resetZ".into())]
        );
        assert_eq!(*submitted.lock().unwrap(), [(2, "resetZ".into())]);
        assert_eq!(*closed.lock().unwrap(), [2]);
    }
}

#[test]
fn input_dialog_mask_native_submission_checks_literals_unicode_and_configuration() {
    use reactive_tui::{
        event::types::{Event, KeyEvent},
        widgets::dialog::{
            DialogComponent, DialogEventResult, DialogId, InputDialog, InputDialogOptions,
            InputFieldConfig,
        },
    };
    for (mask, value, required, valid) in [
        ("AA-##", "界e\u{301}-42", true, true),
        ("AA-##", "AB-4", true, false),
        ("AA-##", "AB-４2", true, false),
        ("AA-##", "🙂B-42", true, false),
        ("AA-##", "AB/42", true, false),
        ("AA-##", "AB-420", true, false),
        (r"\#\A\**", "#A*👩‍💻", true, true),
        (r"\\#", r"\5", true, true),
        ("*", "\n", true, false),
        ("#", "", false, true),
        ("#", "", true, false),
        ("\\", "", false, false),
        ("#\\", "x", false, false),
        ("\n", "", false, false),
    ] {
        let mut dialog = InputDialog::new(
            DialogId::from_u32(20),
            InputDialogOptions {
                input: InputFieldConfig {
                    mask: Some(mask.into()),
                    default_value: Some(value.into()),
                    required,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        let result = dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter)));
        assert_eq!(
            matches!(result, DialogEventResult::Close(_)),
            valid,
            "mask {mask:?}, value {value:?}"
        );
    }
}

#[test]
fn input_dialog_mask_rejects_invalid_format_then_submits_corrected_unicode() {
    use reactive_tui::{
        core::geometry::Rect,
        event::types::{Event, KeyEvent, KeyModifiers, PasteEvent},
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            InputFieldConfig,
        },
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let result = Arc::new(Mutex::new(None));
        let closed = result.clone();
        let dialog = InputDialog::new(
            DialogId::from_u32(19),
            InputDialogOptions {
                title: "MASK".into(),
                prompt: "CODE".into(),
                input: InputFieldConfig {
                    mask: Some("AA-##".into()),
                    required: true,
                    ..Default::default()
                },
                on_close: Some(Arc::new(move |value| {
                    let DialogResult::Confirmed(Some(value)) = value else {
                        panic!("wrong close result");
                    };
                    assert!(closed.lock().unwrap().replace(value).is_none());
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("CODE", Some(Event::Paste(PasteEvent::new("12-AB".into())))),
                ("12-AB", super::key(KeyCode::Enter)),
                (
                    "format",
                    Some(Event::Key(
                        KeyEvent::new(KeyCode::Char('a')).with_modifiers(KeyModifiers::ctrl()),
                    )),
                ),
                (
                    "format",
                    Some(Event::Paste(PasteEvent::new("界e\u{301}-42".into()))),
                ),
                ("界e\u{301}-42", super::key(KeyCode::Enter)),
            ],
            "MASK",
        );
        assert_eq!(result.lock().unwrap().as_deref(), Some("界e\u{301}-42"));
    }
}

#[test]
fn input_dialog_validates_edits_graphemes_and_delivers_submission_through_app() {
    use reactive_tui::{
        core::geometry::Rect,
        event::types::{Event, PasteEvent},
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            InputFieldConfig,
        },
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let output = changes.clone();
        let result = Arc::new(Mutex::new(None));
        let closed = result.clone();
        let dialog = InputDialog::new(
            DialogId::from_u32(10),
            InputDialogOptions {
                title: "INPUT".into(),
                prompt: "NAME".into(),
                input: InputFieldConfig {
                    required: true,
                    max_length: Some(4),
                    ..Default::default()
                },
                on_change: Some(Arc::new(move |value| {
                    output.lock().unwrap().push(value.to_string())
                })),
                on_close: Some(Arc::new(move |value| {
                    let DialogResult::Confirmed(Some(value)) = value else {
                        panic!("wrong close result");
                    };
                    assert!(closed.lock().unwrap().replace(value).is_none());
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("NAME", super::key(KeyCode::Enter)),
                (
                    "required",
                    Some(Event::Paste(PasteEvent::new("界e\u{301}🙂a".into()))),
                ),
                ("界e\u{301}🙂a", super::key(KeyCode::Char('Z'))),
                ("界e\u{301}🙂a", super::key(KeyCode::Left)),
                ("界e\u{301}🙂a", super::key(KeyCode::Backspace)),
                ("界e\u{301}a", super::key(KeyCode::Enter)),
            ],
            "INPUT",
        );
        assert_eq!(*changes.lock().unwrap(), ["界e\u{301}🙂a", "界e\u{301}a"]);
        assert_eq!(result.lock().unwrap().as_deref(), Some("界e\u{301}a"));
    }
}

#[test]
fn input_dialog_readonly_keeps_value_while_allowing_submission_through_app() {
    use reactive_tui::{
        core::geometry::Rect,
        event::types::{Event, PasteEvent},
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            InputFieldConfig,
        },
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let result = Arc::new(Mutex::new(None));
        let closed = result.clone();
        let dialog = InputDialog::new(
            DialogId::from_u32(17),
            InputDialogOptions {
                title: "READONLY".into(),
                prompt: "FIXED VALUE".into(),
                input: InputFieldConfig {
                    default_value: Some("fixed".into()),
                    attributes: [("readonly".into(), "true".into())].into(),
                    ..Default::default()
                },
                on_change: Some(Arc::new(|_| panic!("read-only field changed"))),
                on_close: Some(Arc::new(move |value| {
                    let DialogResult::Confirmed(Some(value)) = value else {
                        panic!("wrong close result");
                    };
                    assert!(closed.lock().unwrap().replace(value).is_none());
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("fixed", super::key(KeyCode::End)),
                ("fixed", super::key(KeyCode::Char('x'))),
                ("fixed", super::key(KeyCode::Backspace)),
                ("fixed", Some(Event::Paste(PasteEvent::new("bad".into())))),
                ("fixed", super::key(KeyCode::Enter)),
            ],
            "READONLY",
        );
        assert_eq!(result.lock().unwrap().as_deref(), Some("fixed"));
    }
}

#[test]
fn input_dialog_direct_readonly_disabled_and_key_release_do_not_edit() {
    use reactive_tui::{
        event::types::{Event, KeyEvent, KeyEventKind, PasteEvent},
        widgets::dialog::{
            DialogComponent, DialogId, InputDialog, InputDialogOptions, InputFieldConfig,
        },
    };
    for attribute in ["readonly", "disabled"] {
        let mut dialog = InputDialog::new(
            DialogId::from_u32(17),
            InputDialogOptions {
                input: InputFieldConfig {
                    default_value: Some("fixed".into()),
                    attributes: [(attribute.into(), "true".into())].into(),
                    ..Default::default()
                },
                on_change: Some(Arc::new(|_| panic!("protected field changed"))),
                ..Default::default()
            },
        );
        dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('x'))));
        dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Backspace)));
        dialog.handle_event(&Event::Paste(PasteEvent::new("bad".into())));
        if attribute == "disabled" {
            assert!(!dialog.set_focus("input"));
            assert!(
                !dialog
                    .get_focusable_elements()
                    .iter()
                    .find(|element| element.id == "input")
                    .unwrap()
                    .enabled
            );
        }
    }
    use std::sync::Arc;
    let mut dialog = InputDialog::new(
        DialogId::from_u32(17),
        InputDialogOptions {
            on_change: Some(Arc::new(|_| panic!("release changed field"))),
            ..Default::default()
        },
    );
    let mut event = KeyEvent::new(KeyCode::Char('x'));
    event.kind = KeyEventKind::Release;
    dialog.handle_event(&Event::Key(event));
}

#[test]
fn input_dialog_validator_types_and_rules_reject_invalid_values() {
    use reactive_tui::widgets::dialog::{
        DialogComponent, DialogId, InputDialog, InputDialogOptions, InputFieldConfig, InputType,
        ValidationConfig, ValidationRule, ValidationRuleType,
    };
    for (input_type, good, bad) in [
        (InputType::Email, "person@example.test", "person@"),
        (InputType::Number, "-12.5", "NaN"),
        (InputType::Url, "https://example.test/path", "example.test"),
        (InputType::Phone, "+1 (234) 567-8901", "123"),
    ] {
        for (value, valid) in [(good, true), (bad, false)] {
            let dialog = InputDialog::new(
                DialogId::from_u32(16),
                InputDialogOptions {
                    input: InputFieldConfig {
                        input_type: input_type.clone(),
                        default_value: Some(value.into()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            let result = dialog.validate();
            assert_eq!(result.valid, valid, "{input_type:?}: {value}");
            assert_eq!(result.errors.is_empty(), valid);
        }
    }
    for (rule_type, good, bad) in [
        (ValidationRuleType::Required, "x", " "),
        (ValidationRuleType::MinLength(2), "界e\u{301}", "e\u{301}"),
        (
            ValidationRuleType::MaxLength(2),
            "界e\u{301}",
            "界e\u{301}🙂",
        ),
        (
            ValidationRuleType::Pattern("^[A-Z]{2}[0-9]+$".into()),
            "AB123",
            "ab123",
        ),
    ] {
        for (value, valid) in [(good, true), (bad, false)] {
            let dialog = InputDialog::new(
                DialogId::from_u32(16),
                InputDialogOptions {
                    input: InputFieldConfig {
                        default_value: Some(value.into()),
                        ..Default::default()
                    },
                    validation: Some(ValidationConfig {
                        rules: vec![ValidationRule {
                            rule_type: rule_type.clone(),
                            message: "Rule rejected value".into(),
                            required: true,
                        }],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            );
            let result = dialog.validate();
            assert_eq!(result.valid, valid, "{rule_type:?}: {value}");
            if !valid {
                assert_eq!(
                    result.errors.get("input").map(String::as_str),
                    Some("Rule rejected value")
                );
            }
        }
    }
}

#[test]
fn input_dialog_blur_validation_runs_before_cancel_closes_it() {
    use reactive_tui::{
        core::geometry::Rect,
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            InputFieldConfig, ValidationConfig,
        },
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(32, 12), (60, 20)] {
        let cancelled = Arc::new(AtomicUsize::new(0));
        let closed = cancelled.clone();
        let dialog = InputDialog::new(
            DialogId::from_u32(15),
            InputDialogOptions {
                title: "BLUR".into(),
                prompt: "REQUIRED VALUE".into(),
                input: InputFieldConfig {
                    required: true,
                    ..Default::default()
                },
                validation: Some(ValidationConfig {
                    validate_on_change: false,
                    validate_on_blur: true,
                    ..Default::default()
                }),
                on_close: Some(Arc::new(move |value| {
                    assert!(matches!(value, DialogResult::Cancelled));
                    closed.fetch_add(1, Ordering::SeqCst);
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        let frames = app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("REQUIRED VALUE", super::key(KeyCode::Tab)),
                ("This field is required", super::key(KeyCode::Enter)),
            ],
            "BLUR",
        );
        assert!(!frames
            .first()
            .unwrap()
            .text
            .contains("This field is required"));
        assert_eq!(cancelled.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn input_dialog_change_validation_shows_warnings_without_blocking_submit() {
    use reactive_tui::{
        core::geometry::Rect,
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            ValidationConfig, ValidationResult,
        },
    };
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    for size in [(32, 12), (60, 20)] {
        let result = Arc::new(Mutex::new(None));
        let closed = result.clone();
        let dialog = InputDialog::new(
            DialogId::from_u32(14),
            InputDialogOptions {
                title: "VALIDATION".into(),
                prompt: "VALUE".into(),
                validation: Some(ValidationConfig {
                    validate_on_change: true,
                    debounce_delay: Duration::from_millis(1),
                    custom_validator: Some(Arc::new(|_| ValidationResult {
                        valid: true,
                        message: None,
                        warnings: vec!["Check spelling".into()],
                    })),
                    ..Default::default()
                }),
                on_close: Some(Arc::new(move |value| {
                    let DialogResult::Confirmed(Some(value)) = value else {
                        panic!("wrong close result");
                    };
                    assert!(closed.lock().unwrap().replace(value).is_none());
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("VALUE", super::key(KeyCode::Char('a'))),
                ("Check spelling", super::key(KeyCode::Enter)),
            ],
            "VALIDATION",
        );
        assert_eq!(result.lock().unwrap().as_deref(), Some("a"));
    }
}

#[test]
fn input_dialog_multiline_enter_edits_and_ok_submits_both_lines() {
    use reactive_tui::{
        core::geometry::Rect,
        event::types::{Event, PasteEvent},
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            InputFieldConfig,
        },
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 14), (60, 20)] {
        let result = Arc::new(Mutex::new(None));
        let closed = result.clone();
        let dialog = InputDialog::new(
            DialogId::from_u32(13),
            InputDialogOptions {
                title: "MULTILINE".into(),
                prompt: "LINES".into(),
                input: InputFieldConfig {
                    multiline: true,
                    rows: Some(3),
                    show_count: true,
                    ..Default::default()
                },
                on_close: Some(Arc::new(move |value| {
                    let DialogResult::Confirmed(Some(value)) = value else {
                        panic!("wrong close result");
                    };
                    assert!(closed.lock().unwrap().replace(value).is_none());
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("LINES", Some(Event::Paste(PasteEvent::new("first".into())))),
                ("first", super::key(KeyCode::Enter)),
                (
                    "6 characters",
                    Some(Event::Paste(PasteEvent::new("second".into()))),
                ),
                ("second", super::key(KeyCode::BackTab)),
                ("second", super::key(KeyCode::BackTab)),
                ("second", super::key(KeyCode::BackTab)),
                ("second", super::key(KeyCode::Enter)),
            ],
            "MULTILINE",
        );
        assert_eq!(result.lock().unwrap().as_deref(), Some("first\nsecond"));
    }
}

#[test]
fn input_dialog_password_masks_frames_and_returns_the_original_value() {
    use reactive_tui::{
        core::geometry::Rect,
        event::types::{Event, PasteEvent},
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            InputFieldConfig, InputType,
        },
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let result = Arc::new(Mutex::new(None));
        let closed = result.clone();
        let dialog = InputDialog::new(
            DialogId::from_u32(12),
            InputDialogOptions {
                title: "PASSWORD".into(),
                prompt: "SECRET".into(),
                input: InputFieldConfig {
                    input_type: InputType::Password,
                    show_count: true,
                    ..Default::default()
                },
                on_close: Some(Arc::new(move |value| {
                    let DialogResult::Confirmed(Some(value)) = value else {
                        panic!("wrong close result");
                    };
                    assert!(closed.lock().unwrap().replace(value).is_none());
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        let frames = app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                (
                    "SECRET",
                    Some(Event::Paste(PasteEvent::new("界e\u{301}🙂".into()))),
                ),
                ("***", super::key(KeyCode::Backspace)),
                ("2 characters", super::key(KeyCode::Enter)),
            ],
            "PASSWORD",
        );
        assert!(frames
            .iter()
            .all(|frame| !frame.text.contains('界') && !frame.text.contains('🙂')));
        assert_eq!(result.lock().unwrap().as_deref(), Some("界e\u{301}"));
    }
}

#[test]
fn input_dialog_app_length_limit_accepts_combining_marks_and_rejects_oversized_paste() {
    use reactive_tui::{
        core::geometry::Rect,
        event::types::{Event, PasteEvent},
        widgets::dialog::{
            DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
            InputFieldConfig,
        },
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let result = Arc::new(Mutex::new(None));
        let closed = result.clone();
        let dialog = InputDialog::new(
            DialogId::from_u32(11),
            InputDialogOptions {
                title: "LIMIT".into(),
                prompt: "LETTER".into(),
                input: InputFieldConfig {
                    default_value: Some("e".into()),
                    max_length: Some(1),
                    ..Default::default()
                },
                on_close: Some(Arc::new(move |value| {
                    let DialogResult::Confirmed(Some(value)) = value else {
                        panic!("wrong close result");
                    };
                    assert!(closed.lock().unwrap().replace(value).is_none());
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("LETTER", super::key(KeyCode::End)),
                ("LETTER", super::key(KeyCode::Char('\u{301}'))),
                (
                    "e\u{301}",
                    Some(Event::Paste(PasteEvent::new("🙂a".into()))),
                ),
                ("e\u{301}", super::key(KeyCode::Enter)),
            ],
            "LIMIT",
        );
        assert_eq!(result.lock().unwrap().as_deref(), Some("e\u{301}"));
    }
}

#[test]
fn input_dialog_direct_length_limit_counts_the_resulting_graphemes() {
    use reactive_tui::{
        event::types::{Event, KeyEvent, PasteEvent},
        widgets::dialog::{
            DialogComponent, DialogEventResult, DialogId, DialogResult, InputDialog,
            InputDialogOptions, InputFieldConfig,
        },
    };
    let mut dialog = InputDialog::new(
        DialogId::from_u32(11),
        InputDialogOptions {
            input: InputFieldConfig {
                default_value: Some("e".into()),
                max_length: Some(1),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('\u{301}'))));
    dialog.handle_event(&Event::Paste(PasteEvent::new("🙂a".into())));
    assert!(matches!(
        dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter))),
        DialogEventResult::Close(DialogResult::Confirmed(Some(value))) if value == "e\u{301}"
    ));
    dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Backspace)));
    dialog.handle_event(&Event::Paste(PasteEvent::new("🙂a".into())));
    assert!(matches!(
        dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter))),
        DialogEventResult::Close(DialogResult::Confirmed(Some(value))) if value.is_empty()
    ));
}

#[test]
fn input_dialog_direct_events_delete_whole_graphemes_and_accept_paste() {
    use reactive_tui::{
        event::types::{Event, KeyEvent, PasteEvent},
        widgets::dialog::{
            DialogComponent, DialogEventResult, DialogId, DialogResult, InputDialog,
            InputDialogOptions,
        },
    };
    let mut dialog = InputDialog::new(DialogId::from_u32(10), InputDialogOptions::default());
    for code in [KeyCode::Char('界'), KeyCode::Backspace] {
        dialog.handle_event(&Event::Key(KeyEvent::new(code)));
    }
    dialog.handle_event(&Event::Paste(PasteEvent::new("e\u{301}🙂".into())));
    dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Backspace)));
    let result = dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter)));
    assert!(
        matches!(result, DialogEventResult::Close(DialogResult::Confirmed(Some(value))) if value == "e\u{301}")
    );
}

#[test]
fn progress_dialog_public_and_builder_routes_show_progress_and_cancel_once() {
    use reactive_tui::{
        core::geometry::Rect,
        widgets::dialog::{
            DialogComponent, DialogId, DialogTheme, ProgressDialog, ProgressDialogOptions,
        },
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let output = calls.clone();
        let mut public = ProgressDialog::new(
            DialogId::from_u32(8),
            ProgressDialogOptions {
                title: "PROGRESS".into(),
                message: "COPYING".into(),
                on_cancel: Some(Arc::new(move || {
                    output.fetch_add(1, Ordering::SeqCst);
                })),
                ..Default::default()
            },
        );
        public.set_progress(0.5);
        for dialog in [
            public.render(Rect::default(), &DialogTheme::default()),
            builder::progress_dialog()
                .title("PROGRESS")
                .message("COPYING")
                .progress(0.5)
                .cancelable(true)
                .build(),
        ] {
            let frames = app_input::run_until_hidden(
                Control(dialog),
                size,
                vec![("50.0%", super::key(KeyCode::Enter))],
                "PROGRESS",
            );
            assert!(frames.iter().any(|frame| frame.text.contains("COPYING")));
            assert!(frames
                .iter()
                .all(|frame| !frame.text.contains("ProgressDialog")));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn progress_dialog_updates_value_cancellation_policy_and_callback_through_app() {
    use reactive_tui::{
        app::RootComponent,
        core::geometry::Rect,
        event::{router::EventResult, types::Event},
        widgets::dialog::{
            DialogComponent, DialogId, DialogTheme, ProgressDialog, ProgressDialogOptions,
        },
    };
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    struct Changed {
        changed: AtomicBool,
        calls: Arc<Mutex<Vec<bool>>>,
    }
    impl RootComponent for Changed {
        fn render(&self) -> super::Element {
            let changed = self.changed.load(Ordering::SeqCst);
            let calls = self.calls.clone();
            let mut dialog = ProgressDialog::new(
                DialogId::from_u32(8),
                ProgressDialogOptions {
                    title: "PROGRESS".into(),
                    message: if changed { "UPDATED" } else { "RUNNING" }.into(),
                    cancellable: changed,
                    show_time_remaining: true,
                    on_cancel: Some(Arc::new(move || calls.lock().unwrap().push(changed))),
                    ..Default::default()
                },
            );
            dialog.set_progress(if changed { 0.75 } else { 0.25 });
            dialog.render(Rect::default(), &DialogTheme::default())
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
                self.changed.store(true, Ordering::SeqCst);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let frames = app_input::run_until_hidden(
            Changed {
                changed: AtomicBool::new(false),
                calls: calls.clone(),
            },
            size,
            vec![
                ("25.0%", super::key(KeyCode::Escape)),
                ("RUNNING", super::key(KeyCode::F(2))),
                ("75.0%", super::key(KeyCode::Enter)),
            ],
            "PROGRESS",
        );
        assert_eq!(*calls.lock().unwrap(), [true]);
        assert!(frames.iter().any(|frame| frame.text.contains("Remaining:")));
    }
}

#[test]
fn confirmation_dialog_builder_paints_buttons_and_closes_after_activation() {
    for size in [(32, 12), (60, 20)] {
        let dialog = builder::confirmation_dialog()
            .title("CONFIRM")
            .message("Continue?")
            .confirm_text("PROCEED")
            .cancel_text("STOP")
            .danger(true)
            .class("bg-blue-700")
            .build();
        let frames = app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![("PROCEED", super::key(KeyCode::Enter))],
            "CONFIRM",
        );
        assert!(frames.iter().any(|frame| frame.text.contains("STOP")));
        assert!(frames
            .iter()
            .all(|frame| !frame.text.contains("ConfirmationDialog")));
    }
}

#[test]
fn toast_public_and_builder_routes_paint_then_expire_without_input() {
    use reactive_tui::{
        core::geometry::Rect,
        widgets::dialog::{DialogComponent, DialogId, DialogTheme, Toast, ToastOptions},
    };
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let output = calls.clone();
        let toast = Toast::new(
            DialogId::from_u32(9),
            ToastOptions {
                message: "SAVED".into(),
                duration: Some(Duration::from_millis(100)),
                on_close: Some(Arc::new(move || {
                    output.fetch_add(1, Ordering::SeqCst);
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        for element in [
            toast,
            builder::toast().success("SAVED").duration(100).build(),
        ] {
            let frames = app_input::run_until_hidden(
                Control(element),
                size,
                vec![("SAVED", super::key(KeyCode::F(2)))],
                "SAVED",
            );
            assert!(frames.iter().any(|frame| frame.text.contains("SAVED")));
            assert!(frames.iter().all(|frame| !frame.text.contains("Toast (")));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn toast_positions_use_measured_message_size_without_stealing_background_input() {
    for size in [(32, 12), (60, 20)] {
        for (position, x, y) in [
            ("top-left", 1, 1),
            ("top-center", (size.0 - 7) / 2 + 1, 1),
            ("top-right", size.0 - 6, 1),
            ("bottom-left", 1, size.1 - 2),
            ("bottom-center", (size.0 - 7) / 2 + 1, size.1 - 2),
            ("bottom-right", size.0 - 6, size.1 - 2),
        ] {
            let toast = builder::toast()
                .message("SAVED")
                .persistent()
                .closable(false)
                .position(position)
                .build();
            let frames = app_input::run_when(Control(toast), size, vec![("SAVED", None)]);
            assert_eq!(
                frames.last().unwrap().screen.cell(y, x).unwrap().contents(),
                "S",
                "{position}: {}",
                frames.last().unwrap().text
            );
        }
        let tree = builder::div()
            .class("relative w-full h-full")
            .child(builder::text_input().value("draft").build().auto_focus())
            .child(
                builder::toast()
                    .message("SAVED")
                    .persistent()
                    .closable(false)
                    .position("bottom-right")
                    .build(),
            )
            .build();
        let frames = app_input::run_when(
            Control(tree),
            size,
            vec![
                ("SAVED", super::key(KeyCode::End)),
                ("SAVED", super::key(KeyCode::Char('X'))),
                ("draftX", None),
            ],
        );
        assert!(frames.last().unwrap().text.contains("SAVED"));
    }
}

#[test]
fn dialog_builder_retains_editable_children_and_closes_through_app() {
    for size in [(32, 12), (60, 20)] {
        let dialog = builder::dialog()
            .title("DIALOG")
            .width(24)
            .height(8)
            .content(builder::text_input().value("draft").build().auto_focus())
            .class("bg-blue-700")
            .build();
        let frames = app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("draft", super::key(KeyCode::End)),
                ("draft", super::key(KeyCode::Char('X'))),
                ("draftX", super::key(KeyCode::Escape)),
            ],
            "DIALOG",
        );
        assert!(frames.iter().any(|frame| frame.text.contains("draftX")));
        assert!(frames.iter().all(|frame| !frame.text.contains("items")));
    }
}

#[test]
fn dialog_builder_modal_flag_controls_background_input_and_closable_false_keeps_it_open() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(32, 12), (60, 20)] {
        for modal in [false, true] {
            let calls = Arc::new(AtomicUsize::new(0));
            let output = calls.clone();
            let tree = builder::div()
                .class("relative w-full h-full")
                .child(
                    builder::button()
                        .text("BACK")
                        .class("absolute left-0 top-0 w-6 h-1 p-0")
                        .on_click(move || {
                            output.fetch_add(1, Ordering::SeqCst);
                        })
                        .build(),
                )
                .child(
                    builder::dialog()
                        .title("LOCKED")
                        .modal(modal)
                        .closable(false)
                        .width(18)
                        .height(6)
                        .content(super::Element::text("CONTENT"))
                        .build(),
                )
                .build();
            let frames = app_input::run_when(
                Control(tree),
                size,
                vec![
                    ("CONTENT", super::key(KeyCode::Escape)),
                    ("CONTENT", super::click(1, 0)),
                    ("CONTENT", None),
                ],
            );
            assert_eq!(calls.load(Ordering::SeqCst), usize::from(!modal));
            assert!(frames.last().unwrap().text.contains("LOCKED"));
        }
    }
}

#[test]
fn confirmation_dialog_buttons_skip_disabled_preserve_veto_and_deliver_close_result() {
    use reactive_tui::{
        core::geometry::Rect,
        widgets::dialog::{
            ConfirmationButton, ConfirmationButtons, ConfirmationDialog, ConfirmationDialogOptions,
            DialogComponent, DialogId, DialogResult, DialogTheme,
        },
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let closed = calls.clone();
        let mut blocked = ConfirmationButton::yes();
        blocked.id = "blocked".into();
        blocked.text = "BLOCKED".into();
        blocked.enabled = false;
        let mut wait = ConfirmationButton::retry();
        wait.id = "wait".into();
        wait.text = "WAIT".into();
        let options = ConfirmationDialogOptions {
            title: "CHOICE".into(),
            message: "Continue?".into(),
            default_button: Some("wait".into()),
            buttons: ConfirmationButtons::Custom(vec![blocked, wait, ConfirmationButton::ok()]),
            on_button_click: Some(Arc::new(move |id| {
                output.lock().unwrap().push(id.to_string());
                id != "wait"
            })),
            on_close: Some(Arc::new(move |result| {
                assert!(matches!(result, DialogResult::Confirmed(None)));
                closed.lock().unwrap().push("closed".into());
            })),
            ..Default::default()
        };
        let dialog = ConfirmationDialog::new(DialogId::from_u32(7), options)
            .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("WAIT", super::key(KeyCode::Enter)),
                ("WAIT", super::key(KeyCode::Tab)),
                ("WAIT", super::key(KeyCode::Enter)),
            ],
            "CHOICE",
        );
        assert_eq!(*calls.lock().unwrap(), ["wait", "ok", "closed"]);
    }
}
