//! Part of tests/review_native.rs.

use reactive_tui::{
    core::{styled_text::StyledLine, surface::Rgba},
    editor::{cursor::Movement, SyntaxEditor},
    markdown::{MarkdownRenderer, MAX_MARKDOWN_BYTES},
    syntax::SyntaxHighlighter,
};

/// The editor's line-number gutter: four digits and a space.
const GUTTER: usize = 5;

/// Each character of a line with its foreground color.
fn colored(line: &StyledLine) -> Vec<(char, Rgba)> {
    line.runs
        .iter()
        .flat_map(|run| run.text.chars().map(move |c| (c, run.fg)))
        .collect()
}

/// Line `index` of `document` as a fresh whole-document highlighter paints it.
fn highlighted(document: &str, index: usize) -> Vec<(char, Rgba)> {
    let mut highlighter = SyntaxHighlighter::new("Rust").expect("the Rust language");
    let lines = highlighter.highlight_text(document);
    colored(&lines[index].to_styled_line())
}

/// Line `index` as the editor paints it, without the gutter and cut to
/// `width` characters.
fn painted(editor: &mut SyntaxEditor, index: usize, width: usize) -> Vec<(char, Rgba)> {
    let lines = editor.get_styled_lines();
    colored(&lines[index])
        .into_iter()
        .skip(GUTTER)
        .take(width)
        .collect()
}

#[test]
fn txt_001_a_line_inside_a_block_comment_is_painted_as_the_comment() {
    let document = "/*\nfn main() {}\n*/";
    let mut editor = SyntaxEditor::with_language(document, "Rust").unwrap();
    let expected = highlighted(document, 1);
    let actual = painted(&mut editor, 1, expected.len());
    assert_eq!(
        actual, expected,
        "TXT-001: the editor painted the line inside the block comment with its own colors"
    );
}

#[test]
fn txt_001_an_edit_above_a_visible_line_rehighlights_it() {
    let mut editor = SyntaxEditor::with_language("let a = 1;\nlet x = 5;\n*/", "Rust").unwrap();
    editor.move_cursor(Movement::DocumentStart, false);
    editor.insert_text("/* ").unwrap();
    let document = editor.content();
    assert_eq!(document, "/* let a = 1;\nlet x = 5;\n*/");
    let expected = highlighted(&document, 1);
    let actual = painted(&mut editor, 1, expected.len());
    assert_eq!(
        actual, expected,
        "TXT-001: after a comment opened above it, `let x = 5;` kept the colors of code"
    );
}

#[test]
fn txt_002_render_with_sourcepos_applies_the_byte_limit() {
    let renderer = MarkdownRenderer::new();
    let source = "x".repeat(MAX_MARKDOWN_BYTES + 1);
    let (lines, positions) = renderer.render_with_sourcepos(&source);
    let text: Vec<String> = lines.iter().map(StyledLine::text).collect();
    assert!(
        text.iter()
            .any(|line| line.contains("Markdown render error")),
        "TXT-002: render_with_sourcepos rendered a source over MAX_MARKDOWN_BYTES: {} line(s)",
        lines.len()
    );
    assert!(
        positions.is_empty(),
        "TXT-002: render_with_sourcepos gave positions for a source over MAX_MARKDOWN_BYTES"
    );
}

#[test]
fn txt_002_the_manual_describes_the_highlighters_reuse() {
    let manual = include_str!("../../manual/text-editing-markdown-and-syntax.md");
    assert!(
        manual.contains("parses a document once per change of its text or theme"),
        "TXT-002: the manual does not say that the highlighter parses once per change and serves repeats from its cache"
    );
}

#[test]
fn txt_003_a_table_gets_a_separator_and_aligned_cells() {
    let lines = MarkdownRenderer::new()
        .render_to_styled_lines("| Name | Qty |\n| :--- | ---: |\n| ab | 1 |");
    let text: Vec<String> = lines.iter().map(StyledLine::text).collect();
    let header = text
        .iter()
        .position(|line| line == "│ Name │ Qty │")
        .unwrap_or_else(|| panic!("TXT-003: no header row `│ Name │ Qty │` in {text:?}"));
    assert_eq!(
        text.get(header + 1).map(String::as_str),
        Some("├──────┼─────┤"),
        "TXT-003: the line after the header row is not a separator with one segment per column: {text:?}"
    );
    assert_eq!(
        text.get(header + 2).map(String::as_str),
        Some("│ ab   │   1 │"),
        "TXT-003: the body row is not padded by its columns' alignment: {text:?}"
    );
}
