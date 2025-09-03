//! Animation System Widget
//!
//! A comprehensive animation system providing smooth transitions, easing functions,
//! and property animations for TUI widgets with frame-based timing and interpolation.

/// Animation API for creating and managing animations
pub mod api;
/// Debug utilities for animation system
pub mod debug;
/// Easing functions for smooth animation transitions
pub mod easing;
/// Keyframe-based animation system
pub mod keyframes;
/// Lock-free animation state management to prevent deadlocks
pub mod lock_free;
/// Performance monitoring and optimization for animations
pub mod performance;
/// Spring physics-based animations
pub mod spring;
/// Staggered animation utilities for coordinated effects
pub mod stagger;

// Re-export commonly used types
pub use easing::ease_value;
pub use keyframes::{Keyframe, KeyframeAnimation};
pub use spring::SpringConfig;
pub use stagger::StaggerConfig;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

// Type aliases for complex function pointer types
type OnStartCallback = Arc<dyn Fn(&Animation) + Send + Sync>;
type OnUpdateCallback = Arc<dyn Fn(&Animation, &AnimatedValue) + Send + Sync>;
type OnCompleteCallback = Arc<dyn Fn(&Animation) + Send + Sync>;
type OnLoopCallback = Arc<dyn Fn(&Animation, u32) + Send + Sync>;
type OnPauseCallback = Arc<dyn Fn(&Animation) + Send + Sync>;
type OnStopCallback = Arc<dyn Fn(&Animation) + Send + Sync>;

/// Unique identifier for animations
pub type AnimationId = String;

/// Unique identifier for animation timelines
pub type TimelineId = String;

/// Monotonic time source resistant to time manipulation attacks
pub struct MonotonicTimer {
    /// Base time from when the timer was created
    pub(crate) base_time: Instant,
    /// Monotonic counter to prevent backward time jumps
    last_time: AtomicU64,
}

impl MonotonicTimer {
    /// Create a new monotonic timer
    pub fn new() -> Self {
        Self {
            base_time: Instant::now(),
            last_time: AtomicU64::new(0),
        }
    }

    /// Get current monotonic time that cannot go backwards
    pub fn now(&self) -> Duration {
        let current = self.base_time.elapsed();
        let current_nanos = current.as_nanos() as u64;
        
        // Ensure time only moves forward
        let last = self.last_time.load(Ordering::Acquire);
        let monotonic_nanos = if current_nanos > last {
            // Normal case: time moved forward
            self.last_time.store(current_nanos, Ordering::Release);
            current_nanos
        } else {
            // Time manipulation detected: use last known good time
            last
        };
        
        // Protect against overflow
        let safe_nanos = monotonic_nanos.min(u64::MAX / 2);
        Duration::from_nanos(safe_nanos)
    }

    /// Get elapsed time since timer creation
    pub fn elapsed(&self) -> Duration {
        self.now()
    }

    /// Check if a duration has elapsed since a given start time
    pub fn has_elapsed(&self, start: Duration, duration: Duration) -> bool {
        let current = self.now();
        current.saturating_sub(start) >= duration
    }
}

impl Default for MonotonicTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Global monotonic timer instance
static MONOTONIC_TIMER: std::sync::OnceLock<MonotonicTimer> = std::sync::OnceLock::new();

/// Get the global monotonic timer
fn get_monotonic_timer() -> &'static MonotonicTimer {
    MONOTONIC_TIMER.get_or_init(MonotonicTimer::new)
}

/// Animation controller for managing animations
pub struct AnimationController {
    animation: Animation,
    is_running: bool,
}

impl AnimationController {
    /// Create a new animation controller with the given animation
    ///
    /// # Arguments
    /// * `animation` - The animation to control
    ///
    /// # Returns
    /// A new `AnimationController` instance
    pub fn new(animation: Animation) -> Self {
        Self {
            animation,
            is_running: false,
        }
    }

    /// Start playing the animation
    ///
    /// Sets the controller state to running and begins animation playback.
    pub fn start(&mut self) {
        self.is_running = true;
        self.animation.play();
    }

    /// Pause the animation
    ///
    /// Temporarily stops animation playback while preserving current position.
    /// Use `resume()` to continue from the current position.
    pub fn pause(&mut self) {
        self.is_running = false;
        self.animation.pause();
    }

    /// Stop the animation completely
    ///
    /// Stops animation playback and resets to the beginning.
    /// Use `start()` to begin playback from the start.
    pub fn stop(&mut self) {
        self.is_running = false;
        self.animation.stop();
    }

    /// Resume a paused animation
    ///
    /// Continues animation playback from the current position if the animation
    /// was previously paused. Has no effect if already playing.
    pub fn resume(&mut self) {
        if !self.is_running {
            self.is_running = true;
            self.animation.play();
        }
    }

    /// Reset the animation to its initial state
    ///
    /// Stops the animation and seeks back to the beginning (progress 0.0).
    pub fn reset(&mut self) {
        self.stop();
        self.animation.seek(0.0);
    }

    /// Check if the animation is currently playing
    ///
    /// # Returns
    /// `true` if the animation is actively playing, `false` otherwise
    pub fn is_playing(&self) -> bool {
        self.is_running
    }
}

/// Animation easing functions for smooth transitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EasingFunction {
    /// Linear interpolation (no easing)
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in and out (slow start and end)
    EaseInOut,
    /// Cubic bezier curve with custom control points
    CubicBezier(f32, f32, f32, f32),
    /// Bounce effect at the end
    Bounce,
    /// Elastic effect with overshoot
    Elastic,
    /// Back effect with slight overshoot
    Back,
    /// Exponential easing
    Expo,
    /// Circular easing
    Circ,
    /// Sine wave easing
    Sine,
    /// Quadratic easing
    Quad,
    /// Cubic easing
    Cubic,
    /// Quartic easing
    Quart,
    /// Quintic easing
    Quint,

    // Anime.js inspired advanced easing functions
    /// Spring physics-based easing
    Spring(spring::SpringConfig),
    /// Stepped easing with specified number of steps
    Steps(u32, bool), // step count, jump at start
    /// Piecewise linear easing with control points
    LinearPoints(Vec<f32>),
    /// Irregular stepping with randomness
    Irregular(u32, f32), // step count, randomness factor

    // Parametric power variations
    /// Ease in with custom power
    InPower(f32),
    /// Ease out with custom power
    OutPower(f32),
    /// Ease in-out with custom power
    InOutPower(f32),

    // Parametric back variations
    /// Ease in back with custom overshoot
    InBack(f32),
    /// Ease out back with custom overshoot
    OutBack(f32),
    /// Ease in-out back with custom overshoot
    InOutBack(f32),

    // Parametric elastic variations
    /// Ease in elastic with custom amplitude and period
    InElastic(f32, f32), // amplitude, period
    /// Ease out elastic with custom amplitude and period
    OutElastic(f32, f32),
    /// Ease in-out elastic with custom amplitude and period
    InOutElastic(f32, f32),
}

impl Default for EasingFunction {
    fn default() -> Self {
        Self::EaseInOut
    }
}

