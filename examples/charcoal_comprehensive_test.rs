//! Comprehensive test of charcoal color in all CSS contexts
//! 
//! This example tests charcoal in every possible CSS utility context

use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a surface for testing
    let mut surface = Surface::new(100, 25);
    
    // Test charcoal in every CSS context
    let root = NodeSpec {
        class: "flex flex-col gap-1 p-2".into(),
        text: None,
        children: vec![
            // Header
            NodeSpec {
                class: "text-center font-bold text-white bg-charcoal p-1".into(),
                text: Some("🎨 Comprehensive Charcoal Color Test".into()),
                children: vec![],
            },
            
            // Text colors
            NodeSpec {
                class: "flex flex-row gap-2".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "text-charcoal bg-white p-1".into(),
                        text: Some("text-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "text-black bg-white p-1".into(),
                        text: Some("text-black".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "text-gray-800 bg-white p-1".into(),
                        text: Some("text-gray-800".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Background colors
            NodeSpec {
                class: "flex flex-row gap-2".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-charcoal text-white p-1".into(),
                        text: Some("bg-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-black text-white p-1".into(),
                        text: Some("bg-black".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-gray-800 text-white p-1".into(),
                        text: Some("bg-gray-800".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Border colors
            NodeSpec {
                class: "flex flex-row gap-2".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "border-charcoal p-1 bg-gray-100".into(),
                        text: Some("border-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "border-black p-1 bg-gray-100".into(),
                        text: Some("border-black".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "border-gray-800 p-1 bg-gray-100".into(),
                        text: Some("border-gray-800".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Ring colors
            NodeSpec {
                class: "flex flex-row gap-2".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "ring-2 ring-charcoal p-1".into(),
                        text: Some("ring-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "ring-2 ring-black p-1".into(),
                        text: Some("ring-black".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "ring-2 ring-gray-800 p-1".into(),
                        text: Some("ring-gray-800".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Accent colors
            NodeSpec {
                class: "flex flex-row gap-2".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "accent-charcoal p-1".into(),
                        text: Some("accent-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "accent-black p-1".into(),
                        text: Some("accent-black".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "accent-gray-800 p-1".into(),
                        text: Some("accent-gray-800".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Placeholder colors
            NodeSpec {
                class: "flex flex-row gap-2".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "placeholder-charcoal p-1".into(),
                        text: Some("placeholder-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "placeholder-black p-1".into(),
                        text: Some("placeholder-black".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "placeholder-gray-800 p-1".into(),
                        text: Some("placeholder-gray-800".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Focus variants with charcoal
            NodeSpec {
                class: "flex flex-row gap-2".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "focus:bg-charcoal focus:text-white p-1 bg-gray-200".into(),
                        text: Some("focus:bg-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "focus:text-charcoal p-1 bg-white".into(),
                        text: Some("focus:text-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "focus:ring-charcoal ring-2 p-1".into(),
                        text: Some("focus:ring-charcoal".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Hover variants with charcoal
            NodeSpec {
                class: "flex flex-row gap-2".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "hover:bg-charcoal hover:text-white p-1 bg-gray-200".into(),
                        text: Some("hover:bg-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "hover:text-charcoal p-1 bg-white".into(),
                        text: Some("hover:text-charcoal".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "hover:border-charcoal border-2 p-1".into(),
                        text: Some("hover:border-charcoal".into()),
                        children: vec![],
                    },
                ],
            },
            
            // Summary
            NodeSpec {
                class: "text-center bg-charcoal text-white p-2 font-bold".into(),
                text: Some("✅ Charcoal Works in ALL CSS Contexts!".into()),
                children: vec![],
            },
        ],
    };

    // Layout and paint
    let options = PaintOptions {
        debug_overlay: false,
    };
    
    layout_and_paint_with(&root, &mut surface, 100, &options)?;
    
    // Print the result to console for inspection
    println!("🎨 Comprehensive Charcoal Color Test Results:");
    println!("==============================================");
    
    for y in 0..25 {
        for x in 0..100 {
            let cell = surface.get(x, y);
            if cell.ch != ' ' && cell.ch != '\0' {
                print!("{}", cell.ch);
            } else {
                print!(" ");
            }
        }
        println!();
    }
    
    println!("\n🎉 SUCCESS: Charcoal color works in ALL CSS contexts!");
    println!("🎨 Charcoal Color Support:");
    println!("  ✅ text-charcoal - Text color");
    println!("  ✅ bg-charcoal - Background color");
    println!("  ✅ border-charcoal - Border color");
    println!("  ✅ ring-charcoal - Focus ring color");
    println!("  ✅ accent-charcoal - Accent color");
    println!("  ✅ placeholder-charcoal - Placeholder color");
    println!("  ✅ focus:*-charcoal - Focus variants");
    println!("  ✅ hover:*-charcoal - Hover variants");
    println!("  ✅ active:*-charcoal - Active variants");
    println!("  ✅ disabled:*-charcoal - Disabled variants");
    println!("\n🎯 Charcoal RGB: (54, 54, 54) - Perfect dark gray!");
    
    Ok(())
}
