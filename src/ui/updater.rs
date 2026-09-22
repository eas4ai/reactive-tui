//! Owned refresh callbacks dispatched by App.
use crate::{app::AppWaker, error::Result};
use std::sync::{Arc, Mutex, Weak};

/// A registered refresh callback. Empty legacy implementations must implement `update`.
/// App invokes this method on its own thread, without holding a request lock.
pub trait Updater: Send + Sync {
    /// Update the state consumed by the intended component, or return an App error.
    fn update(&mut self) -> Result<()>;
}

#[derive(Default)]
struct State {
    pending: bool,
    closed: bool,
}

/// Cloneable request address. It does not keep the updater or App alive.
#[derive(Clone)]
pub struct UpdateHandle {
    state: Weak<Mutex<State>>,
    wake: AppWaker,
}

impl UpdateHandle {
    /// Request a refresh. Repeated pending requests coalesce. Returns false after removal.
    pub fn request(&self) -> bool {
        let Some(state) = self.state.upgrade() else {
            return false;
        };
        let mut state = state.lock().unwrap();
        if state.closed {
            return false;
        }
        let notify = !state.pending;
        state.pending = true;
        drop(state);
        if notify {
            self.wake.wake();
        }
        true
    }
}

/// Keep this token alive while refresh delivery is wanted.
/// Dropping it cancels queued work; App releases the callback on its next turn.
/// An already executing callback is allowed to finish.
pub struct UpdateRegistration {
    handle: UpdateHandle,
}

impl UpdateRegistration {
    /// Obtain a weak request address for a worker or event handler.
    pub fn handle(&self) -> UpdateHandle {
        self.handle.clone()
    }
}

impl Drop for UpdateRegistration {
    fn drop(&mut self) {
        if let Some(state) = self.handle.state.upgrade() {
            let mut state = state.lock().unwrap();
            state.closed = true;
            state.pending = false;
            drop(state);
            self.handle.wake.wake();
        }
    }
}

struct Entry {
    state: Arc<Mutex<State>>,
    updater: Box<dyn Updater>,
}

#[derive(Default)]
pub(crate) struct UpdateRegistry {
    entries: Vec<Entry>,
}

impl UpdateRegistry {
    pub(crate) fn register(
        &mut self,
        updater: impl Updater + 'static,
        wake: AppWaker,
    ) -> UpdateRegistration {
        self.entries
            .retain(|entry| !entry.state.lock().unwrap().closed);
        let state = Arc::new(Mutex::new(State::default()));
        let handle = UpdateHandle {
            state: Arc::downgrade(&state),
            wake,
        };
        self.entries.push(Entry {
            state,
            updater: Box::new(updater),
        });
        UpdateRegistration { handle }
    }

    pub(crate) fn dispatch(&mut self) -> Result<bool> {
        self.entries
            .retain(|entry| !entry.state.lock().unwrap().closed);
        // Snapshot every request before calling user code: reentry belongs to the next turn.
        let pending: Vec<bool> = self
            .entries
            .iter()
            .map(|entry| {
                let mut state = entry.state.lock().unwrap();
                std::mem::take(&mut state.pending)
            })
            .collect();
        let mut changed = false;
        for (entry, pending) in self.entries.iter_mut().zip(pending) {
            if pending && !entry.state.lock().unwrap().closed {
                entry.updater.update()?;
                changed = true;
            }
        }
        Ok(changed)
    }

    pub(crate) fn close(&mut self) {
        // Invalidate all handles before user destructors can request another callback.
        for entry in &self.entries {
            let mut state = entry.state.lock().unwrap();
            state.closed = true;
            state.pending = false;
        }
        self.entries.clear();
    }
}

impl Drop for UpdateRegistry {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Callback(Box<dyn FnMut() -> Result<()> + Send + Sync>);
    impl Updater for Callback {
        fn update(&mut self) -> Result<()> {
            (self.0)()
        }
    }
    #[test]
    fn requests_coalesce_order_and_reentry_waits_for_next_batch() {
        let mut registry = UpdateRegistry::default();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let later = Arc::new(Mutex::new(None::<UpdateHandle>));
        let log = calls.clone();
        let address = later.clone();
        let first = registry.register(
            Callback(Box::new(move || {
                log.lock().unwrap().push(1);
                address.lock().unwrap().as_ref().unwrap().request();
                Ok(())
            })),
            AppWaker::new(),
        );
        let log = calls.clone();
        let second = registry.register(
            Callback(Box::new(move || {
                log.lock().unwrap().push(2);
                Ok(())
            })),
            AppWaker::new(),
        );
        *later.lock().unwrap() = Some(second.handle());
        for _ in 0..1000 {
            assert!(first.handle().request());
        }
        assert!(registry.dispatch().unwrap());
        assert_eq!(*calls.lock().unwrap(), vec![1]);
        assert!(registry.dispatch().unwrap());
        assert_eq!(*calls.lock().unwrap(), vec![1, 2]);
        second.handle().request();
        first.handle().request();
        registry.dispatch().unwrap();
        assert_eq!(*calls.lock().unwrap(), vec![1, 2, 1, 2]);
        registry.dispatch().unwrap();
        assert_eq!(*calls.lock().unwrap(), vec![1, 2, 1, 2, 2]);
        assert!(!registry.dispatch().unwrap());
    }
    #[test]
    fn cancellation_and_owner_close_invalidate_escaped_handles() {
        let mut registry = UpdateRegistry::default();
        let token = registry.register(
            Callback(Box::new(|| panic!("cancelled callback ran"))),
            AppWaker::new(),
        );
        let handle = token.handle();
        handle.request();
        drop(token);
        assert!(!handle.request());
        assert!(!registry.dispatch().unwrap());
        let token = registry.register(Callback(Box::new(|| Ok(()))), AppWaker::new());
        let handle = token.handle();
        registry.close();
        assert!(!handle.request());
    }
}