impl EasingFunction {
    /// Apply easing function to a normalized time value (0.0 to 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Self::CubicBezier(_x1, y1, _x2, y2) => {
                // Simplified cubic bezier approximation
                let t2 = t * t;
                let t3 = t2 * t;
                3.0 * (1.0 - t) * (1.0 - t) * t * y1 + 3.0 * (1.0 - t) * t2 * y2 + t3
            }
            Self::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
            Self::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    let s = p / 4.0;
                    -(2.0_f32.powf(10.0 * (t - 1.0)))
                        * ((t - 1.0 - s) * (2.0 * std::f32::consts::PI) / p).sin()
                }
            }
            Self::Back => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Self::Expo => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * (t - 1.0))
                }
            }
            Self::Circ => 1.0 - (1.0 - t * t).sqrt(),
            Self::Sine => 1.0 - (t * std::f32::consts::PI / 2.0).cos(),
            Self::Quad => t * t,
            Self::Cubic => t * t * t,
            Self::Quart => t * t * t * t,
            Self::Quint => t * t * t * t * t,

            // Advanced easing functions from anime.js specification
            Self::Spring(config) => {
                // For spring easing, we need to calculate based on estimated duration
                // This is a simplified version - for full spring behavior, use the spring directly
                let duration = config.estimate_duration(0.0, 1.0);
                let current_time = t * duration;
                config.calculate_position(current_time, 0.0, 1.0)
            }
            Self::Steps(steps, jump_at_start) => {
                if *steps == 0 {
                    return if *jump_at_start { 1.0 } else { 0.0 };
                }
                let step_size = 1.0 / *steps as f32;
                if *jump_at_start {
                    ((t * *steps as f32).ceil() * step_size).min(1.0)
                } else {
                    ((t * *steps as f32).floor() * step_size).min(1.0)
                }
            }
            Self::LinearPoints(points) => {
                if points.is_empty() {
                    return t;
                }
                if points.len() == 1 {
                    return points[0] * t;
                }

                // Interpolate through the control points
                let segment_count = points.len() - 1;
                let segment_progress = t * segment_count as f32;
                let segment_index = (segment_progress.floor() as usize).min(segment_count - 1);
                let local_t = segment_progress - segment_index as f32;

                let from = points[segment_index];
                let to = points[(segment_index + 1).min(points.len() - 1)];
                from + (to - from) * local_t
            }
            Self::Irregular(steps, randomness) => {
                // Pseudo-random irregular easing based on step position
                if *steps == 0 {
                    return t;
                }
                let step_size = 1.0 / *steps as f32;
                let base_step = (t * *steps as f32).floor();
                let step_progress = (t * *steps as f32) - base_step;

                // Simple pseudo-random function based on step index
                let step_hash = (base_step as u32).wrapping_mul(2654435761);
                let random_factor = ((step_hash % 1000) as f32 / 1000.0 - 0.5) * randomness;

                let base_value = base_step * step_size;
                let random_offset = random_factor * step_size;
                (base_value + step_progress * step_size + random_offset).clamp(0.0, 1.0)
            }

            // Parametric power variations
            Self::InPower(power) => t.powf(*power),
            Self::OutPower(power) => 1.0 - (1.0 - t).powf(*power),
            Self::InOutPower(power) => {
                if t < 0.5 {
                    0.5 * (2.0 * t).powf(*power)
                } else {
                    1.0 - 0.5 * (2.0 * (1.0 - t)).powf(*power)
                }
            }

            // Parametric back variations
            Self::InBack(overshoot) => {
                let c3 = overshoot + 1.0;
                c3 * t * t * t - overshoot * t * t
            }
            Self::OutBack(overshoot) => {
                let c3 = overshoot + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + overshoot * (t - 1.0).powi(2)
            }
            Self::InOutBack(overshoot) => {
                let c2 = overshoot * 1.525;
                if t < 0.5 {
                    0.5 * ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2))
                } else {
                    0.5 * ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0)
                }
            }

            // Parametric elastic variations
            Self::InElastic(amplitude, period) => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c = (2.0 * std::f32::consts::PI) / period;
                    -amplitude * 2.0_f32.powf(10.0 * (t - 1.0)) * ((t - 1.0) * c).sin()
                }
            }
            Self::OutElastic(amplitude, period) => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c = (2.0 * std::f32::consts::PI) / period;
                    amplitude * 2.0_f32.powf(-10.0 * t) * (t * c).sin() + 1.0
                }
            }
            Self::InOutElastic(amplitude, period) => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c = (2.0 * std::f32::consts::PI) / period;
                    if t < 0.5 {
                        -0.5 * amplitude
                            * 2.0_f32.powf(20.0 * t - 10.0)
                            * ((20.0 * t - 11.125) * c).sin()
                    } else {
                        0.5 * amplitude
                            * 2.0_f32.powf(-20.0 * t + 10.0)
                            * ((20.0 * t - 11.125) * c).sin()
                            + 1.0
                    }
                }
            }
        }
    }

    /// Apply easing with explicit from/to values (for spring physics)
    pub fn apply_with_values(&self, t: f32, from: f32, to: f32) -> f32 {
        match self {
            Self::Spring(config) => {
                let duration = config.estimate_duration(from, to);
                let current_time = t * duration;
                config.calculate_position(current_time, from, to)
            }
            _ => {
                let eased_t = self.apply(t);
                from + (to - from) * eased_t
            }
        }
    }
}

// Convenience functions for creating advanced easing functions
impl EasingFunction {
    /// Create a spring easing with custom parameters
    pub fn spring(mass: f32, stiffness: f32, damping: f32) -> Self {
        Self::Spring(SpringConfig::new(mass, stiffness, damping))
    }

    /// Create a gentle spring easing
    pub fn spring_gentle() -> Self {
        Self::Spring(SpringConfig::gentle())
    }

    /// Create a wobbly spring easing
    pub fn spring_wobbly() -> Self {
        Self::Spring(SpringConfig::wobbly())
    }

    /// Create a stiff spring easing
    pub fn spring_stiff() -> Self {
        Self::Spring(SpringConfig::stiff())
    }

    /// Create stepped easing
    pub fn steps(count: u32, jump_at_start: bool) -> Self {
        Self::Steps(count, jump_at_start)
    }

    /// Create linear points easing
    pub fn linear_points(points: Vec<f32>) -> Self {
        Self::LinearPoints(points)
    }

    /// Create irregular easing
    pub fn irregular(steps: u32, randomness: f32) -> Self {
        Self::Irregular(steps, randomness.clamp(0.0, 1.0))
    }

    /// Create power easing variants
    pub fn power_in(power: f32) -> Self {
        Self::InPower(power)
    }

    /// Create power-out easing with custom exponent
    ///
    /// # Arguments
    /// * `power` - The power/exponent to use for the easing curve
    pub fn power_out(power: f32) -> Self {
        Self::OutPower(power)
    }

    /// Create power-in-out easing with custom exponent
    ///
    /// # Arguments
    /// * `power` - The power/exponent to use for the easing curve
    pub fn power_in_out(power: f32) -> Self {
        Self::InOutPower(power)
    }

