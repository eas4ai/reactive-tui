//! Animation and transition CSS utilities
//!
//! This module provides utilities for:
//! - Transition properties (transition-all, transition-colors, etc.)
//! - Duration utilities (duration-150, duration-300, etc.)
//! - Easing functions (ease-in, ease-out, ease-in-out)
//! - Transform utilities (scale, translate, rotate)
//! - Animation utilities (animate-pulse, animate-bounce, etc.)

use crate::layout::style::StyleBuilder;
use crate::animation::{Animation, AnimationBuilder, AnimatedProperty, EasingFunction, LoopMode};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use std::time::Duration;

/// CSS Animation metadata that can be converted to component animations
#[derive(Debug, Clone)]
pub struct CssAnimationSpec {
    /// Animation name (e.g., "pulse", "bounce", "spin")
    pub name: String,
    /// Duration of the animation
    pub duration: Duration,
    /// Easing function
    pub easing: EasingFunction,
    /// Loop behavior
    pub loop_mode: LoopMode,
    /// Animation properties to animate
    pub properties: Vec<AnimatedProperty>,
}

/// Global registry of CSS animations that can be applied to components
static CSS_ANIMATION_REGISTRY: OnceLock<Arc<RwLock<HashMap<String, CssAnimationSpec>>>> = OnceLock::new();

/// Get the global CSS animation registry
fn get_css_animation_registry() -> &'static Arc<RwLock<HashMap<String, CssAnimationSpec>>> {
    CSS_ANIMATION_REGISTRY.get_or_init(|| {
        let mut registry = HashMap::new();

        // Register built-in CSS animations
        register_builtin_animations(&mut registry);

        Arc::new(RwLock::new(registry))
    })
}

/// Register built-in CSS animations
fn register_builtin_animations(registry: &mut HashMap<String, CssAnimationSpec>) {
    // Pulse animation (opacity fade in/out)
    registry.insert("pulse".to_string(), CssAnimationSpec {
        name: "pulse".to_string(),
        duration: Duration::from_millis(2000),
        easing: EasingFunction::EaseInOut,
        loop_mode: LoopMode::Infinite,
        properties: vec![
            AnimatedProperty::Opacity(1.0, 0.5),
        ],
    });

    // Bounce animation (vertical movement simulation)
    registry.insert("bounce".to_string(), CssAnimationSpec {
        name: "bounce".to_string(),
        duration: Duration::from_millis(1000),
        easing: EasingFunction::EaseOut,
        loop_mode: LoopMode::Infinite,
        properties: vec![
            AnimatedProperty::Transform(crate::animation::TransformProperty::TranslateY(-25.0, 0.0)),
        ],
    });

    // Spin animation (rotation)
    registry.insert("spin".to_string(), CssAnimationSpec {
        name: "spin".to_string(),
        duration: Duration::from_millis(1000),
        easing: EasingFunction::Linear,
        loop_mode: LoopMode::Infinite,
        properties: vec![
            AnimatedProperty::Transform(crate::animation::TransformProperty::Rotate(0.0, 360.0)),
        ],
    });

    // Ping animation (scale + opacity)
    registry.insert("ping".to_string(), CssAnimationSpec {
        name: "ping".to_string(),
        duration: Duration::from_millis(1000),
        easing: EasingFunction::EaseOut,
        loop_mode: LoopMode::Infinite,
        properties: vec![
            AnimatedProperty::Transform(crate::animation::TransformProperty::Scale(1.0, 2.0)),
            AnimatedProperty::Opacity(1.0, 0.0),
        ],
    });
}

/// Apply a CSS animation to a component by ID
pub fn apply_css_animation_to_component(
    component_id: &str,
    animation_name: &str,
) -> Result<Animation, String> {
    let registry = get_css_animation_registry();
    let registry_guard = registry.read().map_err(|_| "Failed to read CSS animation registry")?;

    let spec = registry_guard.get(animation_name)
        .ok_or_else(|| format!("CSS animation '{}' not found", animation_name))?;

    // Create component animation from CSS spec
    let mut builder = AnimationBuilder::new(format!("css-{}-{}", animation_name, component_id))
        .duration(spec.duration)
        .easing(spec.easing.clone())
        .loop_mode(spec.loop_mode);

    // Add all properties from the spec
    for property in &spec.properties {
        builder = builder.animate_property(property.clone());
    }

    Ok(builder.build())
}

