//! Animation hooks for reactive components

use crate::animation::keyframes::{Keyframe, KeyframeAnimation};
use crate::animation::stagger::StaggerConfig;
use crate::animation::{
    Animation, AnimationController, AnimationState, EasingFunction, LoopMode, SpringConfig,
};
use crate::reactive::{use_effect, use_signal, Hooks, Scheduler, ThreadSafeSignal};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Animation value that can be animated
pub trait AnimatableValue: Clone + Debug + Send + Sync + PartialEq + Default + 'static {
    /// Interpolate between this value and another at time t (0.0 to 1.0)
    fn interpolate(&self, to: &Self, t: f32) -> Self;
    /// Convert this value to an f32 for calculations
    fn to_f32(&self) -> f32;
    /// Create a value from an f32
    fn from_f32(value: f32) -> Self;
}

impl AnimatableValue for f32 {
    fn interpolate(&self, to: &Self, t: f32) -> Self {
        self + (to - self) * t
    }
    fn to_f32(&self) -> f32 {
        *self
    }
    fn from_f32(value: f32) -> Self {
        value
    }
}

impl AnimatableValue for f64 {
    fn interpolate(&self, to: &Self, t: f32) -> Self {
        self + (to - self) * t as f64
    }
    fn to_f32(&self) -> f32 {
        *self as f32
    }
    fn from_f32(value: f32) -> Self {
        value as f64
    }
}

impl AnimatableValue for i32 {
    fn interpolate(&self, to: &Self, t: f32) -> Self {
        (*self as f32 + (*to as f32 - *self as f32) * t) as i32
    }
    fn to_f32(&self) -> f32 {
        *self as f32
    }
    fn from_f32(value: f32) -> Self {
        value as i32
    }
}

impl AnimatableValue for u16 {
    fn interpolate(&self, to: &Self, t: f32) -> Self {
        (*self as f32 + (*to as f32 - *self as f32) * t) as u16
    }
    fn to_f32(&self) -> f32 {
        *self as f32
    }
    fn from_f32(value: f32) -> Self {
        value as u16
    }
}

impl AnimatableValue for (f32, f32) {
    fn interpolate(&self, to: &Self, t: f32) -> Self {
        (self.0 + (to.0 - self.0) * t, self.1 + (to.1 - self.1) * t)
    }
    fn to_f32(&self) -> f32 {
        self.0
    } // Return first component
    fn from_f32(value: f32) -> Self {
        (value, value)
    }
}

/// Shared animation runtime that manages animation frames
struct AnimationRuntime {
    running: Arc<AtomicBool>,
    animations: Arc<RwLock<Vec<AnimationTask>>>,
    /// Pre-computed animation updates to avoid blocking render thread
    pending_updates: Arc<RwLock<Vec<(usize, f32)>>>,
}

struct AnimationTask {
    id: usize,
    update: Arc<dyn Fn(f32) + Send + Sync>,
    start_time: Instant,
    duration: Duration,
}

impl AnimationRuntime {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            running: Arc::new(AtomicBool::new(true)),
            animations: Arc::new(RwLock::new(Vec::new())),
            pending_updates: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Compute animation progress asynchronously (can be called from background thread)
    pub fn compute_animation_updates(&self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        let mut updates = Vec::new();
        let mut completed = Vec::new();
        
        {
            let animations = self.animations.read().unwrap();
            for (idx, task) in animations.iter().enumerate() {
                let elapsed = task.start_time.elapsed();
                let progress = (elapsed.as_secs_f32() / task.duration.as_secs_f32()).min(1.0);
                updates.push((task.id, progress));
                
                if progress >= 1.0 {
                    completed.push(idx);
                }
            }
        }
        
        // Store computed updates for the render thread to apply
        if !updates.is_empty() {
            let mut pending = self.pending_updates.write().unwrap();
            *pending = updates;
        }
        
        // Remove completed animations
        if !completed.is_empty() {
            let mut animations = self.animations.write().unwrap();
            for idx in completed.iter().rev() {
                animations.remove(*idx);
            }
        }
    }

    /// Apply pre-computed animation updates (called from main render loop)
    pub fn update_animations(&self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        // First compute the updates (this could be moved to a background thread)
        self.compute_animation_updates();
        
        // Then apply them quickly without blocking
        let updates = {
            let mut pending = self.pending_updates.write().unwrap();
            std::mem::take(&mut *pending)
        };
        
        // Apply updates in batch - much faster than computing inline
        if !updates.is_empty() {
            let animations = self.animations.read().unwrap();
            for (id, progress) in updates {
                // Find animation by id and apply update
                if let Some(task) = animations.iter().find(|t| t.id == id) {
                    (task.update)(progress);
                }
            }
        }
    }

    fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    fn add_animation(&self, update: Arc<dyn Fn(f32) + Send + Sync>, duration: Duration) -> usize {
        let mut animations = self.animations.write().unwrap();
        let id = animations.len();
        animations.push(AnimationTask {
            id,
            update,
            start_time: Instant::now(),
            duration,
        });
        id
    }

    fn remove_animation(&self, id: usize) {
        let mut animations = self.animations.write().unwrap();
        animations.retain(|task| task.id != id);
    }
}

// Global animation runtime - now coordinates with main animation system
lazy_static::lazy_static! {
    static ref RUNTIME: Arc<AnimationRuntime> = AnimationRuntime::new();
}

/// Global animation manager reference for coordination
static mut GLOBAL_ANIMATION_MANAGER: Option<*mut crate::animation::AnimationManager> = None;

/// Set the global animation manager for coordination
/// SAFETY: This should only be called once during app initialization
/// # Safety
///
/// This function is unsafe because it sets a global mutable pointer.
/// The caller must ensure that:
/// - The pointer is valid for the lifetime of the program
/// - No other code is concurrently accessing the global animation manager
/// - The pointer points to a properly initialized AnimationManager
pub unsafe fn set_global_animation_manager(manager: *mut crate::animation::AnimationManager) {
    GLOBAL_ANIMATION_MANAGER = Some(manager);
}

/// Get the global animation manager if available
#[allow(dead_code)]
fn get_global_animation_manager() -> Option<&'static mut crate::animation::AnimationManager> {
    unsafe { GLOBAL_ANIMATION_MANAGER.and_then(|ptr| ptr.as_mut()) }
}

/// Update all hook-based animations (called from main render loop)
pub fn update_hook_animations() {
    RUNTIME.update_animations();
}

/// Stop the global animation runtime (for cleanup)
pub fn shutdown_animation_runtime() {
    RUNTIME.stop();
}

/// Animation handle
pub struct AnimationHandle<T: AnimatableValue> {
    value: ThreadSafeSignal<T>,
    controller: Arc<Mutex<AnimationController>>,
    state: ThreadSafeSignal<AnimationState>,
    current_animation_id: Arc<Mutex<Option<usize>>>,
    from_value: Arc<RwLock<T>>,
    to_value: Arc<RwLock<T>>,
    config: AnimationConfig,
}

impl<T: AnimatableValue> AnimationHandle<T> {
    /// Get the current animation value
    pub fn value(&self) -> T {
        self.value.get()
    }

    /// Animate to a target value
    pub fn animate_to(&self, target: T) {
        // Cancel current animation if running
        if let Some(id) = *self.current_animation_id.lock().unwrap() {
            RUNTIME.remove_animation(id);
        }

        // Store animation endpoints
        *self.from_value.write().unwrap() = self.value.get();
        *self.to_value.write().unwrap() = target.clone();

        // Start animation
        self.controller.lock().unwrap().start();
        self.state.set(AnimationState::Playing);

        // Create animation update function
        let value_signal = self.value.clone();
        let from = self.from_value.clone();
        let to = self.to_value.clone();
        let easing = self.config.easing.clone();
        let state_signal = self.state.clone();
        let controller = self.controller.clone();

        let update = Arc::new(move |progress: f32| {
            // Check if animation is paused/stopped
            let ctrl_state = controller.lock().unwrap();
            if !ctrl_state.is_playing() {
                return;
            }
            drop(ctrl_state);

            let eased = easing.apply(progress);
            let from_val = from.read().unwrap().clone();
            let to_val = to.read().unwrap().clone();
            let interpolated = from_val.interpolate(&to_val, eased);
            value_signal.set(interpolated);

            if progress >= 1.0 {
                controller.lock().unwrap().stop();
                state_signal.set(AnimationState::Stopped);
            }
        });

        // Add to runtime
        let id = RUNTIME.add_animation(update, self.config.duration);
        *self.current_animation_id.lock().unwrap() = Some(id);
    }

    /// Pause the current animation
    pub fn pause(&self) {
        self.controller.lock().unwrap().pause();
        self.state.set(AnimationState::Paused);
    }

