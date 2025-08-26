use super::{Component, ComponentInstance};
use super::instance::AnyComponentInstance;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type ComponentFactory = Box<dyn Fn(&dyn std::any::Any) -> AnyComponentInstance + Send + Sync>;

/// Registry for component types, allowing dynamic component creation
pub struct ComponentRegistry {
    factories: Arc<RwLock<HashMap<TypeId, ComponentFactory>>>,
    names: Arc<RwLock<HashMap<String, TypeId>>>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            factories: Arc::new(RwLock::new(HashMap::new())),
            names: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a component type
    pub fn register<C>(&self, name: impl Into<String>)
    where
        C: Component,
        C::Props: Default,
    {
        let type_id = TypeId::of::<C>();
        let name = name.into();

        let factory: ComponentFactory = Box::new(move |props_any| {
            let props = if let Some(props) = props_any.downcast_ref::<C::Props>() {
                props.clone()
            } else {
                C::Props::default()
            };

            let instance = ComponentInstance::<C>::new(props);
            AnyComponentInstance::new(instance)
        });

        self.factories.write().unwrap().insert(type_id, factory);
        self.names.write().unwrap().insert(name, type_id);
    }

    /// Create a component instance by type
    pub fn create_by_type<C>(&self, props: C::Props) -> Option<ComponentInstance<C>>
    where
        C: Component,
    {
        let type_id = TypeId::of::<C>();
        let factories = self.factories.read().unwrap();

        if factories.contains_key(&type_id) {
            Some(ComponentInstance::new(props))
        } else {
            None
        }
    }

    /// Create a component instance by name
    pub fn create_by_name(
        &self,
        name: &str,
        props: &dyn std::any::Any,
    ) -> Option<AnyComponentInstance> {
        let names = self.names.read().unwrap();
        let type_id = names.get(name)?;

        let factories = self.factories.read().unwrap();
        let factory = factories.get(type_id)?;

        Some(factory(props))
    }

    /// Create a component instance by TypeId (dynamic typed creation via stored factory)
    pub fn create_by_type_id(&self, type_id: TypeId, props: &dyn std::any::Any) -> Option<AnyComponentInstance> {
        let factories = self.factories.read().unwrap();
        let factory = factories.get(&type_id)?;
        Some(factory(props))
    }


    /// Check if a component type is registered
    pub fn is_registered<C: Component>(&self) -> bool {
        let type_id = TypeId::of::<C>();
        self.factories.read().unwrap().contains_key(&type_id)
    }

    /// Check if a component name is registered
    pub fn is_name_registered(&self, name: &str) -> bool {
        self.names.read().unwrap().contains_key(name)
    }

    /// Get all registered component names
    pub fn registered_names(&self) -> Vec<String> {
        self.names.read().unwrap().keys().cloned().collect()
    }

    /// Clear all registrations
    pub fn clear(&self) {
        self.factories.write().unwrap().clear();
        self.names.write().unwrap().clear();
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ComponentRegistry {
    fn clone(&self) -> Self {
        Self {
            factories: Arc::clone(&self.factories),
            names: Arc::clone(&self.names),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Element, props::EmptyProps};

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

        registry.register::<TestComponent>("TestComponent");

        assert!(registry.is_registered::<TestComponent>());
        assert!(registry.is_name_registered("TestComponent"));

        let instance = registry.create_by_type::<TestComponent>(EmptyProps);
        assert!(instance.is_some());

        let any_instance = registry.create_by_name("TestComponent", &EmptyProps);
        assert!(any_instance.is_some());

        let names = registry.registered_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"TestComponent".to_string()));
    }
}