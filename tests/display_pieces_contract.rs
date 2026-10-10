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

use common::app_input::{self, FramePredicate, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    theme::{Theme, ThemeVariables},
    widgets::display::pieces::icon::{SPINNER_ASCII_FRAMES, SPINNER_FRAMES},
};
use std::sync::{Arc, Mutex};
use std::time::Instant;

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

// shared helpers

type Rgb = [u8; 3];

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

/// The color the probe theme gives a role.
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
struct Active(Arc<Theme>);
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

/// The glyph in the cell at (column, row).
fn glyph(frame: &Snapshot, column: u16, row: u16) -> String {
    frame
        .screen
        .cell(row, column)
        .map(|cell| cell.contents().to_string())
        .unwrap_or_default()
}

/// The glyph color of the cell at (column, row).
fn fg(frame: &Snapshot, column: u16, row: u16) -> Option<Rgb> {
    frame
        .screen
        .cell(row, column)
        .and_then(|cell| rgb(cell.fgcolor()))
}

/// The background color of the cell at (column, row).
fn bg(frame: &Snapshot, column: u16, row: u16) -> Option<Rgb> {
    frame
        .screen
        .cell(row, column)
        .and_then(|cell| rgb(cell.bgcolor()))
}

/// The number of cells in `row` whose background is `color`.
fn bg_cells(frame: &Snapshot, row: u16, color: Rgb) -> usize {
    let (_, columns) = frame.screen.size();
    (0..columns)
        .filter(|column| bg(frame, *column, row) == Some(color))
        .count()
}

/// The columns of `row` whose glyph is not a space and whose color is `color`.
fn glyph_cols(frame: &Snapshot, row: u16, color: Rgb) -> Vec<u16> {
    let (_, columns) = frame.screen.size();
    (0..columns)
        .filter(|column| {
            glyph(frame, *column, row) != " " && fg(frame, *column, row) == Some(color)
        })
        .collect()
}

/// A box of `width` cells and `height` rows on the page.
fn boxed(width: u16, height: u16, child: Element) -> Element {
    builder::div()
        .class(&format!("flex-col w-{width} h-{height}"))
        .child(child)
        .build()
}

/// The first frame of `root` at `size` that `when` accepts, once the App is
/// idle. For content with no text of its own, such as a skeleton.
fn frame_where(
    root: Element,
    size: (u16, u16),
    when: impl Fn(&Snapshot) -> bool + Send + Sync + 'static,
) -> Snapshot {
    let step: FramePredicate = Box::new(when);
    app_input::run_when_frame(Control(page(root)), size, vec![(step, None)])
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
    widgets::display::pieces::badge::BadgeKind,
};

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

// ===== spinner, skeleton, shimmer =====

/// Whether `glyph` is a spinner frame, braille or ASCII.
fn is_spinner_frame(glyph: &str) -> bool {
    SPINNER_FRAMES.contains(&glyph) || SPINNER_ASCII_FRAMES.contains(&glyph)
}

/// The spinner's glyph in front of `label` in `frame`.
fn spinner_glyph(frame: &Snapshot, label: &str) -> String {
    let (column, row) = at(frame, label);
    glyph(frame, column - 2, row)
}

/// The spinner's glyph beside "Saving" in `frame`, if the label is painted.
fn saving_glyph(frame: &Snapshot) -> Option<String> {
    let (column, row) = find(frame, "Saving")?;
    Some(glyph(frame, column.checked_sub(2)?, row))
}

/// DIS-001: a spinner's glyph and its label are painted in `text-muted`.
#[test]
#[serial_test::serial(theme)]
fn dis_001_a_spinner_paints_its_glyph_and_label_in_text_muted() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::spinner().label("Saving").build(),
        (80, 24),
        "Saving",
    );
    let (column, row) = at(&frame, "Saving");
    assert!(
        is_spinner_frame(&spinner_glyph(&frame, "Saving")),
        "DIS-001: the glyph before the label is not a spinner frame:\n{}",
        frame.text
    );
    assert_eq!(
        fg(&frame, column, row),
        Some(role("text-muted")),
        "DIS-001: the spinner's label is not in text-muted"
    );
    assert_eq!(
        fg(&frame, column - 2, row),
        Some(role("text-muted")),
        "DIS-001: the spinner's glyph is not in text-muted"
    );
}

/// DIS-001: a skeleton's rows are painted in `border`.
#[test]
#[serial_test::serial(theme)]
fn dis_001_a_skeleton_row_is_painted_in_border() {
    let _theme = Active::set(probe());
    let frame = frame_where(
        builder::skeleton().rows(1).animated(false).build(),
        (80, 24),
        |s| bg(s, 0, 0) == Some(role("border")),
    );
    assert_eq!(
        bg(&frame, 0, 0),
        Some(role("border")),
        "DIS-001: a skeleton row is not painted in border"
    );
    assert_eq!(
        bg_cells(&frame, 0, role("border")),
        80,
        "DIS-001: the skeleton row is not border across the whole width"
    );
}

