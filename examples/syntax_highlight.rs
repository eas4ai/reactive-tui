//! Working syntax highlighting example
//!
//! Demonstrates the syntax highlighting system with a simple viewer

use crossterm::event::{self, Event, KeyCode};
use reactive_tui::core::renderer::Renderer;
use reactive_tui::core::surface::{Attr, Cell, Rgba};
use reactive_tui::syntax::highlighter::SyntaxHighlighter;
use std::io::Result;
use std::time::Duration;

const RUST_CODE: &str = r#"use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    // Print the value
    if let Some(val) = map.get("key") {
        println!("Value: {}", val);
    }
}"#;

fn main() -> Result<()> {
    // Get terminal size
    let (width, height) = crossterm::terminal::size()?;

    // Create renderer
    let mut renderer = Renderer::new(width as usize, height as usize)?;

    // Create syntax highlighter
    let mut highlighter =
        SyntaxHighlighter::new("Rust").expect("Failed to create Rust highlighter");

    // Highlight the code
    let highlighted = highlighter.highlight_text(RUST_CODE);

    let mut running = true;

    while running {
        // Begin frame
        renderer.begin_frame()?;

        // Get surface
        let surface = renderer.surface_mut();

        // Clear with dark background
        let bg = Rgba {
            r: 0.05,
            g: 0.05,
            b: 0.05,
            a: 1.0,
        };
        surface.clear(bg);

        // Draw title
        let title = "Syntax Highlighting Demo (ESC to quit)";
        let title_fg = Rgba {
            r: 0.8,
            g: 0.8,
            b: 0.8,
            a: 1.0,
        };
        for (i, ch) in title.chars().enumerate() {
            surface.set(
                i + 1,
                0,
                Cell {
                    ch,
                    fg: title_fg,
                    bg,
                    attr: Attr::BOLD,
                },
            );
        }

        // Draw highlighted code
        let mut y = 2;
        for line in &highlighted {
            let mut x = 1;
            for run in &line.runs {
                for ch in run.text.chars() {
                    if x < width as usize - 1 && y < height as usize - 1 {
                        surface.set(
                            x,
                            y,
                            Cell {
                                ch,
                                fg: run.fg,
                                bg: run.bg,
                                attr: run.attr,
                            },
                        );
                        x += 1;
                    }
                }
            }
            y += 1;
            if y >= height as usize - 1 {
                break;
            }
        }

        // End frame
        renderer.end_frame()?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc {
                    running = false;
                }
            }
        }
    }

    // Cleanup
    renderer.shutdown()?;

    Ok(())
}
