use crate::display::monitor::{PerformanceMetrics, PerformanceMode};
use crate::hooks::perf_context::{get_global_performance_context, PerformanceContext};
use crate::reactive::hooks::{use_context, use_effect, use_signal, Hooks, ThreadSafeSignal};
use std::sync::Arc;

/// FPS and performance state
#[derive(Clone, Debug, PartialEq)]
pub struct FpsState {
    /// Current target FPS
    pub target_fps: u32,
    /// Current actual FPS
    pub current_fps: f32,
    /// Average render time in milliseconds
    pub avg_render_time_ms: f32,
    /// Frame drop percentage
    pub drop_rate_percent: f32,
    /// Whether performance is stable
    pub is_stable: bool,
    /// Current performance mode
    pub mode: PerformanceMode,
}

impl Default for FpsState {
    fn default() -> Self {
        Self {
            target_fps: 60,
            current_fps: 60.0,
            avg_render_time_ms: 16.0,
            drop_rate_percent: 0.0,
            is_stable: true,
            mode: PerformanceMode::Balanced,
        }
    }
}

/// Hook for accessing FPS and performance information
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
///
/// #[component]
/// fn StatusBar(hooks: &Hooks) -> Element {
///     let state = use_fps(hooks).get();
///     Element::text(format!("FPS: {:.0}/{} | Render: {:.1}ms | Mode: {:?}",
///         state.current_fps, state.target_fps, state.avg_render_time_ms, state.mode))
/// }
/// ```
pub fn use_fps(hooks: &Hooks) -> ThreadSafeSignal<FpsState> {
    // Prefer a provided or global performance context; otherwise create local default
    if let Some(ctx) = use_context::<PerformanceContext>(hooks) {
        return ctx.fps_state;
    }
    if let Some(global) = get_global_performance_context() {
        return global.fps_state.clone();
    }
    use_signal(hooks, FpsState::default())
}

/// Hook for monitoring performance and adapting component behavior
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
///
/// #[component]
/// fn AnimatedComponent(hooks: &Hooks) -> Element {
///     let (_performance, low_fps) = use_performance(hooks);
///     let class = if low_fps.get() { "transition-none" } else { "transition-all duration-200" };
///     div().class(class).child(Element::text("Adaptive animations")).build()
/// }
/// ```
pub fn use_performance(
    hooks: &Hooks,
) -> (ThreadSafeSignal<PerformanceMetrics>, ThreadSafeSignal<bool>) {
    // Prefer live metrics from context if available
    if let Some(ctx) = use_context::<PerformanceContext>(hooks) {
        let metrics = ctx.metrics;
        let is_low_fps = use_signal(hooks, false);
        let metrics_clone = metrics.clone();
        let is_low_fps_clone = is_low_fps.clone();
        use_effect(hooks, move || {
            let current = metrics_clone.get();
            is_low_fps_clone.set(current.current_fps < 30.0 || !current.is_stable);
            None
        });
        return (metrics, is_low_fps);
    }
    if let Some(global) = get_global_performance_context() {
        let metrics = global.metrics.clone();
        let is_low_fps = use_signal(hooks, false);
        let metrics_clone = metrics.clone();
        let is_low_fps_clone = is_low_fps.clone();
        use_effect(hooks, move || {
            let current = metrics_clone.get();
            is_low_fps_clone.set(current.current_fps < 30.0 || !current.is_stable);
            None
        });
        return (metrics, is_low_fps);
    }

    let metrics = use_signal(hooks, PerformanceMetrics::default());
    let is_low_fps = use_signal(hooks, false);
    let metrics_clone = metrics.clone();
    let is_low_fps_clone = is_low_fps.clone();
    use_effect(hooks, move || {
        // Update low FPS flag based on metrics
        let current = metrics_clone.get();
        is_low_fps_clone.set(current.current_fps < 30.0 || !current.is_stable);
        None
    });
    (metrics, is_low_fps)
}

/// Hook for requesting a specific performance mode
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::display::monitor::PerformanceMode;
///
/// #[component]
/// fn GameView(hooks: &Hooks) -> Element {
///     let set_mode = use_performance_mode(hooks);
///     use_effect(hooks, move || {
///         set_mode(PerformanceMode::Gaming);
///         Some(Box::new(move || set_mode(PerformanceMode::Balanced)))
///     });
///     Element::text("Game view")
/// }
/// ```
pub fn use_performance_mode(hooks: &Hooks) -> Arc<dyn Fn(PerformanceMode) + Send + Sync> {
    if let Some(ctx) = use_context::<PerformanceContext>(hooks) {
        return ctx.set_mode.clone();
    }
    if let Some(global) = get_global_performance_context() {
        return global.set_mode.clone();
    }
    Arc::new(|_mode| {})
}

