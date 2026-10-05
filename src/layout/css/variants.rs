//! Pseudo-class variant CSS utilities
//!
//! The App decides interaction, child-position, group and viewport conditions
//! before layout. A standalone parser has none of that context, so variant
//! utilities stay inactive. Visited utilities never apply (STY-001).

use crate::layout::style::StyleBuilder;

/// Recognize conditional utilities without applying them outside an App.
pub fn apply_variant_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Only the App can strip a prefix whose condition holds (STY-001).
    [
        "hover:",
        "focus:",
        "focus-within:",
        "disabled:",
        "active:",
        "visited:",
        "first:",
        "last:",
        "odd:",
        "even:",
        "group-hover:",
        "group-focus:",
        "group-active:",
        "sm:",
        "md:",
        "lg:",
        "xl:",
    ]
    .iter()
    .any(|prefix| token.starts_with(prefix))
    .then_some(sb)
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
        let result = apply_variant_utilities("first:p-8", sb.clone());
        assert!(result.is_some());

        let result = apply_variant_utilities("last:p-8", sb.clone());
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
