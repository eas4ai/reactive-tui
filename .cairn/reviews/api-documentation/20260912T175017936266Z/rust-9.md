Example from src/hooks/fps.rs:39

```rust,no_run
fn StatusBar(props: &Props, state: &mut State) -> Element {
    let fps = use_fps(&hooks);
    
    Element::text(format!(
        "FPS: {:.0}/{} | Render: {:.1}ms | Mode: {:?}",
        fps.get().current_fps,
        fps.get().target_fps,
        fps.get().avg_render_time_ms,
        fps.get().mode
    ))
}
```

Example from src/hooks/fps.rs:66

```rust,no_run
fn AnimatedComponent(props: &Props, state: &mut State) -> Element {
    let (performance, is_low_fps) = use_performance(&hooks);
    
    // Disable animations if FPS is low
    let animation_class = if is_low_fps.get() {
        "transition-none"
    } else {
        "transition-all duration-200"
    };
    
    Element::div()
        .class(animation_class)
        .child(text!("Adaptive animations"))
}
```

Example from src/hooks/fps.rs:127

```rust,no_run
fn GameView(props: &Props, state: &mut State) -> Element {
    let set_mode = use_performance_mode(&hooks);
    
    use_effect(&hooks, move || {
        // Request high performance for gaming
        set_mode(PerformanceMode::Gaming);
        
        // Cleanup: return to balanced mode
        Some(Box::new(move || {
            set_mode(PerformanceMode::Balanced);
        }))
    });
    
    Element::div()
        .class("game-container")
        .child(text!("High performance game"))
}
```

Example from src/hooks/fps.rs:159

```rust,no_run
fn TimingDebug(props: &Props, state: &mut State) -> Element {
    let timing = use_frame_timing(&hooks);
    
    Element::text(format!(
        "Frame: {:.2}ms | Target: {:.2}ms",
        timing.get().last_frame_ms,
        timing.get().target_frame_ms
    ))
}
```

Example from src/hooks/fps.rs:204

```rust,no_run
fn AdaptiveContent(props: &Props, state: &mut State) -> Element {
    let quality = use_adaptive_quality(&hooks);
    
    let shadow_class = match quality.get() {
        QualityLevel::Low => "",
        QualityLevel::Medium => "shadow-sm",
        QualityLevel::High => "shadow-lg",
    };
    
    Element::div()
        .class(format!("content {}", shadow_class))
        .child(text!("Adaptive quality content"))
}
```
