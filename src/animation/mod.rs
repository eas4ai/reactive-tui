//! Animation System Widget
//!
//! A comprehensive animation system providing smooth transitions, easing functions,
//! and property animations for TUI widgets with frame-based timing and interpolation.

/// Animation API for creating and managing animations
pub mod api;
mod binding;
mod targets;
pub use targets::{AnimationTarget, AnimationTargetContext, AnimationTargetError};
pub(crate) use targets::{PresentedTargets, TargetRegistry};
/// Core animation types and configuration
pub mod core;
/// Debug utilities for animation system
pub mod debug;
/// Easing functions for smooth animation transitions
pub mod easing;
/// Keyframe-based animation system
pub mod keyframes;
/// Lock-free animation state management to prevent deadlocks
pub mod lock_free;
/// Performance monitoring and optimization for animations
pub mod performance;
/// Animation property types and transformations
pub mod properties;
/// Spring physics-based animations
pub mod spring;
/// Staggered animation utilities for coordinated effects
pub mod stagger;
/// Animation state management types
pub mod state;

// Re-export commonly used types
pub use core::AnimationConfig;
pub use easing::{ease_value, EasingFunction};
pub use keyframes::{Keyframe, KeyframeAnimation, KeyframeError, KeyframeType};
pub use properties::{
    AnimatedProperty, CssValue, PropertyAnimation, TransformMatrix, TransformProperty,
};
pub use spring::SpringConfig;
pub use stagger::StaggerConfig;
pub use state::{
    AnimatedValue, AnimationRuntime, AnimationRuntimeState, AnimationState, AnimationValue,
    LoopMode,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Type aliases for complex function pointer types
type OnStartCallback = Arc<dyn Fn(&Animation) + Send + Sync>;
type OnUpdateCallback = Arc<dyn Fn(&Animation, &AnimatedValue) + Send + Sync>;
type OnCompleteCallback = Arc<dyn Fn(&Animation) + Send + Sync>;
type OnLoopCallback = Arc<dyn Fn(&Animation, u32) + Send + Sync>;
type OnPauseCallback = Arc<dyn Fn(&Animation) + Send + Sync>;
type OnStopCallback = Arc<dyn Fn(&Animation) + Send + Sync>;

/// Unique identifier for animations
pub type AnimationId = String;

/// Unique identifier for animation timelines
pub type TimelineId = String;

/// Monotonic time source resistant to time manipulation attacks
pub struct MonotonicTimer {
    /// Base time from when the timer was created
    pub(crate) base_time: Instant,
    /// Monotonic counter to prevent backward time jumps
    last_time: AtomicU64,
}

impl MonotonicTimer {
    /// Create a new monotonic timer
    pub fn new() -> Self {
        Self {
            base_time: Instant::now(),
            last_time: AtomicU64::new(0),
        }
    }

    /// Get current monotonic time that cannot go backwards
    pub fn now(&self) -> Duration {
        let current = self.base_time.elapsed();
        let current_nanos = current.as_nanos() as u64;

        // Ensure time only moves forward
        let last = self.last_time.load(Ordering::Acquire);
        let monotonic_nanos = if current_nanos > last {
            // Normal case: time moved forward
            self.last_time.store(current_nanos, Ordering::Release);
            current_nanos
        } else {
            // Time manipulation detected: use last known good time
            last
        };

        // Protect against overflow
        let safe_nanos = monotonic_nanos.min(u64::MAX / 2);
        Duration::from_nanos(safe_nanos)
    }

    /// Get elapsed time since timer creation
    pub fn elapsed(&self) -> Duration {
        self.now()
    }

    /// Check if a duration has elapsed since a given start time
    pub fn has_elapsed(&self, start: Duration, duration: Duration) -> bool {
        let current = self.now();
        current.saturating_sub(start) >= duration
    }
}

impl Default for MonotonicTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Global monotonic timer instance
static MONOTONIC_TIMER: std::sync::OnceLock<MonotonicTimer> = std::sync::OnceLock::new();

/// Get the global monotonic timer
fn get_monotonic_timer() -> &'static MonotonicTimer {
    MONOTONIC_TIMER.get_or_init(MonotonicTimer::new)
}

/// Animation controller for managing animations
pub struct AnimationController {
    animation: Animation,
    is_running: bool,
}

impl AnimationController {
    /// Create a new animation controller with the given animation
    ///
    /// # Arguments
    /// * `animation` - The animation to control
    ///
    /// # Returns
    /// A new `AnimationController` instance
    pub fn new(animation: Animation) -> Self {
        Self {
            animation,
            is_running: false,
        }
    }

    /// Start playing the animation
    ///
    /// Sets the controller state to running and begins animation playback.
    pub fn start(&mut self) {
        self.is_running = true;
        self.animation.play();
    }

    /// Pause the animation
    ///
    /// Temporarily stops animation playback while preserving current position.
    /// Use `resume()` to continue from the current position.
    pub fn pause(&mut self) {
        self.is_running = false;
        self.animation.pause();
    }

    /// Stop the animation completely
    ///
    /// Stops animation playback and resets to the beginning.
    /// Use `start()` to begin playback from the start.
    pub fn stop(&mut self) {
        self.is_running = false;
        self.animation.stop();
    }

    /// Resume a paused animation
    ///
    /// Continues animation playback from the current position if the animation
    /// was previously paused. Has no effect if already playing.
    pub fn resume(&mut self) {
        if !self.is_running {
            self.is_running = true;
            self.animation.play();
        }
    }

