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
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

#[path = "support/app_input.rs"]
mod app_input;
use app_input::{key, run};

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
    assert!(frames[2].text.starts_with("C\nB\nA"), "{}", frames[2].text);
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

struct DialogRoot {
    calls: Log,
    focus: Log,
    stage: AtomicUsize,
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
        let mut children = vec![control("OPENER", true, &self.calls, &self.focus)];
        if stage > 0 {
            let mut outer = vec![
                control("OUTER1", true, &self.calls, &self.focus),
                control("OUTER2", false, &self.calls, &self.focus),
            ];
            if stage > 1 {
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
        div()
            .class("flex flex-col w-full h-full")
            .children(children)
            .build()
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        match key.code {
            KeyCode::Char('o') => self.stage.store(1, Ordering::SeqCst),
            KeyCode::Char('i') => self.stage.store(2, Ordering::SeqCst),
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
            stage: AtomicUsize::new(0),
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
