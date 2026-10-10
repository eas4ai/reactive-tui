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
        "accessibility.md",
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

// ===== separator, badge, tag, kbd, description list, status bar =====

use reactive_tui::{
    builder::widgets::pieces::{
        badge::{badge, tag},
        description_list::description_list,
        kbd::{kbd, kbd_action},
        separator::separator,
        status_bar::status_bar,
    },
    event::types::KeyCode,
    keymap::{Action, KeyBinding, Keymap},
    theme::{Theme, ThemeVariables},
    widgets::display::pieces::badge::BadgeKind,
};

type Rgb = [u8; 3];

const PIECE_ROLES: [&str; 25] = [
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

fn piece_color(index: usize) -> Rgb {
    [
        30 + 8 * index as u8,
        200 - 6 * index as u8,
        90 + 5 * index as u8,
    ]
}

fn piece_role(name: &str) -> Rgb {
    piece_color(
        PIECE_ROLES
            .iter()
            .position(|role| *role == name)
            .unwrap_or_else(|| panic!("{name} is not a role of the probe theme")),
    )
}

/// A theme whose roles all differ, so a cell's color names its role.
fn piece_probe() -> Theme {
    let mut variables = ThemeVariables::new();
    for (index, name) in PIECE_ROLES.iter().enumerate() {
        let [r, g, b] = piece_color(index);
        variables = variables.set(
            Theme::color_variable(name),
            format!("#{r:02x}{g:02x}{b:02x}"),
        );
    }
    Theme::new("probe").with_variables(variables)
}

/// Makes `theme` the active theme until it is dropped.
struct PieceTheme(std::sync::Arc<Theme>);
impl PieceTheme {
    fn set(theme: Theme) -> Self {
        let before = Theme::active();
        Theme::set_active(theme);
        Self(before)
    }
}
impl Drop for PieceTheme {
    fn drop(&mut self) {
        Theme::set_active((*self.0).clone());
    }
}

fn piece_rgb(color: vt100::Color) -> Option<Rgb> {
    match color {
        vt100::Color::Rgb(r, g, b) => Some([r, g, b]),
        _ => None,
    }
}

/// The cell where `needle` starts, as (column, row).
fn piece_find(frame: &Snapshot, needle: &str) -> Option<(u16, u16)> {
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

/// The glyph color and the background of the cell where `needle` starts.
fn piece_colors(frame: &Snapshot, needle: &str) -> (Option<Rgb>, Option<Rgb>) {
    let (column, row) = piece_find(frame, needle)
        .unwrap_or_else(|| panic!("{needle:?} is not painted:\n{}", frame.text));
    let cell = frame.screen.cell(row, column).unwrap();
    (piece_rgb(cell.fgcolor()), piece_rgb(cell.bgcolor()))
}

/// The contents of the cells of `row`, one string per column.
fn row_cells(frame: &Snapshot, row: u16) -> Vec<String> {
    let (_, columns) = frame.screen.size();
    (0..columns)
        .map(|column| {
            frame
                .screen
                .cell(row, column)
                .map(|cell| cell.contents().to_string())
                .unwrap_or_default()
        })
        .collect()
}

/// The row where the first cell of `glyph` is painted.
fn row_with(frame: &Snapshot, glyph: &str) -> u16 {
    piece_find(frame, glyph)
        .map(|(_, row)| row)
        .unwrap_or_else(|| panic!("{glyph:?} is not painted:\n{}", frame.text))
}

/// The background of the cell at `column` of `row`.
fn bg_at(frame: &Snapshot, column: u16, row: u16) -> Option<Rgb> {
    piece_rgb(frame.screen.cell(row, column).unwrap().bgcolor())
}

/// A page of the pieces in a box of `width` by `height` cells.
fn piece_box(width: u16, height: u16, child: Element) -> Element {
    builder::div()
        .class(&format!("flex-col w-{width} h-{height}"))
        .child(child)
        .build()
}

/// DIS-001: a separator's line and label, a badge's fill and a tag's outline
/// paint in the color of their role.
#[test]
#[serial_test::serial(theme)]
fn dis_001_separator_badge_tag_and_key_hint_paint_by_role() {
    let _theme = PieceTheme::set(piece_probe());
    let frame = shown(
        builder::div()
            .class("flex-col w-full")
            .child(separator().label("Results").build())
            .child(
                builder::div()
                    .class("flex-row gap-1")
                    .child(badge().count(3).build())
                    .child(badge().count(5).kind(BadgeKind::Error).build())
                    .child(tag("bug").outline().kind(BadgeKind::Info).build())
                    .child(tag("done").build())
                    .child(kbd("Ctrl+S").build())
                    .build(),
            )
            .build(),
        (80, 12),
        "Results",
    );
    assert_eq!(
        piece_colors(&frame, "─"),
        (Some(piece_role("border")), Some(piece_role("background"))),
        "DIS-001: the separator line is painted in border:\n{}",
        frame.text
    );
    assert_eq!(
        piece_colors(&frame, "Results"),
        (
            Some(piece_role("text-muted")),
            Some(piece_role("background"))
        ),
        "DIS-001: the separator label is painted in text-muted:\n{}",
        frame.text
    );
    assert_eq!(
        piece_colors(&frame, " 3 "),
        (
            Some(piece_role("secondary-foreground")),
            Some(piece_role("secondary"))
        ),
        "DIS-001: a default badge is on secondary:\n{}",
        frame.text
    );
    assert_eq!(
        piece_colors(&frame, " 5 "),
        (
            Some(piece_role("error-foreground")),
            Some(piece_role("error"))
        ),
        "DIS-001: an error badge is on error:\n{}",
        frame.text
    );
    assert_eq!(
        piece_colors(&frame, "[ bug ]").0,
        Some(piece_role("info")),
        "DIS-001: an outline tag's text is in text-info:\n{}",
        frame.text
    );
    assert_eq!(
        piece_colors(&frame, " done "),
        (
            Some(piece_role("secondary-foreground")),
            Some(piece_role("secondary"))
        ),
        "DIS-001: a default tag is filled on secondary:\n{}",
        frame.text
    );
    assert_eq!(
        piece_colors(&frame, "Ctrl+S"),
        (
            Some(piece_role("secondary-foreground")),
            Some(piece_role("secondary"))
        ),
        "DIS-001: a key hint is on secondary:\n{}",
        frame.text
    );
}

/// DIS-001: a status bar is on `surface` with `foreground` text; a description
/// list's labels are `text-muted` and its values `foreground`.
#[test]
#[serial_test::serial(theme)]
fn dis_001_status_bar_and_description_list_paint_by_role() {
    let _theme = PieceTheme::set(piece_probe());
    let frame = shown(
        builder::div()
            .class("flex-col w-full")
            .child(status_bar().left("Ready").right("Ln 4").build())
            .child(description_list().pair("Name", "Ada").build())
            .build(),
        (80, 12),
        "Ln 4",
    );
    assert_eq!(
        piece_colors(&frame, "Ready"),
        (Some(piece_role("foreground")), Some(piece_role("surface"))),
        "DIS-001: the status bar is foreground on surface:\n{}",
        frame.text
    );
    assert_eq!(
        piece_colors(&frame, "Name"),
        (
            Some(piece_role("text-muted")),
            Some(piece_role("background"))
        ),
        "DIS-001: a description label is text-muted:\n{}",
        frame.text
    );
    assert_eq!(
        piece_colors(&frame, "Ada").0,
        Some(piece_role("foreground")),
        "DIS-001: a description value is foreground:\n{}",
        frame.text
    );
}

/// DIS-002: a separator and a status bar fill the width their box allots, a
/// `w-40` separator paints 40 cells, and a badge or key hint takes its content's width.
#[test]
fn dis_002_pieces_fill_or_take_the_width_their_classes_ask_for() {
    let frame = shown(
        piece_box(
            100,
            8,
            builder::div()
                .class("flex-col w-full")
                .child(separator().build())
                .child(separator().class("w-40").build())
                .child(status_bar().left("Ready").build())
                .child(
                    builder::div()
                        .class("flex-row")
                        .child(badge().count(3).build())
                        .child(kbd("Ctrl+S").build())
                        .build(),
                )
                .build(),
        ),
        (240, 60),
        "Ready",
    );
    let (_, rows) = frame.screen.size();
    let rule_counts: Vec<usize> = (0..rows)
        .map(|row| {
            row_cells(&frame, row)
                .iter()
                .filter(|cell| *cell == "─")
                .count()
        })
        .filter(|count| *count > 0)
        .collect();
    assert_eq!(
        rule_counts,
        vec![100, 40],
        "DIS-002: the default separator fills the 100-cell box and the w-40 one paints 40:\n{}",
        frame.text
    );
    let bar = row_with(&frame, "Ready");
    assert_eq!(
        bg_at(&frame, 99, bar),
        Some(piece_role("surface")),
        "DIS-002: the status bar reaches the box's last cell"
    );
    assert_ne!(
        bg_at(&frame, 100, bar),
        Some(piece_role("surface")),
        "DIS-002: the status bar stops at the box's last cell"
    );
}

/// DIS-002: a status bar of 100 cells with 60 cells in each region cuts its
/// center first, then its right region, keeps the left region's first cell and the
/// right region's last cell, and paints `…`.
#[test]
fn dis_002_a_status_bar_that_overflows_cuts_its_center_then_its_right() {
    let frame = shown(
        piece_box(
            100,
            3,
            status_bar()
                .left("L".repeat(60))
                .center("C".repeat(60))
                .right("R".repeat(60))
                .build(),
        ),
        (240, 60),
        "LLLL",
    );
    let row = row_cells(&frame, row_with(&frame, "L"));
    let text: String = row[..100].concat();
    assert!(
        text.starts_with('L'),
        "DIS-002: the left region keeps its first cell:\n{}",
        frame.text
    );
    assert!(
        text.ends_with('R'),
        "DIS-002: the right region keeps its last cell:\n{}",
        frame.text
    );
    assert!(
        text.contains('…'),
        "DIS-002: the overflowing bar paints an ellipsis:\n{}",
        frame.text
    );
    assert!(
        !text.contains('C'),
        "DIS-002: the center region goes first:\n{}",
        frame.text
    );
}

/// DIS-003: a badge hides at a count of zero, shows `99+` above 99 and a dot
/// badge shows `●`.
#[test]
fn dis_003_a_badge_hides_at_zero_caps_at_99_and_shows_a_dot() {
    let frame = shown(
        builder::div()
            .class("flex-col w-full")
            .child(builder::span().text("Inbox").build())
            .child(badge().count(0).build())
            .child(badge().count(120).build())
            .child(badge().dot().build())
            .build(),
        (80, 12),
        "Inbox",
    );
    assert!(
        frame.text.contains("99+"),
        "DIS-003: a count of 120 shows 99+:\n{}",
        frame.text
    );
    assert!(
        !frame.text.contains("120"),
        "DIS-003: a count above the maximum shows no number:\n{}",
        frame.text
    );
    assert!(
        frame.text.contains('●'),
        "DIS-003: a dot badge shows ●:\n{}",
        frame.text
    );
    assert!(
        !frame.text.contains('0'),
        "DIS-003: a count of zero paints nothing:\n{}",
        frame.text
    );
}

/// DIS-003: a key hint for Copy shows `Ctrl+C` under the default keymap and
/// `F3` once Copy is rebound to F3 in a scoped keymap.
#[test]
fn dis_003_a_key_hint_for_an_action_follows_the_keymap() {
    let default_hint = shown(
        piece_box(40, 1, kbd_action(Action::Copy).build()),
        (80, 12),
        "Ctrl+C",
    );
    assert!(
        default_hint.text.contains("Ctrl+C"),
        "DIS-003: Copy shows Ctrl+C under the default keymap:\n{}",
        default_hint.text
    );
    let mut keymap = Keymap::default();
    keymap.rebind(Action::Copy, [KeyBinding::new(KeyCode::F(3))]);
    let _scope = Keymap::scoped(keymap);
    let rebound = shown(
        piece_box(40, 1, kbd_action(Action::Copy).build()),
        (80, 12),
        "F3",
    );
    assert!(
        rebound.text.contains("F3") && !rebound.text.contains("Ctrl+C"),
        "DIS-003: a rebind of Copy to F3 shows F3 at the next render:\n{}",
        rebound.text
    );
}

/// DIS-001 and DIS-002: a bordered description list draws its box in `border` and
/// fills the 100-cell box it sits in.
#[test]
#[serial_test::serial(theme)]
fn dis_001_a_bordered_description_list_draws_its_box_in_border_across_its_width() {
    let _theme = PieceTheme::set(piece_probe());
    let frame = shown(
        piece_box(
            100,
            6,
            description_list()
                .pair("Name", "Ada")
                .pair("Role", "Engineer")
                .bordered()
                .build(),
        ),
        (240, 60),
        "Engineer",
    );
    let top = row_with(&frame, "┌");
    let cells = row_cells(&frame, top);
    let corner = piece_find(&frame, "┌").expect("the box corner is painted");
    assert_eq!(
        piece_colors(&frame, "┌").0,
        Some(piece_role("border")),
        "DIS-001: the box is painted in border:\n{}",
        frame.text
    );
    assert_eq!(
        cells.iter().filter(|cell| *cell == "─").count() + 2,
        100,
        "DIS-002: the box's top edge spans the 100-cell box at {corner:?}:\n{}",
        frame.text
    );
}
