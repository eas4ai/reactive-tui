//! Animation Debugging Tools
//!
//! Comprehensive debugging utilities for the animation system including
//! performance monitoring, state inspection, timeline visualization, and
//! debugging overlays for development.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{
    AnimatedValue, Animation, AnimationId, AnimationManager, AnimationState, AnimationTimeline,
    EasingFunction, TimelineId,
};

/// Animation debugging configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable state logging
    pub enable_state_logging: bool,
    /// Enable visual debugging overlays
    pub enable_visual_debug: bool,
    /// Enable timeline visualization
    pub enable_timeline_debug: bool,
    /// Maximum number of debug entries to keep
    pub max_debug_entries: usize,
    /// Debug output verbosity level
    pub verbosity_level: DebugVerbosity,
    /// Whether to log to console
    pub log_to_console: bool,
    /// Whether to save debug data to file
    pub save_to_file: bool,
    /// Debug file path
    pub debug_file_path: String,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            enable_state_logging: true,
            enable_visual_debug: false,
            enable_timeline_debug: true,
            max_debug_entries: 1000,
            verbosity_level: DebugVerbosity::Medium,
            log_to_console: true,
            save_to_file: false,
            debug_file_path: "animation_debug.log".to_string(),
        }
    }
}

/// Debug verbosity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugVerbosity {
    /// Only critical errors and warnings
    Low,
    /// Standard debugging information
    Medium,
    /// Detailed debugging with all events
    High,
    /// Extremely verbose with frame-by-frame data
    Verbose,
}

/// Animation debug event types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DebugEvent {
    /// Animation started
    AnimationStarted {
        /// Unique identifier for the animation
        id: AnimationId,
        /// Time when the animation started
        timestamp: Duration,
        /// Debug configuration information
        config: AnimationDebugInfo,
    },
    /// Animation updated
    AnimationUpdated {
        /// Unique identifier for the animation
        id: AnimationId,
        /// Time when the update occurred
        timestamp: Duration,
        /// Current animation progress (0.0 to 1.0)
        progress: f32,
        /// Current interpolated values
        current_values: Option<AnimatedValue>,
        /// Time taken for this frame
        frame_time: Duration,
    },
    /// Animation completed
    AnimationCompleted {
        /// Unique identifier for the animation
        id: AnimationId,
        /// Time when the animation completed
        timestamp: Duration,
        /// Total time the animation ran
        total_duration: Duration,
        /// Number of loops completed
        loops_completed: u32,
    },
    /// Animation paused
    AnimationPaused {
        /// Unique identifier for the animation
        id: AnimationId,
        /// Time when the animation was paused
        timestamp: Duration,
        /// Progress when paused (0.0 to 1.0)
        progress: f32,
    },
    /// Animation stopped
    AnimationStopped {
        /// Unique identifier for the animation
        id: AnimationId,
        /// Time when the animation was stopped
        timestamp: Duration,
        /// Reason for stopping
        reason: StopReason,
    },
    /// Timeline event
    TimelineEvent {
        /// Unique identifier for the timeline
        timeline_id: TimelineId,
        /// Type of timeline event
        event_type: TimelineEventType,
        /// Time when the event occurred
        timestamp: Duration,
        /// Number of animations in the timeline
        animation_count: usize,
    },
    /// Performance warning
    PerformanceWarning {
        /// Time when the warning occurred
        timestamp: Duration,
        /// Type of performance warning
        warning_type: PerformanceWarningType,
        /// Additional details about the warning
        details: String,
    },
    /// Error occurred
    Error {
        /// Time when the error occurred
        timestamp: Duration,
        /// Type of animation error
        error_type: AnimationErrorType,
        /// Error message
        message: String,
        /// Optional animation ID associated with the error
        animation_id: Option<AnimationId>,
    },
}

/// Simplified animation configuration for debugging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationDebugInfo {
    /// Animation duration
    pub duration: Duration,
    /// Easing function used
    pub easing: EasingFunction,
    /// Delay before animation starts
    pub delay: Duration,
    /// Whether animation starts automatically
    pub auto_play: bool,
    /// Animation playback speed multiplier
    pub speed: f32,
}

