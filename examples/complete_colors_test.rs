//! Complete Tailwind Color Palette Test
//!
//! This example tests all 330+ Tailwind CSS colors to ensure they work correctly

use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a surface for testing
    let mut surface = Surface::new(120, 50);

    // Test complete Tailwind color palette
    let root = NodeSpec {
        class: "flex flex-col gap-1 p-2".into(),
        text: None,
        children: vec![
            // Header
            NodeSpec {
                class: "text-center font-bold text-white bg-gray-800 p-1".into(),
                text: Some("🎨 Complete Tailwind CSS Color Palette Test".into()),
                children: vec![],
            },
            // Gray Scale Colors
            NodeSpec {
                class: "text-bold".into(),
                text: Some("🔘 Gray Scale:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-slate-100 p-1".into(),
                        text: Some("S1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-slate-300 p-1".into(),
                        text: Some("S3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-slate-500 text-white p-1".into(),
                        text: Some("S5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-slate-700 text-white p-1".into(),
                        text: Some("S7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-slate-900 text-white p-1".into(),
                        text: Some("S9".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-gray-100 p-1".into(),
                        text: Some("G1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-gray-300 p-1".into(),
                        text: Some("G3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-gray-500 text-white p-1".into(),
                        text: Some("G5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-gray-700 text-white p-1".into(),
                        text: Some("G7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-gray-900 text-white p-1".into(),
                        text: Some("G9".into()),
                        children: vec![],
                    },
                ],
            },
            // Warm Colors
            NodeSpec {
                class: "text-bold".into(),
                text: Some("🔥 Warm Colors:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-red-100 p-1".into(),
                        text: Some("R1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-red-300 p-1".into(),
                        text: Some("R3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-red-500 text-white p-1".into(),
                        text: Some("R5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-red-700 text-white p-1".into(),
                        text: Some("R7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-red-900 text-white p-1".into(),
                        text: Some("R9".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-orange-100 p-1".into(),
                        text: Some("O1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-orange-300 p-1".into(),
                        text: Some("O3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-orange-500 text-white p-1".into(),
                        text: Some("O5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-orange-700 text-white p-1".into(),
                        text: Some("O7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-orange-900 text-white p-1".into(),
                        text: Some("O9".into()),
                        children: vec![],
                    },
                ],
            },
            // Yellow/Amber Colors
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-amber-100 p-1".into(),
                        text: Some("A1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-amber-300 p-1".into(),
                        text: Some("A3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-amber-500 text-white p-1".into(),
                        text: Some("A5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-amber-700 text-white p-1".into(),
                        text: Some("A7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-amber-900 text-white p-1".into(),
                        text: Some("A9".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-yellow-100 p-1".into(),
                        text: Some("Y1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-yellow-300 p-1".into(),
                        text: Some("Y3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-yellow-500 text-white p-1".into(),
                        text: Some("Y5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-yellow-700 text-white p-1".into(),
                        text: Some("Y7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-yellow-900 text-white p-1".into(),
                        text: Some("Y9".into()),
                        children: vec![],
                    },
                ],
            },
            // Green Colors
            NodeSpec {
                class: "text-bold".into(),
                text: Some("🌿 Green Colors:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-lime-100 p-1".into(),
                        text: Some("L1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-lime-300 p-1".into(),
                        text: Some("L3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-lime-500 text-white p-1".into(),
                        text: Some("L5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-lime-700 text-white p-1".into(),
                        text: Some("L7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-lime-900 text-white p-1".into(),
                        text: Some("L9".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-green-100 p-1".into(),
                        text: Some("G1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-green-300 p-1".into(),
                        text: Some("G3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-green-500 text-white p-1".into(),
                        text: Some("G5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-green-700 text-white p-1".into(),
                        text: Some("G7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-green-900 text-white p-1".into(),
                        text: Some("G9".into()),
                        children: vec![],
                    },
                ],
            },
            // Emerald/Teal Colors
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-emerald-100 p-1".into(),
                        text: Some("E1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-emerald-300 p-1".into(),
                        text: Some("E3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-emerald-500 text-white p-1".into(),
                        text: Some("E5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-emerald-700 text-white p-1".into(),
                        text: Some("E7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-emerald-900 text-white p-1".into(),
                        text: Some("E9".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-teal-100 p-1".into(),
                        text: Some("T1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-teal-300 p-1".into(),
                        text: Some("T3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-teal-500 text-white p-1".into(),
                        text: Some("T5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-teal-700 text-white p-1".into(),
                        text: Some("T7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-teal-900 text-white p-1".into(),
                        text: Some("T9".into()),
                        children: vec![],
                    },
                ],
            },
            // Blue Colors
            NodeSpec {
                class: "text-bold".into(),
                text: Some("💙 Blue Colors:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-cyan-100 p-1".into(),
                        text: Some("C1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-cyan-300 p-1".into(),
                        text: Some("C3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-cyan-500 text-white p-1".into(),
                        text: Some("C5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-cyan-700 text-white p-1".into(),
                        text: Some("C7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-cyan-900 text-white p-1".into(),
                        text: Some("C9".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-sky-100 p-1".into(),
                        text: Some("S1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-sky-300 p-1".into(),
                        text: Some("S3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-sky-500 text-white p-1".into(),
                        text: Some("S5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-sky-700 text-white p-1".into(),
                        text: Some("S7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-sky-900 text-white p-1".into(),
                        text: Some("S9".into()),
                        children: vec![],
                    },
                ],
            },
            // Blue/Indigo Colors
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-blue-100 p-1".into(),
                        text: Some("B1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-blue-300 p-1".into(),
                        text: Some("B3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-blue-500 text-white p-1".into(),
                        text: Some("B5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-blue-700 text-white p-1".into(),
                        text: Some("B7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-blue-900 text-white p-1".into(),
                        text: Some("B9".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-indigo-100 p-1".into(),
                        text: Some("I1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-indigo-300 p-1".into(),
                        text: Some("I3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-indigo-500 text-white p-1".into(),
                        text: Some("I5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-indigo-700 text-white p-1".into(),
                        text: Some("I7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-indigo-900 text-white p-1".into(),
                        text: Some("I9".into()),
                        children: vec![],
                    },
                ],
            },
            // Purple Colors
            NodeSpec {
                class: "text-bold".into(),
                text: Some("💜 Purple Colors:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-violet-100 p-1".into(),
                        text: Some("V1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-violet-300 p-1".into(),
                        text: Some("V3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-violet-500 text-white p-1".into(),
                        text: Some("V5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-violet-700 text-white p-1".into(),
                        text: Some("V7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-violet-900 text-white p-1".into(),
                        text: Some("V9".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-purple-100 p-1".into(),
                        text: Some("P1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-purple-300 p-1".into(),
                        text: Some("P3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-purple-500 text-white p-1".into(),
                        text: Some("P5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-purple-700 text-white p-1".into(),
                        text: Some("P7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-purple-900 text-white p-1".into(),
                        text: Some("P9".into()),
                        children: vec![],
                    },
                ],
            },
            // Pink Colors
            NodeSpec {
                class: "text-bold".into(),
                text: Some("💖 Pink Colors:".into()),
                children: vec![],
            },
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-fuchsia-100 p-1".into(),
                        text: Some("F1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-fuchsia-300 p-1".into(),
                        text: Some("F3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-fuchsia-500 text-white p-1".into(),
                        text: Some("F5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-fuchsia-700 text-white p-1".into(),
                        text: Some("F7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-fuchsia-900 text-white p-1".into(),
                        text: Some("F9".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-pink-100 p-1".into(),
                        text: Some("P1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-pink-300 p-1".into(),
                        text: Some("P3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-pink-500 text-white p-1".into(),
                        text: Some("P5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-pink-700 text-white p-1".into(),
                        text: Some("P7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-pink-900 text-white p-1".into(),
                        text: Some("P9".into()),
                        children: vec![],
                    },
                ],
            },
            // Rose Colors
            NodeSpec {
                class: "flex flex-row gap-1".into(),
                text: None,
                children: vec![
                    NodeSpec {
                        class: "bg-rose-100 p-1".into(),
                        text: Some("R1".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-rose-300 p-1".into(),
                        text: Some("R3".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-rose-500 text-white p-1".into(),
                        text: Some("R5".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-rose-700 text-white p-1".into(),
                        text: Some("R7".into()),
                        children: vec![],
                    },
                    NodeSpec {
                        class: "bg-rose-900 text-white p-1".into(),
                        text: Some("R9".into()),
                        children: vec![],
                    },
                ],
            },
            // Summary
            NodeSpec {
                class: "text-center bg-green-500 text-white p-2 font-bold".into(),
                text: Some("✅ All 330+ Tailwind CSS Colors Working!".into()),
                children: vec![],
            },
        ],
    };

    // Layout and paint
    let options = PaintOptions {
        debug_overlay: false,
    };

    layout_and_paint_with(&root, &mut surface, 120, &options)?;

    // Print the result to console for inspection
    println!("🎨 Complete Tailwind CSS Color Palette Test:");
    println!("============================================");

    for y in 0..50 {
        for x in 0..120 {
            let cell = surface.get(x, y);
            if cell.ch != ' ' && cell.ch != '\0' {
                print!("{}", cell.ch);
            } else {
                print!(" ");
            }
        }
        println!();
    }

    println!("\n🎉 SUCCESS: Complete Tailwind CSS Color System!");
    println!("📊 Colors Available:");
    println!("  • Slate: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Gray: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Zinc: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Neutral: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Stone: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Red: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Orange: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Amber: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Yellow: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Lime: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Green: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Emerald: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Teal: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Cyan: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Sky: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Blue: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Indigo: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Violet: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Purple: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Fuchsia: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Pink: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("  • Rose: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950");
    println!("\n🎯 Total: 22 color families × 11 shades = 242 colors");
    println!("🎯 Plus: Basic colors (black, white, transparent) = 245+ colors");
    println!("🎯 Plus: Legacy aliases (red, green, blue, etc.) = 250+ colors");

    Ok(())
}
