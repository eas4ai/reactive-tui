use super::{key, run, Control};
use reactive_tui::{component::Element, event::types::KeyCode, widgets::layout::ScrollViewBuilder};

fn wheel(x: u16, y: u16, dx: f32, dy: f32) -> Option<reactive_tui::event::Event> {
    use reactive_tui::event::types::{
        Event, MouseEvent, Position, WheelDelta, WheelEvent, WheelPhase,
    };
    Some(Event::Mouse(MouseEvent::wheel(
        Position::cell(x, y),
        WheelEvent {
            delta: WheelDelta::Lines { x: dx, y: dy },
            phase: WheelPhase::Changed,
        },
    )))
}

#[test]
fn scroll_wheel_has_direction_and_horizontal_unicode_uses_cells() {
    for size in [(24, 8), (48, 12)] {
        let scroll = ScrollViewBuilder::new(Element::text("zero\none\ntwo\nthree\nfour\nfive"))
            .viewport_size(10, 3)
            .scroll_x(false)
            .scroll_speed(2)
            .show_scrollbars(false)
            .render();
        let frames = run(
            Control(scroll),
            size,
            vec![
                (1, wheel(2, 1, 0.0, 1.0)),
                (2, wheel(2, 1, 0.0, -1.0)),
                (3, None),
            ],
        );
        assert!(frames[1].text.contains("two"), "{}", frames[1].text);
        assert!(!frames[1].text.contains("zero"));
        assert!(frames.last().unwrap().text.contains("zero"));
        let scroll = ScrollViewBuilder::new(Element::text("界界ABCDEFGH"))
            .viewport_size(6, 1)
            .scroll_y(false)
            .show_scrollbars(false)
            .scroll_speed(1)
            .render()
            .auto_focus();
        let frames = run(
            Control(scroll),
            size,
            vec![(1, wheel(2, 0, 4.0, 0.0)), (2, None)],
        );
        assert!(frames[0].text.contains("界界AB"), "{}", frames[0].text);
        assert!(
            frames.last().unwrap().text.contains("ABCDEF"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn specialized_scroll_keeps_child_input_and_clips_hidden_hits() {
    use reactive_tui::builder;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(24, 8), (48, 12)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let callback = calls.clone();
        let scroll = builder::specialized::ScrollViewBuilder::new()
            .show_scrollbars(false)
            .content(Element::text("one\ntwo\nthree").with_class("h-3 w-10 flex-none"))
            .content(
                builder::button()
                    .text("Apply")
                    .class("w-8 h-1 p-0 flex-none")
                    .on_click(move || {
                        callback.fetch_add(1, Ordering::SeqCst);
                    })
                    .build(),
            )
            .class("w-10 h-2")
            .build()
            .auto_focus();
        let frames = run(
            Control(scroll),
            size,
            vec![
                (1, super::click(2, 3)),
                (1, key(KeyCode::End)),
                (2, super::click(2, 1)),
                (2, None),
            ],
        );
        assert!(!frames[0].text.contains("Apply"), "{}", frames[0].text);
        assert!(
            frames.last().unwrap().text.contains("Apply"),
            "{}",
            frames.last().unwrap().text
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn scroll_resize_preserves_position_then_clamps_to_new_viewport() {
    use reactive_tui::event::types::{Event, ResizeEvent};
    for size in [(24, 8), (48, 12)] {
        let scroll = ScrollViewBuilder::new(Element::text("zero\none\ntwo\nthree\nfour\nfive"))
            .scroll_x(false)
            .show_scrollbars(false)
            .render()
            .with_class("w-full h-full")
            .auto_focus();
        let frames = run(
            Control(scroll),
            (size.0, 3),
            vec![
                (1, key(KeyCode::Down)),
                (2, Some(Event::Resize(ResizeEvent::new(size.0, 2)))),
                (3, key(KeyCode::End)),
                (4, Some(Event::Resize(ResizeEvent::new(size.0, size.1)))),
                (6, None),
            ],
        );
        assert!(frames[1].text.contains("one"), "{}", frames[1].text);
        assert!(frames
            .iter()
            .any(|f| f.text.contains("five") && !f.text.contains("three")));
        assert!(
            frames.last().unwrap().text.contains("zero"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn scrollbars_follow_offsets_and_disabled_axes_and_empty_content_are_inert() {
    for size in [(24, 8), (48, 12)] {
        let scroll = ScrollViewBuilder::new(Element::text("zero\none\ntwo\nthree\nfour\nfive"))
            .viewport_size(8, 3)
            .scroll_x(false)
            .render()
            .auto_focus();
        let frames = run(
            Control(scroll),
            size,
            vec![(2, key(KeyCode::End)), (3, None)],
        );
        assert_eq!(frames[1].screen.cell(0, 7).unwrap().contents(), "█");
        assert_eq!(
            frames.last().unwrap().screen.cell(2, 7).unwrap().contents(),
            "█"
        );
        assert!(frames.last().unwrap().text.contains("five"));
        let scroll = ScrollViewBuilder::new(Element::text("zero\none\ntwo\nthree"))
            .viewport_size(8, 2)
            .scroll_x(false)
            .scroll_y(false)
            .render()
            .auto_focus();
        let frames = run(
            Control(scroll),
            size,
            vec![
                (1, key(KeyCode::End)),
                (1, wheel(1, 0, 1.0, 1.0)),
                (1, None),
            ],
        );
        assert!(frames
            .iter()
            .all(|f| f.text.contains("zero") && !f.text.contains("three")));
        let scroll = ScrollViewBuilder::default()
            .viewport_size(0, 0)
            .render()
            .auto_focus();
        let frames = run(
            Control(scroll),
            size,
            vec![(1, key(KeyCode::End)), (1, None)],
        );
        assert!(frames.iter().all(|f| f.text.trim().is_empty()));
    }
}

#[test]
fn smooth_scroll_paints_intermediate_content() {
    for size in [(24, 8), (48, 12)] {
        let content = (0..30)
            .map(|i| format!("row {i:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        let scroll = ScrollViewBuilder::new(Element::text(content))
            .viewport_size(10, 2)
            .scroll_x(false)
            .show_scrollbars(false)
            .smooth_scroll(true)
            .render()
            .auto_focus();
        let frames = run(
            Control(scroll),
            size,
            vec![(1, key(KeyCode::End)), (3, None)],
        );
        assert!(frames[0].text.contains("row 00"));
        assert!(
            frames
                .iter()
                .skip(1)
                .any(|f| !f.text.contains("row 00") && !f.text.contains("row 29")),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(frames.len() < 12, "smooth scrolling must not spin");
    }
}

#[test]
fn padded_scrollbars_and_wheel_hits_stay_inside_the_viewport() {
    for size in [(24, 8), (48, 12)] {
        let scroll = ScrollViewBuilder::new(Element::text("zero\none\ntwo\nthree\nfour\nfive"))
            .scroll_x(false)
            .scroll_speed(3)
            .render()
            .with_class("w-12 h-6 p-0.5");
        let frames = run(
            Control(scroll),
            size,
            vec![
                (2, wheel(3, 2, 0.0, 1.0)),
                (3, wheel(14, 2, 0.0, -1.0)),
                (3, wheel(3, 2, 0.0, -1.0)),
                (4, None),
            ],
        );
        assert_eq!(
            frames[1].screen.cell(2, 9).unwrap().contents(),
            "█",
            "{}",
            frames[1].text
        );
        assert!(frames.iter().any(|f| f.text.contains("three")));
        assert!(frames.last().unwrap().text.contains("zero"));
        assert!(frames
            .iter()
            .all(|f| f.screen.cell(0, 0).unwrap().contents().trim().is_empty()));
    }
}

#[test]
fn scroll_view_keeps_unicode_content_and_reaches_last_row() {
    for size in [(24, 8), (48, 12)] {
        let scroll = ScrollViewBuilder::new(Element::text(
            "α first\nβ second\nγ third\nδ fourth\nε last",
        ))
        .viewport_size(12, 3)
        .show_scrollbars(false)
        .scroll_x(false)
        .render()
        .auto_focus();
        let frames = run(
            Control(scroll),
            size,
            vec![(1, key(KeyCode::End)), (2, None)],
        );
        assert!(frames[0].text.contains("α first"), "{}", frames[0].text);
        assert!(!frames[0].text.contains("ε last"));
        assert!(
            frames.last().unwrap().text.contains("ε last"),
            "{}",
            frames.last().unwrap().text
        );
        assert!(!frames.last().unwrap().text.contains("α first"));
    }
}
