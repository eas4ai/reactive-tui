#[path = "api_widget_behavior/terminal.rs"]
mod terminal_acceptance;
use reactive_tui::{
    app::RootComponent,
    builder,
    component::{Component, Element, LayoutInfo, Props},
    event::types::{Event, KeyCode, ResizeEvent},
};
use std::sync::{Arc, Mutex};

#[path = "api_widget_behavior/accessibility_styles.rs"]
mod accessibility_styles;
#[path = "api_widget_behavior/accordion.rs"]
mod accordion_acceptance;
mod common;
use common::app_input;
#[path = "api_widget_behavior/autocomplete.rs"]
mod autocomplete_acceptance;
#[path = "api_widget_behavior/breadcrumb.rs"]
mod breadcrumb_acceptance;
#[path = "api_widget_behavior/charts.rs"]
mod chart_acceptance;
#[path = "api_widget_behavior/confirmation.rs"]
mod confirmation_acceptance;
#[path = "api_widget_behavior/core_builders.rs"]
mod core_builder_acceptance;
#[path = "api_widget_behavior/data_table.rs"]
mod data_table_acceptance;
#[path = "api_widget_behavior/dialogs.rs"]
mod dialog_acceptance;
#[path = "api_widget_behavior/dialog_http.rs"]
mod dialog_http_acceptance;
#[path = "api_widget_behavior/dialog_input_lifecycle.rs"]
mod dialog_input_lifecycle_acceptance;
#[path = "api_widget_behavior/dialog_notifications.rs"]
mod dialog_notification_acceptance;
#[path = "api_widget_behavior/dialog_position.rs"]
mod dialog_position_acceptance;
#[path = "api_widget_behavior/file_explorer.rs"]
mod file_explorer_acceptance;
#[path = "api_widget_behavior/image.rs"]
mod image_acceptance;
#[path = "api_widget_behavior/input.rs"]
mod input_acceptance;
#[path = "api_widget_behavior/macros.rs"]
mod macro_acceptance;
#[path = "api_widget_behavior/menus.rs"]
mod menu_acceptance;
#[path = "api_widget_behavior/modal.rs"]
mod modal_acceptance;
#[path = "api_widget_behavior/popover.rs"]
mod popover_acceptance;
#[path = "api_widget_behavior/progress.rs"]
mod progress_acceptance;
#[path = "api_widget_behavior/radio.rs"]
mod radio_acceptance;
#[path = "api_widget_behavior/scroll.rs"]
mod scroll_acceptance;
#[path = "api_widget_behavior/select.rs"]
mod select_acceptance;
#[path = "api_widget_behavior/slider.rs"]
mod slider_acceptance;
#[path = "api_widget_behavior/stack.rs"]
mod stack_acceptance;
#[path = "api_widget_behavior/table.rs"]
mod table_acceptance;
#[path = "api_widget_behavior/tabs.rs"]
mod tabs_acceptance;
#[path = "api_widget_behavior/tree.rs"]
mod tree_acceptance;
#[path = "api_widget_behavior/vdom.rs"]
mod vdom_acceptance;
#[path = "api_widget_behavior/wizard.rs"]
mod wizard_acceptance;
use app_input::{click, key, run};

#[test]
fn button_activation_uses_app_keyboard_and_mouse() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(24, 6), (48, 12)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let callback = calls.clone();
        let button = builder::button()
            .text("Apply")
            .class("w-8 h-1 p-0")
            .on_click(move || {
                callback.fetch_add(1, Ordering::SeqCst);
            })
            .build()
            .auto_focus();
        let frames = run(
            Control(button),
            size,
            vec![(1, key(KeyCode::Enter)), (1, click(2, 0))],
        );
        assert!(frames[0].text.contains("Apply"));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}

