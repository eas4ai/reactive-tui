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
