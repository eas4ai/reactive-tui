use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::{Rc, Weak};
use std::task::Waker;

/// Unique identifier for a signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(usize);

impl SignalId {
    /// Create a new unique signal ID
    pub(crate) fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// A reactive signal that notifies subscribers when its value changes
pub struct Signal<T> {
    /// Unique identifier for this signal
    id: SignalId,
    /// Shared inner state
    inner: Rc<RefCell<SignalInner<T>>>,
    /// The memos this signal marks stale when it changes. Kept beside the
    /// value, not inside it, so a read that subscribes a memo can happen
    /// while the value is borrowed by `with` (SIG-003).
    subscribers: Rc<SubscriberList>,
    app_subscribers: Rc<super::wake::Subscriptions>,
    /// The runtime whose effects a read makes dependents of this signal and
    /// a change runs again: the one of the `RuntimeContext` that created the
    /// signal (SIG-004).
    runtime: Option<Weak<super::runtime::ReactiveRuntime>>,
}

thread_local! {
    /// The memos whose compute functions run on this thread, the innermost
    /// last. A signal read while one runs subscribes it (SIG-003).
    static COMPUTING: RefCell<Vec<Weak<RefCell<dyn Subscriber>>>> =
        const { RefCell::new(Vec::new()) };
}

/// The subscribers of a signal: weak, so a dropped memo falls out of the list.
pub(crate) type SubscriberList = RefCell<Vec<Weak<RefCell<dyn Subscriber>>>>;

/// Internal signal state
struct SignalInner<T> {
    /// Current value of the signal
    value: T,
    /// Version counter for change tracking
    version: usize,
    /// Async wakers for futures
    wakers: Vec<Waker>,
}

/// Trait for objects that can subscribe to signal changes
pub trait Subscriber {
    /// Notify the subscriber that a signal has changed
    fn notify(&mut self, signal_id: SignalId);
}

impl<T> Signal<T> {
    /// Create a new signal with an initial value
    pub fn new(value: T) -> Self {
        Self {
            id: SignalId::new(),
            app_subscribers: Rc::new(super::wake::Subscriptions::default()),
            inner: Rc::new(RefCell::new(SignalInner {
                value,
                version: 0,
                wakers: Vec::new(),
            })),
            subscribers: Rc::new(RefCell::new(Vec::new())),
            runtime: None,
        }
    }

    /// A signal of `runtime`: its effects that read the signal run again
    /// when it changes (SIG-004).
    pub(crate) fn in_runtime(value: T, runtime: Weak<super::runtime::ReactiveRuntime>) -> Self {
        Self {
            runtime: Some(runtime),
            ..Self::new(value)
        }
    }

    /// Get the signal's unique ID
    pub fn id(&self) -> SignalId {
        self.id
    }

    /// Get the current value of the signal
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.track_read();
        self.inner.borrow().value.clone()
    }

