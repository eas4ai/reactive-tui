//! Modern Animation API Layer
//!
//! Provides high-level convenience functions for creating animations with a modern,
//! developer-friendly interface inspired by modern web animation libraries.
//!
//! Features:
//! - `animate()` function for simple property animations
//! - `stagger()` function for creating staggered animation delays
//! - `create_timeline()` function for complex animation sequences
//! - Flexible parameter handling and intuitive API design

//!
//! # Current-value migration
//!
//! `PropertyValue::Single` and `Relative`, including `slide` and `spring_animate`,
//! need a handle from the owning App or screen. A string ID has no owner or
//! current state. `try_animate`, `Animation::try_play` and `Animation::try_seek`
//! report errors; the original convenience methods panic with that error.
//! Explicit `FromTo` and array endpoints continue to accept string IDs.
//!
//! ```
//! use reactive_tui::{animation::api::{try_animate, AnimateParams, PropertyValue},
//!     backend::SuprTuiBackend, builder::core::div, screen::{ScreenId, ScreenManager}};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let backend = SuprTuiBackend::with_writer(20, 4, std::io::sink())?;
//! let mut screens = ScreenManager::new(Box::new(backend));
//! screens.create_screen("main", "Main".into(),
//!     div().id("panel").class("w-full h-full bg-white opacity-50").build())?;
//! let target = screens.animation_target(&ScreenId::from("main"), "panel")?;
//! let mut animation = try_animate(&target, AnimateParams {
//!     opacity: Some(PropertyValue::Relative("+0.25".into())),
//!     autoplay: Some(false), ..Default::default()
//! })?;
//! animation.try_play()?;
//! animation.try_seek(0.5)?;
//! screens.update()?; // Presents opacity 0.625.
//! target.clear()?;   // Restore authored properties on the next frame.
//! # Ok(())
//! # }
//! ```
//!
//! For App, obtain `app.animation_targets()` before `app.run()` and capture that
//! lookup context in callbacks. Resolve a target after its first presented frame.
//! `app.animation_manager()` drives registered animations; standalone screen
//! users call `Animation::update` and `ScreenManager::update` in their loop.
//! Endpoints are captured on play and refreshed at the first sample after delay.
//! Pause/resume keeps captured endpoints; restart reads current presented values.
//! Samples remain visible after stop/completion until replaced or `target.clear()`.
//! Removing a target or cleaning up its owner invalidates its handles. Reusing
//! the same ID does not revive them; duplicate IDs within an owner are errors.
//!
//! Bound targets paint opacity, cell translations, uniform scale and rotation.
//! Custom numeric values read `ElementMetadata::animation_values`; built-in names
//! read actual styles instead. Percentage translations require presented layout.
//! Unrepresentable uniform scale and undeclared custom values return errors.
//! Other property families retain their explicit unbound animation APIs and are
//! rejected by the new bound numeric API rather than accepted without effect.
//!
//! Untyped animation samples retain every property in `AnimationValue::Map`.
//! Single numeric and opaque RGB samples keep their existing representation.
//! Nonopaque RGBA uses an r/g/b/a channel map in 0..=255; CSS values retain units.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Animation targets - can be a single ID or multiple IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationTargets {
    /// Single target element ID
    Single(String),
    /// Multiple target element IDs
    Multiple(Vec<String>),
}

impl From<String> for AnimationTargets {
    fn from(id: String) -> Self {
        Self::Single(id)
    }
}

impl From<&str> for AnimationTargets {
    fn from(id: &str) -> Self {
        Self::Single(id.to_string())
    }
}

impl From<Vec<String>> for AnimationTargets {
    fn from(ids: Vec<String>) -> Self {
        Self::Multiple(ids)
    }
}

impl From<Vec<&str>> for AnimationTargets {
    fn from(ids: Vec<&str>) -> Self {
        Self::Multiple(ids.iter().map(|&s| s.to_string()).collect())
    }
}

/// Animation construction input. Legacy serialized ID lists remain unchanged.
#[derive(Clone, Debug)]
pub enum AnimationTargetInput {
    /// Explicit endpoint animations identified by the legacy ID list.
    Ids(AnimationTargets),
    /// One current-value target in a specific owner.
    Bound(AnimationTarget),
    /// Multiple targets, each with its own current values.
    BoundMultiple(Vec<AnimationTarget>),
}
impl<T: Into<AnimationTargets>> From<T> for AnimationTargetInput {
    fn from(targets: T) -> Self {
        Self::Ids(targets.into())
    }
}
impl From<AnimationTarget> for AnimationTargetInput {
    fn from(target: AnimationTarget) -> Self {
        Self::Bound(target)
    }
}
impl From<&AnimationTarget> for AnimationTargetInput {
    fn from(target: &AnimationTarget) -> Self {
        Self::Bound(target.clone())
    }
}
impl From<Vec<AnimationTarget>> for AnimationTargetInput {
    fn from(targets: Vec<AnimationTarget>) -> Self {
        Self::BoundMultiple(targets)
    }
}

