//! Animation property types and transformations
//!
//! This module contains types for defining what properties can be animated,
//! including CSS-like properties, transformations, and custom values.

use super::easing::EasingFunction;
use super::keyframes;
use super::state::{AnimatedValue, AnimationValue};
use serde::{Deserialize, Serialize};

/// Properties that can be animated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimatedProperty {
    /// Opacity animation (from, to)
    Opacity(f32, f32),
    /// Position animation (from_x, from_y, to_x, to_y)
    Position(i16, i16, i16, i16),
    /// Size animation (from_width, from_height, to_width, to_height)
    Size(u16, u16, u16, u16),
    /// Color animation (from, to)
    Color((u8, u8, u8), (u8, u8, u8)),
    /// Scale animation (from, to)
    Scale(f32, f32),
    /// Rotation animation in degrees (from, to)
    Rotation(f32, f32),
    /// Custom numeric property (name, from, to)
    Custom(String, f32, f32),
    /// Multiple properties animated together
    Multiple(Vec<AnimatedProperty>),

    // New anime.js inspired property types
    /// Animate any numeric property by name
    Property(String, f32, f32),
    /// Transform properties (translateX, translateY, rotate, scaleX, scaleY)
    Transform(TransformProperty),
    /// CSS-like properties with unit handling
    CssProperty(String, CssValue, CssValue),
    /// Multiple properties with individual timing
    PropertySet(Vec<PropertyAnimation>),
    /// Keyframe sequence animation with complex multi-property timelines
    Keyframes(keyframes::KeyframeSequence),
}

/// Transform properties for CSS-like animations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransformProperty {
    /// Translate along X-axis from first value to second value
    TranslateX(f32, f32),
    /// Translate along Y-axis from first value to second value
    TranslateY(f32, f32),
    /// Translate in both X and Y directions (x1, y1, x2, y2)
    Translate(f32, f32, f32, f32),
    /// Scale along X-axis from first value to second value
    ScaleX(f32, f32),
    /// Scale along Y-axis from first value to second value
    ScaleY(f32, f32),
    /// Scale uniformly from first value to second value
    Scale(f32, f32),
    /// Rotate from first angle to second angle (in radians)
    Rotate(f32, f32),
    /// Skew along X-axis from first value to second value
    SkewX(f32, f32),
    /// Skew along Y-axis from first value to second value
    SkewY(f32, f32),
    /// Transform using transformation matrices (from, to)
    Matrix(TransformMatrix, TransformMatrix),
}

/// 2D transformation matrix
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformMatrix {
    /// Horizontal scaling factor
    pub a: f32,
    /// Horizontal skewing factor
    pub b: f32,
    /// Vertical skewing factor
    pub c: f32,
    /// Vertical scaling factor
    pub d: f32,
    /// Horizontal translation offset
    pub e: f32,
    /// Vertical translation offset
    pub f: f32,
}

