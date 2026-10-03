use super::{app_input, Control, Element};
use reactive_tui::widgets::menu::{
    ContextMenu, ContextMenuProps, DialogMenu, DialogMenuProps, MenuBar, MenuBarProps, MenuItem,
    PopupMenu, PopupMenuProps,
};

#[test]
fn menu_outlines_paint_cells_without_recoloring_the_panel() {
    use reactive_tui::widgets::menu::MenuStyle;
    for size in [(32, 12), (60, 20)] {
        for show_border in [true, false] {
            let menu = Element::typed::<DialogMenu>(DialogMenuProps {
                visible: true,
                items: vec![MenuItem::new("off", "OFF").enabled(false)],
                style: MenuStyle {
                    base_classes: "bg-#123456 text-white p-0".into(),
                    border_classes: "border border-#fedcba".into(),
                    padding: 0,
                    ..Default::default()
                },
                show_border,
                ..Default::default()
            });
            let frames = app_input::run_when(Control(menu), size, vec![("OFF", None)]);
            let frame = frames.last().unwrap();
            assert_eq!(frame.text.contains('┌'), show_border, "{}", frame.text);
            assert_eq!(frame.text.contains('┘'), show_border, "{}", frame.text);
            for (y, line) in frame.text.lines().enumerate() {
                for (x, ch) in line.chars().enumerate() {
                    if ch == 'O' || ch == '┌' {
                        let cell = frame.screen.cell(y as u16, x as u16).unwrap();
                        assert_eq!(cell.bgcolor(), vt100::Color::Rgb(18, 52, 86));
                        if ch == '┌' {
                            assert_eq!(cell.fgcolor(), vt100::Color::Rgb(254, 220, 186));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn dialog_menu_styles_reach_selected_and_disabled_rows_at_both_sizes() {
    use reactive_tui::widgets::menu::MenuStyle;
    for size in [(32, 12), (60, 20)] {
        let menu = Element::typed::<DialogMenu>(DialogMenuProps {
            visible: true,
            title: Some("PALETTE".into()),
            items: vec![
                MenuItem::new("ink", "INK"),
                MenuItem::new("off", "OFF").enabled(false),
            ],
            style: MenuStyle {
                base_classes: "bg-#123456 text-#abcdef".into(),
                selected_classes: "bg-#345678 text-#fedcba".into(),
                focused_classes: "bg-#345678 text-#fedcba".into(),
                disabled_classes: "text-#102030".into(),
                padding: 0,
                ..Default::default()
            },
            default_button: Some(0),
            show_border: false,
            ..Default::default()
        });
        let frames = app_input::run_when(Control(menu), size, vec![("OFF", None)]);
        let frame = frames.last().unwrap();
        for (text, foreground, background) in [
            (
                "INK",
                vt100::Color::Rgb(254, 220, 186),
                vt100::Color::Rgb(52, 86, 120),
            ),
            (
                "OFF",
                vt100::Color::Rgb(16, 32, 48),
                vt100::Color::Rgb(18, 52, 86),
            ),
        ] {
            let (y, line) = frame
                .text
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains(text))
                .unwrap();
            let x = line.find(text).unwrap();
            let cell = frame.screen.cell(y as u16, x as u16).unwrap();
            assert_eq!(cell.fgcolor(), foreground, "{text}: {}", frame.text);
            assert_eq!(cell.bgcolor(), background, "{text}: {}", frame.text);
        }
    }
}

#[test]
fn menu_public_state_updates_replace_selection_and_callbacks_through_app() {
    use reactive_tui::{
        app::RootComponent,
        component::Component,
        event::{
            router::EventResult,
            types::{Event, KeyCode},
        },
        widgets::menu::{ContextMenuState, DialogMenuState, MenuBarState, PopupMenuState},
    };
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    struct Changed {
        changed: AtomicBool,
        family: usize,
        calls: Arc<Mutex<Vec<(&'static str, bool)>>>,
    }
    impl RootComponent for Changed {
        fn render(&self) -> Element {
            let changed = self.changed.load(Ordering::SeqCst);
            let items = ["FIRST", "SECOND"]
                .into_iter()
                .map(|id| {
                    let calls = self.calls.clone();
                    MenuItem::action(id, id, move || calls.lock().unwrap().push((id, changed)))
                })
                .collect();
            let selected_index = Some(usize::from(changed));
            let menu = match self.family {
                0 => {
                    let props = MenuBarProps {
                        items: vec![MenuItem::submenu("file", "FILE", items)],
                        ..Default::default()
                    };
                    MenuBar::new(props.clone())
                        .render(
                            &props,
                            &MenuBarState {
                                selected_index: Some(0),
                                submenu_selected_index: selected_index,
                                dropdown_open: true,
                                ..Default::default()
                            },
                        )
                        .auto_focus()
                }
                1 => {
                    let props = PopupMenuProps {
                        items,
                        visible: true,
                        ..Default::default()
                    };
                    PopupMenu::new(props.clone()).render(
                        &props,
                        &PopupMenuState {
                            selected_index,
                            ..Default::default()
                        },
                    )
                }
                2 => {
                    let props = ContextMenuProps {
                        items,
                        ..Default::default()
                    };
                    ContextMenu::new(props.clone()).render(
                        &props,
                        &ContextMenuState {
                            is_visible: true,
                            position: Some((2, 2)),
                            popup_state: PopupMenuState {
                                selected_index,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    )
                }
                _ => {
                    let props = DialogMenuProps {
                        items,
                        visible: true,
                        width: Some(20),
                        ..Default::default()
                    };
                    DialogMenu::new(props.clone()).render(
                        &props,
                        &DialogMenuState {
                            selected_index,
                            ..Default::default()
                        },
                    )
                }
            };
            reactive_tui::builder::div()
                .class("relative w-full h-full")
                .child(Element::text(if changed { "UP" } else { "READY" }))
                .child(menu.with_key("menu"))
                .build()
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
        for family in 0..4 {
            let calls = Arc::new(Mutex::new(Vec::new()));
            app_input::run_until_hidden(
                Changed {
                    changed: AtomicBool::new(false),
                    family,
                    calls: calls.clone(),
                },
                size,
                vec![
                    ("SECOND", super::key(KeyCode::F(2))),
                    ("UP", super::key(KeyCode::Enter)),
                ],
                "SECOND",
            );
            assert_eq!(
                *calls.lock().unwrap(),
                [("SECOND", true)],
                "family {family}"
            );
        }
    }
}

#[test]
fn context_menu_long_press_cancels_after_pointer_leaves_its_owner() {
    use reactive_tui::{
        app::{RootComponent, RootUpdate},
        event::{
            router::EventResult,
            types::{Event, KeyCode, MouseButton, MouseEvent, MouseEventKind, Position},
        },
    };
    use std::{
        sync::Mutex,
        time::{Duration, Instant},
    };
    struct Waiting {
        started: Mutex<Option<Instant>>,
        elapsed: bool,
    }
    impl RootComponent for Waiting {
        fn render(&self) -> Element {
            reactive_tui::builder::div()
                .class("relative w-full h-full")
                .child(Element::text(if self.elapsed {
                    "ELAPSED"
                } else {
                    "READY"
                }))
                .child(
                    Element::typed::<ContextMenu>(ContextMenuProps {
                        items: vec![MenuItem::new("copy", "COPY")],
                        show_on_long_press: true,
                        long_press_duration: 80,
                        ..Default::default()
                    })
                    .class("absolute left-4 top-2 w-12 h-6"),
                )
                .build()
        }
        fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
            if !self.elapsed
                && self
                    .started
                    .lock()
                    .unwrap()
                    // The behavior under test: a press held past the 80 ms long press.
                    .is_some_and(|start| start.elapsed() >= Duration::from_millis(200))
            {
                self.elapsed = true;
                Ok(RootUpdate::Redraw)
            } else {
                Ok(RootUpdate::Unchanged)
            }
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
                *self.started.lock().unwrap() = Some(Instant::now());
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }
    for size in [(32, 12), (60, 20)] {
        for kind in [
            MouseEventKind::Up,
            MouseEventKind::Move,
            MouseEventKind::Drag,
        ] {
            let mouse = |kind, x, y| {
                Some(Event::Mouse(
                    MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left),
                ))
            };
            let frames = app_input::run_when(
                Waiting {
                    started: Mutex::new(None),
                    elapsed: false,
                },
                size,
                vec![
                    ("READY", mouse(MouseEventKind::Down, 6, 4)),
                    ("READY", mouse(kind.clone(), size.0 - 1, size.1 - 1)),
                    ("READY", super::key(KeyCode::F(2))),
                    ("ELAPSED", None),
                ],
            );
            assert!(
                frames.iter().all(|frame| !frame.text.contains("COPY")),
                "{kind:?}: {}",
                frames.last().unwrap().text
            );
        }
    }
}

#[test]
fn menu_public_scroll_seed_keeps_its_window_through_navigation() {
    use reactive_tui::{
        component::Component,
        event::types::KeyCode,
        widgets::menu::{DialogMenuState, MenuStyle, PopupMenuState},
    };
    for size in [(32, 12), (60, 20)] {
        for dialog in [false, true] {
            let items = (0..10)
                .map(|i| MenuItem::new(i.to_string(), format!("ROW{i:02}")))
                .collect();
            let style = MenuStyle {
                padding: 0,
                ..Default::default()
            };
            let menu = if dialog {
                let props = DialogMenuProps {
                    visible: true,
                    items,
                    style,
                    height: Some(3),
                    show_close_button: false,
                    show_border: false,
                    show_shadow: false,
                    ..Default::default()
                };
                DialogMenu::new(props.clone()).render(
                    &props,
                    &DialogMenuState {
                        selected_index: Some(4),
                        scroll_offset: 4,
                        ..Default::default()
                    },
                )
            } else {
                let props = PopupMenuProps {
                    visible: true,
                    items,
                    style,
                    max_visible_items: 3,
                    show_border: false,
                    show_shadow: false,
                    ..Default::default()
                };
                PopupMenu::new(props.clone()).render(
                    &props,
                    &PopupMenuState {
                        selected_index: Some(4),
                        scroll_offset: 4,
                        ..Default::default()
                    },
                )
            };
            let frames = app_input::run_when(
                Control(menu.clone()),
                size,
                vec![
                    ("ROW06", super::key(KeyCode::Down)),
                    ("ROW06", super::key(KeyCode::Up)),
                    ("ROW06", None),
                ],
            );
            let frame = &frames.last().unwrap().text;
            assert!(
                frame.contains("ROW04") && frame.contains("ROW06"),
                "{frame}"
            );
            assert!(
                !frame.contains("ROW03") && !frame.contains("ROW07"),
                "{frame}"
            );
            let frames = app_input::run_when(
                Control(menu),
                size,
                vec![
                    ("ROW06", super::key(KeyCode::Down)),
                    ("ROW06", super::key(KeyCode::Down)),
                    ("ROW06", super::key(KeyCode::Down)),
                    ("ROW07", super::key(KeyCode::Up)),
                    ("ROW07", None),
                ],
            );
            let frame = &frames.last().unwrap().text;
            assert!(
                frame.contains("ROW05") && frame.contains("ROW07"),
                "{frame}"
            );
            assert!(
                !frame.contains("ROW04") && !frame.contains("ROW08"),
                "{frame}"
            );
        }
    }
}

#[test]
fn menu_bar_public_scroll_seed_starts_at_the_authored_row() {
    use reactive_tui::{
        component::Component,
        widgets::menu::{MenuBarState, MenuStyle},
    };
    for size in [(32, 12), (60, 20)] {
        let props = MenuBarProps {
            items: vec![MenuItem::submenu(
                "file",
                "FILE",
                (0..10)
                    .map(|i| MenuItem::new(i.to_string(), format!("ROW{i:02}")))
                    .collect(),
            )],
            max_dropdown_height: 3,
            style: MenuStyle {
                padding: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        let menu = MenuBar::new(props.clone())
            .render(
                &props,
                &MenuBarState {
                    selected_index: Some(0),
                    submenu_selected_index: Some(4),
                    dropdown_open: true,
                    dropdown_scroll_offset: 4,
                    ..Default::default()
                },
            )
            .auto_focus();
        let frames = app_input::run_when(Control(menu), size, vec![("ROW04", None)]);
        let frame = &frames.last().unwrap().text;
        assert!(
            frame.contains("ROW04") && frame.contains("ROW06"),
            "{frame}"
        );
        assert!(
            !frame.contains("ROW03") && !frame.contains("ROW07"),
            "{frame}"
        );
    }
}

#[test]
fn menu_bar_resize_moves_open_dropdown_and_pointer_targets() {
    use reactive_tui::{
        component::Component,
        event::types::{Event, ResizeEvent},
        widgets::menu::{MenuBarState, MenuStyle},
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let props = MenuBarProps {
            items: vec![MenuItem::submenu(
                "file",
                "FILE",
                vec![
                    MenuItem::new("first", "FIRST"),
                    MenuItem::action("second", "SECOND", move || {
                        output.lock().unwrap().push("second")
                    }),
                ],
            )],
            style: MenuStyle {
                padding: 0,
                min_width: 8,
                ..Default::default()
            },
            ..Default::default()
        };
        let menu = MenuBar::new(props.clone())
            .render(
                &props,
                &MenuBarState {
                    selected_index: Some(0),
                    submenu_selected_index: Some(0),
                    dropdown_open: true,
                    ..Default::default()
                },
            )
            .auto_focus()
            .class("absolute right-0 top-0 w-12 h-1");
        let tree = reactive_tui::builder::div()
            .class("relative w-full h-full")
            .child(menu)
            .build();
        let frames = app_input::run(
            Control(tree),
            size,
            vec![
                (3, Some(Event::Resize(ResizeEvent::new(20, 8)))),
                (5, super::click(10, 3)),
                (6, None),
            ],
        );
        assert_eq!(
            frames[2].screen.cell(2, size.0 - 10).unwrap().contents(),
            "F",
            "{}",
            frames[2].text
        );
        // The first frame after the resize already shows the new position.
        assert_eq!(
            frames[3].screen.cell(2, 10).unwrap().contents(),
            "F",
            "{}",
            frames[3].text
        );
        assert_eq!(*calls.lock().unwrap(), ["second"]);
        assert!(!frames.last().unwrap().text.contains("SECOND"));
    }
}

#[test]
fn menu_bar_shortcuts_reach_nested_enabled_actions_once() {
    use reactive_tui::{
        event::types::{Event, KeyCode, KeyEvent, KeyModifiers},
        widgets::menu::MenuShortcut,
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let action = |id: &'static str| {
            let calls = calls.clone();
            MenuItem::action(id, id, move || calls.lock().unwrap().push(id))
                .shortcut(MenuShortcut::new("Ctrl+S", vec!["ctrl", "s"]))
        };
        let menu = Element::typed::<MenuBar>(MenuBarProps {
            items: vec![
                MenuItem::submenu("disabled", "DISABLED", vec![action("blocked")]).enabled(false),
                MenuItem::submenu(
                    "file",
                    "FILE",
                    vec![action("hidden").visible(false), action("save")],
                ),
            ],
            ..Default::default()
        })
        .auto_focus();
        let shortcut = |shift| {
            Some(Event::Key(
                KeyEvent::new(KeyCode::Char('s')).with_modifiers(KeyModifiers {
                    ctrl: true,
                    shift,
                    ..KeyModifiers::empty()
                }),
            ))
        };
        app_input::run_when(
            Control(menu),
            size,
            vec![
                ("FILE", super::key(KeyCode::Char('s'))),
                ("FILE", shortcut(true)),
                ("FILE", shortcut(false)),
                ("FILE", None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), ["save"]);
    }
}

#[test]
fn menu_bar_reorder_retains_nested_selection_and_replaces_callback() {
    use reactive_tui::{
        app::RootComponent,
        event::{
            router::EventResult,
            types::{Event, KeyCode},
        },
    };
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    struct Reorder {
        changed: AtomicBool,
        calls: Arc<Mutex<Vec<(&'static str, bool)>>>,
    }
    impl RootComponent for Reorder {
        fn render(&self) -> Element {
            let changed = self.changed.load(Ordering::SeqCst);
            let mut items: Vec<_> = ["ALPHA", "BRAVO"]
                .into_iter()
                .map(|id| {
                    let calls = self.calls.clone();
                    MenuItem::action(id, id, move || calls.lock().unwrap().push((id, changed)))
                })
                .collect();
            if changed {
                items.reverse();
            }
            Element::typed::<MenuBar>(MenuBarProps {
                title: Some(if changed { "2" } else { "1" }.into()),
                items: vec![MenuItem::submenu("file", "FILE", items)],
                ..Default::default()
            })
            .auto_focus()
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
        app_input::run_when(
            Reorder {
                changed: AtomicBool::new(false),
                calls: calls.clone(),
            },
            size,
            vec![
                ("1", super::key(KeyCode::Down)),
                ("ALPHA", super::key(KeyCode::Down)),
                ("BRAVO", super::key(KeyCode::F(2))),
                ("2", super::key(KeyCode::Enter)),
                ("FILE", None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), [("BRAVO", true)]);
    }
}

#[test]
fn menu_bar_direction_keys_do_not_invoke_leaf_actions() {
    use reactive_tui::event::types::KeyCode;
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let top = calls.clone();
        let nested = calls.clone();
        let menu = Element::typed::<MenuBar>(MenuBarProps {
            items: vec![
                MenuItem::action("top", "TOP", move || top.lock().unwrap().push("top")),
                MenuItem::submenu(
                    "file",
                    "FILE",
                    vec![MenuItem::action("leaf", "LEAF", move || {
                        nested.lock().unwrap().push("leaf")
                    })],
                ),
            ],
            ..Default::default()
        })
        .auto_focus();
        app_input::run_when(
            Control(menu),
            size,
            vec![
                ("TOP", super::key(KeyCode::Down)),
                ("TOP", super::key(KeyCode::Right)),
                ("FILE", super::key(KeyCode::Down)),
                ("LEAF", super::key(KeyCode::Right)),
                ("LEAF", super::key(KeyCode::Escape)),
                ("TOP", None),
            ],
        );
        assert!(calls.lock().unwrap().is_empty());
    }
}

#[test]
fn menu_bar_honors_public_state_for_an_initial_open_dropdown() {
    use reactive_tui::{component::Component, event::types::KeyCode, widgets::menu::MenuBarState};
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let props = MenuBarProps {
            items: vec![MenuItem::submenu(
                "file",
                "FILE",
                vec![
                    MenuItem::new("first", "FIRST"),
                    MenuItem::action("second", "SECOND", move || {
                        output.lock().unwrap().push("second")
                    }),
                ],
            )],
            ..Default::default()
        };
        let menu = MenuBar::new(props.clone())
            .render(
                &props,
                &MenuBarState {
                    selected_index: Some(0),
                    submenu_selected_index: Some(1),
                    dropdown_open: true,
                    ..Default::default()
                },
            )
            .auto_focus();
        app_input::run_when(
            Control(menu),
            size,
            vec![("SECOND", super::key(KeyCode::Enter)), ("FILE", None)],
        );
        assert_eq!(*calls.lock().unwrap(), ["second"]);
    }
}

#[test]
fn menu_bar_outside_click_dismisses_without_a_focus_change() {
    use reactive_tui::{
        builder::widgets::menu::{MenuBarBuilder, MenuItemBuilder},
        event::types::KeyCode,
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let opened = calls.clone();
        let closed = calls.clone();
        let menu = MenuBarBuilder::new()
            .item(
                MenuItemBuilder::new("file", "FILE")
                    .submenu(vec![MenuItemBuilder::new("child", "CHILD").build()])
                    .build(),
            )
            .on_dropdown_opened(move |_| opened.lock().unwrap().push("opened"))
            .on_dropdown_closed(move || closed.lock().unwrap().push("closed"))
            .build()
            .auto_focus();
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                ("FILE", super::key(KeyCode::Down)),
                ("CHILD", super::click(size.0 - 1, size.1 - 1)),
            ],
            "CHILD",
        );
        assert_eq!(*calls.lock().unwrap(), ["opened", "closed"]);
    }
}

#[test]
fn menu_bar_hover_opens_the_measured_nested_submenu() {
    use reactive_tui::event::types::{Event, KeyCode, MouseEvent, MouseEventKind, Position};
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let menu = Element::typed::<MenuBar>(MenuBarProps {
            items: vec![MenuItem::submenu(
                "file",
                "FILE",
                vec![MenuItem::submenu(
                    "recent",
                    "RECENT",
                    vec![MenuItem::action("open", "OPEN", move || {
                        output.lock().unwrap().push("open")
                    })],
                )],
            )],
            ..Default::default()
        })
        .auto_focus();
        let probe = app_input::run_when(
            Control(menu.clone()),
            size,
            vec![("FILE", super::key(KeyCode::Down)), ("RECENT", None)],
        );
        let (y, line) = probe
            .last()
            .unwrap()
            .text
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("RECENT"))
            .unwrap();
        let x = line.find("RECENT").unwrap();
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                ("FILE", super::key(KeyCode::Down)),
                (
                    "RECENT",
                    Some(Event::Mouse(MouseEvent::new(
                        MouseEventKind::Move,
                        Position::cell(x as u16, y as u16),
                    ))),
                ),
                ("OPEN", super::key(KeyCode::Enter)),
            ],
            "OPEN",
        );
        assert_eq!(*calls.lock().unwrap(), ["open"]);
    }
}

#[test]
fn menu_bar_opens_nested_actions_and_calls_once() {
    use super::key;
    use reactive_tui::event::types::KeyCode;
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let menu = Element::typed::<MenuBar>(MenuBarProps {
            items: vec![MenuItem::submenu(
                "file",
                "FILE",
                vec![
                    MenuItem::new("disabled", "DISABLED").enabled(false),
                    MenuItem::submenu(
                        "recent",
                        "RECENT",
                        vec![MenuItem::action("open", "OPEN", move || {
                            output.lock().unwrap().push("open")
                        })],
                    ),
                ],
            )],
            ..Default::default()
        })
        .auto_focus();
        let frames = app_input::run_when(
            Control(menu),
            size,
            vec![
                ("FILE", key(KeyCode::Down)),
                ("RECENT", key(KeyCode::Right)),
                ("OPEN", key(KeyCode::Enter)),
                ("FILE", None),
            ],
        );
        assert!(frames.iter().any(|frame| frame.text.contains("OPEN")));
        assert_eq!(*calls.lock().unwrap(), ["open"]);
    }
}

#[test]
fn menu_bar_builder_keeps_all_callbacks() {
    use reactive_tui::builder::widgets::menu::{MenuBarBuilder, MenuItemBuilder};
    use reactive_tui::event::types::KeyCode;
    use std::sync::{Arc, Mutex};
    let calls = Arc::new(Mutex::new(Vec::new()));
    let action = calls.clone();
    let selected = calls.clone();
    let opened = calls.clone();
    let closed = calls.clone();
    let menu = MenuBarBuilder::new()
        .item(
            MenuItemBuilder::new("file", "FILE")
                .submenu(vec![MenuItemBuilder::new("save", "SAVE")
                    .action(move || action.lock().unwrap().push("action".to_string()))
                    .build()])
                .build(),
        )
        .on_item_selected(move |id| selected.lock().unwrap().push(format!("selected:{id}")))
        .on_dropdown_opened(move |index| opened.lock().unwrap().push(format!("opened:{index}")))
        .on_dropdown_closed(move || closed.lock().unwrap().push("closed".into()))
        .build()
        .auto_focus();
    app_input::run_when(
        Control(menu),
        (48, 20),
        vec![
            ("FILE", super::key(KeyCode::Down)),
            ("SAVE", super::key(KeyCode::Enter)),
            ("FILE", None),
        ],
    );
    assert_eq!(
        *calls.lock().unwrap(),
        ["opened:0", "closed", "action", "selected:save"]
    );
}

#[test]
fn menu_bar_checkbox_keeps_state_between_openings() {
    use reactive_tui::event::types::KeyCode;
    use std::sync::{Arc, Mutex};
    let values = Arc::new(Mutex::new(Vec::new()));
    let output = values.clone();
    let menu = Element::typed::<MenuBar>(MenuBarProps {
        items: vec![MenuItem::submenu(
            "file",
            "FILE",
            vec![MenuItem::checkbox("check", "CHECK", false, move |value| {
                output.lock().unwrap().push(value)
            })],
        )],
        ..Default::default()
    })
    .auto_focus();
    app_input::run_when(
        Control(menu),
        (48, 20),
        vec![
            ("FILE", super::key(KeyCode::Down)),
            ("[ ] CHECK", super::key(KeyCode::Enter)),
            ("FILE", super::key(KeyCode::Down)),
            ("[x] CHECK", super::key(KeyCode::Enter)),
            ("FILE", super::key(KeyCode::Down)),
            ("[ ] CHECK", None),
        ],
    );
    assert_eq!(*values.lock().unwrap(), [true, false]);
}

#[test]
fn menu_bar_mouse_uses_painted_item_positions() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let output = calls.clone();
        let menu = Element::typed::<MenuBar>(MenuBarProps {
            title: Some("TITLE".into()),
            items: vec![MenuItem::action("save", "SAVE", move || {
                output.fetch_add(1, Ordering::SeqCst);
            })],
            ..Default::default()
        });
        let frames = app_input::run_when(Control(menu.clone()), size, vec![("SAVE", None)]);
        let (y, line) = frames
            .last()
            .unwrap()
            .text
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("SAVE"))
            .unwrap();
        let x = line.find("SAVE").unwrap();
        app_input::run_when(
            Control(menu),
            size,
            vec![("SAVE", super::click(x as u16, y as u16)), ("SAVE", None)],
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn menu_bar_scrolls_to_end_and_home_before_activation() {
    use reactive_tui::event::types::KeyCode;
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let items = (0..12)
            .map(|i| {
                let output = calls.clone();
                MenuItem::action(i.to_string(), format!("CHOICE{i:02}"), move || {
                    output.lock().unwrap().push(i)
                })
            })
            .collect();
        let menu = Element::typed::<MenuBar>(MenuBarProps {
            items: vec![MenuItem::submenu("file", "FILE", items)],
            max_dropdown_height: 2,
            ..Default::default()
        })
        .auto_focus();
        let frames = app_input::run_when(
            Control(menu),
            size,
            vec![
                ("FILE", super::key(KeyCode::Down)),
                ("CHOICE00", super::key(KeyCode::End)),
                ("CHOICE11", super::key(KeyCode::Home)),
                ("CHOICE00", super::key(KeyCode::Enter)),
                ("FILE", None),
            ],
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("CHOICE11") && !frame.text.contains("CHOICE00")));
        assert_eq!(*calls.lock().unwrap(), [0]);
    }
}

#[test]
fn menu_bar_disabled_and_empty_states_are_inert() {
    use reactive_tui::event::types::KeyCode;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let output = calls.clone();
    let menu = Element::typed::<MenuBar>(MenuBarProps {
        items: vec![MenuItem::action("item", "INERT", move || {
            output.fetch_add(1, Ordering::SeqCst);
        })],
        enabled: false,
        ..Default::default()
    })
    .auto_focus();
    app_input::run_when(
        Control(menu),
        (32, 12),
        vec![("INERT", super::key(KeyCode::Enter)), ("INERT", None)],
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let empty = Element::typed::<MenuBar>(MenuBarProps {
        title: Some("EMPTY".into()),
        ..Default::default()
    })
    .auto_focus();
    app_input::run_when(
        Control(empty),
        (32, 12),
        vec![
            ("EMPTY", super::key(KeyCode::End)),
            ("EMPTY", super::key(KeyCode::Enter)),
            ("EMPTY", None),
        ],
    );
}

#[test]
fn menu_bar_paints_actual_items_through_app() {
    for size in [(32, 12), (60, 20)] {
        let frames = app_input::run(
            Control(Element::typed::<MenuBar>(MenuBarProps {
                items: vec![MenuItem::new("file", "FILE")],
                ..Default::default()
            })),
            size,
            vec![(1, None)],
        );
        assert!(frames.last().unwrap().text.contains("FILE"));
    }
}

#[test]
fn popup_menu_selects_nested_items_and_closes_after_action() {
    use reactive_tui::event::types::KeyCode;
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let menu = Element::typed::<PopupMenu>(PopupMenuProps {
            visible: true,
            items: vec![
                MenuItem::new("disabled", "DISABLED").enabled(false),
                MenuItem::submenu(
                    "recent",
                    "RECENT",
                    vec![MenuItem::action("open", "OPEN", move || {
                        output.lock().unwrap().push("open")
                    })],
                ),
            ],
            ..Default::default()
        });
        let frames = app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                ("RECENT", super::key(KeyCode::Right)),
                ("OPEN", super::key(KeyCode::Enter)),
            ],
            "OPEN",
        );
        assert_eq!(*calls.lock().unwrap(), ["open"]);
        assert!(!frames.last().unwrap().text.contains("RECENT"));
        assert!(!frames.last().unwrap().text.contains("OPEN"));
    }
}

#[test]
fn popup_menu_builder_paints_and_delivers_all_callbacks() {
    use reactive_tui::{
        builder::widgets::menu::{MenuItemBuilder, PopupMenuBuilder},
        event::types::KeyCode,
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let action = calls.clone();
        let selected = calls.clone();
        let shown = calls.clone();
        let hidden = calls.clone();
        let menu = PopupMenuBuilder::new()
            .item(
                MenuItemBuilder::new("save", "SAVE")
                    .action(move || action.lock().unwrap().push("action".to_string()))
                    .build(),
            )
            .on_item_selected(move |id| selected.lock().unwrap().push(format!("selected:{id}")))
            .on_opened(move || shown.lock().unwrap().push("shown".into()))
            .on_closed(move || hidden.lock().unwrap().push("hidden".into()))
            .build();
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![("SAVE", super::key(KeyCode::Enter))],
            "SAVE",
        );
        assert_eq!(
            *calls.lock().unwrap(),
            ["shown", "action", "selected:save", "hidden"]
        );
    }
}

