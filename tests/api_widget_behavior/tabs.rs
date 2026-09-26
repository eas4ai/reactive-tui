use super::{click, key, run, Control};
use reactive_tui::{
    builder,
    component::Element,
    event::types::KeyCode,
    widgets::layout::{Tab, TabKeyboardActivation, TabsBuilder},
};

#[test]
fn tabs_reconstructed_children_preserve_choice_edits_and_local_close() {
    struct Fresh;
    impl reactive_tui::app::RootComponent for Fresh {
        fn render(&self) -> Element {
            TabsBuilder::new()
                .add_tab("One", Element::text("FIRST"))
                .add_tab(
                    "Two",
                    builder::text_input()
                        .value("seed")
                        .build()
                        .with_key("entry"),
                )
                .closable(true)
                .render()
                .auto_focus()
        }
    }
    for size in [(32, 8), (60, 14)] {
        let frames = run(
            Fresh,
            size,
            vec![
                (1, key(KeyCode::Right)),
                (2, key(KeyCode::Tab)),
                (2, key(KeyCode::End)),
                (2, key(KeyCode::Char('X'))),
                (3, None),
            ],
        );
        assert!(
            frames.last().unwrap().text.contains("seedX"),
            "{}",
            frames.last().unwrap().text
        );
        assert!(!frames.last().unwrap().text.contains("FIRST"));
        let frames = run(Fresh, size, vec![(1, key(KeyCode::Delete)), (2, None)]);
        assert!(frames.last().unwrap().text.contains("seed"));
        assert!(
            !frames.last().unwrap().text.contains("One"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn tabs_builders_switch_real_panels_and_preserve_child_input() {
    for size in [(32, 8), (60, 14)] {
        for native in [true, false] {
            let input = builder::text_input().value("seed").build();
            let control = if native {
                TabsBuilder::new()
                    .add_tab("One", Element::text("FIRST"))
                    .add_tab("Two", input)
                    .render()
            } else {
                builder::tabs()
                    .tab("One", Element::text("FIRST"))
                    .tab("Two", input)
                    .build()
            }
            .auto_focus();
            let frames = run(
                Control(control),
                size,
                vec![
                    (1, key(KeyCode::Right)),
                    (2, key(KeyCode::Tab)),
                    (2, key(KeyCode::End)),
                    (2, key(KeyCode::Char('X'))),
                    (3, None),
                ],
            );
            assert!(frames[0].text.contains("FIRST"), "{}", frames[0].text);
            assert!(
                frames.last().unwrap().text.contains("seedX"),
                "{:?}",
                frames.iter().map(|f| &f.text).collect::<Vec<_>>()
            );
            assert!(!frames.last().unwrap().text.contains("FIRST"));
        }
    }
}

#[test]
fn tabs_manual_activation_skips_disabled_and_empty_is_inert() {
    for size in [(32, 8), (60, 14)] {
        let control = TabsBuilder::new()
            .add_tab("One", Element::text("FIRST"))
            .tab(Tab::new("Locked", Element::text("FORBIDDEN")).disabled(true))
            .add_tab("Three", Element::text("THIRD"))
            .keyboard_activation(TabKeyboardActivation::Manual)
            .render()
            .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Right)),
                (2, key(KeyCode::Enter)),
                (3, None),
            ],
        );
        assert!(frames[1].text.contains("FIRST"), "{}", frames[1].text);
        assert!(
            frames.last().unwrap().text.contains("THIRD"),
            "{}",
            frames.last().unwrap().text
        );
        for control in [
            TabsBuilder::new().active_tab(usize::MAX).render(),
            TabsBuilder::new()
                .add_tab("One", Element::text("FIRST"))
                .add_tab("Two", Element::text("SECOND"))
                .disabled(true)
                .render(),
        ] {
            let frames = run(
                Control(control.auto_focus()),
                size,
                vec![
                    (1, key(KeyCode::Right)),
                    (1, key(KeyCode::Delete)),
                    (1, click(10, 0)),
                    (1, None),
                ],
            );
            assert!(!frames.last().unwrap().text.contains("SECOND"));
        }
    }
}

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
fn tabs_positions_use_measured_unicode_header_and_close_targets() {
    use reactive_tui::{
        component::Component,
        widgets::layout::{TabOrientation, TabPosition, TabVariant, Tabs},
    };
    use std::sync::{Arc, Mutex};
    for size in [(36, 12), (64, 18)] {
        for position in [
            TabPosition::Top,
            TabPosition::Bottom,
            TabPosition::Left,
            TabPosition::Right,
        ] {
            for orientation in [TabOrientation::Horizontal, TabOrientation::Vertical] {
                let changes = Arc::new(Mutex::new(Vec::new()));
                let closes = Arc::new(Mutex::new(Vec::new()));
                let make = || {
                    let changes = changes.clone();
                    let closes = closes.clone();
                    Element::typed_with::<Tabs>(
                        TabsBuilder::new()
                            .tab(Tab::new("界One", Element::text("FIRST")))
                            .tab(Tab::new("Beta", Element::text("SECOND")).closable(true))
                            .position(position.clone())
                            .orientation(orientation.clone())
                            .variant(TabVariant::Unstyled)
                            .build(),
                        move |props| {
                            let changes = changes.clone();
                            let closes = closes.clone();
                            Tabs::new(props)
                                .with_on_change(move |i| changes.lock().unwrap().push(i))
                                .with_on_close(move |i| closes.lock().unwrap().push(i))
                        },
                    )
                    .with_class("w-full h-10 p-0.5")
                    .auto_focus()
                };
                let baseline = run(Control(make()), size, vec![(1, None)]);
                let (x, y) = cell_of(&baseline[0].text, "Beta");
                let (cx, cy) = cell_of(&baseline[0].text, "✕");
                let frames = run(
                    Control(make()),
                    size,
                    vec![
                        (1, click(x, y)),
                        (2, click(x, y)),
                        (2, click(cx, cy)),
                        (2, None),
                    ],
                );
                assert!(
                    frames.last().unwrap().text.contains("SECOND"),
                    "{position:?} {orientation:?}: {:?}",
                    frames.iter().map(|f| &f.text).collect::<Vec<_>>()
                );
                assert_eq!(*changes.lock().unwrap(), vec![1]);
                assert_eq!(*closes.lock().unwrap(), vec![1]);
            }
        }
    }
}

