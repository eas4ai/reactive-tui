use super::effect::EffectId;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Thread-safe hook context for managing component state and effects
#[derive(Clone)]
pub struct Hooks {
    state: Arc<Mutex<HookState>>,

    /// Active effects for this component
    effects: Arc<Mutex<Vec<EffectId>>>,

    /// Context values by type
    contexts: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HookKind {
    Signal,
    Memo,
    Reducer,
    Previous,
}

struct HookSlot {
    kind: HookKind,
    value: Box<dyn Any + Send + Sync>,
}

#[derive(Default)]
struct HookState {
    index: usize,
    slots: Vec<HookSlot>,
    expected_count: Option<usize>,
    rendering: bool,
}

/// Render boundary used by generated components.
#[doc(hidden)]
pub struct HookRender {
    state: Arc<Mutex<HookState>>,
}

impl Drop for HookRender {
    fn drop(&mut self) {
        let mut state = self.state.lock().expect("hook state lock poisoned");
        state.rendering = false;
        if std::thread::panicking() {
            return;
        }
        let count = state.index;
        let expected = state.expected_count.get_or_insert(count);
        let mismatch = *expected != count;
        drop(state);
        assert!(!mismatch, "hook count changed between renders");
    }
}

impl Hooks {
    /// Create a new hooks context
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HookState::default())),
            effects: Arc::new(Mutex::new(Vec::new())),
            contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Begin a generated render, retaining positional state from prior renders.
    /// Panics on overlapping renders or a changed hook count, kind, or type.
    #[doc(hidden)]
    pub fn begin_render(&self) -> HookRender {
        let mut state = self.state.lock().expect("hook state lock poisoned");
        if state.rendering {
            drop(state);
            panic!("overlapping renders on the same hook context");
        }
        state.index = 0;
        state.rendering = true;
        HookRender {
            state: Arc::clone(&self.state),
        }
    }

    /// Reset positional indexing for a manually managed render.
    /// Calls must keep the same order and types. Generated components also
    /// validate the completed render's hook count automatically.
    pub fn reset(&self) {
        let mut state = self.state.lock().expect("hook state lock poisoned");
        if state.rendering {
            drop(state);
            panic!("cannot reset hooks during a generated render");
        }
        state.index = 0;
    }

    fn get_or_create_storage<T: Send + Sync + 'static>(
        &self,
        kind: HookKind,
        initial: T,
    ) -> Arc<Mutex<T>> {
        let mut state = self.state.lock().expect("hook state lock poisoned");
        let index = state.index;
        let value = if let Some(slot) = state.slots.get(index) {
            let existing = slot.value.downcast_ref::<Arc<Mutex<T>>>();
            if slot.kind != kind || existing.is_none() {
                drop(state);
                panic!("hook kind or type changed at slot {index}");
            }
            Arc::clone(existing.expect("hook type checked above"))
        } else {
            if state.rendering && state.expected_count.is_some() {
                drop(state);
                panic!("hook count increased at slot {index}");
            }
            let value = Arc::new(Mutex::new(initial));
            state.slots.push(HookSlot {
                kind,
                value: Box::new(Arc::clone(&value)),
            });
            value
        };
        state.index += 1;
        drop(state);
        value
    }

    /// Cleanup all effects
    pub fn cleanup(&self) {
        // Effects will be cleaned up by the runtime
        if let Ok(mut effects) = self.effects.lock() {
            effects.clear();
        } else {
            log::warn!("Effects lock poisoned during cleanup");
        }
    }
}

