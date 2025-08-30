//! Grid Layout Showcase
//!
//! Interactive demo showing different grid layouts with background colors
//! Navigate with Space/Arrow keys, similar to the reference layout showcase

use crossterm::{
    event::{read, Event, KeyCode, KeyModifiers},
};
use reactive_tui::layout;
use reactive_tui::layout::renderer::render_grid;
use reactive_tui::core::surface::{Surface, Rgba};
use reactive_tui::core::renderer::Renderer;
use std::io::{stdout, Result};

struct GridShowcase {
    current_demo: usize,
    demos: Vec<Demo>,
}

struct Demo {
    title: String,
    description: String,
    grid_fn: fn() -> reactive_tui::layout::grid::DeclarativeGrid,
}

impl GridShowcase {
    fn new() -> Self {
        let demos = vec![
            Demo {
                title: "BASIC GRID LAYOUT".to_string(),
                description: "Simple 3x2 grid with colored cells".to_string(),
                grid_fn: create_basic_grid,
            },
            Demo {
                title: "DASHBOARD LAYOUT".to_string(),
                description: "Header, sidebar, main content, and footer".to_string(),
                grid_fn: create_dashboard_grid,
            },
            Demo {
                title: "CARD GRID LAYOUT".to_string(),
                description: "Responsive card grid with equal spacing".to_string(),
                grid_fn: create_card_grid,
            },
            Demo {
                title: "LAYERED LAYOUT".to_string(),
                description: "Z-index stacking: Background → Window → Modal → Tooltip".to_string(),
                grid_fn: create_layered_grid,
            },
            Demo {
                title: "COMPLEX IDE LAYOUT".to_string(),
                description: "Multi-panel IDE interface with spanning areas".to_string(),
                grid_fn: create_ide_grid,
            },
        ];

        Self {
            current_demo: 0,
            demos,
        }
    }

    fn total_demos(&self) -> usize {
        self.demos.len()
    }

    fn current_demo(&self) -> &Demo {
        &self.demos[self.current_demo]
    }

    fn next_demo(&mut self) {
        self.current_demo = (self.current_demo + 1) % self.total_demos();
    }

    fn prev_demo(&mut self) {
        self.current_demo = if self.current_demo == 0 {
            self.total_demos() - 1
        } else {
            self.current_demo - 1
        };
    }

    fn render(&self, renderer: &mut Renderer) -> reactive_tui::error::Result<()> {
        renderer.begin_frame()?;
        renderer.clear(Rgba::black());

        let surface = renderer.surface_mut();
        let (width, height) = surface.dims();

        // Clear surface
        surface.clear(Rgba::black());

        // Render title
        let title = format!("REACTIVE-TUI GRID SHOWCASE - {}", self.current_demo().title);
        self.draw_centered_text(surface, &title, 1, width, Rgba::white(), Rgba::black());

        // Render description
        let gray = Rgba::new(0.7, 0.7, 0.7, 1.0);
        self.draw_centered_text(surface, &self.current_demo().description, 3, width, gray, Rgba::black());

        // Create and render the grid
        let grid = (self.current_demo().grid_fn)();
        
        // Render grid starting from line 6
        let grid_surface_height = height.saturating_sub(10);
        let mut grid_surface = Surface::new(width, grid_surface_height);
        grid_surface.clear(Rgba::black());
        
        if let Err(e) = render_grid(&grid, &mut grid_surface, width) {
            // If grid rendering fails, show error
            let error_msg = format!("Grid render error: {}", e);
            let red = Rgba::new(1.0, 0.0, 0.0, 1.0);
            self.draw_centered_text(surface, &error_msg, 8, width, red, Rgba::black());
        } else {
            // Copy grid surface to main surface starting at y=6
            for y in 0..grid_surface_height {
                for x in 0..width {
                    let cell = grid_surface.get(x, y);
                    if y + 6 < height {
                        surface.set(x, y + 6, cell);
                    }
                }
            }
        }

        // Render navigation help
        let nav_y = height.saturating_sub(3);
        let nav_text = format!("Demo {}/{} | Space/→: next | ←: prev | Q: quit", 
            self.current_demo + 1, self.total_demos());
        let yellow = Rgba::new(1.0, 1.0, 0.0, 1.0);
        self.draw_centered_text(surface, &nav_text, nav_y, width, yellow, Rgba::black());

        renderer.end_frame()?;
        Ok(())
    }

    fn draw_centered_text(&self, surface: &mut Surface, text: &str, y: usize, width: usize, fg: Rgba, bg: Rgba) {
        let (surface_width, surface_height) = surface.dims();
        if y >= surface_height {
            return;
        }

        let start_x = if text.len() < width {
            (width - text.len()) / 2
        } else {
            0
        };

        for (i, ch) in text.chars().enumerate() {
            let x = start_x + i;
            if x < surface_width {
                let mut cell = surface.get(x, y);
                cell.ch = ch;
                cell.fg = fg;
                cell.bg = bg;
                surface.set(x, y, cell);
            }
        }
    }
}

// Grid creation functions
fn create_basic_grid() -> reactive_tui::layout::grid::DeclarativeGrid {
    layout! {
        grid(cols: 3, rows: 2, gap: 2) {
            "ROW 1, COL 1" at (0, 0) class "bg-red-500 text-white p-4 flex items-center justify-center",
            "ROW 1, COL 2" at (0, 1) class "bg-green-500 text-white p-4 flex items-center justify-center",
            "ROW 1, COL 3" at (0, 2) class "bg-blue-500 text-white p-4 flex items-center justify-center",
            "ROW 2, COL 1" at (1, 0) class "bg-yellow-400 text-black p-4 flex items-center justify-center",
            "ROW 2, COL 2" at (1, 1) class "bg-purple-500 text-white p-4 flex items-center justify-center",
            "ROW 2, COL 3" at (1, 2) class "bg-pink-500 text-white p-4 flex items-center justify-center",
        }
    }
}