#[test]
fn menu_separators_paint_distinct_rules_and_keep_navigation_on_items() {
    use reactive_tui::{
        event::types::KeyCode,
        widgets::menu::{MenuItemType, MenuSeparator, MenuStyle},
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let mut items = vec![MenuItem {
            separator: MenuSeparator::Line,
            ..MenuItem::new("start", "START")
        }];
        for (id, kind) in [
            ("thick", MenuSeparator::ThickLine),
            ("double", MenuSeparator::DoubleLine),
            ("dashed", MenuSeparator::Dashed),
            ("dotted", MenuSeparator::Dotted),
        ] {
            items.push(MenuItem {
                id: id.into(),
                item_type: MenuItemType::Separator,
                separator: kind,
                ..Default::default()
            });
        }
        items.push(MenuItem::action("end", "END", move || {
            output.lock().unwrap().push("end")
        }));
        let menu = Element::typed::<PopupMenu>(PopupMenuProps {
            visible: true,
            items,
            style: MenuStyle {
                padding: 0,
                ..Default::default()
            },
            show_border: false,
            show_shadow: false,
            ..Default::default()
        });
        let frames = app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                ("START", super::key(KeyCode::Down)),
                ("END", super::key(KeyCode::Enter)),
            ],
            "END",
        );
        for glyph in ["─", "━", "═", "╌", "·"] {
            assert!(
                frames.iter().any(|frame| frame.text.contains(glyph)),
                "missing {glyph}"
            );
        }
        assert_eq!(*calls.lock().unwrap(), ["end"]);
    }
}

