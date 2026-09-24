use super::{key, run, Control};
use reactive_tui::{
    component::Element,
    event::types::KeyCode,
    widgets::layout::accordion::{AccordionBuilder, AccordionSection},
};

fn cell_of(text: &str, label: &str) -> (u16, u16) {
    use unicode_width::UnicodeWidthStr;
    text.lines()
        .enumerate()
        .find_map(|(y, line)| {
            line.find(label)
                .map(|x| (line[..x].width() as u16, y as u16))
        })
        .expect(label)
}

#[test]
fn accordion_builder_opens_real_content_and_skips_disabled_headers() {
    for size in [(32, 10), (60, 16)] {
        let control = AccordionBuilder::new()
            .animated(false)
            .section(AccordionSection::new("a", "First").content(Element::text("BODY-A")))
            .section(
                AccordionSection::new("b", "Locked")
                    .disabled(true)
                    .content(Element::text("FORBIDDEN")),
            )
            .section(AccordionSection::new("c", "Third").content(Element::text("BODY-C")))
            .build()
            .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Enter)),
                (2, key(KeyCode::Down)),
                (3, key(KeyCode::Enter)),
                (4, None),
            ],
        );
        assert!(
            frames[1].text.contains("BODY-A"),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(frames.last().unwrap().text.contains("BODY-C"));
        assert!(!frames.last().unwrap().text.contains("BODY-A"));
        assert!(!frames.iter().any(|f| f.text.contains("FORBIDDEN")));
    }
}

