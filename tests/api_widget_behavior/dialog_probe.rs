//! Dialog controls and observable results in the real reader fixture.
use reactive_tui::{
    builder,
    component::Element,
    core::geometry::{Rect, Size},
    widgets::dialog::*,
};
use std::sync::{Arc, Mutex};

pub(super) fn render(stage: usize, calls: &Arc<Mutex<Vec<String>>>) -> Element {
    let dialog = match stage {
        18 => confirmation(calls),
        19 => input(calls),
        20 => autocomplete(calls),
        21 => progress(calls),
        22 => wizard(calls),
        23 => toast(calls),
        24 => generic(),
        _ => unreachable!(),
    };
    builder::div()
        .class("w-full h-full")
        .children(vec![
            Element::text(calls.lock().unwrap().join("; ")),
            dialog,
        ])
        .build()
}

fn progress(calls: &Arc<Mutex<Vec<String>>>) -> Element {
    let cancelled = calls.clone();
    let mut dialog = ProgressDialog::new(
        DialogId::from_u32(21),
        ProgressDialogOptions {
            title: "Building project".into(),
            message: "Compiling sources".into(),
            on_cancel: Some(Arc::new(move || {
                cancelled.lock().unwrap().push("Progress cancelled".into())
            })),
            ..Default::default()
        },
    );
    dialog.set_progress(0.25);
    dialog.render(
        Rect {
            origin: Default::default(),
            size: Size::new(44, 10),
        },
        &DialogTheme::default(),
    )
}

fn wizard(calls: &Arc<Mutex<Vec<String>>>) -> Element {
    let completed = calls.clone();
    WizardDialog::new(
        DialogId::from_u32(22),
        WizardDialogOptions {
            title: "Project setup".into(),
            steps: vec![
                WizardStep {
                    id: "details".into(),
                    title: "Details".into(),
                    content: Element::text("Optional project details"),
                    can_skip: true,
                    validator: Some(Arc::new(|_| ValidationResult {
                        valid: false,
                        message: Some("Missing project details".into()),
                        ..Default::default()
                    })),
                },
                WizardStep {
                    id: "review".into(),
                    title: "Review".into(),
                    content: builder::text_input()
                        .value("ready")
                        .build()
                        .with_accessibility_label("Review notes"),
                    can_skip: false,
                    validator: None,
                },
            ],
            on_complete: Some(Arc::new(move |_| {
                completed.lock().unwrap().push("Wizard complete".into());
                true
            })),
            ..Default::default()
        },
    )
    .render(
        Rect {
            origin: Default::default(),
            size: Size::new(44, 12),
        },
        &DialogTheme::default(),
    )
}

fn toast(calls: &Arc<Mutex<Vec<String>>>) -> Element {
    let closed = calls.clone();
    let toast = Toast::new(
        DialogId::from_u32(23),
        ToastOptions {
            message: "Project saved".into(),
            toast_type: ToastType::Success,
            duration: Some(std::time::Duration::from_secs(3)),
            on_close: Some(Arc::new(move || {
                closed.lock().unwrap().push("Toast closed".into())
            })),
            ..Default::default()
        },
    )
    .render(Rect::default(), &DialogTheme::default());
    builder::div()
        .class("w-full h-full")
        .children(vec![
            builder::text_input()
                .value("draft")
                .build()
                .with_accessibility_label("Background draft")
                .auto_focus(),
            toast,
        ])
        .build()
}

fn generic() -> Element {
    builder::dialog()
        .title("Generic details")
        .width(44)
        .height(10)
        .content(
            builder::text_input()
                .value("draft")
                .build()
                .with_accessibility_label("Generic notes")
                .auto_focus(),
        )
        .build()
}

fn autocomplete(calls: &Arc<Mutex<Vec<String>>>) -> Element {
    let completed = calls.clone();
    AutocompleteDialog::new(
        DialogId::from_u32(20),
        AutocompleteDialogOptions {
            title: "Choose project".into(),
            prompt: "Search projects".into(),
            size: Some(Size::new(44, 12)),
            autocomplete: AutocompleteConfig {
                default_value: Some("a".into()),
                static_suggestions: vec!["Alpha".into(), "Alpine".into(), "Beta".into()],
                ..Default::default()
            },
            on_close: Some(Arc::new(move |result| {
                assert!(matches!(result, DialogResult::Selected(value) if value == "Alpine"));
                completed
                    .lock()
                    .unwrap()
                    .push("Autocomplete complete: Alpine".into());
            })),
            ..Default::default()
        },
    )
    .render(Rect::default(), &DialogTheme::default())
}

fn confirmation(calls: &Arc<Mutex<Vec<String>>>) -> Element {
    let completed = calls.clone();
    let mut disabled = ConfirmationButton::yes();
    disabled.text = "Unavailable action".into();
    disabled.enabled = false;
    ConfirmationDialog::new(
        DialogId::from_u32(18),
        ConfirmationDialogOptions {
            title: "Confirm project".into(),
            message: "Keep this project?".into(),
            size: Some(Size::new(44, 10)),
            buttons: ConfirmationButtons::Custom(vec![disabled, ConfirmationButton::ok()]),
            default_button: Some("ok".into()),
            on_close: Some(Arc::new(move |result| {
                assert!(matches!(result, DialogResult::Confirmed(None)));
                completed
                    .lock()
                    .unwrap()
                    .push("Confirmation complete".into());
            })),
            ..Default::default()
        },
    )
    .render(Rect::default(), &DialogTheme::default())
}

fn input(calls: &Arc<Mutex<Vec<String>>>) -> Element {
    let completed = calls.clone();
    InputDialog::new(
        DialogId::from_u32(19),
        InputDialogOptions {
            title: "Name project".into(),
            prompt: "Project name".into(),
            size: Some(Size::new(44, 12)),
            validation: Some(ValidationConfig {
                validate_on_change: true,
                debounce_delay: std::time::Duration::from_millis(100),
                custom_validator: Some(Arc::new(|value| ValidationResult {
                    valid: true,
                    message: None,
                    warnings: if value == "demo" {
                        vec!["Check spelling".into()]
                    } else {
                        Vec::new()
                    },
                })),
                ..Default::default()
            }),
            input: InputFieldConfig {
                required: true,
                attributes: [("aria-label".into(), "Project name entry".into())].into(),
                ..Default::default()
            },
            on_close: Some(Arc::new(move |result| {
                assert!(matches!(result, DialogResult::Confirmed(Some(value)) if value == "demo"));
                completed
                    .lock()
                    .unwrap()
                    .push("Input complete: demo".into());
            })),
            ..Default::default()
        },
    )
    .render(Rect::default(), &DialogTheme::default())
}
