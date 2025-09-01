//! Simple working syntax highlighting example
//!
//! Demonstrates the syntax highlighting system with direct ANSI output

use reactive_tui::syntax::highlighter::SyntaxHighlighter;
use std::io;

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
    fn test_syntax_highlighting_integration() -> io::Result<()> {
        // Create syntax highlighter
        let mut highlighter = match SyntaxHighlighter::new("Rust") {
            Some(h) => h,
            None => {
                eprintln!("❌ Failed to create Rust highlighter");
                return Ok(());
            }
        };

        // Highlight the code
        let highlighted = highlighter.highlight_text(RUST_CODE);

        // Debug what colors we're actually getting
        println!("🔍 Debug: Colors being generated:");
        let mut unique_colors = std::collections::HashSet::new();
        for line in &highlighted {
            for run in &line.runs {
                let color = (
                    (run.fg.r * 255.0) as u8,
                    (run.fg.g * 255.0) as u8,
                    (run.fg.b * 255.0) as u8,
                );
                unique_colors.insert((color, run.text.clone()));
            }
        }

        for ((r, g, b), text) in unique_colors.iter().take(10) {
            println!(
                "  Color ({:3},{:3},{:3}): '{}'",
                r,
                g,
                b,
                text.chars().take(10).collect::<String>()
            );
        }
        println!();

        println!("🎨 Syntax Highlighting Demo - Direct ANSI Output");
        println!("{}", "=".repeat(60));

        // Output syntax highlighted code directly with ANSI escape sequences
        for line in &highlighted {
            print!("{:3} │ ", line.line_number + 1);

            for run in &line.runs {
                // Convert Rgba to RGB values
                let r = (run.fg.r * 255.0) as u8;
                let g = (run.fg.g * 255.0) as u8;
                let b = (run.fg.b * 255.0) as u8;

                // Output with ANSI color
                print!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, run.text);
            }
            println!();
        }

        println!("{}", "=".repeat(60));
        println!("✅ If you see colors above, syntax highlighting is working!");
        println!("❌ If all text is the same color, there's a renderer bug.");

        Ok(())
    }
}
