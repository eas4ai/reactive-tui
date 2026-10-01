use reactive_tui::{
    app::RootComponent,
    builder::core::div,
    component::{Element, FocusProps},
    event::{
        router::EventResult,
        types::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    },
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

mod common;
use app_input::{click, key, run};
use common::app_input;

type Log = Arc<Mutex<Vec<String>>>;
fn control(label: &'static str, auto_focus: bool, calls: &Log, focus: &Log) -> Element {
    let calls = calls.clone();
    let gained = focus.clone();
    let lost = focus.clone();
    div()
        .key(label)
        .class("w-12 h-1")
        .text(label)
        .on_click(move || calls.lock().unwrap().push(label.into()))
        .build()
        .with_focus(FocusProps {
            auto_focus,
            on_focus: Some(Arc::new(move || {
                gained.lock().unwrap().push(format!("{label}+"))
            })),
            on_blur: Some(Arc::new(move || {
                lost.lock().unwrap().push(format!("{label}-"))
            })),
            ..Default::default()
        })
}

struct ListRoot {
    calls: Log,
    focus: Log,
    reverse: AtomicUsize,
}
impl RootComponent for ListRoot {
    fn render(&self) -> Element {
        let mut controls: Vec<_> = ["A", "B", "C"]
            .into_iter()
            .map(|name| control(name, name == "A", &self.calls, &self.focus))
            .collect();
        if self.reverse.load(Ordering::SeqCst) > 0 {
            controls.reverse();
        }
        div()
            .class("flex flex-col w-full h-full")
            .children(controls)
            .build()
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        if matches!(event, Event::Key(key) if key.code == KeyCode::Char('r')) {
            self.reverse.store(1, Ordering::SeqCst);
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn keyed_redraw_and_reorder_preserve_focus_without_repeat_callbacks() {
    let calls = Log::default();
    let focus = Log::default();
    let frames = run(
        ListRoot {
            calls: calls.clone(),
            focus: focus.clone(),
            reverse: AtomicUsize::new(0),
        },
        (16, 6),
        vec![
            (1, key(KeyCode::Tab)),
            (1, key(KeyCode::Enter)),
            (2, key(KeyCode::Char('r'))),
            (3, key(KeyCode::Enter)),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), vec!["B", "B"]);
    assert_eq!(*focus.lock().unwrap(), vec!["A+", "A-", "B+"]);
    assert_eq!(
        frames[2]
            .text
            .lines()
            .take(3)
            .map(str::trim_end)
            .collect::<Vec<_>>(),
        vec!["C", "B", "A"]
    );
}

#[test]
fn tab_reverse_tab_and_release_follow_one_focus_order_without_a_trap() {
    let calls = Log::default();
    run(
        ListRoot {
            calls: calls.clone(),
            focus: Log::default(),
            reverse: AtomicUsize::new(0),
        },
        (16, 6),
        vec![
            (1, key(KeyCode::Tab)),
            (1, key(KeyCode::Enter)),
            (1, key(KeyCode::BackTab)),
            (1, key(KeyCode::Enter)),
            (
                1,
                Some(Event::Key(
                    KeyEvent::new(KeyCode::Tab).with_modifiers(KeyModifiers::shift()),
                )),
            ),
            (1, key(KeyCode::Enter)),
            (
                1,
                Some(Event::Key(
                    KeyEvent::new(KeyCode::Tab).with_kind(KeyEventKind::Release),
                )),
            ),
            (1, key(KeyCode::Enter)),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), vec!["B", "A", "C", "C"]);
}

#[derive(Default)]
struct DialogRoot {
    calls: Log,
    focus: Log,
    stage: AtomicUsize,
    hide_opener: AtomicBool,
    empty: AtomicBool,
}
fn dialog(key: &str, children: Vec<Element>) -> Element {
    div()
        .key(key)
        .class("flex flex-col w-12 h-auto")
        .children(children)
        .build()
        .with_focus(FocusProps {
            trap_focus: true,
            restore_focus: true,
            auto_focus: true,
            focusable: false,
            ..Default::default()
        })
}
impl RootComponent for DialogRoot {
    fn render(&self) -> Element {
        let stage = self.stage.load(Ordering::SeqCst);
        let empty = self.empty.load(Ordering::SeqCst);
        let mut children = if self.hide_opener.load(Ordering::SeqCst) {
            vec![control("FALLBACK", false, &self.calls, &self.focus)]
        } else {
            vec![control("OPENER", true, &self.calls, &self.focus)]
        };
        if stage > 0 {
            let mut outer = if empty {
                vec![Element::text("EMPTY").class("h-1")]
            } else {
                vec![
                    control("OUTER1", true, &self.calls, &self.focus),
                    control("OUTER2", false, &self.calls, &self.focus),
                ]
            };
            if stage > 1 && !empty {
                outer.push(dialog(
                    "inner",
                    vec![
                        control("INNER1", true, &self.calls, &self.focus),
                        control("INNER2", false, &self.calls, &self.focus),
                    ],
                ));
            }
            children.push(dialog("outer", outer));
        }
        let root = div()
            .class("flex flex-col w-full h-full")
            .children(children);
        if empty && stage > 0 {
            let calls = self.calls.clone();
            root.on_click(move || calls.lock().unwrap().push("ROOT".into()))
                .build()
        } else {
            root.build()
        }
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        match key.code {
            KeyCode::Char('o') => self.stage.store(1, Ordering::SeqCst),
            KeyCode::Char('i') => self.stage.store(2, Ordering::SeqCst),
            KeyCode::Char('a') => self.stage.store(0, Ordering::SeqCst),
            KeyCode::Char('h') => self.hide_opener.store(true, Ordering::SeqCst),
            KeyCode::Char('e') => {
                self.empty.store(true, Ordering::SeqCst);
                self.stage.store(1, Ordering::SeqCst);
            }
            KeyCode::Char('x') => {
                self.stage
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                        Some(n.saturating_sub(1))
                    })
                    .unwrap();
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn nested_dialogs_autofocus_wrap_and_restore_the_last_outer_target() {
    let calls = Log::default();
    let focus = Log::default();
    let frames = run(
        DialogRoot {
            calls: calls.clone(),
            focus: focus.clone(),
            ..Default::default()
        },
        (20, 8),
        vec![
            (1, key(KeyCode::Char('o'))),
            (2, key(KeyCode::Enter)),
            (2, key(KeyCode::Tab)),
            (2, key(KeyCode::Enter)),
            (2, key(KeyCode::Char('i'))),
            (3, key(KeyCode::Enter)),
            (3, key(KeyCode::BackTab)),
            (3, key(KeyCode::Enter)),
            (3, key(KeyCode::Tab)),
            (3, key(KeyCode::Enter)),
            (3, key(KeyCode::Char('x'))),
            (4, key(KeyCode::Enter)),
            (4, key(KeyCode::Char('x'))),
            (5, key(KeyCode::Enter)),
        ],
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["OUTER1", "OUTER2", "INNER1", "INNER2", "INNER1", "OUTER2", "OPENER"]
    );
    assert!(frames[2].text.contains("INNER1"));
    assert!(!frames[4].text.contains("OUTER1"));
    assert_eq!(
        *focus.lock().unwrap(),
        vec![
            "OPENER+", "OPENER-", "OUTER1+", "OUTER1-", "OUTER2+", "OUTER2-", "INNER1+", "INNER1-",
            "INNER2+", "INNER2-", "INNER1+", "INNER1-", "OUTER2+", "OUTER2-", "OPENER+"
        ]
    );
}

#[test]
fn removing_the_outer_dialog_with_inner_open_restores_the_original_opener() {
    let calls = Log::default();
    let focus = Log::default();
    run(
        DialogRoot {
            calls: calls.clone(),
            focus: focus.clone(),
            ..Default::default()
        },
        (20, 8),
        vec![
            (1, key(KeyCode::Char('o'))),
            (2, key(KeyCode::Char('i'))),
            (3, key(KeyCode::Enter)),
            (3, key(KeyCode::Char('a'))),
            (4, key(KeyCode::Enter)),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), vec!["INNER1", "OPENER"]);
    assert!(focus
        .lock()
        .unwrap()
        .ends_with(&["INNER1-".into(), "OPENER+".into()]));
}

#[test]
fn mouse_focus_stays_in_the_active_dialog() {
    let calls = Log::default();
    let focus = Log::default();
    run(
        DialogRoot {
            calls: calls.clone(),
            focus: focus.clone(),
            ..Default::default()
        },
        (20, 8),
        vec![
            (1, key(KeyCode::Char('o'))),
            (2, click(1, 0)),
            (2, key(KeyCode::Enter)),
            (2, click(1, 2)),
            (2, key(KeyCode::Enter)),
        ],
    );
    // A focus trap alone does not provide a modal pointer-event backdrop.
    // The outside callback can run, but it cannot move keyboard focus outside.
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["OPENER", "OUTER1", "OUTER2", "OUTER2"]
    );
    assert_eq!(
        *focus.lock().unwrap(),
        vec!["OPENER+", "OPENER-", "OUTER1+", "OUTER1-", "OUTER2+"]
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
fn positive_tab_indices_precede_default_order_and_negative_indices_are_skipped() {
    let calls = Log::default();
    let focus = Log::default();
    let children = [
        ("ZERO1", 0),
        ("TWO", 2),
        ("NEGATIVE", -1),
        ("ONE", 1),
        ("ZERO2", 0),
    ]
    .into_iter()
    .map(|(label, index)| control(label, false, &calls, &focus).tab_index(index))
    .collect();
    let tree = div()
        .class("flex flex-col w-full h-full")
        .children(children)
        .build();
    let mut steps = Vec::new();
    for _ in 0..5 {
        steps.extend([(1, key(KeyCode::Tab)), (1, key(KeyCode::Enter))]);
    }
    run(TreeRoot(tree), (16, 8), steps);
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["ONE", "TWO", "ZERO1", "ZERO2", "ONE"]
    );
}

#[test]
fn public_app_focus_methods_share_the_router_focus_owner() {
    use reactive_tui::{app::App, backend::SuprTuiBackend, event::router::NodeId};
    let mut app = App::builder()
        .backend(SuprTuiBackend::with_writer(8, 2, std::io::sink()).unwrap())
        .root(TreeRoot(Element::empty()))
        .build()
        .unwrap();
    let first = NodeId::new();
    let second = NodeId::new();
    assert!(app.register_focusable(first, None));
    assert!(app.register_focusable(second, None));
    assert!(app.set_initial_focus(first));
    assert_eq!(app.current_focus(), Some(&first));
    app.focus_next();
    assert_eq!(app.current_focus(), Some(&second));
    app.focus_previous();
    assert_eq!(app.current_focus(), Some(&first));
}

#[test]
fn a_removed_opener_is_not_resurrected_when_nested_dialogs_close() {
    let calls = Log::default();
    let focus = Log::default();
    run(
        DialogRoot {
            calls: calls.clone(),
            focus: focus.clone(),
            ..Default::default()
        },
        (20, 8),
        vec![
            (1, key(KeyCode::Char('o'))),
            (2, key(KeyCode::Char('i'))),
            (3, key(KeyCode::Char('h'))),
            (4, key(KeyCode::Enter)),
            (4, key(KeyCode::Char('a'))),
            (5, key(KeyCode::Enter)),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), vec!["INNER1", "FALLBACK"]);
    assert!(focus
        .lock()
        .unwrap()
        .ends_with(&["INNER1-".into(), "FALLBACK+".into()]));
}

#[test]
fn an_empty_trap_blocks_outside_keyboard_activation_and_restores_on_close() {
    let calls = Log::default();
    let focus = Log::default();
    let frames = run(
        DialogRoot {
            calls: calls.clone(),
            focus: focus.clone(),
            ..Default::default()
        },
        (20, 8),
        vec![
            (1, key(KeyCode::Char('e'))),
            (2, key(KeyCode::Tab)),
            (2, key(KeyCode::BackTab)),
            (2, key(KeyCode::Enter)),
            (2, key(KeyCode::Char('x'))),
            (3, key(KeyCode::Enter)),
        ],
    );
    assert!(frames[1].text.contains("EMPTY"));
    assert_eq!(*calls.lock().unwrap(), vec!["OPENER"]);
    assert_eq!(
        *focus.lock().unwrap(),
        vec!["OPENER+", "OPENER-", "OPENER+"]
    );
}

#[test]
fn removing_an_imperative_router_subtree_also_removes_its_trap() {
    use reactive_tui::event::router::EventRouter;
    let mut router = EventRouter::new();
    let root = router.create_node(None);
    let opener = router.create_node(Some(root));
    let outer = router.create_node(Some(root));
    let first = router.create_node(Some(outer));
    let inner = router.create_node(Some(outer));
    let second = router.create_node(Some(inner));
    for node in [opener, first, second] {
        router.add_focusable(node, None);
    }
    router.set_focus(Some(opener));
    assert!(router.create_focus_trap(outer, vec![first, second]));
    assert!(router.create_focus_trap(inner, vec![second]));
    router.remove_node(outer);
    assert!(!router.is_focus_trapped());
    assert_eq!(router.get_focus(), Some(opener));
    assert_eq!(router.focus_next(), Some(opener));
}

struct AutofocusRoot {
    enabled: AtomicBool,
    calls: Log,
    focus: Log,
}
impl RootComponent for AutofocusRoot {
    fn render(&self) -> Element {
        div()
            .class("flex flex-col w-full h-full")
            .children(vec![
                control(
                    "A",
                    self.enabled.load(Ordering::SeqCst),
                    &self.calls,
                    &self.focus,
                ),
                control("B", true, &self.calls, &self.focus),
            ])
            .build()
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        if matches!(event, Event::Key(key) if key.code == KeyCode::Char('a')) {
            self.enabled.store(true, Ordering::SeqCst);
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn enabling_autofocus_requests_focus_once_on_an_existing_key() {
    let calls = Log::default();
    let focus = Log::default();
    run(
        AutofocusRoot {
            calls: calls.clone(),
            focus: focus.clone(),
            enabled: AtomicBool::new(false),
        },
        (16, 4),
        vec![
            (1, key(KeyCode::Enter)),
            (1, key(KeyCode::Char('a'))),
            (2, key(KeyCode::Enter)),
            (3, None),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), vec!["B", "A"]);
    assert_eq!(*focus.lock().unwrap(), vec!["B+", "B-", "A+"]);
}

#[test]
fn an_explicit_imperative_trap_call_can_reactivate_an_existing_container() {
    use reactive_tui::event::router::EventRouter;
    let mut router = EventRouter::new();
    let first = router.create_node(None);
    let second = router.create_node(Some(first));
    router.add_focusable(first, None);
    router.add_focusable(second, None);
    assert!(router.create_focus_trap(first, vec![first]));
    assert!(router.create_focus_trap(second, vec![second]));
    assert!(router.create_focus_trap(first, vec![first]));
    assert_eq!(router.get_active_focus_trap(), Some(first));
    assert_eq!(router.get_focus(), Some(first));
    router.remove_focus_trap(first);
    assert_eq!(router.get_active_focus_trap(), Some(second));
    assert_eq!(router.get_focus(), Some(second));
}
