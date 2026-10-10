//! charts-goldens mechanism, BAR-004 for the input widgets: the text input,
//! the checkbox, the radio button, the select (open), the slider and the
//! button, on the debug backend under the dark preset, against checked-in
//! goldens of the text grid plus a color digest at 80 by 24 and at 400 by
//! 100. A field, a select's row and a slider's track fill the width the
//! page allots, so the wide golden shows them at 400 columns. Goldens live
//! in `tests/snapshots/input_widgets/<widget>_<size>.ansi`, or under
//! `REACTIVE_TUI_SNAPSHOTS` when the check points there; run with
//! `REGENERATE=1` to write them, review the diff, then commit.

mod common;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::types::{Event, KeyCode, KeyEvent},
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
        .join("input_widgets")
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

fn key(code: KeyCode) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(code)))
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
            "Widget catalog · a line of the page under the control",
        ))
        .build()
}

/// Each control by name, as the widget catalog builds it, with the steps
/// that show it: the text input and the select focused, the select open.
fn controls() -> Vec<(&'static str, Element, Vec<Until>)> {
    vec![
        (
            "text_input",
            page(
                builder::text_input()
                    .value("Reactive TUI")
                    .placeholder("Type here")
                    .build()
                    .auto_focus(),
            ),
            vec![wait("Reactive TUI", None)],
        ),
        (
            "checkbox",
            page(
                builder::checkbox()
                    .label("Capture-ready")
                    .checked(true)
                    .build(),
            ),
            vec![wait("Capture-ready", None)],
        ),
        (
            "radio_button",
            page(
                builder::div()
                    .class("flex-col")
                    .child(
                        builder::radio_button()
                            .group("quality")
                            .value("balanced")
                            .label("Balanced")
                            .checked(true)
                            .build(),
                    )
                    .child(
                        builder::radio_button()
                            .group("quality")
                            .value("high")
                            .label("High detail")
                            .build(),
                    )
                    .build(),
            ),
            vec![wait("High detail", None)],
        ),
        (
            "select",
            page(
                builder::select()
                    .option("cyan", "Cyan")
                    .option("violet", "Violet")
                    .selected("cyan")
                    .build()
                    .auto_focus(),
            ),
            vec![wait("Cyan", key(KeyCode::Enter)), wait("Violet", None)],
        ),
        (
            "slider",
            page(
                builder::slider()
                    .label("Intensity")
                    .min(0.0)
                    .max(100.0)
                    .step(5.0)
                    .value(65.0)
                    .build(),
            ),
            vec![wait("65.0", None)],
        ),
        (
            "button",
            page(
                builder::div()
                    .class("flex-row gap-1")
                    .child(builder::primary_button("Save", || {}))
                    .child(builder::button().text("Cancel").on_click(|| {}).build())
                    .build(),
            ),
            vec![wait("Cancel", None)],
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_004_input_widget_goldens_at_80_by_24_and_400_by_100() {
    let before = Theme::active();
    Theme::set_active(dark_theme());
    let mut mismatches = Vec::new();
    for size in [(80u16, 24u16), (400u16, 100u16)] {
        for (widget, root, steps) in controls() {
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
        "BAR-004: input widget goldens: {mismatches:?}"
    );
}
