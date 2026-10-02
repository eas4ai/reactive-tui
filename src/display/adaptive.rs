use super::capabilities::{
    ColorDepth, DisplayCapabilities, PerformanceProfile, SyncCapabilities, TerminalInfo,
};
use super::monitor::{PerformanceMetrics, PerformanceMode, PerformanceMonitor};
use crate::render::tree::RenderTree;
use futures_util::lock::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

const MAX_FPS_WITH_NONZERO_NANOSECOND_FRAME: u32 = 1_000_000_000;

/// Adaptive FPS manager that automatically adjusts refresh rate
pub struct AdaptiveFpsManager {
    /// Current target FPS
    target_fps: u32,
    /// Detected display capabilities
    capabilities: DisplayCapabilities,
    /// Performance monitoring
    performance_monitor: PerformanceMonitor,
    /// Adaptive settings
    config: AdaptiveConfig,
    /// Cached benchmark results
    benchmark_cache: Option<BenchmarkResults>,
}

/// Configuration for adaptive FPS and performance management
#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    /// Allow automatic FPS adjustment
    pub auto_adapt: bool,
    /// Minimum FPS to maintain
    pub min_fps: u32,
    /// Maximum FPS to attempt
    pub max_fps: u32,
    /// Performance vs quality preference (0.0 = performance, 1.0 = quality)
    pub quality_preference: f32,
    /// Enable power-saving mode
    pub power_save: bool,
    /// Performance mode
    pub mode: PerformanceMode,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            auto_adapt: true,
            min_fps: 30,
            max_fps: 144,
            quality_preference: 0.7, // Prefer quality but not at extreme cost
            power_save: false,
            mode: PerformanceMode::Balanced,
        }
    }
}

#[derive(Debug, Clone)]
struct BenchmarkResults {
    #[allow(dead_code)]
    avg_render_time_us: f32,
    #[allow(dead_code)]
    max_viable_fps: u32,
    timestamp: Instant,
}

impl Default for AdaptiveFpsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveFpsManager {
    /// Create a new adaptive FPS manager with automatic capability detection
    pub fn new() -> Self {
        let capabilities = Self::detect_display_capabilities();
        let recommended_fps = capabilities.calculate_recommended_fps();

        Self {
            target_fps: recommended_fps,
            capabilities,
            performance_monitor: PerformanceMonitor::new(),
            config: AdaptiveConfig::default(),
            benchmark_cache: None,
        }
    }

    /// Create with custom configuration.
    ///
    /// A zero minimum is raised to one. Values above one billion are lowered so
    /// a frame still spans at least one nanosecond. A reversed maximum is raised
    /// to the normalized minimum.
    pub fn with_config(mut config: AdaptiveConfig) -> Self {
        config.min_fps = config
            .min_fps
            .clamp(1, MAX_FPS_WITH_NONZERO_NANOSECOND_FRAME);
        config.max_fps = config
            .max_fps
            .clamp(config.min_fps, MAX_FPS_WITH_NONZERO_NANOSECOND_FRAME);
        let capabilities = Self::detect_display_capabilities();
        let recommended_fps = capabilities
            .calculate_recommended_fps()
            .clamp(config.min_fps, config.max_fps);

        // Override with performance mode if set
        let target_fps = config.mode.target_fps().unwrap_or(recommended_fps);

        Self {
            target_fps,
            capabilities,
            performance_monitor: PerformanceMonitor::new(),
            config,
            benchmark_cache: None,
        }
    }

    /// Detect terminal and system display capabilities
    fn detect_display_capabilities() -> DisplayCapabilities {
        let terminal_info = TerminalInfo::detect();

        // Create estimated profiles without actual benchmarking
        // Real benchmarking would be done lazily on first render
        let performance_profile = PerformanceProfile {
            avg_render_time_us: 1000.0, // 1ms estimate
            high_fps_capable: terminal_info.has_gpu_acceleration,
            cpu_efficiency: 0.8,
            memory_efficiency: 0.9,
        };

        let sync_capabilities = SyncCapabilities {
            smooth_updates: terminal_info.supports_high_refresh,
            max_update_rate: if terminal_info.supports_high_refresh {
                120
            } else {
                60
            },
            input_latency_ms: 16.0,
        };

        let mut capabilities = DisplayCapabilities {
            max_fps: 60,         // Will be calculated
            recommended_fps: 60, // Will be calculated
            terminal_info,
            performance_profile,
            sync_capabilities,
        };

        // Calculate actual FPS limits
        capabilities.max_fps = capabilities.calculate_max_fps();
        capabilities.recommended_fps = capabilities.calculate_recommended_fps();

        capabilities
    }