    /// Create back-in easing with custom overshoot
    ///
    /// # Arguments
    /// * `overshoot` - Amount of overshoot beyond the target value
    pub fn back_in(overshoot: f32) -> Self {
        Self::InBack(overshoot)
    }

    /// Create back-out easing with custom overshoot
    ///
    /// # Arguments
    /// * `overshoot` - Amount of overshoot beyond the target value
    pub fn back_out(overshoot: f32) -> Self {
        Self::OutBack(overshoot)
    }

    /// Create back-in-out easing with custom overshoot
    ///
    /// # Arguments
    /// * `overshoot` - Amount of overshoot beyond the target value
    pub fn back_in_out(overshoot: f32) -> Self {
        Self::InOutBack(overshoot)
    }

    /// Create elastic-in easing with custom amplitude and period
    ///
    /// # Arguments
    /// * `amplitude` - The amplitude of the elastic oscillation
    /// * `period` - The period of the elastic oscillation
    pub fn elastic_in(amplitude: f32, period: f32) -> Self {
        Self::InElastic(amplitude, period)
    }

    /// Create elastic-out easing with custom amplitude and period
    ///
    /// # Arguments
    /// * `amplitude` - The amplitude of the elastic oscillation
    /// * `period` - The period of the elastic oscillation
    pub fn elastic_out(amplitude: f32, period: f32) -> Self {
        Self::OutElastic(amplitude, period)
    }

    /// Create elastic-in-out easing with custom amplitude and period
    ///
    /// # Arguments
    /// * `amplitude` - The amplitude of the elastic oscillation
    /// * `period` - The period of the elastic oscillation
    pub fn elastic_in_out(amplitude: f32, period: f32) -> Self {
        Self::InOutElastic(amplitude, period)
    }
}

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

/// Enhanced animation values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationValue {
    /// Numeric value for mathematical interpolation
    Number(f32),
    /// RGB color value for color transitions
    Color {
        /// Red component (0-255)
        r: u8,
        /// Green component (0-255)
        g: u8,
        /// Blue component (0-255)
        b: u8,
    },
    /// String value for text-based animations
    String(String),
    /// Boolean value for toggle animations
    Boolean(bool),
    /// Array of numeric values for complex animations
    Array(Vec<f32>),
    /// Value with unit (px, %, em, etc.)
    Unit(f32, String),
    /// Transformation matrix for geometric animations
    Transform(TransformMatrix),
    /// Multiple animation values for compound animations
    Multiple(Vec<AnimationValue>),
    /// Property name to value mapping for complex animations
    Map(std::collections::HashMap<String, AnimationValue>),
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
                // Sample the keyframe sequence at the given time
                let sampled_values = sequence.sample(t);

                if sampled_values.len() == 1 {
                    // Single property, return its value directly
                    if let Some((_, value)) = sampled_values.into_iter().next() {
                        AnimatedValue::Animation(value.to_animation_value())
                    } else {
                        AnimatedValue::Animation(AnimationValue::Map(
                            std::collections::HashMap::new(),
                        ))
                    }
                } else {
                    // Multiple properties, return as a map
                    let animation_values: std::collections::HashMap<String, AnimationValue> =
                        sampled_values
                            .into_iter()
                            .map(|(key, value)| (key, value.to_animation_value()))
                            .collect();
                    AnimatedValue::Animation(AnimationValue::Map(animation_values))
                }
            }
        }
    }

    /// Interpolate transform properties
    fn interpolate_transform(transform: &TransformProperty, t: f32) -> AnimationValue {
        match transform {
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
                let angle = (from + (to - from) * t).to_radians();
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
    fn interpolate_css_value(from: &CssValue, to: &CssValue, t: f32) -> AnimationValue {
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
                // Mismatched types - use from value
                match from {
                    CssValue::Number(val) => AnimationValue::Number(*val),
                    CssValue::Color { r, g, b } => AnimationValue::Color {
                        r: *r,
                        g: *g,
                        b: *b,
                    },
                    CssValue::String(s) => AnimationValue::String(s.clone()),
                    _ => AnimationValue::Number(0.0),
                }
            }
        }
    }

    /// Interpolate between two AnimationValues
    fn interpolate_animation_value(
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

/// Current animated values (legacy - kept for compatibility)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimatedValue {
    /// Current opacity value
    Opacity(f32),
    /// Current position (x, y)
    Position(i16, i16),
    /// Current size (width, height)
    Size(u16, u16),
    /// Current color
    Color {
        /// Red component (0-255)
        r: u8,
        /// Green component (0-255)
        g: u8,
        /// Blue component (0-255)
        b: u8,
    },
    /// Current scale factor
    Scale(f32),
    /// Current rotation in degrees
    Rotation(f32),
    /// Custom property value
    Custom(String, f32),
    /// Multiple values
    Multiple(Vec<AnimatedValue>),
    /// New animation value
    Animation(AnimationValue),
}

// Remove duplicate stagger types - they're in the stagger module

// Remove duplicate stagger implementation - it's in the stagger module

/// Animation playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimationState {
    /// Animation is stopped/not started
    #[default]
    Stopped,
    /// Animation is playing
    Playing,
    /// Animation is paused
    Paused,
    /// Animation has completed
    Completed,
    /// Animation is playing in reverse
    Reversed,
}

/// Animation loop behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopMode {
    /// Play once
    None,
    /// Loop indefinitely
    Infinite,
    /// Loop a specific number of times
    Count(u32),
    /// Ping-pong (forward then reverse)
    PingPong,
}

impl Default for LoopMode {
    fn default() -> Self {
        Self::None
    }
}

/// Animation configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Animation duration
    pub duration: Duration,
    /// Easing function
    pub easing: EasingFunction,
    /// Delay before starting
    pub delay: Duration,
    /// Loop behavior
    pub loop_mode: LoopMode,
    /// Animation direction
    pub reverse: bool,
    /// Speed multiplier (1.0 = normal speed)
    pub speed: f32,
    /// Whether to auto-play on creation
    pub auto_play: bool,
    /// Whether to auto-reverse on completion
    pub auto_reverse: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(500),
            easing: EasingFunction::EaseInOut,
            delay: Duration::ZERO,
            loop_mode: LoopMode::None,
            reverse: false,
            speed: 1.0,
            auto_play: false,
            auto_reverse: false,
        }
    }
}

/// Animation runtime state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AnimationRuntimeState {
    /// Current playback state
    pub state: AnimationState,
    /// Current time position in animation
    pub current_time: Duration,
    /// Number of loops completed
    pub loops_completed: u32,
    /// Whether currently playing in reverse
    pub is_reversed: bool,
    /// Current animated values
    pub current_values: Option<AnimatedValue>,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
}

/// Non-serializable runtime data
#[derive(Debug, Default)]
pub struct AnimationRuntime {
    /// Last frame timestamp
    pub last_frame_time: Option<Instant>,
}

