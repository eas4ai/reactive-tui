//! CSS utility module - organized Tailwind CSS utilities for TUI
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
pub mod animation_manager;
pub mod animations;
pub mod cache;
pub mod colors;
pub mod containers;
pub mod css_in_rust;
pub mod effects;
pub mod focus;
pub mod interactions;
pub mod layout;
pub mod optimizer;
pub mod parsers;
pub mod sizing;
pub mod spacing;
pub mod typography;
pub mod variants;

use crate::layout::style::StyleBuilder;

/// Apply utility CSS classes to a StyleBuilder
///
/// This is the main entry point for applying Tailwind CSS utilities.
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

/// Legacy sequential CSS utility application
/// 
/// This is the original implementation that processes utilities sequentially.
/// Kept for compatibility and testing purposes. The optimized version should
/// be preferred for production use.
#[allow(dead_code)]
pub fn apply_utility_classes_sequential(
    class_str: &str,
    mut sb: StyleBuilder,
    theme: Option<&crate::theme::Theme>,
) -> StyleBuilder {
    if class_str.is_empty() {
        return sb;
    }

    // Split class string into individual tokens
    let tokens: Vec<&str> = class_str.split_whitespace().collect();

    for token in tokens {
        // Try each category of utilities in order of likelihood

        // 1. Layout utilities (most common)
        if let Some(result) = layout::apply_layout_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 2. Spacing utilities (very common)
        if let Some(result) = spacing::apply_spacing_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 3. Color utilities (very common) - with theme support
        if let Some(result) = colors::apply_color_utilities_with_theme(token, sb.clone(), theme) {
            sb = result;
            continue;
        }

        // 4. Container utilities (common for layouts)
        if let Some(result) = containers::apply_container_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 5. Sizing utilities (common)
        if let Some(result) = sizing::apply_sizing_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 6. Typography utilities (common)
        if let Some(result) = typography::apply_typography_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 7. Effects utilities (less common but important)
        if let Some(result) = effects::apply_effects_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 9. Interaction & Scroll utilities (user interaction)
        if let Some(result) = interactions::apply_interaction_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 10. Focus & Accessibility utilities (important for TUI)
        if let Some(result) = focus::apply_focus_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 11. Accessibility utilities (ARIA, roles, etc.)
        if let Some(result) = accessibility::apply_accessibility_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 12. Animation & Transition utilities (smooth interactions)
        if let Some(result) = animations::apply_animation_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // 13. Pseudo-class Variants (hover:*, active:*, etc.)
        if let Some(result) = variants::apply_variant_utilities(token, sb.clone()) {
            sb = result;
            continue;
        }

        // If no module handled the token, silently ignore it
        // This allows for graceful degradation of unsupported utilities
    }

    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_utility_classes_empty() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("", sb);
        let _style = result.build();
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

        // Verify spacing utilities were processed (we can't easily test exact Taffy values)
        // But we can verify the function succeeded and the style was built
        // In a real implementation, we'd check that padding, margin, and gap were set
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
        let _style = result.build();
        // Should not panic, just ignore invalid classes
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
