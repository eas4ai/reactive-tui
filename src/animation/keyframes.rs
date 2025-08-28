//! Keyframe sequence system for multi-property animations
//!
//! Provides anime.js-inspired keyframe animations with multi-property interpolation,
//! timeline support, and advanced sequencing capabilities.

use super::{AnimationValue, CssValue, EasingFunction, TransformMatrix};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Animation built from keyframes with generic value type
pub struct KeyframeAnimation<T: Clone> {
    keyframes: Vec<TypedKeyframe<T>>,
    duration: Duration,
}

/// Keyframe with typed value
pub struct TypedKeyframe<T: Clone> {
    pub offset: f32,
    pub value: T,
    pub easing: Option<EasingFunction>,
}

impl<T: Clone> KeyframeAnimation<T> {
    pub fn new(keyframes: Vec<Keyframe>) -> Self
    where
        T: Default,
    {
        // Convert untyped keyframes to typed ones
        let typed_keyframes = keyframes
            .into_iter()
            .map(|kf| TypedKeyframe {
                offset: kf.offset,
                value: T::default(),
                easing: kf.easing,
            })
            .collect();

        Self {
            keyframes: typed_keyframes,
            duration: Duration::from_secs(1),
        }
    }

    pub fn from_typed(keyframes: Vec<TypedKeyframe<T>>, duration: Duration) -> Self {
        Self {
            keyframes,
            duration,
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn get_value_at_time(&self, progress: f32) -> Option<T> {
        if self.keyframes.is_empty() {
            return None;
        }

        let progress = progress.clamp(0.0, 1.0);

        // Find surrounding keyframes
        let mut prev_kf = None;
        let mut next_kf = None;

        for kf in &self.keyframes {
            if kf.offset <= progress {
                prev_kf = Some(kf);
            } else if next_kf.is_none() {
                next_kf = Some(kf);
                break;
            }
        }

        match (prev_kf, next_kf) {
            (Some(prev), Some(_next)) => {
                // For now, return the previous keyframe's value
                // Full interpolation would require T to implement an interpolation trait
                Some(prev.value.clone())
            }
            (Some(prev), None) => Some(prev.value.clone()),
            (None, Some(next)) => Some(next.value.clone()),
            (None, None) => None,
        }
    }
}

/// A single keyframe defining property values at a specific time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Time offset as percentage (0.0 to 1.0) of total animation duration
    pub offset: f32,
    /// Property values at this keyframe
    pub properties: HashMap<String, KeyframeValue>,
    /// Easing function to use when transitioning TO this keyframe
    pub easing: Option<EasingFunction>,
}

impl Keyframe {
    /// Create a new keyframe at the specified offset
    pub fn new(offset: f32) -> Self {
        Self {
            offset: offset.clamp(0.0, 1.0),
            properties: HashMap::new(),
            easing: None,
        }
    }

    /// Set a property value for this keyframe
    pub fn set_property(mut self, name: &str, value: KeyframeValue) -> Self {
        self.properties.insert(name.to_string(), value);
        self
    }

    /// Set easing function for transitioning to this keyframe
    pub fn with_easing(mut self, easing: EasingFunction) -> Self {
        self.easing = Some(easing);
        self
    }

    /// Set opacity value
    pub fn opacity(mut self, value: f32) -> Self {
        self.properties.insert(
            "opacity".to_string(),
            KeyframeValue::Number(value.clamp(0.0, 1.0)),
        );
        self
    }

    /// Set transform matrix
    pub fn transform(mut self, matrix: TransformMatrix) -> Self {
        self.properties
            .insert("transform".to_string(), KeyframeValue::Transform(matrix));
        self
    }

    /// Set color value
    pub fn color(mut self, r: u8, g: u8, b: u8, a: Option<u8>) -> Self {
        self.properties.insert(
            "color".to_string(),
            KeyframeValue::Color(r, g, b, a.unwrap_or(255)),
        );
        self
    }