struct Control(Element);
impl RootComponent for Control {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn text_input_builder_edits_through_app_at_two_sizes() {
    for size in [(24, 6), (48, 12)] {
        let frames = run(
            Control(builder::text_input().value("seed").build().auto_focus()),
            size,
            vec![
                (1, key(KeyCode::End)),
                (1, key(KeyCode::Char('X'))),
                (2, None),
            ],
        );
        assert!(frames[0].text.contains("seed"));
        assert!(
            frames.iter().any(|frame| frame.text.contains("seedX")),
            "typing must edit the painted input: {:?}",
            frames.iter().map(|frame| &frame.text).collect::<Vec<_>>()
        );
    }
}

#[test]
fn checkbox_builder_toggles_through_app_at_two_sizes() {
    for size in [(24, 6), (48, 12)] {
        let frames = run(
            Control(builder::checkbox().label("agree").build().auto_focus()),
            size,
            vec![(1, key(KeyCode::Char(' '))), (2, None)],
        );
        assert!(frames[0].text.contains("[ ]"), "{}", frames[0].text);
        assert!(frames.iter().any(|frame| frame.text.contains("[✓]")));
    }
}

#[test]
fn checkbox_hover_covers_the_label_and_clears_outside() {
    use reactive_tui::event::types::{MouseEvent, MouseEventKind, Position};
    for size in [(24, 6), (48, 12)] {
        let movement = |x, y| {
            Some(Event::Mouse(MouseEvent::new(
                MouseEventKind::Move,
                Position::cell(x, y),
            )))
        };
        let frames = run(
            Control(builder::checkbox().label("agreement").build()),
            size,
            vec![(1, movement(10, 0)), (2, movement(20, 3)), (3, None)],
        );
        assert!(frames[1].text.contains("_agreement_"), "{}", frames[1].text);
        assert!(!frames.last().unwrap().text.contains("_agreement_"));
        assert!(frames.last().unwrap().text.contains("agreement"));
    }
}

#[test]
fn select_builder_selects_through_app_at_two_sizes() {
    for size in [(24, 6), (48, 12)] {
        let frames = run(
            Control(
                builder::select()
                    .option("a", "Alpha")
                    .option("b", "Beta")
                    .selected("a")
                    .build()
                    .auto_focus(),
            ),
            size,
            vec![
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Down)),
                (1, key(KeyCode::Enter)),
                (2, None),
            ],
        );
        assert!(frames[0].text.contains("Alpha"));
        assert!(frames.last().unwrap().text.contains("Beta"));
    }
}

#[derive(Clone, Default)]
struct GeometryLog {
    clicks: Arc<Mutex<Vec<(u16, u16)>>>,
    layouts: Arc<Mutex<Vec<LayoutInfo>>>,
    keys: Arc<Mutex<Vec<KeyCode>>>,
}
impl PartialEq for GeometryLog {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.clicks, &other.clicks)
            && Arc::ptr_eq(&self.layouts, &other.layouts)
            && Arc::ptr_eq(&self.keys, &other.keys)
    }
}
impl Props for GeometryLog {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct GeometryProbe;
impl Component for GeometryProbe {
    type Props = GeometryLog;
    type State = ();
    fn new(_: Self::Props) -> Self {
        Self
    }
    fn render(&self, _: &Self::Props, _: &()) -> Element {
        Element::text("ABCDEFGHIJ")
    }
    fn layout(&mut self, layout: LayoutInfo, props: &mut Self::Props, _: &mut ()) -> bool {
        props.layouts.lock().unwrap().push(layout);
        false
    }
    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        _: &mut (),
    ) -> reactive_tui::event::router::EventResult {
        if let Event::Mouse(mouse) = event {
            if mouse.kind != reactive_tui::event::types::MouseEventKind::Down {
                return reactive_tui::event::router::EventResult::Ignored;
            }
            props
                .clicks
                .lock()
                .unwrap()
                .push((mouse.position.x() as u16, mouse.position.y() as u16));
            return reactive_tui::event::router::EventResult::Consumed;
        }
        if let Event::Key(key) = event {
            props.keys.lock().unwrap().push(key.code.clone());
        }
        reactive_tui::event::router::EventResult::Ignored
    }
}