fn create_dashboard_grid() -> reactive_tui::layout::grid::DeclarativeGrid {
    layout! {
        grid(cols: 4, rows: 4, gap: 1) {
            "📊 HEADER" at (0, 0) span (1, 4) class "bg-blue-600 text-white p-3 flex items-center justify-center",
            "📁 SIDEBAR" at (1, 0) class "bg-gray-400 text-black p-3 flex items-center justify-center",
            "📄 MAIN" at (1, 1) span (2, 2) class "bg-white text-black p-4 flex items-center justify-center",
            "🔔 ALERTS" at (1, 3) class "bg-yellow-400 text-black p-3 flex items-center justify-center",
            "ℹ️ FOOTER" at (3, 0) span (1, 4) class "bg-gray-600 text-white p-2 flex items-center justify-center",
        }
    }
}

fn create_card_grid() -> reactive_tui::layout::grid::DeclarativeGrid {
    layout! {
        grid(cols: 3, rows: 3, gap: 3) {
            "🎯 PROJECT A" at (0, 0) class "bg-indigo-500 text-white p-4 flex items-center justify-center",
            "📈 ANALYTICS" at (0, 1) class "bg-green-500 text-white p-4 flex items-center justify-center",
            "⚠️ ALERTS" at (0, 2) class "bg-red-500 text-white p-4 flex items-center justify-center",
            "📊 REPORTS" at (1, 0) class "bg-yellow-500 text-black p-4 flex items-center justify-center",
            "👥 USERS" at (1, 1) class "bg-purple-500 text-white p-4 flex items-center justify-center",
            "💬 MESSAGES" at (1, 2) class "bg-pink-500 text-white p-4 flex items-center justify-center",
            "🔧 SETTINGS" at (2, 0) class "bg-blue-500 text-white p-4 flex items-center justify-center",
            "📱 MOBILE" at (2, 1) class "bg-teal-500 text-white p-4 flex items-center justify-center",
            "🌐 WEB" at (2, 2) class "bg-orange-500 text-white p-4 flex items-center justify-center",
        }
    }
}

fn create_layered_grid() -> reactive_tui::layout::grid::DeclarativeGrid {
    layout! {
        grid(cols: 6, rows: 6, gap: 0) {
            // Layer 0: Desktop Background (lowest)
            "DESKTOP BACKGROUND (z:0)" at (0, 0) span (6, 6) class "bg-blue-600 text-white flex items-center justify-center" z 0,

            // Layer 1: Application Window
            "APP WINDOW (z:1)" at (1, 1) span (4, 3) class "bg-gray-200 text-black flex items-center justify-center" z 1,

            // Layer 2: Document Window (overlaps app window)
            "DOCUMENT (z:2)" at (2, 2) span (3, 3) class "bg-white text-gray-800 flex items-center justify-center" z 2,

            // Layer 10: Modal Dialog (high z-index, covers most things)
            "MODAL DIALOG (z:10)" at (1, 2) span (3, 2) class "bg-red-500 text-white flex items-center justify-center" z 10,

            // Layer 20: Tooltip (highest z-index, always on top)
            "TOOLTIP (z:20)" at (0, 5) span (2, 1) class "bg-yellow-400 text-black flex items-center justify-center" z 20,
        }
    }
}

fn create_ide_grid() -> reactive_tui::layout::grid::DeclarativeGrid {
    layout! {
        grid(cols: 5, rows: 5, gap: 1) {
            "🪟 TITLE BAR" at (0, 0) span (1, 5) class "bg-gray-600 text-white p-2 flex items-center justify-center",
            "📁 FILES" at (1, 0) span (3, 1) class "bg-gray-400 text-black p-3 flex items-center justify-center",
            "📝 EDITOR" at (1, 1) span (2, 3) class "bg-white text-black p-4 flex items-center justify-center",
            "🔍 PROPS" at (1, 4) span (2, 1) class "bg-blue-400 text-white p-3 flex items-center justify-center",
            "💻 TERMINAL" at (3, 1) span (1, 3) class "bg-black text-green-500 p-3 flex items-center justify-center",
            "📊 STATUS" at (4, 0) span (1, 5) class "bg-blue-600 text-white p-1 flex items-center justify-center",
        }
    }
}

fn main() -> Result<()> {
    // Framework handles terminal cleanup automatically via Drop traits

    let mut showcase = GridShowcase::new();
    let mut renderer = Renderer::new(100, 30).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Renderer error: {}", e))
    })?;

    loop {
        if let Err(e) = showcase.render(&mut renderer) {
            eprintln!("Render error: {}", e);
            break;
        }

        match read()? {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char(' ') | KeyCode::Right | KeyCode::Enter => showcase.next_demo(),
                    KeyCode::Left | KeyCode::Backspace => showcase.prev_demo(),
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    println!("Grid showcase completed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_showcase_creation() {
        let showcase = GridShowcase::new();
        assert_eq!(showcase.total_demos(), 5);
        assert_eq!(showcase.current_demo, 0);
    }

    #[test]
    fn test_grid_creation() {
        // Test that all grid creation functions work
        let _basic = create_basic_grid();
        let _dashboard = create_dashboard_grid();
        let _cards = create_card_grid();
        let _layered = create_layered_grid();
        let _ide = create_ide_grid();
    }
}
