//! The data widgets' contract (docs/spec/data-widgets.md, DAT-001 to
//! DAT-004): the table, the data table, the tree, the file explorer and
//! the progress bar under a theme whose roles all differ, so a cell's color
//! names its role; the width and height each fills; numeric sorting and a
//! revealed selection; and the keys behind every pointer action. What each
//! widget tells the screen reader is read by the `dat_004_` unit tests
//! beside the widgets.

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

use reactive_tui::widgets::display::table::{Table, TableColumn, TableProps, TableRow};
use reactive_tui::widgets::display::TreeNode;

/// The catalog's table: two columns, two rows, sortable.
fn catalog_table() -> Element {
    Table::with_props(TableProps {
        columns: vec![
            TableColumn::new("Widget", "widget"),
            TableColumn::new("State", "state"),
        ],
        rows: vec![
            TableRow::new("input")
                .with_cell("widget", "Input")
                .with_cell("state", "Ready"),
            TableRow::new("layout")
                .with_cell("widget", "Layout")
                .with_cell("state", "Ready"),
        ],
        sortable: true,
        ..Default::default()
    })
}

/// A table whose rows come as gamma, alpha, beta, with the numbers 2, 9
/// and 10 in the second column.
fn counted_table(sort_column: Option<usize>) -> Element {
    Table::with_props(TableProps {
        columns: vec![
            TableColumn::new("Name", "name"),
            TableColumn::new("Count", "count"),
        ],
        rows: vec![
            TableRow::new("c")
                .with_cell("name", "gamma")
                .with_cell("count", "2"),
            TableRow::new("a")
                .with_cell("name", "alpha")
                .with_cell("count", "9"),
            TableRow::new("b")
                .with_cell("name", "beta")
                .with_cell("count", "10"),
        ],
        sortable: true,
        sort_column,
        sort_ascending: true,
        ..Default::default()
    })
}

/// The directory the catalog's file explorer shows.
fn manual_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manual")
}

/// The rows of `names`, each found in `frame`, in the order they are painted.
fn painted_order(frame: &Snapshot, names: &[&str]) -> Vec<u16> {
    names.iter().map(|name| at(frame, name).1).collect()
}

// ---------------------------------------------------------------- DAT-001

#[test]
#[serial_test::serial(theme)]
fn dat_001_a_table_paints_its_box_border_header_and_cursor_row_by_role() {
    let _theme = Active::set(probe());
    let frame = shown(catalog_table().auto_focus(), (80, 10), "Layout");
    let (column, row) = at(&frame, "Widget");
    // The box's top border stands on the row above the header, its left
    // border one cell left of the header's first cell.
    assert_eq!(
        cell_colors(&frame, column - 1, row - 1),
        (Some(role("border")), Some(role("background"))),
        "the box's corner is border on the page's background, with no fill of its own:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, column, row),
        (Some(role("foreground")), Some(role("background"))),
        "the header's title is foreground on the page's background:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Input"),
        (Some(role("selection-foreground")), Some(role("selection"))),
        "the cursor row is selection while the table holds the focus:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Layout").0,
        Some(role("foreground")),
        "another row's text is foreground:\n{}",
        frame.text
    );
}

