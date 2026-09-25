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
    closed: Arc<AtomicBool>,
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

    /// A handle that says whether these resources are still open.
    pub(crate) fn liveness(&self) -> Liveness {
        Liveness(Arc::clone(&self.closed))
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
            super::local_hooks::sweep();
            let effects = std::mem::take(&mut *self.effects.lock().unwrap());
            for effect in effects {
                effect.dispose();
            }
        }
    }
}

/// Whether a hook owner is still open, read without holding the owner. A
/// check through a `Weak<HookResources>` holds a strong reference while it
/// runs; if that is the last one, dropping it closes the owner on the
/// checking thread, whose cleanup may then wait for a lock that thread holds.
#[derive(Clone)]
pub(crate) struct Liveness(Arc<AtomicBool>);

impl Liveness {
    pub(crate) fn is_alive(&self) -> bool {
        !self.0.load(Ordering::Acquire)
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

#[cfg(test)]
mod tests {
    use syn::visit::Visit;

    /// The animation, timer, throttle and clipboard hooks check their owner
    /// through `Liveness`. Getting a `Weak<HookResources>` needs
    /// `Hooks::resource_token` or the type's name, so a hook that goes back
    /// to upgrading its owner shows up here on every run, where the race
    /// tests catch it only when a drop lands inside the check.
    #[test]
    fn hooks_check_their_owner_through_liveness_never_a_weak_upgrade() {
        struct OwnerUpgrades<'a> {
            file: &'a str,
            violations: Vec<String>,
        }
        impl<'ast> Visit<'ast> for OwnerUpgrades<'_> {
            fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
                let last = path.path.segments.last();
                if let Some(syn::PathSegment {
                    ident,
                    arguments: syn::PathArguments::AngleBracketed(arguments),
                }) = last
                {
                    let names_resources = arguments.args.iter().any(|argument| match argument {
                        syn::GenericArgument::Type(syn::Type::Path(inner)) => inner
                            .path
                            .segments
                            .last()
                            .is_some_and(|segment| segment.ident == "HookResources"),
                        _ => false,
                    });
                    if ident == "Weak" && names_resources {
                        self.violations
                            .push(format!("{} holds a Weak<HookResources>", self.file));
                    }
                }
                syn::visit::visit_type_path(self, path);
            }

            fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
                if call.method == "resource_token" {
                    self.violations
                        .push(format!("{} calls resource_token", self.file));
                }
                syn::visit::visit_expr_method_call(self, call);
            }

            fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
                if path
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "resource_token")
                {
                    self.violations
                        .push(format!("{} names resource_token", self.file));
                }
                syn::visit::visit_expr_path(self, path);
            }
        }

        let sources = [
            ("hooks/animation.rs", include_str!("../hooks/animation.rs")),
            ("hooks/timer.rs", include_str!("../hooks/timer.rs")),
            ("hooks/clipboard.rs", include_str!("../hooks/clipboard.rs")),
        ];
        let mut upgrades = OwnerUpgrades {
            file: "",
            violations: Vec::new(),
        };
        for (file, source) in sources {
            upgrades.file = file;
            upgrades.visit_file(&syn::parse_file(source).unwrap());
        }
        assert!(
            upgrades.violations.is_empty(),
            "hooks must check their owner through Liveness, not a Weak upgrade: {:?}",
            upgrades.violations
        );
    }
}
