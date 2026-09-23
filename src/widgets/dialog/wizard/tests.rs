use super::*;

fn step(id: &str, valid: bool) -> WizardStep {
    WizardStep {
        id: id.into(),
        title: id.into(),
        content: Element::text(id),
        can_skip: false,
        validator: Some(Arc::new(move |_| ValidationResult {
            valid,
            ..Default::default()
        })),
    }
}

#[test]
fn empty_wizard_rejects_navigation_and_completion_without_panicking() {
    let mut dialog = WizardDialog::new(DialogId::from_u32(1), Default::default());
    assert!(!dialog.next_step());
    assert!(!dialog.validate().valid);
    assert!(matches!(
        dialog.finish_wizard(),
        DialogEventResult::NotHandled
    ));
}

#[test]
fn wizard_validates_before_navigation_and_completion() {
    let mut dialog = WizardDialog::new(
        DialogId::from_u32(1),
        WizardDialogOptions {
            steps: vec![step("first", false), step("second", true)],
            ..Default::default()
        },
    );
    assert!(!dialog.next_step());
    assert_eq!(dialog.current_step, 0);
    assert!(!dialog.validate().valid);
    assert!(matches!(
        dialog.finish_wizard(),
        DialogEventResult::NotHandled
    ));
    dialog.options.steps[0].validator = None;
    assert!(dialog.next_step());
    assert!(matches!(
        dialog.finish_wizard(),
        DialogEventResult::Close(_)
    ));
}

#[test]
fn wizard_rejects_ambiguous_step_identifiers() {
    for steps in [
        vec![step("", true)],
        vec![step("same", true), step("same", true)],
    ] {
        let dialog = WizardDialog::new(
            DialogId::from_u32(1),
            WizardDialogOptions {
                steps,
                ..Default::default()
            },
        );
        assert!(!dialog.validate().valid);
    }
}

#[test]
fn wizard_data_reaches_validator_and_completion_and_skip_is_optional() {
    let received = Arc::new(std::sync::Mutex::new(None));
    let completed = received.clone();
    let mut first = step("first", false);
    first.validator = Some(Arc::new(|data| ValidationResult {
        valid: data.get("name").is_some_and(|name| name == "Ada"),
        ..Default::default()
    }));
    let mut dialog = WizardDialog::new(
        DialogId::from_u32(1),
        WizardDialogOptions {
            steps: vec![first, step("second", true)],
            on_complete: Some(Arc::new(move |data| {
                *completed.lock().unwrap() = Some(data.clone());
                true
            })),
            ..Default::default()
        },
    );
    assert!(!dialog.skip_step());
    assert!(!dialog.next_step());
    let data = HashMap::from([("name".into(), "Ada".into())]);
    dialog.set_data(data.clone());
    assert_eq!(dialog.data(), &data);
    assert!(dialog.next_step());
    assert!(matches!(
        dialog.finish_wizard(),
        DialogEventResult::Close(_)
    ));
    assert_eq!(*received.lock().unwrap(), Some(data));
    assert!(dialog.previous_step());
    dialog.set_data(HashMap::new());
    dialog.options.steps[0].can_skip = true;
    assert!(dialog.skip_step());
}
