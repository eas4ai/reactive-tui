//! Visual effects utilities: opacity, z-index, visibility, shadows, borders, transforms
//!
//! This module consolidates all visual effects including:
//! - Opacity and visibility controls
//! - Z-index layering
//! - Shadow effects adapted for terminal rendering
//! - Border styling and radius
//! - Transform utilities (scale, translate, rotate)
//! - Backdrop effects for overlays

use super::parsers::{parse_opacity, parse_z_index};
use crate::layout::style::StyleBuilder;

/// Apply opacity utilities
pub fn apply_opacity(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    parse_opacity(token).map(|alpha| sb.opacity(alpha))
}

/// Apply z-index utilities
pub fn apply_z_index(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    parse_z_index(token).map(|z| sb.z_index(z))
}

/// Apply visibility utilities
pub fn apply_visibility(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "visible" => Some(sb.opacity(1.0)),
        "invisible" => Some(sb.opacity(0.0)),
        "collapse" => Some(sb.size_px(Some(0.0), Some(0.0))),
        "sr-only" | "not-sr-only" => super::focus::apply_focus_utilities(token, sb),
        "pointer-events-none" => Some(sb.opacity(0.5)),
        "pointer-events-auto" => Some(sb.opacity(1.0)),
        _ => None,
    }
}

/// Apply cursor utilities
pub fn apply_cursor(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "cursor-pointer" => Some(sb.underline(true)),
        "cursor-not-allowed" => Some(sb.opacity(0.5).strike(true)),
        "cursor-text" => Some(sb.italic(true)),
        "cursor-default" => Some(sb),
        "cursor-wait" => Some(sb.opacity(0.7)),
        "cursor-help" => Some(sb.underline(true)),
        _ => None,
    }
}

/// Apply border utilities (TUI-adapted)
pub fn apply_border(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "border" => {
            // Default border - add subtle background for border effect
            if !sb.has_bg_color() {
                Some(sb.bg_rgba(0.2, 0.2, 0.2, 1.0))
            } else {
                Some(sb)
            }
        }
        "border-0" => {
            // Remove border - this would need special handling
            Some(sb)
        }
        "border-2" => {
            // Thicker border - could use different background intensity
            if !sb.has_bg_color() {
                Some(sb.bg_rgba(0.3, 0.3, 0.3, 1.0))
            } else {
                Some(sb)
            }
        }
        "border-4" => {
            // Even thicker border
            if !sb.has_bg_color() {
                Some(sb.bg_rgba(0.4, 0.4, 0.4, 1.0))
            } else {
                Some(sb)
            }
        }
        _ => {
            // Border colors - apply as background for border effect
            if token.starts_with("border-")
                && (token.contains("gray") || token.contains("black") || token.contains("white"))
            {
                // This is a TUI approximation of borders using background colors
                Some(sb)
            } else {
                None
            }
        }
    }
}

/// Apply rounded corner utilities (TUI-appropriate implementations)
pub fn apply_rounded(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "rounded" | "rounded-md" => {
            // For TUI, we can't actually round corners, but we can add subtle styling
            // Add a subtle background to indicate rounded styling
            if !sb.has_bg_color() {
                Some(sb.bg_rgba(0.95, 0.95, 0.95, 1.0))
            } else {
                Some(sb)
            }
        }
        "rounded-lg" | "rounded-xl" | "rounded-2xl" | "rounded-3xl" => {
            // More pronounced rounded effect with slightly darker background
            if !sb.has_bg_color() {
                Some(sb.bg_rgba(0.9, 0.9, 0.9, 1.0))
            } else {
                Some(sb)
            }
        }
        "rounded-none" => {
            // Sharp corners - no special styling needed
            Some(sb)
        }
        "rounded-full" => {
            // Full border radius (circle/pill shape)
            Some(sb)
        }
        _ => {
            // Individual corner rounding
            if token.starts_with("rounded-") {
                Some(sb)
            } else {
                None
            }
        }
    }
}

/// Apply shadow utilities (TUI-adapted)
pub fn apply_shadow(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "shadow-sm" => {
            // Small shadow - subtle background darkening
            if !sb.has_bg_color() {
                Some(sb.bg_rgba(0.95, 0.95, 0.95, 1.0))
            } else {
                Some(sb)
            }
        }
        "shadow" | "shadow-md" => {
            // Medium shadow
            if !sb.has_bg_color() {
                Some(sb.bg_rgba(0.9, 0.9, 0.9, 1.0))
            } else {
                Some(sb)
            }
        }
        "shadow-lg" => {
            // Large shadow
            if !sb.has_bg_color() {
                Some(sb.bg_rgba(0.85, 0.85, 0.85, 1.0))
            } else {
                Some(sb)
            }
        }
        "shadow-xl" | "shadow-2xl" => {
            // Extra large shadow
            if !sb.has_bg_color() {
                Some(sb.bg_rgba(0.8, 0.8, 0.8, 1.0))
            } else {
                Some(sb)
            }
        }
        "shadow-none" => {
            // No shadow
            Some(sb)
        }
        _ => None,
    }
}

