//! Simple tests for CSS-in-Rust functionality

use reactive_tui::prelude::*;
use reactive_tui::{absolute_fill, css, flex_center, flex_column};
use taffy::style::{AlignItems, Display, FlexDirection, JustifyContent, Position};

#[test]
fn test_basic_css_macro() {
    // Test that the css! macro compiles and creates a StyleBuilder
    let mut styles = css! {
        display: Display::Flex,
        opacity: 0.8,
    };

    // Check the resulting layout and visual properties.
    let visuals = styles.take_visuals().expect("opacity");
    assert_eq!(visuals.fg.a, 0.8);
    assert_eq!(styles.build().display, Display::Flex);
}

#[test]
fn test_layout_properties() {
    // Test layout-related CSS properties
    let layout_styles = css! {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        position: Position::Absolute,
    };

    let style = layout_styles.build();
    assert_eq!(style.display, Display::Flex);
    assert_eq!(style.flex_direction, FlexDirection::Column);
    assert_eq!(style.align_items, Some(AlignItems::Center));
    assert_eq!(style.justify_content, Some(JustifyContent::SpaceBetween));
    assert_eq!(style.position, Position::Absolute);
}

#[test]
fn test_color_properties() {
    // Test color-related CSS properties
    let mut color_styles = css! {
        color: (1.0, 1.0, 1.0, 1.0), // White
        background_color: (0.0, 0.0, 1.0, 1.0), // Blue
    };

    let visuals = color_styles.take_visuals().expect("colors");
    assert_eq!(visuals.fg, reactive_tui::core::surface::Rgba::white());
    assert_eq!(
        visuals.bg,
        reactive_tui::core::surface::Rgba::new(0.0, 0.0, 1.0, 1.0)
    );
}

#[test]
fn test_spacing_properties() {
    // Test spacing-related CSS properties
    let spacing_styles = css! {
        padding: 16.0,
        margin: 8.0,
    };

    let style = spacing_styles.build();
    for padding in [
        style.padding.left,
        style.padding.right,
        style.padding.top,
        style.padding.bottom,
    ] {
        assert_eq!(padding, taffy::style::LengthPercentage::length(16.0));
    }
    for margin in [
        style.margin.left,
        style.margin.right,
        style.margin.top,
        style.margin.bottom,
    ] {
        assert_eq!(margin, taffy::style::LengthPercentageAuto::length(8.0));
    }
}

#[test]
fn test_numeric_properties() {
    // Test numeric CSS properties
    let mut numeric_styles = css! {
        opacity: 0.9,
        width: 100.0,
        height: 50.0,
    };

    assert_eq!(numeric_styles.take_visuals().expect("opacity").fg.a, 0.9);
    let style = numeric_styles.build();
    assert_eq!(style.size.width, taffy::style::Dimension::length(100.0));
    assert_eq!(style.size.height, taffy::style::Dimension::length(50.0));
}

#[test]
fn test_convenience_macros() {
    // Test convenience macros
    let center = flex_center!();
    let column = flex_column!();
    let fill = absolute_fill!();

    // Verify the styles produced by each convenience macro.
    let center = center.build();
    assert_eq!(center.align_items, Some(AlignItems::Center));
    assert_eq!(center.justify_content, Some(JustifyContent::Center));
    assert_eq!(column.build().flex_direction, FlexDirection::Column);
    let fill = fill.build();
    assert_eq!(fill.position, Position::Absolute);
    assert_eq!(fill.size.width, taffy::style::Dimension::percent(1.0));
    assert_eq!(fill.size.height, taffy::style::Dimension::percent(1.0));
}

#[test]
fn test_builder_integration() {
    // Test CSS-in-Rust integration with builders
    let element = div()
        .text("Styled element")
        .styles(css! {
            display: Display::Flex,
            background_color: (0.0, 0.0, 1.0, 1.0),
            padding: 16.0,
        })
        .build();

    // Verify the element was created (text elements become Text type)
    assert_eq!(
        element.element_type,
        ElementType::Text("Styled element".to_string())
    );
}

#[test]
fn test_hybrid_styling() {
    // Test combining CSS classes with CSS-in-Rust
    let element = div()
        .class("hover:shadow-lg transition-all")
        .styles(css! {
            background_color: (0.0, 1.0, 0.0, 1.0),
            padding: 12.0,
        })
        .text("Hybrid styling")
        .build();

    // Verify the element was created (text elements become Text type)
    assert_eq!(
        element.element_type,
        ElementType::Text("Hybrid styling".to_string())
    );

    // Verify CSS classes were applied
    if let Some(class) = &element.class {
        assert!(class.contains("hover:shadow-lg"));
        assert!(class.contains("transition-all"));
        assert!(class.contains("css-in-rust-applied")); // Our placeholder
    }
}

