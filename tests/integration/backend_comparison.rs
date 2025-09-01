//! Compare crossterm backend vs direct TTY backend
//!
//! This example shows the differences in capabilities between the two backends

use reactive_tui::backend::{direct_tty::DirectTtyBackend, Backend, CrosstermBackend};
use reactive_tui::component::{Element, ElementType};
use reactive_tui::error::Result;
use reactive_tui::render::tree::{element_to_render_node, RenderTree};
use std::sync::Arc;

fn main() -> Result<()> {
    println!("🔄 Backend Comparison: Crossterm vs Direct TTY\n");

    // Test Crossterm Backend
    println!("📦 Testing Crossterm Backend:");
    let mut crossterm_backend = CrosstermBackend::new()?;
    let (ct_width, ct_height) = crossterm_backend.size();
    println!("  ✅ Initialized successfully");
    println!("  📐 Size: {}x{}", ct_width, ct_height);
    println!("  🎯 Features: Basic terminal support via crossterm");
    println!("  ⚡ Performance: Good (abstraction layer)");
    println!("  🌐 Compatibility: Excellent (cross-platform)");

    // Test Direct TTY Backend
    println!("\n🔧 Testing Direct TTY Backend:");
    match DirectTtyBackend::new() {
        Ok(mut direct_backend) => {
            let (dt_width, dt_height) = direct_backend.size();
            let caps = direct_backend.capabilities().clone();

            println!("  ✅ Initialized successfully");
            println!("  📐 Size: {}x{}", dt_width, dt_height);
            println!("  🎯 Advanced Features Available:");
            println!(
                "    🎨 True Color: {}",
                if caps.true_color { "✅" } else { "❌" }
            );
            println!(
                "    🖼️  Kitty Graphics: {}",
                if caps.kitty_graphics { "✅" } else { "❌" }
            );
            println!(
                "    🎭 Sixel Graphics: {}",
                if caps.sixel_graphics { "✅" } else { "❌" }
            );
            println!(
                "    🔗 Hyperlinks: {}",
                if caps.hyperlinks { "✅" } else { "❌" }
            );
            println!(
                "    🖱️  Pixel Mouse: {}",
                if caps.pixel_mouse { "✅" } else { "❌" }
            );
            println!(
                "    ⚡ Sync Output: {}",
                if caps.synchronized_output {
                    "✅"
                } else {
                    "❌"
                }
            );
            println!(
                "    ⌨️  Enhanced Keys: {}",
                if caps.enhanced_keyboard { "✅" } else { "❌" }
            );
            println!(
                "    📋 Bracketed Paste: {}",
                if caps.bracketed_paste { "✅" } else { "❌" }
            );
            println!(
                "    👁️  Focus Events: {}",
                if caps.focus_events { "✅" } else { "❌" }
            );
            println!("  ⚡ Performance: Excellent (direct system calls)");
            println!("  🌐 Compatibility: Unix/Linux (platform-specific)");

            // Demonstrate hyperlink capability
            if caps.hyperlinks {
                println!("\n🔗 Hyperlink Demo (Direct TTY only):");
                direct_backend.write_hyperlink(
                    "https://github.com/entrepeneur4lyf/reactive-tui",
                    "Click me! (Reactive-TUI Repository)",
                )?;
                println!(" <- This link only works with Direct TTY backend");
            }

            // Performance comparison
            println!("\n⚡ Performance Comparison:");

            // Create test content
            let test_element = Element {
                element_type: ElementType::Text(
                    "Performance Test Content\nRendering with different backends...".to_string(),
                ),
                props: Arc::new(()),
                children: vec![],
                key: None,
                class: None,
            };

            let mut tree = RenderTree::new();
            let root = element_to_render_node(test_element.clone());
            tree.set_root(root);

            // Time crossterm backend
            let start = std::time::Instant::now();
            for _ in 0..10 {
                crossterm_backend.clear()?;
                crossterm_backend.apply_patches(&[], &tree)?;
                crossterm_backend.present()?;
            }
            let crossterm_time = start.elapsed();

            // Time direct TTY backend
            let start = std::time::Instant::now();
            for _ in 0..10 {
                direct_backend.clear()?;
                direct_backend.apply_patches(&[], &tree)?;
                direct_backend.present()?;
            }
            let direct_time = start.elapsed();

            println!("  📦 Crossterm: {:?} (10 frames)", crossterm_time);
            println!("  🔧 Direct TTY: {:?} (10 frames)", direct_time);

            if direct_time < crossterm_time {
                let speedup = crossterm_time.as_nanos() as f64 / direct_time.as_nanos() as f64;
                println!("  🚀 Direct TTY is {:.2}x faster!", speedup);
            } else {
                println!("  📊 Performance is similar");
            }
        }
        Err(e) => {
            println!("  ❌ Failed to initialize: {}", e);
            println!("  ℹ️  This is expected on Windows or non-terminal environments");
            println!("  💡 Direct TTY backend requires Unix/Linux with terminal access");
        }
    }

    println!("\n📊 Summary:");
    println!("┌─────────────────┬─────────────────┬─────────────────┐");
    println!("│ Feature         │ Crossterm       │ Direct TTY      │");
    println!("├─────────────────┼─────────────────┼─────────────────┤");
    println!("│ Cross-platform  │ ✅ Excellent    │ ❌ Unix only    │");
    println!("│ Setup ease      │ ✅ Simple       │ ⚠️  Advanced    │");
    println!("│ Performance     │ ⚡ Good         │ 🚀 Excellent   │");
    println!("│ Image support   │ ❌ None         │ ✅ Multiple     │");
    println!("│ Hyperlinks      │ ❌ None         │ ✅ OSC 8        │");
    println!("│ Pixel mouse     │ ❌ None         │ ✅ Available    │");
    println!("│ Sync output     │ ❌ None         │ ✅ Flicker-free │");
    println!("│ Enhanced keys   │ ❌ Basic        │ ✅ Full support │");
    println!("└─────────────────┴─────────────────┴─────────────────┘");

    println!("\n💡 Recommendations:");
    println!("  📦 Use Crossterm for: Cross-platform apps, simple TUIs, prototyping");
    println!("  🔧 Use Direct TTY for: Advanced features, performance, Unix-specific apps");
    println!("  🎯 Hybrid approach: Feature detection + graceful fallback");

    Ok(())
}
