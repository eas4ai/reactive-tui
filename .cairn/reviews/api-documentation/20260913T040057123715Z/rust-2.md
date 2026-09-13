Example from docs/ANIMATION_INTEGRATION.md:12

```rust
use reactive_tui::backend::DebugBackend;
use reactive_tui::component::Element;
use reactive_tui::screen::{ScreenId, ScreenManager, TransitionConfig, TransitionType};

fn example() -> Result<(), String> {
    let mut manager = ScreenManager::new(Box::new(DebugBackend::new(80, 24)));
    manager.create_screen("home", "Home".into(), Element::text("Home"))?;
    manager.create_screen("settings", "Settings".into(), Element::text("Settings"))?;
    manager.switch_to_with_transition(ScreenId::new("settings"), Some(TransitionConfig {
        transition_type: TransitionType::None,
        ..Default::default()
    }))?;
    Ok(())
}
example().unwrap();
```

Example from docs/ANIMATION_INTEGRATION.md:35

```rust
use reactive_tui::screen::{EasingFunction, ScreenId, ScreenManager, TransitionConfig, TransitionType};
use std::time::Duration;

fn slide_to_settings(manager: &mut ScreenManager) -> Result<(), String> {
    manager.switch_to_with_transition(ScreenId::new("settings"), Some(TransitionConfig {
        transition_type: TransitionType::SlideLeft,
        duration: Duration::from_millis(300),
        easing: EasingFunction::SlideSmooth,
        ..Default::default()
    }))
}
```

Example from docs/ANIMATION_INTEGRATION.md:56

```rust
use reactive_tui::screen::{EasingFunction, TransitionConfig, TransitionType};

let presets = [
    TransitionConfig::preset_smooth_fade(),
    TransitionConfig::preset_quick_slide(),
    TransitionConfig::preset_bouncy_scale(),
    TransitionConfig::preset_dramatic_flip(),
];
for config in presets { println!("{}", config.description()); }
let choices = EasingFunction::recommended_for_transition(TransitionType::Fade);
assert!(!choices.is_empty());
let easing = EasingFunction::EaseOutBounce;
for fraction in [0.0, 0.25, 0.5, 0.75, 1.0] {
    println!("{fraction}: {}", easing.apply(fraction));
}
```

Example from docs/ANIMATION_INTEGRATION.md:80

```rust
use reactive_tui::animation::keyframes::{KeyframeAnimation, TypedKeyframe};
use std::time::Duration;

let motion = KeyframeAnimation::from_typed(vec![
    TypedKeyframe { offset: 0.0, value: 0.0_f32, easing: None },
    TypedKeyframe { offset: 1.0, value: 10.0_f32, easing: None },
], Duration::from_secs(1));
assert_eq!(motion.get_value_at_time(0.5), Some(5.0));
```

Example from docs/ANIMATION_INTEGRATION.md:100

```rust
use reactive_tui::screen::TransitionConfig;

let config = TransitionConfig::preset_smooth_fade()
    .with_animation_id("home-to-settings")
    .with_custom_property("smoothness", 0.8)
    .with_hardware_acceleration();
assert_eq!(config.animation_id.as_deref(), Some("home-to-settings"));
assert_eq!(config.custom_properties["smoothness"], 0.8);
assert!(config.use_hardware_acceleration);
```
