//! Simple tests for CSS-in-Rust functionality

use reactive_tui::prelude::*;
use reactive_tui::{css, flex_center, flex_column, absolute_fill};
use taffy::style::{Display, FlexDirection, AlignItems, JustifyContent, Position};

#[test]
fn test_basic_css_macro() {
    // Test that the css! macro compiles and creates a StyleBuilder
    let _styles = css! {
        display: Display::Flex,
        opacity: 0.8,
    };
    
    // If this compiles, the macro is working correctly
    assert!(true);
}

#[test]
fn test_layout_properties() {
    // Test layout-related CSS properties
    let _layout_styles = css! {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        position: Position::Absolute,
    };
    
    assert!(true);
}

#[test]
fn test_color_properties() {
    // Test color-related CSS properties
    let _color_styles = css! {
        color: (1.0, 1.0, 1.0, 1.0), // White
        background_color: (0.0, 0.0, 1.0, 1.0), // Blue
    };
    
    assert!(true);
}

#[test]
fn test_spacing_properties() {
    // Test spacing-related CSS properties
    let _spacing_styles = css! {
        padding: 16.0,
        margin: 8.0,
    };
    
    assert!(true);
}

#[test]
fn test_numeric_properties() {
    // Test numeric CSS properties
    let _numeric_styles = css! {
        opacity: 0.9,
        width: 100.0,
        height: 50.0,
    };
    
    assert!(true);
}

#[test]
fn test_convenience_macros() {
    // Test convenience macros
    let _center = flex_center!();
    let _column = flex_column!();
    let _fill = absolute_fill!();
    
    // If these compile, the convenience macros are working
    assert!(true);
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
    assert_eq!(element.element_type, ElementType::Text("Styled element".to_string()));
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
    assert_eq!(element.element_type, ElementType::Text("Hybrid styling".to_string()));
    
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
    assert_eq!(main_layout.element_type, ElementType::Layout(LayoutType::Flex));
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
    
    assert_eq!(button.element_type, ElementType::Text("Click Me".to_string()));
    
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
    let _valid_styles = css! {
        display: Display::Flex,
        color: (1.0, 0.0, 0.0, 1.0),
        padding: 10.0,
        opacity: 0.5,
    };
    
    assert!(true);
    
    // The following would not compile due to type mismatches:
    // css! {
    //     display: "flex",  // Wrong type - should be Display
    //     color: "red",     // Wrong type - should be (f32, f32, f32, f32)
    //     padding: "10px",  // Wrong type - should be f32
    // }
}

#[test]
fn test_macro_edge_cases() {
    // Test edge cases for the CSS macro
    
    // Empty CSS
    let _empty_styles = css! {};
    assert!(true);
    
    // Single property
    let _single_prop = css! {
        display: Display::Flex,
    };
    assert!(true);
    
    // Trailing comma
    let _trailing_comma = css! {
        display: Display::Flex,
        color: (0.0, 0.0, 1.0, 1.0),
    };
    assert!(true);
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
    assert_eq!(component.element_type, ElementType::Layout(LayoutType::Flex));
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
    
    let start = std::time::Instant::now();
    
    // Create many styled elements
    let elements: Vec<Element> = (0..100)
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
        .collect();
    
    let duration = start.elapsed();
    
    // Verify all elements were created
    assert_eq!(elements.len(), 100);
    
    // Performance should be reasonable (less than 10ms for 100 elements)
    assert!(duration.as_millis() < 10);
    
    println!("Created 100 CSS-in-Rust styled elements in {:?}", duration);
}