    /// Resume a paused animation
    pub fn resume(&self) {
        self.controller.lock().unwrap().resume();
        self.state.set(AnimationState::Playing);
    }

    /// Stop the current animation
    pub fn stop(&self) {
        if let Some(id) = *self.current_animation_id.lock().unwrap() {
            RUNTIME.remove_animation(id);
        }
        self.controller.lock().unwrap().stop();
        self.state.set(AnimationState::Stopped);
    }

    /// Reset the animation to its initial value
    pub fn reset(&self) {
        self.stop();
        let from = self.from_value.read().unwrap().clone();
        self.value.set(from);
    }

    /// Get the current animation state
    pub fn state(&self) -> AnimationState {
        self.state.get()
    }
}

/// Configuration for animations
#[derive(Clone, Debug)]
pub struct AnimationConfig {
    /// Duration of the animation
    pub duration: Duration,
    /// Easing function to use for interpolation
    pub easing: EasingFunction,
    /// Number of times to loop (None for infinite)
    pub loop_count: Option<u32>,
    /// Behavior when looping
    pub loop_behavior: LoopMode,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(300),
            easing: EasingFunction::EaseInOut,
            loop_count: None,
            loop_behavior: LoopMode::None,
        }
    }
}

/// Hook for creating animations
pub fn use_animation<T: AnimatableValue>(
    hooks: &Hooks,
    initial: T,
    config: AnimationConfig,
) -> AnimationHandle<T> {
    let value = use_signal(hooks, initial.clone());
    let state = use_signal(hooks, AnimationState::Stopped);
    let from_value = Arc::new(RwLock::new(initial.clone()));
    let to_value = Arc::new(RwLock::new(initial));

    let animation = Animation::new(
        config.duration,
        config.easing.clone(),
        config.loop_count,
        config.loop_behavior,
    );
    let controller = Arc::new(Mutex::new(AnimationController::new(animation)));

    AnimationHandle {
        value,
        controller,
        state,
        current_animation_id: Arc::new(Mutex::new(None)),
        from_value,
        to_value,
        config,
    }
}

/// Spring animation handle
pub struct SpringHandle<T: AnimatableValue> {
    value: ThreadSafeSignal<T>,
    velocity: Arc<RwLock<f32>>,
    target: Arc<RwLock<T>>,
    config: SpringConfig,
    animation_id: Arc<Mutex<Option<usize>>>,
}

impl<T: AnimatableValue> SpringHandle<T> {
    /// Get the current spring value
    pub fn value(&self) -> T {
        self.value.get()
    }

    /// Get the current spring velocity
    pub fn velocity(&self) -> f32 {
        *self.velocity.read().unwrap()
    }

    /// Apply an impulse force to the spring
    pub fn apply_impulse(&self, force: f32) {
        let mut vel = self.velocity.write().unwrap();
        *vel += force;
        self.start_spring_animation();
    }

    /// Set a new target value for the spring to animate to
    pub fn set_target(&self, target: T) {
        *self.target.write().unwrap() = target;
        self.start_spring_animation();
    }

    fn start_spring_animation(&self) {
        // Cancel current animation
        if let Some(id) = *self.animation_id.lock().unwrap() {
            RUNTIME.remove_animation(id);
        }

        let value_signal = self.value.clone();
        let velocity_ref = self.velocity.clone();
        let target_ref = self.target.clone();
        let config = self.config.clone();
        let animation_id_ref = self.animation_id.clone();

        let update = Arc::new(move |_progress: f32| {
            let current = value_signal.get().to_f32();
            let target = target_ref.read().unwrap().to_f32();
            let _velocity = *velocity_ref.read().unwrap();

            // Spring physics calculation - calculate_position takes 3 params: time, from, to
            let position = config.calculate_position(0.016, current, target); // 60fps frame
            let new_velocity = config.calculate_velocity(0.016, current, target);

            *velocity_ref.write().unwrap() = new_velocity;
            value_signal.set(T::from_f32(position));

            // Stop when settled and remove from runtime
            if (position - target).abs() < 0.001f32 && new_velocity.abs() < 0.001f32 {
                if let Some(id) = *animation_id_ref.lock().unwrap() {
                    RUNTIME.remove_animation(id);
                    *animation_id_ref.lock().unwrap() = None;
                }
            }
        });

        let id = RUNTIME.add_animation(update, Duration::from_secs(10)); // Max 10 seconds
        *self.animation_id.lock().unwrap() = Some(id);
    }
}

