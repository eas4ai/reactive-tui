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
    app_subscribers: Rc<super::wake::Subscriptions>,
}

/// Internal signal state
struct SignalInner<T> {
    /// Current value of the signal
    value: T,
    /// Version counter for change tracking
    version: usize,
    /// Weak references to subscribers
    subscribers: Vec<Weak<RefCell<dyn Subscriber>>>,
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
                subscribers: Vec::new(),
                wakers: Vec::new(),
            })),
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
        self.app_subscribers.track();
        self.inner.borrow().value.clone()
    }

    /// Get a reference to the current value
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.app_subscribers.track();
        let inner = self.inner.borrow();
        f(&inner.value)
    }

    /// Set a new value for the signal
    pub fn set(&self, value: T)
    where
        T: PartialEq,
    {
        // Collect subscribers and wakers while holding the lock, then notify after releasing
        let (should_notify, subscribers_to_notify, wakers_to_wake) = {
            let mut inner = self.inner.borrow_mut();
            if inner.value != value {
                inner.value = value;
                inner.version += 1;

                // Collect valid subscribers
                let mut valid_subscribers = Vec::new();
                let mut new_subscribers = Vec::new();

                for weak in inner.subscribers.drain(..) {
                    if let Some(subscriber) = weak.upgrade() {
                        valid_subscribers.push(subscriber);
                        new_subscribers.push(weak);
                    }
                }

                inner.subscribers = new_subscribers;

                // Collect wakers
                let wakers = inner.wakers.drain(..).collect::<Vec<_>>();

                (true, valid_subscribers, wakers)
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
        }
    }

    /// Update the signal value using a function
    pub fn update(&self, f: impl FnOnce(&mut T))
    where
        T: PartialEq + Clone,
    {
        // Collect subscribers and wakers while holding the lock, then notify after releasing
        let (should_notify, subscribers_to_notify, wakers_to_wake) = {
            let mut inner = self.inner.borrow_mut();
            let old_value = inner.value.clone();
            f(&mut inner.value);

            if inner.value != old_value {
                inner.version += 1;

                // Collect valid subscribers
                let mut valid_subscribers = Vec::new();
                let mut new_subscribers = Vec::new();

                for weak in inner.subscribers.drain(..) {
                    if let Some(subscriber) = weak.upgrade() {
                        valid_subscribers.push(subscriber);
                        new_subscribers.push(weak);
                    }
                }

                inner.subscribers = new_subscribers;

                // Collect wakers
                let wakers = inner.wakers.drain(..).collect::<Vec<_>>();

                (true, valid_subscribers, wakers)
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
        }
    }

    /// Get the current version number
    pub fn version(&self) -> usize {
        self.inner.borrow().version
    }

    /// Subscribe to changes in this signal
    pub fn subscribe(&self, subscriber: Weak<RefCell<dyn Subscriber>>) {
        self.inner.borrow_mut().subscribers.push(subscriber);
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
            app_subscribers: Rc::clone(&self.app_subscribers),
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

/// Computed signal that derives its value from other signals
#[derive(Clone)]
pub struct Memo<T> {
    signal: Signal<T>,
    compute: Rc<RefCell<Box<dyn Fn() -> T>>>,
    dependencies: Rc<RefCell<HashSet<SignalId>>>,
}

impl<T> Memo<T> {
    /// Create a new computed signal
    pub fn new(compute: impl Fn() -> T + 'static) -> Self
    where
        T: Clone + PartialEq + 'static,
    {
        let initial = compute();
        let memo = Self {
            signal: Signal::new(initial),
            compute: Rc::new(RefCell::new(Box::new(compute))),
            dependencies: Rc::new(RefCell::new(HashSet::new())),
        };
        memo.recompute();
        memo
    }

    /// Recompute the memo's value
    fn recompute(&self)
    where
        T: PartialEq,
    {
        let new_value = (self.compute.borrow())();
        self.signal.set(new_value);
    }

    /// Get the current computed value
    pub fn get(&self) -> T
    where
        T: Clone,
    {
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

        count.set(5);
        doubled.recompute();
        assert_eq!(doubled.get(), 10);
    }
}
