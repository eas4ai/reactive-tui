//! Animation hooks for reactive components

use crate::animation::keyframes::{Keyframe, KeyframeAnimation, KeyframeType};
use crate::animation::stagger::StaggerConfig;
use crate::animation::{
    Animation, AnimationController, AnimationState, EasingFunction, LoopMode, SpringConfig,
};
use crate::reactive::{use_effect, use_signal, Hooks, Scheduler, ThreadSafeSignal};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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

/// Shared animation runtime that manages animation frames.
struct AnimationRuntime {
    running: AtomicBool,
    updating: AtomicBool,
    next_id: AtomicUsize,
    app_subscribers: crate::reactive::wake::Subscriptions,
    animations: RwLock<Vec<AnimationTask>>,
}

#[derive(Clone)]
struct AnimationTask {
    id: usize,
    update: Arc<dyn Fn(f32) + Send + Sync>,
    active: Arc<AtomicBool>,
    start_time: Instant,
    duration: Duration,
}

// Concurrent Apps and callback reentry must not deliver an older frame after a
// newer one. Reentry skips this update; adding work still wakes subscribed Apps.
struct UpdateGuard<'a>(&'a AtomicBool);
impl Drop for UpdateGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl AnimationRuntime {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            running: AtomicBool::new(true),
            updating: AtomicBool::new(false),
            next_id: AtomicUsize::new(0),
            app_subscribers: crate::reactive::wake::Subscriptions::default(),
            animations: RwLock::new(Vec::new()),
        })
    }

    /// Deliver a coherent snapshot without holding the registry lock in callbacks.
    pub fn update_animations(&self) {
        if !self.running.load(Ordering::Acquire)
            || self
                .updating
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
        {
            return;
        }
        let _guard = UpdateGuard(&self.updating);
        let tasks = self.animations.read().unwrap().clone();
        let now = Instant::now();
        for task in tasks {
            if !task.active.load(Ordering::Acquire) {
                continue;
            }
            let progress = if task.duration.is_zero() {
                1.0
            } else {
                (now.saturating_duration_since(task.start_time).as_secs_f64()
                    / task.duration.as_secs_f64())
                .min(1.0) as f32
            };
            (task.update)(progress);
            if progress >= 1.0 {
                self.remove_animation(task.id);
            }
        }
    }

    fn stop(&self) {
        self.running.store(false, Ordering::Release);
        let removed = {
            let mut tasks = self.animations.write().unwrap();
            for task in tasks.iter() {
                task.active.store(false, Ordering::Release);
            }
            std::mem::take(&mut *tasks)
        };
        drop(removed);
    }

    fn add_animation(&self, update: Arc<dyn Fn(f32) + Send + Sync>, duration: Duration) -> usize {
        let id = self
            .next_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("animation ID space exhausted");
        let mut tasks = self.animations.write().unwrap();
        if self.running.load(Ordering::Acquire) {
            tasks.push(AnimationTask {
                id,
                update,
                active: Arc::new(AtomicBool::new(true)),
                start_time: Instant::now(),
                duration,
            });
        }
        drop(tasks);
        self.app_subscribers.notify();
        id
    }

    fn remove_animation(&self, id: usize) {
        let removed = {
            let mut tasks = self.animations.write().unwrap();
            tasks.iter().position(|task| task.id == id).map(|index| {
                tasks[index].active.store(false, Ordering::Release);
                tasks.remove(index)
            })
        };
        drop(removed);
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
    RUNTIME.app_subscribers.track();
    RUNTIME.update_animations();
}