/// Event callbacks for animation lifecycle
#[derive(Default)]
pub struct AnimationCallbacks {
    /// Called when animation starts
    pub on_start: Option<OnStartCallback>,
    /// Called on each frame update
    pub on_update: Option<OnUpdateCallback>,
    /// Called when animation completes
    pub on_complete: Option<OnCompleteCallback>,
    /// Called when animation loops
    pub on_loop: Option<OnLoopCallback>,
    /// Called when animation is paused
    pub on_pause: Option<OnPauseCallback>,
    /// Called when animation is stopped
    pub on_stop: Option<OnStopCallback>,
}

/// Main Animation widget
pub struct Animation {
    /// Unique animation identifier
    pub id: AnimationId,
    /// Property to animate
    pub property: AnimatedProperty,
    /// Animation configuration
    pub config: AnimationConfig,
    /// Runtime state
    pub state: Arc<std::sync::RwLock<AnimationRuntimeState>>,
    /// Non-serializable runtime data
    pub runtime: AnimationRuntime,
    /// Event callbacks
    pub callbacks: AnimationCallbacks,
    /// Start time for delay calculations
    start_time: Option<Instant>,
}

impl Animation {
    /// Create a new animation
    pub fn new(
        duration: Duration,
        easing: EasingFunction,
        loop_count: Option<u32>,
        loop_behavior: LoopMode,
    ) -> Self {
        let loop_mode = match loop_count {
            None => LoopMode::None,
            Some(0) => LoopMode::Infinite,
            Some(n) => LoopMode::Count(n),
        };

        Self {
            id: format!(
                "anim_{}",
                get_monotonic_timer()
                    .elapsed()
                    .as_millis()
            ),
            property: AnimatedProperty::Opacity(0.0, 1.0),
            config: AnimationConfig {
                duration,
                delay: Duration::ZERO,
                easing,
                loop_mode,
                reverse: false,
                speed: 1.0,
                auto_play: false,
                auto_reverse: loop_behavior == LoopMode::PingPong,
            },
            state: Arc::new(std::sync::RwLock::new(AnimationRuntimeState::default())),
            runtime: AnimationRuntime::default(),
            callbacks: AnimationCallbacks::default(),
            start_time: None,
        }
    }

    /// Create a spring animation with physics-based easing
    ///
    /// # Arguments
    /// * `duration` - The duration of the animation
    /// * `config` - Spring configuration parameters (mass, stiffness, damping)
    ///
    /// # Returns
    /// A new `Animation` instance with spring easing
    pub fn spring(duration: Duration, config: SpringConfig) -> Self {
        Self::new(
            duration,
            EasingFunction::Spring(config),
            None,
            LoopMode::None,
        )
    }

    /// Create a new animation builder for fluent configuration
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the animation
    ///
    /// # Returns
    /// An `AnimationBuilder` instance for fluent configuration
    pub fn builder<S: Into<String>>(id: S) -> AnimationBuilder {
        AnimationBuilder::new(id)
    }

    /// Start playing the animation
    ///
    /// Sets the animation state to playing and triggers the start callback.
    /// Records the start time for delay calculations.
    pub fn play(&mut self) {
        if let Ok(mut state) = self.state.write() {
            state.state = AnimationState::Playing;
        }

        let now = get_monotonic_timer().now();
        // Convert monotonic Duration to Instant for compatibility
        let base = get_monotonic_timer().base_time;
        self.runtime.last_frame_time = Some(base + now);
        self.start_time = Some(base + now);

        if let Some(callback) = &self.callbacks.on_start {
            callback(self);
        }
    }

    /// Pause the animation at its current position
    ///
    /// Sets the animation state to paused and triggers the pause callback.
    /// The animation can be resumed from this position using `play()`.
    pub fn pause(&mut self) {
        if let Ok(mut state) = self.state.write() {
            state.state = AnimationState::Paused;
        }

        if let Some(callback) = &self.callbacks.on_pause {
            callback(self);
        }
    }

    /// Stop the animation and reset to the beginning
    ///
    /// Resets all animation state including time, progress, and loop count.
    /// Triggers the stop callback and clears timing information.
    pub fn stop(&mut self) {
        if let Ok(mut state) = self.state.write() {
            state.state = AnimationState::Stopped;
            state.current_time = Duration::ZERO;
            state.progress = 0.0;
            state.loops_completed = 0;
            state.is_reversed = false;
            state.current_values = None;
        }

        self.runtime.last_frame_time = None;
        self.start_time = None;

        if let Some(callback) = &self.callbacks.on_stop {
            callback(self);
        }
    }

    /// Reverse the animation direction
    ///
    /// Toggles the animation direction. If currently playing forward,
    /// it will play backward and vice versa.
    pub fn reverse(&mut self) {
        if let Ok(mut state) = self.state.write() {
            state.is_reversed = !state.is_reversed;
            if state.state == AnimationState::Playing {
                state.state = AnimationState::Reversed;
            }
        }
    }

    /// Update the animation for one frame
    ///
    /// This should be called every frame in your main loop to advance the animation.
    /// Handles timing, easing, looping, and triggers appropriate callbacks.
    ///
    /// # Arguments
    /// * `delta_time` - Time elapsed since the last frame
    ///
    /// # Returns
    /// `true` if the animation is still active, `false` if completed or stopped
    pub fn update(&mut self, delta_time: Duration) -> bool {
        let mut state_guard = match self.state.write() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        let state = &mut *state_guard;

        if state.state != AnimationState::Playing {
            return false;
        }

        // Handle delay using monotonic time
        if let Some(start_time) = self.start_time {
            let monotonic_now = get_monotonic_timer().base_time + get_monotonic_timer().now();
            if monotonic_now.duration_since(start_time) < self.config.delay {
                return false;
            }
        }

        // Update time
        let adjusted_delta = Duration::from_secs_f32(delta_time.as_secs_f32() * self.config.speed);
        state.current_time += adjusted_delta;

        // Calculate progress
        let total_duration = self.config.duration;
        let raw_progress = if total_duration > Duration::ZERO {
            (state.current_time.as_secs_f32() / total_duration.as_secs_f32()).min(1.0)
        } else {
            1.0
        };

        // Apply direction
        let progress = if state.is_reversed || self.config.reverse {
            1.0 - raw_progress
        } else {
            raw_progress
        };

        // Apply easing
        let eased_progress = self.config.easing.apply(progress);

        // Update state
        state.progress = eased_progress;
        state.current_values = Some(self.property.interpolate(eased_progress));

        // Trigger update callback
        if let Some(callback) = &self.callbacks.on_update {
            if let Some(ref values) = state.current_values {
                callback(self, values);
            }
        }

        // Check for completion
        if raw_progress >= 1.0 {
            // Handle animation completion inline to avoid borrow issues
            match self.config.loop_mode {
                LoopMode::None => {
                    state.state = AnimationState::Completed;
                }
                LoopMode::Infinite => {
                    state.current_time = Duration::ZERO;
                    state.progress = 0.0;
                    state.loops_completed += 1;
                }
                LoopMode::Count(count) => {
                    state.loops_completed += 1;
                    if state.loops_completed < count {
                        state.current_time = Duration::ZERO;
                        state.progress = 0.0;
                    } else {
                        state.state = AnimationState::Completed;
                    }
                }
                LoopMode::PingPong => {
                    state.is_reversed = !state.is_reversed;
                    state.current_time = Duration::ZERO;
                    state.loops_completed += 1;
                }
            }
        }

        // Drop the guard before calling callbacks
        drop(state_guard);

        // Call callbacks if needed after releasing the lock
        if raw_progress >= 1.0
            && matches!(self.config.loop_mode, LoopMode::None | LoopMode::Count(_))
        {
            if let Ok(state) = self.state.read() {
                if state.state == AnimationState::Completed {
                    if let Some(callback) = &self.callbacks.on_complete {
                        callback(self);
                    }
                }
            }
        }

        true
    }