/// Parameters for the animate function
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimateParams {
    /// Animation ID (auto-generated if not provided)
    pub id: Option<String>,
    /// Duration in milliseconds
    pub duration: Option<f32>,
    /// Delay before animation starts
    pub delay: Option<DelayValue>,
    /// Easing function
    pub easing: Option<EasingFunction>,
    /// Loop mode
    pub loop_mode: Option<LoopMode>,
    /// Animation direction
    pub direction: Option<AnimationDirection>,
    /// Auto-play the animation
    pub autoplay: Option<bool>,
    /// Keyframe sequence for complex animations
    pub keyframes: Option<keyframes::KeyframeSequence>,

    // Common property animations
    /// Opacity animation
    pub opacity: Option<PropertyValue>,
    /// X translation
    pub translate_x: Option<PropertyValue>,
    /// Y translation
    pub translate_y: Option<PropertyValue>,
    /// Scale factor
    pub scale: Option<PropertyValue>,
    /// Rotation in degrees
    pub rotate: Option<PropertyValue>,
    /// Color animation
    pub color: Option<ColorValue>,
    /// Size animation (width, height)
    pub size: Option<SizeValue>,
    /// Position animation (x, y)
    pub position: Option<PositionValue>,

    // Custom properties
    /// Custom numeric properties
    pub custom: Option<HashMap<String, PropertyValue>>,
    /// CSS-style properties
    pub css: Option<HashMap<String, CssValue>>,
    /// Transform properties
    pub transform: Option<HashMap<String, f32>>,
}

/// Value for animating properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    /// Single target value (animates from current to this value)
    Single(f32),
    /// Explicit from-to range
    FromTo {
        /// Starting value
        from: f32,
        /// Ending value
        to: f32,
    },
    /// Array of values for keyframe-like progression
    Array(Vec<f32>),
    /// Relative change (e.g., "+50" means add 50 to current value)
    Relative(String),
}

/// Value for color animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorValue {
    /// RGB color
    Rgb(u8, u8, u8),
    /// RGBA color
    Rgba(u8, u8, u8, u8),
    /// From-to color animation
    FromTo {
        /// Starting RGB color (r, g, b)
        from: (u8, u8, u8),
        /// Ending RGB color (r, g, b)
        to: (u8, u8, u8),
    },
}

/// Value for size animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SizeValue {
    /// Both width and height
    Both(u16, u16),
    /// From-to size animation
    FromTo {
        /// Starting size (width, height)
        from: (u16, u16),
        /// Ending size (width, height)
        to: (u16, u16),
    },
}

/// Value for position animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionValue {
    /// Both x and y
    Both(i16, i16),
    /// From-to position animation
    FromTo {
        /// Starting position (x, y)
        from: (i16, i16),
        /// Ending position (x, y)
        to: (i16, i16),
    },
}

/// Delay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelayValue {
    /// Fixed delay in milliseconds
    Fixed(f32),
    /// Staggered delay configuration
    Stagger(stagger::StaggerConfig),
}

/// Animation direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationDirection {
    /// Play animation forward (default)
    Normal,
    /// Play animation in reverse
    Reverse,
    /// Alternate between forward and reverse on each iteration
    Alternate,
    /// Alternate starting with reverse
    AlternateReverse,
}

