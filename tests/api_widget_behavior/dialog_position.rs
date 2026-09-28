use super::{app_input, Control};
use reactive_tui::{
    builder,
    component::Element,
    core::geometry::{Point, Rect, Size},
    widgets::dialog::{
        ConfirmationDialog, ConfirmationDialogOptions, DialogAnchor, DialogComponent, DialogId,
        DialogPosition, DialogTheme,
    },
};

#[test]
fn confirmation_positions_follow_viewport_resize_and_explicit_bounds() {
    use reactive_tui::event::types::{Event, ResizeEvent};
    use std::sync::Arc;
    for (initial, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let origin = |mode, (width, height): (u16, u16)| match mode {
            0 => ((width - 14) / 2, (height - 6) / 2),
            1 => ((width - 14) / 2, 0),
            2 => ((width - 14) / 2, height - 6),
            3 => (0, (height - 6) / 2),
            4 => (width - 14, (height - 6) / 2),
            5 => (2, 1),
            6 => (width / 4, height / 4),
            _ => (3, 2),
        };
        for (mode, position) in [
            DialogPosition::Center,
            DialogPosition::TopCenter,
            DialogPosition::BottomCenter,
            DialogPosition::LeftCenter,
            DialogPosition::RightCenter,
            DialogPosition::Fixed(Point::new(2, 1)),
            DialogPosition::Custom(Arc::new(|size| Point::new(size.width / 4, size.height / 4))),
            DialogPosition::Center,
        ]
        .into_iter()
        .enumerate()
        {
            let element = ConfirmationDialog::new(
                DialogId::from_u32(97),
                ConfirmationDialogOptions {
                    title: "DIALOG".into(),
                    message: "BODY".into(),
                    size: Some(Size::new(14, 6)),
                    position,
                    ..Default::default()
                },
            )
            .render(
                if mode == 7 {
                    Rect::new(Point::new(3, 2), Size::new(14, 6))
                } else {
                    Rect::default()
                },
                &DialogTheme::default(),
            );
            let before = origin(mode, initial);
            let after = origin(mode, resized);
            let frames = app_input::run_when_cell(
                Control(element),
                initial,
                vec![
                    app_input::CellStep {
                        x: before.0,
                        y: before.1,
                        content: "┌",
                        event: Some(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                    },
                    app_input::CellStep {
                        x: after.0,
                        y: after.1,
                        content: "┌",
                        event: None,
                    },
                ],
            );
            let corner = frames
                .last()
                .and_then(|frame| frame.screen.cell(after.1, after.0))
                .map(|cell| cell.contents());
            assert_eq!(
                corner,
                Some("┌"),
                "dialog corner for mode {mode} sits at {after:?} after resizing {initial:?} to {resized:?}"
            );
        }
    }
}

fn anchored(anchor: DialogAnchor) -> Element {
    ConfirmationDialog::new(
        DialogId::from_u32(82),
        ConfirmationDialogOptions {
            title: "DIALOG".into(),
            message: "BODY".into(),
            size: Some(Size::new(14, 6)),
            position: DialogPosition::RelativeTo {
                element_id: "target".into(),
                offset: Point::new(1, 1),
                anchor,
            },
            ..Default::default()
        },
    )
    .render(Rect::default(), &DialogTheme::default())
}

#[test]
fn relative_dialog_uses_all_nine_measured_anchor_points_and_offset() {
    for size in [(32, 12), (60, 20)] {
        for (anchor, x, y) in [
            (DialogAnchor::TopLeft, 3, 2),
            (DialogAnchor::TopCenter, 7, 2),
            (DialogAnchor::TopRight, 11, 2),
            (DialogAnchor::CenterLeft, 3, 3),
            (DialogAnchor::Center, 7, 3),
            (DialogAnchor::CenterRight, 11, 3),
            (DialogAnchor::BottomLeft, 3, 4),
            (DialogAnchor::BottomCenter, 7, 4),
            (DialogAnchor::BottomRight, 11, 4),
        ] {
            let element = builder::div()
                .class("w-full h-full")
                .children(vec![
                    builder::div()
                        .id("target")
                        .class("absolute left-2 top-1 w-8 h-2")
                        .text("ANCHOR")
                        .build(),
                    anchored(anchor),
                ])
                .build();
            let frames = app_input::run_when(Control(element), size, vec![("BODY", None)]);
            let frame = frames.last().unwrap();
            assert_eq!(
                frame.screen.cell(y, x).unwrap().contents(),
                "┌",
                "{}",
                frame.text
            );
        }
    }
}

#[test]
fn relative_dialog_reports_ambiguous_and_removed_anchors() {
    use reactive_tui::{
        app::RootComponent,
        event::{
            router::EventResult,
            types::{Event, KeyCode},
        },
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Changing {
        removed: AtomicBool,
        duplicate: bool,
    }
    impl RootComponent for Changing {
        fn render(&self) -> Element {
            let mut children = vec![anchored(DialogAnchor::TopLeft)];
            if !self.removed.load(Ordering::SeqCst) {
                children.push(
                    builder::div()
                        .class("absolute left-2 top-1 w-8 h-2")
                        .id("target")
                        .text("ANCHOR")
                        .build(),
                );
                if self.duplicate {
                    children.push(
                        builder::div()
                            .class("absolute left-15 top-1 w-8 h-2")
                            .child(
                                builder::div()
                                    .id("target")
                                    .class("w-8 h-2")
                                    .text("OTHER")
                                    .build(),
                            )
                            .build(),
                    );
                }
            }
            builder::div()
                .class("w-full h-full")
                .children(children)
                .build()
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
                self.removed.store(true, Ordering::SeqCst);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }
    for size in [(32, 12), (60, 20)] {
        app_input::run_when(
            Changing {
                removed: AtomicBool::new(false),
                duplicate: true,
            },
            size,
            vec![("anchor is ambiguous", None)],
        );
        let frames = app_input::run_when(
            Changing {
                removed: AtomicBool::new(false),
                duplicate: false,
            },
            size,
            vec![
                ("BODY", app_input::key(KeyCode::F(2))),
                ("anchor not resolved", None),
            ],
        );
        assert!(!frames.last().unwrap().text.contains("BODY"));
        assert!(!frames.last().unwrap().text.contains("ANCHOR"));
    }
}

#[test]
fn relative_dialog_tracks_changed_parent_layout_and_viewport_resize() {
    use reactive_tui::{
        app::RootComponent,
        event::{
            router::EventResult,
            types::{Event, KeyCode, ResizeEvent},
        },
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Moving(AtomicBool);
    impl RootComponent for Moving {
        fn render(&self) -> Element {
            let anchor = builder::div()
                .id("target")
                .class(if self.0.load(Ordering::SeqCst) {
                    "absolute right-0 top-1 w-8 h-2"
                } else {
                    "absolute left-2 top-1 w-8 h-2"
                })
                .text("ANCHOR")
                .build();
            builder::div()
                .class("w-full h-full")
                .children(vec![
                    builder::div().class("w-1/2 h-full").child(anchor).build(),
                    anchored(DialogAnchor::TopLeft),
                ])
                .build()
        }
        fn handle_event(&self, event: &Event) -> EventResult {
            if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
                self.0.store(true, Ordering::SeqCst);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }
    for (size, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        let frames = app_input::run_when_cell(
            Moving(AtomicBool::new(false)),
            size,
            vec![
                app_input::CellStep {
                    x: 3,
                    y: 2,
                    content: "┌",
                    event: app_input::key(KeyCode::F(2)),
                },
                app_input::CellStep {
                    x: size.0 / 2 - 7,
                    y: 2,
                    content: "┌",
                    event: Some(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                },
                app_input::CellStep {
                    x: resized.0 / 2 - 7,
                    y: 2,
                    content: "┌",
                    event: None,
                },
            ],
        );
        let last = frames.last().expect("a frame after the resize");
        assert!(
            last.screen
                .cell(2, resized.0 / 2 - 7)
                .is_some_and(|c| c.contents() == "┌"),
            "the dialog must sit at the new center after resizing to {resized:?}:\n{}",
            last.text
        );
        // The anchor is refreshed before the first frame at the new size is
        // presented, so that frame already places the dialog.
        let first = frames
            .iter()
            .find(|frame| frame.screen.size() == (resized.1, resized.0))
            .expect("a frame at the new size");
        assert!(
            first
                .screen
                .cell(2, resized.0 / 2 - 7)
                .is_some_and(|c| c.contents() == "┌"),
            "the first frame after resizing to {resized:?} must place the dialog at the new anchor:\n{}",
            first.text
        );
    }
}
