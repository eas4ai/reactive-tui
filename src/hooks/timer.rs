use crate::reactive::hooks::{Hooks, ThreadSafeSignal, use_effect, use_signal};
use crate::reactive::scheduler::{Scheduler, TimerId};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// Hook for creating an interval timer that calls a callback repeatedly
///
/// # Example
/// ```rust
/// fn Counter(props: &Props, state: &mut State) -> Element {
///     let count = use_signal(&hooks, 0);
///     
///     use_interval(&hooks, Duration::from_secs(1), move || {
///         count.update(|c| *c += 1);
///     });
///     
///     Element::text(format!("Count: {}", count.get()))
/// }
/// ```
pub fn use_interval<F>(hooks: &Hooks, duration: Duration, callback: F) -> TimerHandle
where
    F: FnMut() + Send + Sync + 'static,
{
    let timer_id = use_signal(hooks, None::<TimerId>);
    let handle = TimerHandle {
        timer_id: timer_id.clone(),
    };

    // Wrap the callback to make it work with the effect system
    let callback = Arc::new(Mutex::new(callback));
    let callback_clone = callback.clone();

    use_effect(hooks, move || {
        // Schedule the interval
        if let Some(scheduler) = get_scheduler() {
            let id = scheduler.schedule_interval(duration, move || {
                let mut cb = callback_clone.lock().unwrap();
                cb();
            });
            timer_id.set(Some(id));
        }

        // Cleanup function to cancel the timer
        Some(Box::new(move || {
            if let Some(id) = timer_id.get() {
                if let Some(scheduler) = get_scheduler() {
                    scheduler.cancel_timer(id);
                }
            }
        }) as Box<dyn FnOnce() + Send + Sync>)
    });

    handle
}

/// Hook for creating a one-time timeout that calls a callback after a delay
///
/// # Example
/// ```rust
/// fn DelayedMessage(props: &Props, state: &mut State) -> Element {
///     let show_message = use_signal(&hooks, false);
///     
///     use_timeout(&hooks, Duration::from_secs(3), move || {
///         show_message.set(true);
///     });
///     
///     if show_message.get() {
///         Element::text("Time's up!")
///     } else {
///         Element::text("Waiting...")
///     }
/// }
/// ```
pub fn use_timeout<F>(hooks: &Hooks, duration: Duration, callback: F) -> TimerHandle
where
    F: FnOnce() + Send + Sync + 'static,
{
    let timer_id = use_signal(hooks, None::<TimerId>);
    let handle = TimerHandle {
        timer_id: timer_id.clone(),
    };

    // Wrap FnOnce in an Option so we can take it
    let callback = Arc::new(Mutex::new(Some(callback)));
    let callback_clone = callback.clone();

    use_effect(hooks, move || {
        // Schedule the timeout
        if let Some(scheduler) = get_scheduler() {
            let id = scheduler.schedule_timeout(duration, move || {
                let mut cb_opt = callback_clone.lock().unwrap();
                if let Some(cb) = cb_opt.take() {
                    cb();
                }
            });
            timer_id.set(Some(id));
        }

        // Cleanup function to cancel the timer if component unmounts before it fires
        Some(Box::new(move || {
            if let Some(id) = timer_id.get() {
                if let Some(scheduler) = get_scheduler() {
                    scheduler.cancel_timer(id);
                }
            }
        }) as Box<dyn FnOnce() + Send + Sync>)
    });

    handle
}

