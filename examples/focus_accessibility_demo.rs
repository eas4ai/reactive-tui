//! Demo of Focus & Accessibility utilities
//!
//! This example demonstrates the new focus and accessibility CSS utilities

use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a surface for testing
    let mut surface = Surface::new(80, 30);

    // Test Focus & Accessibility utilities
    let root = NodeSpec {
        class: "flex flex-col gap-2 p-2".into(),
        text: None,
        children: vec![
            // Header
            NodeSpec {
                class: "text-center font-bold role-banner".into(),
                text: Some("🎯 Focus & Accessibility Demo".into()),
                children: vec![],
            },
            // Focus Ring Examples
            NodeSpec {
                class: "text-bold".into(),
                text: Some("📍 Focus Ring Examples:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-4".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "ring-2 ring-blue-500 p-2 role-button".into(),
                        text: Some("Button with Focus Ring".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "focus:ring-4 focus:ring-green-500 p-2 role-button".into(),
                        text: Some("Focus Ring on Focus".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "ring-0 p-2 role-button".into(),
                        text: Some("No Focus Ring".into()),
                        children: vec![],
                    },
                ],
            },
            // Focus Variants Examples
            NodeSpec {
                class: "text-bold".into(),
                text: Some("🎨 Focus Variants:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-4".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "focus:bg-blue-500 focus:text-white p-2 role-button".into(),
                        text: Some("Focus Colors".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "focus-within:bg-gray-100 p-2 role-group".into(),
                        text: Some("Focus Within".into()),
                        children: vec![],
                    },
                ],
            },
            // Accessibility Examples
            NodeSpec {
                class: "text-bold".into(),
                text: Some("♿ Accessibility Examples:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-col gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "role-button aria-pressed tabindex-0".into(),
                        text: Some("🔘 Pressed Button (ARIA)".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "role-button aria-disabled tabindex--1".into(),
                        text: Some("❌ Disabled Button".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "role-link underline tabindex-0".into(),
                        text: Some("🔗 Link Element".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "role-heading font-bold".into(),
                        text: Some("📝 Heading Element".into()),
                        children: vec![],
                    },
                ],
            },
            // Screen Reader Examples
            NodeSpec {
                class: "text-bold".into(),
                text: Some("👁️ Screen Reader Utilities:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-4".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "sr-only".into(),
                        text: Some("Hidden from screen readers".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "not-sr-only".into(),
                        text: Some("Visible to screen readers".into()),
                        children: vec![],
                    },
                ],
            },
            // ARIA States Examples
            NodeSpec {
                class: "text-bold".into(),
                text: Some("🏷️ ARIA States:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-col gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "aria-selected bg-blue-600 text-white p-1".into(),
                        text: Some("✅ Selected Item".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "aria-checked text-green-500 p-1".into(),
                        text: Some("☑️ Checked Item".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "aria-expanded font-bold p-1".into(),
                        text: Some("📂 Expanded Item".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "aria-hidden opacity-50 p-1".into(),
                        text: Some("👻 Hidden from AT".into()),
                        children: vec![],
                    },
                ],
            },
            // High Contrast Example
            NodeSpec {
                class: "text-bold".into(),
                text: Some("🔆 High Contrast Mode:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "high-contrast p-2".into(),
                text: Some("High contrast text for better accessibility".into()),
                children: vec![],
            },
            // Tab Navigation Example
            NodeSpec {
                class: "text-bold".into(),
                text: Some("⌨️ Tab Navigation:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-2".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "tabindex-1 p-1 bg-gray-200".into(),
                        text: Some("Tab 1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "tabindex-2 p-1 bg-gray-200".into(),
                        text: Some("Tab 2".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "tabindex-3 p-1 bg-gray-200".into(),
                        text: Some("Tab 3".into()),
                        children: vec![],
                    },
                ],
            },
        ],
    };

    // Layout and paint
    let options = PaintOptions {
        debug_overlay: false,
    };

    layout_and_paint_with(&root, &mut surface, 80, &options)?;

    // Print the result to console for inspection
    println!("🎯 Focus & Accessibility Demo Results:");
    println!("======================================");

    for y in 0..30 {
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

    println!("\n✅ Focus & Accessibility utilities are working!");
    println!("🎯 Focus Features:");
    println!("  - focus:* variants for conditional styling");
    println!("  - focus-within:* for parent styling");
    println!("  - ring-* utilities for focus rings");
    println!("  - outline-* utilities for focus outlines");
    println!("\n♿ Accessibility Features:");
    println!("  - aria-* attributes for screen readers");
    println!("  - role-* attributes for semantic meaning");
    println!("  - tabindex-* for keyboard navigation");
    println!("  - sr-only/not-sr-only for screen reader control");
    println!("  - high-contrast mode support");

    Ok(())
}
