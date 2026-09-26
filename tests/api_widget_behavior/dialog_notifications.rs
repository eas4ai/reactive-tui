use super::{app_input, Control};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    core::geometry::Rect,
    event::{
        router::EventResult,
        types::{Event, KeyCode, ResizeEvent},
    },
    widgets::dialog::{
        DialogComponent, DialogId, DialogTheme, ProgressDialog, ProgressDialogOptions, Toast,
        ToastOptions, ToastPosition, ToastType,
    },
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

#[test]
fn progress_dialog_cancels_once_from_its_resized_button() {
    for (initial, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let calls = Arc::new(Mutex::new(0));
        let output = calls.clone();
        let mut dialog = ProgressDialog::new(
            DialogId::from_u32(91),
            ProgressDialogOptions {
                title: "TRANSFER".into(),
                message: "COPYING".into(),
                on_cancel: Some(Arc::new(move || *output.lock().unwrap() += 1)),
                ..Default::default()
            },
        );
        dialog.set_progress(0.5);
        app_input::run_actions_until_hidden(
            Control(dialog.render(Rect::default(), &DialogTheme::default())),
            initial,
            vec![
                (
                    "COPYING",
                    app_input::Action::Event(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                ),
                ("Cancel", app_input::Action::ClickText("Cancel", 1)),
            ],
            "TRANSFER",
        );
        assert_eq!(*calls.lock().unwrap(), 1);
    }
}

#[test]
fn indeterminate_progress_dialog_animates_without_a_false_percentage() {
    for size in [(32, 12), (60, 20)] {
        let dialog = builder::progress_dialog()
            .title("TRANSFER")
            .message("WAITING")
            .indeterminate(true)
            .cancelable(false)
            .build();
        let frames = app_input::run_when_seen(Control(dialog), size, &["WAITING"], 6);
        let content: Vec<_> = frames
            .iter()
            .filter(|frame| frame.text.contains("WAITING"))
            .collect();
        assert!(content.len() >= 6);
        assert!(content
            .iter()
            .all(|frame| !frame.text.contains('%') && !frame.text.contains("Cancel")));
        assert!(
            content.windows(2).any(|pair| pair[0].text != pair[1].text),
            "indeterminate bar never moved"
        );
    }
}

#[test]
fn toast_pointer_close_tracks_resize_and_emits_one_callback() {
    use app_input::CellStep;
    for (initial, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let calls = Arc::new(Mutex::new(0));
        let output = calls.clone();
        let toast = Toast::new(
            DialogId::from_u32(92),
            ToastOptions {
                message: "NOTICE".into(),
                duration: None,
                position: ToastPosition::BottomRight,
                closable: true,
                on_close: Some(Arc::new(move || *output.lock().unwrap() += 1)),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_when_cell(
            Control(toast),
            initial,
            vec![
                CellStep {
                    x: initial.0 - 2,
                    y: initial.1 - 3,
                    content: "✕",
                    event: Some(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                },
                CellStep {
                    x: resized.0 - 2,
                    y: resized.1 - 3,
                    content: "✕",
                    event: super::click(resized.0 - 2, resized.1 - 3),
                },
                CellStep {
                    x: resized.0 - 2,
                    y: resized.1 - 3,
                    content: " ",
                    event: None,
                },
            ],
        );
        assert_eq!(*calls.lock().unwrap(), 1);
    }
}

#[test]
fn toast_updates_message_duration_and_callback_without_reconstructing_its_owner() {
    struct Changed {
        changed: AtomicBool,
        calls: Arc<Mutex<Vec<bool>>>,
    }
    impl RootComponent for Changed {
        fn render(&self) -> Element {
            let changed = self.changed.load(Ordering::SeqCst);
            let calls = self.calls.clone();
            Toast::new(
                DialogId::from_u32(93),
                ToastOptions {
                    message: if changed { "UPDATED" } else { "ORIGINAL" }.into(),
                    duration: changed.then_some(Duration::from_millis(100)),
                    toast_type: if changed {
                        ToastType::Error
                    } else {
                        ToastType::Success
                    },
                    closable: false,
                    on_close: Some(Arc::new(move || calls.lock().unwrap().push(changed))),
                    ..Default::default()
                },
            )
            .render(Rect::default(), &DialogTheme::default())
            .with_key("notice")
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
                ("ORIGINAL", super::key(KeyCode::F(2))),
                ("UPDATED", super::key(KeyCode::F(3))),
            ],
            "UPDATED",
        );
        assert!(frames.iter().any(|frame| frame.text.contains("UPDATED")));
        assert_eq!(*calls.lock().unwrap(), [true]);
    }
}
