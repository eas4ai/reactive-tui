//! charts-goldens mechanism, BAR-004 for the overlays: the modal, the
//! popover (open), the toast and the confirmation, input, autocomplete,
//! progress and wizard dialogs, on the debug backend under the dark preset,
//! against checked-in goldens of the text grid plus a color digest at 80 by
//! 24 and at 400 by 100. Each box is sized by its content and placed on
//! the screen, so the wide golden shows where it sits in 400 columns.
//! Goldens live in `tests/snapshots/overlays/<overlay>_<size>.ansi`, or
//! under `REACTIVE_TUI_SNAPSHOTS` when the check points there; run with
//! `REGENERATE=1` to write them, review the diff, then commit.

mod common;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder::{self, specialized::WizardStep},
    component::{Element, FocusProps},
    core::geometry::Rect,
    event::types::{Event, KeyCode, KeyEvent},
    theme::{dark_theme, Theme},
    widgets::{
        dialog::{
            AutocompleteConfig, AutocompleteDialog, AutocompleteDialogOptions, DialogComponent,
            DialogId, DialogTheme, InputDialog, InputDialogOptions,
        },
        display::modal::PRIMARY_BUTTON,
    },
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
        .join("overlays")
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

/// A page in the theme's background with a line of text, which a veil
/// dims and a box covers.
fn page(child: Element) -> Element {
    builder::div()
        .class("relative w-full h-full bg-background text-foreground")
        .child(Element::text(
            "Widget catalog · a line of the page under the overlay",
        ))
        .child(child)
        .build()
}

/// Each overlay by name, as the widget catalog builds it, with the steps
/// that show it whole.
fn overlays() -> Vec<(&'static str, Element, Vec<Until>)> {
    vec![
        (
            "modal",
            page(
                builder::modal()
                    .title("Modal")
                    .content(Element::text("Focused overlay · Escape closes"))
                    .visible(true)
                    .build(),
            ),
            vec![wait("Escape closes", None)],
        ),
        (
            "popover",
            page(
                builder::popover()
                    .trigger(
                        builder::div()
                            .class(PRIMARY_BUTTON)
                            .text("Open popover")
                            .build()
                            .with_focus(FocusProps::button())
                            .auto_focus(),
                    )
                    .content(Element::text("Popover content · Escape closes"))
                    .build(),
            ),
            vec![
                wait("Open popover", key(KeyCode::Enter)),
                wait("Popover content", None),
            ],
        ),
        (
            "confirmation_dialog",
            page(
                builder::confirmation_dialog()
                    .title("ConfirmationDialog")
                    .message("Ready to record?")
                    .build(),
            ),
            vec![wait("Ready to record?", None)],
        ),
        (
            "input_dialog",
            page(
                InputDialog::new(
                    DialogId::from_u32(7),
                    InputDialogOptions {
                        title: "InputDialog".into(),
                        prompt: "Capture name".into(),
                        ..Default::default()
                    },
                )
                .render(Rect::default(), &DialogTheme::default()),
            ),
            vec![wait("Capture name", None)],
        ),
        (
            "autocomplete_dialog",
            page(
                AutocompleteDialog::new(
                    DialogId::from_u32(8),
                    AutocompleteDialogOptions {
                        title: "AutocompleteDialog".into(),
                        prompt: "Find a widget".into(),
                        autocomplete: AutocompleteConfig {
                            min_chars: 0,
                            static_suggestions: vec![
                                "Accordion".into(),
                                "Checkbox".into(),
                                "Slider".into(),
                                "Tabs".into(),
                            ],
                            debounce_delay: std::time::Duration::ZERO,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
                .render(Rect::default(), &DialogTheme::default()),
            ),
            vec![wait("Find a widget", None), wait("Accordion", None)],
        ),
        (
            "progress_dialog",
            page(
                builder::progress_dialog()
                    .title("ProgressDialog")
                    .message("Rendering")
                    .progress(0.64)
                    .build(),
            ),
            vec![wait("Rendering", None)],
        ),
        (
            "toast",
            page(
                builder::toast()
                    .success("Toast: capture saved")
                    .persistent()
                    .build(),
            ),
            vec![wait("capture saved", None)],
        ),
        (
            "wizard_dialog",
            page(
                builder::wizard()
                    .title("WizardDialog")
                    .step(WizardStep::new("Compose").content(Element::text("Choose widgets")))
                    .step(WizardStep::new("Capture").content(Element::text("Record clip")))
                    .build(),
            ),
            vec![wait("Choose widgets", None)],
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_004_overlay_goldens_at_80_by_24_and_400_by_100() {
    let before = Theme::active();
    Theme::set_active(dark_theme());
    let mut mismatches = Vec::new();
    for size in [(80u16, 24u16), (400u16, 100u16)] {
        for (overlay, root, steps) in overlays() {
            let frame = app_input::run_until_on_debug(
                Root(root),
                size,
                steps,
                std::time::Duration::from_secs(10),
            )
            .pop()
            .expect("a painted frame");
            let name = format!("{overlay}_{}x{}", size.0, size.1);
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
        "BAR-004: overlay goldens: {mismatches:?}"
    );
}