#[test]
fn popup_menu_uses_cell_padding_and_caps_fixed_width() {
    use reactive_tui::widgets::menu::{MenuStyle, PopupPlacement};
    for size in [(32, 12), (60, 20)] {
        for padding in [0, 2] {
            let frames = app_input::run_when(
                Control(Element::typed::<PopupMenu>(PopupMenuProps {
                    visible: true,
                    items: vec![MenuItem::new("item", "ITEM")],
                    placement: PopupPlacement::Position { x: 3, y: 2 },
                    width: Some(18),
                    show_border: false,
                    show_shadow: false,
                    style: MenuStyle {
                        padding,
                        max_width: Some(14),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
                size,
                vec![("ITEM", None)],
            );
            let frame = frames.last().unwrap();
            let (y, line) = frame
                .text
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains("ITEM"))
                .unwrap();
            assert_eq!(
                (line.find("ITEM").unwrap(), y),
                (4 + usize::from(padding), 2 + usize::from(padding))
            );
            assert_ne!(
                frame.screen.cell(2, 16).unwrap().bgcolor(),
                frame.screen.cell(0, 0).unwrap().bgcolor()
            );
            assert_eq!(
                frame.screen.cell(2, 17).unwrap().bgcolor(),
                frame.screen.cell(0, 0).unwrap().bgcolor()
            );
        }
    }
}

#[test]
fn menu_builder_style_colors_and_spacing_reach_the_painted_items() {
    use reactive_tui::builder::widgets::menu::{MenuItemBuilder, MenuStyle, PopupMenuBuilder};
    let menu = PopupMenuBuilder::new()
        .style(MenuStyle {
            background: Some("#123456".into()),
            text_color: Some("#abcdef".into()),
            selected_background: Some("#345678".into()),
            selected_text_color: Some("#fedcba".into()),
            disabled_text_color: Some("#102030".into()),
            padding: Some("p-0".into()),
            margin: Some("m-0".into()),
            border: Some("border-0".into()),
        })
        .item(MenuItemBuilder::new("ink", "INK").build())
        .item(MenuItemBuilder::new("off", "OFF").enabled(false).build())
        .build();
    let frames = app_input::run_when(Control(menu), (40, 12), vec![("OFF", None)]);
    let frame = frames.last().unwrap();
    for (text, foreground, background) in [
        (
            "INK",
            vt100::Color::Rgb(254, 220, 186),
            vt100::Color::Rgb(52, 86, 120),
        ),
        (
            "OFF",
            vt100::Color::Rgb(16, 32, 48),
            vt100::Color::Rgb(18, 52, 86),
        ),
    ] {
        let (y, line) = frame
            .text
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains(text))
            .unwrap();
        let x = line.find(text).unwrap();
        let cell = frame.screen.cell(y as u16, x as u16).unwrap();
        assert_eq!(cell.fgcolor(), foreground);
        assert_eq!(cell.bgcolor(), background);
    }
}

#[test]
fn popup_and_dialog_menu_clicks_use_the_painted_overlay_position() {
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        for dialog in [false, true] {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let output = calls.clone();
            let item = MenuItem::action("choose", "CHOOSE", move || {
                output.lock().unwrap().push("choose")
            });
            let menu = if dialog {
                Element::typed::<DialogMenu>(DialogMenuProps {
                    visible: true,
                    items: vec![item],
                    ..Default::default()
                })
            } else {
                Element::typed::<PopupMenu>(PopupMenuProps {
                    visible: true,
                    items: vec![item],
                    placement: reactive_tui::widgets::menu::PopupPlacement::Position { x: 7, y: 3 },
                    ..Default::default()
                })
            };
            let frames = app_input::run_when(Control(menu.clone()), size, vec![("CHOOSE", None)]);
            let (y, line) = frames
                .last()
                .unwrap()
                .text
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains("CHOOSE"))
                .unwrap();
            let x = line.find("CHOOSE").unwrap();
            app_input::run_until_hidden(
                Control(menu),
                size,
                vec![("CHOOSE", super::click(x as u16, y as u16))],
                "CHOOSE",
            );
            assert_eq!(*calls.lock().unwrap(), ["choose"]);
        }
    }
}

