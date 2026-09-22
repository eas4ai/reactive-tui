//! Color utilities: text colors, background colors

use crate::layout::colors::parse_color_token;
use crate::layout::style::StyleBuilder;

/// Apply text color utilities (legacy version without theme support)
pub fn apply_text_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_text_color_with_theme(token, sb, None)
}

/// Apply text color utilities with optional theme support
pub fn apply_text_color_with_theme(
    token: &str,
    sb: StyleBuilder,
    theme: Option<&crate::theme::Theme>,
) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("text-") {
        // Try theme variables first
        if let Some(theme) = theme {
            if let Some((r, g, b, a)) = resolve_theme_color(color_part, theme) {
                return Some(sb.fg_rgba(r, g, b, a));
            }
        }

        // Fallback to standard color parsing
        if let Some((r, g, b, a)) = parse_color_token(color_part) {
            Some(sb.fg_rgba(r, g, b, a))
        } else {
            None
        }
    } else {
        None
    }
}

/// Apply background color utilities (legacy version without theme support)
pub fn apply_bg_color(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_bg_color_with_theme(token, sb, None)
}

/// Apply background color utilities with optional theme support
pub fn apply_bg_color_with_theme(
    token: &str,
    sb: StyleBuilder,
    theme: Option<&crate::theme::Theme>,
) -> Option<StyleBuilder> {
    if let Some(color_part) = token.strip_prefix("bg-") {
        // Check for dynamic RGB colors like bg-[rgb(255,0,0)]
        if let Some(rgb_part) = color_part
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
        {
            if let Some((r, g, b, a)) = parse_dynamic_color(rgb_part) {
                return Some(sb.bg_rgba(r, g, b, a));
            }
        }

        // Try theme variables first
        if let Some(theme) = theme {
            if let Some((r, g, b, a)) = resolve_theme_color(color_part, theme) {
                return Some(sb.bg_rgba(r, g, b, a));
            }
        }

        // Fallback to standard color parsing
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

/// Apply all color utilities (legacy version without theme support)
pub fn apply_color_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    apply_color_utilities_with_theme(token, sb, None)
}

/// Apply all color utilities with optional theme support
pub fn apply_color_utilities_with_theme(
    token: &str,
    sb: StyleBuilder,
    theme: Option<&crate::theme::Theme>,
) -> Option<StyleBuilder> {
    // Try text color with theme support
    if let Some(result) = apply_text_color_with_theme(token, sb.clone(), theme) {
        return Some(result);
    }

    // Try background color with theme support
    if let Some(result) = apply_bg_color_with_theme(token, sb.clone(), theme) {
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

/// Resolve a color token to theme variable if possible
fn resolve_theme_color(token: &str, theme: &crate::theme::Theme) -> Option<(f32, f32, f32, f32)> {
    theme.resolve_variable(token)
}

/// Parse dynamic color values like rgb(255,0,0), rgba(255,0,0,0.5), #ff0000
pub(crate) fn parse_dynamic_color(color_str: &str) -> Option<(f32, f32, f32, f32)> {
    // Parse rgb(r,g,b) format
    if let Some(inner) = color_str
        .strip_prefix("rgb(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0));
        }
    }

    // Parse rgba(r,g,b,a) format
    if let Some(inner) = color_str
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 4 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            let a = parts[3].parse::<f32>().ok()?;
            return Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a));
        }
    }

    // Parse hex colors like #ff0000
    if color_str.starts_with('#') {
        return parse_hex_to_rgba(color_str);
    }

    None
}

/// Parse hex color string to RGBA floats. Delegates to the canonical
/// parser, so `#rgba` is accepted here exactly as everywhere else.
fn parse_hex_to_rgba(hex: &str) -> Option<(f32, f32, f32, f32)> {
    let (r, g, b, a) = crate::layout::colors::parse_hex_bytes(hex)?;

    Some((
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_text_color() {
        let sb = StyleBuilder::new();

        // Test basic color - should apply red-500 color
        let result = apply_text_color("text-red-500", sb).expect("CSS colors test should succeed");
        let _style = result.build();
        // Verify that foreground color was set (we can't easily test exact RGBA values due to internal representation)
        // But we can verify the function succeeded and returned a modified StyleBuilder

        // Test white/black
        let sb = StyleBuilder::new();
        let result = apply_text_color("text-white", sb).expect("CSS colors test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_text_color("text-black", sb).expect("CSS colors test should succeed");
        let _style = result.build();

        // Test invalid
        let sb = StyleBuilder::new();
        let result = apply_text_color("invalid", sb);
        assert!(result.is_none());

        // Test that non-text prefixes are ignored
        let sb = StyleBuilder::new();
        let result = apply_text_color("bg-red-500", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_bg_color() {
        let sb = StyleBuilder::new();

        // Test basic color
        let result = apply_bg_color("bg-blue-600", sb).expect("CSS colors test should succeed");
        let _style = result.build();

        // Test gray scale
        let sb = StyleBuilder::new();
        let result = apply_bg_color("bg-gray-100", sb).expect("CSS colors test should succeed");
        let _style = result.build();

        // Test invalid
        let sb = StyleBuilder::new();
        let result = apply_bg_color("invalid", sb);
        assert!(result.is_none());

        // Test that non-bg prefixes are ignored
        let sb = StyleBuilder::new();
        let result = apply_bg_color("text-blue-600", sb);
        assert!(result.is_none());

        // Test transparent
        let sb = StyleBuilder::new();
        let result = apply_bg_color("bg-transparent", sb).expect("CSS colors test should succeed");
        let _style = result.build();
    }

    #[test]
    fn test_apply_border_color() {
        let sb = StyleBuilder::new();

        let result =
            apply_border_color("border-gray-300", sb).expect("CSS colors test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_border_color("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_ring_color() {
        let sb = StyleBuilder::new();

        let result = apply_ring_color("ring-blue-500", sb).expect("CSS colors test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_ring_color("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_accent_color() {
        let sb = StyleBuilder::new();

        let result =
            apply_accent_color("accent-purple-500", sb).expect("CSS colors test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_accent_color("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_placeholder_color() {
        let sb = StyleBuilder::new();

        let result = apply_placeholder_color("placeholder-gray-400", sb)
            .expect("CSS colors test should succeed");
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