impl Default for Hooks {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe signal wrapper
pub struct ThreadSafeSignal<T> {
    inner: Arc<Mutex<T>>,
    version: Arc<Mutex<usize>>,
    app_subscribers: Arc<super::wake::Subscriptions>,
}

impl<T: Clone + Default> ThreadSafeSignal<T> {
    /// Create a new thread-safe signal with initial value
    pub fn new(initial: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(initial)),
            version: Arc::new(Mutex::new(0)),
            app_subscribers: Arc::new(super::wake::Subscriptions::default()),
        }
    }

    /// Get the current value of the signal
    pub fn get(&self) -> T {
        self.inner
            .lock()
            .map(|guard| {
                self.app_subscribers.track();
                guard.clone()
            })
            .unwrap_or_else(|_| {
                log::warn!("Signal lock poisoned, returning default value");
                T::default()
            })
    }

    /// Set a new value for the signal
    pub fn set(&self, value: T)
    where
        T: PartialEq,
    {
        let changed =
            if let (Ok(mut inner), Ok(mut version)) = (self.inner.lock(), self.version.lock()) {
                if *inner != value {
                    *inner = value;
                    *version += 1;
                    true
                } else {
                    false
                }
            } else {
                log::warn!("Signal locks poisoned during set operation");
                false
            };
        if changed {
            self.app_subscribers.notify();
        }
    }

    /// Update the signal value using a function
    pub fn update(&self, f: impl FnOnce(&mut T))
    where
        T: PartialEq,
    {
        let changed =
            if let (Ok(mut inner), Ok(mut version)) = (self.inner.lock(), self.version.lock()) {
                let old = inner.clone();
                f(&mut *inner);
                if *inner != old {
                    *version += 1;
                    true
                } else {
                    false
                }
            } else {
                log::warn!("Signal locks poisoned during update operation");
                false
            };
        if changed {
            self.app_subscribers.notify();
        }
    }
}

impl<T> Clone for ThreadSafeSignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            version: Arc::clone(&self.version),
            app_subscribers: Arc::clone(&self.app_subscribers),
        }
    }
}

/// Create a reactive signal (thread-safe version)
pub fn use_signal<T: Send + Sync + Clone + Default + 'static>(
    hooks: &Hooks,
    initial: T,
) -> ThreadSafeSignal<T> {
    use_signal_kind(hooks, initial, HookKind::Signal)
}

fn use_signal_kind<T: Send + Sync + Clone + Default + 'static>(
    hooks: &Hooks,
    initial: T,
    kind: HookKind,
) -> ThreadSafeSignal<T> {
    let signal = hooks.get_or_create_storage(kind, ThreadSafeSignal::new(initial));
    let result = signal.lock().expect("hook signal lock poisoned").clone();
    result
}

/// Run a side effect (thread-safe version)
pub fn use_effect<F>(hooks: &Hooks, effect: F) -> EffectId
where
    F: FnOnce() -> Option<Box<dyn FnOnce() + Send + Sync>> + Send + Sync + 'static,
{
    // Prefer scheduling via the global reactive runtime if present
    // Move effect into an Option so we can take it exactly once
    let mut effect_opt = Some(effect);
    if let Some(id) = crate::reactive::runtime::with_runtime(|ctx| {
        // Take ownership of the effect for the scheduled closure
        let effect_inner = effect_opt.take().expect("effect already taken");
        ctx.runtime()
            .register_effect(super::effect::Effect::new(move || {
                // Adapt Send+Sync cleanup to non-threaded cleanup expected by Effect
                let cleanup = effect_inner();
                cleanup.map(|boxed| Box::new(boxed) as Box<dyn FnOnce()>)
            }))
    }) {
        // Use try_lock to avoid deadlock
        if let Ok(mut effects) = hooks.effects.try_lock() {
            effects.push(id);
        } else {
            log::warn!("Could not acquire effects lock, effect may not be tracked properly");
        }
        return id;
    }

    // Fallback: no runtime set. Execute effect now (best-effort) and return a fresh id.
    let effect_id = EffectId::new();
    if let Ok(mut effects) = hooks.effects.try_lock() {
        effects.push(effect_id);
    } else {
        log::warn!("Could not acquire effects lock in fallback path");
    }
    if let Some(cleanup) = effect_opt.and_then(|f| f()) {
        // Without a scheduler to own deferred cleanup, execute immediately to avoid leaks
        cleanup();
    }
    effect_id
}

/// Access context value (thread-safe version)
pub fn use_context<T: Clone + Send + Sync + 'static>(hooks: &Hooks) -> Option<T> {
    hooks
        .contexts
        .lock()
        .map_err(|_| log::warn!("Context lock poisoned in use_context"))
        .ok()?
        .get(&TypeId::of::<T>())
        .and_then(|any| any.downcast_ref::<T>())
        .cloned()
}

