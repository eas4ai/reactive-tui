Example from src/animation/api.rs:281

```rust,no_run
use reactive_tui::widgets::animation::*;

// Simple fade in
let fade = animate("my-element", AnimateParams {
    opacity: Some(PropertyValue::FromTo { from: 0.0, to: 1.0 }),
    duration: Some(500.0),
    easing: Some(EasingFunction::EaseOut),
    ..Default::default()
});

// Complex transform animation
let transform = animate("element", AnimateParams {
    translate_x: Some(PropertyValue::FromTo{from:0.0,to:100.0}),
    scale: Some(PropertyValue::FromTo { from: 0.8, to: 1.2 }),
    rotate: Some(PropertyValue::FromTo{from:0.0,to:360.0}),
    duration: Some(1000.0),
    easing: Some(EasingFunction::spring_wobbly()),
    ..Default::default()
});
```

Example from src/animation/api.rs:690

```rust,no_run
use reactive_tui::widgets::animation::*;

// Basic stagger with 100ms delay
let stagger_config = stagger_delay(100.0, None);

// Stagger from center with easing
let center_stagger = stagger_delay(150.0, Some(StaggerOptions {
    from: StaggerOrigin::Center,
    easing: Some(EasingFunction::EaseOut),
    ..Default::default()
}));
```

Example from src/animation/api.rs:858

```rust,no_run
use reactive_tui::widgets::animation::*;

let timeline = create_timeline(Some(TimelineParams {
    id: Some("main-timeline".to_string()),
    autoplay: Some(true),
    ..Default::default()
}))
.add("element1", AnimateParams {
    opacity: Some(PropertyValue::FromTo { from: 0.0, to: 1.0 }),
    duration: Some(500.0),
    ..Default::default()
}, None)
.add("element2", AnimateParams {
    translate_x: Some(PropertyValue::FromTo{from:0.0,to:100.0}),
    duration: Some(300.0),
    ..Default::default()
}, Some("-=200")) // Start 200ms before previous ends
.build();
```
