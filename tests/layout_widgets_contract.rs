//! The layout widgets' contract (docs/spec/layout-widgets.md, NAV-001 to
//! NAV-004): the tabs, the accordion, the breadcrumb, the scroll view and
//! the stack under a theme whose roles all differ, so a cell's color names
//! its role; the size each fills; what happens when a tab bar, a trail or
//! a scroll view's content does not fit; and the keys behind every pointer
//! action. What each widget tells the screen reader is read by the
//! `nav_004_` unit tests beside the widgets.

mod common;

use std::time::Duration;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::types::{Event, KeyCode},
    theme::{Theme, ThemeVariables},
};

type Rgb = [u8; 3];

const WAIT: Duration = Duration::from_secs(5);

struct Control(Element);
impl RootComponent for Control {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

const ROLES: [&str; 25] = [
    "background",
    "surface",
    "foreground",
    "text-muted",
    "border",
    "input",
    "ring",
    "hover",
    "selection",
    "selection-foreground",
    "primary",
    "primary-foreground",
    "secondary",
    "secondary-foreground",
    "error",
    "error-foreground",
    "warning",
    "warning-foreground",
    "success",
    "success-foreground",
    "info",
    "info-foreground",
    "accent",
    "accent-foreground",
    "overlay",
];

fn probe_color(index: usize) -> Rgb {
    [
        30 + 8 * index as u8,
        200 - 6 * index as u8,
        90 + 5 * index as u8,
    ]
}

fn role(name: &str) -> Rgb {
    probe_color(
        ROLES
            .iter()
            .position(|role| *role == name)
            .unwrap_or_else(|| panic!("{name} is not a role of the probe theme")),
    )
}

fn hex(color: Rgb) -> String {
    let [r, g, b] = color;
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// A theme whose roles all differ, so a cell's color names its role.
fn probe() -> Theme {
    let mut variables = ThemeVariables::new();
    for (index, name) in ROLES.iter().enumerate() {
        variables = variables.set(Theme::color_variable(name), hex(probe_color(index)));
    }
    Theme::new("probe").with_variables(variables)
}

/// Makes `theme` the active theme until it is dropped, also when the test
/// fails in between.
struct Active(std::sync::Arc<Theme>);
impl Active {
    fn set(theme: Theme) -> Self {
        let before = Theme::active();
        Theme::set_active(theme);
        Self(before)
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        Theme::set_active((*self.0).clone());
    }
}

fn rgb(color: vt100::Color) -> Option<Rgb> {
    match color {
        vt100::Color::Rgb(r, g, b) => Some([r, g, b]),
        _ => None,
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

/// The glyph color and the background of the cell at (column, row).
fn cell_colors(frame: &Snapshot, column: u16, row: u16) -> (Option<Rgb>, Option<Rgb>) {
    let cell = frame.screen.cell(row, column).unwrap();
    (rgb(cell.fgcolor()), rgb(cell.bgcolor()))
}

/// The glyph color and the background of the cell where `needle` starts.
fn colors(frame: &Snapshot, needle: &str) -> (Option<Rgb>, Option<Rgb>) {
    let (column, row) = at(frame, needle);
    cell_colors(frame, column, row)
}

/// The glyph in the cell at (column, row).
fn glyph(frame: &Snapshot, column: u16, row: u16) -> String {
    frame
        .screen
        .cell(row, column)
        .map(|cell| cell.contents().to_string())
        .unwrap_or_default()
}

/// The last column of `row` that holds a glyph other than a space.
fn last_glyph_column(frame: &Snapshot, row: u16) -> Option<u16> {
    let (_, columns) = frame.screen.size();
    (0..columns)
        .rev()
        .find(|column| !glyph(frame, *column, row).trim().is_empty())
}

/// A page in the theme's background that holds `child`.
fn page(child: Element) -> Element {
    builder::div()
        .class("relative w-full h-full bg-background text-foreground")
        .child(child)
        .build()
}

/// The last frame of `root` at `size` once `text` is painted and the App
/// is idle.
fn shown(root: Element, size: (u16, u16), text: &'static str) -> Snapshot {
    app_input::run_until(
        Control(page(root)),
        size,
        vec![Until {
            text,
            cell: None,
            event: None,
        }],
        WAIT,
    )
    .pop()
    .unwrap()
}

/// Every frame `root` presents at `size` through `steps`: each step waits
/// until its text is painted and the App is idle, then sends its event;
/// the frames are picked by content.
fn shown_after(
    root: Element,
    size: (u16, u16),
    steps: Vec<(&'static str, Option<Event>)>,
) -> Vec<Snapshot> {
    let until = steps
        .into_iter()
        .map(|(text, event)| Until {
            text,
            cell: None,
            event,
        })
        .collect();
    app_input::run_until(Control(page(root)), size, until, WAIT)
}

/// The first frame that paints `needle`.
fn first_with<'a>(frames: &'a [Snapshot], needle: &str) -> &'a Snapshot {
    frames
        .iter()
        .find(|frame| find(frame, needle).is_some())
        .unwrap_or_else(|| panic!("no frame paints {needle:?}"))
}

fn last(frames: &[Snapshot]) -> &Snapshot {
    frames.last().expect("a frame")
}

fn key(code: KeyCode) -> Option<Event> {
    app_input::key(code)
}

/// A box of `width` cells and `height` rows on the page.
fn boxed(width: u16, height: u16, child: Element) -> Element {
    builder::div()
        .class(&format!("flex-col w-{width} h-{height}"))
        .child(child)
        .build()
}

/// Ten tabs of 20 cells each: an 18-cell label and one cell of padding at
/// each side.
fn ten_tabs() -> Element {
    let mut tabs = builder::tabs();
    for n in 1..=10 {
        let label = format!("{:<18}", format!("Tab {n:02}"));
        tabs = tabs.tab(&label, Element::text(format!("Panel {n:02}")));
    }
    tabs.build()
}

// ---------------------------------------------------------------- NAV-001

#[test]
#[serial_test::serial(theme)]
fn nav_001_a_tab_bar_paints_its_labels_by_role_and_shows_focus_by_color_alone() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::div()
            .class("flex-col gap-1 w-full")
            .child(
                builder::tabs()
                    .tab("Preview", Element::text("Live preview"))
                    .tab("Source", Element::text("Public builder API"))
                    .build()
                    .auto_focus(),
            )
            .child(
                builder::tabs()
                    .tab("Summary", Element::text("Totals"))
                    .tab("Details", Element::text("Every row"))
                    .build(),
            )
            .build(),
        (80, 12),
        "Totals",
    );
    assert_eq!(
        colors(&frame, "Preview"),
        (Some(role("selection-foreground")), Some(role("selection"))),
        "the tab with the focus is selection while the tabs hold the focus:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Source").0,
        Some(role("text-muted")),
        "a tab's label is text-muted:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Summary").0,
        Some(role("foreground")),
        "the selected tab's label is foreground when the tabs do not hold the focus:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Details").0,
        Some(role("text-muted")),
        "the other label is text-muted:\n{}",
        frame.text
    );
    for mark in ["▶", "→", "~"] {
        assert!(
            find(&frame, mark).is_none(),
            "focus and hover add no glyph ({mark}):\n{}",
            frame.text
        );
    }
}