    /// Reset the animation to its initial state
    ///
    /// Stops the animation and seeks back to the beginning (progress 0.0).
    pub fn reset(&mut self) {
        self.stop();
        self.animation.seek(0.0);
    }

    /// Check if the animation is currently playing
    ///
    /// # Returns
    /// `true` if the animation is actively playing, `false` otherwise
    pub fn is_playing(&self) -> bool {
        self.is_running
    }
}

// EasingFunction is now re-exported from the easing module
// See easing::EasingFunction for the complete implementation

/// Callbacks for animation lifecycle events
///
/// This struct holds optional callbacks that are triggered at various
/// points in an animation's lifecycle, allowing for custom behavior
/// on start, update, completion, loop, pause, and stop events.
#[derive(Default)]
pub struct AnimationCallbacks {
    /// Called when animation starts
    pub on_start: Option<OnStartCallback>,
    /// Called on each frame update
    pub on_update: Option<OnUpdateCallback>,
    /// Called when animation completes
    pub on_complete: Option<OnCompleteCallback>,
    /// Called when animation loops
    pub on_loop: Option<OnLoopCallback>,
    /// Called when animation is paused
    pub on_pause: Option<OnPauseCallback>,
    /// Called when animation is stopped
    pub on_stop: Option<OnStopCallback>,
}

/// Main Animation widget
pub struct Animation {
    /// Unique animation identifier
    pub id: AnimationId,
    /// Property to animate
    pub property: AnimatedProperty,
    /// Animation configuration
    pub config: AnimationConfig,
    /// Runtime state
    pub state: Arc<std::sync::RwLock<AnimationRuntimeState>>,
    /// Non-serializable runtime data
    pub runtime: AnimationRuntime,
    /// Event callbacks
    pub callbacks: AnimationCallbacks,
    /// Start time for delay calculations
    start_time: Option<Instant>,
    target_binding: Option<binding::TargetBinding>,
}

impl Animation {
    /// Create a new animation
    pub fn new(
        duration: Duration,
        easing: EasingFunction,
        loop_count: Option<u32>,
        loop_behavior: LoopMode,
    ) -> Self {
        let loop_mode = match loop_count {
            None => LoopMode::None,
            Some(0) => LoopMode::Infinite,
            Some(n) => LoopMode::Count(n),
        };

        Self {
            id: format!("anim_{}", get_monotonic_timer().elapsed().as_millis()),
            property: AnimatedProperty::Opacity(0.0, 1.0),
            config: AnimationConfig {
                duration,
                delay: Duration::ZERO,
                easing,
                loop_mode,
                reverse: false,
                speed: 1.0,
                auto_play: false,
                auto_reverse: loop_behavior == LoopMode::PingPong,
            },
            state: Arc::new(std::sync::RwLock::new(AnimationRuntimeState::default())),
            runtime: AnimationRuntime::default(),
            callbacks: AnimationCallbacks::default(),
            start_time: None,
            target_binding: None,
        }
    }

    /// Create a spring animation with physics-based easing
    ///
    /// # Arguments
    /// * `duration` - The duration of the animation
    /// * `config` - Spring configuration parameters (mass, stiffness, damping)
    ///
    /// # Returns
    /// A new `Animation` instance with spring easing
    pub fn spring(duration: Duration, config: SpringConfig) -> Self {
        Self::new(
            duration,
            EasingFunction::Spring(config),
            None,
            LoopMode::None,
        )
    }

    /// Create a new animation builder for fluent configuration
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the animation
    ///
    /// # Returns
    /// An `AnimationBuilder` instance for fluent configuration
    pub fn builder<S: Into<String>>(id: S) -> AnimationBuilder {
        AnimationBuilder::new(id)
    }

    /// Start playing the animation
    ///
    /// Sets the animation state to playing and triggers the start callback.
    /// Records the start time for delay calculations.
    pub fn play(&mut self) {
        self.try_play()
            .unwrap_or_else(|error| panic!("cannot play animation: {error}"));
    }

    /// Start or resume playback, rejecting missing targets and invalid values.
    /// A stopped/completed bound animation captures fresh presented endpoints.
    pub fn try_play(&mut self) -> Result<(), AnimationTargetError> {
        let resume = self.get_state() == AnimationState::Paused;
        if let Some(binding) = &mut self.target_binding {
            if !resume {
                self.property = binding.resolve()?;
                binding.capture_pending = true;
                if let Ok(mut state) = self.state.write() {
                    *state = AnimationRuntimeState::default();
                }
            }
            binding.live()?;
        }
        if let Ok(mut state) = self.state.write() {
            state.state = AnimationState::Playing;
        }

        let now = get_monotonic_timer().now();
        // Convert monotonic Duration to Instant for compatibility
        let base = get_monotonic_timer().base_time;
        self.runtime.last_frame_time = Some(base + now);
        self.start_time = Some(base + now);

        if let Some(callback) = &self.callbacks.on_start {
            callback(self);
        }
        Ok(())
    }

    /// Pause the animation at its current position
    ///
    /// Sets the animation state to paused and triggers the pause callback.
    /// The animation can be resumed from this position using `play()`.
    pub fn pause(&mut self) {
        if let Ok(mut state) = self.state.write() {
            state.state = AnimationState::Paused;
        }

        if let Some(callback) = &self.callbacks.on_pause {
            callback(self);
        }
    }