#[test]
fn accordion_clicks_measured_custom_headers_and_not_expanded_content() {
    use super::click;
    use reactive_tui::component::LayoutType;
    for size in [(36, 14), (64, 20)] {
        let make = || {
            Element::layout(LayoutType::Flex)
                .with_class("flex flex-col p-1")
                .with_child(
                    AccordionBuilder::new()
                        .animated(false)
                        .section(AccordionSection::new("a", "First").expanded(true).content(
                            Element::text("BODY-A\nline2\nline3").with_class("whitespace-pre"),
                        ))
                        .section(
                            AccordionSection::new("b", "Ignored title")
                                .custom_header(
                                    Element::text("界Custom\nheader2").with_class("whitespace-pre"),
                                )
                                .content(Element::text("BODY-B")),
                        )
                        .build(),
                )
        };
        let initial = run(Control(make()), size, vec![(2, None)]);
        let (x, y) = cell_of(&initial.last().unwrap().text, "header2");
        let (body_x, body_y) = cell_of(&initial.last().unwrap().text, "line2");
        let frames = run(
            Control(make()),
            size,
            vec![(2, click(body_x, body_y)), (2, click(x, y)), (3, None)],
        );
        assert!(
            frames.last().unwrap().text.contains("BODY-B"),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(!frames.last().unwrap().text.contains("BODY-A"));
    }
}

#[test]
fn accordion_modes_emit_one_notification_per_actual_change() {
    use reactive_tui::{
        app::RootComponent,
        event::{router::EventResult, Event},
        widgets::layout::accordion::{Accordion, AccordionMode, AccordionProps},
    };
    use std::sync::{Arc, Mutex};
    struct Root {
        control: Element,
        changes: Arc<Mutex<Vec<serde_json::Value>>>,
    }
    impl RootComponent for Root {
        fn render(&self) -> Element {
            self.control.clone()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if let Event::Custom(event) = event {
                assert_eq!(event.name, "changed");
                self.changes
                    .lock()
                    .unwrap()
                    .push(serde_json::from_slice(&event.data).unwrap());
                return EventResult::Consumed;
            }
            EventResult::Ignored
        }
    }
    for size in [(32, 10), (60, 16)] {
        for mode in [
            AccordionMode::Single,
            AccordionMode::Multiple,
            AccordionMode::AlwaysOne,
        ] {
            let changes = Arc::new(Mutex::new(Vec::new()));
            let control = Element::typed::<Accordion>(AccordionProps {
                sections: vec![
                    AccordionSection::new("a", "First")
                        .expanded(true)
                        .content(Element::text("BODY-A")),
                    AccordionSection::new("b", "Second").content(Element::text("BODY-B")),
                ],
                mode: mode.clone(),
                reduced_motion: true,
                on_change: Some("changed".into()),
                ..Default::default()
            })
            .auto_focus();
            let frames = run(
                Root {
                    control,
                    changes: changes.clone(),
                },
                size,
                vec![
                    (2, key(KeyCode::Enter)),
                    (3, key(KeyCode::Down)),
                    (4, key(KeyCode::Enter)),
                    (5, None),
                ],
            );
            let changes = changes.lock().unwrap();
            assert_eq!(
                changes.len(),
                if mode == AccordionMode::AlwaysOne {
                    1
                } else {
                    2
                }
            );
            assert_eq!(changes.last().unwrap()["section_id"], "b");
            assert_eq!(changes.last().unwrap()["expanded"], true);
            assert!(frames.last().unwrap().text.contains("BODY-B"));
            assert_eq!(
                frames.last().unwrap().text.contains("BODY-A"),
                mode == AccordionMode::AlwaysOne
            );
        }
    }
}

#[test]
fn accordion_empty_disabled_and_key_release_do_not_toggle() {
    use reactive_tui::{
        event::{types::KeyEventKind, Event, KeyEvent},
        widgets::layout::accordion::{Accordion, AccordionProps},
    };
    for size in [(32, 10), (60, 16)] {
        for sections in [
            vec![],
            vec![AccordionSection::new("a", "Locked")
                .disabled(true)
                .content(Element::text("FORBIDDEN"))],
        ] {
            let frames = run(
                Control(
                    Element::typed::<Accordion>(AccordionProps {
                        sections,
                        reduced_motion: true,
                        ..Default::default()
                    })
                    .auto_focus(),
                ),
                size,
                vec![
                    (1, key(KeyCode::Down)),
                    (1, key(KeyCode::Home)),
                    (1, key(KeyCode::Enter)),
                    (1, None),
                ],
            );
            assert!(!frames.iter().any(|f| f.text.contains("FORBIDDEN")));
        }
        let mut release = KeyEvent::new(KeyCode::Enter);
        release.kind = KeyEventKind::Release;
        let frames = run(
            Control(
                AccordionBuilder::new()
                    .animated(false)
                    .section(
                        AccordionSection::new("a", "First").content(Element::text("FORBIDDEN")),
                    )
                    .build()
                    .auto_focus(),
            ),
            size,
            vec![(2, Some(Event::Key(release))), (2, None)],
        );
        assert!(!frames.iter().any(|f| f.text.contains("FORBIDDEN")));
    }
}

#[test]
fn accordion_keeps_nested_input_state_across_collapse_and_resize() {
    use super::click;
    use reactive_tui::{
        builder,
        event::{Event, ResizeEvent},
    };
    for size in [(36, 12), (64, 18)] {
        let make = || {
            AccordionBuilder::new()
                .animated(false)
                .section(
                    AccordionSection::new("a", "Header")
                        .expanded(true)
                        .content(builder::text_input().value("seed").build()),
                )
                .build()
        };
        let initial = run(Control(make()), size, vec![(2, None)]);
        let (x, y) = cell_of(&initial.last().unwrap().text, "seed");
        let (hx, hy) = cell_of(&initial.last().unwrap().text, "Header");
        let frames = run(
            Control(make()),
            size,
            vec![
                (2, click(x, y)),
                (3, key(KeyCode::End)),
                (3, key(KeyCode::Char('X'))),
                (4, click(hx, hy)),
                (5, click(hx, hy)),
                (
                    6,
                    Some(Event::Resize(ResizeEvent::new(size.0 + 10, size.1 + 2))),
                ),
                (8, None),
            ],
        );
        assert!(
            frames.last().unwrap().text.contains("seedX"),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(frames.iter().skip(3).any(|f| !f.text.contains("seed")));
    }
}

#[test]
fn accordion_animation_paints_partial_content_and_reduced_motion_is_immediate() {
    use reactive_tui::widgets::layout::accordion::{Accordion, AccordionAnimation, AccordionProps};
    use std::time::Duration;
    for size in [(36, 14), (64, 20)] {
        for reduced in [false, true] {
            let body = (0..8)
                .map(|i| format!("row{i}"))
                .collect::<Vec<_>>()
                .join("\n");
            let control = Element::typed::<Accordion>(AccordionProps {
                sections: vec![AccordionSection::new("a", "Header")
                    .content(Element::text(body).with_class("whitespace-pre"))],
                reduced_motion: reduced,
                animation: AccordionAnimation {
                    duration: Duration::from_millis(400),
                    ..Default::default()
                },
                ..Default::default()
            })
            .auto_focus();
            let frames = run(
                Control(control),
                size,
                vec![
                    (2, key(KeyCode::Enter)),
                    (if reduced { 3 } else { 8 }, None),
                ],
            );
            if reduced {
                assert!(frames.last().unwrap().text.contains("row7"));
            } else {
                assert!(
                    frames
                        .iter()
                        .any(|f| f.text.contains("row0") && !f.text.contains("row7")),
                    "{:?}",
                    frames.iter().map(|f| &f.text).collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
fn accordion_closing_body_leaves_tab_order_before_animation_finishes() {
    use reactive_tui::{
        builder,
        component::LayoutType,
        widgets::layout::accordion::{Accordion, AccordionAnimation, AccordionProps},
    };
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };
    for size in [(36, 14), (64, 20)] {
        let inside = Arc::new(AtomicUsize::new(0));
        let outside = Arc::new(AtomicUsize::new(0));
        let inner = inside.clone();
        let outer = outside.clone();
        let element = Element::layout(LayoutType::Flex)
            .with_class("flex flex-col")
            .with_child(
                Element::typed::<Accordion>(AccordionProps {
                    sections: vec![AccordionSection::new("a", "Header").expanded(true).content(
                        Element::layout(LayoutType::Flex)
                            .with_class("flex flex-col")
                            .with_children(vec![
                                builder::button()
                                    .text("Inside")
                                    .class("p-0 h-1 w-full")
                                    .on_click(move || {
                                        inner.fetch_add(1, Ordering::SeqCst);
                                    })
                                    .build(),
                                Element::text("row1\nrow2\nrow3\nrow4")
                                    .with_class("whitespace-pre"),
                            ]),
                    )],
                    animation: AccordionAnimation {
                        duration: Duration::from_secs(2),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .auto_focus(),
            )
            .with_child(
                builder::button()
                    .text("Outside")
                    .class("p-0 h-1 w-full")
                    .on_click(move || {
                        outer.fetch_add(1, Ordering::SeqCst);
                    })
                    .build(),
            );
        let frames = run(
            Control(element),
            size,
            vec![
                (2, key(KeyCode::Enter)),
                (3, super::click(2, 1)),
                (4, key(KeyCode::Tab)),
                (5, key(KeyCode::Enter)),
                (6, None),
            ],
        );
        assert!(
            frames[3].text.contains("Inside"),
            "test must exercise a still-painted closing body: {:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert_eq!(inside.load(Ordering::SeqCst), 0);
        assert_eq!(outside.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn accordion_persistence_and_mode_changes_follow_supplied_props() {
    use reactive_tui::{
        app::RootComponent,
        error::Result,
        event::{router::EventResult, CustomEvent, Event},
        widgets::layout::accordion::{Accordion, AccordionMode, AccordionProps},
    };
    struct Root(AccordionProps);
    impl RootComponent for Root {
        fn render(&self) -> Element {
            Element::typed::<Accordion>(self.0.clone()).auto_focus()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn try_handle_event(&mut self, event: &Event) -> Result<EventResult> {
            if let Event::Custom(event) = event {
                if event.name == "props" {
                    self.0.mode = AccordionMode::Single;
                    self.0.sections.reverse();
                    return Ok(EventResult::Consumed);
                }
            }
            Ok(EventResult::Ignored)
        }
    }
    for size in [(36, 12), (64, 18)] {
        for persist in [false, true] {
            let root = Root(AccordionProps {
                sections: vec![
                    AccordionSection::new("a", "First").content(Element::text("BODY-A")),
                    AccordionSection::new("b", "Second").content(Element::text("BODY-B")),
                ],
                mode: AccordionMode::Multiple,
                persist_state: persist,
                reduced_motion: true,
                ..Default::default()
            });
            let frames = run(
                root,
                size,
                vec![
                    (2, key(KeyCode::Enter)),
                    (3, Some(Event::Custom(CustomEvent::new("props", vec![])))),
                    (4, None),
                ],
            );
            assert_eq!(
                frames.last().unwrap().text.contains("BODY-A"),
                persist,
                "{:?}",
                frames.iter().map(|f| &f.text).collect::<Vec<_>>()
            );
            assert!(!frames.last().unwrap().text.contains("BODY-B"));
        }
    }
}
