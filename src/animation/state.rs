//! Animation state management types
//!
//! This module contains types related to animation playback state,
//! runtime state tracking, and animated value representation.

use super::properties::TransformMatrix;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Current animated value during animation
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LoopMode {
    /// Play once
    #[default]
    None,
    /// Loop indefinitely
    Infinite,
    /// Loop a specific number of times
    Count(u32),
    /// Ping-pong (forward then reverse)
    PingPong,
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

/// Animation value types (used in property animations)
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
    Map(HashMap<String, AnimationValue>),
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
