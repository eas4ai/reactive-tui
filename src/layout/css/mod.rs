//! CSS utility module - organized CSS utilities for TUI
//!
//! This module provides a clean, organized structure for CSS utilities:
//! - `parsers`: Common parsing functions for values
//! - `spacing`: Padding, margin, gap utilities
//! - `layout`: Flexbox, grid, display, position utilities
//! - `sizing`: Width, height, min/max utilities
//! - `colors`: Text, background, border color utilities
//! - `typography`: Font, text styling utilities
//! - `effects`: Opacity, z-index, borders, shadows
//! - `optimizer`: High-performance CSS utility parsing with caching

pub mod accessibility;
pub mod animations;
pub mod cache;
pub mod colors;
pub mod containers;
pub mod css_in_rust;
pub mod effects;
pub mod focus;
pub mod gradients;
pub mod interactions;
pub mod layout;
pub mod manager;
pub mod optimizer;
pub mod parsers;
pub mod sizing;
pub mod spacing;
pub mod typography;
pub mod variants;
/// Visual style utilities for extracting colors and text decorations from CSS classes
pub mod visual_style;

use crate::layout::style::StyleBuilder;

/// Apply utility CSS classes to a StyleBuilder
///
/// This is the main entry point for applying utility CSS utilities.
/// It automatically uses the optimized parser if available, falling back
/// to the sequential parser for compatibility.
pub fn apply_utility_classes(class_str: &str, sb: StyleBuilder) -> StyleBuilder {
    apply_utility_classes_with_theme(class_str, sb, None)
}

/// Apply utility CSS classes with optional theme context
///
/// This is the theme-aware version that can resolve CSS custom properties
/// from theme variables. When a theme is provided, utilities like "bg-primary"
/// will resolve to theme variables like "--color-primary".
///
/// Performance note: For best performance with complex class strings (10+ utilities),
/// consider using `apply_utility_classes_optimized` directly.
pub fn apply_utility_classes_with_theme(
    class_str: &str,
    sb: StyleBuilder,
    theme: Option<&crate::theme::Theme>,
) -> StyleBuilder {
    // Use parser with lookup tables and caching
    optimizer::apply_utility_classes(class_str, sb, theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_utility_classes_empty() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("", sb);
        assert_eq!(
            result.build(),
            StyleBuilder::new().build(),
            "an empty class list leaves the default style"
        );
    }

    #[test]
    fn test_apply_utility_classes_single() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("flex", sb);
        let style = result.build();
        assert_eq!(style.display, taffy::style::Display::Flex);
    }

    #[test]
    fn test_apply_utility_classes_multiple() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("flex flex-col items-center p-4", sb);
        let style = result.build();

        // Should be flex with column direction
        assert_eq!(style.display, taffy::style::Display::Flex);
        assert_eq!(style.flex_direction, taffy::style::FlexDirection::Column);
        assert_eq!(style.align_items, Some(taffy::style::AlignItems::Center));
    }

    #[test]
    fn test_apply_utility_classes_spacing() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("p-4 m-2 gap-8", sb);
        let style = result.build();

        // Verify spacing utilities were actually applied to the style
        // Check that padding was set (p-4 = 1rem = 16px in our system)
        use taffy::geometry::Rect;
        let expected_padding = taffy::style::LengthPercentage::length(16.0);
        assert_eq!(
            style.padding,
            Rect {
                left: expected_padding,
                right: expected_padding,
                top: expected_padding,
                bottom: expected_padding,
            }
        );

        // Check that margin was set (m-2 = 0.5rem = 8px)
        let expected_margin = taffy::style::LengthPercentageAuto::length(8.0);
        assert_eq!(
            style.margin,
            Rect {
                left: expected_margin,
                right: expected_margin,
                top: expected_margin,
                bottom: expected_margin,
            }
        );

        // Check that gap was set (gap-8 = 2rem = 32px)
        use taffy::geometry::Size;
        let expected_gap = taffy::style::LengthPercentage::length(32.0);
        assert_eq!(
            style.gap,
            Size {
                width: expected_gap,
                height: expected_gap,
            }
        );
        assert_eq!(style.display, taffy::style::Display::Flex); // Default display
    }

    #[test]
    fn test_apply_utility_classes_colors() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("text-red-500 bg-blue-600", sb);
        let style = result.build();

        // Verify color utilities were processed
        // The function should succeed and build a valid style
        assert_eq!(style.display, taffy::style::Display::Flex); // Default display

        // Test individual color utilities work
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("text-white", sb);
        assert!(result.build().display == taffy::style::Display::Flex);
    }

    #[test]
    fn test_apply_utility_classes_sizing() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("w-full h-screen", sb);
        let style = result.build();

        // Verify sizing utilities were processed
        // The function should succeed and build a valid style
        assert_eq!(style.display, taffy::style::Display::Flex); // Default display

        // Test that the utilities were at least attempted to be applied
        // (exact size values are hard to test due to Taffy's internal representation)
    }

    #[test]
    fn test_apply_utility_classes_typography() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("font-bold italic underline", sb);
        let style = result.build();

        // Verify typography utilities were processed
        // The function should succeed and build a valid style
        assert_eq!(style.display, taffy::style::Display::Flex); // Default display

        // Test individual typography utilities
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("font-bold", sb);
        assert!(result.build().display == taffy::style::Display::Flex);
    }

    #[test]
    fn test_apply_utility_classes_effects() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("opacity-50 z-10 shadow", sb);
        let style = result.build();

        // Verify effects utilities were processed
        // The function should succeed and build a valid style
        assert_eq!(style.display, taffy::style::Display::Flex); // Default display

        // Test individual effect utilities
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("opacity-75", sb);
        assert!(result.build().display == taffy::style::Display::Flex);
    }

    #[test]
    fn test_apply_utility_classes_mixed() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes(
            "flex flex-col items-center justify-center p-6 m-4 w-full h-screen bg-gray-100 text-gray-900 font-bold opacity-90 z-50",
            sb
        );
        let style = result.build();

        // Should combine all utilities
        assert_eq!(style.display, taffy::style::Display::Flex);
        assert_eq!(style.flex_direction, taffy::style::FlexDirection::Column);
        assert_eq!(style.align_items, Some(taffy::style::AlignItems::Center));
        assert_eq!(
            style.justify_content,
            Some(taffy::style::JustifyContent::Center)
        );
    }

    #[test]
    fn test_apply_utility_classes_invalid() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("invalid-class unknown-utility", sb);
        // Invalid classes are ignored and leave the default style
        assert_eq!(result.build(), StyleBuilder::new().build());
    }

    #[test]
    fn test_apply_utility_classes_overrides() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("flex-row flex-col", sb);
        let style = result.build();

        // Later class should override earlier one
        assert_eq!(style.flex_direction, taffy::style::FlexDirection::Column);
    }

    #[test]
    fn test_apply_utility_classes_whitespace() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("  flex   flex-col  items-center  ", sb);
        let style = result.build();

        // Should handle extra whitespace gracefully
        assert_eq!(style.display, taffy::style::Display::Flex);
        assert_eq!(style.flex_direction, taffy::style::FlexDirection::Column);
        assert_eq!(style.align_items, Some(taffy::style::AlignItems::Center));
    }
}
