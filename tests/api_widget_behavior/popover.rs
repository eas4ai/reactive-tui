use super::{click, key, run, Control};
use reactive_tui::{
    builder, component::Element, event::types::KeyCode, widgets::display::popover::*,
};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct StatusRoot {
    config: PopoverProps,
    status: Arc<AtomicUsize>,
}
impl reactive_tui::app::RootComponent for StatusRoot {
    fn render(&self) -> Element {
        builder::div()
            .class("relative w-full h-full")
            .children(vec![
                Element::typed::<Popover>(self.config.clone()),
                Element::text(format!("STATE{}", self.status.load(Ordering::SeqCst)))
                    .class("absolute left-0 bottom-0 h-1"),
            ])
            .build()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
fn status_config(config: PopoverProps) -> StatusRoot {
    let status = Arc::new(AtomicUsize::new(0));
    let opened = status.clone();
    let closed = status.clone();
    StatusRoot {
        config: PopoverProps {
            on_open: Some(Arc::new(move || opened.store(1, Ordering::SeqCst))),
            on_close: Some(Arc::new(move || closed.store(2, Ordering::SeqCst))),
            ..config
        },
        status,
    }
}

#[test]
fn popover_hover_deadlines_open_and_close_an_idle_app() {
    use reactive_tui::event::types::{Event, MouseEvent, MouseEventKind, Position};
    let root = status_config(PopoverProps {
        trigger: PopoverTrigger::Hover,
        hover_delay: Duration::from_millis(60),
        hover_leave_delay: Duration::from_millis(60),
        ..props()
    });
    let frames = super::app_input::run_when(
        root,
        (30, 10),
        vec![
            (
                "OPEN",
                Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Move,
                    Position::cell(1, 0),
                ))),
            ),
            (
                "DETAILS",
                Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Move,
                    Position::cell(20, 5),
                ))),
            ),
            ("STATE2", None),
        ],
    );
    assert!(frames.iter().any(|frame| frame.text.contains("DETAILS")));
    assert!(!frames.last().unwrap().text.contains("DETAILS"));
}

#[test]
fn leaving_the_terminal_closes_hover_popover_at_the_origin() {
    use reactive_tui::event::types::{Event, MouseEvent, MouseEventKind, Position};
    let root = status_config(PopoverProps {
        trigger: PopoverTrigger::Hover,
        hover_delay: Duration::ZERO,
        hover_leave_delay: Duration::from_millis(10),
        ..props()
    });
    let frames = super::app_input::run_when(
        root,
        (24, 10),
        vec![
            (
                "OPEN",
                Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Move,
                    Position::cell(0, 0),
                ))),
            ),
            (
                "DETAILS",
                Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Leave,
                    Position::cell(0, 0),
                ))),
            ),
            ("STATE2", None),
        ],
    );
    assert!(!frames.last().unwrap().text.contains("DETAILS"));
}

#[test]
fn focus_trigger_opens_on_focus_and_escape_can_dismiss_without_reopening() {
    let root = status_config(PopoverProps {
        trigger: PopoverTrigger::Focus,
        auto_focus: true,
        ..props()
    });
    let frames = super::app_input::run_when(
        root,
        (30, 10),
        vec![("DETAILS", key(KeyCode::Escape)), ("STATE2", None)],
    );
    assert!(!frames.last().unwrap().text.contains("DETAILS"));
}

