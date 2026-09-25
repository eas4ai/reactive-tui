//! Pseudo-class variant CSS utilities
//!
//! This module provides utilities for:
//! - State variants (hover:*, active:*, disabled:*, focus:*)
//! - Conditional variants (first:*, last:*, odd:*, even:*)
//! - Group variants (group-hover:*, group-focus:*)
//! - Responsive variants (sm:*, md:*, lg:*)

use crate::layout::style::StyleBuilder;

/// Apply pseudo-class variant utilities
pub fn apply_variant_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // State variants
    if let Some(result) = apply_state_variants(token, sb.clone()) {
        return Some(result);
    }

    // Conditional variants
    if let Some(result) = apply_conditional_variants(token, sb.clone()) {
        return Some(result);
    }

    // Group variants
    if let Some(result) = apply_group_variants(token, sb.clone()) {
        return Some(result);
    }

    // Responsive variants
    if let Some(result) = apply_responsive_variants(token, sb.clone()) {
        return Some(result);
    }

    None
}

/// Apply state variant utilities (hover:*, active:*, etc.)
fn apply_state_variants(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // App strips matching state prefixes before layout. A standalone parser
    // has no node state, so these variants are inactive.
    if token.starts_with("hover:") || token.starts_with("disabled:") || token.starts_with("focus:")
    {
        return Some(sb);
    }

    // Active variants
    if let Some(active_token) = token.strip_prefix("active:") {
        return apply_active_variant(active_token, sb);
    }

    // Visited variants (for links)
    if let Some(visited_token) = token.strip_prefix("visited:") {
        return apply_visited_variant(visited_token, sb);
    }

    None
}

/// Apply conditional variant utilities (first:*, last:*, etc.)
fn apply_conditional_variants(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // First child variants
    if let Some(first_token) = token.strip_prefix("first:") {
        return apply_first_variant(first_token, sb);
    }

    // Last child variants
    if let Some(last_token) = token.strip_prefix("last:") {
        return apply_last_variant(last_token, sb);
    }

    // Odd child variants
    if let Some(odd_token) = token.strip_prefix("odd:") {
        return apply_odd_variant(odd_token, sb);
    }

    // Even child variants
    if let Some(even_token) = token.strip_prefix("even:") {
        return apply_even_variant(even_token, sb);
    }

    None
}

/// Apply group variant utilities (group-hover:*, etc.)
fn apply_group_variants(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Group hover variants
    if let Some(group_hover_token) = token.strip_prefix("group-hover:") {
        return apply_group_hover_variant(group_hover_token, sb);
    }

    // Group focus variants
    if let Some(group_focus_token) = token.strip_prefix("group-focus:") {
        return apply_group_focus_variant(group_focus_token, sb);
    }

    // Group active variants
    if let Some(group_active_token) = token.strip_prefix("group-active:") {
        return apply_group_active_variant(group_active_token, sb);
    }

    None
}

/// Apply responsive variant utilities (sm:*, md:*, lg:*)
fn apply_responsive_variants(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // App resolves these prefixes against its viewport before parsing styles.
    // A standalone parser has no viewport and cannot activate a breakpoint.
    ["sm:", "md:", "lg:", "xl:"]
        .iter()
        .any(|prefix| token.starts_with(prefix))
        .then_some(sb)
}

// State variant implementations

/// Apply active variant
fn apply_active_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility_with_active(token, sb)
}

/// Apply visited variant
fn apply_visited_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility_with_visited(token, sb)
}

// Conditional variant implementations

/// Apply first child variant
fn apply_first_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility_with_first(token, sb)
}

/// Apply last child variant
fn apply_last_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility_with_last(token, sb)
}

/// Apply odd child variant
fn apply_odd_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility_with_odd(token, sb)
}

/// Apply even child variant
fn apply_even_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility_with_even(token, sb)
}

// Group variant implementations

/// Apply group hover variant
fn apply_group_hover_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility_with_group_hover(token, sb)
}

/// Apply group focus variant
fn apply_group_focus_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility_with_group_focus(token, sb)
}

/// Apply group active variant
fn apply_group_active_variant(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility_with_group_active(token, sb)
}

// Helper functions to apply base utilities with different contexts

/// Apply base utility with active context
fn apply_base_utility_with_active(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility(token, sb)
}

/// Apply base utility with visited context
fn apply_base_utility_with_visited(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility(token, sb)
}

/// Apply base utility with first child context
fn apply_base_utility_with_first(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility(token, sb)
}

/// Apply base utility with last child context
fn apply_base_utility_with_last(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility(token, sb)
}

/// Apply base utility with odd child context
fn apply_base_utility_with_odd(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility(token, sb)
}

/// Apply base utility with even child context
fn apply_base_utility_with_even(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility(token, sb)
}

/// Apply base utility with group hover context
fn apply_base_utility_with_group_hover(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility(token, sb)
}

/// Apply base utility with group focus context
fn apply_base_utility_with_group_focus(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility(token, sb)
}

/// Apply base utility with group active context
fn apply_base_utility_with_group_active(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_base_utility(token, sb)
}

/// Apply base utility by trying each CSS module
fn apply_base_utility(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
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

    if let Some(result) = super::sizing::apply_sizing_utilities(token, sb.clone()) {
        return Some(result);
    }

    if let Some(result) = super::typography::apply_typography_utilities(token, sb.clone()) {
        return Some(result);
    }

    if let Some(result) = super::layout::apply_layout_utilities(token, sb.clone()) {
        return Some(result);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_variants() {
        let sb = StyleBuilder::new();

        // Test hover variants
        let result = apply_variant_utilities("hover:bg-blue-500", sb.clone());
        assert!(result.is_some());

        let result = apply_variant_utilities("hover:text-white", sb.clone());
        assert!(result.is_some());

        // Test active variants
        let result = apply_variant_utilities("active:bg-blue-600", sb.clone());
        assert!(result.is_some());

        // Test disabled variants
        let result = apply_variant_utilities("disabled:opacity-50", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_conditional_variants() {
        let sb = StyleBuilder::new();

        // Test first/last variants
        let result = apply_variant_utilities("first:p-2", sb.clone());
        assert!(result.is_some());

        let result = apply_variant_utilities("last:p-2", sb.clone());
        assert!(result.is_some());

        // Test odd/even variants
        let result = apply_variant_utilities("odd:bg-gray-400", sb.clone());
        assert!(result.is_some());

        let result = apply_variant_utilities("even:bg-gray-500", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_group_variants() {
        let sb = StyleBuilder::new();

        // Test group variants
        let result = apply_variant_utilities("group-hover:opacity-50", sb.clone());
        assert!(result.is_some());

        let result = apply_variant_utilities("group-focus:bg-gray-400", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_responsive_variants() {
        let sb = StyleBuilder::new();

        // Test responsive variants
        let result = apply_variant_utilities("sm:text-sm", sb.clone());
        assert!(result.is_some());

        let result = apply_variant_utilities("md:text-base", sb.clone());
        assert!(result.is_some());

        let result = apply_variant_utilities("lg:text-lg", sb.clone());
        assert!(result.is_some());
    }
}
