//! Interaction utilities for TUI
//!
//! This module provides basic interaction utilities using existing StyleBuilder methods.
//! All utilities are adapted for terminal constraints and use visual styling to indicate states.

use crate::layout::style::StyleBuilder;

/// Apply interaction utilities
pub fn apply_interaction_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // User selection (represented through styling)
        "select-none" => Some(sb.opacity(0.7)),
        "select-text" => Some(sb.italic(true)),
        "select-all" => Some(sb.bold(true)),
        "select-auto" => Some(sb),

        // Resize (represented through styling)
        "resize-none" => Some(sb),
        "resize" => Some(sb.underline(true)),
        "resize-y" => Some(sb.italic(true)),
        "resize-x" => Some(sb.bold(true)),

        // Pointer events (represented through opacity)
        "pointer-events-none" => Some(sb.opacity(0.3)),
        "pointer-events-auto" => Some(sb.opacity(1.0)),

        // Scroll behavior (visual hints)
        "scroll-smooth" => Some(sb),
        "scroll-auto" => Some(sb),
        "scroll-instant" => Some(sb),

        // Snap alignment (visual hints)
        "snap-start" => Some(sb),
        "snap-center" => Some(sb),
        "snap-end" => Some(sb),
        "snap-none" => Some(sb),

        // Basic cursor styles using existing methods
        "cursor-pointer" => Some(sb.underline(true)),
        "cursor-not-allowed" => Some(sb.opacity(0.5).strike(true)),
        "cursor-text" => Some(sb.italic(true)),
        "cursor-default" => Some(sb),
        "cursor-wait" => Some(sb.opacity(0.7)),
        "cursor-help" => Some(sb.underline(true)),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_select() {
        let sb = StyleBuilder::new();

        let result = apply_interaction_utilities("select-none", sb.clone())
            .expect("CSS interactions test should succeed");
        assert_eq!(result.opacity, Some(0.7), "select-none dims the text");

        let result = apply_interaction_utilities("select-all", sb.clone())
            .expect("CSS interactions test should succeed");
        assert_eq!(
            result.text.bold,
            Some(true),
            "select-all emboldens the text"
        );
    }

    #[test]
    fn test_cursor() {
        let sb = StyleBuilder::new();

        let result = apply_interaction_utilities("cursor-pointer", sb.clone())
            .expect("CSS interactions test should succeed");
        assert_eq!(
            result.text.underline,
            Some(true),
            "cursor-pointer underlines"
        );

        let result = apply_interaction_utilities("cursor-not-allowed", sb.clone())
            .expect("CSS interactions test should succeed");
        assert_eq!(result.opacity, Some(0.5), "cursor-not-allowed dims");
        assert!(result.get_strike(), "cursor-not-allowed strikes through");
    }

    #[test]
    fn test_pointer_events() {
        let sb = StyleBuilder::new();

        let result = apply_interaction_utilities("pointer-events-none", sb.clone())
            .expect("CSS interactions test should succeed");
        assert_eq!(result.opacity, Some(0.3), "pointer-events-none fades");

        let result = apply_interaction_utilities("pointer-events-auto", sb.clone())
            .expect("CSS interactions test should succeed");
        assert_eq!(result.opacity, Some(1.0), "pointer-events-auto is opaque");
    }
}