    /// Set CSS unit value
    pub fn css_value(mut self, property: &str, value: CssValue) -> Self {
        self.properties
            .insert(property.to_string(), KeyframeValue::Css(value));
        self
    }

    /// Set numeric value
    pub fn number(mut self, property: &str, value: f32) -> Self {
        self.properties
            .insert(property.to_string(), KeyframeValue::Number(value));
        self
    }
}

/// Value type for keyframe properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyframeValue {
    /// Numeric value
    Number(f32),
    /// Color value (r, g, b, a)
    Color(u8, u8, u8, u8),
    /// Transform matrix
    Transform(TransformMatrix),
    /// CSS value with units
    Css(CssValue),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Multiple values (for complex properties)
    Multiple(Vec<KeyframeValue>),
}

impl KeyframeValue {
    /// Interpolate between two keyframe values
    pub fn interpolate(&self, other: &KeyframeValue, t: f32) -> Option<KeyframeValue> {
        match (self, other) {
            (KeyframeValue::Number(from), KeyframeValue::Number(to)) => {
                Some(KeyframeValue::Number(from + (to - from) * t))
            }
            (KeyframeValue::Color(r1, g1, b1, a1), KeyframeValue::Color(r2, g2, b2, a2)) => {
                Some(KeyframeValue::Color(
                    (*r1 as f32 + (*r2 as f32 - *r1 as f32) * t) as u8,
                    (*g1 as f32 + (*g2 as f32 - *g1 as f32) * t) as u8,
                    (*b1 as f32 + (*b2 as f32 - *b1 as f32) * t) as u8,
                    (*a1 as f32 + (*a2 as f32 - *a1 as f32) * t) as u8,
                ))
            }
            (KeyframeValue::Transform(from), KeyframeValue::Transform(to)) => {
                Some(KeyframeValue::Transform(TransformMatrix {
                    a: from.a + (to.a - from.a) * t,
                    b: from.b + (to.b - from.b) * t,
                    c: from.c + (to.c - from.c) * t,
                    d: from.d + (to.d - from.d) * t,
                    e: from.e + (to.e - from.e) * t,
                    f: from.f + (to.f - from.f) * t,
                }))
            }
            (KeyframeValue::Css(from), KeyframeValue::Css(to)) => {
                Self::interpolate_css_values(from, to, t).map(KeyframeValue::Css)
            }
            (KeyframeValue::Multiple(from), KeyframeValue::Multiple(to))
                if from.len() == to.len() =>
            {
                let interpolated: Option<Vec<KeyframeValue>> = from
                    .iter()
                    .zip(to.iter())
                    .map(|(f, t_val)| f.interpolate(t_val, t))
                    .collect();
                interpolated.map(KeyframeValue::Multiple)
            }
            // Cannot interpolate between different types or strings/booleans
            _ => None,
        }
    }

    /// Interpolate between CSS values
    fn interpolate_css_values(from: &CssValue, to: &CssValue, t: f32) -> Option<CssValue> {
        match (from, to) {
            (CssValue::Number(f), CssValue::Number(t_val)) => {
                Some(CssValue::Number(f + (t_val - f) * t))
            }
            (CssValue::Percentage(f), CssValue::Percentage(t_val)) => {
                Some(CssValue::Percentage(f + (t_val - f) * t))
            }
            (CssValue::Pixels(f), CssValue::Pixels(t_val)) => {
                Some(CssValue::Pixels(f + (t_val - f) * t))
            }
            (CssValue::Em(f), CssValue::Em(t_val)) => Some(CssValue::Em(f + (t_val - f) * t)),
            (CssValue::Rem(f), CssValue::Rem(t_val)) => Some(CssValue::Rem(f + (t_val - f) * t)),
            (CssValue::ViewportWidth(f), CssValue::ViewportWidth(t_val)) => {
                Some(CssValue::ViewportWidth(f + (t_val - f) * t))
            }
            (CssValue::ViewportHeight(f), CssValue::ViewportHeight(t_val)) => {
                Some(CssValue::ViewportHeight(f + (t_val - f) * t))
            }
            (
                CssValue::Color {
                    r: fr,
                    g: fg,
                    b: fb,
                },
                CssValue::Color {
                    r: tr,
                    g: tg,
                    b: tb,
                },
            ) => {
                // Interpolate between colors
                let r = (*fr as f32 + (*tr as f32 - *fr as f32) * t) as u8;
                let g = (*fg as f32 + (*tg as f32 - *fg as f32) * t) as u8;
                let b = (*fb as f32 + (*tb as f32 - *fb as f32) * t) as u8;
                Some(CssValue::Color { r, g, b })
            }
            // Cannot interpolate between different units or non-numeric types like strings
            _ => None,
        }
    }