#[test]
fn popup_menu_paints_actual_items_through_app() {
    for size in [(32, 12), (60, 20)] {
        let frames = app_input::run_when(
            Control(Element::typed::<PopupMenu>(PopupMenuProps {
                visible: true,
                items: vec![MenuItem::new("open", "OPEN")],
                ..Default::default()
            })),
            size,
            vec![("OPEN", None)],
        );
        assert!(frames.last().unwrap().text.contains("OPEN"));
    }
}

#[test]
fn context_menu_long_press_opens_without_another_mouse_event() {
    use reactive_tui::event::types::{Event, MouseButton, MouseEvent, MouseEventKind, Position};
    for size in [(32, 12), (60, 20)] {
        let menu = Element::typed::<ContextMenu>(ContextMenuProps {
            items: vec![MenuItem::new("copy", "COPY")],
            show_on_long_press: true,
            long_press_duration: 20,
            ..Default::default()
        });
        let frames = app_input::run_when(
            Control(menu),
            size,
            vec![
                (
                    "",
                    Some(Event::Mouse(
                        MouseEvent::new(MouseEventKind::Down, Position::cell(3, 2))
                            .with_button(MouseButton::Left),
                    )),
                ),
                ("COPY", None),
            ],
        );
        let last = &frames.last().unwrap().text;
        assert!(
            last.contains("COPY"),
            "long press opens the context menu at {size:?}:\n{last}"
        );
    }
}

