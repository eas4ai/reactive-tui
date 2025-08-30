use reactive_tui::core::surface::Rgba;
use std::time::Instant;

fn main() {
    println!("Color Optimization Demo");
    println!("======================");
    println!();

    // Test epsilon-based color comparisons
    test_epsilon_comparisons();

    // Test color blending performance
    test_color_blending();

    // Test color operations
    test_color_operations();

    // Test SIMD availability
    test_simd_availability();
}

fn test_epsilon_comparisons() {
    println!("🎨 Testing Epsilon-based Color Comparisons");
    println!("------------------------------------------");

    // Colors that are very close but not exactly equal
    let color1 = Rgba::new(0.5, 0.3, 0.8, 1.0);
    let color2 = Rgba::new(0.5001, 0.3001, 0.8001, 1.0); // Slightly different
    let color3 = Rgba::new(0.6, 0.4, 0.9, 1.0); // Clearly different

    println!(
        "Color 1: r={:.4}, g={:.4}, b={:.4}, a={:.4}",
        color1.r, color1.g, color1.b, color1.a
    );
    println!(
        "Color 2: r={:.4}, g={:.4}, b={:.4}, a={:.4}",
        color2.r, color2.g, color2.b, color2.a
    );
    println!(
        "Color 3: r={:.4}, g={:.4}, b={:.4}, a={:.4}",
        color3.r, color3.g, color3.b, color3.a
    );
    println!();

    // Test exact equality
    println!("Exact equality (==):");
    println!("  Color1 == Color2: {}", color1 == color2);
    println!("  Color1 == Color3: {}", color1 == color3);
    println!();

    // Test epsilon-based equality
    println!("Epsilon-based equality (approx_eq):");
    println!("  Color1 ≈ Color2: {}", color1.approx_eq(color2));
    println!("  Color1 ≈ Color3: {}", color1.approx_eq(color3));
    println!();

    // Test custom epsilon
    println!("Custom epsilon (0.01):");
    println!("  Color1 ≈ Color2: {}", color1.equals_epsilon(color2, 0.01));
    println!("  Color1 ≈ Color3: {}", color1.equals_epsilon(color3, 0.01));
    println!();

    // Performance comparison
    let iterations = 1_000_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = color1 == color2;
    }
    let exact_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = color1.approx_eq(color2);
    }
    let epsilon_time = start.elapsed();

    println!("Performance comparison ({} iterations):", iterations);
    println!(
        "  Exact equality: {:.2}ms",
        exact_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Epsilon equality: {:.2}ms",
        epsilon_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Overhead: {:.1}%",
        ((epsilon_time.as_secs_f64() - exact_time.as_secs_f64()) / exact_time.as_secs_f64())
            * 100.0
    );
    println!();
}

fn test_color_blending() {
    println!("🌈 Testing Color Blending Operations");
    println!("-----------------------------------");

    let red = Rgba::new(1.0, 0.0, 0.0, 1.0);
    let blue = Rgba::new(0.0, 0.0, 1.0, 1.0);
    let transparent_green = Rgba::new(0.0, 1.0, 0.0, 0.5);

    println!("Base colors:");
    println!("  Red: {:?}", red);
    println!("  Blue: {:?}", blue);
    println!("  Transparent Green: {:?}", transparent_green);
    println!();

    // Test blending
    let purple = red.blend(blue, 0.5);
    let red_with_green = red.blend(transparent_green, 0.3);

    println!("Blended colors:");
    println!("  Red + Blue (50%): {:?}", purple);
    println!("  Red + Green (30%): {:?}", red_with_green);
    println!();

    // Test alpha operations
    let semi_red = red.with_alpha(0.5);
    let premult_red = red.premultiply_alpha();

    println!("Alpha operations:");
    println!("  Red with 50% alpha: {:?}", semi_red);
    println!("  Red premultiplied: {:?}", premult_red);
    println!();

    // Performance test
    let iterations = 1_000_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = red.blend(blue, 0.5);
    }
    let blend_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = red.blend_fast(blue, 0.5);
    }
    let fast_blend_time = start.elapsed();

    println!("Blending performance ({} iterations):", iterations);
    println!(
        "  Standard blend: {:.2}ms",
        blend_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Fast blend: {:.2}ms",
        fast_blend_time.as_secs_f64() * 1000.0
    );

    if fast_blend_time < blend_time {
        let improvement = ((blend_time.as_secs_f64() - fast_blend_time.as_secs_f64())
            / blend_time.as_secs_f64())
            * 100.0;
        println!("  Improvement: {:.1}% faster", improvement);
    } else {
        println!("  No improvement (SIMD not available or overhead)");
    }
    println!();
}

