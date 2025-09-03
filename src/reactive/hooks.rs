use super::effect::EffectId;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Thread-safe hook context for managing component state and effects
#[derive(Clone)]
pub struct Hooks {
    /// Current hook index for ordering
    index: Arc<Mutex<usize>>,

    /// Storage for hook state
    storage: Arc<Mutex<Vec<Box<dyn Any + Send + Sync>>>>,

    /// Active effects for this component
    effects: Arc<Mutex<Vec<EffectId>>>,

    /// Context values by type
    contexts: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl Hooks {
    /// Create a new hooks context
    pub fn new() -> Self {
        Self {
            index: Arc::new(Mutex::new(0)),
            storage: Arc::new(Mutex::new(Vec::new())),
            effects: Arc::new(Mutex::new(Vec::new())),
            contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Reset hook index for new render
    pub fn reset(&self) {
        if let Ok(mut index) = self.index.lock() {
            *index = 0;
        } else {
            log::error!("Failed to acquire hook index lock during reset");
        }
    }

    /// Get or create storage at current index
    fn get_or_create_storage<T: Send + Sync + 'static>(
        &self,
        init: impl FnOnce() -> T,
    ) -> Arc<Mutex<T>> {
        // Use try_lock to avoid deadlock, with fallback behavior
        let index_result = self.index.try_lock();
        let storage_result = self.storage.try_lock();

        match (index_result, storage_result) {
            (Ok(mut index), Ok(mut storage)) => {
                if *index >= storage.len() {
                    let value = Arc::new(Mutex::new(init()));
                    storage.push(Box::new(value.clone()));
                    *index += 1;
                    value
                } else {
                    let stored = &storage[*index];
                    *index += 1;

                    if let Some(arc) = stored.downcast_ref::<Arc<Mutex<T>>>() {
                        arc.clone()
                    } else {
                        log::error!("Type mismatch in hook storage, creating new value");
                        Arc::new(Mutex::new(init()))
                    }
                }
            }
            _ => {
                log::error!("Failed to acquire hook storage locks, creating isolated value");
                // Fallback: create isolated value (not ideal but prevents panic)
                Arc::new(Mutex::new(init()))
            }
        }
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
}

impl<T: Clone + Default> ThreadSafeSignal<T> {
    /// Create a new thread-safe signal with initial value
    pub fn new(initial: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(initial)),
            version: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the current value of the signal
    pub fn get(&self) -> T {
        self.inner.lock()
            .map(|guard| guard.clone())
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
        if let (Ok(mut inner), Ok(mut version)) = (self.inner.lock(), self.version.lock()) {
            if *inner != value {
                *inner = value;
                *version += 1;
            }
        } else {
            log::warn!("Signal locks poisoned during set operation");
        }
    }

    /// Update the signal value using a function
    pub fn update(&self, f: impl FnOnce(&mut T))
    where
        T: PartialEq,
    {
        if let (Ok(mut inner), Ok(mut version)) = (self.inner.lock(), self.version.lock()) {
            let old = inner.clone();
            f(&mut *inner);
            if *inner != old {
                *version += 1;
            }
        } else {
            log::warn!("Signal locks poisoned during update operation");
        }
    }
}

impl<T> Clone for ThreadSafeSignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            version: Arc::clone(&self.version),
        }
    }
}

/// Create a reactive signal (thread-safe version)
pub fn use_signal<T: Send + Sync + Clone + Default + 'static>(
    hooks: &Hooks,
    initial: T,
) -> ThreadSafeSignal<T> {
    let initial_clone = initial.clone();
    let signal = hooks.get_or_create_storage(|| ThreadSafeSignal::new(initial));
    signal.lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| {
            log::warn!("Signal lock poisoned in use_signal, returning default");
            ThreadSafeSignal::new(initial_clone)
        })
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
    hooks.contexts.lock()
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
    let state = use_signal(hooks, initial);
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
    let prev: Arc<Mutex<Option<T>>> = hooks.get_or_create_storage(|| None::<T>);
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

/// Create a memoized computed value
pub fn use_memo<T>(
    hooks: &Hooks,
    compute: impl Fn() -> T + Send + Sync + 'static,
) -> ThreadSafeSignal<T>
where
    T: Clone + PartialEq + Send + Sync + Default + 'static,
{
    // Compute and return a reactive signal
    use_signal(hooks, compute())
}

#[cfg(test)]
mod tests {
    use super::*;

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