/// Register a custom CSS animation
pub fn register_css_animation(name: String, spec: CssAnimationSpec) -> Result<(), String> {
    let registry = get_css_animation_registry();
    let mut registry_guard = registry.write().map_err(|_| "Failed to write to CSS animation registry")?;

    registry_guard.insert(name, spec);
    Ok(())
}

/// Get all available CSS animation names
pub fn get_available_css_animations() -> Vec<String> {
    let registry = get_css_animation_registry();
    if let Ok(registry_guard) = registry.read() {
        registry_guard.keys().cloned().collect()
    } else {
        vec![]
    }
}

/// Create a CSS animation spec from parameters
pub fn create_css_animation_spec(
    name: &str,
    duration_ms: u64,
    easing: EasingFunction,
    loop_mode: LoopMode,
    properties: Vec<AnimatedProperty>,
) -> CssAnimationSpec {
    CssAnimationSpec {
        name: name.to_string(),
        duration: Duration::from_millis(duration_ms),
        easing,
        loop_mode,
        properties,
    }
}

/// Extract CSS animation names from a class string
///
/// Parses CSS classes like "animate-pulse", "animate-bounce", etc.
/// and returns the animation names (e.g., "pulse", "bounce")
///
/// Returns an empty vector if "animate-none" is present, as it disables all animations
pub fn extract_css_animations_from_classes(class_str: &str) -> Vec<String> {
    let mut animations = Vec::new();
    let mut has_animate_none = false;

    for class in class_str.split_whitespace() {
        if let Some(animation_name) = class.strip_prefix("animate-") {
            if animation_name == "none" {
                has_animate_none = true;
                break; // animate-none disables all animations
            } else {
                animations.push(animation_name.to_string());
            }
        }
    }

    // If animate-none is present, return empty vector (no animations)
    if has_animate_none {
        Vec::new()
    } else {
        animations
    }
}

/// Apply animation and transition utilities
pub fn apply_animation_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Transition utilities
    if let Some(result) = apply_transition_utilities(token, sb.clone()) {
        return Some(result);
    }

    // Duration utilities
    if let Some(result) = apply_duration_utilities(token, sb.clone()) {
        return Some(result);
    }

    // Easing utilities
    if let Some(result) = apply_easing_utilities(token, sb.clone()) {
        return Some(result);
    }

    // Transform utilities
    if let Some(result) = apply_transform_utilities(token, sb.clone()) {
        return Some(result);
    }

    // Animation presets
    if let Some(result) = apply_animation_presets(token, sb.clone()) {
        return Some(result);
    }

    None
}

/// Apply transition utilities
fn apply_transition_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Transition properties
        "transition" | "transition-all" => Some(apply_transition_all(sb)),
        "transition-none" => Some(apply_transition_none(sb)),
        "transition-colors" => Some(apply_transition_colors(sb)),
        "transition-opacity" => Some(apply_transition_opacity(sb)),
        "transition-shadow" => Some(apply_transition_shadow(sb)),
        "transition-transform" => Some(apply_transition_transform(sb)),

        _ => None,
    }
}

/// Apply duration utilities
fn apply_duration_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    let duration_ms = match token {
        "duration-75" => Some(75),
        "duration-100" => Some(100),
        "duration-150" => Some(150),
        "duration-200" => Some(200),
        "duration-300" => Some(300),
        "duration-500" => Some(500),
        "duration-700" => Some(700),
        "duration-1000" => Some(1000),
        _ => {
            // Try parsing custom duration like "duration-250"
            if let Some(duration_str) = token.strip_prefix("duration-") {
                duration_str.parse::<u64>().ok()
            } else {
                None
            }
        }
    };

    duration_ms.map(|ms| apply_duration(sb, Duration::from_millis(ms)))
}

