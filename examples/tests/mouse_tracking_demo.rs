use crossterm::event::{self, Event, KeyCode, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use reactive_tui::core::mouse_tracker::{
    CoordinateSystem, FallbackStrategy, MouseConfig, MouseLevel, MouseTracker, ReportingFormat,
};
use reactive_tui::core::terminal_capabilities::MouseCapabilities;
use std::io;
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    println!("Mouse Tracking Levels Demo");
    println!("=========================");
    println!();

    // Test different mouse levels without terminal interaction
    test_mouse_levels();

    // Interactive demo (commented out for CI compatibility)
    // interactive_mouse_demo()?;

    Ok(())
}

fn test_mouse_levels() {
    println!("🖱️  Testing Mouse Tracking Levels");
    println!("----------------------------------");

    // Create mock capabilities for testing
    let capabilities = MouseCapabilities {
        basic: true,
        drag: true,
        motion: true,
        pixels: false, // Simulate limited terminal
        sgr_mode: true,
        wheel: true,
    };

    let mut tracker = MouseTracker::with_capabilities(capabilities.clone());

    // Test each level
    test_level(&mut tracker, MouseLevel::None, "None - No mouse tracking");
    test_level(&mut tracker, MouseLevel::Basic, "Basic - Click events only");
    test_level(&mut tracker, MouseLevel::Drag, "Drag - Click + drag events");
    test_level(
        &mut tracker,
        MouseLevel::Motion,
        "Motion - All motion events",
    );
    test_level(
        &mut tracker,
        MouseLevel::Pixels,
        "Pixels - Pixel coordinates (not supported)",
    );

    println!();
    println!("📊 Capability Analysis:");
    println!("  Max supported level: {:?}", tracker.max_supported_level());
    println!(
        "  Supports Basic: {}",
        tracker.supports_level(MouseLevel::Basic)
    );
    println!(
        "  Supports Drag: {}",
        tracker.supports_level(MouseLevel::Drag)
    );
    println!(
        "  Supports Motion: {}",
        tracker.supports_level(MouseLevel::Motion)
    );
    println!(
        "  Supports Pixels: {}",
        tracker.supports_level(MouseLevel::Pixels)
    );

    println!();
    test_fallback_strategies(&capabilities);
    test_configurations(&capabilities);

    // Cleanup
    let _ = tracker.shutdown();
}

fn test_level(tracker: &mut MouseTracker, level: MouseLevel, description: &str) {
    println!("Testing {}", description);

    let config = MouseConfig {
        level,
        fallback_strategy: FallbackStrategy::BestAvailable,
        ..Default::default()
    };

    match tracker.initialize(config) {
        Ok(actual_level) => {
            if actual_level == level {
                println!("  ✅ Successfully enabled {:?}", level);
            } else {
                println!(
                    "  ⚠️  Requested {:?}, got {:?} (fallback)",
                    level, actual_level
                );
            }
            println!("  Current level: {:?}", tracker.current_level());
        }
        Err(e) => {
            println!("  ❌ Failed to enable {:?}: {}", level, e);
        }
    }

    println!();
}

fn test_fallback_strategies(capabilities: &MouseCapabilities) {
    println!("🔄 Testing Fallback Strategies");
    println!("------------------------------");

    let strategies = [
        (FallbackStrategy::BestAvailable, "Best Available"),
        (FallbackStrategy::Strict, "Strict"),
        (FallbackStrategy::Disable, "Disable"),
    ];

    for (strategy, name) in &strategies {
        println!("Strategy: {}", name);

        let mut tracker = MouseTracker::with_capabilities(capabilities.clone());

        // Try to enable pixel tracking (not supported in our mock)
        let config = MouseConfig {
            level: MouseLevel::Pixels,
            fallback_strategy: *strategy,
            ..Default::default()
        };

        match tracker.initialize(config) {
            Ok(actual_level) => {
                println!("  ✅ Requested Pixels, got {:?}", actual_level);
            }
            Err(e) => {
                println!("  ❌ Failed: {}", e);
            }
        }

        let _ = tracker.shutdown();
        println!();
    }
}

fn test_configurations(capabilities: &MouseCapabilities) {
    println!("⚙️  Testing Different Configurations");
    println!("------------------------------------");

    let configs = [
        (
            "Basic with SGR",
            MouseConfig {
                level: MouseLevel::Basic,
                reporting_format: ReportingFormat::Sgr,
                ..Default::default()
            },
        ),
        (
            "Motion with Character coords",
            MouseConfig {
                level: MouseLevel::Motion,
                coordinate_system: CoordinateSystem::Character,
                ..Default::default()
            },
        ),
        (
            "Drag with Focus tracking",
            MouseConfig {
                level: MouseLevel::Drag,
                focus_tracking: true,
                ..Default::default()
            },
        ),
        (
            "Motion with all features",
            MouseConfig {
                level: MouseLevel::Motion,
                focus_tracking: true,
                bracketed_paste: true,
                reporting_format: ReportingFormat::Sgr,
                ..Default::default()
            },
        ),
    ];

    for (name, config) in &configs {
        println!("Configuration: {}", name);

        let mut tracker = MouseTracker::with_capabilities(capabilities.clone());

        match tracker.initialize(config.clone()) {
            Ok(level) => {
                println!("  ✅ Enabled level: {:?}", level);
                println!("  Current level: {:?}", tracker.current_level());

                let stats = tracker.stats();
                println!("  Stats: {} events processed", stats.events_processed);
            }
            Err(e) => {
                println!("  ❌ Failed: {}", e);
            }
        }

        let _ = tracker.shutdown();
        println!();
    }
}

