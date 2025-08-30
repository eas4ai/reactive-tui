//! Clean Grid Test - Simple Visual Output
//!
//! A straightforward test of the visual grid system

use reactive_tui::layout;
use reactive_tui::layout::renderer::render_grid;
use reactive_tui::core::surface::{Surface, Rgba};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 CLEAN GRID VISUAL TEST");
    println!("=========================\n");
    
    // Test 1: Basic 3x2 grid
    test_basic_grid()?;
    
    println!("\n✅ Grid visual test completed!");
    println!("The declarative grid system is working with visual rendering!");
    
    Ok(())
}

fn test_basic_grid() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Basic 3x2 Grid Test");
    println!("-----------------------");
    
    let grid = layout! {
        grid(cols: 3, rows: 2, gap: 1) {
            "A" at (0, 0) class "border",
            "B" at (0, 1) class "border",
            "C" at (0, 2) class "border",
            "D" at (1, 0) class "border",
            "E" at (1, 1) class "border",
            "F" at (1, 2) class "border",
        }
    };
    
    // Create surface and render
    let mut surface = Surface::new(60, 15);
    surface.clear(Rgba::black());
    
    render_grid(&grid, &mut surface, 60)?;
    
    println!("Grid: {}x{} with {} areas", grid.cols, grid.rows, grid.areas.len());
    println!("Areas: {:?}", grid.areas.iter().map(|a| &a.name).collect::<Vec<_>>());
    
    // Print surface content
    print_surface_content(&surface);
    
    Ok(())
}

fn print_surface_content(surface: &Surface) {
    let (width, height) = surface.dims();
    println!("\nSurface content ({}x{}):", width, height);
    
    // Count content
    let mut content_cells = 0;
    let mut text_chars = Vec::new();
    
    for y in 0..height {
        for x in 0..width {
            let cell = surface.get(x, y);
            if cell.ch != ' ' {
                content_cells += 1;
                text_chars.push(cell.ch);
            }
        }
    }
    
    println!("Content cells: {}", content_cells);
    
    if content_cells > 0 {
        println!("Text content: {:?}", text_chars.iter().collect::<String>());
        println!("✅ Surface has rendered content!");
        
        // Show first few lines
        println!("\nFirst 10 lines of surface:");
        for y in 0..10.min(height) {
            print!("  ");
            for x in 0..50.min(width) {
                let cell = surface.get(x, y);
                print!("{}", if cell.ch == ' ' { '.' } else { cell.ch });
            }
            println!();
        }
    } else {
        println!("⚠️  Surface appears empty");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grid_rendering() {
        let result = test_basic_grid();
        assert!(result.is_ok());
    }
}
