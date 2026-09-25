//! Animation hooks for reactive components

use crate::animation::keyframes::{Keyframe, KeyframeAnimation, KeyframeType};
use crate::animation::stagger::StaggerConfig;
use crate::animation::{
    Animation, AnimationController, AnimationState, EasingFunction, LoopMode, SpringConfig,
};
use crate::reactive::hooks::{HookKind, Liveness};
use crate::reactive::scheduler::TimerId;
use crate::reactive::{use_effect_with_deps, Hooks, Scheduler, ThreadSafeSignal};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};
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
        self.deliver();
    }

    /// One pass over a snapshot of the tasks. The caller holds `updating`.
    fn deliver(&self) {
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

/// Animation handle. Clones share one component-owned animation.
#[derive(Clone)]
pub struct AnimationHandle<T: AnimatableValue> {
    owner: Arc<AnimationOwner<T>>,
}

struct AnimationOwner<T: AnimatableValue> {
    value: ThreadSafeSignal<T>,
    controller: Mutex<AnimationController>,
    state: ThreadSafeSignal<AnimationState>,
    current_animation_id: Mutex<Option<usize>>,
    from_value: RwLock<T>,
    to_value: RwLock<T>,
    config: RwLock<AnimationConfig>,
    scheduler: Arc<Scheduler>,
    pending_transition: Mutex<Option<TimerId>>,
    transition_request: Mutex<Option<(T, Duration)>>,
    cleanup_registered: AtomicBool,
    generation: AtomicUsize,
    lifetime: Liveness,
}

struct AnimationCleanup<T: AnimatableValue>(Weak<AnimationOwner<T>>);

impl<T: AnimatableValue> Drop for AnimationCleanup<T> {
    fn drop(&mut self) {
        if let Some(owner) = self.0.upgrade() {
            owner.cleanup_registered.store(false, Ordering::Release);
            owner.cancel_owned_work();
        }
    }
}

impl<T: AnimatableValue> AnimationOwner<T> {
    fn is_alive(&self) -> bool {
        self.lifetime.is_alive()
    }

    fn cancel_owned_work(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        if let Some(id) = self.current_animation_id.lock().unwrap().take() {
            RUNTIME.remove_animation(id);
        }
        if let Some(id) = self.pending_transition.lock().unwrap().take() {
            self.scheduler.cancel_timer(id);
        }
    }

    fn start(self: &Arc<Self>, target: T) {
        self.cancel_owned_work();
        if !self.is_alive() {
            return;
        }
        let generation = self
            .generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        *self.from_value.write().unwrap() = self.value.get();
        *self.to_value.write().unwrap() = target;
        self.controller.lock().unwrap().start();
        self.state.set(AnimationState::Playing);
        let config = self.config.read().unwrap().clone();
        let owner = Arc::downgrade(self);
        let update = Arc::new(move |progress: f32| {
            let Some(owner) = owner.upgrade() else { return };
            if !owner.is_alive() || owner.generation.load(Ordering::Acquire) != generation {
                return;
            }
            if !owner.controller.lock().unwrap().is_playing() {
                return;
            }
            let value = owner.from_value.read().unwrap().interpolate(
                &owner.to_value.read().unwrap(),
                config.easing.apply(progress),
            );
            let current = owner.current_animation_id.lock().unwrap();
            if owner.is_alive()
                && owner.generation.load(Ordering::Acquire) == generation
                && current.is_some()
            {
                owner.value.set(value);
                if progress >= 1.0 {
                    owner.controller.lock().unwrap().stop();
                    owner.state.set(AnimationState::Stopped);
                }
            }
        });
        let id = RUNTIME.add_animation(update, config.duration);
        *self.current_animation_id.lock().unwrap() = Some(id);
    }

    fn schedule_transition(self: &Arc<Self>, target: T, delay: Duration) {
        let request = (target.clone(), delay);
        let mut prior = self.transition_request.lock().unwrap();
        if prior.as_ref() == Some(&request) {
            return;
        }
        *prior = Some(request);
        drop(prior);
        if let Some(id) = self.pending_transition.lock().unwrap().take() {
            self.scheduler.cancel_timer(id);
        }
        if !self.is_alive() || self.value.get() == target {
            return;
        }
        let owner = Arc::downgrade(self);
        let id = self.scheduler.schedule_timeout(delay, move || {
            let Some(owner) = owner.upgrade() else { return };
            owner.pending_transition.lock().unwrap().take();
            if owner.is_alive() {
                owner.start(target);
            }
        });
        *self.pending_transition.lock().unwrap() = Some(id);
    }
}

impl<T: AnimatableValue> AnimationHandle<T> {
    /// Get the current animation value
    pub fn value(&self) -> T {
        self.owner.value.get()
    }

    /// Animate to a target value
    pub fn animate_to(&self, target: T) {
        self.owner.start(target);
    }

    /// Pause the current animation
    pub fn pause(&self) {
        if self.owner.is_alive() {
            self.owner.controller.lock().unwrap().pause();
            self.owner.state.set(AnimationState::Paused);
        }
    }

    /// Resume a paused animation
    pub fn resume(&self) {
        if self.owner.is_alive() {
            self.owner.controller.lock().unwrap().resume();
            self.owner.state.set(AnimationState::Playing);
        }
    }

    /// Stop the current animation
    pub fn stop(&self) {
        self.owner.cancel_owned_work();
        self.owner.controller.lock().unwrap().stop();
        if self.owner.is_alive() {
            self.owner.state.set(AnimationState::Stopped);
        }
    }

    /// Reset the animation to its initial value
    pub fn reset(&self) {
        self.stop();
        let from = self.owner.from_value.read().unwrap().clone();
        if self.owner.is_alive() {
            self.owner.value.set(from);
        }
    }

    /// Get the current animation state
    pub fn state(&self) -> AnimationState {
        self.owner.state.get()
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
    let lifetime = hooks.liveness();
    let scheduler = crate::hooks::timer::get_scheduler(hooks);
    let initial_config = config.clone();
    let storage = hooks.get_or_create_storage(HookKind::Memo, || {
        let animation = Animation::new(
            initial_config.duration,
            initial_config.easing.clone(),
            initial_config.loop_count,
            initial_config.loop_behavior,
        );
        Arc::new(AnimationOwner {
            value: ThreadSafeSignal::new(initial.clone()),
            controller: Mutex::new(AnimationController::new(animation)),
            state: ThreadSafeSignal::new(AnimationState::Stopped),
            current_animation_id: Mutex::new(None),
            from_value: RwLock::new(initial.clone()),
            to_value: RwLock::new(initial),
            config: RwLock::new(initial_config),
            scheduler,
            pending_transition: Mutex::new(None),
            transition_request: Mutex::new(None),
            cleanup_registered: AtomicBool::new(false),
            generation: AtomicUsize::new(0),
            lifetime,
        })
    });
    let owner = storage.lock().unwrap().clone();
    *owner.config.write().unwrap() = config;
    let cleanup = (!owner.cleanup_registered.swap(true, Ordering::AcqRel))
        .then(|| AnimationCleanup(Arc::downgrade(&owner)));
    use_effect_with_deps(hooks, (), move || Some(Box::new(move || drop(cleanup))));
    AnimationHandle { owner }
}

/// Spring animation handle. Clones share one component-owned spring.
#[derive(Clone)]
pub struct SpringHandle<T: AnimatableValue> {
    owner: Arc<SpringOwner<T>>,
}

struct SpringOwner<T: AnimatableValue> {
    value: ThreadSafeSignal<T>,
    velocity: RwLock<f32>,
    target: RwLock<T>,
    config: RwLock<SpringConfig>,
    animation_id: Mutex<Option<usize>>,
    cleanup_registered: AtomicBool,
    generation: AtomicUsize,
    lifetime: Liveness,
}

struct SpringCleanup<T: AnimatableValue>(Weak<SpringOwner<T>>);

impl<T: AnimatableValue> Drop for SpringCleanup<T> {
    fn drop(&mut self) {
        if let Some(owner) = self.0.upgrade() {
            owner.cleanup_registered.store(false, Ordering::Release);
            owner.cancel_owned_work();
        }
    }
}

impl<T: AnimatableValue> SpringOwner<T> {
    fn is_alive(&self) -> bool {
        self.lifetime.is_alive()
    }

    fn cancel_owned_work(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        if let Some(id) = self.animation_id.lock().unwrap().take() {
            RUNTIME.remove_animation(id);
        }
    }
}

impl<T: AnimatableValue> SpringHandle<T> {
    /// Get the current spring value
    pub fn value(&self) -> T {
        self.owner.value.get()
    }

    /// Get the current spring velocity
    pub fn velocity(&self) -> f32 {
        *self.owner.velocity.read().unwrap()
    }

    /// Apply an impulse force to the spring
    pub fn apply_impulse(&self, force: f32) {
        let mut vel = self.owner.velocity.write().unwrap();
        *vel += force;
        drop(vel);
        self.start_spring_animation();
    }

    /// Set a new target value for the spring to animate to
    pub fn set_target(&self, target: T) {
        *self.owner.target.write().unwrap() = target;
        self.start_spring_animation();
    }

    fn start_spring_animation(&self) {
        self.owner.cancel_owned_work();
        if !self.owner.is_alive() {
            return;
        }
        let generation = self
            .owner
            .generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        let owner = Arc::downgrade(&self.owner);
        let update = Arc::new(move |_progress: f32| {
            let Some(owner) = owner.upgrade() else { return };
            if !owner.is_alive() || owner.generation.load(Ordering::Acquire) != generation {
                return;
            }
            let current = owner.value.get().to_f32();
            let target = owner.target.read().unwrap().to_f32();
            let config = owner.config.read().unwrap().clone();
            let position = config.calculate_position(0.016, current, target); // 60fps frame
            let new_velocity = config.calculate_velocity(0.016, current, target);
            let mut animation_id = owner.animation_id.lock().unwrap();
            if !owner.is_alive() || owner.generation.load(Ordering::Acquire) != generation {
                return;
            }
            *owner.velocity.write().unwrap() = new_velocity;
            owner.value.set(T::from_f32(position));
            if (position - target).abs() < 0.001f32 && new_velocity.abs() < 0.001f32 {
                if let Some(id) = animation_id.take() {
                    RUNTIME.remove_animation(id);
                }
            }
        });
        let id = RUNTIME.add_animation(update, Duration::from_secs(10)); // Max 10 seconds
        *self.owner.animation_id.lock().unwrap() = Some(id);
    }
}

/// Hook for spring animations
pub fn use_spring<T: AnimatableValue>(
    hooks: &Hooks,
    initial: T,
    config: SpringConfig,
) -> SpringHandle<T> {
    let lifetime = hooks.liveness();
    let initial_config = config.clone();
    let storage = hooks.get_or_create_storage(HookKind::Memo, || {
        Arc::new(SpringOwner {
            value: ThreadSafeSignal::new(initial.clone()),
            velocity: RwLock::new(0.0),
            target: RwLock::new(initial),
            config: RwLock::new(initial_config),
            animation_id: Mutex::new(None),
            cleanup_registered: AtomicBool::new(false),
            generation: AtomicUsize::new(0),
            lifetime,
        })
    });
    let owner = storage.lock().unwrap().clone();
    *owner.config.write().unwrap() = config;
    let cleanup = (!owner.cleanup_registered.swap(true, Ordering::AcqRel))
        .then(|| SpringCleanup(Arc::downgrade(&owner)));
    use_effect_with_deps(hooks, (), move || Some(Box::new(move || drop(cleanup))));
    SpringHandle { owner }
}

/// Stagger animation handle. Clones share one component-owned sequence.
#[derive(Clone)]
pub struct StaggerHandle<T: AnimatableValue> {
    owner: Arc<StaggerOwner<T>>,
}

struct StaggerOwner<T: AnimatableValue> {
    items: Vec<ThreadSafeSignal<T>>,
    delays: RwLock<Vec<Duration>>,
    config: RwLock<StaggerConfig>,
    animation_ids: Mutex<Vec<Option<usize>>>,
    timer_ids: Mutex<Vec<Option<TimerId>>>,
    scheduler: Arc<Scheduler>,
    animation_duration: Duration,
    cleanup_registered: AtomicBool,
    generation: AtomicUsize,
    lifetime: Liveness,
}

struct StaggerCleanup<T: AnimatableValue>(Weak<StaggerOwner<T>>);

impl<T: AnimatableValue> Drop for StaggerCleanup<T> {
    fn drop(&mut self) {
        if let Some(owner) = self.0.upgrade() {
            owner.cleanup_registered.store(false, Ordering::Release);
            owner.cancel_owned_work();
        }
    }
}

impl<T: AnimatableValue> StaggerOwner<T> {
    fn is_alive(&self) -> bool {
        self.lifetime.is_alive()
    }

    fn cancel_owned_work(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        let animations = std::mem::take(&mut *self.animation_ids.lock().unwrap());
        for id in animations.into_iter().flatten() {
            RUNTIME.remove_animation(id);
        }
        let timers = std::mem::take(&mut *self.timer_ids.lock().unwrap());
        for id in timers.into_iter().flatten() {
            self.scheduler.cancel_timer(id);
        }
        // An update pass that checked the old generation may be storing an
        // item now; wait for it, so none stores after this returns.
        for item in &self.items {
            item.settle();
        }
    }

    fn start_sequence(self: &Arc<Self>, targets: Vec<T>) {
        self.cancel_owned_work();
        if !self.is_alive() {
            return;
        }
        let generation = self
            .generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        let delays = self.delays.read().unwrap().clone();
        let easing = self
            .config
            .read()
            .unwrap()
            .ease
            .clone()
            .unwrap_or(EasingFunction::EaseInOut);
        let count = self.items.len().min(targets.len());
        *self.animation_ids.lock().unwrap() = vec![None; count];
        *self.timer_ids.lock().unwrap() = vec![None; count];
        for (index, target) in targets.into_iter().take(count).enumerate() {
            self.schedule_item(index, target, delays[index], easing.clone(), generation);
        }
    }

    fn schedule_item(
        self: &Arc<Self>,
        index: usize,
        target: T,
        delay: Duration,
        easing: EasingFunction,
        generation: usize,
    ) {
        let owner = Arc::downgrade(self);
        let duration = self.animation_duration;
        let timer = self.scheduler.schedule_timeout(delay, move || {
            let Some(owner) = owner.upgrade() else { return };
            if !owner.is_alive() || owner.generation.load(Ordering::Acquire) != generation {
                return;
            }
            if let Some(timer) = owner.timer_ids.lock().unwrap().get_mut(index) {
                *timer = None;
            }
            let from = owner.items[index].get();
            let update_owner = Arc::downgrade(&owner);
            let update = Arc::new(move |progress: f32| {
                let Some(owner) = update_owner.upgrade() else {
                    return;
                };
                if !owner.is_alive() || owner.generation.load(Ordering::Acquire) != generation {
                    return;
                }
                let value = from.interpolate(&target, easing.apply(progress));
                // The checks run again inside the store. cancel_owned_work
                // settles every item after raising the generation, so a pass
                // stores before cancellation returns or not at all.
                owner.items[index].set_if(value, || {
                    owner.is_alive()
                        && owner.generation.load(Ordering::Acquire) == generation
                        && owner
                            .animation_ids
                            .lock()
                            .unwrap()
                            .get(index)
                            .is_some_and(Option::is_some)
                });
            });
            let id = RUNTIME.add_animation(update, duration);
            let mut ids = owner.animation_ids.lock().unwrap();
            if owner.is_alive() && owner.generation.load(Ordering::Acquire) == generation {
                ids[index] = Some(id);
            } else {
                RUNTIME.remove_animation(id);
            }
        });
        self.timer_ids.lock().unwrap()[index] = Some(timer);
    }
}

impl<T: AnimatableValue> StaggerHandle<T> {
    /// Get current values of all items
    pub fn items(&self) -> Vec<T> {
        self.owner.items.iter().map(|s| s.get()).collect()
    }

    /// Get current value of item at index
    pub fn item(&self, index: usize) -> Option<T> {
        self.owner.items.get(index).map(|s| s.get())
    }

    /// Get delay for item at index
    pub fn delay(&self, index: usize) -> Option<Duration> {
        self.owner.delays.read().unwrap().get(index).copied()
    }

    /// Process any pending timers (should be called in the render loop)
    pub fn process_timers(&self) {
        self.owner.scheduler.process_timers();
    }

    /// Animate all items to target values
    pub fn animate_all_to(&self, targets: Vec<T>) {
        self.owner.start_sequence(targets);
    }
}

/// Hook for staggered animations
pub fn use_stagger<T: AnimatableValue>(
    hooks: &Hooks,
    items: Vec<T>,
    config: StaggerConfig,
) -> StaggerHandle<T> {
    let lifetime = hooks.liveness();
    let scheduler = crate::hooks::timer::get_scheduler(hooks);
    let initial_config = config.clone();
    let item_count = items.len();
    let storage = hooks.get_or_create_storage(HookKind::Memo, || {
        Arc::new(StaggerOwner {
            items: items.into_iter().map(ThreadSafeSignal::new).collect(),
            delays: RwLock::new(initial_config.calculate_delays(item_count, &[])),
            config: RwLock::new(initial_config),
            animation_ids: Mutex::new(Vec::new()),
            timer_ids: Mutex::new(Vec::new()),
            scheduler,
            animation_duration: Duration::from_millis(300),
            cleanup_registered: AtomicBool::new(false),
            generation: AtomicUsize::new(0),
            lifetime,
        })
    });
    let owner = storage.lock().unwrap().clone();
    *owner.delays.write().unwrap() = config.calculate_delays(owner.items.len(), &[]);
    *owner.config.write().unwrap() = config;
    let cleanup = (!owner.cleanup_registered.swap(true, Ordering::AcqRel))
        .then(|| StaggerCleanup(Arc::downgrade(&owner)));
    use_effect_with_deps(hooks, (), move || Some(Box::new(move || drop(cleanup))));
    StaggerHandle { owner }
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

    let handle = animated.clone();
    let delay = config.delay;
    let deps = (state.clone(), delay);
    use_effect_with_deps(hooks, deps, move || {
        handle.owner.schedule_transition(state, delay);
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
    lifetime: Liveness,
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
        self.lifetime.is_alive()
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
    let lifetime = hooks.liveness();
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

#[cfg(test)]
// Every test here shares the global animation RUNTIME, whose update
// guard silently skips updates under contention. Running them in
// parallel makes exact post-unmount assertions flaky, so each test is
// serialized with the repo-standard `serial_test::serial` attribute.
// Apps in other modules' tests also run passes on RUNTIME, so these
// tests update it with `run_pass`, never `update_animations`.
mod tests {
    use super::*;
    use crate::reactive::Hooks;
    use std::thread;

    /// Run one update pass on RUNTIME and return when it has delivered.
    /// `update_animations` skips its pass while another thread's runs;
    /// this waits for the runtime instead, so every pass begun before the
    /// call has ended when this one delivers.
    fn run_pass() {
        without_other_passes(|| {
            if RUNTIME.running.load(Ordering::Acquire) {
                RUNTIME.deliver();
            }
        });
    }

    /// Run `work` while no other thread can run a pass on RUNTIME: wait for
    /// a running pass to end, then hold the update guard, so passes other
    /// threads start meanwhile are skipped. `work` must not update RUNTIME
    /// itself except through `RUNTIME.deliver`.
    fn without_other_passes<R>(work: impl FnOnce() -> R) -> R {
        let deadline = Instant::now() + Duration::from_secs(5);
        while RUNTIME
            .updating
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            assert!(
                Instant::now() < deadline,
                "an animation update pass never ended"
            );
            thread::yield_now();
        }
        let _guard = UpdateGuard(&RUNTIME.updating);
        work()
    }

    #[derive(Clone, Copy)]
    struct Rac001Observation {
        owner_count: usize,
        runtime_tasks_after_unmount: usize,
        scheduler_timers_after_unmount: usize,
        updates_after_unmount: usize,
        thread_growth: usize,
        stagger_timers_on_component_scheduler: usize,
    }

    fn validate_rac_001(observation: Rac001Observation) -> Result<(), &'static str> {
        if observation.owner_count != 1 {
            return Err("rerender replaced the scoped state owner");
        }
        if observation.runtime_tasks_after_unmount != 0 {
            return Err("unmount left animation-runtime work active");
        }
        if observation.scheduler_timers_after_unmount != 0 {
            return Err("unmount left component-scheduler work active");
        }
        if observation.updates_after_unmount != 0 {
            return Err("a closed owner received an update");
        }
        if observation.thread_growth != 0 {
            return Err("frame progress created operating-system threads");
        }
        if observation.stagger_timers_on_component_scheduler == 0 {
            return Err("stagger work is absent from the component scheduler");
        }
        Ok(())
    }

    #[test]
    #[serial_test::serial]
    fn rac_001_validator_rejects_each_controlled_violation() {
        let valid = Rac001Observation {
            owner_count: 1,
            runtime_tasks_after_unmount: 0,
            scheduler_timers_after_unmount: 0,
            updates_after_unmount: 0,
            thread_growth: 0,
            stagger_timers_on_component_scheduler: 1,
        };
        assert!(validate_rac_001(valid).is_ok());
        for invalid in [
            Rac001Observation {
                owner_count: 2,
                ..valid
            },
            Rac001Observation {
                runtime_tasks_after_unmount: 1,
                ..valid
            },
            Rac001Observation {
                scheduler_timers_after_unmount: 1,
                ..valid
            },
            Rac001Observation {
                updates_after_unmount: 1,
                ..valid
            },
            Rac001Observation {
                thread_growth: 1,
                ..valid
            },
            Rac001Observation {
                stagger_timers_on_component_scheduler: 0,
                ..valid
            },
        ] {
            assert!(validate_rac_001(invalid).is_err());
        }
    }

    #[test]
    #[serial_test::serial]
    fn rac_001_hook_source_forbids_threads_and_unscoped_schedulers() {
        use syn::visit::Visit;

        struct HookCalls {
            current: Option<String>,
            violations: Vec<String>,
        }

        impl<'ast> Visit<'ast> for HookCalls {
            fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
                let name = function.sig.ident.to_string();
                let inspected = matches!(
                    name.as_str(),
                    "use_animation" | "use_spring" | "use_stagger" | "use_transition"
                );
                if inspected {
                    self.current = Some(name);
                    syn::visit::visit_block(self, &function.block);
                    self.current = None;
                }
            }

            fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
                if let (Some(hook), syn::Expr::Path(path)) = (&self.current, &*call.func) {
                    let path = path
                        .path
                        .segments
                        .iter()
                        .map(|segment| segment.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");
                    if path.ends_with("thread::spawn") || path == "Scheduler::new" {
                        self.violations.push(format!("{hook} calls {path}"));
                    }
                }
                syn::visit::visit_expr_call(self, call);
            }
        }

        let syntax = syn::parse_file(include_str!("animation.rs")).unwrap();
        let mut calls = HookCalls {
            current: None,
            violations: Vec::new(),
        };
        calls.visit_file(&syntax);
        assert!(
            calls.violations.is_empty(),
            "animation hooks must use retained owners and the component scheduler: {:?}",
            calls.violations
        );
    }

    #[test]
    #[serial_test::serial]
    fn rac_001_animation_and_spring_retain_one_owner_and_cancel_on_unmount() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = crate::reactive::component_scope::ComponentScope::new(scheduler);
        let animation_hooks = Hooks::new();
        let spring_hooks = Hooks::new();
        let animation_config = AnimationConfig {
            duration: Duration::from_secs(60),
            ..AnimationConfig::default()
        };

        let animation = {
            let _scope = scope.enter(true);
            let _frame = animation_hooks.begin_render();
            use_animation(&animation_hooks, 0.0_f32, animation_config.clone())
        };
        animation.animate_to(10.0);
        let animation_id = animation
            .owner
            .current_animation_id
            .lock()
            .unwrap()
            .unwrap();
        let rerendered_animation = {
            let _scope = scope.enter(true);
            let _frame = animation_hooks.begin_render();
            use_animation(&animation_hooks, 99.0_f32, animation_config)
        };
        assert!(Arc::ptr_eq(&animation.owner, &rerendered_animation.owner));
        assert_eq!(
            *rerendered_animation
                .owner
                .current_animation_id
                .lock()
                .unwrap(),
            Some(animation_id),
            "rerender must not replace or restart the active animation"
        );

        let spring = {
            let _scope = scope.enter(true);
            let _frame = spring_hooks.begin_render();
            use_spring(&spring_hooks, 0.0_f32, SpringConfig::default())
        };
        spring.set_target(10.0);
        let spring_id = spring.owner.animation_id.lock().unwrap().unwrap();
        let rerendered_spring = {
            let _scope = scope.enter(true);
            let _frame = spring_hooks.begin_render();
            use_spring(&spring_hooks, 99.0_f32, SpringConfig::default())
        };
        assert!(Arc::ptr_eq(&spring.owner, &rerendered_spring.owner));
        assert_eq!(
            *rerendered_spring.owner.animation_id.lock().unwrap(),
            Some(spring_id),
            "rerender must not replace or restart the active spring"
        );

        run_pass();
        scope.close();
        let live_ids = RUNTIME
            .animations
            .read()
            .unwrap()
            .iter()
            .map(|task| task.id)
            .collect::<Vec<_>>();
        assert!(!live_ids.contains(&animation_id));
        assert!(!live_ids.contains(&spring_id));
        // close() has waited for any pass that was storing a value, so
        // nothing changes from here on.
        let animation_value = animation.value();
        let spring_value = spring.value();
        run_pass();
        assert_eq!(animation.value(), animation_value);
        assert_eq!(spring.value(), spring_value);
        animation.animate_to(20.0);
        spring.set_target(20.0);
        assert!(animation
            .owner
            .current_animation_id
            .lock()
            .unwrap()
            .is_none());
        assert!(spring.owner.animation_id.lock().unwrap().is_none());
    }

    #[test]
    #[serial_test::serial]
    fn rac_001_transition_reuses_one_scheduler_timer_without_thread_growth() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = crate::reactive::component_scope::ComponentScope::new(scheduler.clone());
        let hooks = Hooks::new();
        let config = TransitionConfig {
            delay: Duration::from_secs(3600),
            ..TransitionConfig::default()
        };
        let render = |state| {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            use_transition(&hooks, state, config.clone())
        };
        assert_eq!(render(0.0_f32), 0.0);
        let threads_before = std::fs::read_dir("/proc/self/task")
            .ok()
            .map(|entries| entries.count());
        assert_eq!(render(10.0), 0.0);
        let deadline = scheduler.next_deadline().expect("transition timer missing");
        for _ in 0..32 {
            assert_eq!(render(10.0), 0.0);
            assert_eq!(scheduler.next_deadline(), Some(deadline));
            run_pass();
        }
        let thread_growth = threads_before
            .zip(
                std::fs::read_dir("/proc/self/task")
                    .ok()
                    .map(|entries| entries.count()),
            )
            .map_or(0, |(before, after)| after.saturating_sub(before));
        assert!(
            thread_growth < 8,
            "repeated frames grew the process by {thread_growth} threads"
        );
        scope.close();
        assert!(scheduler.next_deadline().is_none());
    }

    #[test]
    #[serial_test::serial]
    fn rac_001_stagger_uses_component_scheduler_and_cancels_all_work() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = crate::reactive::component_scope::ComponentScope::new(scheduler.clone());
        let hooks = Hooks::new();
        let render = || {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            use_stagger(&hooks, vec![0.0_f32, 1.0, 2.0], StaggerConfig::default())
        };
        let stagger = render();
        stagger.animate_all_to(vec![10.0, 11.0, 12.0]);
        let deadline = scheduler.next_deadline().expect("stagger timer missing");
        let rerendered = render();
        assert!(Arc::ptr_eq(&stagger.owner, &rerendered.owner));
        assert_eq!(scheduler.next_deadline(), Some(deadline));
        scheduler.process_timers();
        let animation_ids = stagger
            .owner
            .animation_ids
            .lock()
            .unwrap()
            .iter()
            .copied()
            .flatten()
            .collect::<Vec<_>>();
        assert!(!animation_ids.is_empty());
        run_pass();
        scope.close();
        assert!(scheduler.next_deadline().is_none());
        let live_ids = RUNTIME
            .animations
            .read()
            .unwrap()
            .iter()
            .map(|task| task.id)
            .collect::<Vec<_>>();
        assert!(animation_ids.iter().all(|id| !live_ids.contains(id)));
        // close() has waited for any pass that was storing an item, so
        // nothing changes from here on.
        let values = stagger.items();
        scheduler.process_timers();
        run_pass();
        assert_eq!(stagger.items(), values);
        stagger.animate_all_to(vec![20.0, 21.0, 22.0]);
        assert!(scheduler.next_deadline().is_none());
    }

    /// A value whose interpolation or comparison can hold the thread making
    /// it. When a gate is armed, the next call of its kind reports that it
    /// began and waits to be let go. The stagger interpolates after its first
    /// generation check and before its store, and ThreadSafeSignal compares
    /// inside the store after the store's own check, so the two gates hold an
    /// update pass before its store or inside it.
    #[derive(Clone, Debug, Default)]
    struct Gated(f32);
    type Gate = (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>);
    static INTERPOLATE_GATE: Mutex<Option<Gate>> = Mutex::new(None);
    static COMPARE_GATE: Mutex<Option<Gate>> = Mutex::new(None);
    fn pass_gate(gate: &Mutex<Option<Gate>>) {
        let gate = gate.lock().unwrap().take();
        if let Some((began, go)) = gate {
            began.send(()).unwrap();
            go.recv_timeout(Duration::from_secs(30))
                .expect("the test never let the held call go");
        }
    }
    impl PartialEq for Gated {
        fn eq(&self, other: &Self) -> bool {
            pass_gate(&COMPARE_GATE);
            self.0 == other.0
        }
    }
    impl AnimatableValue for Gated {
        fn interpolate(&self, to: &Self, _: f32) -> Self {
            pass_gate(&INTERPOLATE_GATE);
            to.clone()
        }
        fn to_f32(&self) -> f32 {
            self.0
        }
        fn from_f32(value: f32) -> Self {
            Self(value)
        }
    }

    /// A stagger of one Gated item animating from 0 to 10, and an update pass
    /// on another thread held at `gate`.
    struct HeldPass {
        scope: Arc<crate::reactive::component_scope::ComponentScope>,
        _hooks: Hooks,
        stagger: StaggerHandle<Gated>,
        pass: thread::JoinHandle<()>,
        go: std::sync::mpsc::Sender<()>,
    }

    fn hold_pass_at(gate: &'static Mutex<Option<Gate>>) -> HeldPass {
        let scheduler = Arc::new(Scheduler::new());
        let scope = crate::reactive::component_scope::ComponentScope::new(scheduler.clone());
        let hooks = Hooks::new();
        let stagger = {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            use_stagger(&hooks, vec![Gated(0.0)], StaggerConfig::default())
        };
        stagger.animate_all_to(vec![Gated(10.0)]);
        scheduler.process_timers();
        assert!(
            stagger.owner.animation_ids.lock().unwrap()[0].is_some(),
            "the item's animation did not start"
        );
        let (began, began_receiver) = std::sync::mpsc::channel();
        let (go, go_receiver) = std::sync::mpsc::channel();
        *gate.lock().unwrap() = Some((began, go_receiver));
        let pass = thread::spawn(run_pass);
        began_receiver
            .recv_timeout(Duration::from_secs(30))
            .expect("the update pass never reached the gate");
        HeldPass {
            scope,
            _hooks: hooks,
            stagger,
            pass,
            go,
        }
    }

    /// An update pass past the stagger's first checks, but not yet storing,
    /// when its owner closes stores nothing after close() returns: the store
    /// checks again. The pass is held in the value's interpolation until
    /// close() has returned.
    #[test]
    #[serial_test::serial]
    fn stagger_pass_not_yet_storing_at_close_stores_nothing_after_it() {
        let held = hold_pass_at(&INTERPOLATE_GATE);
        held.scope.close();
        held.go.send(()).unwrap();
        held.pass.join().unwrap();
        assert_eq!(
            held.stagger.item(0).unwrap(),
            Gated(0.0),
            "an update pass set the item after close() returned"
        );
    }

    /// An update pass already inside the item's store when its owner closes
    /// finishes before close() returns. The pass is held in the store's
    /// comparison while another thread closes the owner. If close() returns
    /// while the pass is held, letting the pass go must leave the value as it
    /// was.
    #[test]
    #[serial_test::serial]
    fn stagger_pass_storing_at_close_finishes_before_close_returns() {
        let held = hold_pass_at(&COMPARE_GATE);
        let (closed, closed_receiver) = std::sync::mpsc::channel();
        let closer = {
            let scope = held.scope.clone();
            thread::spawn(move || {
                scope.close();
                closed.send(()).unwrap();
            })
        };
        // A close() that does not wait for the held store reports well
        // within this wait; one that waits lets it run out.
        let returned_while_held = closed_receiver
            .recv_timeout(Duration::from_millis(500))
            .is_ok();
        held.go.send(()).unwrap();
        held.pass.join().unwrap();
        closer.join().unwrap();
        let after = held.stagger.item(0).unwrap();
        assert!(
            !returned_while_held || after == Gated(0.0),
            "an update pass set the item to {after:?} after close() returned"
        );
        run_pass();
        assert_eq!(
            held.stagger.item(0).unwrap(),
            after,
            "the item changed after close()"
        );
    }

    /// The update closure RUNTIME holds for the animation `id`.
    fn task_update(id: usize) -> Arc<dyn Fn(f32) + Send + Sync> {
        RUNTIME
            .animations
            .read()
            .unwrap()
            .iter()
            .find(|task| task.id == id)
            .expect("the animation is not registered")
            .update
            .clone()
    }

    /// Whether dropping `hooks`, the last clone, while another thread calls
    /// `update` in a loop leaves that thread unable to finish.
    fn drop_during_updates_hangs(update: Arc<dyn Fn(f32) + Send + Sync>, hooks: Hooks) -> bool {
        let stop = Arc::new(AtomicBool::new(false));
        let calls = Arc::new(AtomicUsize::new(0));
        let (done, done_receiver) = std::sync::mpsc::channel();
        {
            let (stop, calls) = (stop.clone(), calls.clone());
            thread::spawn(move || {
                let mut progress = 0.0_f32;
                while !stop.load(Ordering::Acquire) {
                    progress = (progress + 0.001) % 0.99;
                    update(progress);
                    calls.fetch_add(1, Ordering::AcqRel);
                }
                done.send(()).unwrap();
            });
        }
        while calls.load(Ordering::Acquire) < 20 {
            thread::yield_now();
        }
        drop(hooks);
        stop.store(true, Ordering::Release);
        done_receiver.recv_timeout(Duration::from_secs(30)).is_err()
    }

    /// Dropping the last Hooks clone closes its owner on the dropping thread,
    /// never on an update pass: a pass checks liveness without holding the
    /// owner, so it cannot run the owner's cleanup while it holds a lock the
    /// cleanup takes. Each attempt calls an animation's update in a loop on
    /// another thread and drops the last Hooks clone meanwhile; a pass that
    /// ran the cleanup would never finish.
    #[test]
    #[serial_test::serial]
    fn dropping_the_last_hooks_during_update_passes_never_hangs() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = crate::reactive::component_scope::ComponentScope::new(scheduler.clone());
        for attempt in 0..200 {
            let hooks = Hooks::new();
            let (stagger, tween, spring, keyframes) = {
                let _scope = scope.enter(true);
                let _frame = hooks.begin_render();
                (
                    use_stagger(&hooks, vec![0.0_f32], StaggerConfig::default()),
                    use_animation(
                        &hooks,
                        0.0_f32,
                        AnimationConfig {
                            duration: Duration::from_secs(60),
                            ..AnimationConfig::default()
                        },
                    ),
                    use_spring(&hooks, 0.0_f32, SpringConfig::default()),
                    use_keyframes(
                        &hooks,
                        0.0_f32,
                        vec![
                            Keyframe::new(0.0).number("x", 0.0),
                            Keyframe::new(1.0).number("x", 10.0),
                        ],
                    ),
                )
            };
            stagger.animate_all_to(vec![1000.0]);
            scheduler.process_timers();
            tween.animate_to(1000.0);
            spring.set_target(1000.0);
            keyframes.play();
            let updates = [
                ("stagger", stagger.owner.animation_ids.lock().unwrap()[0]),
                ("tween", *tween.owner.current_animation_id.lock().unwrap()),
                ("spring", *spring.owner.animation_id.lock().unwrap()),
                ("keyframes", *keyframes.owner.animation_id.lock().unwrap()),
            ]
            .map(|(kind, id)| (kind, task_update(id.expect("the animation did not start"))));
            // Each kind in turn gets the final drop; the others' updates are
            // released first so that drop is the last one.
            let (kind, update) = updates[attempt % 4].clone();
            drop(updates);
            assert!(
                !drop_during_updates_hangs(update, hooks),
                "a {kind} update pass hung when the last Hooks clone dropped (attempt {attempt})"
            );
        }
    }

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
    #[serial_test::serial]
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
            // Another thread's pass would sample before the task is moved
            // to its midpoint, and cancel it before `find` sees it.
            without_other_passes(|| {
                handle.play();
                let id = handle.owner.animation_id.lock().unwrap().unwrap();
                let mut tasks = RUNTIME.animations.write().unwrap();
                let task = tasks.iter_mut().find(|task| task.id == id).unwrap();
                task.duration = Duration::from_secs(100);
                task.start_time = Instant::now() - Duration::from_secs(50);
            });
            run_pass();
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
    #[serial_test::serial]
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
        // No pass may deliver a frame to `first` between play and stop, or
        // to `second` before its duration is cut to zero.
        let second_id = without_other_passes(|| {
            first.play();
            second.play();
            first.stop();
            let second_id = second.owner.animation_id.lock().unwrap().unwrap();
            let mut tasks = RUNTIME.animations.write().unwrap();
            tasks
                .iter_mut()
                .find(|task| task.id == second_id)
                .unwrap()
                .duration = Duration::ZERO;
            second_id
        });
        run_pass();
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
    #[serial_test::serial]
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
        // No other thread's pass may deliver a frame between play and the
        // drop, or after it from a pass begun before it.
        without_other_passes(|| {
            handle.play();
            handle.seek(0.5);
            drop(hooks);
        });
        let retained = handle.value();
        assert!(handle.owner.animation_id.lock().unwrap().is_none());
        handle.play();
        // Choose an endpoint different from the retained value to detect a live seek.
        handle.seek(if retained == 10.0 { 0.0 } else { 1.0 });
        assert_eq!(handle.value(), retained);
        assert!(handle.owner.animation_id.lock().unwrap().is_none());
    }

    #[test]
    #[serial_test::serial]
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
    #[serial_test::serial]
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
        // No other thread's pass may deliver a frame between play and
        // cleanup, or after it from a pass begun before it.
        let id = without_other_passes(|| {
            handle.play();
            let id = handle.owner.animation_id.lock().unwrap().unwrap();
            hooks.cleanup();
            id
        });
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
    #[serial_test::serial]
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
    #[serial_test::serial]
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
                send.send(runtime.animations.read().unwrap().len()).unwrap();
            });
            let remaining = receive
                .recv_timeout(Duration::from_secs(2))
                .expect("callback capture drop could not reenter the runtime");
            assert_eq!(remaining, 0, "no animation survives with stop={stop}");
            worker.join().unwrap();
        }
    }

    #[test]
    #[serial_test::serial]
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
    #[serial_test::serial]
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
    #[serial_test::serial]
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
            send.send(runtime.animations.read().unwrap().len()).unwrap();
        });
        // A failing isolated test process exits rather than joining a deadlocked worker.
        let remaining = receive
            .recv_timeout(Duration::from_secs(2))
            .expect("animation callback could not reenter the runtime");
        assert_eq!(
            remaining, 1,
            "the outer animation stays registered after its callback added and cancelled another"
        );
        worker.join().unwrap();
    }

    #[test]
    #[serial_test::serial]
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
        run_pass();

        let value = handle.value();
        assert!(value > 0.0 && value <= 1.0);
    }

    #[test]
    #[serial_test::serial]
    fn test_spring_physics() {
        let hooks = Hooks::new();
        let spring = use_spring(&hooks, 0.0f32, SpringConfig::default());

        spring.set_target(10.0);
        thread::sleep(Duration::from_millis(100));

        // In tests, we need to manually update animations since there's no render loop
        run_pass();

        let value = spring.value();
        assert!(value > 0.0); // Should have moved toward target
    }

    #[test]
    #[serial_test::serial]
    fn test_transition_detects_changes() {
        let hooks = Hooks::new();
        let config = TransitionConfig::default();

        let value = use_transition(&hooks, 0.0f32, config);
        assert_eq!(value, 0.0);
    }

    #[test]
    #[serial_test::serial]
    fn test_stagger_delays() {
        let hooks = Hooks::new();
        let items = vec![0.0f32, 1.0, 2.0];
        let config = StaggerConfig::default();

        let stagger = use_stagger(&hooks, items, config);
        assert_eq!(stagger.items().len(), 3);
        assert!(stagger.delay(0).is_some());
    }
}
