//! Test the direct TTY backend
//!
//! This example demonstrates the direct TTY backend with advanced terminal features

use reactive_tui::backend::direct_tty::DirectTtyBackend;
use reactive_tui::backend::Backend;
use reactive_tui::component::{Element, ElementType};
use reactive_tui::error::Result;
use reactive_tui::render::tree::{element_to_render_node, RenderTree};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<()> {
    println!("Testing Direct TTY Backend...");

    // Try to create the direct TTY backend
    let mut backend = match DirectTtyBackend::new() {
        Ok(backend) => {
            println!("✅ Direct TTY backend initialized successfully!");
            backend
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize direct TTY backend: {}", e);
            eprintln!("This is expected on some systems or when not running in a terminal.");
            return Ok(());
        }
    };

    // Display terminal capabilities
    let caps = backend.capabilities().clone();
    println!("🔍 Terminal Capabilities:");
    println!("  True Color: {}", caps.true_color);
    println!("  Kitty Graphics: {}", caps.kitty_graphics);
    println!("  Sixel Graphics: {}", caps.sixel_graphics);
    println!("  iTerm2 Images: {}", caps.iterm2_images);
    println!("  Hyperlinks: {}", caps.hyperlinks);
    println!("  Pixel Mouse: {}", caps.pixel_mouse);
    println!("  Synchronized Output: {}", caps.synchronized_output);
    println!("  Enhanced Keyboard: {}", caps.enhanced_keyboard);
    println!("  Bracketed Paste: {}", caps.bracketed_paste);
    println!("  Focus Events: {}", caps.focus_events);

    // Get terminal size
    let (width, height) = backend.size();
    println!("📐 Terminal Size: {}x{}", width, height);

    // Test hyperlink support
    if caps.hyperlinks {
        println!("🔗 Testing hyperlink support...");
        backend.write_hyperlink(
            "https://github.com/entrepeneur4lyf/reactive-tui",
            "Reactive-TUI on GitHub",
        )?;
        println!(" <- This should be a clickable link if your terminal supports it!");
    }

    // Create a simple test element
    let test_element = Element {
        element_type: ElementType::Text(
            "Hello from Direct TTY Backend!\nThis is a test of the direct terminal interface."
                .to_string(),
        ),
        props: Arc::new(()),
        children: vec![],
        key: None,
        class: None,
    };

    // Create render tree
    let mut tree = RenderTree::new();
    let root = element_to_render_node(test_element);
    tree.set_root(root);

    // Clear and render
    backend.clear()?;
    backend.apply_patches(&[], &tree)?;
    backend.present()?;

    println!("\n⏱️  Waiting 3 seconds to show the rendered content...");
    std::thread::sleep(Duration::from_secs(3));

    // Test event polling
    println!("🎮 Testing event polling (press any key or wait 2 seconds)...");
    match backend.poll_event(Some(2000)) {
        Ok(Some(event)) => {
            println!("📥 Received event: {:?}", event);
        }
        Ok(None) => {
            println!("⏰ No events received (timeout)");
        }
        Err(e) => {
            println!("❌ Error polling events: {}", e);
        }
    }

    // Test a few more frames with different content
    for i in 1..=3 {
        let frame_element = Element {
            element_type: ElementType::Text(format!(
                "Frame {} - Direct TTY is working!\nAdvanced features: {}",
                i,
                if caps.synchronized_output {
                    "Synchronized Output ✅"
                } else {
                    "Basic Output"
                }
            )),
            props: Arc::new(()),
            children: vec![],
            key: None,
            class: None,
        };

        let mut new_tree = RenderTree::new();
        let new_root = element_to_render_node(frame_element);
        new_tree.set_root(new_root);

        backend.clear()?;
        backend.apply_patches(&[], &new_tree)?;
        backend.present()?;

        std::thread::sleep(Duration::from_millis(500));
    }

    println!("\n✅ Direct TTY backend test completed successfully!");
    println!(
        "🎉 The direct TTY backend is working and provides access to advanced terminal features!"
    );

    Ok(())
}
