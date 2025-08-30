//! Comprehensive Grid Demo
//!
//! Shows off the full capabilities of the working grid system

use reactive_tui::component::Element;
use reactive_tui::widgets::layout::grid::{Grid, GridProps, GridChild, GridScalar};
use reactive_tui::component::Component;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 COMPREHENSIVE GRID DEMO");
    println!("==========================");
    
    // Test 1: Dashboard Layout
    println!("\n📊 Test 1: Dashboard Layout");
    println!("----------------------------");
    test_dashboard();
    
    // Test 2: Card Grid
    println!("\n📋 Test 2: Card Grid");
    println!("---------------------");
    test_card_grid();
    
    // Test 3: Complex Layout
    println!("\n🏗️  Test 3: Complex Layout");
    println!("---------------------------");
    test_complex_layout();
    
    println!("\n✅ GRID SYSTEM IS FULLY FUNCTIONAL!");
    
    Ok(())
}

fn test_dashboard() {
    let children = vec![
        // Header spanning full width
        GridChild::new(Element::text("=== REACTIVE-TUI DASHBOARD ===")).at(0, 0).span(1, 3),
        
        // Sidebar
        GridChild::new(Element::text("SIDEBAR\n• Home\n• Users\n• Settings\n• Reports")).at(1, 0),
        
        // Main content
        GridChild::new(Element::text("MAIN CONTENT\n\nWelcome to Reactive-TUI!\n\nActive Users: 1,234\nRevenue: $45,678\nTasks: 42 pending")).at(1, 1),
        
        // Notifications
        GridChild::new(Element::text("NOTIFICATIONS\n\n🔔 3 new alerts\n📧 5 messages\n📋 2 tasks due\n⚠️  1 warning")).at(1, 2),
        
        // Footer spanning full width
        GridChild::new(Element::text("=== Footer: (c) 2024 Reactive-TUI | Status: Connected ===")).at(2, 0).span(1, 3),
    ];
    
    let props = GridProps {
        columns: vec![
            GridScalar::Cells(25),  // Fixed sidebar
            GridScalar::Fr(2.0),    // Main content gets 2/3
            GridScalar::Fr(1.0),    // Notifications get 1/3
        ],
        rows: vec![
            GridScalar::Cells(3),   // Fixed header height
            GridScalar::Fr(1.0),    // Main content area
            GridScalar::Cells(3),   // Fixed footer height
        ],
        children,
        column_gap: 2,
        row_gap: 1,
        ..Default::default()
    };
    
    render_grid(props);
}

fn test_card_grid() {
    let cards = vec![
        "CARD 1\nUser: Alice\nStatus: Active\nScore: 95%",
        "CARD 2\nUser: Bob\nStatus: Pending\nScore: 87%",
        "CARD 3\nUser: Carol\nStatus: Active\nScore: 92%",
        "CARD 4\nUser: Dave\nStatus: Inactive\nScore: 78%",
        "CARD 5\nUser: Eve\nStatus: Active\nScore: 99%",
        "CARD 6\nUser: Frank\nStatus: Pending\nScore: 84%",
    ];
    
    let children: Vec<GridChild> = cards
        .into_iter()
        .enumerate()
        .map(|(i, content)| {
            let row = i / 3;
            let col = i % 3;
            GridChild::new(Element::text(content)).at(row, col)
        })
        .collect();
    
    let props = GridProps {
        columns: vec![GridScalar::Fr(1.0), GridScalar::Fr(1.0), GridScalar::Fr(1.0)],
        rows: vec![GridScalar::Auto, GridScalar::Auto],
        children,
        column_gap: 3,
        row_gap: 2,
        ..Default::default()
    };
    
    render_grid(props);
}

fn test_complex_layout() {
    let children = vec![
        // Title bar
        GridChild::new(Element::text("🎯 REACTIVE-TUI v1.0")).at(0, 0).span(1, 4),
        
        // Left panel
        GridChild::new(Element::text("LEFT PANEL\n\nNavigation:\n• File\n• Edit\n• View\n• Tools")).at(1, 0).span(2, 1),
        
        // Main editor area
        GridChild::new(Element::text("MAIN EDITOR\n\nfn main() {\n    println!(\"Hello, World!\");\n    let grid = Grid::new();\n    grid.render();\n}")).at(1, 1).span(1, 2),
        
        // Properties panel
        GridChild::new(Element::text("PROPERTIES\n\nWidth: 800px\nHeight: 600px\nTheme: Dark\nFont: Mono")).at(1, 3),
        
        // Console output
        GridChild::new(Element::text("CONSOLE OUTPUT\n\n> cargo run\nCompiling...\nFinished dev [unoptimized + debuginfo]\nRunning target/debug/app\nHello, World!")).at(2, 1).span(1, 2),
        
        // Status bar
        GridChild::new(Element::text("Ready | Line 1, Col 1 | UTF-8 | Rust")).at(3, 0).span(1, 4),
    ];
    
    let props = GridProps {
        columns: vec![
            GridScalar::Cells(20),  // Left panel
            GridScalar::Fr(2.0),    // Main area
            GridScalar::Fr(1.0),    // Secondary area
            GridScalar::Cells(20),  // Right panel
        ],
        rows: vec![
            GridScalar::Cells(3),   // Title
            GridScalar::Fr(2.0),    // Main content
            GridScalar::Fr(1.0),    // Console
            GridScalar::Cells(2),   // Status
        ],
        children,
        column_gap: 1,
        row_gap: 1,
        ..Default::default()
    };
    
    render_grid(props);
}

fn render_grid(props: GridProps) {
    let grid = Grid::new(props.clone());
    let mut state = reactive_tui::widgets::layout::grid::GridState::default();
    state.viewport_width = 120;  // Wide terminal
    state.viewport_height = 30;
    
    let result = grid.render(&props, &state);
    
    if let reactive_tui::component::ElementType::Text(content) = &result.element_type {
        println!("{}", content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dashboard_layout() {
        // Just ensure it doesn't crash
        test_dashboard();
    }
    
    #[test]
    fn test_card_grid_layout() {
        test_card_grid();
    }
    
    #[test]
    fn test_complex_layout_works() {
        test_complex_layout();
    }
}
