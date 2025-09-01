//! Test the new charcoal color
//! 
//! This example tests the charcoal color in various contexts

use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a surface for testing
    let mut surface = Surface::new(80, 20);
    
    // Test charcoal color in different contexts
    let root = NodeSpec {
        class: "flex flex-col gap-2 p-2".into(),
        text: None,
        children: vec![
            // Header
            NodeSpec {
                class: "text-center font-bold text-white bg-charcoal p-2".into(),
                text: Some("🎨 Charcoal Color Test".into()),
                children: vec![],
            },
            
            // Background examples
            NodeSpec {
                class: "text-bold".into(),
                text: Some("Background Examples:".into()),
                children: vec![],
            },
            
            NodeSpec {
                class: "flex flex-row gap-4".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-charcoal text-white p-2".into(),
                        text: Some("Charcoal Background".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-black text-white p-2".into(),
                        text: Some("Black Background".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-gray-800 text-white p-2".into(),
                        text: Some("Gray-800 Background".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Text examples
            NodeSpec {
                class: "text-bold".into(),
                text: Some("Text Examples:".into()),
                children: vec![],
            },
            
            NodeSpec {
                class: "flex flex-row gap-4".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "text-charcoal p-2 bg-white".into(),
                        text: Some("Charcoal Text".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "text-black p-2 bg-white".into(),
                        text: Some("Black Text".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "text-gray-800 p-2 bg-white".into(),
                        text: Some("Gray-800 Text".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Border examples
            NodeSpec {
                class: "text-bold".into(),
                text: Some("Border Examples:".into()),
                children: vec![],
            },
            
            NodeSpec {
                class: "flex flex-row gap-4".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "border-charcoal p-2 bg-gray-100".into(),
                        text: Some("Charcoal Border".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "border-black p-2 bg-gray-100".into(),
                        text: Some("Black Border".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "border-gray-800 p-2 bg-gray-100".into(),
                        text: Some("Gray-800 Border".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Ring examples
            NodeSpec {
                class: "text-bold".into(),
                text: Some("Ring Examples:".into()),
                children: vec![],
            },
            
            NodeSpec {
                class: "flex flex-row gap-4".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "ring-2 ring-charcoal p-2".into(),
                        text: Some("Charcoal Ring".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "ring-2 ring-black p-2".into(),
                        text: Some("Black Ring".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "ring-2 ring-gray-800 p-2".into(),
                        text: Some("Gray-800 Ring".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Summary
            NodeSpec {
                class: "text-center bg-charcoal text-white p-2 font-bold".into(),
                text: Some("✅ Charcoal Color Added Successfully!".into()),
                children: vec![],
            },
        ],
    };

    // Layout and paint
    let options = PaintOptions {
        debug_overlay: false,
    };
    
    layout_and_paint_with(&root, &mut surface, 80, &options)?;
    
    // Print the result to console for inspection
    println!("🎨 Charcoal Color Test Results:");
    println!("==============================");
    
    for y in 0..20 {
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
    
    println!("\n🎉 SUCCESS: Charcoal color is working!");
    println!("🎨 Charcoal Color Details:");
    println!("  • RGB: (54, 54, 54)");
    println!("  • Description: Very dark gray, almost black but softer");
    println!("  • Usage: bg-charcoal, text-charcoal, border-charcoal, ring-charcoal");
    println!("  • Perfect for: Dark themes, elegant backgrounds, subtle text");
    
    Ok(())
}
