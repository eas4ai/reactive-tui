use super::{click, key, run, Control};
use reactive_tui::{
    component::Element,
    event::types::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind, Position,
        ResizeEvent, WheelDelta, WheelEvent, WheelPhase,
    },
    widgets::display::tree::{Tree, TreeBuilder, TreeNode, TreeProps},
};
use std::sync::{Arc, Mutex};

fn root() -> TreeNode {
    TreeNode::new("root", "Root").expanded(true).children(vec![
        TreeNode::new("folder", "Folder").children(vec![TreeNode::new("leaf", "Leaf")]),
        TreeNode::new("other", "Other"),
    ])
}
fn props() -> TreeProps {
    let mut props = TreeProps {
        root: Some(root()),
        show_icons: false,
        show_lines: false,
        ..Default::default()
    };
    props.border.enabled = false;
    props
}
fn mouse(kind: MouseEventKind, x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(
        MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left),
    ))
}
fn modified(code: KeyCode, modifiers: KeyModifiers) -> Option<Event> {
    let mut event = KeyEvent::new(code);
    event.modifiers = modifiers;
    Some(Event::Key(event))
}

#[test]
fn tree_named_keyboard_and_mouse_keep_expansion_and_callbacks() {
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.on_select = Some(Arc::new(move |id| {
            sink.lock().unwrap().push(format!("select:{}", id.unwrap()))
        }));
        let sink = calls.clone();
        config.on_expand = Some(Arc::new(move |id, value| {
            sink.lock().unwrap().push(format!("expand:{id}:{value}"))
        }));
        let sink = calls.clone();
        config.on_node_action = Some(Arc::new(move |id, action| {
            sink.lock().unwrap().push(format!("{action}:{id}"))
        }));
        let frames = run(
            Control(Tree::with_props(config).auto_focus()),
            size,
            vec![
                (2, click(5, 1)),
                (3, key(KeyCode::Right)),
                (4, key(KeyCode::Down)),
                (5, key(KeyCode::Enter)),
                (6, click(2, 1)),
                (7, None),
            ],
        );
        assert!(
            frames.iter().any(|frame| frame.text.contains("Leaf")),
            "{:?}",
            frames.iter().map(|frame| &frame.text).collect::<Vec<_>>()
        );
        assert!(!frames.last().unwrap().text.contains("Leaf"));
        assert_eq!(
            *calls.lock().unwrap(),
            vec![
                "select:folder",
                "expand:folder:true",
                "select:leaf",
                "activate:leaf",
                "expand:folder:false"
            ]
        );
    }
}

#[test]
fn tree_both_builders_render_real_hierarchy_and_keep_classes() {
    for size in [(24, 8), (48, 14)] {
        for element in [
            TreeBuilder::new()
                .root(root())
                .show_icons(false)
                .show_lines(false)
                .render(),
            reactive_tui::builder::tree()
                .root(root())
                .show_icons(false)
                .show_lines(false)
                .class("ml-2 mt-1")
                .build(),
        ] {
            let frames = run(
                Control(element.auto_focus()),
                size,
                vec![
                    (2, key(KeyCode::Home)),
                    (3, key(KeyCode::Right)),
                    (4, key(KeyCode::Right)),
                    (5, None),
                ],
            );
            assert!(frames.iter().any(|frame| frame.text.contains("Folder")));
            assert!(
                frames.last().unwrap().text.contains("Leaf"),
                "{}",
                frames.last().unwrap().text
            );
            assert!(!frames.last().unwrap().text.contains("Tree ("));
        }
    }
}

