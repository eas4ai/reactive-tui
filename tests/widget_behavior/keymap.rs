//! KEY-001 and KEY-002: the named key actions, the active keymap and a
//! binding's text form, as the widgets obey them. The harness runs each App
//! on the test's thread, so `Keymap::scoped` rebinds the keys for that App
//! alone; the probe that follows a key that must do nothing is a text input
//! after the widget, reached by Tab, that types one letter: the App handles
//! events in order, so a frame showing the letter proves the earlier key
//! was handled and had no effect.

use std::sync::{Arc, Mutex};

use crate::common::app_input::{self, key, FramePredicate, Snapshot};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::{
        router::EventResult,
        types::{Event, KeyCode, KeyEvent, KeyModifiers},
    },
    keymap::{Action, KeyBinding, Keymap},
    widgets::{
        layout::BreadcrumbSegment,
        menu::{ContextMenu, ContextMenuProps, MenuItem},
        AccordionSection,
    },
};

type Log<T> = Arc<Mutex<Vec<T>>>;

fn recorder<T: Send + 'static>(log: &Log<T>) -> impl Fn(T) + Send + Sync + 'static {
    let log = log.clone();
    move |value| log.lock().unwrap().push(value)
}

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

/// A root that records the segment a breadcrumb navigates to.
struct Navigation {
    element: Element,
    chosen: Log<String>,
}

impl RootComponent for Navigation {
    fn render(&self) -> Element {
        self.element.clone()
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        let Event::Custom(event) = event else {
            return EventResult::Ignored;
        };
        if event.name != "navigate" {
            return EventResult::Ignored;
        }
        let data: serde_json::Value = serde_json::from_slice(&event.data).expect("JSON data");
        self.chosen
            .lock()
            .unwrap()
            .push(data["segment_id"].as_str().unwrap_or_default().to_owned());
        EventResult::Consumed
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn shown(needle: &'static str) -> FramePredicate {
    Box::new(move |frame: &Snapshot| frame.text.contains(needle))
}

fn gone(needle: &'static str) -> FramePredicate {
    Box::new(move |frame: &Snapshot| !frame.text.contains(needle))
}

fn f2() -> Option<Event> {
    key(KeyCode::F(2))
}

fn typed(c: char) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(KeyCode::Char(c))))
}

fn shift_f10() -> Option<Event> {
    Some(Event::Key(
        KeyEvent::new(KeyCode::F(10)).with_modifiers(KeyModifiers::shift()),
    ))
}

/// The default keymap with Confirm moved from Enter to F2.
fn confirm_on_f2() -> Keymap {
    let mut keymap = Keymap::default();
    keymap.rebind(Action::Confirm, [KeyBinding::new(KeyCode::F(2))]);
    keymap
}

/// The default keymap with Down moved from the arrow to `j`.
fn down_on_j() -> Keymap {
    let mut keymap = Keymap::default();
    keymap.rebind(Action::Down, [KeyBinding::new(KeyCode::Char('j'))]);
    keymap
}

/// `control` with the focus, and a probe field after it that Tab reaches.
fn with_probe(control: Element) -> Element {
    builder::div()
        .class("flex flex-col w-full h-full")
        .child(control.auto_focus())
        .child(builder::text_input().placeholder("PROBE").build())
        .build()
}

/// Sends `ignored` to the focused control, then Tab and `x` into the probe
/// field; the returned frames end with the one that shows the `x`, so
/// `ignored` was handled before it.
fn ignored_then_probed(
    control: Element,
    size: (u16, u16),
    ignored: Option<Event>,
) -> Vec<Snapshot> {
    app_input::run_when_frame(
        Root(with_probe(control)),
        size,
        vec![
            (shown("PROBE"), ignored),
            (shown("PROBE"), key(KeyCode::Tab)),
            (shown("PROBE"), typed('x')),
            (gone("PROBE"), None),
        ],
    )
}

#[test]
#[serial_test::serial(theme)]
fn key_001_confirm_rebound_to_f2_reaches_a_checkbox_and_not_on_enter() {
    let _scope = Keymap::scoped(confirm_on_f2());
    let log: Log<bool> = Arc::default();
    let checkbox = || {
        builder::checkbox()
            .label("Agree")
            .on_change(recorder(&log))
            .build()
    };
    let frames = ignored_then_probed(checkbox(), (80, 8), key(KeyCode::Enter));
    assert_eq!(
        held(&log),
        Vec::<bool>::new(),
        "Enter no longer toggles the box:\n{}",
        frames.last().map(|f| f.text.clone()).unwrap_or_default()
    );
    app_input::run_when_frame(
        Root(checkbox().auto_focus()),
        (80, 8),
        vec![(shown("Agree"), f2()), (shown("[✓]"), None)],
    );
    assert_eq!(held(&log), vec![true], "F2 toggles the box");
}