    /// Convert to AnimationValue for compatibility with existing system
    pub fn to_animation_value(&self) -> AnimationValue {
        match self {
            KeyframeValue::Number(n) => AnimationValue::Number(*n),
            KeyframeValue::Color(r, g, b, _a) => AnimationValue::Color {
                r: *r,
                g: *g,
                b: *b,
            },
            KeyframeValue::Transform(matrix) => AnimationValue::Transform(matrix.clone()),
            KeyframeValue::Css(css) => match css {
                CssValue::Number(n) => AnimationValue::Number(*n),
                CssValue::Percentage(p) => AnimationValue::Unit(*p, "%".to_string()),
                CssValue::Pixels(px) => AnimationValue::Unit(*px, "px".to_string()),
                CssValue::Em(em) => AnimationValue::Unit(*em, "em".to_string()),
                CssValue::Rem(rem) => AnimationValue::Unit(*rem, "rem".to_string()),
                CssValue::ViewportWidth(vw) => AnimationValue::Unit(*vw, "vw".to_string()),
                CssValue::ViewportHeight(vh) => AnimationValue::Unit(*vh, "vh".to_string()),
                CssValue::Color { r, g, b } => AnimationValue::Color {
                    r: *r,
                    g: *g,
                    b: *b,
                },
                CssValue::String(s) => AnimationValue::String(s.clone()),
            },
            KeyframeValue::String(s) => AnimationValue::String(s.clone()),
            KeyframeValue::Boolean(b) => AnimationValue::Boolean(*b),
            KeyframeValue::Multiple(values) => {
                let animation_values: Vec<AnimationValue> =
                    values.iter().map(|v| v.to_animation_value()).collect();
                AnimationValue::Multiple(animation_values)
            }
        }
    }
}

/// A sequence of keyframes defining a complex animation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframeSequence {
    /// List of keyframes ordered by offset
    pub keyframes: Vec<Keyframe>,
    /// Total duration of the sequence
    pub duration: Duration,
    /// Default easing function (used when keyframes don't specify easing)
    pub default_easing: EasingFunction,
}

impl KeyframeSequence {
    /// Create a new keyframe sequence with the specified duration
    pub fn new(duration: Duration) -> Self {
        Self {
            keyframes: Vec::new(),
            duration,
            default_easing: EasingFunction::EaseInOut,
        }
    }

