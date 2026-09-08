//! Spring physics system for natural animations
//!
//! Provides physics-based animations using spring-damper systems for realistic motion.
//! Includes support for different damping ratios (underdamped, critically damped, overdamped)
//! and customizable spring parameters.

use super::EasingFunction;
use serde::{Deserialize, Serialize};

/// Configuration for spring physics calculations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpringConfig {
    /// Mass of the spring system (affects inertia)
    pub mass: f32,
    /// Spring stiffness (affects oscillation frequency)
    pub stiffness: f32,
    /// Damping factor (affects energy dissipation)
    pub damping: f32,
    /// Initial velocity of the system
    pub velocity: f32,
    /// Precision threshold for determining when spring has settled
    pub precision: f32,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            mass: 1.0,
            stiffness: 100.0,
            damping: 10.0,
            velocity: 0.0,
            precision: 0.01,
        }
    }
}

impl SpringConfig {
    /// Create a new spring configuration
    pub fn new(mass: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            mass: mass.max(0.1), // Prevent division by zero
            stiffness: stiffness.max(0.1),
            damping: damping.max(0.0),
            velocity: 0.0,
            precision: 0.01,
        }
    }

    /// Set initial velocity
    pub fn with_velocity(mut self, velocity: f32) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set precision threshold
    pub fn with_precision(mut self, precision: f32) -> Self {
        self.precision = precision.max(0.001);
        self
    }

    /// Calculate spring position at given time
    pub fn calculate_position(&self, time: f32, from: f32, to: f32) -> f32 {
        if time <= 0.0 {
            return from;
        }

        let displacement = to - from;
        if displacement.abs() < self.precision {
            return to;
        }

        let angular_frequency = (self.stiffness / self.mass).sqrt();
        let damping_ratio = self.damping / (2.0 * (self.mass * self.stiffness).sqrt());

        let position = if damping_ratio < 1.0 {
            // Underdamped oscillation
            self.calculate_underdamped(time, displacement, angular_frequency, damping_ratio)
        } else if damping_ratio == 1.0 {
            // Critically damped
            self.calculate_critically_damped(time, displacement, angular_frequency)
        } else {
            // Overdamped
            self.calculate_overdamped(time, displacement, angular_frequency, damping_ratio)
        };

        from + displacement - position
    }

    /// Calculate underdamped spring response
    fn calculate_underdamped(
        &self,
        time: f32,
        displacement: f32,
        angular_frequency: f32,
        damping_ratio: f32,
    ) -> f32 {
        let damped_frequency = angular_frequency * (1.0 - damping_ratio * damping_ratio).sqrt();
        let a = displacement;
        let b =
            (self.velocity + damping_ratio * angular_frequency * displacement) / damped_frequency;

        let envelope = (-damping_ratio * angular_frequency * time).exp();
        let oscillation = a * (damped_frequency * time).cos() + b * (damped_frequency * time).sin();

        envelope * oscillation
    }

    /// Calculate critically damped spring response
    fn calculate_critically_damped(
        &self,
        time: f32,
        displacement: f32,
        angular_frequency: f32,
    ) -> f32 {
        let a = displacement;
        let b = self.velocity + angular_frequency * displacement;

        (a + b * time) * (-angular_frequency * time).exp()
    }

    /// Calculate overdamped spring response
    fn calculate_overdamped(
        &self,
        time: f32,
        displacement: f32,
        angular_frequency: f32,
        damping_ratio: f32,
    ) -> f32 {
        let sqrt_term = (damping_ratio * damping_ratio - 1.0).sqrt();
        let r1 = -angular_frequency * (damping_ratio + sqrt_term);
        let r2 = -angular_frequency * (damping_ratio - sqrt_term);

        let a = (self.velocity - r2 * displacement) / (r1 - r2);
        let b = displacement - a;

        a * (r1 * time).exp() + b * (r2 * time).exp()
    }

    /// Calculate velocity at given time
    pub fn calculate_velocity(&self, time: f32, from: f32, to: f32) -> f32 {
        if time <= 0.0 {
            return self.velocity;
        }

        let displacement = to - from;
        if displacement.abs() < self.precision {
            return 0.0;
        }

        let angular_frequency = (self.stiffness / self.mass).sqrt();
        let damping_ratio = self.damping / (2.0 * (self.mass * self.stiffness).sqrt());

        if damping_ratio < 1.0 {
            // Underdamped
            let damped_frequency = angular_frequency * (1.0 - damping_ratio * damping_ratio).sqrt();
            let a = displacement;
            let b = (self.velocity + damping_ratio * angular_frequency * displacement)
                / damped_frequency;

            let envelope = (-damping_ratio * angular_frequency * time).exp();
            let envelope_derivative = -damping_ratio * angular_frequency * envelope;
            let oscillation =
                a * (damped_frequency * time).cos() + b * (damped_frequency * time).sin();
            let oscillation_derivative = damped_frequency
                * (-a * (damped_frequency * time).sin() + b * (damped_frequency * time).cos());

            -(envelope_derivative * oscillation + envelope * oscillation_derivative)
        } else if damping_ratio == 1.0 {
            // Critically damped
            let a = displacement;
            let b = self.velocity + angular_frequency * displacement;

            let exp_term = (-angular_frequency * time).exp();
            -(-angular_frequency * (a + b * time) + b) * exp_term
        } else {
            // Overdamped
            let sqrt_term = (damping_ratio * damping_ratio - 1.0).sqrt();
            let r1 = -angular_frequency * (damping_ratio + sqrt_term);
            let r2 = -angular_frequency * (damping_ratio - sqrt_term);

            let a = (self.velocity - r2 * displacement) / (r1 - r2);
            let b = displacement - a;

            -(a * r1 * (r1 * time).exp() + b * r2 * (r2 * time).exp())
        }
    }

    /// Estimate the total duration for the spring to settle
    pub fn estimate_duration(&self, from: f32, to: f32) -> f32 {
        let displacement = (to - from).abs();
        if displacement < self.precision {
            return 0.0;
        }

        let angular_frequency = (self.stiffness / self.mass).sqrt();
        let damping_ratio = self.damping / (2.0 * (self.mass * self.stiffness).sqrt());

        if damping_ratio < 1.0 {
            // Underdamped: estimate based on envelope decay
            let decay_constant = damping_ratio * angular_frequency;
            // Time for envelope to decay to precision level
            (-self.precision.ln() / decay_constant).max(0.0)
        } else {
            // Critically damped or overdamped: estimate based on exponential decay
            let decay_constant = angular_frequency * damping_ratio;
            (-self.precision.ln() / decay_constant).max(0.0)
        }
    }

    /// Check if spring has settled at given time
    pub fn is_settled(&self, time: f32, from: f32, to: f32) -> bool {
        let position = self.calculate_position(time, from, to);
        let velocity = self.calculate_velocity(time, from, to);

        (position - to).abs() < self.precision && velocity.abs() < self.precision
    }

    /// Create a spring easing function that can be used with the animation system
    pub fn to_easing_function(self) -> EasingFunction {
        EasingFunction::Spring(self)
    }
}

