//! Tests for markdown rendering functionality

use super::*;
use crate::core::surface::Attr;

#[test]
fn test_basic_markdown_rendering() {
    let renderer = MarkdownRenderer::new();
    let markdown = "# Hello World\n\nThis is a **bold** text with *italic* and `code`.";
    let lines = renderer.render_to_styled_lines(markdown);

    assert!(!lines.is_empty(), "Should render at least one line");

    // Check that heading is rendered
    let first_line = &lines[0];
    assert!(
        !first_line.runs.is_empty(),
        "First line should have content"
    );

    // Check that bold attribute is applied somewhere
    let has_bold = lines
        .iter()
        .any(|line| line.runs.iter().any(|run| run.attr.contains(Attr::BOLD)));
    assert!(has_bold, "Should have bold text somewhere");

    // Check that italic attribute is applied somewhere
    let has_italic = lines
        .iter()
        .any(|line| line.runs.iter().any(|run| run.attr.contains(Attr::ITALIC)));
    assert!(has_italic, "Should have italic text somewhere");
}

#[test]
fn test_list_rendering() {
    let renderer = MarkdownRenderer::new();
    let markdown = "- Item 1\n- Item 2\n\n1. First\n2. Second";
    let lines = renderer.render_to_styled_lines(markdown);

    assert!(!lines.is_empty(), "Should render list items");

    // Check for bullet point
    let has_bullet = lines
        .iter()
        .any(|line| line.runs.iter().any(|run| run.text.contains("•")));
    assert!(has_bullet, "Should have bullet points");

    // Check for numbered list
    let has_number = lines.iter().any(|line| {
        line.runs
            .iter()
            .any(|run| run.text.contains("1.") || run.text.contains("2."))
    });
    assert!(has_number, "Should have numbered list items");
}

#[test]
fn test_code_block_rendering() {
    let renderer = MarkdownRenderer::new();
    let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
    let lines = renderer.render_to_styled_lines(markdown);

    assert!(!lines.is_empty(), "Should render code block");

    // Check that code has different background
    let has_code_bg = lines.iter().any(|line| {
        line.runs
            .iter()
            .any(|run| run.bg != crate::core::surface::Rgba::transparent())
    });
    assert!(has_code_bg, "Code should have background color");
}

#[test]
fn api019_fenced_rust_code_uses_lumis_and_disabled_mode_keeps_code_style() {
    let markdown = "```Rust\nfn main() { let answer = 42; }\n```";
    let highlighted = MarkdownRenderer::new().render_to_styled_lines(markdown);
    let plain = MarkdownRenderer::new()
        .with_syntax_highlighting(false)
        .render_to_styled_lines(markdown);
    let highlighted_runs = highlighted
        .iter()
        .flat_map(|line| &line.runs)
        .collect::<Vec<_>>();
    let plain_runs = plain.iter().flat_map(|line| &line.runs).collect::<Vec<_>>();
    assert!(highlighted_runs.iter().any(|run| run.text.contains("fn")));
    assert!(highlighted_runs
        .windows(2)
        .any(|pair| pair[0].fg != pair[1].fg));
    assert_ne!(highlighted_runs, plain_runs);
}

#[test]
fn api019_markdown_and_syntax_checked_entry_points_reject_oversized_sources() {
    let oversized = "x".repeat(MAX_MARKDOWN_BYTES + 1);
    let renderer = MarkdownRenderer::new();
    let error = renderer.try_render_to_styled_lines(&oversized).unwrap_err();
    assert!(error.to_string().contains("1048576-byte"));
    let diagnostic = renderer.render_to_styled_lines(&oversized);
    assert!(diagnostic[0].text().contains("Markdown render error"));

    let mut highlighter = crate::syntax::SyntaxHighlighter::new("Rust").unwrap();
    let syntax_error = highlighter.try_highlight_text(&oversized).unwrap_err();
    assert!(syntax_error.to_string().contains("1048576-byte"));
    let diagnostic = highlighter.highlight_text(&oversized);
    assert!(diagnostic[0]
        .to_styled_line()
        .text()
        .contains("Syntax highlight error"));
}

#[test]
fn test_empty_markdown() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.render_to_styled_lines("");

    // Empty markdown should still produce at least an empty line
    assert!(
        !lines.is_empty(),
        "Should produce at least one line for empty input"
    );
}

#[test]
fn test_convenience_function() {
    let markdown = "# Test\n\nHello world!";
    let lines = render_markdown(markdown);

    assert!(!lines.is_empty(), "Convenience function should work");
    assert!(
        lines[0]
            .runs
            .iter()
            .any(|run| run.attr.contains(Attr::BOLD)),
        "Should have heading"
    );
}

#[test]
fn test_gfm_extensions() {
    let renderer = MarkdownRenderer::new();
    let markdown = "~~strikethrough~~ text and a table:\n\n| A | B |\n|---|---|\n| 1 | 2 |";
    let lines = renderer.render_to_styled_lines(markdown);

    assert!(!lines.is_empty(), "Should render GFM content");

    // Check for strikethrough
    let has_strike = lines
        .iter()
        .any(|line| line.runs.iter().any(|run| run.attr.contains(Attr::STRIKE)));
    assert!(has_strike, "Should support strikethrough extension");

    // Check for table markers
    let has_table = lines
        .iter()
        .any(|line| line.runs.iter().any(|run| run.text.contains("│")));
    assert!(has_table, "Should support table extension");
}
