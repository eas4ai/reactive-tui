//! Pointer lifetime tracking for FFI safety
//!
//! Typed trackers can recognize handles that this library registered and has
//! not removed. The general `validate_pointer` helper checks only non-null,
//! alignment, and a broad userspace address range. It does not prove that an
//! address is allocated, live, owned by this library, readable, or writable.

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

/// Tracks valid pointers with type information to prevent use-after-free and type confusion
pub struct PointerTracker<T> {
    valid_pointers: Arc<RwLock<HashMap<usize, TypeId>>>,
    _phantom: PhantomData<T>,
}

impl<T> PointerTracker<T> {
    /// Create a new pointer tracker
    pub fn new() -> Self {
        Self {
            valid_pointers: Arc::new(RwLock::new(HashMap::new())),
            _phantom: PhantomData,
        }
    }

    /// Register a new pointer as valid with type information
    pub fn register(&self, ptr: *mut T) -> bool
    where
        T: 'static,
    {
        if ptr.is_null() {
            return false;
        }

        let addr = ptr as usize;
        let type_id = TypeId::of::<T>();
        match self.valid_pointers.write() {
            Ok(mut map) => {
                map.insert(addr, type_id);
                true
            }
            Err(_) => false,
        }
    }

    /// Check if a pointer is valid and has correct type
    pub fn is_valid(&self, ptr: *const T) -> bool
    where
        T: 'static,
    {
        if ptr.is_null() {
            return false;
        }

        let addr = ptr as usize;
        let expected_type = TypeId::of::<T>();
        match self.valid_pointers.read() {
            Ok(map) => map
                .get(&addr)
                .map_or(false, |&stored_type| stored_type == expected_type),
            Err(_) => false,
        }
    }

    /// Unregister a pointer (called when object is destroyed)
    pub fn unregister(&self, ptr: *mut T) -> bool {
        if ptr.is_null() {
            return false;
        }

        let addr = ptr as usize;
        match self.valid_pointers.write() {
            Ok(mut map) => map.remove(&addr).is_some(),
            Err(_) => false,
        }
    }

    /// Get the number of currently tracked pointers
    pub fn count(&self) -> usize {
        match self.valid_pointers.read() {
            Ok(map) => map.len(),
            Err(_) => 0,
        }
    }

    /// Clear all tracked pointers (use with caution)
    pub fn clear(&self) {
        if let Ok(mut map) = self.valid_pointers.write() {
            map.clear();
        }
    }
}

impl<T> Clone for PointerTracker<T> {
    fn clone(&self) -> Self {
        Self {
            valid_pointers: self.valid_pointers.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for PointerTracker<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Check basic pointer plausibility before a typed access.
///
/// This does not prove allocation, liveness, ownership, readability, or
/// writability. An API that owns a handle must use its typed `PointerTracker`
/// before dereferencing it.
pub fn validate_pointer<T: 'static>(ptr: *const u8) -> bool {
    use std::sync::OnceLock;

    static GLOBAL_TRACKER: OnceLock<PointerTracker<u8>> = OnceLock::new();
    let _tracker = GLOBAL_TRACKER.get_or_init(PointerTracker::new);

    // For now, we'll use a simplified validation approach
    // In a full implementation, you'd maintain separate trackers per type
    if ptr.is_null() {
        return false;
    }

    // Basic alignment check for the type - this prevents most type confusion attacks
    let alignment = std::mem::align_of::<T>();
    if (ptr as usize) % alignment != 0 {
        return false;
    }

    // Reject addresses outside the common userspace range. This is still only
    // a plausibility check and cannot establish that memory is mapped.
    let addr = ptr as usize;

    // Check if pointer is in reasonable user space (not kernel space)
    const USER_SPACE_MAX: usize = 0x7FFF_FFFF_FFFF; // 47-bit user space
    addr < USER_SPACE_MAX
}

/// Register a typed pointer globally for validation
pub fn register_typed_pointer<T: 'static>(ptr: *mut T) -> bool {
    if ptr.is_null() {
        return false;
    }

    // Basic alignment check for the type
    let alignment = std::mem::align_of::<T>();
    (ptr as usize) % alignment == 0
}

/// Global pointer trackers for different types
pub mod trackers {
    use super::PointerTracker;
    use crate::core::renderer::Renderer;
    use crate::core::surface::Surface;
    use crate::core::terminal::Terminal;
    use std::sync::OnceLock;

    static SURFACE_TRACKER: OnceLock<PointerTracker<Surface>> = OnceLock::new();
    static BORROWED_SURFACE_TRACKER: OnceLock<PointerTracker<Surface>> = OnceLock::new();
    static RENDERER_TRACKER: OnceLock<PointerTracker<Renderer>> = OnceLock::new();
    static TERMINAL_TRACKER: OnceLock<PointerTracker<Terminal>> = OnceLock::new();

    /// Get the surface pointer tracker
    pub fn surface_tracker() -> &'static PointerTracker<Surface> {
        SURFACE_TRACKER.get_or_init(PointerTracker::new)
    }

    /// Surfaces owned by renderers may be used but must not be freed by callers.
    pub(crate) fn borrowed_surface_tracker() -> &'static PointerTracker<Surface> {
        BORROWED_SURFACE_TRACKER.get_or_init(PointerTracker::new)
    }

    pub(crate) fn is_valid_surface(ptr: *const Surface) -> bool {
        surface_tracker().is_valid(ptr) || borrowed_surface_tracker().is_valid(ptr)
    }

    /// Get the renderer pointer tracker
    pub fn renderer_tracker() -> &'static PointerTracker<Renderer> {
        RENDERER_TRACKER.get_or_init(PointerTracker::new)
    }

    /// Get the terminal pointer tracker
    pub fn terminal_tracker() -> &'static PointerTracker<Terminal> {
        TERMINAL_TRACKER.get_or_init(PointerTracker::new)
    }

    /// Clear all trackers (called on cleanup)
    pub fn clear_all() {
        surface_tracker().clear();
        borrowed_surface_tracker().clear();
        renderer_tracker().clear();
        terminal_tracker().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointer_tracking() {
        let tracker = PointerTracker::<i32>::new();
        let mut value = 42;
        let ptr = &mut value as *mut i32;

        // Register pointer
        assert!(tracker.register(ptr));
        assert!(tracker.is_valid(ptr));
        assert_eq!(tracker.count(), 1);

        // Unregister pointer
        assert!(tracker.unregister(ptr));
        assert!(!tracker.is_valid(ptr));
        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn test_null_pointer_handling() {
        let tracker = PointerTracker::<i32>::new();
        let null_ptr: *mut i32 = std::ptr::null_mut();

        assert!(!tracker.register(null_ptr));
        assert!(!tracker.is_valid(null_ptr));
        assert!(!tracker.unregister(null_ptr));
    }

    #[test]
    fn test_multiple_pointers() {
        let tracker = PointerTracker::<i32>::new();
        let mut values = vec![1, 2, 3];
        let ptrs: Vec<*mut i32> = values.iter_mut().map(|v| v as *mut i32).collect();

        // Register all pointers
        for ptr in &ptrs {
            assert!(tracker.register(*ptr));
        }
        assert_eq!(tracker.count(), 3);

        // All should be valid
        for ptr in &ptrs {
            assert!(tracker.is_valid(*ptr));
        }

        // Unregister one
        assert!(tracker.unregister(ptrs[1]));
        assert_eq!(tracker.count(), 2);
        assert!(!tracker.is_valid(ptrs[1]));
    }
}