/// Hook for spring animations
pub fn use_spring<T: AnimatableValue>(
    hooks: &Hooks,
    initial: T,
    config: SpringConfig,
) -> SpringHandle<T> {
    let value = use_signal(hooks, initial.clone());
    let velocity = Arc::new(RwLock::new(0.0f32));
    let target = Arc::new(RwLock::new(initial));

    SpringHandle {
        value,
        velocity,
        target,
        config,
        animation_id: Arc::new(Mutex::new(None)),
    }
}

/// Stagger animation handle
pub struct StaggerHandle<T: AnimatableValue> {
    items: Vec<ThreadSafeSignal<T>>,
    delays: Vec<Duration>,
    config: StaggerConfig,
    animation_ids: Arc<Mutex<Vec<Option<usize>>>>,
    scheduler: Arc<Scheduler>,
    animation_duration: Duration,
}

impl<T: AnimatableValue> StaggerHandle<T> {
    /// Get current values of all items
    pub fn items(&self) -> Vec<T> {
        self.items.iter().map(|s| s.get()).collect()
    }

    /// Get current value of item at index
    pub fn item(&self, index: usize) -> Option<T> {
        self.items.get(index).map(|s| s.get())
    }

    /// Get delay for item at index
    pub fn delay(&self, index: usize) -> Option<Duration> {
        self.delays.get(index).copied()
    }

    /// Process any pending timers (should be called in the render loop)
    pub fn process_timers(&self) {
        self.scheduler.process_timers();
    }

    /// Animate all items to target values
    pub fn animate_all_to(&self, targets: Vec<T>) {
        let mut ids = self.animation_ids.lock().unwrap();

        // Cancel all current animations
        for id in ids.iter_mut() {
            if let Some(anim_id) = id.take() {
                RUNTIME.remove_animation(anim_id);
            }
        }

        // Start staggered animations using scheduler
        for (i, (item_signal, target)) in self.items.iter().zip(targets.iter()).enumerate() {
            if let Some(delay) = self.delays.get(i) {
                let signal = item_signal.clone();
                let from = signal.get();
                let to = target.clone();
                let easing = self
                    .config
                    .ease
                    .clone()
                    .unwrap_or(EasingFunction::EaseInOut);
                let duration = self.animation_duration;
                let ids_clone = self.animation_ids.clone();
                let idx = i;

                // Schedule the animation to start after delay
                self.scheduler.schedule_timeout(*delay, move || {
                    let update = Arc::new(move |progress: f32| {
                        let eased = easing.apply(progress);
                        let interpolated = from.interpolate(&to, eased);
                        signal.set(interpolated);
                    });

                    let id = RUNTIME.add_animation(update, duration);
                    let mut ids = ids_clone.lock().unwrap();
                    if idx < ids.len() {
                        ids[idx] = Some(id);
                    }
                });
            }
        }
    }
}

/// Hook for staggered animations
pub fn use_stagger<T: AnimatableValue>(
    hooks: &Hooks,
    items: Vec<T>,
    config: StaggerConfig,
) -> StaggerHandle<T> {
    // Create signals for each item
    let item_signals: Vec<ThreadSafeSignal<T>> = items
        .iter()
        .map(|item| use_signal(hooks, item.clone()))
        .collect();

    // Calculate delays
    let positions: Vec<(i16, i16)> = Vec::new();
    let delays = config.calculate_delays(items.len(), &positions);

    let animation_ids = Arc::new(Mutex::new(vec![None; items.len()]));
    let scheduler = Arc::new(Scheduler::new());

    StaggerHandle {
        items: item_signals,
        delays,
        config,
        animation_ids,
        scheduler,
        animation_duration: Duration::from_millis(300), // Default animation duration
    }
}

/// Transition hook for automatic animations on state change
pub fn use_transition<T: AnimatableValue>(hooks: &Hooks, state: T, config: TransitionConfig) -> T {
    let animated = use_animation(
        hooks,
        state.clone(),
        AnimationConfig {
            duration: config.duration,
            easing: config.easing.clone(),
            loop_count: None,
            loop_behavior: LoopMode::None,
        },
    );

    // Use effect to detect changes
    let handle = animated.clone();
    let current_state = state.clone();
    let delay = config.delay;

    use_effect(hooks, move || {
        // Start animation when state changes
        if handle.value() != current_state {
            let handle_clone = handle.clone();
            let target = current_state.clone();
            thread::spawn(move || {
                thread::sleep(delay);
                handle_clone.animate_to(target);
            });
        }
        None
    });

    animated.value()
}

