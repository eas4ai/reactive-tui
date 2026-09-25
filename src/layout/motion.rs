//! Owned motion settings shared by CSS parsing and App's animation clock.

use crate::animation::EasingFunction;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct CellTransform {
    pub x: f32,
    pub y: f32,
    pub x_percent: f32,
    pub y_percent: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub skew_x: f32,
    pub skew_y: f32,
    pub matrix: [f32; 6],
}

impl Default for CellTransform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            x_percent: 0.0,
            y_percent: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            skew_x: 0.0,
            skew_y: 0.0,
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) enum Transition {
    #[default]
    None,
    All,
    Colors,
    Opacity,
    Transform,
    Shadow,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct MotionStyle {
    pub transform: CellTransform,
    pub transition: Transition,
    pub duration: Option<Duration>,
    pub easing: Option<EasingFunction>,
}
