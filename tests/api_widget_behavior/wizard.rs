use super::{app_input, Control};
use reactive_tui::{
    component::Element,
    core::geometry::Rect,
    event::types::{Event, KeyCode, ResizeEvent},
    widgets::dialog::{
        DialogComponent, DialogId, DialogTheme, ValidationResult, WizardDialog,
        WizardDialogOptions, WizardStep,
    },
};
use std::sync::{Arc, Mutex};

fn step(id: &str) -> WizardStep {
    WizardStep {
        id: id.into(),
        title: id.into(),
        content: Element::text(format!("CONTENT-{id}")),
        can_skip: false,
        validator: None,
    }
}
fn dialog(options: WizardDialogOptions) -> Element {
    WizardDialog::new(DialogId::from_u32(71), options)
        .render(Rect::default(), &DialogTheme::default())
}

#[test]
fn wizard_navigates_back_and_finishes_once_after_resize() {
    use app_input::Action;
    for (size, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let calls = Arc::new(Mutex::new(0));
        let completed = calls.clone();
        let element = dialog(WizardDialogOptions {
            title: "WIZARD".into(),
            steps: vec![step("FIRST"), step("SECOND")],
            on_complete: Some(Arc::new(move |_| {
                *completed.lock().unwrap() += 1;
                true
            })),
            ..Default::default()
        });
        app_input::run_actions_until_hidden(
            Control(element),
            size,
            vec![
                ("CONTENT-FIRST", Action::ClickText("Next", 1)),
                (
                    "CONTENT-SECOND",
                    Action::Event(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                ),
                ("Back", Action::ClickText("Back", 1)),
                ("CONTENT-FIRST", Action::ClickText("Next", 1)),
                ("CONTENT-SECOND", Action::ClickText("Finish", 1)),
            ],
            "WIZARD",
        );
        assert_eq!(*calls.lock().unwrap(), 1);
    }
}

#[test]
fn wizard_validation_blocks_next_but_optional_skip_advances() {
    use app_input::Action;
    for size in [(32, 12), (60, 20)] {
        let mut first = step("FIRST");
        first.can_skip = true;
        first.validator = Some(Arc::new(|_| ValidationResult {
            valid: false,
            message: Some("Missing name".into()),
            ..Default::default()
        }));
        let element = dialog(WizardDialogOptions {
            title: "WIZARD".into(),
            steps: vec![first, step("SECOND")],
            ..Default::default()
        });
        let frames = app_input::run_actions_until_hidden(
            Control(element),
            size,
            vec![
                ("CONTENT-FIRST", Action::ClickText("Next", 1)),
                ("Missing name", Action::ClickText("Skip", 1)),
                ("CONTENT-SECOND", Action::ClickText("Finish", 1)),
            ],
            "WIZARD",
        );
        let last = &frames.last().unwrap().text;
        assert!(
            !last.contains("WIZARD") && !last.contains("CONTENT-SECOND"),
            "finishing after a skip closes the wizard at {size:?}:\n{last}"
        );
    }
}

#[test]
fn wizard_veto_keeps_final_step_open_and_cancel_delivers_once() {
    use app_input::Action;
    for size in [(32, 12), (60, 20)] {
        let completed = Arc::new(Mutex::new(0));
        let cancelled = Arc::new(Mutex::new(0));
        let finish = completed.clone();
        let cancel = cancelled.clone();
        let element = dialog(WizardDialogOptions {
            title: "WIZARD".into(),
            steps: vec![step("ONLY")],
            on_complete: Some(Arc::new(move |_| {
                *finish.lock().unwrap() += 1;
                false
            })),
            on_cancel: Some(Arc::new(move || {
                *cancel.lock().unwrap() += 1;
            })),
            ..Default::default()
        });
        app_input::run_actions_until_hidden(
            Control(element),
            size,
            vec![
                ("CONTENT-ONLY", Action::ClickText("Finish", 1)),
                (
                    "CONTENT-ONLY",
                    Action::Event(app_input::key(KeyCode::Escape).unwrap()),
                ),
            ],
            "WIZARD",
        );
        assert_eq!(*completed.lock().unwrap(), 1);
        assert_eq!(*cancelled.lock().unwrap(), 1);
    }
}

#[test]
fn empty_wizard_displays_error_and_can_cancel() {
    for size in [(32, 12), (60, 20)] {
        let element = dialog(WizardDialogOptions {
            title: "WIZARD".into(),
            ..Default::default()
        });
        let frames = app_input::run_until_hidden(
            Control(element),
            size,
            vec![("Wizard has no steps", app_input::key(KeyCode::Escape))],
            "WIZARD",
        );
        let last = &frames.last().unwrap().text;
        assert!(
            !last.contains("WIZARD") && !last.contains("Wizard has no steps"),
            "escape closes the empty wizard at {size:?}:\n{last}"
        );
    }
}

#[test]
fn wizard_keyboard_advances_with_back_disabled() {
    for size in [(32, 12), (60, 20)] {
        let element = dialog(WizardDialogOptions {
            title: "WIZARD".into(),
            steps: vec![step("FIRST"), step("SECOND")],
            allow_back: false,
            ..Default::default()
        });
        let frames = app_input::run_until_hidden(
            Control(element),
            size,
            vec![
                ("CONTENT-FIRST", app_input::key(KeyCode::Enter)),
                ("CONTENT-SECOND", app_input::key(KeyCode::Left)),
                ("CONTENT-SECOND", app_input::key(KeyCode::Enter)),
            ],
            "WIZARD",
        );
        assert!(!frames.iter().any(|frame| frame.text.contains("Back")));
    }
}

#[test]
fn wizard_builder_enforces_can_proceed_and_initial_step() {
    use app_input::Action;
    use reactive_tui::builder::dialog_builders::{WizardBuilder, WizardStep as BuilderStep};
    for size in [(32, 12), (60, 20)] {
        let element = WizardBuilder::new()
            .title("BUILDER")
            .steps(vec![
                BuilderStep::new("FIRST"),
                BuilderStep::new("LOCKED").can_proceed(false),
            ])
            .current_step(1)
            .build();
        let frames = app_input::run_actions_until_hidden(
            Control(element),
            size,
            vec![
                ("LOCKED", Action::ClickText("Finish", 1)),
                ("Complete this step", Action::ClickText("Cancel", 1)),
            ],
            "BUILDER",
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("Step 2 of 2")));
        assert!(!frames.iter().any(|frame| frame.text.contains("FIRST")));
    }
}

