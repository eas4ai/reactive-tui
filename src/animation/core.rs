//! Core animation types and configuration
//!
//! This module contains the fundamental types for animation configuration
//! and lifecycle management.

use super::easing::EasingFunction;
use super::state::LoopMode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