    /// Benchmark rendering performance (lazy, cached)
    pub fn benchmark_if_needed(&mut self, tree: &RenderTree) {
        // Check if we have recent benchmark results
        if let Some(ref cache) = self.benchmark_cache {
            if cache.timestamp.elapsed() < Duration::from_secs(60) {
                return; // Use cached results
            }
        }

        // Simple benchmark: measure tree traversal time
        let _start = Instant::now();
        let mut render_times = Vec::new();

        for _ in 0..10 {
            let iter_start = Instant::now();
            // Simulate rendering by traversing the tree
            self.simulate_render(tree);
            render_times.push(iter_start.elapsed());
        }

        let avg_render_time = render_times
            .iter()
            .map(|d| d.as_micros() as f32)
            .sum::<f32>()
            / render_times.len() as f32;

        // Update performance profile
        self.capabilities.performance_profile.avg_render_time_us = avg_render_time;
        self.capabilities.performance_profile.high_fps_capable = avg_render_time < 2000.0;

        // Cache results
        self.benchmark_cache = Some(BenchmarkResults {
            avg_render_time_us: avg_render_time,
            max_viable_fps: self.capabilities.calculate_max_fps(),
            timestamp: Instant::now(),
        });

        // Update target FPS based on new benchmark
        if self.config.auto_adapt && self.config.mode == PerformanceMode::Auto {
            let new_recommended = self.capabilities.calculate_recommended_fps();
            self.target_fps = new_recommended.clamp(self.config.min_fps, self.config.max_fps);
        }
    }

    /// Simulate rendering for benchmarking
    fn simulate_render(&self, tree: &RenderTree) {
        fn walk_node(node: &dyn crate::render::tree::RenderNode, depth: usize, budget: &mut u64) {
            if depth > 64 {
                return;
            } // safety cap
              // Simulate some layout/paint work proportional to children
            *budget += 1;
            for child in node.children() {
                walk_node(child.as_ref(), depth + 1, budget);
            }
        }
        if let Some(root) = tree.root() {
            let mut budget: u64 = 0;
            walk_node(root, 0, &mut budget);
            // Prevent optimizer from removing the loop work
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            let _ = budget;
        }
    }

    pub(crate) fn performance_mode(&self) -> PerformanceMode {
        self.config.mode
    }

    /// Get current target FPS
    pub fn get_target_fps(&self) -> u32 {
        self.target_fps
    }

    /// Get target frame duration
    pub fn get_frame_duration(&self) -> Duration {
        Duration::from_nanos(1_000_000_000 / self.target_fps as u64)
    }

    /// Record frame performance and potentially adjust FPS
    pub fn record_frame_performance(
        &mut self,
        frame_time: Duration,
        render_time: Duration,
        dropped: bool,
    ) {
        self.performance_monitor
            .record_frame(frame_time, render_time, dropped);

        if self.config.auto_adapt
            && self.config.mode == PerformanceMode::Auto
            && self.performance_monitor.can_adjust()
        {
            self.adaptive_adjustment();
        }
    }

    /// Perform adaptive FPS adjustment based on performance
    fn adaptive_adjustment(&mut self) {
        let metrics = self.performance_monitor.get_current_performance();

        // Adjustment logic
        if metrics.should_reduce_fps() {
            // Too many dropped frames - reduce FPS
            let new_fps = (self.target_fps as f32 * 0.8) as u32;
            self.target_fps = new_fps.clamp(self.config.min_fps, self.target_fps);
            self.performance_monitor.mark_adjustment();
        } else if metrics.can_increase_fps(self.target_fps, self.config.max_fps) {
            // Very stable and fast - try increasing FPS
            let new_fps = (self.target_fps as f32 * 1.2) as u32;
            self.target_fps = new_fps.min(self.config.max_fps);
            self.performance_monitor.mark_adjustment();
        }
    }

