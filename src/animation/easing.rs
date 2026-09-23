use super::spring::SpringConfig;
use serde::{Deserialize, Serialize};

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

    // Advanced easing functions
    /// Spring physics-based easing
    Spring(SpringConfig),
    /// Stepped easing with a positive step count.
    /// `true` jumps at zero and each boundary; `false` jumps at interval ends.
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
}

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

    /// Create power-in easing with custom exponent
    pub fn power_in(power: f32) -> Self {
        Self::InPower(power)
    }

    /// Create power-out easing with custom exponent
    pub fn power_out(power: f32) -> Self {
        Self::OutPower(power)
    }

    /// Create power-in-out easing with custom exponent
    pub fn power_in_out(power: f32) -> Self {
        Self::InOutPower(power)
    }

    /// Create back-in easing with custom overshoot
    ///
    /// # Arguments
    /// * `overshoot` - The amount of overshoot (typically around 1.70158)
    pub fn back_in(overshoot: f32) -> Self {
        Self::InBack(overshoot)
    }

    /// Create back-out easing with custom overshoot
    ///
    /// # Arguments
    /// * `overshoot` - The amount of overshoot (typically around 1.70158)
    pub fn back_out(overshoot: f32) -> Self {
        Self::OutBack(overshoot)
    }

    /// Create back-in-out easing with custom overshoot
    ///
    /// # Arguments
    /// * `overshoot` - The amount of overshoot (typically around 1.70158)
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

    /// Apply the easing function to a normalized time value (0.0 to 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Linear => t,
            Self::EaseIn => self.ease_in_quad(t),
            Self::EaseOut => self.ease_out_quad(t),
            Self::EaseInOut => self.ease_in_out_quad(t),
            Self::CubicBezier(x1, y1, x2, y2) => self.cubic_bezier(t, *x1, *y1, *x2, *y2),
            Self::Bounce => self.ease_out_bounce(t),
            Self::Elastic => self.ease_out_elastic(t),
            Self::Back => self.ease_in_out_back(t),
            Self::Expo => self.ease_in_out_expo(t),
            Self::Circ => self.ease_in_out_circ(t),
            Self::Sine => self.ease_in_out_sine(t),
            Self::Quad => self.ease_in_out_quad(t),
            Self::Cubic => self.ease_in_out_cubic(t),
            Self::Quart => self.ease_in_out_quart(t),
            Self::Quint => self.ease_in_out_quint(t),
            Self::Spring(config) => config.calculate_position(t, 0.0, 1.0),
            Self::Steps(steps, jump_start) => self.apply_steps(t, *steps, *jump_start),
            Self::LinearPoints(points) => self.apply_linear_points(t, points),
            Self::Irregular(steps, randomness) => self.apply_irregular(t, *steps, *randomness),
            Self::InPower(power) => t.powf(*power),
            Self::OutPower(power) => 1.0 - (1.0 - t).powf(*power),
            Self::InOutPower(power) => {
                if t < 0.5 {
                    (2.0 * t).powf(*power) / 2.0
                } else {
                    1.0 - (2.0 * (1.0 - t)).powf(*power) / 2.0
                }
            }
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
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0) / 2.0
                }
            }
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

    // Basic easing functions
    fn ease_in_quad(&self, t: f32) -> f32 {
        t * t
    }

    fn ease_out_quad(&self, t: f32) -> f32 {
        1.0 - (1.0 - t) * (1.0 - t)
    }

    fn ease_in_out_quad(&self, t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
        }
    }

    fn ease_in_out_cubic(&self, t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    fn ease_in_out_quart(&self, t: f32) -> f32 {
        if t < 0.5 {
            8.0 * t * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
        }
    }

    fn ease_in_out_quint(&self, t: f32) -> f32 {
        if t < 0.5 {
            16.0 * t * t * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
        }
    }

    fn ease_in_out_sine(&self, t: f32) -> f32 {
        -(t * std::f32::consts::PI).cos() / 2.0 + 0.5
    }

    fn ease_in_out_expo(&self, t: f32) -> f32 {
        if t == 0.0 {
            0.0
        } else if t == 1.0 {
            1.0
        } else if t < 0.5 {
            2.0_f32.powf(20.0 * t - 10.0) / 2.0
        } else {
            (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
        }
    }

    fn ease_in_out_circ(&self, t: f32) -> f32 {
        if t < 0.5 {
            (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
        } else {
            ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
        }
    }

    fn ease_in_out_back(&self, t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C2: f32 = C1 * 1.525;

        if t < 0.5 {
            ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
        } else {
            ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) + 2.0) / 2.0
        }
    }

    fn ease_out_elastic(&self, t: f32) -> f32 {
        const C4: f32 = 2.0 * std::f32::consts::PI / 3.0;

        if t == 0.0 {
            0.0
        } else if t == 1.0 {
            1.0
        } else {
            2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
        }
    }

    fn ease_out_bounce(&self, t: f32) -> f32 {
        const N1: f32 = 7.5625;
        const D1: f32 = 2.75;

        if t < 1.0 / D1 {
            N1 * t * t
        } else if t < 2.0 / D1 {
            let t = t - 1.5 / D1;
            N1 * t * t + 0.75
        } else if t < 2.5 / D1 {
            let t = t - 2.25 / D1;
            N1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / D1;
            N1 * t * t + 0.984375
        }
    }

    fn cubic_bezier(&self, t: f32, _x1: f32, y1: f32, _x2: f32, y2: f32) -> f32 {
        // Simplified cubic bezier approximation
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        mt3 * 0.0 + 3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3 * 1.0
    }

    fn apply_steps(&self, t: f32, steps: u32, jump_start: bool) -> f32 {
        let step_size = 1.0 / steps as f32;
        let current_step = (t / step_size).floor();

        if jump_start {
            ((current_step + 1.0) * step_size).min(1.0)
        } else {
            (current_step * step_size).min(1.0)
        }
    }

    fn apply_linear_points(&self, t: f32, points: &[f32]) -> f32 {
        if points.is_empty() {
            return t;
        }

        let segment_size = 1.0 / (points.len() - 1) as f32;
        let segment = (t / segment_size).floor() as usize;

        if segment >= points.len() - 1 {
            return points[points.len() - 1];
        }

        let local_t = (t % segment_size) / segment_size;
        let from = points[segment];
        let to = points[segment + 1];

        from + (to - from) * local_t
    }

    fn apply_irregular(&self, t: f32, steps: u32, randomness: f32) -> f32 {
        let step_size = 1.0 / steps as f32;
        let current_step = (t / step_size).floor();

        // Add pseudo-random variation
        let variation = ((current_step * 12.9898).sin() * 43_758.547).fract() * randomness;

        ((current_step * step_size) + variation * step_size).clamp(0.0, 1.0)
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

/// Convenience function to ease a value between start and end
pub fn ease_value(t: f32, from: f32, to: f32, easing: &EasingFunction) -> f32 {
    let eased_t = easing.apply(t);
    from + (to - from) * eased_t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_easing() {
        let easing = EasingFunction::Linear;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_in_out() {
        let easing = EasingFunction::EaseInOut;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(1.0), 1.0);
        // Middle value should be around 0.5
        assert!((easing.apply(0.5) - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_steps_easing() {
        let easing = EasingFunction::Steps(4, false);
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.24), 0.0);
        assert_eq!(easing.apply(0.26), 0.25);
        assert_eq!(easing.apply(0.99), 0.75);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_value() {
        let result = ease_value(0.5, 0.0, 100.0, &EasingFunction::Linear);
        assert_eq!(result, 50.0);

        let result = ease_value(1.0, 10.0, 20.0, &EasingFunction::Linear);
        assert_eq!(result, 20.0);
    }
}
