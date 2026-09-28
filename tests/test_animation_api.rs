//! Test that animation API is fully accessible
use reactive_tui::animation::*;

#[test]
fn test_animation_api_accessibility() {
    // Core types
    let _id: AnimationId = String::from("test");
    let _tid: TimelineId = String::from("timeline");

    // State types
    let _state = AnimationState::Stopped;
    let _loop = LoopMode::None;

    // Property types
    let _prop = AnimatedProperty::Opacity(0.0, 1.0);
    let _css = CssValue::pixels(10.0);
    let _val = AnimationValue::pixels(10.0);

    // Config types
    let config = AnimationConfig::default();
    let _spring = SpringConfig::gentle();

    // Easing
    let ease = EasingFunction::Linear;

    // Builder
    let _builder = Animation::builder("test");

    // Manager
    let _manager = AnimationManager::new();

    // Functions
    let _ = fade_in("test", std::time::Duration::from_secs(1));
    let _ = translate_x("test", 0.0, 100.0, std::time::Duration::from_secs(1));
    let _ = keyframe_fade_in(1000);

    assert_eq!(config.loop_mode, LoopMode::None);
    assert_eq!(config.speed, 1.0);
    assert_eq!(ease.apply(0.5), 0.5);
}
