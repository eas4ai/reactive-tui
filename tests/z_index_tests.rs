use reactive_tui::layout::css::apply_utility_classes;
use reactive_tui::layout::style::StyleBuilder;

#[test]
fn test_z_index_utilities() {
    // Test common z-index values
    let test_cases = [
        ("z-auto", 0),
        ("z-0", 0),
        ("z-10", 10),
        ("z-20", 20),
        ("z-30", 30),
        ("z-40", 40),
        ("z-50", 50),
    ];

    for (class, expected_z) in test_cases {
        let sb = apply_utility_classes(class, StyleBuilder::new());
        assert_eq!(
            sb.get_z_index(),
            Some(expected_z),
            "Z-index mismatch for {}: expected {}, got {:?}",
            class,
            expected_z,
            sb.get_z_index()
        );
    }
}

#[test]
fn test_custom_z_index_values() {
    // Test custom z-index values
    let test_cases = [
        ("z-100", 100),
        ("z-999", 999),
        ("z-1000", 1000),
        ("z--10", -10), // Negative z-index
    ];

    for (class, expected_z) in test_cases {
        let sb = apply_utility_classes(class, StyleBuilder::new());
        assert_eq!(
            sb.get_z_index(),
            Some(expected_z),
            "Custom z-index mismatch for {}: expected {}, got {:?}",
            class,
            expected_z,
            sb.get_z_index()
        );
    }
}

#[test]
fn test_position_utilities_set_z_index() {
    // Test that position utilities set appropriate z-index values
    let test_cases = [
        ("static", 0),
        ("relative", 1),
        ("absolute", 10),
        ("fixed", 50),
        ("sticky", 20),
    ];

    for (class, expected_z) in test_cases {
        let sb = apply_utility_classes(class, StyleBuilder::new());
        assert_eq!(
            sb.get_z_index(),
            Some(expected_z),
            "Position z-index mismatch for {}: expected {}, got {:?}",
            class,
            expected_z,
            sb.get_z_index()
        );
    }
}

#[test]
fn test_z_index_override() {
    // Test that explicit z-index overrides position-based z-index
    let sb = apply_utility_classes("absolute z-100", StyleBuilder::new());
    assert_eq!(
        sb.get_z_index(),
        Some(100),
        "Explicit z-index should override position z-index"
    );

    let sb2 = apply_utility_classes("z-5 fixed", StyleBuilder::new());
    assert_eq!(
        sb2.get_z_index(),
        Some(50),
        "Later position should override earlier z-index"
    );
}

#[test]
fn test_modal_z_index_scenario() {
    // Test realistic modal scenario
    let backdrop = apply_utility_classes("fixed z-40 bg-black opacity-50", StyleBuilder::new());
    let modal = apply_utility_classes("fixed z-50 bg-white", StyleBuilder::new());

    assert_eq!(backdrop.get_z_index(), Some(40), "Backdrop should be z-40");
    assert_eq!(modal.get_z_index(), Some(50), "Modal should be z-50");

    // Modal should be above backdrop
    assert!(
        modal.get_z_index() > backdrop.get_z_index(),
        "Modal should have higher z-index than backdrop"
    );
}

#[test]
fn test_popover_z_index_scenario() {
    // Test realistic popover scenario
    let content = apply_utility_classes("relative z-10", StyleBuilder::new());
    let popover = apply_utility_classes("absolute z-20", StyleBuilder::new());
    let tooltip = apply_utility_classes("absolute z-30", StyleBuilder::new());

    assert_eq!(content.get_z_index(), Some(10), "Content should be z-10");
    assert_eq!(popover.get_z_index(), Some(20), "Popover should be z-20");
    assert_eq!(tooltip.get_z_index(), Some(30), "Tooltip should be z-30");

    // Verify layering order
    assert!(
        popover.get_z_index() > content.get_z_index(),
        "Popover should be above content"
    );
    assert!(
        tooltip.get_z_index() > popover.get_z_index(),
        "Tooltip should be above popover"
    );
}

#[test]
fn test_dropdown_z_index_scenario() {
    // Test realistic dropdown scenario
    let button = apply_utility_classes("relative z-0", StyleBuilder::new());
    let dropdown = apply_utility_classes("absolute z-10", StyleBuilder::new());
    let modal = apply_utility_classes("fixed z-50", StyleBuilder::new());

    assert_eq!(button.get_z_index(), Some(0), "Button should be z-0");
    assert_eq!(dropdown.get_z_index(), Some(10), "Dropdown should be z-10");
    assert_eq!(modal.get_z_index(), Some(50), "Modal should be z-50");

    // Verify proper stacking
    assert!(
        dropdown.get_z_index() > button.get_z_index(),
        "Dropdown should be above button"
    );
    assert!(
        modal.get_z_index() > dropdown.get_z_index(),
        "Modal should be above dropdown"
    );
}

#[test]
fn test_z_index_with_other_utilities() {
    // Test z-index combined with other utilities
    let sb = apply_utility_classes(
        "fixed z-50 top-0 left-0 w-full h-full bg-black opacity-75 flex items-center justify-center", 
        StyleBuilder::new()
    );

    assert_eq!(
        sb.get_z_index(),
        Some(50),
        "Should preserve z-index with other utilities"
    );
}

#[test]
fn test_invalid_z_index() {
    // Test that invalid z-index values don't crash
    let sb = apply_utility_classes("z-invalid", StyleBuilder::new());
    assert_eq!(sb.get_z_index(), None, "Invalid z-index should not be set");

    let sb2 = apply_utility_classes("z-", StyleBuilder::new());
    assert_eq!(sb2.get_z_index(), None, "Empty z-index should not be set");
}

#[test]
fn test_z_index_order_independence() {
    // Test that z-index works regardless of class order
    let sb1 = apply_utility_classes("z-20 absolute", StyleBuilder::new());
    let sb2 = apply_utility_classes("absolute z-20", StyleBuilder::new());

    // Both should end up with the same final z-index (last one wins)
    assert_eq!(
        sb1.get_z_index(),
        Some(10),
        "Position should override earlier z-index"
    );
    assert_eq!(
        sb2.get_z_index(),
        Some(20),
        "Explicit z-index should override position"
    );
}

#[test]
fn numeric_side_offsets_preserve_position_and_layer() {
    for position in ["static", "relative", "absolute", "fixed", "sticky"] {
        let expected_position = apply_utility_classes(position, StyleBuilder::new())
            .build()
            .position;
        for side in ["top", "right", "bottom", "left"] {
            for value in [0.0_f32, 2.0, -2.0, 0.5] {
                let classes = format!("{position} z-37 {side}-{value}");
                let sb = apply_utility_classes(&classes, StyleBuilder::new());
                assert_eq!(sb.get_z_index(), Some(37), "{classes}");
                let style = sb.build();
                assert_eq!(style.position, expected_position, "{classes}");
                let inset = match side {
                    "top" => style.inset.top,
                    "right" => style.inset.right,
                    "bottom" => style.inset.bottom,
                    _ => style.inset.left,
                };
                assert_eq!(
                    inset,
                    taffy::style::LengthPercentageAuto::length(value),
                    "{classes}"
                );
            }
        }
    }
}

#[test]
fn side_offsets_do_not_implicitly_choose_a_position_or_layer() {
    for side in ["top", "right", "bottom", "left"] {
        let sb = apply_utility_classes(&format!("{side}-2"), StyleBuilder::new());
        assert_eq!(sb.get_z_index(), None);
        assert_eq!(sb.build().position, StyleBuilder::new().build().position);
    }
}
