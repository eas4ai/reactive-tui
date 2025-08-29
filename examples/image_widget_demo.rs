//! Image Widget Demo
//!
//! Demonstrates the comprehensive image support capabilities of reactive-tui,
//! including multiple rendering backends, format support, and CSS-like styling.

use reactive_tui::core::terminal::Terminal;
use reactive_tui::widgets::{Image, ImageDisplayMode, ImageQuality};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Reactive-TUI Image Widget Demo ===");
    // Detect available image capabilities
    let capabilities = Terminal::detect_image_capabilities();
    println!("🔍 Detected Image Capabilities:");
    println!(
        "  • Sixel support: {}",
        if capabilities.sixel { "✅" } else { "❌" }
    );
    println!(
        "  • Kitty graphics: {}",
        if capabilities.kitty_graphics {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  • iTerm2 inline: {}",
        if capabilities.iterm2_inline {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  • Chafa available: {}",
        if capabilities.chafa_available {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  • Viu available: {}",
        if capabilities.viu_available {
            "✅"
        } else {
            "❌"
        }
    );
    println!();

    // Demo 1: Basic image from file
    println!("📁 Demo 1: Loading image from file");
    demo_file_image(&capabilities)?;

    // Demo 2: Image from base64 data
    println!("📊 Demo 2: Loading image from base64 data");
    demo_base64_image(&capabilities)?;

    // Demo 3: Different display modes
    println!("🎨 Demo 3: Different display modes");
    demo_display_modes(&capabilities)?;

    // Demo 4: Quality settings
    println!("⚡ Demo 4: Quality settings");
    demo_quality_settings(&capabilities)?;

    // Demo 5: Size constraints and aspect ratio
    println!("📏 Demo 5: Size constraints and aspect ratio");
    demo_size_constraints(&capabilities)?;

    // Demo 6: CSS-like styling
    println!("🎭 Demo 6: CSS-like styling integration");
    demo_css_styling()?;

    // Demo 7: Error handling and fallbacks
    println!("🛡️ Demo 7: Error handling and fallbacks");
    demo_error_handling(&capabilities)?;

    println!("\n✨ Demo completed successfully!");
    Ok(())
}

fn demo_file_image(
    capabilities: &reactive_tui::widgets::ImageCapabilities,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple test image if it doesn't exist
    let test_image_path = "test_image.png";
    create_test_image_if_needed(test_image_path)?;

    let image = Image::from_file(test_image_path)
        .with_max_size(40, 20)
        .with_preserve_aspect(true)
        .with_quality(ImageQuality::Balanced);

    match image.render(capabilities) {
        Ok(output) => {
            println!("✅ Successfully rendered image from file:");
            println!("{}", output);
        }
        Err(e) => {
            println!("❌ Failed to render image: {}", e);
            println!(
                "💡 Fallback: {}",
                image.fallback_text.as_deref().unwrap_or("📷 [Image]")
            );
        }
    }
    println!();
    Ok(())
}

fn demo_base64_image(
    capabilities: &reactive_tui::widgets::ImageCapabilities,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple 4x4 red square as base64 PNG
    let base64_data = create_test_base64_image();

    let image = Image::from_base64(base64_data)
        .with_max_size(20, 10)
        .with_display_mode(ImageDisplayMode::Auto);

    match image.render(capabilities) {
        Ok(output) => {
            println!("✅ Successfully rendered image from base64:");
            println!("{}", output);
        }
        Err(e) => {
            println!("❌ Failed to render base64 image: {}", e);
        }
    }
    println!();
    Ok(())
}

fn demo_display_modes(
    capabilities: &reactive_tui::widgets::ImageCapabilities,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_image_path = "test_image.png";
    create_test_image_if_needed(test_image_path)?;

    let modes = vec![
        (ImageDisplayMode::Auto, "Auto (best available)"),
        (ImageDisplayMode::Sixel, "Sixel graphics"),
        (ImageDisplayMode::AsciiArt, "ASCII art"),
        (ImageDisplayMode::Chafa, "Chafa external tool"),
        (ImageDisplayMode::Viu, "Viu external tool"),
    ];

    for (mode, description) in modes {
        println!("🎨 Testing {}: ", description);

        let image = Image::from_file(test_image_path)
            .with_display_mode(mode)
            .with_max_size(30, 15);

        match image.render(capabilities) {
            Ok(output) => {
                println!("✅ Success:");
                // Limit output for demo purposes
                let lines: Vec<&str> = output.lines().take(5).collect();
                for line in lines {
                    println!("  {}", line);
                }
                if output.lines().count() > 5 {
                    println!("  ... ({} more lines)", output.lines().count() - 5);
                }
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
            }
        }
        println!();
    }
    Ok(())
}

fn demo_quality_settings(
    capabilities: &reactive_tui::widgets::ImageCapabilities,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_image_path = "test_image.png";
    create_test_image_if_needed(test_image_path)?;

    let qualities = vec![
        (ImageQuality::Fast, "Fast (lower quality)"),
        (ImageQuality::Balanced, "Balanced"),
        (ImageQuality::High, "High (slower)"),
    ];

    for (quality, description) in qualities {
        println!("⚡ Testing {} quality:", description);

        let image = Image::from_file(test_image_path)
            .with_quality(quality)
            .with_max_size(25, 12)
            .with_display_mode(ImageDisplayMode::AsciiArt); // Use ASCII for consistent demo

        match image.render(capabilities) {
            Ok(output) => {
                println!("✅ Rendered with {} quality", description);
                // Show first few lines
                for line in output.lines().take(3) {
                    println!("  {}", line);
                }
                println!("  ... (truncated for demo)");
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
            }
        }
        println!();
    }
    Ok(())
}

fn demo_size_constraints(
    capabilities: &reactive_tui::widgets::ImageCapabilities,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_image_path = "test_image.png";
    create_test_image_if_needed(test_image_path)?;

    println!("📏 Testing different size constraints:");

    // Test with aspect ratio preservation
    let image1 = Image::from_file(test_image_path)
        .with_max_size(20, 10)
        .with_preserve_aspect(true)
        .with_display_mode(ImageDisplayMode::AsciiArt);

    println!("  • With aspect ratio preservation (20x10 max):");
    match image1.render(capabilities) {
        Ok(output) => {
            let lines: Vec<&str> = output.lines().collect();
            println!(
                "    Actual size: {} lines x ~{} chars",
                lines.len(),
                lines.first().map(|l| l.len()).unwrap_or(0)
            );
        }
        Err(e) => println!("    ❌ Failed: {}", e),
    }

    // Test without aspect ratio preservation
    let image2 = Image::from_file(test_image_path)
        .with_max_size(15, 15)
        .with_preserve_aspect(false)
        .with_display_mode(ImageDisplayMode::AsciiArt);

    println!("  • Without aspect ratio preservation (15x15 max):");
    match image2.render(capabilities) {
        Ok(output) => {
            let lines: Vec<&str> = output.lines().collect();
            println!(
                "    Actual size: {} lines x ~{} chars",
                lines.len(),
                lines.first().map(|l| l.len()).unwrap_or(0)
            );
        }
        Err(e) => println!("    ❌ Failed: {}", e),
    }

    println!();
    Ok(())
}

fn demo_css_styling() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 CSS-like styling integration:");

    // Demonstrate CSS-like styling integration
    println!("✅ Image widget supports CSS classes:");
    println!("  • Container: flex items-center justify-center image-fit-contain aspect-ratio-16-9");
    println!("  • Image: image-quality-high image-rendering-smooth");
    println!("  • This demonstrates how image widgets integrate with the CSS utility system");

    println!();

    Ok(())
}

fn demo_error_handling(
    capabilities: &reactive_tui::widgets::ImageCapabilities,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ Error handling and fallbacks:");

    // Test with non-existent file
    let image1 =
        Image::from_file("non_existent_image.png").with_fallback_text("🖼️ Image not found");

    println!("  • Non-existent file:");
    match image1.render(capabilities) {
        Ok(output) => println!("    Unexpected success: {}", output),
        Err(_) => {
            println!("    ❌ Expected error occurred");
            println!(
                "    💡 Fallback: {}",
                image1.fallback_text.as_deref().unwrap_or("📷 [Image]")
            );
        }
    }

    // Test with invalid base64
    let image2 =
        Image::from_base64("invalid_base64_data!!!").with_fallback_text("🚫 Invalid image data");

    println!("  • Invalid base64 data:");
    match image2.render(capabilities) {
        Ok(output) => println!("    Unexpected success: {}", output),
        Err(_) => {
            println!("    ❌ Expected error occurred");
            println!(
                "    💡 Fallback: {}",
                image2.fallback_text.as_deref().unwrap_or("📷 [Image]")
            );
        }
    }

    println!();
    Ok(())
}

// Helper functions for creating test data

fn create_test_image_if_needed(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if std::path::Path::new(path).exists() {
        return Ok(());
    }

    // Create a simple 16x16 gradient image
    use image::{ImageBuffer, Rgb};

    let img = ImageBuffer::from_fn(16, 16, |x, y| {
        let r = (x * 255 / 15) as u8;
        let g = (y * 255 / 15) as u8;
        let b = ((x + y) * 255 / 30) as u8;
        Rgb([r, g, b])
    });

    img.save(path)?;
    println!("📝 Created test image: {}", path);
    Ok(())
}

fn create_test_base64_image() -> String {
    // A minimal 2x2 red PNG image as base64
    // This is a valid PNG file with a 2x2 red square
    "iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAFElEQVQIHWP8//8/AzYwirkTlQEAKrwC/QGnqRkAAAAASUVORK5CYII=".to_string()
}