/// Provide context value to children (thread-safe version)
pub fn provide_context<T: Send + Sync + 'static>(hooks: &Hooks, value: T) {
    if let Ok(mut contexts) = hooks.contexts.lock() {
        contexts.insert(TypeId::of::<T>(), Box::new(value));
    } else {
        log::warn!("Context lock poisoned in provide_context");
    }
}

/// Hook for managing state with a reducer (thread-safe version)
pub fn use_reducer<S, A, F>(
    hooks: &Hooks,
    reducer: F,
    initial: S,
) -> (ThreadSafeSignal<S>, Arc<dyn Fn(A) + Send + Sync>)
where
    S: Clone + PartialEq + Send + Sync + Default + 'static,
    A: Send + Sync + 'static,
    F: Fn(&S, A) -> S + Send + Sync + 'static,
{
    let state = use_signal_kind(hooks, initial, HookKind::Reducer);
    let state_clone = state.clone();
    let reducer = Arc::new(reducer);
    let reducer_clone = Arc::clone(&reducer);

    let dispatch = Arc::new(move |action: A| {
        state_clone.update(|s| {
            *s = reducer_clone(s, action);
        });
    });

    (state, dispatch as Arc<dyn Fn(A) + Send + Sync>)
}

/// Hook for managing previous value (thread-safe version)
pub fn use_previous<T: Clone + Send + Sync + 'static>(hooks: &Hooks, value: T) -> Option<T> {
    let prev: Arc<Mutex<Option<T>>> = hooks.get_or_create_storage(HookKind::Previous, None::<T>);
    let result = {
        match prev.lock() {
            Ok(mut prev_guard) => {
                let old = prev_guard.clone();
                *prev_guard = Some(value);
                old
            }
            Err(_) => {
                log::warn!("Previous value lock poisoned in use_previous");
                None
            }
        }
    };
    result
}

/// Recompute once per call (normally once per render), retaining the output signal.
/// There is no dependency list; only changed values notify subscribers.
pub fn use_memo<T>(
    hooks: &Hooks,
    compute: impl Fn() -> T + Send + Sync + 'static,
) -> ThreadSafeSignal<T>
where
    T: Clone + PartialEq + Send + Sync + Default + 'static,
{
    let value = compute();
    let signal = use_signal_kind(hooks, value.clone(), HookKind::Memo);
    signal.set(value);
    signal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_frames_keep_a_bounded_slot_count() {
        let hooks = Hooks::new();
        for n in 0..1000 {
            let _frame = hooks.begin_render();
            let state = use_signal(&hooks, 0usize);
            assert_eq!(state.get(), n);
            state.set(n + 1);
            assert_eq!(use_memo(&hooks, move || n).get(), n);
            assert_eq!(use_previous(&hooks, n), n.checked_sub(1));
        }
        assert_eq!(hooks.state.lock().unwrap().slots.len(), 3);
    }

    #[test]
    fn overlapping_frames_fail_without_poisoning_state() {
        let hooks = Hooks::new();
        let frame = hooks.begin_render();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            hooks.begin_render()
        }))
        .is_err());
        drop(frame);
        let _next = hooks.begin_render();
    }

    #[test]
    fn test_thread_safe_signal() {
        let hooks = Hooks::new();

        let count = use_signal(&hooks, 0);
        assert_eq!(count.get(), 0);

        count.set(5);
        assert_eq!(count.get(), 5);

        // Test that it can be sent across threads
        let count_clone = count.clone();
        std::thread::spawn(move || {
            count_clone.set(10);
        })
        .join()
        .expect("Thread should not panic");

        assert_eq!(count.get(), 10);
    }

    #[test]
    fn test_thread_safe_context() {
        let hooks = Hooks::new();

        provide_context(&hooks, "Hello".to_string());

        let ctx: Option<String> = use_context(&hooks);
        assert_eq!(ctx, Some("Hello".to_string()));

        // Test thread safety
        let hooks_clone = hooks.clone();
        std::thread::spawn(move || {
            let ctx: Option<String> = use_context(&hooks_clone);
            assert_eq!(ctx, Some("Hello".to_string()));
        })
        .join()
        .expect("Thread should not panic");
    }
}