    /// Add a keyframe to the sequence
    pub fn add_keyframe(mut self, keyframe: Keyframe) -> Self {
        self.keyframes.push(keyframe);
        // Keep keyframes sorted by offset
        self.keyframes.sort_by(|a, b| {
            a.offset
                .partial_cmp(&b.offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self
    }

    /// Set default easing function
    pub fn default_easing(mut self, easing: EasingFunction) -> Self {
        self.default_easing = easing;
        self
    }

    /// Add a keyframe at the specified offset with a builder pattern
    pub fn at(self, offset: f32) -> KeyframeBuilder {
        KeyframeBuilder::new(self, offset)
    }

    /// Sample the sequence at a normalized time (0.0 to 1.0)
    pub fn sample(&self, t: f32) -> HashMap<String, KeyframeValue> {
        let t = t.clamp(0.0, 1.0);
        let mut result = HashMap::new();

        if self.keyframes.is_empty() {
            return result;
        }

        // Find the two keyframes we're interpolating between
        let (from_keyframe, to_keyframe) = self.find_keyframe_pair(t);

        // Collect all property names that appear in any keyframe
        let mut all_properties = std::collections::HashSet::new();
        for keyframe in &self.keyframes {
            all_properties.extend(keyframe.properties.keys());
        }

        // Interpolate each property
        for property in all_properties {
            if let Some(interpolated_value) =
                self.interpolate_property(property, &from_keyframe, &to_keyframe, t)
            {
                result.insert(property.clone(), interpolated_value);
            }
        }

        result
    }

    /// Find the keyframe pair to interpolate between for the given time
    fn find_keyframe_pair(&self, t: f32) -> (Option<&Keyframe>, Option<&Keyframe>) {
        if self.keyframes.is_empty() {
            return (None, None);
        }

        // Find the keyframes before and after the current time
        let mut from_keyframe = None;
        let mut to_keyframe = None;

        for keyframe in &self.keyframes {
            if keyframe.offset <= t {
                from_keyframe = Some(keyframe);
            }
            if keyframe.offset > t && to_keyframe.is_none() {
                to_keyframe = Some(keyframe);
                break;
            }
        }

        // If we're past the last keyframe, use the last keyframe as both
        if to_keyframe.is_none() && from_keyframe.is_some() {
            to_keyframe = from_keyframe;
        }

        // If we're before the first keyframe, use the first keyframe as both
        if from_keyframe.is_none() && !self.keyframes.is_empty() {
            from_keyframe = Some(&self.keyframes[0]);
            to_keyframe = Some(&self.keyframes[0]);
        }

        (from_keyframe, to_keyframe)
    }

    /// Interpolate a specific property between two keyframes
    fn interpolate_property(
        &self,
        property: &str,
        from_keyframe: &Option<&Keyframe>,
        to_keyframe: &Option<&Keyframe>,
        global_t: f32,
    ) -> Option<KeyframeValue> {
        match (from_keyframe, to_keyframe) {
            (Some(from), Some(to)) if from.offset != to.offset => {
                // Get values from both keyframes
                let from_value = from.properties.get(property);
                let to_value = to.properties.get(property);

                match (from_value, to_value) {
                    (Some(from_val), Some(to_val)) => {
                        // Calculate local interpolation factor between keyframes
                        let local_t = (global_t - from.offset) / (to.offset - from.offset);
                        let local_t = local_t.clamp(0.0, 1.0);

                        // Apply easing
                        let easing = to.easing.as_ref().unwrap_or(&self.default_easing);
                        let eased_t = easing.apply(local_t);

                        from_val.interpolate(to_val, eased_t)
                    }
                    (Some(value), None) => Some(value.clone()),
                    (None, Some(value)) => Some(value.clone()),
                    (None, None) => None,
                }
            }
            (Some(keyframe), _) => {
                // Single keyframe or at exact keyframe time
                keyframe.properties.get(property).cloned()
            }
            _ => None,
        }
    }

    /// Get all property names used in this sequence
    pub fn get_property_names(&self) -> Vec<String> {
        let mut properties = std::collections::HashSet::new();
        for keyframe in &self.keyframes {
            properties.extend(keyframe.properties.keys());
        }
        properties.into_iter().cloned().collect()
    }

    /// Validate the sequence (ensure keyframes are properly ordered and have valid offsets)
    pub fn validate(&self) -> Result<(), String> {
        if self.keyframes.is_empty() {
            return Err("Keyframe sequence must have at least one keyframe".to_string());
        }

        for keyframe in &self.keyframes {
            if keyframe.offset < 0.0 || keyframe.offset > 1.0 {
                return Err(format!(
                    "Keyframe offset {} is out of range [0.0, 1.0]",
                    keyframe.offset
                ));
            }
        }

        // Check that keyframes are sorted
        for i in 1..self.keyframes.len() {
            if self.keyframes[i - 1].offset > self.keyframes[i].offset {
                return Err("Keyframes are not sorted by offset".to_string());
            }
        }

        Ok(())
    }
}

/// Builder for creating keyframes within a sequence
pub struct KeyframeBuilder {
    sequence: KeyframeSequence,
    keyframe: Keyframe,
}

impl KeyframeBuilder {
    fn new(sequence: KeyframeSequence, offset: f32) -> Self {
        Self {
            sequence,
            keyframe: Keyframe::new(offset),
        }
    }

    /// Set opacity for this keyframe
    pub fn opacity(mut self, value: f32) -> Self {
        self.keyframe = self.keyframe.opacity(value);
        self
    }

    /// Set transform for this keyframe
    pub fn transform(mut self, matrix: TransformMatrix) -> Self {
        self.keyframe = self.keyframe.transform(matrix);
        self
    }

    /// Set color for this keyframe
    pub fn color(mut self, r: u8, g: u8, b: u8, a: Option<u8>) -> Self {
        self.keyframe = self.keyframe.color(r, g, b, a);
        self
    }

    /// Set CSS value for this keyframe
    pub fn css_value(mut self, property: &str, value: CssValue) -> Self {
        self.keyframe = self.keyframe.css_value(property, value);
        self
    }

    /// Set numeric value for this keyframe
    pub fn number(mut self, property: &str, value: f32) -> Self {
        self.keyframe = self.keyframe.number(property, value);
        self
    }

    /// Set easing for this keyframe
    pub fn easing(mut self, easing: EasingFunction) -> Self {
        self.keyframe = self.keyframe.with_easing(easing);
        self
    }

    /// Finish building this keyframe and return the sequence for further building
    pub fn finish(mut self) -> KeyframeSequence {
        self.sequence.keyframes.push(self.keyframe);
        // Keep keyframes sorted
        self.sequence.keyframes.sort_by(|a, b| {
            a.offset
                .partial_cmp(&b.offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.sequence
    }
}

// Convenience functions for creating keyframe sequences

/// Create a new keyframe sequence with the specified duration
pub fn keyframes(duration_ms: u64) -> KeyframeSequence {
    KeyframeSequence::new(Duration::from_millis(duration_ms))
}

/// Create a simple fade in keyframe sequence
pub fn fade_in(duration_ms: u64) -> KeyframeSequence {
    keyframes(duration_ms)
        .at(0.0)
        .opacity(0.0)
        .finish()
        .at(1.0)
        .opacity(1.0)
        .finish()
}

/// Create a simple fade out keyframe sequence
pub fn fade_out(duration_ms: u64) -> KeyframeSequence {
    keyframes(duration_ms)
        .at(0.0)
        .opacity(1.0)
        .finish()
        .at(1.0)
        .opacity(0.0)
        .finish()
}

/// Create a simple slide in keyframe sequence
pub fn slide_in_from_left(duration_ms: u64, distance: f32) -> KeyframeSequence {
    let start_transform = TransformMatrix {
        e: -distance,
        ..Default::default()
    };

    let end_transform = TransformMatrix::default();

    keyframes(duration_ms)
        .at(0.0)
        .transform(start_transform)
        .finish()
        .at(1.0)
        .transform(end_transform)
        .finish()
}

/// Create a bounce entrance keyframe sequence
pub fn bounce_in(duration_ms: u64) -> KeyframeSequence {
    keyframes(duration_ms)
        .at(0.0)
        .opacity(0.0)
        .css_value("scale", CssValue::Number(0.3))
        .easing(EasingFunction::EaseOut)
        .finish()
        .at(0.5)
        .opacity(1.0)
        .css_value("scale", CssValue::Number(1.05))
        .finish()
        .at(0.7)
        .css_value("scale", CssValue::Number(0.9))
        .finish()
        .at(1.0)
        .opacity(1.0)
        .css_value("scale", CssValue::Number(1.0))
        .easing(EasingFunction::Bounce)
        .finish()
}

/// Create a pulse keyframe sequence
pub fn pulse(duration_ms: u64) -> KeyframeSequence {
    keyframes(duration_ms)
        .at(0.0)
        .css_value("scale", CssValue::Number(1.0))
        .finish()
        .at(0.5)
        .css_value("scale", CssValue::Number(1.1))
        .finish()
        .at(1.0)
        .css_value("scale", CssValue::Number(1.0))
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_creation() {
        let keyframe = Keyframe::new(0.5)
            .opacity(0.8)
            .color(255, 0, 0, Some(255))
            .number("custom", 42.0);

        assert_eq!(keyframe.offset, 0.5);
        assert_eq!(keyframe.properties.len(), 3);
        assert_eq!(
            keyframe.properties.get("opacity"),
            Some(&KeyframeValue::Number(0.8))
        );
    }

    #[test]
    fn test_keyframe_value_interpolation() {
        let from = KeyframeValue::Number(0.0);
        let to = KeyframeValue::Number(100.0);

        let result = from.interpolate(&to, 0.5).unwrap();
        assert_eq!(result, KeyframeValue::Number(50.0));
    }

    #[test]
    fn test_keyframe_sequence_sampling() {
        let sequence = keyframes(1000)
            .at(0.0)
            .opacity(0.0)
            .finish()
            .at(0.5)
            .opacity(0.5)
            .finish()
            .at(1.0)
            .opacity(1.0)
            .finish();

        let sample = sequence.sample(0.25);
        if let Some(KeyframeValue::Number(opacity)) = sample.get("opacity") {
            assert!((opacity - 0.25).abs() < 0.01);
        } else {
            panic!("Expected opacity value");
        }
    }

    #[test]
    fn test_bounce_in_sequence() {
        let sequence = bounce_in(1000);
        assert_eq!(sequence.keyframes.len(), 4);
        assert_eq!(sequence.keyframes[0].offset, 0.0);
        assert_eq!(sequence.keyframes[3].offset, 1.0);
    }

    #[test]
    fn test_sequence_validation() {
        let valid_sequence = keyframes(1000)
            .at(0.0)
            .opacity(0.0)
            .finish()
            .at(1.0)
            .opacity(1.0)
            .finish();

        assert!(valid_sequence.validate().is_ok());

        let empty_sequence = KeyframeSequence::new(Duration::from_millis(1000));
        assert!(empty_sequence.validate().is_err());
    }

    #[test]
    fn test_complex_multi_property_sequence() {
        let sequence = keyframes(2000)
            .at(0.0)
            .opacity(0.0)
            .css_value("scale", CssValue::Number(0.5))
            .color(255, 0, 0, Some(255))
            .finish()
            .at(0.3)
            .opacity(0.7)
            .css_value("scale", CssValue::Number(1.2))
            .color(255, 255, 0, Some(255))
            .easing(EasingFunction::EaseOut)
            .finish()
            .at(1.0)
            .opacity(1.0)
            .css_value("scale", CssValue::Number(1.0))
            .color(0, 255, 0, Some(255))
            .easing(EasingFunction::Bounce)
            .finish();

        // Test sampling at different points
        let sample_start = sequence.sample(0.0);
        let sample_mid = sequence.sample(0.15);
        let sample_end = sequence.sample(1.0);

        assert!(sample_start.contains_key("opacity"));
        assert!(sample_mid.contains_key("opacity"));
        assert!(sample_end.contains_key("opacity"));

        // Verify all properties are present
        let properties = sequence.get_property_names();
        assert!(properties.contains(&"opacity".to_string()));
        assert!(properties.contains(&"scale".to_string()));
        assert!(properties.contains(&"color".to_string()));
    }
}
