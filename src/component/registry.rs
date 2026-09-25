use super::instance::AnyComponentInstance;
use super::tracked_instance::{create_shared_tracked_instance, SharedTrackedInstance};
use super::{Component, ComponentInstance, LifecycleEvent};
use crate::error::{ReactiveError, Result};
use crate::layout::css::manager::apply_css_animation_global;
use crate::render::tree::NodeKey;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};
use string_cache::DefaultAtom;

type ComponentFactory = Arc<dyn Fn(&dyn std::any::Any) -> AnyComponentInstance + Send + Sync>;

/// Registry for component types, allowing dynamic component creation with automatic memory management
pub struct ComponentRegistry {
    factories: Arc<RwLock<HashMap<TypeId, ComponentFactory>>>,
    names: Arc<RwLock<HashMap<DefaultAtom, TypeId>>>,
    /// Active component instances tracked by node key for automatic cleanup
    active_instances: Arc<RwLock<HashMap<NodeKey, SharedTrackedInstance>>>,
    /// Performance tracking
    performance_stats: Arc<RwLock<ComponentPerformanceStats>>,
}

/// Component performance statistics
#[derive(Debug, Default)]
struct ComponentPerformanceStats {
    total_created: u64,
    total_destroyed: u64,
    total_creation_time: Duration,
    total_cleanup_time: Duration,
    creation_times: Vec<Duration>,
    cleanup_times: Vec<Duration>,
    max_history: usize,
}

impl ComponentPerformanceStats {
    fn new() -> Self {
        Self {
            max_history: 1000, // Keep last 1000 operations for averaging
            ..Default::default()
        }
    }

    fn record_creation(&mut self, duration: Duration) {
        self.total_created += 1;
        self.total_creation_time += duration;

        self.creation_times.push(duration);
        if self.creation_times.len() > self.max_history {
            self.creation_times.remove(0);
        }
    }

    fn record_cleanup(&mut self, duration: Duration) {
        self.total_destroyed += 1;
        self.total_cleanup_time += duration;

        self.cleanup_times.push(duration);
        if self.cleanup_times.len() > self.max_history {
            self.cleanup_times.remove(0);
        }
    }

    fn avg_creation_time(&self) -> Duration {
        if self.creation_times.is_empty() {
            Duration::ZERO
        } else {
            self.creation_times.iter().sum::<Duration>() / self.creation_times.len() as u32
        }
    }