/// Generate a unique animation ID
fn generate_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("anim_{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Main animation creation function
///
/// # Panics
/// Panics if target or numeric value resolution fails. Use [`try_animate`] for
/// checked errors. Current-value animations require an owner-bound target handle.
///
/// Creates animations with a modern, flexible API that supports property animations,
/// keyframes, staggering, and advanced easing functions.
///
/// # Examples
///
/// ```rust, ignore
/// use reactive_tui::animation::*;
///
/// // Simple fade in
/// let fade = animate("my-element", AnimateParams {
///     opacity: Some(PropertyValue::FromTo { from: 0.0, to: 1.0 }),
///     duration: Some(500.0),
///     easing: Some(EasingFunction::EaseOut),
///     ..Default::default()
/// });
///
/// // Complex transform animation
/// let transform = animate("element", AnimateParams {
///     translate_x: Some(PropertyValue::FromTo{from:0.0,to:100.0}),
///     scale: Some(PropertyValue::FromTo { from: 0.8, to: 1.2 }),
///     rotate: Some(PropertyValue::FromTo{from:0.0,to:360.0}),
///     duration: Some(1000.0),
///     easing: Some(EasingFunction::spring_wobbly()),
///     ..Default::default()
/// });
/// ```
pub fn animate<T>(targets: T, params: AnimateParams) -> Animation
where
    T: Into<AnimationTargetInput>,
{
    try_animate(targets, params).unwrap_or_else(|error| panic!("cannot create animation: {error}"))
}

/// Checked animation construction. Single/Relative numeric values require a live
/// target handle. Explicit FromTo values still support string IDs.
pub fn try_animate<T: Into<AnimationTargetInput>>(
    targets: T,
    params: AnimateParams,
) -> Result<Animation, AnimationTargetError> {
    let targets = match targets.into() {
        AnimationTargetInput::Bound(target) => Some(vec![target]),
        AnimationTargetInput::BoundMultiple(targets) => Some(targets),
        AnimationTargetInput::Ids(_) => None,
    };
    if targets.is_none() {
        for (name, value) in super::binding::numeric_values(&params) {
            if matches!(value, PropertyValue::Single(_) | PropertyValue::Relative(_)) {
                return Err(AnimationTargetError::HandleRequired(name.into()));
            }
            super::binding::resolve_value(name, value, &HashMap::new())?;
        }
    }
    let mut binding = targets
        .map(|targets| super::binding::TargetBinding::new(targets, params.clone()))
        .transpose()?;
    let bound_property = binding
        .as_mut()
        .map(|binding| binding.resolve())
        .transpose()?;
    let animation_id = params.id.clone().unwrap_or_else(generate_id);

    let mut builder = AnimationBuilder::new(animation_id);

    // Set duration
    if let Some(duration) = params.duration {
        builder = builder.duration(Duration::from_millis(duration as u64));
    }

    // Set delay
    if let Some(ref delay) = params.delay {
        match delay {
            DelayValue::Fixed(ms) => {
                builder = builder.delay(Duration::from_millis(*ms as u64));
            }
            DelayValue::Stagger(config) => {
                // For stagger, we'll need to handle multiple targets
                // This is a simplified version - full stagger support would need timeline
                builder = builder.delay(config.delay);
            }
        }
    }

    // Set easing
    if let Some(ref easing) = params.easing {
        builder = builder.easing(easing.clone());
    }

    // Set loop mode
    if let Some(loop_mode) = params.loop_mode {
        builder = builder.loop_mode(loop_mode);
    }

    // Handle keyframes if provided
    if let Some(property) = bound_property {
        builder = builder.animate_property(property);
    } else if let Some(keyframes) = params.keyframes.clone() {
        builder = builder.animate_property(AnimatedProperty::Keyframes(keyframes));
    } else {
        // Extract animated properties from parameters
        let properties = extract_animated_properties(&params);

        if properties.len() == 1 {
            if let Some(prop) = properties.into_iter().next() {
                builder = builder.animate_property(prop);
            }
        } else if !properties.is_empty() {
            builder = builder.animate_property(AnimatedProperty::Multiple(properties));
        }
    }

    // Set auto-play
    let mut animation = builder.auto_play(false).build();
    animation.target_binding = binding;
    if params.autoplay.unwrap_or(true) {
        animation.try_play()?;
    }

    Ok(animation)
}

pub(super) fn combine_properties(mut properties: Vec<AnimatedProperty>) -> AnimatedProperty {
    if properties.len() == 1 {
        properties.remove(0)
    } else {
        AnimatedProperty::Multiple(properties)
    }
}

/// Extract animated properties from parameters
pub(super) fn extract_animated_properties(params: &AnimateParams) -> Vec<AnimatedProperty> {
    let mut properties = Vec::new();

    // Handle opacity
    if let Some(opacity) = &params.opacity {
        properties.push(convert_property_value_to_animated("opacity", opacity));
    }

    // Handle translation
    if let Some(translate_x) = &params.translate_x {
        properties.push(convert_transform_value("translateX", translate_x));
    }

    if let Some(translate_y) = &params.translate_y {
        properties.push(convert_transform_value("translateY", translate_y));
    }

    // Handle scale
    if let Some(scale) = &params.scale {
        properties.push(convert_transform_value("scale", scale));
    }

    // Handle rotation
    if let Some(rotate) = &params.rotate {
        properties.push(convert_transform_value("rotate", rotate));
    }

    // Handle color
    if let Some(color) = &params.color {
        properties.push(convert_color_value_to_animated(color));
    }

    // Handle size
    if let Some(size) = &params.size {
        properties.push(convert_size_value_to_animated(size));
    }

    // Handle position
    if let Some(position) = &params.position {
        properties.push(convert_position_value_to_animated(position));
    }

    // Handle custom properties
    if let Some(custom) = &params.custom {
        for (name, value) in custom {
            properties.push(convert_property_value_to_animated(name, value));
        }
    }

    // Handle CSS properties
    if let Some(css) = &params.css {
        for (name, value) in css {
            // Parse from/to values for CSS properties
            let value_str = match value {
                CssValue::String(s) => s.clone(),
                CssValue::Number(n) => n.to_string(),
                CssValue::Pixels(p) => format!("{p}px"),
                CssValue::Percentage(p) => format!("{p}%"),
                CssValue::Em(e) => format!("{e}em"),
                CssValue::Rem(r) => format!("{r}rem"),
                CssValue::ViewportWidth(vw) => format!("{vw}vw"),
                CssValue::ViewportHeight(vh) => format!("{vh}vh"),
                CssValue::Color { r: _, g: _, b: _ } => "inherit".to_string(), // Simplified
            };
            let (from_str, to_str) = parse_css_property_values(&value_str);
            let from_value = CssValue::String(from_str);
            let to_value = CssValue::String(to_str);
            properties.push(AnimatedProperty::CssProperty(
                name.clone(),
                from_value,
                to_value,
            ));
        }
    }

    // Handle transform properties
    if let Some(transform) = &params.transform {
        for (name, value) in transform {
            let transform_prop = match name.as_str() {
                "translateX" => TransformProperty::TranslateX(0.0, *value),
                "translateY" => TransformProperty::TranslateY(0.0, *value),
                "scale" | "scaleX" => TransformProperty::ScaleX(1.0, *value),
                "scaleY" => TransformProperty::ScaleY(1.0, *value),
                "rotate" => TransformProperty::Rotate(0.0, value.to_radians()),
                "skewX" => TransformProperty::SkewX(0.0, *value),
                "skewY" => TransformProperty::SkewY(0.0, *value),
                _ => continue,
            };
            properties.push(AnimatedProperty::Transform(transform_prop));
        }
    }

    properties
}

/// Parse CSS property values to extract from/to values
fn parse_css_property_values(value: &str) -> (String, String) {
    // Handle different value formats:
    // "10px" -> ("0px", "10px") - single value assumes from 0
    // "10px to 20px" -> ("10px", "20px") - explicit from/to
    // "from 5px to 15px" -> ("5px", "15px") - explicit from/to with keywords

    if value.contains(" to ") {
        let parts: Vec<&str> = value.split(" to ").collect();
        if parts.len() == 2 {
            let from_part = parts[0].trim();
            let to_part = parts[1].trim();

            // Handle "from X to Y" format
            let from_value = if from_part.starts_with("from ") {
                from_part.strip_prefix("from ").unwrap_or(from_part).trim()
            } else {
                from_part
            };

            return (from_value.to_string(), to_part.to_string());
        }
    }

    // Single value - assume starting from a reasonable default
    let default_from = match value {
        v if v.contains("px") || v.contains("%") || v.contains("em") => "0px",
        v if v.contains("deg") => "0deg",
        v if v.parse::<f32>().is_ok() => "0",
        _ => "initial",
    };

    (default_from.to_string(), value.to_string())
}

/// Convert PropertyValue to AnimatedProperty
fn convert_property_value_to_animated(name: &str, value: &PropertyValue) -> AnimatedProperty {
    match value {
        PropertyValue::Single(to) => {
            // Determine the appropriate property type based on name
            match name {
                "opacity" => AnimatedProperty::Opacity(0.0, *to), // Will be overridden by current value
                _ => AnimatedProperty::Property(name.to_string(), 0.0, *to),
            }
        }
        PropertyValue::FromTo { from, to } => match name {
            "opacity" => AnimatedProperty::Opacity(*from, *to),
            _ => AnimatedProperty::Property(name.to_string(), *from, *to),
        },
        PropertyValue::Array(values) if values.len() > 2 => numeric_keyframes(name, values),
        PropertyValue::Array(values) => {
            // Two explicit endpoints retain their existing representation.
            if values.len() >= 2 {
                match name {
                    "opacity" => AnimatedProperty::Opacity(values[0], values[values.len() - 1]),
                    _ => AnimatedProperty::Property(
                        name.to_string(),
                        values[0],
                        values[values.len() - 1],
                    ),
                }
            } else {
                AnimatedProperty::Property(
                    name.to_string(),
                    0.0,
                    values.first().copied().unwrap_or(0.0),
                )
            }
        }
        PropertyValue::Relative(_) => {
            unreachable!("relative values require checked target resolution")
        }
    }
}

fn numeric_keyframes(name: &str, values: &[f32]) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::KeyframeSequence {
        keyframes: values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                keyframes::Keyframe::new(index as f32 / (values.len() - 1) as f32)
                    .set_property(name, keyframes::KeyframeValue::Number(*value))
            })
            .collect(),
        duration: Duration::from_secs(1),
        default_easing: EasingFunction::Linear,
    })
}