    /// Stop the animation and reset to the beginning
    ///
    /// Resets all animation state including time, progress, and loop count.
    /// Triggers the stop callback and clears timing information.
    pub fn stop(&mut self) {
        if let Ok(mut state) = self.state.write() {
            state.state = AnimationState::Stopped;
            state.current_time = Duration::ZERO;
            state.progress = 0.0;
            state.loops_completed = 0;
            state.is_reversed = false;
            state.current_values = None;
        }

        self.runtime.last_frame_time = None;
        self.start_time = None;

        if let Some(callback) = &self.callbacks.on_stop {
            callback(self);
        }
    }

    /// Reverse the animation direction
    ///
    /// Toggles the animation direction. If currently playing forward,
    /// it will play backward and vice versa.
    pub fn reverse(&mut self) {
        if let Ok(mut state) = self.state.write() {
            state.is_reversed = !state.is_reversed;
            if state.state == AnimationState::Playing {
                state.state = AnimationState::Reversed;
            }
        }
    }

    /// Update the animation for one frame
    ///
    /// This should be called every frame in your main loop to advance the animation.
    /// Handles timing, easing, looping, and triggers appropriate callbacks.
    ///
    /// # Arguments
    /// * `delta_time` - Time elapsed since the last frame
    ///
    /// # Returns
    /// `true` if the animation is still active, `false` if completed or stopped
    pub fn update(&mut self, delta_time: Duration) -> bool {
        if self
            .target_binding
            .as_ref()
            .is_some_and(|binding| binding.live().is_err())
        {
            self.stop();
            return false;
        }
        let mut state_guard = match self.state.write() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        let state = &mut *state_guard;

        if state.state != AnimationState::Playing {
            return false;
        }

        // Handle delay using monotonic time
        if let Some(start_time) = self.start_time {
            let monotonic_now = get_monotonic_timer().base_time + get_monotonic_timer().now();
            if monotonic_now.duration_since(start_time) < self.config.delay {
                return false;
            }
        }

        if let Some(binding) = &mut self.target_binding {
            if binding.capture_pending {
                match binding.resolve() {
                    Ok(property) => {
                        self.property = property;
                        binding.capture_pending = false;
                    }
                    Err(_) => {
                        state.state = AnimationState::Stopped;
                        state.current_values = None;
                        return false;
                    }
                }
            }
        }

        // Update time
        let adjusted_delta = Duration::from_secs_f32(delta_time.as_secs_f32() * self.config.speed);
        state.current_time += adjusted_delta;

        // Calculate progress
        let total_duration = self.config.duration;
        let raw_progress = if total_duration > Duration::ZERO {
            (state.current_time.as_secs_f32() / total_duration.as_secs_f32()).min(1.0)
        } else {
            1.0
        };

        // Apply direction
        let progress = if state.is_reversed || self.config.reverse {
            1.0 - raw_progress
        } else {
            raw_progress
        };

        // Apply easing
        let eased_progress = self.config.easing.apply(progress);

        // Update state
        state.progress = eased_progress;
        state.current_values = Some(self.property.interpolate(eased_progress));
        if let Some(binding) = &self.target_binding {
            if binding.sample(eased_progress).is_err() {
                state.state = AnimationState::Stopped;
                state.current_values = None;
                return false;
            }
        }

        // Trigger update callback
        if let Some(callback) = &self.callbacks.on_update {
            if let Some(ref values) = state.current_values {
                callback(self, values);
            }
        }

        // Check for completion
        if raw_progress >= 1.0 {
            // Handle animation completion inline to avoid borrow issues
            match self.config.loop_mode {
                LoopMode::None => {
                    state.state = AnimationState::Completed;
                }
                LoopMode::Infinite => {
                    state.current_time = Duration::ZERO;
                    state.progress = 0.0;
                    state.loops_completed += 1;
                }
                LoopMode::Count(count) => {
                    state.loops_completed += 1;
                    if state.loops_completed < count {
                        state.current_time = Duration::ZERO;
                        state.progress = 0.0;
                    } else {
                        state.state = AnimationState::Completed;
                    }
                }
                LoopMode::PingPong => {
                    state.is_reversed = !state.is_reversed;
                    state.current_time = Duration::ZERO;
                    state.loops_completed += 1;
                }
            }
        }

        // Drop the guard before calling callbacks
        drop(state_guard);

        // Call callbacks if needed after releasing the lock
        if raw_progress >= 1.0
            && matches!(self.config.loop_mode, LoopMode::None | LoopMode::Count(_))
        {
            if let Ok(state) = self.state.read() {
                if state.state == AnimationState::Completed {
                    if let Some(callback) = &self.callbacks.on_complete {
                        callback(self);
                    }
                }
            }
        }

        true
    }

    /// Handle animation completion and looping
    #[allow(dead_code)]
    fn handle_animation_complete(&mut self, state: &mut AnimationRuntimeState) {
        match self.config.loop_mode {
            LoopMode::None => {
                state.state = AnimationState::Completed;
                if let Some(callback) = &self.callbacks.on_complete {
                    callback(self);
                }
            }
            LoopMode::Infinite => {
                self.restart_animation(state);
            }
            LoopMode::Count(count) => {
                state.loops_completed += 1;
                if state.loops_completed < count {
                    self.restart_animation(state);
                } else {
                    state.state = AnimationState::Completed;
                    if let Some(callback) = &self.callbacks.on_complete {
                        callback(self);
                    }
                }
            }
            LoopMode::PingPong => {
                state.is_reversed = !state.is_reversed;
                state.current_time = Duration::ZERO;
                state.loops_completed += 1;

                if let Some(callback) = &self.callbacks.on_loop {
                    callback(self, state.loops_completed);
                }
            }
        }
    }

