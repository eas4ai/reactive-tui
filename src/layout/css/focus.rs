//! Focus and accessibility CSS utilities
//!
//! This module provides utilities for:
//! - Focus states (focus:*, focus-within:*)
//! - Accessibility attributes (aria-*, role-*, tabindex-*)
//! - Focus rings and outlines
//! - Screen reader utilities

use crate::layout::colors::parse_color_token;
use crate::layout::style::StyleBuilder;

/// Apply focus and accessibility utilities
pub fn apply_focus_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Handle pseudo-class variants first
    if let Some(focus_token) = token.strip_prefix("focus:") {
        return apply_focus_variant(focus_token, sb);
    }

    if let Some(focus_within_token) = token.strip_prefix("focus-within:") {
        return apply_focus_within_variant(focus_within_token, sb);
    }

    // Handle direct focus utilities
    match token {
        // Focus ring utilities
        "focus-ring" | "ring" => Some(apply_focus_ring(sb, None, None)),
        "focus-ring-0" | "ring-0" => Some(sb), // No ring
        "focus-ring-1" | "ring-1" => Some(apply_focus_ring(sb, Some(1), None)),
        "focus-ring-2" | "ring-2" => Some(apply_focus_ring(sb, Some(2), None)),
        "focus-ring-4" | "ring-4" => Some(apply_focus_ring(sb, Some(4), None)),
        "focus-ring-8" | "ring-8" => Some(apply_focus_ring(sb, Some(8), None)),

        // Focus outline utilities
        "focus-outline-none" | "outline-none" => Some(sb), // Remove outline
        "focus-outline" | "outline" => Some(apply_focus_outline(sb, None)),
        "focus-outline-dashed" | "outline-dashed" => Some(apply_focus_outline(sb, Some("dashed"))),
        "focus-outline-dotted" | "outline-dotted" => Some(apply_focus_outline(sb, Some("dotted"))),

        // Accessibility utilities
        "sr-only" => Some(apply_screen_reader_only(sb)),
        "not-sr-only" => Some(apply_not_screen_reader_only(sb)),

        _ => {
            // Try parsing dynamic focus ring colors
            if let Some(color_part) = token.strip_prefix("focus-ring-") {
                if let Some((r, g, b, a)) = parse_color_token(color_part) {
                    return Some(apply_focus_ring_color(sb, r, g, b, a));
                }
            }

            if let Some(color_part) = token.strip_prefix("ring-") {
                if let Some((r, g, b, a)) = parse_color_token(color_part) {
                    return Some(apply_focus_ring_color(sb, r, g, b, a));
                }
            }

            // Try parsing tabindex utilities
            if let Some(index_str) = token.strip_prefix("tabindex-") {
                if let Ok(index) = index_str.parse::<i32>() {
                    return Some(apply_tabindex(sb, index));
                }
            }

            None
        }
    }
}

/// Apply focus variant utilities (focus:*)
fn apply_focus_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // For now, we'll apply the styles directly since we don't have conditional styling yet
    // In a full implementation, these would be stored and applied when focus state changes

    match token {
        // Focus ring variants
        "ring" | "ring-2" => Some(apply_focus_ring(sb, Some(2), Some((59, 130, 246, 0.5)))), // blue-500 with opacity
        "ring-0" => Some(sb), // No ring
        "ring-1" => Some(apply_focus_ring(sb, Some(1), Some((59, 130, 246, 0.5)))),
        "ring-4" => Some(apply_focus_ring(sb, Some(4), Some((59, 130, 246, 0.5)))),

        // Focus outline variants
        "outline-none" => Some(sb), // Remove outline
        "outline" => Some(apply_focus_outline(sb, None)),

        _ => {
            // Try to apply the base utility with focus context
            // This allows focus:bg-blue-500, focus:text-white, etc.
            apply_base_utility_with_focus(token, sb)
        }
    }
}

/// Apply focus-within variant utilities (focus-within:*)
fn apply_focus_within_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Similar to focus variants, but for parent elements when child has focus
    apply_base_utility_with_focus_within(token, sb)
}

