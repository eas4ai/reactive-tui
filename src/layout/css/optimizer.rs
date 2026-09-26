//! CSS utility parsing with lookup tables and caching
//!
//! This module provides performance optimizations for CSS utility parsing
//! through static lookup tables and caching. Currently optimizes:
//! - Static utility lookups (flex, grid, display, etc.)
//! - Dynamic spacing utilities (padding, margin)
//!
//! Note: This is a partial optimization. Many dynamic utilities still
//! delegate to the original sequential parsers for correctness.

use crate::layout::style::StyleBuilder;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Type alias for utility application functions
type UtilityFn = fn(&str, StyleBuilder) -> Option<StyleBuilder>;

/// Utility registry entry
struct UtilityEntry {
    handler: UtilityFn,
}

/// Global utility lookup table for static utilities
static UTILITY_REGISTRY: Lazy<HashMap<&'static str, UtilityEntry>> = Lazy::new(|| {
    let mut registry = HashMap::with_capacity(200);

    // Register only static utilities that we can optimize
    register_display_utilities(&mut registry);
    register_flexbox_utilities(&mut registry);
    register_static_spacing_utilities(&mut registry);
    register_static_sizing_utilities(&mut registry);
    register_static_color_utilities(&mut registry);
    register_typography_utilities(&mut registry);
    register_position_utilities(&mut registry);

    registry
});

/// Prefix-based handlers for dynamic spacing values only
/// Other dynamic utilities delegate to existing implementations
static SPACING_PREFIXES: Lazy<Vec<(&'static str, UtilityFn)>> = Lazy::new(|| {
    vec![
        // Padding prefixes - fully implemented
        ("p-", apply_padding_dynamic),
        ("pt-", apply_padding_top_dynamic),
        ("pr-", apply_padding_right_dynamic),
        ("pb-", apply_padding_bottom_dynamic),
        ("pl-", apply_padding_left_dynamic),
        ("px-", apply_padding_x_dynamic),
        ("py-", apply_padding_y_dynamic),
        // Margin prefixes - fully implemented
        ("m-", apply_margin_dynamic),
        ("mt-", apply_margin_top_dynamic),
        ("mr-", apply_margin_right_dynamic),
        ("mb-", apply_margin_bottom_dynamic),
        ("ml-", apply_margin_left_dynamic),
        ("mx-", apply_margin_x_dynamic),
        ("my-", apply_margin_y_dynamic),
    ]
});

// Thread-local cache for complete class strings using proper LRU. A class
// such as `text-primary` resolves through the active theme, so each entry
// holds that theme's colors and the cache keeps the theme generation it was
// filled under.
thread_local! {
    static CLASS_CACHE: std::cell::RefCell<(u64, lru::LruCache<String, StyleBuilder>)> =
        std::cell::RefCell::new((
            crate::theme::Theme::generation(),
            lru::LruCache::new(
                std::num::NonZeroUsize::new(128).expect("Cache size must be non-zero"),
            ),
        ));
}

/// Runs `f` on this thread's class cache while the active theme is still the
/// one of `generation`, and returns `None` once it has changed. The cache is
/// emptied first whenever the theme changed since it was filled, so no entry
/// outlives the theme it was resolved under.
fn with_class_cache<R>(
    generation: u64,
    f: impl FnOnce(&mut lru::LruCache<String, StyleBuilder>) -> R,
) -> Option<R> {
    CLASS_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let current = crate::theme::Theme::generation();
        if cache.0 != current {
            cache.1.clear();
            cache.0 = current;
        }
        (current == generation).then(|| f(&mut cache.1))
    })
}

/// CSS utility class application with performance improvements
///
/// Provides performance improvements for:
/// - Static utility lookups via HashMap (O(1))
/// - Common spacing utilities via optimized handlers
/// - LRU caching of complete class strings
///
/// Falls back to original implementations for:
/// - Dynamic colors, sizes, grid values
/// - Theme-aware utilities
/// - Complex variant utilities
pub fn apply_utility_classes(
    class_str: &str,
    mut sb: StyleBuilder,
    theme: Option<&crate::theme::Theme>,
) -> StyleBuilder {
    if class_str.is_empty() {
        return sb;
    }

    // Check cache first
    // A class-only key is valid only for the default base and no theme, and
    // only while the active theme it resolved through is unchanged.
    let cacheable = sb == StyleBuilder::default() && theme.is_none();
    let generation = crate::theme::Theme::generation();
    let cached = cacheable
        .then(|| with_class_cache(generation, |cache| cache.get(class_str).cloned()))
        .flatten()
        .flatten();

    if let Some(cached_sb) = cached {
        return cached_sb;
    }

    // Process tokens
    for token in class_str.split_whitespace() {
        sb = apply_single_utility(token, sb, theme);
    }

    // Cache the result, unless the theme changed while it was resolved.
    if cacheable {
        with_class_cache(generation, |cache| {
            cache.put(class_str.to_string(), sb.clone());
        });
    }

    sb
}

