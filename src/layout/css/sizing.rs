//! Sizing utilities: width, height, min/max sizes

use super::parsers::{parse_fraction, parse_percentage, parse_px, parse_spacing_terminal};
use crate::layout::style::StyleBuilder;

/// Apply width utilities
pub fn apply_width(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Handle screen width
    if token == "w-screen" {
        return Some(sb.width_percent(100.0));
    }

    // For width, use terminal spacing (direct character cells)
    if let Some(px) = parse_spacing_terminal(token, "w-") {
        return Some(sb.width_px(px));
    }

    // Try percentage/fraction values (w-1/2, w-full, etc.)
    if let Some(pct) = parse_percentage(token, "w-") {
        return Some(sb.width_percent(pct));
    }

    // Try fraction values (w-1/3, w-2/3, etc.)
    if let Some(pct) = parse_fraction(token, "w-") {
        return Some(sb.width_percent(pct));
    }

    // Try pixel values (w-100px)
    if let Some(px) = parse_px(token, "w-") {
        return Some(sb.width_px(px));
    }

    // Special width values
    match token {
        "w-auto" => Some(sb.width_auto()),
        "w-screen" => Some(sb.width_percent(100.0)),
        "w-min" => Some(sb.width_auto()), // min-content
        "w-max" => Some(sb.width_auto()), // max-content
        "w-fit" => Some(sb.width_auto()), // fit-content
        _ => None,
    }
}

/// Apply height utilities
pub fn apply_height(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Handle screen height
    if token == "h-screen" {
        return Some(sb.height_percent(100.0));
    }

    // For height, use terminal spacing (direct character cells)
    if let Some(px) = parse_spacing_terminal(token, "h-") {
        return Some(sb.height_px(px));
    }

    // Try percentage/fraction values (h-1/2, h-full, etc.)
    if let Some(pct) = parse_percentage(token, "h-") {
        return Some(sb.height_percent(pct));
    }

    // Try fraction values (h-1/3, h-2/3, etc.)
    if let Some(pct) = parse_fraction(token, "h-") {
        return Some(sb.height_percent(pct));
    }

    // Try pixel values (h-100px)
    if let Some(px) = parse_px(token, "h-") {
        return Some(sb.height_px(px));
    }

    // Special height values
    match token {
        "h-auto" => Some(sb.height_auto()),
        "h-screen" => Some(sb.height_percent(100.0)),
        "h-min" => Some(sb.height_auto()), // min-content
        "h-max" => Some(sb.height_auto()), // max-content
        "h-fit" => Some(sb.height_auto()), // fit-content
        _ => None,
    }
}

/// Apply min-width utilities
pub fn apply_min_width(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Use terminal spacing for min-width
    if let Some(px) = parse_spacing_terminal(token, "min-w-") {
        return Some(sb.min_width_px(px));
    }

    // Try percentage values
    if let Some(pct) = parse_percentage(token, "min-w-") {
        return Some(sb.min_width_percent(pct));
    }

    // Try fraction values
    if let Some(pct) = parse_fraction(token, "min-w-") {
        return Some(sb.min_width_percent(pct));
    }

    // Special min-width values
    match token {
        "min-w-0" => Some(sb.min_width_px(0.0)),
        "min-w-full" => Some(sb.min_width_percent(100.0)),
        "min-w-min" => Some(sb.min_width_auto()), // min-content
        "min-w-max" => Some(sb.min_width_auto()), // max-content
        "min-w-fit" => Some(sb.min_width_auto()), // fit-content
        _ => None,
    }
}

/// Apply max-width utilities
pub fn apply_max_width(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Use terminal spacing for max-width
    if let Some(px) = parse_spacing_terminal(token, "max-w-") {
        return Some(sb.max_width_px(px));
    }

    // Try percentage values
    if let Some(pct) = parse_percentage(token, "max-w-") {
        return Some(sb.max_width_percent(pct));
    }

    // Try fraction values
    if let Some(pct) = parse_fraction(token, "max-w-") {
        return Some(sb.max_width_percent(pct));
    }

    // Special max-width values
    match token {
        "max-w-none" => Some(sb.max_width_auto()), // No max width
        "max-w-full" => Some(sb.max_width_percent(100.0)),
        "max-w-min" => Some(sb.max_width_auto()), // min-content
        "max-w-max" => Some(sb.max_width_auto()), // max-content
        "max-w-fit" => Some(sb.max_width_auto()), // fit-content
        "max-w-screen" => Some(sb.max_width_percent(100.0)),
        _ => None,
    }
}

/// Apply min-height utilities
pub fn apply_min_height(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Use terminal spacing for min-height
    if let Some(px) = parse_spacing_terminal(token, "min-h-") {
        return Some(sb.min_height_px(px));
    }

    // Try percentage values
    if let Some(pct) = parse_percentage(token, "min-h-") {
        return Some(sb.min_height_percent(pct));
    }

    // Try fraction values
    if let Some(pct) = parse_fraction(token, "min-h-") {
        return Some(sb.min_height_percent(pct));
    }

    // Special min-height values
    match token {
        "min-h-0" => Some(sb.min_height_px(0.0)),
        "min-h-full" => Some(sb.min_height_percent(100.0)),
        "min-h-screen" => Some(sb.min_height_percent(100.0)),
        "min-h-min" => Some(sb.min_height_auto()), // min-content
        "min-h-max" => Some(sb.min_height_auto()), // max-content
        "min-h-fit" => Some(sb.min_height_auto()), // fit-content
        _ => None,
    }
}