fn convert_transform_value(name: &str, value: &PropertyValue) -> AnimatedProperty {
    if let PropertyValue::Array(values) = value {
        if values.len() > 2 {
            return numeric_keyframes(name, values);
        }
    }
    AnimatedProperty::Transform(convert_property_to_transform(name, value))
}

/// Convert PropertyValue to TransformProperty
fn convert_property_to_transform(transform_type: &str, value: &PropertyValue) -> TransformProperty {
    match value {
        PropertyValue::Single(to) => match transform_type {
            "translateX" => TransformProperty::TranslateX(0.0, *to),
            "translateY" => TransformProperty::TranslateY(0.0, *to),
            "scale" => TransformProperty::Scale(1.0, *to),
            "rotate" => TransformProperty::Rotate(0.0, to.to_radians()),
            _ => TransformProperty::TranslateX(0.0, *to),
        },
        PropertyValue::FromTo { from, to } => match transform_type {
            "translateX" => TransformProperty::TranslateX(*from, *to),
            "translateY" => TransformProperty::TranslateY(*from, *to),
            "scale" => TransformProperty::Scale(*from, *to),
            "rotate" => TransformProperty::Rotate(from.to_radians(), to.to_radians()),
            _ => TransformProperty::TranslateX(*from, *to),
        },
        PropertyValue::Array(values) => {
            if values.len() >= 2 {
                let from = values[0];
                let to = values[values.len() - 1];
                match transform_type {
                    "translateX" => TransformProperty::TranslateX(from, to),
                    "translateY" => TransformProperty::TranslateY(from, to),
                    "scale" => TransformProperty::Scale(from, to),
                    "rotate" => TransformProperty::Rotate(from.to_radians(), to.to_radians()),
                    _ => TransformProperty::TranslateX(from, to),
                }
            } else {
                TransformProperty::TranslateX(0.0, values.first().copied().unwrap_or(0.0))
            }
        }
        PropertyValue::Relative(_) => {
            unreachable!("relative transforms require checked target resolution")
        }
    }
}

