use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};
use std::borrow::Cow;

fn main() -> reactive_tui::Result<()> {
    let mut surface = Surface::new(40, 10);
    
    // Test 1: h-screen with width
    eprintln!("\n=== Test 1: h-screen with w-full ===");
    let spec1 = NodeSpec {
        class: Cow::Borrowed("h-screen w-full"),
        text: Some(Cow::Borrowed("H-SCREEN")),
        children: vec![],
    };
    
    layout_and_paint_with(&spec1, &mut surface, 40, &PaintOptions::default())?;
    
    // Check if anything rendered
    let mut found = false;
    for y in 0..10 {
        for x in 0..40 {
            let cell = surface.get_at(reactive_tui::core::geometry::Point::new(x, y));
            if cell.ch != ' ' && cell.ch != '\0' {
                if !found {
                    eprintln!("Found content at ({}, {}): '{}'", x, y, cell.ch);
                    found = true;
                }
            }
        }
    }
    if !found {
        eprintln!("No content found!");
    }
    
    // Test 2: h-10 (fixed height) with width
    eprintln!("\n=== Test 2: h-10 with w-20 (fixed) ===");
    surface.clear(reactive_tui::core::surface::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
    
    let spec2 = NodeSpec {
        class: Cow::Borrowed("h-10 w-20"),
        text: Some(Cow::Borrowed("H-10")),
        children: vec![],
    };
    
    layout_and_paint_with(&spec2, &mut surface, 40, &PaintOptions::default())?;
    
    found = false;
    for y in 0..10 {
        for x in 0..40 {
            let cell = surface.get_at(reactive_tui::core::geometry::Point::new(x, y));
            if cell.ch != ' ' && cell.ch != '\0' {
                if !found {
                    eprintln!("Found content at ({}, {}): '{}'", x, y, cell.ch);
                    found = true;
                }
            }
        }
    }
    if !found {
        eprintln!("No content found!");
    }
    
    // Test 3: h-full (100% height) with width
    eprintln!("\n=== Test 3: h-full with w-1/2 ===");
    surface.clear(reactive_tui::core::surface::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
    
    let spec3 = NodeSpec {
        class: Cow::Borrowed("h-full w-1/2"),
        text: Some(Cow::Borrowed("H-FULL")),
        children: vec![],
    };
    
    layout_and_paint_with(&spec3, &mut surface, 40, &PaintOptions::default())?;
    
    found = false;
    for y in 0..10 {
        for x in 0..40 {
            let cell = surface.get_at(reactive_tui::core::geometry::Point::new(x, y));
            if cell.ch != ' ' && cell.ch != '\0' {
                if !found {
                    eprintln!("Found content at ({}, {}): '{}'", x, y, cell.ch);
                    found = true;
                }
            }
        }
    }
    if !found {
        eprintln!("No content found!");
    }
    
    Ok(())
}