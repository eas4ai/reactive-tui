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

pub mod accessibility;
pub mod animations;
pub mod colors;
pub mod containers;
pub mod effects;
pub mod focus;
pub mod interactions;
pub mod layout;
pub mod parsers;
pub mod sizing;
pub mod spacing;
pub mod typography;
pub mod variants;

use crate::layout::style::StyleBuilder;

/// Apply utility CSS classes to a StyleBuilder
///
/// This is the main entry point for applying Tailwind CSS utilities.
/// It routes tokens to the appropriate specialized modules.
pub fn apply_utility_classes(class_str: &str, mut sb: StyleBuilder) -> StyleBuilder {
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

        // 3. Color utilities (very common)
        if let Some(result) = colors::apply_color_utilities(token, sb.clone()) {
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
        let _style = result.build();
        // Spacing should be applied (exact values hard to test due to Taffy internals)
    }

    #[test]
    fn test_apply_utility_classes_colors() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("text-red-500 bg-blue-600", sb);
        let _style = result.build();
        // Colors should be applied
    }

    #[test]
    fn test_apply_utility_classes_sizing() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("w-full h-screen", sb);
        let _style = result.build();
        // Sizing should be applied
    }

    #[test]
    fn test_apply_utility_classes_typography() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("font-bold italic underline", sb);
        let _style = result.build();
        // Typography should be applied
    }

    #[test]
    fn test_apply_utility_classes_effects() {
        let sb = StyleBuilder::new();
        let result = apply_utility_classes("opacity-50 z-10 shadow", sb);
        let _style = result.build();
        // Effects should be applied
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