/// Hook for creating a debounced callback that only fires after a delay of inactivity
///
/// # Example
/// ```rust
/// fn SearchBox(props: &Props, state: &mut State) -> Element {
///     let search_term = use_signal(&hooks, String::new());
///     let search_results = use_signal(&hooks, Vec::<String>::new());
///     
///     let debounced_search = use_debounce(&hooks, Duration::from_millis(300), move |term: String| {
///         // Perform search with the term
///         let results = perform_search(&term);
///         search_results.set(results);
///     });
///     
///     Element::input()
///         .on_change(move |e| {
///             let new_term = e.value.clone();
///             search_term.set(new_term.clone());
///             debounced_search.call(new_term);
///         })
/// }
/// ```
pub fn use_debounce<T, F>(hooks: &Hooks, delay: Duration, callback: F) -> DebouncedFunction<T>
where
    T: Send + 'static,
    F: Fn(T) + Send + 'static,
{
    let timer_id = use_signal(hooks, None::<TimerId>);
    let callback = Arc::new(Mutex::new(callback));

    DebouncedFunction {
        timer_id,
        delay,
        callback,
    }
}

/// Hook for creating a throttled callback that fires at most once per interval
///
/// # Example
/// ```rust
/// fn ScrollTracker(props: &Props, state: &mut State) -> Element {
///     let scroll_position = use_signal(&hooks, 0);
///     
///     let throttled_save = use_throttle(&hooks, Duration::from_millis(1000), move |pos: i32| {
///         // Save scroll position to backend
///         save_scroll_position(pos);
///     });
///     
///     Element::scrollable()
///         .on_scroll(move |e| {
///             let pos = e.scroll_top;
///             scroll_position.set(pos);
///             throttled_save.call(pos);
///         })
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
        last_call,
        interval,
        callback,
    }
}

/// Handle for controlling a timer
#[derive(Clone)]
pub struct TimerHandle {
    timer_id: ThreadSafeSignal<Option<TimerId>>,
}

impl TimerHandle {
    /// Cancel the timer
    pub fn cancel(&self) {
        if let Some(id) = self.timer_id.get() {
            if let Some(scheduler) = get_scheduler() {
                scheduler.cancel_timer(id);
            }
            self.timer_id.set(None);
        }
    }

    /// Check if the timer is active
    pub fn is_active(&self) -> bool {
        self.timer_id.get().is_some()
    }
}

/// A debounced function that delays execution until after a period of inactivity
pub struct DebouncedFunction<T> {
    timer_id: ThreadSafeSignal<Option<TimerId>>,
    delay: Duration,
    callback: Arc<Mutex<dyn Fn(T) + Send>>,
}

impl<T: Send + 'static> DebouncedFunction<T> {
    /// Call the debounced function
    pub fn call(&self, value: T) {
        // Cancel any existing timer
        if let Some(id) = self.timer_id.get() {
            if let Some(scheduler) = get_scheduler() {
                scheduler.cancel_timer(id);
            }
        }

        // Schedule a new timer
        if let Some(scheduler) = get_scheduler() {
            let callback = self.callback.clone();
            let timer_id = self.timer_id.clone();

            let id = scheduler.schedule_timeout(self.delay, move || {
                let cb = callback.lock().unwrap();
                cb(value);
                timer_id.set(None);
            });

            self.timer_id.set(Some(id));
        }
    }

    /// Cancel any pending execution
    pub fn cancel(&self) {
        if let Some(id) = self.timer_id.get() {
            if let Some(scheduler) = get_scheduler() {
                scheduler.cancel_timer(id);
            }
            self.timer_id.set(None);
        }
    }
}

/// A throttled function that limits execution to once per interval
pub struct ThrottledFunction<T> {
    last_call: ThreadSafeSignal<Option<std::time::Instant>>,
    interval: Duration,
    callback: Arc<Mutex<dyn Fn(T) + Send>>,
}

impl<T: Send + 'static> ThrottledFunction<T> {
    /// Call the throttled function
    pub fn call(&self, value: T) {
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

/// Get the global scheduler instance
fn get_scheduler() -> Option<Arc<Scheduler>> {
    // Prefer the runtime scheduler when available
    if let Some(s) = crate::reactive::runtime::with_runtime(|ctx| ctx.scheduler().clone()) {
        return Some(s);
    }
    // Fallback: a lightweight global scheduler with a background tick thread for timers
    Some(get_fallback_scheduler())
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
