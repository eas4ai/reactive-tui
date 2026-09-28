use super::{click, key, run, Control};
use reactive_tui::{
    event::types::KeyCode,
    widgets::input::{RadioButtonBuilder, RadioOrientation},
};

#[test]
fn radio_generic_builder_selects_by_keyboard_and_horizontal_mouse() {
    for size in [(32, 6), (60, 12)] {
        let control = RadioButtonBuilder::new()
            .option(1, "界")
            .option(2, "Beta")
            .selected(1)
            .orientation(RadioOrientation::Horizontal)
            .render()
            .with_class("w-28 h-5 p-0.5")
            .auto_focus();
        let frames = run(Control(control), size, vec![(1, click(14, 2)), (2, None)]);
        assert!(
            frames[0].text.contains("(●) 界"),
            "frames={:?}, geometry={:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>(),
            frames[0].geometry
        );
        assert!(
            frames.last().unwrap().text.contains("(●) _Beta_"),
            "{}",
            frames.last().unwrap().text
        );
        let control = RadioButtonBuilder::new()
            .option(1, "Alpha")
            .disabled_option(2, "Disabled")
            .option(3, "Gamma")
            .selected(1)
            .render()
            .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![(1, key(KeyCode::Down)), (1, key(KeyCode::Enter)), (2, None)],
        );
        assert!(
            frames.last().unwrap().text.contains("(●) Gamma"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn radio_empty_and_disabled_controls_ignore_activation() {
    for size in [(32, 6), (60, 12)] {
        for control in [
            RadioButtonBuilder::<u8>::new().render(),
            RadioButtonBuilder::new()
                .option(1, "Locked")
                .disabled(true)
                .render(),
        ] {
            let frames = run(
                Control(control.auto_focus()),
                size,
                vec![
                    (1, key(KeyCode::Down)),
                    (1, key(KeyCode::Enter)),
                    (1, click(2, 0)),
                    (1, None),
                ],
            );
            assert!(!frames.last().unwrap().text.contains("(●)"));
        }
    }
}

#[test]
fn radio_callback_runs_once_for_a_changed_selection() {
    use reactive_tui::{
        component::{Component, Element},
        widgets::input::RadioButton,
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 6), (60, 12)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let callback = changes.clone();
        let props = RadioButtonBuilder::new()
            .option(1, "Alpha")
            .option(2, "Beta")
            .selected(1)
            .build();
        let control = Element::typed_with::<RadioButton<i32>>(props, move |props| {
            let callback = callback.clone();
            RadioButton::new(props)
                .with_on_change(move |value| callback.lock().unwrap().push(value))
        })
        .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Down)),
                (1, key(KeyCode::Enter)),
                (2, key(KeyCode::Enter)),
                (2, None),
            ],
        );
        assert_eq!(*changes.lock().unwrap(), vec![2]);
        assert!(frames.last().unwrap().text.contains("(●) Beta"));
    }
}

fn named_radio(
    label: &str,
    checked: bool,
    group: Option<&str>,
) -> reactive_tui::component::Element {
    let mut radio = reactive_tui::builder::radio_button()
        .value("duplicate")
        .label(label)
        .checked(checked)
        .class("w-24 h-1");
    if let Some(group) = group {
        radio = radio.group(group);
    }
    radio.build().with_key(label)
}

fn named_radios(
    children: Vec<reactive_tui::component::Element>,
) -> reactive_tui::component::Element {
    reactive_tui::component::Element::layout(reactive_tui::component::LayoutType::Flex)
        .with_class("flex flex-col")
        .with_children(children)
}

#[test]
fn named_radio_builder_groups_distinct_owners_and_preserves_keyed_selection() {
    use reactive_tui::{
        app::RootComponent,
        component::Element,
        event::{router::EventResult, types::Event},
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Radios(AtomicUsize);
    impl RootComponent for Radios {
        fn wake_driven(&self) -> bool {
            true
        }
        fn render(&self) -> Element {
            let a = named_radio("Alpha", true, Some("choice"));
            let b = named_radio("Beta", true, Some("choice"));
            let other = named_radio("Other", false, Some("other"));
            match self.0.load(Ordering::Relaxed) {
                0 => named_radios(vec![a.auto_focus(), b, other]),
                1 => named_radios(vec![b, a, other]),
                _ => named_radios(vec![a, other]),
            }
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if matches!(event, Event::Key(key) if key.code == KeyCode::Char('r')) {
                self.0.fetch_add(1, Ordering::Relaxed);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }
    for size in [(32, 6), (60, 12)] {
        let frames = run(
            Radios(AtomicUsize::new(0)),
            size,
            vec![
                (1, key(KeyCode::Tab)),
                (1, key(KeyCode::Enter)),
                (2, key(KeyCode::Char('r'))),
                (3, click(3, 2)),
                (4, key(KeyCode::Char('r'))),
                (5, click(3, 0)),
                (6, None),
            ],
        );
        assert!(frames[0].text.contains("(●) Alpha"));
        assert!(!frames[0].text.contains("(●) Beta"));
        assert!(
            frames
                .iter()
                .any(|frame| frame.text.lines().next().unwrap_or("").contains("(●) Beta")),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(frames.last().unwrap().text.contains("(●) Alpha"));
        assert!(frames.last().unwrap().text.contains("(●) Other"));
        for frame in &frames {
            assert!(!(frame.text.contains("(●) Alpha") && frame.text.contains("(●) Beta")));
        }
    }
}

#[test]
fn nested_apps_do_not_share_named_radio_selection() {
    use reactive_tui::{
        app::RootComponent,
        component::Element,
        event::{router::EventResult, types::Event},
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Outer(AtomicBool);
    impl RootComponent for Outer {
        fn wake_driven(&self) -> bool {
            true
        }
        fn render(&self) -> Element {
            if self.0.swap(false, Ordering::Relaxed) {
                let inner = run(
                    Control(named_radios(vec![
                        named_radio("Inner", false, Some("shared")),
                        named_radio("Second", false, Some("shared")),
                    ])),
                    (32, 6),
                    vec![(1, click(3, 1)), (2, None)],
                );
                assert!(!inner[0].text.contains("(●)"));
                assert!(inner.last().unwrap().text.contains("(●) Second"));
            }
            named_radio("Outer", true, Some("shared")).auto_focus()
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if matches!(event, Event::Key(key) if key.code == KeyCode::Char('n')) {
                self.0.store(true, Ordering::Relaxed);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }
    let frames = run(
        Outer(AtomicBool::new(false)),
        (32, 6),
        vec![(1, key(KeyCode::Char('n'))), (2, None)],
    );
    assert!(frames.last().unwrap().text.contains("(●) Outer"));
}

#[test]
fn named_radio_ungrouped_and_disabled_controls_keep_independent_state() {
    for size in [(32, 6), (60, 12)] {
        let root = named_radios(vec![
            named_radio("Alpha", false, None),
            named_radio("Beta", false, None),
            reactive_tui::builder::radio_button()
                .label("Locked")
                .disabled(true)
                .class("w-24 h-1")
                .build(),
        ]);
        let frames = run(
            Control(root),
            size,
            vec![
                (1, click(3, 0)),
                (2, click(3, 1)),
                (3, click(3, 2)),
                (3, None),
            ],
        );
        assert!(frames.last().unwrap().text.contains("(●) Alpha"));
        assert!(frames.last().unwrap().text.contains("(●) Beta"));
        assert!(!frames.last().unwrap().text.contains("(●) Locked"));
    }
}
