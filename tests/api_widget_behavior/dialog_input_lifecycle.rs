use super::app_input;
use reactive_tui::{
    app::{RootComponent, RootUpdate},
    component::Element,
    core::geometry::Rect,
    event::{
        router::EventResult,
        types::{Event, KeyCode},
    },
    widgets::dialog::{
        DialogComponent, DialogId, DialogTheme, InputDialog, InputDialogOptions, ValidationConfig,
        ValidationResult,
    },
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq)]
enum Change {
    None,
    Immediate,
    Disable,
    Remove,
}

struct PendingInput {
    change: Change,
    changed: AtomicBool,
    changed_at: Mutex<Option<Instant>>,
    quiet: bool,
    validations: Arc<Mutex<Vec<String>>>,
}

impl RootComponent for PendingInput {
    fn render(&self) -> Element {
        let changed = self.changed.load(Ordering::SeqCst);
        if changed && self.change == Change::Remove {
            return Element::text(if self.quiet { "QUIET" } else { "REMOVED" });
        }
        let calls = self.validations.clone();
        InputDialog::new(
            DialogId::from_u32(24),
            InputDialogOptions {
                title: "DEBOUNCE".into(),
                prompt: if self.quiet { "QUIET" } else { "VALUE" }.into(),
                validation: Some(ValidationConfig {
                    validate_on_change: !(changed && self.change == Change::Disable),
                    validate_on_blur: false,
                    debounce_delay: if self.change == Change::Immediate {
                        if changed {
                            Duration::ZERO
                        } else {
                            Duration::from_secs(30)
                        }
                    } else {
                        Duration::from_secs(1)
                    },
                    custom_validator: Some(Arc::new(move |value| {
                        calls.lock().unwrap().push(value.into());
                        ValidationResult {
                            valid: true,
                            message: None,
                            warnings: vec!["VALIDATED".into()],
                        }
                    })),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default())
    }

    fn handle_event(&self, event: &Event) -> EventResult {
        if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
            self.changed.store(true, Ordering::SeqCst);
            *self.changed_at.lock().unwrap() = Some(Instant::now());
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }

    fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
        if !self.quiet
            && self
                .changed_at
                .lock()
                .unwrap()
                // The behavior under test: 1.2 s after the last change, past the
                // 1 s validation delay.
                .is_some_and(|at| at.elapsed() >= Duration::from_millis(1200))
        {
            self.quiet = true;
            Ok(RootUpdate::Redraw)
        } else {
            Ok(RootUpdate::Unchanged)
        }
    }
}

fn pending(change: Change, validations: Arc<Mutex<Vec<String>>>) -> PendingInput {
    PendingInput {
        change,
        changed: AtomicBool::new(false),
        changed_at: Mutex::new(None),
        quiet: false,
        validations,
    }
}

#[test]
fn input_dialog_debounce_coalesces_edits_and_reschedules_when_delay_changes() {
    for size in [(32, 12), (60, 20)] {
        for change in [Change::None, Change::Immediate] {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let mut steps = vec![
                ("VALUE", super::key(KeyCode::Char('a'))),
                ("VALUE", super::key(KeyCode::Char('b'))),
                ("VALUE", super::key(KeyCode::Char('c'))),
            ];
            if change == Change::Immediate {
                steps.push(("abc", super::key(KeyCode::F(2))));
            }
            steps.push(("VALIDATED", super::key(KeyCode::Escape)));
            app_input::run_until_hidden(pending(change, calls.clone()), size, steps, "DEBOUNCE");
            assert_eq!(*calls.lock().unwrap(), ["abc"]);
        }
    }
}

#[test]
fn input_dialog_disabling_change_validation_or_removing_cancels_the_pending_debounce() {
    for size in [(32, 12), (60, 20)] {
        for change in [Change::Disable, Change::Remove] {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let mut steps = vec![
                ("VALUE", super::key(KeyCode::Char('a'))),
                ("VALUE", super::key(KeyCode::F(2))),
            ];
            if change == Change::Remove {
                steps.push(("QUIET", None));
                app_input::run_when(pending(change, calls.clone()), size, steps);
            } else {
                steps.push(("QUIET", super::key(KeyCode::Escape)));
                app_input::run_until_hidden(
                    pending(change, calls.clone()),
                    size,
                    steps,
                    "DEBOUNCE",
                );
            }
            assert!(calls.lock().unwrap().is_empty(), "cancelled validation ran");
        }
    }
}
