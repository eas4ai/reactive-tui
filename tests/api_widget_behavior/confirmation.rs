use super::{app_input, Control};
use reactive_tui::{
    core::geometry::Rect,
    event::types::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::dialog::{
        ConfirmationButton, ConfirmationButtons, ConfirmationDialog, ConfirmationDialogOptions,
        DialogComponent, DialogId, DialogTheme,
    },
};
use std::sync::{Arc, Mutex};

#[test]
fn confirmation_updates_content_buttons_and_callbacks_before_resized_activation() {
    use reactive_tui::{
        app::RootComponent,
        component::Element,
        event::{router::EventResult, types::ResizeEvent},
        widgets::dialog::{ConfirmationIcon, DialogResult},
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Updating {
        changed: AtomicBool,
        actions: Arc<Mutex<Vec<(bool, String)>>>,
        results: Arc<Mutex<Vec<(bool, DialogResult)>>>,
    }
    impl RootComponent for Updating {
        fn render(&self) -> Element {
            let changed = self.changed.load(Ordering::SeqCst);
            let actions = self.actions.clone();
            let results = self.results.clone();
            let mut retry = ConfirmationButton::retry();
            retry.enabled = changed;
            ConfirmationDialog::new(
                DialogId::from_u32(96),
                ConfirmationDialogOptions {
                    title: if changed { "UPDATED" } else { "ORIGINAL" }.into(),
                    message: if changed { "READY" } else { "WAITING" }.into(),
                    description: Some(
                        if changed {
                            "NEW DETAILS"
                        } else {
                            "OLD DETAILS"
                        }
                        .into(),
                    ),
                    icon: Some(ConfirmationIcon::Custom(
                        if changed { "@" } else { "#" }.into(),
                    )),
                    buttons: ConfirmationButtons::Custom(vec![retry, ConfirmationButton::cancel()]),
                    on_button_click: Some(Arc::new(move |id| {
                        actions.lock().unwrap().push((changed, id.into()));
                        true
                    })),
                    on_close: Some(Arc::new(move |result| {
                        results.lock().unwrap().push((changed, result));
                    })),
                    ..Default::default()
                },
            )
            .render(Rect::default(), &DialogTheme::default())
            .with_key("confirmation")
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
    for (initial, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let actions = Arc::new(Mutex::new(Vec::new()));
        let results = Arc::new(Mutex::new(Vec::new()));
        let frames = app_input::run_actions_until_hidden(
            Updating {
                changed: AtomicBool::new(false),
                actions: actions.clone(),
                results: results.clone(),
            },
            initial,
            vec![
                ("OLD DETAILS", app_input::Action::ClickText("Retry", 1)),
                (
                    "# WAITING",
                    app_input::Action::Event(super::key(KeyCode::Char('r')).unwrap()),
                ),
                (
                    "OLD DETAILS",
                    app_input::Action::Event(super::key(KeyCode::F(2)).unwrap()),
                ),
                (
                    "NEW DETAILS",
                    app_input::Action::Event(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                ),
                ("@ READY", app_input::Action::ClickText("Retry", 1)),
            ],
            "UPDATED",
        );
        assert!(frames.iter().any(|frame| frame.text.contains("NEW DETAILS")
            && !frame.text.contains("OLD DETAILS")
            && !frame.text.contains("WAITING")));
        assert_eq!(*actions.lock().unwrap(), [(true, "retry".into())]);
        assert!(matches!(results.lock().unwrap().as_slice(),
            [(true, DialogResult::Confirmed(Some(value)))] if value == "retry"));
    }
}

#[test]
fn confirmation_disabling_escape_preserves_button_keyboard_navigation() {
    use reactive_tui::widgets::dialog::DialogResult;
    for size in [(32, 12), (60, 20)] {
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let dialog = ConfirmationDialog::new(
            DialogId::from_u32(79),
            ConfirmationDialogOptions {
                title: "KEEP OPEN".into(),
                message: "CHOOSE".into(),
                escape_closable: false,
                buttons: ConfirmationButtons::YesNo,
                on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
                ..Default::default()
            },
        );
        app_input::run_until_hidden(
            Control(dialog.render(Rect::default(), &DialogTheme::default())),
            size,
            vec![
                ("CHOOSE", super::key(KeyCode::Escape)),
                ("CHOOSE", super::key(KeyCode::Tab)),
                ("CHOOSE", super::key(KeyCode::Enter)),
            ],
            "KEEP OPEN",
        );
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Confirmed(Some(value))] if value == "no"),
            "results: {:?}",
            results.lock().unwrap()
        );
    }
}

#[test]
fn confirmation_custom_cancel_button_returns_cancellation_after_resized_click() {
    use reactive_tui::{
        event::types::ResizeEvent,
        widgets::dialog::{DialogEventResult, DialogResult},
    };
    for (initial, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let mut button = ConfirmationButton::cancel();
        button.id = "dismiss".into();
        button.text = "DISMISS".into();
        let mut dialog = ConfirmationDialog::new(
            DialogId::from_u32(78),
            ConfirmationDialogOptions {
                title: "CANCEL CHOICE".into(),
                message: "BODY".into(),
                buttons: ConfirmationButtons::Custom(vec![button]),
                on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
                ..Default::default()
            },
        );
        app_input::run_actions_until_hidden(
            Control(dialog.render(Rect::default(), &DialogTheme::default())),
            initial,
            vec![
                (
                    "BODY",
                    app_input::Action::Event(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                ),
                ("DISMISS", app_input::Action::ClickText("DISMISS", 1)),
            ],
            "CANCEL CHOICE",
        );
        assert!(matches!(
            results.lock().unwrap().as_slice(),
            [DialogResult::Cancelled]
        ));
        assert!(matches!(
            dialog.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter))),
            DialogEventResult::Close(DialogResult::Cancelled)
        ));
    }
}

