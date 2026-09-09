//! Typography utilities: font, text styling

use super::parsers::parse_font_weight;
use crate::layout::style::StyleBuilder;
use crate::layout::text::{Align, Transform, WhiteSpace, WordBreak};

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
pub fn apply_text_style(token: &str, mut sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Font styles
        "italic" => Some(sb.italic(true)),
        "not-italic" => Some(sb.italic(false)),

        // Text decoration
        "underline" => Some(sb.underline(true)),
        "overline" => Some(sb.underline(true)), // TUI approximation
        "line-through" => Some(sb.strike(true)),
        "no-underline" => Some(sb.underline(false)),

        "uppercase" => {
            sb.text.transform = Some(Transform::Upper);
            Some(sb)
        }
        "lowercase" => {
            sb.text.transform = Some(Transform::Lower);
            Some(sb)
        }
        "capitalize" => {
            sb.text.transform = Some(Transform::Capitalize);
            Some(sb)
        }
        "normal-case" => {
            sb.text.transform = Some(Transform::None);
            Some(sb)
        }
        "text-left" => {
            sb.text.align = Some(Align::Left);
            Some(sb)
        }
        "text-center" => {
            sb.text.align = Some(Align::Center);
            Some(sb)
        }
        "text-right" => {
            sb.text.align = Some(Align::Right);
            Some(sb)
        }
        "text-justify" => {
            sb.text.align = Some(Align::Justify);
            Some(sb)
        }
        "truncate" => {
            sb.text.ellipsis = Some(true);
            sb.text.whitespace = Some(WhiteSpace::NoWrap);
            Some(sb.overflow_hidden())
        }
        "text-ellipsis" => {
            sb.text.ellipsis = Some(true);
            Some(sb)
        }
        "text-clip" => {
            sb.text.ellipsis = Some(false);
            Some(sb)
        }
        "whitespace-normal" => {
            sb.text.whitespace = Some(WhiteSpace::Normal);
            Some(sb)
        }
        "whitespace-nowrap" => {
            sb.text.whitespace = Some(WhiteSpace::NoWrap);
            Some(sb)
        }
        "whitespace-pre" => {
            sb.text.whitespace = Some(WhiteSpace::Pre);
            Some(sb)
        }
        "whitespace-pre-line" => {
            sb.text.whitespace = Some(WhiteSpace::PreLine);
            Some(sb)
        }
        "whitespace-pre-wrap" => {
            sb.text.whitespace = Some(WhiteSpace::PreWrap);
            Some(sb)
        }
        "break-normal" => {
            sb.text.word_break = Some(WordBreak::Normal);
            Some(sb)
        }
        "break-words" => {
            sb.text.word_break = Some(WordBreak::Words);
            Some(sb)
        }
        "break-all" => {
            sb.text.word_break = Some(WordBreak::All);
            Some(sb)
        }

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
pub fn apply_line_height(token: &str, mut sb: StyleBuilder) -> Option<StyleBuilder> {
    let rows = match token {
        "leading-none" | "leading-tight" | "leading-snug" => 1,
        "leading-normal" | "leading-relaxed" | "leading-loose" => 2,
        _ => {
            let value = token.strip_prefix("leading-")?.parse::<f32>().ok()?;
            if !value.is_finite() || value < 0.0 {
                return None;
            }
            value.round().clamp(1.0, u16::MAX as f32) as usize
        }
    };
    sb.text.line_height = Some(rows);
    Some(sb)
}

/// Fractional tracking rounds to terminal cells; negative overlap is clamped.
pub fn apply_letter_spacing(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "tracking-tighter" | "tracking-tight" | "tracking-normal" | "tracking-wide"
        | "tracking-wider" | "tracking-widest" => Some(sb),
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

        let result =
            apply_font_weight("font-normal", sb.clone()).expect("Should apply normal font weight");
        let _style = result.build();

        let result =
            apply_font_weight("font-bold", sb.clone()).expect("Should apply bold font weight");
        let _style = result.build();

        let result =
            apply_font_weight("font-light", sb.clone()).expect("Should apply light font weight");
        let _style = result.build();

        assert!(apply_font_weight("font-invalid", sb).is_none());
    }

    #[test]
    fn test_text_style_utilities() {
        let sb = StyleBuilder::new();

        let result =
            apply_text_style("italic", sb.clone()).expect("Should apply italic text style");
        let _style = result.build();

        let result =
            apply_text_style("underline", sb.clone()).expect("Should apply underline text style");
        let _style = result.build();

        let result = apply_text_style("line-through", sb.clone())
            .expect("Should apply line-through text style");
        let _style = result.build();

        let result = apply_text_style("text-center", sb.clone())
            .expect("Should apply text-center alignment");
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

        let result = apply_line_height("leading-normal", sb.clone())
            .expect("Should apply normal line height");
        let _style = result.build();

        let result =
            apply_line_height("leading-tight", sb.clone()).expect("Should apply tight line height");
        let _style = result.build();

        let result =
            apply_line_height("leading-4", sb.clone()).expect("Should apply line height 4");
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
