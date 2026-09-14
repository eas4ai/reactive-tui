use crate::reactive::hooks::{
    use_effect_with_deps, use_signal, HookKind, HookResources, Hooks, ThreadSafeSignal,
};
use crate::reactive::scheduler::{Scheduler, TimerId};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Duration;

/// Create an interval owned by this hook context.
/// Unchanged durations preserve the deadline while callbacks refresh on render.
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use std::time::Duration;
///
/// #[component]
/// fn Counter(hooks: &Hooks) -> Element {
///     let count = use_signal(hooks, 0);
///     let tick_count = count.clone();
///     use_interval(hooks, Duration::from_secs(1), move || {
///         tick_count.update(|count| *count += 1);
///     });
///     Element::text(format!("Count: {}", count.get()))
/// }
/// ```
pub fn use_interval<F>(hooks: &Hooks, duration: Duration, callback: F) -> TimerHandle
where
    F: FnMut() + Send + Sync + 'static,
{
    let timer = use_timer(hooks, HookKind::Interval);
    timer.replace_callback(Box::new(callback));
    install_timer(hooks, timer.clone(), duration, true);
    TimerHandle { timer }
}

/// Create a timeout owned by this hook context.
/// It fires once; changing duration restarts it, while redraw alone does not.
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use std::time::Duration;
///
/// #[component]
/// fn DelayedMessage(hooks: &Hooks) -> Element {
///     let visible = use_signal(hooks, false);
///     let delayed = visible.clone();
///     use_timeout(hooks, Duration::from_secs(3), move || delayed.set(true));
///     Element::text(if visible.get() { "Time's up!" } else { "Waiting..." })
/// }
/// ```
pub fn use_timeout<F>(hooks: &Hooks, duration: Duration, callback: F) -> TimerHandle
where
    F: FnOnce() + Send + Sync + 'static,
{
    let timer = use_timer(hooks, HookKind::Timeout);
    let mut callback = Some(callback);
    timer.replace_callback(Box::new(move || {
        if let Some(callback) = callback.take() {
            callback();
        }
    }));
    install_timer(hooks, timer.clone(), duration, false);
    TimerHandle { timer }
}

/// Hook for creating a debounced callback that only fires after a delay of inactivity
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::hooks::use_debounce;
/// use reactive_tui::reactive::Hooks;
/// use std::time::Duration;
///
/// fn queue_search(hooks: &Hooks, query: String) {
///     let search = use_debounce(hooks, Duration::from_millis(300), |term: String| {
///         println!("Search for {term}");
///     });
///     search.call(query);
/// }
/// ```
pub fn use_debounce<T, F>(hooks: &Hooks, delay: Duration, callback: F) -> DebouncedFunction<T>
where
    T: Send + 'static,
    F: Fn(T) + Send + 'static,
{
    let timer = use_timer(hooks, HookKind::Debounce);
    let owned = timer.clone();
    use_effect_with_deps(hooks, (), move || Some(Box::new(move || owned.close())));
    DebouncedFunction {
        timer,
        delay,
        callback: Arc::new(Mutex::new(callback)),
    }
}

/// Hook for creating a throttled callback that fires at most once per interval
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::hooks::use_throttle;
/// use reactive_tui::reactive::Hooks;
/// use std::time::Duration;
///
/// fn report_scroll(hooks: &Hooks, position: i32) {
///     let report = use_throttle(hooks, Duration::from_secs(1), |position: i32| {
///         println!("Scroll position: {position}");
///     });
///     report.call(position);
/// }
/// ```
pub fn use_throttle<T, F>(hooks: &Hooks, interval: Duration, callback: F) -> ThrottledFunction<T>
where
    T: Send + 'static,
    F: Fn(T) + Send + 'static,
{
    let last_call = use_signal(hooks, None::<std::time::Instant>);
    let callback = Arc::new(Mutex::new(callback));

    ThrottledFunction {
        owner: hooks.resource_token(),
        last_call,
        interval,
        callback,
    }
}

/// Handle for controlling a timer. It retains the scheduler that created it.
#[derive(Clone)]
pub struct TimerHandle {
    timer: Arc<HookTimer>,
}