    /// Get a reference to the current value
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.track_read();
        let inner = self.inner.borrow();
        f(&inner.value)
    }

    /// Record a read: for the App rendering on this thread, for the memo
    /// computing on this thread (SIG-003), and for the effect the signal's
    /// runtime is running (SIG-004).
    fn track_read(&self) {
        self.app_subscribers.track();
        let memo = COMPUTING.with(|computing| computing.borrow().last().cloned());
        if let Some(memo) = memo {
            // The list is borrowed only here and while a change collects
            // the subscribers to notify, never across a callback; a read
            // during that collection would find it busy and is skipped.
            if let Ok(mut subscribers) = self.subscribers.try_borrow_mut() {
                if !subscribers
                    .iter()
                    .any(|subscriber| Weak::ptr_eq(subscriber, &memo))
                {
                    subscribers.push(memo);
                }
            }
        }
        if let Some(runtime) = self.runtime.as_ref().and_then(Weak::upgrade) {
            runtime.track_signal(self.id);
        }
    }

    /// The subscribers still alive, with the dropped ones taken off the list.
    fn live_subscribers(&self) -> Vec<Rc<RefCell<dyn Subscriber>>> {
        let mut subscribers = self.subscribers.borrow_mut();
        subscribers.retain(|weak| weak.strong_count() > 0);
        subscribers.iter().filter_map(Weak::upgrade).collect()
    }

    /// Run the effects of the signal's runtime whose last run read it
    /// (SIG-004).
    fn changed_in_runtime(&self) {
        if let Some(runtime) = self.runtime.as_ref().and_then(Weak::upgrade) {
            runtime.signal_changed(self.id);
        }
    }

    /// Set a new value for the signal
    pub fn set(&self, value: T)
    where
        T: PartialEq,
    {
        // Change the value and collect the wakers while holding the lock, then notify after releasing
        let (should_notify, subscribers_to_notify, wakers_to_wake) = {
            let mut inner = self.inner.borrow_mut();
            if inner.value != value {
                inner.value = value;
                inner.version += 1;

                // Collect wakers
                let wakers = inner.wakers.drain(..).collect::<Vec<_>>();

                (true, self.live_subscribers(), wakers)
            } else {
                (false, Vec::new(), Vec::new())
            }
        }; // Lock released here

        // Now notify subscribers without holding the lock
        if should_notify {
            self.app_subscribers.notify();
            for subscriber in subscribers_to_notify {
                // Use try_borrow_mut to avoid panics on re-entrant calls
                if let Ok(mut sub) = subscriber.try_borrow_mut() {
                    sub.notify(self.id);
                }
            }

            // Wake all async tasks
            for waker in wakers_to_wake {
                waker.wake();
            }

            self.changed_in_runtime();
        }
    }

    /// Update the signal value using a function
    pub fn update(&self, f: impl FnOnce(&mut T))
    where
        T: PartialEq + Clone,
    {
        // Change the value and collect the wakers while holding the lock, then notify after releasing
        let (should_notify, subscribers_to_notify, wakers_to_wake) = {
            let mut inner = self.inner.borrow_mut();
            let old_value = inner.value.clone();
            f(&mut inner.value);

            if inner.value != old_value {
                inner.version += 1;

                // Collect wakers
                let wakers = inner.wakers.drain(..).collect::<Vec<_>>();

                (true, self.live_subscribers(), wakers)
            } else {
                (false, Vec::new(), Vec::new())
            }
        }; // Lock released here

        // Now notify subscribers without holding the lock
        if should_notify {
            self.app_subscribers.notify();
            for subscriber in subscribers_to_notify {
                // Use try_borrow_mut to avoid panics on re-entrant calls
                if let Ok(mut sub) = subscriber.try_borrow_mut() {
                    sub.notify(self.id);
                }
            }

            // Wake all async tasks
            for waker in wakers_to_wake {
                waker.wake();
            }

            self.changed_in_runtime();
        }
    }

    /// Get the current version number
    pub fn version(&self) -> usize {
        self.inner.borrow().version
    }

    /// Subscribe to changes in this signal
    pub fn subscribe(&self, subscriber: Weak<RefCell<dyn Subscriber>>) {
        self.subscribers.borrow_mut().push(subscriber);
    }

    /// Register a waker to be notified on changes
    pub fn register_waker(&self, waker: Waker) {
        self.inner.borrow_mut().wakers.push(waker);
    }

    /// Create a split read/write handle
    pub fn split(&self) -> (ReadSignal<T>, WriteSignal<T>)
    where
        T: Clone,
    {
        (
            ReadSignal {
                signal: self.clone(),
            },
            WriteSignal {
                signal: self.clone(),
            },
        )
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: Rc::clone(&self.inner),
            subscribers: Rc::clone(&self.subscribers),
            app_subscribers: Rc::clone(&self.app_subscribers),
            runtime: self.runtime.clone(),
        }
    }
}

/// Read-only handle to a signal
pub struct ReadSignal<T> {
    signal: Signal<T>,
}

impl<T> ReadSignal<T> {
    /// Get the current value
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.signal.get()
    }

    /// Access the value with a function
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.signal.with(f)
    }

    /// Get the signal ID
    pub fn id(&self) -> SignalId {
        self.signal.id()
    }

    /// Get the version number
    pub fn version(&self) -> usize {
        self.signal.version()
    }
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
        }
    }
}

/// Write-only handle to a signal
pub struct WriteSignal<T> {
    signal: Signal<T>,
}

impl<T> WriteSignal<T> {
    /// Set a new value
    pub fn set(&self, value: T)
    where
        T: PartialEq,
    {
        self.signal.set(value);
    }

    /// Update the value with a function
    pub fn update(&self, f: impl FnOnce(&mut T))
    where
        T: PartialEq + Clone,
    {
        self.signal.update(f);
    }
}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
        }
    }
}

/// What a memo knows between computations: whether a `Signal` its last
/// computation read has changed since, and which memos read its value, to
/// mark stale in turn (SIG-003).
struct MemoState {
    stale: bool,
    /// The subscribers of the memo's own signal: the memos that read it.
    dependents: Weak<SubscriberList>,
}