// Interactive demo (commented out for CI compatibility)
#[allow(dead_code)]
fn interactive_mouse_demo() -> io::Result<()> {
    println!("🎮 Interactive Mouse Demo");
    println!("-------------------------");
    println!("This demo will test mouse tracking in your terminal.");
    println!("Press 'q' to quit, '1'-'4' to change mouse levels.");
    println!();

    enable_raw_mode()?;

    // Detect terminal capabilities
    let capabilities = detect_mouse_capabilities();
    let mut tracker = MouseTracker::with_capabilities(capabilities);

    // Start with basic tracking
    let config = MouseConfig {
        level: MouseLevel::Basic,
        ..Default::default()
    };

    let mut current_level = tracker.initialize(config)?;

    println!("Started with mouse level: {:?}", current_level);
    println!("Move mouse and click to test. Use keys 1-4 to change levels:");
    println!("  1 = Basic, 2 = Drag, 3 = Motion, 4 = Pixels");
    println!();

    let mut event_count = 0;
    let start_time = Instant::now();

    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('1') => {
                        current_level = change_level(&mut tracker, MouseLevel::Basic)?;
                    }
                    KeyCode::Char('2') => {
                        current_level = change_level(&mut tracker, MouseLevel::Drag)?;
                    }
                    KeyCode::Char('3') => {
                        current_level = change_level(&mut tracker, MouseLevel::Motion)?;
                    }
                    KeyCode::Char('4') => {
                        current_level = change_level(&mut tracker, MouseLevel::Pixels)?;
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => {
                    event_count += 1;

                    let event_type = match mouse.kind {
                        MouseEventKind::Down(_) => "Down",
                        MouseEventKind::Up(_) => "Up",
                        MouseEventKind::Moved => "Move",
                        MouseEventKind::Drag(_) => "Drag",
                        MouseEventKind::ScrollDown => "ScrollDown",
                        MouseEventKind::ScrollUp => "ScrollUp",
                        MouseEventKind::ScrollLeft => "ScrollLeft",
                        MouseEventKind::ScrollRight => "ScrollRight",
                    };

                    println!(
                        "Mouse {}: ({}, {}) - Level: {:?} - Event #{}",
                        event_type, mouse.column, mouse.row, current_level, event_count
                    );

                    // Show statistics every 10 events
                    if event_count % 10 == 0 {
                        let elapsed = start_time.elapsed();
                        let rate = event_count as f32 / elapsed.as_secs_f32();
                        println!(
                            "  📊 {} events in {:.1}s ({:.1} events/sec)",
                            event_count,
                            elapsed.as_secs_f32(),
                            rate
                        );
                    }
                }
                Event::Resize(width, height) => {
                    println!("Terminal resized: {}x{}", width, height);
                }
                _ => {}
            }
        }
    }

    tracker.shutdown()?;
    disable_raw_mode()?;

    println!();
    println!("Demo completed. Final statistics:");
    let stats = tracker.stats();
    println!("  Total events: {}", stats.events_processed);
    println!("  Click events: {}", stats.click_events);
    println!("  Move events: {}", stats.move_events);
    println!("  Drag events: {}", stats.drag_events);
    println!("  Wheel events: {}", stats.wheel_events);

    Ok(())
}

#[allow(dead_code)]
fn change_level(tracker: &mut MouseTracker, level: MouseLevel) -> io::Result<MouseLevel> {
    let config = MouseConfig {
        level,
        fallback_strategy: FallbackStrategy::BestAvailable,
        ..Default::default()
    };

    let actual_level = tracker.initialize(config)?;

    if actual_level == level {
        println!("✅ Changed to mouse level: {:?}", level);
    } else {
        println!(
            "⚠️  Requested {:?}, got {:?} (fallback)",
            level, actual_level
        );
    }

    Ok(actual_level)
}

fn detect_mouse_capabilities() -> MouseCapabilities {
    // In a real implementation, this would probe the terminal
    // For demo purposes, we'll create reasonable defaults
    let term_program = std::env::var("TERM_PROGRAM")
        .unwrap_or_default()
        .to_lowercase();

    let is_modern = term_program.contains("kitty")
        || term_program.contains("wezterm")
        || term_program.contains("iterm");

    let is_high_perf = term_program.contains("kitty") || term_program.contains("wezterm");

    MouseCapabilities {
        basic: true,
        drag: is_modern,
        motion: is_modern,
        pixels: is_high_perf,
        sgr_mode: is_modern,
        wheel: true,
    }
}