#[test]
fn tree_lazy_load_uses_returned_children_once_and_reports_expansion() {
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut lazy = TreeNode::new("lazy", "Lazy");
        lazy.lazy = true;
        let mut config = props();
        config.root = Some(lazy);
        config.lazy_loading = true;
        config.on_load_children = Some(Arc::new(move |id| {
            sink.lock().unwrap().push(id);
            vec![TreeNode::new("loaded", "Loaded child")]
        }));
        let frames = run(
            Control(Element::typed::<Tree>(config).auto_focus()),
            size,
            vec![
                (2, key(KeyCode::Home)),
                (3, key(KeyCode::Right)),
                (5, key(KeyCode::Left)),
                (7, key(KeyCode::Right)),
                (9, None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), vec!["lazy"]);
        assert!(
            frames.last().unwrap().text.contains("Loaded child"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn tree_checking_and_multiple_selection_have_independent_cursor() {
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.multi_select = true;
        config.checkable = true;
        config.on_multi_select = Some(Arc::new(move |ids| {
            sink.lock().unwrap().push(ids.join(","))
        }));
        let checks = Arc::new(Mutex::new(Vec::new()));
        let sink = checks.clone();
        config.on_check = Some(Arc::new(move |id, value| {
            sink.lock().unwrap().push((id, value))
        }));
        let shift = KeyModifiers {
            shift: true,
            ..KeyModifiers::empty()
        };
        let frames = run(
            Control(Tree::with_props(config).auto_focus()),
            size,
            vec![
                (2, key(KeyCode::Home)),
                (3, modified(KeyCode::Down, shift)),
                (4, modified(KeyCode::Down, shift)),
                (5, key(KeyCode::Char(' '))),
                (6, click(4, 2)),
                (7, None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec!["root", "root,folder", "root,folder,other"]
        );
        assert_eq!(
            *checks.lock().unwrap(),
            vec![("other".to_string(), true), ("other".to_string(), false)]
        );
        assert!(frames.last().unwrap().text.contains("Other"));
    }
}

#[test]
fn tree_search_filters_hidden_nodes_before_mouse_hit_testing() {
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.search_term = Some("leaf".to_string());
        config.filter_visible = true;
        config.on_select = Some(Arc::new(move |id| sink.lock().unwrap().push(id)));
        let frames = run(
            Control(Tree::with_props(config)),
            size,
            vec![(2, click(7, 2)), (3, None)],
        );
        assert_eq!(*calls.lock().unwrap(), vec![Some("leaf".to_string())]);
        assert!(frames.last().unwrap().text.contains("Folder"));
        assert!(!frames.last().unwrap().text.contains("Other"));
    }
}

#[test]
fn tree_virtual_scroll_uses_measured_height_and_signed_wheel() {
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.virtual_scrolling = true;
        config.max_height = Some(4);
        config.root = Some(
            TreeNode::new("root", "Root").expanded(true).children(
                (0..100)
                    .map(|i| TreeNode::new(format!("n{i}"), format!("Item{i:03}")))
                    .collect(),
            ),
        );
        config.on_select = Some(Arc::new(move |id| sink.lock().unwrap().push(id)));
        let wheel = MouseEvent::wheel(
            Position::cell(6, 2),
            WheelEvent {
                delta: WheelDelta::Lines { x: 0.0, y: -2.0 },
                phase: WheelPhase::Changed,
            },
        );
        let frames = run(
            Control(Tree::with_props(config).auto_focus()),
            size,
            vec![
                (2, key(KeyCode::End)),
                (3, Some(Event::Mouse(wheel))),
                (4, click(6, 0)),
                (
                    5,
                    Some(Event::Resize(ResizeEvent::new(size.0 + 4, size.1 + 2))),
                ),
                (7, None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec![Some("n99".to_string()), Some("n94".to_string())]
        );
        assert!(frames.iter().any(|frame| frame.text.contains("Item099")));
        assert!(!frames.last().unwrap().text.contains("Root"));
        assert!(frames.iter().all(|frame| frame.geometry.len() < 40));
    }
}

#[test]
fn tree_drop_moves_owned_nodes_and_rejects_descendant_cycles() {
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.drag_drop = true;
        config.on_node_action = Some(Arc::new(move |id, action| {
            sink.lock().unwrap().push(format!("{id}:{action}"))
        }));
        let frames = run(
            Control(Tree::with_props(config)),
            size,
            vec![
                (2, mouse(MouseEventKind::Down, 5, 1)),
                (2, mouse(MouseEventKind::Drag, 5, 2)),
                (3, mouse(MouseEventKind::Up, 5, 2)),
                (4, mouse(MouseEventKind::Down, 5, 1)),
                (4, mouse(MouseEventKind::Drag, 7, 2)),
                (5, mouse(MouseEventKind::Up, 7, 2)),
                (6, None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), vec!["folder:drop:other"]);
        let last = &frames.last().unwrap().text;
        assert!(
            last.find("Other").unwrap() < last.find("Folder").unwrap(),
            "{last}"
        );
    }
}

#[test]
fn tree_empty_disabled_and_invalid_roots_are_observable_and_inert() {
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.on_select = Some(Arc::new(move |id| sink.lock().unwrap().push(id)));
        let frames = run(
            Control(Tree::with_props(config).disabled(true).auto_focus()),
            size,
            vec![(2, click(5, 1)), (2, key(KeyCode::Down)), (2, None)],
        );
        assert!(calls.lock().unwrap().is_empty());
        assert!(frames.last().unwrap().text.contains("Folder"));
        let mut config = props();
        config.root = None;
        let frames = run(
            Control(Tree::with_props(config)),
            size,
            vec![(2, click(5, 0)), (2, None)],
        );
        assert!(frames.last().unwrap().text.contains("No data"));
        let mut config = props();
        config.root =
            Some(TreeNode::new("same", "Root").children(vec![TreeNode::new("same", "Child")]));
        let frames = run(Control(Tree::with_props(config)), size, vec![(1, None)]);
        assert!(frames.last().unwrap().text.contains("Duplicate tree node"));
    }
}

#[test]
fn tree_initial_lazy_expansion_loads_and_duplicate_children_fail_visibly() {
    for size in [(24, 8), (48, 14)] {
        for invalid in [false, true] {
            let calls = Arc::new(Mutex::new(0));
            let sink = calls.clone();
            let mut lazy = TreeNode::new("lazy", "Lazy").expanded(true);
            lazy.lazy = true;
            let mut config = props();
            config.root = Some(lazy);
            config.lazy_loading = true;
            config.on_load_children = Some(Arc::new(move |_| {
                *sink.lock().unwrap() += 1;
                vec![TreeNode::new(
                    if invalid { "lazy" } else { "child" },
                    "Initial child",
                )]
            }));
            let frames = run(
                Control(Tree::with_props(config).auto_focus()),
                size,
                vec![(2, None)],
            );
            assert_eq!(*calls.lock().unwrap(), 1);
            assert!(
                frames.last().unwrap().text.contains(if invalid {
                    "Duplicate tree node"
                } else {
                    "Initial child"
                }),
                "{}",
                frames.last().unwrap().text
            );
        }
        let mut lazy = TreeNode::new("lazy", "Lazy");
        lazy.lazy = true;
        let mut config = props();
        config.root = Some(lazy);
        config.lazy_loading = true;
        let frames = run(
            Control(Tree::with_props(config).auto_focus()),
            size,
            vec![(2, key(KeyCode::Right)), (3, None)],
        );
        let compact: String = frames.last().unwrap().text.split_whitespace().collect();
        assert!(compact.contains("needsalazy-loadcallback"), "{compact}");
    }
}

#[test]
fn tree_down_up_click_does_not_toggle_twice_and_release_keys_are_inert() {
    use reactive_tui::event::types::KeyEventKind;
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.on_expand = Some(Arc::new(move |id, value| {
            sink.lock().unwrap().push((id, value))
        }));
        let mut release = KeyEvent::new(KeyCode::Left);
        release.kind = KeyEventKind::Release;
        let frames = run(
            Control(Tree::with_props(config).auto_focus()),
            size,
            vec![
                (2, click(2, 1)),
                (4, mouse(MouseEventKind::Up, 2, 1)),
                (4, mouse(MouseEventKind::Click, 2, 1)),
                (4, Some(Event::Key(release))),
                (5, None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), vec![("folder".to_string(), true)]);
        assert!(frames.last().unwrap().text.contains("Leaf"));
    }
}

#[test]
fn tree_changed_props_preserve_ids_and_replace_authored_flags() {
    use reactive_tui::{app::RootComponent, event::router::EventResult};
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Changing {
        phase: AtomicUsize,
        calls: Arc<Mutex<Vec<String>>>,
    }
    impl RootComponent for Changing {
        fn render(&self) -> Element {
            let mut config = props();
            let root = config.root.as_mut().unwrap();
            if self.phase.load(Ordering::SeqCst) >= 1 {
                root.children.reverse();
                root.label = "Reordered".to_string();
            }
            if self.phase.load(Ordering::SeqCst) >= 2 {
                root.expanded = false;
            }
            let sink = self.calls.clone();
            config.on_node_action = Some(Arc::new(move |id, _| sink.lock().unwrap().push(id)));
            Tree::with_props(config).with_key("tree").auto_focus()
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if matches!(event,Event::Key(key) if key.code==KeyCode::F(2)) {
                self.phase.fetch_add(1, Ordering::SeqCst);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
        fn wake_driven(&self) -> bool {
            true
        }
    }
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let frames = run(
            Changing {
                phase: AtomicUsize::new(0),
                calls: calls.clone(),
            },
            size,
            vec![
                (2, click(5, 1)),
                (3, key(KeyCode::Right)),
                (5, key(KeyCode::Down)),
                (6, key(KeyCode::F(2))),
                (7, key(KeyCode::Enter)),
                (8, key(KeyCode::F(2))),
                (10, None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), vec!["leaf"]);
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("Reordered") && frame.text.contains("Leaf")));
        assert!(
            !frames.last().unwrap().text.contains("Folder"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn tree_per_node_flags_skip_selection_and_allow_independent_checks() {
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        let root = config.root.as_mut().unwrap();
        root.selectable = false;
        root.children[0].selectable = false;
        root.children[0].expandable = false;
        root.children[1].checkable = true;
        config.on_select = Some(Arc::new(move |id| sink.lock().unwrap().push(id.unwrap())));
        let checks = Arc::new(Mutex::new(Vec::new()));
        let sink = checks.clone();
        config.on_check = Some(Arc::new(move |id, value| {
            sink.lock().unwrap().push((id, value))
        }));
        let frames = run(
            Control(Tree::with_props(config).auto_focus()),
            size,
            vec![
                (2, key(KeyCode::Home)),
                (3, key(KeyCode::Char(' '))),
                (4, click(5, 1)),
                (5, key(KeyCode::Right)),
                (6, None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), vec!["other"]);
        assert_eq!(*checks.lock().unwrap(), vec![("other".to_string(), true)]);
        assert!(!frames.last().unwrap().text.contains("Leaf"));
    }
}

#[test]
fn tree_lines_icons_styles_and_padded_targets_follow_resize() {
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = calls.clone();
        let mut config = props();
        config.show_lines = true;
        config.show_icons = true;
        config.expanded_style = Some("bg-red-500".to_string());
        config.leaf_style = Some("text-green-500".to_string());
        config.line_style = Some("text-blue-500".to_string());
        config.root.as_mut().unwrap().children[0].icon = Some("界".to_string());
        config.on_expand = Some(Arc::new(move |id, value| {
            sink.lock().unwrap().push((id, value))
        }));
        let frames = run(
            Control(
                Element::layout(reactive_tui::component::LayoutType::Flex)
                    .with_class("flex flex-col pl-2 pt-1")
                    .with_child(Tree::with_props(config).auto_focus()),
            ),
            size,
            vec![
                (2, click(13, 5)),
                (
                    4,
                    Some(Event::Resize(ResizeEvent::new(size.0 + 6, size.1 + 2))),
                ),
                (6, click(13, 5)),
                (7, None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec![("folder".to_string(), true), ("folder".to_string(), false)]
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("界 Folder") && frame.text.contains("Leaf")));
        let screen = &frames.last().unwrap().screen;
        assert_ne!(screen.cell(4, 8).unwrap().bgcolor(), vt100::Color::Default);
        assert_ne!(screen.cell(5, 10).unwrap().fgcolor(), vt100::Color::Default);
        assert_ne!(screen.cell(6, 18).unwrap().fgcolor(), vt100::Color::Default);
    }
}

#[test]
fn tree_border_and_builder_classes_reach_painted_cells() {
    use reactive_tui::widgets::display::{Border, BorderStyle};
    for size in [(24, 8), (48, 14)] {
        let config = TreeBuilder::new()
            .root(root())
            .show_icons(false)
            .show_lines(false)
            .border(Border {
                enabled: true,
                style: BorderStyle::Double,
                color: Some("red-500".to_string()),
            })
            .build();
        let frames = run(Control(Tree::with_props(config)), size, vec![(2, None)]);
        let screen = &frames.last().unwrap().screen;
        assert_eq!(screen.cell(0, 0).unwrap().contents(), "╔");
        assert_eq!(screen.cell(4, size.0 - 1).unwrap().contents(), "╝");
        assert_ne!(screen.cell(0, 0).unwrap().fgcolor(), vt100::Color::Default);
        let frames = run(
            Control(
                reactive_tui::builder::tree()
                    .root(root())
                    .show_icons(false)
                    .show_lines(false)
                    .class("text-red-500")
                    .build(),
            ),
            size,
            vec![(2, None)],
        );
        assert_ne!(
            frames.last().unwrap().screen.cell(1, 3).unwrap().fgcolor(),
            vt100::Color::Default
        );
    }
}
