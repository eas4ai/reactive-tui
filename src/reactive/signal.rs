use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::task::Waker;
use std::fmt::Debug;
use std::hash::Hash;
use std::collections::HashSet;

/// Unique identifier for a signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(usize);

impl SignalId {
    pub(crate) fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// A reactive signal that notifies subscribers when its value changes
pub struct Signal<T> {
    id: SignalId,
    inner: Rc<RefCell<SignalInner<T>>>,
}

struct SignalInner<T> {
    value: T,
    version: usize,
    subscribers: Vec<Weak<RefCell<dyn Subscriber>>>,
    wakers: Vec<Waker>,
}

/// Trait for objects that can subscribe to signal changes
pub trait Subscriber {
    fn notify(&mut self, signal_id: SignalId);
}

impl<T> Signal<T> {
    /// Create a new signal with an initial value
    pub fn new(value: T) -> Self {
        Self {
            id: SignalId::new(),
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
        T: Clone
    {
        self.inner.borrow().value.clone()
    }
    
    /// Get a reference to the current value
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let inner = self.inner.borrow();
        f(&inner.value)
    }
    
    /// Set a new value for the signal
    pub fn set(&self, value: T)
    where
        T: PartialEq
    {
        let mut inner = self.inner.borrow_mut();
        if inner.value != value {
            inner.value = value;
            inner.version += 1;
            
            // Notify all subscribers
            inner.subscribers.retain(|weak| {
                if let Some(subscriber) = weak.upgrade() {
                    subscriber.borrow_mut().notify(self.id);
                    true
                } else {
                    false
                }
            });
            
            // Wake all async tasks
            for waker in inner.wakers.drain(..) {
                waker.wake();
            }
        }
    }
    
    /// Update the signal value using a function
    pub fn update(&self, f: impl FnOnce(&mut T))
    where
        T: PartialEq + Clone
    {
        let mut inner = self.inner.borrow_mut();
        let old_value = inner.value.clone();
        f(&mut inner.value);
        
        if inner.value != old_value {
            inner.version += 1;
            
            // Notify all subscribers
            inner.subscribers.retain(|weak| {
                if let Some(subscriber) = weak.upgrade() {
                    subscriber.borrow_mut().notify(self.id);
                    true
                } else {
                    false
                }
            });
            
            // Wake all async tasks
            for waker in inner.wakers.drain(..) {
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
        T: Clone
    {
        (
            ReadSignal { signal: self.clone() },
            WriteSignal { signal: self.clone() }
        )
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: Rc::clone(&self.inner),
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
        T: Clone
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
        Self { signal: self.signal.clone() }
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
        T: PartialEq
    {
        self.signal.set(value);
    }
    
    /// Update the value with a function
    pub fn update(&self, f: impl FnOnce(&mut T))
    where
        T: PartialEq + Clone
    {
        self.signal.update(f);
    }
}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        Self { signal: self.signal.clone() }
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
        T: Clone + PartialEq + 'static
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
        T: PartialEq
    {
        let new_value = (self.compute.borrow())();
        self.signal.set(new_value);
    }
    
    /// Get the current computed value
    pub fn get(&self) -> T
    where
        T: Clone
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