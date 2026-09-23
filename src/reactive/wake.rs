//! Per-application notifications and weak subscriptions for rendered signals.
use std::cell::RefCell;
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::task::Waker;
use std::time::Duration;

const REDRAW: u8 = 1;
const WORK: u8 = 2;
const STOP: u8 = 4;

#[derive(Default)]
struct State {
    pending: u8,
    closed: bool,
    generation: u64,
    task: Option<Waker>,
}
#[derive(Default)]
struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}

/// A thread-safe handle to wake one App. Repeated requests coalesce.
/// Handles become inert when their App exits; they never own the App itself.
#[derive(Clone, Default)]
pub struct AppWaker {
    shared: Arc<Shared>,
}

impl AppWaker {
    /// Create a notification source, also useful for custom backend waits.
    pub fn new() -> Self {
        Self::default()
    }
    /// Request a redraw of the latest application state.
    pub fn request_redraw(&self) {
        self.notify(REDRAW);
    }
    /// Wake App to inspect queued work or changed timer deadlines.
    pub fn wake(&self) {
        self.notify(WORK);
    }
    /// Wake App and request graceful termination.
    pub fn request_stop(&self) {
        self.notify(STOP);
    }
    /// Whether a request is waiting to be processed.
    pub fn is_pending(&self) -> bool {
        self.shared.state.lock().unwrap().pending != 0
    }
    /// Whether the owning application has finished.
    pub fn is_closed(&self) -> bool {
        self.shared.state.lock().unwrap().closed
    }

    fn notify(&self, flags: u8) {
        let task = {
            let mut state = self.shared.state.lock().unwrap();
            if state.closed {
                return;
            }
            state.pending |= flags;
            state.task.take()
        };
        self.shared.changed.notify_all();
        if let Some(task) = task {
            task.wake();
        }
    }

    /// Wait for a request without consuming it. None waits without a deadline.
    /// Custom backends can combine this with their own nonblocking input polls.
    pub fn wait(&self, timeout: Option<Duration>) {
        let state = self.shared.state.lock().unwrap();
        let idle = |state: &mut State| state.pending == 0 && !state.closed;
        match timeout {
            Some(duration) => {
                drop(
                    self.shared
                        .changed
                        .wait_timeout_while(state, duration, idle)
                        .unwrap(),
                );
            }
            None => {
                drop(self.shared.changed.wait_while(state, idle).unwrap());
            }
        }
    }

    pub(crate) fn register(&self, task: &Waker) {
        let mut state = self.shared.state.lock().unwrap();
        state.task = Some(task.clone());
    }
    pub(crate) fn unregister(&self) {
        self.shared.state.lock().unwrap().task = None;
    }
    pub(crate) fn take(&self) -> Requests {
        let mut state = self.shared.state.lock().unwrap();
        let flags = std::mem::take(&mut state.pending);
        Requests {
            redraw: flags & REDRAW != 0,
            stop: flags & STOP != 0,
        }
    }
    pub(crate) fn close(&self) {
        let task = {
            let mut state = self.shared.state.lock().unwrap();
            state.closed = true;
            state.pending = 0;
            state.task.take()
        };
        self.shared.changed.notify_all();
        if let Some(task) = task {
            task.wake();
        }
    }
}

pub(crate) struct Requests {
    pub redraw: bool,
    pub stop: bool,
}

#[derive(Clone)]
struct Context {
    wake: AppWaker,
    generation: u64,
}
thread_local! { static CURRENT: RefCell<Option<Context>> = const { RefCell::new(None) }; }

/// Scoped context restores the previous App even during unwinding.
pub(crate) struct Scope(Option<Context>);
impl Scope {
    pub(crate) fn enter(wake: &AppWaker) -> Self {
        let generation = {
            let mut state = wake.shared.state.lock().unwrap();
            state.generation = state.generation.wrapping_add(1);
            state.generation
        };
        Self(CURRENT.with(|current| {
            current.replace(Some(Context {
                wake: wake.clone(),
                generation,
            }))
        }))
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        CURRENT.with(|current| {
            current.replace(self.0.take());
        });
    }
}

