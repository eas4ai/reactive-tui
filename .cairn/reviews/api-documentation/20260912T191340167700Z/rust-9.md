Example from src/hooks/fps.rs:39

```rust,no_run
use reactive_tui::prelude::*;

#[component]
fn StatusBar(hooks: &Hooks) -> Element {
    let state = use_fps(hooks).get();
    Element::text(format!("FPS: {:.0}/{} | Render: {:.1}ms | Mode: {:?}",
        state.current_fps, state.target_fps, state.avg_render_time_ms, state.mode))
}
```

Example from src/hooks/fps.rs:63

```rust,no_run
use reactive_tui::prelude::*;

#[component]
fn AnimatedComponent(hooks: &Hooks) -> Element {
    let (_performance, low_fps) = use_performance(hooks);
    let class = if low_fps.get() { "transition-none" } else { "transition-all duration-200" };
    div().class(class).child(Element::text("Adaptive animations")).build()
}
```

Example from src/hooks/fps.rs:118

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::display::monitor::PerformanceMode;

#[component]
fn GameView(hooks: &Hooks) -> Element {
    let set_mode = use_performance_mode(hooks);
    use_effect(hooks, move || {
        set_mode(PerformanceMode::Gaming);
        Some(Box::new(move || set_mode(PerformanceMode::Balanced)))
    });
    Element::text("Game view")
}
```

Example from src/hooks/fps.rs:145

```rust,no_run
use reactive_tui::prelude::*;

#[component]
fn TimingDebug(hooks: &Hooks) -> Element {
    let timing = use_frame_timing(hooks).get();
    Element::text(format!("Frame: {:.2}ms | Target: {:.2}ms",
        timing.last_frame_ms, timing.target_frame_ms))
}
```

Example from src/hooks/fps.rs:189

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::hooks::fps::QualityLevel;

#[component]
fn AdaptiveContent(hooks: &Hooks) -> Element {
    let class = match use_adaptive_quality(hooks).get() {
        QualityLevel::Low => "",
        QualityLevel::Medium => "shadow-sm",
        QualityLevel::High => "shadow-lg",
    };
    div().class(class).child(Element::text("Adaptive quality")).build()
}
```