/// Preset spring configurations for common animation types
impl SpringConfig {
    /// Gentle spring with minimal overshoot
    pub fn gentle() -> Self {
        Self::new(1.0, 120.0, 14.0)
    }

    /// Wobbly spring with noticeable oscillation
    pub fn wobbly() -> Self {
        Self::new(1.0, 180.0, 12.0)
    }

    /// Stiff spring with quick response
    pub fn stiff() -> Self {
        Self::new(1.0, 400.0, 26.0)
    }

    /// Smooth spring with balanced response
    pub fn smooth() -> Self {
        Self::new(1.0, 200.0, 20.0)
    }

    /// Slow spring with gradual movement
    pub fn slow() -> Self {
        Self::new(1.0, 60.0, 15.0)
    }

    /// Bouncy spring with multiple oscillations
    pub fn bouncy() -> Self {
        Self::new(1.0, 200.0, 8.0)
    }

    /// No-overshoot spring (critically damped)
    pub fn no_overshoot() -> Self {
        Self::new(1.0, 100.0, 20.0) // Approximately critically damped
    }
}

/// Spring animation utility functions
pub fn spring(mass: f32, stiffness: f32, damping: f32) -> SpringConfig {
    SpringConfig::new(mass, stiffness, damping)
}

/// Create a spring with velocity
pub fn spring_with_velocity(
    mass: f32,
    stiffness: f32,
    damping: f32,
    velocity: f32,
) -> SpringConfig {
    SpringConfig::new(mass, stiffness, damping).with_velocity(velocity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_config_creation() {
        let spring = SpringConfig::new(1.0, 100.0, 10.0);
        assert_eq!(spring.mass, 1.0);
        assert_eq!(spring.stiffness, 100.0);
        assert_eq!(spring.damping, 10.0);
    }

    #[test]
    fn test_spring_position_calculation() {
        let spring = SpringConfig::new(1.0, 100.0, 10.0);

        // At t=0, should be at start position
        let pos_start = spring.calculate_position(0.0, 0.0, 100.0);
        assert!((pos_start - 0.0).abs() < 0.01);

        // Should move towards target
        let pos_mid = spring.calculate_position(0.1, 0.0, 100.0);
        assert!(pos_mid > 0.0 && pos_mid < 100.0);
    }

    #[test]
    fn test_spring_velocity_calculation() {
        let spring = SpringConfig::new(1.0, 100.0, 10.0).with_velocity(0.0);

        // At t=0, should have initial velocity
        let vel_start = spring.calculate_velocity(0.0, 0.0, 100.0);
        assert!((vel_start - 0.0).abs() < 0.01);

        // Should have positive velocity when moving towards target
        let vel_early = spring.calculate_velocity(0.01, 0.0, 100.0);
        assert!(vel_early > 0.0);
    }

    #[test]
    fn test_spring_duration_estimation() {
        let spring = SpringConfig::new(1.0, 100.0, 10.0);
        let duration = spring.estimate_duration(0.0, 100.0);

        // Should be a reasonable duration (not zero, not infinity)
        assert!(duration > 0.0 && duration < 10.0);
    }

    #[test]
    fn test_spring_settling() {
        let spring = SpringConfig::stiff();
        let duration = spring.estimate_duration(0.0, 100.0);

        // Test that duration estimation is reasonable
        assert!(duration > 0.0 && duration < 2.0);

        // Test basic settling mechanics - position should get close to target over time
        let early_pos = spring.calculate_position(duration * 0.1, 0.0, 100.0);
        let late_pos = spring.calculate_position(duration * 5.0, 0.0, 100.0);

        // Early should be moving towards target
        assert!(early_pos > 0.0 && early_pos < 200.0); // Allow for overshoot

        // Late should be much closer to target
        assert!((late_pos - 100.0).abs() < 10.0);

        // Velocity should decrease over time
        let early_vel = spring.calculate_velocity(duration * 0.1, 0.0, 100.0).abs();
        let late_vel = spring.calculate_velocity(duration * 5.0, 0.0, 100.0).abs();
        assert!(late_vel < early_vel);
    }

    #[test]
    fn test_damping_ratios() {
        // Underdamped (should oscillate)
        let underdamped = SpringConfig::new(1.0, 100.0, 5.0);
        let pos1 = underdamped.calculate_position(0.1, 0.0, 100.0);
        let pos2 = underdamped.calculate_position(0.2, 0.0, 100.0);
        let pos3 = underdamped.calculate_position(0.3, 0.0, 100.0);

        // Should show oscillation (not monotonic)
        assert!(pos1 < pos2 || pos2 < pos3 || pos1 > pos3);

        // Overdamped (should not overshoot)
        let overdamped = SpringConfig::new(1.0, 100.0, 50.0);
        let positions: Vec<f32> = (0..20)
            .map(|i| overdamped.calculate_position(i as f32 * 0.1, 0.0, 100.0))
            .collect();

        // Should be monotonically increasing and never overshoot
        for i in 1..positions.len() {
            assert!(positions[i] >= positions[i - 1]);
            assert!(positions[i] <= 100.0);
        }
    }

    #[test]
    fn test_preset_springs() {
        let gentle = SpringConfig::gentle();
        let wobbly = SpringConfig::wobbly();
        let stiff = SpringConfig::stiff();

        // Each should have different characteristics
        assert_ne!(gentle.stiffness, wobbly.stiffness);
        assert_ne!(wobbly.damping, stiff.damping);

        // All should be valid configurations
        assert!(gentle.mass > 0.0);
        assert!(wobbly.stiffness > 0.0);
        assert!(stiff.damping >= 0.0);
    }

    #[test]
    fn test_spring_easing_function() {
        let spring = SpringConfig::gentle();
        let expected_mass = spring.mass;
        let expected_stiffness = spring.stiffness;
        let expected_damping = spring.damping;
        let easing = spring.to_easing_function();

        match easing {
            EasingFunction::Spring(config) => {
                assert_eq!(config.mass, expected_mass);
                assert_eq!(config.stiffness, expected_stiffness);
                assert_eq!(config.damping, expected_damping);
            }
            other => {
                panic!("Expected Spring easing function, got: {:?}", other);
            }
        }
    }

    #[test]
    fn test_edge_cases() {
        // Zero displacement
        let spring = SpringConfig::default();
        let pos = spring.calculate_position(1.0, 50.0, 50.0);
        assert!((pos - 50.0).abs() < 0.01);

        // Negative time
        let pos_neg = spring.calculate_position(-1.0, 0.0, 100.0);
        assert!((pos_neg - 0.0).abs() < 0.01);

        // Very small precision
        let precise_spring = SpringConfig::default().with_precision(0.001);
        assert_eq!(precise_spring.precision, 0.001);
    }
}