struct ManualRoot {
    popover: Arc<Popover>,
    config: PopoverProps,
    /// Set once the App has rendered its first frame.
    rendered: Arc<AtomicBool>,
}
impl reactive_tui::app::RootComponent for ManualRoot {
    fn render(&self) -> Element {
        self.rendered.store(true, Ordering::SeqCst);
        reactive_tui::component::Component::render(
            self.popover.as_ref(),
            &self.config,
            &PopoverState::default(),
        )
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn imperative_popover_handle_wakes_idle_app_and_callbacks_can_reenter_it() {
    let popover = Arc::new(Popover::new());
    let weak = Arc::downgrade(&popover);
    let called = Arc::new(AtomicBool::new(false));
    let observed = called.clone();
    let config = PopoverProps {
        trigger: PopoverTrigger::Manual,
        on_open: Some(Arc::new(move || {
            assert!(weak.upgrade().unwrap().is_visible());
            observed.store(true, Ordering::SeqCst);
        })),
        ..props()
    };
    let handle = popover.clone();
    let rendered = Arc::new(AtomicBool::new(false));
    let first_frame = rendered.clone();
    let worker = std::thread::spawn(move || {
        // Wait for the App's first frame; the deadline is a hang guard.
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        while !first_frame.load(Ordering::SeqCst) && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        // The behavior under test: a show from another thread once the App
        // has settled into its idle wait after that frame.
        std::thread::sleep(Duration::from_millis(60));
        handle.show();
    });
    let frames = super::app_input::run_when(
        ManualRoot {
            popover,
            config,
            rendered,
        },
        (30, 10),
        vec![("DETAILS", None)],
    );
    worker.join().unwrap();
    assert!(called.load(Ordering::SeqCst));
    assert!(frames.last().unwrap().text.contains("DETAILS"));
}

#[test]
fn explicit_popover_anchors_use_cells_and_invalid_rectangles_restore_measurement() {
    use taffy::geometry::Rect;
    for size in [(24, 10), (48, 16)] {
        for (rect, x, y) in [
            (
                Rect {
                    left: 5.0,
                    top: 3.0,
                    right: 9.0,
                    bottom: 4.0,
                },
                5,
                4,
            ),
            (
                Rect {
                    left: f32::NAN,
                    top: 3.0,
                    right: 9.0,
                    bottom: 4.0,
                },
                0,
                1,
            ),
            (
                Rect {
                    left: 9.0,
                    top: 3.0,
                    right: 5.0,
                    bottom: 4.0,
                },
                0,
                1,
            ),
        ] {
            let popover = Arc::new(Popover::new());
            popover.set_trigger_rect(rect);
            let config = PopoverProps {
                visible: true,
                ..props()
            };
            let frames = run(
                ManualRoot {
                    popover,
                    config,
                    rendered: Default::default(),
                },
                size,
                vec![(3, None)],
            );
            assert_eq!(
                frames.last().unwrap().screen.cell(y, x).unwrap().contents(),
                "D"
            );
        }
    }
}

#[test]
fn popover_size_constraints_clip_content_and_empty_bodies_keep_the_trigger() {
    for size in [(24, 10), (48, 16)] {
        for (content, max_width, expected) in [
            (
                Element::text("ABCDEFGHIJKLMNO").class("w-15 h-2"),
                Some(6),
                "ABCDEF",
            ),
            (
                Element::text("ABCDEFGHIJKLMNO").class("w-15 h-2"),
                Some(0),
                "",
            ),
            (Element::empty(), None, ""),
        ] {
            let config = PopoverProps {
                visible: true,
                content,
                max_width,
                ..props()
            };
            let frames = run(
                Control(Element::typed::<Popover>(config)),
                size,
                vec![(3, None)],
            );
            let frame = frames.last().unwrap();
            assert!(frame.text.contains("OPEN"));
            assert_eq!(
                frame.text.lines().nth(1).unwrap().trim(),
                expected,
                "{}",
                frame.text
            );
        }
    }
}

#[test]
fn arrow_does_not_cover_the_body_mouse_target() {
    let calls = Arc::new(AtomicUsize::new(0));
    let clicked = calls.clone();
    let config = PopoverProps {
        visible: true,
        offset: (0, 2),
        content: builder::button()
            .text("BODY")
            .class("w-8 h-1 p-0")
            .on_click(move || {
                clicked.fetch_add(1, Ordering::SeqCst);
            })
            .build(),
        arrow: PopoverArrow {
            enabled: true,
            size: 2,
            offset: 0,
            style: ArrowStyle::Solid,
        },
        ..props()
    };
    run(
        Control(Element::typed::<Popover>(config)),
        (30, 10),
        vec![(3, click(1, 3)), (4, None)],
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

struct FocusRoot {
    config: PopoverProps,
    removed: Arc<AtomicBool>,
    calls: Arc<Mutex<Vec<&'static str>>>,
}

struct UpdatedPopover {
    config: Mutex<PopoverProps>,
    calls: Arc<Mutex<Vec<&'static str>>>,
}
impl reactive_tui::app::RootComponent for UpdatedPopover {
    fn render(&self) -> Element {
        Element::typed::<Popover>(self.config.lock().unwrap().clone())
    }
    fn handle_event(
        &self,
        event: &reactive_tui::event::types::Event,
    ) -> reactive_tui::event::router::EventResult {
        use reactive_tui::event::{router::EventResult, types::Event};
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        let mut props = self.config.lock().unwrap();
        match key.code {
            KeyCode::Char('r') => {
                props.content = Element::text("REPLACED").class("w-8 h-2");
                let calls = self.calls.clone();
                props.on_open = Some(Arc::new(move || calls.lock().unwrap().push("new open")));
                let calls = self.calls.clone();
                props.on_close = Some(Arc::new(move || calls.lock().unwrap().push("new close")));
            }
            KeyCode::Char('o') => props.visible = true,
            KeyCode::Char('c') => props.visible = false,
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn popover_retains_open_state_on_content_replacement_and_uses_new_callbacks() {
    for size in [(24, 10), (48, 16)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let old = calls.clone();
        let config = PopoverProps {
            on_open: Some(Arc::new(move || old.lock().unwrap().push("old open"))),
            ..props()
        };
        let frames = run(
            UpdatedPopover {
                config: Mutex::new(config),
                calls: calls.clone(),
            },
            size,
            vec![
                (2, key(KeyCode::Enter)),
                (4, key(KeyCode::Char('r'))),
                (6, key(KeyCode::Escape)),
                (7, key(KeyCode::Char('o'))),
                (9, key(KeyCode::Char('c'))),
                (10, None),
            ],
        );
        assert!(frames.iter().any(|frame| frame.text.contains("REPLACED")));
        assert_eq!(
            *calls.lock().unwrap(),
            ["old open", "new close", "new open", "new close"]
        );
        assert!(!frames.last().unwrap().text.contains("REPLACED"));
    }
}

#[test]
fn disabled_popover_trigger_does_not_open_from_mouse_input() {
    let mut trigger = builder::button().text("OPEN").class("w-4 h-1 p-0").build();
    trigger.metadata.disabled = true;
    let calls = Arc::new(Mutex::new(Vec::new()));
    let config = PopoverProps {
        trigger_element: trigger,
        ..props()
    };
    let frames = run(
        UpdatedPopover {
            config: Mutex::new(config),
            calls: calls.clone(),
        },
        (24, 10),
        vec![(2, click(1, 0)), (2, key(KeyCode::Char('r'))), (3, None)],
    );
    assert!(calls.lock().unwrap().is_empty());
    assert!(!frames.last().unwrap().text.contains("REPLACED"));
}

#[test]
fn popover_dismissal_flags_are_independent() {
    for size in [(24, 10), (48, 16)] {
        let config = PopoverProps {
            visible: true,
            close_on_escape: false,
            close_on_outside_click: false,
            close_on_trigger_click: false,
            ..props()
        };
        let frames = run(
            UpdatedPopover {
                config: Mutex::new(config),
                calls: Arc::default(),
            },
            size,
            vec![
                (3, key(KeyCode::Escape)),
                (3, click(20, 8)),
                (3, click(1, 0)),
                (3, key(KeyCode::Char('r'))),
                (4, None),
            ],
        );
        assert!(
            frames.last().unwrap().text.contains("REPLACED"),
            "{:?}",
            frames.iter().map(|frame| &frame.text).collect::<Vec<_>>()
        );
        let frames = run(
            Control(Element::typed::<Popover>(PopoverProps {
                close_on_trigger_click: true,
                ..props()
            })),
            size,
            vec![
                (2, key(KeyCode::Enter)),
                (4, key(KeyCode::Enter)),
                (5, None),
            ],
        );
        assert!(frames.iter().any(|frame| frame.text.contains("DETAILS")));
        assert!(!frames.last().unwrap().text.contains("DETAILS"));
    }
}

#[test]
fn invalid_popover_constraints_report_errors_without_open_callbacks() {
    for size in [(24, 10), (64, 16)] {
        for mode in 0..3 {
            let calls = Arc::new(AtomicUsize::new(0));
            let opened = calls.clone();
            let mut config = PopoverProps {
                visible: true,
                on_open: Some(Arc::new(move || {
                    opened.fetch_add(1, Ordering::SeqCst);
                })),
                ..props()
            };
            match mode {
                0 => {
                    config.min_width = Some(10);
                    config.max_width = Some(5);
                }
                1 => {
                    config.min_height = Some(10);
                    config.max_height = Some(5);
                }
                _ => config.hover_delay = Duration::MAX,
            }
            let frames = run(
                Control(Element::typed::<Popover>(config)),
                size,
                vec![(1, None)],
            );
            assert!(frames.last().unwrap().text.contains("Invalid Popover"));
            assert_eq!(calls.load(Ordering::SeqCst), 0);
        }
    }
}
impl reactive_tui::app::RootComponent for FocusRoot {
    fn render(&self) -> Element {
        let mut children = Vec::new();
        if !self.removed.load(Ordering::SeqCst) {
            children.push(Element::typed::<Popover>(self.config.clone()).with_key("popover"));
        }
        let removed = self.removed.clone();
        let calls = self.calls.clone();
        children.push(
            builder::button()
                .text("OUTSIDE")
                .class("w-8 h-1 p-0")
                .on_click(move || {
                    calls.lock().unwrap().push("outside");
                    removed.store(true, Ordering::SeqCst);
                })
                .build()
                .with_key("outside"),
        );
        builder::div()
            .class("w-full h-full")
            .children(children)
            .build()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn focus_props(trap: bool, calls: &Arc<Mutex<Vec<&'static str>>>) -> PopoverProps {
    let make = |label: &'static str| {
        let calls = calls.clone();
        builder::button()
            .text(label)
            .class("w-8 h-1 p-0")
            .on_click(move || calls.lock().unwrap().push(label))
            .build()
    };
    PopoverProps {
        trigger_element: make("OPEN").auto_focus(),
        content: builder::div()
            .class("flex-col w-8")
            .children(vec![make("FIRST"), make("SECOND")])
            .build(),
        auto_focus: true,
        focus_trap: trap,
        ..props()
    }
}

#[test]
fn popover_autofocus_targets_real_children_and_escape_restores_trigger() {
    for trap in [false, true] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let config = focus_props(trap, &calls);
        run(
            Control(Element::typed::<Popover>(config)),
            (30, 10),
            vec![
                (2, key(KeyCode::Enter)),
                (4, key(KeyCode::Enter)),
                (4, key(KeyCode::Escape)),
                (5, key(KeyCode::Enter)),
                (6, None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            ["OPEN", "FIRST", "OPEN"],
            "trap={trap}"
        );
    }
}

#[test]
fn nontrapping_popover_allows_tab_out_and_removal_does_not_steal_focus() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let config = focus_props(false, &calls);
    run(
        FocusRoot {
            config,
            calls: calls.clone(),
            removed: Arc::new(AtomicBool::new(false)),
        },
        (30, 10),
        vec![
            (2, key(KeyCode::Enter)),
            (4, key(KeyCode::Tab)),
            (4, key(KeyCode::Tab)),
            (4, key(KeyCode::Enter)),
            (5, key(KeyCode::Enter)),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), ["OPEN", "outside", "outside"]);
}

#[test]
fn nested_popovers_close_in_order_and_restore_focus_for_mixed_traps() {
    for outer_trap in [false, true] {
        for inner_trap in [false, true] {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let button = |label: &'static str| {
                let calls = calls.clone();
                builder::button()
                    .text(label)
                    .class("w-8 h-1 p-0")
                    .on_click(move || calls.lock().unwrap().push(label))
                    .build()
            };
            let inner = PopoverProps {
                trigger_element: button("INNER"),
                content: button("LEAF"),
                auto_focus: true,
                focus_trap: inner_trap,
                ..props()
            };
            let outer = PopoverProps {
                trigger_element: button("OUTER").auto_focus(),
                content: Element::typed::<Popover>(inner),
                auto_focus: true,
                focus_trap: outer_trap,
                ..props()
            };
            run(
                Control(Element::typed::<Popover>(outer)),
                (40, 16),
                vec![
                    (2, key(KeyCode::Enter)),
                    (4, key(KeyCode::Enter)),
                    (7, key(KeyCode::Enter)),
                    (7, key(KeyCode::Escape)),
                    (8, key(KeyCode::Enter)),
                    (9, key(KeyCode::Escape)),
                    (10, key(KeyCode::Escape)),
                    (11, key(KeyCode::Enter)),
                    (12, None),
                ],
            );
            assert_eq!(
                *calls.lock().unwrap(),
                ["OUTER", "INNER", "LEAF", "INNER", "OUTER"],
                "outer trap={outer_trap}, inner trap={inner_trap}"
            );
        }
    }
}

#[test]
fn popover_positions_follow_measured_trigger_and_content_at_two_viewports() {
    use PopoverPosition::*;
    for (width, height) in [(24u16, 12u16), (48, 20)] {
        let (x, y) = (width / 2, height / 2);
        for (position, left, top) in [
            (Top, x - 1, y - 3),
            (TopStart, x, y - 3),
            (TopEnd, x - 2, y - 3),
            (Bottom, x - 1, y + 1),
            (BottomStart, x, y + 1),
            (BottomEnd, x - 2, y + 1),
            (Left, x - 6, y - 1),
            (LeftStart, x - 6, y),
            (LeftEnd, x - 6, y - 2),
            (Right, x + 4, y - 1),
            (RightStart, x + 4, y),
            (RightEnd, x + 4, y - 2),
        ] {
            let config = PopoverProps {
                visible: true,
                position,
                content: Element::text("BODY").class("w-6 h-3"),
                ..props()
            };
            let tree = builder::div()
                .class("relative w-full h-full")
                .child(
                    Element::typed::<Popover>(config).class(format!("absolute left-{x} top-{y}")),
                )
                .build();
            let frames = run(Control(tree), (width, height), vec![(3, None)]);
            let frame = frames.last().unwrap();
            assert_eq!(
                frame.screen.cell(top, left).unwrap().contents(),
                "B",
                "{position:?}: {}",
                frame.text
            );
            assert_eq!(
                frame.screen.cell(y, x).unwrap().contents(),
                "O",
                "trigger moved: {}",
                frame.text
            );
        }
    }
}

#[test]
fn popover_builder_defaults_open_real_content_in_small_viewports() {
    for size in [(24, 8), (48, 12)] {
        let tree = builder::popover()
            .trigger(Element::text("HELP").auto_focus())
            .content(Element::text("Useful content"))
            .build();
        let frames = reactive_tui_test_open(tree, size);
        assert!(
            frames
                .iter()
                .any(|frame| frame.text.contains("Useful content")),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(!frames.iter().any(|frame| frame.text.contains("Popover")));
    }
}

fn reactive_tui_test_open(tree: Element, size: (u16, u16)) -> Vec<super::app_input::Snapshot> {
    super::app_input::run_when(
        Control(tree),
        size,
        vec![("HELP", key(KeyCode::Enter)), ("Useful content", None)],
    )
}

fn props() -> PopoverProps {
    PopoverProps {
        trigger_element: Element::text("OPEN").class("w-4 h-1").auto_focus(),
        content: Element::text("DETAILS").class("w-7 h-2"),
        animation: PopoverAnimation::None,
        arrow: PopoverArrow {
            enabled: false,
            ..Default::default()
        },
        offset: (0, 0),
        position: PopoverPosition::BottomStart,
        ..Default::default()
    }
}

#[test]
fn popover_boundary_modes_use_the_presented_viewport() {
    for (width, height) in [(24u16, 12u16), (48, 20)] {
        for behavior in [
            BoundaryBehavior::Flip,
            BoundaryBehavior::Shift,
            BoundaryBehavior::Hide,
            BoundaryBehavior::Ignore,
        ] {
            let config = PopoverProps {
                visible: true,
                boundary_behavior: behavior,
                content: Element::text("BODY").class("w-6 h-3"),
                ..props()
            };
            let tree = builder::div()
                .class("relative w-full h-full")
                .child(
                    Element::typed::<Popover>(config)
                        .class(format!("absolute left-2 top-{}", height - 2)),
                )
                .build();
            let frames = run(Control(tree), (width, height), vec![(3, None)]);
            let frame = frames.last().unwrap();
            let row = match behavior {
                BoundaryBehavior::Flip => Some(height - 5),
                BoundaryBehavior::Shift => Some(height - 3),
                BoundaryBehavior::Ignore => Some(height - 1),
                BoundaryBehavior::Hide => None,
            };
            if let Some(row) = row {
                assert_eq!(
                    frame.screen.cell(row, 2).unwrap().contents(),
                    "B",
                    "{behavior:?}: {}",
                    frame.text
                );
            } else {
                assert!(!frame.text.contains("BODY"), "{}", frame.text);
            }
        }
    }
}

#[test]
fn popover_reports_measured_position_and_repositions_after_resize() {
    use reactive_tui::event::types::{Event, ResizeEvent};
    let positions = Arc::new(Mutex::new(Vec::new()));
    let reported = positions.clone();
    let config = PopoverProps {
        visible: true,
        position: PopoverPosition::RightStart,
        on_position_change: Some(Arc::new(move |position| {
            reported.lock().unwrap().push(position)
        })),
        content: builder::button().text("BODY").class("w-6 h-2 p-0").build(),
        ..props()
    };
    let tree = builder::div()
        .class("relative w-full h-full")
        .child(Element::typed::<Popover>(config).class("absolute right-4 top-2"))
        .build();
    let frames = run(
        Control(tree),
        (32, 12),
        vec![(3, Some(Event::Resize(ResizeEvent::new(20, 8)))), (5, None)],
    );
    assert_eq!(*positions.lock().unwrap(), [PopoverPosition::LeftStart]);
    assert_eq!(
        frames[2].screen.cell(2, 18).unwrap().contents(),
        "B",
        "{}",
        frames[2].text
    );
    // The first frame after the resize already shows the new position.
    assert_eq!(
        frames[3].screen.cell(2, 6).unwrap().contents(),
        "B",
        "{}",
        frames[3].text
    );
    assert_eq!(
        frames.last().unwrap().screen.cell(2, 6).unwrap().contents(),
        "B",
        "{}",
        frames.last().unwrap().text
    );
}

#[test]
fn popover_initial_position_callback_waits_for_measured_content() {
    let positions = Arc::new(Mutex::new(Vec::new()));
    let reported = positions.clone();
    let config = PopoverProps {
        visible: true,
        position: PopoverPosition::RightStart,
        on_position_change: Some(Arc::new(move |position| {
            reported.lock().unwrap().push(position)
        })),
        ..props()
    };
    run(
        Control(Element::typed::<Popover>(config)),
        (32, 12),
        vec![(3, None)],
    );
    assert_eq!(*positions.lock().unwrap(), [PopoverPosition::RightStart]);
}

#[test]
fn ignoring_boundaries_keeps_intrinsic_content_size_before_terminal_clipping() {
    let config = PopoverProps {
        visible: true,
        position: PopoverPosition::BottomEnd,
        boundary_behavior: BoundaryBehavior::Ignore,
        content: Element::text("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789ABCD").class("w-40 h-1"),
        ..props()
    };
    let tree = builder::div()
        .class("relative w-full h-full")
        .child(Element::typed::<Popover>(config).class("absolute left-10 top-2"))
        .build();
    let frames = run(Control(tree), (24, 10), vec![(3, None)]);
    assert_eq!(
        frames
            .last()
            .unwrap()
            .screen
            .cell(3, 13)
            .unwrap()
            .contents(),
        "D",
        "{}",
        frames.last().unwrap().text
    );
}

#[test]
fn popover_animation_paints_intermediate_frames_and_slide_keeps_arrow_attached() {
    for size in [(24, 16), (48, 24)] {
        for animation in [
            PopoverAnimation::Fade,
            PopoverAnimation::Scale,
            PopoverAnimation::Slide,
            PopoverAnimation::Bounce,
        ] {
            let config = PopoverProps {
                visible: true,
                animation,
                animation_duration: Duration::from_secs(1),
                content: Element::text("BODYBBBBBBBBBB").class("w-14 h-5 text-red-500 bg-blue-500"),
                offset: (0, 3),
                arrow: PopoverArrow {
                    enabled: true,
                    size: 2,
                    offset: 0,
                    style: ArrowStyle::Solid,
                },
                ..props()
            };
            let tree = builder::div()
                .class("relative w-full h-full")
                .child(Element::typed::<Popover>(config).class("absolute left-4 top-4"))
                .build();
            let frames = if animation == PopoverAnimation::Slide {
                super::app_input::run_when_seen(Control(tree), size, &["BODY", "▲"], 3)
            } else {
                run(Control(tree), size, vec![(12, None)])
            };
            if animation == PopoverAnimation::Fade {
                let colors: Vec<_> = frames
                    .iter()
                    .skip(3)
                    .map(|frame| frame.screen.cell(8, 4).unwrap().fgcolor())
                    .collect();
                assert!(colors.iter().any(|color| *color != colors[0]), "{colors:?}");
            } else {
                assert!(
                    frames
                        .iter()
                        .skip(3)
                        .any(|frame| frame.screen.contents_formatted()
                            != frames[2].screen.contents_formatted()),
                    "{animation:?}: {:?}",
                    frames.iter().map(|f| &f.text).collect::<Vec<_>>()
                );
            }
            if animation == PopoverAnimation::Slide {
                let mut matched = 0;
                for frame in &frames {
                    let body = frame.text.lines().position(|line| line.contains("BODY"));
                    let arrow = frame.text.lines().position(|line| line.contains('▲'));
                    if let (Some(body), Some(arrow)) = (body, arrow) {
                        assert_eq!(body - arrow, 2, "{}", frame.text);
                        matched += 1;
                    }
                }
                assert!(
                    matched >= 3,
                    "only {matched} slide frames at {size:?}: {:?}",
                    frames.iter().map(|frame| &frame.text).collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
fn popover_arrows_point_toward_trigger_and_render_each_style() {
    for size in [(24, 16), (48, 24)] {
        for (position, x, y, tips) in [
            (PopoverPosition::Bottom, 12, 9, ["▲", "△", "⇈"]),
            (PopoverPosition::Top, 12, 3, ["▼", "▽", "⇊"]),
            (PopoverPosition::Right, 16, 8, ["◀", "◁", "⇇"]),
            (PopoverPosition::Left, 7, 8, ["▶", "▷", "⇉"]),
        ] {
            for (style, tip) in [ArrowStyle::Solid, ArrowStyle::Outline, ArrowStyle::Double]
                .into_iter()
                .zip(tips)
            {
                let config = PopoverProps {
                    visible: true,
                    position,
                    offset: (4, 4),
                    boundary_behavior: BoundaryBehavior::Ignore,
                    content: Element::text("BODY").class("w-7 h-5"),
                    arrow: PopoverArrow {
                        enabled: true,
                        size: 2,
                        offset: 0,
                        style,
                    },
                    ..props()
                };
                let tree = builder::div()
                    .class("relative w-full h-full")
                    .child(Element::typed::<Popover>(config).class("absolute left-10 top-6"))
                    .build();
                let frames = run(Control(tree), size, vec![(3, None)]);
                let frame = frames.last().unwrap();
                assert_eq!(
                    frame.screen.cell(y, x).unwrap().contents(),
                    tip,
                    "{position:?}/{style:?}: {}",
                    frame.text
                );
            }
        }
    }
}

#[test]
fn popover_keeps_trigger_and_opens_actual_content_then_closes_on_escape() {
    for size in [(24, 8), (48, 12)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let opened = calls.clone();
        let closed = calls.clone();
        let config = PopoverProps {
            on_open: Some(Arc::new(move || opened.lock().unwrap().push("open"))),
            on_close: Some(Arc::new(move || closed.lock().unwrap().push("close"))),
            ..props()
        };
        let frames = run(
            Control(Element::typed::<Popover>(config)),
            size,
            vec![
                (2, key(KeyCode::Enter)),
                (4, key(KeyCode::Escape)),
                (5, None),
            ],
        );
        assert!(!frames[0].text.contains("DETAILS"));
        assert!(
            frames.iter().any(|f| f
                .text
                .lines()
                .nth(1)
                .is_some_and(|line| line.starts_with("DETAILS"))),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(!frames.last().unwrap().text.contains("DETAILS"));
        assert_eq!(*calls.lock().unwrap(), ["open", "close"]);
    }
}

#[test]
fn popover_trigger_observes_consuming_button_and_outside_click_does_not_leak() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let button = calls.clone();
    let background = calls.clone();
    let config = PopoverProps {
        trigger_element: builder::button()
            .text("OPEN")
            .class("w-4 h-1 p-0")
            .on_click(move || button.lock().unwrap().push("trigger"))
            .build(),
        ..props()
    };
    let tree = builder::div()
        .class("w-full h-full")
        .children(vec![
            Element::typed::<Popover>(config),
            builder::button()
                .text("BACKGROUND")
                .class("absolute left-12 top-5 w-10 h-1 p-0")
                .on_click(move || background.lock().unwrap().push("background"))
                .build(),
        ])
        .build();
    let frames = run(
        Control(tree),
        (30, 10),
        vec![(2, click(1, 0)), (4, click(13, 5)), (5, None)],
    );
    assert_eq!(*calls.lock().unwrap(), ["trigger"]);
    assert!(!frames.last().unwrap().text.contains("DETAILS"));
}