fn test_color_operations() {
    println!("🔬 Testing Advanced Color Operations");
    println!("-----------------------------------");

    let color = Rgba::new(0.8, 0.4, 0.2, 1.0);

    // Color space conversions
    let linear = color.to_linear();
    let back_to_srgb = linear.to_srgb();

    println!("Color space conversions:");
    println!("  Original sRGB: {:?}", color);
    println!("  Linear: {:?}", linear);
    println!("  Back to sRGB: {:?}", back_to_srgb);
    println!(
        "  Conversion accuracy: {}",
        if color.approx_eq(back_to_srgb) {
            "✅ Good"
        } else {
            "⚠️ Poor"
        }
    );
    println!();

    // Luminance and contrast
    let white = Rgba::white();
    let black = Rgba::black();
    let gray = Rgba::new(0.5, 0.5, 0.5, 1.0);

    println!("Luminance values:");
    println!("  White: {:.3}", white.luminance());
    println!("  Gray: {:.3}", gray.luminance());
    println!("  Black: {:.3}", black.luminance());
    println!("  Test color: {:.3}", color.luminance());
    println!();

    println!("Contrast ratios (WCAG standard):");
    println!("  White vs Black: {:.2}:1", white.contrast_ratio(black));
    println!("  White vs Gray: {:.2}:1", white.contrast_ratio(gray));
    println!(
        "  Test color vs White: {:.2}:1",
        color.contrast_ratio(white)
    );
    println!(
        "  Test color vs Black: {:.2}:1",
        color.contrast_ratio(black)
    );
    println!();

    // WCAG compliance check
    let contrast_white = color.contrast_ratio(white);
    let contrast_black = color.contrast_ratio(black);

    println!("WCAG Compliance:");
    println!(
        "  AA Normal (4.5:1): {} vs white, {} vs black",
        if contrast_white >= 4.5 {
            "✅ Pass"
        } else {
            "❌ Fail"
        },
        if contrast_black >= 4.5 {
            "✅ Pass"
        } else {
            "❌ Fail"
        }
    );
    println!(
        "  AAA Normal (7:1): {} vs white, {} vs black",
        if contrast_white >= 7.0 {
            "✅ Pass"
        } else {
            "❌ Fail"
        },
        if contrast_black >= 7.0 {
            "✅ Pass"
        } else {
            "❌ Fail"
        }
    );
    println!();
}

fn test_simd_availability() {
    println!("⚡ SIMD Availability Test");
    println!("------------------------");

    #[cfg(feature = "simd")]
    {
        println!("✅ SIMD feature is enabled");

        // Test SIMD operations
        let color1 = Rgba::new(1.0, 0.5, 0.2, 1.0);
        let color2 = Rgba::new(0.2, 0.8, 0.9, 0.7);

        let blended_scalar = color1.blend(color2, 0.5);
        let blended_simd = color1.blend_simd(color2, 0.5);

        println!("SIMD vs Scalar comparison:");
        println!("  Scalar result: {:?}", blended_scalar);
        println!("  SIMD result: {:?}", blended_simd);
        println!(
            "  Results match: {}",
            blended_scalar.approx_eq(blended_simd)
        );

        // Performance comparison
        let iterations = 1_000_000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = color1.blend(color2, 0.5);
        }
        let scalar_time = start.elapsed();

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = color1.blend_simd(color2, 0.5);
        }
        let simd_time = start.elapsed();

        println!("Performance comparison ({} iterations):", iterations);
        println!("  Scalar: {:.2}ms", scalar_time.as_secs_f64() * 1000.0);
        println!("  SIMD: {:.2}ms", simd_time.as_secs_f64() * 1000.0);

        if simd_time < scalar_time {
            let speedup = scalar_time.as_secs_f64() / simd_time.as_secs_f64();
            println!("  SIMD speedup: {:.1}x faster", speedup);
        } else {
            println!("  No speedup (overhead or not optimized)");
        }
    }

    #[cfg(not(feature = "simd"))]
    {
        println!("❌ SIMD feature is not enabled");
        println!("To enable SIMD optimizations:");
        println!("  1. Use nightly Rust: rustup default nightly");
        println!("  2. Build with SIMD: cargo build --features simd");
        println!("  3. Note: Requires portable_simd feature (unstable)");
    }

    println!();
    println!("💡 Tips for optimal color performance:");
    println!("  • Use approx_eq() instead of == for color comparisons");
    println!("  • Use blend_fast() for color blending operations");
    println!("  • Enable SIMD feature for maximum performance");
    println!("  • Consider premultiplied alpha for complex blending");
}