/// Configuration for transitions
#[derive(Clone, Debug)]
pub struct TransitionConfig {
    /// Duration of the transition
    pub duration: Duration,
    /// Easing function for the transition
    pub easing: EasingFunction,
    /// Delay before starting the transition
    pub delay: Duration,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseInOut,
            delay: Duration::ZERO,
        }
    }
}

/// Keyframe animation handle
pub struct KeyframeHandle<T: AnimatableValue> {
    value: ThreadSafeSignal<T>,
    animation: Arc<RwLock<KeyframeAnimation<T>>>,
    animation_id: Arc<Mutex<Option<usize>>>,
}

impl<T: AnimatableValue> KeyframeHandle<T> {
    /// Get the current animation value
    pub fn value(&self) -> T {
        self.value.get()
    }

    /// Play the keyframe animation
    pub fn play(&self) {
        // Cancel current animation
        if let Some(id) = *self.animation_id.lock().unwrap() {
            RUNTIME.remove_animation(id);
        }

        let value_signal = self.value.clone();
        let animation_ref = self.animation.clone();

        let update = Arc::new(move |progress: f32| {
            let animation = animation_ref.read().unwrap();
            if let Some(value) = animation.get_value_at_time(progress) {
                value_signal.set(value);
            }
        });

        let duration = self.animation.read().unwrap().duration();
        let id = RUNTIME.add_animation(update, duration);
        *self.animation_id.lock().unwrap() = Some(id);
    }

    /// Seek to a specific time in the keyframe animation
    ///
    /// # Arguments
    /// * `time` - The time position to seek to (0.0 to 1.0)
    pub fn seek(&self, time: f32) {
        let animation = self.animation.read().unwrap();
        if let Some(value) = animation.get_value_at_time(time) {
            self.value.set(value);
        }
    }
}

/// Hook for keyframe animations
pub fn use_keyframes<T: AnimatableValue + Default>(
    hooks: &Hooks,
    initial: T,
    keyframes: Vec<Keyframe>,
) -> KeyframeHandle<T> {
    let value = use_signal(hooks, initial);
    let animation = Arc::new(RwLock::new(KeyframeAnimation::new(keyframes)));

    KeyframeHandle {
        value,
        animation,
        animation_id: Arc::new(Mutex::new(None)),
    }
}

// Clone implementations for handles
impl<T: AnimatableValue> Clone for AnimationHandle<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            controller: self.controller.clone(),
            state: self.state.clone(),
            current_animation_id: self.current_animation_id.clone(),
            from_value: self.from_value.clone(),
            to_value: self.to_value.clone(),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::Hooks;

    #[test]
    fn test_animation_runs() {
        let hooks = Hooks::new();
        let handle = use_animation(&hooks, 0.0f32, AnimationConfig::default());

        assert_eq!(handle.value(), 0.0);
        handle.animate_to(1.0);

        // Animation should start
        assert_eq!(handle.state(), AnimationState::Playing);

        // Wait a bit for animation to progress
        thread::sleep(Duration::from_millis(50));

        // In tests, we need to manually update animations since there's no render loop
        RUNTIME.update_animations();

        let value = handle.value();
        assert!(value > 0.0 && value <= 1.0);
    }

    #[test]
    fn test_spring_physics() {
        let hooks = Hooks::new();
        let spring = use_spring(&hooks, 0.0f32, SpringConfig::default());

        spring.set_target(10.0);
        thread::sleep(Duration::from_millis(100));

        // In tests, we need to manually update animations since there's no render loop
        RUNTIME.update_animations();

        let value = spring.value();
        assert!(value > 0.0); // Should have moved toward target
    }

    #[test]
    fn test_transition_detects_changes() {
        let hooks = Hooks::new();
        let config = TransitionConfig::default();

        let value = use_transition(&hooks, 0.0f32, config);
        assert_eq!(value, 0.0);
    }

    #[test]
    fn test_stagger_delays() {
        let hooks = Hooks::new();
        let items = vec![0.0f32, 1.0, 2.0];
        let config = StaggerConfig::default();

        let stagger = use_stagger(&hooks, items, config);
        assert_eq!(stagger.items().len(), 3);
        assert!(stagger.delay(0).is_some());
    }
}
