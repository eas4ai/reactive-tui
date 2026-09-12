Example from docs/ANIMATION_INTEGRATION.md:18

```rust
// New screen-specific easing functions
EasingFunction::SlideSmooth     // Perfect for screen slides with slight overshoot
EasingFunction::SnapBounce      // Quick snap with bounce - great for scale transitions
EasingFunction::SpringGentle    // Gentle spring motion - perfect for fades
EasingFunction::SpringDramatic  // Dramatic spring - attention-grabbing transitions

// Enhanced elastic functions
EasingFunction::EaseInElastic
EasingFunction::EaseOutElastic
EasingFunction::EaseInOutElastic
```

Example from docs/ANIMATION_INTEGRATION.md:35

```rust
// Get recommended easing functions for a transition type
let recommendations = EasingFunction::recommended_for_transition(TransitionType::Fade);
// Returns: [SpringGentle, EaseOutCubic, EaseInOutQuad]

let slide_recommendations = EasingFunction::recommended_for_transition(TransitionType::SlideLeft);
// Returns: [SlideSmooth, EaseOutBack, EaseInOutCubic]
```

Example from docs/ANIMATION_INTEGRATION.md:48

```rust
pub struct TransitionConfig {
    pub transition_type: TransitionType,
    pub duration: Duration,
    pub easing: EasingFunction,
    
    // Animation system integration
    pub animation_id: Option<String>,                    // For animation hooks
    pub use_hardware_acceleration: bool,                 // Performance optimization
    pub custom_properties: HashMap<String, f32>,         // Custom animation properties
}
```

Example from docs/ANIMATION_INTEGRATION.md:63

```rust
// Optimized presets for common scenarios
let smooth_fade = TransitionConfig::preset_smooth_fade()
    .with_animation_id("my-fade-transition")
    .with_custom_property("smoothness", 0.8);

let quick_slide = TransitionConfig::preset_quick_slide()
    .with_hardware_acceleration()
    .with_custom_property("slide_distance", 100.0);

let bouncy_scale = TransitionConfig::preset_bouncy_scale()
    .with_animation_id("scale-animation")
    .with_custom_property("bounce_intensity", 1.2);

let dramatic_flip = TransitionConfig::preset_dramatic_flip()
    .with_hardware_acceleration()
    .with_custom_property("flip_perspective", 0.8);
```

Example from docs/ANIMATION_INTEGRATION.md:88

```rust
// Animation ID for hook registration
let config = TransitionConfig::default()
    .with_animation_id("screen-transition-main-to-settings");

// Custom properties for animation system
let config = config
    .with_custom_property("spring_tension", 0.8)
    .with_custom_property("damping_ratio", 0.6)
    .with_custom_property("velocity_threshold", 0.01);

// Hardware acceleration flag
let config = config.with_hardware_acceleration();
```

Example from docs/ANIMATION_INTEGRATION.md:107

```rust
// The screen manager will automatically register animations with the hook system
screen_manager.switch_to_with_transition(
    ScreenId::new("settings"),
    Some(TransitionConfig::preset_smooth_fade()
        .with_animation_id("main-to-settings")
        .with_custom_property("hook_enabled", 1.0))
)?;

// Animation hooks can then:
// 1. Listen for "main-to-settings" animation events
// 2. Access custom properties for fine-tuning
// 3. Provide additional animation layers
// 4. Handle performance optimization
```

Example from docs/ANIMATION_INTEGRATION.md:127

```rust
let config = TransitionConfig::default()
    .with_hardware_acceleration(); // Enable GPU acceleration when available
```

Example from docs/ANIMATION_INTEGRATION.md:134

```rust
// Built-in performance analysis
let start_time = std::time::Instant::now();
screen_manager.switch_to_with_transition(screen_id, Some(config))?;
let elapsed = start_time.elapsed();
println!("Transition completed in: {:?}", elapsed);
```

Example from docs/ANIMATION_INTEGRATION.md:144

```rust
let config = TransitionConfig::default()
    .with_custom_property("performance_mode", 1.0)
    .with_custom_property("quality_level", 0.8)
    .with_custom_property("frame_rate_target", 60.0);
```

Example from docs/ANIMATION_INTEGRATION.md:155

```rust
// Get easing function description
let description = EasingFunction::SpringGentle.description();
// Returns: "Spring Gentle (Screen Optimized)"

// Test easing curve at different points
let easing = EasingFunction::EaseOutBounce;
for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
    println!("t={:.2}: {:.3}", t, easing.apply(t));
}
```

Example from docs/ANIMATION_INTEGRATION.md:169

```rust
let config = TransitionConfig::preset_bouncy_scale();
println!("Config: {}", config.description());
// Output: "Scale transition with Snap Bounce (Screen Optimized) easing over 400ms"
```

Example from docs/ANIMATION_INTEGRATION.md:179

```rust
// Use screen-optimized easing
screen_manager.switch_to_with_transition(
    ScreenId::new("settings"),
    Some(TransitionConfig {
        transition_type: TransitionType::SlideLeft,
        duration: Duration::from_millis(300),
        easing: EasingFunction::SlideSmooth,
        ..Default::default()
    })
)?;
```

Example from docs/ANIMATION_INTEGRATION.md:194

```rust
// Full animation system integration
let config = TransitionConfig::preset_dramatic_flip()
    .with_animation_id("main-menu-to-game")
    .with_hardware_acceleration()
    .with_custom_property("perspective_depth", 1.5)
    .with_custom_property("rotation_axis", 1.0)
    .with_custom_property("lighting_intensity", 0.8);

screen_manager.switch_to_with_transition(ScreenId::new("game"), Some(config))?;
```

Example from docs/ANIMATION_INTEGRATION.md:208

```rust
// Optimized for mobile/low-power devices
let mobile_config = TransitionConfig {
    transition_type: TransitionType::Fade,
    duration: Duration::from_millis(200),
    easing: EasingFunction::Linear,
    animation_id: Some("mobile-optimized".to_string()),
    use_hardware_acceleration: true,
    custom_properties: {
        let mut props = HashMap::new();
        props.insert("performance_mode".to_string(), 1.0);
        props.insert("quality_level".to_string(), 0.6);
        props
    },
};
```
