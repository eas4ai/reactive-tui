//! Comprehensive test of all direct TTY features

use reactive_tui::error::Result;
use reactive_tui::platform::{DirectTty, EnhancedKeyboardFeatures, KittyImageFormat};
use std::time::Duration;

fn main() -> Result<()> {
    println!("🚀 COMPREHENSIVE Direct TTY Feature Test");
    println!("=========================================");

    // Initialize TTY
    let mut tty = DirectTty::init()?;
    let caps = tty.capabilities().clone();

    // Display detailed capability information
    println!("\n📊 DETECTED CAPABILITIES:");
    println!("  Terminal Name: {:?}", caps.terminal_name);
    println!("  True Color (24-bit): {}", caps.true_color);
    println!("  Kitty Graphics: {}", caps.kitty_graphics);
    println!("  Sixel Graphics: {}", caps.sixel_graphics);
    println!("  iTerm2 Images: {}", caps.iterm2_images);
    println!("  Hyperlinks (OSC 8): {}", caps.hyperlinks);
    println!("  Pixel Mouse: {}", caps.pixel_mouse);
    println!("  Synchronized Output: {}", caps.synchronized_output);
    println!("  Enhanced Keyboard: {}", caps.enhanced_keyboard);
    println!("  Bracketed Paste: {}", caps.bracketed_paste);
    println!("  Focus Events: {}", caps.focus_events);

    // Test 1: Advanced Color Support
    println!("\n🎨 TESTING ADVANCED COLOR SUPPORT:");
    if caps.true_color {
        // Test true color gradients
        tty.write(b"  True Color Gradient: ")?;
        for i in 0..20 {
            let r = (i * 255 / 20) as u8;
            let g = ((20 - i) * 255 / 20) as u8;
            let b = 128u8;
            tty.write_str(&format!("\x1b[38;2;{};{};{}m█", r, g, b))?;
        }
        tty.write(b"\x1b[0m\r\n")?;
    }

    // Test 2: Enhanced Keyboard Protocol
    println!("\n⌨️  TESTING ENHANCED KEYBOARD:");
    if caps.enhanced_keyboard {
        println!("  Enabling enhanced keyboard protocol...");
        let features = EnhancedKeyboardFeatures {
            disambiguate_escape_codes: true,
            report_all_keys: true,
            report_alternate_keys: true,
            report_all_events: true,
            report_text_as_codepoints: false,
        };
        tty.enable_enhanced_keyboard_features(features)?;
        println!("  ✅ Enhanced keyboard enabled");

        println!("  Press any key to test enhanced events (5 second timeout):");
        match tty.poll_events(Some(Duration::from_secs(5))) {
            Ok(events) if !events.is_empty() => {
                println!("  ✅ Enhanced events received: {:?}", events);
            }
            Ok(_) => {
                println!("  ⏰ No enhanced events (timeout)");
            }
            Err(e) => {
                println!("  ❌ Error: {}", e);
            }
        }

        tty.disable_enhanced_keyboard()?;
    } else {
        println!("  ❌ Enhanced keyboard not supported");
    }

    // Test 3: Mouse Events
    println!("\n🖱️  TESTING MOUSE EVENTS:");
    if caps.pixel_mouse {
        tty.enable_pixel_mouse()?;
        println!("  Pixel mouse enabled - move mouse or click (3 seconds):");
    } else {
        tty.enable_basic_mouse()?;
        println!("  Basic mouse enabled - move mouse or click (3 seconds):");
    }

    match tty.poll_events(Some(Duration::from_secs(3))) {
        Ok(events) if !events.is_empty() => {
            println!("  ✅ Mouse events received: {:?}", events);
        }
        Ok(_) => {
            println!("  ⏰ No mouse events (timeout)");
        }
        Err(e) => {
            println!("  ❌ Error: {}", e);
        }
    }
    tty.disable_mouse()?;

    // Test 4: Synchronized Output
    println!("\n🔄 TESTING SYNCHRONIZED OUTPUT:");

    // Test if sync output actually works
    println!("  Testing sync output support...");
    let sync_works = tty.test_sync_output_support()?;
    println!("  Sync output query result: {}", sync_works);

    if caps.synchronized_output || sync_works {
        println!("  Testing flicker-free updates...");

        // Test synchronized update with callback
        tty.synchronized_update(|tty| {
            for i in 0..5 {
                tty.write_str(&format!("  Synchronized update {}/5\r\n", i + 1))?;
                std::thread::sleep(Duration::from_millis(50));
            }
            Ok(())
        })?;

        // Test batch operations
        tty.batch_operations(|tty| {
            tty.write(b"  Batch operation 1\r\n")?;
            tty.write(b"  Batch operation 2\r\n")?;
            tty.write(b"  Batch operation 3\r\n")?;
            Ok(())
        })?;

        println!("  ✅ Synchronized output test completed");
    } else {
        println!("  ⚠️  Synchronized output not supported, testing fallback...");

        // Test fallback synchronization
        tty.enable_sync_output()?;
        tty.write(b"  Fallback sync test\r\n")?;
        tty.disable_sync_output()?;

        println!("  ✅ Fallback synchronization works");
    }

    // Test 5: Hyperlinks
    println!("\n🔗 TESTING HYPERLINKS:");
    if caps.hyperlinks {
        println!("  Creating clickable links:");
        tty.write(b"  ")?;
        tty.write_hyperlink(
            "https://github.com/entrepeneur4lyf/reactive-tui",
            "GitHub Repo",
        )?;
        tty.write(b" | ")?;
        tty.write_hyperlink("https://docs.rs/reactive-tui", "Documentation")?;
        tty.write(b"\r\n")?;
        println!("  ✅ Hyperlinks created (click them if your terminal supports it!)");
    } else {
        println!("  ❌ Hyperlinks not supported");
    }

    // Test 6: Image Support
    println!("\n🖼️  TESTING IMAGE SUPPORT:");

    // Test auto image detection and transmission
    println!("  Testing auto image format detection...");

    // Create test PNG data
    let test_png = create_test_png();
    match tty.transmit_image_auto(&test_png, Some(100), Some(50)) {
        Ok(_) => {
            println!("  ✅ Auto image transmission successful");
            std::thread::sleep(Duration::from_millis(500));
        }
        Err(e) => {
            println!("  ⚠️  Auto image transmission failed: {}", e);
        }
    }

    if caps.kitty_graphics {
        println!("  Testing Kitty graphics protocol...");
        // Create a simple test image (red square)
        let test_image = create_test_image();
        match tty.transmit_kitty_image(1, 8, 8, &test_image, KittyImageFormat::Rgb) {
            Ok(_) => {
                tty.place_kitty_image(1, 1, 1, 4, 2)?;
                println!("  ✅ Kitty image transmitted and placed");
                std::thread::sleep(Duration::from_millis(500));
                tty.delete_kitty_image(1)?;
            }
            Err(e) => {
                println!("  ❌ Kitty graphics error: {}", e);
            }
        }

        // Test file transmission
        println!("  Testing Kitty file transmission...");
        match tty.transmit_kitty_image_file(2, "/tmp/test.png") {
            Ok(_) => {
                println!("  ✅ Kitty file transmission initiated");
            }
            Err(e) => {
                println!("  ⚠️  Kitty file transmission failed: {}", e);
            }
        }
    } else if caps.sixel_graphics {
        println!("  Testing Sixel graphics...");
        let sixel_data = create_test_sixel();
        match tty.transmit_sixel(&sixel_data) {
            Ok(_) => {
                println!("  ✅ Sixel image transmitted");
            }
            Err(e) => {
                println!("  ❌ Sixel error: {}", e);
            }
        }
    } else if caps.iterm2_images {
        println!("  Testing iTerm2 images...");
        let test_png = create_test_png();
        match tty.transmit_iterm2_image(&test_png, Some(100), Some(50)) {
            Ok(_) => {
                println!("  ✅ iTerm2 image transmitted");
            }
            Err(e) => {
                println!("  ❌ iTerm2 error: {}", e);
            }
        }
    } else {
        println!("  ❌ No image protocols supported");
    }

    // Test 7: Focus Events
    println!("\n👁️  TESTING FOCUS EVENTS:");
    if caps.focus_events {
        tty.enable_focus_events()?;
        println!("  Focus events enabled - switch to another window and back (3 seconds):");

        match tty.poll_events(Some(Duration::from_secs(3))) {
            Ok(events) if !events.is_empty() => {
                println!("  ✅ Focus events received: {:?}", events);
            }
            Ok(_) => {
                println!("  ⏰ No focus events (timeout)");
            }
            Err(e) => {
                println!("  ❌ Error: {}", e);
            }
        }

        tty.disable_focus_events()?;
    } else {
        println!("  ❌ Focus events not supported");
    }

    // Test 8: Bracketed Paste
    println!("\n📋 TESTING BRACKETED PASTE:");
    if caps.bracketed_paste {
        tty.enable_bracketed_paste()?;
        println!("  Bracketed paste enabled - try pasting text (3 seconds):");

        match tty.poll_events(Some(Duration::from_secs(3))) {
            Ok(events) if !events.is_empty() => {
                println!("  ✅ Paste events received: {:?}", events);
            }
            Ok(_) => {
                println!("  ⏰ No paste events (timeout)");
            }
            Err(e) => {
                println!("  ❌ Error: {}", e);
            }
        }

        tty.disable_bracketed_paste()?;
    } else {
        println!("  ❌ Bracketed paste not supported");
    }

    println!("\n🎉 COMPREHENSIVE TEST COMPLETED!");
    println!("✅ Direct TTY implementation provides advanced terminal features");
    println!("🚀 This goes far beyond what crossterm can do!");

    Ok(())
}

/// Create a simple test image (8x8 red square)
fn create_test_image() -> Vec<u8> {
    let mut image = Vec::new();
    for _y in 0..8 {
        for _x in 0..8 {
            image.push(255); // R
            image.push(0); // G
            image.push(0); // B
        }
    }
    image
}

/// Create a simple test Sixel image
fn create_test_sixel() -> Vec<u8> {
    // Simple Sixel: red square
    b"\"1;1;8;8#0;2;0;0;0#1;2;100;0;0$#1!8~!8~!8~!8~!8~!8~".to_vec()
}

/// Create a minimal test PNG
fn create_test_png() -> Vec<u8> {
    // Minimal 1x1 PNG (transparent pixel)
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // Width: 1
        0x00, 0x00, 0x00, 0x01, // Height: 1
        0x08, 0x06, 0x00, 0x00,
        0x00, // Bit depth: 8, Color type: 6 (RGBA), Compression: 0, Filter: 0, Interlace: 0
        0x1F, 0x15, 0xC4, 0x89, // CRC
        0x00, 0x00, 0x00, 0x0A, // IDAT chunk length
        0x49, 0x44, 0x41, 0x54, // IDAT
        0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, // Compressed data
        0xE2, 0x21, 0xBC, 0x33, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ]
}
