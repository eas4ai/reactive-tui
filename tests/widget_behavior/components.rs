//! CMP-009: the builders' change callbacks.

use std::sync::{Arc, Mutex};

use crate::common::app_input::{self, key, FramePredicate, Snapshot};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::{
        router::EventResult,
        types::{Event, KeyCode},
    },
    widgets::AccordionSection,
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

/// The frame once `needle` is no longer painted.
fn gone(needle: &'static str) -> FramePredicate {
    Box::new(move |frame: &Snapshot| !frame.text.contains(needle))
}

/// Any frame: the step after the last event, which stops the App once the
/// event before it has been handled.
fn any() -> FramePredicate {
    Box::new(|_: &Snapshot| true)
}

#[test]
#[serial_test::serial(theme)]
fn cmp_009_a_checkbox_built_through_its_builder_reports_its_change() {
    let log: Log<bool> = Arc::default();
    let control = builder::checkbox()
        .label("Agree")
        .on_change(recorder(&log))
        .build()
        .auto_focus();
    app_input::run_when_frame(
        Root(control),
        (80, 6),
        vec![
            (shown("Agree"), key(KeyCode::Char(' '))),
            (shown("[✓]"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec![true],
        "Space checks the box and the builder's callback hears it checked"
    );
}

#[test]
#[serial_test::serial(theme)]
fn cmp_009_a_text_input_built_through_its_builder_reports_typing_and_submit() {
    let changes: Log<String> = Arc::default();
    let submits: Log<String> = Arc::default();
    let control = builder::text_input()
        .placeholder("Type here")
        .on_change(recorder(&changes))
        .on_submit(recorder(&submits))
        .build()
        .auto_focus();
    app_input::run_when_frame(
        Root(control),
        (80, 6),
        vec![
            (shown("Type here"), key(KeyCode::Char('a'))),
            (gone("Type here"), key(KeyCode::Enter)),
            (any(), None),
        ],
    );
    assert_eq!(
        held(&changes),
        vec!["a".to_owned()],
        "a typed character reaches on_change with the text"
    );
    assert_eq!(
        held(&submits),
        vec!["a".to_owned()],
        "Enter reaches on_submit with the text"
    );
}

#[test]
#[serial_test::serial(theme)]
fn cmp_009_a_radio_built_through_its_builder_reports_its_choice() {
    let log: Log<String> = Arc::default();
    let control = builder::radio_button()
        .value("high")
        .label("High detail")
        .on_change(recorder(&log))
        .build()
        .auto_focus();
    app_input::run_when_frame(
        Root(control),
        (80, 6),
        vec![
            (shown("( ) High detail"), key(KeyCode::Char(' '))),
            (shown("(●) High detail"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec!["high".to_owned()],
        "Space chooses the radio and the builder's callback hears its value"
    );
}

#[test]
#[serial_test::serial(theme)]
fn cmp_009_a_select_built_through_its_builder_reports_its_choice() {
    let log: Log<String> = Arc::default();
    let control = builder::select()
        .option("cyan", "Cyan")
        .option("violet", "Violet")
        .selected("cyan")
        .on_change(recorder(&log))
        .build()
        .auto_focus();
    app_input::run_when_frame(
        Root(control),
        (80, 10),
        vec![
            (shown("[Cyan"), key(KeyCode::Enter)),
            (shown("Violet"), key(KeyCode::Down)),
            (shown("Violet"), key(KeyCode::Enter)),
            (shown("[Violet"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec!["violet".to_owned()],
        "Enter, Down, Enter chooses Violet and the builder's callback hears its value"
    );
}

#[test]
#[serial_test::serial(theme)]
fn cmp_009_a_slider_built_through_its_builder_reports_its_value() {
    let log: Log<f64> = Arc::default();
    let control = builder::slider()
        .value(50.0)
        .on_change(recorder(&log))
        .build()
        .auto_focus();
    app_input::run_when_frame(
        Root(control),
        (80, 6),
        vec![(shown("50.0"), key(KeyCode::Right)), (shown("51.0"), None)],
    );
    assert_eq!(
        held(&log),
        vec![51.0],
        "Right moves the slider one step and the builder's callback hears the new value"
    );
}

#[test]
#[serial_test::serial(theme)]
fn cmp_009_tabs_built_through_their_builder_report_the_new_index() {
    let log: Log<usize> = Arc::default();
    let control = builder::tabs()
        .tab("One", Element::text("PANEL-ONE"))
        .tab("Two", Element::text("PANEL-TWO"))
        .on_change(recorder(&log))
        .build()
        .auto_focus();
    app_input::run_when_frame(
        Root(control),
        (80, 10),
        vec![
            (shown("PANEL-ONE"), key(KeyCode::Right)),
            (shown("PANEL-TWO"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec![1],
        "Right selects the second tab and the builder's callback hears its index"
    );
}

#[test]
#[serial_test::serial(theme)]
fn cmp_009_an_accordion_built_through_its_builder_reports_a_section_opening() {
    let log: Log<(usize, bool)> = Arc::default();
    let recorded = log.clone();
    let control = builder::accordion()
        .section(AccordionSection::new("a", "First").content(Element::text("BODY-A")))
        .section(AccordionSection::new("b", "Second").content(Element::text("BODY-B")))
        .animated(false)
        .on_change(move |index, open| recorded.lock().unwrap().push((index, open)))
        .build()
        .auto_focus();
    app_input::run_when_frame(
        Root(control),
        (80, 12),
        vec![
            (shown("Second"), key(KeyCode::Down)),
            (shown("Second"), key(KeyCode::Enter)),
            (shown("BODY-B"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec![(1, true)],
        "Down, Enter opens the second section and the builder's callback hears its index, open"
    );
}

/// A root that renders one checkbox, equal on every render, whose
/// `on_change` records under the root's current tag: "old" at first, "new"
/// after F2, and no callback after F3. A line under the box names the tag,
/// so a frame shows that the rebuild has happened.
struct Rebuilt {
    tag: Mutex<Option<&'static str>>,
    log: Log<(&'static str, bool)>,
}

impl RootComponent for Rebuilt {
    fn render(&self) -> Element {
        let tag = *self.tag.lock().unwrap();
        let mut checkbox = builder::checkbox().label("Agree");
        if let Some(tag) = tag {
            let log = self.log.clone();
            checkbox = checkbox.on_change(move |checked| log.lock().unwrap().push((tag, checked)));
        }
        builder::div()
            .class("flex-col")
            .child(checkbox.build().auto_focus())
            .child(Element::text(format!(
                "callback: {}",
                tag.unwrap_or("none")
            )))
            .build()
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        let tag = match key.code {
            KeyCode::F(2) => Some("new"),
            KeyCode::F(3) => None,
            _ => return EventResult::Ignored,
        };
        *self.tag.lock().unwrap() = tag;
        EventResult::Consumed
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
#[serial_test::serial(theme)]
fn cmp_009_a_rebuilt_checkbox_calls_the_new_callback_and_not_the_old_one() {
    let log: Log<(&'static str, bool)> = Arc::default();
    let root = Rebuilt {
        tag: Mutex::new(Some("old")),
        log: log.clone(),
    };
    app_input::run_when_frame(
        root,
        (80, 6),
        vec![
            (shown("callback: old"), key(KeyCode::F(2))),
            (shown("callback: new"), key(KeyCode::Char(' '))),
            (shown("[✓]"), key(KeyCode::F(3))),
            (shown("callback: none"), key(KeyCode::Char(' '))),
            (shown("[ ]"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec![("new", true)],
        "after a rebuild with an equal checkbox and another closure, Space calls the new \
         closure and not the old one; after a rebuild without one, it calls none"
    );
}
