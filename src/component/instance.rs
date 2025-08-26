use super::{Component, AnyComponent, Element, Lifecycle, LifecycleEvent};
use std::any::{Any, TypeId};
use std::pin::Pin;
use std::task::{Context, Poll};

/// A wrapper around a component instance that manages its lifecycle and state
pub struct ComponentInstance<C: Component> {
    component: C,
    props: C::Props,
    state: C::State,
    lifecycle: Lifecycle,
    needs_update: bool,
}

impl<C: Component> ComponentInstance<C> {
    /// Create a new component instance
    pub fn new(props: C::Props) -> Self {
        let component = C::new(props.clone());
        Self {
            component,
            props,
            state: C::State::default(),
            lifecycle: Lifecycle::new(),
            needs_update: true,
        }
    }
    
    /// Update the component with new props
    pub fn update_props(&mut self, new_props: C::Props) -> bool {
        if self.props != new_props {
            self.props = new_props;
            self.lifecycle.begin_update();
            self.needs_update = self.component.update(&self.props, &mut self.state);
            self.component.on_lifecycle(LifecycleEvent::PropsChanged, &mut self.state);
            self.lifecycle.complete_update();
            self.needs_update
        } else {
            false
        }
    }
    
    /// Render the component
    pub fn render(&self) -> Element {
        self.component.render(&self.props, &self.state)
    }
    
    /// Mount the component
    pub fn mount(&mut self) {
        self.lifecycle.mount();
        self.component.on_lifecycle(LifecycleEvent::Mount, &mut self.state);
        self.lifecycle.complete_mount();
    }
    
    /// Unmount the component
    pub fn unmount(&mut self) {
        self.lifecycle.unmount();
        self.component.on_lifecycle(LifecycleEvent::Unmount, &mut self.state);
        self.lifecycle.complete_unmount();
    }
    
    /// Check if the component needs an update
    pub fn needs_update(&self) -> bool {
        self.needs_update
    }
    
    /// Mark that the component has been updated
    pub fn mark_updated(&mut self) {
        self.needs_update = false;
    }
    
    /// Get a reference to the props
    pub fn props(&self) -> &C::Props {
        &self.props
    }
    
    /// Get a reference to the state
    pub fn state(&self) -> &C::State {
        &self.state
    }
    
    /// Get a mutable reference to the state
    pub fn state_mut(&mut self) -> &mut C::State {
        &mut self.state
    }
    
    /// Get the lifecycle phase
    pub fn lifecycle(&self) -> &Lifecycle {
        &self.lifecycle
    }
}

/// Type-erased component instance for storing different component types together
pub struct AnyComponentInstance {
    inner: Box<dyn AnyComponent>,
}

impl AnyComponentInstance {
    /// Create from a typed component instance
    pub fn new<C: Component>(instance: ComponentInstance<C>) -> Self {
        Self {
            inner: Box::new(ComponentInstanceWrapper(instance)),
        }
    }
    
    /// Update with type-erased props
    pub fn update(&mut self, props: &dyn Any) -> bool {
        self.inner.update_any(props)
    }
    
    /// Render the component
    pub fn render(&self) -> Element {
        self.inner.render_any()
    }
    
    /// Handle lifecycle event
    pub fn on_lifecycle(&mut self, event: LifecycleEvent) {
        self.inner.on_lifecycle_any(event)
    }
    
    /// Get type ID
    pub fn type_id(&self) -> TypeId {
        AnyComponent::type_id(&*self.inner)
    }
}

/// Wrapper to make ComponentInstance implement AnyComponent
struct ComponentInstanceWrapper<C: Component>(ComponentInstance<C>);

impl<C: Component> AnyComponent for ComponentInstanceWrapper<C> {
    fn update_any(&mut self, props: &dyn Any) -> bool {
        if let Some(typed_props) = props.downcast_ref::<C::Props>() {
            self.0.update_props(typed_props.clone())
        } else {
            false
        }
    }
    
    fn render_any(&self) -> Element {
        self.0.render()
    }
    
    fn poll_change_any(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        unsafe {
            let component = &mut self.as_mut().get_unchecked_mut().0.component;
            Pin::new_unchecked(component).poll_change(cx)
        }
    }
    
    fn on_lifecycle_any(&mut self, event: LifecycleEvent) {
        self.0.component.on_lifecycle(event, &mut self.0.state)
    }
    
    fn type_id(&self) -> TypeId {
        Component::type_id(&self.0.component)
    }
    
    fn as_any(&self) -> &dyn Any {
        &self.0
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}