#[test]
fn context_menu_builder_delivers_one_open_action_selection_and_close() {
    use reactive_tui::{
        builder::widgets::menu::{ContextMenuBuilder, MenuItemBuilder},
        event::types::{Event, KeyCode, MouseButton, MouseEvent, MouseEventKind, Position},
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let action = calls.clone();
        let selected = calls.clone();
        let shown = calls.clone();
        let hidden = calls.clone();
        let menu = ContextMenuBuilder::new()
            .item(
                MenuItemBuilder::new("copy", "COPY")
                    .action(move || action.lock().unwrap().push("action".to_string()))
                    .build(),
            )
            .on_item_selected(move |id| selected.lock().unwrap().push(format!("selected:{id}")))
            .on_opened(move || shown.lock().unwrap().push("shown".into()))
            .on_closed(move || hidden.lock().unwrap().push("hidden".into()))
            .build();
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                (
                    "",
                    Some(Event::Mouse(
                        MouseEvent::new(MouseEventKind::Down, Position::cell(3, 2))
                            .with_button(MouseButton::Right),
                    )),
                ),
                ("COPY", super::key(KeyCode::Enter)),
            ],
            "COPY",
        );
        assert_eq!(
            *calls.lock().unwrap(),
            ["shown", "action", "selected:copy", "hidden"]
        );
    }
}

#[test]
fn context_menu_right_click_paints_actual_items_through_app() {
    use reactive_tui::event::types::{Event, MouseButton, MouseEvent, MouseEventKind, Position};
    for size in [(32, 12), (60, 20)] {
        let frames = app_input::run_when(
            Control(Element::typed::<ContextMenu>(ContextMenuProps {
                items: vec![MenuItem::new("copy", "COPY")],
                ..Default::default()
            })),
            size,
            vec![
                (
                    "",
                    Some(Event::Mouse(
                        MouseEvent::new(MouseEventKind::Down, Position::cell(3, 2))
                            .with_button(MouseButton::Right),
                    )),
                ),
                ("COPY", None),
            ],
        );
        assert!(frames.last().unwrap().text.contains("COPY"));
    }
}

#[test]
fn dialog_menu_selection_reports_action_selection_and_hide_once() {
    use reactive_tui::{
        component::Component, event::types::KeyCode, widgets::menu::DialogMenuState,
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let action = calls.clone();
        let selected = calls.clone();
        let shown = calls.clone();
        let hidden = calls.clone();
        let props = DialogMenuProps {
            visible: true,
            title: Some("SELECT".into()),
            items: vec![
                MenuItem::new("disabled", "DISABLED").enabled(false),
                MenuItem::action("choose", "CHOOSE", move || {
                    action.lock().unwrap().push("action".to_string())
                }),
            ],
            ..Default::default()
        };
        let menu = DialogMenu::new(props.clone())
            .with_on_item_selected(move |id| {
                selected.lock().unwrap().push(format!("selected:{id}"))
            })
            .with_on_show(move || shown.lock().unwrap().push("shown".into()))
            .with_on_hide(move || hidden.lock().unwrap().push("hidden".into()))
            .render(&props, &DialogMenuState::default());
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![("CHOOSE", super::key(KeyCode::Enter))],
            "SELECT",
        );
        assert_eq!(
            *calls.lock().unwrap(),
            ["shown", "action", "selected:choose", "hidden"]
        );
    }
}

#[test]
fn dialog_menu_multi_selection_toggles_and_confirms_ids() {
    use reactive_tui::{
        component::Component,
        event::types::KeyCode,
        widgets::menu::{DialogMenuState, DialogMenuType},
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let values = Arc::new(Mutex::new(Vec::new()));
        let output = values.clone();
        let props = DialogMenuProps {
            visible: true,
            dialog_type: DialogMenuType::MultiSelection,
            items: vec![MenuItem::new("one", "ONE"), MenuItem::new("two", "TWO")],
            ..Default::default()
        };
        let menu = DialogMenu::new(props.clone())
            .with_on_confirmed(move |ids| output.lock().unwrap().push(ids))
            .render(&props, &DialogMenuState::default());
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                ("[ ] ONE", super::key(KeyCode::Space)),
                ("[x] ONE", super::key(KeyCode::Down)),
                ("TWO", super::key(KeyCode::Enter)),
                ("[x] TWO", super::key(KeyCode::Tab)),
            ],
            "Confirm",
        );
        assert_eq!(
            *values.lock().unwrap(),
            [vec!["one".to_string(), "two".to_string()]]
        );
    }
}

#[test]
fn dialog_menu_input_edits_unicode_and_submits_current_text() {
    use reactive_tui::{
        component::Component,
        event::types::KeyCode,
        widgets::menu::{DialogMenuState, DialogMenuType},
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let values = Arc::new(Mutex::new(Vec::new()));
        let output = values.clone();
        let props = DialogMenuProps {
            visible: true,
            dialog_type: DialogMenuType::Input,
            title: Some("PROMPT".into()),
            ..Default::default()
        };
        let menu = DialogMenu::new(props.clone())
            .with_on_input_submitted(move |value| output.lock().unwrap().push(value.to_owned()))
            .render(&props, &DialogMenuState::default());
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                ("PROMPT", super::key(KeyCode::Char('界'))),
                ("界", super::key(KeyCode::Char('e'))),
                ("界e", super::key(KeyCode::Char('\u{301}'))),
                ("界e\u{301}", super::key(KeyCode::Backspace)),
                ("界", super::key(KeyCode::Enter)),
            ],
            "PROMPT",
        );
        assert_eq!(*values.lock().unwrap(), ["界"]);
    }
}

#[test]
fn dialog_menu_confirmation_defaults_and_escape_cancellation_are_distinct() {
    use reactive_tui::{
        component::Component,
        event::types::KeyCode,
        widgets::menu::{DialogMenuState, DialogMenuType},
    };
    use std::sync::{Arc, Mutex};
    for key in [KeyCode::Enter, KeyCode::Escape] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let yes = calls.clone();
        let no = calls.clone();
        let cancelled = calls.clone();
        let props = DialogMenuProps {
            visible: true,
            dialog_type: DialogMenuType::Confirmation,
            title: Some("CONFIRM".into()),
            items: vec![
                MenuItem::action("no", "NO", move || no.lock().unwrap().push("no")),
                MenuItem::action("yes", "YES", move || yes.lock().unwrap().push("yes")),
            ],
            default_button: Some(1),
            cancel_button: Some(0),
            ..Default::default()
        };
        let menu = DialogMenu::new(props.clone())
            .with_on_cancelled(move || cancelled.lock().unwrap().push("cancelled"))
            .render(&props, &DialogMenuState::default());
        app_input::run_until_hidden(
            Control(menu),
            (48, 16),
            vec![("CONFIRM", super::key(key.clone()))],
            "CONFIRM",
        );
        assert_eq!(
            *calls.lock().unwrap(),
            if key == KeyCode::Enter {
                vec!["yes"]
            } else {
                vec!["no", "cancelled"]
            }
        );
    }
}

#[test]
fn dialog_menu_paints_actual_items_through_app() {
    for size in [(32, 12), (60, 20)] {
        let frames = app_input::run_when(
            Control(Element::typed::<DialogMenu>(DialogMenuProps {
                visible: true,
                items: vec![MenuItem::new("choose", "CHOOSE")],
                ..Default::default()
            })),
            size,
            vec![("CHOOSE", None)],
        );
        assert!(frames.last().unwrap().text.contains("CHOOSE"));
    }
}

#[test]
fn popup_menu_hover_opens_and_preserves_the_nested_panel() {
    use reactive_tui::event::types::{Event, KeyCode, MouseEvent, MouseEventKind, Position};
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let menu = Element::typed::<PopupMenu>(PopupMenuProps {
            visible: true,
            items: vec![MenuItem::submenu(
                "recent",
                "RECENT",
                vec![MenuItem::action("open", "OPEN", move || {
                    output.lock().unwrap().push("open")
                })],
            )],
            ..Default::default()
        });
        let probe = app_input::run_when(Control(menu.clone()), size, vec![("RECENT", None)]);
        let (y, line) = probe
            .last()
            .unwrap()
            .text
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("RECENT"))
            .unwrap();
        let x = line.find("RECENT").unwrap();
        let hover = || {
            Some(Event::Mouse(MouseEvent::new(
                MouseEventKind::Move,
                Position::cell(x as u16, y as u16),
            )))
        };
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                ("RECENT", hover()),
                ("OPEN", hover()),
                ("OPEN", super::key(KeyCode::Enter)),
            ],
            "OPEN",
        );
        assert_eq!(*calls.lock().unwrap(), ["open"]);
    }
}

#[test]
fn popup_menu_scroll_keeps_selected_rows_visible_with_separators() {
    use reactive_tui::{event::types::KeyCode, widgets::menu::MenuSeparator};
    use std::sync::{Arc, Mutex};
    for size in [(32, 8), (60, 10)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let items = (0..20)
            .map(|i| {
                let output = calls.clone();
                MenuItem {
                    separator: MenuSeparator::Line,
                    ..MenuItem::action(format!("row{i}"), format!("ROW{i:02}"), move || {
                        output.lock().unwrap().push(i)
                    })
                }
            })
            .collect();
        let menu = Element::typed::<PopupMenu>(PopupMenuProps {
            visible: true,
            items,
            ..Default::default()
        });
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                ("ROW00", super::key(KeyCode::End)),
                ("ROW19", super::key(KeyCode::Enter)),
            ],
            "ROW19",
        );
        assert_eq!(*calls.lock().unwrap(), [19]);
    }
}