/// Reasons for animation stopping
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopReason {
    /// Manually stopped by user
    Manual,
    /// Completed naturally
    Completed,
    /// Error occurred
    Error,
    /// Cleanup/garbage collection
    Cleanup,
}

/// Timeline event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimelineEventType {
    /// Timeline was started
    Started,
    /// Timeline was paused
    Paused,
    /// Timeline was stopped
    Stopped,
    /// Animation was added to timeline
    AnimationAdded,
    /// Animation was removed from timeline
    AnimationRemoved,
    /// Timeline sequence advanced to next step
    SequenceAdvanced,
}

/// Performance warning types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceWarningType {
    /// Frame time exceeded threshold
    SlowFrame,
    /// Too many active animations
    TooManyAnimations,
    /// Memory usage high
    HighMemoryUsage,
    /// Inefficient easing function
    InefficientEasing,
    /// Large interpolation delta
    LargeInterpolationDelta,
}

/// Animation error types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationErrorType {
    /// Invalid configuration
    InvalidConfig,
    /// Property interpolation failed
    InterpolationError,
    /// Timeline synchronization error
    TimelineError,
    /// Callback execution error
    CallbackError,
    /// Resource allocation error
    ResourceError,
}

/// Performance metrics for debugging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugPerformanceMetrics {
    /// Average frame time
    pub avg_frame_time: Duration,
    /// Maximum frame time
    pub max_frame_time: Duration,
    /// Minimum frame time
    pub min_frame_time: Duration,
    /// Total animations processed
    pub total_animations: u64,
    /// Active animations count
    pub active_animations: usize,
    /// Memory usage estimate (bytes)
    pub memory_usage: usize,
    /// Cache hit rate
    pub cache_hit_rate: f32,
    /// Interpolation operations per second
    pub interpolations_per_second: f32,
}

impl Default for DebugPerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_frame_time: Duration::ZERO,
            max_frame_time: Duration::ZERO,
            min_frame_time: Duration::MAX,
            total_animations: 0,
            active_animations: 0,
            memory_usage: 0,
            cache_hit_rate: 0.0,
            interpolations_per_second: 0.0,
        }
    }
}

/// Animation state snapshot for debugging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationSnapshot {
    /// Unique identifier for the animation
    pub id: AnimationId,
    /// Current animation state
    pub state: AnimationState,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Current elapsed time
    pub current_time: Duration,
    /// Number of loops completed
    pub loops_completed: u32,
    /// Whether animation is currently reversed
    pub is_reversed: bool,
    /// Current interpolated values
    pub current_values: Option<AnimatedValue>,
    /// Animation configuration
    pub config: AnimationDebugInfo,
}

/// Timeline state snapshot for debugging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineSnapshot {
    /// Unique identifier for the timeline
    pub id: TimelineId,
    /// Current timeline state
    pub state: AnimationState,
    /// Whether animations run sequentially
    pub sequential: bool,
    /// Current animation index in sequence
    pub current_index: usize,
    /// Total number of animations
    pub animation_count: usize,
    /// Snapshots of all animations in timeline
    pub animations: Vec<AnimationSnapshot>,
}

/// Main animation debugger
pub struct AnimationDebugger {
    /// Debug configuration
    config: DebugConfig,
    /// Debug event history
    events: Vec<DebugEvent>,
    /// Performance metrics
    performance: DebugPerformanceMetrics,
    /// Start time for relative timestamps
    start_time: Instant,
    /// Frame time tracking
    frame_times: Vec<Duration>,
    /// Animation snapshots
    snapshots: HashMap<AnimationId, Vec<AnimationSnapshot>>,
    /// Timeline snapshots
    timeline_snapshots: HashMap<TimelineId, Vec<TimelineSnapshot>>,
    /// Performance warning thresholds
    performance_thresholds: PerformanceThresholds,
}

