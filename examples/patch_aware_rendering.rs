use reactive_tui::backend::{Backend, DebugBackend};
use reactive_tui::component::Element;
use reactive_tui::render::reconcile::Reconciler;
use reactive_tui::render::tree::{RenderTree, element_to_render_node};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Patch-Aware Rendering Demo ===");

    // Create a debug backend for deterministic testing
    let mut backend = DebugBackend::new(40, 10);
    let mut reconciler = Reconciler::new();

    // Initial render: Simple text
    println!("\n1. Initial render:");
    let element1 = Element::text("Hello, World!");
    let root1 = element_to_render_node(element1);
    let mut tree1 = RenderTree::new();
    tree1.set_root(root1);

    // Apply initial render (no patches, just full render)
    backend.apply_patches(&[], &tree1)?;
    backend.present()?;

    println!("Screen content:");
    print_screen_content(&backend);
    println!("Frame count: {}", backend.frame_count());

    // Second render: Update text
    println!("\n2. Update text:");
    let element2 = Element::text("Hello, Reactive TUI!");
    let root2 = element_to_render_node(element2);
    let mut tree2 = RenderTree::new();
    tree2.set_root(root2);

    // Generate and apply patches
    let diff_result = reconciler.diff(&tree1, &tree2);
    println!("Generated {} patches:", diff_result.patches.len());
    for (i, patch) in diff_result.patches.iter().enumerate() {
        println!("  Patch {}: {:?}", i, patch);
    }

    backend.apply_patches(&diff_result.patches, &tree2)?;
    backend.present()?;

    println!("Screen content:");
    print_screen_content(&backend);
    println!("Frame count: {}", backend.frame_count());

    // Third render: Multiline text
    println!("\n3. Multiline text:");
    let element3 = Element::text("Line 1\nLine 2\nLine 3");
    let root3 = element_to_render_node(element3);
    let mut tree3 = RenderTree::new();
    tree3.set_root(root3);

    let diff_result2 = reconciler.diff(&tree2, &tree3);
    println!("Generated {} patches:", diff_result2.patches.len());
    for (i, patch) in diff_result2.patches.iter().enumerate() {
        println!("  Patch {}: {:?}", i, patch);
    }

    backend.apply_patches(&diff_result2.patches, &tree3)?;
    backend.present()?;

    println!("Screen content:");
    print_screen_content(&backend);
    println!("Frame count: {}", backend.frame_count());

    // Test event handling
    println!("\n4. Event handling test:");
    use reactive_tui::event::types as rt_event;

    // Add some test events
    backend.push_event(rt_event::Event::Key(rt_event::KeyEvent::new(
        rt_event::KeyCode::Char('a'),
    )));
    backend.push_event(rt_event::Event::Key(rt_event::KeyEvent::new(
        rt_event::KeyCode::Enter,
    )));

    // Poll events
    while let Some(event) = backend.poll_event(None)? {
        match event {
            rt_event::Event::Key(key_event) => {
                println!("Received key event: {:?}", key_event.code);
            }
            rt_event::Event::Mouse(mouse_event) => {
                println!("Received mouse event: {:?}", mouse_event.kind);
            }
            _ => {
                println!("Received other event: {:?}", event);
            }
        }
    }

    // Show patch history
    println!("\n5. Patch history:");
    for (frame, patches) in backend.patch_history().iter().enumerate() {
        println!("Frame {}: {} patches", frame, patches.len());
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn print_screen_content(backend: &DebugBackend) {
    let content = backend.screen_content();
    let lines: Vec<&str> = content.split('\n').collect();

    println!("┌{}┐", "─".repeat(40));
    for (i, line) in lines.iter().enumerate() {
        if i < 10 {
            // Only show first 10 lines
            let display_line = if line.chars().all(|c| c == '\0') {
                " ".repeat(40) // Convert null chars to spaces for display
            } else {
                format!("{:40}", line.replace('\0', " "))
            };
            println!("│{}│", display_line);
        }
    }
    println!("└{}┘", "─".repeat(40));
}