    /// Handle animation completion and looping
    #[allow(dead_code)]
    fn handle_animation_complete(&mut self, state: &mut AnimationRuntimeState) {
        match self.config.loop_mode {
            LoopMode::None => {
                state.state = AnimationState::Completed;
                if let Some(callback) = &self.callbacks.on_complete {
                    callback(self);
                }
            }
            LoopMode::Infinite => {
                self.restart_animation(state);
            }
            LoopMode::Count(count) => {
                state.loops_completed += 1;
                if state.loops_completed < count {
                    self.restart_animation(state);
                } else {
                    state.state = AnimationState::Completed;
                    if let Some(callback) = &self.callbacks.on_complete {
                        callback(self);
                    }
                }
            }
            LoopMode::PingPong => {
                state.is_reversed = !state.is_reversed;
                state.current_time = Duration::ZERO;
                state.loops_completed += 1;

                if let Some(callback) = &self.callbacks.on_loop {
                    callback(self, state.loops_completed);
                }
            }
        }
    }

    /// Restart animation for looping
    #[allow(dead_code)]
    fn restart_animation(&mut self, state: &mut AnimationRuntimeState) {
        state.current_time = Duration::ZERO;
        state.progress = 0.0;
        state.loops_completed += 1;

        if self.config.auto_reverse {
            state.is_reversed = !state.is_reversed;
        }

        if let Some(callback) = &self.callbacks.on_loop {
            callback(self, state.loops_completed);
        }
    }

    /// Get the current animation state
    ///
    /// # Returns
    /// The current `AnimationState` (Playing, Paused, Stopped, etc.)
    pub fn get_state(&self) -> AnimationState {
        self.state.read().unwrap().state
    }

    /// Get the current animation progress
    ///
    /// # Returns
    /// Progress value between 0.0 (start) and 1.0 (complete)
    pub fn get_progress(&self) -> f32 {
        self.state.read().unwrap().progress
    }

    /// Get the current interpolated animated values
    ///
    /// # Returns
    /// `Some(AnimatedValue)` if animation is active, `None` if stopped
    pub fn get_current_values(&self) -> Option<AnimatedValue> {
        self.state.read().unwrap().current_values.clone()
    }

    /// Check if the animation is currently playing
    ///
    /// # Returns
    /// `true` if animation is playing (forward or reverse), `false` otherwise
    pub fn is_playing(&self) -> bool {
        matches!(
            self.get_state(),
            AnimationState::Playing | AnimationState::Reversed
        )
    }

    /// Check if the animation has completed
    ///
    /// # Returns
    /// `true` if animation has finished all loops, `false` otherwise
    pub fn is_completed(&self) -> bool {
        self.get_state() == AnimationState::Completed
    }

    /// Set the animation speed multiplier
    ///
    /// # Arguments
    /// * `speed` - Speed multiplier (1.0 = normal, 2.0 = double speed, 0.5 = half speed)
    ///   Values less than 0.0 are clamped to 0.0
    pub fn set_speed(&mut self, speed: f32) {
        self.config.speed = speed.max(0.0);
    }

    /// Seek to a specific progress position in the animation
    ///
    /// # Arguments
    /// * `progress` - Target progress (0.0 to 1.0, values outside range are clamped)
    pub fn seek(&mut self, progress: f32) {
        let progress = progress.clamp(0.0, 1.0);
        let target_time = Duration::from_secs_f32(self.config.duration.as_secs_f32() * progress);

        if let Ok(mut state) = self.state.write() {
            state.current_time = target_time;
            state.progress = progress;
            state.current_values = Some(self.property.interpolate(progress));
        }
    }

    // Element conversion removed - incompatible with current Element struct
}

/// Builder for creating animations
pub struct AnimationBuilder {
    id: AnimationId,
    property: Option<AnimatedProperty>,
    config: AnimationConfig,
    callbacks: AnimationCallbacks,
}

impl AnimationBuilder {
    /// Create a new animation builder with the specified ID
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the animation
    ///
    /// # Returns
    /// A new `AnimationBuilder` instance with default configuration
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self {
            id: id.into(),
            property: None,
            config: AnimationConfig::default(),
            callbacks: AnimationCallbacks::default(),
        }
    }

    /// Set the property to animate
    ///
    /// # Arguments
    /// * `property` - The animated property (opacity, position, color, etc.)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn animate_property(mut self, property: AnimatedProperty) -> Self {
        self.property = Some(property);
        self
    }

    /// Set the animation duration
    ///
    /// # Arguments
    /// * `duration` - How long the animation should take to complete
    ///
    /// # Returns
    /// Self for method chaining
    pub fn duration(mut self, duration: Duration) -> Self {
        self.config.duration = duration;
        self
    }

    /// Set the easing function for the animation
    ///
    /// # Arguments
    /// * `easing` - The easing function to use (linear, ease-in, bounce, etc.)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn easing(mut self, easing: EasingFunction) -> Self {
        self.config.easing = easing;
        self
    }

    /// Set a delay before the animation starts
    ///
    /// # Arguments
    /// * `delay` - Time to wait before starting the animation
    ///
    /// # Returns
    /// Self for method chaining
    pub fn delay(mut self, delay: Duration) -> Self {
        self.config.delay = delay;
        self
    }

    /// Set the loop behavior for the animation
    ///
    /// # Arguments
    /// * `loop_mode` - How the animation should loop (none, infinite, count, ping-pong)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn loop_mode(mut self, loop_mode: LoopMode) -> Self {
        self.config.loop_mode = loop_mode;
        self
    }

    /// Set the animation speed multiplier
    ///
    /// # Arguments
    /// * `speed` - Speed multiplier (1.0 = normal, 2.0 = double speed, 0.5 = half speed)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn speed(mut self, speed: f32) -> Self {
        self.config.speed = speed;
        self
    }

    /// Enable or disable auto-play
    ///
    /// # Arguments
    /// * `auto_play` - Whether the animation should start playing immediately when built
    ///
    /// # Returns
    /// Self for method chaining
    pub fn auto_play(mut self, auto_play: bool) -> Self {
        self.config.auto_play = auto_play;
        self
    }

    /// Enable or disable auto-reverse
    ///
    /// # Arguments
    /// * `auto_reverse` - Whether the animation should automatically reverse direction on completion
    ///
    /// # Returns
    /// Self for method chaining
    pub fn auto_reverse(mut self, auto_reverse: bool) -> Self {
        self.config.auto_reverse = auto_reverse;
        self
    }

    /// Set a callback to be called when the animation starts
    ///
    /// # Arguments
    /// * `callback` - Function to call when animation begins playing
    ///
    /// # Returns
    /// Self for method chaining
    pub fn on_start<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Animation) + Send + Sync + 'static,
    {
        self.callbacks.on_start = Some(Arc::new(callback));
        self
    }

    /// Set a callback to be called on each animation frame update
    ///
    /// # Arguments
    /// * `callback` - Function to call with animation and current values on each frame
    ///
    /// # Returns
    /// Self for method chaining
    pub fn on_update<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Animation, &AnimatedValue) + Send + Sync + 'static,
    {
        self.callbacks.on_update = Some(Arc::new(callback));
        self
    }

    /// Set a callback to be called when the animation completes
    ///
    /// # Arguments
    /// * `callback` - Function to call when animation finishes (after all loops)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn on_complete<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Animation) + Send + Sync + 'static,
    {
        self.callbacks.on_complete = Some(Arc::new(callback));
        self
    }

    /// Set a callback to be called when the animation loops
    ///
    /// # Arguments
    /// * `callback` - Function to call with animation and loop count when looping
    ///
    /// # Returns
    /// Self for method chaining
    pub fn on_loop<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Animation, u32) + Send + Sync + 'static,
    {
        self.callbacks.on_loop = Some(Arc::new(callback));
        self
    }

    /// Build the final animation from the configured builder
    ///
    /// Creates an `Animation` instance with all the configured properties,
    /// callbacks, and settings. If auto-play is enabled, the animation
    /// will start playing immediately.
    ///
    /// # Returns
    /// A fully configured `Animation` instance ready for use
    pub fn build(self) -> Animation {
        let property = self.property.unwrap_or(AnimatedProperty::Opacity(0.0, 1.0));

        let mut animation = Animation {
            id: self.id,
            property,
            config: self.config,
            state: Arc::new(std::sync::RwLock::new(AnimationRuntimeState::default())),
            runtime: AnimationRuntime::default(),
            callbacks: self.callbacks,
            start_time: None,
        };

        if animation.config.auto_play {
            animation.play();
        }

        animation
    }
}