/// Performance warning thresholds
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceThresholds {
    /// Maximum acceptable frame time
    pub max_frame_time: Duration,
    /// Maximum number of active animations
    pub max_active_animations: usize,
    /// Maximum memory usage (bytes)
    pub max_memory_usage: usize,
    /// Minimum acceptable cache hit rate
    pub min_cache_hit_rate: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_frame_time: Duration::from_millis(16), // 60 FPS
            max_active_animations: 100,
            max_memory_usage: 50 * 1024 * 1024, // 50 MB
            min_cache_hit_rate: 0.8,            // 80%
        }
    }
}

impl AnimationDebugger {
    /// Create a new animation debugger
    pub fn new(config: DebugConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
            performance: DebugPerformanceMetrics::default(),
            start_time: Instant::now(),
            frame_times: Vec::new(),
            snapshots: HashMap::new(),
            timeline_snapshots: HashMap::new(),
            performance_thresholds: PerformanceThresholds::default(),
        }
    }

    /// Log a debug event
    pub fn log_event(&mut self, event: DebugEvent) {
        if self.events.len() >= self.config.max_debug_entries {
            self.events.remove(0);
        }

        if self.config.log_to_console {
            self.log_to_console(&event);
        }

        self.events.push(event);
    }

    /// Log animation start
    pub fn log_animation_start(&mut self, animation: &Animation) {
        if !self.config.enable_state_logging {
            return;
        }

        let event = DebugEvent::AnimationStarted {
            id: animation.id.clone(),
            timestamp: self.start_time.elapsed(),
            config: AnimationDebugInfo {
                duration: animation.config.duration,
                easing: animation.config.easing.clone(),
                delay: animation.config.delay,
                auto_play: animation.config.auto_play,
                speed: animation.config.speed,
            },
        };

        self.log_event(event);
    }

    /// Log animation update
    pub fn log_animation_update(&mut self, animation: &Animation, frame_time: Duration) {
        if !self.config.enable_state_logging {
            return;
        }

        if matches!(
            self.config.verbosity_level,
            DebugVerbosity::High | DebugVerbosity::Verbose
        ) {
            let state = match animation.state.read() {
                Ok(guard) => guard.clone(),
                Err(_) => {
                    log::warn!("Animation state lock poisoned during debug update");
                    return;
                }
            };
            let event = DebugEvent::AnimationUpdated {
                id: animation.id.clone(),
                timestamp: self.start_time.elapsed(),
                progress: state.progress,
                current_values: state.current_values.clone(),
                frame_time,
            };

            self.log_event(event);
        }

        // Update performance metrics
        if self.config.enable_performance_monitoring {
            self.update_performance_metrics(frame_time);
        }
    }

    /// Log animation completion
    pub fn log_animation_complete(&mut self, animation: &Animation) {
        if !self.config.enable_state_logging {
            return;
        }

        let state = match animation.state.read() {
            Ok(guard) => guard.clone(),
            Err(_) => {
                log::warn!("Animation state lock poisoned during completion debug");
                return;
            }
        };
        let event = DebugEvent::AnimationCompleted {
            id: animation.id.clone(),
            timestamp: self.start_time.elapsed(),
            total_duration: state.current_time,
            loops_completed: state.loops_completed,
        };

        self.log_event(event);
    }

    /// Log animation pause
    pub fn log_animation_pause(&mut self, animation: &Animation) {
        if !self.config.enable_state_logging {
            return;
        }

        let state = match animation.state.read() {
            Ok(guard) => guard.clone(),
            Err(_) => {
                log::warn!("Animation state lock poisoned during pause debug");
                return;
            }
        };
        let event = DebugEvent::AnimationPaused {
            id: animation.id.clone(),
            timestamp: self.start_time.elapsed(),
            progress: state.progress,
        };

        self.log_event(event);
    }

    /// Log animation stop
    pub fn log_animation_stop(&mut self, animation: &Animation, reason: StopReason) {
        if !self.config.enable_state_logging {
            return;
        }

        let event = DebugEvent::AnimationStopped {
            id: animation.id.clone(),
            timestamp: self.start_time.elapsed(),
            reason,
        };

        self.log_event(event);
    }

    /// Log timeline event
    pub fn log_timeline_event(
        &mut self,
        timeline: &AnimationTimeline,
        event_type: TimelineEventType,
    ) {
        if !self.config.enable_timeline_debug {
            return;
        }

        let event = DebugEvent::TimelineEvent {
            timeline_id: timeline.id.clone(),
            event_type,
            timestamp: self.start_time.elapsed(),
            animation_count: timeline.animations.len(),
        };

        self.log_event(event);
    }

    /// Log performance warning
    pub fn log_performance_warning(
        &mut self,
        warning_type: PerformanceWarningType,
        details: String,
    ) {
        if !self.config.enable_performance_monitoring {
            return;
        }

        let event = DebugEvent::PerformanceWarning {
            timestamp: self.start_time.elapsed(),
            warning_type,
            details,
        };

        self.log_event(event);
    }

    /// Log error
    pub fn log_error(
        &mut self,
        error_type: AnimationErrorType,
        message: String,
        animation_id: Option<AnimationId>,
    ) {
        let event = DebugEvent::Error {
            timestamp: self.start_time.elapsed(),
            error_type,
            message,
            animation_id,
        };

        self.log_event(event);
    }

    /// Take a snapshot of an animation's current state
    pub fn take_animation_snapshot(&mut self, animation: &Animation) {
        if !self.config.enable_state_logging {
            return;
        }

        let state = match animation.state.read() {
            Ok(guard) => guard.clone(),
            Err(_) => {
                log::warn!("Animation state lock poisoned during snapshot");
                return;
            }
        };
        let snapshot = AnimationSnapshot {
            id: animation.id.clone(),
            state: state.state,
            progress: state.progress,
            current_time: state.current_time,
            loops_completed: state.loops_completed,
            is_reversed: state.is_reversed,
            current_values: state.current_values.clone(),
            config: AnimationDebugInfo {
                duration: animation.config.duration,
                easing: animation.config.easing.clone(),
                delay: animation.config.delay,
                auto_play: animation.config.auto_play,
                speed: animation.config.speed,
            },
        };

        self.snapshots
            .entry(animation.id.clone())
            .or_default()
            .push(snapshot);

        // Limit snapshot history
        if let Some(snapshots) = self.snapshots.get_mut(&animation.id) {
            if snapshots.len() > self.config.max_debug_entries / 10 {
                snapshots.remove(0);
            }
        }
    }

    /// Take a snapshot of a timeline's current state
    pub fn take_timeline_snapshot(&mut self, timeline: &AnimationTimeline) {
        if !self.config.enable_timeline_debug {
            return;
        }

        let animation_snapshots: Vec<AnimationSnapshot> = timeline
            .animations
            .iter()
            .filter_map(|animation| {
                let state = match animation.state.read() {
                    Ok(guard) => guard.clone(),
                    Err(_) => {
                        log::warn!("Animation state lock poisoned during timeline snapshot");
                        return None;
                    }
                };
                Some(AnimationSnapshot {
                    id: animation.id.clone(),
                    state: state.state,
                    progress: state.progress,
                    current_time: state.current_time,
                    loops_completed: state.loops_completed,
                    is_reversed: state.is_reversed,
                    current_values: state.current_values.clone(),
                    config: AnimationDebugInfo {
                        duration: animation.config.duration,
                        easing: animation.config.easing.clone(),
                        delay: animation.config.delay,
                        auto_play: animation.config.auto_play,
                        speed: animation.config.speed,
                    },
                })
            })
            .collect();

        let timeline_state = match timeline.state.read() {
            Ok(guard) => *guard,
            Err(_) => {
                log::warn!("Timeline state lock poisoned during snapshot");
                return;
            }
        };

        let snapshot = TimelineSnapshot {
            id: timeline.id.clone(),
            state: timeline_state,
            sequential: timeline.sequential,
            current_index: timeline.current_index,
            animation_count: timeline.animations.len(),
            animations: animation_snapshots,
        };

        self.timeline_snapshots
            .entry(timeline.id.clone())
            .or_default()
            .push(snapshot);

        // Limit snapshot history
        if let Some(snapshots) = self.timeline_snapshots.get_mut(&timeline.id) {
            if snapshots.len() > self.config.max_debug_entries / 10 {
                snapshots.remove(0);
            }
        }
    }

    /// Update performance metrics
    fn update_performance_metrics(&mut self, frame_time: Duration) {
        self.frame_times.push(frame_time);

        // Keep only recent frame times
        if self.frame_times.len() > 60 {
            // Last 60 frames
            self.frame_times.remove(0);
        }

        // Update metrics
        if !self.frame_times.is_empty() {
            let total_time: Duration = self.frame_times.iter().sum();
            self.performance.avg_frame_time = total_time / self.frame_times.len() as u32;
            self.performance.max_frame_time =
                self.frame_times.iter().max().copied().unwrap_or_default();
            self.performance.min_frame_time =
                self.frame_times.iter().min().copied().unwrap_or_default();
        }

        // Check for performance warnings
        self.check_performance_warnings(frame_time);
    }

    /// Check for performance warnings
    fn check_performance_warnings(&mut self, frame_time: Duration) {
        if frame_time > self.performance_thresholds.max_frame_time {
            self.log_performance_warning(
                PerformanceWarningType::SlowFrame,
                format!(
                    "Frame time {}ms exceeds threshold {}ms",
                    frame_time.as_millis(),
                    self.performance_thresholds.max_frame_time.as_millis()
                ),
            );
        }

        if self.performance.active_animations > self.performance_thresholds.max_active_animations {
            self.log_performance_warning(
                PerformanceWarningType::TooManyAnimations,
                format!(
                    "Active animations ({}) exceeds threshold ({})",
                    self.performance.active_animations,
                    self.performance_thresholds.max_active_animations
                ),
            );
        }

        if self.performance.memory_usage > self.performance_thresholds.max_memory_usage {
            self.log_performance_warning(
                PerformanceWarningType::HighMemoryUsage,
                format!(
                    "Memory usage ({} bytes) exceeds threshold ({} bytes)",
                    self.performance.memory_usage, self.performance_thresholds.max_memory_usage
                ),
            );
        }

        if self.performance.cache_hit_rate < self.performance_thresholds.min_cache_hit_rate {
            self.log_performance_warning(
                PerformanceWarningType::InefficientEasing,
                format!(
                    "Cache hit rate ({:.2}) below threshold ({:.2})",
                    self.performance.cache_hit_rate, self.performance_thresholds.min_cache_hit_rate
                ),
            );
        }
    }

    /// Log event to console (only in debug builds)
    fn log_to_console(&self, event: &DebugEvent) {
        #[cfg(feature = "debug")]
        {
            match self.config.verbosity_level {
                DebugVerbosity::Low => {
                    if matches!(
                        event,
                        DebugEvent::Error { .. } | DebugEvent::PerformanceWarning { .. }
                    ) {
                        log::debug!("[ANIMATION DEBUG] {event:?}");
                    }
                }
                DebugVerbosity::Medium => {
                    if !matches!(event, DebugEvent::AnimationUpdated { .. }) {
                        log::debug!("[ANIMATION DEBUG] {event:?}");
                    }
                }
                DebugVerbosity::High | DebugVerbosity::Verbose => {
                    log::trace!("[ANIMATION TRACE] {event:?}");
                }
            }
        }

        #[cfg(not(feature = "debug"))]
        {
            // In production, only log errors and warnings
            match event {
                DebugEvent::Error { message, .. } => {
                    log::error!("Animation error: {message}");
                }
                DebugEvent::PerformanceWarning { details, .. } => {
                    log::warn!("Animation performance warning: {details}");
                }
                _ => {} // Ignore other debug events in production
            }
        }
    }

    /// Get debug events
    pub fn get_events(&self) -> &[DebugEvent] {
        &self.events
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> &DebugPerformanceMetrics {
        &self.performance
    }

    /// Get animation snapshots
    pub fn get_animation_snapshots(&self, id: &AnimationId) -> Option<&[AnimationSnapshot]> {
        self.snapshots.get(id).map(|v| v.as_slice())
    }

    /// Get timeline snapshots
    pub fn get_timeline_snapshots(&self, id: &TimelineId) -> Option<&[TimelineSnapshot]> {
        self.timeline_snapshots.get(id).map(|v| v.as_slice())
    }

    /// Clear debug data
    pub fn clear(&mut self) {
        self.events.clear();
        self.frame_times.clear();
        self.snapshots.clear();
        self.timeline_snapshots.clear();
        self.performance = DebugPerformanceMetrics::default();
        self.start_time = Instant::now();
    }

    /// Generate debug report
    pub fn generate_report(&self) -> DebugReport {
        DebugReport {
            config: self.config.clone(),
            performance: self.performance.clone(),
            event_count: self.events.len(),
            animation_count: self.snapshots.len(),
            timeline_count: self.timeline_snapshots.len(),
            uptime: self.start_time.elapsed(),
            events: self.events.clone(),
        }
    }

    /// Update active animation count
    pub fn update_active_count(&mut self, count: usize) {
        self.performance.active_animations = count;
    }

    /// Update memory usage estimate
    pub fn update_memory_usage(&mut self, usage: usize) {
        self.performance.memory_usage = usage;
    }

    /// Update cache hit rate
    pub fn update_cache_hit_rate(&mut self, rate: f32) {
        self.performance.cache_hit_rate = rate;
    }
}

