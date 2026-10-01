//! charts-goldens mechanism, BAR-004 for the data widgets: the table, the
//! data table, the tree, the file explorer and the progress bar, on the
//! debug backend under the dark preset, against checked-in goldens of the
//! text grid plus a color digest at 80 by 24 and at 400 by 100. Each fills
//! the width the page allots, so the wide golden shows them at 400 columns.
//! Goldens live in `tests/snapshots/data_widgets/<widget>_<size>.ansi`, or under
//! `REACTIVE_TUI_SNAPSHOTS` when the check points there; run with
//! `REGENERATE=1` to write them, review the diff, then commit.

mod common;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::types::Event,
    theme::{dark_theme, Theme},
};

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn snapshots_dir() -> std::path::PathBuf {
    std::env::var_os("REACTIVE_TUI_SNAPSHOTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
        })
        .join("data_widgets")
}

/// Every cell of every row as text, then a digest of every cell's colors.
fn golden_bytes(frame: &Snapshot) -> Vec<u8> {
    let (rows, columns) = frame.screen.size();
    let mut hasher = common::digest::Digest::default();
    let mut grid = String::new();
    for row in 0..rows {
        for column in 0..columns {
            let Some(cell) = frame.screen.cell(row, column) else {
                continue;
            };
            hasher.field(format!("{:?}{:?}", cell.fgcolor(), cell.bgcolor()).as_bytes());
            if !cell.is_wide_continuation() {
                grid.push_str(if cell.contents().is_empty() {
                    " "
                } else {
                    cell.contents()
                });
            }
        }
        grid.push('\n');
    }
    format!("{grid}colors: {:016x}\n", hasher.finish()).into_bytes()
}

fn wait(text: &'static str, event: Option<Event>) -> Until {
    Until {
        text,
        cell: None,
        event,
    }
}

/// A page in the theme's background that holds `child` in a column of the
/// page's width, with a line of text under it.
fn page(child: Element) -> Element {
    builder::div()
        .class("w-full h-full bg-background text-foreground flex-col gap-1")
        .child(child)
        .child(Element::text(
            "Widget catalog · a line of the page under the widget",
        ))
        .build()
}

/// Each widget by name, as the widget catalog builds it, with the steps
/// that show it: the table focused, the tree with a selected node.
fn widgets() -> Vec<(&'static str, Element, Vec<Until>)> {
    use reactive_tui::widgets::display::table::{Table, TableColumn, TableProps, TableRow};
    use reactive_tui::widgets::display::TreeNode;
    let manual = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manual");
    vec![
        (
            "table",
            page(
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
                .auto_focus(),
            ),
            vec![wait("Layout", None)],
        ),
        (
            "data_table",
            page(
                builder::data_table()
                    .column("Widget", "widget")
                    .column("State", "state")
                    .simple_row(vec![("widget", "Input"), ("state", "Ready")])
                    .simple_row(vec![("widget", "Layout"), ("state", "Ready")])
                    .build(),
            ),
            vec![wait("Layout", None)],
        ),
        (
            "tree",
            page(
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
            ),
            vec![wait("manual", None)],
        ),
        (
            "file_explorer",
            page(
                builder::file_explorer()
                    .root_path(&manual)
                    .current_path(&manual)
                    .max_visible_items(5)
                    .show_preview(false)
                    .build(),
            ),
            vec![wait("files", None)],
        ),
        (
            "progress_bar",
            page(
                builder::progress_bar()
                    .label("Catalog coverage")
                    .value(82.0)
                    .show_percentage(true)
                    .build(),
            ),
            vec![wait("82.0%", None)],
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_004_data_widget_goldens_at_80_by_24_and_400_by_100() {
    let before = Theme::active();
    Theme::set_active(dark_theme());
    let mut mismatches = Vec::new();
    for size in [(80u16, 24u16), (400u16, 100u16)] {
        for (widget, root, steps) in widgets() {
            let frame = app_input::run_until_on_debug(
                Root(root),
                size,
                steps,
                std::time::Duration::from_secs(10),
            )
            .pop()
            .expect("a painted frame");
            let name = format!("{widget}_{}x{}", size.0, size.1);
            let path = snapshots_dir().join(format!("{name}.ansi"));
            let bytes = golden_bytes(&frame);
            if std::env::var("REGENERATE").as_deref() == Ok("1") {
                std::fs::create_dir_all(snapshots_dir()).expect("snapshot dir");
                std::fs::write(&path, &bytes).expect("write golden");
                continue;
            }
            match std::fs::read(&path) {
                Ok(expected) if expected == bytes => {}
                Ok(_) => mismatches.push(format!("golden mismatch for {name}")),
                Err(_) => mismatches.push(format!("{name}: no golden at {}", path.display())),
            }
        }
    }
    Theme::set_active((*before).clone());
    assert!(
        mismatches.is_empty(),
        "BAR-004: data widget goldens: {mismatches:?}"
    );
}
