//! Simple working syntax highlighting example
//!
//! Demonstrates the syntax highlighting system with direct ANSI output

use reactive_tui::syntax::highlighter::SyntaxHighlighter;

const RUST_CODE: &str = r#"use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    // Print the value
    if let Some(val) = map.get("key") {
        println!("Value: {}", val);
    }
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlighting_integration() {
        // A missing Rust grammar is a failure, not a skip: returning Ok
        // here once hid a broken highlighter behind a passing test.
        let mut highlighter = SyntaxHighlighter::new("Rust").expect("Rust syntax highlighter");

        let highlighted = highlighter.highlight_text(RUST_CODE);

        // Every source line survives, in order, with its line number.
        assert_eq!(highlighted.len(), RUST_CODE.lines().count());
        for (line, source) in highlighted.iter().zip(RUST_CODE.lines()) {
            let text: String = line.runs.iter().map(|run| run.text.as_str()).collect();
            assert_eq!(text, source, "line {} mangled", line.line_number + 1);
        }

        // Runs carry more than one foreground color: keywords, strings
        // and comments must not all render identically.
        let colors: std::collections::HashSet<(u8, u8, u8)> = highlighted
            .iter()
            .flat_map(|line| line.runs.iter())
            .map(|run| {
                (
                    (run.fg.r * 255.0) as u8,
                    (run.fg.g * 255.0) as u8,
                    (run.fg.b * 255.0) as u8,
                )
            })
            .collect();
        assert!(
            colors.len() > 1,
            "all runs share one color; highlighting is dead"
        );

        // The comment line is tokenized, not swallowed. Runs may split
        // the comment text, so match on the joined line.
        let comment = highlighted
            .iter()
            .find(|line| {
                line.runs
                    .iter()
                    .map(|run| run.text.as_str())
                    .collect::<String>()
                    .contains("// Print the value")
            })
            .expect("comment line highlighted");
        assert!(!comment.runs.is_empty());
    }
}