/// Apply easing utilities
fn apply_easing_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "ease-linear" => Some(apply_easing_linear(sb)),
        "ease-in" => Some(apply_easing_in(sb)),
        "ease-out" => Some(apply_easing_out(sb)),
        "ease-in-out" => Some(apply_easing_in_out(sb)),
        _ => None,
    }
}

/// Apply transform utilities
fn apply_transform_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Scale utilities
    if let Some(scale_str) = token.strip_prefix("scale-") {
        if let Ok(scale) = scale_str.parse::<u16>() {
            let scale_factor = scale as f32 / 100.0;
            return Some(apply_scale(sb, scale_factor));
        }
    }

    // Translate utilities
    if let Some(translate_str) = token.strip_prefix("translate-x-") {
        if let Ok(pixels) = translate_str.parse::<i16>() {
            return Some(apply_translate_x(sb, pixels as f32));
        }
    }

    if let Some(translate_str) = token.strip_prefix("translate-y-") {
        if let Ok(pixels) = translate_str.parse::<i16>() {
            return Some(apply_translate_y(sb, pixels as f32));
        }
    }

    // Rotate utilities (limited for TUI)
    if let Some(rotate_str) = token.strip_prefix("rotate-") {
        if let Ok(degrees) = rotate_str.parse::<i16>() {
            return Some(apply_rotate(sb, degrees as f32));
        }
    }

    match token {
        // Common scale values
        "scale-0" => Some(apply_scale(sb, 0.0)),
        "scale-50" => Some(apply_scale(sb, 0.5)),
        "scale-75" => Some(apply_scale(sb, 0.75)),
        "scale-90" => Some(apply_scale(sb, 0.9)),
        "scale-95" => Some(apply_scale(sb, 0.95)),
        "scale-100" => Some(apply_scale(sb, 1.0)),
        "scale-105" => Some(apply_scale(sb, 1.05)),
        "scale-110" => Some(apply_scale(sb, 1.1)),
        "scale-125" => Some(apply_scale(sb, 1.25)),
        "scale-150" => Some(apply_scale(sb, 1.5)),

        // Transform reset
        "transform-none" => Some(apply_transform_none(sb)),

        _ => None,
    }
}

/// Apply animation presets
fn apply_animation_presets(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Animation presets
        "animate-none" => Some(apply_animate_none(sb)),
        "animate-spin" => Some(apply_animate_spin(sb)),
        "animate-ping" => Some(apply_animate_ping(sb)),
        "animate-pulse" => Some(apply_animate_pulse(sb)),
        "animate-bounce" => Some(apply_animate_bounce(sb)),

        _ => None,
    }
}

// Implementation functions

/// Apply transition to all properties
fn apply_transition_all(sb: StyleBuilder) -> StyleBuilder {
    // In TUI, we can simulate transitions with opacity changes or color shifts
    // This is metadata that would be used by the animation system
    sb
}

/// Remove all transitions
fn apply_transition_none(sb: StyleBuilder) -> StyleBuilder {
    // Disable transitions
    sb
}

/// Apply transition to color properties
fn apply_transition_colors(sb: StyleBuilder) -> StyleBuilder {
    // Transition colors only
    sb
}

/// Apply transition to opacity
fn apply_transition_opacity(sb: StyleBuilder) -> StyleBuilder {
    // Transition opacity only
    sb
}

/// Apply transition to shadow (TUI-adapted)
fn apply_transition_shadow(sb: StyleBuilder) -> StyleBuilder {
    // In TUI, shadows might be background color changes
    sb
}

/// Apply transition to transform properties
fn apply_transition_transform(sb: StyleBuilder) -> StyleBuilder {
    // Transition transforms
    sb
}

/// Apply animation duration
fn apply_duration(sb: StyleBuilder, _duration: Duration) -> StyleBuilder {
    // Duration is metadata for the animation system
    sb
}

/// Apply linear easing
fn apply_easing_linear(sb: StyleBuilder) -> StyleBuilder {
    // Easing is metadata for the animation system
    sb
}