/// Apply backdrop utilities (for modals, overlays)
pub fn apply_backdrop(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "backdrop-blur-sm" => {
            // Small backdrop blur - simulate with opacity
            Some(sb.opacity(0.95))
        }
        "backdrop-blur" | "backdrop-blur-md" => {
            // Medium backdrop blur
            Some(sb.opacity(0.9))
        }
        "backdrop-blur-lg" => {
            // Large backdrop blur
            Some(sb.opacity(0.8))
        }
        "backdrop-brightness-50" => {
            // Darken backdrop
            Some(sb.bg_rgba(0.0, 0.0, 0.0, 0.5))
        }
        "backdrop-brightness-75" => {
            // Slightly darken backdrop
            Some(sb.bg_rgba(0.0, 0.0, 0.0, 0.25))
        }
        _ => None,
    }
}

/// Apply transform utilities
pub fn apply_transform(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    super::animations::apply_animation_utilities(token, sb)
}

/// Apply scale transform utilities
pub fn apply_scale_transform(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    super::animations::apply_animation_utilities(token, sb)
}

/// Apply translate transform utilities
pub fn apply_translate_transform(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    super::animations::apply_animation_utilities(token, sb)
}

/// Apply rotate transform utilities
pub fn apply_rotate_transform(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    super::animations::apply_animation_utilities(token, sb)
}

/// Apply all effects utilities
pub fn apply_effects_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Try transform utilities first
    if let Some(result) = apply_transform(token, sb.clone()) {
        return Some(result);
    }

    // Try visibility
    if let Some(result) = apply_visibility(token, sb.clone()) {
        return Some(result);
    }

    // Try cursor
    if let Some(result) = apply_cursor(token, sb.clone()) {
        return Some(result);
    }

    // Try opacity
    if let Some(result) = apply_opacity(token, sb.clone()) {
        return Some(result);
    }

    // Try z-index
    if let Some(result) = apply_z_index(token, sb.clone()) {
        return Some(result);
    }

    // Try border
    if let Some(result) = apply_border(token, sb.clone()) {
        return Some(result);
    }

    // Try rounded
    if let Some(result) = apply_rounded(token, sb.clone()) {
        return Some(result);
    }

    // Try shadow
    if let Some(result) = apply_shadow(token, sb.clone()) {
        return Some(result);
    }

    // Try backdrop
    if let Some(result) = apply_backdrop(token, sb) {
        return Some(result);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_opacity() {
        let sb = StyleBuilder::new();

        let result = apply_opacity("opacity-50", sb).expect("CSS effects test should succeed");
        // Can't easily test the exact opacity value, but verify it doesn't panic
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_opacity("opacity-0", sb).expect("CSS effects test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_opacity("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_z_index() {
        let sb = StyleBuilder::new();

        let result = apply_z_index("z-10", sb).expect("CSS effects test should succeed");
        assert_eq!(result.get_z_index(), Some(10));

        let sb = StyleBuilder::new();
        let result = apply_z_index("z-50", sb).expect("CSS effects test should succeed");
        assert_eq!(result.get_z_index(), Some(50));

        let sb = StyleBuilder::new();
        let result = apply_z_index("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_border() {
        let sb = StyleBuilder::new();

        let result = apply_border("border", sb).expect("CSS effects test should succeed");
        // Should add background color for border effect
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_border("border-2", sb).expect("CSS effects test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_border("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_rounded() {
        let sb = StyleBuilder::new();

        let result = apply_rounded("rounded", sb).expect("CSS effects test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_rounded("rounded-full", sb).expect("CSS effects test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_rounded("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_shadow() {
        let sb = StyleBuilder::new();

        let result = apply_shadow("shadow", sb).expect("CSS effects test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_shadow("shadow-lg", sb).expect("CSS effects test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_shadow("invalid", sb);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_backdrop() {
        let sb = StyleBuilder::new();

        let result = apply_backdrop("backdrop-blur", sb).expect("CSS effects test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result =
            apply_backdrop("backdrop-brightness-50", sb).expect("CSS effects test should succeed");
        let _style = result.build();

        let sb = StyleBuilder::new();
        let result = apply_backdrop("invalid", sb);
        assert!(result.is_none());
    }
}
