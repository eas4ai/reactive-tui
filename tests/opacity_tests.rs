use reactive_tui::layout::css::apply_utility_classes;
use reactive_tui::layout::style::StyleBuilder;
use reactive_tui::ui::paint::extract_paint_style;

#[test]
fn test_opacity_utilities() {
    // Test various opacity values
    let test_cases = [
        ("opacity-0", 0.0),
        ("opacity-25", 0.25),
        ("opacity-50", 0.5),
        ("opacity-75", 0.75),
        ("opacity-100", 1.0),
    ];

    for (class, expected_alpha) in test_cases {
        let mut sb = apply_utility_classes(
            &format!("text-red-500 bg-blue-500 {}", class),
            StyleBuilder::new(),
        );

        if let Some(paint_style) = extract_paint_style(&mut sb) {
            // Check that opacity was applied to both fg and bg
            assert!(
                (paint_style.fg.a - expected_alpha).abs() < 0.01,
                "FG alpha mismatch for {}: expected {}, got {}",
                class,
                expected_alpha,
                paint_style.fg.a
            );
            assert!(
                (paint_style.bg.a - expected_alpha).abs() < 0.01,
                "BG alpha mismatch for {}: expected {}, got {}",
                class,
                expected_alpha,
                paint_style.bg.a
            );
        } else {
            panic!("No paint style extracted for class: {}", class);
        }
    }
}

#[test]
fn test_opacity_with_existing_alpha() {
    // Test that opacity multiplies with existing alpha
    let mut sb = StyleBuilder::new()
        .text_rgba(1.0, 0.0, 0.0, 0.8) // Red with 80% alpha
        .opacity(0.5); // Apply 50% opacity

    if let Some(paint_style) = extract_paint_style(&mut sb) {
        // Should be 0.8 * 0.5 = 0.4
        assert!(
            (paint_style.fg.a - 0.4).abs() < 0.01,
            "Expected alpha 0.4, got {}",
            paint_style.fg.a
        );
    } else {
        panic!("No paint style extracted");
    }
}

#[test]
fn test_opacity_order_independence() {
    // Test that opacity works regardless of order
    let mut sb1 = apply_utility_classes("opacity-50 text-red-500", StyleBuilder::new());
    let mut sb2 = apply_utility_classes("text-red-500 opacity-50", StyleBuilder::new());

    let paint1 = extract_paint_style(&mut sb1).expect("paint1");
    let paint2 = extract_paint_style(&mut sb2).expect("paint2");

    assert!(
        (paint1.fg.a - paint2.fg.a).abs() < 0.01,
        "Opacity should work regardless of order"
    );
}

#[test]
fn test_opacity_edge_cases() {
    // Test edge cases
    let mut sb_over = apply_utility_classes("opacity-150", StyleBuilder::new()); // Over 100
    let mut sb_negative = apply_utility_classes("opacity--10", StyleBuilder::new()); // Negative (should fail to parse)

    // Over 100 should clamp to 1.0
    if let Some(paint_style) = extract_paint_style(&mut sb_over) {
        // If it parsed, it should be clamped to 1.0
        assert!(paint_style.fg.a <= 1.0);
    }

    // Negative should fail to parse (no paint style should be created from opacity alone)
    let paint_negative = extract_paint_style(&mut sb_negative);
    assert!(
        paint_negative.is_none(),
        "Negative opacity should not parse"
    );
}

#[test]
fn test_border_utility() {
    // Test that border utility adds background when none exists
    let mut sb = apply_utility_classes("border", StyleBuilder::new());

    if let Some(paint_style) = extract_paint_style(&mut sb) {
        // Should have added a dark gray background
        assert!(
            paint_style.bg.r < 0.5 && paint_style.bg.g < 0.5 && paint_style.bg.b < 0.5,
            "Border should add dark background"
        );
    } else {
        panic!("Border should create a paint style");
    }
}

#[test]
fn test_border_with_existing_bg() {
    // Test that border doesn't override existing background
    let mut sb = apply_utility_classes("bg-red-500 border", StyleBuilder::new());

    if let Some(paint_style) = extract_paint_style(&mut sb) {
        // Should still be red-ish (border shouldn't override)
        assert!(
            paint_style.bg.r > 0.5,
            "Border should not override existing background"
        );
    } else {
        panic!("Should have paint style");
    }
}

#[test]
fn test_comprehensive_opacity_styling() {
    // Test a realistic combination with opacity
    let mut sb = apply_utility_classes(
        "text-white bg-blue-600 opacity-75 font-bold p-4",
        StyleBuilder::new(),
    );

    if let Some(paint_style) = extract_paint_style(&mut sb) {
        // Check colors are present and have correct opacity
        assert!(
            paint_style.fg.r > 0.9 && paint_style.fg.g > 0.9 && paint_style.fg.b > 0.9,
            "Should be white-ish"
        );
        assert!(paint_style.bg.b > 0.5, "Should be blue-ish");
        assert!(
            (paint_style.fg.a - 0.75).abs() < 0.01,
            "FG should have 75% opacity"
        );
        assert!(
            (paint_style.bg.a - 0.75).abs() < 0.01,
            "BG should have 75% opacity"
        );
        assert!(
            paint_style
                .attr
                .contains(reactive_tui::core::surface::Attr::BOLD),
            "Should be bold"
        );
    } else {
        panic!("Should have comprehensive paint style");
    }
}