#[test]
fn confirmation_shortcuts_ignore_release_and_command_modifiers() {
    for size in [(32, 12), (60, 20)] {
        for shortcut in [KeyCode::Char('a'), KeyCode::F(4)] {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let received = calls.clone();
            let mut button = ConfirmationButton::ok();
            button.shortcut = Some(shortcut.clone());
            let mut dialog = ConfirmationDialog::new(
                DialogId::from_u32(77),
                ConfirmationDialogOptions {
                    title: "SHORTCUT".into(),
                    message: "ACTION".into(),
                    buttons: ConfirmationButtons::Custom(vec![button]),
                    on_button_click: Some(Arc::new(move |id| {
                        received.lock().unwrap().push(id.to_owned());
                        false
                    })),
                    ..Default::default()
                },
            );
            let mut steps = vec![(
                "ACTION",
                Some(Event::Key(
                    KeyEvent::new(shortcut.clone()).with_kind(KeyEventKind::Release),
                )),
            )];
            for modifiers in [
                KeyModifiers {
                    ctrl: true,
                    ..KeyModifiers::empty()
                },
                KeyModifiers {
                    alt: true,
                    ..KeyModifiers::empty()
                },
                KeyModifiers {
                    meta: true,
                    ..KeyModifiers::empty()
                },
            ] {
                steps.push((
                    "ACTION",
                    Some(Event::Key(
                        KeyEvent::new(shortcut.clone()).with_modifiers(modifiers),
                    )),
                ));
            }
            steps.push(("ACTION", super::key(shortcut.clone())));
            let second = if shortcut == KeyCode::Char('a') {
                KeyCode::Char('A')
            } else {
                shortcut
            };
            steps.push(("ACTION", super::key(second)));
            steps.push(("ACTION", None));
            let events: Vec<_> = steps
                .iter()
                .filter_map(|(_, event)| event.clone())
                .collect();
            app_input::run_when(
                Control(dialog.render(Rect::default(), &DialogTheme::default())),
                size,
                steps,
            );
            assert_eq!(*calls.lock().unwrap(), ["ok", "ok"]);
            calls.lock().unwrap().clear();
            for event in events {
                dialog.handle_event(&event);
            }
            assert_eq!(*calls.lock().unwrap(), ["ok", "ok"]);
        }
    }
}
