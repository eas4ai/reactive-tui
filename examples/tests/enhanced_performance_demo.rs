use reactive_tui::core::renderer::Renderer;
use reactive_tui::core::surface::{Attr, Cell, Rgba};
use reactive_tui::error::Result;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    println!("Enhanced Performance Demo - Comprehensive Statistics");
    println!("==================================================");
    println!();

    // Create renderer with detailed stats enabled
    let mut renderer = Renderer::new(80, 24)?;
    renderer.enable_high_performance_mode()?;
    renderer.enable_detailed_stats();
    renderer.set_debug_overlay(true);

    println!("Running performance test with detailed statistics...");
    println!("(This would normally display in terminal with debug overlay)");
    println!();

    // Run test frames
    let test_start = Instant::now();
    for frame in 0..60 {
        renderer.begin_frame()?;

        // Create varying workload to test statistics
        let surface = renderer.surface_mut();
        let workload_intensity = if frame < 20 {
            1.0 // Light workload
        } else if frame < 40 {
            0.5 // Medium workload  
        } else {
            0.2 // Heavy workload (less updates)
        };

        // Fill surface with pattern based on workload
        for y in 0..24 {
            for x in 0..80 {
                if (x + y + frame) as f32 * workload_intensity % 10.0 < 1.0 {
                    let intensity = ((x + y + frame) % 256) as f32 / 255.0;
                    let bg = Rgba {
                        r: intensity * 0.2,
                        g: intensity * 0.4,
                        b: intensity * 0.8,
                        a: 1.0,
                    };
                    let fg = Rgba {
                        r: 1.0 - intensity,
                        g: intensity,
                        b: 0.9,
                        a: 1.0,
                    };

                    let ch = match frame % 4 {
                        0 => '█',
                        1 => '▓',
                        2 => '▒',
                        _ => '░',
                    };

                    let cell = Cell {
                        ch,
                        fg,
                        bg,
                        attr: Attr::empty(),
                    };
                    surface.set(x, y, cell);
                }
            }
        }

        renderer.end_frame()?;

        // Add small delay to simulate realistic frame timing
        std::thread::sleep(Duration::from_millis(8));
    }

    let total_time = test_start.elapsed();

    // Display comprehensive statistics
    println!("Performance Test Results:");
    println!("========================");
    println!("Total test time: {:.2}s", total_time.as_secs_f64());
    println!();

    // Get performance metrics
    let metrics = renderer.performance_metrics();
    println!("📊 Performance Metrics:");
    println!("  FPS: {:.1}", metrics.fps);
    println!(
        "  Average frame time: {:.2}ms",
        metrics.avg_frame_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Min frame time: {:.2}ms",
        metrics.min_frame_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Max frame time: {:.2}ms",
        metrics.max_frame_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Efficiency ratio: {:.1}%",
        metrics.efficiency_ratio * 100.0
    );
    println!("  Avg bytes per frame: {:.0}", metrics.avg_bytes_per_frame);
    println!(
        "  Avg write time: {:.2}ms",
        metrics.avg_write_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Avg diff time: {:.2}ms",
        metrics.avg_diff_time.as_secs_f64() * 1000.0
    );
    println!("  Total frames: {}", metrics.total_frames);
    println!(
        "  Memory usage: {:.1} KB",
        metrics.memory_usage as f64 / 1024.0
    );
    println!("  Stability score: {:.2}", metrics.stability_score);
    println!();

    // Performance assessment
    println!("🎯 Performance Assessment:");
    println!("  Grade: {}", renderer.performance_grade());
    println!(
        "  Status: {}",
        if renderer.is_performance_good() {
            "✅ Good performance"
        } else {
            "⚠️  Performance could be improved"
        }
    );
    println!();

    // Latest frame details
    if let Some(latest) = renderer.latest_detailed_stats() {
        println!("🔍 Latest Frame Details:");
        println!(
            "  Frame time: {:.2}ms",
            latest.frame_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Diff time: {:.2}ms",
            latest.diff_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Write time: {:.2}ms",
            latest.write_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Surface time: {:.2}ms",
            latest.surface_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Cells updated: {} / {}",
            latest.cells_updated, latest.cells_total
        );
        println!("  Bytes written: {}", latest.bytes_written);
        println!("  Spans written: {}", latest.spans_written);
        println!("  Rows changed: {}", latest.rows_changed);
        println!();
    }

    // Terminal write statistics
    let write_stats = renderer.write_stats();
    println!("💾 Terminal Write Statistics:");
    println!(
        "  Buffer size: {:.1} MB",
        write_stats.buffer_size as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  Total bytes written: {:.1} KB",
        write_stats.bytes_written as f64 / 1024.0
    );
    println!("  Flush count: {}", write_stats.flush_count);
    println!(
        "  Write time: {:.2}ms",
        write_stats.write_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Buffer utilization: {:.1}%",
        write_stats.buffer_utilization * 100.0
    );
    println!();

    // Performance recommendations
    println!("💡 Performance Recommendations:");
    if metrics.fps < 30.0 {
        println!("  • Consider reducing update frequency or complexity");
    }
    if metrics.efficiency_ratio < 0.1 {
        println!("  • High efficiency - most cells are being updated each frame");
    } else if metrics.efficiency_ratio > 0.8 {
        println!("  • Low efficiency - consider optimizing diff algorithm");
    }
    if metrics.stability_score < 0.7 {
        println!("  • Frame times are inconsistent - check for blocking operations");
    }
    if write_stats.buffer_utilization > 0.9 {
        println!(
            "  • Buffer utilization is high - consider larger buffer or more frequent flushes"
        );
    }

    if metrics.fps >= 30.0 && metrics.stability_score >= 0.8 {
        println!("  ✨ Performance is excellent! No recommendations.");
    }

    renderer.shutdown()?;

    Ok(())
}