// ---------------------------------------------------------------- NAV-002

#[test]
#[serial_test::serial(theme)]
fn nav_002_a_default_scroll_view_fills_its_box_and_a_segment_has_one_cell_of_padding() {
    let _theme = Active::set(probe());
    let rows = (1..=60)
        .map(|n| Element::text(format!("Scrollable row {n}")))
        .collect();
    let frame = shown(
        builder::div()
            .class("flex-col gap-1")
            .child(boxed(
                100,
                20,
                builder::scroll_view()
                    .contents(rows)
                    .vertical_scroll(true)
                    .show_scrollbars(true)
                    .build(),
            ))
            .child(boxed(
                100,
                1,
                builder::path_breadcrumb("/catalog/layout/widgets"),
            ))
            .build(),
        (240, 60),
        "widgets",
    );
    let (_, top) = at(&frame, "Scrollable row 1");
    assert!(
        find(&frame, "Scrollable row 20").is_some_and(|(_, row)| row == top + 19)
            && find(&frame, "Scrollable row 21").is_none(),
        "the scroll view shows the 20 rows its box holds:\n{}",
        frame.text
    );
    assert!(
        ["█", "░"].contains(&glyph(&frame, 99, top).as_str()),
        "the bar stands in the box's last column:\n{}",
        frame.text
    );
    let (column, _) = at(&frame, "Root");
    // One cell of padding, the two-cell home icon and its space.
    assert_eq!(
        column, 4,
        "a segment has one cell of padding before its icon:\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- NAV-003

#[test]
#[serial_test::serial(theme)]
fn nav_003_a_tab_bar_scrolls_to_keep_the_focused_tab_in_view() {
    let _theme = Active::set(probe());
    let frames = shown_after(
        boxed(100, 10, ten_tabs().auto_focus()),
        (240, 60),
        vec![("Panel 01", key(KeyCode::End)), ("Panel 10", None)],
    );
    let end = last(&frames);
    let (column, _) = at(end, "Tab 10");
    assert!(
        column + 18 <= 100,
        "End brings the last tab whole into view:\n{}",
        end.text
    );
    assert!(
        find(end, "Tab 01").is_none(),
        "the bar scrolled, so the first tab left the view:\n{}",
        end.text
    );
}

// ---------------------------------------------------------------- NAV-004

#[test]
#[serial_test::serial(theme)]
fn nav_004_delete_closes_a_tab_as_a_click_on_its_mark_does() {
    let _theme = Active::set(probe());
    let tabs = || {
        builder::tabs()
            .tab("Preview", Element::text("Live preview"))
            .tab("Source", Element::text("Public builder API"))
            .tab("Notes", Element::text("Release notes"))
            .closable(true)
            .build()
            .auto_focus()
    };
    // The close mark of the second tab: the first "✕" after "Source".
    let opened = shown(tabs(), (80, 8), "Live preview");
    let (source, row) = at(&opened, "Source");
    let mark = (source..80)
        .find(|column| glyph(&opened, *column, row) == "✕")
        .expect("the second tab's close mark");
    let frames = shown_after(
        tabs(),
        (80, 8),
        vec![
            ("Live preview", app_input::click(mark, row)),
            ("Live preview", key(KeyCode::Delete)),
            ("Notes", None),
        ],
    );
    let end = last(&frames);
    assert!(
        find(end, "Source").is_none(),
        "a click on its mark closed the second tab:\n{}",
        end.text
    );
    assert!(
        find(end, "Preview").is_none() && find(end, "Notes").is_some(),
        "Delete closed the focused tab:\n{}",
        end.text
    );
}