/// Animation Timeline for managing multiple animations
pub struct AnimationTimeline {
    /// Unique timeline identifier
    pub id: TimelineId,
    /// Animations in this timeline
    pub animations: Vec<Animation>,
    /// Timeline state
    pub state: Arc<std::sync::RwLock<AnimationState>>,
    /// Whether animations play in sequence or parallel
    pub sequential: bool,
    /// Current animation index (for sequential)
    current_index: usize,
}

impl AnimationTimeline {
    /// Create a new animation timeline
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the timeline
    /// * `sequential` - If true, animations play one after another; if false, they play in parallel
    ///
    /// # Returns
    /// A new `AnimationTimeline` instance
    pub fn new<S: Into<String>>(id: S, sequential: bool) -> Self {
        Self {
            id: id.into(),
            animations: Vec::new(),
            state: Arc::new(std::sync::RwLock::new(AnimationState::Stopped)),
            sequential,
            current_index: 0,
        }
    }

    /// Add an animation to the timeline
    ///
    /// # Arguments
    /// * `animation` - The animation to add to this timeline
    pub fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    /// Start playing the timeline
    ///
    /// For sequential timelines, starts the first animation.
    /// For parallel timelines, starts all animations simultaneously.
    pub fn play(&mut self) {
        *self.state.write().unwrap() = AnimationState::Playing;

        if self.sequential {
            if !self.animations.is_empty() {
                self.current_index = 0;
                self.animations[0].play();
            }
        } else {
            for animation in &mut self.animations {
                animation.play();
            }
        }
    }

    /// Update the timeline for one frame
    ///
    /// This should be called every frame in your main loop to advance all animations
    /// in the timeline. Handles sequential playback and completion detection.
    ///
    /// # Arguments
    /// * `delta_time` - Time elapsed since the last frame
    ///
    /// # Returns
    /// `true` if the timeline is still active, `false` if completed
    pub fn update(&mut self, delta_time: Duration) -> bool {
        let state = *self.state.read().unwrap();
        if state != AnimationState::Playing {
            return false;
        }

        if self.sequential {
            if self.current_index < self.animations.len() {
                let current_animation = &mut self.animations[self.current_index];

                if !current_animation.update(delta_time) && current_animation.is_completed() {
                    self.current_index += 1;

                    if self.current_index < self.animations.len() {
                        self.animations[self.current_index].play();
                    } else {
                        *self.state.write().unwrap() = AnimationState::Completed;
                        return false;
                    }
                }
            }
        } else {
            let mut any_playing = false;
            for animation in &mut self.animations {
                if animation.update(delta_time) {
                    any_playing = true;
                }
            }

            if !any_playing {
                *self.state.write().unwrap() = AnimationState::Completed;
                return false;
            }
        }

        true
    }

    /// Stop the timeline and all its animations
    ///
    /// Stops all animations in the timeline and resets the timeline state.
    /// For sequential timelines, resets the current animation index to 0.
    pub fn stop(&mut self) {
        *self.state.write().unwrap() = AnimationState::Stopped;
        self.current_index = 0;

        for animation in &mut self.animations {
            animation.stop();
        }
    }
}

/// Animation Manager for coordinating multiple animations
pub struct AnimationManager {
    /// All managed animations
    animations: HashMap<AnimationId, Animation>,
    /// All managed timelines
    timelines: HashMap<TimelineId, AnimationTimeline>,
    /// Last update time
    last_update: Instant,
}

