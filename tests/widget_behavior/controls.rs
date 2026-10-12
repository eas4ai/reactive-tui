//! CTL-005: the screen reader's actions beyond Focus and Click.
//!
//! Each test gives the App an in-process screen reader, a
//! `ReaderChannel`: the App publishes its accessibility tree to it and
//! takes its requests as it takes a screen reader's over AT-SPI. The
//! screen-reader integration runs on Linux only, and so do these tests.
#![cfg(target_os = "linux")]

use std::sync::{Arc, Mutex};

use crate::common::app_input::{self, FramePredicate, Input, ReaderRequest, Snapshot};
use accesskit::{Action, ActionData};
use reactive_tui::{
    accessibility::{ReaderChannel, Role},
    app::RootComponent,
    builder,
    component::Element,
};

/// The values a callback was called with, in order.
type Log<T> = Arc<Mutex<Vec<T>>>;

/// A closure that records each value it is called with in `log`.
fn recorder<T: Send + 'static>(log: &Log<T>) -> impl Fn(T) + Send + Sync + 'static {
    let log = log.clone();
    move |value| log.lock().unwrap().push(value)
}

/// What a log holds now.
fn held<T: Clone>(log: &Log<T>) -> Vec<T> {
    log.lock().unwrap().clone()
}

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// The frame once `needle` is painted.
fn shown(needle: &'static str) -> FramePredicate {
    Box::new(move |frame: &Snapshot| frame.text.contains(needle))
}

/// The screen reader's `action`, with `data`, on the node with `role`
/// named `label`.
fn request(
    role: Role,
    label: &'static str,
    action: Action,
    data: Option<ActionData>,
) -> Option<Input> {
    Some(Input::Request(ReaderRequest {
        role,
        label: Some(label),
        action,
        data,
    }))
}

/// A request's string value, as AT-SPI's SetTextContents sends it.
fn text(value: &str) -> Option<ActionData> {
    Some(ActionData::Value(value.into()))
}

/// A request's numeric value, as AT-SPI's SetCurrentValue sends it.
fn number(value: f64) -> Option<ActionData> {
    Some(ActionData::NumericValue(value))
}

/// Whether the node with `role` named `label`, in the tree the App
/// published last, advertises `action`.
fn advertises(reader: &ReaderChannel, role: Role, label: &str, action: Action) -> bool {
    let tree = reader.tree().expect("the App published a tree");
    tree.nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no {role:?} node named {label:?} in the published tree"))
        .1
        .supports_action(action)
}

