//! Test the direct TTY implementation

use reactive_tui::error::Result;
use reactive_tui::platform::DirectTty;
use std::time::Duration;

fn main() -> Result<()> {
    println!("🔧 Testing Direct TTY Implementation");

    // Test 1: Basic TTY initialization
    println!("1. Testing TTY initialization...");
    let mut tty = DirectTty::init()?;
    println!("   ✅ TTY initialized successfully");

    // Test 2: Terminal size
    println!("2. Testing terminal size detection...");
    let (width, height) = tty.size()?;
    println!("   ✅ Terminal size: {}x{}", width, height);

    // Test 3: Capability detection
    println!("3. Testing capability detection...");
    let caps = tty.capabilities().clone();
    println!("   📊 Capabilities detected:");
    println!("      True Color: {}", caps.true_color);
    println!("      Kitty Graphics: {}", caps.kitty_graphics);
    println!("      Sixel Graphics: {}", caps.sixel_graphics);
    println!("      Hyperlinks: {}", caps.hyperlinks);
    println!("      Pixel Mouse: {}", caps.pixel_mouse);
    println!("      Synchronized Output: {}", caps.synchronized_output);
    println!("      Enhanced Keyboard: {}", caps.enhanced_keyboard);

    // Test 4: Basic writing
    println!("4. Testing basic terminal writing...");
    tty.write(b"Hello from Direct TTY!\r\n")?;
    println!("   ✅ Basic writing works");

    // Test 5: Control sequences
    println!("5. Testing control sequences...");
    tty.write(b"\x1b[31mRed text\x1b[0m\r\n")?; // Red text
    tty.write(b"\x1b[1mBold text\x1b[0m\r\n")?; // Bold text
    println!("   ✅ Control sequences work");

    // Test 6: Hyperlinks (if supported)
    if caps.hyperlinks {
        println!("6. Testing hyperlinks...");
        tty.write_hyperlink(
            "https://github.com/entrepeneur4lyf/reactive-tui",
            "Click me!",
        )?;
        tty.write(b"\r\n")?;
        println!("   ✅ Hyperlinks work");
    } else {
        println!("6. Hyperlinks not supported by terminal");
    }

    // Test 7: Event polling
    println!("7. Testing event polling (press any key or wait 2 seconds)...");
    match tty.poll_events(Some(Duration::from_secs(2))) {
        Ok(events) if !events.is_empty() => {
            println!("   ✅ Received {} events: {:?}", events.len(), events);
        }
        Ok(_) => {
            println!("   ⏰ No events received (timeout)");
        }
        Err(e) => {
            println!("   ❌ Error polling events: {}", e);
        }
    }

    // Test 8: Synchronized output (if supported)
    if caps.synchronized_output {
        println!("8. Testing synchronized output...");
        tty.enable_sync_output()?;
        tty.write(b"Synchronized update 1\r\n")?;
        tty.write(b"Synchronized update 2\r\n")?;
        tty.disable_sync_output()?;
        println!("   ✅ Synchronized output works");
    } else {
        println!("8. Synchronized output not supported");
    }

    println!("\n🎉 Direct TTY implementation test completed!");
    println!("✅ All basic functionality is working");

    Ok(())
}