/// Apply a single utility token
#[inline]
fn apply_single_utility(
    token: &str,
    sb: StyleBuilder,
    theme: Option<&crate::theme::Theme>,
) -> StyleBuilder {
    // App resolves these against acknowledged node state before layout.
    // A direct, stateless layout call has no active interaction state.
    if token
        .split(':')
        .any(|part| matches!(part, "focus" | "focus-within" | "hover" | "disabled"))
    {
        return sb;
    }
    // First, try exact match in registry (O(1) lookup)
    if let Some(entry) = UTILITY_REGISTRY.get(token) {
        if let Some(result) = (entry.handler)(token, sb.clone()) {
            return result;
        }
    }

    // Second, try spacing handlers
    for (prefix, handler) in SPACING_PREFIXES.iter() {
        if token.starts_with(prefix) {
            if let Some(result) = handler(token, sb.clone()) {
                return result;
            }
        }
    }

    // Finally, delegate to existing implementations for everything else
    // This ensures correctness while still providing optimization where possible
    delegate_to_existing_modules(token, sb, theme)
}

/// Delegate to existing module implementations for non-optimized utilities
fn delegate_to_existing_modules(
    token: &str,
    sb: StyleBuilder,
    theme: Option<&crate::theme::Theme>,
) -> StyleBuilder {
    // Try each module in order of likelihood

    // Layout utilities (includes position, display not in static registry)
    if let Some(result) = super::layout::apply_layout_utilities(token, sb.clone()) {
        return result;
    }

    // Spacing utilities (for gap, space-between utilities not in SPACING_PREFIXES)
    if let Some(result) = super::spacing::apply_spacing_utilities(token, sb.clone()) {
        return result;
    }

    // Sizing utilities (for dynamic values like w-[100px])
    if let Some(result) = super::sizing::apply_sizing_utilities(token, sb.clone()) {
        return result;
    }

    if let Some(result) = super::typography::apply_typography_utilities(token, sb.clone()) {
        return result;
    }

    // Color utilities (with theme support)
    if let Some(result) = super::gradients::apply_gradient_utility(token, sb.clone()) {
        return result;
    }
    if let Some(result) = super::colors::apply_color_utilities_with_theme(token, sb.clone(), theme)
    {
        return result;
    }

    // Container utilities
    if let Some(result) = super::containers::apply_container_utilities(token, sb.clone()) {
        return result;
    }

    // Effects utilities
    if let Some(result) = super::animations::apply_animation_utilities(token, sb.clone()) {
        return result;
    }
    if let Some(result) = super::effects::apply_effects_utilities(token, sb.clone()) {
        return result;
    }

    // Other utilities...
    if let Some(result) = super::interactions::apply_interaction_utilities(token, sb.clone()) {
        return result;
    }

    if let Some(result) = super::focus::apply_focus_utilities(token, sb.clone()) {
        return result;
    }

    if let Some(result) = super::accessibility::apply_accessibility_utilities(token, sb.clone()) {
        return result;
    }

    if let Some(result) = super::animations::apply_animation_utilities(token, sb.clone()) {
        return result;
    }

    if let Some(result) = super::variants::apply_variant_utilities(token, sb.clone()) {
        return result;
    }

    // Token not recognized
    sb
}

// Registration functions for static utilities only
fn register_display_utilities(registry: &mut HashMap<&'static str, UtilityEntry>) {
    let displays = [
        "block",
        "inline",
        "inline-block",
        "flex",
        "inline-flex",
        "grid",
        "inline-grid",
        "hidden",
        "contents",
        "flow-root",
    ];

    for display in displays {
        registry.insert(
            display,
            UtilityEntry {
                handler: |token, sb| super::layout::apply_layout_utilities(token, sb),
            },
        );
    }
}

fn register_flexbox_utilities(registry: &mut HashMap<&'static str, UtilityEntry>) {
    let flexbox = [
        "flex-row",
        "flex-row-reverse",
        "flex-col",
        "flex-col-reverse",
        "flex-wrap",
        "flex-nowrap",
        "flex-1",
        "flex-auto",
        "flex-initial",
        "flex-none",
        "grow",
        "grow-0",
        "shrink",
        "shrink-0",
        "justify-start",
        "justify-end",
        "justify-center",
        "justify-between",
        "justify-around",
        "justify-evenly",
        "items-start",
        "items-end",
        "items-center",
        "items-baseline",
        "items-stretch",
    ];

    for flex in flexbox {
        registry.insert(
            flex,
            UtilityEntry {
                handler: |token, sb| super::layout::apply_layout_utilities(token, sb),
            },
        );
    }
}