impl TimerHandle {
    /// Cancel the timer.
    pub fn cancel(&self) {
        self.timer.cancel();
    }

    /// Whether a callback remains scheduled; false after a timeout fires or unmount.
    pub fn is_active(&self) -> bool {
        self.timer.active.get()
    }
}

/// A debounced function that delays execution until after a period of inactivity.
pub struct DebouncedFunction<T> {
    timer: Arc<HookTimer>,
    delay: Duration,
    callback: Arc<Mutex<dyn Fn(T) + Send>>,
}

impl<T: Send + 'static> DebouncedFunction<T> {
    /// Replace pending execution. Calls after owner cleanup are ignored.
    pub fn call(&self, value: T) {
        let callback = self.callback.clone();
        let mut value = Some(value);
        self.timer.restart(
            self.delay,
            false,
            Some(Box::new(move || {
                if let Some(value) = value.take() {
                    callback.lock().unwrap()(value);
                }
            })),
        );
    }

    /// Cancel any pending execution.
    pub fn cancel(&self) {
        self.timer.cancel();
    }
}

/// A throttled function that limits execution to once per interval
pub struct ThrottledFunction<T> {
    owner: Weak<HookResources>,
    last_call: ThreadSafeSignal<Option<std::time::Instant>>,
    interval: Duration,
    callback: Arc<Mutex<dyn Fn(T) + Send>>,
}

impl<T: Send + 'static> ThrottledFunction<T> {
    /// Call the throttled function
    pub fn call(&self, value: T) {
        if !self.owner.upgrade().is_some_and(|owner| owner.is_alive()) {
            return;
        }
        let now = std::time::Instant::now();
        let should_call = match self.last_call.get() {
            None => true,
            Some(last) => now.duration_since(last) >= self.interval,
        };

        if should_call {
            self.last_call.set(Some(now));
            let cb = self.callback.lock().unwrap();
            cb(value);
        }
    }

    /// Force the next call to execute regardless of throttling
    pub fn reset(&self) {
        self.last_call.set(None);
    }
}

type Callback = Box<dyn FnMut() + Send>;

#[derive(Default)]
struct TimerState {
    id: Option<TimerId>,
    generation: u64,
    closed: bool,
}

#[derive(Default)]
struct CallbackState {
    callback: Option<Callback>,
    generation: u64,
}

struct HookTimer {
    scheduler: Arc<Scheduler>,
    owner: Weak<HookResources>,
    state: Mutex<TimerState>,
    callback: Mutex<CallbackState>,
    active: ThreadSafeSignal<bool>,
}

impl HookTimer {
    fn new(scheduler: Arc<Scheduler>, owner: Weak<HookResources>) -> Self {
        Self {
            scheduler,
            owner,
            state: Mutex::new(TimerState::default()),
            callback: Mutex::new(CallbackState::default()),
            active: ThreadSafeSignal::new(false),
        }
    }

    fn replace_callback(&self, callback: Callback) {
        drop(self.swap_callback(callback));
    }

    fn swap_callback(&self, callback: Callback) -> Option<Callback> {
        let mut state = self.callback.lock().unwrap();
        state.generation = state.generation.wrapping_add(1);
        let old = state.callback.replace(callback);
        drop(state);
        old
    }

    fn restart(self: &Arc<Self>, duration: Duration, repeat: bool, callback: Option<Callback>) {
        let mut state = self.state.lock().unwrap();
        if state.closed || !self.owner.upgrade().is_some_and(|owner| owner.is_alive()) {
            return;
        }
        if let Some(id) = state.id.take() {
            self.scheduler.cancel_timer(id);
        }
        let old_callback = callback.and_then(|callback| self.swap_callback(callback));
        state.generation = state.generation.wrapping_add(1);
        let generation = state.generation;
        let timer = Arc::downgrade(self);
        let fire = move || {
            if let Some(timer) = timer.upgrade() {
                timer.fire(generation, repeat);
            }
        };
        state.id = Some(if repeat {
            self.scheduler.schedule_interval(duration, fire)
        } else {
            self.scheduler.schedule_timeout(duration, fire)
        });
        self.active.set(true);
        drop(state);
        drop(old_callback);
    }