/// Debug report containing all debugging information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugReport {
    /// Debug configuration settings
    pub config: DebugConfig,
    /// Performance metrics and statistics
    pub performance: DebugPerformanceMetrics,
    /// Total number of debug events recorded
    pub event_count: usize,
    /// Total number of animations tracked
    pub animation_count: usize,
    /// Total number of timelines tracked
    pub timeline_count: usize,
    /// Total uptime of the debug session
    pub uptime: Duration,
    /// All recorded debug events
    pub events: Vec<DebugEvent>,
}

/// Enhanced Animation Manager with debugging capabilities
pub struct DebugAnimationManager {
    /// Base animation manager
    manager: AnimationManager,
    /// Debugger instance
    debugger: AnimationDebugger,
}

impl DebugAnimationManager {
    /// Create a new debug animation manager
    pub fn new(debug_config: DebugConfig) -> Self {
        Self {
            manager: AnimationManager::new(),
            debugger: AnimationDebugger::new(debug_config),
        }
    }

    /// Add an animation with debugging
    pub fn add_animation(&mut self, mut animation: Animation) {
        self.debugger.log_animation_start(&animation);
        self.debugger.take_animation_snapshot(&animation);

        // Wrap callbacks to include debugging
        self.wrap_animation_callbacks(&mut animation);

        self.manager.add_animation(animation);
    }