#[test]
fn tabs_eager_panels_retain_input_state_and_hidden_controls_ignore_clicks() {
    for size in [(32, 8), (60, 14)] {
        let control = TabsBuilder::new()
            .lazy_loading(false)
            .add_tab("One", builder::text_input().value("seed").build())
            .add_tab("Two", Element::text("SECOND"))
            .render()
            .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Tab)),
                (1, key(KeyCode::End)),
                (1, key(KeyCode::Char('X'))),
                (2, click(13, 0)),
                (3, click(3, 0)),
                (5, None),
            ],
        );
        assert!(
            frames[1].text.contains("seedX"),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(
            frames[2].text.contains("SECOND"),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(!frames[2].text.contains("seed"));
        assert!(
            frames.last().unwrap().text.contains("seedX"),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
    }
}

#[test]
fn tabs_resize_moves_header_targets_and_key_release_does_not_close() {
    use reactive_tui::{
        component::Component,
        event::types::{Event, KeyEvent, KeyEventKind, ResizeEvent},
        widgets::layout::{TabPosition, TabVariant, Tabs},
    };
    use std::sync::{Arc, Mutex};
    for size in [(36, 12), (64, 18)] {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let closes = Arc::new(Mutex::new(Vec::new()));
        let make = || {
            let changes = changes.clone();
            let closes = closes.clone();
            Element::typed_with::<Tabs>(
                TabsBuilder::new()
                    .add_tab("One", Element::text("FIRST"))
                    .tab(Tab::new("Beta", Element::text("SECOND")).closable(true))
                    .position(TabPosition::Right)
                    .variant(TabVariant::Unstyled)
                    .build(),
                move |props| {
                    let changes = changes.clone();
                    let closes = closes.clone();
                    Tabs::new(props)
                        .with_on_change(move |i| changes.lock().unwrap().push(i))
                        .with_on_close(move |i| closes.lock().unwrap().push(i))
                },
            )
            .with_class("w-full h-8")
            .auto_focus()
        };
        let small = (size.0 - 12, size.1);
        let before = run(Control(make()), size, vec![(1, None)]);
        let after = run(Control(make()), small, vec![(1, None)]);
        let (ox, oy) = cell_of(&before[0].text, "Beta");
        let (x, y) = cell_of(&after[0].text, "Beta");
        assert!(ox > x);
        let mut release = KeyEvent::new(KeyCode::Delete);
        release.kind = KeyEventKind::Release;
        let frames = run(
            Control(make()),
            size,
            vec![
                (1, Some(Event::Resize(ResizeEvent::new(small.0, small.1)))),
                (2, click(ox, oy)),
                (2, click(x, y)),
                (3, Some(Event::Key(release))),
                (3, key(KeyCode::Delete)),
                (3, None),
            ],
        );
        assert!(frames.last().unwrap().text.contains("SECOND"));
        assert_eq!(*changes.lock().unwrap(), vec![1]);
        assert_eq!(*closes.lock().unwrap(), vec![1]);
    }
}

#[test]
fn tabs_lazy_loading_controls_mounting_and_hidden_panels_cannot_activate() {
    use reactive_tui::{component::Component, widgets::layout::TabVariant};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    #[derive(Clone, PartialEq)]
    struct PanelProps;
    impl reactive_tui::component::Props for PanelProps {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct Panel(Arc<AtomicUsize>);
    impl Component for Panel {
        type Props = PanelProps;
        type State = ();
        fn new(_: PanelProps) -> Self {
            unreachable!("factory supplies counter")
        }
        fn render(&self, _: &PanelProps, _: &()) -> Element {
            let calls = self.0.clone();
            builder::button()
                .text("Hidden action")
                .class("w-18 h-1 p-0")
                .on_click(move || {
                    calls.fetch_add(1, Ordering::SeqCst);
                })
                .build()
        }
    }
    for size in [(32, 8), (60, 14)] {
        for lazy in [true, false] {
            let mounts = Arc::new(AtomicUsize::new(0));
            let calls = Arc::new(AtomicUsize::new(0));
            let make = || {
                let mounts = mounts.clone();
                let calls = calls.clone();
                TabsBuilder::new()
                    .lazy_loading(lazy)
                    .variant(TabVariant::Unstyled)
                    .add_tab("One", Element::text("FIRST"))
                    .add_tab(
                        "Two",
                        Element::typed_with::<Panel>(PanelProps, move |_| {
                            mounts.fetch_add(1, Ordering::SeqCst);
                            Panel(calls.clone())
                        }),
                    )
                    .render()
                    .auto_focus()
            };
            let initial = run(Control(make()), size, vec![(1, click(3, 1)), (1, None)]);
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            assert_eq!(mounts.load(Ordering::SeqCst), usize::from(!lazy));
            assert!(!initial[0].text.contains("Hidden action"));
            mounts.store(0, Ordering::SeqCst);
            let frames = run(
                Control(make()),
                size,
                vec![(1, key(KeyCode::Right)), (2, click(3, 1)), (2, None)],
            );
            assert!(frames.last().unwrap().text.contains("Hidden action"));
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(mounts.load(Ordering::SeqCst), 1);
        }
    }
}

#[test]
fn tabs_builder_close_removes_panels_without_a_parent_callback() {
    for size in [(32, 8), (60, 14)] {
        let control = builder::tabs()
            .tab("One", Element::text("FIRST"))
            .tab("Two", Element::text("SECOND"))
            .closable(true)
            .build()
            .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Delete)),
                (2, key(KeyCode::Delete)),
                (3, None),
            ],
        );
        assert!(frames[1].text.contains("SECOND"));
        assert!(!frames[1].text.contains("FIRST"));
        assert!(!frames.last().unwrap().text.contains("SECOND"));
        assert!(!frames.last().unwrap().text.contains("✕"));
    }
}

