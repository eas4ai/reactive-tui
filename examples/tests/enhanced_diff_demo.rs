use reactive_tui::core::surface::{Attr, Cell, DiffWriter, Rgba, Surface};
use std::time::Instant;

fn main() {
    println!("Enhanced Diff Algorithm Demo");
    println!("===========================");
    println!();

    // Test different scenarios
    test_no_changes();
    test_small_changes();
    test_large_changes();
    test_color_precision();
    test_performance_comparison();
}

fn test_no_changes() {
    println!("🔍 Test 1: No Changes");
    println!("---------------------");

    let surface1 = create_test_surface(40, 10, "Hello World");
    let surface2 = surface1.clone_into_new();

    let mut diff = DiffWriter::new();
    diff.diff(&surface1, &surface2, false);

    let stats = diff.detailed_stats();
    println!(
        "  Cells changed: {} / {}",
        stats.cells_changed, stats.cells_total
    );
    println!("  Efficiency ratio: {:.1}%", stats.efficiency_ratio * 100.0);
    println!("  Bytes written: {}", stats.bytes_written);
    println!("  Assessment: {}", diff.performance_assessment());
    println!(
        "  Is efficient: {}",
        if diff.is_efficient() {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!();
}

fn test_small_changes() {
    println!("🔍 Test 2: Small Changes");
    println!("------------------------");

    let surface1 = create_test_surface(40, 10, "Hello World");
    let mut surface2 = surface1.clone_into_new();

    // Make small changes
    let fg = Rgba::new(1.0, 0.0, 0.0, 1.0);
    let bg = Rgba::new(0.0, 0.0, 0.0, 1.0);
    surface2.set(
        5,
        2,
        Cell {
            ch: 'X',
            fg,
            bg,
            attr: Attr::BOLD,
        },
    );
    surface2.set(
        10,
        3,
        Cell {
            ch: 'Y',
            fg,
            bg,
            attr: Attr::ITALIC,
        },
    );
    surface2.set(
        15,
        4,
        Cell {
            ch: 'Z',
            fg,
            bg,
            attr: Attr::UNDERLINE,
        },
    );

    let mut diff = DiffWriter::new();
    diff.diff(&surface1, &surface2, false);

    let stats = diff.detailed_stats();
    println!(
        "  Cells changed: {} / {}",
        stats.cells_changed, stats.cells_total
    );
    println!("  Efficiency ratio: {:.1}%", stats.efficiency_ratio * 100.0);
    println!("  Rows changed: {}", stats.rows_changed);
    println!("  Spans written: {}", stats.spans_written);
    println!("  Color changes: {}", stats.color_changes);
    println!("  Attr changes: {}", stats.attr_changes);
    println!("  Cursor moves: {}", stats.cursor_moves);
    println!("  Bytes written: {}", stats.bytes_written);
    println!("  Assessment: {}", diff.performance_assessment());
    println!();
}

fn test_large_changes() {
    println!("🔍 Test 3: Large Changes");
    println!("------------------------");

    let surface1 = create_test_surface(40, 10, "Hello World");
    let surface2 = create_test_surface(40, 10, "Goodbye Universe");

    let mut diff = DiffWriter::new();
    diff.diff(&surface1, &surface2, false);

    let stats = diff.detailed_stats();
    println!(
        "  Cells changed: {} / {}",
        stats.cells_changed, stats.cells_total
    );
    println!("  Efficiency ratio: {:.1}%", stats.efficiency_ratio * 100.0);
    println!("  Rows changed: {}", stats.rows_changed);
    println!("  Spans written: {}", stats.spans_written);
    println!("  Color changes: {}", stats.color_changes);
    println!("  Bytes written: {}", stats.bytes_written);
    println!("  Assessment: {}", diff.performance_assessment());
    println!();
}

fn test_color_precision() {
    println!("🎨 Test 4: Color Precision");
    println!("--------------------------");

    let mut surface1 = Surface::new(20, 5);
    let mut surface2 = Surface::new(20, 5);

    // Create colors that are very close but not identical
    let color1 = Rgba::new(0.5, 0.3, 0.8, 1.0);
    let color2 = Rgba::new(0.5001, 0.3001, 0.8001, 1.0); // Slightly different

    surface1.fill_rect(
        reactive_tui::core::geometry::Rect::from_coords(0, 0, 20, 5),
        '█',
        Rgba::white(),
        color1,
        Attr::empty(),
    );

    surface2.fill_rect(
        reactive_tui::core::geometry::Rect::from_coords(0, 0, 20, 5),
        '█',
        Rgba::white(),
        color2,
        Attr::empty(),
    );

    // Test with exact comparison
    let mut diff_exact = DiffWriter::with_options(false, true);
    diff_exact.diff(&surface1, &surface2, false);
    let stats_exact = diff_exact.detailed_stats();

    // Test with epsilon comparison
    let mut diff_epsilon = DiffWriter::with_options(true, true);
    diff_epsilon.diff(&surface1, &surface2, false);
    let stats_epsilon = diff_epsilon.detailed_stats();

    println!("  Color difference: {:.4}", (color1.r - color2.r).abs());
    println!("  Exact comparison:");
    println!(
        "    Cells changed: {} / {}",
        stats_exact.cells_changed, stats_exact.cells_total
    );
    println!(
        "    Efficiency: {:.1}%",
        stats_exact.efficiency_ratio * 100.0
    );
    println!("  Epsilon comparison:");
    println!(
        "    Cells changed: {} / {}",
        stats_epsilon.cells_changed, stats_epsilon.cells_total
    );
    println!(
        "    Efficiency: {:.1}%",
        stats_epsilon.efficiency_ratio * 100.0
    );
    println!(
        "  Improvement: {} fewer cell updates with epsilon comparison",
        stats_exact
            .cells_changed
            .saturating_sub(stats_epsilon.cells_changed)
    );
    println!();
}

fn test_performance_comparison() {
    println!("⚡ Test 5: Performance Comparison");
    println!("--------------------------------");

    let surface1 = create_complex_surface(80, 24);
    let surface2 = create_complex_surface_variant(80, 24);

    let iterations = 1000;

    // Test exact comparison performance
    let start = Instant::now();
    for _ in 0..iterations {
        let mut diff = DiffWriter::with_options(false, true);
        diff.diff(&surface1, &surface2, false);
    }
    let exact_time = start.elapsed();

    // Test epsilon comparison performance
    let start = Instant::now();
    for _ in 0..iterations {
        let mut diff = DiffWriter::with_options(true, true);
        diff.diff(&surface1, &surface2, false);
    }
    let epsilon_time = start.elapsed();

    // Get final statistics
    let mut diff_final = DiffWriter::new();
    diff_final.diff(&surface1, &surface2, false);
    let final_stats = diff_final.detailed_stats();

    println!("  Surface size: {}x{} ({} cells)", 80, 24, 80 * 24);
    println!("  Test iterations: {}", iterations);
    println!(
        "  Exact comparison: {:.2}ms",
        exact_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Epsilon comparison: {:.2}ms",
        epsilon_time.as_secs_f64() * 1000.0
    );

    if epsilon_time < exact_time {
        let improvement = ((exact_time.as_secs_f64() - epsilon_time.as_secs_f64())
            / exact_time.as_secs_f64())
            * 100.0;
        println!("  Performance improvement: {:.1}% faster", improvement);
    } else {
        let overhead = ((epsilon_time.as_secs_f64() - exact_time.as_secs_f64())
            / exact_time.as_secs_f64())
            * 100.0;
        println!("  Performance overhead: {:.1}% slower", overhead);
    }

    println!("  Final diff statistics:");
    println!(
        "    Efficiency ratio: {:.1}%",
        final_stats.efficiency_ratio * 100.0
    );
    println!("    Color changes: {}", final_stats.color_changes);
    println!("    Attribute changes: {}", final_stats.attr_changes);
    println!("    Cursor moves: {}", final_stats.cursor_moves);
    println!("    Assessment: {}", diff_final.performance_assessment());
    println!();

    println!("💡 Optimization Tips:");
    println!("  • Use epsilon comparison for better color accuracy");
    println!("  • Monitor efficiency ratio - aim for < 30%");
    println!("  • Minimize color and attribute changes");
    println!("  • Group changes into spans to reduce cursor moves");
}

fn create_test_surface(width: usize, height: usize, text: &str) -> Surface {
    let mut surface = Surface::new(width, height);
    let fg = Rgba::white();
    let bg = Rgba::black();

    surface.write_str(2, 2, text, fg, bg, Attr::empty());
    surface
}

fn create_complex_surface(width: usize, height: usize) -> Surface {
    let mut surface = Surface::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let intensity = ((x + y) % 256) as f32 / 255.0;
            let fg = Rgba::new(intensity, 1.0 - intensity, 0.5, 1.0);
            let bg = Rgba::new(0.1, 0.1, intensity * 0.3, 1.0);

            let ch = match (x + y) % 4 {
                0 => '█',
                1 => '▓',
                2 => '▒',
                _ => '░',
            };

            let attr = if (x + y) % 8 == 0 {
                Attr::BOLD
            } else if (x + y) % 12 == 0 {
                Attr::ITALIC
            } else {
                Attr::empty()
            };

            surface.set(x, y, Cell { ch, fg, bg, attr });
        }
    }

    surface
}

fn create_complex_surface_variant(width: usize, height: usize) -> Surface {
    let mut surface = create_complex_surface(width, height);

    // Make some subtle changes
    for y in 0..height {
        for x in 0..width {
            if (x + y) % 20 == 0 {
                let mut cell = surface.get(x, y);
                // Slightly modify colors
                cell.fg.r = (cell.fg.r + 0.001).min(1.0);
                cell.bg.g = (cell.bg.g + 0.001).min(1.0);
                surface.set(x, y, cell);
            }
        }
    }

    surface
}