    /// Add a timeline with debugging
    pub fn add_timeline(&mut self, timeline: AnimationTimeline) {
        self.debugger
            .log_timeline_event(&timeline, TimelineEventType::Started);
        self.debugger.take_timeline_snapshot(&timeline);
        self.manager.add_timeline(timeline);
    }

    /// Update with debugging
    pub fn update(&mut self) {
        let start_time = Instant::now();

        self.manager.update();

        let _frame_time = start_time.elapsed();

        // Update debugger with performance data
        self.debugger
            .update_active_count(self.manager.active_count());

        // Take snapshots of all animations if verbose debugging
        if matches!(
            self.debugger.config.verbosity_level,
            DebugVerbosity::Verbose
        ) {
            // Note: We would need public access to animations to take snapshots
            // For now, snapshots are taken when animations are added/updated
        }
    }

    /// Wrap animation callbacks to include debugging
    fn wrap_animation_callbacks(&mut self, _animation: &mut Animation) {
        // Note: This would require modifying the Animation struct to allow
        // callback wrapping, which is complex due to the Arc<dyn Fn> types.
        // For now, we rely on explicit logging calls in the application code.
    }

    /// Get debugger reference
    pub fn debugger(&self) -> &AnimationDebugger {
        &self.debugger
    }

    /// Get mutable debugger reference
    pub fn debugger_mut(&mut self) -> &mut AnimationDebugger {
        &mut self.debugger
    }

