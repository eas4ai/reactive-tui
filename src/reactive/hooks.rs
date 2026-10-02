use super::effect::EffectId;
pub(crate) use super::hook_resources::{HookResources, Liveness, OwnedEffect};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

/// Thread-safe hook context for managing component state and effects
#[derive(Clone)]
pub struct Hooks {
    state: Arc<Mutex<HookState>>,

    resources: Arc<HookResources>,

    /// Context values by type
    contexts: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HookKind {
    Signal,
    Memo,
    Reducer,
    Previous,
    Effect,
    EffectDeps,
    Interval,
    Timeout,
    Debounce,
    Ref,
    CallbackRef,
    MultiRef,
    LocalRef,
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
    local_scope: Option<Weak<()>>,
}

/// Render boundary used by generated components.
#[doc(hidden)]
pub struct HookRender {
    state: Arc<Mutex<HookState>>,
    resources: Arc<HookResources>,
}

impl Drop for HookRender {
    fn drop(&mut self) {
        let mut state = self.state.lock().expect("hook state lock poisoned");
        state.rendering = false;
        if std::thread::panicking() {
            drop(state);
            self.resources.discard_pending();
            return;
        }
        let count = state.index;
        let expected = state.expected_count.get_or_insert(count);
        let mismatch = *expected != count;
        drop(state);
        if mismatch {
            self.resources.discard_pending();
            panic!("hook count changed between renders");
        }
        self.resources.flush();
    }
}

impl Hooks {
    /// Create a new hooks context
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HookState::default())),
            resources: Arc::new(HookResources::default()),
            contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Begin a generated render, retaining positional state from prior renders.
    /// Panics on overlapping renders or a changed hook count, kind, or type.
    #[doc(hidden)]
    pub fn begin_render(&self) -> HookRender {
        self.resources.bind();
        let mut state = self.state.lock().expect("hook state lock poisoned");
        if state.rendering {
            drop(state);
            panic!("overlapping renders on the same hook context");
        }
        state.index = 0;
        state.rendering = true;
        HookRender {
            state: Arc::clone(&self.state),
            resources: Arc::clone(&self.resources),
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

    pub(crate) fn bind_local_scope(&self, scope: &Arc<()>) {
        let mut state = self.state.lock().expect("hook state lock poisoned");
        if let Some(bound) = &state.local_scope {
            let matches = bound
                .upgrade()
                .is_some_and(|bound| Arc::ptr_eq(&bound, scope));
            drop(state);
            assert!(matches, "local hook owner used on another thread, in another scope, or after its scope ended");
        } else {
            state.local_scope = Some(Arc::downgrade(scope));
        }
    }

    pub(crate) fn get_or_create_storage<T: Send + Sync + 'static>(
        &self,
        kind: HookKind,
        init: impl FnOnce() -> T,
    ) -> Arc<Mutex<T>> {
        self.resources.bind();
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
            let value = Arc::new(Mutex::new(init()));
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

    /// End this hook context's resource lifetime, running cleanup once.
    /// Retained timer handles cannot schedule work after cleanup.
    pub fn cleanup(&self) {
        self.resources.close();
    }

    pub(crate) fn resource_scheduler(&self) -> Option<Arc<super::scheduler::Scheduler>> {
        self.resources.bind();
        self.resources.scheduler()
    }

    pub(crate) fn resource_token(&self) -> Weak<HookResources> {
        self.resources.bind();
        Arc::downgrade(&self.resources)
    }

    /// Bind the resources to the rendering component, like `resource_token`,
    /// and return a handle that reads whether they are still open.
    pub(crate) fn liveness(&self) -> Liveness {
        self.resources.bind();
        self.resources.liveness()
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
        self.set_except(value, None);
    }

    /// Publish an App's completed measurements without making it render itself.
    pub(crate) fn set_except(&self, value: T, excluded: Option<&super::wake::AppWaker>)
    where
        T: PartialEq,
    {
        self.store(value, || true, excluded);
    }

    /// Set a new value only if `holds` returns true. `holds` runs first,
    /// while the signal's store is locked, so once it would return false and
    /// `settle` has returned, no call begun earlier stores anything.
    /// Subscribers are notified after the lock is released.
    pub(crate) fn set_if(&self, value: T, holds: impl FnOnce() -> bool)
    where
        T: PartialEq,
    {
        self.store(value, holds, None);
    }

    /// Wait until a store under way has finished.
    pub(crate) fn settle(&self) {
        drop(self.inner.lock());
    }

    fn store(
        &self,
        value: T,
        holds: impl FnOnce() -> bool,
        excluded: Option<&super::wake::AppWaker>,
    ) where
        T: PartialEq,
    {
        let changed =
            if let (Ok(mut inner), Ok(mut version)) = (self.inner.lock(), self.version.lock()) {
                if holds() && *inner != value {
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
            self.app_subscribers.notify_except(excluded);
        }
    }

    /// Update the signal value using a function
    pub fn update(&self, f: impl FnOnce(&mut T))
    where
        T: PartialEq,
    {
        let mut next = match self.inner.lock() {
            Ok(inner) => inner.clone(),
            Err(_) => {
                log::warn!("Signal lock poisoned during update operation");
                return;
            }
        };
        let original = next.clone();
        f(&mut next);
        if next != original {
            self.set(next);
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
    let signal = hooks.get_or_create_storage(kind, || ThreadSafeSignal::new(initial));
    let result = signal.lock().expect("hook signal lock poisoned").clone();
    result
}

/// Run after each completed render, cleaning up the previous invocation first.
/// Without a render frame, run immediately and retain cleanup until `cleanup`
/// or the last Hooks clone drops. Use `use_effect_with_deps` to skip unchanged inputs.
pub fn use_effect<F>(hooks: &Hooks, effect: F) -> EffectId
where
    F: FnOnce() -> Option<Box<dyn FnOnce() + Send + Sync>> + Send + Sync + 'static,
{
    effect_hook(hooks, HookKind::Effect, (), true, effect)
}

/// Run after the first completed render and whenever `deps` changes by PartialEq.
/// Cleanup runs before replacement and when the component or Hooks owner ends.
pub fn use_effect_with_deps<D, F>(hooks: &Hooks, deps: D, effect: F) -> EffectId
where
    D: PartialEq + Send + Sync + 'static,
    F: FnOnce() -> Option<Box<dyn FnOnce() + Send + Sync>> + Send + Sync + 'static,
{
    effect_hook(hooks, HookKind::EffectDeps, deps, false, effect)
}

struct EffectHook<D> {
    deps: Option<D>,
    effect: Arc<OwnedEffect>,
}

fn effect_hook<D, F>(hooks: &Hooks, kind: HookKind, deps: D, always: bool, effect: F) -> EffectId
where
    D: PartialEq + Send + Sync + 'static,
    F: FnOnce() -> Option<Box<dyn FnOnce() + Send + Sync>> + Send + Sync + 'static,
{
    let storage = hooks.get_or_create_storage(kind, || EffectHook::<D> {
        deps: None,
        effect: Arc::new(OwnedEffect::new()),
    });
    let (owned, changed) = {
        let stored = storage.lock().unwrap();
        let changed = always || stored.deps.as_ref() != Some(&deps);
        (Arc::clone(&stored.effect), changed)
    };
    hooks.resources.track(&owned);
    if changed {
        let storage = Arc::downgrade(&storage);
        owned.queue(Box::new(move || {
            let cleanup = effect();
            if let Some(storage) = storage.upgrade() {
                storage.lock().unwrap().deps = Some(deps);
            }
            cleanup
        }));
    }
    if !hooks.state.lock().unwrap().rendering {
        owned.flush();
    }
    owned.id
}

/// Read the nearest provider in this component render.
/// Outside rendering, attached hooks read their last inherited context;
/// standalone hooks read their own provided values.
pub fn use_context<T: Clone + Send + Sync + 'static>(hooks: &Hooks) -> Option<T> {
    if super::component_scope::current().is_some() {
        return super::component_scope::lookup();
    }
    if let Some(value) = hooks.resources.context() {
        return value;
    }
    hooks
        .contexts
        .lock()
        .map_err(|_| log::warn!("Context lock poisoned in use_context"))
        .ok()?
        .get(&TypeId::of::<T>())
        .and_then(|any| any.downcast_ref::<T>())
        .cloned()
}

/// Provide a value to this component and its descendants for the current render.
/// Standalone calls store the value locally in Hooks.
pub fn provide_context<T: Send + Sync + 'static>(hooks: &Hooks, value: T) {
    let Err(value) = super::component_scope::provide(value) else {
        return;
    };
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
    let prev: Arc<Mutex<Option<T>>> = hooks.get_or_create_storage(HookKind::Previous, || None::<T>);
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

/// The active [`crate::theme::Theme`], as the application set it. Chart and
/// widget colors resolve through it (CHT-017).
pub fn use_theme() -> std::sync::Arc<crate::theme::Theme> {
    crate::theme::Theme::active()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rac_002_thread_safe_signal_update_allows_reentrant_read_and_write() {
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let signal = ThreadSafeSignal::new(1_i32);
            let nested = signal.clone();
            signal.update(|value| {
                assert_eq!(nested.get(), 1);
                nested.update(|nested_value| *nested_value = 2);
                assert_eq!(nested.get(), 2);
                *value = 3;
            });
            signal.set(4);
            send.send(signal.get()).unwrap();
        });
        assert_eq!(
            receive
                .recv_timeout(std::time::Duration::from_secs(30))
                .unwrap(),
            4
        );
    }

    #[test]
    fn aborted_render_does_not_commit_effect_dependencies() {
        let hooks = Hooks::new();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let render = |value: i32, abort: bool| {
            let _frame = hooks.begin_render();
            let calls = calls.clone();
            use_effect_with_deps(&hooks, value, move || {
                calls.lock().unwrap().push(value);
                Some(Box::new(move || calls.lock().unwrap().push(-value)))
            });
            assert!(!abort, "deliberately aborted render");
        };
        render(1, false);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render(2, true))).is_err()
        );
        assert_eq!(*calls.lock().unwrap(), [1]);
        render(2, false);
        assert_eq!(*calls.lock().unwrap(), [1, -1, 2]);
        hooks.cleanup();
        assert_eq!(*calls.lock().unwrap(), [1, -1, 2, -2]);
    }

    #[test]
    fn panicking_effect_does_not_leave_its_slot_running() {
        let hooks = Hooks::new();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _frame = hooks.begin_render();
            use_effect_with_deps(&hooks, (), || panic!("deliberate effect panic"));
        }))
        .is_err());
        let calls = Arc::new(Mutex::new(0));
        {
            let _frame = hooks.begin_render();
            let calls = calls.clone();
            use_effect_with_deps(&hooks, (), move || {
                *calls.lock().unwrap() += 1;
                None
            });
        }
        assert_eq!(*calls.lock().unwrap(), 1);
    }

    #[test]
    fn final_hook_clone_drop_runs_cleanup_once() {
        let hooks = Hooks::new();
        let other = hooks.clone();
        let calls = Arc::new(Mutex::new(0));
        let copy = calls.clone();
        use_effect(&hooks, move || {
            Some(Box::new(move || *copy.lock().unwrap() += 1))
        });
        drop(hooks);
        assert_eq!(*calls.lock().unwrap(), 0);
        drop(other);
        assert_eq!(*calls.lock().unwrap(), 1);
    }

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
