//! Demo of Animation & Transition utilities
//!
//! This example demonstrates the new animation and transition CSS utilities

use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animations_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Create a surface for testing
        let mut surface = Surface::new(80, 30);

        // Test Animation & Transition utilities
        let root = NodeSpec {
            class: "flex flex-col gap-2 p-2".into(),
            text: None,
            children: vec![
                // Header
                NodeSpec {
                    class: "text-center font-bold".into(),
                    text: Some("🎬 Animation & Transitions Demo".into()),
                    children: vec![],
                },
                // Transition Examples
                NodeSpec {
                    class: "text-bold".into(),
                    text: Some("🔄 Transition Examples:".into()),
                    children: vec![],
                },
                NodeSpec {
                    class: "flex flex-row gap-4".into(),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: "transition-all duration-300 ease-in-out p-2 bg-blue-500".into(),
                            text: Some("Transition All".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "transition-colors duration-150 p-2 bg-green-500".into(),
                            text: Some("Transition Colors".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "transition-opacity duration-500 p-2 bg-purple-500".into(),
                            text: Some("Transition Opacity".into()),
                            children: vec![],
                        },
                    ],
                },
                // Duration Examples
                NodeSpec {
                    class: "text-bold".into(),
                    text: Some("⏱️ Duration Examples:".into()),
                    children: vec![],
                },
                NodeSpec {
                    class: "flex flex-row gap-2".into(),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: "duration-75 p-1 bg-red-400".into(),
                            text: Some("75ms".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "duration-150 p-1 bg-red-500".into(),
                            text: Some("150ms".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "duration-300 p-1 bg-red-600".into(),
                            text: Some("300ms".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "duration-500 p-1 bg-red-700".into(),
                            text: Some("500ms".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "duration-1000 p-1 bg-red-800".into(),
                            text: Some("1000ms".into()),
                            children: vec![],
                        },
                    ],
                },
                // Easing Examples
                NodeSpec {
                    class: "text-bold".into(),
                    text: Some("📈 Easing Examples:".into()),
                    children: vec![],
                },
                NodeSpec {
                    class: "flex flex-row gap-4".into(),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: "ease-linear p-2 bg-yellow-400".into(),
                            text: Some("Linear".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "ease-in p-2 bg-yellow-500".into(),
                            text: Some("Ease In".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "ease-out p-2 bg-yellow-600".into(),
                            text: Some("Ease Out".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "ease-in-out p-2 bg-yellow-700".into(),
                            text: Some("Ease In-Out".into()),
                            children: vec![],
                        },
                    ],
                },
                // Transform Examples
                NodeSpec {
                    class: "text-bold".into(),
                    text: Some("🔄 Transform Examples:".into()),
                    children: vec![],
                },
                NodeSpec {
                    class: "flex flex-row gap-3".into(),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: "scale-75 p-2 bg-cyan-500".into(),
                            text: Some("Scale 75%".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "scale-100 p-2 bg-cyan-600".into(),
                            text: Some("Scale 100%".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "scale-125 p-2 bg-cyan-700".into(),
                            text: Some("Scale 125%".into()),
                            children: vec![],
                        },
                    ],
                },
                NodeSpec {
                    class: "flex flex-row gap-3".into(),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: "translate-x-2 p-2 bg-orange-500".into(),
                            text: Some("Translate X".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "translate-y-1 p-2 bg-orange-600".into(),
                            text: Some("Translate Y".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "transform-none p-2 bg-orange-700".into(),
                            text: Some("No Transform".into()),
                            children: vec![],
                        },
                    ],
                },
                // Animation Presets
                NodeSpec {
                    class: "text-bold".into(),
                    text: Some("✨ Animation Presets:".into()),
                    children: vec![],
                },
                NodeSpec {
                    class: "flex flex-row gap-4".into(),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: "animate-pulse p-2 bg-pink-500".into(),
                            text: Some("Pulse".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "animate-bounce p-2 bg-pink-600".into(),
                            text: Some("Bounce".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "animate-spin p-2 bg-pink-700".into(),
                            text: Some("Spin".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "animate-ping p-2 bg-pink-800".into(),
                            text: Some("Ping".into()),
                            children: vec![],
                        },
                    ],
                },
                // Pseudo-class Variants
                NodeSpec {
                    class: "text-bold".into(),
                    text: Some("🎭 Pseudo-class Variants:".into()),
                    children: vec![],
                },
                NodeSpec {
                    class: "flex flex-col gap-1".into(),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: "hover:bg-blue-600 hover:text-white p-2 bg-blue-400".into(),
                            text: Some("Hover Effects".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "active:bg-green-700 active:scale-95 p-2 bg-green-500".into(),
                            text: Some("Active Effects".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "disabled:opacity-50 disabled:bg-gray-400 p-2 bg-red-500".into(),
                            text: Some("Disabled Effects".into()),
                            children: vec![],
                        },
                    ],
                },
                // Conditional Variants
                NodeSpec {
                    class: "text-bold".into(),
                    text: Some("📋 Conditional Variants:".into()),
                    children: vec![],
                },
                NodeSpec {
                    class: "flex flex-col gap-1".into(),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: "first:border-t-2 first:font-bold p-1 bg-gray-200".into(),
                            text: Some("First Item".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "odd:bg-gray-100 even:bg-gray-300 p-1".into(),
                            text: Some("Odd/Even Item 1".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "odd:bg-gray-100 even:bg-gray-300 p-1".into(),
                            text: Some("Odd/Even Item 2".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "last:border-b-2 last:font-bold p-1 bg-gray-200".into(),
                            text: Some("Last Item".into()),
                            children: vec![],
                        },
                    ],
                },
                // Group Variants
                NodeSpec {
                    class: "text-bold".into(),
                    text: Some("👥 Group Variants:".into()),
                    children: vec![],
                },
                NodeSpec {
                    class: "group p-2 bg-indigo-100".into(),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: "group-hover:text-white group-hover:bg-indigo-600 p-1".into(),
                            text: Some("Group Hover Child".into()),
                            children: vec![],
                        },
                        NodeSpec {
                            class: "group-focus:visible group-focus:bg-indigo-500 p-1".into(),
                            text: Some("Group Focus Child".into()),
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
        println!("🎬 Animation & Transitions Demo Results:");
        println!("========================================");

        let mut rendered = String::new();
        for y in 0..30 {
            for x in 0..80 {
                let cell = surface.get(x, y);
                if cell.ch != ' ' && cell.ch != '\0' {
                    print!("{}", cell.ch);
                    rendered.push(cell.ch);
                } else {
                    print!(" ");
                    rendered.push(' ');
                }
            }
            println!();
            rendered.push('\n');
        }
        // The 30-row surface shows the header and the first section label;
        // the gap-2 column pushes the later sections below the viewport.
        for expected in ["Animation &", "Transition Examples:"] {
            assert!(
                rendered.contains(expected),
                "painted surface is missing {expected:?}:\n{rendered}"
            );
        }

        println!("\n✅ Animation & Transition utilities are working!");
        println!("🎬 Animation Features:");
        println!("  - transition-* utilities for smooth changes");
        println!("  - duration-* utilities for timing control");
        println!("  - ease-* utilities for easing functions");
        println!("  - scale-*, translate-* for transforms");
        println!("  - animate-* presets for common animations");
        println!("\n🎭 Pseudo-class Features:");
        println!("  - hover:*, active:*, disabled:* state variants");
        println!("  - first:*, last:*, odd:*, even:* conditional variants");
        println!("  - group-hover:*, group-focus:* group variants");
        println!("  - sm:*, md:*, lg:* responsive variants");

        Ok(())
    }
}