#[test]
fn wizard_builder_hides_progress_and_disables_cancel_without_disabling_finish() {
    use app_input::Action;
    use reactive_tui::builder::dialog_builders::{WizardBuilder, WizardStep as BuilderStep};
    use unicode_width::UnicodeWidthStr;
    for size in [(32, 12), (60, 20)] {
        let element = WizardBuilder::new()
            .title("BUILDER")
            .step(BuilderStep::new("ONLY"))
            .show_progress(false)
            .cancelable(false)
            .class("bg-#123456 reduced-motion")
            .build();
        let frames = app_input::run_actions_until_hidden(
            Control(element),
            size,
            vec![
                (
                    "Finish",
                    Action::Event(app_input::key(KeyCode::Escape).unwrap()),
                ),
                ("Finish", Action::ClickText("Finish", 1)),
            ],
            "BUILDER",
        );
        assert!(!frames
            .iter()
            .any(|frame| frame.text.contains("Step 1 of") || frame.text.contains("Cancel")));
        let frame = frames
            .iter()
            .find(|frame| frame.text.contains("ONLY"))
            .unwrap();
        let (row, column) = frame
            .text
            .lines()
            .enumerate()
            .find_map(|(row, line)| {
                line.find("ONLY")
                    .map(|column| (row, line[..column].width()))
            })
            .unwrap();
        assert_eq!(
            frame
                .screen
                .cell(row as u16, column as u16)
                .unwrap()
                .bgcolor(),
            vt100::Color::Rgb(18, 52, 86),
            "{}",
            frame.text
        );
    }
}

