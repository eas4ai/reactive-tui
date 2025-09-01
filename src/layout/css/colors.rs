//! Color utilities: text colors, background colors

use crate::layout::colors::parse_color_token;
use crate::layout::style::StyleBuilder;

/// Apply text color utilities
pub fn apply_text_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("text-") {
        // Remove "text-" prefix
        if let Some((r, g, b, a)) = parse_color_token(color_part) {
            Some(sb.fg_rgba(r, g, b, a))
        } else {
            None
        }
    } else {
        None
    }
}

/// Apply background color utilities
pub fn apply_bg_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("bg-") {
        // Remove "bg-" prefix
        if let Some((r, g, b, a)) = parse_color_token(color_part) {
            Some(sb.bg_rgba(r, g, b, a))
        } else {
            None
        }
    } else {
        None
    }
}

/// Apply border color utilities (TUI-adapted)
pub fn apply_border_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("border-") {
        // Remove "border-" prefix
        if let Some((r, g, b, a)) = parse_color_token(color_part) {
            // In TUI, we can approximate border colors by using them as background
            // This is a compromise since true borders aren't available
            Some(sb.bg_rgba(r, g, b, a))
        } else {
            None
        }
    } else {
        None
    }
}

/// Apply ring color utilities (focus rings, TUI-adapted)
pub fn apply_ring_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("ring-") {
        // Remove "ring-" prefix
        if let Some((r, g, b, a)) = parse_color_token(color_part) {
            // Ring colors can be used for focus states
            // In TUI, we might use this for background highlighting
            Some(sb.bg_rgba(r, g, b, a * 0.3)) // Make it more subtle
        } else {
            None
        }
    } else {
        None
    }
}

/// Apply accent color utilities
pub fn apply_accent_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("accent-") {
        // Remove "accent-" prefix
        if let Some((r, g, b, a)) = parse_color_token(color_part) {
            // Accent colors for form controls, checkboxes, etc.
            Some(sb.fg_rgba(r, g, b, a))
        } else {
            None
        }
    } else {
        None
    }
}

/// Apply caret color utilities (text cursor)
pub fn apply_caret_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("caret-") {
        // Remove "caret-" prefix
        if let Some((r, g, b, a)) = parse_color_token(color_part) {
            // Caret color for text inputs
            Some(sb.fg_rgba(r, g, b, a))
        } else {
            None
        }
    } else {
        None
    }
}

/// Apply placeholder color utilities
pub fn apply_placeholder_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("placeholder-") {
        // Remove "placeholder-" prefix
        if let Some((r, g, b, a)) = parse_color_token(color_part) {
            // Placeholder text color (dimmed)
            Some(sb.fg_rgba(r, g, b, a * 0.6)) // Make it more subtle
        } else {
            None
        }
    } else {
        None
    }
}

/// Apply selection color utilities
pub fn apply_selection_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("selection-") {
        // Remove "selection-" prefix
        if let Some((r, g, b, a)) = parse_color_token(color_part) {
            // Text selection background color
            Some(sb.bg_rgba(r, g, b, a * 0.3)) // Make it more subtle
        } else {
            None
        }
    } else {
        None
    }
}

/// Apply all color utilities
pub fn apply_color_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Try text color
    if let Some(result) = apply_text_color(token, sb.clone()) {
        return Some(result);
    }

    // Try background color
    if let Some(result) = apply_bg_color(token, sb.clone()) {
        return Some(result);
    }

    // Try border color
    if let Some(result) = apply_border_color(token, sb.clone()) {
        return Some(result);
    }

    // Try ring color
    if let Some(result) = apply_ring_color(token, sb.clone()) {
        return Some(result);
    }

    // Try accent color
    if let Some(result) = apply_accent_color(token, sb.clone()) {
        return Some(result);
    }

    // Try caret color
    if let Some(result) = apply_caret_color(token, sb.clone()) {
        return Some(result);
    }

    // Try placeholder color
    if let Some(result) = apply_placeholder_color(token, sb.clone()) {
        return Some(result);
    }

    // Try selection color
    if let Some(result) = apply_selection_color(token, sb) {
        return Some(result);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_text_color() {
        let sb = StyleBuilder::new();

        // Test basic color
        let result = apply_text_color("text-red-500", sb).unwrap();
        let _style = result.build();

        // Test white/black
        let sb = StyleBuilder::new();
        let result = apply_text_color("text-white", sb).unwrap();
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_text_color("text-black", sb).unwrap();
        let _style = result.build();

        // Test invalid
        let sb = StyleBuilder::new();
        let result = apply_text_color("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_bg_color() {
        let sb = StyleBuilder::new();

        // Test basic color
        let result = apply_bg_color("bg-blue-600", sb).unwrap();
        let _style = result.build();

        // Test gray scale
        let sb = StyleBuilder::new();
        let result = apply_bg_color("bg-gray-100", sb).unwrap();
        let _style = result.build();

        // Test invalid
        let sb = StyleBuilder::new();
        let result = apply_bg_color("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_border_color() {
        let sb = StyleBuilder::new();

        let result = apply_border_color("border-gray-300", sb).unwrap();
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_border_color("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_ring_color() {
        let sb = StyleBuilder::new();

        let result = apply_ring_color("ring-blue-500", sb).unwrap();
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_ring_color("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_accent_color() {
        let sb = StyleBuilder::new();

        let result = apply_accent_color("accent-purple-500", sb).unwrap();
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_accent_color("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_placeholder_color() {
        let sb = StyleBuilder::new();

        let result = apply_placeholder_color("placeholder-gray-400", sb).unwrap();
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_placeholder_color("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_color_utilities_integration() {
        let sb = StyleBuilder::new();

        // Test that the main function routes correctly
        assert!(apply_color_utilities("text-red-500", sb.clone()).is_some());
        assert!(apply_color_utilities("bg-blue-600", sb.clone()).is_some());
        assert!(apply_color_utilities("border-gray-300", sb.clone()).is_some());
        assert!(apply_color_utilities("invalid", sb.clone()).is_none());
    }
}
