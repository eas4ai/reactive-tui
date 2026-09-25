//! Spacing utilities: padding, margin, gap

use super::parsers::parse_spacing;
use crate::layout::style::StyleBuilder;

/// Apply padding utilities
pub fn apply_padding(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // All-sides padding
    if let Some(px) = parse_spacing(token, "p-") {
        return Some(sb.padding_all_px(px));
    }

    // Horizontal padding
    if let Some(px) = parse_spacing(token, "px-") {
        return Some(sb.padding_x_px(px));
    }

    // Vertical padding
    if let Some(px) = parse_spacing(token, "py-") {
        return Some(sb.padding_y_px(px));
    }

    // Individual sides
    if let Some(px) = parse_spacing(token, "pl-") {
        return Some(sb.padding_l_px(px));
    }
    if let Some(px) = parse_spacing(token, "pr-") {
        return Some(sb.padding_r_px(px));
    }
    if let Some(px) = parse_spacing(token, "pt-") {
        return Some(sb.padding_t_px(px));
    }
    if let Some(px) = parse_spacing(token, "pb-") {
        return Some(sb.padding_b_px(px));
    }

    None
}

/// Apply margin utilities
pub fn apply_margin(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // All-sides margin
    if let Some(px) = parse_spacing(token, "m-") {
        return Some(sb.margin_all_px(px));
    }

    // Horizontal margin
    if let Some(px) = parse_spacing(token, "mx-") {
        return Some(sb.margin_x_px(px));
    }

    // Vertical margin
    if let Some(px) = parse_spacing(token, "my-") {
        return Some(sb.margin_y_px(px));
    }

    // Individual sides
    if let Some(px) = parse_spacing(token, "ml-") {
        return Some(sb.margin_l_px(px));
    }
    if let Some(px) = parse_spacing(token, "mr-") {
        return Some(sb.margin_r_px(px));
    }
    if let Some(px) = parse_spacing(token, "mt-") {
        return Some(sb.margin_t_px(px));
    }
    if let Some(px) = parse_spacing(token, "mb-") {
        return Some(sb.margin_b_px(px));
    }

    None
}

/// Apply gap utilities
pub fn apply_gap(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // All-direction gap
    if let Some(px) = parse_spacing(token, "gap-") {
        return Some(sb.gap_px(px, px));
    }

    // Horizontal gap
    if let Some(px) = parse_spacing(token, "gap-x-") {
        return Some(sb.gap_px(px, 0.0));
    }

    // Vertical gap
    if let Some(px) = parse_spacing(token, "gap-y-") {
        return Some(sb.gap_px(0.0, px));
    }

    None
}

/// Apply space-between utilities (for flex/grid children)
pub fn apply_space_between(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Horizontal space between
    if let Some(px) = parse_spacing(token, "space-x-") {
        // This would need special handling in the layout system
        // For now, we can approximate with gap
        return Some(sb.gap_px(px, 0.0));
    }

    // Vertical space between
    if let Some(px) = parse_spacing(token, "space-y-") {
        return Some(sb.gap_px(0.0, px));
    }

    None
}

/// Apply all spacing utilities
pub fn apply_spacing_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Try padding first
    if let Some(result) = apply_padding(token, sb.clone()) {
        return Some(result);
    }

    // Try margin
    if let Some(result) = apply_margin(token, sb.clone()) {
        return Some(result);
    }

    // Try gap
    if let Some(result) = apply_gap(token, sb.clone()) {
        return Some(result);
    }

    // Try space-between
    if let Some(result) = apply_space_between(token, sb) {
        return Some(result);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_utilities() {
        let sb = StyleBuilder::new();

        use taffy::style::LengthPercentage;
        let sides = |style: &taffy::style::Style| {
            (
                style.padding.left,
                style.padding.right,
                style.padding.top,
                style.padding.bottom,
            )
        };
        let cells = LengthPercentage::length;

        // Test all-sides padding (p-4 is 16 cells on the utility scale)
        let result = apply_padding("p-4", sb).expect("CSS spacing test should succeed");
        assert_eq!(
            sides(&result.build()),
            (cells(16.0), cells(16.0), cells(16.0), cells(16.0))
        );

        // Test directional padding
        let sb = StyleBuilder::new();
        let result = apply_padding("px-2", sb).expect("CSS spacing test should succeed");
        assert_eq!(
            sides(&result.build()),
            (cells(8.0), cells(8.0), cells(0.0), cells(0.0))
        );

        let sb = StyleBuilder::new();
        let result = apply_padding("py-8", sb).expect("CSS spacing test should succeed");
        assert_eq!(
            sides(&result.build()),
            (cells(0.0), cells(0.0), cells(32.0), cells(32.0))
        );
    }

    #[test]
    fn test_margin_utilities() {
        let sb = StyleBuilder::new();

        // Test all-sides margin
        let result = apply_margin("m-4", sb).expect("CSS spacing test should succeed");
        let _style = result.build();

        // Test directional margin
        let sb = StyleBuilder::new();
        let result = apply_margin("mx-auto", sb);
        assert!(result.is_none()); // mx-auto needs special handling

        let sb = StyleBuilder::new();
        let result = apply_margin("ml-2", sb).expect("CSS spacing test should succeed");
        let _style = result.build();
    }

    #[test]
    fn test_gap_utilities() {
        let sb = StyleBuilder::new();

        use taffy::style::LengthPercentage;
        let gap = |style: &taffy::style::Style| (style.gap.width, style.gap.height);
        let cells = LengthPercentage::length;

        // Test gap
        let result = apply_gap("gap-4", sb).expect("CSS spacing test should succeed");
        assert_eq!(gap(&result.build()), (cells(16.0), cells(16.0)));

        // Test directional gap
        let sb = StyleBuilder::new();
        let result = apply_gap("gap-x-2", sb).expect("CSS spacing test should succeed");
        assert_eq!(gap(&result.build()), (cells(8.0), cells(0.0)));

        let sb = StyleBuilder::new();
        let result = apply_gap("gap-y-8", sb).expect("CSS spacing test should succeed");
        assert_eq!(gap(&result.build()), (cells(0.0), cells(32.0)));
    }

    #[test]
    fn test_invalid_spacing() {
        let sb = StyleBuilder::new();

        // Test invalid tokens
        assert!(apply_padding("p-invalid", sb.clone()).is_none());
        assert!(apply_margin("m-invalid", sb.clone()).is_none());
        assert!(apply_gap("gap-", sb.clone()).is_none());
    }

    #[test]
    fn test_spacing_utilities_integration() {
        let sb = StyleBuilder::new();

        // Test that the main function routes correctly
        assert!(apply_spacing_utilities("p-4", sb.clone()).is_some());
        assert!(apply_spacing_utilities("m-2", sb.clone()).is_some());
        assert!(apply_spacing_utilities("gap-8", sb.clone()).is_some());
        assert!(apply_spacing_utilities("invalid", sb.clone()).is_none());
    }
}