/// Shows the content width its last layout gave it.
#[derive(Default)]
struct WidthProbe {
    width: Option<f32>,
}
impl Component for WidthProbe {
    type Props = reactive_tui::component::props::EmptyProps;
    type State = ();
    fn new(_: Self::Props) -> Self {
        Self::default()
    }
    fn render(&self, _: &Self::Props, _: &()) -> Element {
        Element::text(format!(
            "width={}",
            self.width.map_or("none".into(), |w| w.to_string())
        ))
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut ()) -> bool {
        let width = Some(layout.content_size().0);
        let changed = self.width != width;
        self.width = width;
        changed
    }
}

/// The App lays a resized frame out before presenting it, so the first
/// frame at the new size already shows the size each component was given
/// (BAR-003: no stale geometry after a resize).
#[test]
fn the_first_frame_after_a_resize_shows_components_their_new_size() {
    let frames = run(
        Control(
            Element::typed::<WidthProbe>(reactive_tui::component::props::EmptyProps)
                .class("w-full"),
        ),
        (24, 6),
        vec![
            (2, Some(Event::Resize(ResizeEvent::new(48, 12)))),
            (3, None),
        ],
    );
    let before = frames
        .iter()
        .rfind(|frame| frame.screen.size() == (6, 24))
        .expect("a frame at 24 by 6");
    assert!(before.text.contains("width=24"), "{}", before.text);
    let first = frames
        .iter()
        .find(|frame| frame.screen.size() == (12, 48))
        .expect("a frame at 48 by 12");
    assert!(
        first.text.contains("width=48"),
        "the first frame at 48 by 12 must show the new width, not the old one:\n{}",
        first.text
    );
}

#[test]
fn an_unhandled_tab_reaches_a_component_only_once() {
    let log = GeometryLog::default();
    run(
        Control(Element::typed::<GeometryProbe>(log.clone())),
        (24, 6),
        vec![(1, key(KeyCode::Tab))],
    );
    assert_eq!(*log.keys.lock().unwrap(), vec![KeyCode::Tab]);
}

#[test]
fn clipping_preserves_original_component_size_and_local_mouse_position() {
    for size in [(24, 6), (48, 12)] {
        let log = GeometryLog::default();
        let frames = run(
            Control(
                builder::div()
                    .class("relative w-4 h-1 overflow-hidden")
                    .child(
                        Element::typed::<GeometryProbe>(log.clone())
                            .class("absolute w-10 h-1 -translate-x-2"),
                    )
                    .build(),
            ),
            size,
            vec![(1, click(1, 0))],
        );
        assert!(frames[0].text.contains("CDEF"), "{}", frames[0].text);
        assert_eq!(*log.clicks.lock().unwrap(), vec![(3, 0)]);
        assert_eq!(log.layouts.lock().unwrap()[0].size, (10.0, 1.0));
    }
}

#[test]
fn checkbox_callback_and_state_survive_resize_without_duplication() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let callback = calls.clone();
    let control = Element::typed_with::<reactive_tui::widgets::Checkbox>(
        reactive_tui::widgets::CheckboxProps {
            label: Some("agree".into()),
            ..Default::default()
        },
        move |props| {
            let callback = callback.clone();
            reactive_tui::widgets::Checkbox::new(props)
                .with_on_change(move |value| callback.lock().unwrap().push(value))
        },
    )
    .auto_focus();
    let frames = run(
        Control(control),
        (24, 6),
        vec![
            (1, key(KeyCode::Char(' '))),
            (2, Some(Event::Resize(ResizeEvent::new(48, 12)))),
            (3, None),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), vec![true]);
    assert!(frames[1].text.contains("[✓]"));
    assert!(frames.last().unwrap().text.contains("[✓]"));
}

