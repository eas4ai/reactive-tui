//! NAV-006: the breadcrumb's ellipsis and the tab bar's overflow mark.

use crate::common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::{
        router::EventResult,
        types::{Event, KeyCode, KeyEvent, KeyModifiers},
    },
    widgets::layout::BreadcrumbSegment,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

/// How long a run may take to reach every step before it fails: a hang
/// guard, not a timing check.
const WAIT: Duration = Duration::from_secs(10);

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

/// Ten tabs of 20 cells each: an 18-cell label with one cell of padding on
/// each side.
fn ten_tabs() -> Element {
    let mut tabs = builder::tabs();
    for n in 1..=10 {
        let label = format!("{:<18}", format!("Tab {n:02}"));
        tabs = tabs.tab(&label, Element::text(format!("Panel {n:02}")));
    }
    tabs.build()
}

/// A box of `width` cells and `height` rows on the page.
fn boxed(width: u16, height: u16, child: Element) -> Element {
    builder::div()
        .class(&format!("flex-col w-{width} h-{height}"))
        .child(child)
        .build()
}

/// A step that waits until `text` is painted and the App is idle, then
/// sends `event`.
fn until(text: &'static str, event: Option<Event>) -> Until {
    Until {
        text,
        cell: None,
        event,
    }
}

/// The cell where `needle` starts, as (column, row).
fn find(frame: &Snapshot, needle: &str) -> Option<(u16, u16)> {
    let (rows, columns) = frame.screen.size();
    let glyphs: Vec<String> = needle.chars().map(String::from).collect();
    let length = glyphs.len() as u16;
    (0..rows)
        .flat_map(|row| (0..=columns.saturating_sub(length)).map(move |column| (column, row)))
        .find(|(column, row)| {
            glyphs.iter().enumerate().all(|(offset, glyph)| {
                frame
                    .screen
                    .cell(*row, column + offset as u16)
                    .is_some_and(|cell| cell.contents() == *glyph)
            })
        })
}

fn at(frame: &Snapshot, needle: &str) -> (u16, u16) {
    find(frame, needle).unwrap_or_else(|| panic!("{needle:?} is not painted:\n{}", frame.text))
}

/// The glyph in the cell at (column, row).
fn glyph(frame: &Snapshot, column: u16, row: u16) -> String {
    frame
        .screen
        .cell(row, column)
        .map(|cell| cell.contents().to_string())
        .unwrap_or_default()
}

fn shift_f10() -> Option<Event> {
    Some(Event::Key(KeyEvent::new(KeyCode::F(10)).with_modifiers(
        KeyModifiers {
            shift: true,
            ..KeyModifiers::empty()
        },
    )))
}

