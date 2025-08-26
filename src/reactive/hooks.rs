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
        *self.index.lock().unwrap() = 0;
    }
    
    /// Get or create storage at current index
    fn get_or_create_storage<T: Send + Sync + 'static>(&self, init: impl FnOnce() -> T) -> Arc<Mutex<T>> {
        let mut index = self.index.lock().unwrap();
        let mut storage = self.storage.lock().unwrap();
        
        if *index >= storage.len() {
            let value = Arc::new(Mutex::new(init()));
            storage.push(Box::new(value.clone()));
            *index += 1;
            value
        } else {
            let stored = &storage[*index];
            *index += 1;
            
            stored
                .downcast_ref::<Arc<Mutex<T>>>()
                .expect("Hook type mismatch")
                .clone()
        }
    }
    
    /// Cleanup all effects
    pub fn cleanup(&self) {
        // Effects will be cleaned up by the runtime
        self.effects.lock().unwrap().clear();
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

impl<T: Clone> ThreadSafeSignal<T> {
    pub fn new(initial: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(initial)),
            version: Arc::new(Mutex::new(0)),
        }
    }
    
    pub fn get(&self) -> T {
        self.inner.lock().unwrap().clone()
    }
    
    pub fn set(&self, value: T) where T: PartialEq {
        let mut inner = self.inner.lock().unwrap();
        if *inner != value {
            *inner = value;
            *self.version.lock().unwrap() += 1;
        }
    }
    
    pub fn update(&self, f: impl FnOnce(&mut T)) where T: PartialEq {
        let mut inner = self.inner.lock().unwrap();
        let old = inner.clone();
        f(&mut *inner);
        if *inner != old {
            *self.version.lock().unwrap() += 1;
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
pub fn use_signal<T: Send + Sync + Clone + 'static>(hooks: &Hooks, initial: T) -> ThreadSafeSignal<T> {
    let signal = hooks.get_or_create_storage(|| ThreadSafeSignal::new(initial));
    signal.lock().unwrap().clone()
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
        ctx.runtime().register_effect(super::effect::Effect::new(move || {
            // Adapt Send+Sync cleanup to non-threaded cleanup expected by Effect
            let cleanup = effect_inner();
            cleanup.map(|boxed| {
                Box::new(move || {
                    boxed()
                }) as Box<dyn FnOnce()>
            })
        }))
    }) {
        hooks.effects.lock().unwrap().push(id);
        return id;
    }

    // Fallback: no runtime set. Execute effect now (best-effort) and return a fresh id.
    let effect_id = EffectId::new();
    hooks.effects.lock().unwrap().push(effect_id);
    if let Some(cleanup) = effect_opt.and_then(|f| f()) {
        // Without a scheduler to own deferred cleanup, execute immediately to avoid leaks
        cleanup();
    }
    effect_id
}

/// Access context value (thread-safe version)
pub fn use_context<T: Clone + Send + Sync + 'static>(hooks: &Hooks) -> Option<T> {
    let contexts = hooks.contexts.lock().unwrap();
    contexts
        .get(&TypeId::of::<T>())
        .and_then(|any| any.downcast_ref::<T>())
        .cloned()
}

/// Provide context value to children (thread-safe version)
pub fn provide_context<T: Send + Sync + 'static>(hooks: &Hooks, value: T) {
    let mut contexts = hooks.contexts.lock().unwrap();
    contexts.insert(TypeId::of::<T>(), Box::new(value));
}

/// Hook for managing state with a reducer (thread-safe version)
pub fn use_reducer<S, A, F>(
    hooks: &Hooks,
    reducer: F,
    initial: S,
) -> (ThreadSafeSignal<S>, Arc<dyn Fn(A) + Send + Sync>)
where
    S: Clone + PartialEq + Send + Sync + 'static,
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
    let mut prev_guard = prev.lock().unwrap();
    let old = prev_guard.clone();
    *prev_guard = Some(value);
    old
}

/// Create a memoized computed value
pub fn use_memo<T>(hooks: &Hooks, compute: impl Fn() -> T + Send + Sync + 'static) -> ThreadSafeSignal<T>
where
    T: Clone + PartialEq + Send + Sync + 'static
{
    let memo = use_signal(hooks, compute());
    // In a real implementation, this would track dependencies
    // For now, just return a signal
    memo
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
        }).join().unwrap();
        
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
        }).join().unwrap();
    }
}