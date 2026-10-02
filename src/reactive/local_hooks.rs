//! Explicit ownership for retained, thread-confined hook values.

use super::hooks::{HookKind, HookResources, Hooks};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak as LocalWeak};
use std::sync::{Arc, Weak};

thread_local! {
    // Bindings do not own arenas; the synchronous scope guard does.
    static SCOPES: RefCell<Vec<LocalWeak<Arena>>> = const { RefCell::new(Vec::new()) };
}

#[derive(Default)]
struct Arena {
    identity: Arc<()>,
    entries: RefCell<HashMap<usize, Entry>>,
}

struct Entry {
    _identity: Arc<()>,
    owner: Weak<HookResources>,
    value: Box<dyn Any>,
}

struct LocalSlot {
    identity: Arc<()>,
    value_type: TypeId,
}

struct Scope(Rc<Arena>);

impl Drop for Scope {
    fn drop(&mut self) {
        SCOPES.with(|scopes| {
            scopes.borrow_mut().pop();
        });
        // Remove the binding and release the borrow before running destructors.
        let entries = std::mem::take(&mut *self.0.entries.borrow_mut());
        drop(entries);
    }
}

/// Own retained local-hook values for a sequence of synchronous renders.
///
/// `App::run` supplies this scope automatically. Manual hook or component renders
/// using `use_local_ref` must wrap their related renders once. Local slots cannot
/// move to a different thread or scope. Ending the scope, including on unwinding,
/// releases its retained shares; escaped LocalRef handles keep their own shares.
/// Same-thread Hooks cleanup releases its local entries immediately. Cleanup on
/// another thread is reclaimed at the creator's next local-hook use or scope exit.
/// Nested explicit calls create independent scopes.
pub fn with_local_hooks<R>(body: impl FnOnce() -> R) -> R {
    sweep();
    let scope = Scope(Rc::new(Arena::default()));
    SCOPES.with(|scopes| scopes.borrow_mut().push(Rc::downgrade(&scope.0)));
    body()
}

pub(crate) fn with_current_scope<R>(body: impl FnOnce() -> R) -> R {
    if current().is_some() {
        sweep();
        body()
    } else {
        with_local_hooks(body)
    }
}

fn current() -> Option<Rc<Arena>> {
    SCOPES
        .try_with(|scopes| scopes.borrow().last().and_then(LocalWeak::upgrade))
        .ok()
        .flatten()
}

pub(crate) fn sweep() {
    let arenas = SCOPES
        .try_with(|scopes| {
            scopes
                .borrow()
                .iter()
                .filter_map(LocalWeak::upgrade)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for arena in arenas {
        // Upgrading the final owner can transfer its destructor to this thread.
        // Keep upgrades alive until every arena borrow has been released.
        let mut owners = Vec::new();
        let removed = {
            let mut entries = arena.entries.borrow_mut();
            let dead = entries
                .iter()
                .filter_map(|(key, entry)| {
                    let alive = entry.owner.upgrade().is_some_and(|owner| {
                        let alive = owner.is_alive();
                        owners.push(owner);
                        alive
                    });
                    (!alive).then_some(*key)
                })
                .collect::<Vec<_>>();
            dead.into_iter()
                .filter_map(|key| entries.remove(&key))
                .collect::<Vec<_>>()
        };
        drop(removed);
        drop(owners);
    }
}

pub(crate) fn storage<T: 'static>(hooks: &Hooks, initial: T) -> Rc<RefCell<T>> {
    let arena = current().expect("use_local_ref requires a with_local_hooks scope");
    hooks.bind_local_scope(&arena.identity);
    sweep();
    let slot = hooks.get_or_create_storage(HookKind::LocalRef, || LocalSlot {
        identity: Arc::new(()),
        value_type: TypeId::of::<T>(),
    });
    let (identity, value_type) = {
        let slot = slot.lock().expect("local hook slot lock poisoned");
        (Arc::clone(&slot.identity), slot.value_type)
    };
    assert_eq!(
        value_type,
        TypeId::of::<T>(),
        "local hook value type changed"
    );
    let key = Arc::as_ptr(&identity) as usize;
    let existing = {
        let entries = arena.entries.borrow();
        entries.get(&key).map(|entry| {
            Rc::clone(
                entry
                    .value
                    .downcast_ref::<Rc<RefCell<T>>>()
                    .expect("local hook arena type disagrees with its slot"),
            )
        })
    };
    if let Some(value) = existing {
        return value;
    }
    let value = Rc::new(RefCell::new(initial));
    let entry = Entry {
        _identity: identity,
        owner: hooks.resource_token(),
        value: Box::new(Rc::clone(&value)),
    };
    let replaced = arena.entries.borrow_mut().insert(key, entry);
    debug_assert!(replaced.is_none());
    drop(replaced);
    value
}
