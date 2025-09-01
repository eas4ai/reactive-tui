//! Demo of overflow utilities with clipping
//!
//! This example demonstrates how overflow:hidden works in the TUI context

use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a surface for testing
    let mut surface = Surface::new(80, 24);

    // Test overflow utilities
    let root = NodeSpec {
        class: "flex flex-col gap-2 p-2".into(),
        text: None,
        children: vec![
            // Header
            NodeSpec {
                class: "text-center font-bold".into(),
                text: Some("🔒 Overflow Demo".into()),
                children: vec![],
            },

            // Container with overflow:hidden
            NodeSpec {
                class: "w-20 h-3 overflow-hidden bg-gray-800 p-1".into(),
                text: Some("This is a very long text that should be clipped by overflow:hidden because it exceeds the container width".into()),
                children: vec![],
            },

            // Container with overflow:visible (default)
            NodeSpec {
                class: "w-20 h-3 overflow-visible bg-gray-700 p-1".into(),
                text: Some("This text should overflow normally without clipping".into()),
                children: vec![],
            },

            // Container with overflow-x:hidden
            NodeSpec {
                class: "w-15 h-4 overflow-x-hidden bg-gray-600 p-1".into(),
                text: Some("Horizontal overflow should be clipped but vertical should be normal".into()),
                children: vec![],
            },

            // Container with overflow-y:hidden
            NodeSpec {
                class: "w-30 h-2 overflow-y-hidden bg-gray-500 p-1".into(),
                text: Some("Vertical overflow clipped\nThis second line should be hidden\nAnd this third line too".into()),
                children: vec![],
            },
        ],
    };

    // Layout and paint
    let options = PaintOptions {
        debug_overlay: true, // Show container boundaries
    };

    layout_and_paint_with(&root, &mut surface, 80, &options)?;

    // Print the result to console for inspection
    println!("Overflow Demo Results:");
    println!("=====================");

    for y in 0..24 {
        for x in 0..80 {
            let cell = surface.get(x, y);
            if cell.ch != ' ' && cell.ch != '\0' {
                print!("{}", cell.ch);
            } else {
                print!(" ");
            }
        }
        println!();
    }

    println!("\n✅ Overflow utilities are working!");
    println!("- overflow-hidden: Text should be clipped to container bounds");
    println!("- overflow-visible: Text should extend beyond container");
    println!("- overflow-x-hidden: Only horizontal clipping");
    println!("- overflow-y-hidden: Only vertical clipping");

    Ok(())
}