fn register_static_spacing_utilities(registry: &mut HashMap<&'static str, UtilityEntry>) {
    // Only register common static spacing values
    let spacings = [
        "p-0", "p-1", "p-2", "p-3", "p-4", "p-5", "p-6", "p-8", "m-0", "m-1", "m-2", "m-3", "m-4",
        "m-5", "m-6", "m-8", "px-0", "px-1", "px-2", "px-3", "px-4", "py-0", "py-1", "py-2",
        "py-3", "py-4", "mx-0", "mx-1", "mx-2", "mx-3", "mx-4", "my-0", "my-1", "my-2", "my-3",
        "my-4", "m-auto", "mx-auto", "my-auto",
    ];

    for spacing in spacings {
        registry.insert(
            spacing,
            UtilityEntry {
                handler: |token, sb| super::spacing::apply_spacing_utilities(token, sb),
            },
        );
    }
}

fn register_static_sizing_utilities(registry: &mut HashMap<&'static str, UtilityEntry>) {
    let sizes = [
        "w-full", "w-screen", "w-auto", "w-min", "w-max", "w-fit", "h-full", "h-screen", "h-auto",
        "h-min", "h-max", "h-fit", "w-1/2", "w-1/3", "w-2/3", "w-1/4", "w-3/4", "h-1/2", "h-1/3",
        "h-2/3", "h-1/4", "h-3/4",
    ];

    for size in sizes {
        registry.insert(
            size,
            UtilityEntry {
                handler: |token, sb| super::sizing::apply_sizing_utilities(token, sb),
            },
        );
    }
}

fn register_static_color_utilities(registry: &mut HashMap<&'static str, UtilityEntry>) {
    // Only register non-theme colors
    let colors = [
        "bg-transparent",
        "bg-white",
        "bg-black",
        "text-white",
        "text-black",
        "text-transparent",
        "border-transparent",
        "border-white",
        "border-black",
    ];

    for color in colors {
        registry.insert(
            color,
            UtilityEntry {
                handler: |token, sb| super::colors::apply_color_utilities(token, sb),
            },
        );
    }
}

fn register_typography_utilities(registry: &mut HashMap<&'static str, UtilityEntry>) {
    let typography = [
        "text-xs",
        "text-sm",
        "text-base",
        "text-lg",
        "text-xl",
        "text-2xl",
        "font-thin",
        "font-light",
        "font-normal",
        "font-medium",
        "font-semibold",
        "font-bold",
        "font-extrabold",
        "font-black",
        "italic",
        "not-italic",
        "underline",
        "no-underline",
        "line-through",
        "uppercase",
        "lowercase",
        "capitalize",
        "normal-case",
    ];

    for typo in typography {
        registry.insert(
            typo,
            UtilityEntry {
                handler: |token, sb| super::typography::apply_typography_utilities(token, sb),
            },
        );
    }
}

fn register_position_utilities(registry: &mut HashMap<&'static str, UtilityEntry>) {
    let positions = [
        "static",
        "fixed",
        "absolute",
        "relative",
        "sticky",
        "inset-0",
        "inset-x-0",
        "inset-y-0",
        "top-0",
        "right-0",
        "bottom-0",
        "left-0",
    ];

    for pos in positions {
        registry.insert(
            pos,
            UtilityEntry {
                handler: |token, sb| super::layout::apply_layout_utilities(token, sb),
            },
        );
    }
}

// Dynamic spacing handlers
fn apply_padding_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("p-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.padding_all_px(pixels));
        }
    }
    None
}

fn apply_padding_top_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("pt-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.padding_t_px(pixels));
        }
    }
    None
}

fn apply_padding_right_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("pr-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.padding_r_px(pixels));
        }
    }
    None
}

fn apply_padding_bottom_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("pb-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.padding_b_px(pixels));
        }
    }
    None
}

fn apply_padding_left_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("pl-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.padding_l_px(pixels));
        }
    }
    None
}

fn apply_padding_x_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("px-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.padding_l_px(pixels).padding_r_px(pixels));
        }
    }
    None
}

fn apply_padding_y_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("py-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.padding_t_px(pixels).padding_b_px(pixels));
        }
    }
    None
}

fn apply_margin_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("m-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.margin_all_px(pixels));
        }
    }
    None
}

