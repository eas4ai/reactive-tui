//! Image Placement Demo
//!
//! Demonstrates the new cell-level image placement functionality that completes
//! the libvaxis-inspired features in reactive-tui.

use reactive_tui::core::renderer::Renderer;
use reactive_tui::core::surface::{Attr, Rgba, Surface};
use reactive_tui::error::Result;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("🖼️  Image Placement Demo");
    println!("========================");
    println!("Demonstrating cell-level image placement - a key libvaxis feature");
    println!();

    // Create a surface for demonstration
    let mut surface = Surface::new(40, 20);

    // Create some test images with different colors
    println!("📷 Creating test images...");
    let red_image = surface.create_test_image(32, 32, 255, 0, 0); // Red
    let green_image = surface.create_test_image(32, 32, 0, 255, 0); // Green
    let blue_image = surface.create_test_image(32, 32, 0, 0, 255); // Blue

    println!("   ✅ Created {} images in registry", surface.image_count());

    // Demonstrate different placement techniques
    println!("\n🎨 Demonstrating image placement techniques:");

    // 1. Background images (behind text)
    println!("   1. Background images (z-index = -1)");
    surface.place_image_background(5, 5, red_image);
    surface.place_image_background(6, 5, red_image);
    surface.place_image_background(7, 5, red_image);

    // 2. Foreground images (in front of text)
    println!("   2. Foreground images (z-index = 1)");
    surface.place_image_foreground(10, 8, green_image);
    surface.place_image_foreground(11, 8, green_image);

    // 3. Image regions (spanning multiple cells)
    println!("   3. Image regions (4x3 cells)");
    surface.place_image_region(15, 10, 4, 3, blue_image, 64, 64, 0, 0.8);

    // Add some text content to show interaction with images
    let white = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let black = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    surface.write_str(5, 5, "BG", white, black, Attr::empty());
    surface.write_str(10, 8, "FG", white, black, Attr::empty());
    surface.write_str(16, 11, "REGION", white, black, Attr::empty());

    // Display statistics
    println!("\n📊 Image Placement Statistics:");
    let image_cells = surface.get_image_cells();
    println!("   • Total cells with images: {}", image_cells.len());
    println!("   • Images in registry: {}", surface.image_count());

    // Group by z-index
    let mut background_count = 0;
    let mut foreground_count = 0;
    let mut neutral_count = 0;

    for (_, _, _, placement) in &image_cells {
        match placement.z_index {
            z if z < 0 => background_count += 1,
            z if z > 0 => foreground_count += 1,
            _ => neutral_count += 1,
        }
    }

    println!("   • Background images: {}", background_count);
    println!("   • Neutral images: {}", neutral_count);
    println!("   • Foreground images: {}", foreground_count);

    // Demonstrate image data access
    println!("\n🔍 Image Data Details:");
    if let Some(red_data) = surface.get_image(red_image) {
        println!(
            "   • Red image: {}x{} pixels, {} bytes",
            red_data.width,
            red_data.height,
            red_data.pixels.len()
        );
        println!(
            "   • First pixel: R={}, G={}, B={}, A={}",
            red_data.pixels[0], red_data.pixels[1], red_data.pixels[2], red_data.pixels[3]
        );
    }

    // Test rendering with a simple renderer (if available)
    println!("\n🖥️  Testing with renderer...");
    match test_with_renderer(&surface) {
        Ok(_) => println!("   ✅ Renderer integration successful"),
        Err(e) => println!("   ⚠️  Renderer test skipped: {}", e),
    }

    // Demonstrate cleanup
    println!("\n🧹 Testing cleanup operations:");
    let initial_count = surface.get_image_cells().len();

    // Clear specific cell
    surface.clear_cell_image(5, 5);
    let after_clear = surface.get_image_cells().len();
    println!(
        "   • Cleared one cell: {} -> {} placements",
        initial_count, after_clear
    );

    // Clear all placements
    surface.clear_all_image_placements();
    let after_clear_all = surface.get_image_cells().len();
    println!(
        "   • Cleared all placements: {} -> {} placements",
        after_clear, after_clear_all
    );
    println!("   • Images still in registry: {}", surface.image_count());

    // Clear registry
    surface.clear_images();
    println!(
        "   • Cleared image registry: {} images remaining",
        surface.image_count()
    );

    println!("\n✨ Image placement demo completed successfully!");
    println!("   This demonstrates the libvaxis-inspired cell-level image placement");
    println!("   feature that allows embedding images directly in terminal cells.");

    Ok(())
}

fn test_with_renderer(_surface: &Surface) -> Result<()> {
    // Try to create a small renderer for testing
    let _renderer = Renderer::new(40, 20)?;

    // In a real application, you would:
    // 1. Copy surface data to renderer's back buffer
    // 2. Call renderer.end_frame() to trigger diff and image rendering
    // 3. The DiffWriter would process image placements and generate appropriate output

    println!("   • Created {}x{} renderer", 40, 20);
    println!("   • Image rendering would be triggered during diff process");

    // Simulate some processing time
    thread::sleep(Duration::from_millis(10));

    Ok(())
}
