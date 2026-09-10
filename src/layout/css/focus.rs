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
    // App resolves variants before layout. Standalone builders have no focus.
    if token.starts_with("focus:") || token.starts_with("focus-within:") {
        return Some(sb);
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
fn apply_screen_reader_only(mut sb: StyleBuilder) -> StyleBuilder {
    sb.accessibility
        .insert("sr-only".into(), Some("true".into()));
    // In TUI, sr-only content is completely hidden
    sb.opacity(0.0)
}

/// Apply not screen reader only styling
fn apply_not_screen_reader_only(mut sb: StyleBuilder) -> StyleBuilder {
    sb.accessibility
        .insert("sr-only".into(), Some("false".into()));
    // Make content visible
    sb.opacity(1.0)
}

/// Apply tabindex utility
fn apply_tabindex(mut sb: StyleBuilder, index: i32) -> StyleBuilder {
    sb.accessibility
        .insert("tabindex".into(), Some(index.to_string()));
    sb
}

/// Apply accessibility role utility
pub fn apply_role(mut sb: StyleBuilder, role: &str) -> StyleBuilder {
    sb.accessibility
        .insert("role".into(), Some(role.to_owned()));
    sb
}

/// Apply ARIA attribute utility
///
/// Accepts attribute names with or without the `aria-` prefix. App validates
/// values and resolves `labelledby`/`describedby` against Element accessibility
/// IDs. Missing values, duplicate IDs and unresolved references are errors.
///
/// ```
/// use reactive_tui::{builder, layout::{css::focus::apply_aria_attribute, style::StyleBuilder}};
/// let styles = apply_aria_attribute(StyleBuilder::new(), "label", "Save preferences");
/// let button = builder::button().text("Save").styles(styles).build();
/// ```
pub fn apply_aria_attribute(mut sb: StyleBuilder, attribute: &str, value: &str) -> StyleBuilder {
    let attribute = attribute.strip_prefix("aria-").unwrap_or(attribute);
    sb.accessibility
        .insert(format!("aria-{attribute}"), Some(value.to_owned()));
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