#[test]
fn tabs_badge_icon_tooltip_and_variants_reach_frames() {
    use reactive_tui::{
        event::types::{Event, MouseEvent, MouseEventKind, Position},
        widgets::layout::{TabBadge, TabBadgeVariant, TabSize, TabVariant},
    };
    for size in [(32, 8), (60, 14)] {
        for variant in [
            TabVariant::Line,
            TabVariant::Enclosed,
            TabVariant::Soft,
            TabVariant::Solid,
            TabVariant::Unstyled,
        ] {
            for tab_size in [TabSize::Small, TabSize::Medium, TabSize::Large] {
                let make = || {
                    TabsBuilder::new()
                        .tab(
                            Tab::new("Title", Element::text("BODY"))
                                .with_icon("界")
                                .with_badge(TabBadge::new("5").with_variant(TabBadgeVariant::Error))
                                .with_tooltip("Helpful hint"),
                        )
                        .variant(variant.clone())
                        .size(tab_size.clone())
                        .render()
                        .with_class("w-full h-7")
                };
                let initial = run(Control(make()), size, vec![(1, None)]);
                let (x, y) = cell_of(&initial[0].text, "Title");
                assert!(initial[0].text.contains("界"));
                assert!(initial[0].text.contains("✗5"));
                let hover = Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Move,
                    Position::cell(x, y),
                )));
                let frames = run(Control(make()), size, vec![(1, hover), (2, None)]);
                assert!(
                    frames.last().unwrap().text.contains("Helpful hint"),
                    "{variant:?} {tab_size:?}: {}",
                    frames.last().unwrap().text
                );
                assert!(frames.last().unwrap().text.contains("BODY"));
            }
        }
    }
}

#[test]
fn closing_a_preceding_tab_preserves_the_active_unkeyed_editor() {
    for size in [(32, 8), (60, 14)] {
        let make = || {
            TabsBuilder::new()
                .add_tab("One", Element::text("FIRST"))
                .add_tab("Two", builder::text_input().value("seed").build())
                .closable(true)
                .render()
                .auto_focus()
        };
        let initial = run(Control(make()), size, vec![(1, None)]);
        let (x, y) = cell_of(&initial[0].text, "✕");
        let frames = run(
            Control(make()),
            size,
            vec![
                (1, key(KeyCode::Right)),
                (2, key(KeyCode::Tab)),
                (2, key(KeyCode::End)),
                (2, key(KeyCode::Char('X'))),
                (3, click(x, y)),
                (4, None),
            ],
        );
        assert!(
            frames.last().unwrap().text.contains("seedX"),
            "{}",
            frames.last().unwrap().text
        );
        assert!(!frames.last().unwrap().text.contains("One"));
    }
}
