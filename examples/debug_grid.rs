//! Debug Grid Test - Check what's happening with grid rendering

use reactive_tui::layout;
use reactive_tui::layout::renderer::render_grid;
use reactive_tui::core::surface::{Surface, Rgba};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 DEBUG GRID TEST");
    println!("==================\n");
    
    // Create a simple 2x2 grid
    let grid = layout! {
        grid(cols: 2, rows: 2, gap: 2) {
            "A" at (0, 0) class "bg-red-500 text-white",
            "B" at (0, 1) class "bg-green-500 text-white",
            "C" at (1, 0) class "bg-blue-500 text-white",
            "D" at (1, 1) class "bg-yellow-500 text-black",
        }
    };
    
    println!("Grid created:");
    println!("  Cols: {}, Rows: {}, Gap: {}", grid.cols, grid.rows, grid.gap);
    println!("  Areas: {}", grid.areas.len());
    
    for (i, area) in grid.areas.iter().enumerate() {
        println!("  Area {}: '{}' at ({},{}) span ({},{}) z:{} class:{:?}",
            i, area.name, area.row, area.col, area.row_span, area.col_span,
            area.z_index, area.css_class);
    }

    // Debug: Check what NodeSpec is generated
    println!("\nDebugging NodeSpec generation...");
    let node_spec = reactive_tui::layout::renderer::grid_to_node_spec(&grid);
    println!("Grid container class: '{}'", node_spec.class);
    println!("Grid children: {}", node_spec.children.len());

    for (i, child) in node_spec.children.iter().enumerate() {
        println!("  Child {}: class='{}' text={:?}", i, child.class, child.text);
    }

    // Debug: Test simple flexbox layout instead of grid
    println!("\nTesting simple flexbox layout...");
    let simple_node = reactive_tui::layout::paint_tree::NodeSpec {
        class: std::borrow::Cow::Borrowed("flex flex-row gap-4 w-400 h-200"),
        text: None,
        children: vec![
            reactive_tui::layout::paint_tree::NodeSpec {
                class: std::borrow::Cow::Borrowed("bg-red-500 text-white p-4 flex items-center justify-center min-w-80 min-h-40"),
                text: Some("A".to_string().into()),
                children: vec![],
            },
            reactive_tui::layout::paint_tree::NodeSpec {
                class: std::borrow::Cow::Borrowed("bg-green-500 text-white p-4 flex items-center justify-center min-w-80 min-h-40"),
                text: Some("B".to_string().into()),
                children: vec![],
            },
        ],
    };

    let mut flex_surface = Surface::new(80, 20);
    flex_surface.clear(Rgba::black());

    match reactive_tui::layout::paint_tree::layout_and_paint_with(&simple_node, &mut flex_surface, 80, &reactive_tui::layout::paint_tree::PaintOptions::default()) {
        Ok(()) => {
            println!("✅ Flexbox render successful!");
            print_surface_debug(&flex_surface);
        }
        Err(e) => {
            println!("❌ Flexbox render failed: {}", e);
        }
    }
    
    // Create surface and render
    let mut surface = Surface::new(80, 20);
    surface.clear(Rgba::black());
    
    println!("\nRendering to surface...");
    match render_grid(&grid, &mut surface, 80) {
        Ok(()) => {
            println!("✅ Render successful!");
            
            // Print surface content
            print_surface_debug(&surface);
        }
        Err(e) => {
            println!("❌ Render failed: {}", e);
        }
    }
    
    Ok(())
}

fn print_surface_debug(surface: &Surface) {
    let (width, height) = surface.dims();
    println!("\nSurface analysis ({}x{}):", width, height);
    
    // Count content by type
    let mut text_chars = Vec::new();
    let mut colored_cells = 0;
    let mut total_content = 0;
    
    for y in 0..height {
        for x in 0..width {
            let cell = surface.get(x, y);
            if cell.ch != ' ' {
                text_chars.push((x, y, cell.ch));
                total_content += 1;
            }
            if cell.bg != Rgba::black() {
                colored_cells += 1;
            }
        }
    }
    
    println!("  Text characters: {}", text_chars.len());
    println!("  Colored cells: {}", colored_cells);
    println!("  Total content: {}", total_content);
    
    if !text_chars.is_empty() {
        println!("\nText character positions:");
        for (x, y, ch) in text_chars.iter().take(20) {  // Show first 20
            println!("  '{}' at ({}, {})", ch, x, y);
        }
    }
    
    // Show first 15 lines of surface
    println!("\nSurface content (first 15 lines):");
    for y in 0..15.min(height) {
        print!("  {:2}: ", y);
        for x in 0..60.min(width) {
            let cell = surface.get(x, y);
            if cell.ch == ' ' && cell.bg == Rgba::black() {
                print!(".");
            } else if cell.ch == ' ' {
                print!("█");  // Colored background
            } else {
                print!("{}", cell.ch);
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_grid() {
        let result = main();
        assert!(result.is_ok());
    }
}
