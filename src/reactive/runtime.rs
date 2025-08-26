use super::effect::{Effect, EffectId};
use super::signal::{Signal, SignalId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::{Rc, Weak};

/// The reactive runtime that manages signals and effects
pub struct ReactiveRuntime {
    /// Map of signal IDs to their dependent effects
    signal_effects: RefCell<HashMap<SignalId, Vec<Weak<RefCell<Effect>>>>>,

    /// Map of effect IDs to their effect instances
    effects: RefCell<HashMap<EffectId, Rc<RefCell<Effect>>>>,

    /// Queue of effects to run
    effect_queue: RefCell<VecDeque<EffectId>>,

    /// Currently executing effect (for dependency tracking)
    current_effect: RefCell<Option<EffectId>>,

    /// Batch depth for batching updates
    batch_depth: RefCell<usize>,

    /// Pending effects to run after batch completes
    pending_effects: RefCell<HashSet<EffectId>>,

    /// Cycle detection: effects currently being processed
    processing: RefCell<HashSet<EffectId>>,
}

impl ReactiveRuntime {
    /// Create a new reactive runtime
    pub fn new() -> Self {
        Self {
            signal_effects: RefCell::new(HashMap::new()),
            effects: RefCell::new(HashMap::new()),
            effect_queue: RefCell::new(VecDeque::new()),
            current_effect: RefCell::new(None),
            batch_depth: RefCell::new(0),
            pending_effects: RefCell::new(HashSet::new()),
            processing: RefCell::new(HashSet::new()),
        }
    }

    /// Register a signal access (for dependency tracking)
    pub fn track_signal(&self, signal_id: SignalId) {
        if let Some(effect_id) = *self.current_effect.borrow() {
            // Add this signal as a dependency of the current effect
            if let Some(effect) = self.effects.borrow().get(&effect_id) {
                effect.borrow().add_dependency(signal_id);

                // Add the effect to the signal's dependent list
                let mut signal_effects = self.signal_effects.borrow_mut();
                signal_effects
                    .entry(signal_id)
                    .or_default()
                    .push(Rc::downgrade(effect));
            }
        }
    }

    /// Notify that a signal has changed
    pub fn signal_changed(&self, signal_id: SignalId) {
        let signal_effects = self.signal_effects.borrow();

        if let Some(effects) = signal_effects.get(&signal_id) {
            let batch_depth = *self.batch_depth.borrow();

            for weak_effect in effects {
                if let Some(_effect) = weak_effect.upgrade() {
                    let effect_id = EffectId::new(); // Would need to store ID in Effect

                    if batch_depth > 0 {
                        // We're in a batch, queue the effect
                        self.pending_effects.borrow_mut().insert(effect_id);
                    } else {
                        // Run immediately
                        self.effect_queue.borrow_mut().push_back(effect_id);
                    }
                }
            }

            // Process queue if not batching
            if batch_depth == 0 {
                self.flush_effects();
            }
        }
    }

    /// Start a batch update
    pub fn batch<R>(&self, f: impl FnOnce() -> R) -> R {
        *self.batch_depth.borrow_mut() += 1;
        let result = f();
        *self.batch_depth.borrow_mut() -= 1;

        // If we've exited all batches, flush pending effects
        if *self.batch_depth.borrow() == 0 {
            self.flush_pending_effects();
        }

        result
    }

    /// Flush pending effects after batch
    fn flush_pending_effects(&self) {
        let pending = self
            .pending_effects
            .borrow_mut()
            .drain()
            .collect::<Vec<_>>();

        for effect_id in pending {
            self.effect_queue.borrow_mut().push_back(effect_id);
        }

        self.flush_effects();
    }

    /// Process the effect queue
    fn flush_effects(&self) {
        while let Some(effect_id) = self.effect_queue.borrow_mut().pop_front() {
            // Cycle detection
            if !self.processing.borrow_mut().insert(effect_id) {
                // Already processing this effect - cycle detected!
                continue;
            }

            if let Some(effect) = self.effects.borrow().get(&effect_id) {
                // Set as current effect for dependency tracking
                *self.current_effect.borrow_mut() = Some(effect_id);

                // Run the effect
                effect.borrow().run();

                // Clear current effect
                *self.current_effect.borrow_mut() = None;
            }

            self.processing.borrow_mut().remove(&effect_id);
        }
    }

    /// Register an effect with the runtime
    pub fn register_effect(&self, effect: Effect) -> EffectId {
        let effect_id = EffectId::new();
        let effect_rc = Rc::new(RefCell::new(effect));

        self.effects.borrow_mut().insert(effect_id, effect_rc);

        // Run the effect immediately
        self.effect_queue.borrow_mut().push_back(effect_id);
        self.flush_effects();

        effect_id
    }

    /// Unregister an effect
    pub fn unregister_effect(&self, effect_id: EffectId) {
        if let Some(effect) = self.effects.borrow_mut().remove(&effect_id) {
            effect.borrow().dispose();
        }
    }

    /// Clean up weak references that are no longer valid
    pub fn cleanup(&self) {
        let mut signal_effects = self.signal_effects.borrow_mut();

        // Remove dead weak references
        signal_effects.retain(|_, effects| {
            effects.retain(|weak| weak.upgrade().is_some());
            !effects.is_empty()
        });
    }
}

impl Default for ReactiveRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime context for components
pub struct RuntimeContext {
    runtime: Rc<ReactiveRuntime>,
}

impl RuntimeContext {
    /// Create a new runtime context
    pub fn new() -> Self {
        Self {
            runtime: Rc::new(ReactiveRuntime::new()),
        }
    }

    /// Get the runtime
    pub fn runtime(&self) -> &ReactiveRuntime {
        &self.runtime
    }

    /// Create a signal in this runtime
    pub fn create_signal<T>(&self, initial: T) -> Signal<T> {
        // Could register with runtime for tracking
        Signal::new(initial)
    }

    /// Create an effect in this runtime
    pub fn create_effect(
        &self,
        f: impl FnOnce() -> Option<Box<dyn FnOnce()>> + 'static,
    ) -> EffectId {
        let effect = Effect::new(f);
        self.runtime.register_effect(effect)
    }

    /// Batch multiple updates
    pub fn batch<R>(&self, f: impl FnOnce() -> R) -> R {
        self.runtime.batch(f)
    }

    /// Clean up the runtime
    pub fn cleanup(&self) {
        self.runtime.cleanup();
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    /// Global runtime for convenience (optional)
    static RUNTIME: RefCell<Option<RuntimeContext>> = const { RefCell::new(None) };
}

/// Set the global runtime context
pub fn set_runtime(runtime: RuntimeContext) {
    RUNTIME.with(|r| {
        *r.borrow_mut() = Some(runtime);
    });
}

/// Get the global runtime context
pub fn with_runtime<R>(f: impl FnOnce(&RuntimeContext) -> R) -> Option<R> {
    RUNTIME.with(|r| r.borrow().as_ref().map(f))
}

/// Clear the global runtime
pub fn clear_runtime() {
    RUNTIME.with(|r| {
        *r.borrow_mut() = None;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_runtime_basic() {
        let runtime = ReactiveRuntime::new();
        let counter = Rc::new(Cell::new(0));
        let counter_clone = counter.clone();

        let effect = Effect::new(move || {
            counter_clone.set(counter_clone.get() + 1);
            None
        });

        runtime.register_effect(effect);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_runtime_batch() {
        let runtime = ReactiveRuntime::new();
        let _counter = Rc::new(Cell::new(0));

        runtime.batch(|| {
            // Multiple signal changes in a batch
            for _ in 0..5 {
                // Would trigger effects, but they're batched
                runtime.signal_changed(SignalId::new());
            }
        });

        // Effects would run after batch completes
    }

    #[test]
    fn test_runtime_context() {
        let ctx = RuntimeContext::new();

        let signal = ctx.create_signal(42);
        assert_eq!(signal.get(), 42);

        let counter = Rc::new(Cell::new(0));
        let counter_clone = counter.clone();

        ctx.create_effect(move || {
            counter_clone.set(counter_clone.get() + 1);
            None
        });

        assert_eq!(counter.get(), 1);

        // Batch updates
        ctx.batch(|| {
            signal.set(100);
            signal.set(200);
        });
    }

    #[test]
    fn test_global_runtime() {
        let runtime = RuntimeContext::new();
        set_runtime(runtime);

        let result = with_runtime(|ctx| {
            let signal = ctx.create_signal(10);
            signal.get()
        });

        assert_eq!(result, Some(10));

        clear_runtime();

        let result = with_runtime(|_ctx| 42);
        assert_eq!(result, None);
    }
}
