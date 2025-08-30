use super::instance::AnyComponentInstance;
use super::{Component, ComponentInstance};
use crate::error::{ReactiveError, Result};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use string_cache::DefaultAtom;

type ComponentFactory = Box<dyn Fn(&dyn std::any::Any) -> AnyComponentInstance + Send + Sync>;

/// Registry for component types, allowing dynamic component creation
pub struct ComponentRegistry {
    factories: Arc<RwLock<HashMap<TypeId, ComponentFactory>>>,
    names: Arc<RwLock<HashMap<DefaultAtom, TypeId>>>,
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
    pub fn register<C>(&self, name: impl Into<String>) -> Result<()>
    where
        C: Component,
        C::Props: Default,
    {
        let type_id = TypeId::of::<C>();
        let name = DefaultAtom::from(name.into());

        let factory: ComponentFactory = Box::new(move |props_any| {
            let props = if let Some(props) = props_any.downcast_ref::<C::Props>() {
                props.clone()
            } else {
                C::Props::default()
            };

            let instance = ComponentInstance::<C>::new(props);
            AnyComponentInstance::new(instance)
        });

        self.factories
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .insert(type_id, factory);
        self.names
            .write()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?
            .insert(name, type_id);
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

    /// Create a component instance by name
    pub fn create_by_name(
        &self,
        name: &str,
        props: &dyn std::any::Any,
    ) -> Result<Option<AnyComponentInstance>> {
        let names = self
            .names
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        let name_atom = DefaultAtom::from(name);
        let type_id = match names.get(&name_atom) {
            Some(id) => id,
            None => return Ok(None),
        };

        let factories = self
            .factories
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        let factory = match factories.get(type_id) {
            Some(f) => f,
            None => return Ok(None),
        };

        Ok(Some(factory(props)))
    }

    /// Create a component instance by TypeId (dynamic typed creation via stored factory)
    pub fn create_by_type_id(
        &self,
        type_id: TypeId,
        props: &dyn std::any::Any,
    ) -> Result<Option<AnyComponentInstance>> {
        let factories = self
            .factories
            .read()
            .map_err(|_| ReactiveError::internal("Component registry lock poisoned"))?;
        let factory = match factories.get(&type_id) {
            Some(f) => f,
            None => return Ok(None),
        };
        Ok(Some(factory(props)))
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
        Ok(())
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
