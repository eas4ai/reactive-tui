use reactive_tui::core::terminal::Terminal;
use std::time::Instant;

fn main() {
    println!("Reactive-TUI Performance Demo");
    println!("Demonstrating large output buffer improvements");
    println!();

    // Test buffer statistics
    test_buffer_statistics();

    // Test write performance
    test_write_performance();
}

fn test_buffer_statistics() {
    println!("=== Buffer Statistics Test ===");

    let mut terminal = Terminal::new().expect("Failed to create terminal");

    // Show initial stats
    let stats = terminal.write_stats();
    println!(
        "Initial buffer size: {} bytes ({:.1} MB)",
        stats.buffer_size,
        stats.buffer_size as f64 / (1024.0 * 1024.0)
    );

    // Enable buffered mode
    if let Ok(_) = terminal.enable_buffered_mode() {
        println!("✓ Buffered mode enabled");

        // Simulate some writes
        let test_data = "█".repeat(1000).into_bytes();
        for _ in 0..100 {
            let _ = terminal.write_all_buffered(&test_data);
        }

        let stats = terminal.write_stats();
        println!("After writes:");
        println!("  Bytes written: {}", stats.bytes_written);
        println!(
            "  Buffer utilization: {:.1}%",
            stats.buffer_utilization * 100.0
        );
        println!("  Flush count: {}", stats.flush_count);
        println!(
            "  Write time: {:.2}ms",
            stats.write_time.as_secs_f64() * 1000.0
        );

        let _ = terminal.disable_buffered_mode();
        println!("✓ Buffered mode disabled");
    } else {
        println!("⚠ Buffered mode not available in test environment");
    }

    println!();
}

fn test_write_performance() {
    println!("=== Write Performance Test ===");

    // Test direct writes vs buffered writes
    let test_data = generate_test_data();

    // Direct write test
    let direct_time = {
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = Terminal::write_all(&test_data);
        }
        start.elapsed()
    };

    // Buffered write test
    let buffered_time = {
        let mut terminal = Terminal::new().expect("Failed to create terminal");
        let _ = terminal.enable_buffered_mode();

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = terminal.write_all_buffered(&test_data);
        }
        let _ = terminal.flush_buffered();
        let elapsed = start.elapsed();

        let stats = terminal.write_stats();
        println!("Buffered write stats:");
        println!("  Total bytes: {}", stats.bytes_written);
        println!("  Flush count: {}", stats.flush_count);
        println!(
            "  Buffer utilization: {:.1}%",
            stats.buffer_utilization * 100.0
        );

        let _ = terminal.disable_buffered_mode();
        elapsed
    };

    println!();
    println!("Performance comparison:");
    println!(
        "  Direct writes:  {:.2}ms",
        direct_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Buffered writes: {:.2}ms",
        buffered_time.as_secs_f64() * 1000.0
    );

    if buffered_time < direct_time {
        let improvement = ((direct_time.as_secs_f64() - buffered_time.as_secs_f64())
            / direct_time.as_secs_f64())
            * 100.0;
        println!("  Improvement: {:.1}% faster with buffering", improvement);
    } else {
        println!("  Note: Buffering overhead visible in test environment");
    }
}

fn generate_test_data() -> Vec<u8> {
    // Generate typical terminal output data
    let mut data = Vec::new();

    // ANSI escape sequences for colors and positioning
    data.extend_from_slice(b"\x1b[2J\x1b[H"); // Clear screen, home cursor

    for y in 0..24 {
        for x in 0..80 {
            // Move cursor
            data.extend_from_slice(format!("\x1b[{};{}H", y + 1, x + 1).as_bytes());

            // Set colors
            let r = ((x + y) * 3) % 256;
            let g = ((x + y) * 5) % 256;
            let b = ((x + y) * 7) % 256;
            data.extend_from_slice(format!("\x1b[38;2;{};{};{}m", r, g, b).as_bytes());
            data.extend_from_slice(
                format!("\x1b[48;2;{};{};{}m", 255 - r, 255 - g, 255 - b).as_bytes(),
            );

            // Character
            let ch = match (x + y) % 4 {
                0 => '█',
                1 => '▓',
                2 => '▒',
                _ => '░',
            };
            data.extend_from_slice(ch.to_string().as_bytes());
        }
    }

    data.extend_from_slice(b"\x1b[0m"); // Reset
    data
}
