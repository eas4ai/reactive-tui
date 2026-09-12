# Adaptive Performance and FPS Hooks

This document explains how the framework exposes live performance metrics to components and how you can adapt behavior based on FPS.

It covers:
- PerformanceContext: what it is and how it’s provided
- Hooks: use_fps, use_performance, use_performance_mode, use_frame_timing, use_adaptive_quality
- Fallback behavior when context is not available
- Requesting performance modes from components

## Overview

The App maintains an AdaptiveFpsManager that measures frame timing and drops/raises FPS as needed. Each frame, the App updates a PerformanceContext that provides:

- fps_state: ThreadSafeSignal<FpsState>
- metrics: ThreadSafeSignal<PerformanceMetrics>
- frame_timing: ThreadSafeSignal<FrameTiming>
- set_mode: Arc<dyn Fn(PerformanceMode) + Send + Sync>

Hooks read from this context to provide easy access to performance data. If the context isn’t available, hooks fall back to sensible defaults so your component code remains portable.

## Types

- PerformanceMode: PowerSave | Balanced | Performance | Gaming | Auto
- PerformanceMetrics: { current_fps: f32, avg_render_time_ms: f32, drop_rate_percent: f32, is_stable: bool }
- FpsState: { target_fps: u32, current_fps: f32, avg_render_time_ms: f32, drop_rate_percent: f32, is_stable: bool, mode: PerformanceMode }
- FrameTiming: { last_frame_ms: f32, target_frame_ms: f32, budget_remaining_ms: f32 }

## Using the hooks

### use_fps
Read live FPS state (target vs current, drop rate, stability, mode).

```rust
use reactive_tui::hooks::fps::use_fps;
use reactive_tui::reactive::hooks::Hooks;

fn status_bar(hooks: &Hooks) -> String {
    let fps = use_fps(hooks);
    let s = fps.get();
    format!("FPS: {:.0}/{} | Render: {:.1}ms | Mode: {:?}", s.current_fps, s.target_fps, s.avg_render_time_ms, s.mode)
}
```

### use_performance
Read aggregate metrics and a computed low_fps flag you can use for gating animations.

```rust
use reactive_tui::hooks::fps::use_performance;
use reactive_tui::reactive::hooks::Hooks;

fn adaptive_enabled(hooks: &Hooks) -> bool {
    let (_metrics, is_low_fps) = use_performance(hooks);
    !is_low_fps.get() // enable effects only when not low FPS
}
```

### use_performance_mode
Request a performance mode from inside a component (e.g., Gaming for a heavy view). The cleanup callback below explicitly restores Balanced; a mode request alone does not install restoration.

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

### use_frame_timing
Access last and target frame durations, useful for micro-profiling or budgeting work per frame.

```rust
use reactive_tui::hooks::fps::use_frame_timing;
use reactive_tui::reactive::hooks::Hooks;

fn frame_stats(hooks: &Hooks) -> (f32, f32) {
    let timing = use_frame_timing(hooks);
    let t = timing.get();
    (t.last_frame_ms, t.target_frame_ms)
}
```

### use_adaptive_quality
A convenience hook that returns a QualityLevel (Low/Medium/High) based on live FPS.

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

## How PerformanceContext is provided

Currently, the App updates a global PerformanceContext each frame so hooks can access live metrics anywhere. Internally this is done by calling set_global_performance_context once and then updating its signals every frame.

In a future step, we can scope this to the component tree using provide_context so you can swap or mock performance data locally for tests. Hooks already prefer a locally provided context over the global one.

## Fallback behavior

If no PerformanceContext is available, hooks fall back to reasonable defaults:
- use_fps: returns a signal initialized with FpsState::default()
- use_performance: returns PerformanceMetrics::default() and computes low_fps=false initially
- use_performance_mode: returns a no-op setter
- use_frame_timing: returns FrameTiming::default()

This lets you write components without worrying about wiring until you actually integrate into an App.

## Requesting modes and App integration

- Components call the setter returned by use_performance_mode
- The App reads and applies requested modes after presenting a frame
- AdaptiveFpsManager continues to adjust FPS based on real measurements while respecting explicit mode requests

## Best practices

- Use use_performance for coarse gating of effects/animations
- Use use_adaptive_quality when you want a simple Low/Med/High mapping
- Keep heavy work budgeted to t.target_frame_ms (from use_frame_timing) and yield if over budget
- Use a local PerformanceContext when isolating tests. The legacy global App publication path is included in the API-019 multi-App review.

## Troubleshooting

- Seeing defaults? Ensure your App is running the main loop and updating PerformanceContext each frame
- Mode requests not sticking? Confirm that App applies take_requested_performance_mode() after present
- Unit tests without runtime: hooks fallback should still work; for timer-dependent code, a fallback scheduler is provided


## Acceptance limits

These examples compile against the current hook signatures. App still publishes
the legacy global performance context, and frame mode reporting and multi-App
isolation remain under API-019 review. Local context preference is implemented;
it does not by itself prove independent App ownership. The supported-API matrix
keeps that runtime work pending rather than treating this guide as acceptance.