/// DAT-001: a data table paints the sorted column's mark in `primary`, as
/// the table does (its wrapper owns the sort).
#[test]
#[serial_test::serial(theme)]
fn dat_001_a_data_tables_sort_mark_is_primary() {
    use reactive_tui::widgets::display::DataTableProps;
    let _theme = Active::set(probe());
    let mut props = DataTableProps::new(
        vec![
            TableColumn::new("Name", "name"),
            TableColumn::new("Count", "count"),
        ],
        vec![
            TableRow::new("c")
                .with_cell("name", "gamma")
                .with_cell("count", "2"),
            TableRow::new("a")
                .with_cell("name", "alpha")
                .with_cell("count", "9"),
        ],
    );
    props.table_props.sort_column = Some(0);
    props.table_props.sort_ascending = true;
    let frame = shown(
        Element::typed::<reactive_tui::widgets::display::DataTable>(props),
        (80, 12),
        "gamma",
    );
    assert_eq!(
        colors(&frame, "↑").0,
        Some(role("primary")),
        "the sort mark is primary:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Name").0,
        Some(role("foreground")),
        "the sorted column's title is foreground:\n{}",
        frame.text
    );
    let rows = painted_order(&frame, &["alpha", "gamma"]);
    assert!(
        rows[0] < rows[1],
        "sorted ascending by name:\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- DAT-002

/// A table of three columns with short titles and cells.
fn three_column_table() -> Element {
    Table::with_props(TableProps {
        columns: vec![
            TableColumn::new("Widget", "widget"),
            TableColumn::new("State", "state"),
            TableColumn::new("Notes", "notes"),
        ],
        rows: vec![
            TableRow::new("input")
                .with_cell("widget", "Input")
                .with_cell("state", "Ready")
                .with_cell("notes", "Six controls"),
            TableRow::new("layout")
                .with_cell("widget", "Layout")
                .with_cell("state", "Ready")
                .with_cell("notes", "Five widgets"),
        ],
        ..Default::default()
    })
}

#[test]
#[serial_test::serial(theme)]
fn dat_002_a_default_table_of_three_columns_fits_a_box_of_100_cells() {
    let _theme = Active::set(probe());
    let frame = shown(boxed(100, 10, three_column_table()), (240, 60), "Layout");
    assert!(
        find(&frame, "State").is_some() && find(&frame, "Notes").is_some(),
        "all three column titles are painted in a 100-cell box:\n{}",
        frame.text
    );
    let (_, row) = at(&frame, "Input");
    assert!(
        !glyph(&frame, 99, row).trim().is_empty(),
        "the row ends at the box's last cell (its border):\n{}",
        frame.text
    );
}

/// DAT-002: no default caps a data table's rows: forty rows in a box tall
/// enough for them all are all painted, with no page controls.
#[test]
#[serial_test::serial(theme)]
fn dat_002_a_default_data_table_of_40_rows_shows_them_all_in_a_box_of_60_rows() {
    let _theme = Active::set(probe());
    let mut table = builder::data_table().column("Name", "name");
    for i in 0..40 {
        let name = format!("Row{i:02}");
        table = table.simple_row(vec![("name", name.as_str())]);
    }
    let frame = shown(boxed(100, 60, table.build()), (240, 70), "Row39");
    assert!(find(&frame, "Row00").is_some() && find(&frame, "Row39").is_some());
    assert!(
        find(&frame, "Prev").is_none(),
        "no page controls by default:\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- DAT-003

#[test]
#[serial_test::serial(theme)]
fn dat_003_a_column_of_numbers_sorts_by_its_numbers() {
    let _theme = Active::set(probe());
    let frame = shown(counted_table(Some(1)), (80, 10), "gamma");
    let rows = painted_order(&frame, &["gamma", "alpha", "beta"]);
    assert!(
        rows[0] < rows[1] && rows[1] < rows[2],
        "2, 9, 10 sort ascending as numbers (gamma, alpha, beta):\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- DAT-004

#[test]
#[serial_test::serial(theme)]
fn dat_004_s_sorts_by_the_current_column_as_a_click_on_its_header_does() {
    let _theme = Active::set(probe());
    let frames = shown_after(
        counted_table(None).auto_focus(),
        (80, 10),
        vec![("gamma", key(KeyCode::Char('s'))), ("gamma", None)],
    );
    let end = last(&frames);
    let rows = painted_order(end, &["alpha", "beta", "gamma"]);
    assert!(
        rows[0] < rows[1] && rows[1] < rows[2],
        "s sorts by the first column ascending (alpha, beta, gamma):\n{}",
        end.text
    );
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

/// Each data widget as the widget catalog builds it, focused or not, with
/// the text that shows it is painted.
fn widgets() -> Vec<(&'static str, Element, &'static str)> {
    let manual = manual_dir();
    vec![
        ("focused table", catalog_table().auto_focus(), "Layout"),
        (
            "data table",
            builder::data_table()
                .column("Widget", "widget")
                .column("State", "state")
                .simple_row(vec![("widget", "Input"), ("state", "Ready")])
                .simple_row(vec![("widget", "Layout"), ("state", "Ready")])
                .build(),
            "Layout",
        ),
        (
            "tree with a selected node",
            builder::tree()
                .root(
                    TreeNode::new("root", "reactive-tui")
                        .expanded(true)
                        .add_child(TreeNode::new("src", "src").selected(true))
                        .add_child(TreeNode::new("manual", "manual")),
                )
                .show_lines(true)
                .show_icons(true)
                .build(),
            "manual",
        ),
        (
            "file explorer",
            builder::file_explorer()
                .root_path(&manual)
                .current_path(&manual)
                .max_visible_items(5)
                .show_preview(false)
                .build(),
            "files",
        ),
        (
            "progress bar",
            builder::progress_bar()
                .label("Catalog coverage")
                .value(82.0)
                .show_percentage(true)
                .build(),
            "82.0%",
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_a_data_widget_takes_every_color_from_the_active_theme() {
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
        "BAR-003: a data widget paints a color the theme does not define:\n{}",
        wrong.join("\n")
    );
}

// ---------------------------------------------------------------- BAR-005

/// The App's work per frame over `frames` frames, with the work and
/// present time of each.
fn max_work_ms(root: Element, size: (u16, u16), frames: usize) -> (f64, String) {
    let out = app_input::run_on_debug(Control(page(root)), size, vec![(frames, None)]);
    assert!(
        out.len() >= frames,
        "harness painted {} of {frames} frames",
        out.len()
    );
    let split: Vec<String> = out
        .iter()
        .map(|f| format!("{:.2}/{:.2}", f.work_ms, f.present_ms))
        .collect();
    (
        out.iter().map(|f| f.work_ms).fold(0.0, f64::max),
        split.join(" "),
    )
}

/// BAR-005: an indeterminate progress bar, whose marker moves every frame,
/// keeps the App's work per frame under 16.6 ms at 700 by 200.
#[test]
#[serial_test::serial(theme)]
fn bar_005_an_indeterminate_progress_bar_stays_under_the_frame_budget_at_700_by_200() {
    use reactive_tui::widgets::display::progress_bar::ProgressBarBuilder;
    let bar = || {
        ProgressBarBuilder::new()
            .label("Rendering")
            .indeterminate(true)
            .render()
    };
    if cfg!(debug_assertions) {
        // The App's per-element cost is about ten times higher without
        // optimization, so the budget is only meaningful on the optimized
        // build, which is what the frame-budget mechanism runs.
        eprintln!("SKIP: the frame budget is measured on the optimized build");
        let (_, split) = max_work_ms(bar(), (80, 24), 3);
        assert!(!split.is_empty());
        return;
    }
    let (ms, split) = max_work_ms(bar(), (700, 200), 12);
    eprintln!("an indeterminate progress bar: work/present ms per frame at 700x200: {split}");
    assert!(
        ms < 16.6,
        "BAR-005: per-frame work exceeds 16.6 ms at 700x200: {ms:.2} ms ({split})"
    );
}
