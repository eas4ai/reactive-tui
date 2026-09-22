//! Container and layout utilities
//!
//! This module provides utilities for:
//! - Container utilities (container, responsive containers)
//! - Centering utilities (mx-auto, my-auto)
//! - Aspect ratio utilities (aspect-square, aspect-video, aspect-[4/3])
//! - Layout containers and responsive breakpoints

use crate::layout::style::StyleBuilder;

/// Apply container utilities
pub fn apply_container_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Container utilities
    if let Some(result) = apply_container(token, sb.clone()) {
        return Some(result);
    }

    // Centering utilities
    if let Some(result) = apply_centering(token, sb.clone()) {
        return Some(result);
    }

    // Aspect ratio utilities
    if let Some(result) = apply_aspect_ratio(token, sb.clone()) {
        return Some(result);
    }

    None
}

/// Apply container utilities
pub fn apply_container(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "container" => {
            // Responsive container with max-width constraints
            // In TUI, we'll use full width but with proper centering
            Some(sb.width_pct(100.0))
        }
        "container-sm" => {
            // Small container (max-width: 640px equivalent in TUI)
            Some(sb.width_px(80.0)) // ~80 chars wide
        }
        "container-md" => {
            // Medium container (max-width: 768px equivalent)
            Some(sb.width_px(96.0)) // ~96 chars wide
        }
        "container-lg" => {
            // Large container (max-width: 1024px equivalent)
            Some(sb.width_px(128.0)) // ~128 chars wide
        }
        "container-xl" => {
            // Extra large container (max-width: 1280px equivalent)
            Some(sb.width_px(160.0)) // ~160 chars wide
        }
        "container-2xl" => {
            // 2X large container (max-width: 1536px equivalent)
            Some(sb.width_px(192.0)) // ~192 chars wide
        }
        _ => None,
    }
}

/// Apply centering utilities
pub fn apply_centering(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "mx-auto" => {
            // Center horizontally with auto margins
            // In TUI, we can use justify-content center on parent or margin auto
            Some(sb.margin_x_auto())
        }
        "my-auto" => {
            // Center vertically with auto margins
            Some(sb.margin_y_auto())
        }
        "m-auto" => {
            // Center both horizontally and vertically
            Some(sb.margin_auto())
        }
        _ => None,
    }
}

/// Apply aspect ratio utilities
pub fn apply_aspect_ratio(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "aspect-square" => {
            // 1:1 aspect ratio
            Some(apply_aspect_ratio_constraint(sb, 1.0, 1.0))
        }
        "aspect-video" => {
            // 16:9 aspect ratio
            Some(apply_aspect_ratio_constraint(sb, 16.0, 9.0))
        }
        "aspect-[4/3]" => {
            // 4:3 aspect ratio
            Some(apply_aspect_ratio_constraint(sb, 4.0, 3.0))
        }
        "aspect-[3/2]" => {
            // 3:2 aspect ratio
            Some(apply_aspect_ratio_constraint(sb, 3.0, 2.0))
        }
        "aspect-[5/4]" => {
            // 5:4 aspect ratio
            Some(apply_aspect_ratio_constraint(sb, 5.0, 4.0))
        }
        "aspect-auto" => {
            // Remove aspect ratio constraint
            Some(sb)
        }
        _ => {
            // Try to parse custom aspect ratio like aspect-[2/1]
            parse_custom_aspect_ratio(token).map(|custom_ratio| {
                apply_aspect_ratio_constraint(sb, custom_ratio.0, custom_ratio.1)
            })
        }
    }
}

/// Apply aspect ratio constraint to StyleBuilder
fn apply_aspect_ratio_constraint(
    sb: StyleBuilder,
    width_ratio: f32,
    height_ratio: f32,
) -> StyleBuilder {
    // In TUI, we can approximate aspect ratios using character dimensions
    // Since characters are typically taller than wide (~2:1 ratio), we adjust
    let char_aspect_ratio = 0.5; // Characters are about half as wide as tall
    let adjusted_width = width_ratio * char_aspect_ratio;

    // Set aspect ratio constraint using reactive-tui's advanced layout system
    // reactive-tui provides sophisticated aspect ratio handling for TUI applications
    sb.aspect_ratio(adjusted_width / height_ratio)
}

/// Parse custom aspect ratio from token like "aspect-[2/1]"
fn parse_custom_aspect_ratio(token: &str) -> Option<(f32, f32)> {
    if !token.starts_with("aspect-[") || !token.ends_with(']') {
        return None;
    }

    let ratio_str = &token[8..token.len() - 1]; // Remove "aspect-[" and "]"
    let parts: Vec<&str> = ratio_str.split('/').collect();

    if parts.len() != 2 {
        return None;
    }

    let width = parts[0].parse::<f32>().ok()?;
    let height = parts[1].parse::<f32>().ok()?;

    if height == 0.0 {
        return None;
    }

    Some((width, height))
}