/// Convert ColorValue to AnimatedProperty
fn convert_color_value_to_animated(color: &ColorValue) -> AnimatedProperty {
    match color {
        ColorValue::Rgb(r, g, b) => {
            let to_color = (*r, *g, *b);
            let from_color = (0, 0, 0); // Will be overridden
            AnimatedProperty::Color(from_color, to_color)
        }
        ColorValue::Rgba(r, g, b, a) => {
            // Handle RGBA with alpha channel - convert to premultiplied RGB
            let alpha = (*a) as f32 / 255.0;
            let premult_r = ((*r as f32) * alpha) as u8;
            let premult_g = ((*g as f32) * alpha) as u8;
            let premult_b = ((*b as f32) * alpha) as u8;

            let to_color = (premult_r, premult_g, premult_b);
            let from_color = (0, 0, 0); // Will be overridden
            AnimatedProperty::Color(from_color, to_color)
        }
        ColorValue::FromTo { from, to } => {
            let from_color = (from.0, from.1, from.2);
            let to_color = (to.0, to.1, to.2);
            AnimatedProperty::Color(from_color, to_color)
        }
    }
}

/// Convert SizeValue to AnimatedProperty
fn convert_size_value_to_animated(size: &SizeValue) -> AnimatedProperty {
    match size {
        SizeValue::Both(w, h) => {
            AnimatedProperty::Size(0, 0, *w, *h) // Will be overridden by current size
        }
        SizeValue::FromTo { from, to } => AnimatedProperty::Size(from.0, from.1, to.0, to.1),
    }
}

/// Convert PositionValue to AnimatedProperty
fn convert_position_value_to_animated(position: &PositionValue) -> AnimatedProperty {
    match position {
        PositionValue::Both(x, y) => {
            AnimatedProperty::Position(0, 0, *x, *y) // Will be overridden by current position
        }
        PositionValue::FromTo { from, to } => {
            AnimatedProperty::Position(from.0, from.1, to.0, to.1)
        }
    }
}

