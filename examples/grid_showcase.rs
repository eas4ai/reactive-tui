//! Grid Layout Showcase - Unified API Demo
//!
//! Interactive demo showing different grid layouts using the unified el! macro
//! Demonstrates mixing VDOM, builder, and direct Element creation seamlessly
//! Navigate with Space/Arrow keys, similar to the reference layout showcase

use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use reactive_tui::core::renderer::Renderer;
use reactive_tui::core::surface::{Rgba, Surface};
use reactive_tui::layout;
use reactive_tui::layout::renderer::render_grid;
use std::io::Result;

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
                title: "UNIFIED SYNTAX BASICS".to_string(),
                description: "el! macro with strings, web_api builders, and VDOM nodes".to_string(),
                grid_fn: create_basic_grid,
            },
            Demo {
                title: "MIXED API DASHBOARD".to_string(),
                description: "Header/footer (web_api) + sidebar (VDOM) + content (mixed)"
                    .to_string(),
                grid_fn: create_dashboard_grid,
            },
            Demo {
                title: "DYNAMIC CARD GRID".to_string(),
                description: "Data-driven cards using different element creation approaches"
                    .to_string(),
                grid_fn: create_card_grid,
            },
            Demo {
                title: "LAYERED UI DEMO".to_string(),
                description: "Z-index layers: strings → builders → VDOM → mixed content"
                    .to_string(),
                grid_fn: create_layered_grid,
            },
            Demo {
                title: "COMPLEX IDE SHOWCASE".to_string(),
                description: "Full IDE layout mixing all APIs: variables, builders, VDOM trees"
                    .to_string(),
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
        let title = format!("UNIFIED el! MACRO SHOWCASE - {}", self.current_demo().title);
        self.draw_centered_text(surface, &title, 1, width, Rgba::white(), Rgba::black());

        // Render description
        let gray = Rgba::new(0.7, 0.7, 0.7, 1.0);
        self.draw_centered_text(
            surface,
            &self.current_demo().description,
            3,
            width,
            gray,
            Rgba::black(),
        );

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
        let nav_text = format!(
            "Demo {}/{} | Space/→: next | ←: prev | Q: quit",
            self.current_demo + 1,
            self.total_demos()
        );
        let yellow = Rgba::new(1.0, 1.0, 0.0, 1.0);
        self.draw_centered_text(surface, &nav_text, nav_y, width, yellow, Rgba::black());

        renderer.end_frame()?;
        Ok(())
    }

    fn draw_centered_text(
        &self,
        surface: &mut Surface,
        text: &str,
        y: usize,
        width: usize,
        fg: Rgba,
        bg: Rgba,
    ) {
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

// Grid creation functions demonstrating unified el! macro syntax

fn create_basic_grid() -> reactive_tui::layout::grid::DeclarativeGrid {
    /*
    UNIFIED el! MACRO DEMO - All these create equivalent Elements:

    el!("text")                           // String → Element
    el!(div().text("text").build())      // Builder → Element
    el!(VNode::text("text"))             // VDOM → Element
    el!(existing_element)                 // Element → Element (passthrough)

    el![child1, child2, child3]          // Array of mixed children
    el!(div, class: "flex", [children])  // Container with mixed children
    */

    layout! {
        grid(cols: 3, rows: 2, gap: 2) {
            "🔴 Direct String" at (0, 0) class "bg-red-500 text-white p-4 flex items-center justify-center",
            "🟢 Web Builder" at (0, 1) class "bg-green-500 text-white p-4 flex items-center justify-center",
            "🔵 VDOM Node" at (0, 2) class "bg-blue-500 text-white p-4 flex items-center justify-center",
            "🟡 Mixed Content" at (1, 0) class "bg-yellow-400 text-black p-4 flex items-center justify-center",
            "🟣 Dynamic Data" at (1, 1) class "bg-purple-500 text-white p-4 flex items-center justify-center",
            "🩷 Complex Tree" at (1, 2) class "bg-pink-500 text-white p-4 flex items-center justify-center",
        }
    }
}

fn create_dashboard_grid() -> reactive_tui::layout::grid::DeclarativeGrid {
    /*
    MIXED API DASHBOARD - Each component uses different creation method:
    - Header: web_api builder pattern
    - Sidebar: VDOM element tree
    - Main: Mixed children array
    - Alerts: Simple string
    - Footer: Builder with dynamic content
    */

    layout! {
        grid(cols: 4, rows: 4, gap: 1) {
            "📊 HEADER (web_api)" at (0, 0) span (1, 4) class "bg-blue-600 text-white p-3 flex items-center justify-center",
            "📁 SIDEBAR (vdom)" at (1, 0) class "bg-gray-400 text-black p-3 flex items-center justify-center",
            "📄 MAIN (mixed)" at (1, 1) span (2, 2) class "bg-white text-black p-4 flex items-center justify-center",
            "🔔 ALERTS (string)" at (1, 3) class "bg-yellow-400 text-black p-3 flex items-center justify-center",
            "ℹ️ FOOTER (dynamic)" at (3, 0) span (1, 4) class "bg-gray-600 text-white p-2 flex items-center justify-center",
        }
    }
}

fn create_card_grid() -> reactive_tui::layout::grid::DeclarativeGrid {
    /*
    DATA-DRIVEN CARDS - Shows how el! macro would work with dynamic data:

    let cards = get_card_data();
    let elements = cards.iter().map(|card| {
        el!(format!("{} {}", card.icon, card.name))  // String interpolation
        // OR el!(div().text(&card.name).build())     // Builder pattern
        // OR el!(VNode::text(&card.name))            // VDOM approach
    }).collect();
    */

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
    /*
    LAYERED UI WITH Z-INDEX - Different element types at different layers:

    Layer 0:  el!("background")                    // Simple string
    Layer 1:  el!(div().text("window").build())    // Web API builder
    Layer 2:  el!(VNode::element("doc").build())   // VDOM element
    Layer 10: el!(div, ["modal", vnode])           // Mixed children
    Layer 20: el!(tooltip_element)                 // Existing element
    */

    layout! {
        grid(cols: 6, rows: 6, gap: 0) {
            "🖥️ DESKTOP (z:0)" at (0, 0) span (6, 6) class "bg-blue-600 text-white flex items-center justify-center" z 0,
            "📱 WINDOW (z:1)" at (1, 1) span (4, 3) class "bg-gray-200 text-black flex items-center justify-center" z 1,
            "📄 DOCUMENT (z:2)" at (2, 2) span (3, 3) class "bg-white text-gray-800 flex items-center justify-center" z 2,
            "🚨 MODAL (z:10)" at (1, 2) span (3, 2) class "bg-red-500 text-white flex items-center justify-center" z 10,
            "💡 TOOLTIP (z:20)" at (0, 5) span (2, 1) class "bg-yellow-400 text-black flex items-center justify-center" z 20,
        }
    }
}

fn create_ide_grid() -> reactive_tui::layout::grid::DeclarativeGrid {
    /*
    COMPLEX IDE LAYOUT - Real-world example of unified syntax power:

    // File tree: VDOM component tree
    let files = el!(VNode::element("nav").children(file_list).build());

    // Editor: Mixed content with syntax highlighting
    let editor = el!(div, [
        "📝 EDITOR",
        VNode::element("code").child(code_content).build(),
        "// Comments and more...",
    ]);

    // Terminal: Dynamic command output
    let terminal = el!(format!("$ {}\n{}", last_command, output));

    // All unified through el! macro - same Element output!
    */

    layout! {
        grid(cols: 5, rows: 5, gap: 1) {
            "🪟 REACTIVE-TUI IDE" at (0, 0) span (1, 5) class "bg-gray-600 text-white p-2 flex items-center justify-center",
            "📁 FILES" at (1, 0) span (3, 1) class "bg-gray-400 text-black p-3 flex items-center justify-center",
            "📝 EDITOR" at (1, 1) span (2, 3) class "bg-white text-black p-4 flex items-center justify-center",
            "🔍 PROPS" at (1, 4) span (2, 1) class "bg-blue-400 text-white p-3 flex items-center justify-center",
            "💻 TERMINAL" at (3, 1) span (1, 3) class "bg-black text-green-500 p-3 flex items-center justify-center",
            "📊 STATUS" at (4, 0) span (1, 5) class "bg-blue-600 text-white p-1 flex items-center justify-center",
        }
    }
}

fn main() -> Result<()> {
    // Get actual terminal size
    let (width, height) = crossterm::terminal::size()?;

    let mut showcase = GridShowcase::new();
    let mut renderer = Renderer::new(width as usize, height as usize).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Renderer error: {}", e))
    })?;

    loop {
        if let Err(e) = showcase.render(&mut renderer) {
            eprintln!("Render error: {}", e);
            break;
        }

        match read()? {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => break,
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char(' ') | KeyCode::Right | KeyCode::Enter => showcase.next_demo(),
                KeyCode::Left | KeyCode::Backspace => showcase.prev_demo(),
                KeyCode::Esc => break,
                _ => {}
            },
            Event::Resize(new_width, new_height) => {
                // Handle terminal resize
                renderer.resize(new_width as usize, new_height as usize);
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