/// Apply ease-in easing
fn apply_easing_in(sb: StyleBuilder) -> StyleBuilder {
    sb
}

/// Apply ease-out easing
fn apply_easing_out(sb: StyleBuilder) -> StyleBuilder {
    sb
}

/// Apply ease-in-out easing
fn apply_easing_in_out(sb: StyleBuilder) -> StyleBuilder {
    sb
}

/// Apply scale transform
fn apply_scale(sb: StyleBuilder, _scale: f32) -> StyleBuilder {
    // In TUI, scaling might be simulated with different character densities
    // or by affecting the layout size
    sb
}

/// Apply X-axis translation
fn apply_translate_x(sb: StyleBuilder, _pixels: f32) -> StyleBuilder {
    // Translation in TUI could affect positioning
    sb
}

/// Apply Y-axis translation
fn apply_translate_y(sb: StyleBuilder, _pixels: f32) -> StyleBuilder {
    sb
}

/// Apply rotation (limited TUI support)
fn apply_rotate(sb: StyleBuilder, _degrees: f32) -> StyleBuilder {
    // Rotation in TUI is very limited, might affect text orientation
    sb
}

/// Remove all transforms
fn apply_transform_none(sb: StyleBuilder) -> StyleBuilder {
    sb
}

/// Remove all animations
fn apply_animate_none(sb: StyleBuilder) -> StyleBuilder {
    sb
}

/// Apply spin animation
fn apply_animate_spin(sb: StyleBuilder) -> StyleBuilder {
    // Mark this element as having a CSS animation
    sb.with_css_animation("spin")
}

/// Apply ping animation (scale + opacity)
fn apply_animate_ping(sb: StyleBuilder) -> StyleBuilder {
    // Mark this element as having a CSS animation
    sb.with_css_animation("ping")
}

/// Apply pulse animation (opacity)
fn apply_animate_pulse(sb: StyleBuilder) -> StyleBuilder {
    // Mark this element as having a CSS animation
    sb.with_css_animation("pulse")
}

/// Apply bounce animation
fn apply_animate_bounce(sb: StyleBuilder) -> StyleBuilder {
    // Mark this element as having a CSS animation
    sb.with_css_animation("bounce")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_utilities() {
        let sb = StyleBuilder::new();

        // Test transition utilities
        let result = apply_animation_utilities("transition-all", sb.clone());
        assert!(result.is_some());

        let result = apply_animation_utilities("transition-colors", sb.clone());
        assert!(result.is_some());

        let result = apply_animation_utilities("transition-none", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_duration_utilities() {
        let sb = StyleBuilder::new();

        // Test duration utilities
        let result = apply_animation_utilities("duration-150", sb.clone());
        assert!(result.is_some());

        let result = apply_animation_utilities("duration-300", sb.clone());
        assert!(result.is_some());

        let result = apply_animation_utilities("duration-1000", sb.clone());
        assert!(result.is_some());

        // Test custom duration
        let result = apply_animation_utilities("duration-250", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_easing_utilities() {
        let sb = StyleBuilder::new();

        // Test easing utilities
        let result = apply_animation_utilities("ease-linear", sb.clone());
        assert!(result.is_some());

        let result = apply_animation_utilities("ease-in-out", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_transform_utilities() {
        let sb = StyleBuilder::new();

        // Test scale utilities
        let result = apply_animation_utilities("scale-95", sb.clone());
        assert!(result.is_some());

        let result = apply_animation_utilities("scale-105", sb.clone());
        assert!(result.is_some());

        // Test translate utilities
        let result = apply_animation_utilities("translate-x-4", sb.clone());
        assert!(result.is_some());

        let result = apply_animation_utilities("translate-y-2", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_animation_presets() {
        let sb = StyleBuilder::new();

        // Test animation presets
        let result = apply_animation_utilities("animate-pulse", sb.clone());
        assert!(result.is_some());

        let result = apply_animation_utilities("animate-bounce", sb.clone());
        assert!(result.is_some());

        let result = apply_animation_utilities("animate-spin", sb.clone());
        assert!(result.is_some());
    }
}
