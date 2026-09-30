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
            .with_class("w-12 h-6 p-2");
        let frames = settled(
            scroll,
            size,
            vec![
                Until {
                    text: "zero",
                    cell: Some((9, 2, "█")),
                    event: wheel(3, 2, 0.0, 1.0),
                },
                step("three", wheel(14, 2, 0.0, -1.0)),
                step("three", wheel(3, 2, 0.0, -1.0)),
                step("zero", None),
            ],
        );
        assert_eq!(
            frames.last().unwrap().screen.cell(2, 9).unwrap().contents(),
            "█",
            "{}",
            frames.last().unwrap().text
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

use super::app_input::{run_until_on_debug, Snapshot, Until};
use reactive_tui::{
    builder,
    event::types::{Event, MouseButton, MouseEvent, MouseEventKind, Position},
};
use std::time::Duration;

fn settled(scroll: Element, size: (u16, u16), steps: Vec<Until>) -> Vec<Snapshot> {
    run_until_on_debug(Control(scroll), size, steps, Duration::from_secs(5))
}

fn step(text: &'static str, event: Option<Event>) -> Until {
    Until {
        text,
        cell: None,
        event,
    }
}

fn rows(count: usize, width: usize) -> Element {
    Element::text(
        (0..count)
            .map(|i| format!("row {i:02}{}", "x".repeat(width.saturating_sub(6))))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn pointer(kind: MouseEventKind, x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(
        MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left),
    ))
}

#[test]
fn nav_002_scroll_default_fills_parent() {
    let scroll = ScrollViewBuilder::new(rows(60, 100)).render();
    let root = builder::div().class("w-100 h-20").child(scroll).build();
    let frames = settled(root, (120, 30), vec![step("row 00", None)]);
    let frame = frames.last().unwrap();
    assert_eq!(
        frame.screen.cell(0, 99).unwrap().contents(),
        "█",
        "bar fills parent width"
    );
    assert_eq!(
        frame.screen.cell(19, 98).unwrap().contents(),
        "x",
        "content fills parent height"
    );
    assert!(
        frame
            .screen
            .cell(20, 0)
            .unwrap()
            .contents()
            .trim()
            .is_empty(),
        "view ends at row 20"
    );
}

#[test]
fn nav_002_scroll_fitting_content_keeps_full_width() {
    let frames = settled(
        ScrollViewBuilder::new(rows(5, 20))
            .viewport_size(20, 10)
            .scroll_x(false)
            .render(),
        (30, 12),
        vec![step("row 00", None)],
    );
    let frame = frames.last().unwrap();
    assert_eq!(
        frame.screen.cell(0, 19).unwrap().contents(),
        "x",
        "fitting content keeps bar column"
    );
    assert!(
        !frame.text.contains(['█', '░']),
        "fitting content has no bar"
    );
}

#[test]
fn nav_002_scroll_builder_classes_set_size() {
    let scroll = builder::scroll_view()
        .content(rows(60, 50))
        .class("w-40 h-10")
        .build();
    let frames = settled(scroll, (100, 20), vec![step("row 00", None)]);
    let frame = frames.last().unwrap();
    assert_eq!(frame.screen.cell(0, 39).unwrap().contents(), "█");
    assert_eq!(frame.screen.cell(9, 38).unwrap().contents(), "x");
    assert!(frame
        .screen
        .cell(10, 0)
        .unwrap()
        .contents()
        .trim()
        .is_empty());
}

#[test]
fn nav_003_scroll_track_click_pages() {
    let scroll = ScrollViewBuilder::new(rows(60, 6))
        .viewport_size(12, 20)
        .scroll_x(false)
        .render();
    let frames = settled(
        scroll,
        (30, 24),
        vec![step("row 00", super::click(11, 12)), step("", None)],
    );
    assert_eq!(
        frames.last().unwrap().screen.cell(0, 0).unwrap().contents(),
        "r"
    );
    assert!(
        frames.last().unwrap().text.starts_with("row 20"),
        "track click scrolls one page: {}",
        frames.last().unwrap().text
    );
}

#[test]
fn nav_003_scroll_thumb_drag_uses_travel_and_stops_on_release() {
    let scroll = ScrollViewBuilder::new(rows(60, 6))
        .viewport_size(12, 20)
        .scroll_x(false)
        .smooth_scroll(true)
        .render();
    let frames = settled(
        scroll,
        (30, 24),
        vec![
            step("row 00", pointer(MouseEventKind::Down, 11, 0)),
            step("row 00", pointer(MouseEventKind::Drag, 11, 5)),
            step("", pointer(MouseEventKind::Up, 11, 5)),
            step("", pointer(MouseEventKind::Drag, 11, 10)),
            step("", None),
        ],
    );
    // ceil(20 * 20 / 60) = 7 thumb cells; 5 * 40 / 13 rounds to 15.
    assert!(
        frames.last().unwrap().text.starts_with("row 15"),
        "drag follows thumb travel: {}",
        frames.last().unwrap().text
    );
    assert!(
        frames
            .iter()
            .filter(|frame| frame.text.contains("row"))
            .all(|frame| frame.text.starts_with("row 00") || frame.text.starts_with("row 15")),
        "drag paints its offset immediately even with smooth scrolling enabled"
    );
}

#[test]
fn nav_003_scroll_horizontal_track_and_thumb_are_symmetric() {
    let content = Element::text("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwx");
    for drag in [false, true] {
        let scroll = ScrollViewBuilder::new(content.clone())
            .viewport_size(20, 3)
            .scroll_y(false)
            .render();
        let steps = if drag {
            vec![
                step("012345", pointer(MouseEventKind::Down, 0, 2)),
                step("012345", pointer(MouseEventKind::Drag, 5, 2)),
                step("", pointer(MouseEventKind::Up, 5, 2)),
                step("", None),
            ]
        } else {
            vec![step("012345", super::click(12, 2)), step("", None)]
        };
        let frames = settled(scroll, (30, 6), steps);
        assert!(
            frames
                .last()
                .unwrap()
                .text
                .starts_with(if drag { "FGHIJ" } else { "KLMNO" }),
            "horizontal pointer scroll: {}",
            frames.last().unwrap().text
        );
    }
}

struct ScrollTheme(std::sync::Arc<reactive_tui::theme::Theme>);
impl Drop for ScrollTheme {
    fn drop(&mut self) {
        reactive_tui::theme::Theme::set_active((*self.0).clone());
    }
}

#[test]
#[serial_test::serial(theme)]
fn nav_001_scroll_thumb_and_track_use_theme_roles() {
    use reactive_tui::theme::{Theme, ThemeVariables};
    let _restore = ScrollTheme(Theme::active());
    let mut variables = ThemeVariables::new();
    for (name, color) in [
        ("foreground", [200, 201, 202]),
        ("text-muted", [31, 41, 51]),
        ("border", [61, 71, 81]),
    ] {
        let [r, g, b] = color;
        variables = variables.set(
            Theme::color_variable(name),
            format!("#{r:02x}{g:02x}{b:02x}"),
        );
    }
    Theme::set_active(Theme::new("scroll-probe").with_variables(variables));
    for scroll in [
        ScrollViewBuilder::new(rows(60, 6))
            .viewport_size(12, 20)
            .scroll_x(false)
            .render(),
        builder::scroll_view()
            .content(rows(60, 6))
            .class("w-12 h-20")
            .build(),
    ] {
        let frames = settled(scroll, (30, 24), vec![step("row 00", None)]);
        let frame = frames.last().unwrap();
        assert_eq!(frame.screen.cell(0, 11).unwrap().contents(), "█");
        assert_eq!(
            frame.screen.cell(0, 11).unwrap().fgcolor(),
            vt100::Color::Rgb(31, 41, 51),
            "thumb uses text-muted"
        );
        assert_eq!(frame.screen.cell(12, 11).unwrap().contents(), "░");
        assert_eq!(
            frame.screen.cell(12, 11).unwrap().fgcolor(),
            vt100::Color::Rgb(61, 71, 81),
            "track uses border"
        );
    }
}

#[test]
fn nav_002_scroll_specialized_viewport_setters() {
    for scroll in [
        builder::scroll_view()
            .content(rows(60, 50))
            .viewport_size(40, 10)
            .build(),
        builder::scroll_view()
            .content(rows(60, 50))
            .viewport_width(40)
            .viewport_height(10)
            .build(),
    ] {
        let frames = settled(scroll, (100, 20), vec![step("row 00", None)]);
        let frame = frames.last().unwrap();
        assert_eq!(frame.screen.cell(0, 39).unwrap().contents(), "█");
        assert_eq!(frame.screen.cell(9, 38).unwrap().contents(), "x");
        assert!(frame
            .screen
            .cell(10, 0)
            .unwrap()
            .contents()
            .trim()
            .is_empty());
    }
}