/// DIS-001: a shimmer's text is `text-muted`, and its band is `foreground`
/// over the text while it moves.
#[test]
#[serial_test::serial(theme)]
fn dis_001_a_shimmer_paints_its_text_muted_and_its_band_foreground() {
    let _theme = Active::set(probe());
    let frame = frame_where(
        builder::shimmer("Loading the report").build(),
        (80, 24),
        |s| !glyph_cols(s, 0, role("foreground")).is_empty(),
    );
    assert!(
        !glyph_cols(&frame, 0, role("foreground")).is_empty(),
        "DIS-001: the band is not foreground over the text"
    );
    assert!(
        !glyph_cols(&frame, 0, role("text-muted")).is_empty(),
        "DIS-001: the text outside the band is not text-muted"
    );
}

/// DIS-002: a spinner takes the width of its glyph and label and no more.
#[test]
#[serial_test::serial(theme)]
fn dis_002_a_spinner_takes_the_width_of_its_content() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::spinner()
            .label("Saving")
            .class("bg-accent")
            .build(),
        (80, 24),
        "Saving",
    );
    // One glyph, one space and "Saving".
    assert_eq!(
        bg_cells(&frame, 0, role("accent")),
        8,
        "DIS-002: the spinner's box is not as wide as its content:\n{}",
        frame.text
    );
}

/// DIS-002: a shimmer takes the width of its text and no more.
#[test]
#[serial_test::serial(theme)]
fn dis_002_a_shimmer_takes_the_width_of_its_text() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::shimmer("Loading the report")
            .class("bg-accent")
            .animated(false)
            .build(),
        (80, 24),
        "Loading the report",
    );
    assert_eq!(
        bg_cells(&frame, 0, role("accent")),
        "Loading the report".chars().count(),
        "DIS-002: the shimmer's box is not as wide as its text:\n{}",
        frame.text
    );
}

/// DIS-002: a skeleton fills the 100 cells of the box it sits in.
#[test]
#[serial_test::serial(theme)]
fn dis_002_a_skeleton_fills_100_cells_of_a_box_in_a_240_by_60_viewport() {
    let _theme = Active::set(probe());
    let frame = frame_where(
        boxed(100, 10, builder::skeleton().rows(2).animated(false).build()),
        (240, 60),
        |s| bg_cells(s, 0, role("border")) > 0,
    );
    assert_eq!(
        bg_cells(&frame, 0, role("border")),
        100,
        "DIS-002: the skeleton does not fill the box's 100 cells"
    );
}

/// DIS-003: the spinner changes its glyph within 300 ms of the first frame
/// that shows the first glyph.
#[test]
#[serial_test::serial(theme)]
fn dis_003_a_spinner_changes_its_glyph_within_300_ms() {
    let _theme = Active::set(probe());
    // One predicate sees every frame: it records when the first glyph shows
    // and when another frame's glyph shows, and holds once both are seen.
    let first_seen = Arc::new(Mutex::new(None::<Instant>));
    let second_seen = Arc::new(Mutex::new(None::<Instant>));
    let (first, second) = (first_seen.clone(), second_seen.clone());
    let first_glyph = SPINNER_FRAMES[0];
    let ascii_first = SPINNER_ASCII_FRAMES[0];
    let on_change: FramePredicate = Box::new(move |s| {
        let Some(glyph) = saving_glyph(s) else {
            return false;
        };
        if glyph == first_glyph || glyph == ascii_first {
            first.lock().unwrap().get_or_insert_with(Instant::now);
        } else if is_spinner_frame(&glyph) {
            second.lock().unwrap().get_or_insert_with(Instant::now);
        }
        first.lock().unwrap().is_some() && second.lock().unwrap().is_some()
    });
    app_input::run_when_frame(
        Control(page(builder::spinner().label("Saving").build())),
        (80, 24),
        vec![(on_change, None)],
    );
    let start = first_seen
        .lock()
        .unwrap()
        .expect("the first glyph is shown");
    let next = second_seen
        .lock()
        .unwrap()
        .expect("a second glyph is shown");
    let gap = next.duration_since(start);
    assert!(
        gap < Duration::from_millis(300),
        "DIS-003: the spinner's glyph changed {gap:?} after the first glyph, not within 300 ms"
    );
}

/// DIS-003: a spinner under reduced motion stands on its first frame.
#[test]
#[serial_test::serial(theme)]
fn dis_003_a_spinner_stands_on_one_glyph_under_reduced_motion() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::spinner()
            .label("Saving")
            .class("reduced-motion")
            .build(),
        (80, 24),
        "Saving",
    );
    let first_glyph = SPINNER_FRAMES[0];
    let ascii_first = SPINNER_ASCII_FRAMES[0];
    let glyph = spinner_glyph(&frame, "Saving");
    assert!(
        glyph == first_glyph || glyph == ascii_first,
        "DIS-003: a spinner under reduced motion paints {glyph:?}, not its first frame"
    );
}