#[test]
fn test_complex_layout() {
    // Test creating a complex layout with CSS-in-Rust
    let navbar = div()
        .styles(css! {
            display: Display::Flex,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            background_color: (0.12, 0.16, 0.22, 1.0), // Dark gray
            color: (1.0, 1.0, 1.0, 1.0), // White
            padding: 16.0,
            height: 64.0,
        })
        .children(vec![
            div().text("Logo").build(),
            div().text("Navigation").build(),
            div().text("User Menu").build(),
        ])
        .build();

    // Verify navbar structure
    assert_eq!(navbar.element_type, ElementType::Layout(LayoutType::Flex));
    assert_eq!(navbar.children.len(), 3);

    let main_layout = div()
        .styles(css! {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: 100.0,
            height: 100.0,
        })
        .children(vec![
            navbar,
            div()
                .styles(css! {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                })
                .children(vec![
                    div().text("Sidebar").build(),
                    div().text("Main Content").build(),
                ])
                .build(),
        ])
        .build();

    // Verify main layout structure
    assert_eq!(
        main_layout.element_type,
        ElementType::Layout(LayoutType::Flex)
    );
    assert_eq!(main_layout.children.len(), 2);

    // Verify nested structure
    let content_area = &main_layout.children[1];
    assert_eq!(content_area.children.len(), 2);
}

#[test]
fn test_component_patterns() {
    // Test common component styling patterns

    // Button component
    let button = div()
        .text("Click Me")
        .styles(css! {
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            background_color: (0.0, 0.0, 1.0, 1.0),
            color: (1.0, 1.0, 1.0, 1.0),
            padding: 8.0,
        })
        .build();

    assert_eq!(
        button.element_type,
        ElementType::Text("Click Me".to_string())
    );

    // Card component
    let card = div()
        .class("shadow-md border")
        .styles(css! {
            background_color: (1.0, 1.0, 1.0, 1.0),
            padding: 20.0,
        })
        .children(vec![
            div().text("Card Title").build(),
            div().text("Card content").build(),
        ])
        .build();

    assert_eq!(card.element_type, ElementType::Layout(LayoutType::Flex));
    assert_eq!(card.children.len(), 2);
}

#[test]
fn test_type_safety() {
    // Test that the CSS-in-Rust system is type-safe

    // This should compile - correct types
    let mut valid_styles = css! {
        display: Display::Flex,
        color: (1.0, 0.0, 0.0, 1.0),
        padding: 10.0,
        opacity: 0.5,
    };

    assert_eq!(
        valid_styles.take_visuals().expect("color and opacity").fg,
        reactive_tui::core::surface::Rgba::new(1.0, 0.0, 0.0, 0.5)
    );
    let style = valid_styles.build();
    assert_eq!(style.display, Display::Flex);
    assert_eq!(
        style.padding.left,
        taffy::style::LengthPercentage::length(10.0)
    );
}

#[test]
fn test_macro_edge_cases() {
    // Test edge cases for the CSS macro

    // Empty CSS
    let empty_styles = css! {};
    assert_eq!(
        empty_styles.build(),
        reactive_tui::layout::style::StyleBuilder::new().build()
    );

    // Single property
    let single_prop = css! {
        display: Display::Flex,
    };
    assert_eq!(single_prop.build().display, Display::Flex);

    // Trailing comma
    let mut trailing_comma = css! {
        display: Display::Flex,
        color: (0.0, 0.0, 1.0, 1.0),
    };
    assert_eq!(
        trailing_comma.take_visuals().expect("color").fg,
        reactive_tui::core::surface::Rgba::new(0.0, 0.0, 1.0, 1.0)
    );
    assert_eq!(trailing_comma.build().display, Display::Flex);
}

#[test]
fn test_integration_with_existing_system() {
    // Test that CSS-in-Rust works with existing reactive-tui features

    let component = div()
        .class("existing-utility-classes")
        .styles(css! {
            background_color: (0.0, 0.0, 1.0, 1.0),
            padding: 16.0,
        })
        .children(vec![
            span().text("Child 1").build(),
            span().text("Child 2").build(),
        ])
        .build();

    // Verify integration
    assert_eq!(
        component.element_type,
        ElementType::Layout(LayoutType::Flex)
    );
    assert_eq!(component.children.len(), 2);

    // Verify CSS classes include both utility classes and CSS-in-Rust
    if let Some(class) = &component.class {
        assert!(class.contains("existing-utility-classes"));
        assert!(class.contains("css-in-rust-applied"));
    }
}

#[test]
fn test_performance_considerations() {
    // Test that CSS-in-Rust doesn't significantly impact performance

    // Create many styled elements; the best of three runs is measured so a
    // loaded machine (the full workspace suite in parallel) cannot fail it.
    let build = || -> Vec<Element> {
        (0..100)
            .map(|i| {
                div()
                    .text(&format!("Element {}", i))
                    .styles(css! {
                        display: Display::Flex,
                        background_color: (0.0, 0.0, 1.0, 1.0),
                        padding: 8.0,
                    })
                    .build()
            })
            .collect()
    };
    let mut duration = std::time::Duration::MAX;
    let mut elements = Vec::new();
    for _ in 0..3 {
        let start = std::time::Instant::now();
        elements = build();
        duration = duration.min(start.elapsed());
    }

    // Verify all elements were created
    assert_eq!(elements.len(), 100);

    // Performance should be reasonable. Debug builds under a parallel test
    // run take several milliseconds for this loop, so the bound leaves room
    // for load while still catching a regression of an order of magnitude.
    assert!(
        duration.as_millis() < 50,
        "100 styled elements took {duration:?}"
    );

    println!("Created 100 CSS-in-Rust styled elements in {:?}", duration);
}
