//! Typography utilities: font, text styling

use super::parsers::parse_font_weight;
use crate::layout::style::StyleBuilder;

/// Apply font weight utilities
pub fn apply_font_weight(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(weight) = parse_font_weight(token) {
        match weight {
            100..=300 => Some(sb.font_weight_light()),
            400 => Some(sb.font_weight_normal()),
            500..=600 => Some(sb.font_weight_medium()),
            700..=800 => Some(sb.font_weight_bold()),
            900.. => Some(sb.font_weight_black()),
            _ => Some(sb.font_weight_normal()),
        }
    } else {
        None
    }
}

/// Apply text style utilities
pub fn apply_text_style(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Font styles
        "italic" => Some(sb.italic(true)),
        "not-italic" => Some(sb.italic(false)),

        // Text decoration
        "underline" => Some(sb.underline(true)),
        "overline" => Some(sb.underline(true)), // TUI approximation
        "line-through" => Some(sb.strike(true)),
        "no-underline" => Some(sb.underline(false)),

        // Text transform
        "uppercase" => Some(sb),  // Would need text processing
        "lowercase" => Some(sb),  // Would need text processing
        "capitalize" => Some(sb), // Would need text processing
        "normal-case" => Some(sb),

        // Text alignment (handled by layout)
        "text-left" => Some(sb.justify_content(crate::layout::style::JustifyContent::Start)),
        "text-center" => Some(sb.justify_content(crate::layout::style::JustifyContent::Center)),
        "text-right" => Some(sb.justify_content(crate::layout::style::JustifyContent::End)),
        "text-justify" => {
            Some(sb.justify_content(crate::layout::style::JustifyContent::SpaceBetween))
        }

        // Text overflow
        "truncate" => Some(sb),      // Would need text clipping
        "text-ellipsis" => Some(sb), // Would need text processing
        "text-clip" => Some(sb),

        // White space
        "whitespace-normal" => Some(sb),
        "whitespace-nowrap" => Some(sb),
        "whitespace-pre" => Some(sb),
        "whitespace-pre-line" => Some(sb),
        "whitespace-pre-wrap" => Some(sb),

        // Word break
        "break-normal" => Some(sb),
        "break-words" => Some(sb),
        "break-all" => Some(sb),

        _ => None,
    }
}

/// Apply font size utilities
pub fn apply_font_size(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Font sizes (TUI doesn't support different font sizes, but we acknowledge them)
        "text-xs" => Some(sb),   // 12px
        "text-sm" => Some(sb),   // 14px
        "text-base" => Some(sb), // 16px (default)
        "text-lg" => Some(sb),   // 18px
        "text-xl" => Some(sb),   // 20px
        "text-2xl" => Some(sb),  // 24px
        "text-3xl" => Some(sb),  // 30px
        "text-4xl" => Some(sb),  // 36px
        "text-5xl" => Some(sb),  // 48px
        "text-6xl" => Some(sb),  // 60px
        "text-7xl" => Some(sb),  // 72px
        "text-8xl" => Some(sb),  // 96px
        "text-9xl" => Some(sb),  // 128px
        _ => None,
    }
}