    /// Manually set target FPS (disables auto-adaptation)
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps.clamp(self.config.min_fps, self.config.max_fps);
        self.config.auto_adapt = false;
    }

    /// Set performance mode
    pub fn set_performance_mode(&mut self, mode: PerformanceMode) {
        self.config.mode = mode;

        if let Some(target) = mode.target_fps() {
            self.set_target_fps(target);
        } else {
            // Auto mode
            self.config.auto_adapt = true;
            self.target_fps = self.capabilities.calculate_recommended_fps();
        }

        self.config.quality_preference = mode.quality_preference();
    }

    /// Get display capabilities
    pub fn get_capabilities(&self) -> &DisplayCapabilities {
        &self.capabilities
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_monitor.get_current_performance()
    }

    /// Enable/disable auto-adaptation
    pub fn set_auto_adapt(&mut self, enabled: bool) {
        self.config.auto_adapt = enabled;
    }

    /// Get recommendation summary for user
    pub fn get_recommendation_summary(&self) -> String {
        format!(
            "Display Analysis:\n  \
            Terminal: {} ({})\n  \
            Connection: {:?}\n  \
            Max FPS: {}\n  \
            Recommended: {} FPS\n  \
            Current Target: {} FPS\n  \
            Mode: {:?}\n  \
            GPU Acceleration: {}\n  \
            High Refresh Support: {}",
            self.capabilities
                .terminal_info
                .program
                .as_deref()
                .unwrap_or("Unknown"),
            match self.capabilities.terminal_info.color_depth {
                ColorDepth::TrueColor => "TrueColor",
                ColorDepth::Color256 => "256 Color",
                ColorDepth::Color16 => "16 Color",
                ColorDepth::Monochrome => "Monochrome",
            },
            self.capabilities.terminal_info.connection_type,
            self.capabilities.max_fps,
            self.capabilities.recommended_fps,
            self.target_fps,
            self.config.mode,
            if self.capabilities.terminal_info.has_gpu_acceleration {
                "Yes"
            } else {
                "No"
            },
            if self.capabilities.terminal_info.supports_high_refresh {
                "Yes"
            } else {
                "No"
            }
        )
    }

    /// Reset performance monitoring
    pub fn reset_monitoring(&mut self) {
        self.performance_monitor.reset();
    }
}

/// Thread-safe wrapper for use in async contexts
pub struct AsyncAdaptiveFpsManager {
    inner: Arc<Mutex<AdaptiveFpsManager>>,
}

impl Default for AsyncAdaptiveFpsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncAdaptiveFpsManager {
    /// Create a new async adaptive FPS manager
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AdaptiveFpsManager::new())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AdaptiveConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AdaptiveFpsManager::with_config(config))),
        }
    }

    /// Get current target FPS
    pub async fn get_target_fps(&self) -> u32 {
        self.inner.lock().await.get_target_fps()
    }

    /// Get target frame duration
    pub async fn get_frame_duration(&self) -> Duration {
        self.inner.lock().await.get_frame_duration()
    }

    /// Record frame performance and potentially adjust FPS
    pub async fn record_frame_performance(
        &self,
        frame_time: Duration,
        render_time: Duration,
        dropped: bool,
    ) {
        self.inner
            .lock()
            .await
            .record_frame_performance(frame_time, render_time, dropped);
    }

    /// Set performance mode
    pub async fn set_performance_mode(&self, mode: PerformanceMode) {
        self.inner.lock().await.set_performance_mode(mode);
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.inner.lock().await.get_performance_metrics()
    }

    /// Get recommendation summary for user
    pub async fn get_recommendation_summary(&self) -> String {
        self.inner.lock().await.get_recommendation_summary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::ConnectionType;

    #[test]
    fn test_adaptive_fps_manager_creation() {
        let manager = AdaptiveFpsManager::new();
        assert!(manager.get_target_fps() >= 30);
        assert!(manager.get_target_fps() <= 240);
    }

    #[test]
    fn test_frame_duration_calculation() {
        let mut manager = AdaptiveFpsManager::new();
        manager.set_target_fps(60);

        let frame_duration = manager.get_frame_duration();
        let expected = Duration::from_nanos(1_000_000_000 / 60);
        assert_eq!(frame_duration, expected);
    }

    #[test]
    fn test_performance_mode_switching() {
        let mut manager = AdaptiveFpsManager::new();

        manager.set_performance_mode(PerformanceMode::PowerSave);
        assert_eq!(manager.get_target_fps(), 30);

        manager.set_performance_mode(PerformanceMode::Gaming);
        assert!(manager.get_target_fps() >= 90);
    }

    #[test]
    fn test_terminal_detection() {
        let info = TerminalInfo::detect();
        // Should not panic and should provide reasonable defaults
        assert!(matches!(
            info.connection_type,
            ConnectionType::Local
                | ConnectionType::SSH
                | ConnectionType::Tmux
                | ConnectionType::Unknown
        ));
    }
}
