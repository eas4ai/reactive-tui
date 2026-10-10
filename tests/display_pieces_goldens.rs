//! charts-goldens mechanism, BAR-004 for the display pieces: the icon,
//! spinner, separator, badge, tag, key hint, empty state, skeleton, shimmer,
//! status bar, description list, alert, link, pagination bar and stepper,
//! on the debug backend under the dark preset, against checked-in goldens of
//! the text grid plus a color digest at 80 by 24 and at 400 by 100. Each
//! fills the width the page allots, so the wide golden shows them at 400
//! columns. The animated pieces are built with `.animated(false)`, so each
//! frame is the same every run. Goldens live in
//! `tests/snapshots/display_pieces/<piece>_<size>.ansi`, or under
//! `REACTIVE_TUI_SNAPSHOTS` when the check points there; run with
//! `REGENERATE=1` to write them, review the diff, then commit.

mod common;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder::{
        self,
        widgets::pieces::{
            badge::{badge, tag},
            description_list::description_list,
            kbd::{kbd, kbd_action},
            separator::separator,
            status_bar::status_bar,
        },
    },
    component::Element,
    event::types::Event,
    keymap::Action,
    theme::{dark_theme, Theme},
    widgets::display::pieces::{alert::AlertKind, badge::BadgeKind, icon::Icon},
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
        .join("display_pieces")
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

/// Each piece by name, as the widget catalog builds it, with the text that
/// the page shows once it is painted.
fn pieces() -> Vec<(&'static str, Element)> {
    vec![
        (
            "icon",
            builder::icon(Icon::Success).aria_label("Saved").build(),
        ),
        (
            "spinner",
            builder::spinner().label("Saving").animated(false).build(),
        ),
        ("separator", separator().label("Options").build()),
        ("badge", badge().kind(BadgeKind::Info).count(120).build()),
        (
            "tag",
            tag("done").kind(BadgeKind::Success).outline().build(),
        ),
        (
            "key_hint",
            div_row(vec![
                kbd("Ctrl+S").build(),
                kbd_action(Action::Copy).build(),
            ]),
        ),
        (
            "empty",
            builder::empty()
                .icon(Icon::Search)
                .title("No results")
                .description("Try another search term.")
                .action("Clear search", || {})
                .build(),
        ),
        (
            "skeleton",
            builder::skeleton().rows(3).animated(false).build(),
        ),
        (
            "shimmer",
            builder::shimmer("Loading the table")
                .animated(false)
                .build(),
        ),
        (
            "status_bar",
            status_bar()
                .left("Ready")
                .center("Saved 2 s ago")
                .right("Ln 4, Col 12")
                .build(),
        ),
        (
            "description_list",
            description_list()
                .pair("Name", "Ada")
                .pair("Role", "Engineer")
                .bordered()
                .build(),
        ),
        (
            "alert",
            builder::alert(AlertKind::Warning)
                .title("Disk almost full")
                .message("Free 2 GB to keep saving.")
                .closable(true)
                .build(),
        ),
        (
            "link",
            builder::link("Reactive TUI docs", "https://example.com/docs").build(),
        ),
        (
            "pagination",
            builder::pagination()
                .pages(20)
                .current(5)
                .visible_pages(5)
                .build(),
        ),
        (
            "stepper",
            builder::stepper()
                .step("Account")
                .step("Review")
                .step("Done")
                .current(1)
                .build(),
        ),
    ]
}

fn div_row(children: Vec<Element>) -> Element {
    builder::div()
        .class("flex-row gap-1")
        .children(children)
        .build()
}

#[test]
#[serial_test::serial(theme)]
fn bar_004_display_pieces_goldens_at_80_by_24_and_400_by_100() {
    let before = Theme::active();
    Theme::set_active(dark_theme());
    let mut mismatches = Vec::new();
    for size in [(80u16, 24u16), (400u16, 100u16)] {
        for (piece, root) in pieces() {
            let frame = app_input::run_until_on_debug(
                Root(page(root)),
                size,
                vec![wait("a line of the page under the widget", None)],
                std::time::Duration::from_secs(10),
            )
            .pop()
            .expect("a painted frame");
            let name = format!("{piece}_{}x{}", size.0, size.1);
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
        "BAR-004: display piece goldens: {mismatches:?}"
    );
}
