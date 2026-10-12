use super::{
    app_input::{self, Action},
    Control,
};
use reactive_tui::{
    builder::{self, IntoElement},
    component::Element,
    event::types::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ResizeEvent},
    vdom::VNode,
};
use std::sync::{Arc, Mutex};

#[test]
fn mixed_routes_preserve_native_payloads_disabled_state_and_resized_targets() {
    for (size, resized) in [((40, 16), (80, 24)), ((80, 24), (40, 16))] {
        for route in 0..12 {
            for disabled in [false, true] {
                let calls = Arc::new(Mutex::new(Vec::new()));
                let called = calls.clone();
                let node = VNode::element("flex")
                    .key("action")
                    .class("w-12 h-1")
                    .attr("disabled", disabled.to_string())
                    .on("click", move |payload| {
                        let event = payload
                            .downcast_ref::<Event>()
                            .expect("native Event payload");
                        called
                            .lock()
                            .unwrap()
                            .push(if event.is_key() { "key" } else { "mouse" });
                    })
                    .child(VNode::text("ACTIVATE"))
                    .build();
                let child = match route {
                    0 => builder::from_vdom(node),
                    1 => node.into_element(),
                    2 => reactive_tui::vdom!(node),
                    3 => reactive_tui::div![node],
                    4 => builder::mixed_container().child_vdom(node).build(),
                    5 => builder::mixed_container()
                        .mixed_children(vec![node.into()])
                        .build(),
                    6 => reactive_tui::el!(node),
                    7 => reactive_tui::el!(div, [node]),
                    8 => reactive_tui::el!(div, class: "w-full", [node]),
                    9 => reactive_tui::span![node],
                    10 => reactive_tui::span![class: "w-full", node],
                    _ => reactive_tui::div![class: "w-full", node],
                };
                let root = builder::div()
                    .class("w-1/2 h-full justify-end")
                    .child(child)
                    .build();
                app_input::run_actions_until_hidden(
                    Control(root),
                    size,
                    vec![
                        (
                            "ACTIVATE",
                            Action::Event(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                        ),
                        ("ACTIVATE", Action::ClickText("ACTIVATE", 1)),
                        (
                            "ACTIVATE",
                            Action::Event(Event::Key(KeyEvent::new(KeyCode::Enter))),
                        ),
                        (
                            "ACTIVATE",
                            Action::Event(Event::Key(KeyEvent::new(KeyCode::Space))),
                        ),
                        (
                            "ACTIVATE",
                            Action::Event(Event::Key(
                                KeyEvent::new(KeyCode::Enter).with_kind(KeyEventKind::Release),
                            )),
                        ),
                        (
                            "ACTIVATE",
                            Action::Event(Event::Key(
                                KeyEvent::new(KeyCode::Enter).with_modifiers(KeyModifiers::ctrl()),
                            )),
                        ),
                        (
                            "ACTIVATE",
                            Action::Event(Event::Key(KeyEvent::new(KeyCode::F(8)))),
                        ),
                    ],
                    "NEVER",
                );
                assert_eq!(
                    *calls.lock().unwrap(),
                    if disabled {
                        vec![]
                    } else {
                        vec!["mouse", "key", "key"]
                    },
                    "route {route}, size {size:?}"
                );
            }
        }
    }
}

#[test]
fn inline_styles_override_classes_and_retain_terminal_paint_and_geometry() {
    for size in [(40, 16), (80, 24)] {
        let node = VNode::element("flex").class("w-2 h-1 bg-red-500")
            .style("width: 50%; height: 4; padding: 1px 2ch; background-color: #123456; flex-direction: row; gap: 0 1")
            .child(VNode::element("flex").style("color: #abcdef; font-weight: bold; font-style: italic; text-decoration: underline")
                .child(VNode::text("A")).build())
            .child(VNode::text("B")).build();
        let frames = app_input::run(Control(builder::from_vdom(node)), size, vec![(1, None)]);
        let screen = &frames[0].screen;
        assert_eq!(screen.cell(1, 2).unwrap().contents(), "A");
        assert_eq!(screen.cell(1, 4).unwrap().contents(), "B");
        let cell = screen.cell(1, 2).unwrap();
        assert_eq!(cell.fgcolor(), vt100::Color::Rgb(171, 205, 239));
        assert!(cell.bold() && cell.italic() && cell.underline());
        assert_eq!(
            screen.cell(3, size.0 / 2 - 1).unwrap().bgcolor(),
            vt100::Color::Rgb(18, 52, 86)
        );
        assert_ne!(
            screen.cell(3, size.0 / 2).unwrap().bgcolor(),
            vt100::Color::Rgb(18, 52, 86)
        );
    }
}

#[test]
fn native_elements_round_trip_through_vdom_without_losing_controls() {
    for size in [(40, 16), (80, 24)] {
        let clicks = Arc::new(Mutex::new(0));
        let called = clicks.clone();
        let native = builder::button()
            .text("NATIVE")
            .class("w-12 h-1 p-0")
            .on_click(move || *called.lock().unwrap() += 1)
            .key("native")
            .build()
            .auto_focus();
        let node = VNode::from_element(native);
        app_input::run(
            Control(node.to_element()),
            size,
            vec![(1, app_input::key(KeyCode::Enter)), (1, None)],
        );
        assert_eq!(*clicks.lock().unwrap(), 1);
        let editor = builder::text_input().value("seed").build().auto_focus();
        app_input::run_when(
            Control(VNode::from_element(editor).to_element()),
            size,
            vec![
                ("seed", app_input::key(KeyCode::End)),
                ("seed", app_input::key(KeyCode::Char('X'))),
                ("seedX", None),
            ],
        );
    }
}

#[test]
fn typed_vdom_properties_and_component_payloads_survive_conversion() {
    let node = VNode::element("Consumer")
        .prop("count", 42u32)
        .prop("name", String::from("named"))
        .build();
    let element = node.to_element();
    let props = element
        .props_as::<reactive_tui::vdom::VElementProps>()
        .unwrap();
    assert_eq!(props.get::<u32>("count"), Some(&42));
    assert_eq!(props.get::<String>("name").unwrap(), "named");
    let original = Element::component("Consumer")
        .props(String::from("payload"))
        .key("consumer");
    let restored = VNode::from_element(original).to_element();
    assert_eq!(restored.props_as::<String>().unwrap(), "payload");
    assert_eq!(restored.key.as_deref(), Some("consumer"));
}

struct PropertyConsumer;
impl reactive_tui::component::Component for PropertyConsumer {
    type Props = reactive_tui::vdom::VElementProps;
    type State = ();
    fn new(_: Self::Props) -> Self {
        Self
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        Element::text(format!(
            "{}{}",
            props.attrs.get("title").unwrap(),
            props.get::<u32>("count").unwrap()
        ))
    }
}

struct ChangingVdom {
    count: u32,
    calls: Arc<Mutex<Vec<u32>>>,
}
impl reactive_tui::app::RootComponent for ChangingVdom {
    fn render(&self) -> Element {
        let count = self.count;
        let calls = self.calls.clone();
        VNode::element("VdomAcceptancePropertyConsumer")
            .key("consumer")
            .prop("count", count)
            .attr("title", "COUNT")
            .style(if count == 0 {
                "color: #112233"
            } else {
                "color: #abcdef"
            })
            .on("click", move |_| calls.lock().unwrap().push(count))
            .build()
            .to_element()
            .auto_focus()
    }
    fn wake_driven(&self) -> bool {
        true
    }
    fn try_handle_event(
        &mut self,
        event: &Event,
    ) -> reactive_tui::error::Result<reactive_tui::event::router::EventResult> {
        use reactive_tui::event::router::EventResult;
        if matches!(event, Event::Key(key) if key.code == KeyCode::F(4)) {
            self.count += 1;
            return Ok(EventResult::Handled);
        }
        Ok(EventResult::Ignored)
    }
}

#[test]
fn registered_vdom_components_update_props_styles_and_replace_handlers() {
    reactive_tui::component::registry::get_global_registry()
        .register::<PropertyConsumer>("VdomAcceptancePropertyConsumer")
        .unwrap();
    for size in [(40, 16), (80, 24)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let frames = app_input::run_when(
            ChangingVdom {
                count: 0,
                calls: calls.clone(),
            },
            size,
            vec![
                ("COUNT0", app_input::key(KeyCode::Enter)),
                ("COUNT0", app_input::key(KeyCode::F(4))),
                ("COUNT1", app_input::key(KeyCode::Enter)),
                ("COUNT1", None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), [0, 1]);
        assert_eq!(
            frames[0].screen.cell(0, 0).unwrap().fgcolor(),
            vt100::Color::Rgb(17, 34, 51)
        );
        assert_eq!(
            frames.last().unwrap().screen.cell(0, 0).unwrap().fgcolor(),
            vt100::Color::Rgb(171, 205, 239)
        );
    }
}

#[test]
fn invalid_inline_declarations_return_app_errors() {
    use reactive_tui::{app::App, backend::SuprTuiBackend};
    for declaration in [
        "width: NaN",
        "padding: -1",
        "color: #界",
        "height: 2vw",
        "unknown: value",
        "opacity: 5",
        "color",
        "width:",
    ] {
        let element = VNode::element("flex")
            .style(declaration)
            .child(VNode::text("bad"))
            .build()
            .to_element();
        let backend = SuprTuiBackend::with_writer(40, 16, std::io::sink()).unwrap();
        let error = App::builder()
            .backend(backend)
            .root(Control(element))
            .build()
            .unwrap()
            .run()
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Invalid terminal inline style declaration"),
            "{declaration}: {error}"
        );
    }
}

#[test]
fn vdom_menu_helpers_paint_and_activate_native_actions() {
    use reactive_tui::{
        vdom::menu,
        widgets::menu::{DialogMenuProps, MenuBarProps, MenuItem, PopupMenuProps, PopupPlacement},
    };
    for size in [(40, 16), (80, 24)] {
        for route in 0..8 {
            let calls = Arc::new(Mutex::new(0));
            let called = calls.clone();
            let items = vec![MenuItem::action("action", "EXECUTE", move || {
                *called.lock().unwrap() += 1
            })];
            let node = match route {
                0 => menu::menubar_with_props(MenuBarProps {
                    items,
                    ..Default::default()
                }),
                1 => menu::menubar()
                    .props(MenuBarProps {
                        items,
                        ..Default::default()
                    })
                    .build(),
                2 => menu::simple_menubar(items),
                3 => menu::popup_menu_with_props(PopupMenuProps {
                    items,
                    visible: true,
                    ..Default::default()
                }),
                4 => menu::popup_menu()
                    .props(PopupMenuProps {
                        items,
                        visible: true,
                        ..Default::default()
                    })
                    .build(),
                5 => menu::simple_popup_menu(items, PopupPlacement::Cursor),
                6 => menu::dialog_menu_with_props(DialogMenuProps {
                    items,
                    visible: true,
                    width: Some(20),
                    ..Default::default()
                }),
                _ => menu::dialog_menu()
                    .props(DialogMenuProps {
                        items,
                        visible: true,
                        width: Some(20),
                        ..Default::default()
                    })
                    .build(),
            };
            let root = builder::div()
                .class("relative w-full h-full")
                .child(node.to_element().auto_focus())
                .build();
            app_input::run_actions_until_hidden(
                Control(root),
                size,
                vec![("EXECUTE", Action::ClickText("EXECUTE", 1))],
                "NEVER",
            );
            assert_eq!(*calls.lock().unwrap(), 1, "route {route}, {size:?}");
        }
    }
}

#[test]
fn vdom_context_menu_helpers_open_and_deliver_one_action() {
    use reactive_tui::{
        event::types::{MouseButton, MouseEvent, MouseEventKind, Position},
        vdom::menu,
        widgets::menu::{ContextMenuProps, MenuItem},
    };
    for size in [(40, 16), (80, 24)] {
        for route in 0..3 {
            let calls = Arc::new(Mutex::new(0));
            let called = calls.clone();
            let items = vec![MenuItem::action("copy", "COPY", move || {
                *called.lock().unwrap() += 1
            })];
            let node = match route {
                0 => menu::context_menu_with_props(ContextMenuProps {
                    items,
                    ..Default::default()
                }),
                1 => menu::context_menu()
                    .props(ContextMenuProps {
                        items,
                        ..Default::default()
                    })
                    .build(),
                _ => menu::simple_context_menu(items),
            };
            app_input::run_actions_until_hidden(
                Control(node.to_element()),
                size,
                vec![
                    (
                        "",
                        Action::Event(Event::Mouse(
                            MouseEvent::new(MouseEventKind::Down, Position::cell(3, 2))
                                .with_button(MouseButton::Right),
                        )),
                    ),
                    ("COPY", Action::ClickText("COPY", 1)),
                ],
                "COPY",
            );
            assert_eq!(*calls.lock().unwrap(), 1, "route {route}, {size:?}");
        }
    }
}
