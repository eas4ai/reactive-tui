use reactive_tui::core::render_stats::{DetailedFrameStats, RenderStatsCollector};
use std::time::{Duration, Instant};

fn main() {
    println!("Reactive-TUI Enhanced Statistics Demo");
    println!("====================================");
    println!();

    // Create a stats collector
    let mut collector = RenderStatsCollector::new(60);

    println!("Simulating 60 frames of rendering with varying performance...");

    // Simulate different performance scenarios
    let scenarios = [
        "Excellent Performance",
        "Good Performance",
        "Poor Performance",
        "Inconsistent Performance",
    ];

    for (i, name) in scenarios.iter().enumerate() {
        println!("\n🎯 Testing: {}", name);
        println!("{}", "=".repeat(50));

        // Clear previous stats
        collector.clear();

        // Run simulation based on index
        match i {
            0 => simulate_excellent_performance(&mut collector),
            1 => simulate_good_performance(&mut collector),
            2 => simulate_poor_performance(&mut collector),
            3 => simulate_inconsistent_performance(&mut collector),
            _ => unreachable!(),
        }

        // Display results
        display_performance_analysis(&collector);
    }
}

fn simulate_excellent_performance(collector: &mut RenderStatsCollector) {
    let start_time = Instant::now();

    for i in 0..60 {
        let frame_start = start_time + Duration::from_millis(i * 16); // 60 FPS

        let stats = DetailedFrameStats {
            frame_time: Duration::from_millis(15), // Consistent 15ms frames
            diff_time: Duration::from_millis(2),
            write_time: Duration::from_millis(1),
            surface_time: Duration::from_millis(12),
            cells_updated: 50, // Low update rate - efficient
            cells_total: 1920, // 80x24
            bytes_written: 200,
            spans_written: 5,
            rows_changed: 3,
            timestamp: frame_start,
            memory_usage: 64 * 1024, // 64KB
        };

        collector.record_frame(stats);
    }
}

fn simulate_good_performance(collector: &mut RenderStatsCollector) {
    let start_time = Instant::now();

    for i in 0..60 {
        let frame_start = start_time + Duration::from_millis(i * 20); // 50 FPS

        let stats = DetailedFrameStats {
            frame_time: Duration::from_millis(18 + (i % 4)), // Slight variation
            diff_time: Duration::from_millis(3),
            write_time: Duration::from_millis(2),
            surface_time: Duration::from_millis(13),
            cells_updated: 150, // Medium update rate
            cells_total: 1920,
            bytes_written: 600,
            spans_written: 12,
            rows_changed: 8,
            timestamp: frame_start,
            memory_usage: 96 * 1024, // 96KB
        };

        collector.record_frame(stats);
    }
}

fn simulate_poor_performance(collector: &mut RenderStatsCollector) {
    let start_time = Instant::now();

    for i in 0..60 {
        let frame_start = start_time + Duration::from_millis(i * 50); // 20 FPS

        let stats = DetailedFrameStats {
            frame_time: Duration::from_millis(45), // Slow frames
            diff_time: Duration::from_millis(15),
            write_time: Duration::from_millis(10),
            surface_time: Duration::from_millis(20),
            cells_updated: 1500, // High update rate - inefficient
            cells_total: 1920,
            bytes_written: 5000,
            spans_written: 80,
            rows_changed: 24,
            timestamp: frame_start,
            memory_usage: 256 * 1024, // 256KB
        };

        collector.record_frame(stats);
    }
}

fn simulate_inconsistent_performance(collector: &mut RenderStatsCollector) {
    let start_time = Instant::now();
    let mut cumulative_time = 0u64;

    for i in 0..60 {
        // Highly variable frame times
        let frame_time_ms = if i % 10 == 0 {
            100 // Occasional spike
        } else if i % 3 == 0 {
            30 // Sometimes slow
        } else {
            15 // Usually fast
        };

        cumulative_time += frame_time_ms;
        let frame_start = start_time + Duration::from_millis(cumulative_time);

        let stats = DetailedFrameStats {
            frame_time: Duration::from_millis(frame_time_ms),
            diff_time: Duration::from_millis(frame_time_ms / 3),
            write_time: Duration::from_millis(frame_time_ms / 5),
            surface_time: Duration::from_millis(frame_time_ms / 2),
            cells_updated: (frame_time_ms * 10) as u32, // Updates correlate with time
            cells_total: 1920,
            bytes_written: frame_time_ms * 50,
            spans_written: (frame_time_ms / 2) as u32,
            rows_changed: (frame_time_ms / 4) as u32,
            timestamp: frame_start,
            memory_usage: (128 + frame_time_ms * 2) as usize * 1024,
        };

        collector.record_frame(stats);
    }
}

fn display_performance_analysis(collector: &RenderStatsCollector) {
    let metrics = collector.get_metrics();

    println!("📊 Performance Metrics:");
    println!("  FPS: {:.1}", metrics.fps);
    println!(
        "  Avg frame time: {:.1}ms",
        metrics.avg_frame_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Min frame time: {:.1}ms",
        metrics.min_frame_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Max frame time: {:.1}ms",
        metrics.max_frame_time.as_secs_f64() * 1000.0
    );
    println!("  Efficiency: {:.1}%", metrics.efficiency_ratio * 100.0);
    println!("  Avg bytes/frame: {:.0}", metrics.avg_bytes_per_frame);
    println!("  Stability: {:.2}", metrics.stability_score);
    println!("  Memory: {:.0} KB", metrics.memory_usage as f64 / 1024.0);

    println!("\n🎯 Assessment:");
    println!("  Grade: {}", collector.performance_grade());
    println!(
        "  Status: {}",
        if collector.is_performance_good() {
            "✅ Good"
        } else {
            "⚠️  Needs improvement"
        }
    );

    // Performance insights
    println!("\n💡 Analysis:");
    if metrics.fps >= 60.0 {
        println!("  • Excellent frame rate");
    } else if metrics.fps >= 30.0 {
        println!("  • Acceptable frame rate");
    } else {
        println!("  • Frame rate below optimal");
    }

    if metrics.efficiency_ratio < 0.2 {
        println!("  • Very efficient - few cells updated per frame");
    } else if metrics.efficiency_ratio > 0.8 {
        println!("  • Low efficiency - many cells updated per frame");
    }

    if metrics.stability_score >= 0.9 {
        println!("  • Very stable frame times");
    } else if metrics.stability_score >= 0.7 {
        println!("  • Reasonably stable frame times");
    } else {
        println!("  • Inconsistent frame times detected");
    }

    println!("  • Uptime: {:.1}s", collector.uptime().as_secs_f64());
}
