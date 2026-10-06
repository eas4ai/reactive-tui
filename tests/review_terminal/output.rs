//! Part of tests/review_terminal.rs: the legacy output types of src/core.

use reactive_tui::core::grapheme_cell::{CellType, GraphemeSurface, Span};
use reactive_tui::core::render_ops::RenderOpsBuilder;
use reactive_tui::core::span_diff::SpanDiffWriter;
use reactive_tui::core::surface::{Attr, Rgba};
use reactive_tui::core::writer::render_ops_to_ansi;
use unicode_width::UnicodeWidthStr;

/// A VT100 model of `rows` by `cols` cells that has interpreted `bytes`.
fn screen(bytes: &[u8], rows: u16, cols: u16) -> vt100::Parser {
    let mut parser = vt100::Parser::new(rows, cols, 0);
    parser.process(bytes);
    parser
}

fn row_text(parser: &vt100::Parser, row: u16, cols: u16) -> String {
    (0..cols)
        .map(|col| {
            parser
                .screen()
                .cell(row, col)
                .map(|cell| cell.contents())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join("")
}

/// PLT-007: the writer changes attributes without losing the colors in
/// force, and an earlier run's attribute does not leak into a later run.
#[test]
fn plt_007_the_writer_keeps_colors_across_attribute_changes() {
    let red = Rgba::new(1.0, 0.0, 0.0, 1.0);
    let blue = Rgba::new(0.0, 0.0, 1.0, 1.0);
    let mut ops = RenderOpsBuilder::new();
    ops.move_to(0, 0)
        .print_styled("A", red, blue, Attr::BOLD)
        .print_styled("B", red, blue, Attr::ITALIC);
    let bytes = render_ops_to_ansi(&ops.build());
    let parser = screen(&bytes, 1, 10);
    let screen = parser.screen();
    let a = screen.cell(0, 0).expect("cell A");
    let b = screen.cell(0, 1).expect("cell B");
    assert_eq!(
        (a.fgcolor(), a.bgcolor(), a.bold()),
        (
            vt100::Color::Rgb(255, 0, 0),
            vt100::Color::Rgb(0, 0, 255),
            true
        ),
        "PLT-007: the bold run A is painted {:?} on {:?}, bold {}; the output was {:?}",
        a.fgcolor(),
        a.bgcolor(),
        a.bold(),
        String::from_utf8_lossy(&bytes)
    );
    assert_eq!(
        (b.fgcolor(), b.bgcolor(), b.italic(), b.bold()),
        (
            vt100::Color::Rgb(255, 0, 0),
            vt100::Color::Rgb(0, 0, 255),
            true,
            false
        ),
        "PLT-007: the italic run B is painted {:?} on {:?}, italic {}, bold {}",
        b.fgcolor(),
        b.bgcolor(),
        b.italic(),
        b.bold()
    );
}

fn surface(width: usize, height: usize, rows: &[&str]) -> GraphemeSurface {
    let mut surface = GraphemeSurface::new(width, height);
    for (y, row) in rows.iter().enumerate() {
        surface.write_str(0, y, row, Rgba::white(), Rgba::black(), Attr::empty());
    }
    surface
}

/// PLT-008: `diff_with_stats` clears what a smaller surface no longer covers,
/// as `diff` does.
#[test]
fn plt_008_diff_with_stats_clears_what_a_smaller_surface_leaves() {
    let old = surface(5, 1, &["ABCDE"]);
    let new = surface(3, 1, &["ABC"]);
    let mut writer = SpanDiffWriter::new();
    writer.diff(&GraphemeSurface::new(5, 1), &old);
    let mut terminal = vt100::Parser::new(1, 5, 0);
    terminal.process(writer.output());
    assert_eq!(
        row_text(&terminal, 0, 5),
        "ABCDE",
        "the old surface is shown"
    );
    writer.diff_with_stats(&old, &new);
    terminal.process(writer.output());
    let row = row_text(&terminal, 0, 5);
    assert_eq!(
        row.trim_end(),
        "ABC",
        "PLT-008: after the surface shrank to three columns, the screen shows {row:?}"
    );

    let old = surface(1, 2, &["A", "B"]);
    let new = surface(1, 1, &["A"]);
    let mut writer = SpanDiffWriter::new();
    writer.diff(&GraphemeSurface::new(1, 2), &old);
    let mut terminal = vt100::Parser::new(2, 1, 0);
    terminal.process(writer.output());
    writer.diff_with_stats(&old, &new);
    terminal.process(writer.output());
    let second = row_text(&terminal, 1, 1);
    assert_eq!(
        second.trim(),
        "",
        "PLT-008: after the surface shrank to one row, the second row still shows {second:?}"
    );
}

fn glyph(cell: &CellType) -> Option<String> {
    match cell {
        CellType::Glyph { grapheme, .. } => Some(grapheme.as_str().to_string()),
        _ => None,
    }
}

/// The column at which the spans print `needle`, by the display width of the
/// text before it in its span.
fn printed_column(spans: &[Span], needle: &str) -> Option<usize> {
    spans.iter().find_map(|span| {
        span.text
            .find(needle)
            .map(|at| span.start_col + UnicodeWidthStr::width(&span.text[..at]))
    })
}

/// PLT-009: overwriting either half of a wide glyph clears both of its cells,
/// the spans print each character at the column that holds it, and a wide
/// glyph without room is not placed.
#[test]
fn plt_009_a_wide_glyph_overwrite_keeps_cells_and_spans_consistent() {
    let (fg, bg, attr) = (Rgba::white(), Rgba::black(), Attr::empty());

    let mut surface = GraphemeSurface::new(3, 1);
    surface.write_str(0, 0, "界B", fg, bg, attr);
    surface.write_str(0, 0, "X", fg, bg, attr);
    assert_eq!(glyph(&surface.get_cell(0, 0)).as_deref(), Some("X"));
    assert!(
        matches!(surface.get_cell(1, 0), CellType::Spacer { .. }),
        "PLT-009: after X over the first half of 界, column 1 holds {:?}, not a blank",
        surface.get_cell(1, 0)
    );
    assert_eq!(
        glyph(&surface.get_cell(2, 0)).as_deref(),
        Some("B"),
        "PLT-009: B moved from column 2"
    );
    let spans = surface.to_row_spans(0);
    let column = printed_column(&spans, "B");
    assert_eq!(
        column,
        Some(2),
        "PLT-009: the row's spans print B at column {column:?}: {spans:?}"
    );

    let mut surface = GraphemeSurface::new(3, 1);
    surface.write_str(0, 0, "界B", fg, bg, attr);
    surface.write_str(1, 0, "X", fg, bg, attr);
    assert_ne!(
        glyph(&surface.get_cell(0, 0)).as_deref(),
        Some("界"),
        "PLT-009: X over the second half of 界 left its first half in column 0"
    );
    assert_eq!(glyph(&surface.get_cell(1, 0)).as_deref(), Some("X"));
    assert_eq!(glyph(&surface.get_cell(2, 0)).as_deref(), Some("B"));

    let mut surface = GraphemeSurface::new(3, 1);
    surface.write_str(2, 0, "界", fg, bg, attr);
    assert!(
        glyph(&surface.get_cell(2, 0)).is_none(),
        "PLT-009: a wide glyph was placed in the last column, where it does not fit"
    );
}

/// PLT-010: `contrast_ratio` is the WCAG contrast of the linearized colors.
#[test]
fn plt_010_contrast_ratio_is_the_wcag_contrast() {
    let gray = |value: f32| Rgba::new(value, value, value, 1.0);
    for (color, against, expected) in [
        (gray(0.4), Rgba::black(), 3.657_f32),
        (gray(0.4), Rgba::white(), 5.742),
        (gray(0.02), Rgba::black(), 1.031),
    ] {
        let ratio = color.contrast_ratio(against);
        assert!(
            (ratio - expected).abs() < 0.01,
            "PLT-010: the contrast of {color:?} against {against:?} is {ratio}, not {expected}"
        );
    }
}