/// Modern stagger function for creating staggered delays
///
/// # Examples
///
/// ```rust, ignore
/// use reactive_tui::animation::*;
///
/// // Basic stagger with 100ms delay
/// let stagger_config = stagger_delay(100.0, None);
///
/// // Stagger from center with easing
/// let center_stagger = stagger_delay(150.0, Some(StaggerOptions {
///     from: StaggerOrigin::Center,
///     easing: Some(EasingFunction::EaseOut),
///     ..Default::default()
/// }));
/// ```
pub fn stagger_delay(delay_ms: f32, options: Option<StaggerOptions>) -> stagger::StaggerConfig {
    let opts = options.unwrap_or_default();

    stagger::StaggerConfig {
        delay: Duration::from_millis(delay_ms as u64),
        from: opts.from,
        direction: opts.direction,
        ease: opts.easing,
        grid: opts.grid,
        range: opts.range,
    }
}

/// Options for stagger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaggerOptions {
    /// Origin point for stagger calculation
    pub from: stagger::StaggerOrigin,
    /// Direction of stagger
    pub direction: stagger::StaggerDirection,
    /// Easing function for stagger delay calculation
    pub easing: Option<EasingFunction>,
    /// Grid dimensions for 2D stagger
    pub grid: Option<(usize, usize)>,
    /// Range modifier for stagger delays
    pub range: Option<(f32, f32)>,
}

impl Default for StaggerOptions {
    fn default() -> Self {
        Self {
            from: stagger::StaggerOrigin::First,
            direction: stagger::StaggerDirection::Normal,
            easing: None,
            grid: None,
            range: None,
        }
    }
}

/// Parameters for timeline creation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimelineParams {
    /// Timeline ID
    pub id: Option<String>,
    /// Total timeline duration
    pub duration: Option<Duration>,
    /// Loop mode for the entire timeline
    pub loop_mode: Option<LoopMode>,
    /// Auto-play the timeline
    pub autoplay: Option<bool>,
}

/// Timeline builder for creating complex animation sequences
pub struct TimelineBuilder {
    timeline: AnimationTimeline,
    current_time: Duration,
    labels: std::collections::HashMap<String, Duration>,
    loop_mode: Option<LoopMode>,
}

impl TimelineBuilder {
    /// Create a new timeline builder
    pub fn new(id: String) -> Self {
        Self {
            timeline: AnimationTimeline::new(id, false), // non-sequential by default
            current_time: Duration::ZERO,
            labels: std::collections::HashMap::new(),
            loop_mode: None,
        }
    }

    /// Add an animation to the timeline
    pub fn add<T>(mut self, targets: T, params: AnimateParams, position: Option<&str>) -> Self
    where
        T: Into<AnimationTargetInput>,
    {
        let animation = animate(targets, params);

        let timeline_position = position.map(parse_timeline_position).unwrap_or_else(|| {
            // Default: start after previous animation
            let pos = self.current_time;
            self.current_time += animation.config.duration;
            pos
        });

        // Implement precise timeline positioning by adjusting animation delay
        let mut positioned_animation = animation;
        positioned_animation.config.delay = timeline_position;

        // Update current time to track timeline length
        let animation_end = timeline_position + positioned_animation.config.duration;
        if animation_end > self.current_time {
            self.current_time = animation_end;
        }

        self.timeline.add_animation(positioned_animation);
        self
    }

    /// Add a label at the current timeline position
    pub fn add_label(mut self, name: &str) -> Self {
        // Store timeline labels for seeking and synchronization
        self.labels.insert(name.to_string(), self.current_time);
        self
    }

    /// Set timeline loop mode
    pub fn loop_mode(mut self, loop_mode: LoopMode) -> Self {
        // Set how the timeline should repeat
        self.loop_mode = Some(loop_mode);

        // For infinite loops, ensure we have a proper duration
        if matches!(loop_mode, LoopMode::Infinite) && self.current_time.as_secs_f64() == 0.0 {
            self.current_time = Duration::from_secs(1); // Default 1 second loop
        }

        self
    }

    /// Build the final timeline
    pub fn build(self) -> AnimationTimeline {
        self.timeline
    }
}

