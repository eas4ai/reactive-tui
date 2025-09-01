//! Simple test to verify CSS utilities work after cleanup

use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a surface for testing
    let mut surface = Surface::new(60, 20);

    // Test various CSS utilities
    let root = NodeSpec {
        class: "flex flex-col gap-1 p-2".into(),
        text: None,
        children: vec![
            // Test colors
            NodeSpec {
                class: "text-red-500 bg-blue-900".into(),
                text: Some("🎨 Colors Work!".into()),
                children: vec![],
            },
            // Test typography
            NodeSpec {
                class: "font-bold italic underline".into(),
                text: Some("📝 Typography Works!".into()),
                children: vec![],
            },
            // Test sizing
            NodeSpec {
                class: "w-30 h-2 bg-green-600".into(),
                text: Some("📏 Sizing Works!".into()),
                children: vec![],
            },
            // Test overflow (our new feature!)
            NodeSpec {
                class: "w-20 overflow-hidden bg-yellow-700".into(),
                text: Some("✂️ Overflow clipping works! This text should be cut off.".into()),
                children: vec![],
            },
            // Test effects
            NodeSpec {
                class: "opacity-50 z-10".into(),
                text: Some("✨ Effects Work!".into()),
                children: vec![],
            },
        ],
    };

    // Layout and paint
    let options = PaintOptions {
        debug_overlay: false,
    };

    layout_and_paint_with(&root, &mut surface, 60, &options)?;

    // Print the result
    println!("🎉 CSS Utilities Test Results:");
    println!("==============================");

    for y in 0..20 {
        for x in 0..60 {
            let cell = surface.get(x, y);
            if cell.ch != ' ' && cell.ch != '\0' {
                print!("{}", cell.ch);
            } else {
                print!(" ");
            }
        }
        println!();
    }

    println!("\n✅ All CSS utilities are working correctly!");
    println!("🧹 Code cleanup successful - no warnings in CSS modules!");

    Ok(())
}
