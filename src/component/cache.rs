//! Component registry performance optimizations
//!
//! Pre-computed lookups and caching for common components

use super::instance::AnyComponentInstance;
use lru::LruCache;
use once_cell::sync::Lazy;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Pre-computed TypeIds for common components
/// These are computed at compile-time and stored in a perfect hash map
pub struct CommonComponents {
    map: HashMap<&'static str, TypeId>,
}

impl CommonComponents {
    /// Get TypeId for a common component name
    #[inline(always)]
    pub fn get(&self, name: &str) -> Option<TypeId> {
        self.map.get(name).copied()
    }

    /// Register a common component
    pub fn register(&mut self, name: &'static str, type_id: TypeId) {
        self.map.insert(name, type_id);
    }
}

/// Global common components lookup
static COMMON_COMPONENTS: Lazy<CommonComponents> = Lazy::new(|| {
    // Register common built-in components here
    // These will be populated by the registry when components are registered

    CommonComponents {
        map: HashMap::with_capacity(32),
    }
});

/// Get the common components registry
pub fn common_components() -> &'static CommonComponents {
    &COMMON_COMPONENTS
}

thread_local! {
    /// Thread-local LRU cache for component instances
    static INSTANCE_CACHE: RefCell<LruCache<(TypeId, u64), Arc<AnyComponentInstance>>> =
        RefCell::new(LruCache::new(NonZeroUsize::new(64).unwrap()));
}

/// Cache key for component instances
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct InstanceCacheKey {
    type_id: TypeId,
    props_hash: u64,
}

impl InstanceCacheKey {
    /// Create a new cache key
    pub fn new(type_id: TypeId, props_hash: u64) -> Self {
        Self {
            type_id,
            props_hash,
        }
    }
}

/// Get a cached component instance
pub fn get_cached_instance(key: InstanceCacheKey) -> Option<Arc<AnyComponentInstance>> {
    INSTANCE_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .get(&(key.type_id, key.props_hash))
            .cloned()
    })
}

/// Store a component instance in the cache
pub fn cache_instance(key: InstanceCacheKey, instance: Arc<AnyComponentInstance>) {
    INSTANCE_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .put((key.type_id, key.props_hash), instance);
    });
}

/// Clear the instance cache
pub fn clear_instance_cache() {
    INSTANCE_CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
}

/// Statistics for cache performance monitoring
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of cache evictions
    pub evictions: u64,
}

thread_local! {
    static CACHE_STATS: RefCell<CacheStats> = RefCell::new(CacheStats::default());
}

/// Get cache statistics
pub fn get_cache_stats() -> CacheStats {
    CACHE_STATS.with(|stats| stats.borrow().clone())
}

/// Record a cache hit
pub fn record_hit() {
    CACHE_STATS.with(|stats| {
        stats.borrow_mut().hits += 1;
    });
}

/// Record a cache miss
pub fn record_miss() {
    CACHE_STATS.with(|stats| {
        stats.borrow_mut().misses += 1;
    });
}
