use crate::reactive::hooks::{HookKind, Hooks};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// A reference to a value that persists across renders but doesn't trigger re-renders
///
/// Similar to React's useRef, this provides a mutable reference that:
/// - Persists for the full lifetime of the component
/// - Can be mutated without causing re-renders
/// - Is useful for storing DOM references, timers, or any mutable value
#[derive(Clone)]
pub struct Ref<T> {
    value: Arc<Mutex<T>>,
}

impl<T> Ref<T> {
    /// Create a new ref with an initial value
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(Mutex::new(initial)),
        }
    }

    /// Get the current value
    pub fn current(&self) -> T
    where
        T: Clone,
    {
        self.value.lock().unwrap().clone()
    }

    /// Set the current value
    pub fn set_current(&self, value: T) {
        *self.value.lock().unwrap() = value;
    }

    /// Update the current value with a function
    pub fn update<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.value.lock().unwrap();
        f(&mut *guard)
    }

    /// Get a raw pointer to the inner value (unsafe)
    ///
    /// This is useful for FFI or when you need to pass a raw pointer
    /// to a C library. The pointer is only valid while the Ref exists.
    /// # Safety
    /// The returned pointer is only valid as long as the RefHandle exists
    /// and the underlying value has not been moved.
    pub unsafe fn as_ptr(&self) -> *const T {
        &*self.value.lock().unwrap() as *const T
    }
}

/// A local (non-thread-safe) reference for single-threaded contexts
#[derive(Clone)]
pub struct LocalRef<T> {
    value: Rc<RefCell<T>>,
}

impl<T> LocalRef<T> {
    /// Create a new local ref with an initial value
    pub fn new(initial: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(initial)),
        }
    }

    /// Get the current value
    pub fn current(&self) -> T
    where
        T: Clone,
    {
        self.value.borrow().clone()
    }

    /// Set the current value
    pub fn set_current(&self, value: T) {
        *self.value.borrow_mut() = value;
    }

    /// Update the current value with a function
    pub fn update<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut borrow = self.value.borrow_mut();
        f(&mut *borrow)
    }
}

/// Hook for creating a mutable reference that doesn't trigger re-renders
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::hooks::use_ref;
/// use reactive_tui::reactive::Hooks;
///
/// fn remember_value(hooks: &Hooks) {
///     let value = use_ref(hooks, String::from("initial"));
///     let shared = value.clone();
///     shared.set_current("updated".into());
///     assert_eq!(value.current(), "updated");
/// }
/// ```
pub fn use_ref<T>(hooks: &Hooks, initial: T) -> Ref<T>
where
    T: Send + Sync + 'static,
{
    Ref {
        value: hooks.get_or_create_storage(HookKind::Ref, || initial),
    }
}

/// Hook for creating a local (non-thread-safe) mutable reference
///
/// Requires a live `with_local_hooks` scope, supplied automatically by App::run.
/// Retains arbitrary non-Send values on the creating thread. Related manual
/// renders must use the same scope; missing, changed or expired scopes panic.
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::hooks::{use_local_ref, with_local_hooks};
/// use reactive_tui::reactive::Hooks;
///
/// with_local_hooks(|| {
///     let hooks = Hooks::new();
///     let value = use_local_ref(&hooks, String::new());
///     value.set_current("draft".into());
///     hooks.reset();
///     assert_eq!(use_local_ref(&hooks, String::new()).current(), "draft");
/// });
/// ```
pub fn use_local_ref<T>(hooks: &Hooks, initial: T) -> LocalRef<T>
where
    T: 'static,
{
    LocalRef {
        value: crate::reactive::local_hooks::storage(hooks, initial),
    }
}

/// A callback ref that calls a function when the reference changes
///
/// Useful for accessing DOM elements or handling cleanup when refs change.
#[derive(Clone)]
pub struct CallbackRef<T> {
    current: Arc<Mutex<Option<T>>>,
    callback: Arc<Mutex<RefCallback<T>>>,
}

type RefCallback<T> = Arc<dyn Fn(Option<T>) + Send + Sync>;