    /// Restart animation for looping
    #[allow(dead_code)]
    fn restart_animation(&mut self, state: &mut AnimationRuntimeState) {
        state.current_time = Duration::ZERO;
        state.progress = 0.0;
        state.loops_completed += 1;

        if self.config.auto_reverse {
            state.is_reversed = !state.is_reversed;
        }

        if let Some(callback) = &self.callbacks.on_loop {
            callback(self, state.loops_completed);
        }
    }

    /// Get the current animation state
    ///
    /// # Returns
    /// The current `AnimationState` (Playing, Paused, Stopped, etc.)
    pub fn get_state(&self) -> AnimationState {
        self.state.read().unwrap().state
    }

    /// Get the current animation progress
    ///
    /// # Returns
    /// Progress value between 0.0 (start) and 1.0 (complete)
    pub fn get_progress(&self) -> f32 {
        self.state.read().unwrap().progress
    }

    /// Get the current interpolated animated values
    ///
    /// # Returns
    /// `Some(AnimatedValue)` if animation is active, `None` if stopped
    pub fn get_current_values(&self) -> Option<AnimatedValue> {
        self.state.read().unwrap().current_values.clone()
    }

    /// Check if the animation is currently playing
    ///
    /// # Returns
    /// `true` if animation is playing (forward or reverse), `false` otherwise
    pub fn is_playing(&self) -> bool {
        matches!(
            self.get_state(),
            AnimationState::Playing | AnimationState::Reversed
        )
    }

    /// Check if the animation has completed
    ///
    /// # Returns
    /// `true` if animation has finished all loops, `false` otherwise
    pub fn is_completed(&self) -> bool {
        self.get_state() == AnimationState::Completed
    }

    /// Set the animation speed multiplier
    ///
    /// # Arguments
    /// * `speed` - Speed multiplier (1.0 = normal, 2.0 = double speed, 0.5 = half speed)
    ///   Values less than 0.0 are clamped to 0.0
    pub fn set_speed(&mut self, speed: f32) {
        self.config.speed = speed.max(0.0);
    }

    /// Seek to a specific progress position in the animation
    ///
    /// # Arguments
    /// * `progress` - Target progress (0.0 to 1.0, values outside range are clamped)
    pub fn seek(&mut self, progress: f32) {
        self.try_seek(progress)
            .unwrap_or_else(|error| panic!("cannot seek animation: {error}"));
    }

    /// Seek a bound animation, reporting a removed target instead of updating it.
    pub fn try_seek(&mut self, progress: f32) -> Result<(), AnimationTargetError> {
        if !progress.is_finite() {
            return Err(AnimationTargetError::InvalidValue(
                "progress".into(),
                "must be finite".into(),
            ));
        }
        let progress = progress.clamp(0.0, 1.0);
        if let Some(binding) = &mut self.target_binding {
            if binding.capture_pending {
                self.property = binding.resolve()?;
                binding.capture_pending = false;
            }
            binding.sample(progress)?;
        }
        let target_time = Duration::from_secs_f32(self.config.duration.as_secs_f32() * progress);

        if let Ok(mut state) = self.state.write() {
            state.current_time = target_time;
            state.progress = progress;
            state.current_values = Some(self.property.interpolate(progress));
        }
        Ok(())
    }

    // Element conversion removed - incompatible with current Element struct
}

/// Builder for creating animations
pub struct AnimationBuilder {
    id: AnimationId,
    property: Option<AnimatedProperty>,
    config: AnimationConfig,
    callbacks: AnimationCallbacks,
}