/// DIS-003: a shimmer's band moves across its text, starting at its left.
#[test]
#[serial_test::serial(theme)]
fn dis_003_a_shimmer_band_moves_across_its_text() {
    let _theme = Active::set(probe());
    // One predicate sees every frame: the band has to reach the text's left
    // and later its middle before the run stops.
    let at_left = Arc::new(Mutex::new(false));
    let past_middle = Arc::new(Mutex::new(false));
    let (left_seen, middle_seen) = (at_left.clone(), past_middle.clone());
    let on_band: FramePredicate = Box::new(move |s| {
        let bright = glyph_cols(s, 0, role("foreground"));
        if bright.first().is_some_and(|column| *column <= 1) {
            *left_seen.lock().unwrap() = true;
        }
        if bright.iter().any(|column| *column >= 8) {
            *middle_seen.lock().unwrap() = true;
        }
        *left_seen.lock().unwrap() && *middle_seen.lock().unwrap()
    });
    app_input::run_when_frame(
        Control(page(builder::shimmer("Loading the report").build())),
        (80, 24),
        vec![(on_band, None)],
    );
    assert!(
        *at_left.lock().unwrap(),
        "DIS-003: the band never starts at the text's left"
    );
    assert!(
        *past_middle.lock().unwrap(),
        "DIS-003: the band never moves past the text's middle"
    );
}

/// DIS-003: a shimmer under reduced motion paints its text plain, in `text-muted`.
#[test]
#[serial_test::serial(theme)]
fn dis_003_a_shimmer_stays_plain_under_reduced_motion() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::shimmer("Loading the report")
            .animated(false)
            .build(),
        (80, 24),
        "Loading the report",
    );
    assert!(
        glyph_cols(&frame, 0, role("foreground")).is_empty(),
        "DIS-003: a shimmer under reduced motion still paints a band:\n{}",
        frame.text
    );
    assert_eq!(
        glyph_cols(&frame, 0, role("text-muted")).len(),
        "Loading the report"
            .chars()
            .filter(|c| !c.is_whitespace())
            .count(),
        "DIS-003: a shimmer under reduced motion does not paint its text plain"
    );
}

/// DIS-003: a skeleton pulses between `border` and `surface`.
#[test]
#[serial_test::serial(theme)]
fn dis_003_a_skeleton_pulses_between_border_and_surface() {
    let _theme = Active::set(probe());
    // One predicate sees every frame: the row has to show both colors.
    let showed_border = Arc::new(Mutex::new(false));
    let showed_surface = Arc::new(Mutex::new(false));
    let (border_seen, surface_seen) = (showed_border.clone(), showed_surface.clone());
    let on_pulse: FramePredicate = Box::new(move |s| {
        if bg(s, 0, 0) == Some(role("border")) {
            *border_seen.lock().unwrap() = true;
        }
        if bg(s, 0, 0) == Some(role("surface")) {
            *surface_seen.lock().unwrap() = true;
        }
        *border_seen.lock().unwrap() && *surface_seen.lock().unwrap()
    });
    app_input::run_when_frame(
        Control(page(builder::skeleton().rows(1).build())),
        (80, 24),
        vec![(on_pulse, None)],
    );
    assert!(
        *showed_border.lock().unwrap(),
        "DIS-003: the skeleton never shows border"
    );
    assert!(
        *showed_surface.lock().unwrap(),
        "DIS-003: the skeleton never shows surface"
    );
}

/// DIS-003: a skeleton under reduced motion stands still in `border`.
#[test]
#[serial_test::serial(theme)]
fn dis_003_a_skeleton_stands_still_in_border_under_reduced_motion() {
    let _theme = Active::set(probe());
    let frame = frame_where(
        builder::skeleton().rows(1).animated(false).build(),
        (80, 24),
        |s| bg(s, 0, 0) == Some(role("border")),
    );
    assert_eq!(
        bg(&frame, 0, 0),
        Some(role("border")),
        "DIS-003: a skeleton under reduced motion is not in border"
    );
}

// ===== empty, alert, pagination, stepper =====

mod interactive {
    use super::*;
    use reactive_tui::event::types::{Event, KeyCode};
    use reactive_tui::theme::{Theme, ThemeVariables};
    use reactive_tui::widgets::display::pieces::{alert::AlertKind, icon::Icon};
    use std::sync::{Arc, Mutex};

    type Rgb = [u8; 3];

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

    /// A theme whose roles all differ, so a cell's color names its role.
    fn probe() -> Theme {
        let mut variables = ThemeVariables::new();
        for (index, name) in ROLES.iter().enumerate() {
            let [r, g, b] = probe_color(index);
            variables = variables.set(
                Theme::color_variable(name),
                format!("#{r:02x}{g:02x}{b:02x}"),
            );
        }
        Theme::new("probe").with_variables(variables)
    }