fn apply_margin_top_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("mt-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.margin_t_px(pixels));
        }
    }
    None
}

fn apply_margin_right_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("mr-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.margin_r_px(pixels));
        }
    }
    None
}

fn apply_margin_bottom_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("mb-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.margin_b_px(pixels));
        }
    }
    None
}

fn apply_margin_left_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("ml-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.margin_l_px(pixels));
        }
    }
    None
}

fn apply_margin_x_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("mx-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.margin_l_px(pixels).margin_r_px(pixels));
        }
    }
    None
}

fn apply_margin_y_dynamic(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    if let Some(value) = token.strip_prefix("my-") {
        if let Some(pixels) = parse_spacing_value(value) {
            return Some(sb.margin_t_px(pixels).margin_b_px(pixels));
        }
    }
    None
}

/// Parse utility spacing scale to pixel values
fn parse_spacing_value(value: &str) -> Option<f32> {
    match value {
        "0" => Some(0.0),
        "px" => Some(1.0),
        "0.5" => Some(2.0),
        "1" => Some(4.0),
        "1.5" => Some(6.0),
        "2" => Some(8.0),
        "2.5" => Some(10.0),
        "3" => Some(12.0),
        "3.5" => Some(14.0),
        "4" => Some(16.0),
        "5" => Some(20.0),
        "6" => Some(24.0),
        "7" => Some(28.0),
        "8" => Some(32.0),
        "9" => Some(36.0),
        "10" => Some(40.0),
        "11" => Some(44.0),
        "12" => Some(48.0),
        "14" => Some(56.0),
        "16" => Some(64.0),
        "20" => Some(80.0),
        "24" => Some(96.0),
        "28" => Some(112.0),
        "32" => Some(128.0),
        "36" => Some(144.0),
        "40" => Some(160.0),
        "44" => Some(176.0),
        "48" => Some(192.0),
        "52" => Some(208.0),
        "56" => Some(224.0),
        "60" => Some(240.0),
        "64" => Some(256.0),
        "72" => Some(288.0),
        "80" => Some(320.0),
        "96" => Some(384.0),
        _ => value.parse::<f32>().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::style::StyleBuilder;
    use taffy::style::{
        AlignItems, Dimension, Display, FlexDirection, JustifyContent, LengthPercentage,
    };

    #[test]
    fn test_fast_parsing() {
        let sb = StyleBuilder::new();
        let classes = "flex flex-col justify-center items-center p-4 m-2";
        let result = apply_utility_classes(classes, sb, None);
        // Every utility in the class list lands in the built style
        let style = result.build();
        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.justify_content, Some(JustifyContent::Center));
        assert_eq!(style.align_items, Some(AlignItems::Center));
        assert_eq!(style.padding.left, LengthPercentage::length(16.0));
        assert_eq!(style.padding.bottom, LengthPercentage::length(16.0));
    }

    #[test]
    fn test_cache_hit() {
        let sb = StyleBuilder::new();
        let classes = "flex p-4";

        // First call - cache miss
        let _result1 = apply_utility_classes(classes, sb.clone(), None);

        // Second call - cache hit
        let _result2 = apply_utility_classes(classes, sb, None);

        // Cache should contain the entry
        CLASS_CACHE.with(|cache| {
            let mut cache_ref = cache.borrow_mut();
            assert!(cache_ref.1.get(classes).is_some());
        });
    }

    #[test]
    fn test_dynamic_spacing() {
        let sb = StyleBuilder::new();

        // Test padding
        let result = apply_padding_dynamic("p-4", sb.clone());
        assert!(result.is_some());

        // Test margin
        let result = apply_margin_dynamic("m-2", sb);
        assert!(result.is_some());
    }

    #[test]
    fn test_delegation_to_existing() {
        let sb = StyleBuilder::new();

        // These should delegate to existing modules
        let result = apply_utility_classes("bg-red-500", sb.clone(), None);
        assert!(
            result.bg_rgba.is_some(),
            "colour module sets the background"
        );

        // Bracketed arbitrary widths are not on the utility scale and leave the width alone
        let result = apply_utility_classes("w-[200px]", sb.clone(), None);
        assert_eq!(
            result.build().size.width,
            StyleBuilder::new().build().size.width
        );
        // The plain pixel form delegates to the sizing module
        let result = apply_utility_classes("w-200px", sb.clone(), None);
        assert_eq!(result.build().size.width, Dimension::length(200.0));

        let result = apply_utility_classes("hover:bg-blue-500", sb, None);
        assert!(
            result.bg_rgba.is_none(),
            "a hover variant does not paint the resting background"
        );
    }
}