impl AnimationBuilder {
    /// Create a new animation builder with the specified ID
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the animation
    ///
    /// # Returns
    /// A new `AnimationBuilder` instance with default configuration
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self {
            id: id.into(),
            property: None,
            config: AnimationConfig::default(),
            callbacks: AnimationCallbacks::default(),
        }
    }

    /// Set the property to animate
    ///
    /// # Arguments
    /// * `property` - The animated property (opacity, position, color, etc.)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn animate_property(mut self, property: AnimatedProperty) -> Self {
        self.property = Some(property);
        self
    }

    /// Set the animation duration
    ///
    /// # Arguments
    /// * `duration` - How long the animation should take to complete
    ///
    /// # Returns
    /// Self for method chaining
    pub fn duration(mut self, duration: Duration) -> Self {
        self.config.duration = duration;
        self
    }

    /// Set the easing function for the animation
    ///
    /// # Arguments
    /// * `easing` - The easing function to use (linear, ease-in, bounce, etc.)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn easing(mut self, easing: EasingFunction) -> Self {
        self.config.easing = easing;
        self
    }

    /// Set a delay before the animation starts
    ///
    /// # Arguments
    /// * `delay` - Time to wait before starting the animation
    ///
    /// # Returns
    /// Self for method chaining
    pub fn delay(mut self, delay: Duration) -> Self {
        self.config.delay = delay;
        self
    }

    /// Set the loop behavior for the animation
    ///
    /// # Arguments
    /// * `loop_mode` - How the animation should loop (none, infinite, count, ping-pong)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn loop_mode(mut self, loop_mode: LoopMode) -> Self {
        self.config.loop_mode = loop_mode;
        self
    }

    /// Set the animation speed multiplier
    ///
    /// # Arguments
    /// * `speed` - Speed multiplier (1.0 = normal, 2.0 = double speed, 0.5 = half speed)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn speed(mut self, speed: f32) -> Self {
        self.config.speed = speed;
        self
    }

    /// Enable or disable auto-play
    ///
    /// # Arguments
    /// * `auto_play` - Whether the animation should start playing immediately when built
    ///
    /// # Returns
    /// Self for method chaining
    pub fn auto_play(mut self, auto_play: bool) -> Self {
        self.config.auto_play = auto_play;
        self
    }

    /// Enable or disable auto-reverse
    ///
    /// # Arguments
    /// * `auto_reverse` - Whether the animation should automatically reverse direction on completion
    ///
    /// # Returns
    /// Self for method chaining
    pub fn auto_reverse(mut self, auto_reverse: bool) -> Self {
        self.config.auto_reverse = auto_reverse;
        self
    }

    /// Set a callback to be called when the animation starts
    ///
    /// # Arguments
    /// * `callback` - Function to call when animation begins playing
    ///
    /// # Returns
    /// Self for method chaining
    pub fn on_start<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Animation) + Send + Sync + 'static,
    {
        self.callbacks.on_start = Some(Arc::new(callback));
        self
    }

    /// Set a callback to be called on each animation frame update
    ///
    /// # Arguments
    /// * `callback` - Function to call with animation and current values on each frame
    ///
    /// # Returns
    /// Self for method chaining
    pub fn on_update<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Animation, &AnimatedValue) + Send + Sync + 'static,
    {
        self.callbacks.on_update = Some(Arc::new(callback));
        self
    }

    /// Set a callback to be called when the animation completes
    ///
    /// # Arguments
    /// * `callback` - Function to call when animation finishes (after all loops)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn on_complete<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Animation) + Send + Sync + 'static,
    {
        self.callbacks.on_complete = Some(Arc::new(callback));
        self
    }

    /// Set a callback to be called when the animation loops
    ///
    /// # Arguments
    /// * `callback` - Function to call with animation and loop count when looping
    ///
    /// # Returns
    /// Self for method chaining
    pub fn on_loop<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Animation, u32) + Send + Sync + 'static,
    {
        self.callbacks.on_loop = Some(Arc::new(callback));
        self
    }

    /// Build the final animation from the configured builder
    ///
    /// Creates an `Animation` instance with all the configured properties,
    /// callbacks, and settings. If auto-play is enabled, the animation
    /// will start playing immediately.
    ///
    /// # Returns
    /// A fully configured `Animation` instance ready for use
    pub fn build(self) -> Animation {
        let property = self.property.unwrap_or(AnimatedProperty::Opacity(0.0, 1.0));

        let mut animation = Animation {
            id: self.id,
            property,
            config: self.config,
            state: Arc::new(std::sync::RwLock::new(AnimationRuntimeState::default())),
            runtime: AnimationRuntime::default(),
            callbacks: self.callbacks,
            start_time: None,
            target_binding: None,
        };

        if animation.config.auto_play {
            animation.play();
        }

        animation
    }
}

/// Animation Timeline for managing multiple animations
pub struct AnimationTimeline {
    /// Unique timeline identifier
    pub id: TimelineId,
    /// Animations in this timeline
    pub animations: Vec<Animation>,
    /// Timeline state
    pub state: Arc<std::sync::RwLock<AnimationState>>,
    /// Whether animations play in sequence or parallel
    pub sequential: bool,
    /// Current animation index (for sequential)
    current_index: usize,
}

impl AnimationTimeline {
    /// Create a new animation timeline
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the timeline
    /// * `sequential` - If true, animations play one after another; if false, they play in parallel
    ///
    /// # Returns
    /// A new `AnimationTimeline` instance
    pub fn new<S: Into<String>>(id: S, sequential: bool) -> Self {
        Self {
            id: id.into(),
            animations: Vec::new(),
            state: Arc::new(std::sync::RwLock::new(AnimationState::Stopped)),
            sequential,
            current_index: 0,
        }
    }

    /// Add an animation to the timeline
    ///
    /// # Arguments
    /// * `animation` - The animation to add to this timeline
    pub fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    /// Start playing the timeline
    ///
    /// For sequential timelines, starts the first animation.
    /// For parallel timelines, starts all animations simultaneously.
    pub fn play(&mut self) {
        *self.state.write().unwrap() = AnimationState::Playing;

        if self.sequential {
            if !self.animations.is_empty() {
                self.current_index = 0;
                self.animations[0].play();
            }
        } else {
            for animation in &mut self.animations {
                animation.play();
            }
        }
    }

    /// Update the timeline for one frame
    ///
    /// This should be called every frame in your main loop to advance all animations
    /// in the timeline. Handles sequential playback and completion detection.
    ///
    /// # Arguments
    /// * `delta_time` - Time elapsed since the last frame
    ///
    /// # Returns
    /// `true` if the timeline is still active, `false` if completed
    pub fn update(&mut self, delta_time: Duration) -> bool {
        let state = *self.state.read().unwrap();
        if state != AnimationState::Playing {
            return false;
        }

        if self.sequential {
            if self.current_index < self.animations.len() {
                let current_animation = &mut self.animations[self.current_index];

                if !current_animation.update(delta_time) && current_animation.is_completed() {
                    self.current_index += 1;

                    if self.current_index < self.animations.len() {
                        self.animations[self.current_index].play();
                    } else {
                        *self.state.write().unwrap() = AnimationState::Completed;
                        return false;
                    }
                }
            }
        } else {
            let mut any_playing = false;
            for animation in &mut self.animations {
                if animation.update(delta_time) {
                    any_playing = true;
                }
            }

            if !any_playing {
                *self.state.write().unwrap() = AnimationState::Completed;
                return false;
            }
        }

        true
    }