#[test]
#[serial_test::serial(theme)]
fn key_001_confirm_rebound_to_f2_reaches_a_select_and_a_text_inputs_submit() {
    let _scope = Keymap::scoped(confirm_on_f2());

    let select = || {
        builder::select()
            .option("cyan", "Cyan")
            .option("violet", "Violet")
            .selected("cyan")
            .build()
    };
    let frames = ignored_then_probed(select(), (80, 10), key(KeyCode::Enter));
    assert!(
        frames.iter().all(|frame| !frame.text.contains("Violet")),
        "Enter no longer opens the select's list"
    );
    app_input::run_when_frame(
        Root(select().auto_focus()),
        (80, 10),
        vec![(shown("[Cyan"), f2()), (shown("Violet"), None)],
    );

    let submits: Log<String> = Arc::default();
    let input = || {
        builder::text_input()
            .value("hello")
            .on_submit(recorder(&submits))
            .build()
    };
    ignored_then_probed(input(), (80, 8), key(KeyCode::Enter));
    assert_eq!(
        held(&submits),
        Vec::<String>::new(),
        "Enter no longer submits the text"
    );
    app_input::run_when_frame(
        Root(input().auto_focus()),
        (80, 8),
        vec![
            (shown("hello"), f2()),
            (shown("hello"), key(KeyCode::End)),
            (shown("hello"), typed('!')),
            (shown("hello!"), None),
        ],
    );
    assert_eq!(
        held(&submits),
        vec!["hello".to_owned()],
        "F2 submits the text"
    );
}

#[test]
#[serial_test::serial(theme)]
fn key_001_confirm_rebound_to_f2_reaches_an_accordion_and_a_breadcrumbs_ellipsis() {
    let _scope = Keymap::scoped(confirm_on_f2());

    let opened: Log<(usize, bool)> = Arc::default();
    let accordion = || {
        let log = opened.clone();
        builder::accordion()
            .section(AccordionSection::new("a", "First").content(Element::text("BODY-A")))
            .section(AccordionSection::new("b", "Second").content(Element::text("BODY-B")))
            .animated(false)
            .on_change(move |index, open| log.lock().unwrap().push((index, open)))
            .build()
    };
    ignored_then_probed(accordion(), (80, 12), key(KeyCode::Enter));
    assert_eq!(
        held(&opened),
        Vec::<(usize, bool)>::new(),
        "Enter no longer opens a section"
    );
    app_input::run_when_frame(
        Root(accordion().auto_focus()),
        (80, 12),
        vec![(shown("Second"), f2()), (shown("BODY-A"), None)],
    );
    assert_eq!(
        held(&opened),
        vec![(0, true)],
        "F2 opens the focused section"
    );

    // Five 12-cell segments in 40 cells: the ellipsis stands for three. Right
    // from the first segment reaches it. Enter, no action now, opens nothing;
    // F2 opens the menu of the hidden segments. Had Enter opened it, F2 would
    // have chosen the first hidden segment and closed the menu, so the frame
    // listing the hidden segments would not come and `chosen` would name s2.
    let mut trail = builder::breadcrumb().show_icons(false).on_click("navigate");
    for n in 1..=5 {
        trail = trail.segment(
            BreadcrumbSegment::new(format!("s{n}"), format!("Segment {n:02}"), format!("/{n}"))
                .current(n == 5),
        );
    }
    let chosen: Log<String> = Arc::default();
    app_input::run_when_frame(
        Navigation {
            element: builder::div()
                .class("flex-col w-40 h-10")
                .child(trail.build().auto_focus())
                .build(),
            chosen: chosen.clone(),
        },
        (120, 30),
        vec![
            // The trail ellipsizes on its second frame: wait for the "...".
            (shown("/.../"), key(KeyCode::Home)),
            (shown("/.../"), key(KeyCode::Right)),
            (shown("/.../"), key(KeyCode::Enter)),
            (shown("/.../"), f2()),
            (shown("Segment 03"), None),
        ],
    );
    assert_eq!(
        held(&chosen),
        Vec::<String>::new(),
        "Enter chose nothing; F2 opened the hidden segments"
    );
}