#[test]
fn readonly_and_disabled_input_do_not_accept_edits() {
    for control in [
        builder::text_input().value("seed").readonly(true).build(),
        builder::text_input().value("seed").disabled(true).build(),
    ] {
        let frames = run(
            Control(control.auto_focus()),
            (24, 6),
            vec![
                (1, key(KeyCode::End)),
                (1, key(KeyCode::Char('X'))),
                (1, Some(Event::Resize(ResizeEvent::new(48, 12)))),
                (2, None),
            ],
        );
        assert!(frames.last().unwrap().text.contains("seed"));
        assert!(!frames.last().unwrap().text.contains('X'));
    }
}

#[test]
fn multiple_select_retains_independent_choices_and_escape_closes() {
    for size in [(24, 6), (48, 12)] {
        let control = builder::select()
            .option("a", "Alpha")
            .option("b", "Beta")
            .multiple(true)
            .build()
            .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Down)),
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Escape)),
                (2, None),
            ],
        );
        assert!(
            frames.last().unwrap().text.contains("Alpha, Beta"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

fn ctrl(code: char) -> Option<Event> {
    use reactive_tui::event::types::{KeyEvent, KeyModifiers};
    Some(Event::Key(
        KeyEvent::new(KeyCode::Char(code)).with_modifiers(KeyModifiers::ctrl()),
    ))
}

#[test]
fn text_input_unicode_replacement_and_undo_preserve_graphemes() {
    for size in [(24, 6), (48, 12)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let callback = changes.clone();
        let element = Element::typed_with::<reactive_tui::widgets::TextInput>(
            reactive_tui::widgets::TextInputProps {
                value: "界🙂e\u{301}".into(),
                ..Default::default()
            },
            move |props| {
                let callback = callback.clone();
                reactive_tui::widgets::TextInput::new(props)
                    .with_on_change(move |value| callback.lock().unwrap().push(value))
            },
        )
        .auto_focus();
        let frames = run(
            Control(element),
            size,
            vec![
                (1, key(KeyCode::End)),
                (1, key(KeyCode::Char('X'))),
                (1, key(KeyCode::Backspace)),
                (1, key(KeyCode::Backspace)),
                (1, ctrl('z')),
                (1, ctrl('a')),
                (1, key(KeyCode::Char('Z'))),
                (1, ctrl('z')),
                (2, None),
            ],
        );
        assert_eq!(
            *changes.lock().unwrap(),
            vec![
                "界🙂e\u{301}X",
                "界🙂e\u{301}",
                "界🙂",
                "界🙂e\u{301}",
                "Z",
                "界🙂e\u{301}"
            ]
        );
        assert!(frames.last().unwrap().text.contains("界🙂e\u{301}"));
    }
}

#[test]
fn tab_focus_notifications_leave_all_input_controls_ready_for_input() {
    let controls = [
        builder::checkbox().label("Check").build(),
        builder::text_input().build(),
        builder::select()
            .option("one", "One")
            .option("two", "Two")
            .build(),
        reactive_tui::widgets::input::SliderBuilder::new()
            .value(0.0)
            .render(),
        reactive_tui::widgets::input::RadioButtonBuilder::new()
            .option(1, "Radio")
            .render(),
    ];
    for size in [(32, 8), (60, 12)] {
        for (index, control) in controls.iter().enumerate() {
            let first = builder::button()
                .text("Start")
                .class("w-24 h-1 p-0")
                .build()
                .auto_focus();
            let root = Element::layout(reactive_tui::component::LayoutType::Flex)
                .with_class("flex flex-col")
                .with_children(vec![first, control.clone().with_class("w-24 h-5")]);
            let activate = match index {
                0 | 4 => KeyCode::Enter,
                1 => KeyCode::Char('X'),
                2 => KeyCode::Enter,
                _ => KeyCode::Right,
            };
            let frames = run(
                Control(root),
                size,
                vec![(1, key(KeyCode::Tab)), (1, key(activate)), (2, None)],
            );
            let expected = match index {
                0 => "[✓]",
                1 => "X",
                2 => "Two",
                3 => "1.0",
                _ => "(●) Radio",
            };
            assert!(
                frames.last().unwrap().text.contains(expected),
                "control {index}: {}",
                frames.last().unwrap().text
            );
        }
    }
}