    /// Stop the timeline and all its animations
    ///
    /// Stops all animations in the timeline and resets the timeline state.
    /// For sequential timelines, resets the current animation index to 0.
    pub fn stop(&mut self) {
        *self.state.write().unwrap() = AnimationState::Stopped;
        self.current_index = 0;

        for animation in &mut self.animations {
            animation.stop();
        }
    }
}

/// Animation Manager for coordinating multiple animations
pub struct AnimationManager {
    /// All managed animations
    animations: HashMap<AnimationId, Animation>,
    /// All managed timelines
    timelines: HashMap<TimelineId, AnimationTimeline>,
    /// Last update time
    last_update: Instant,
}

impl AnimationManager {
    /// Create a new animation manager
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            timelines: HashMap::new(),
            last_update: get_monotonic_timer().base_time,
        }
    }

    /// Add an animation
    pub fn add_animation(&mut self, animation: Animation) -> AnimationId {
        let id = animation.id.clone();
        self.animations.insert(id.clone(), animation);
        id
    }

    /// Add a timeline
    pub fn add_timeline(&mut self, timeline: AnimationTimeline) {
        self.timelines.insert(timeline.id.clone(), timeline);
    }

    /// Remove an animation by ID
    pub fn remove_animation(&mut self, id: &AnimationId) {
        self.animations.remove(id);
    }

    /// Remove a timeline by ID
    pub fn remove_timeline(&mut self, id: &TimelineId) {
        self.timelines.remove(id);
    }

    /// Update all animations (call in main loop)
    pub fn update(&mut self) {
        let timer = get_monotonic_timer();
        let now = timer.base_time + timer.now();
        let delta_time = now.duration_since(self.last_update);
        self.last_update = now;

        // Update standalone animations
        self.animations.retain(|_, animation| {
            animation.update(delta_time);
            !animation.is_completed()
                || matches!(
                    animation.config.loop_mode,
                    LoopMode::Infinite | LoopMode::Count(_)
                )
        });

        // Update timelines
        for timeline in self.timelines.values_mut() {
            timeline.update(delta_time);
        }
    }

    /// Get animation by ID
    pub fn get_animation(&self, id: &str) -> Option<&Animation> {
        self.animations.get(id)
    }

    /// Get mutable animation by ID
    pub fn get_animation_mut(&mut self, id: &str) -> Option<&mut Animation> {
        self.animations.get_mut(id)
    }

    /// Remove completed, failed, or stuck animations
    pub fn cleanup_completed(&mut self) {
        // Remove completed animations
        self.animations
            .retain(|_, animation| !animation.is_completed());

        // Remove completed timelines, handling lock failures gracefully
        self.timelines.retain(|id, timeline| {
            match timeline.state.read() {
                Ok(state) => *state != AnimationState::Completed,
                Err(e) => {
                    // If we can't read the state, the lock is poisoned - remove it
                    log::warn!(
                        "Warning: Removing timeline {} due to poisoned lock: {}",
                        id,
                        e
                    );
                    false
                }
            }
        });
    }

    /// Clean up all animations including failed/stuck ones
    /// Returns the number of animations cleaned up
    pub fn cleanup_all_stale(&mut self, stale_threshold: Duration) -> usize {
        let timer = get_monotonic_timer();
        let now = timer.base_time + timer.now();
        let mut removed = 0;

        // Remove animations that are completed or haven't updated recently
        self.animations.retain(|_id, animation| {
            let should_keep = if animation.is_completed() {
                false
            } else if let Some(start_time) = animation.start_time {
                // Keep animations that started recently or are still progressing
                now.duration_since(start_time) < stale_threshold
                    || animation
                        .runtime
                        .last_frame_time
                        .is_some_and(|t| now.duration_since(t) < stale_threshold)
            } else {
                true // Keep animations that haven't started yet
            };

            if !should_keep {
                removed += 1;
                #[cfg(debug_assertions)]
                log::debug!("Cleaning up stale animation: {}", _id);
            }
            should_keep
        });

        // Remove stale timelines
        self.timelines.retain(|_id, timeline| {
            let should_keep = match timeline.state.read() {
                Ok(state) => *state != AnimationState::Completed,
                Err(_) => {
                    removed += 1;
                    false // Remove poisoned timelines
                }
            };

            if !should_keep {
                #[cfg(debug_assertions)]
                log::debug!("Cleaning up stale timeline: {}", _id);
            }
            should_keep
        });

        removed
    }

    /// Get active animation count
    pub fn active_count(&self) -> usize {
        self.animations.len() + self.timelines.len()
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common animations
/// Create a fade in animation
pub fn fade_in(id: impl Into<String>, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Opacity(0.0, 1.0))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a fade out animation
pub fn fade_out(id: impl Into<String>, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Opacity(1.0, 0.0))
        .duration(duration)
        .easing(EasingFunction::EaseIn)
        .build()
}

/// Create a slide in from left animation
pub fn slide_in_left(
    id: impl Into<String>,
    from_x: i16,
    to_x: i16,
    y: i16,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Position(from_x, y, to_x, y))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a bounce animation
pub fn bounce(id: impl Into<String>, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Scale(1.0, 1.2))
        .duration(duration)
        .easing(EasingFunction::Bounce)
        .loop_mode(LoopMode::PingPong)
        .build()
}

/// Create a pulse animation
pub fn pulse(id: impl Into<String>, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Opacity(1.0, 0.5))
        .duration(duration)
        .easing(EasingFunction::EaseInOut)
        .loop_mode(LoopMode::PingPong)
        .build()
}