/// Parse timeline position strings (e.g., "-=500", "+=200", "50%", "1.5s")
fn parse_timeline_position(position: &str) -> Duration {
    if let Some(stripped) = position.strip_prefix("-=") {
        // Relative to previous animation end, subtract time
        let ms: f32 = stripped.parse().unwrap_or(0.0);
        Duration::from_millis(ms as u64) // This would need proper context
    } else if let Some(stripped) = position.strip_prefix("+=") {
        // Relative to previous animation end, add time
        let ms: f32 = stripped.parse().unwrap_or(0.0);
        Duration::from_millis(ms as u64)
    } else if let Some(stripped) = position.strip_suffix('%') {
        // Percentage of timeline
        let percent: f32 = stripped.parse().unwrap_or(0.0);
        Duration::from_millis((percent * 10.0) as u64) // Simplified
    } else if let Some(stripped) = position.strip_suffix('s') {
        // Seconds
        let secs: f32 = stripped.parse().unwrap_or(0.0);
        Duration::from_secs_f32(secs)
    } else {
        // Assume milliseconds
        let ms: f32 = position.parse().unwrap_or(0.0);
        Duration::from_millis(ms as u64)
    }
}

/// Create a timeline with optional parameters
///
/// # Examples
///
/// ```rust, ignore
/// use reactive_tui::animation::*;
///
/// let timeline = create_timeline(Some(TimelineParams {
///     id: Some("main-timeline".to_string()),
///     autoplay: Some(true),
///     ..Default::default()
/// }))
/// .add("element1", AnimateParams {
///     opacity: Some(PropertyValue::FromTo { from: 0.0, to: 1.0 }),
///     duration: Some(500.0),
///     ..Default::default()
/// }, None)
/// .add("element2", AnimateParams {
///     translate_x: Some(PropertyValue::FromTo{from:0.0,to:100.0}),
///     duration: Some(300.0),
///     ..Default::default()
/// }, Some("-=200")) // Start 200ms before previous ends
/// .build();
/// ```
pub fn create_timeline(params: Option<TimelineParams>) -> TimelineBuilder {
    let params = params.unwrap_or_default();
    let id = params.id.unwrap_or_else(generate_id);

    let mut builder = TimelineBuilder::new(id);

    if let Some(loop_mode) = params.loop_mode {
        builder = builder.loop_mode(loop_mode);
    }

    builder
}

// Convenience functions for common animations

/// Create a fade in animation
pub fn fade_in<T>(targets: T, duration_ms: f32) -> Animation
where
    T: Into<AnimationTargetInput>,
{
    animate(
        targets,
        AnimateParams {
            opacity: Some(PropertyValue::FromTo { from: 0.0, to: 1.0 }),
            duration: Some(duration_ms),
            easing: Some(EasingFunction::EaseOut),
            ..Default::default()
        },
    )
}

/// Create a fade out animation
pub fn fade_out<T>(targets: T, duration_ms: f32) -> Animation
where
    T: Into<AnimationTargetInput>,
{
    animate(
        targets,
        AnimateParams {
            opacity: Some(PropertyValue::FromTo { from: 1.0, to: 0.0 }),
            duration: Some(duration_ms),
            easing: Some(EasingFunction::EaseIn),
            ..Default::default()
        },
    )
}

/// Create a slide animation
pub fn slide<T>(targets: T, x: f32, y: f32, duration_ms: f32) -> Animation
where
    T: Into<AnimationTargetInput>,
{
    animate(
        targets,
        AnimateParams {
            translate_x: Some(PropertyValue::Single(x)),
            translate_y: Some(PropertyValue::Single(y)),
            duration: Some(duration_ms),
            easing: Some(EasingFunction::EaseInOut),
            ..Default::default()
        },
    )
}

/// Create a scale animation
pub fn scale<T>(targets: T, scale_factor: f32, duration_ms: f32) -> Animation
where
    T: Into<AnimationTargetInput>,
{
    animate(
        targets,
        AnimateParams {
            scale: Some(PropertyValue::FromTo {
                from: 1.0,
                to: scale_factor,
            }),
            duration: Some(duration_ms),
            easing: Some(EasingFunction::EaseInOut),
            ..Default::default()
        },
    )
}