    /// Makes `theme` the active theme until it is dropped.
    struct Active(Arc<Theme>);
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
    fn at(frame: &Snapshot, needle: &str) -> (u16, u16) {
        let (rows, columns) = frame.screen.size();
        let glyphs: Vec<String> = needle.chars().map(String::from).collect();
        (0..rows)
            .flat_map(|row| {
                (0..=columns.saturating_sub(glyphs.len() as u16)).map(move |c| (c, row))
            })
            .find(|(column, row)| {
                glyphs.iter().enumerate().all(|(offset, glyph)| {
                    frame
                        .screen
                        .cell(*row, column + offset as u16)
                        .is_some_and(|cell| cell.contents() == *glyph)
                })
            })
            .unwrap_or_else(|| panic!("{needle:?} is not painted:\n{}", frame.text))
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

    /// Runs `root` at `size` through `steps`: each step waits until its text
    /// is painted and the App is idle, then sends its event.
    /// A last step that only observes, so the final frame shows the effect
    /// of the step before it.
    fn run(
        root: Element,
        size: (u16, u16),
        steps: Vec<(&'static str, Option<Event>)>,
    ) -> Vec<Snapshot> {
        let observe = steps.last().map(|(text, _)| *text);
        let mut until: Vec<Until> = steps
            .into_iter()
            .map(|(text, event)| Until {
                text,
                cell: None,
                event,
            })
            .collect();
        if let Some(text) = observe {
            until.push(Until {
                text,
                cell: None,
                event: None,
            });
        }
        app_input::run_until(Control(page(root)), size, until, WAIT)
    }

    fn last(frames: &[Snapshot]) -> &Snapshot {
        frames.last().expect("a frame")
    }

    fn key(code: KeyCode) -> Option<Event> {
        app_input::key(code)
    }

    /// The text of a frame with every run of spaces made one space.
    fn collapsed(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// A box of `width` cells and 5 rows on the page, with `child` in it.
    fn boxed(width: u16, child: Element) -> Element {
        builder::div()
            .class(&format!("flex-col w-{width} h-5 bg-background"))
            .child(child)
            .build()
    }

    /// The arrows and the ellipsis as the catalog paints them (DIS-005).
    fn arrows() -> (&'static str, &'static str, &'static str) {
        (
            Icon::ChevronLeft.glyph(),
            Icon::ChevronRight.glyph(),
            Icon::Ellipsis.glyph(),
        )
    }

    // ---------------------------------------------------------- DIS-001

    /// DIS-001: an alert's bar and icon take its kind's color, its title
    /// `foreground`, its message `text-muted`, and its row `surface`.
    #[test]
    #[serial_test::serial(theme)]
    fn dis_001_an_alert_paints_its_bar_by_kind_and_its_text_by_role() {
        let _theme = Active::set(probe());
        for (kind, color) in [
            (AlertKind::Default, "text-muted"),
            (AlertKind::Info, "info"),
            (AlertKind::Success, "success"),
            (AlertKind::Warning, "warning"),
            (AlertKind::Error, "error"),
        ] {
            let frame = super::shown(
                builder::alert(kind)
                    .title("Disk full")
                    .message("Free some space")
                    .build(),
                (80, 10),
                "Free some space",
            );
            assert_eq!(
                colors(&frame, "▌").0,
                Some(role(color)),
                "DIS-001: the bar of a {kind:?} alert:\n{}",
                frame.text
            );
            assert_eq!(
                colors(&frame, "Disk full"),
                (Some(role("foreground")), Some(role("surface"))),
                "DIS-001: the title of a {kind:?} alert:\n{}",
                frame.text
            );
            assert_eq!(
                colors(&frame, "Free some space").0,
                Some(role("text-muted")),
                "DIS-001: the message of a {kind:?} alert:\n{}",
                frame.text
            );
        }
    }

    /// DIS-001: the stepper paints passed steps `success`, the current step
    /// `foreground` and the rest `text-muted`.
    #[test]
    #[serial_test::serial(theme)]
    fn dis_001_a_stepper_paints_passed_current_and_coming_steps_by_role() {
        let _theme = Active::set(probe());
        let frame = super::shown(
            builder::stepper()
                .step("Compose")
                .step("Capture")
                .step("Review")
                .current(2)
                .build(),
            (80, 6),
            "Capture",
        );
        assert_eq!(
            colors(&frame, Icon::Check.glyph()).0,
            Some(role("success")),
            "DIS-001: a passed step:\n{}",
            frame.text
        );
        assert_eq!(
            colors(&frame, Icon::Dot.glyph()).0,
            Some(role("foreground"))
        );
        assert_eq!(
            colors(&frame, Icon::Circle.glyph()).0,
            Some(role("text-muted"))
        );
    }

    /// DIS-001: the pagination bar's current page is `primary` with
    /// `primary-foreground` text, and its other pages are `foreground`.
    #[test]
    #[serial_test::serial(theme)]
    fn dis_001_a_pagination_bar_paints_its_current_page_on_primary() {
        let _theme = Active::set(probe());
        let frame = super::shown(
            builder::pagination().pages(20).current(5).build(),
            (80, 6),
            "5",
        );
        assert_eq!(
            colors(&frame, "5"),
            (Some(role("primary-foreground")), Some(role("primary"))),
            "DIS-001: the current page:\n{}",
            frame.text
        );
        assert_eq!(colors(&frame, "4").0, Some(role("foreground")));
    }

    // ---------------------------------------------------------- DIS-002

    /// DIS-002: an alert fills the box it sits in, and a `w-40` class sets 40
    /// cells instead.
    #[test]
    #[serial_test::serial(theme)]
    fn dis_002_an_alert_fills_its_box_and_a_width_class_sets_it() {
        let _theme = Active::set(probe());
        let filled = super::shown(
            boxed(
                100,
                builder::alert(AlertKind::Info).title("Disk full").build(),
            ),
            (120, 12),
            "Disk full",
        );
        let (_, row) = at(&filled, "Disk full");
        assert_eq!(
            cell_colors(&filled, 99, row).1,
            Some(role("surface")),
            "DIS-002: the alert stops short of its box:\n{}",
            filled.text
        );
        let sized = super::shown(
            boxed(
                100,
                builder::alert(AlertKind::Info)
                    .title("Disk full")
                    .class("w-40")
                    .build(),
            ),
            (120, 12),
            "Disk full",
        );
        let (_, row) = at(&sized, "Disk full");
        assert_eq!(cell_colors(&sized, 39, row).1, Some(role("surface")));
        assert_ne!(
            cell_colors(&sized, 40, row).1,
            Some(role("surface")),
            "DIS-002: `w-40` paints wider than 40 cells:\n{}",
            sized.text
        );
    }

    /// DIS-002: a pagination bar and a horizontal stepper fill the width
    /// their box allots.
    #[test]
    #[serial_test::serial(theme)]
    fn dis_002_a_pagination_bar_and_a_stepper_fill_their_width() {
        let _theme = Active::set(probe());
        let bar = super::shown(
            boxed(
                100,
                builder::pagination()
                    .pages(20)
                    .current(5)
                    .class("bg-surface")
                    .build(),
            ),
            (120, 12),
            "5",
        );
        let (_, row) = at(&bar, "5");
        assert_eq!(cell_colors(&bar, 99, row).1, Some(role("surface")));
        let steps = super::shown(
            boxed(
                100,
                builder::stepper()
                    .step("Compose")
                    .step("Capture")
                    .current(1)
                    .class("bg-surface")
                    .build(),
            ),
            (120, 12),
            "Capture",
        );
        let (_, row) = at(&steps, "Capture");
        assert_eq!(cell_colors(&steps, 99, row).1, Some(role("surface")));
    }

    // ---------------------------------------------------------- DIS-003

    /// DIS-003: the bar shows the ends and the window around the current
    /// page, with an ellipsis for each hidden run, and Right, End and a
    /// click move the page.
    #[test]
    fn dis_003_the_pagination_bar_shows_its_window_and_moves_by_keys() {
        let (left, right, ellipsis) = arrows();
        // The first frame is the bar at page 5; the last one follows End.
        let frames = run(
            builder::pagination()
                .pages(20)
                .current(5)
                .build()
                .auto_focus(),
            (80, 10),
            vec![
                (ellipsis, key(KeyCode::Right)),
                (ellipsis, key(KeyCode::End)),
            ],
        );
        let text = collapsed(&last(&frames).text);
        assert!(
            text.contains(&format!("{left} 1 {ellipsis} 17 18 19 20 {right}")),
            "DIS-003: End does not move to 20:\n{}",
            last(&frames).text
        );
        let first = collapsed(&frames[0].text);
        assert!(
            first.contains(&format!("{left} 1 {ellipsis} 4 5 6 {ellipsis} 20 {right}")),
            "DIS-003: the bar at page 5 is not `1 … 4 5 6 … 20`:\n{}",
            frames[0].text
        );
    }

    /// DIS-003: Right moves the page to 6, and Confirm reports 6 through
    /// `on_change`.
    #[test]
    fn dis_003_confirm_on_the_moved_page_reports_it_through_on_change() {
        let (_, _, ellipsis) = arrows();
        let reported = Arc::new(Mutex::new(Vec::new()));
        let record = reported.clone();
        let bar = builder::pagination()
            .pages(20)
            .current(5)
            .on_change(move |page| record.lock().unwrap().push(page))
            .build()
            .auto_focus();
        let frames = run(
            bar,
            (80, 10),
            vec![
                (ellipsis, key(KeyCode::Right)),
                (ellipsis, key(KeyCode::Enter)),
            ],
        );
        assert_eq!(*reported.lock().unwrap(), vec![6]);
        assert!(
            collapsed(&last(&frames).text).contains("5 6 7"),
            "DIS-003: Right does not show page 6 as the window's middle:\n{}",
            last(&frames).text
        );
    }

    /// DIS-003: Confirm on an ellipsis opens a menu of the pages it hides.
    #[test]
    fn dis_003_confirm_on_an_ellipsis_opens_a_menu_of_the_hidden_pages() {
        let (_, _, ellipsis) = arrows();
        let frames = run(
            builder::pagination()
                .pages(20)
                .current(5)
                .build()
                .auto_focus(),
            (80, 30),
            vec![
                (ellipsis, key(KeyCode::Down)),
                (ellipsis, key(KeyCode::Enter)),
                ("Page 7", None),
            ],
        );
        assert!(
            last(&frames).text.contains("Page 19"),
            "DIS-003: the trailing ellipsis opens no menu of the pages it hides:\n{}",
            last(&frames).text
        );
    }

    /// DIS-003: a stepper marks the passed step `✓`, the current `●` and the
    /// rest `○`, and with `on_change` set Right moves the focus and Confirm
    /// reports the focused step.
    #[test]
    fn dis_003_a_stepper_marks_its_steps_and_reports_the_focused_one() {
        let (check, dot, circle) = (Icon::Check.glyph(), Icon::Dot.glyph(), Icon::Circle.glyph());
        let reported = Arc::new(Mutex::new(Vec::new()));
        let record = reported.clone();
        let steps = builder::stepper()
            .step("Compose")
            .step("Capture")
            .step("Review")
            .current(2)
            .on_change(move |step| record.lock().unwrap().push(step))
            .build()
            .auto_focus();
        let frames = run(
            steps,
            (80, 6),
            vec![(circle, key(KeyCode::Right)), (circle, key(KeyCode::Enter))],
        );
        let marks = collapsed(&frames[0].text);
        let (a, b, c) = (
            marks.find(check).expect("the passed mark"),
            marks.find(dot).expect("the current mark"),
            marks.find(circle).expect("the coming mark"),
        );
        assert!(
            a < b && b < c,
            "DIS-003: the marks are out of order:\n{marks}"
        );
        assert_eq!(*reported.lock().unwrap(), vec![3]);
    }

    /// DIS-003: the alert's `[×]` closes it on Confirm and reports `on_close`.
    #[test]
    #[serial_test::serial(theme)]
    fn dis_003_the_alert_close_mark_closes_it_on_confirm() {
        let _theme = Active::set(probe());
        let closed = Arc::new(Mutex::new(0));
        let count = closed.clone();
        let frames = app_input::run_until_hidden(
            Control(page(
                builder::alert(AlertKind::Warning)
                    .title("Disk full")
                    .closable(true)
                    .on_close(move || *count.lock().unwrap() += 1)
                    .build(),
            )),
            (80, 10),
            vec![
                ("Disk full", key(KeyCode::Tab)),
                ("Disk full", key(KeyCode::Enter)),
            ],
            "Disk full",
        );
        assert_eq!(*closed.lock().unwrap(), 1);
        assert!(!last(&frames).text.contains("Disk full"));
    }

    /// DIS-003: an empty state's action runs on Confirm.
    #[test]
    fn dis_003_an_empty_states_action_runs_on_confirm() {
        let ran = Arc::new(Mutex::new(0));
        let count = ran.clone();
        let state = builder::empty()
            .title("No files")
            .description("Create one")
            .action("Add", move || *count.lock().unwrap() += 1)
            .build();
        run(
            state,
            (80, 10),
            vec![
                ("No files", key(KeyCode::Tab)),
                ("Add", key(KeyCode::Enter)),
            ],
        );
        assert_eq!(*ran.lock().unwrap(), 1);
    }

    // ---------------------------------------------------------- DIS-005

    /// DIS-005: the stepper's marks and the alert's kind icons are the
    /// catalog's glyphs, one cell each.
    #[test]
    fn dis_005_the_pieces_paint_their_marks_from_the_catalog() {
        for icon in [
            Icon::Check,
            Icon::Dot,
            Icon::Circle,
            Icon::Ellipsis,
            Icon::ChevronLeft,
            Icon::ChevronRight,
            Icon::Info,
            Icon::Warning,
            Icon::Error,
            Icon::Success,
        ] {
            assert_eq!(
                unicode_width::UnicodeWidthStr::width(icon.unicode()),
                1,
                "DIS-005: {icon:?} is not one cell"
            );
        }
    }
}

// ===== link =====

mod link {
    //! The link (DIS-001 to DIS-003): underlined in `text-accent`, in `ring`
    //! while it holds the focus, as wide as its text, opened by Confirm,
    //! Activate or a click, and written between OSC 8 sequences where the
    //! terminal takes hyperlinks.

    use super::*;
    use common::app_input::{click, key};
    use reactive_tui::{
        event::types::KeyCode,
        theme::{Theme, ThemeVariables},
        widgets::display::pieces::{Link, LinkProps},
    };
    use std::sync::{Arc, Mutex};

    type Rgb = [u8; 3];

    const URL: &str = "https://example.com/docs";

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

    /// A theme whose roles all differ, so a cell's color names its role.
    fn probe() -> Theme {
        let mut variables = ThemeVariables::new();
        for (index, name) in ROLES.iter().enumerate() {
            let [r, g, b] = probe_color(index);
            variables = variables.set(
                Theme::color_variable(name),
                format!("#{r:02x}{g:02x}{b:02x}"),
            );
        }
        Theme::new("probe").with_variables(variables)
    }

    /// Makes `theme` the active theme until it is dropped, also when the
    /// test fails in between.
    struct Active(Arc<Theme>);
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

    /// The cell where `needle` starts, as (column, row).
    fn at(frame: &Snapshot, needle: &str) -> (u16, u16) {
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
            .unwrap_or_else(|| panic!("{needle:?} is not painted:\n{}", frame.text))
    }

    /// The glyph color, the background and whether it is underlined, of
    /// each cell of `needle` where it is painted.
    fn looks(frame: &Snapshot, needle: &str) -> Vec<(Option<Rgb>, Option<Rgb>, bool)> {
        let rgb = |color| match color {
            vt100::Color::Rgb(r, g, b) => Some([r, g, b]),
            _ => None,
        };
        let (column, row) = at(frame, needle);
        (0..needle.chars().count() as u16)
            .map(|offset| {
                let cell = frame.screen.cell(row, column + offset).unwrap();
                (rgb(cell.fgcolor()), rgb(cell.bgcolor()), cell.underline())
            })
            .collect()
    }

    /// A link to `URL` that records each URL it is opened with.
    fn recorded(text: &str) -> (builder::LinkBuilder, Arc<Mutex<Vec<String>>>) {
        let opened = Arc::new(Mutex::new(Vec::new()));
        let sink = opened.clone();
        let link =
            builder::link(text, URL).on_open(move |url| sink.lock().unwrap().push(url.into()));
        (link, opened)
    }

    /// The output with every CSI sequence (colors, attributes, moves)
    /// removed, so the text and the OSC 8 sequences around it read in order.
    fn without_csi(output: &[u8]) -> String {
        let text = String::from_utf8_lossy(output);
        let mut out = String::new();
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' && chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&c) {
                        break;
                    }
                }
                continue;
            }
            out.push(c);
        }
        out
    }

    /// DIS-001: a link is underlined in `text-accent`, in `ring` while it
    /// holds the focus, the same from its props and its builder, and its
    /// focus and disabled states add no glyph.
    #[test]
    #[serial_test::serial(theme)]
    fn dis_001_a_link_is_underlined_in_accent_and_in_ring_while_focused() {
        let _theme = Active::set(probe());
        let frame = shown(
            builder::div()
                .class("flex-col w-full")
                .child(Element::typed::<Link>(LinkProps::new("Props", URL)))
                .child(builder::link("Builder", URL).build())
                .child(builder::link("Focused", URL).build().auto_focus())
                .child(builder::link("Disabled", URL).disabled(true).build())
                .build(),
            (60, 8),
            "Disabled",
        );
        let background = Some(role("background"));
        for text in ["Props", "Builder"] {
            for (offset, look) in looks(&frame, text).into_iter().enumerate() {
                assert_eq!(
                    look,
                    (Some(role("accent")), background, true),
                    "DIS-001: cell {offset} of the link {text:?} is not underlined text-accent:\n{}",
                    frame.text
                );
            }
        }
        for (offset, look) in looks(&frame, "Focused").into_iter().enumerate() {
            assert_eq!(
                look,
                (Some(role("ring")), background, true),
                "DIS-001: cell {offset} of the focused link is not underlined ring:\n{}",
                frame.text
            );
        }
        for (offset, look) in looks(&frame, "Disabled").into_iter().enumerate() {
            assert_eq!(
                look.0,
                Some(role("text-muted")),
                "DIS-001: cell {offset} of the disabled link is not text-muted:\n{}",
                frame.text
            );
        }
        let rows: Vec<&str> = frame.text.lines().map(str::trim_end).collect();
        assert_eq!(
            rows[..4],
            ["Props", "Builder", "Focused", "Disabled"],
            "DIS-001: a focused or disabled link paints a glyph besides its text:\n{}",
            frame.text
        );
    }

    /// DIS-002: a link takes the width of its text and no more, in a column
    /// that stretches its children; a click beside its text is not on it.
    #[test]
    fn dis_002_a_link_takes_the_width_of_its_text() {
        let (link, opened) = recorded("Docs");
        let frames = app_input::run_until(
            Control(page(
                builder::div()
                    .class("flex-col w-40")
                    .child(link.build())
                    .build(),
            )),
            (60, 4),
            vec![
                Until {
                    text: "Docs",
                    cell: None,
                    event: click(10, 0),
                },
                Until {
                    text: "Docs",
                    cell: None,
                    event: None,
                },
            ],
            WAIT,
        );
        let frame = frames.last().unwrap();
        assert_eq!(at(frame, "Docs"), (0, 0));
        // The page, its box, then the link, in preorder.
        let node = frame
            .geometry
            .iter()
            .find(|node| node.element_index == 2)
            .expect("the link's node");
        assert_eq!(
            (node.bounds.x, node.bounds.width),
            (0.0, 4.0),
            "DIS-002: the link of four cells spans other cells than its text"
        );
        assert!(
            opened.lock().unwrap().is_empty(),
            "DIS-002: a click six cells right of the link's text opened it"
        );
    }

    /// DIS-003: Confirm, Activate and a click each run `on_open` with the
    /// link's URL.
    #[test]
    fn dis_003_a_link_opens_on_confirm_activate_and_a_click() {
        let (link, opened) = recorded("Docs");
        app_input::run_until_on_debug_with_hyperlinks(
            Control(page(link.build().auto_focus())),
            (40, 4),
            vec![
                Until {
                    text: "Docs",
                    cell: None,
                    event: key(KeyCode::Enter),
                },
                Until {
                    text: "Docs",
                    cell: None,
                    event: key(KeyCode::Char(' ')),
                },
                Until {
                    text: "Docs",
                    cell: None,
                    event: click(1, 0),
                },
                Until {
                    text: "Docs",
                    cell: None,
                    event: None,
                },
            ],
            WAIT,
            true,
        );
        assert_eq!(
            *opened.lock().unwrap(),
            vec![URL; 3],
            "DIS-003: Confirm, Activate and a click each open the link with its URL"
        );
    }

    /// DIS-003: a disabled link takes no action on Confirm, Activate or a
    /// click.
    #[test]
    fn dis_003_a_disabled_link_takes_no_action() {
        let (link, opened) = recorded("Docs");
        app_input::run_until_on_debug_with_hyperlinks(
            Control(page(link.disabled(true).build().auto_focus())),
            (40, 4),
            vec![
                Until {
                    text: "Docs",
                    cell: None,
                    event: key(KeyCode::Enter),
                },
                Until {
                    text: "Docs",
                    cell: None,
                    event: key(KeyCode::Char(' ')),
                },
                Until {
                    text: "Docs",
                    cell: None,
                    event: click(1, 0),
                },
                Until {
                    text: "Docs",
                    cell: None,
                    event: None,
                },
            ],
            WAIT,
            true,
        );
        assert!(
            opened.lock().unwrap().is_empty(),
            "DIS-003: a disabled link ran on_open"
        );
    }

    /// The last frame's output of a link "Docs" beside plain text, on the
    /// debug backend reporting hyperlinks or not, with its CSI sequences
    /// removed.
    fn written(hyperlinks: bool) -> String {
        let frames = app_input::run_until_on_debug_with_hyperlinks(
            Control(page(
                builder::div()
                    .class("flex-row gap-1")
                    .child(builder::link("Docs", URL).build())
                    .child(Element::text("and more"))
                    .build(),
            )),
            (40, 4),
            vec![Until {
                text: "Docs and more",
                cell: None,
                event: None,
            }],
            WAIT,
            hyperlinks,
        );
        without_csi(&frames.last().unwrap().output)
    }

    /// DIS-003: with hyperlinks reported, the link's text is written
    /// between one `ESC ] 8 ; ; <url> ESC \` and one `ESC ] 8 ; ; ESC \`,
    /// and the text beside it outside them.
    #[test]
    fn dis_003_a_link_is_written_between_osc_8_sequences_where_hyperlinks_are_reported() {
        let output = written(true);
        let start = format!("\x1b]8;;{URL}\x1b\\");
        let end = "\x1b]8;;\x1b\\";
        assert!(
            output.contains(&format!("{start}Docs{end} and more")),
            "DIS-003: the link's text is not between the OSC 8 start and end: {output:?}"
        );
        assert_eq!(
            output.matches(start.as_str()).count(),
            1,
            "DIS-003: one link starts once: {output:?}"
        );
        assert_eq!(
            output.matches(end).count(),
            1,
            "DIS-003: one link ends once: {output:?}"
        );
    }

    /// DIS-003: without hyperlinks reported, no OSC 8 sequence is written.
    #[test]
    fn dis_003_a_link_is_written_plain_where_hyperlinks_are_not_reported() {
        let output = written(false);
        assert!(
            output.contains("Docs and more"),
            "the link's text is written: {output:?}"
        );
        assert!(
            !output.contains("\x1b]8;"),
            "DIS-003: an OSC 8 sequence is written although the terminal reports no hyperlinks: {output:?}"
        );
    }

    /// DIS-003: the default backend's terminal bytes carry the same OSC 8
    /// sequences around the link's text where the terminal takes
    /// hyperlinks.
    #[test]
    fn dis_003_the_default_backend_writes_the_link_between_osc_8_sequences() {
        let frames = app_input::run_until(
            Control(page(
                builder::div()
                    .class("flex-row gap-1")
                    .child(builder::link("Docs", URL).build())
                    .child(Element::text("and more"))
                    .build(),
            )),
            (40, 4),
            vec![Until {
                text: "Docs and more",
                cell: None,
                event: None,
            }],
            WAIT,
        );
        let output = without_csi(&frames.last().unwrap().output);
        let expected = format!("\x1b]8;;{URL}\x1b\\Docs\x1b]8;;\x1b\\ and more");
        assert!(
            output.contains(&expected),
            "DIS-003: the terminal bytes lack the OSC 8 start before the link's text or its end after it: {output:?}"
        );
    }
}