pub(crate) fn has_hook_animations() -> bool {
    RUNTIME.running.load(Ordering::Relaxed) && !RUNTIME.animations.read().unwrap().is_empty()
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

/// Keyframe animation handle. Clones share one component-owned sequence.
#[derive(Clone)]
pub struct KeyframeHandle<T: AnimatableValue + KeyframeType> {
    owner: Arc<KeyframeOwner<T>>,
}

struct KeyframeOwner<T: AnimatableValue + KeyframeType> {
    value: ThreadSafeSignal<T>,
    animation: KeyframeAnimation<T>,
    animation_id: Mutex<Option<usize>>,
    cleanup_registered: AtomicBool,
    generation: AtomicUsize,
    lifetime: std::sync::Weak<crate::reactive::hooks::HookResources>,
}

// Also cancels playback if a render aborts before its effect is committed.
struct KeyframeCleanup<T: AnimatableValue + KeyframeType>(std::sync::Weak<KeyframeOwner<T>>);

impl<T: AnimatableValue + KeyframeType> Drop for KeyframeCleanup<T> {
    fn drop(&mut self) {
        if let Some(owner) = self.0.upgrade() {
            owner.cleanup_registered.store(false, Ordering::Release);
            owner.stop();
        }
    }
}

impl<T: AnimatableValue + KeyframeType> KeyframeOwner<T> {
    fn is_alive(&self) -> bool {
        self.lifetime
            .upgrade()
            .is_some_and(|owner| owner.is_alive())
    }

    fn stop(&self) {
        let id = {
            let mut current = self.animation_id.lock().unwrap();
            self.generation.fetch_add(1, Ordering::AcqRel);
            current.take()
        };
        if let Some(id) = id {
            RUNTIME.remove_animation(id);
        }
    }
}

impl<T: AnimatableValue + KeyframeType> Drop for KeyframeOwner<T> {
    fn drop(&mut self) {
        self.stop();
    }
}

impl<T: AnimatableValue + KeyframeType> KeyframeHandle<T> {
    /// Get the current animation value, including the final value after completion.
    pub fn value(&self) -> T {
        self.owner.value.get()
    }

    /// Restart the retained keyframe sequence. Has no effect after owner cleanup.
    pub fn play(&self) {
        let mut current = self.owner.animation_id.lock().unwrap();
        if let Some(id) = current.take() {
            RUNTIME.remove_animation(id);
        }
        if !self.owner.is_alive() {
            return;
        }
        let generation = self
            .owner
            .generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        let owner = Arc::downgrade(&self.owner);
        let update = Arc::new(move |progress: f32| {
            let Some(owner) = owner.upgrade() else { return };
            if owner.is_alive() {
                if let Some(value) = owner.animation.get_value_at_time(progress) {
                    // Sampling user-defined values can stop or restart playback.
                    // Serialize delivery with cancellation after sampling returns.
                    let current = owner.animation_id.lock().unwrap();
                    if current.is_some()
                        && owner.is_alive()
                        && owner.generation.load(Ordering::Acquire) == generation
                    {
                        owner.value.set(value);
                    }
                }
            }
        });
        *current = Some(RUNTIME.add_animation(update, self.owner.animation.duration()));
    }

    /// Cancel scheduled updates, keeping the most recently delivered value.
    pub fn stop(&self) {
        self.owner.stop();
    }

    /// Sample a normalized time without starting playback.
    /// Has no effect after the component or hook owner has been cleaned up.
    pub fn seek(&self, time: f32) {
        if self.owner.is_alive() {
            if let Some(value) = self.owner.animation.get_value_at_time(time) {
                let _current = self.owner.animation_id.lock().unwrap();
                if self.owner.is_alive() {
                    self.owner.value.set(value);
                }
            }
        }
    }
}

/// Retain the initial keyframe sequence across renders. Cleanup cancels playback,
/// including when a handle escapes the component. Call `play` to restart it.
pub fn use_keyframes<T: AnimatableValue + KeyframeType>(
    hooks: &Hooks,
    initial: T,
    keyframes: Vec<Keyframe>,
) -> KeyframeHandle<T> {
    let lifetime = hooks.resource_token();
    let storage = hooks.get_or_create_storage(crate::reactive::hooks::HookKind::Memo, || {
        Arc::new(KeyframeOwner {
            value: ThreadSafeSignal::new(initial),
            animation: KeyframeAnimation::new(keyframes),
            animation_id: Mutex::new(None),
            cleanup_registered: AtomicBool::new(false),
            generation: AtomicUsize::new(0),
            lifetime,
        })
    });
    let owner = storage.lock().unwrap().clone();
    // Unchanged effects discard their incoming closure. Only the first pending
    // effect owns cancellation; a discarded rerender must not stop playback.
    let cleanup = (!owner.cleanup_registered.swap(true, Ordering::AcqRel))
        .then(|| KeyframeCleanup(Arc::downgrade(&owner)));
    crate::reactive::use_effect_with_deps(hooks, (), move || Some(Box::new(move || drop(cleanup))));
    KeyframeHandle { owner }
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

    #[derive(Clone, Debug, Default, PartialEq)]
    struct CancelOnSample(f32);
    static CANCEL_ON_SAMPLE: Mutex<Option<KeyframeHandle<CancelOnSample>>> = Mutex::new(None);
    impl AnimatableValue for CancelOnSample {
        fn interpolate(&self, to: &Self, t: f32) -> Self {
            Self(self.0 + (to.0 - self.0) * t)
        }
        fn to_f32(&self) -> f32 {
            self.0
        }
        fn from_f32(value: f32) -> Self {
            Self(value)
        }
    }
    impl KeyframeType for CancelOnSample {
        fn interpolate_keyframe(&self, _: &Self, _: f32, _: &EasingFunction) -> Self {
            CANCEL_ON_SAMPLE.lock().unwrap().as_ref().unwrap().stop();
            Self(999.0)
        }
        fn from_keyframe(
            value: &crate::animation::keyframes::KeyframeValue,
        ) -> Result<Self, crate::animation::KeyframeError> {
            f32::from_keyframe(value).map(Self)
        }
    }

    #[test]
    fn keyframe_hook_sampling_can_cancel_without_delivering_a_stale_value() {
        let (send, receive) = std::sync::mpsc::channel();
        let worker = thread::spawn(move || {
            let hooks = Hooks::new();
            let handle = use_keyframes(
                &hooks,
                CancelOnSample(0.0),
                vec![
                    Keyframe::new(0.0).number("x", 0.0),
                    Keyframe::new(1.0).number("x", 10.0),
                ],
            );
            *CANCEL_ON_SAMPLE.lock().unwrap() = Some(handle.clone());
            handle.play();
            let id = handle.owner.animation_id.lock().unwrap().unwrap();
            {
                let mut tasks = RUNTIME.animations.write().unwrap();
                let task = tasks.iter_mut().find(|task| task.id == id).unwrap();
                task.duration = Duration::from_secs(100);
                task.start_time = Instant::now() - Duration::from_secs(50);
            }
            RUNTIME.update_animations();
            assert_eq!(handle.value(), CancelOnSample(0.0));
            assert!(handle.owner.animation_id.lock().unwrap().is_none());
            CANCEL_ON_SAMPLE.lock().unwrap().take();
            hooks.cleanup();
            send.send(()).unwrap();
        });
        receive
            .recv_timeout(Duration::from_secs(2))
            .expect("keyframe interpolation deadlocked during cancellation");
        worker.join().unwrap();
    }

    #[test]
    fn keyframe_hook_delivers_midpoint_and_completion_and_stop_is_isolated() {
        let hooks = Hooks::new();
        let frames = || {
            vec![
                Keyframe::new(0.0).number("x", 2.0),
                Keyframe::new(1.0).number("x", 10.0),
            ]
        };
        let first = use_keyframes(&hooks, 2.0_f32, frames());
        let second = use_keyframes(&hooks, 2.0_f32, frames());
        first.seek(0.5);
        assert_eq!(first.value(), 6.0);
        first.play();
        second.play();
        first.stop();
        let second_id = second.owner.animation_id.lock().unwrap().unwrap();
        {
            let mut tasks = RUNTIME.animations.write().unwrap();
            tasks
                .iter_mut()
                .find(|task| task.id == second_id)
                .unwrap()
                .duration = Duration::ZERO;
        }
        RUNTIME.update_animations();
        assert_eq!(first.value(), 6.0);
        assert_eq!(second.value(), 10.0);
        assert!(!RUNTIME
            .animations
            .read()
            .unwrap()
            .iter()
            .any(|task| task.id == second_id));
        hooks.cleanup();
    }

    #[test]
    fn keyframe_hook_last_context_drop_cancels_escaped_handle() {
        let hooks = Hooks::new();
        let handle = use_keyframes(
            &hooks,
            0.0_f32,
            vec![
                Keyframe::new(0.0).number("x", 0.0),
                Keyframe::new(1.0).number("x", 10.0),
            ],
        );
        handle.play();
        handle.seek(0.5);
        drop(hooks);
        // Other runtime users may have delivered a frame before cleanup.
        let retained = handle.value();
        assert!(handle.owner.animation_id.lock().unwrap().is_none());
        handle.play();
        // Choose an endpoint different from the retained value to detect a live seek.
        handle.seek(if retained == 10.0 { 0.0 } else { 1.0 });
        assert_eq!(handle.value(), retained);
        assert!(handle.owner.animation_id.lock().unwrap().is_none());
    }

    #[test]
    fn keyframe_hook_aborted_render_drops_pending_playback() {
        let hooks = Hooks::new();
        let mut escaped = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _render = hooks.begin_render();
            let handle = use_keyframes(
                &hooks,
                0.0_f32,
                vec![
                    Keyframe::new(0.0).number("x", 0.0),
                    Keyframe::new(1.0).number("x", 10.0),
                ],
            );
            handle.play();
            escaped = Some(handle);
            panic!("deliberately aborted render");
        }));
        assert!(result.is_err());
        let handle = escaped.unwrap();
        assert!(handle.owner.animation_id.lock().unwrap().is_none());
    }

    #[test]
    fn keyframe_hook_cleanup_removes_work_and_disables_escaped_handle() {
        let hooks = Hooks::new();
        let handle = use_keyframes(
            &hooks,
            0.0_f32,
            vec![
                Keyframe::new(0.0).number("x", 0.0),
                Keyframe::new(1.0).number("x", 10.0),
            ],
        );
        handle.play();
        let id = handle.owner.animation_id.lock().unwrap().unwrap();
        hooks.cleanup();
        assert!(!RUNTIME
            .animations
            .read()
            .unwrap()
            .iter()
            .any(|task| task.id == id));
        handle.seek(1.0);
        assert_eq!(handle.value(), 0.0);
        handle.play();
        assert!(handle.owner.animation_id.lock().unwrap().is_none());
    }

    #[test]
    fn keyframe_hook_rerender_retains_the_running_sequence() {
        let hooks = Hooks::new();
        let frames = || {
            vec![
                Keyframe::new(0.0).number("x", 0.0),
                Keyframe::new(1.0).number("x", 10.0),
            ]
        };
        let first = use_keyframes(&hooks, 0.0_f32, frames());
        first.play();
        hooks.reset();
        let second = use_keyframes(&hooks, 0.0_f32, frames());
        assert!(Arc::ptr_eq(&first.owner, &second.owner));
        assert!(
            second.owner.animation_id.lock().unwrap().is_some(),
            "rerender cancelled playback"
        );
        hooks.cleanup();
    }

    #[test]
    fn keyframe_runtime_releases_callback_captures_outside_its_lock() {
        struct ReenterOnDrop(Arc<AnimationRuntime>);
        impl Drop for ReenterOnDrop {
            fn drop(&mut self) {
                let id = self
                    .0
                    .add_animation(Arc::new(|_| {}), Duration::from_secs(10));
                self.0.remove_animation(id);
            }
        }
        for stop in [false, true] {
            let (send, receive) = std::sync::mpsc::channel();
            let worker = thread::spawn(move || {
                let runtime = AnimationRuntime::new();
                let captured = ReenterOnDrop(runtime.clone());
                let id = runtime.add_animation(
                    Arc::new(move |_| {
                        std::hint::black_box(&captured);
                    }),
                    Duration::from_secs(10),
                );
                if stop {
                    runtime.stop();
                } else {
                    runtime.remove_animation(id);
                }
                send.send(()).unwrap();
            });
            receive
                .recv_timeout(Duration::from_secs(2))
                .expect("callback capture drop could not reenter the runtime");
            worker.join().unwrap();
        }
    }

    #[test]
    fn keyframe_runtime_delivers_the_final_update_before_removal() {
        let runtime = AnimationRuntime::new();
        let delivered = Arc::new(Mutex::new(Vec::new()));
        let output = delivered.clone();
        runtime.add_animation(
            Arc::new(move |progress| output.lock().unwrap().push(progress)),
            Duration::ZERO,
        );
        runtime.update_animations();
        assert_eq!(*delivered.lock().unwrap(), vec![1.0]);
        assert!(runtime.animations.read().unwrap().is_empty());
    }

    #[test]
    fn keyframe_runtime_ids_do_not_reuse_a_live_animation_after_cancellation() {
        let runtime = AnimationRuntime::new();
        let first = runtime.add_animation(Arc::new(|_| {}), Duration::from_secs(10));
        let second = runtime.add_animation(Arc::new(|_| {}), Duration::from_secs(10));
        runtime.remove_animation(first);
        let third = runtime.add_animation(Arc::new(|_| {}), Duration::from_secs(10));
        assert_ne!(second, third);
        runtime.remove_animation(second);
        assert_eq!(runtime.animations.read().unwrap().len(), 1);
        assert_eq!(runtime.animations.read().unwrap()[0].id, third);
    }

    #[test]
    fn keyframe_runtime_callback_can_start_and_cancel_an_animation() {
        let (send, receive) = std::sync::mpsc::channel();
        let worker = thread::spawn(move || {
            let runtime = AnimationRuntime::new();
            let target = runtime.clone();
            runtime.add_animation(
                Arc::new(move |_| {
                    let id = target.add_animation(Arc::new(|_| {}), Duration::from_secs(10));
                    target.remove_animation(id);
                }),
                Duration::from_secs(10),
            );
            runtime.update_animations();
            send.send(()).unwrap();
        });
        // A failing isolated test process exits rather than joining a deadlocked worker.
        receive
            .recv_timeout(Duration::from_secs(2))
            .expect("animation callback could not reenter the runtime");
        worker.join().unwrap();
    }

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
