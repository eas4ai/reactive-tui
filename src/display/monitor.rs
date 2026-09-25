use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Real-time performance monitoring
pub struct PerformanceMonitor {
    frame_times: VecDeque<Duration>,
    render_times: VecDeque<Duration>,
    dropped_frames: u64,
    total_frames: u64,
    last_adjustment: Instant,
    adjustment_cooldown: Duration,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120), // 2 seconds at 60fps
            render_times: VecDeque::with_capacity(120),
            dropped_frames: 0,
            total_frames: 0,
            last_adjustment: Instant::now(),
            adjustment_cooldown: Duration::from_secs(2),
        }
    }

    /// Record frame performance metrics
    pub fn record_frame(&mut self, frame_time: Duration, render_time: Duration, dropped: bool) {
        // Keep sliding window of recent performance
        if self.frame_times.len() >= 120 {
            self.frame_times.pop_front();
            self.render_times.pop_front();
        }

        self.frame_times.push_back(frame_time);
        self.render_times.push_back(render_time);
        self.total_frames += 1;

        if dropped {
            self.dropped_frames += 1;
        }
    }

    /// Get current performance metrics
    pub fn get_current_performance(&self) -> PerformanceMetrics {
        if self.frame_times.is_empty() {
            return PerformanceMetrics::default();
        }

        let avg_frame_time =
            self.frame_times.iter().sum::<Duration>().as_secs_f32() / self.frame_times.len() as f32;
        let avg_render_time = self.render_times.iter().sum::<Duration>().as_secs_f32()
            / self.render_times.len() as f32;

        let current_fps = 1.0 / avg_frame_time;
        let drop_rate = self.dropped_frames as f32 / self.total_frames.max(1) as f32;

        PerformanceMetrics {
            current_fps,
            avg_render_time_ms: avg_render_time * 1000.0,
            drop_rate_percent: drop_rate * 100.0,
            is_stable: drop_rate < 0.05, // Less than 5% drops is stable
        }
    }

    /// Check if enough time has passed to allow FPS adjustment
    pub fn can_adjust(&self) -> bool {
        self.last_adjustment.elapsed() > self.adjustment_cooldown
    }

    /// Mark that an FPS adjustment was made
    pub fn mark_adjustment(&mut self) {
        self.last_adjustment = Instant::now();
    }

    /// Reset all performance monitoring data
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.render_times.clear();
        self.dropped_frames = 0;
        self.total_frames = 0;
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for display monitoring
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    /// Current frames per second
    pub current_fps: f32,
    /// Average render time in milliseconds
    pub avg_render_time_ms: f32,
    /// Frame drop rate as percentage
    pub drop_rate_percent: f32,
    /// Whether performance is stable
    pub is_stable: bool,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            current_fps: 60.0, // Assume good FPS initially
            avg_render_time_ms: 0.0,
            drop_rate_percent: 0.0,
            is_stable: true, // Assume stable initially
        }
    }
}

impl PerformanceMetrics {
    /// Check if performance is good enough for target FPS
    pub fn can_sustain_fps(&self, target_fps: u32) -> bool {
        self.is_stable
            && self.current_fps >= target_fps as f32 * 0.95 // Within 5% of target
            && self.drop_rate_percent < 5.0
    }

    /// Check if we should reduce FPS
    pub fn should_reduce_fps(&self) -> bool {
        !self.is_stable || self.drop_rate_percent > 10.0
    }

    /// Check if we can increase FPS
    pub fn can_increase_fps(&self, current_fps: u32, max_fps: u32) -> bool {
        self.is_stable
            && self.avg_render_time_ms < 5.0
            && current_fps < max_fps
            && self.drop_rate_percent < 2.0
    }
}

/// Performance modes for user selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PerformanceMode {
    /// Minimize CPU usage, 30 FPS
    PowerSave,
    /// Balanced experience, 60 FPS
    #[default]
    Balanced,
    /// Smooth animations, 90+ FPS
    Performance,
    /// Maximum FPS, lowest latency
    Gaming,
    /// Automatic adjustment based on content
    Auto,
}

impl PerformanceMode {
    /// Get the target FPS for this performance mode
    pub fn target_fps(&self) -> Option<u32> {
        match self {
            Self::PowerSave => Some(30),
            Self::Balanced => Some(60),
            Self::Performance => Some(90),
            Self::Gaming => Some(144),
            Self::Auto => None, // Determined dynamically
        }
    }

    /// Get the quality preference value (0.0 = performance, 1.0 = quality)
    pub fn quality_preference(&self) -> f32 {
        match self {
            Self::PowerSave => 0.2,   // Prefer performance
            Self::Balanced => 0.5,    // Balance
            Self::Performance => 0.7, // Prefer quality
            Self::Gaming => 0.9,      // Maximum quality
            Self::Auto => 0.6,        // Slight quality preference
        }
    }
}
