use crate::common::app_input::{self, click, Snapshot};
use reactive_tui::{
    app::RootComponent,
    component::Element,
    event::types::{
        Event, MouseButton, MouseEvent, MouseEventKind, Position, WheelDelta, WheelEvent,
        WheelPhase,
    },
    widgets::layout::ScrollViewBuilder,
};

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

fn wheel(x: u16, y: u16, dx: f32, dy: f32) -> Option<Event> {
    Some(Event::Mouse(MouseEvent::wheel(
        Position::cell(x, y),
        WheelEvent {
            delta: WheelDelta::Lines { x: dx, y: dy },
            phase: WheelPhase::Changed,
        },
    )))
}

fn drag(x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(
        MouseEvent::new(MouseEventKind::Drag, Position::cell(x, y)).with_button(MouseButton::Left),
    ))
}

/// Fixed content of 10 columns by 20 rows, each row numbered, in a 10 by 5
/// scroll view with both directions on and bars shown.
fn tall_and_wide() -> Element {
    let content = (0..20)
        .map(|row| format!("{row:02}cdefghij"))
        .collect::<Vec<_>>()
        .join("\n");
    ScrollViewBuilder::new(Element::text(content))
        .viewport_size(10, 5)
        .scroll_x(true)
        .scroll_y(true)
        .show_scrollbars(true)
        .render()
        .auto_focus()
}

/// Whether the cell holds a bar's track or thumb.
fn is_bar(frame: &Snapshot, row: u16, column: u16) -> bool {
    frame
        .screen
        .cell(row, column)
        .is_some_and(|cell| matches!(cell.contents(), "█" | "░"))
}

/// NAV-005: the vertical bar's column makes the ten-column content overflow
/// sideways, so the horizontal bar comes too, a wheel scrolls the column it
/// exposed, and the vertical bar takes the pointer at the column the joint
/// decision gives it.
#[test]
fn nav_005_a_bar_that_causes_overflow_brings_the_other_bar() {
    let size = (24, 8);
    let frames = app_input::run(Root(tall_and_wide()), size, vec![(2, None)]);
    let frame = frames.last().unwrap();
    assert!(
        (0..4).all(|row| is_bar(frame, row, 9)),
        "a vertical bar in the last column:\n{}",
        frame.text
    );
    assert!(
        (0..9).all(|column| is_bar(frame, 4, column)),
        "a horizontal bar in the last row:\n{}",
        frame.text
    );
    assert!(frame.text.lines().next().unwrap().starts_with("00cdefghi"));
    let frames = app_input::run(
        Root(tall_and_wide()),
        size,
        vec![(2, wheel(1, 1, 1.0, 0.0)), (3, None)],
    );
    let frame = frames.last().unwrap();
    assert!(
        frame.text.lines().next().unwrap().starts_with("0cdefghij"),
        "a horizontal wheel scrolls the column the vertical bar took:\n{}",
        frame.text
    );
    assert!(
        (0..9).all(|column| is_bar(frame, 4, column)),
        "{}",
        frame.text
    );
    let frames = app_input::run(
        Root(tall_and_wide()),
        size,
        vec![(2, click(9, 0)), (3, drag(9, 3)), (4, None)],
    );
    let frame = frames.last().unwrap();
    assert!(
        frame.text.lines().next().unwrap().starts_with("16cdefghi"),
        "a drag along the vertical bar, at the column the decision gives it, scrolls to the end:\n{}",
        frame.text
    );
}