impl Subscriber for MemoState {
    fn notify(&mut self, signal_id: SignalId) {
        if self.stale {
            // The dependents were marked when this memo was, and stay stale
            // until they compute again, which reads this memo.
            return;
        }
        self.stale = true;
        // A memo that read this one holds a value computed from the old
        // value: stale too, through any number of memos (SIG-003).
        let dependents: Vec<_> = match self.dependents.upgrade() {
            Some(dependents) => match dependents.try_borrow() {
                Ok(dependents) => dependents.clone(),
                Err(_) => return,
            },
            None => return,
        };
        for dependent in dependents.iter().filter_map(Weak::upgrade) {
            if let Ok(mut dependent) = dependent.try_borrow_mut() {
                dependent.notify(signal_id);
            }
        }
    }
}

/// Computed signal that derives its value from other signals. `get`
/// computes the value again after a `Signal` the compute function read has
/// changed (SIG-003).
#[derive(Clone)]
pub struct Memo<T> {
    signal: Signal<T>,
    compute: Rc<RefCell<Box<dyn Fn() -> T>>>,
    dependencies: Rc<RefCell<HashSet<SignalId>>>,
    state: Rc<RefCell<MemoState>>,
}

impl<T> Memo<T> {
    /// Create a new computed signal
    pub fn new(compute: impl Fn() -> T + 'static) -> Self
    where
        T: Clone + PartialEq + 'static,
    {
        let state = Rc::new(RefCell::new(MemoState {
            stale: false,
            dependents: Weak::new(),
        }));
        let initial = Self::computing(&state, &compute);
        let signal = Signal::new(initial);
        state.borrow_mut().dependents = Rc::downgrade(&signal.subscribers);
        Self {
            signal,
            compute: Rc::new(RefCell::new(Box::new(compute))),
            dependencies: Rc::new(RefCell::new(HashSet::new())),
            state,
        }
    }

    /// `compute()`, with every `Signal` it reads subscribing `state`, so a
    /// change of one marks the memo stale (SIG-003). A signal a later
    /// computation no longer reads keeps its subscription: it can only mark
    /// the memo stale once more than needed.
    fn computing(state: &Rc<RefCell<MemoState>>, compute: &dyn Fn() -> T) -> T {
        struct Computing;
        impl Drop for Computing {
            fn drop(&mut self) {
                COMPUTING.with(|computing| computing.borrow_mut().pop());
            }
        }
        let reader: Weak<RefCell<dyn Subscriber>> = Rc::<RefCell<MemoState>>::downgrade(state);
        COMPUTING.with(|computing| computing.borrow_mut().push(reader));
        let _computing = Computing;
        compute()
    }

    /// Recompute the memo's value
    fn recompute(&self)
    where
        T: PartialEq,
    {
        let compute = self.compute.borrow();
        let new_value = Self::computing(&self.state, &**compute);
        drop(compute);
        // Fresh now: the computation read the current values, even where a
        // memo it read computed again on the way and marked this one stale.
        self.state.borrow_mut().stale = false;
        self.signal.set(new_value);
    }

    /// The computed value, computed again first when a `Signal` the compute
    /// function read has changed (SIG-003).
    pub fn get(&self) -> T
    where
        T: Clone + PartialEq,
    {
        if self.state.borrow().stale {
            self.recompute();
        }
        self.signal.get()
    }

    /// Track a dependency
    pub fn track_dependency(&self, signal_id: SignalId) {
        self.dependencies.borrow_mut().insert(signal_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_basic() {
        let signal = Signal::new(5);
        assert_eq!(signal.get(), 5);

        signal.set(10);
        assert_eq!(signal.get(), 10);
        assert_eq!(signal.version(), 1);

        // Setting same value doesn't increment version
        signal.set(10);
        assert_eq!(signal.version(), 1);
    }

    #[test]
    fn test_signal_update() {
        let signal = Signal::new(vec![1, 2, 3]);

        signal.update(|v| v.push(4));
        assert_eq!(signal.get(), vec![1, 2, 3, 4]);
        assert_eq!(signal.version(), 1);
    }

    #[test]
    fn test_signal_split() {
        let signal = Signal::new(0);
        let (read, write) = signal.split();

        assert_eq!(read.get(), 0);
        write.set(42);
        assert_eq!(read.get(), 42);
    }

    #[test]
    fn test_memo_basic() {
        let count = Signal::new(1);
        let count_clone = count.clone();

        let doubled = Memo::new(move || count_clone.get() * 2);
        assert_eq!(doubled.get(), 2);

        // The next get computes again after the source changed (SIG-003)
        count.set(5);
        assert_eq!(doubled.get(), 10);
    }
}
