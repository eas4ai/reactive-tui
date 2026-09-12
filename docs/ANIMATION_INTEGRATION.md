# Screen transitions and animation

ScreenManager retains each screen's component tree, routes input to the active
screen and presents transitions through the complete Element painter. API-013
checks intermediate frames, endpoints, event delivery, keyframe interpolation and
relative values. [The mechanism](../.cairn/mechanisms/api-animation-screens.md) names
the exact tests; [paint properties](../.cairn/mechanisms/api-paint-properties.md)
cover gradients, keyed App animation and transformed hit bounds.

## Create and switch screens

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

For a timed transition, the caller drives `ScreenManager::update` from its event
loop and forwards input through `process_event`. A successful switch call starts
the transition; timing that call does not measure transition completion. The source
screen remains active until the transition completes.

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

Fade blends cell colors. Directional slides translate layers. Scale, flip and cube
use the approved terminal cell approximations; they do not rotate the host's glyph
bitmaps. The [composition decision](decisions/compose-screen-transitions-through-the-complete-element-painter.md)
records this contract and its input-geometry implications.

## Presets and easing

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

Screen easing (`screen::EasingFunction`) and property animation easing
(`animation::EasingFunction`) are separate public types. Import the one expected
by the configuration you are constructing.

## Property keyframes

```rust
use reactive_tui::animation::keyframes::{KeyframeAnimation, TypedKeyframe};
use std::time::Duration;

let motion = KeyframeAnimation::from_typed(vec![
    TypedKeyframe { offset: 0.0, value: 0.0_f32, easing: None },
    TypedKeyframe { offset: 1.0, value: 10.0_f32, easing: None },
], Duration::from_secs(1));
assert_eq!(motion.get_value_at_time(0.5), Some(5.0));
```

Keyframes calculate values. To affect a frame, use the retained App target/property
path verified by API-010 and API-013; constructing a value alone does not register
an application target. [App wakeups](app-wakeups.md) describes owned scheduling.

## Legacy transition metadata under review

The public builder retains animation IDs, custom properties and a hardware
acceleration preference:

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

The current screen renderer stores these fields but does not connect arbitrary
custom properties or IDs to the animation hooks, and the flag does not enable a
GPU renderer. The earlier guide promised that integration. API-019/020 must
reconcile those claims; this compiling metadata example is not acceptance of
them. Existing transition painting remains verified independently. No GPU window
is part of the agreed terminal-rendering direction.