impl Default for TransformMatrix {
    fn default() -> Self {
        // Identity matrix
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
}

/// CSS-like values with units
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CssValue {
    /// Plain numeric value without units
    Number(f32),
    /// Percentage value (0.0 to 100.0)
    Percentage(f32),
    /// Pixel value
    Pixels(f32),
    /// Em unit (relative to font size)
    Em(f32),
    /// Rem unit (relative to root font size)
    Rem(f32),
    /// Viewport width percentage
    ViewportWidth(f32),
    /// Viewport height percentage
    ViewportHeight(f32),
    /// RGB color value
    Color {
        /// Red component (0-255)
        r: u8,
        /// Green component (0-255)
        g: u8,
        /// Blue component (0-255)
        b: u8,
    },
    /// String value
    String(String),
}

impl CssValue {
    /// Create a pixel value
    pub fn pixels(value: f32) -> Self {
        Self::Pixels(value)
    }
    /// Create a percentage value
    pub fn percentage(value: f32) -> Self {
        Self::Percentage(value)
    }
    /// Create an em unit value
    pub fn em(value: f32) -> Self {
        Self::Em(value)
    }
    /// Create a rem unit value
    pub fn rem(value: f32) -> Self {
        Self::Rem(value)
    }
    /// Create a viewport width value
    pub fn vw(value: f32) -> Self {
        Self::ViewportWidth(value)
    }
    /// Create a viewport height value
    pub fn vh(value: f32) -> Self {
        Self::ViewportHeight(value)
    }
    /// Create a unitless number value
    pub fn number(value: f32) -> Self {
        Self::Number(value)
    }
    /// Create a color value from RGB components
    pub fn color(r: u8, g: u8, b: u8) -> Self {
        Self::Color { r, g, b }
    }
    /// Create a string value
    pub fn string(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

/// Individual property animation with timing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyAnimation {
    /// Name of the property being animated
    pub name: String,
    /// Starting value for the animation
    pub from: AnimationValue,
    /// Ending value for the animation
    pub to: AnimationValue,
    /// Timing offset within the animation duration (0.0 to 1.0)
    pub duration_offset: f32,
    /// Optional easing function override for this property
    pub easing_override: Option<EasingFunction>,
}

impl AnimatedProperty {
    /// Get the current interpolated value at time t (0.0 to 1.0)
    pub fn interpolate(&self, t: f32) -> AnimatedValue {
        match self {
            Self::Opacity(from, to) => AnimatedValue::Opacity(from + (to - from) * t),
            Self::Position(fx, fy, tx, ty) => {
                let x = *fx as f32 + (*tx as f32 - *fx as f32) * t;
                let y = *fy as f32 + (*ty as f32 - *fy as f32) * t;
                AnimatedValue::Position(x as i16, y as i16)
            }
            Self::Size(fw, fh, tw, th) => {
                let w = *fw as f32 + (*tw as f32 - *fw as f32) * t;
                let h = *fh as f32 + (*th as f32 - *fh as f32) * t;
                AnimatedValue::Size(w as u16, h as u16)
            }
            Self::Color(from, to) => {
                let r = from.0 as f32 + (to.0 as f32 - from.0 as f32) * t;
                let g = from.1 as f32 + (to.1 as f32 - from.1 as f32) * t;
                let b = from.2 as f32 + (to.2 as f32 - from.2 as f32) * t;
                AnimatedValue::Color {
                    r: r as u8,
                    g: g as u8,
                    b: b as u8,
                }
            }
            Self::Scale(from, to) => AnimatedValue::Scale(from + (to - from) * t),
            Self::Rotation(from, to) => AnimatedValue::Rotation(from + (to - from) * t),
            Self::Custom(name, from, to) => {
                AnimatedValue::Custom(name.clone(), from + (to - from) * t)
            }
            Self::Multiple(properties) => {
                let values: Vec<AnimatedValue> =
                    properties.iter().map(|prop| prop.interpolate(t)).collect();
                AnimatedValue::Multiple(values)
            }

            // New property types
            Self::Property(_name, from, to) => {
                AnimatedValue::Animation(AnimationValue::Number(from + (to - from) * t))
            }
            Self::Transform(transform_prop) => {
                AnimatedValue::Animation(Self::interpolate_transform(transform_prop, t))
            }
            Self::CssProperty(_name, from, to) => {
                AnimatedValue::Animation(Self::interpolate_css_value(from, to, t))
            }
            Self::PropertySet(properties) => {
                let values: Vec<AnimationValue> = properties
                    .iter()
                    .map(|prop| Self::interpolate_animation_value(&prop.from, &prop.to, t))
                    .collect();
                AnimatedValue::Animation(AnimationValue::Multiple(values))
            }
            Self::Keyframes(sequence) => {
                let sampled = sequence.sample(t);
                if sampled.len() == 1 {
                    let (key, value) = sampled.iter().next().expect("one sampled property");
                    fn single(key: &str, value: &keyframes::KeyframeValue) -> AnimatedValue {
                        match value {
                            keyframes::KeyframeValue::Number(n)
                            | keyframes::KeyframeValue::Css(CssValue::Number(n)) => {
                                AnimatedValue::Custom(key.into(), *n)
                            }
                            keyframes::KeyframeValue::Color(r, g, b, 255)
                            | keyframes::KeyframeValue::Css(CssValue::Color { r, g, b }) => {
                                AnimatedValue::Color {
                                    r: *r,
                                    g: *g,
                                    b: *b,
                                }
                            }
                            keyframes::KeyframeValue::Multiple(values) => AnimatedValue::Multiple(
                                values.iter().map(|value| single(key, value)).collect(),
                            ),
                            value => AnimatedValue::Animation(value.to_animation_value()),
                        }
                    }
                    single(key, value)
                } else {
                    AnimatedValue::Animation(AnimationValue::Map(
                        sampled
                            .into_iter()
                            .map(|(key, value)| (key, value.to_animation_value()))
                            .collect(),
                    ))
                }
            }
        }
    }

