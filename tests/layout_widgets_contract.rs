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

#[test]
#[serial_test::serial(theme)]
fn nav_001_an_accordion_paints_its_title_glyph_and_focused_header_by_role() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::simple_accordion(vec![
            ("one", "Focused demo", "One primary state per page"),
            ("two", "Variants", "Useful alternatives stay nearby"),
        ])
        .auto_focus(),
        (80, 12),
        "Variants",
    );
    assert_eq!(
        colors(&frame, "Focused demo"),
        (Some(role("selection-foreground")), Some(role("selection"))),
        "the header with the focus is selection while the accordion holds the focus:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Variants").0,
        Some(role("foreground")),
        "a title is foreground:\n{}",
        frame.text
    );
    let (column, row) = at(&frame, "Variants");
    assert_eq!(
        cell_colors(&frame, column + 9, row).0,
        Some(role("text-muted")),
        "the glyph after the title is text-muted:\n{}",
        frame.text
    );
    assert!(
        find(&frame, "▶").is_none(),
        "focus adds no glyph:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_001_a_disabled_tab_bar_paints_its_selected_label_muted_without_the_variant_fill() {
    use reactive_tui::widgets::layout::{Tab, TabVariant, TabsBuilder};
    let _theme = Active::set(probe());
    let frame = shown(
        TabsBuilder::new()
            .tab(Tab::new("Summary", Element::text("Totals")))
            .tab(Tab::new("Details", Element::text("Every row")))
            .variant(TabVariant::Solid)
            .disabled(true)
            .render(),
        (80, 8),
        "Totals",
    );
    assert_eq!(
        colors(&frame, "Summary").0,
        Some(role("text-muted")),
        "the selected tab of disabled tabs is text-muted (finding 1):\n{}",
        frame.text
    );
    assert_ne!(
        colors(&frame, "Summary").1,
        Some(role("primary")),
        "disabled tabs paint no variant fill (finding 1):\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_001_a_scroll_view_looks_the_same_from_its_props_and_its_builder() {
    use reactive_tui::widgets::layout::{ScrollView, ScrollViewProps};
    let _theme = Active::set(probe());
    let wide = || {
        Element::text("A row of fifty cells that is wider than the box holds")
            .with_class("whitespace-pre")
    };
    let from_props = shown(
        boxed(
            20,
            4,
            Element::typed::<ScrollView>(ScrollViewProps {
                content: wide(),
                ..Default::default()
            }),
        ),
        (80, 8),
        "A row of",
    );
    let from_builder = shown(
        boxed(20, 4, builder::scroll_view().content(wide()).build()),
        (80, 8),
        "A row of",
    );
    assert_eq!(
        from_props.text, from_builder.text,
        "a default scroll view paints the same from its props and its builder (finding 2)"
    );
    for row in 0..4 {
        for column in 0..20 {
            assert_eq!(
                cell_colors(&from_props, column, row),
                cell_colors(&from_builder, column, row),
                "cell ({column}, {row}) differs between the props and the builder (finding 2):\n{}\n{}",
                from_props.text,
                from_builder.text
            );
        }
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

#[test]
#[serial_test::serial(theme)]
fn nav_002_a_stack_fills_a_row_box_of_100_cells() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::div()
            .class("flex-row w-100 h-3")
            .child(
                builder::stack()
                    .child(
                        builder::div()
                            .class("w-full h-1 bg-primary")
                            .text("Layer one")
                            .build(),
                    )
                    .build(),
            )
            .build(),
        (240, 60),
        "Layer one",
    );
    let (_, row) = at(&frame, "Layer one");
    assert_eq!(
        cell_colors(&frame, 99, row).1,
        Some(role("primary")),
        "a stack in a row box fills its 100 cells (finding 3):\n{}",
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

/// A wheel event of `lines` lines at (x, y).
fn wheel(x: u16, y: u16, lines: f32) -> Option<Event> {
    use reactive_tui::event::types::{MouseEvent, Position, WheelDelta, WheelEvent, WheelPhase};
    Some(Event::Mouse(MouseEvent::wheel(
        Position::cell(x, y),
        WheelEvent {
            delta: WheelDelta::Lines { x: 0.0, y: lines },
            phase: WheelPhase::Changed,
        },
    )))
}

/// A left-button mouse event of `kind` at (x, y).
fn mouse(kind: reactive_tui::event::types::MouseEventKind, x: u16, y: u16) -> Option<Event> {
    use reactive_tui::event::types::{MouseButton, MouseEvent, Position};
    Some(Event::Mouse(
        MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left),
    ))
}

#[test]
#[serial_test::serial(theme)]
fn nav_003_a_scroll_view_follows_the_wheel_a_click_on_its_track_and_a_drag_of_its_thumb() {
    use reactive_tui::event::types::MouseEventKind;
    let _theme = Active::set(probe());
    let rows = (1..=60)
        .map(|index| Element::text(format!("Scrollable row {index}")))
        .collect();
    // The view stands 5 cells in and 2 rows down, 60 by 12, so its bar is
    // column 64 and its rows 2 to 13; the thumb is 3 cells (12 of 60 rows).
    let frames = shown_after(
        builder::div()
            .class("flex-col pl-5 pt-2")
            .child(boxed(
                60,
                12,
                builder::scroll_view()
                    .contents(rows)
                    .vertical_scroll(true)
                    .show_scrollbars(true)
                    .build(),
            ))
            .build(),
        (240, 60),
        vec![
            // One wheel line at speed 3 scrolls 3 rows.
            ("Scrollable row 1", wheel(30, 5, 1.0)),
            // A click on the track below the thumb scrolls one page: 12 rows.
            ("Scrollable row 4", mouse(MouseEventKind::Down, 64, 13)),
            // The thumb now starts 3 cells down (15 of 48, over 9 cells of
            // travel); a drag of 3 cells moves the content by 16 rows.
            ("Scrollable row 16", mouse(MouseEventKind::Down, 64, 5)),
            ("Scrollable row 16", mouse(MouseEventKind::Drag, 64, 8)),
            ("Scrollable row 32", mouse(MouseEventKind::Up, 64, 8)),
            ("Scrollable row 32", None),
        ],
    );
    let end = last(&frames);
    let top = find(end, "Scrollable row 32").map(|(_, row)| row);
    assert_eq!(
        top,
        Some(2),
        "the wheel, the track and the thumb each scrolled the view:\n{}",
        end.text
    );
    assert!(
        ["█", "░"].contains(&glyph(end, 64, 2).as_str()),
        "the bar stands in the box's last column:\n{}",
        end.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_003_a_vertical_tab_bar_scrolls_to_keep_the_focused_tab_in_view() {
    use reactive_tui::widgets::layout::{Tab, TabOrientation, TabsBuilder};
    let _theme = Active::set(probe());
    // A vertical bar above its panel: the root is a column, so nothing
    // stretches the bar to the box's height.
    let mut tabs = TabsBuilder::new().orientation(TabOrientation::Vertical);
    for n in 1..=10 {
        tabs = tabs.tab(Tab::new(
            format!("Tab {n:02}"),
            Element::text(format!("Panel {n:02}")),
        ));
    }
    let frames = shown_after(
        boxed(40, 5, tabs.render().auto_focus()),
        (240, 60),
        vec![("Tab 01", key(KeyCode::End)), ("Tab 10", None)],
    );
    let end = last(&frames);
    let (_, row) = at(end, "Tab 10");
    assert!(
        row < 5,
        "End brings the last tab of a vertical bar into the box's five rows (finding 4):\n{}",
        end.text
    );
    assert!(
        find(end, "Tab 01").is_none(),
        "the bar scrolled, so the first tab left the view (finding 4):\n{}",
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

#[test]
#[serial_test::serial(theme)]
fn nav_002_tabs_an_accordion_and_a_stack_fill_a_box_of_100_cells() {
    let _theme = Active::set(probe());
    let wide = |text: &str| {
        builder::div()
            .class("w-full h-1 bg-primary")
            .text(text)
            .build()
    };
    let frame = shown(
        builder::div()
            .class("flex-col gap-1")
            .child(boxed(
                100,
                3,
                builder::tabs()
                    .tab("Preview", wide("Live preview"))
                    .tab("Source", wide("Public builder API"))
                    .build(),
            ))
            .child(boxed(
                100,
                3,
                builder::accordion()
                    .section(
                        reactive_tui::widgets::layout::AccordionSection::new("one", "Totals")
                            .content(wide("Every row"))
                            .expanded(true),
                    )
                    .build(),
            ))
            .child(boxed(
                100,
                3,
                builder::stack()
                    .spacing(1.0)
                    .child(wide("Layer one"))
                    .child(wide("Layer two"))
                    .build(),
            ))
            .build(),
        (240, 60),
        "Layer two",
    );
    for needle in ["Live preview", "Every row", "Layer one", "Layer two"] {
        let (_, row) = at(&frame, needle);
        assert_eq!(
            cell_colors(&frame, 99, row).1,
            Some(role("primary")),
            "{needle}'s row fills the box's 100 cells:\n{}",
            frame.text
        );
    }
}

// ---------------------------------------------------------------- BAR-003

/// Whether `color` is one the probe theme defines.
fn of_the_theme(color: Option<Rgb>) -> bool {
    color.is_some_and(|color| (0..ROLES.len()).any(|index| probe_color(index) == color))
}

/// The colors in `frame` that the probe theme does not define, each with
/// the first cell that has it.
fn foreign(frame: &Snapshot) -> Vec<String> {
    let (rows, columns) = frame.screen.size();
    let mut found: Vec<(vt100::Color, String)> = Vec::new();
    for (row, column) in (0..rows).flat_map(|row| (0..columns).map(move |column| (row, column))) {
        let cell = frame.screen.cell(row, column).unwrap();
        // The second cell of a wide glyph carries no colors of its own: a
        // terminal paints it with the glyph's.
        if cell.is_wide_continuation() {
            continue;
        }
        let mut colors = vec![("background", cell.bgcolor())];
        if !cell.contents().trim().is_empty() {
            colors.push(("glyph", cell.fgcolor()));
        }
        for (part, color) in colors {
            if !of_the_theme(rgb(color)) && !found.iter().any(|(seen, _)| *seen == color) {
                found.push((
                    color,
                    format!(
                        "{part} {color:?} at ({column}, {row}) {:?}",
                        cell.contents()
                    ),
                ));
            }
        }
    }
    found.into_iter().map(|(_, where_)| where_).collect()
}

/// Each layout widget as the widget catalog builds it, focused or not,
/// with the text that shows it is painted.
fn widgets() -> Vec<(&'static str, Element, &'static str)> {
    use reactive_tui::widgets::layout::{
        AccordionSection, Tab, TabBadge, TabBadgeVariant, TabVariant, TabsBuilder,
    };
    let rows = || {
        (1..=30)
            .map(|index| Element::text(format!("Scrollable row {index}")))
            .collect()
    };
    vec![
        (
            "focused tabs with a badge and a disabled tab",
            TabsBuilder::new()
                .tab(Tab::new("Preview", Element::text("Live preview")))
                .tab(
                    Tab::new("Source", Element::text("Public builder API"))
                        .with_badge(TabBadge::new("3").with_variant(TabBadgeVariant::Error)),
                )
                .tab(Tab::new("Locked", Element::text("Nothing")).disabled(true))
                .closable(true)
                .render()
                .auto_focus(),
            "Live preview",
        ),
        (
            "solid tabs",
            TabsBuilder::new()
                .tab(Tab::new("Summary", Element::text("Totals")))
                .tab(Tab::new("Details", Element::text("Every row")))
                .variant(TabVariant::Solid)
                .render(),
            "Totals",
        ),
        (
            "soft tabs",
            TabsBuilder::new()
                .tab(Tab::new("Summary", Element::text("Totals")))
                .tab(Tab::new("Details", Element::text("Every row")))
                .variant(TabVariant::Soft)
                .render(),
            "Totals",
        ),
        (
            "focused accordion with a disabled section",
            builder::accordion()
                .section(
                    AccordionSection::new("one", "Focused demo")
                        .content(Element::text("One primary state per page"))
                        .expanded(true),
                )
                .section(
                    AccordionSection::new("two", "Locked")
                        .content(Element::text("Nothing"))
                        .disabled(true),
                )
                .build()
                .auto_focus(),
            "One primary state per page",
        ),
        (
            "breadcrumb",
            builder::path_breadcrumb("/catalog/layout/widgets"),
            "widgets",
        ),
        (
            "focused breadcrumb",
            builder::path_breadcrumb("/catalog/layout/widgets").auto_focus(),
            "widgets",
        ),
        (
            "scroll view with a bar",
            boxed(
                60,
                8,
                builder::scroll_view()
                    .contents(rows())
                    .vertical_scroll(true)
                    .show_scrollbars(true)
                    .build(),
            ),
            "Scrollable row 8",
        ),
        (
            "stack",
            builder::stack()
                .spacing(1.0)
                .child(Element::text("Layer one"))
                .child(Element::text("Layer two"))
                .build(),
            "Layer two",
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_a_layout_widget_takes_every_color_from_the_active_theme() {
    let _theme = Active::set(probe());
    let mut wrong = Vec::new();
    for (name, root, text) in widgets() {
        let frame = shown(root, (80, 24), text);
        let foreign = foreign(&frame);
        if !foreign.is_empty() {
            wrong.push(format!("{name}: {foreign:#?}\n{}", frame.text));
        }
    }
    assert!(
        wrong.is_empty(),
        "BAR-003: a layout widget paints a color the theme does not define:\n{}",
        wrong.join("\n")
    );
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_a_layout_widget_follows_a_resize() {
    use reactive_tui::event::types::ResizeEvent;
    let _theme = Active::set(probe());
    let rows = (1..=30)
        .map(|index| Element::text(format!("Scrollable row {index}")))
        .collect();
    let frames = shown_after(
        builder::div()
            .class("flex-col gap-1 w-full")
            .child(
                builder::tabs()
                    .tab(
                        "Preview",
                        builder::div()
                            .class("w-full h-1 bg-primary")
                            .text("Live preview")
                            .build(),
                    )
                    .build(),
            )
            .child(
                builder::simple_accordion(vec![("one", "Focused demo", "One primary state")])
                    .auto_focus(),
            )
            .child(
                builder::div()
                    .class("flex-col w-full h-6")
                    .child(
                        builder::scroll_view()
                            .contents(rows)
                            .vertical_scroll(true)
                            .show_scrollbars(true)
                            .build(),
                    )
                    .build(),
            )
            .build(),
        (80, 24),
        vec![
            (
                "Scrollable row 6",
                Some(Event::Resize(ResizeEvent::new(160, 48))),
            ),
            ("Scrollable row 6", None),
        ],
    );
    let before = frames
        .iter()
        .rev()
        .find(|frame| frame.screen.size().1 == 80)
        .expect("a frame at 80 columns");
    let after = last(&frames);
    for (frame, last_column) in [(before, 79u16), (after, 159u16)] {
        let width = frame.screen.size().1;
        let (_, row) = at(frame, "Live preview");
        assert_eq!(
            cell_colors(frame, last_column, row).1,
            Some(role("primary")),
            "the tabs' panel fills {width} columns:\n{}",
            frame.text
        );
        let (_, row) = at(frame, "Focused demo");
        assert_eq!(
            cell_colors(frame, last_column, row).1,
            Some(role("selection")),
            "the accordion's header fills {width} columns:\n{}",
            frame.text
        );
        let (_, row) = at(frame, "Scrollable row 1");
        assert!(
            ["█", "░"].contains(&glyph(frame, last_column, row).as_str()),
            "the scroll view's bar stands in column {last_column}:\n{}",
            frame.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_every_pointer_action_of_a_layout_widget_has_a_key() {
    let _theme = Active::set(probe());
    let rows = (1..=30)
        .map(|index| Element::text(format!("Scrollable row {index}")))
        .collect();
    let frames = shown_after(
        builder::div()
            .class("flex-col gap-1 w-full")
            .child(
                builder::tabs()
                    .tab("Preview", Element::text("Live preview"))
                    .tab("Source", Element::text("Public builder API"))
                    .build()
                    .auto_focus(),
            )
            .child(builder::simple_accordion(vec![
                ("one", "Focused demo", "One primary state per page"),
                ("two", "Variants", "Useful alternatives stay nearby"),
            ]))
            .child(boxed(
                60,
                6,
                builder::scroll_view()
                    .contents(rows)
                    .vertical_scroll(true)
                    .show_scrollbars(true)
                    .build(),
            ))
            .child(builder::path_breadcrumb("/catalog/layout/widgets"))
            .build(),
        (80, 24),
        vec![
            // Right activates the next tab, as a click on it does.
            ("Live preview", key(KeyCode::Right)),
            // Tab reaches the accordion; Down and Space open its second
            // section, as a click on the header does.
            ("Public builder API", key(KeyCode::Tab)),
            ("Public builder API", key(KeyCode::Down)),
            ("Public builder API", key(KeyCode::Char(' '))),
            // Tab reaches the scroll view; End scrolls to its end, as a
            // click at the bottom of its track does.
            ("Useful alternatives stay nearby", key(KeyCode::Tab)),
            ("Useful alternatives stay nearby", key(KeyCode::End)),
            // Tab reaches the breadcrumb, whose focus starts on the segment
            // before the current one; Left moves it back, as the pointer
            // would.
            ("Scrollable row 30", key(KeyCode::Tab)),
            ("Scrollable row 30", key(KeyCode::Left)),
            ("Scrollable row 30", None),
        ],
    );
    let end = last(&frames);
    assert!(
        find(end, "Public builder API").is_some() && find(end, "Live preview").is_none(),
        "Right activated the second tab:\n{}",
        end.text
    );
    assert!(
        find(end, "Useful alternatives stay nearby").is_some(),
        "Space opened the second section:\n{}",
        end.text
    );
    assert_eq!(
        colors(end, "catalog"),
        (Some(role("selection-foreground")), Some(role("selection"))),
        "Left moved the breadcrumb's focus to the earlier segment:\n{}",
        end.text
    );
    assert!(
        find(end, "Scrollable row 30").is_some(),
        "End scrolled the view to its end:\n{}",
        end.text
    );
}

// ---------------------------------------------------------------- BAR-005

/// The App's work per frame over `frames` frames after `event`, with the
/// work and present time of each.
fn max_work_ms_after(
    root: Element,
    size: (u16, u16),
    event: Option<Event>,
    frames: usize,
) -> (f64, String) {
    let out = app_input::run_on_debug(Control(page(root)), size, vec![(1, event), (frames, None)]);
    let measured: Vec<_> = out.iter().skip(1).collect();
    assert!(
        measured.len() >= frames - 1,
        "harness painted {} of {frames} frames",
        measured.len()
    );
    let split: Vec<String> = measured
        .iter()
        .map(|f| format!("{:.2}/{:.2}", f.work_ms, f.present_ms))
        .collect();
    (
        measured.iter().map(|f| f.work_ms).fold(0.0, f64::max),
        split.join(" "),
    )
}

/// BAR-005: an accordion whose section opens over two seconds keeps the
/// App's work per frame under 16.6 ms at 700 by 200.
#[test]
#[serial_test::serial(theme)]
fn bar_005_an_opening_accordion_stays_under_the_frame_budget_at_700_by_200() {
    use reactive_tui::widgets::layout::accordion::{
        AccordionAnimation, AccordionProps, AccordionSection,
    };
    let accordion = || {
        let props = AccordionProps {
            sections: vec![
                AccordionSection::new("one", "Focused demo").content(
                    Element::text((1..=60).map(|n| format!("Row {n}\n")).collect::<String>())
                        .with_class("whitespace-pre"),
                ),
                AccordionSection::new("two", "Variants")
                    .content(Element::text("Useful alternatives stay nearby")),
            ],
            animation: AccordionAnimation {
                duration: Duration::from_secs(2),
                ..Default::default()
            },
            ..Default::default()
        };
        Element::component_with_props("Accordion", props).auto_focus()
    };
    if cfg!(debug_assertions) {
        // The App's per-element cost is about ten times higher without
        // optimization, so the budget is only meaningful on the optimized
        // build, which is what the frame-budget mechanism runs.
        eprintln!("SKIP: the frame budget is measured on the optimized build");
        let (_, split) = max_work_ms_after(accordion(), (80, 24), key(KeyCode::Enter), 3);
        assert!(!split.is_empty());
        return;
    }
    let (ms, split) = max_work_ms_after(accordion(), (700, 200), key(KeyCode::Enter), 12);
    eprintln!("an opening accordion: work/present ms per frame at 700x200: {split}");
    assert!(
        ms < 16.6,
        "BAR-005: per-frame work exceeds 16.6 ms at 700x200: {ms:.2} ms ({split})"
    );
}