#[test]
#[serial_test::serial(theme)]
fn key_001_confirm_rebound_to_f2_reaches_a_menu_item_and_a_confirmation_dialog() {
    let _scope = Keymap::scoped(confirm_on_f2());

    let calls: Log<&'static str> = Arc::default();
    let menu = || {
        let (first, second) = (calls.clone(), calls.clone());
        builder::div()
            .class("flex flex-col w-full h-full")
            .child(
                Element::typed::<ContextMenu>(ContextMenuProps {
                    items: vec![
                        MenuItem::action("one", "ONE", move || first.lock().unwrap().push("one")),
                        MenuItem::action("two", "TWO", move || second.lock().unwrap().push("two")),
                    ],
                    ..Default::default()
                })
                .class("w-full h-4")
                .auto_focus(),
            )
            .child(builder::text_input().placeholder("PROBE").build())
            .build()
    };
    let frames = app_input::run_when_frame(
        Root(menu()),
        (60, 20),
        vec![
            (shown("PROBE"), shift_f10()),
            (shown("TWO"), key(KeyCode::Down)),
            (shown("TWO"), key(KeyCode::Enter)),
            (shown("TWO"), f2()),
            (gone("TWO"), None),
        ],
    );
    assert_eq!(
        held(&calls),
        vec!["two"],
        "Enter runs no item any more and F2 runs the current one:\n{}",
        frames.last().map(|f| f.text.clone()).unwrap_or_default()
    );

    let dialog = || {
        builder::confirmation_dialog()
            .title("TITLE")
            .message("MESSAGE")
            .confirm_text("YES")
            .cancel_text("NO")
            .build()
    };
    let frames = app_input::run_when_frame(
        Root(dialog()),
        (60, 20),
        vec![
            (shown("MESSAGE"), key(KeyCode::Enter)),
            (shown("MESSAGE"), f2()),
            (gone("MESSAGE"), None),
        ],
    );
    assert!(
        frames.iter().any(|frame| frame.text.contains("MESSAGE"))
            && frames.last().is_some_and(|f| !f.text.contains("MESSAGE")),
        "the dialog, shown before, closes on F2 and not on the Enter sent first ({} frames)",
        frames.len()
    );
}

#[test]
#[serial_test::serial(theme)]
fn key_001_down_rebound_to_j_moves_a_selects_list_and_a_menu_and_the_arrow_no_longer_does() {
    let _scope = Keymap::scoped(down_on_j());

    let log: Log<String> = Arc::default();
    let select = builder::select()
        .option("cyan", "Cyan")
        .option("violet", "Violet")
        .selected("cyan")
        .on_change(recorder(&log))
        .build()
        .auto_focus();
    app_input::run_when_frame(
        Root(select),
        (80, 10),
        vec![
            (shown("[Cyan"), key(KeyCode::Enter)),
            // The arrow is no action now: the highlight stays on Cyan.
            (shown("Violet"), key(KeyCode::Down)),
            (shown("Violet"), typed('j')),
            (shown("Violet"), key(KeyCode::Enter)),
            (shown("[Violet"), None),
        ],
    );
    assert_eq!(
        held(&log),
        vec!["violet".to_owned()],
        "`j` moves to Violet and Enter chooses it"
    );

    let calls: Log<&'static str> = Arc::default();
    let (first, second) = (calls.clone(), calls.clone());
    let menu = Element::typed::<ContextMenu>(ContextMenuProps {
        items: vec![
            MenuItem::action("one", "ONE", move || first.lock().unwrap().push("one")),
            MenuItem::action("two", "TWO", move || second.lock().unwrap().push("two")),
        ],
        ..Default::default()
    })
    .class("w-full h-4")
    .auto_focus();
    app_input::run_when_frame(
        Root(menu),
        (60, 20),
        vec![
            (gone("TWO"), shift_f10()),
            (shown("TWO"), key(KeyCode::Down)),
            (shown("TWO"), typed('j')),
            (shown("TWO"), key(KeyCode::Enter)),
            (gone("TWO"), None),
        ],
    );
    assert_eq!(
        held(&calls),
        vec!["two"],
        "`j` moves to TWO and Enter runs it"
    );
}

#[test]
#[serial_test::serial(theme)]
fn key_001_a_text_input_keeps_typing_a_letter_bound_to_an_action() {
    let _scope = Keymap::scoped(down_on_j());
    let frames = app_input::run_when_frame(
        Root(builder::text_input().value("a").build().auto_focus()),
        (80, 6),
        vec![
            (shown("[a"), key(KeyCode::End)),
            (shown("[a"), typed('j')),
            (shown("[aj"), None),
        ],
    );
    assert!(
        frames.last().is_some_and(|f| f.text.contains("[aj")),
        "the field takes `j` as text although Down is bound to it"
    );
}

#[test]
#[serial_test::serial(theme)]
fn key_001_unbinding_activate_leaves_a_checkbox_untoggled_by_space() {
    let mut keymap = Keymap::default();
    keymap.unbind(Action::Activate, &KeyBinding::new(KeyCode::Space));
    let _scope = Keymap::scoped(keymap);
    let log: Log<bool> = Arc::default();
    let checkbox = builder::checkbox()
        .label("Agree")
        .on_change(recorder(&log))
        .build();
    ignored_then_probed(checkbox, (80, 8), key(KeyCode::Char(' ')));
    assert_eq!(
        held(&log),
        Vec::<bool>::new(),
        "Space no longer toggles the box"
    );
}