// New convenience functions for anime.js-style animations

/// Create a translateX animation
pub fn translate_x(id: impl Into<String>, from: f32, to: f32, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::TranslateX(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a translateY animation
pub fn translate_y(id: impl Into<String>, from: f32, to: f32, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::TranslateY(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a scale animation
pub fn scale_animation(id: impl Into<String>, from: f32, to: f32, duration: Duration) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::Scale(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a rotate animation with endpoints in radians.
pub fn rotate_animation(
    id: impl Into<String>,
    from: f32,
    to: f32,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::Rotate(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a CSS property animation
pub fn css_property(
    id: impl Into<String>,
    property: &str,
    from: CssValue,
    to: CssValue,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::CssProperty(
            property.to_string(),
            from,
            to,
        ))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a numeric property animation
pub fn numeric_property(
    id: impl Into<String>,
    property: &str,
    from: f32,
    to: f32,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Property(property.to_string(), from, to))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

/// Create a transform matrix animation
pub fn matrix_animation(
    id: impl Into<String>,
    from: TransformMatrix,
    to: TransformMatrix,
    duration: Duration,
) -> Animation {
    AnimationBuilder::new(id)
        .animate_property(AnimatedProperty::Transform(TransformProperty::Matrix(
            from, to,
        )))
        .duration(duration)
        .easing(EasingFunction::EaseOut)
        .build()
}

// Helper functions for creating CssValues

// Additional re-exports from submodules
pub use stagger::{
    stagger, stagger_builder, stagger_from_center, stagger_from_index, stagger_from_last,
    stagger_from_position, stagger_grid, stagger_grid_center, stagger_random, StaggerBuilder,
    StaggerDirection, StaggerOrigin,
};

pub use spring::{spring, spring_with_velocity};

pub use api::{
    animate, create_timeline, fade_in as api_fade_in, fade_out as api_fade_out, scale, slide,
    spring_animate, stagger_delay, AnimateParams, AnimationTargets, ColorValue, DelayValue,
    PositionValue, PropertyValue, SizeValue, StaggerOptions, TimelineBuilder, TimelineParams,
};

pub use performance::{
    AnimationBatch, BatchedUpdate, CacheStats, InterpolationCache, OptimizationLevel,
    OptimizedAnimationManager, PerformanceMetrics, PerformanceReport,
};
// Re-export debug types and functions for direct use
pub use debug::{
    create_debug_manager, create_performance_debug_manager, create_verbose_debug_manager,
    AnimationDebugInfo, AnimationDebugger, AnimationErrorType, AnimationSnapshot,
    DebugAnimationManager, DebugConfig, DebugEvent, DebugOverlay, DebugPerformanceMetrics,
    DebugReport, DebugVerbosity, PerformanceThresholds, PerformanceWarningType, StopReason,
    TimelineEventType, TimelineSnapshot,
};

// Re-export keyframe types and functions for direct use (rename to avoid conflicts)
pub use keyframes::{keyframes, KeyframeBuilder, KeyframeSequence, KeyframeValue};

// Convenience functions for keyframe animations

/// Create a keyframe-based animation property
pub fn keyframe_animation(sequence: keyframes::KeyframeSequence) -> AnimatedProperty {
    AnimatedProperty::Keyframes(sequence)
}

/// Create a fade in animation using keyframes
pub fn keyframe_fade_in(duration_ms: u64) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::fade_in(duration_ms))
}

/// Create a fade out animation using keyframes
pub fn keyframe_fade_out(duration_ms: u64) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::fade_out(duration_ms))
}

/// Create a slide in animation using keyframes
pub fn keyframe_slide_in(duration_ms: u64, distance: f32) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::slide_in_from_left(duration_ms, distance))
}

/// Create a bounce in animation using keyframes
pub fn keyframe_bounce_in(duration_ms: u64) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::bounce_in(duration_ms))
}