    fn avg_cleanup_time(&self) -> Duration {
        if self.cleanup_times.is_empty() {
            Duration::ZERO
        } else {
            self.cleanup_times.iter().sum::<Duration>() / self.cleanup_times.len() as u32
        }
    }
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            factories: Arc::new(RwLock::new(HashMap::new())),
            names: Arc::new(RwLock::new(HashMap::new())),
            active_instances: Arc::new(RwLock::new(HashMap::new())),
            performance_stats: Arc::new(RwLock::new(ComponentPerformanceStats::new())),
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
            let names = self
                .names
                .read()
                .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
            if names.contains_key(&name_atom) {
                return Err(ReactiveError::internal(format!(
                    "Component '{}' is already registered",
                    name_atom
                )));
            }
        }

        let factory: ComponentFactory = Arc::new(move |props_any| {
            let props = if let Some(props) = props_any.downcast_ref::<C::Props>() {
                props.clone()
            } else {
                C::Props::default()
            };

            let instance = ComponentInstance::<C>::new(props);
            AnyComponentInstance::new(instance)
        });

        // Atomic registration: acquire both locks in consistent order to prevent deadlock
        let mut factories = self
            .factories
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        let mut names = self
            .names
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

        // Double-check after acquiring write locks (TOCTOU protection)
        if names.contains_key(&name_atom) {
            return Err(ReactiveError::internal(format!(
                "Component '{}' was registered by another thread",
                name_atom
            )));
        }

        // Atomic insertion
        factories.insert(type_id, factory);
        names.insert(name_atom, type_id);

        Ok(())
    }

    /// Create a component instance by type
    pub fn create_by_type<C>(&self, props: C::Props) -> Result<Option<ComponentInstance<C>>>
    where
        C: Component,
    {
        let type_id = TypeId::of::<C>();
        let registered = self
            .factories
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .contains_key(&type_id);
        // Constructors are user code and may register other components.
        Ok(registered.then(|| ComponentInstance::new(props)))
    }

    /// Create a component instance using this registry's current name map.
    /// Clones share the map; independent registries never share lookup state.
    pub fn create_by_name(
        &self,
        name: &str,
        props: &dyn std::any::Any,
    ) -> Result<Option<AnyComponentInstance>> {
        let type_id = self
            .names
            .read()
            .map_err(|_| ReactiveError::internal("Component registry name lock poisoned"))?
            .get(&DefaultAtom::from(name))
            .copied();
        // Release the name guard before factory lookup and user constructors.
        match type_id {
            Some(type_id) => self.create_by_type_id(type_id, props),
            None => Ok(None),
        }
    }

    /// Create a component instance by TypeId (optimized for direct TypeId access)
    pub fn create_by_type_id(
        &self,
        type_id: TypeId,
        props: &dyn std::any::Any,
    ) -> Result<Option<AnyComponentInstance>> {
        let factory = self
            .factories
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .get(&type_id)
            .cloned();
        Ok(factory.map(|factory| factory(props)))
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
        {
            // Match registration order and make both tables empty together.
            let mut factories = self
                .factories
                .write()
                .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
            let mut names = self
                .names
                .write()
                .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
            factories.clear();
            names.clear();
        }
        self.cleanup_all()?;
        Ok(())
    }

    /// Register a component instance for automatic cleanup tracking
    pub fn register_instance(
        &self,
        node_key: NodeKey,
        mut instance: AnyComponentInstance,
    ) -> Result<()> {
        let start = Instant::now();

        // Call mount lifecycle event before wrapping
        instance.on_lifecycle(LifecycleEvent::Mount);

        // Create a tracked instance with automatic cleanup
        let tracked = create_shared_tracked_instance(
            instance,
            node_key.clone(),
            Arc::new(self.clone()), // Retained constructor argument for API compatibility
        );

        let replaced = self
            .active_instances
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .insert(node_key, tracked);

        self.performance_stats
            .write()
            .map_err(|_| ReactiveError::internal("Component registry statistics lock poisoned"))?
            .record_creation(start.elapsed());
        self.dispose_instances(replaced)?;
        Ok(())
    }

    /// Register a component instance with CSS animation support
    pub fn register_instance_with_element(
        &self,
        node_key: NodeKey,
        instance: AnyComponentInstance,
        element: &crate::component::Element,
    ) -> Result<()> {
        // Extract CSS animations from the element's class string
        if let Some(class_str) = &element.class {
            let animations =
                crate::layout::css::animations::extract_css_animations_from_classes(class_str);
            let component_id = format!("{:?}", node_key); // Use NodeKey as component ID

            // Apply each CSS animation found
            for animation_name in animations {
                if let Err(e) = apply_css_animation_global(&component_id, &animation_name) {
                    // Log error but don't fail the registration
                    log::warn!(
                        "Warning: Failed to apply CSS animation '{}' to component '{}': {}",
                        animation_name,
                        component_id,
                        e
                    );
                }
            }
        }

        self.register_instance(node_key, instance)
    }

    /// Unregister and cleanup a component instance
    pub fn unregister_instance(&self, node_key: &NodeKey) -> Result<()> {
        let removed = self
            .active_instances
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .remove(node_key);
        self.dispose_instances(removed)?;
        Ok(())
    }

    // Map guards must be gone before this runs: Drop invokes user callbacks.
    fn dispose_instances(
        &self,
        removed: impl IntoIterator<Item = SharedTrackedInstance>,
    ) -> Result<usize> {
        let mut count = 0;
        for instance in removed {
            let start = Instant::now();
            drop(instance);
            self.performance_stats
                .write()
                .map_err(|_| {
                    ReactiveError::internal("Component registry statistics lock poisoned")
                })?
                .record_cleanup(start.elapsed());
            count += 1;
        }
        Ok(count)
    }

    /// Get the number of active component instances
    pub fn active_count(&self) -> Result<usize> {
        let instances = self
            .active_instances
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        Ok(instances.len())
    }

    /// Perform a cleanup sweep to remove orphaned components
    /// This is a fallback mechanism that should be called periodically
    /// Returns the number of orphaned components cleaned up
    pub fn cleanup_orphaned_components(
        &self,
        active_node_keys: &std::collections::HashSet<NodeKey>,
    ) -> Result<usize> {
        let removed = {
            let mut instances = self
                .active_instances
                .write()
                .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
            let orphaned_keys: Vec<_> = instances
                .keys()
                .filter(|key| !active_node_keys.contains(key))
                .cloned()
                .collect();
            orphaned_keys
                .into_iter()
                .filter_map(|key| instances.remove(&key))
                .collect::<Vec<_>>()
        };
        self.dispose_instances(removed)
    }

    /// Get component performance metrics.
    /// Counts may span concurrent operations; they settle once those operations
    /// complete. This does not hold the instance and statistics locks together.
    pub fn performance_metrics(&self) -> Result<ComponentRegistryMetrics> {
        let active_count = self.active_count()?;
        let stats = self
            .performance_stats
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;

        Ok(ComponentRegistryMetrics {
            total_created: stats.total_created,
            total_destroyed: stats.total_destroyed,
            active_components: active_count as u32,
            avg_creation_time: stats.avg_creation_time(),
            avg_cleanup_time: stats.avg_cleanup_time(),
            total_creation_time: stats.total_creation_time,
            total_cleanup_time: stats.total_cleanup_time,
        })
    }

    /// Reset performance statistics
    pub fn reset_performance_stats(&self) -> Result<()> {
        let mut stats = self
            .performance_stats
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        *stats = ComponentPerformanceStats::new();
        Ok(())
    }

    /// Get a component instance by node key (for debugging)
    pub fn get_instance(&self, node_key: &NodeKey) -> Result<Option<AnyComponentInstance>> {
        let tracked = self
            .active_instances
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .get(node_key)
            .cloned();
        match tracked {
            Some(tracked) => {
                let guard = tracked
                    .read()
                    .map_err(|_| ReactiveError::internal("Tracked component lock poisoned"))?;
                Ok(guard.instance().cloned())
            }
            None => Ok(None),
        }
    }

    /// Cleanup all instances (for shutdown)
    pub fn cleanup_all(&self) -> Result<usize> {
        let removed = {
            let mut instances = self
                .active_instances
                .write()
                .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
            std::mem::take(&mut *instances)
        };
        self.dispose_instances(removed.into_values())
    }

    /// Clear all registered components (for testing only)
    /// WARNING: This is a testing utility and should not be used in production
    pub fn clear_all(&self) -> Result<()> {
        self.clear()
    }
}

