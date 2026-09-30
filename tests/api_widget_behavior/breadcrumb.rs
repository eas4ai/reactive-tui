use super::{key, run, Control};
use reactive_tui::{
    app::RootComponent,
    component::{Element, LayoutType},
    event::{router::EventResult, types::KeyCode, Event},
    widgets::layout::breadcrumb::{Breadcrumb, BreadcrumbProps, BreadcrumbSegment},
};
use std::sync::{Arc, Mutex};

struct Navigation {
    element: Element,
    events: Arc<Mutex<Vec<serde_json::Value>>>,
}

impl RootComponent for Navigation {
    fn render(&self) -> Element {
        self.element.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        if let Event::Custom(event) = event {
            assert_eq!(event.name, "navigate");
            self.events
                .lock()
                .unwrap()
                .push(serde_json::from_slice(&event.data).unwrap());
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }
}

fn segments() -> Vec<BreadcrumbSegment> {
    vec![
        BreadcrumbSegment::new("root", "Root", "/"),
        BreadcrumbSegment::new("locked", "Locked", "/locked").clickable(false),
        BreadcrumbSegment::new("docs", "界Docs", "/docs"),
        BreadcrumbSegment::new("current", "Current", "/docs/current").current(true),
    ]
}

#[test]
fn breadcrumb_replacing_focused_accordion_receives_keyboard_focus() {
    use reactive_tui::widgets::layout::accordion::{Accordion, AccordionProps, AccordionSection};
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Replacement {
        replaced: AtomicBool,
        events: Arc<Mutex<Vec<serde_json::Value>>>,
    }
    impl RootComponent for Replacement {
        fn render(&self) -> Element {
            if self.replaced.load(Ordering::SeqCst) {
                Element::typed::<Breadcrumb>(BreadcrumbProps {
                    segments: segments(),
                    show_icons: false,
                    compact: true,
                    on_click: Some("navigate".into()),
                    ..Default::default()
                })
                .auto_focus()
            } else {
                Element::typed::<Accordion>(AccordionProps {
                    sections: vec![AccordionSection::new("before", "Before")],
                    reduced_motion: true,
                    ..Default::default()
                })
                .auto_focus()
            }
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            match event {
                Event::Key(key) if key.code == KeyCode::F(8) => {
                    self.replaced.store(true, Ordering::SeqCst);
                    EventResult::Consumed
                }
                Event::Custom(event) if event.name == "navigate" => {
                    self.events
                        .lock()
                        .unwrap()
                        .push(serde_json::from_slice(&event.data).unwrap());
                    EventResult::Consumed
                }
                _ => EventResult::Ignored,
            }
        }
    }
    for size in [(40, 6), (64, 10)] {
        let events = Arc::new(Mutex::new(Vec::new()));
        run(
            Replacement {
                replaced: AtomicBool::new(false),
                events: events.clone(),
            },
            size,
            vec![
                (2, key(KeyCode::F(8))),
                (4, super::click(1, 0)),
                (5, key(KeyCode::Enter)),
                (6, key(KeyCode::Right)),
                (7, key(KeyCode::Enter)),
                (8, None),
            ],
        );
        let events = events.lock().unwrap();
        assert_eq!(
            events
                .iter()
                .map(|event| event["segment_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["root", "root", "docs"]
        );
    }
}

#[test]
fn named_breadcrumb_builder_paints_real_segments() {
    for size in [(40, 6), (64, 10)] {
        let element = reactive_tui::builder::breadcrumb()
            .show_icons(false)
            .compact(true)
            .separator(" > ")
            .segment(BreadcrumbSegment::new("root", "Root", "/"))
            .segment(BreadcrumbSegment::new("docs", "Docs", "/docs"))
            .segment(BreadcrumbSegment::new("current", "Current", "/docs/current").current(true))
            .build();
        let frames = run(Control(element), size, vec![(2, None)]);
        let text = &frames.last().unwrap().text;
        assert!(text.contains("Root > Docs > Current"), "{text}");
    }
}

#[test]
fn breadcrumb_keyboard_callbacks_skip_nonclickable_and_current_segments() {
    for size in [(40, 6), (64, 10)] {
        let events = Arc::new(Mutex::new(Vec::new()));
        let element = Element::typed::<Breadcrumb>(BreadcrumbProps {
            segments: segments(),
            show_icons: false,
            compact: true,
            on_click: Some("navigate".into()),
            ..Default::default()
        })
        .auto_focus();
        run(
            Navigation {
                element,
                events: events.clone(),
            },
            size,
            vec![
                (1, key(KeyCode::Home)),
                (1, key(KeyCode::Enter)),
                (2, key(KeyCode::Right)),
                (2, key(KeyCode::Enter)),
                (3, key(KeyCode::End)),
                (3, key(KeyCode::Char(' '))),
                (4, None),
            ],
        );
        let events = events.lock().unwrap();
        assert_eq!(
            events
                .iter()
                .map(|event| event["segment_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["root", "docs", "docs"]
        );
        assert_eq!(events[1]["path"], "/docs");
    }
}

#[test]
fn breadcrumb_clicks_use_presented_unicode_bounds_inside_padding() {
    use super::click;
    use unicode_width::UnicodeWidthStr;
    for size in [(44, 20), (68, 24)] {
        let make = || {
            Element::layout(LayoutType::Flex)
                .with_class("flex flex-col p-8")
                .with_child(Element::typed::<Breadcrumb>(BreadcrumbProps {
                    segments: segments(),
                    show_icons: false,
                    compact: true,
                    on_click: Some("navigate".into()),
                    ..Default::default()
                }))
        };
        let initial = run(Control(make()), size, vec![(2, None)]);
        let (x, y) = initial
            .last()
            .unwrap()
            .text
            .lines()
            .enumerate()
            .find_map(|(y, line)| {
                line.find("界Docs")
                    .map(|x| (line[..x].width() as u16, y as u16))
            })
            .unwrap_or_else(|| {
                panic!(
                    "painted Docs segment: {:?}",
                    initial.iter().map(|frame| &frame.text).collect::<Vec<_>>()
                )
            });
        let events = Arc::new(Mutex::new(Vec::new()));
        run(
            Navigation {
                element: make(),
                events: events.clone(),
            },
            size,
            vec![(2, click(x + 2, y)), (3, None)],
        );
        let events = events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["segment_id"], "docs");
    }
}

#[test]
fn breadcrumb_overflow_counts_separators_and_excludes_hidden_keyboard_targets() {
    use reactive_tui::widgets::layout::breadcrumb::OverflowStrategy;
    for size in [(32, 8), (60, 12)] {
        for (strategy, text, expected) in [
            (
                OverflowStrategy::MiddleEllipsis,
                "Root/.../Current",
                ["root", "root"],
            ),
            (
                OverflowStrategy::TruncateStart,
                "Locked/界Docs/Current",
                ["docs", "docs"],
            ),
            (
                OverflowStrategy::TruncateEnd,
                "Root/Locked/界Docs",
                ["root", "docs"],
            ),
        ] {
            let element = Element::typed::<Breadcrumb>(BreadcrumbProps {
                segments: segments(),
                max_width: Some(22),
                overflow_strategy: strategy,
                show_icons: false,
                compact: true,
                on_click: Some("navigate".into()),
                ..Default::default()
            })
            .auto_focus();
            let events = Arc::new(Mutex::new(Vec::new()));
            let frames = run(
                Navigation {
                    element,
                    events: events.clone(),
                },
                size,
                vec![
                    (2, key(KeyCode::Home)),
                    (2, key(KeyCode::Enter)),
                    (3, key(KeyCode::End)),
                    (3, key(KeyCode::Enter)),
                    (4, None),
                ],
            );
            assert!(
                frames.last().unwrap().text.contains(text),
                "{:?}",
                frames.iter().map(|f| &f.text).collect::<Vec<_>>()
            );
            assert_eq!(
                events
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|event| event["segment_id"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }
}

#[test]
fn breadcrumb_wrap_has_measured_targets_on_later_rows() {
    use super::click;
    use reactive_tui::widgets::layout::breadcrumb::OverflowStrategy;
    use unicode_width::UnicodeWidthStr;
    for size in [(24, 8), (48, 12)] {
        let make = || {
            Element::typed::<Breadcrumb>(BreadcrumbProps {
                segments: segments(),
                max_width: Some(12),
                overflow_strategy: OverflowStrategy::Wrap,
                show_icons: false,
                compact: true,
                on_click: Some("navigate".into()),
                ..Default::default()
            })
        };
        let initial = run(Control(make()), size, vec![(2, None)]);
        let (x, y) = initial
            .last()
            .unwrap()
            .text
            .lines()
            .enumerate()
            .find_map(|(y, line)| {
                line.find("界Docs")
                    .map(|x| (line[..x].width() as u16, y as u16))
            })
            .unwrap_or_else(|| panic!("{:?}", initial.iter().map(|f| &f.text).collect::<Vec<_>>()));
        assert!(y > 0, "wrapped segment must use a later row");
        let events = Arc::new(Mutex::new(Vec::new()));
        run(
            Navigation {
                element: make(),
                events: events.clone(),
            },
            size,
            vec![(2, click(x + 2, y)), (3, None)],
        );
        assert_eq!(events.lock().unwrap()[0]["segment_id"], "docs");
    }
}

#[test]
fn breadcrumb_scroll_reveals_keyboard_focus_and_returns_home() {
    use reactive_tui::widgets::layout::breadcrumb::OverflowStrategy;
    for size in [(24, 8), (48, 12)] {
        let element = Element::typed::<Breadcrumb>(BreadcrumbProps {
            segments: segments(),
            max_width: Some(12),
            overflow_strategy: OverflowStrategy::Scroll,
            show_icons: false,
            compact: true,
            on_click: Some("navigate".into()),
            ..Default::default()
        })
        .auto_focus();
        let events = Arc::new(Mutex::new(Vec::new()));
        let frames = run(
            Navigation {
                element,
                events: events.clone(),
            },
            size,
            vec![
                (2, key(KeyCode::End)),
                (3, key(KeyCode::Enter)),
                (3, key(KeyCode::Home)),
                (4, key(KeyCode::Enter)),
                (5, None),
            ],
        );
        assert!(
            frames[2].text.contains("界Docs"),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(frames.last().unwrap().text.contains("Root/Locked/"));
        assert_eq!(
            events
                .lock()
                .unwrap()
                .iter()
                .map(|event| event["segment_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["docs", "root"]
        );
    }
}

#[test]
fn breadcrumb_tooltips_follow_hover_and_can_be_disabled() {
    use reactive_tui::event::types::{MouseEvent, MouseEventKind, Position};
    for size in [(32, 8), (60, 12)] {
        for show in [true, false] {
            let element = reactive_tui::builder::breadcrumb()
                .show_icons(false)
                .compact(true)
                .show_tooltips(show)
                .segment(
                    BreadcrumbSegment::new("root", "Root", "/").tooltip("Navigate to the root"),
                )
                .build();
            let frames = run(
                Control(element),
                size,
                vec![
                    (
                        2,
                        Some(Event::Mouse(MouseEvent::new(
                            MouseEventKind::Move,
                            Position::cell(1, 0),
                        ))),
                    ),
                    (
                        3,
                        Some(Event::Mouse(MouseEvent::new(
                            MouseEventKind::Move,
                            Position::cell(size.0 - 1, size.1 - 1),
                        ))),
                    ),
                    (4, None),
                ],
            );
            assert_eq!(frames[2].text.contains("Navigate to the root"), show);
            assert!(!frames.last().unwrap().text.contains("Navigate to the root"));
        }
    }
}

#[test]
fn breadcrumb_icons_home_override_compact_spacing_and_segment_classes_reach_frames() {
    for size in [(40, 8), (68, 12)] {
        for (icons, home, expected) in [
            (false, false, "ROOT/Docs"),
            (false, true, "ROOT/Docs"),
            (true, false, "R ROOT/D Docs"),
            (true, true, "H ROOT/D Docs"),
        ] {
            for compact in [true, false] {
                let frames = run(
                    Control(Element::typed::<Breadcrumb>(BreadcrumbProps {
                        segments: vec![
                            BreadcrumbSegment::new("root", "Root", "/")
                                .icon("R")
                                .class("uppercase"),
                            BreadcrumbSegment::new("docs", "Docs", "/docs")
                                .icon("D")
                                .current(true),
                        ],
                        show_icons: icons,
                        show_home_icon: home,
                        home_icon: "H".into(),
                        compact,
                        ..Default::default()
                    })),
                    size,
                    vec![(2, None)],
                );
                let text = &frames.last().unwrap().text;
                if compact {
                    assert!(text.contains(expected), "{text:?}");
                } else {
                    assert!(text.contains(" / "), "{text:?}");
                    assert_eq!(
                        text.split_whitespace().collect::<String>(),
                        expected.replace(' ', "")
                    );
                }
            }
        }
    }
}

#[test]
fn breadcrumb_current_and_nonclickable_segments_ignore_mouse_activation() {
    for size in [(40, 8), (68, 12)] {
        let events = Arc::new(Mutex::new(Vec::new()));
        run(
            Navigation {
                element: Element::typed::<Breadcrumb>(BreadcrumbProps {
                    segments: segments(),
                    compact: true,
                    show_icons: false,
                    on_click: Some("navigate".into()),
                    ..Default::default()
                }),
                events: events.clone(),
            },
            size,
            vec![(2, super::click(6, 0)), (3, super::click(20, 0)), (4, None)],
        );
        assert!(events.lock().unwrap().is_empty());
    }
}

#[test]
fn breadcrumb_wheel_scroll_clamps_and_updates_mouse_targets() {
    use reactive_tui::{
        event::types::{MouseEvent, Position, WheelDelta, WheelEvent, WheelPhase},
        widgets::layout::breadcrumb::OverflowStrategy,
    };
    let wheel = |delta| {
        Some(Event::Mouse(MouseEvent::wheel(
            Position::cell(1, 0),
            WheelEvent {
                delta: WheelDelta::Lines { x: delta, y: 0.0 },
                phase: WheelPhase::Changed,
            },
        )))
    };
    for size in [(32, 8), (60, 12)] {
        let events = Arc::new(Mutex::new(Vec::new()));
        let frames = run(
            Navigation {
                element: Element::typed::<Breadcrumb>(BreadcrumbProps {
                    segments: segments(),
                    compact: true,
                    show_icons: false,
                    max_width: Some(12),
                    overflow_strategy: OverflowStrategy::Scroll,
                    on_click: Some("navigate".into()),
                    ..Default::default()
                }),
                events: events.clone(),
            },
            size,
            vec![
                (2, wheel(1.0)),
                (3, wheel(0.0)),
                (4, wheel(-1.0)),
                (5, wheel(1000.0)),
                (6, super::click(1, 0)),
                (7, wheel(-1000.0)),
                (8, super::click(1, 0)),
                (9, None),
            ],
        );
        assert!(frames[2].text.contains("Locked"));
        assert!(!frames[2].text.contains("Root"));
        assert_eq!(frames[2].text, frames[3].text);
        assert!(frames[4].text.contains("Root/Locked/"));
        assert!(frames[5].text.contains("Current"));
        assert!(frames.last().unwrap().text.contains("Root/Locked/"));
        assert_eq!(
            events
                .lock()
                .unwrap()
                .iter()
                .map(|event| event["segment_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["docs", "root"]
        );
    }
}

#[test]
fn breadcrumb_resize_recomputes_overflow_and_retains_the_focused_segment() {
    use reactive_tui::event::types::ResizeEvent;
    for size in [(32, 8), (60, 12)] {
        let events = Arc::new(Mutex::new(Vec::new()));
        let element = Element::typed::<Breadcrumb>(BreadcrumbProps {
            segments: segments(),
            show_icons: false,
            compact: true,
            on_click: Some("navigate".into()),
            ..Default::default()
        })
        .auto_focus();
        let frames = run(
            Navigation {
                element,
                events: events.clone(),
            },
            size,
            vec![
                (2, key(KeyCode::Home)),
                (3, Some(Event::Resize(ResizeEvent::new(22, size.1)))),
                (4, key(KeyCode::Enter)),
                (5, None),
            ],
        );
        assert!(
            frames.last().unwrap().text.contains("Root/.../Current"),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert_eq!(events.lock().unwrap()[0]["segment_id"], "root");
    }
}

#[test]
fn breadcrumb_props_reorder_keeps_user_focus_and_remeasures_labels() {
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Reordered {
        changed: AtomicBool,
        events: Arc<Mutex<Vec<serde_json::Value>>>,
    }
    impl RootComponent for Reordered {
        fn render(&self) -> Element {
            let mut items = segments();
            if self.changed.load(Ordering::SeqCst) {
                items.swap(0, 2);
                items[2].label = "Changed root".into();
            }
            Element::typed::<Breadcrumb>(BreadcrumbProps {
                segments: items,
                show_icons: false,
                compact: true,
                on_click: Some("navigate".into()),
                ..Default::default()
            })
            .auto_focus()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            match event {
                Event::Key(key) if key.code == KeyCode::F(2) => {
                    self.changed.store(true, Ordering::SeqCst);
                }
                Event::Custom(event) => self
                    .events
                    .lock()
                    .unwrap()
                    .push(serde_json::from_slice(&event.data).unwrap()),
                _ => return EventResult::Ignored,
            }
            EventResult::Consumed
        }
    }
    for size in [(44, 8), (68, 12)] {
        let events = Arc::new(Mutex::new(Vec::new()));
        let frames = run(
            Reordered {
                changed: AtomicBool::new(false),
                events: events.clone(),
            },
            size,
            vec![
                (2, key(KeyCode::Home)),
                (3, key(KeyCode::F(2))),
                (4, key(KeyCode::Enter)),
                (5, None),
            ],
        );
        assert!(frames
            .last()
            .unwrap()
            .text
            .contains("界Docs/Locked/Changed root/Current"));
        assert_eq!(events.lock().unwrap()[0]["segment_id"], "root");
        assert_eq!(events.lock().unwrap()[0]["label"], "Changed root");
    }
}

#[test]
fn breadcrumb_empty_keyboard_disabled_and_release_inputs_do_not_activate() {
    use reactive_tui::event::types::{KeyEvent, KeyEventKind};
    for size in [(32, 8), (60, 12)] {
        for items in [Vec::new(), segments()] {
            let events = Arc::new(Mutex::new(Vec::new()));
            let element = Element::typed::<Breadcrumb>(BreadcrumbProps {
                segments: items,
                keyboard_navigation: false,
                show_icons: false,
                on_click: Some("navigate".into()),
                ..Default::default()
            })
            .auto_focus();
            run(
                Navigation {
                    element,
                    events: events.clone(),
                },
                size,
                vec![(1, key(KeyCode::Home)), (1, key(KeyCode::Enter)), (1, None)],
            );
            assert!(events.lock().unwrap().is_empty());
        }
        let events = Arc::new(Mutex::new(Vec::new()));
        let element = Element::typed::<Breadcrumb>(BreadcrumbProps {
            segments: segments(),
            show_icons: false,
            compact: true,
            on_click: Some("navigate".into()),
            ..Default::default()
        })
        .auto_focus();
        run(
            Navigation {
                element,
                events: events.clone(),
            },
            size,
            vec![
                (
                    2,
                    Some(Event::Key(
                        KeyEvent::new(KeyCode::Enter).with_kind(KeyEventKind::Release),
                    )),
                ),
                (2, None),
            ],
        );
        assert!(events.lock().unwrap().is_empty());
    }
}

#[test]
fn breadcrumb_nav_002_one_cell_padding() {
    for compact in [false, true] {
        let frames = super::app_input::run_until(
            Control(
                reactive_tui::builder::breadcrumb()
                    .show_icons(false)
                    .compact(compact)
                    .segment(BreadcrumbSegment::new("root", "Root", "/").current(true))
                    .build(),
            ),
            (50, 6),
            vec![super::app_input::Until {
                text: "Root",
                cell: None,
                event: None,
            }],
            std::time::Duration::from_secs(5),
        );
        let screen = &frames.last().unwrap().screen;
        assert_eq!(
            screen.cell(0, u16::from(!compact)).unwrap().contents(),
            "R",
            "the label must start after exactly one padding cell (none when compact)"
        );
    }
}

#[test]
fn breadcrumb_nav_003_catalog_path_fits_fifty_cells() {
    let frames = super::app_input::run_until(
        Control(
            Element::layout(LayoutType::Flex)
                .with_class("w-50 flex flex-col")
                .with_child(reactive_tui::builder::path_breadcrumb(
                    "/catalog/layout/widgets",
                )),
        ),
        (80, 6),
        vec![super::app_input::Until {
            text: "Root",
            cell: None,
            event: None,
        }],
        std::time::Duration::from_secs(5),
    );
    let text = &frames.last().unwrap().text;
    for label in ["🏠 Root", "📁 catalog", "📁 layout", "📁 widgets"] {
        assert!(
            text.contains(label),
            "missing whole segment {label:?}: {text:?}"
        );
    }
    assert_eq!(
        frames
            .last()
            .unwrap()
            .geometry
            .iter()
            .find(|node| node.element_index == 1)
            .unwrap()
            .bounds
            .width,
        50.0,
        "the trail fills the parent width"
    );
    assert!(
        !text.contains("..."),
        "a fitting path must not collapse: {text:?}"
    );
}

#[test]
fn breadcrumb_nav_003_middle_ellipsis_keeps_endpoints_whole() {
    let labels = [
        "FirstSegment",
        "SecondLabel!",
        "MiddleLabel!",
        "FourthLabel!",
        "LastSegment!",
    ];
    let frames = super::app_input::run_until(
        Control(
            Element::layout(LayoutType::Flex)
                .with_class("w-40 flex flex-col")
                .with_child(Element::typed::<Breadcrumb>(BreadcrumbProps {
                    segments: labels
                        .iter()
                        .enumerate()
                        .map(|(index, label)| {
                            BreadcrumbSegment::new(index.to_string(), *label, "/")
                                .current(index == 4)
                        })
                        .collect(),
                    show_icons: false,
                    ..Default::default()
                })),
        ),
        (80, 6),
        vec![super::app_input::Until {
            text: "First",
            cell: None,
            event: None,
        }],
        std::time::Duration::from_secs(5),
    );
    let text = &frames.last().unwrap().text;
    assert!(
        text.contains(labels[0]),
        "first segment must remain whole: {text:?}"
    );
    assert!(
        text.contains(labels[4]),
        "last segment must remain whole: {text:?}"
    );
    assert!(
        text.contains("..."),
        "middle segments must collapse: {text:?}"
    );
}

struct BreadcrumbTheme(Arc<reactive_tui::theme::Theme>);
impl Drop for BreadcrumbTheme {
    fn drop(&mut self) {
        reactive_tui::theme::Theme::set_active((*self.0).clone());
    }
}

fn color(index: usize) -> vt100::Color {
    vt100::Color::Rgb(
        30 + 8 * index as u8,
        200 - 6 * index as u8,
        90 + 5 * index as u8,
    )
}

/// Distinct role colors let rendered cells identify the role they use.
fn breadcrumb_probe() -> BreadcrumbTheme {
    use reactive_tui::theme::{Theme, ThemeVariables};
    let roles = [
        "background",
        "surface",
        "foreground",
        "text-muted",
        "hover",
        "selection",
        "selection-foreground",
        "border",
        "input",
        "ring",
        "primary",
        "primary-foreground",
        "secondary",
        "secondary-foreground",
        "accent",
        "accent-foreground",
        "success",
        "success-foreground",
        "warning",
        "warning-foreground",
        "error",
        "error-foreground",
        "info",
        "info-foreground",
        "overlay",
        "shadow",
    ];

    let restore = BreadcrumbTheme(Theme::active());
    let mut variables = ThemeVariables::new();
    for (index, role) in roles.iter().enumerate() {
        let vt100::Color::Rgb(r, g, b) = color(index) else {
            unreachable!()
        };
        variables = variables.set(
            Theme::color_variable(role),
            format!("#{r:02x}{g:02x}{b:02x}"),
        );
    }
    Theme::set_active(Theme::new("breadcrumb-probe").with_variables(variables));
    restore
}

#[test]
#[serial_test::serial]
fn breadcrumb_nav_001_theme_colors_and_focus() {
    let _theme = breadcrumb_probe();
    let element = reactive_tui::builder::breadcrumb()
        .show_icons(false)
        .compact(true)
        .segment(BreadcrumbSegment::new("root", "Root", "/"))
        .segment(BreadcrumbSegment::new("docs", "Docs", "/docs").tooltip("Go docs"))
        .segment(BreadcrumbSegment::new("current", "Current", "/docs/current").current(true))
        .build();
    // On a page in the theme's background: the trail takes its parent's
    // fill and paints none of its own.
    let page = Element::layout(LayoutType::Flex)
        .with_class("flex flex-col w-full h-full bg-background text-foreground")
        .with_child(element.clone().auto_focus());
    let frames = super::app_input::run_until(
        Control(page),
        (50, 6),
        vec![
            super::app_input::Until {
                text: "Root/Docs/Current",
                cell: None,
                event: key(KeyCode::Home),
            },
            super::app_input::Until {
                text: "Root/Docs/Current",
                cell: None,
                event: None,
            },
        ],
        std::time::Duration::from_secs(5),
    );
    let screen = &frames.last().unwrap().screen;
    assert_eq!(
        screen.cell(0, 10).unwrap().fgcolor(),
        color(2),
        "current segment uses foreground"
    );
    assert_eq!(
        screen.cell(0, 5).unwrap().fgcolor(),
        color(3),
        "earlier clickable segment uses text-muted"
    );
    assert_eq!(
        screen.cell(0, 0).unwrap().fgcolor(),
        color(6),
        "focused segment uses selection-foreground"
    );
    assert_eq!(
        screen.cell(0, 0).unwrap().bgcolor(),
        color(5),
        "focused segment uses selection"
    );
    for column in 0..17 {
        let cell = screen.cell(0, column).unwrap();
        for painted in [cell.fgcolor(), cell.bgcolor()] {
            assert!(
                painted == vt100::Color::Default || (0..26).any(|index| color(index) == painted),
                "painted color outside the theme: {painted:?}"
            );
        }
    }
    assert!(frames.last().unwrap().text.contains("Root/Docs/Current"));
    // Moving focus to another widget leaves the trail in its ordinary colors.
    let config = element
        .props
        .downcast_ref::<BreadcrumbProps>()
        .unwrap()
        .clone();
    let mut ordinary = Vec::new();
    for trail in [element, Element::typed::<Breadcrumb>(config)] {
        let root = Element::layout(LayoutType::Flex)
            .with_class("flex flex-col w-full h-full bg-background text-foreground")
            .with_child(trail)
            .with_child(
                reactive_tui::builder::button()
                    .text("Elsewhere")
                    .build()
                    .auto_focus(),
            );
        let frames = super::app_input::run_until(
            Control(root),
            (50, 6),
            vec![super::app_input::Until {
                text: "Elsewhere",
                cell: None,
                event: None,
            }],
            std::time::Duration::from_secs(5),
        );
        let screen = &frames.last().unwrap().screen;
        assert_eq!(screen.cell(0, 0).unwrap().fgcolor(), color(3));
        assert_eq!(screen.cell(0, 0).unwrap().bgcolor(), color(0));
        assert_eq!(
            screen.cell(0, 4).unwrap().fgcolor(),
            color(3),
            "separator uses text-muted"
        );
        ordinary.push(
            (0..17)
                .map(|column| {
                    let cell = screen.cell(0, column).unwrap();
                    (cell.contents().to_owned(), cell.fgcolor(), cell.bgcolor())
                })
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(
        ordinary[0], ordinary[1],
        "props and builder paint the same trail"
    );
}

#[test]
#[serial_test::serial]
fn breadcrumb_nav_001_hover_disabled_icons_and_tooltip_colors() {
    let _theme = breadcrumb_probe();
    use reactive_tui::event::types::{MouseEvent, MouseEventKind, Position};
    let trail = reactive_tui::builder::breadcrumb()
        .compact(true)
        .home_icon("H")
        .segment(BreadcrumbSegment::new("root", "Root", "/"))
        .segment(
            BreadcrumbSegment::new("locked", "Locked", "/locked")
                .clickable(false)
                .tooltip("Unavailable"),
        )
        .build();
    let root = Element::layout(LayoutType::Flex)
        .with_class("flex flex-col")
        .with_child(trail)
        .with_child(
            reactive_tui::builder::button()
                .text("Elsewhere")
                .build()
                .auto_focus(),
        );
    let frames = super::app_input::run_until(
        Control(root),
        (50, 6),
        vec![
            super::app_input::Until {
                text: "H Root/Locked",
                cell: None,
                event: Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Move,
                    Position::cell(7, 0),
                ))),
            },
            super::app_input::Until {
                text: "Unavailable",
                cell: None,
                event: None,
            },
        ],
        std::time::Duration::from_secs(5),
    );
    let screen = &frames.last().unwrap().screen;
    assert_eq!(
        screen.cell(0, 0).unwrap().fgcolor(),
        color(3),
        "icon inherits its segment color"
    );
    assert_eq!(
        screen.cell(0, 7).unwrap().fgcolor(),
        color(3),
        "disabled segment uses text-muted"
    );
    assert_eq!(
        screen.cell(0, 7).unwrap().bgcolor(),
        color(4),
        "hover uses hover fill"
    );
    assert_eq!(
        screen.cell(1, 0).unwrap().fgcolor(),
        color(2),
        "tooltip uses foreground"
    );
    assert_eq!(
        screen.cell(1, 0).unwrap().bgcolor(),
        color(1),
        "tooltip uses surface"
    );
    assert!(
        frames.last().unwrap().text.contains("H Root/Locked"),
        "hover adds no glyph or label decoration"
    );
}