impl AnimationManager {
    /// Create a new animation manager
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            timelines: HashMap::new(),
            last_update: get_monotonic_timer().base_time,
        }
    }

    /// Add an animation
    pub fn add_animation(&mut self, animation: Animation) -> AnimationId {
        let id = animation.id.clone();
        self.animations.insert(id.clone(), animation);
        id
    }

    /// Add a timeline
    pub fn add_timeline(&mut self, timeline: AnimationTimeline) {
        self.timelines.insert(timeline.id.clone(), timeline);
    }

    /// Remove an animation by ID
    pub fn remove_animation(&mut self, id: &AnimationId) {
        self.animations.remove(id);
    }

    /// Remove a timeline by ID
    pub fn remove_timeline(&mut self, id: &TimelineId) {
        self.timelines.remove(id);
    }

    /// Update all animations (call in main loop)
    pub fn update(&mut self) {
        let timer = get_monotonic_timer();
        let now = timer.base_time + timer.now();
        let delta_time = now.duration_since(self.last_update);
        self.last_update = now;

        // Update standalone animations
        self.animations.retain(|_, animation| {
            animation.update(delta_time);
            !animation.is_completed()
                || matches!(
                    animation.config.loop_mode,
                    LoopMode::Infinite | LoopMode::Count(_)
                )
        });

        // Update timelines
        for timeline in self.timelines.values_mut() {
            timeline.update(delta_time);
        }
    }

    /// Get animation by ID
    pub fn get_animation(&self, id: &str) -> Option<&Animation> {
        self.animations.get(id)
    }

    /// Get mutable animation by ID
    pub fn get_animation_mut(&mut self, id: &str) -> Option<&mut Animation> {
        self.animations.get_mut(id)
    }

    /// Remove completed, failed, or stuck animations
    pub fn cleanup_completed(&mut self) {
        // Remove completed animations
        self.animations
            .retain(|_, animation| !animation.is_completed());
        
        // Remove completed timelines, handling lock failures gracefully
        self.timelines.retain(|id, timeline| {
            match timeline.state.read() {
                Ok(state) => *state != AnimationState::Completed,
                Err(e) => {
                    // If we can't read the state, the lock is poisoned - remove it
                    eprintln!("Warning: Removing timeline {} due to poisoned lock: {}", id, e);
                    false
                }
            }
        });
    }
    
    /// Clean up all animations including failed/stuck ones
    /// Returns the number of animations cleaned up
    pub fn cleanup_all_stale(&mut self, stale_threshold: Duration) -> usize {
        let timer = get_monotonic_timer();
        let now = timer.base_time + timer.now();
        let mut removed = 0;
        
        // Remove animations that are completed or haven't updated recently
        self.animations.retain(|id, animation| {
            let should_keep = if animation.is_completed() {
                false
            } else if let Some(start_time) = animation.start_time {
                // Keep animations that started recently or are still progressing
                now.duration_since(start_time) < stale_threshold || 
                animation.runtime.last_frame_time.is_some_and(|t| 
                    now.duration_since(t) < stale_threshold
                )
            } else {
                true // Keep animations that haven't started yet
            };
            
            if !should_keep {
                removed += 1;
                #[cfg(debug_assertions)]
                eprintln!("Cleaning up stale animation: {}", id);
            }
            should_keep
        });
        
        // Remove stale timelines
        self.timelines.retain(|id, timeline| {
            let should_keep = match timeline.state.read() {
                Ok(state) => *state != AnimationState::Completed,
                Err(_) => {
                    removed += 1;
                    false // Remove poisoned timelines
                }
            };
            
            if !should_keep {
                #[cfg(debug_assertions)]
                eprintln!("Cleaning up stale timeline: {}", id);
            }
            should_keep
        });
        
        removed
    }

    /// Get active animation count
    pub fn active_count(&self) -> usize {
        self.animations.len() + self.timelines.len()
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common animations
/// Create a fade in animation
pub fn fade_in(id: impl Into<String>, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Opacity(0.0, 1.0))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a fade out animation
pub fn fade_out(id: impl Into<String>, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Opacity(1.0, 0.0))
        .duration(duration)
        .easing(EasingFunction::EaseIn)
        .build()
}

/// Create a slide in from left animation
pub fn slide_in_left(
    id: impl Into<String>,
    from_x: i16,
    to_x: i16,
    y: i16,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Position(from_x, y, to_x, y))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a bounce animation
pub fn bounce(id: impl Into<String>, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Scale(1.0, 1.2))
        .duration(duration)
        .easing(EasingFunction::Bounce)
        .loop_mode(LoopMode::PingPong)
        .build()
}

/// Create a pulse animation
pub fn pulse(id: impl Into<String>, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Opacity(1.0, 0.5))
        .duration(duration)
        .easing(EasingFunction::EaseInOut)
        .loop_mode(LoopMode::PingPong)
        .build()
}

// New convenience functions for anime.js-style animations