    fn fire(&self, generation: u64, repeat: bool) {
        let mut state = self.state.lock().unwrap();
        if state.closed
            || state.generation != generation
            || !self.owner.upgrade().is_some_and(|owner| owner.is_alive())
        {
            return;
        }
        if !repeat {
            state.id = None;
            self.active.set(false);
        }
        drop(state);
        let mut callbacks = self.callback.lock().unwrap();
        let Some(mut callback) = callbacks.callback.take() else {
            return;
        };
        let generation = callbacks.generation;
        drop(callbacks);
        callback();
        let mut callbacks = self.callback.lock().unwrap();
        if repeat && callbacks.generation == generation {
            callbacks.callback = Some(callback);
        }
    }

    fn cancel(&self) {
        let mut state = self.state.lock().unwrap();
        state.generation = state.generation.wrapping_add(1);
        let id = state.id.take();
        self.active.set(false);
        drop(state);
        if let Some(id) = id {
            self.scheduler.cancel_timer(id);
        }
    }

    fn close(&self) {
        self.state.lock().unwrap().closed = true;
        self.cancel();
        let mut callbacks = self.callback.lock().unwrap();
        callbacks.generation = callbacks.generation.wrapping_add(1);
        let callback = callbacks.callback.take();
        drop(callbacks);
        drop(callback);
    }
}

impl Drop for HookTimer {
    fn drop(&mut self) {
        self.close();
    }
}

fn use_timer(hooks: &Hooks, kind: HookKind) -> Arc<HookTimer> {
    let storage = hooks.get_or_create_storage(kind, || {
        Arc::new(HookTimer::new(get_scheduler(hooks), hooks.resource_token()))
    });
    let timer = storage.lock().unwrap().clone();
    timer
}

fn install_timer(hooks: &Hooks, timer: Arc<HookTimer>, duration: Duration, repeat: bool) {
    use_effect_with_deps(hooks, duration, move || {
        timer.restart(duration, repeat, None);
        Some(Box::new(move || {
            if timer.owner.upgrade().is_some_and(|owner| owner.is_alive()) {
                timer.cancel();
            } else {
                timer.close();
            }
        }))
    });
}

/// Get the global scheduler instance
pub(crate) fn get_scheduler(hooks: &Hooks) -> Arc<Scheduler> {
    if let Some(scheduler) = hooks.resource_scheduler() {
        return scheduler;
    }
    // Prefer the runtime scheduler when available
    if let Some(s) = crate::reactive::runtime::with_runtime(|ctx| ctx.scheduler().clone()) {
        return s;
    }
    // Fallback: a lightweight global scheduler with a background tick thread for timers
    get_fallback_scheduler()
}