#[test]
fn popup_and_dialog_menu_outside_click_ignores_closed_submenu_bounds() {
    use reactive_tui::event::types::KeyCode;
    for size in [(40, 14), (70, 22)] {
        for dialog in [false, true] {
            let items = vec![MenuItem::submenu(
                "recent",
                "RECENT",
                vec![MenuItem::new("open", "OPEN")],
            )];
            let menu = if dialog {
                Element::typed::<DialogMenu>(DialogMenuProps {
                    visible: true,
                    modal: false,
                    close_on_outside_click: true,
                    centered: false,
                    position: Some((1, 1)),
                    width: Some(14),
                    items,
                    ..Default::default()
                })
            } else {
                Element::typed::<PopupMenu>(PopupMenuProps {
                    visible: true,
                    width: Some(14),
                    items,
                    ..Default::default()
                })
            };
            let frames = app_input::run_when(
                Control(menu.clone()),
                size,
                vec![("RECENT", super::key(KeyCode::Right)), ("OPEN", None)],
            );
            let (y, line) = frames
                .last()
                .unwrap()
                .text
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains("OPEN"))
                .unwrap();
            let x = line[..line.find("OPEN").unwrap()].chars().count();
            let frames = app_input::run_until_hidden(
                Control(menu),
                size,
                vec![
                    ("RECENT", super::key(KeyCode::Right)),
                    ("OPEN", super::key(KeyCode::Left)),
                    ("RECENT", super::click(x as u16, y as u16)),
                ],
                "RECENT",
            );
            let last = &frames.last().unwrap().text;
            assert!(
                !last.contains("RECENT") && !last.contains("OPEN"),
                "click on the closed submenu's old cell dismisses the menu (dialog={dialog}) at {size:?}:\n{last}"
            );
        }
    }
}

#[test]
fn menu_wheel_navigation_reaches_actions_through_app() {
    use reactive_tui::event::types::{
        Event, KeyCode, MouseEvent, MouseEventKind, Position, WheelDelta, WheelEvent,
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        for family in 0..3 {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let output = calls.clone();
            let items = vec![
                MenuItem::new("first", "FIRST"),
                MenuItem::action("second", "SECOND", move || {
                    output.lock().unwrap().push("second")
                }),
            ];
            let menu = match family {
                0 => Element::typed::<MenuBar>(MenuBarProps {
                    items: vec![MenuItem::submenu("file", "FILE", items)],
                    ..Default::default()
                })
                .auto_focus(),
                1 => Element::typed::<PopupMenu>(PopupMenuProps {
                    visible: true,
                    items,
                    ..Default::default()
                }),
                _ => Element::typed::<DialogMenu>(DialogMenuProps {
                    visible: true,
                    items,
                    ..Default::default()
                }),
            };
            let mut steps = Vec::new();
            if family == 0 {
                steps.push(("FILE", super::key(KeyCode::Down)));
            }
            steps.push(("FIRST", None));
            let frames = app_input::run_when(Control(menu.clone()), size, steps);
            let (y, line) = frames
                .last()
                .unwrap()
                .text
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains("FIRST"))
                .unwrap();
            let x = line.find("FIRST").unwrap();
            let mut wheel =
                MouseEvent::new(MouseEventKind::Wheel, Position::cell(x as u16, y as u16));
            wheel.wheel = Some(WheelEvent {
                delta: WheelDelta::Lines { x: 0.0, y: 1.0 },
                phase: reactive_tui::event::types::WheelPhase::Changed,
            });
            let mut steps = Vec::new();
            if family == 0 {
                steps.push(("FILE", super::key(KeyCode::Down)));
            }
            steps.extend([
                ("FIRST", Some(Event::Mouse(wheel))),
                ("SECOND", super::key(KeyCode::Enter)),
            ]);
            app_input::run_until_hidden(Control(menu), size, steps, "SECOND");
            assert_eq!(*calls.lock().unwrap(), ["second"], "family {family}");
        }
    }
}

#[test]
fn dialog_menu_input_pastes_scrolls_and_deletes_whole_graphemes() {
    use reactive_tui::{
        component::Component,
        event::types::{Event, KeyCode, PasteEvent},
        widgets::menu::{DialogMenuState, DialogMenuType},
    };
    use std::sync::{Arc, Mutex};
    for size in [(24, 10), (48, 14)] {
        let values = Arc::new(Mutex::new(Vec::new()));
        let output = values.clone();
        let props = DialogMenuProps {
            visible: true,
            dialog_type: DialogMenuType::Input,
            title: Some("PROMPT".into()),
            width: Some(18),
            ..Default::default()
        };
        let menu = DialogMenu::new(props.clone())
            .with_on_input_submitted(move |value| output.lock().unwrap().push(value.to_owned()))
            .render(&props, &DialogMenuState::default());
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                (
                    "PROMPT",
                    Some(Event::Paste(PasteEvent::new(
                        "界e\u{301}abcdefghijklmnopqrstEND".into(),
                    ))),
                ),
                ("END", super::key(KeyCode::Home)),
                ("界", super::key(KeyCode::Delete)),
                ("e\u{301}", super::key(KeyCode::Delete)),
                ("abc", super::key(KeyCode::End)),
                ("END", super::key(KeyCode::Enter)),
            ],
            "PROMPT",
        );
        assert_eq!(*values.lock().unwrap(), ["abcdefghijklmnopqrstEND"]);
    }
}

#[test]
fn menu_page_navigation_uses_the_presented_row_count() {
    use reactive_tui::event::types::KeyCode;
    use std::sync::{Arc, Mutex};
    for size in [(32, 10), (60, 14)] {
        for family in 0..3 {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let items = (0..30)
                .map(|index| {
                    let output = calls.clone();
                    MenuItem::action(format!("row{index}"), format!("ROW{index:02}"), move || {
                        output.lock().unwrap().push(index)
                    })
                })
                .collect();
            let menu = match family {
                0 => Element::typed::<MenuBar>(MenuBarProps {
                    items: vec![MenuItem::submenu("file", "FILE", items)],
                    ..Default::default()
                })
                .auto_focus(),
                1 => Element::typed::<PopupMenu>(PopupMenuProps {
                    visible: true,
                    items,
                    ..Default::default()
                }),
                _ => Element::typed::<DialogMenu>(DialogMenuProps {
                    visible: true,
                    items,
                    ..Default::default()
                }),
            };
            let mut steps = Vec::new();
            if family == 0 {
                steps.push(("FILE", super::key(KeyCode::Down)));
            }
            steps.push(("ROW00", None));
            let frames = app_input::run_when(Control(menu.clone()), size, steps);
            let visible = frames
                .last()
                .unwrap()
                .text
                .lines()
                .filter(|line| line.contains("ROW"))
                .count();
            assert!(visible > 1 && visible < 30);
            let mut steps = Vec::new();
            if family == 0 {
                steps.push(("FILE", super::key(KeyCode::Down)));
            }
            steps.extend([
                ("ROW00", super::key(KeyCode::PageDown)),
                ("ROW", super::key(KeyCode::Enter)),
            ]);
            app_input::run_until_hidden(Control(menu), size, steps, "ROW");
            assert_eq!(
                *calls.lock().unwrap(),
                [visible],
                "family {family}, size {size:?}"
            );
        }
    }
}

#[test]
fn dialog_menu_hover_selects_the_painted_item_without_activating_it() {
    use reactive_tui::event::types::{Event, KeyCode, MouseEvent, MouseEventKind, Position};
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let menu = Element::typed::<DialogMenu>(DialogMenuProps {
            visible: true,
            items: vec![
                MenuItem::new("first", "FIRST"),
                MenuItem::action("second", "SECOND", move || {
                    output.lock().unwrap().push("second")
                }),
            ],
            ..Default::default()
        });
        let frames = app_input::run_when(Control(menu.clone()), size, vec![("SECOND", None)]);
        let (y, line) = frames
            .last()
            .unwrap()
            .text
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("SECOND"))
            .unwrap();
        let x = line.find("SECOND").unwrap();
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                (
                    "SECOND",
                    Some(Event::Mouse(MouseEvent::new(
                        MouseEventKind::Move,
                        Position::cell(x as u16, y as u16),
                    ))),
                ),
                ("SECOND", super::key(KeyCode::Enter)),
            ],
            "SECOND",
        );
        assert_eq!(*calls.lock().unwrap(), ["second"]);
    }
}