    /// Interpolate transform properties - helper method
    fn interpolate_transform(transform_prop: &TransformProperty, t: f32) -> AnimationValue {
        match transform_prop {
            TransformProperty::TranslateX(from, to) => AnimationValue::Transform(TransformMatrix {
                e: from + (to - from) * t,
                ..Default::default()
            }),
            TransformProperty::TranslateY(from, to) => AnimationValue::Transform(TransformMatrix {
                f: from + (to - from) * t,
                ..Default::default()
            }),
            TransformProperty::Translate(fx, fy, tx, ty) => {
                AnimationValue::Transform(TransformMatrix {
                    e: fx + (tx - fx) * t,
                    f: fy + (ty - fy) * t,
                    ..Default::default()
                })
            }
            TransformProperty::ScaleX(from, to) => AnimationValue::Transform(TransformMatrix {
                a: from + (to - from) * t,
                ..Default::default()
            }),
            TransformProperty::ScaleY(from, to) => AnimationValue::Transform(TransformMatrix {
                d: from + (to - from) * t,
                ..Default::default()
            }),
            TransformProperty::Scale(from, to) => {
                let scale = from + (to - from) * t;
                AnimationValue::Transform(TransformMatrix {
                    a: scale,
                    d: scale,
                    ..Default::default()
                })
            }
            TransformProperty::Rotate(from, to) => {
                let angle = from + (to - from) * t;
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let matrix = TransformMatrix {
                    a: cos_a,
                    b: sin_a,
                    c: -sin_a,
                    d: cos_a,
                    e: 0.0,
                    f: 0.0,
                };
                AnimationValue::Transform(matrix)
            }
            TransformProperty::SkewX(from, to) => AnimationValue::Transform(TransformMatrix {
                c: (from + (to - from) * t).to_radians().tan(),
                ..Default::default()
            }),
            TransformProperty::SkewY(from, to) => AnimationValue::Transform(TransformMatrix {
                b: (from + (to - from) * t).to_radians().tan(),
                ..Default::default()
            }),
            TransformProperty::Matrix(from, to) => {
                let matrix = TransformMatrix {
                    a: from.a + (to.a - from.a) * t,
                    b: from.b + (to.b - from.b) * t,
                    c: from.c + (to.c - from.c) * t,
                    d: from.d + (to.d - from.d) * t,
                    e: from.e + (to.e - from.e) * t,
                    f: from.f + (to.f - from.f) * t,
                };
                AnimationValue::Transform(matrix)
            }
        }
    }

    /// Interpolate CSS values
    pub(crate) fn interpolate_css_value(from: &CssValue, to: &CssValue, t: f32) -> AnimationValue {
        match (from, to) {
            (CssValue::Number(f), CssValue::Number(t_val)) => {
                AnimationValue::Number(f + (t_val - f) * t)
            }
            (CssValue::Percentage(f), CssValue::Percentage(t_val)) => {
                AnimationValue::Unit(f + (t_val - f) * t, "%".to_string())
            }
            (CssValue::Pixels(f), CssValue::Pixels(t_val)) => {
                AnimationValue::Unit(f + (t_val - f) * t, "px".to_string())
            }
            (CssValue::Em(f), CssValue::Em(t_val)) => {
                AnimationValue::Unit(f + (t_val - f) * t, "em".to_string())
            }
            (CssValue::Rem(f), CssValue::Rem(t_val)) => {
                AnimationValue::Unit(f + (t_val - f) * t, "rem".to_string())
            }
            (CssValue::ViewportWidth(f), CssValue::ViewportWidth(t_val)) => {
                AnimationValue::Unit(f + (t_val - f) * t, "vw".to_string())
            }
            (CssValue::ViewportHeight(f), CssValue::ViewportHeight(t_val)) => {
                AnimationValue::Unit(f + (t_val - f) * t, "vh".to_string())
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
                let r = *fr as f32 + (*tr as f32 - *fr as f32) * t;
                let g = *fg as f32 + (*tg as f32 - *fg as f32) * t;
                let b = *fb as f32 + (*tb as f32 - *fb as f32) * t;
                AnimationValue::Color {
                    r: r as u8,
                    g: g as u8,
                    b: b as u8,
                }
            }
            (CssValue::String(f), CssValue::String(t_val)) => {
                // For strings, we can't interpolate - just switch at midpoint
                if t < 0.5 {
                    AnimationValue::String(f.clone())
                } else {
                    AnimationValue::String(t_val.clone())
                }
            }
            _ => {
                // Different units cannot be interpolated without a layout context.
                // Preserve the selected endpoint's value and unit at the midpoint.
                keyframes::KeyframeValue::Css(if t < 0.5 { from.clone() } else { to.clone() })
                    .to_animation_value()
            }
        }
    }

