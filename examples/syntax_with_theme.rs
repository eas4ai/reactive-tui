//! Demonstrates syntax highlighting integrated with CSS theme system
//!
//! Shows how syntect themes are bridged to reactive-tui's CSS variables

use crossterm::event::{self, Event, KeyCode};
use reactive_tui::core::renderer::Renderer;
use reactive_tui::core::surface::{Attr, Cell, Rgba};
use reactive_tui::syntax::{SyntaxHighlighter, create_syntax_theme};
use reactive_tui::theme::presets::dark_theme;
use std::io::Result;
use std::time::Duration;

const RUST_CODE: &str = r#"// Example Rust code
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    // Print the value
    if let Some(val) = map.get("key") {
        println!("Value: {}", val);
    }
}

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}"#;

fn main() -> Result<()> {
    // Create a theme with syntax highlighting support
    let theme = create_syntax_theme(dark_theme(), "base16-ocean.dark");

    // Get terminal size
    let (width, height) = crossterm::terminal::size()?;

    // Create renderer
    let mut renderer = Renderer::new(width as usize, height as usize)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Create syntax highlighter
    let mut highlighter =
        SyntaxHighlighter::new("Rust").expect("Failed to create Rust highlighter");

    // Highlight the code
    let highlighted = highlighter.highlight_text(RUST_CODE);

    let mut running = true;

    println!("CSS Theme Integration Demo");
    println!("Theme variables are available as:");
    println!("  --syntax-keyword");
    println!("  --syntax-type");
    println!("  --syntax-function");
    println!("  --syntax-string");
    println!("  --syntax-comment");
    println!("  --syntax-number");
    println!("");
    println!("Press any key to start, ESC to quit");

    // Wait for key press
    while event::poll(Duration::from_millis(100))? {
        if let Event::Key(_) = event::read()? {
            break;
        }
    }

    while running {
        // Begin frame
        renderer
            .begin_frame()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        // Get surface
        let surface = renderer.surface_mut();

        // Clear with theme background
        let bg = if let Some(bg_color) = theme.get_variable("--color-background") {
            // Parse hex color from theme
            let hex = bg_color.trim_start_matches('#');
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
            Rgba { r, g, b, a: 1.0 }
        } else {
            Rgba {
                r: 0.05,
                g: 0.05,
                b: 0.05,
                a: 1.0,
            }
        };
        surface.clear(bg);

        // Draw title with theme colors
        let title = "Syntax Highlighting with CSS Theme Integration (ESC to quit)";
        let title_fg = if let Some(fg_color) = theme.get_variable("--color-text") {
            let hex = fg_color.trim_start_matches('#');
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(200) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(200) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(200) as f32 / 255.0;
            Rgba { r, g, b, a: 1.0 }
        } else {
            Rgba {
                r: 0.8,
                g: 0.8,
                b: 0.8,
                a: 1.0,
            }
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
        for (line_num, line) in highlighted.iter().enumerate() {
            // Draw line number
            let line_num_str = format!("{:3} ", line_num + 1);
            let line_num_fg = Rgba {
                r: 0.4,
                g: 0.4,
                b: 0.4,
                a: 1.0,
            };
            for (i, ch) in line_num_str.chars().enumerate() {
                surface.set(
                    i,
                    y,
                    Cell {
                        ch,
                        fg: line_num_fg,
                        bg,
                        attr: Attr::empty(),
                    },
                );
            }

            // Draw code with syntax highlighting
            let mut x = 5;
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
        renderer
            .end_frame()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

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
    renderer
        .shutdown()
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok(())
}