#[derive(Default)]
pub(crate) struct Subscriptions(Mutex<Vec<(Weak<Shared>, u64)>>);
impl Subscriptions {
    /// Call while holding the value lock, before reading the signal.
    pub(crate) fn track(&self) {
        let context = CURRENT.with(|current| current.borrow().clone());
        let Some(Context { wake, generation }) = context else {
            return;
        };
        let weak = Arc::downgrade(&wake.shared);
        let mut entries = self.0.lock().unwrap();
        entries.retain(|(entry, _)| entry.strong_count() > 0);
        if let Some(entry) = entries.iter_mut().find(|(entry, _)| entry.ptr_eq(&weak)) {
            entry.1 = generation;
        } else {
            entries.push((weak, generation));
        }
    }
    /// Notify after releasing the value lock. No user callbacks run under locks.
    pub(crate) fn notify(&self) {
        self.notify_except(None);
    }

    /// Publishing a completed frame must not schedule that same App again.
    pub(crate) fn notify_except(&self, excluded: Option<&AppWaker>) {
        let mut targets = Vec::new();
        self.0.lock().unwrap().retain(|(weak, generation)| {
            let Some(shared) = weak.upgrade() else {
                return false;
            };
            let state = shared.state.lock().unwrap();
            if state.closed || state.generation != *generation {
                return false;
            }
            drop(state);
            if !excluded.is_some_and(|wake| Arc::ptr_eq(&wake.shared, &shared)) {
                targets.push(AppWaker { shared });
            }
            true
        });
        for target in targets {
            target.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    #[test]
    fn requests_coalesce_and_survive_entry_to_wait() {
        let wake = AppWaker::new();
        for _ in 0..10_000 {
            wake.request_redraw();
        }
        let before_wait = Instant::now();
        wake.wait(Some(Duration::from_secs(1)));
        assert!(
            before_wait.elapsed() < Duration::from_millis(500),
            "pending wake must not wait for the timeout"
        );
        assert!(wake.take().redraw);
        assert!(!wake.take().redraw);
        let other = wake.clone();
        let sender = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            other.request_stop();
        });
        let start = Instant::now();
        wake.wait(Some(Duration::from_secs(2)));
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(wake.take().stop);
        sender.join().unwrap();
        wake.close();
        wake.request_redraw();
        assert!(!wake.is_pending());
    }
    #[test]
    fn signal_subscriptions_are_scoped_weak_and_ignore_equal_values() {
        use crate::reactive::{Signal, ThreadSafeSignal};
        let first = AppWaker::new();
        let second = AppWaker::new();
        let signal = ThreadSafeSignal::new(0);
        let local = Signal::new(0);
        {
            let _scope = Scope::enter(&first);
            assert_eq!(signal.get(), 0);
            assert_eq!(local.with(|v| *v), 0);
        }
        signal.set(0);
        local.update(|v| *v = 0);
        assert!(!first.is_pending());
        signal.set(1);
        assert!(first.take().redraw);
        assert!(!second.is_pending());
        local.set(2);
        assert!(first.take().redraw);
        {
            let _scope = Scope::enter(&second);
            signal.get();
        }
        signal.update(|v| *v += 1);
        assert!(first.take().redraw);
        assert!(second.take().redraw);
        // A subsequent render that no longer reads the signal drops that dependency.
        drop(Scope::enter(&first));
        signal.set(3);
        assert!(!first.is_pending());
        assert!(second.take().redraw);
        second.close();
        signal.set(4);
        assert!(!second.is_pending());
        let weak = Arc::downgrade(&first.shared);
        drop(first);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn notifications_racing_wait_registration_are_not_lost() {
        use std::sync::mpsc;
        let wake = AppWaker::new();
        let worker_wake = wake.clone();
        let (send, receive) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            for _ in 0..200 {
                receive.recv().unwrap();
                worker_wake.request_redraw();
            }
        });
        for _ in 0..200 {
            send.send(()).unwrap();
            wake.wait(Some(Duration::from_secs(1)));
            assert!(wake.take().redraw);
        }
        worker.join().unwrap();
    }

    #[test]
    fn completed_metrics_notify_other_apps_and_preserve_owner_subscription() {
        use crate::reactive::ThreadSafeSignal;
        let owner = AppWaker::new();
        let observer = AppWaker::new();
        let value = ThreadSafeSignal::new(0);
        for app in [&owner, &observer] {
            let _scope = Scope::enter(app);
            value.get();
        }
        value.set_except(1, Some(&owner));
        assert!(!owner.is_pending());
        assert!(observer.take().redraw);
        value.set(2);
        assert!(owner.take().redraw);
        assert!(observer.take().redraw);
        value.set_except(2, Some(&owner));
        assert!(!owner.is_pending());
        assert!(!observer.is_pending());
    }
}