    /// Get manager reference
    pub fn manager(&self) -> &AnimationManager {
        &self.manager
    }

    /// Get mutable manager reference
    pub fn manager_mut(&mut self) -> &mut AnimationManager {
        &mut self.manager
    }
}

/// Visual debug overlay for animations
#[derive(Debug, Clone, PartialEq)]
pub struct DebugOverlay {
    /// Whether overlay is visible
    pub visible: bool,
    /// Overlay position
    pub position: (u16, u16),
    /// Overlay size
    pub size: (u16, u16),
    /// Background color
    pub background_color: (u8, u8, u8),
    /// Text color
    pub text_color: (u8, u8, u8),
    /// Border color
    pub border_color: (u8, u8, u8),
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            visible: false,
            position: (0, 0),
            size: (40, 20),
            background_color: (0, 0, 0),
            text_color: (255, 255, 255),
            border_color: (128, 128, 128),
        }
    }
}

impl DebugOverlay {
    /// Create a new debug overlay
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the overlay
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the overlay
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle overlay visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Set overlay position
    pub fn set_position(&mut self, x: u16, y: u16) {
        self.position = (x, y);
    }

    /// Set overlay size
    pub fn set_size(&mut self, width: u16, height: u16) {
        self.size = (width, height);
    }

    /// Render debug information to string
    pub fn render_debug_info(&self, debugger: &AnimationDebugger) -> Vec<String> {
        let mut lines = Vec::new();

        lines.push("=== Animation Debug Info ===".to_string());
        lines.push(format!(
            "Active Animations: {}",
            debugger.performance.active_animations
        ));
        lines.push(format!(
            "Avg Frame Time: {:.2}ms",
            debugger.performance.avg_frame_time.as_secs_f32() * 1000.0
        ));
        lines.push(format!(
            "Max Frame Time: {:.2}ms",
            debugger.performance.max_frame_time.as_secs_f32() * 1000.0
        ));
        lines.push(format!(
            "Memory Usage: {:.2}MB",
            debugger.performance.memory_usage as f32 / (1024.0 * 1024.0)
        ));
        lines.push(format!(
            "Cache Hit Rate: {:.1}%",
            debugger.performance.cache_hit_rate * 100.0
        ));
        lines.push(format!("Total Events: {}", debugger.events.len()));

        // Recent events
        lines.push("".to_string());
        lines.push("=== Recent Events ===".to_string());
        for event in debugger.events.iter().rev().take(5) {
            match event {
                DebugEvent::AnimationStarted { id, .. } => {
                    lines.push(format!("Started: {id}"));
                }
                DebugEvent::AnimationCompleted { id, .. } => {
                    lines.push(format!("Completed: {id}"));
                }
                DebugEvent::PerformanceWarning { warning_type, .. } => {
                    lines.push(format!("Warning: {warning_type:?}"));
                }
                DebugEvent::Error { error_type, .. } => {
                    lines.push(format!("Error: {error_type:?}"));
                }
                _ => {}
            }
        }

        lines
    }
}

/// Convenience functions for debugging
///
/// Create a debug animation manager with default configuration
pub fn create_debug_manager() -> DebugAnimationManager {
    DebugAnimationManager::new(DebugConfig::default())
}

/// Create a debug animation manager with high verbosity
pub fn create_verbose_debug_manager() -> DebugAnimationManager {
    let config = DebugConfig {
        verbosity_level: DebugVerbosity::Verbose,
        enable_visual_debug: true,
        ..Default::default()
    };
    DebugAnimationManager::new(config)
}

/// Create a performance-focused debug manager
pub fn create_performance_debug_manager() -> DebugAnimationManager {
    let config = DebugConfig {
        enable_performance_monitoring: true,
        enable_state_logging: false,
        verbosity_level: DebugVerbosity::Low,
        ..Default::default()
    };
    DebugAnimationManager::new(config)
}