/// The last frame's text.
fn last(frames: &[Snapshot]) -> &str {
    frames.last().map_or("", |frame| frame.text.as_str())
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_set_value_request_replaces_a_text_inputs_text_and_reports_it() {
    let log: Log<String> = Arc::default();
    let reader = ReaderChannel::new();
    let control = builder::text_input()
        .placeholder("Name")
        .value("xyz")
        .on_change(recorder(&log))
        .build();
    let frames = app_input::run_when_frame_reading(
        Root(control),
        (80, 6),
        &reader,
        vec![
            (
                shown("xyz"),
                request(Role::TextInput, "Name", Action::SetValue, text("abc")),
            ),
            (shown("abc"), None),
        ],
    );
    assert!(
        advertises(&reader, Role::TextInput, "Name", Action::SetValue),
        "CTL-005: the text input's published node advertises SetValue"
    );
    assert!(
        !last(&frames).contains("xyz"),
        "CTL-005: SetValue `abc` replaces the whole text, not part of it: {}",
        last(&frames)
    );
    assert_eq!(
        held(&log),
        vec!["abc".to_owned()],
        "CTL-005: SetValue `abc` reports the new text to on_change"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_set_value_request_on_a_numeric_input_keeps_what_typing_takes() {
    let log: Log<String> = Arc::default();
    let reader = ReaderChannel::new();
    let control = builder::text_input()
        .input_type("number")
        .placeholder("Amount")
        .value("7")
        .on_change(recorder(&log))
        .build();
    app_input::run_when_frame_reading(
        Root(control),
        (80, 6),
        &reader,
        vec![
            (
                shown("7"),
                request(
                    Role::NumberInput,
                    "Amount",
                    Action::SetValue,
                    text("1a2.5-x"),
                ),
            ),
            (shown("12.5-"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec!["12.5-".to_owned()],
        "CTL-005: a numeric field keeps the digits, `.` and `-` of the request, as typing does"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_an_increment_request_moves_a_slider_one_step_up() {
    let log: Log<f64> = Arc::default();
    let reader = ReaderChannel::new();
    let control = builder::slider()
        .label("Volume")
        .value(50.0)
        .step(10.0)
        .on_change(recorder(&log))
        .build();
    app_input::run_when_frame_reading(
        Root(control),
        (80, 6),
        &reader,
        vec![
            (
                shown("50.0"),
                request(Role::Slider, "Volume", Action::Increment, None),
            ),
            (shown("60.0"), None),
        ],
    );
    for action in [Action::Increment, Action::Decrement, Action::SetValue] {
        assert!(
            advertises(&reader, Role::Slider, "Volume", action),
            "CTL-005: the slider's published node advertises {action:?}"
        );
    }
    assert_eq!(
        held(&log),
        vec![60.0],
        "CTL-005: Increment moves the slider at 50 one step of 10 up and reports 60"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_decrement_request_moves_a_slider_one_step_down() {
    let log: Log<f64> = Arc::default();
    let reader = ReaderChannel::new();
    let control = builder::slider()
        .label("Volume")
        .value(50.0)
        .step(10.0)
        .on_change(recorder(&log))
        .build();
    app_input::run_when_frame_reading(
        Root(control),
        (80, 6),
        &reader,
        vec![
            (
                shown("50.0"),
                request(Role::Slider, "Volume", Action::Decrement, None),
            ),
            (shown("40.0"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec![40.0],
        "CTL-005: Decrement moves the slider at 50 one step of 10 down and reports 40"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_set_value_request_above_a_sliders_maximum_ends_at_the_maximum() {
    let log: Log<f64> = Arc::default();
    let reader = ReaderChannel::new();
    let control = builder::slider()
        .label("Volume")
        .value(50.0)
        .max(100.0)
        .step(10.0)
        .on_change(recorder(&log))
        .build();
    app_input::run_when_frame_reading(
        Root(control),
        (80, 6),
        &reader,
        vec![
            (
                shown("50.0"),
                request(Role::Slider, "Volume", Action::SetValue, number(1000.0)),
            ),
            (shown("100.0"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec![100.0],
        "CTL-005: SetValue 1000 on a slider whose maximum is 100 ends at 100"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_set_value_request_snaps_a_slider_to_its_step() {
    let log: Log<f64> = Arc::default();
    let reader = ReaderChannel::new();
    let control = builder::slider()
        .label("Volume")
        .value(50.0)
        .step(10.0)
        .on_change(recorder(&log))
        .build();
    app_input::run_when_frame_reading(
        Root(control),
        (80, 6),
        &reader,
        vec![
            (
                shown("50.0"),
                request(Role::Slider, "Volume", Action::SetValue, number(37.0)),
            ),
            (shown("40.0"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec![40.0],
        "CTL-005: SetValue 37 on a slider with a step of 10 snaps to 40"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_set_value_request_chooses_a_select_option_by_its_value() {
    let log: Log<String> = Arc::default();
    let reader = ReaderChannel::new();
    let control = builder::select()
        .placeholder("Fruit")
        .option("b", "Banana")
        .option("c", "Cherry")
        .selected("b")
        .on_change(recorder(&log))
        .build();
    app_input::run_when_frame_reading(
        Root(control),
        (80, 10),
        &reader,
        vec![
            (
                shown("[Banana"),
                request(Role::ComboBox, "Fruit", Action::SetValue, text("c")),
            ),
            (shown("[Cherry"), None),
        ],
    );
    assert!(
        advertises(&reader, Role::ComboBox, "Fruit", Action::SetValue),
        "CTL-005: the select's published node advertises SetValue"
    );
    assert_eq!(
        held(&log),
        vec!["c".to_owned()],
        "CTL-005: SetValue `c` chooses the option whose value is `c`"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_set_value_request_chooses_a_select_option_by_its_label() {
    let log: Log<String> = Arc::default();
    let reader = ReaderChannel::new();
    let control = builder::select()
        .placeholder("Fruit")
        .option("b", "Banana")
        .option("c", "Cherry")
        .selected("b")
        .on_change(recorder(&log))
        .build();
    app_input::run_when_frame_reading(
        Root(control),
        (80, 10),
        &reader,
        vec![
            (
                shown("[Banana"),
                request(Role::ComboBox, "Fruit", Action::SetValue, text("Cherry")),
            ),
            (shown("[Cherry"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec!["c".to_owned()],
        "CTL-005: SetValue `Cherry` chooses the option whose label is `Cherry`"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_set_value_request_on_a_disabled_text_input_changes_nothing() {
    let log: Log<String> = Arc::default();
    let reader = ReaderChannel::new();
    // The second field is the witness: its change shows that the App has
    // taken both requests, the disabled field's first.
    let controls = builder::div()
        .class("flex-col")
        .child(
            builder::text_input()
                .placeholder("Locked")
                .value("keep")
                .disabled(true)
                .on_change(recorder(&log))
                .build(),
        )
        .child(builder::text_input().placeholder("Open").build())
        .build();
    let frames = app_input::run_when_frame_reading(
        Root(controls),
        (80, 8),
        &reader,
        vec![
            (
                shown("keep"),
                request(Role::TextInput, "Locked", Action::SetValue, text("changed")),
            ),
            (
                shown("keep"),
                request(Role::TextInput, "Open", Action::SetValue, text("done")),
            ),
            (shown("done"), None),
        ],
    );
    assert!(
        last(&frames).contains("keep") && !last(&frames).contains("changed"),
        "CTL-005: a SetValue request on a disabled text input leaves its text: {}",
        last(&frames)
    );
    assert!(
        held(&log).is_empty(),
        "CTL-005: a disabled text input reports no change"
    );
    assert!(
        !advertises(&reader, Role::TextInput, "Locked", Action::SetValue),
        "CTL-005: a disabled text input's published node advertises no SetValue"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_request_a_node_does_not_advertise_changes_nothing() {
    let log: Log<bool> = Arc::default();
    let reader = ReaderChannel::new();
    // A checkbox advertises Focus and Click only; the text input after it
    // is the witness that the App has taken the request before it.
    let controls = builder::div()
        .class("flex-col")
        .child(
            builder::checkbox()
                .label("Agree")
                .on_change(recorder(&log))
                .build(),
        )
        .child(builder::text_input().placeholder("Open").build())
        .build();
    let frames = app_input::run_when_frame_reading(
        Root(controls),
        (80, 8),
        &reader,
        vec![
            (
                shown("[ ] Agree"),
                request(Role::CheckBox, "Agree", Action::SetValue, text("true")),
            ),
            (
                shown("[ ] Agree"),
                request(Role::TextInput, "Open", Action::SetValue, text("done")),
            ),
            (shown("done"), None),
        ],
    );
    assert!(
        !advertises(&reader, Role::CheckBox, "Agree", Action::SetValue),
        "a checkbox advertises no SetValue"
    );
    assert!(
        last(&frames).contains("[ ] Agree") && held(&log).is_empty(),
        "CTL-005: a request the node does not advertise changes nothing: {}",
        last(&frames)
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_005_a_read_only_text_input_advertises_no_set_value_and_keeps_its_text() {
    let log: Log<String> = Arc::default();
    let reader = ReaderChannel::new();
    // As in the disabled case, the second field is the witness.
    let controls = builder::div()
        .class("flex-col")
        .child(
            builder::text_input()
                .placeholder("Fixed")
                .value("keep")
                .readonly(true)
                .on_change(recorder(&log))
                .build(),
        )
        .child(builder::text_input().placeholder("Open").build())
        .build();
    let frames = app_input::run_when_frame_reading(
        Root(controls),
        (80, 8),
        &reader,
        vec![
            (
                shown("keep"),
                request(Role::TextInput, "Fixed", Action::SetValue, text("changed")),
            ),
            (
                shown("keep"),
                request(Role::TextInput, "Open", Action::SetValue, text("done")),
            ),
            (shown("done"), None),
        ],
    );
    assert!(
        !advertises(&reader, Role::TextInput, "Fixed", Action::SetValue),
        "CTL-005: a read-only text input takes no SetValue, so its node advertises none"
    );
    assert!(
        last(&frames).contains("keep")
            && !last(&frames).contains("changed")
            && held(&log).is_empty(),
        "CTL-005: a SetValue request on a read-only text input leaves its text: {}",
        last(&frames)
    );
}
