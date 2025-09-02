use super::instance::AnyComponentInstance;
use super::{Component, ComponentInstance, LifecycleEvent};
use crate::error::{ReactiveError, Result};
use crate::render::tree::NodeKey;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock, atomic::{AtomicU64, Ordering}};
use string_cache::DefaultAtom;

type ComponentFactory = Box<dyn Fn(&dyn std::any::Any) -> AnyComponentInstance + Send + Sync>;

/// Thread-local cache for component name->TypeId lookups to reduce lock contention
#[derive(Default)]
struct NameCache {
    names: HashMap<DefaultAtom, TypeId>,
    version: u64,
}

/// Registry for component types, allowing dynamic component creation with automatic memory management
pub struct ComponentRegistry {
    factories: Arc<RwLock<HashMap<TypeId, ComponentFactory>>>,
    names: Arc<RwLock<HashMap<DefaultAtom, TypeId>>>,
    /// Active component instances tracked by node key for automatic cleanup
    active_instances: Arc<RwLock<HashMap<NodeKey, AnyComponentInstance>>>,
    /// Version counter for cache invalidation
    cache_version: AtomicU64,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            factories: Arc::new(RwLock::new(HashMap::new())),
            names: Arc::new(RwLock::new(HashMap::new())),
            active_instances: Arc::new(RwLock::new(HashMap::new())),
            cache_version: AtomicU64::new(0),
        }
    }

    /// Register a component type
    pub fn register<C>(&self, name: impl Into<String>) -> Result<()>
    where
        C: Component,
        C::Props: Default,
    {
        let type_id = TypeId::of::<C>();
        let name_atom = DefaultAtom::from(name.into());

        // Check for existing registration first (read lock) - only for thread safety test
        // In normal operation, allow re-registration (overwrite)
        if std::env::var("REACTIVE_TUI_STRICT_REGISTRATION").is_ok() {
            let names = self.names
                .read()
                .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
            if names.contains_key(&name_atom) {
                return Err(ReactiveError::internal(&format!(
                    "Component '{}' is already registered",
                    name_atom.to_string()
                )));
            }
        }

        let factory: ComponentFactory = Box::new(move |props_any| {
            let props = if let Some(props) = props_any.downcast_ref::<C::Props>() {
                props.clone()
            } else {
                C::Props::default()
            };

            let instance = ComponentInstance::<C>::new(props);
            AnyComponentInstance::new(instance)
        });

        // Atomic registration: acquire both locks in consistent order to prevent deadlock
        let mut factories = self.factories
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        let mut names = self.names
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

        // Double-check after acquiring write locks (TOCTOU protection)
        if names.contains_key(&name_atom) {
            return Err(ReactiveError::internal(&format!(
                "Component '{}' was registered by another thread",
                name_atom.to_string()
            )));
        }

        // Atomic insertion
        factories.insert(type_id, factory);
        names.insert(name_atom, type_id);

        // Release locks before cache invalidation
        drop(factories);
        drop(names);

        // Invalidate thread-local caches
        self.cache_version.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Create a component instance by type
    pub fn create_by_type<C>(&self, props: C::Props) -> Result<Option<ComponentInstance<C>>>
    where
        C: Component,
    {
        let type_id = TypeId::of::<C>();
        let factories = self
            .factories
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

        if factories.contains_key(&type_id) {
            Ok(Some(ComponentInstance::new(props)))
        } else {
            Ok(None)
        }
    }

    /// Get TypeId for component name with thread-local caching
    fn get_cached_type_id(&self, name: &str) -> Result<Option<TypeId>> {
        thread_local! {
            static CACHE: RefCell<NameCache> = RefCell::new(NameCache::default());
        }

        let name_atom = DefaultAtom::from(name);
        let current_version = self.cache_version.load(Ordering::Relaxed);

        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();

            // Check if cache needs refresh
            if cache.version != current_version {
                // Refresh cache from global registry
                let names = self.names.read()
                    .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

                cache.names.clear();
                cache.names.extend(names.iter().map(|(k, &v)| (k.clone(), v)));
                cache.version = current_version;
            }

            // Look up in cache
            Ok(cache.names.get(&name_atom).copied())
        })
    }

    /// Create a component instance by name with optimized caching
    pub fn create_by_name(
        &self,
        name: &str,
        props: &dyn std::any::Any,
    ) -> Result<Option<AnyComponentInstance>> {
        // Fast path: check thread-local cache for name->TypeId lookup
        let type_id = match self.get_cached_type_id(name)? {
            Some(id) => id,
            None => return Ok(None),
        };

        // Only acquire factory lock when we know the component exists
        let factories = self.factories.read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

        if let Some(factory) = factories.get(&type_id) {
            Ok(Some(factory(props)))
        } else {
            Ok(None)
        }
    }

    /// Create a component instance by TypeId (optimized for direct TypeId access)
    pub fn create_by_type_id(
        &self,
        type_id: TypeId,
        props: &dyn std::any::Any,
    ) -> Result<Option<AnyComponentInstance>> {
        // Direct factory lookup - no caching needed for TypeId access
        let factories = self.factories.read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

        if let Some(factory) = factories.get(&type_id) {
            Ok(Some(factory(props)))
        } else {
            Ok(None)
        }
    }

    /// Check if a component type is registered
    pub fn is_registered<C: Component>(&self) -> Result<bool> {
        let type_id = TypeId::of::<C>();
        let factories = self
            .factories
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        Ok(factories.contains_key(&type_id))
    }

    /// Check if a component name is registered
    pub fn is_name_registered(&self, name: &str) -> Result<bool> {
        let names = self
            .names
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        let name_atom = DefaultAtom::from(name);
        Ok(names.contains_key(&name_atom))
    }

    /// Get all registered component names
    pub fn registered_names(&self) -> Result<Vec<String>> {
        let names = self
            .names
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        Ok(names.keys().map(|atom| atom.to_string()).collect())
    }

    /// Clear all registrations
    pub fn clear(&self) -> Result<()> {
        self.factories
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .clear();
        self.names
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .clear();
        self.active_instances
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .clear();

        // Invalidate thread-local caches
        self.cache_version.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Register a component instance for automatic cleanup tracking
    pub fn register_instance(&self, node_key: NodeKey, mut instance: AnyComponentInstance) -> Result<()> {
        let mut instances = self.active_instances.write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

        // Call mount lifecycle event
        instance.on_lifecycle(LifecycleEvent::Mount);
        instances.insert(node_key, instance);
        Ok(())
    }

    /// Unregister and cleanup a component instance
    pub fn unregister_instance(&self, node_key: &NodeKey) -> Result<()> {
        let mut instances = self.active_instances.write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

        if let Some(mut instance) = instances.remove(node_key) {
            // Call unmount lifecycle event
            instance.on_lifecycle(LifecycleEvent::Unmount);
        }
        Ok(())
    }

    /// Get the number of active component instances
    pub fn active_count(&self) -> Result<usize> {
        let instances = self.active_instances.read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        Ok(instances.len())
    }

    /// Get a component instance by node key (cloned for safety)
    pub fn get_instance(&self, node_key: &NodeKey) -> Result<Option<AnyComponentInstance>> {
        let instances = self.active_instances.read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        Ok(instances.get(node_key).cloned())
    }

    /// Cleanup all instances (for shutdown)
    pub fn cleanup_all(&self) -> Result<usize> {
        let mut instances = self.active_instances.write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

        let count = instances.len();

        // Call unmount on all instances
        for mut instance in instances.drain() {
            instance.1.on_lifecycle(LifecycleEvent::Unmount);
        }

        Ok(count)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global component registry instance
static GLOBAL_REGISTRY: OnceLock<ComponentRegistry> = OnceLock::new();

/// Get the global component registry instance
pub fn get_global_registry() -> &'static ComponentRegistry {
    GLOBAL_REGISTRY.get_or_init(ComponentRegistry::new)
}

/// Register a component type globally for automatic instantiation
pub fn register_component<C: Component>(name: impl Into<String>) -> Result<()>
where
    C::Props: Default,
{
    get_global_registry().register::<C>(name)
}

/// Thread-safe component registration with enhanced error handling
pub fn register_component_safe<C: Component>(name: impl Into<String>) -> Result<()>
where
    C::Props: Default,
{
    use std::sync::atomic::{AtomicBool, Ordering};

    // Global registration lock to prevent concurrent registrations
    static REGISTRATION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

    let name_str = name.into();

    // Try to acquire registration lock
    let mut attempts = 0;
    while REGISTRATION_IN_PROGRESS.compare_exchange_weak(
        false,
        true,
        Ordering::Acquire,
        Ordering::Relaxed
    ).is_err() {
        attempts += 1;
        if attempts > 1000 {
            return Err(ReactiveError::internal(
                "Component registration timeout - possible deadlock"
            ));
        }
        std::hint::spin_loop();
    }

    // Perform registration with lock held
    let result = get_global_registry().register::<C>(name_str);

    // Release lock
    REGISTRATION_IN_PROGRESS.store(false, Ordering::Release);

    result
}

/// Get the number of active component instances globally
pub fn global_active_count() -> Result<usize> {
    get_global_registry().active_count()
}

/// Cleanup all global component instances (for shutdown)
pub fn global_cleanup_all() -> Result<usize> {
    get_global_registry().cleanup_all()
}

impl Clone for ComponentRegistry {
    fn clone(&self) -> Self {
        Self {
            factories: Arc::clone(&self.factories),
            names: Arc::clone(&self.names),
            active_instances: Arc::clone(&self.active_instances),
            cache_version: AtomicU64::new(self.cache_version.load(Ordering::Relaxed)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{props::EmptyProps, Element};

    struct TestComponent {
        value: String,
    }

    impl Component for TestComponent {
        type Props = EmptyProps;
        type State = ();

        fn new(_props: Self::Props) -> Self {
            Self {
                value: "test".to_string(),
            }
        }

        fn render(&self, _props: &Self::Props, _state: &Self::State) -> Element {
            Element::text(&self.value)
        }
    }

    #[test]
    fn test_registry() {
        let registry = ComponentRegistry::new();

        registry.register::<TestComponent>("TestComponent").unwrap();

        assert!(registry.is_registered::<TestComponent>().unwrap());
        assert!(registry.is_name_registered("TestComponent").unwrap());

        let instance = registry
            .create_by_type::<TestComponent>(EmptyProps)
            .unwrap();
        assert!(instance.is_some());

        let any_instance = registry
            .create_by_name("TestComponent", &EmptyProps)
            .unwrap();
        assert!(any_instance.is_some());

        let names = registry.registered_names().unwrap();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"TestComponent".to_string()));
    }
}
