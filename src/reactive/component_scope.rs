//! Temporary bindings for resources owned by one App component.

use super::{hooks::HookResources, scheduler::Scheduler};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};

type ContextValues = HashMap<TypeId, Arc<dyn Any + Send + Sync>>;

pub(crate) struct ComponentScope {
    scheduler: Arc<Scheduler>,
    root: bool,
    contexts: Mutex<ContextValues>,
    hooks: Mutex<Vec<Weak<HookResources>>>,
}

impl ComponentScope {
    pub(crate) fn new(scheduler: Arc<Scheduler>) -> Arc<Self> {
        Self::create(scheduler, true)
    }

    pub(crate) fn child(scheduler: Arc<Scheduler>) -> Arc<Self> {
        Self::create(scheduler, false)
    }

    fn create(scheduler: Arc<Scheduler>, root: bool) -> Arc<Self> {
        Arc::new(Self {
            scheduler,
            root,
            contexts: Mutex::new(HashMap::new()),
            hooks: Mutex::new(Vec::new()),
        })
    }

    pub(crate) fn enter(self: &Arc<Self>, new_render: bool) -> ScopeGuard {
        if new_render {
            let inherited = if self.root {
                HashMap::new()
            } else {
                current()
                    .map(|parent| parent.contexts.lock().unwrap().clone())
                    .unwrap_or_default()
            };
            let old = std::mem::replace(&mut *self.contexts.lock().unwrap(), inherited);
            drop(old);
        }
        SCOPES.with(|scopes| scopes.borrow_mut().push(Arc::clone(self)));
        ScopeGuard {
            _thread: PhantomData,
        }
    }

    pub(crate) fn scheduler(&self) -> Arc<Scheduler> {
        Arc::clone(&self.scheduler)
    }

    pub(crate) fn track(&self, hooks: &Arc<HookResources>) {
        let mut tracked = self.hooks.lock().unwrap();
        tracked.retain(|weak| weak.strong_count() > 0);
        if !tracked
            .iter()
            .any(|weak| weak.ptr_eq(&Arc::downgrade(hooks)))
        {
            tracked.push(Arc::downgrade(hooks));
        }
    }

    pub(crate) fn lookup<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        let value = self
            .contexts
            .lock()
            .unwrap()
            .get(&TypeId::of::<T>())
            .cloned();
        value.and_then(|value| value.downcast_ref::<T>().cloned())
    }

    pub(crate) fn close(&self) {
        let hooks = std::mem::take(&mut *self.hooks.lock().unwrap());
        for hooks in hooks.into_iter().filter_map(|weak| weak.upgrade()) {
            hooks.close();
        }
    }
}

impl Drop for ComponentScope {
    fn drop(&mut self) {
        self.close();
    }
}

thread_local! {
    static SCOPES: RefCell<Vec<Arc<ComponentScope>>> = const { RefCell::new(Vec::new()) };
}

pub(crate) struct ScopeGuard {
    // A thread-local scope cannot be moved to a different thread.
    _thread: PhantomData<Rc<()>>,
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let scope = SCOPES.with(|scopes| scopes.borrow_mut().pop());
        drop(scope);
    }
}

pub(crate) fn current() -> Option<Arc<ComponentScope>> {
    SCOPES.with(|scopes| scopes.borrow().last().cloned())
}

pub(crate) fn provide<T: Send + Sync + 'static>(value: T) -> Result<(), T> {
    let Some(scope) = current() else {
        return Err(value);
    };
    let old = scope
        .contexts
        .lock()
        .unwrap()
        .insert(TypeId::of::<T>(), Arc::new(value));
    drop(old);
    Ok(())
}

pub(crate) fn lookup<T: Clone + Send + Sync + 'static>() -> Option<T> {
    current().and_then(|scope| scope.lookup())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::{provide_context, use_context, use_effect, Hooks};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn nested_app_scopes_isolate_context_and_cleanup_retained_hooks() {
        let app_a = ComponentScope::new(Arc::new(Scheduler::new()));
        let app_b = ComponentScope::new(Arc::new(Scheduler::new()));
        let child = ComponentScope::child(app_a.scheduler());
        let hooks_a = Hooks::new();
        let hooks_b = Hooks::new();
        let hooks_child = Hooks::new();
        let cleanups = Arc::new(AtomicUsize::new(0));
        {
            let _a = app_a.enter(true);
            provide_context(&hooks_a, "a".to_string());
            {
                // Separate Apps are roots, even when called from another App.
                let _b = app_b.enter(true);
                assert!(
                    use_context::<String>(&hooks_b).is_none(),
                    "another App must not inherit the caller App context"
                );
                provide_context(&hooks_b, "b".to_string());
                assert_eq!(use_context::<String>(&hooks_b).as_deref(), Some("b"));
            }
            let _child = child.enter(true);
            let _frame = hooks_child.begin_render();
            assert_eq!(use_context::<String>(&hooks_child).as_deref(), Some("a"));
            let cleanups = cleanups.clone();
            use_effect(&hooks_child, move || {
                Some(Box::new(move || {
                    cleanups.fetch_add(1, Ordering::SeqCst);
                }))
            });
        }
        assert_eq!(cleanups.load(Ordering::SeqCst), 0);
        child.close();
        assert_eq!(cleanups.load(Ordering::SeqCst), 1);
        drop(hooks_child);
        assert_eq!(cleanups.load(Ordering::SeqCst), 1);
        {
            let _a = app_a.enter(true);
            assert!(
                use_context::<String>(&hooks_a).is_none(),
                "provider from previous render must not linger"
            );
        }
    }
}