impl<T: Clone + Send + 'static> CallbackRef<T> {
    /// Create a new callback ref
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(Option<T>) + Send + Sync + 'static,
    {
        Self {
            current: Arc::new(Mutex::new(None)),
            callback: Arc::new(Mutex::new(Arc::new(callback))),
        }
    }

    /// Set the reference, calling the callback with the new value
    pub fn set(&self, value: Option<T>) {
        let next = value.clone();
        let previous = std::mem::replace(&mut *self.current.lock().unwrap(), next);
        let callback = Arc::clone(&self.callback.lock().unwrap());
        // Both destructors and callbacks may reenter this reference.
        drop(previous);
        callback(value);
    }

    /// Get the current value
    pub fn current(&self) -> Option<T> {
        self.current.lock().unwrap().clone()
    }
}

/// Hook for creating a callback ref
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::hooks::use_callback_ref;
/// use reactive_tui::reactive::Hooks;
///
/// fn selected_id(hooks: &Hooks) {
///     let reference = use_callback_ref(hooks, |id: Option<String>| {
///         println!("Selected: {id:?}");
///     });
///     reference.set(Some("entry-1".into()));
///     assert_eq!(reference.current().as_deref(), Some("entry-1"));
///     reference.set(None);
/// }
/// ```
pub fn use_callback_ref<T, F>(hooks: &Hooks, callback: F) -> CallbackRef<T>
where
    T: Clone + Send + 'static,
    F: Fn(Option<T>) + Send + Sync + 'static,
{
    let callback: RefCallback<T> = Arc::new(callback);
    let storage = hooks.get_or_create_storage(HookKind::CallbackRef, || CallbackRef {
        current: Arc::new(Mutex::new(None)),
        callback: Arc::new(Mutex::new(Arc::clone(&callback))),
    });
    let reference = storage.lock().unwrap().clone();
    let previous = std::mem::replace(&mut *reference.callback.lock().unwrap(), callback);
    drop(previous);
    reference
}

/// A forwarded ref that can be passed through component props
///
/// This allows parent components to access child component internals.
#[derive(Clone)]
pub struct ForwardedRef<T> {
    inner: Option<Ref<T>>,
}

impl<T> ForwardedRef<T> {
    /// Create a new forwarded ref
    pub fn new(inner: Option<Ref<T>>) -> Self {
        Self { inner }
    }

    /// Check if the ref is set
    pub fn is_set(&self) -> bool {
        self.inner.is_some()
    }

    /// Get the inner ref if set
    pub fn get(&self) -> Option<&Ref<T>> {
        self.inner.as_ref()
    }

    /// Set a value on the forwarded ref if it exists
    pub fn set_if_exists(&self, value: T) {
        if let Some(ref inner) = self.inner {
            inner.set_current(value);
        }
    }
}

/// Hook for forwarding refs through components
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::hooks::{use_forwarded_ref, Ref};
/// use reactive_tui::reactive::Hooks;
///
/// fn update_parent(hooks: &Hooks, parent: Option<Ref<String>>) {
///     let forwarded = use_forwarded_ref(hooks, parent);
///     forwarded.set_if_exists("child value".into());
/// }
/// ```
pub fn use_forwarded_ref<T>(_hooks: &Hooks, forward_ref: Option<Ref<T>>) -> ForwardedRef<T>
where
    T: Send + Sync + 'static,
{
    // Forwarded refs don't need to be tracked by signals since they don't trigger re-renders
    ForwardedRef::new(forward_ref)
}

/// Multiple refs that can all point to the same value
///
/// Useful when multiple components need to share a reference.
#[derive(Clone)]
pub struct MultiRef<T> {
    refs: Arc<Mutex<Vec<Ref<T>>>>,
}