/// Hook for frame timing information
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
///
/// #[component]
/// fn TimingDebug(hooks: &Hooks) -> Element {
///     let timing = use_frame_timing(hooks).get();
///     Element::text(format!("Frame: {:.2}ms | Target: {:.2}ms",
///         timing.last_frame_ms, timing.target_frame_ms))
/// }
/// ```
pub fn use_frame_timing(hooks: &Hooks) -> ThreadSafeSignal<FrameTiming> {
    if let Some(ctx) = use_context::<PerformanceContext>(hooks) {
        return ctx.frame_timing;
    }
    if let Some(global) = get_global_performance_context() {
        return global.frame_timing.clone();
    }
    use_signal(hooks, FrameTiming::default())
}

/// Frame timing information for performance monitoring
#[derive(Clone, Debug, PartialEq)]
pub struct FrameTiming {
    /// Last frame duration in milliseconds
    pub last_frame_ms: f32,
    /// Target frame duration in milliseconds
    pub target_frame_ms: f32,
    /// Frame budget remaining (positive = under budget)
    pub budget_remaining_ms: f32,
}

impl Default for FrameTiming {
    fn default() -> Self {
        Self {
            last_frame_ms: 16.67,
            target_frame_ms: 16.67,
            budget_remaining_ms: 0.0,
        }
    }
}

/// Hook for adaptive quality settings based on performance
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::hooks::fps::QualityLevel;
///
/// #[component]
/// fn AdaptiveContent(hooks: &Hooks) -> Element {
///     let class = match use_adaptive_quality(hooks).get() {
///         QualityLevel::Low => "",
///         QualityLevel::Medium => "shadow-sm",
///         QualityLevel::High => "shadow-lg",
///     };
///     div().class(class).child(Element::text("Adaptive quality")).build()
/// }
/// ```
pub fn use_adaptive_quality(hooks: &Hooks) -> ThreadSafeSignal<QualityLevel> {
    let quality = use_signal(hooks, QualityLevel::High);

    // If we have live FPS, adapt quality; otherwise leave default
    if let Some(ctx) = use_context::<PerformanceContext>(hooks) {
        let quality_clone = quality.clone();
        use_effect(hooks, move || {
            let fps_state = ctx.fps_state.get();
            let new_quality = if fps_state.current_fps < 30.0 {
                QualityLevel::Low
            } else if fps_state.current_fps < 50.0 {
                QualityLevel::Medium
            } else {
                QualityLevel::High
            };
            quality_clone.set(new_quality);
            None
        });
    } else if let Some(global) = get_global_performance_context() {
        let quality_clone = quality.clone();
        use_effect(hooks, move || {
            let fps_state = global.fps_state.get();
            let new_quality = if fps_state.current_fps < 30.0 {
                QualityLevel::Low
            } else if fps_state.current_fps < 50.0 {
                QualityLevel::Medium
            } else {
                QualityLevel::High
            };
            quality_clone.set(new_quality);
            None
        });
    }

    quality
}

/// Quality level for rendering performance
#[derive(Clone, Debug, PartialEq, Default)]
pub enum QualityLevel {
    /// Low quality, prioritize performance
    Low,
    /// Medium quality, balanced performance
    #[default]
    Medium,
    /// High quality, prioritize visual fidelity
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps_state_default() {
        let state = FpsState::default();
        assert_eq!(state.target_fps, 60);
        assert_eq!(state.current_fps, 60.0);
        assert!(state.is_stable);
    }

    #[test]
    fn test_use_fps() {
        let hooks = Hooks::new();
        let fps = use_fps(&hooks);

        assert_eq!(fps.get().target_fps, 60);
        assert_eq!(fps.get().mode, PerformanceMode::Balanced);
    }

    #[test]
    fn test_use_performance() {
        let hooks = Hooks::new();
        let (metrics, is_low_fps) = use_performance(&hooks);

        assert_eq!(metrics.get().current_fps, 60.0); // Default metrics with good FPS
        assert!(!is_low_fps.get()); // Not low FPS initially
    }

    #[test]
    fn test_quality_level() {
        let hooks = Hooks::new();
        let quality = use_adaptive_quality(&hooks);

        // Should start at High quality
        assert_eq!(quality.get(), QualityLevel::High);
    }

    #[test]
    fn test_frame_timing() {
        let hooks = Hooks::new();
        let timing = use_frame_timing(&hooks);

        assert_eq!(timing.get().target_frame_ms, 16.67);
        assert_eq!(timing.get().last_frame_ms, 16.67);
    }
}
