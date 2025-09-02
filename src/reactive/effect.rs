use super::signal::SignalId;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

/// Unique identifier for an effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(usize);

impl Default for EffectId {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectId {
    /// Create a new unique effect ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// A cleanup function to run when an effect is re-executed or disposed
pub type Cleanup = Box<dyn FnOnce()>;

/// Represents a side effect that runs in response to signal changes
pub struct Effect {
    /// Unique identifier for this effect
    id: EffectId,
    /// Shared inner state
    inner: Rc<RefCell<EffectInner>>,
}

struct EffectInner {
    /// The effect function to run
    effect_fn: Option<Box<dyn FnOnce() -> Option<Cleanup>>>,

    /// Cleanup function from the last execution
    cleanup: Option<Cleanup>,

    /// Set of signal dependencies
    dependencies: HashSet<SignalId>,

    /// Whether this effect is currently running
    running: bool,

    /// Whether this effect has been disposed
    disposed: bool,
}

impl Effect {
    /// Create a new effect
    pub fn new(effect_fn: impl FnOnce() -> Option<Cleanup> + 'static) -> Self {
        Self {
            id: EffectId::new(),
            inner: Rc::new(RefCell::new(EffectInner {
                effect_fn: Some(Box::new(effect_fn)),
                cleanup: None,
                dependencies: HashSet::new(),
                running: false,
                disposed: false,
            })),
        }
    }

    /// Get this effect's ID
    pub fn id(&self) -> EffectId {
        self.id
    }

    /// Run the effect
    /// Execute the effect
    pub fn run(&self) {
        let mut inner = self.inner.borrow_mut();

        if inner.disposed || inner.running {
            return;
        }

        // Run cleanup from previous execution
        if let Some(cleanup) = inner.cleanup.take() {
            drop(inner); // Release borrow before running cleanup
            cleanup();
            inner = self.inner.borrow_mut();
        }

        // Clear previous dependencies
        inner.dependencies.clear();
        inner.running = true;

        // Take the effect function (it can only run once)
        if let Some(effect_fn) = inner.effect_fn.take() {
            drop(inner); // Release borrow before running effect

            // Run the effect and capture cleanup
            let cleanup = effect_fn();

            let mut inner = self.inner.borrow_mut();
            inner.cleanup = cleanup;
            inner.running = false;
        } else {
            inner.running = false;
        }
    }

    /// Add a signal dependency
    pub fn add_dependency(&self, signal_id: SignalId) {
        self.inner.borrow_mut().dependencies.insert(signal_id);
    }

    /// Check if this effect depends on a signal
    pub fn depends_on(&self, signal_id: SignalId) -> bool {
        self.inner.borrow().dependencies.contains(&signal_id)
    }

    /// Get all dependencies
    pub fn dependencies(&self) -> Vec<SignalId> {
        self.inner.borrow().dependencies.iter().copied().collect()
    }

    /// Dispose of the effect, running cleanup
    pub fn dispose(&self) {
        let mut inner = self.inner.borrow_mut();

        if inner.disposed {
            return;
        }

        inner.disposed = true;

        // Run final cleanup
        if let Some(cleanup) = inner.cleanup.take() {
            drop(inner);
            cleanup();
        }
    }

    /// Check if the effect is disposed
    pub fn is_disposed(&self) -> bool {
        self.inner.borrow().disposed
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        self.dispose();
    }
}

/// A batch of effects to run together
pub struct EffectBatch {
    effects: Vec<Effect>,
}

impl EffectBatch {
    /// Create a new effect batch
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Add an effect to the batch
    pub fn add(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    /// Run all effects in the batch
    pub fn run_all(&self) {
        for effect in &self.effects {
            effect.run();
        }
    }

    /// Dispose all effects in the batch
    pub fn dispose_all(&self) {
        for effect in &self.effects {
            effect.dispose();
        }
    }
}

impl Default for EffectBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_effect_runs() {
        let counter = Rc::new(Cell::new(0));
        let counter_clone = counter.clone();

        let effect = Effect::new(move || {
            counter_clone.set(counter_clone.get() + 1);
            None
        });

        assert_eq!(counter.get(), 0);
        effect.run();
        assert_eq!(counter.get(), 1);

        // Effect function is consumed, running again does nothing
        effect.run();
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_effect_cleanup() {
        let cleanup_counter = Rc::new(Cell::new(0));
        let cleanup_clone = cleanup_counter.clone();

        let effect_counter = Rc::new(Cell::new(0));
        let effect_clone = effect_counter.clone();

        let effect = Effect::new(move || {
            effect_clone.set(effect_clone.get() + 1);
            Some(Box::new(move || {
                cleanup_clone.set(cleanup_clone.get() + 1);
            }))
        });

        effect.run();
        assert_eq!(effect_counter.get(), 1);
        assert_eq!(cleanup_counter.get(), 0);

        effect.dispose();
        assert_eq!(cleanup_counter.get(), 1);
    }

    #[test]
    fn test_effect_dependencies() {
        let effect = Effect::new(|| None);

        let signal1 = SignalId::new();
        let signal2 = SignalId::new();

        effect.add_dependency(signal1);
        effect.add_dependency(signal2);

        assert!(effect.depends_on(signal1));
        assert!(effect.depends_on(signal2));

        let deps = effect.dependencies();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&signal1));
        assert!(deps.contains(&signal2));
    }

    #[test]
    fn test_effect_batch() {
        let counter = Rc::new(Cell::new(0));
        let mut batch = EffectBatch::new();

        for i in 1..=3 {
            let counter_clone = counter.clone();
            batch.add(Effect::new(move || {
                counter_clone.set(counter_clone.get() + i);
                None
            }));
        }

        assert_eq!(counter.get(), 0);
        batch.run_all();
        assert_eq!(counter.get(), 6); // 1 + 2 + 3
    }
}