/// Apply responsive container utilities
pub fn apply_responsive_container(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Handle responsive prefixes like sm:container, md:container, etc.
    if token.contains(":container") {
        // reactive-tui's responsive system handles viewport-aware containers
        // This implementation provides consistent responsive behavior
        return Some(sb.width_pct(100.0));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use taffy::style::{Dimension, LengthPercentageAuto};

    #[test]
    fn test_container_utilities() {
        let sb = StyleBuilder::new();

        // Test basic container: full width
        let result =
            apply_container("container", sb.clone()).expect("container class should be valid");
        assert_eq!(result.build().size.width, Dimension::percent(1.0));

        // Test sized containers: fixed cell widths
        let result = apply_container("container-sm", sb.clone())
            .expect("container-sm class should be valid");
        assert_eq!(result.build().size.width, Dimension::length(80.0));

        let result = apply_container("container-lg", sb.clone())
            .expect("container-lg class should be valid");
        assert_eq!(result.build().size.width, Dimension::length(128.0));
    }

    #[test]
    fn test_centering_utilities() {
        let sb = StyleBuilder::new();

        let auto = LengthPercentageAuto::auto();

        // Test horizontal centering
        let result = apply_centering("mx-auto", sb.clone()).expect("mx-auto class should be valid");
        let style = result.build();
        assert_eq!((style.margin.left, style.margin.right), (auto, auto));
        assert_ne!(
            style.margin.top, auto,
            "mx-auto leaves vertical margins alone"
        );

        // Test vertical centering
        let result = apply_centering("my-auto", sb.clone()).expect("my-auto class should be valid");
        let style = result.build();
        assert_eq!((style.margin.top, style.margin.bottom), (auto, auto));
        assert_ne!(
            style.margin.left, auto,
            "my-auto leaves horizontal margins alone"
        );

        // Test full centering
        let result = apply_centering("m-auto", sb.clone()).expect("m-auto class should be valid");
        let style = result.build();
        assert_eq!(
            (
                style.margin.left,
                style.margin.right,
                style.margin.top,
                style.margin.bottom
            ),
            (auto, auto, auto, auto)
        );
    }

    #[test]
    fn test_aspect_ratio_utilities() {
        let sb = StyleBuilder::new();

        // Ratios are halved horizontally because a cell is about twice as tall as wide
        // Test square aspect ratio
        let result = apply_aspect_ratio("aspect-square", sb.clone())
            .expect("aspect-square class should be valid");
        assert_eq!(result.build().aspect_ratio, Some(0.5));

        // Test video aspect ratio
        let result = apply_aspect_ratio("aspect-video", sb.clone())
            .expect("aspect-video class should be valid");
        assert_eq!(result.build().aspect_ratio, Some(16.0_f32 * 0.5 / 9.0));

        // Test custom aspect ratio
        let result = apply_aspect_ratio("aspect-[4/3]", sb.clone())
            .expect("aspect-[4/3] class should be valid");
        assert_eq!(result.build().aspect_ratio, Some(4.0_f32 * 0.5 / 3.0));
    }

    #[test]
    fn test_custom_aspect_ratio_parsing() {
        assert_eq!(parse_custom_aspect_ratio("aspect-[2/1]"), Some((2.0, 1.0)));
        assert_eq!(
            parse_custom_aspect_ratio("aspect-[16/9]"),
            Some((16.0, 9.0))
        );
        assert_eq!(parse_custom_aspect_ratio("aspect-[3/4]"), Some((3.0, 4.0)));

        // Invalid cases
        assert_eq!(parse_custom_aspect_ratio("aspect-2/1"), None);
        assert_eq!(parse_custom_aspect_ratio("aspect-[2]"), None);
        assert_eq!(parse_custom_aspect_ratio("aspect-[2/0]"), None);
    }

    #[test]
    fn test_container_utilities_integration() {
        let sb = StyleBuilder::new();

        // Test that all container utilities return Some
        assert!(apply_container_utilities("container", sb.clone()).is_some());
        assert!(apply_container_utilities("mx-auto", sb.clone()).is_some());
        assert!(apply_container_utilities("aspect-square", sb.clone()).is_some());

        // Test that invalid utilities return None
        assert!(apply_container_utilities("invalid-utility", sb.clone()).is_none());
    }
}