/// Apply line height utilities
pub fn apply_line_height(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Line heights (TUI doesn't support line height, but we acknowledge them)
        "leading-none" => Some(sb),    // 1
        "leading-tight" => Some(sb),   // 1.25
        "leading-snug" => Some(sb),    // 1.375
        "leading-normal" => Some(sb),  // 1.5
        "leading-relaxed" => Some(sb), // 1.625
        "leading-loose" => Some(sb),   // 2
        _ => {
            // Numeric line heights (leading-3, leading-4, etc.)
            if let Some(num_str) = token.strip_prefix("leading-") {
                // Only accept valid numeric values
                if num_str.parse::<f32>().is_ok() {
                    Some(sb)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

/// Apply letter spacing utilities
pub fn apply_letter_spacing(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Letter spacing (TUI doesn't support letter spacing, but we acknowledge them)
        "tracking-tighter" => Some(sb), // -0.05em
        "tracking-tight" => Some(sb),   // -0.025em
        "tracking-normal" => Some(sb),  // 0em
        "tracking-wide" => Some(sb),    // 0.025em
        "tracking-wider" => Some(sb),   // 0.05em
        "tracking-widest" => Some(sb),  // 0.1em
        _ => None,
    }
}

/// Apply font family utilities
pub fn apply_font_family(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Font families (TUI uses system font, but we acknowledge them)
        "font-sans" => Some(sb),  // Sans-serif
        "font-serif" => Some(sb), // Serif
        "font-mono" => Some(sb),  // Monospace (most appropriate for TUI)
        _ => None,
    }
}

/// Apply all typography utilities
pub fn apply_typography_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Try font weight
    if let Some(result) = apply_font_weight(token, sb.clone()) {
        return Some(result);
    }

    // Try text style
    if let Some(result) = apply_text_style(token, sb.clone()) {
        return Some(result);
    }

    // Try font size
    if let Some(result) = apply_font_size(token, sb.clone()) {
        return Some(result);
    }

    // Try line height
    if let Some(result) = apply_line_height(token, sb.clone()) {
        return Some(result);
    }

    // Try letter spacing
    if let Some(result) = apply_letter_spacing(token, sb.clone()) {
        return Some(result);
    }

    // Try font family
    if let Some(result) = apply_font_family(token, sb) {
        return Some(result);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_weight_utilities() {
        let sb = StyleBuilder::new();

        let result = apply_font_weight("font-normal", sb.clone()).expect("Should apply normal font weight");
        let _style = result.build();

        let result = apply_font_weight("font-bold", sb.clone()).expect("Should apply bold font weight");
        let _style = result.build();

        let result = apply_font_weight("font-light", sb.clone()).expect("Should apply light font weight");
        let _style = result.build();

        assert!(apply_font_weight("font-invalid", sb).is_none());
    }

    #[test]
    fn test_text_style_utilities() {
        let sb = StyleBuilder::new();

        let result = apply_text_style("italic", sb.clone()).expect("Should apply italic text style");
        let _style = result.build();

        let result = apply_text_style("underline", sb.clone()).expect("Should apply underline text style");
        let _style = result.build();

        let result = apply_text_style("line-through", sb.clone()).expect("Should apply line-through text style");
        let _style = result.build();

        let result = apply_text_style("text-center", sb.clone()).expect("Should apply text-center alignment");
        let _style = result.build();

        assert!(apply_text_style("invalid", sb).is_none());
    }

    #[test]
    fn test_font_size_utilities() {
        let sb = StyleBuilder::new();

        let result = apply_font_size("text-base", sb.clone()).expect("Should apply base font size");
        let _style = result.build();

        let result = apply_font_size("text-xl", sb.clone()).expect("Should apply xl font size");
        let _style = result.build();

        let result = apply_font_size("text-xs", sb.clone()).expect("Should apply xs font size");
        let _style = result.build();

        assert!(apply_font_size("text-invalid", sb).is_none());
    }

    #[test]
    fn test_line_height_utilities() {
        let sb = StyleBuilder::new();

        let result = apply_line_height("leading-normal", sb.clone()).expect("Should apply normal line height");
        let _style = result.build();

        let result = apply_line_height("leading-tight", sb.clone()).expect("Should apply tight line height");
        let _style = result.build();

        let result = apply_line_height("leading-4", sb.clone()).expect("Should apply line height 4");
        let _style = result.build();

        assert!(apply_line_height("leading-invalid", sb).is_none());
    }

    #[test]
    fn test_typography_utilities_integration() {
        let sb = StyleBuilder::new();

        // Test that the main function routes correctly
        assert!(apply_typography_utilities("font-bold", sb.clone()).is_some());
        assert!(apply_typography_utilities("italic", sb.clone()).is_some());
        assert!(apply_typography_utilities("text-xl", sb.clone()).is_some());
        assert!(apply_typography_utilities("leading-normal", sb.clone()).is_some());
        assert!(apply_typography_utilities("invalid", sb).is_none());
    }
}
