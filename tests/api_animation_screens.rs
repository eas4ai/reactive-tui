//! Acceptance checks for API-013. Screen and relative-value cases remain to be added.
use reactive_tui::animation::keyframes::{Keyframe, KeyframeAnimation, TypedKeyframe};
use std::time::Duration;

#[test]
fn typed_numeric_midpoint_interpolates() {
    let animation = KeyframeAnimation::from_typed(
        vec![
            TypedKeyframe {
                offset: 0.0,
                value: 0.0_f32,
                easing: None,
            },
            TypedKeyframe {
                offset: 1.0,
                value: 10.0_f32,
                easing: None,
            },
        ],
        Duration::from_secs(1),
    );
    assert_eq!(animation.get_value_at_time(0.5), Some(5.0));
}

#[test]
fn untyped_conversion_preserves_numeric_endpoints() {
    let animation = KeyframeAnimation::<f32>::new(vec![
        Keyframe::new(0.0).number("x", 4.0),
        Keyframe::new(1.0).number("x", 12.0),
    ]);
    assert_eq!(animation.get_value_at_time(0.0), Some(4.0));
    assert_eq!(animation.get_value_at_time(1.0), Some(12.0));
}
