Example from docs/ADAPTIVE_PERFORMANCE.md:34

```rust
use reactive_tui::hooks::fps::use_fps;
use reactive_tui::reactive::hooks::Hooks;

fn status_bar(hooks: &Hooks) -> String {
    let fps = use_fps(hooks);
    let s = fps.get();
    format!("FPS: {:.0}/{} | Render: {:.1}ms | Mode: {:?}", s.current_fps, s.target_fps, s.avg_render_time_ms, s.mode)
}
```

Example from docs/ADAPTIVE_PERFORMANCE.md:48

```rust
use reactive_tui::hooks::fps::use_performance;
use reactive_tui::reactive::hooks::Hooks;

fn adaptive_enabled(hooks: &Hooks) -> bool {
    let (_metrics, is_low_fps) = use_performance(hooks);
    !is_low_fps.get() // enable effects only when not low FPS
}
```

Example from docs/ADAPTIVE_PERFORMANCE.md:61

```rust
use reactive_tui::hooks::fps::use_performance_mode;
use reactive_tui::display::monitor::PerformanceMode;
use reactive_tui::reactive::hooks::{Hooks, use_effect};

fn enter_game_view(hooks: &Hooks) {
    let set_mode = use_performance_mode(hooks);
    use_effect(hooks, move || {
        set_mode(PerformanceMode::Gaming);
        Some(Box::new(move || set_mode(PerformanceMode::Balanced)))
    });
}
```

Example from docs/ADAPTIVE_PERFORMANCE.md:78

```rust
use reactive_tui::hooks::fps::use_frame_timing;
use reactive_tui::reactive::hooks::Hooks;

fn frame_stats(hooks: &Hooks) -> (f32, f32) {
    let timing = use_frame_timing(hooks);
    let t = timing.get();
    (t.last_frame_ms, t.target_frame_ms)
}
```

Example from docs/ADAPTIVE_PERFORMANCE.md:92

```rust
use reactive_tui::hooks::fps::{use_adaptive_quality, QualityLevel};
use reactive_tui::reactive::hooks::Hooks;

fn quality_class(hooks: &Hooks) -> &'static str {
    match use_adaptive_quality(hooks).get() {
        QualityLevel::Low => "transition-none",
        QualityLevel::Medium => "transition-all duration-100",
        QualityLevel::High => "transition-all duration-200",
    }
}
```
