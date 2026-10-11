use reactive_tui::{
    app::RootComponent,
    builder::core::div,
    component::Element,
    error::Result,
    event::types::{Event, KeyCode, KeyEvent, KeyEventKind, ResizeEvent},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
mod common;
use app_input::{click, key, run};
use common::app_input;

struct ClickRoot(Arc<AtomicUsize>);
impl RootComponent for ClickRoot {
    fn render(&self) -> Element {
        let calls = self.0.clone();
        div()
            .class("w-8 h-1")
            .on_click(move || {
                calls.fetch_add(1, Ordering::SeqCst);
            })
            .build()
            .auto_focus()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn builder_callback_receives_keyboard_activation_once() {
    let calls = Arc::new(AtomicUsize::new(0));
    run(
        ClickRoot(calls.clone()),
        (16, 4),
        vec![
            (1, key(KeyCode::Enter)),
            (
                1,
                Some(Event::Key(
                    KeyEvent::new(KeyCode::Enter).with_kind(KeyEventKind::Release),
                )),
            ),
            (
                1,
                Some(Event::Key(
                    KeyEvent::new(KeyCode::Enter).with_kind(KeyEventKind::Repeat),
                )),
            ),
            (1, key(KeyCode::Char('x'))),
        ],
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "the focused builder callback must run once"
    );
}

struct PairRoot {
    calls: Arc<Mutex<Vec<usize>>>,
}
impl RootComponent for PairRoot {
    fn render(&self) -> Element {
        div()
            .class("flex flex-row w-full h-full")
            .children(
                (0..2)
                    .map(|index| {
                        let calls = self.calls.clone();
                        div()
                            .class("flex-1 h-1")
                            .text(if index == 0 { "LEFT" } else { "RIGHT" })
                            .on_click(move || calls.lock().unwrap().push(index))
                            .build()
                    })
                    .collect(),
            )
            .build()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn mouse_bounds_follow_painted_resize_and_misses_do_not_activate_focus() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let frames = run(
        PairRoot {
            calls: calls.clone(),
        },
        (20, 4),
        vec![
            (1, click(11, 0)),
            (1, key(KeyCode::Enter)),
            (1, Some(Event::Resize(ResizeEvent::new(10, 4)))),
            (2, click(6, 0)),
            (2, click(11, 0)),
            (2, click(1, 0)),
            (2, click(1, 3)),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), vec![1, 1, 1, 0]);
    assert!(
        frames[0].text.contains("LEFT      RIGHT"),
        "{}",
        frames[0].text
    );
    assert!(frames[1].text.contains("LEFT RIGHT"), "{}", frames[1].text);
    assert_eq!(frames[0].geometry[2].bounds.x, 10.0);
    assert_eq!(frames[1].geometry[2].bounds.x, 5.0);
}

struct ChangingRoot {
    calls: Arc<Mutex<Vec<usize>>>,
    version: Arc<AtomicUsize>,
}
impl RootComponent for ChangingRoot {
    fn render(&self) -> Element {
        let version = self.version.load(Ordering::SeqCst);
        let state = self.version.clone();
        let calls = self.calls.clone();
        div()
            .class("w-8 h-1")
            .key("same")
            .text(&format!("V{version}"))
            .on_click(move || {
                calls.lock().unwrap().push(version);
                state.fetch_add(1, Ordering::SeqCst);
            })
            .build()
            .auto_focus()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn redraw_replaces_callback_without_duplicate_registration() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let frames = run(
        ChangingRoot {
            calls: calls.clone(),
            version: Arc::new(AtomicUsize::new(0)),
        },
        (12, 3),
        vec![
            (1, key(KeyCode::Enter)),
            (2, key(KeyCode::Space)),
            (3, None),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), vec![0, 1]);
    assert!(frames[2].text.contains("V2"));
}

struct OverlapRoot(Arc<Mutex<Vec<&'static str>>>);
impl RootComponent for OverlapRoot {
    fn render(&self) -> Element {
        let callback = |label| {
            let calls = self.0.clone();
            move || calls.lock().unwrap().push(label)
        };
        div()
            .class("relative w-full h-full")
            .children(vec![
                div()
                    .class("absolute left-0 top-0 w-8 h-1 z-10")
                    .text("TOP")
                    .on_click(callback("top"))
                    .build(),
                div()
                    .class("absolute left-0 top-0 w-8 h-1 z-0")
                    .text("BOTTOM")
                    .on_click(callback("bottom"))
                    .build(),
                div()
                    .class("absolute left-0 top-1 w-3 h-1 overflow-hidden")
                    .child(
                        div()
                            .class("w-8 h-1")
                            .text("CLIPPED")
                            .on_click(callback("clipped"))
                            .build(),
                    )
                    .build(),
            ])
            .build()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn overlap_and_ancestor_clipping_match_visible_cells() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let frames = run(
        OverlapRoot(calls.clone()),
        (12, 4),
        vec![(1, click(1, 0)), (1, click(1, 1)), (1, click(5, 1))],
    );
    assert_eq!(*calls.lock().unwrap(), vec!["top", "clipped"]);
    assert!(frames[0].text.starts_with("TOP"));
    assert!(frames[0].text.contains("CLI"));
    assert!(!frames[0].text.contains("CLIPPED"));
}

struct RemovingRoot {
    removed: Arc<AtomicUsize>,
    marker: Mutex<Option<Arc<()>>>,
    weak_marker: std::sync::Weak<()>,
}
impl RootComponent for RemovingRoot {
    fn render(&self) -> Element {
        let children = if self.removed.load(Ordering::SeqCst) == 0 {
            let marker = self
                .marker
                .lock()
                .unwrap()
                .take()
                .or_else(|| self.weak_marker.upgrade())
                .unwrap();
            let removed = self.removed.clone();
            vec![div()
                .class("w-8 h-1")
                .text("REMOVE")
                .on_click(move || {
                    let _owned = &marker;
                    removed.fetch_add(1, Ordering::SeqCst);
                })
                .build()
                .auto_focus()]
        } else {
            assert!(self.marker.lock().unwrap().is_none());
            return div().class("w-full h-full").text("GONE").build();
        };
        div().class("w-full h-full").children(children).build()
    }
    fn update(&mut self) -> Result<reactive_tui::app::RootUpdate> {
        // Once a replacement frame has rendered, App and backend must release
        // the removed closure before any later input is accepted.
        if self.removed.load(Ordering::SeqCst) > 0 && self.weak_marker.upgrade().is_none() {
            self.removed.store(10, Ordering::SeqCst);
        }
        Ok(reactive_tui::app::RootUpdate::Unchanged)
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn removed_callback_releases_its_capture_and_cannot_run_again() {
    let removed = Arc::new(AtomicUsize::new(0));
    let marker = Arc::new(());
    let weak_marker = Arc::downgrade(&marker);
    let frames = run(
        RemovingRoot {
            removed: removed.clone(),
            marker: Mutex::new(Some(marker)),
            weak_marker: weak_marker.clone(),
        },
        (12, 3),
        vec![(1, click(1, 0)), (2, click(1, 0)), (2, key(KeyCode::Enter))],
    );
    assert_eq!(
        removed.load(Ordering::SeqCst),
        10,
        "closure must be released while App is still running"
    );
    assert!(weak_marker.upgrade().is_none());
    assert!(frames[1].text.contains("GONE"));
}

#[test]
fn capture_target_bubble_order_and_handled_result_are_preserved() {
    use reactive_tui::event::router::{EventPhase, EventResult, EventRouter};
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut router = EventRouter::new();
    let parent = router.create_node(None);
    let child = router.create_node(Some(parent));
    for (node, phase, label) in [
        (parent, EventPhase::Capture, "parent capture"),
        (child, EventPhase::Capture, "target capture"),
        (child, EventPhase::Target, "target"),
        (parent, EventPhase::Bubble, "parent bubble"),
    ] {
        let calls = calls.clone();
        router.add_handler(
            node,
            "key",
            phase,
            Arc::new(move |_| {
                calls.lock().unwrap().push(label);
                EventResult::Handled
            }),
        );
    }
    assert_eq!(
        router.route_event(&Event::Key(KeyEvent::new(KeyCode::Enter)), child),
        EventResult::Handled
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            "parent capture",
            "target capture",
            "target",
            "parent bubble"
        ]
    );
}

struct TreeRoot(Element);
impl RootComponent for TreeRoot {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn child_activation_runs_each_registration_once_without_activating_parent() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let callback = |label| {
        let calls = calls.clone();
        move || calls.lock().unwrap().push(label)
    };
    let tree = div()
        .class("w-full h-full")
        .on_click(callback("parent"))
        .child(
            div()
                .class("w-8 h-1")
                .on_click(callback("first"))
                .on_click(callback("second"))
                .child(Element::text("CHILD").class("w-8 h-1"))
                .build(),
        )
        .build();
    run(TreeRoot(tree), (12, 3), vec![(1, click(1, 0))]);
    assert_eq!(*calls.lock().unwrap(), vec!["first", "second"]);
}

#[reactive_tui::component]
fn RoutedLeaf(hooks: &reactive_tui::reactive::Hooks) -> Element {
    Element::text("COMPONENT")
}

#[test]
fn registered_component_keeps_callers_callback_on_expanded_output() {
    reactive_tui::component::registry::register_component::<RoutedLeaf>("ApiEventRoutedLeaf")
        .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let handler = calls.clone();
    let tree = reactive_tui::builder::core::ElementBuilder::new(
        reactive_tui::component::ElementType::Component("ApiEventRoutedLeaf".into()),
    )
    .class("w-12 h-1")
    .on_click(move || {
        handler.fetch_add(1, Ordering::SeqCst);
    })
    .build();
    let frames = run(TreeRoot(tree), (16, 3), vec![(1, click(1, 0))]);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(frames[0].text.contains("COMPONENT"));
}

#[test]
fn hit_index_uses_actual_initial_viewport_beyond_default_eighty_columns() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let tree = div()
        .class("flex flex-row w-full h-full")
        .children(
            (0..12)
                .map(|index| {
                    let calls = calls.clone();
                    div()
                        .class("w-10 h-1")
                        .text(&index.to_string())
                        .on_click(move || calls.lock().unwrap().push(index))
                        .build()
                })
                .collect(),
        )
        .build();
    let frames = run(TreeRoot(tree), (120, 4), vec![(1, click(115, 0))]);
    assert_eq!(*calls.lock().unwrap(), vec![11]);
    assert_eq!(frames[0].geometry[12].bounds.x, 110.0);
}