    /// Interpolate between two AnimationValues
    pub(crate) fn interpolate_animation_value(
        from: &AnimationValue,
        to: &AnimationValue,
        t: f32,
    ) -> AnimationValue {
        match (from, to) {
            (AnimationValue::Number(f), AnimationValue::Number(t_val)) => {
                AnimationValue::Number(f + (t_val - f) * t)
            }
            (
                AnimationValue::Color {
                    r: fr,
                    g: fg,
                    b: fb,
                },
                AnimationValue::Color {
                    r: tr,
                    g: tg,
                    b: tb,
                },
            ) => {
                let r = *fr as f32 + (*tr as f32 - *fr as f32) * t;
                let g = *fg as f32 + (*tg as f32 - *fg as f32) * t;
                let b = *fb as f32 + (*tb as f32 - *fb as f32) * t;
                AnimationValue::Color {
                    r: r as u8,
                    g: g as u8,
                    b: b as u8,
                }
            }
            (AnimationValue::Unit(f_val, f_unit), AnimationValue::Unit(t_val, t_unit)) => {
                if f_unit == t_unit {
                    AnimationValue::Unit(f_val + (t_val - f_val) * t, f_unit.clone())
                } else {
                    // Different units - just switch at midpoint
                    if t < 0.5 {
                        from.clone()
                    } else {
                        to.clone()
                    }
                }
            }
            (AnimationValue::Array(f_arr), AnimationValue::Array(t_arr)) => {
                let min_len = f_arr.len().min(t_arr.len());
                let result: Vec<f32> = (0..min_len)
                    .map(|i| f_arr[i] + (t_arr[i] - f_arr[i]) * t)
                    .collect();
                AnimationValue::Array(result)
            }
            (AnimationValue::Transform(f_matrix), AnimationValue::Transform(t_matrix)) => {
                let matrix = TransformMatrix {
                    a: f_matrix.a + (t_matrix.a - f_matrix.a) * t,
                    b: f_matrix.b + (t_matrix.b - f_matrix.b) * t,
                    c: f_matrix.c + (t_matrix.c - f_matrix.c) * t,
                    d: f_matrix.d + (t_matrix.d - f_matrix.d) * t,
                    e: f_matrix.e + (t_matrix.e - f_matrix.e) * t,
                    f: f_matrix.f + (t_matrix.f - f_matrix.f) * t,
                };
                AnimationValue::Transform(matrix)
            }
            _ => {
                // For mismatched or unsupported types, switch at midpoint
                if t < 0.5 {
                    from.clone()
                } else {
                    to.clone()
                }
            }
        }
    }
}

impl TransformProperty {
    /// Interpolate transform property at time t
    pub fn interpolate(&self, t: f32) -> AnimatedValue {
        match self {
            Self::TranslateX(from, to) => {
                AnimatedValue::Custom("translateX".to_string(), from + (to - from) * t)
            }
            Self::TranslateY(from, to) => {
                AnimatedValue::Custom("translateY".to_string(), from + (to - from) * t)
            }
            Self::Translate(fx, fy, tx, ty) => {
                let x = fx + (tx - fx) * t;
                let y = fy + (ty - fy) * t;
                AnimatedValue::Position(x as i16, y as i16)
            }
            Self::ScaleX(from, to) => {
                AnimatedValue::Custom("scaleX".to_string(), from + (to - from) * t)
            }
            Self::ScaleY(from, to) => {
                AnimatedValue::Custom("scaleY".to_string(), from + (to - from) * t)
            }
            Self::Scale(from, to) => AnimatedValue::Scale(from + (to - from) * t),
            Self::Rotate(from, to) => {
                AnimatedValue::Rotation((from + (to - from) * t).to_degrees())
            }
            Self::SkewX(from, to) => {
                AnimatedValue::Custom("skewX".to_string(), from + (to - from) * t)
            }
            Self::SkewY(from, to) => {
                AnimatedValue::Custom("skewY".to_string(), from + (to - from) * t)
            }
            Self::Matrix(from, to) => {
                let interpolated = TransformMatrix {
                    a: from.a + (to.a - from.a) * t,
                    b: from.b + (to.b - from.b) * t,
                    c: from.c + (to.c - from.c) * t,
                    d: from.d + (to.d - from.d) * t,
                    e: from.e + (to.e - from.e) * t,
                    f: from.f + (to.f - from.f) * t,
                };
                AnimatedValue::Animation(AnimationValue::Transform(interpolated))
            }
        }
    }
}
