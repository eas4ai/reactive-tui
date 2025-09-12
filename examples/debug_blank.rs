
use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};
use std::borrow::Cow;

fn main() -> reactive_tui::Result<()> {
    // Test that layout_showcase's CSS actually works
    let mut surface = Surface::new(80, 20);
    
    // First test with simple working CSS
    eprintln!("\n=== Test 1: Simple CSS that should work ===");
    let simple_spec = NodeSpec {
        class: Cow::Borrowed("w-40 h-10"),
        text: Some(Cow::Borrowed("SIMPLE TEST")),
        children: vec![],
    };
    
    let result1 = layout_and_paint_with(&simple_spec, &mut surface, 80, &PaintOptions::default());
    eprintln!("Simple test result: {:?}", result1);
    
    // Check simple test output
    for y in 0..3 {
        eprint!("  ");
        for x in 0..20 {
            let cell = surface.get_at(reactive_tui::core::geometry::Point::new(x, y));
            eprint!("{}", cell.ch);
        }
        eprintln!();
    }
    
    // Clear for next test
    surface.clear(reactive_tui::core::surface::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
    
    // Test with fixed heights instead of percentages
    eprintln!("\n=== Test 2: Fixed heights instead of h-screen ===");
    let fixed_spec = NodeSpec {
        class: Cow::Borrowed("h-20 w-60 flex flex-col"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed("h-5"),
                text: Some(Cow::Borrowed("HEADER")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("flex-1"),
                text: Some(Cow::Borrowed("MAIN CONTENT")),
                children: vec![],
            },
        ],
    };
    
    let result2 = layout_and_paint_with(&fixed_spec, &mut surface, 80, &PaintOptions::default());
    eprintln!("Fixed heights result: {:?}", result2);
    
    // Check output
    for y in 0..8 {
        eprint!("  ");
        for x in 0..30 {
            let cell = surface.get_at(reactive_tui::core::geometry::Point::new(x, y));
            eprint!("{}", cell.ch);
        }
        eprintln!();
    }
    
    surface.clear(reactive_tui::core::surface::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
    
    // Now test original layout_showcase CSS with width added
    eprintln!("\n=== Test 3: Original layout_showcase CSS with w-full ===");
    let spec = NodeSpec {
        class: Cow::Borrowed("h-screen w-full flex flex-col bg-gray-100"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed("h-16 flex items-center justify-center bg-blue-600 text-white"),
                text: Some(Cow::Borrowed("HEADER")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("flex-1 bg-green-500"),
                text: Some(Cow::Borrowed("MAIN CONTENT")),
                children: vec![],
            },
        ],
    };
    
    eprintln!("Testing layout_showcase CSS classes...");
    let result = layout_and_paint_with(&spec, &mut surface, 80, &PaintOptions::default());
    
    match result {
        Ok(()) => {
            eprintln!("✅ Layout succeeded!");
            
            // Check what was painted - scan entire surface to find content
            let mut has_content = false;
            let mut first_content = None;
            for y in 0..20 {
                for x in 0..80 {
                    let cell = surface.get_at(reactive_tui::core::geometry::Point::new(x, y));
                    if cell.ch != ' ' && cell.ch != '\0' {
                        has_content = true;
                        if first_content.is_none() {
                            first_content = Some((x, y, cell.ch));
                        }
                    }
                }
            }
            
            if let Some((x, y, ch)) = first_content {
                eprintln!("First non-space character '{}' found at ({}, {})", ch, x, y);
                
                // Show area around first content
                eprintln!("\nArea around first content:");
                for dy in 0..5 {
                    eprint!("Line {}: |", y + dy);
                    for dx in 0..40 {
                        let px = x.saturating_add(dx).min(79);
                        let py = y.saturating_add(dy).min(19);
                        let cell = surface.get_at(reactive_tui::core::geometry::Point::new(px, py));
                        eprint!("{}", cell.ch);
                    }
                    eprintln!("|");
                }
            } else {
                eprintln!("No content found in entire 80x20 surface!");
            }
            
            if has_content {
                eprintln!("✅ Surface has content!");
            } else {
                eprintln!("❌ Surface is blank!");
            }
        },
        Err(e) => {
            eprintln!("❌ Layout failed: {}", e);
        }
    }
    
    Ok(())
}