#[test]
fn popup_and_dialog_menus_reopen_and_restore_the_trigger_focus() {
    use reactive_tui::{
        app::RootComponent,
        component::Component,
        event::types::KeyCode,
        widgets::menu::{DialogMenuState, PopupMenuState},
    };
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    struct Reopen {
        family: usize,
        visible: Arc<AtomicBool>,
        calls: Arc<Mutex<Vec<&'static str>>>,
    }
    impl RootComponent for Reopen {
        fn render(&self) -> Element {
            let visible = self.visible.clone();
            let trigger_calls = self.calls.clone();
            let trigger = reactive_tui::builder::button()
                .text("OPEN")
                .on_click(move || {
                    trigger_calls.lock().unwrap().push("open");
                    visible.store(true, Ordering::SeqCst);
                })
                .build()
                .auto_focus();
            let output = self.calls.clone();
            let items = vec![MenuItem::action("select", "SELECT", move || {
                output.lock().unwrap().push("select")
            })];
            let visible = self.visible.clone();
            let hidden_calls = self.calls.clone();
            let hidden = move || {
                hidden_calls.lock().unwrap().push("hide");
                visible.store(false, Ordering::SeqCst);
            };
            let menu = if self.family == 0 {
                let props = PopupMenuProps {
                    visible: self.visible.load(Ordering::SeqCst),
                    items,
                    ..Default::default()
                };
                PopupMenu::new(props.clone())
                    .with_on_hide(hidden)
                    .render(&props, &PopupMenuState::default())
            } else {
                let props = DialogMenuProps {
                    visible: self.visible.load(Ordering::SeqCst),
                    modal: self.family == 1,
                    items,
                    ..Default::default()
                };
                DialogMenu::new(props.clone())
                    .with_on_hide(hidden)
                    .render(&props, &DialogMenuState::default())
            };
            reactive_tui::builder::div()
                .children(vec![trigger, menu.with_key("menu")])
                .build()
        }
        fn wake_driven(&self) -> bool {
            true
        }
    }
    for size in [(32, 12), (60, 20)] {
        for family in 0..3 {
            let calls = Arc::new(Mutex::new(Vec::new()));
            app_input::run_visibility(
                Reopen {
                    family,
                    visible: Arc::new(AtomicBool::new(false)),
                    calls: calls.clone(),
                },
                size,
                vec![
                    ("OPEN", Some("SELECT"), super::key(KeyCode::Enter)),
                    ("SELECT", None, super::key(KeyCode::Escape)),
                    ("OPEN", Some("SELECT"), super::key(KeyCode::Enter)),
                    ("SELECT", None, super::key(KeyCode::Enter)),
                    ("OPEN", Some("SELECT"), super::key(KeyCode::Enter)),
                    ("SELECT", None, super::key(KeyCode::Escape)),
                    ("OPEN", Some("SELECT"), None),
                ],
            );
            assert_eq!(
                *calls.lock().unwrap(),
                ["open", "hide", "open", "select", "hide", "open", "hide"],
                "family {family}"
            );
        }
    }
}

#[test]
fn dialog_menu_nested_multi_selection_paints_and_survives_prop_updates() {
    use reactive_tui::{
        app::RootComponent,
        component::Component,
        event::{
            router::EventResult,
            types::{Event, KeyCode},
        },
        widgets::menu::{DialogMenuState, DialogMenuType},
    };
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    struct Nested {
        changed: AtomicBool,
        disable_group: bool,
        values: Arc<Mutex<Vec<Vec<String>>>>,
    }
    impl RootComponent for Nested {
        fn render(&self) -> Element {
            let props = DialogMenuProps {
                visible: true,
                title: Some(
                    if self.changed.load(Ordering::SeqCst) {
                        "UPDATED"
                    } else {
                        "INITIAL"
                    }
                    .into(),
                ),
                dialog_type: DialogMenuType::MultiSelection,
                width: Some(14),
                items: vec![
                    MenuItem::submenu("group", "GROUP", vec![MenuItem::new("one", "ONE")])
                        .enabled(!(self.disable_group && self.changed.load(Ordering::SeqCst))),
                ],
                ..Default::default()
            };
            let values = self.values.clone();
            DialogMenu::new(props.clone())
                .with_on_confirmed(move |ids| values.lock().unwrap().push(ids))
                .render(&props, &DialogMenuState::default())
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
    for (size, disable_group) in [(32, 12), (60, 20)]
        .into_iter()
        .flat_map(|size| [false, true].map(|disabled| (size, disabled)))
    {
        let values = Arc::new(Mutex::new(Vec::new()));
        app_input::run_until_hidden(
            Nested {
                changed: AtomicBool::new(false),
                disable_group,
                values: values.clone(),
            },
            size,
            vec![
                ("GROUP", super::key(KeyCode::Right)),
                ("[ ] ONE", super::key(KeyCode::Space)),
                ("[x] ONE", super::key(KeyCode::F(2))),
                ("UPDATED", super::key(KeyCode::Left)),
                ("GROUP", super::key(KeyCode::Tab)),
            ],
            "Confirm",
        );
        assert_eq!(
            *values.lock().unwrap(),
            [if disable_group {
                vec![]
            } else {
                vec!["one".to_string()]
            }]
        );
    }
}

#[test]
fn disabled_and_empty_menus_ignore_activation_through_app() {
    use reactive_tui::event::types::KeyCode;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(32, 12), (60, 20)] {
        for family in 0..3 {
            for empty in [false, true] {
                let calls = Arc::new(AtomicUsize::new(0));
                let output = calls.clone();
                let items = if empty {
                    vec![]
                } else {
                    vec![MenuItem::action("blocked", "BLOCKED", move || {
                        output.fetch_add(1, Ordering::SeqCst);
                    })]
                };
                let menu = match family {
                    0 => Element::typed::<MenuBar>(MenuBarProps {
                        enabled: empty,
                        items,
                        title: Some("MENU".into()),
                        ..Default::default()
                    })
                    .auto_focus(),
                    1 => Element::typed::<PopupMenu>(PopupMenuProps {
                        visible: true,
                        enabled: empty,
                        items,
                        ..Default::default()
                    }),
                    _ => Element::typed::<DialogMenu>(DialogMenuProps {
                        visible: true,
                        enabled: empty,
                        items,
                        title: Some("MENU".into()),
                        ..Default::default()
                    }),
                };
                let root = reactive_tui::builder::stack()
                    .child(Element::text("READY"))
                    .child(menu)
                    .build();
                app_input::run_when(
                    Control(root),
                    size,
                    vec![
                        ("READY", super::key(KeyCode::Enter)),
                        ("READY", super::key(KeyCode::Down)),
                        ("READY", super::key(KeyCode::Space)),
                        ("READY", super::key(KeyCode::Right)),
                        ("READY", super::key(KeyCode::Enter)),
                        ("READY", None),
                    ],
                );
                assert_eq!(
                    calls.load(Ordering::SeqCst),
                    0,
                    "family {family}, empty {empty}"
                );
            }
        }
    }
}

#[test]
fn popup_menu_placements_use_screen_coordinates() {
    use reactive_tui::{
        component::Component,
        widgets::menu::{MenuStyle, PopupMenuState, PopupPlacement},
    };
    for size in [(32, 12), (60, 20)] {
        for (placement, expected) in [
            (PopupPlacement::Cursor, (13, 6)),
            (PopupPlacement::Position { x: 12, y: 6 }, (13, 6)),
            (
                PopupPlacement::Widget {
                    x: 12,
                    y: 6,
                    width: 8,
                    height: 2,
                },
                (13, 8),
            ),
            (
                PopupPlacement::Below {
                    x: 12,
                    y: 6,
                    width: 8,
                },
                (13, 6),
            ),
            (
                PopupPlacement::Above {
                    x: 12,
                    y: 6,
                    width: 8,
                },
                (13, 5),
            ),
            (
                PopupPlacement::Left {
                    x: 12,
                    y: 6,
                    height: 2,
                },
                (5, 6),
            ),
            (
                PopupPlacement::Right {
                    x: 12,
                    y: 6,
                    height: 2,
                },
                (13, 6),
            ),
        ] {
            let props = PopupMenuProps {
                visible: true,
                placement: placement.clone(),
                width: Some(8),
                style: MenuStyle {
                    padding: 0,
                    min_width: 0,
                    ..Default::default()
                },
                show_border: false,
                show_shadow: false,
                items: vec![MenuItem::new("item", "ITEM")],
                ..Default::default()
            };
            let menu = PopupMenu::new(props.clone()).render(
                &props,
                &PopupMenuState {
                    mouse_position: Some((12, 6)),
                    ..Default::default()
                },
            );
            let frames = app_input::run_when(Control(menu), size, vec![("ITEM", None)]);
            let frame = &frames.last().unwrap().text;
            let (y, line) = frame
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains("ITEM"))
                .unwrap();
            assert_eq!(
                (line.find("ITEM").unwrap(), y),
                expected,
                "{placement:?}: {frame}"
            );
        }
    }
}

#[test]
fn menu_overlay_resize_repositions_paint_and_mouse_targets() {
    use reactive_tui::{
        component::Component,
        event::types::{Event, ResizeEvent},
        widgets::menu::{ContextMenuState, MenuStyle, PopupPlacement},
    };
    use std::sync::{Arc, Mutex};
    for (size, family) in [(32, 12), (60, 20)]
        .into_iter()
        .flat_map(|size| (0..3).map(move |family| (size, family)))
    {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let props = PopupMenuProps {
            visible: true,
            placement: PopupPlacement::Position {
                x: size.0 - 2,
                y: size.1 - 2,
            },
            width: Some(8),
            style: MenuStyle {
                padding: 0,
                min_width: 0,
                ..Default::default()
            },
            show_border: false,
            show_shadow: false,
            items: vec![
                MenuItem::new("first", "FIRST"),
                MenuItem::action("second", "SECOND", move || {
                    output.lock().unwrap().push("second")
                }),
            ],
            ..Default::default()
        };
        let menu = match family {
            0 => Element::typed::<PopupMenu>(props),
            1 => {
                let config = ContextMenuProps {
                    items: props.items,
                    width: props.width,
                    style: props.style,
                    show_border: false,
                    show_shadow: false,
                    ..Default::default()
                };
                ContextMenu::new(config.clone()).render(
                    &config,
                    &ContextMenuState {
                        is_visible: true,
                        position: Some((size.0 - 2, size.1 - 2)),
                        ..Default::default()
                    },
                )
            }
            _ => Element::typed::<DialogMenu>(DialogMenuProps {
                items: props.items,
                width: props.width,
                style: props.style,
                visible: true,
                centered: false,
                position: Some((size.0 - 2, size.1 - 2)),
                show_close_button: false,
                show_border: false,
                show_shadow: false,
                ..Default::default()
            }),
        };
        let frames = app_input::run(
            Control(menu),
            size,
            vec![
                (3, Some(Event::Resize(ResizeEvent::new(20, 8)))),
                (5, super::click(13, 7)),
                (6, None),
            ],
        );
        assert_eq!(
            frames[2]
                .screen
                .cell(size.1 - 2, size.0 - 7)
                .unwrap()
                .contents(),
            "F"
        );
        // The first frame after the resize already shows the new position.
        assert_eq!(
            frames[3].screen.cell(6, 13).unwrap().contents(),
            "F",
            "{}",
            frames[3].text
        );
        assert_eq!(*calls.lock().unwrap(), ["second"]);
        assert!(!frames.last().unwrap().text.contains("SECOND"));
    }
}