#[test]
fn noncancelable_wizard_keeps_keyboard_finish_available() {
    use reactive_tui::builder::dialog_builders::{WizardBuilder, WizardStep as BuilderStep};
    for size in [(32, 12), (60, 20)] {
        let element = WizardBuilder::new()
            .title("KEYBOARD WIZARD")
            .step(BuilderStep::new("ONLY"))
            .cancelable(false)
            .build();
        let frames = app_input::run_until_hidden(
            Control(element),
            size,
            vec![
                ("ONLY", app_input::key(KeyCode::Escape)),
                ("ONLY", app_input::key(KeyCode::Enter)),
            ],
            "KEYBOARD WIZARD",
        );
        let last = &frames.last().unwrap().text;
        assert!(
            !last.contains("KEYBOARD WIZARD") && !last.contains("ONLY"),
            "enter finishes the non-cancelable wizard at {size:?}:\n{last}"
        );
    }
}

#[test]
fn wizard_removed_step_discards_its_edited_child_before_reinsertion() {
    use app_input::Action;
    use reactive_tui::{app::RootComponent, event::router::EventResult};
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Changing(AtomicUsize);
    impl RootComponent for Changing {
        fn render(&self) -> Element {
            let phase = self.0.load(Ordering::SeqCst);
            let mut first = step("FIRST");
            first.content = reactive_tui::builder::text_input().value("seed").build();
            dialog(WizardDialogOptions {
                title: ["ORIGINAL", "REMOVED", "RESTORED"][phase].into(),
                steps: if phase == 1 {
                    vec![step("SECOND")]
                } else {
                    vec![first, step("SECOND")]
                },
                ..Default::default()
            })
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
    for size in [(32, 12), (60, 20)] {
        let frames = app_input::run_actions_until_hidden(
            Changing(AtomicUsize::new(0)),
            size,
            vec![
                ("seed", Action::ClickText("seed", 1)),
                ("seed", Action::Event(app_input::key(KeyCode::End).unwrap())),
                (
                    "seed",
                    Action::Event(app_input::key(KeyCode::Char('X')).unwrap()),
                ),
                ("seedX", Action::ClickText("Next", 1)),
                (
                    "CONTENT-SECOND",
                    Action::Event(app_input::key(KeyCode::F(2)).unwrap()),
                ),
                (
                    "REMOVED",
                    Action::Event(app_input::key(KeyCode::F(3)).unwrap()),
                ),
                ("RESTORED", Action::ClickText("Back", 1)),
                ("seed", Action::ClickText("Cancel", 1)),
            ],
            "RESTORED",
        );
        assert!(frames.iter().any(|frame| frame.text.contains("RESTORED")
            && frame.text.contains("seed")
            && !frame.text.contains("seedX")));
        assert!(!frames
            .iter()
            .any(|frame| frame.text.contains("REMOVED") && frame.text.contains("Back")));
    }
}

#[test]
fn wizard_retains_edited_child_on_back() {
    use app_input::Action;
    for size in [(32, 12), (60, 20)] {
        let mut first = step("FIRST");
        first.content = reactive_tui::builder::text_input().value("seed").build();
        let element = dialog(WizardDialogOptions {
            title: "WIZARD".into(),
            steps: vec![first, step("SECOND")],
            ..Default::default()
        });
        let frames = app_input::run_actions_until_hidden(
            Control(element),
            size,
            vec![
                ("seed", Action::ClickText("seed", 1)),
                ("seed", Action::Event(app_input::key(KeyCode::End).unwrap())),
                (
                    "seed",
                    Action::Event(app_input::key(KeyCode::Char('X')).unwrap()),
                ),
                ("seedX", Action::ClickText("Next", 1)),
                ("CONTENT-SECOND", Action::ClickText("Back", 1)),
                ("seedX", Action::ClickText("Cancel", 1)),
            ],
            "WIZARD",
        );
        let last = &frames.last().unwrap().text;
        assert!(
            !last.contains("WIZARD") && !last.contains("seedX"),
            "cancel closes the wizard with its edited child at {size:?}:\n{last}"
        );
    }
}

#[test]
fn wizard_app_supplies_authored_data_to_validation_and_completion() {
    use app_input::Action;
    use std::collections::HashMap;
    for size in [(32, 12), (60, 20)] {
        let completed = Arc::new(Mutex::new(None));
        let called = completed.clone();
        let mut only = step("ONLY");
        only.validator = Some(Arc::new(|data| ValidationResult {
            valid: data.get("name").is_some_and(|value| value == "Ada"),
            ..Default::default()
        }));
        let mut wizard = WizardDialog::new(
            DialogId::from_u32(71),
            WizardDialogOptions {
                title: "WIZARD".into(),
                steps: vec![only],
                on_complete: Some(Arc::new(move |data| {
                    *called.lock().unwrap() = Some(data.clone());
                    true
                })),
                ..Default::default()
            },
        );
        let data = HashMap::from([("name".into(), "Ada".into())]);
        wizard.set_data(data.clone());
        app_input::run_actions_until_hidden(
            Control(wizard.render(Rect::default(), &DialogTheme::default())),
            size,
            vec![("Finish", Action::ClickText("Finish", 1))],
            "WIZARD",
        );
        assert_eq!(*completed.lock().unwrap(), Some(data));
    }
}

#[test]
fn wizard_reorder_retains_active_id_and_uses_updated_data_and_callback() {
    use app_input::Action;
    use reactive_tui::{app::RootComponent, event::router::EventResult};
    use std::{
        collections::HashMap,
        sync::atomic::{AtomicBool, Ordering},
    };
    struct Updating {
        changed: AtomicBool,
        results: Arc<Mutex<Vec<(bool, String)>>>,
    }
    impl RootComponent for Updating {
        fn render(&self) -> Element {
            let changed = self.changed.load(Ordering::SeqCst);
            let results = self.results.clone();
            let mut wizard = WizardDialog::new(
                DialogId::from_u32(71),
                WizardDialogOptions {
                    title: if changed { "UPDATED" } else { "ORIGINAL" }.into(),
                    steps: if changed {
                        vec![step("SECOND"), step("FIRST")]
                    } else {
                        vec![step("FIRST"), step("SECOND")]
                    },
                    on_complete: Some(Arc::new(move |data| {
                        results
                            .lock()
                            .unwrap()
                            .push((changed, data["name"].clone()));
                        true
                    })),
                    ..Default::default()
                },
            );
            wizard.set_data(HashMap::from([(
                "name".into(),
                if changed { "Ada" } else { "old" }.into(),
            )]));
            wizard.render(Rect::default(), &DialogTheme::default())
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
        let results = Arc::new(Mutex::new(Vec::new()));
        let frames = app_input::run_actions_until_hidden(
            Updating {
                changed: AtomicBool::new(false),
                results: results.clone(),
            },
            size,
            vec![
                ("CONTENT-FIRST", Action::ClickText("Next", 1)),
                (
                    "CONTENT-SECOND",
                    Action::Event(app_input::key(KeyCode::F(2)).unwrap()),
                ),
                ("UPDATED", Action::ClickText("Next", 1)),
                ("CONTENT-FIRST", Action::ClickText("Finish", 1)),
            ],
            "UPDATED",
        );
        assert!(frames.iter().any(|frame| frame.text.contains("UPDATED")
            && frame.text.contains("CONTENT-SECOND")
            && frame.text.contains("Step 1 of 2")));
        assert_eq!(*results.lock().unwrap(), [(true, "Ada".into())]);
    }
}