/// Create a spring animation
pub fn spring_animate<T>(
    targets: T,
    property: &str,
    to_value: f32,
    spring_config: spring::SpringConfig,
) -> Animation
where
    T: Into<AnimationTargetInput>,
{
    let mut custom = HashMap::new();
    custom.insert(property.to_string(), PropertyValue::Single(to_value));

    animate(
        targets,
        AnimateParams {
            custom: Some(custom),
            easing: Some(EasingFunction::Spring(spring_config)),
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> (crate::animation::TargetRegistry, AnimationTarget) {
        let mut owner =
            crate::animation::TargetRegistry::new(crate::reactive::wake::AppWaker::new());
        owner
            .publish(&crate::builder::core::div().id("element").build(), None, 0)
            .unwrap();
        let target = owner.context().target("element").unwrap();
        (owner, target)
    }

    #[test]
    fn test_animate_function() {
        let animation = animate(
            "test-element",
            AnimateParams {
                opacity: Some(PropertyValue::FromTo { from: 0.0, to: 1.0 }),
                duration: Some(500.0),
                ..Default::default()
            },
        );

        assert_eq!(animation.config.duration, Duration::from_millis(500));
        assert!(animation.is_playing());
    }

    #[test]
    fn test_animation_targets() {
        let single: AnimationTargets = "element".into();
        let multiple: AnimationTargets = vec!["el1", "el2", "el3"].into();

        match single {
            AnimationTargets::Single(id) => assert_eq!(id, "element"),
            AnimationTargets::Multiple(_) => panic!("Expected single target, got multiple"),
        }

        match multiple {
            AnimationTargets::Multiple(ids) => assert_eq!(ids.len(), 3),
            AnimationTargets::Single(_) => panic!("Expected multiple targets, got single"),
        }
    }

    #[test]
    fn test_property_value_conversion() {
        let opacity_prop = convert_property_value_to_animated(
            "opacity",
            &PropertyValue::FromTo { from: 0.0, to: 1.0 },
        );

        match opacity_prop {
            AnimatedProperty::Opacity(from, to) => {
                assert_eq!(from, 0.0);
                assert_eq!(to, 1.0);
            }
            other => panic!("Expected opacity property, got: {:?}", other),
        }
    }

    #[test]
    fn test_stagger_delay() {
        let stagger_config = stagger_delay(
            100.0,
            Some(StaggerOptions {
                from: stagger::StaggerOrigin::Center,
                ..Default::default()
            }),
        );

        assert_eq!(stagger_config.delay, Duration::from_millis(100));
        assert_eq!(stagger_config.from, stagger::StaggerOrigin::Center);
    }

    #[test]
    fn test_timeline_creation() {
        let timeline = create_timeline(Some(TimelineParams {
            id: Some("test-timeline".to_string()),
            ..Default::default()
        }))
        .add(
            "element1",
            AnimateParams {
                opacity: Some(PropertyValue::FromTo { from: 0.0, to: 1.0 }),
                duration: Some(500.0),
                ..Default::default()
            },
            None,
        )
        .add_label("middle")
        .add(
            "element2",
            AnimateParams {
                translate_x: Some(PropertyValue::FromTo {
                    from: 0.0,
                    to: 100.0,
                }),
                duration: Some(300.0),
                ..Default::default()
            },
            None,
        )
        .build();

        assert_eq!(timeline.id, "test-timeline");
        assert!(!timeline.animations.is_empty());
    }

    #[test]
    fn test_convenience_functions() {
        let (_owner, target) = target();
        let fade = fade_in("element", 500.0);
        assert_eq!(fade.config.duration, Duration::from_millis(500));

        let slide_anim = slide(&target, 100.0, 50.0, 750.0);
        assert_eq!(slide_anim.config.duration, Duration::from_millis(750));

        let scale_anim = scale("element", 1.5, 400.0);
        assert_eq!(scale_anim.config.duration, Duration::from_millis(400));
    }

    #[test]
    fn test_timeline_position_parsing() {
        assert_eq!(parse_timeline_position("500"), Duration::from_millis(500));
        assert_eq!(parse_timeline_position("1.5s"), Duration::from_millis(1500));
        assert_eq!(parse_timeline_position("+=200"), Duration::from_millis(200));
        assert_eq!(parse_timeline_position("-=100"), Duration::from_millis(100));
    }

    #[test]
    fn test_complex_animation() {
        let complex = animate(
            "element",
            AnimateParams {
                translate_x: Some(PropertyValue::Array(vec![0.0, 50.0, 100.0])),
                scale: Some(PropertyValue::FromTo { from: 0.8, to: 1.2 }),
                color: Some(ColorValue::FromTo {
                    from: (255, 0, 0),
                    to: (0, 255, 0),
                }),
                duration: Some(1000.0),
                easing: Some(EasingFunction::Spring(SpringConfig::bouncy())),
                ..Default::default()
            },
        );

        assert_eq!(complex.config.duration, Duration::from_millis(1000));
        assert!(complex.is_playing());
    }

    #[test]
    fn test_spring_animation() {
        let (_owner, target) = target();
        let spring_anim =
            spring_animate(&target, "translateX", 100.0, spring::SpringConfig::bouncy());
        assert!(spring_anim.is_playing());
    }
}
