//! charts-goldens mechanism, BAR-004 for the layout widgets: the tabs, the
//! accordion, the breadcrumb, the scroll view and the stack, on the debug
//! backend under the dark preset, against checked-in goldens of the text
//! grid plus a color digest at 80 by 24 and at 400 by 100. Each fills the
//! width the page allots, so the wide golden shows them at 400 columns.
//! Goldens live in `tests/snapshots/layout_widgets/<widget>_<size>.ansi`, or under
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
        .join("layout_widgets")
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
/// that show it: the tabs and the accordion focused.
fn widgets() -> Vec<(&'static str, Element, Vec<Until>)> {
    let rows = (1..=10)
        .map(|index| Element::text(format!("Scrollable row {index}")))
        .collect();
    vec![
        (
            "tabs",
            page(
                builder::tabs()
                    .tab("Preview", Element::text("Live preview"))
                    .tab("Source", Element::text("Public builder API"))
                    .build()
                    .auto_focus(),
            ),
            vec![wait("Live preview", None)],
        ),
        (
            "accordion",
            page(
                builder::simple_accordion(vec![
                    ("one", "Focused demo", "One primary state per page"),
                    ("two", "Variants", "Useful alternatives stay nearby"),
                ])
                .auto_focus(),
            ),
            vec![wait("Variants", None)],
        ),
        (
            "breadcrumb",
            page(builder::path_breadcrumb("/catalog/layout/widgets")),
            vec![wait("widgets", None)],
        ),
        (
            "scroll_view",
            page(
                builder::scroll_view()
                    .contents(rows)
                    .vertical_scroll(true)
                    .show_scrollbars(true)
                    .class("h-7")
                    .build(),
            ),
            vec![wait("Scrollable row 7", None)],
        ),
        (
            "stack",
            page(
                builder::stack()
                    .spacing(1.0)
                    .child(Element::text("Layer one"))
                    .child(Element::text("Layer two"))
                    .build(),
            ),
            vec![wait("Layer two", None)],
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_004_layout_widget_goldens_at_80_by_24_and_400_by_100() {
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
        "BAR-004: layout widget goldens: {mismatches:?}"
    );
}