#[test]
fn menu_prop_replacement_and_remount_keep_callbacks_and_state_local() {
    use reactive_tui::{
        app::RootComponent,
        component::Component,
        event::{
            router::EventResult,
            types::{Event, KeyCode},
        },
        widgets::menu::{DialogMenuState, PopupMenuState},
    };
    use std::sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    };
    struct ChangingMenu {
        stage: AtomicU8,
        dialog: bool,
        calls: Arc<Mutex<Vec<String>>>,
    }
    impl RootComponent for ChangingMenu {
        fn render(&self) -> Element {
            let stage = self.stage.load(Ordering::SeqCst);
            let mut root = reactive_tui::builder::div()
                .class("w-full h-full")
                .child(Element::text(format!("ROOT{stage}")));
            if stage != 3 {
                let mut items: Vec<_> = ["A", "B"]
                    .into_iter()
                    .map(|id| {
                        let calls = self.calls.clone();
                        MenuItem::action(id, format!("{id}{stage}"), move || {
                            calls.lock().unwrap().push(format!("action:{id}:{stage}"))
                        })
                    })
                    .collect();
                if stage == 2 {
                    items.reverse();
                }
                let selected = self.calls.clone();
                let shown = self.calls.clone();
                let hidden = self.calls.clone();
                let on_selected = move |id: &str| {
                    selected
                        .lock()
                        .unwrap()
                        .push(format!("selected:{id}:{stage}"))
                };
                let on_shown = move || shown.lock().unwrap().push(format!("shown:{stage}"));
                let on_hidden = move || hidden.lock().unwrap().push(format!("hidden:{stage}"));
                let menu = if self.dialog {
                    let props = DialogMenuProps {
                        visible: true,
                        items,
                        ..Default::default()
                    };
                    DialogMenu::new(props.clone())
                        .with_on_item_selected(on_selected)
                        .with_on_show(on_shown)
                        .with_on_hide(on_hidden)
                        .render(&props, &DialogMenuState::default())
                } else {
                    let props = PopupMenuProps {
                        visible: true,
                        items,
                        ..Default::default()
                    };
                    PopupMenu::new(props.clone())
                        .with_on_item_selected(on_selected)
                        .with_on_show(on_shown)
                        .with_on_hide(on_hidden)
                        .render(&props, &PopupMenuState::default())
                };
                root = root.child(menu.with_key("menu"));
            }
            root.build()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if let Event::Key(key) = event {
                if let KeyCode::F(stage @ 2..=5) = key.code {
                    self.stage.store(stage, Ordering::SeqCst);
                    return EventResult::Consumed;
                }
            }
            EventResult::Ignored
        }
    }
    for size in [(32, 12), (60, 20)] {
        for dialog in [false, true] {
            // Fresh Apps run the same scenario; no edited selection or closed state may leak.
            for _ in 0..2 {
                let calls = Arc::new(Mutex::new(Vec::new()));
                app_input::run_visibility(
                    ChangingMenu {
                        stage: AtomicU8::new(0),
                        dialog,
                        calls: calls.clone(),
                    },
                    size,
                    vec![
                        ("B0", None, super::key(KeyCode::Down)),
                        ("B0", None, super::key(KeyCode::F(2))),
                        ("B2", None, super::key(KeyCode::Enter)),
                        ("ROOT2", Some("B2"), super::key(KeyCode::F(3))),
                        ("ROOT3", Some("B2"), super::key(KeyCode::F(4))),
                        ("A4", None, super::key(KeyCode::Enter)),
                        ("ROOT4", Some("A4"), super::key(KeyCode::F(3))),
                        ("ROOT3", None, super::key(KeyCode::F(5))),
                        ("A5", None, super::key(KeyCode::F(3))),
                        ("ROOT3", Some("A5"), None),
                    ],
                );
                assert_eq!(
                    *calls.lock().unwrap(),
                    [
                        "shown:0",
                        "action:B:2",
                        "selected:B:2",
                        "hidden:2",
                        "shown:4",
                        "action:A:4",
                        "selected:A:4",
                        "hidden:4",
                        "shown:5",
                        "hidden:5"
                    ]
                );
            }
        }
    }
}

#[test]
fn dialog_menu_multi_selection_seed_excludes_unselectable_entries() {
    use reactive_tui::{
        component::Component,
        event::types::KeyCode,
        widgets::menu::{DialogMenuState, DialogMenuType},
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let values = Arc::new(Mutex::new(Vec::new()));
        let output = values.clone();
        let props = DialogMenuProps {
            visible: true,
            dialog_type: DialogMenuType::MultiSelection,
            show_close_button: false,
            items: vec![
                MenuItem::new("valid", "VALID"),
                MenuItem::new("disabled", "DISABLED").enabled(false),
                MenuItem::new("hidden", "HIDDEN").visible(false),
                MenuItem::submenu("group", "GROUP", vec![MenuItem::new("leaf", "LEAF")]),
            ],
            ..Default::default()
        };
        let menu = DialogMenu::new(props.clone())
            .with_on_confirmed(move |ids| output.lock().unwrap().push(ids))
            .render(
                &props,
                &DialogMenuState {
                    selected_items: vec![0, 1, 2, 3, usize::MAX],
                    ..Default::default()
                },
            );
        app_input::run_until_hidden(
            Control(menu),
            size,
            vec![
                ("[x] VALID", super::key(KeyCode::Tab)),
                ("Confirm", super::key(KeyCode::Enter)),
            ],
            "Confirm",
        );
        assert_eq!(*values.lock().unwrap(), [vec!["valid".to_string()]]);
    }
}

#[test]
fn public_menu_props_builders_produce_activatable_controls() {
    use reactive_tui::{
        event::types::KeyCode,
        widgets::menu::{DialogMenuBuilder, MenuBarBuilder, MenuItemBuilder, PopupMenuBuilder},
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        for family in 0..3 {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let output = calls.clone();
            let item = MenuItemBuilder::new("run", "RUN")
                .action(move || output.lock().unwrap().push("run"))
                .build();
            let menu = match family {
                0 => {
                    Element::typed::<MenuBar>(MenuBarBuilder::new().item(item).build()).auto_focus()
                }
                1 => Element::typed::<PopupMenu>(
                    PopupMenuBuilder::new().item(item).visible(true).build(),
                ),
                _ => {
                    let mut props = DialogMenuBuilder::selection().item(item).build();
                    props.visible = true;
                    Element::typed::<DialogMenu>(props)
                }
            };
            if family == 0 {
                app_input::run_when(
                    Control(menu),
                    size,
                    vec![("RUN", super::key(KeyCode::Enter)), ("RUN", None)],
                );
            } else {
                app_input::run_until_hidden(
                    Control(menu),
                    size,
                    vec![("RUN", super::key(KeyCode::Enter))],
                    "RUN",
                );
            }
            assert_eq!(*calls.lock().unwrap(), ["run"]);
        }
    }
}

#[test]
fn context_menu_builder_trigger_areas_and_clicks_use_screen_coordinates() {
    use reactive_tui::{
        component::Component,
        event::types::{Event, MouseButton, MouseEvent, MouseEventKind, Position},
        widgets::menu::{ContextMenuBuilder, ContextMenuState, MenuStyle},
    };
    use std::sync::{Arc, Mutex};
    for size in [(32, 12), (60, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let positions = Arc::new(Mutex::new(Vec::new()));
        let output = calls.clone();
        let shown = positions.clone();
        let props = ContextMenuBuilder::new()
            .trigger_area(8, 5, 4, 2)
            .width(Some(8))
            .show_border(false)
            .show_shadow(false)
            .style(MenuStyle {
                padding: 0,
                min_width: 0,
                ..Default::default()
            })
            .item(MenuItem::action("run", "RUN", move || {
                output.lock().unwrap().push("run")
            }))
            .build();
        let menu = ContextMenu::new(props.clone())
            .with_on_show(move |x, y| shown.lock().unwrap().push((x, y)))
            .render(&props, &ContextMenuState::default())
            .class("absolute left-4 top-2 w-20 h-8");
        let tree = reactive_tui::builder::div()
            .class("relative w-full h-full")
            .child(Element::text("ROOT"))
            .child(menu)
            .build();
        let right = |x, y| {
            Some(Event::Mouse(
                MouseEvent::new(MouseEventKind::Down, Position::cell(x, y))
                    .with_button(MouseButton::Right),
            ))
        };
        app_input::run_visibility(
            Control(tree),
            size,
            vec![
                ("ROOT", Some("RUN"), right(6, 4)),
                ("ROOT", Some("RUN"), right(12, 5)),
                ("ROOT", Some("RUN"), right(9, 5)),
                ("RUN", None, super::click(10, 5)),
                ("ROOT", Some("RUN"), None),
            ],
        );
        assert_eq!(*positions.lock().unwrap(), [(9, 5)]);
        assert_eq!(*calls.lock().unwrap(), ["run"]);
    }
}