/// Create a translateX animation
pub fn translate_x(id: impl Into<String>, from: f32, to: f32, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::TranslateX(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a translateY animation
pub fn translate_y(id: impl Into<String>, from: f32, to: f32, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::TranslateY(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a scale animation
pub fn scale_animation(id: impl Into<String>, from: f32, to: f32, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::Scale(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a rotate animation
pub fn rotate_animation(
    id: impl Into<String>,
    from: f32,
    to: f32,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::Rotate(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a CSS property animation
pub fn css_property(
    id: impl Into<String>,
    property: &str,
    from: CssValue,
    to: CssValue,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::CssProperty(
            property.to_string(),
            from,
            to,
        ))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a numeric property animation
pub fn numeric_property(
    id: impl Into<String>,
    property: &str,
    from: f32,
    to: f32,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Property(property.to_string(), from, to))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a transform matrix animation
pub fn matrix_animation(
    id: impl Into<String>,
    from: TransformMatrix,
    to: TransformMatrix,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::Matrix(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

// Helper functions for creating CssValues
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

// Helper functions for creating AnimationValues
impl AnimationValue {
    /// Create a pixel animation value
    pub fn pixels(value: f32) -> Self {
        Self::Unit(value, "px".to_string())
    }
    /// Create a percentage animation value
    pub fn percentage(value: f32) -> Self {
        Self::Unit(value, "%".to_string())
    }
    /// Create an em unit animation value
    pub fn em(value: f32) -> Self {
        Self::Unit(value, "em".to_string())
    }
    /// Create a rem unit animation value
    pub fn rem(value: f32) -> Self {
        Self::Unit(value, "rem".to_string())
    }
    /// Create a unitless number animation value
    pub fn number(value: f32) -> Self {
        Self::Number(value)
    }
    /// Create a color animation value from RGB components
    pub fn color(r: u8, g: u8, b: u8) -> Self {
        Self::Color { r, g, b }
    }
    /// Create a string animation value
    pub fn string(value: &str) -> Self {
        Self::String(value.to_string())
    }
    /// Create an array animation value from a vector of floats
    pub fn array(values: Vec<f32>) -> Self {
        Self::Array(values)
    }
}

// Additional re-exports from submodules
pub use stagger::{
    stagger, stagger_builder, stagger_from_center, stagger_from_index, stagger_from_last,
    stagger_from_position, stagger_grid, stagger_grid_center, stagger_random, StaggerBuilder,
    StaggerDirection, StaggerOrigin,
};

pub use spring::{spring, spring_with_velocity};

pub use api::{
    animate, create_timeline, fade_in as api_fade_in, fade_out as api_fade_out, scale, slide,
    spring_animate, stagger_delay, AnimateParams, AnimationTargets, ColorValue, DelayValue,
    PositionValue, PropertyValue, SizeValue, StaggerOptions, TimelineBuilder, TimelineParams,
};

pub use performance::{
    AnimationBatch, BatchedUpdate, CacheStats, InterpolationCache, OptimizationLevel,
    OptimizedAnimationManager, PerformanceMetrics, PerformanceReport,
};
// Re-export debug types and functions for direct use
pub use debug::{
    create_debug_manager, create_performance_debug_manager, create_verbose_debug_manager,
    AnimationDebugInfo, AnimationDebugger, AnimationErrorType, AnimationSnapshot,
    DebugAnimationManager, DebugConfig, DebugEvent, DebugOverlay, DebugPerformanceMetrics,
    DebugReport, DebugVerbosity, PerformanceThresholds, PerformanceWarningType, StopReason,
    TimelineEventType, TimelineSnapshot,
};

// Re-export keyframe types and functions for direct use (rename to avoid conflicts)
pub use keyframes::{keyframes, KeyframeBuilder, KeyframeSequence, KeyframeValue};

// Convenience functions for keyframe animations

/// Create a keyframe-based animation property
pub fn keyframe_animation(sequence: keyframes::KeyframeSequence) -> AnimatedProperty {
    AnimatedProperty::Keyframes(sequence)
}

/// Create a fade in animation using keyframes
pub fn keyframe_fade_in(duration_ms: u64) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::fade_in(duration_ms))
}

/// Create a fade out animation using keyframes
pub fn keyframe_fade_out(duration_ms: u64) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::fade_out(duration_ms))
}

/// Create a slide in animation using keyframes
pub fn keyframe_slide_in(duration_ms: u64, distance: f32) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::slide_in_from_left(duration_ms, distance))
}

/// Create a bounce in animation using keyframes
pub fn keyframe_bounce_in(duration_ms: u64) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::bounce_in(duration_ms))
}

/// Create a pulse animation using keyframes
pub fn keyframe_pulse(duration_ms: u64) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::pulse(duration_ms))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_easing_function_linear() {
        let linear = EasingFunction::Linear;

        assert_eq!(linear.apply(0.0), 0.0);
        assert_eq!(linear.apply(0.5), 0.5);
        assert_eq!(linear.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_function_ease_in() {
        let ease_in = EasingFunction::EaseIn;

        assert_eq!(ease_in.apply(0.0), 0.0);
        assert_eq!(ease_in.apply(1.0), 1.0);

        // Ease-in should start slow
        let quarter = ease_in.apply(0.25);
        assert!(quarter < 0.25);
    }

    #[test]
    fn test_easing_function_ease_out() {
        let ease_out = EasingFunction::EaseOut;

        assert_eq!(ease_out.apply(0.0), 0.0);
        assert_eq!(ease_out.apply(1.0), 1.0);

        // Ease-out should start fast
        let quarter = ease_out.apply(0.25);
        assert!(quarter > 0.25);
    }

    #[test]
    fn test_easing_function_ease_in_out() {
        let ease_in_out = EasingFunction::EaseInOut;

        assert_eq!(ease_in_out.apply(0.0), 0.0);
        assert_eq!(ease_in_out.apply(0.5), 0.5);
        assert_eq!(ease_in_out.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_function_steps() {
        let steps = EasingFunction::Steps(4, false);

        assert_eq!(steps.apply(0.0), 0.0);
        assert_eq!(steps.apply(0.1), 0.0);
        assert_eq!(steps.apply(0.3), 0.25);
        assert_eq!(steps.apply(0.6), 0.5);
        assert_eq!(steps.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_function_steps_jump_start() {
        let steps = EasingFunction::Steps(4, true);

        // With jump_start=true, we should get the next step value immediately
        assert_eq!(steps.apply(0.0), 0.0); // Actually starts at 0, then jumps
        assert_eq!(steps.apply(0.1), 0.25); // First step
        assert_eq!(steps.apply(0.3), 0.5); // Second step
        assert_eq!(steps.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_function_linear_points() {
        let points = vec![0.0, 0.5, 1.0];
        let linear_points = EasingFunction::LinearPoints(points);

        assert_eq!(linear_points.apply(0.0), 0.0);
        assert_eq!(linear_points.apply(0.5), 0.5);
        assert_eq!(linear_points.apply(1.0), 1.0);
    }

    #[test]
    fn test_spring_config() {
        let spring = SpringConfig::new(1.0, 100.0, 10.0); // mass, stiffness, damping

        assert_eq!(spring.mass, 1.0);
        assert_eq!(spring.stiffness, 100.0);
        assert_eq!(spring.damping, 10.0);
        assert_eq!(spring.velocity, 0.0);

        // Test position calculation
        let pos = spring.calculate_position(0.1, 0.0, 1.0);
        assert!(pos >= 0.0 && pos <= 1.0);
    }

    #[test]
    fn test_spring_presets() {
        let gentle = SpringConfig::gentle();
        assert!(gentle.damping > 0.5);

        let wobbly = SpringConfig::wobbly();
        assert!(wobbly.stiffness > gentle.stiffness);

        let stiff = SpringConfig::stiff();
        assert!(stiff.stiffness > wobbly.stiffness);
    }

    #[test]
    fn test_animated_property_opacity() {
        let opacity = AnimatedProperty::Opacity(0.0, 1.0);

        let values = opacity.interpolate(0.5);
        if let AnimatedValue::Opacity(value) = values {
            assert_eq!(value, 0.5);
        } else {
            panic!("Expected opacity value");
        }
    }

    #[test]
    fn test_animated_property_size() {
        let size = AnimatedProperty::Size(10, 20, 100, 80); // from_width, from_height, to_width, to_height

        let values = size.interpolate(0.5);
        if let AnimatedValue::Size(width, height) = values {
            assert_eq!(width, 55);
            assert_eq!(height, 50);
        } else {
            panic!("Expected size value");
        }
    }

    #[test]
    fn test_animated_property_position() {
        let position = AnimatedProperty::Position(0, 0, 100, 50); // from_x, from_y, to_x, to_y

        let values = position.interpolate(0.5);
        if let AnimatedValue::Position(x, y) = values {
            assert_eq!(x, 50);
            assert_eq!(y, 25);
        } else {
            panic!("Expected position value");
        }
    }

    #[test]
    fn test_animated_property_scale() {
        let scale = AnimatedProperty::Scale(1.0, 2.0);

        let values = scale.interpolate(0.5);
        if let AnimatedValue::Scale(value) = values {
            assert_eq!(value, 1.5);
        } else {
            panic!("Expected scale value");
        }
    }

    #[test]
    fn test_animated_property_rotation() {
        let rotation = AnimatedProperty::Rotation(0.0, 360.0);

        let values = rotation.interpolate(0.5);
        if let AnimatedValue::Rotation(value) = values {
            assert_eq!(value, 180.0);
        } else {
            panic!("Expected rotation value");
        }
    }

    #[test]
    fn test_animated_property_color() {
        let color = AnimatedProperty::Color((255, 0, 0), (0, 255, 0)); // red to green

        let values = color.interpolate(0.5);
        if let AnimatedValue::Color { r, g, b } = values {
            assert_eq!(r, 127);
            assert_eq!(g, 127);
            assert_eq!(b, 0);
        } else {
            panic!("Expected color value");
        }
    }

    #[test]
    fn test_animation_creation() {
        let duration = Duration::from_millis(1000);
        let animation = Animation::new(duration, EasingFunction::Linear, Some(1), LoopMode::None);

        assert_eq!(animation.config.duration, duration);
        assert!(matches!(animation.config.easing, EasingFunction::Linear));
        assert!(matches!(animation.config.loop_mode, LoopMode::Count(1)));
    }

    #[test]
    fn test_animation_spring_creation() {
        let duration = Duration::from_millis(500);
        let spring_config = SpringConfig::gentle();
        let animation = Animation::spring(duration, spring_config);

        assert_eq!(animation.config.duration, duration);
        assert!(matches!(animation.config.easing, EasingFunction::Spring(_)));
    }
}