impl<T: Clone> Default for MultiRef<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> MultiRef<T> {
    /// Create a new multi-ref
    pub fn new() -> Self {
        Self {
            refs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a new ref to the collection
    pub fn add_ref(&self, initial: T) -> Ref<T> {
        let new_ref = Ref::new(initial);
        self.refs.lock().unwrap().push(new_ref.clone());
        new_ref
    }

    /// Update all refs with the same value
    pub fn set_all(&self, value: T) {
        let refs = self.refs.lock().unwrap();
        for ref_item in refs.iter() {
            ref_item.set_current(value.clone());
        }
    }

    /// Get the number of refs
    pub fn count(&self) -> usize {
        self.refs.lock().unwrap().len()
    }

    /// Clear all refs
    pub fn clear(&self) {
        self.refs.lock().unwrap().clear();
    }
}

/// Hook for creating multiple refs that share updates
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::hooks::use_multi_ref;
/// use reactive_tui::reactive::Hooks;
///
/// fn clear_selections(hooks: &Hooks) {
///     let selected = use_multi_ref(hooks);
///     let first = selected.add_ref(true);
///     let second = selected.add_ref(true);
///     selected.set_all(false);
///     assert!(!first.current() && !second.current());
///     selected.clear();
///     assert_eq!(selected.count(), 0);
/// }
/// ```
pub fn use_multi_ref<T>(hooks: &Hooks) -> MultiRef<T>
where
    T: Clone + Send + Sync + 'static,
{
    MultiRef {
        refs: hooks.get_or_create_storage(HookKind::MultiRef, Vec::new),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_basic() {
        let ref_value = Ref::new(42);
        assert_eq!(ref_value.current(), 42);

        ref_value.set_current(100);
        assert_eq!(ref_value.current(), 100);

        let result = ref_value.update(|v| {
            *v += 1;
            *v * 2
        });
        assert_eq!(result, 202);
        assert_eq!(ref_value.current(), 101);
    }

    #[test]
    fn test_local_ref() {
        let local_ref = LocalRef::new("hello");
        assert_eq!(local_ref.current(), "hello");

        local_ref.set_current("world");
        assert_eq!(local_ref.current(), "world");

        local_ref.update(|s| {
            *s = "updated";
        });
        assert_eq!(local_ref.current(), "updated");
    }

    #[test]
    fn test_callback_ref() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let callback_ref = CallbackRef::new(move |value: Option<i32>| {
            if value.is_some() {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }
        });

        assert_eq!(callback_ref.current(), None);
        assert_eq!(counter.load(Ordering::Relaxed), 0);

        callback_ref.set(Some(42));
        assert_eq!(callback_ref.current(), Some(42));
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        callback_ref.set(Some(100));
        assert_eq!(callback_ref.current(), Some(100));
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_forwarded_ref() {
        let base_ref = Ref::new(10);
        let forwarded = ForwardedRef::new(Some(base_ref.clone()));

        assert!(forwarded.is_set());
        forwarded.set_if_exists(20);
        assert_eq!(base_ref.current(), 20);

        let empty_forwarded = ForwardedRef::<i32>::new(None);
        assert!(!empty_forwarded.is_set());
        empty_forwarded.set_if_exists(30); // Should do nothing
    }

    #[test]
    fn test_multi_ref() {
        let multi = MultiRef::new();

        let ref1 = multi.add_ref(1);
        let ref2 = multi.add_ref(2);
        let ref3 = multi.add_ref(3);

        assert_eq!(multi.count(), 3);
        assert_eq!(ref1.current(), 1);
        assert_eq!(ref2.current(), 2);
        assert_eq!(ref3.current(), 3);

        multi.set_all(100);
        assert_eq!(ref1.current(), 100);
        assert_eq!(ref2.current(), 100);
        assert_eq!(ref3.current(), 100);

        multi.clear();
        assert_eq!(multi.count(), 0);
    }

    #[test]
    fn test_use_ref() {
        let hooks = Hooks::new();

        let counter_ref = use_ref(&hooks, 0);
        assert_eq!(counter_ref.current(), 0);

        counter_ref.set_current(10);
        assert_eq!(counter_ref.current(), 10);

        // Refs don't trigger re-renders
        counter_ref.update(|v| *v += 5);
        assert_eq!(counter_ref.current(), 15);
    }

    #[test]
    fn test_use_local_ref() {
        crate::hooks::with_local_hooks(|| {
            let hooks = Hooks::new();

            let text_ref = use_local_ref(&hooks, String::from("initial"));
            assert_eq!(text_ref.current(), "initial");

            text_ref.set_current(String::from("updated"));
            assert_eq!(text_ref.current(), "updated");

            text_ref.update(|s| s.push_str(" text"));
            assert_eq!(text_ref.current(), "updated text");
        });
    }

    #[test]
    fn test_ref_thread_safety() {
        use std::thread;

        let shared_ref = Ref::new(0);
        let mut handles = vec![];

        for i in 0..10 {
            let ref_clone = shared_ref.clone();
            let handle = thread::spawn(move || {
                ref_clone.update(|v| *v += i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Sum of 0..10 = 45
        assert_eq!(shared_ref.current(), 45);
    }
}