/// Component registry performance metrics
#[derive(Debug, Clone)]
pub struct ComponentRegistryMetrics {
    /// Total components created
    pub total_created: u64,
    /// Total tracked registrations removed, including replacement and bulk cleanup.
    /// A concurrent get_instance call can briefly retain a removed instance.
    pub total_destroyed: u64,
    /// Current active components
    pub active_components: u32,
    /// Average component creation time
    pub avg_creation_time: Duration,
    /// Average component cleanup time
    pub avg_cleanup_time: Duration,
    /// Total time spent creating components
    pub total_creation_time: Duration,
    /// Total time spent cleaning up components
    pub total_cleanup_time: Duration,
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
    while REGISTRATION_IN_PROGRESS
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        attempts += 1;
        if attempts > 1000 {
            return Err(ReactiveError::internal(
                "Component registration timeout - possible deadlock",
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

/// Get global component performance metrics
pub fn global_component_performance() -> Result<ComponentRegistryMetrics> {
    get_global_registry().performance_metrics()
}

/// Reset global component performance statistics
pub fn reset_global_component_performance() -> Result<()> {
    get_global_registry().reset_performance_stats()
}

/// Clear all registered components and instances (for testing only)
/// WARNING: This is a testing utility and should not be used in production
pub fn global_clear_all() -> Result<()> {
    get_global_registry().clear_all()
}

impl Clone for ComponentRegistry {
    fn clone(&self) -> Self {
        Self {
            factories: Arc::clone(&self.factories),
            names: Arc::clone(&self.names),
            active_instances: Arc::clone(&self.active_instances),
            performance_stats: Arc::clone(&self.performance_stats),
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
