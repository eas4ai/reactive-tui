//! Explicit value semantics for typed keyframes.

use super::{CssValue, EasingFunction, KeyframeValue, TransformMatrix};
use std::fmt;

/// Invalid keyframe input or a conversion that would lose information.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyframeError(String);

impl KeyframeError {
    /// Describe invalid input, including the property or value when useful.
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for KeyframeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for KeyframeError {}

/// Interpolation and lossless input conversion for a typed keyframe value.
///
/// Custom value types implement this trait instead of relying on `Clone` and
/// `Default`. Numeric implementations interpolate; strings and booleans retain
/// the source value until the destination keyframe. Numeric easing may overshoot.
///
/// To migrate a custom `Clone` value, define its interpolation and conversion:
///
/// ```
/// use reactive_tui::animation::{EasingFunction, KeyframeError, KeyframeType};
/// use reactive_tui::animation::keyframes::KeyframeValue;
/// #[derive(Clone)]
/// struct Distance(f32);
/// impl KeyframeType for Distance {
///     fn interpolate_keyframe(&self, to: &Self, t: f32, easing: &EasingFunction) -> Self {
///         Self(self.0.interpolate_keyframe(&to.0, t, easing))
///     }
///     fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
///         f32::from_keyframe(value).map(Self)
///     }
/// }
/// ```
pub trait KeyframeType: Clone {
    /// Interpolate at local progress, applying easing only to interpolable values.
    fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self;

    /// Convert an authored value, rejecting incompatible types or lost data.
    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError>;
}

fn number(value: &KeyframeValue) -> Result<f32, KeyframeError> {
    match value {
        KeyframeValue::Number(value) | KeyframeValue::Css(CssValue::Number(value))
            if value.is_finite() =>
        {
            Ok(*value)
        }
        _ => Err(KeyframeError::new("expected a finite unitless number")),
    }
}

impl KeyframeType for f32 {
    fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self {
        let progress = easing.apply(progress);
        self * (1.0 - progress) + to * progress
    }

    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
        number(value)
    }
}

impl KeyframeType for f64 {
    fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self {
        let progress = f64::from(easing.apply(progress));
        self * (1.0 - progress) + to * progress
    }

    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
        number(value).map(f64::from)
    }
}

// These integer domains fit exactly in f64. Conversion never truncates input;
// interpolation follows Rust's truncating, saturating float-to-integer cast.
macro_rules! integer {
    ($($kind:ty),+ $(,)?) => {$ (
        impl KeyframeType for $kind {
            fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self {
                let progress = easing.apply(progress);
                (f64::from(*self)
                    + (f64::from(*to) - f64::from(*self)) * f64::from(progress)) as Self
            }

            fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
                let value = f64::from(number(value)?);
                if value.fract() != 0.0
                    || value < f64::from(Self::MIN)
                    || value > f64::from(Self::MAX)
                {
                    return Err(KeyframeError::new(concat!(
                        "number is not exactly representable as ", stringify!($kind)
                    )));
                }
                Ok(value as Self)
            }
        }
    )+};
}

integer!(i8, i16, i32, u8, u16, u32);

impl KeyframeType for String {
    fn interpolate_keyframe(&self, to: &Self, progress: f32, _easing: &EasingFunction) -> Self {
        if progress < 1.0 {
            self.clone()
        } else {
            to.clone()
        }
    }

    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
        match value {
            KeyframeValue::String(value) | KeyframeValue::Css(CssValue::String(value)) => {
                Ok(value.clone())
            }
            _ => Err(KeyframeError::new("expected a string")),
        }
    }
}

impl KeyframeType for bool {
    fn interpolate_keyframe(&self, to: &Self, progress: f32, _easing: &EasingFunction) -> Self {
        if progress < 1.0 {
            *self
        } else {
            *to
        }
    }

    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
        match value {
            KeyframeValue::Boolean(value) => Ok(*value),
            _ => Err(KeyframeError::new("expected a boolean")),
        }
    }
}

impl<A: KeyframeType, B: KeyframeType> KeyframeType for (A, B) {
    fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self {
        (
            self.0.interpolate_keyframe(&to.0, progress, easing),
            self.1.interpolate_keyframe(&to.1, progress, easing),
        )
    }

    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
        match value {
            KeyframeValue::Multiple(values) if values.len() == 2 => {
                Ok((A::from_keyframe(&values[0])?, B::from_keyframe(&values[1])?))
            }
            _ => Err(KeyframeError::new("expected exactly two compound values")),
        }
    }
}

impl KeyframeType for (u8, u8, u8, u8) {
    fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self {
        (
            self.0.interpolate_keyframe(&to.0, progress, easing),
            self.1.interpolate_keyframe(&to.1, progress, easing),
            self.2.interpolate_keyframe(&to.2, progress, easing),
            self.3.interpolate_keyframe(&to.3, progress, easing),
        )
    }

    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
        match value {
            KeyframeValue::Color(r, g, b, a) => Ok((*r, *g, *b, *a)),
            _ => Err(KeyframeError::new("expected an RGBA color")),
        }
    }
}

impl KeyframeType for TransformMatrix {
    fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self {
        Self {
            a: self.a.interpolate_keyframe(&to.a, progress, easing),
            b: self.b.interpolate_keyframe(&to.b, progress, easing),
            c: self.c.interpolate_keyframe(&to.c, progress, easing),
            d: self.d.interpolate_keyframe(&to.d, progress, easing),
            e: self.e.interpolate_keyframe(&to.e, progress, easing),
            f: self.f.interpolate_keyframe(&to.f, progress, easing),
        }
    }

    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
        match value {
            KeyframeValue::Transform(value) => Ok(value.clone()),
            _ => Err(KeyframeError::new("expected a transform matrix")),
        }
    }
}

impl KeyframeType for CssValue {
    fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self {
        KeyframeValue::interpolate_css_values(self, to, easing.apply(progress)).unwrap_or_else(
            || {
                if progress < 1.0 {
                    self.clone()
                } else {
                    to.clone()
                }
            },
        )
    }

    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
        match value {
            KeyframeValue::Css(value) => Ok(value.clone()),
            KeyframeValue::Number(value) => Ok(Self::Number(*value)),
            KeyframeValue::String(value) => Ok(Self::String(value.clone())),
            _ => Err(KeyframeError::new("expected a CSS value")),
        }
    }
}

impl KeyframeType for KeyframeValue {
    fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self {
        match (self, to) {
            (Self::Multiple(from), Self::Multiple(to)) if from.len() == to.len() => Self::Multiple(
                from.iter()
                    .zip(to)
                    .map(|(from, to)| from.interpolate_keyframe(to, progress, easing))
                    .collect(),
            ),
            _ => self
                .interpolate(to, easing.apply(progress))
                .unwrap_or_else(|| {
                    if progress < 1.0 {
                        self.clone()
                    } else {
                        to.clone()
                    }
                }),
        }
    }

    fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
        Ok(value.clone())
    }
}
