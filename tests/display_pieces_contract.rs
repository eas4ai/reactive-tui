//! The display pieces' contract (docs/spec/display-pieces.md, DIS-001 to
//! DIS-006): the icon catalog, spinner, separator, badge and tag, key hint,
//! empty state, skeleton and shimmer, status bar, description list, inline
//! alert, link, pagination bar and stepper under a theme whose roles all
//! differ, so a cell's color names its role; the width each fills or takes;
//! their frames, counts, page math, keys and the link's OSC 8; and the
//! widgets that draw the shared pieces in place of their own. What each
//! piece tells the screen reader, and the glyph an icon paints, are read by
//! the `dis_004_` and `dis_005_` unit tests beside the widgets.

mod common;

use std::time::Duration;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{app::RootComponent, builder, component::Element};

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

/// A page the pieces are shown on: the full viewport in the page's roles.
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

fn manual_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manual")
}

/// Whether `text` holds an emoji or a glyph wider than one cell: a code
/// point in the supplementary planes, or a variation selector.
fn has_wide_glyph(text: &str) -> bool {
    text.chars()
        .any(|c| (c as u32) >= 0x1F000 || c == '\u{FE0F}')
}

/// DIS-005: the file explorer's file and folder marks come from the icon
/// catalog, single-width glyphs with an ASCII fallback, not emoji.
#[test]
fn dis_005_the_file_explorer_paints_no_emoji_for_files_and_folders() {
    let manual = manual_dir();
    let frame = shown(
        builder::file_explorer()
            .root_path(&manual)
            .current_path(&manual)
            .max_visible_items(8)
            .show_preview(false)
            .build(),
        (80, 24),
        "README.md",
    );
    assert!(
        !has_wide_glyph(&frame.text),
        "DIS-005: the file explorer paints an emoji or a two-cell glyph for a file or a folder:\n{}",
        frame.text
    );
}

/// DIS-006: the wizard dialog draws its step line through the stepper, the
/// current step marked `●` and the ones to come `○`, not "Step N of M".
#[test]
fn dis_006_the_wizard_draws_its_steps_through_the_stepper() {
    use reactive_tui::builder::specialized::WizardStep;
    let frame = shown(
        builder::wizard()
            .title("Wizard")
            .step(WizardStep::new("Compose").content(Element::text("Choose widgets")))
            .step(WizardStep::new("Capture").content(Element::text("Record clip")))
            .build(),
        (80, 24),
        "Choose widgets",
    );
    assert!(
        !frame.text.contains("Step 1 of 2"),
        "DIS-006: the wizard still paints its own step line:\n{}",
        frame.text
    );
    assert!(
        frame.text.contains('●') && frame.text.contains('○'),
        "DIS-006: the wizard paints no stepper marks for its current and coming steps:\n{}",
        frame.text
    );
}
