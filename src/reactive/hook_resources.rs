//! Cleanup ownership shared by clones of a hook context.

use super::{component_scope, effect::EffectId, scheduler::Scheduler};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, Weak,
};

type Cleanup = Box<dyn FnOnce() + Send + Sync>;
type EffectBody = Box<dyn FnOnce() -> Option<Cleanup> + Send + Sync>;

#[derive(Default)]
pub(crate) struct HookResources {
    closed: AtomicBool,
    owner: Mutex<Option<Weak<component_scope::ComponentScope>>>,
    effects: Mutex<Vec<Arc<OwnedEffect>>>,
}

impl HookResources {
    pub(crate) fn bind(self: &Arc<Self>) {
        assert!(self.is_alive(), "hook resource owner has been cleaned up");
        let Some(scope) = component_scope::current() else {
            return;
        };
        let mut owner = self.owner.lock().unwrap();
        if let Some(old) = owner.as_ref().and_then(Weak::upgrade) {
            if !Arc::ptr_eq(&old, &scope) {
                drop(owner);
                panic!("a hook context cannot belong to two components");
            }
        } else {
            *owner = Some(Arc::downgrade(&scope));
        }
        drop(owner);
        scope.track(self);
    }

    pub(crate) fn scheduler(&self) -> Option<Arc<Scheduler>> {
        let owner = self.owner.lock().unwrap().as_ref().and_then(Weak::upgrade);
        owner.map(|owner| owner.scheduler())
    }

    pub(crate) fn context<T: Clone + Send + Sync + 'static>(&self) -> Option<Option<T>> {
        let owner = self.owner.lock().unwrap().as_ref().and_then(Weak::upgrade);
        owner.map(|owner| owner.lookup())
    }

    pub(crate) fn is_alive(&self) -> bool {
        !self.closed.load(Ordering::Acquire)
    }

    pub(crate) fn track(&self, effect: &Arc<OwnedEffect>) {
        let mut effects = self.effects.lock().unwrap();
        if self.is_alive() {
            if !effects.iter().any(|owned| Arc::ptr_eq(owned, effect)) {
                effects.push(Arc::clone(effect));
            }
        } else {
            drop(effects);
            effect.dispose();
        }
    }

    fn snapshot(&self) -> Vec<Arc<OwnedEffect>> {
        self.effects.lock().unwrap().clone()
    }

    pub(crate) fn flush(&self) {
        for effect in self.snapshot() {
            effect.flush();
        }
    }

    pub(crate) fn discard_pending(&self) {
        for effect in self.snapshot() {
            let pending = effect.state.lock().unwrap().pending.take();
            drop(pending);
        }
    }

    pub(crate) fn close(&self) {
        if !self.closed.swap(true, Ordering::AcqRel) {
            let effects = std::mem::take(&mut *self.effects.lock().unwrap());
            for effect in effects {
                effect.dispose();
            }
        }
    }
}

impl Drop for HookResources {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Default)]
struct EffectState {
    pending: Option<EffectBody>,
    cleanup: Option<Cleanup>,
    running: bool,
    disposed: bool,
}

struct RunningEffect<'a>(&'a Mutex<EffectState>);

impl Drop for RunningEffect<'_> {
    fn drop(&mut self) {
        self.0.lock().unwrap().running = false;
    }
}

pub(crate) struct OwnedEffect {
    pub(crate) id: EffectId,
    state: Mutex<EffectState>,
}

impl OwnedEffect {
    pub(crate) fn new() -> Self {
        Self {
            id: EffectId::new(),
            state: Mutex::new(EffectState::default()),
        }
    }

    pub(crate) fn queue(&self, effect: EffectBody) {
        let mut state = self.state.lock().unwrap();
        let old = if state.disposed {
            Some(effect)
        } else {
            state.pending.replace(effect)
        };
        drop(state);
        drop(old);
    }

    pub(crate) fn flush(&self) {
        let mut state = self.state.lock().unwrap();
        if state.disposed || state.running {
            return;
        }
        let Some(effect) = state.pending.take() else {
            return;
        };
        state.running = true;
        let cleanup = state.cleanup.take();
        drop(state);
        let _running = RunningEffect(&self.state);
        if let Some(cleanup) = cleanup {
            cleanup();
        }
        if self.state.lock().unwrap().disposed {
            return;
        }
        let mut cleanup = effect();
        let mut state = self.state.lock().unwrap();
        if !state.disposed {
            state.cleanup = cleanup.take();
        }
        drop(state);
        if let Some(cleanup) = cleanup {
            cleanup();
        }
    }

    fn dispose(&self) {
        let mut state = self.state.lock().unwrap();
        state.disposed = true;
        let pending = state.pending.take();
        let cleanup = state.cleanup.take();
        drop(state);
        drop(pending);
        if let Some(cleanup) = cleanup {
            cleanup();
        }
    }
}

impl Drop for OwnedEffect {
    fn drop(&mut self) {
        self.dispose();
    }
}
