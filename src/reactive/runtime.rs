use super::effect::{Effect, EffectId};
use super::scheduler::Scheduler;
use super::signal::{Signal, SignalId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::{Rc, Weak};
use std::sync::Arc;

/// The reactive runtime that manages signals and effects
pub struct ReactiveRuntime {
    /// Map of signal IDs to their dependent effects
    signal_effects: RefCell<HashMap<SignalId, Vec<Weak<Effect>>>>,

    /// Map of effect IDs to their effect instances
    effects: RefCell<HashMap<EffectId, Rc<Effect>>>,

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

    /// Effects whose run on the stack changed a signal that run read: each
    /// runs again when its run returns (SIG-004)
    rerun: RefCell<HashSet<EffectId>>,
}

/// How many times in a row an effect runs again because its own run changed
/// a signal it read. A run that changes such a signal every time would never
/// settle; after this many reruns the runtime stops and the last run stands
/// until the next change from outside.
pub const RERUN_LIMIT: usize = 100;

struct RuntimeEffectGuard<'a> {
    runtime: &'a ReactiveRuntime,
    effect_id: EffectId,
    previous: Option<EffectId>,
}

impl Drop for RuntimeEffectGuard<'_> {
    fn drop(&mut self) {
        self.runtime.current_effect.replace(self.previous);
        self.runtime.processing.borrow_mut().remove(&self.effect_id);
    }
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
            rerun: RefCell::new(HashSet::new()),
        }
    }

    /// Register a signal access (for dependency tracking)
    pub fn track_signal(&self, signal_id: SignalId) {
        if let Some(effect_id) = *self.current_effect.borrow() {
            // Add this signal as a dependency of the current effect
            if let Some(effect) = self.effects.borrow().get(&effect_id).cloned() {
                effect.add_dependency(signal_id);

                // Add the effect to the signal's dependent list, once however
                // many times a run reads the signal (SIG-004)
                let mut signal_effects = self.signal_effects.borrow_mut();
                let dependents = signal_effects.entry(signal_id).or_default();
                let dependent = Rc::downgrade(&effect);
                if !dependents
                    .iter()
                    .any(|known| Weak::ptr_eq(known, &dependent))
                {
                    dependents.push(dependent);
                }
            }
        }
    }

    /// Notify that a signal has changed: the effects whose last run read it
    /// run again (SIG-004)
    pub fn signal_changed(&self, signal_id: SignalId) {
        let effects = self
            .signal_effects
            .borrow()
            .get(&signal_id)
            .cloned()
            .unwrap_or_default();
        if !effects.is_empty() {
            let batch_depth = *self.batch_depth.borrow();
            let mut queued = Vec::new();
            let mut dead = false;
            for weak_effect in &effects {
                let Some(effect) = weak_effect.upgrade() else {
                    dead = true;
                    continue;
                };
                let effect_id = effect.id();
                // An effect whose last run did not read the signal is no
                // longer one of its dependents
                if !effect.depends_on(signal_id) || queued.contains(&effect_id) {
                    continue;
                }
                queued.push(effect_id);

                if batch_depth > 0 {
                    // We're in a batch, queue the effect
                    self.pending_effects.borrow_mut().insert(effect_id);
                } else {
                    // Run immediately
                    self.effect_queue.borrow_mut().push_back(effect_id);
                }
            }
            if dead {
                // A dependent that no longer exists leaves the record now
                let mut signal_effects = self.signal_effects.borrow_mut();
                if let Some(dependents) = signal_effects.get_mut(&signal_id) {
                    dependents.retain(|known| known.strong_count() > 0);
                    if dependents.is_empty() {
                        signal_effects.remove(&signal_id);
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

    /// Process the effect queue. An effect queued while its own run is on
    /// the stack, because that run changed a signal it read, runs again when
    /// the run returns, up to [`RERUN_LIMIT`] times in a row, so the change
    /// is not dropped and its last run sees the final value (SIG-004).
    fn flush_effects(&self) {
        loop {
            let Some(effect_id) = self.effect_queue.borrow_mut().pop_front() else {
                break;
            };
            if !self.processing.borrow_mut().insert(effect_id) {
                // Its run is on the stack and changed a signal it read: the
                // effect runs again when that run returns.
                self.rerun.borrow_mut().insert(effect_id);
                continue;
            }
            let effect = self.effects.borrow().get(&effect_id).cloned();
            let Some(effect) = effect else {
                self.processing.borrow_mut().remove(&effect_id);
                continue;
            };
            let previous = self.current_effect.replace(Some(effect_id));
            let _running = RuntimeEffectGuard {
                runtime: self,
                effect_id,
                previous,
            };
            let before = effect.dependencies();
            let mut reruns = 0;
            loop {
                effect.run();
                let again = self.rerun.borrow_mut().remove(&effect_id);
                if !again || effect.is_disposed() || reruns == RERUN_LIMIT {
                    break;
                }
                reruns += 1;
            }
            // The records follow the last run: a signal it no longer reads
            // forgets the effect, and every signal does once it is disposed.
            let after = effect.dependencies();
            let stale = before.into_iter().filter(|signal| !after.contains(signal));
            if effect.is_disposed() {
                self.forget_dependents(&effect, stale.chain(after.iter().copied()));
            } else {
                self.forget_dependents(&effect, stale);
            }
        }
    }

    /// Register an effect with the runtime
    pub fn register_effect(&self, effect: Effect) -> EffectId {
        let effect_id = effect.id();
        let effect_rc = Rc::new(effect);

        self.effects.borrow_mut().insert(effect_id, effect_rc);

        // Run the effect immediately
        self.effect_queue.borrow_mut().push_back(effect_id);
        self.flush_effects();

        effect_id
    }

    /// Run a registered effect by hand with cleanup and dependency tracking.
    pub fn run_effect(&self, effect_id: EffectId) {
        self.effect_queue.borrow_mut().push_back(effect_id);
        self.flush_effects();
    }

    /// Unregister an effect: it runs no more, and no signal lists it as a
    /// dependent any longer (SIG-004).
    pub fn unregister_effect(&self, effect_id: EffectId) {
        self.rerun.borrow_mut().remove(&effect_id);
        let effect = self.effects.borrow_mut().remove(&effect_id);
        if let Some(effect) = effect {
            effect.dispose();
            self.forget_dependents(&effect, effect.dependencies());
        }
    }

    /// Take `effect` off the dependents of `signals`, together with any
    /// dependent that no longer exists, and drop a signal's record once it
    /// lists nobody, so the records are bounded by the live effects and the
    /// signals their last runs read.
    fn forget_dependents(&self, effect: &Rc<Effect>, signals: impl IntoIterator<Item = SignalId>) {
        let gone = Rc::downgrade(effect);
        let mut signal_effects = self.signal_effects.borrow_mut();
        for signal_id in signals {
            if let Some(dependents) = signal_effects.get_mut(&signal_id) {
                dependents.retain(|known| !Weak::ptr_eq(known, &gone) && known.strong_count() > 0);
                if dependents.is_empty() {
                    signal_effects.remove(&signal_id);
                }
            }
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

    /// Comprehensive cleanup of dead effects and references. A dead effect is
    /// one that was disposed; a registered effect lives until
    /// `unregister_effect` or the runtime's drop, however many references
    /// it has (SIG-004).
    pub fn cleanup_dead_effects(&self) {
        let initial_effects_count;
        let initial_signal_effects_count;

        // Clean up main effects map - remove the effects that were disposed
        {
            let mut effects = self.effects.borrow_mut();
            initial_effects_count = effects.len();
            let stale = effects
                .iter()
                .filter_map(|(id, effect)| effect.is_disposed().then_some(*id))
                .collect::<Vec<_>>();
            let removed = stale
                .into_iter()
                .filter_map(|id| effects.remove(&id))
                .collect::<Vec<_>>();
            drop(effects);
            for effect in removed {
                effect.dispose();
            }
        }

        // Clean up signal_effects weak references
        {
            let mut signal_effects = self.signal_effects.borrow_mut();
            initial_signal_effects_count = signal_effects.len();

            signal_effects.retain(|_, effects| {
                effects.retain(|weak| weak.upgrade().is_some());
                !effects.is_empty()
            });
        }

        // Clean up effect queue - remove effects that no longer exist
        {
            let mut queue = self.effect_queue.borrow_mut();
            let effects = self.effects.borrow();
            queue.retain(|effect_id| effects.contains_key(effect_id));
        }

        // Clean up pending effects
        {
            let mut pending = self.pending_effects.borrow_mut();
            let effects = self.effects.borrow();
            pending.retain(|effect_id| effects.contains_key(effect_id));
        }

        // Clean up processing set
        {
            let mut processing = self.processing.borrow_mut();
            let effects = self.effects.borrow();
            processing.retain(|effect_id| effects.contains_key(effect_id));
        }

        // Clean up rerun requests
        {
            let mut rerun = self.rerun.borrow_mut();
            let effects = self.effects.borrow();
            rerun.retain(|effect_id| effects.contains_key(effect_id));
        }

        let final_effects_count = self.effects.borrow().len();
        let final_signal_effects_count = self.signal_effects.borrow().len();

        let cleaned_effects = initial_effects_count - final_effects_count;
        let cleaned_signal_effects = initial_signal_effects_count - final_signal_effects_count;

        if cleaned_effects > 0 || cleaned_signal_effects > 0 {
            // log::debug!(
            //     "Cleaned up {} dead effects and {} signal effect mappings",
            //     cleaned_effects,
            //     cleaned_signal_effects
            // );
        }
    }

    /// Periodic cleanup - call this regularly in long-running applications
    pub fn periodic_cleanup(&self) {
        self.cleanup_dead_effects();

        // Additional periodic maintenance
        let effects_count = self.effects.borrow().len();
        let queue_size = self.effect_queue.borrow().len();
        let pending_size = self.pending_effects.borrow().len();

        if effects_count > 1000 || queue_size > 100 || pending_size > 100 {
            // log::warn!(
            //     "High reactive system usage: {} effects, {} queued, {} pending",
            //     effects_count, queue_size, pending_size
            // );
        }
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
    scheduler: Arc<Scheduler>,
}

impl RuntimeContext {
    /// Create a new runtime context
    pub fn new() -> Self {
        Self {
            runtime: Rc::new(ReactiveRuntime::new()),
            scheduler: Arc::new(Scheduler::new()),
        }
    }

    /// Get the scheduler
    pub fn scheduler(&self) -> Arc<Scheduler> {
        self.scheduler.clone()
    }

    /// Get the runtime
    pub fn runtime(&self) -> &ReactiveRuntime {
        &self.runtime
    }

    /// Create a signal in this runtime. An effect of this context that reads
    /// it runs again when it changes (SIG-004).
    pub fn create_signal<T>(&self, initial: T) -> Signal<T> {
        Signal::in_runtime(initial, Rc::downgrade(&self.runtime))
    }

    /// Create an effect in this runtime. It runs now, and again after each
    /// change of a signal of this context it read during its last run, after
    /// the cleanup that run returned, until `unregister_effect` on
    /// [`Self::runtime`] removes it or the context is dropped (SIG-004).
    pub fn create_effect(&self, f: impl Fn() -> Option<Box<dyn FnOnce()>> + 'static) -> EffectId {
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

    /// Comprehensive cleanup of dead effects
    pub fn cleanup_dead_effects(&self) {
        self.runtime.cleanup_dead_effects();
    }

    /// Periodic cleanup for long-running applications
    pub fn periodic_cleanup(&self) {
        self.runtime.periodic_cleanup();
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod rac_002_tests {
    use super::*;
    use std::cell::Cell;
    use std::time::Duration;

    fn expression_has_guard(expression: &syn::Expr) -> bool {
        use syn::visit::Visit;
        struct GuardFinder(bool);
        impl<'ast> Visit<'ast> for GuardFinder {
            fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
                if matches!(
                    call.method.to_string().as_str(),
                    "lock" | "borrow" | "borrow_mut"
                ) {
                    self.0 = true;
                }
                syn::visit::visit_expr_method_call(self, call);
            }
        }
        let mut finder = GuardFinder(false);
        finder.visit_expr(expression);
        finder.0
    }

    #[test]
    fn rac_002_source_inventory_rejects_callbacks_called_from_guard_expressions() {
        use syn::visit::Visit;

        struct Inventory {
            violations: Vec<String>,
        }
        impl<'ast> Visit<'ast> for Inventory {
            fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
                if expression_has_guard(&call.func) {
                    self.violations
                        .push("callback callee retains a mutex or RefCell guard".into());
                }
                syn::visit::visit_expr_call(self, call);
            }

            fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
                if matches!(call.method.to_string().as_str(), "run" | "dispose")
                    && expression_has_guard(&call.receiver)
                {
                    self.violations
                        .push("effect callback receiver retains a RefCell guard".into());
                }
                syn::visit::visit_expr_method_call(self, call);
            }
        }

        let sources = [
            include_str!("runtime.rs"),
            include_str!("hooks.rs"),
            include_str!("../hooks/refs.rs"),
            include_str!("../hooks/timer.rs"),
        ];
        let mut inventory = Inventory {
            violations: Vec::new(),
        };
        for source in sources {
            inventory.visit_file(&syn::parse_file(source).unwrap());
        }
        assert!(
            inventory.violations.is_empty(),
            "user callbacks must not be invoked through live guards: {:?}",
            inventory.violations
        );
    }

    #[test]
    fn rac_002_runtime_effect_can_create_track_and_unregister_reentrantly() {
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let runtime = Rc::new(ReactiveRuntime::new());
            let weak = Rc::downgrade(&runtime);
            let nested_calls = Rc::new(Cell::new(0));
            let nested_result = Rc::clone(&nested_calls);
            let outer = Effect::new(move || {
                let runtime = weak.upgrade().unwrap();
                let nested_calls = Rc::clone(&nested_result);
                let nested = runtime.register_effect(Effect::new(move || {
                    nested_calls.set(nested_calls.get() + 1);
                    None
                }));
                runtime.track_signal(SignalId::new());
                runtime.unregister_effect(nested);
                None
            });
            let outer = runtime.register_effect(outer);
            runtime.unregister_effect(outer);
            let later = runtime.register_effect(Effect::new(|| None));
            runtime.unregister_effect(later);
            send.send(nested_calls.get()).unwrap();
        });
        assert_eq!(receive.recv_timeout(Duration::from_secs(30)).unwrap(), 1);
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
            assert_eq!(
                *runtime.batch_depth.borrow(),
                1,
                "batch depth tracks the open batch"
            );
            // Multiple signal changes in a batch
            for _ in 0..5 {
                // Would trigger effects, but they're batched
                runtime.signal_changed(SignalId::new());
            }
        });

        // The batch closed and left no queued work behind
        assert_eq!(*runtime.batch_depth.borrow(), 0);
        assert!(runtime.pending_effects.borrow().is_empty());
        assert!(runtime.effect_queue.borrow().is_empty());
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

    /// FFI-006, SIG-004: a run that changes a signal it read makes the
    /// effect run again when it returns, and that run sees the result.
    #[test]
    fn ffi_006_a_run_that_changes_its_own_signal_runs_again() {
        let ctx = RuntimeContext::new();
        let signal = ctx.create_signal(0);
        let seen = Rc::new(RefCell::new(Vec::new()));
        let log = Rc::clone(&seen);
        let read = signal.clone();
        ctx.create_effect(move || {
            let value = read.get();
            log.borrow_mut().push(value);
            if value == 1 {
                read.set(2);
            }
            None
        });
        signal.set(1);
        assert_eq!(
            *seen.borrow(),
            vec![0, 1, 2],
            "the effect's runs saw {:?}, not the final value 2 last",
            seen.borrow()
        );
        assert_eq!(signal.get(), 2);
    }

    /// FFI-006: an effect whose every run changes the signal it read stops
    /// after RERUN_LIMIT reruns instead of running forever, and runs again
    /// on the next change from outside.
    #[test]
    fn ffi_006_an_effect_that_never_settles_stops_at_the_limit() {
        let ctx = RuntimeContext::new();
        let signal = ctx.create_signal(0u64);
        let runs = Rc::new(Cell::new(0usize));
        let counted = Rc::clone(&runs);
        let read = signal.clone();
        ctx.create_effect(move || {
            counted.set(counted.get() + 1);
            let value = read.get();
            read.set(value + 1);
            None
        });
        assert_eq!(
            runs.get(),
            RERUN_LIMIT + 1,
            "creation ran the effect {} times (expected the first run and {RERUN_LIMIT} reruns)",
            runs.get()
        );
        signal.set(1_000);
        assert_eq!(runs.get(), 2 * (RERUN_LIMIT + 1));
        assert_eq!(signal.get(), 1_000 + RERUN_LIMIT as u64 + 1);
    }

    /// FFI-006: an unregistered effect leaves no dependency record behind,
    /// so a signal and an effect made and removed over and over do not
    /// grow the runtime.
    #[test]
    fn ffi_006_unregistered_effects_leave_no_dependency_records() {
        let ctx = RuntimeContext::new();
        for round in 0..50 {
            let signal = ctx.create_signal(round);
            let read = signal.clone();
            let effect = ctx.create_effect(move || {
                let _ = read.get();
                None
            });
            ctx.runtime().unregister_effect(effect);
            drop(signal);
            let records = ctx.runtime().signal_effects.borrow().len();
            assert_eq!(
                records, 0,
                "round {round}: {records} signals still list a dependent after the effect was unregistered"
            );
        }
    }

    /// FFI-006: a run that no longer reads a signal takes the effect off
    /// that signal's dependents, so the records follow the last run.
    #[test]
    fn ffi_006_a_run_forgets_the_signals_it_no_longer_reads() {
        let ctx = RuntimeContext::new();
        let first = ctx.create_signal(1);
        let second = ctx.create_signal(2);
        let which = ctx.create_signal(true);
        let (read_first, read_second, read_which) = (first.clone(), second.clone(), which.clone());
        ctx.create_effect(move || {
            if read_which.get() {
                let _ = read_first.get();
            } else {
                let _ = read_second.get();
            }
            None
        });
        let listed = |signal: SignalId| {
            ctx.runtime()
                .signal_effects
                .borrow()
                .get(&signal)
                .map_or(0, |dependents| dependents.len())
        };
        assert_eq!((listed(first.id()), listed(second.id())), (1, 0));
        which.set(false);
        assert_eq!(
            (listed(first.id()), listed(second.id())),
            (0, 1),
            "after the run that read only the second signal, the first still lists the effect"
        );
    }
}