/// Create a pulse animation using keyframes
pub fn keyframe_pulse(duration_ms: u64) -> AnimatedProperty {
    AnimatedProperty::Keyframes(keyframes::pulse(duration_ms))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_easing_function_linear() {
        let linear = EasingFunction::Linear;

        assert_eq!(linear.apply(0.0), 0.0);
        assert_eq!(linear.apply(0.5), 0.5);
        assert_eq!(linear.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_function_ease_in() {
        let ease_in = EasingFunction::EaseIn;

        assert_eq!(ease_in.apply(0.0), 0.0);
        assert_eq!(ease_in.apply(1.0), 1.0);

        // Ease-in should start slow
        let quarter = ease_in.apply(0.25);
        assert!(quarter < 0.25);
    }

    #[test]
    fn test_easing_function_ease_out() {
        let ease_out = EasingFunction::EaseOut;

        assert_eq!(ease_out.apply(0.0), 0.0);
        assert_eq!(ease_out.apply(1.0), 1.0);

        // Ease-out should start fast
        let quarter = ease_out.apply(0.25);
        assert!(quarter > 0.25);
    }

    #[test]
    fn test_easing_function_ease_in_out() {
        let ease_in_out = EasingFunction::EaseInOut;

        assert_eq!(ease_in_out.apply(0.0), 0.0);
        assert_eq!(ease_in_out.apply(0.5), 0.5);
        assert_eq!(ease_in_out.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_function_steps() {
        let steps = EasingFunction::Steps(4, false);

        assert_eq!(steps.apply(0.0), 0.0);
        assert_eq!(steps.apply(0.1), 0.0);
        assert_eq!(steps.apply(0.3), 0.25);
        assert_eq!(steps.apply(0.6), 0.5);
        assert_eq!(steps.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_function_steps_jump_start() {
        let steps = EasingFunction::Steps(4, true);

        // With jump_start=true, we should get the next step value immediately
        assert_eq!(steps.apply(0.0), 0.25); // First jump occurs at zero
        assert_eq!(steps.apply(0.1), 0.25); // First step
        assert_eq!(steps.apply(0.3), 0.5); // Second step
        assert_eq!(steps.apply(1.0), 1.0);
    }

    #[test]
    fn test_step_easing_boundaries() {
        for count in [1, 2, 4, 8] {
            let start = EasingFunction::Steps(count, true);
            let end = EasingFunction::Steps(count, false);
            for boundary in 0..=count {
                let progress = boundary as f32 / count as f32;
                assert_eq!(
                    start.apply(progress),
                    (boundary + 1).min(count) as f32 / count as f32
                );
                assert_eq!(end.apply(progress), boundary as f32 / count as f32);
                if boundary > 0 {
                    let before = progress - 0.001;
                    assert_eq!(start.apply(before), boundary as f32 / count as f32);
                    assert_eq!(end.apply(before), (boundary - 1) as f32 / count as f32);
                }
            }
        }
    }

    #[test]
    fn test_easing_function_linear_points() {
        let points = vec![0.0, 0.5, 1.0];
        let linear_points = EasingFunction::LinearPoints(points);

        assert_eq!(linear_points.apply(0.0), 0.0);
        assert_eq!(linear_points.apply(0.5), 0.5);
        assert_eq!(linear_points.apply(1.0), 1.0);
    }

    #[test]
    fn test_spring_config() {
        let spring = SpringConfig::new(1.0, 100.0, 10.0); // mass, stiffness, damping

        assert_eq!(spring.mass, 1.0);
        assert_eq!(spring.stiffness, 100.0);
        assert_eq!(spring.damping, 10.0);
        assert_eq!(spring.velocity, 0.0);

        // Test position calculation
        let pos = spring.calculate_position(0.1, 0.0, 1.0);
        assert!((0.0..=1.0).contains(&pos));
    }

    #[test]
    fn test_spring_presets() {
        let gentle = SpringConfig::gentle();
        assert!(gentle.damping > 0.5);

        let wobbly = SpringConfig::wobbly();
        assert!(wobbly.stiffness > gentle.stiffness);

        let stiff = SpringConfig::stiff();
        assert!(stiff.stiffness > wobbly.stiffness);
    }

    #[test]
    fn test_animated_property_opacity() {
        let opacity = AnimatedProperty::Opacity(0.0, 1.0);

        let values = opacity.interpolate(0.5);
        if let AnimatedValue::Opacity(value) = values {
            assert_eq!(value, 0.5);
        } else {
            panic!("Expected opacity value");
        }
    }

    #[test]
    fn test_animated_property_size() {
        let size = AnimatedProperty::Size(10, 20, 100, 80); // from_width, from_height, to_width, to_height

        let values = size.interpolate(0.5);
        if let AnimatedValue::Size(width, height) = values {
            assert_eq!(width, 55);
            assert_eq!(height, 50);
        } else {
            panic!("Expected size value");
        }
    }

    #[test]
    fn test_animated_property_position() {
        let position = AnimatedProperty::Position(0, 0, 100, 50); // from_x, from_y, to_x, to_y

        let values = position.interpolate(0.5);
        if let AnimatedValue::Position(x, y) = values {
            assert_eq!(x, 50);
            assert_eq!(y, 25);
        } else {
            panic!("Expected position value");
        }
    }

    #[test]
    fn test_animated_property_scale() {
        let scale = AnimatedProperty::Scale(1.0, 2.0);

        let values = scale.interpolate(0.5);
        if let AnimatedValue::Scale(value) = values {
            assert_eq!(value, 1.5);
        } else {
            panic!("Expected scale value");
        }
    }

    #[test]
    fn test_animated_property_rotation() {
        let rotation = AnimatedProperty::Rotation(0.0, 360.0);

        let values = rotation.interpolate(0.5);
        if let AnimatedValue::Rotation(value) = values {
            assert_eq!(value, 180.0);
        } else {
            panic!("Expected rotation value");
        }
    }

    #[test]
    fn test_animated_property_color() {
        let color = AnimatedProperty::Color((255, 0, 0), (0, 255, 0)); // red to green

        let values = color.interpolate(0.5);
        if let AnimatedValue::Color { r, g, b } = values {
            assert_eq!(r, 127);
            assert_eq!(g, 127);
            assert_eq!(b, 0);
        } else {
            panic!("Expected color value");
        }
    }

    #[test]
    fn test_animation_creation() {
        let duration = Duration::from_millis(1000);
        let animation = Animation::new(duration, EasingFunction::Linear, Some(1), LoopMode::None);

        assert_eq!(animation.config.duration, duration);
        assert!(matches!(animation.config.easing, EasingFunction::Linear));
        assert!(matches!(animation.config.loop_mode, LoopMode::Count(1)));
    }

    #[test]
    fn test_animation_spring_creation() {
        let duration = Duration::from_millis(500);
        let spring_config = SpringConfig::gentle();
        let animation = Animation::spring(duration, spring_config);

        assert_eq!(animation.config.duration, duration);
        assert!(matches!(animation.config.easing, EasingFunction::Spring(_)));
    }
}