/// Apply base utility with focus context
fn apply_base_utility_with_focus(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // For now, apply the utility directly
    // In a full implementation, this would be conditional on focus state

    // Try each CSS module to handle the base utility
    if let Some(result) = super::colors::apply_color_utilities(token, sb.clone()) {
        return Some(result);
    }

    if let Some(result) = super::effects::apply_effects_utilities(token, sb.clone()) {
        return Some(result);
    }

    if let Some(result) = super::spacing::apply_spacing_utilities(token, sb.clone()) {
        return Some(result);
    }

    None
}

/// Apply base utility with focus-within context
fn apply_base_utility_with_focus_within(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Similar to focus variants
    apply_base_utility_with_focus(token, sb)
}

/// Apply focus ring styling
fn apply_focus_ring(
    sb: StyleBuilder,
    width: Option<u8>,
    color: Option<(u8, u8, u8, f32)>,
) -> StyleBuilder {
    let _ring_width = width.unwrap_or(2); // For future use
    let (r, g, b, a) = color.unwrap_or((59, 130, 246, 0.5)); // Default blue-500

    // In TUI, we simulate focus rings with background color or border effects
    // This is a simplified implementation - a full version might use box drawing characters
    sb.bg_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

/// Apply focus ring color
fn apply_focus_ring_color(sb: StyleBuilder, r: f32, g: f32, b: f32, a: f32) -> StyleBuilder {
    sb.bg_rgba(r, g, b, a * 0.5)
}

/// Apply focus outline styling
fn apply_focus_outline(sb: StyleBuilder, style: Option<&str>) -> StyleBuilder {
    // In TUI, outlines can be simulated with underline or reverse video
    match style {
        Some("dashed") => sb.underline(true), // Dashed outline as underline
        Some("dotted") => sb.underline(true), // Dotted outline as underline
        _ => sb.reverse(true),                // Default outline as reverse video
    }
}

/// Apply screen reader only styling
fn apply_screen_reader_only(sb: StyleBuilder) -> StyleBuilder {
    // In TUI, sr-only content is completely hidden
    sb.opacity(0.0)
}

/// Apply not screen reader only styling
fn apply_not_screen_reader_only(sb: StyleBuilder) -> StyleBuilder {
    // Make content visible
    sb.opacity(1.0)
}

/// Apply tabindex utility
fn apply_tabindex(sb: StyleBuilder, _index: i32) -> StyleBuilder {
    // Tabindex is metadata that doesn't affect visual styling
    // In a full implementation, this would be stored as element metadata
    sb
}

/// Apply accessibility role utility
pub fn apply_role(sb: StyleBuilder, _role: &str) -> StyleBuilder {
    // ARIA roles are metadata that don't affect visual styling
    // In a full implementation, this would be stored as element metadata
    sb
}

/// Apply ARIA attribute utility
pub fn apply_aria_attribute(sb: StyleBuilder, _attribute: &str, _value: &str) -> StyleBuilder {
    // ARIA attributes are metadata that don't affect visual styling
    // In a full implementation, this would be stored as element metadata
    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_ring_utilities() {
        let sb = StyleBuilder::new();

        // Test basic focus ring
        let result = apply_focus_utilities("ring-2", sb.clone());
        assert!(result.is_some());

        // Test focus ring with color
        let result = apply_focus_utilities("ring-blue-500", sb.clone());
        assert!(result.is_some());

        // Test focus variant
        let result = apply_focus_utilities("focus:ring-2", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_accessibility_utilities() {
        let sb = StyleBuilder::new();

        // Test screen reader utilities
        let result = apply_focus_utilities("sr-only", sb.clone());
        assert!(result.is_some());

        let result = apply_focus_utilities("not-sr-only", sb.clone());
        assert!(result.is_some());

        // Test tabindex
        let result = apply_focus_utilities("tabindex-0", sb.clone());
        assert!(result.is_some());

        let result = apply_focus_utilities("tabindex--1", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_focus_variants() {
        let sb = StyleBuilder::new();

        // Test focus color variants
        let result = apply_focus_utilities("focus:bg-blue-500", sb.clone());
        assert!(result.is_some());

        let result = apply_focus_utilities("focus:text-white", sb.clone());
        assert!(result.is_some());

        // Test focus-within variants
        let result = apply_focus_utilities("focus-within:bg-gray-400", sb.clone());
        assert!(result.is_some());
    }
}