fn get_fallback_scheduler() -> Arc<Scheduler> {
    static SCHED: OnceLock<Arc<Scheduler>> = OnceLock::new();
    static START: OnceLock<()> = OnceLock::new();
    let sched = SCHED.get_or_init(|| Arc::new(Scheduler::new())).clone();
    START.get_or_init(|| {
        let s2 = sched.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(1));
            s2.process_timers();
        });
    });
    sched
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn owned_interval_preserves_deadlines_and_refreshes_callbacks() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = crate::reactive::component_scope::ComponentScope::new(scheduler.clone());
        let hooks = Hooks::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let handle = {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            let calls = calls.clone();
            use_interval(&hooks, Duration::from_secs(3600), move || {
                calls.fetch_add(1, Ordering::SeqCst);
            })
        };
        let first = scheduler.next_deadline().unwrap();
        {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            let calls = calls.clone();
            use_interval(&hooks, Duration::from_secs(3600), move || {
                calls.fetch_add(10, Ordering::SeqCst);
            });
        }
        assert_eq!(scheduler.next_deadline(), Some(first));
        {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            let calls = calls.clone();
            use_interval(&hooks, Duration::ZERO, move || {
                calls.fetch_add(100, Ordering::SeqCst);
            });
        }
        assert!(scheduler.next_deadline().unwrap() < first);
        scheduler.process_timers();
        assert_eq!(calls.load(Ordering::SeqCst), 100);
        {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            let calls = calls.clone();
            use_interval(&hooks, Duration::ZERO, move || {
                calls.fetch_add(1000, Ordering::SeqCst);
            });
        }
        scheduler.process_timers();
        assert_eq!(calls.load(Ordering::SeqCst), 1100);
        scope.close();
        assert!(!handle.is_active());
        assert!(scheduler.next_deadline().is_none());
        scheduler.process_timers();
        assert_eq!(calls.load(Ordering::SeqCst), 1100);
    }

    #[test]
    fn timeout_completes_once_and_only_changed_duration_restarts_it() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = crate::reactive::component_scope::ComponentScope::new(scheduler.clone());
        let hooks = Hooks::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let render = |duration| {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            let calls = calls.clone();
            use_timeout(&hooks, duration, move || {
                calls.fetch_add(1, Ordering::SeqCst);
            })
        };
        let handle = render(Duration::ZERO);
        assert!(handle.is_active());
        scheduler.process_timers();
        assert!(!handle.is_active());
        render(Duration::ZERO);
        assert!(scheduler.next_deadline().is_none());
        scheduler.process_timers();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        render(Duration::from_secs(3600));
        assert!(handle.is_active());
        // Cancel after leaving the ambient scope; it still targets the original scheduler.
        handle.cancel();
        assert!(scheduler.next_deadline().is_none());
        assert!(!handle.is_active());
    }

    #[test]
    fn retained_debounce_cannot_schedule_after_owner_removal() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = crate::reactive::component_scope::ComponentScope::new(scheduler.clone());
        let hooks = Hooks::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let debounce = {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            let calls = calls.clone();
            use_debounce(&hooks, Duration::ZERO, move |value| {
                calls.fetch_add(value, Ordering::SeqCst);
            })
        };
        debounce.call(1);
        debounce.call(2);
        scheduler.process_timers();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        debounce.call(4);
        scope.close();
        assert!(scheduler.next_deadline().is_none());
        debounce.call(8);
        assert!(scheduler.next_deadline().is_none());
        scheduler.process_timers();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn interval_callback_can_cancel_its_own_handle() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = crate::reactive::component_scope::ComponentScope::new(scheduler.clone());
        let hooks = Hooks::new();
        let shared: Arc<Mutex<Option<TimerHandle>>> = Arc::new(Mutex::new(None));
        let handle = {
            let _scope = scope.enter(true);
            let _frame = hooks.begin_render();
            let shared = shared.clone();
            use_interval(&hooks, Duration::ZERO, move || {
                shared.lock().unwrap().as_ref().unwrap().cancel();
            })
        };
        *shared.lock().unwrap() = Some(handle.clone());
        scheduler.process_timers();
        assert!(!handle.is_active());
        assert!(scheduler.next_deadline().is_none());
    }

    #[test]
    fn dropping_standalone_hooks_releases_callback_even_with_a_retained_handle() {
        let hooks = Hooks::new();
        let lifetime = Arc::new(());
        let weak = Arc::downgrade(&lifetime);
        let handle = use_interval(&hooks, Duration::from_secs(3600), move || {
            let _ = &lifetime;
        });
        assert!(weak.upgrade().is_some());
        drop(hooks);
        assert!(!handle.is_active());
        assert!(
            weak.upgrade().is_none(),
            "unmount must release callback captures"
        );
    }

    #[test]
    fn test_timer_handle() {
        let hooks = Hooks::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        let handle = use_timeout(&hooks, Duration::from_millis(10), move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        assert!(handle.is_active());
        handle.cancel();
        assert!(!handle.is_active());
    }

    #[test]
    fn test_debounced_function() {
        let hooks = Hooks::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        let debounced = use_debounce(&hooks, Duration::from_millis(10), move |_: i32| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        // Multiple rapid calls should only result in one execution
        debounced.call(1);
        debounced.call(2);
        debounced.call(3);

        // Cancel before it executes
        debounced.cancel();

        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_throttled_function() {
        let hooks = Hooks::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        let throttled = use_throttle(&hooks, Duration::from_millis(50), move |_: i32| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        // First call should execute immediately
        throttled.call(1);
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        // Rapid calls within interval should be ignored
        throttled.call(2);
        throttled.call(3);
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        // After interval, next call should execute
        std::thread::sleep(Duration::from_millis(60));
        throttled.call(4);
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }
}