#[test]
#[serial_test::serial(theme)]
fn nav_006_a_tab_bar_with_tabs_out_of_view_paints_an_overflow_mark() {
    // The bar knows its tabs are out of view once it has been laid out, so
    // the run ends once the App is idle, not at the first frame.
    let frames = app_input::run_until(
        Root(boxed(100, 10, ten_tabs())),
        (240, 60),
        vec![until("Panel 01", None)],
        WAIT,
    );
    let frame = frames.last().expect("a frame");
    let bar = frame
        .text
        .lines()
        .find(|line| line.contains("Tab 01"))
        .expect("the tab bar's row");
    assert!(
        bar.contains('»'),
        "ten tabs of 20 cells in a box of 100 cells paint an overflow mark at the bar's end:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_006_a_bar_whose_tabs_all_fit_paints_no_overflow_mark() {
    let tabs = builder::tabs()
        .tab("Preview", Element::text("Panel 01"))
        .tab("Source", Element::text("Panel 02"))
        .tab("Notes", Element::text("Panel 03"))
        .build();
    // The run ends once the App is idle, so a mark painted a frame after
    // the first is seen.
    let frames = app_input::run_until(
        Root(boxed(100, 10, tabs)),
        (240, 60),
        vec![until("Panel 01", None)],
        WAIT,
    );
    let frame = frames.last().expect("a frame");
    let (_, row) = at(frame, "Preview");
    let bar = frame.text.lines().nth(usize::from(row)).unwrap_or_default();
    assert!(
        !bar.contains('»') && !frame.text.contains('⌄'),
        "three tabs that fit a box of 100 cells paint no overflow mark:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_006_shift_f10_lists_every_tab_and_choosing_the_last_brings_it_into_view() {
    let frames = app_input::run_until(
        Root(boxed(100, 10, ten_tabs().auto_focus())),
        (240, 60),
        vec![
            until("Panel 01", shift_f10()),
            // The menu lists the tenth tab: End moves to it, Enter chooses it.
            until("[ ] Tab 10", app_input::key(KeyCode::End)),
            until("[ ] Tab 10", app_input::key(KeyCode::Enter)),
            until("Panel 10", None),
        ],
        WAIT,
    );
    let menu = frames
        .iter()
        .rev()
        .find(|frame| frame.text.contains("[ ] Tab 10"))
        .expect("a frame with the tab menu open");
    let rows: Vec<u16> = (1..=10)
        .map(|n| {
            let mark = if n == 1 { "[x]" } else { "[ ]" };
            at(menu, &format!("{mark} Tab {n:02}")).1
        })
        .collect();
    assert!(
        rows.windows(2).all(|pair| pair[0] < pair[1]),
        "Shift+F10 lists every tab in order, the selected one checked (rows {rows:?}):\n{}",
        menu.text
    );
    assert_eq!(
        menu.text.matches("[x]").count(),
        1,
        "only the selected tab is checked:\n{}",
        menu.text
    );
    let end = frames.last().expect("a frame");
    assert!(
        find(end, "[ ] Tab").is_none(),
        "choosing a tab closes the menu:\n{}",
        end.text
    );
    let (column, row) = at(end, "Tab 10");
    assert!(
        column + 18 <= 100,
        "choosing the tenth tab brings it whole into view:\n{}",
        end.text
    );
    assert_eq!(
        glyph(end, 99, row),
        "»",
        "the bar still has tabs out of view, so the mark stays at its end:\n{}",
        end.text
    );
}

/// A root that records the segment each navigation of the trail names.
struct Navigation {
    element: Element,
    chosen: Arc<Mutex<Vec<String>>>,
}

impl RootComponent for Navigation {
    fn render(&self) -> Element {
        self.element.clone()
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        let Event::Custom(event) = event else {
            return EventResult::Ignored;
        };
        if event.name != "navigate" {
            return EventResult::Ignored;
        }
        let data: serde_json::Value = serde_json::from_slice(&event.data).expect("JSON data");
        self.chosen
            .lock()
            .unwrap()
            .push(data["segment_id"].as_str().unwrap_or_default().to_owned());
        EventResult::Consumed
    }
}

#[test]
#[serial_test::serial(theme)]
fn nav_006_the_breadcrumb_ellipsis_is_a_stop_that_opens_the_hidden_segments() {
    // Five segments of 12 cells: a 10-cell label and one cell of padding at
    // each side. In 40 cells the first and the last stay and the ellipsis
    // stands for the three between.
    let mut trail = builder::breadcrumb().show_icons(false).on_click("navigate");
    for n in 1..=5 {
        trail = trail.segment(
            BreadcrumbSegment::new(format!("s{n}"), format!("Segment {n:02}"), format!("/{n}"))
                .current(n == 5),
        );
    }
    let chosen = Arc::new(Mutex::new(Vec::new()));
    let frames = app_input::run_until(
        Navigation {
            element: boxed(40, 20, trail.build().auto_focus()),
            chosen: chosen.clone(),
        },
        (120, 30),
        vec![
            until("Segment 01", app_input::key(KeyCode::Home)),
            // Right from the first segment reaches the ellipsis, and Enter
            // on it opens the menu of the hidden segments.
            until("Segment 01", app_input::key(KeyCode::Right)),
            until("Segment 01", app_input::key(KeyCode::Enter)),
            // Down moves from the first hidden segment to the second, and
            // Enter chooses it.
            until("Segment 04", app_input::key(KeyCode::Down)),
            until("Segment 04", app_input::key(KeyCode::Enter)),
            until("Segment 01", None),
        ],
        WAIT,
    );
    let menu = frames
        .iter()
        .rev()
        .find(|frame| frame.text.contains("Segment 04"))
        .expect("a frame with the menu of hidden segments open");
    let (_, trail_row) = at(menu, "Segment 01");
    let rows: Vec<u16> = (2..=4)
        .map(|n| at(menu, &format!("Segment {n:02}")).1)
        .collect();
    assert!(
        trail_row < rows[0] && rows.windows(2).all(|pair| pair[0] < pair[1]),
        "the menu under the ellipsis lists the hidden segments in trail order (trail row {trail_row}, rows {rows:?}):\n{}",
        menu.text
    );
    assert_eq!(
        *chosen.lock().unwrap(),
        ["s3"],
        "choosing the second hidden segment acts as a click on it"
    );
    let end = frames.last().expect("a frame");
    assert!(
        find(end, "Segment 03").is_none(),
        "choosing a segment closes the menu:\n{}",
        end.text
    );
}