/// Apply max-height utilities
pub fn apply_max_height(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Use terminal spacing for max-height
    if let Some(px) = parse_spacing_terminal(token, "max-h-") {
        return Some(sb.max_height_px(px));
    }

    // Try percentage values
    if let Some(pct) = parse_percentage(token, "max-h-") {
        return Some(sb.max_height_percent(pct));
    }

    // Try fraction values
    if let Some(pct) = parse_fraction(token, "max-h-") {
        return Some(sb.max_height_percent(pct));
    }

    // Special max-height values
    match token {
        "max-h-none" => Some(sb.max_height_auto()), // No max height
        "max-h-full" => Some(sb.max_height_percent(100.0)),
        "max-h-screen" => Some(sb.max_height_percent(100.0)),
        "max-h-min" => Some(sb.max_height_auto()), // min-content
        "max-h-max" => Some(sb.max_height_auto()), // max-content
        "max-h-fit" => Some(sb.max_height_auto()), // fit-content
        _ => None,
    }
}

/// Apply all sizing utilities
pub fn apply_sizing_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Try width utilities
    if let Some(result) = apply_width(token, sb.clone()) {
        return Some(result);
    }

    // Try height utilities
    if let Some(result) = apply_height(token, sb.clone()) {
        return Some(result);
    }

    // Try min-width utilities
    if let Some(result) = apply_min_width(token, sb.clone()) {
        return Some(result);
    }

    // Try max-width utilities
    if let Some(result) = apply_max_width(token, sb.clone()) {
        return Some(result);
    }

    // Try min-height utilities
    if let Some(result) = apply_min_height(token, sb.clone()) {
        return Some(result);
    }

    // Try max-height utilities
    if let Some(result) = apply_max_height(token, sb) {
        return Some(result);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width_utilities() {
        let sb = StyleBuilder::new();

        // Test spacing scale
        let result = apply_width("w-4", sb.clone()).expect("CSS sizing test should succeed");
        let _style = result.build();

        // Test percentage
        let result = apply_width("w-full", sb.clone()).expect("CSS sizing test should succeed");
        let _style = result.build();

        // Test fraction
        let result = apply_width("w-1/2", sb.clone()).expect("CSS sizing test should succeed");
        let _style = result.build();

        // Test special values
        let result = apply_width("w-auto", sb.clone()).expect("CSS sizing test should succeed");
        let _style = result.build();

        // Test invalid
        assert!(apply_width("w-invalid", sb).is_none());
    }

    #[test]
    fn test_height_utilities() {
        let sb = StyleBuilder::new();

        // Test spacing scale
        let result = apply_height("h-8", sb.clone()).expect("CSS sizing test should succeed");
        let _style = result.build();

        // Test percentage
        let result = apply_height("h-full", sb.clone()).expect("CSS sizing test should succeed");
        let _style = result.build();

        // Test special values
        let result = apply_height("h-screen", sb.clone()).expect("CSS sizing test should succeed");
        let _style = result.build();

        // Test invalid
        assert!(apply_height("h-invalid", sb).is_none());
    }

    #[test]
    fn test_min_max_utilities() {
        let sb = StyleBuilder::new();

        use taffy::style::Dimension;

        // Test min-width
        let result =
            apply_min_width("min-w-0", sb.clone()).expect("CSS sizing test should succeed");
        assert_eq!(result.build().min_size.width, Dimension::length(0.0));

        // Test max-width
        let result =
            apply_max_width("max-w-full", sb.clone()).expect("CSS sizing test should succeed");
        assert_eq!(result.build().max_size.width, Dimension::percent(1.0));

        // Test min-height
        let result =
            apply_min_height("min-h-screen", sb.clone()).expect("CSS sizing test should succeed");
        assert_eq!(result.build().min_size.height, Dimension::percent(1.0));

        // Test max-height
        let result =
            apply_max_height("max-h-none", sb.clone()).expect("CSS sizing test should succeed");
        assert_eq!(result.build().max_size.height, Dimension::auto());
    }

    #[test]
    fn test_sizing_utilities_integration() {
        let sb = StyleBuilder::new();

        // Test that the main function routes correctly
        assert!(apply_sizing_utilities("w-4", sb.clone()).is_some());
        assert!(apply_sizing_utilities("h-8", sb.clone()).is_some());
        assert!(apply_sizing_utilities("min-w-0", sb.clone()).is_some());
        assert!(apply_sizing_utilities("max-h-full", sb.clone()).is_some());
        assert!(apply_sizing_utilities("invalid", sb).is_none());
    }
}
