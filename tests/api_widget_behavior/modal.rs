use super::{app_input, builder, click, key, run, Control, Element};
use reactive_tui::event::types::KeyCode;
use reactive_tui::widgets::display::{modal::*, Border};
use std::sync::{Arc, Mutex};

struct LaunchRoot {
    config: Arc<Mutex<ModalProps>>,
    calls: Arc<Mutex<Vec<String>>>,
}
impl reactive_tui::app::RootComponent for LaunchRoot {
    fn render(&self) -> Element {
        let config = self.config.clone();
        let calls = self.calls.clone();
        builder::div()
            .class("w-full h-full")
            .children(vec![
                builder::button()
                    .text("LAUNCH")
                    .class("w-6 h-1 p-0")
                    .on_click(move || {
                        calls.lock().unwrap().push("launch".into());
                        config.lock().unwrap().visible = true;
                    })
                    .build()
                    .auto_focus(),
                Element::typed::<Modal>(self.config.lock().unwrap().clone()),
                Element::text({
                    let calls = self.calls.lock().unwrap();
                    format!("C{}:{}", calls.len(), calls.join(","))
                })
                .class("absolute bottom-0 left-0 z-[30000]"),
            ])
            .build()
    }
    fn wake_driven(&self) -> bool {
        true
    }
    fn handle_event(
        &self,
        event: &reactive_tui::event::types::Event,
    ) -> reactive_tui::event::router::EventResult {
        use reactive_tui::event::{router::EventResult, types::Event};
        if matches!(event, Event::Key(key) if key.code == KeyCode::Char('r')) {
            let mut props = self.config.lock().unwrap();
            props.content = Some(Element::text("NEW"));
            props.footer = Some(Element::text("FOOT"));
            let calls = self.calls.clone();
            let weak = Arc::downgrade(&self.config);
            props.on_close = Some(Arc::new(move |_| {
                weak.upgrade().unwrap().lock().unwrap().visible = false;
                calls.lock().unwrap().push("new close".into());
            }));
            EventResult::Consumed
        } else if matches!(event, Event::Key(key) if key.code == KeyCode::Char('v')) {
            self.config.lock().unwrap().width = ModalSize::Fixed(12);
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }
}

fn props() -> ModalProps {
    ModalProps {
        visible: true,
        content: Some(Element::text("DETAILS")),
        width: ModalSize::Fixed(12),
        height: ModalSize::Fixed(6),
        closable: false,
        scrollable: false,
        animation: ModalAnimation::None,
        border: Border {
            enabled: false,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn modal_positions_actual_content_in_the_measured_viewport() {
    for (width, height) in [(24u16, 12u16), (48, 20)] {
        let frames = app_input::run_when(
            Control(Element::typed::<Modal>(props())),
            (width, height),
            vec![("DETAILS", None)],
        );
        let frame = frames.last().unwrap();
        assert_eq!(
            frame
                .screen
                .cell((height - 6) / 2, (width - 12) / 2)
                .unwrap()
                .contents(),
            "D",
            "{}",
            frame.text
        );
        let cell = frame
            .screen
            .cell((height - 6) / 2, (width - 12) / 2)
            .unwrap();
        assert_ne!(
            cell.fgcolor(),
            cell.bgcolor(),
            "default modal text must contrast with its background"
        );
    }
}

#[test]
fn modal_position_and_size_options_use_cells_and_actual_content() {
    use ModalPosition::*;
    for (width, height) in [(24u16, 12u16), (48, 20)] {
        for (position, x, y) in [
            (Center, (width - 12) / 2, (height - 6) / 2),
            (Top, (width - 12) / 2, 0),
            (Bottom, (width - 12) / 2, height - 6),
            (Left, 0, (height - 6) / 2),
            (Right, width - 12, (height - 6) / 2),
            (TopLeft, 0, 0),
            (TopRight, width - 12, 0),
            (BottomLeft, 0, height - 6),
            (BottomRight, width - 12, height - 6),
            (Custom { x: 3, y: 2 }, 3, 2),
        ] {
            let config = ModalProps {
                position: position.clone(),
                ..props()
            };
            let frames = app_input::run_when(
                Control(Element::typed::<Modal>(config)),
                (width, height),
                vec![("DETAILS", None)],
            );
            assert_eq!(
                frames.last().unwrap().screen.cell(y, x).unwrap().contents(),
                "D",
                "{position:?}: {}",
                frames.last().unwrap().text
            );
        }
        for (modal_width, modal_height, w, h) in [
            (
                ModalSize::Percent(50.0),
                ModalSize::Percent(50.0),
                width / 2,
                height / 2,
            ),
            (
                ModalSize::Viewport(0.5),
                ModalSize::Viewport(0.5),
                width / 2,
                height / 2,
            ),
            (ModalSize::Auto, ModalSize::Auto, 7, 1),
        ] {
            let config = ModalProps {
                width: modal_width,
                height: modal_height,
                scrollable: true,
                ..props()
            };
            let frames = app_input::run_when(
                Control(Element::typed::<Modal>(config)),
                (width, height),
                vec![("DETAILS", None)],
            );
            assert_eq!(
                frames
                    .last()
                    .unwrap()
                    .screen
                    .cell((height - h) / 2, (width - w) / 2)
                    .unwrap()
                    .contents(),
                "D",
                "{w}x{h}: {}",
                frames.last().unwrap().text
            );
        }
    }
}

#[test]
fn modal_close_button_and_backdrop_deliver_distinct_reasons() {
    for size in [(24, 12), (48, 20)] {
        for close_button in [false, true] {
            let reasons = Arc::new(Mutex::new(Vec::new()));
            let output = reasons.clone();
            let config = ModalProps {
                closable: true,
                title: Some("TITLE".into()),
                on_close: Some(Arc::new(move |reason| output.lock().unwrap().push(reason))),
                ..props()
            };
            let point = if close_button {
                ((size.0 - 12) / 2 + 11, (size.1 - 6) / 2)
            } else {
                (0, 0)
            };
            let frames = run(
                Control(Element::typed::<Modal>(config)),
                size,
                vec![(3, click(point.0, point.1)), (4, None)],
            );
            assert_eq!(
                *reasons.lock().unwrap(),
                [if close_button {
                    ModalCloseReason::CloseButton
                } else {
                    ModalCloseReason::BackdropClick
                }]
            );
            assert!(!frames.last().unwrap().text.contains("DETAILS"));
        }
    }
}

fn motion(
    kind: reactive_tui::event::types::MouseEventKind,
    x: u16,
    y: u16,
) -> Option<reactive_tui::event::types::Event> {
    use reactive_tui::event::types::{Event, MouseEvent, Position};
    Some(Event::Mouse(MouseEvent::new(kind, Position::cell(x, y))))
}

#[test]
fn modal_title_drag_follows_pointer_and_stops_on_release() {
    use reactive_tui::event::types::MouseEventKind;
    for size in [(24, 12), (48, 20)] {
        let config = ModalProps {
            position: ModalPosition::Custom { x: 2, y: 2 },
            title: Some("TITLE".into()),
            draggable: true,
            ..props()
        };
        let frames = run(
            Control(Element::typed::<Modal>(config)),
            size,
            vec![
                (3, click(3, 2)),
                (4, motion(MouseEventKind::Move, 7, 4)),
                (5, motion(MouseEventKind::Up, 7, 4)),
                (6, motion(MouseEventKind::Move, 9, 7)),
                (6, None),
            ],
        );
        let frame = frames.last().unwrap();
        assert_eq!(
            frame.screen.cell(4, 6).unwrap().contents(),
            "T",
            "{}",
            frame.text
        );
        assert_eq!(
            frame.screen.cell(5, 6).unwrap().contents(),
            "D",
            "{}",
            frame.text
        );
    }
}

#[test]
fn modal_resize_handles_change_all_edges() {
    use reactive_tui::event::types::MouseEventKind;
    for size in [(24, 14), (48, 24)] {
        for (start, end, expected) in [
            ((3, 3), (2, 2), (2, 2, 13, 7)),
            ((8, 3), (8, 2), (3, 2, 12, 7)),
            ((14, 3), (16, 2), (3, 2, 14, 7)),
            ((3, 5), (2, 5), (2, 3, 13, 6)),
            ((14, 5), (16, 5), (3, 3, 14, 6)),
            ((3, 8), (2, 10), (2, 3, 13, 8)),
            ((8, 8), (8, 10), (3, 3, 12, 8)),
            ((14, 8), (16, 10), (3, 3, 14, 8)),
        ] {
            let config = ModalProps {
                position: ModalPosition::Custom { x: 3, y: 3 },
                resizable: true,
                title: Some("TITLE".into()),
                ..props()
            };
            let frames = run(
                Control(Element::typed::<Modal>(config)),
                size,
                vec![
                    (3, click(start.0, start.1)),
                    (4, motion(MouseEventKind::Move, end.0, end.1)),
                    (5, motion(MouseEventKind::Up, end.0, end.1)),
                    (6, None),
                ],
            );
            assert_eq!(
                frames
                    .last()
                    .unwrap()
                    .screen
                    .cell(expected.1, expected.0)
                    .unwrap()
                    .contents(),
                "T",
                "{start:?}->{end:?}: {}",
                frames.last().unwrap().text
            );
            let screen = &frames.last().unwrap().screen;
            let (right, bottom) = (expected.0 + expected.2 - 1, expected.1 + expected.3 - 1);
            let inside = screen
                .cell(expected.1 + 2, expected.0 + 2)
                .unwrap()
                .bgcolor();
            assert_eq!(screen.cell(bottom, right).unwrap().bgcolor(), inside);
            assert_ne!(screen.cell(bottom + 1, right).unwrap().bgcolor(), inside);
            assert_ne!(screen.cell(bottom, right + 1).unwrap().bgcolor(), inside);
        }
    }
}

#[test]
fn modal_keeps_real_child_editing_and_scrolls_overflowing_content() {
    for size in [(24, 12), (48, 20)] {
        let config = ModalProps {
            content: Some(
                builder::text_input()
                    .placeholder("TYPE")
                    .build()
                    .auto_focus(),
            ),
            ..props()
        };
        let frames = app_input::run_when(
            Control(Element::typed::<Modal>(config)),
            size,
            vec![
                ("TYPE", key(KeyCode::Char('界'))),
                ("界", key(KeyCode::Char('x'))),
                ("界x", None),
            ],
        );
        assert!(frames.last().unwrap().text.contains("界x"));
        let content = (0..20)
            .map(|i| format!("ROW{i:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        let config = ModalProps {
            content: Some(Element::text(content).class("whitespace-pre w-8 h-20")),
            scrollable: true,
            ..props()
        };
        let frames = app_input::run_when(
            Control(Element::typed::<Modal>(config)),
            size,
            vec![("ROW00", key(KeyCode::End)), ("ROW19", None)],
        );
        assert!(!frames.last().unwrap().text.contains("ROW00"));
    }
}

#[test]
fn modal_custom_cancel_and_disabled_buttons_use_actual_focus_and_restore_opener() {
    for trap in [false, true] {
        let config = Arc::new(Mutex::new(ModalProps {
            visible: false,
            focus_trap: trap,
            backdrop_style: None,
            closable: true,
            ..props()
        }));
        let calls = Arc::new(Mutex::new(Vec::new()));
        let weak = Arc::downgrade(&config);
        let closed = calls.clone();
        let cancelled = calls.clone();
        let button = calls.clone();
        {
            let mut props = config.lock().unwrap();
            props.buttons = vec![
                ModalButton::ok().disabled(true),
                ModalButton::new(
                    "apply",
                    "Apply",
                    ModalButtonAction::Custom("apply-action".into()),
                )
                .autofocus(true),
                ModalButton::cancel(),
            ];
            props.on_button_click = Some(Arc::new(move |id| button.lock().unwrap().push(id)));
            props.on_cancel = Some(Arc::new(move || {
                cancelled.lock().unwrap().push("cancelled".into())
            }));
            props.on_close = Some(Arc::new(move |_| {
                weak.upgrade().unwrap().lock().unwrap().visible = false;
                closed.lock().unwrap().push("closed".into());
            }));
        }
        app_input::run_when(
            LaunchRoot {
                config,
                calls: calls.clone(),
            },
            (40, 16),
            vec![
                ("C0", key(KeyCode::Enter)),
                ("DETAILS", key(KeyCode::Enter)),
                ("C2", key(KeyCode::Tab)),
                ("C2", key(KeyCode::Enter)),
                ("C5", key(KeyCode::Enter)),
                ("C6", key(KeyCode::Escape)),
                ("C7", None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            [
                "launch",
                "apply-action",
                "cancel",
                "cancelled",
                "closed",
                "launch",
                "closed"
            ],
            "trap={trap}"
        );
    }
}

#[test]
fn modal_animations_change_presented_frames_without_input() {
    struct Settled {
        config: ModalProps,
        started: std::time::Instant,
        done: bool,
    }
    impl reactive_tui::app::RootComponent for Settled {
        fn render(&self) -> Element {
            let mut children = vec![Element::typed::<Modal>(self.config.clone())];
            if self.done {
                children.push(Element::text("SETTLED"));
            }
            reactive_tui::builder::div()
                .class("relative w-full h-full")
                .children(children)
                .build()
        }
        fn update(&mut self) -> reactive_tui::error::Result<reactive_tui::app::RootUpdate> {
            if !self.done && self.started.elapsed() >= std::time::Duration::from_millis(500) {
                self.done = true;
                Ok(reactive_tui::app::RootUpdate::Redraw)
            } else {
                Ok(reactive_tui::app::RootUpdate::Unchanged)
            }
        }
    }
    for size in [(32, 16), (60, 24)] {
        for animation in [
            ModalAnimation::Fade,
            ModalAnimation::Slide,
            ModalAnimation::Scale,
            ModalAnimation::Bounce,
        ] {
            let config = ModalProps {
                width: ModalSize::Fixed(24),
                height: ModalSize::Fixed(8),
                content: Some(
                    Element::text("ABCDEFGHIJKLMNOPQRSTUVWX").class("w-24 h-6 text-red-500"),
                ),
                animation: animation.clone(),
                ..props()
            };
            let frames = super::app_input::run_when(
                Settled {
                    config,
                    started: std::time::Instant::now(),
                    done: false,
                },
                size,
                vec![("SETTLED", None)],
            );
            let states: Vec<_> = frames
                .iter()
                .skip(2)
                .filter(|frame| !frame.text.contains("SETTLED"))
                .map(|frame| frame.screen.contents_formatted())
                .collect();
            assert!(
                states.iter().skip(1).any(|state| *state != states[0]),
                "{animation:?}: {:?}",
                frames.iter().map(|frame| &frame.text).collect::<Vec<_>>()
            );
            assert!(frames
                .last()
                .unwrap()
                .text
                .contains("ABCDEFGHIJKLMNOPQRSTUVWX"));
        }
    }
}

#[test]
fn modal_reduced_motion_and_invalid_or_zero_dimensions_are_bounded() {
    for size in [(24, 12), (60, 20)] {
        for mode in 0..5 {
            let mut config = props();
            match mode {
                0 => config.width = ModalSize::Percent(f32::NAN),
                1 => config.height = ModalSize::Viewport(-1.0),
                2 => config.buttons = vec![ModalButton::ok(), ModalButton::ok()],
                3 => config.width = ModalSize::Fixed(0),
                _ => {
                    config.animation = ModalAnimation::Bounce;
                    config.modal_style = Some("reduced-motion".into());
                }
            }
            let frames = run(
                Control(Element::typed::<Modal>(config)),
                size,
                vec![(if mode < 3 { 1 } else { 3 }, None)],
            );
            let frame = frames.last().unwrap();
            if mode < 3 {
                assert!(frame.text.contains("Invalid Modal"));
            } else if mode == 3 {
                assert!(!frame.text.contains("DETAILS"));
            } else {
                assert!(frame.text.contains("DETAILS"));
            }
        }
    }
}

#[test]
fn modal_replaces_content_footer_and_close_callback_without_remounting() {
    for size in [(24, 12), (48, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let old = calls.clone();
        let config = Arc::new(Mutex::new(ModalProps {
            closable: true,
            backdrop_style: None,
            on_close: Some(Arc::new(move |_| {
                old.lock().unwrap().push("old close".into())
            })),
            ..props()
        }));
        let frames = app_input::run_when(
            LaunchRoot {
                config,
                calls: calls.clone(),
            },
            size,
            vec![
                ("DETAILS", key(KeyCode::Char('r'))),
                ("FOOT", key(KeyCode::Escape)),
                ("C1", None),
            ],
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("NEW") && frame.text.contains("FOOT")));
        assert_eq!(*calls.lock().unwrap(), ["new close"]);
    }
}

#[test]
fn modal_viewport_resize_moves_its_close_target() {
    use reactive_tui::event::types::{Event, ResizeEvent};
    let reasons = Arc::new(Mutex::new(Vec::new()));
    let closed = reasons.clone();
    let config = ModalProps {
        closable: true,
        title: Some("TITLE".into()),
        on_close: Some(Arc::new(move |reason| closed.lock().unwrap().push(reason))),
        ..props()
    };
    let frames = run(
        Control(Element::typed::<Modal>(config)),
        (32, 16),
        vec![
            (3, Some(Event::Resize(ResizeEvent::new(24, 12)))),
            (5, click(17, 3)),
            (6, None),
        ],
    );
    assert_eq!(*reasons.lock().unwrap(), [ModalCloseReason::CloseButton]);
    assert!(!frames.last().unwrap().text.contains("TITLE"));
}

#[test]
fn modal_border_and_region_styles_reach_actual_cells() {
    use reactive_tui::widgets::display::BorderStyle;
    for (style, corner) in [
        (BorderStyle::Single, "┌"),
        (BorderStyle::Double, "╔"),
        (BorderStyle::Rounded, "╭"),
        (BorderStyle::Thick, "┏"),
    ] {
        let config = ModalProps {
            border: Border {
                enabled: true,
                style,
                color: Some("red-500".into()),
            },
            ..props()
        };
        let frames = app_input::run_when(
            Control(Element::typed::<Modal>(config)),
            (24, 12),
            vec![("DETAILS", None)],
        );
        let cell = frames.last().unwrap().screen.cell(3, 6).unwrap();
        assert_eq!(cell.contents(), corner);
        assert_eq!(cell.fgcolor(), vt100::Color::Rgb(239, 68, 68));
    }
    let config = ModalProps {
        width: ModalSize::Fixed(16),
        height: ModalSize::Fixed(10),
        title: Some("TITLE".into()),
        footer: Some(Element::text("FOOT")),
        header_style: Some("text-blue-500".into()),
        content_style: Some("p-0.5 text-red-500".into()),
        footer_style: Some("text-green-500".into()),
        border: Border::default(),
        ..props()
    };
    let frames = app_input::run_when(
        Control(Element::typed::<Modal>(config)),
        (32, 16),
        vec![("FOOT", None)],
    );
    let frame = frames.last().unwrap();
    for (y, x, letter, color) in [
        (4, 9, "T", (59, 130, 246)),
        (7, 11, "D", (239, 68, 68)),
        (11, 9, "F", (34, 197, 94)),
    ] {
        let cell = frame.screen.cell(y, x).unwrap();
        assert_eq!(cell.contents(), letter, "{}", frame.text);
        assert_eq!(cell.fgcolor(), vt100::Color::Rgb(color.0, color.1, color.2));
    }
}

#[test]
fn foreground_inheritance_preserves_child_overrides_and_sibling_isolation() {
    let tree = builder::div()
        .class("flex-col w-full h-full text-red-500")
        .children(vec![
            Element::text("INHERIT"),
            builder::div()
                .class("text-blue-500")
                .child(Element::text("OVERRIDE"))
                .build(),
            Element::text("SIBLING"),
        ])
        .build();
    let frames = run(Control(tree), (24, 8), vec![(1, None)]);
    let screen = &frames.last().unwrap().screen;
    assert_eq!(
        screen.cell(0, 0).unwrap().fgcolor(),
        vt100::Color::Rgb(239, 68, 68)
    );
    assert_eq!(
        screen.cell(1, 0).unwrap().fgcolor(),
        vt100::Color::Rgb(59, 130, 246)
    );
    assert_eq!(
        screen.cell(2, 0).unwrap().fgcolor(),
        vt100::Color::Rgb(239, 68, 68)
    );
}

#[test]
fn modal_builder_honors_hidden_state_instead_of_painting_a_description() {
    let tree = builder::div()
        .class("w-full h-full")
        .children(vec![
            Element::text("OUTSIDE"),
            builder::modal()
                .title("Secret")
                .content(Element::text("PRIVATE"))
                .visible(false)
                .size(12, 6)
                .build(),
        ])
        .build();
    let frames = run(Control(tree), (24, 12), vec![(1, None)]);
    let frame = frames.last().unwrap();
    assert!(frame.text.contains("OUTSIDE"));
    assert!(!frame.text.contains("PRIVATE"));
    assert!(!frame.text.contains("Modal:"));
}

#[test]
fn modal_named_route_runs_button_action_and_closes_once() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let confirmed = calls.clone();
    let closed = calls.clone();
    let config = ModalProps {
        closable: true,
        buttons: vec![ModalButton::ok().autofocus(true)],
        on_confirm: Some(Arc::new(move || confirmed.lock().unwrap().push("confirm"))),
        on_close: Some(Arc::new(move |reason| {
            assert_eq!(reason, ModalCloseReason::ButtonClick("ok".into()));
            closed.lock().unwrap().push("close");
        })),
        ..props()
    };
    let frames = run(
        Control(Modal::with_props(config)),
        (24, 12),
        vec![
            (3, key(KeyCode::Enter)),
            (4, key(KeyCode::Escape)),
            (4, None),
        ],
    );
    assert_eq!(*calls.lock().unwrap(), ["confirm", "close"]);
    assert!(!frames.last().unwrap().text.contains("DETAILS"));
}

#[test]
fn modal_dismissal_flags_preserve_explicit_action_buttons() {
    for size in [(24, 12), (48, 20)] {
        for (closable, keyboard_navigation, backdrop_clickable) in [
            (false, true, true),
            (true, false, false),
            (true, true, false),
        ] {
            let reasons = Arc::new(Mutex::new(Vec::new()));
            let closed = reasons.clone();
            let config = ModalProps {
                closable,
                keyboard_navigation,
                backdrop_clickable,
                buttons: vec![ModalButton::ok()],
                on_close: Some(Arc::new(move |reason| closed.lock().unwrap().push(reason))),
                ..props()
            };
            // Escape is disabled in the first two cases. The third case verifies
            // a disabled backdrop independently of keyboard dismissal.
            let mut events = vec![(3, click(0, 0))];
            if !closable || !keyboard_navigation {
                events.push((4, key(KeyCode::Escape)));
            }
            events.push((4, None));
            let frames = run(
                Control(Element::typed::<Modal>(config.clone())),
                size,
                events,
            );
            assert!(reasons.lock().unwrap().is_empty());
            let frame = frames.last().unwrap();
            assert!(frame.text.contains("DETAILS"));
            let point = (0..size.1)
                .flat_map(|y| (0..size.0).map(move |x| (x, y)))
                .find(|&(x, y)| frame.screen.cell(y, x).unwrap().contents() == "O")
                .expect("OK button must remain visible");
            let frames = run(
                Control(Element::typed::<Modal>(config)),
                size,
                vec![(3, click(point.0, point.1)), (4, None)],
            );
            assert_eq!(
                *reasons.lock().unwrap(),
                [ModalCloseReason::ButtonClick("ok".into())]
            );
            assert!(!frames.last().unwrap().text.contains("DETAILS"));
        }
    }
}

#[test]
fn modal_drag_clamps_to_viewport_and_cancels_on_focus_loss() {
    use reactive_tui::event::types::{Event, FocusEvent, FocusEventKind, MouseEventKind};
    for size in [(24, 12), (48, 20)] {
        let config = ModalProps {
            position: ModalPosition::Custom { x: 2, y: 2 },
            title: Some("TITLE".into()),
            draggable: true,
            ..props()
        };
        let frames = run(
            Control(Element::typed::<Modal>(config)),
            size,
            vec![
                (3, click(3, 2)),
                (4, motion(MouseEventKind::Move, size.0 - 1, size.1 - 1)),
                (
                    5,
                    Some(Event::Focus(FocusEvent {
                        kind: FocusEventKind::Lost,
                        timestamp: std::time::Instant::now(),
                    })),
                ),
                (6, motion(MouseEventKind::Move, 0, 0)),
                (7, None),
            ],
        );
        let frame = frames.last().unwrap();
        assert_eq!(
            frame
                .screen
                .cell(size.1 - 6, size.0 - 12)
                .unwrap()
                .contents(),
            "T",
            "{}",
            frame.text
        );
    }
}

#[test]
fn modal_generic_builder_preserves_multiple_children_and_dismissal_options() {
    for size in [(32, 16), (48, 24)] {
        let element: Element = builder::modal()
            .title("SETTINGS")
            .content(Element::text("FIRST"))
            .contents(vec![Element::text("SECOND")])
            .visible(true)
            .backdrop_dismissible(false)
            .size(20, 10)
            .class("bg-white text-blue-500 reduced-motion")
            .into();
        let frames = run(
            Control(element),
            size,
            vec![(3, click(0, 0)), (4, key(KeyCode::Escape)), (5, None)],
        );
        assert!(frames.iter().any(|frame| frame.text.contains("SETTINGS")
            && frame.text.contains("FIRST")
            && frame.text.contains("SECOND")));
        assert!(!frames.last().unwrap().text.contains("SECOND"));
    }
}

#[test]
fn modal_empty_content_still_offers_a_functional_close_action() {
    for size in [(24, 12), (48, 20)] {
        let config = Modal::visible(ModalProps::default(), true);
        let config = Modal::with_title(config, "EMPTY");
        let config = Modal::with_size(config, ModalSize::Fixed(12), ModalSize::Fixed(6));
        let config = Modal::with_position(config, ModalPosition::TopLeft);
        let config = Modal::closable(config, false);
        let config = Modal::with_buttons(config, vec![ModalButton::close().autofocus(true)]);
        let frames = run(
            Control(Modal::with_props(ModalProps {
                animation: ModalAnimation::None,
                ..config
            })),
            size,
            vec![(3, key(KeyCode::Enter)), (4, None)],
        );
        assert!(!frames.last().unwrap().text.contains("EMPTY"));
        assert!(!frames.last().unwrap().text.contains("Close"));
    }
}

#[test]
fn modal_z_order_routes_overlapping_close_targets_to_the_visible_dialog() {
    for size in [(24, 12), (48, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let upper = calls.clone();
        let lower = calls.clone();
        let tree = builder::div()
            .class("w-full h-full")
            .children(vec![
                Element::typed::<Modal>(ModalProps {
                    title: Some("UPPER".into()),
                    closable: true,
                    z_index: 2000,
                    on_close: Some(Arc::new(move |_| upper.lock().unwrap().push("upper"))),
                    ..props()
                })
                .with_key("upper"),
                Element::typed::<Modal>(ModalProps {
                    title: Some("LOWER".into()),
                    closable: true,
                    z_index: 1000,
                    on_close: Some(Arc::new(move |_| lower.lock().unwrap().push("lower"))),
                    ..props()
                })
                .with_key("lower"),
            ])
            .build();
        let close = ((size.0 - 12) / 2 + 11, (size.1 - 6) / 2);
        let frames = run(
            Control(tree),
            size,
            vec![
                (3, click(close.0, close.1)),
                (4, click(close.0, close.1)),
                (5, None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), ["upper", "lower"]);
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("UPPER") && !frame.text.contains("LOWER")));
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("LOWER") && !frame.text.contains("UPPER")));
        assert!(!frames.last().unwrap().text.contains("DETAILS"));
    }
}

#[test]
fn modal_nested_escape_closes_inner_then_restores_outer_focus() {
    for size in [(32, 16), (48, 24)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let inner = calls.clone();
        let outer = calls.clone();
        let child = Element::typed::<Modal>(ModalProps {
            title: Some("INNER".into()),
            closable: true,
            width: ModalSize::Fixed(8),
            height: ModalSize::Fixed(4),
            z_index: 2000,
            on_close: Some(Arc::new(move |_| inner.lock().unwrap().push("inner"))),
            ..props()
        });
        let tree = Element::typed::<Modal>(ModalProps {
            title: Some("OUTER".into()),
            content: Some(builder::div().class("w-full h-full").child(child).build()),
            closable: true,
            on_close: Some(Arc::new(move |_| outer.lock().unwrap().push("outer"))),
            ..props()
        });
        let frames = run(
            Control(tree),
            size,
            vec![
                (5, key(KeyCode::Escape)),
                (6, key(KeyCode::Escape)),
                (7, None),
            ],
        );
        assert!(frames.iter().any(|frame| frame.text.contains("INNER")));
        assert_eq!(*calls.lock().unwrap(), ["inner", "outer"]);
        assert!(!frames.last().unwrap().text.contains("OUTER"));
    }
}

#[test]
fn modal_closing_fade_is_inert_and_reopening_restores_child_focus() {
    for size in [(32, 16), (48, 24)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let action = calls.clone();
        let config = Arc::new(Mutex::new(ModalProps {
            visible: false,
            animation: ModalAnimation::Fade,
            backdrop_style: None,
            closable: true,
            content: Some(
                builder::button()
                    .text("ACTION")
                    .class("h-1 p-0")
                    .on_click(move || action.lock().unwrap().push("action".into()))
                    .build()
                    .auto_focus(),
            ),
            ..props()
        }));
        let weak = Arc::downgrade(&config);
        let closed = calls.clone();
        config.lock().unwrap().on_close = Some(Arc::new(move |_| {
            weak.upgrade().unwrap().lock().unwrap().visible = false;
            closed.lock().unwrap().push("closed".into());
        }));
        let frames = app_input::run_when_all(
            LaunchRoot {
                config,
                calls: calls.clone(),
            },
            size,
            vec![
                (&["C0"], key(KeyCode::Enter)),
                (&["ACTION"], key(KeyCode::Escape)),
                (
                    &["C2", "ACTION"],
                    click((size.0 - 12) / 2, (size.1 - 6) / 2 + 1),
                ),
                (&["C2"], key(KeyCode::Enter)),
                (&["C3", "ACTION"], key(KeyCode::Enter)),
                (&["C4"], None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            ["launch", "closed", "launch", "action"]
        );
        assert!(frames.last().unwrap().text.contains("ACTION"));
    }
}

#[test]
fn modal_invalid_dimensions_recover_on_prop_update_and_deliver_close() {
    for size in [(24, 12), (48, 20)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let closed = calls.clone();
        let config = Arc::new(Mutex::new(ModalProps {
            width: ModalSize::Percent(f32::INFINITY),
            closable: true,
            backdrop_style: None,
            on_close: Some(Arc::new(move |_| {
                closed.lock().unwrap().push("closed".into())
            })),
            ..props()
        }));
        let frames = app_input::run_when(
            LaunchRoot {
                config,
                calls: calls.clone(),
            },
            size,
            vec![
                ("Invalid Modal", key(KeyCode::Char('v'))),
                ("DETAILS", key(KeyCode::Escape)),
                ("C1", None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), ["closed"]);
        assert!(!frames.last().unwrap().text.contains("Invalid Modal"));
        assert!(!frames.last().unwrap().text.contains("DETAILS"));
    }
}
