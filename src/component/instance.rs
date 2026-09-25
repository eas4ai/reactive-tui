use super::{AnyComponent, Component, Element, Lifecycle, LifecycleEvent};
use std::any::{Any, TypeId};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// A wrapper around a component instance that manages its lifecycle and state
pub struct ComponentInstance<C: Component> {
    component: C,
    props: C::Props,
    supplied_props: C::Props,
    state: C::State,
    lifecycle: Lifecycle,
    needs_update: bool,
}

impl<C: Component> ComponentInstance<C> {
    /// Create a new component instance
    pub fn new(props: C::Props) -> Self {
        Self::from_component(C::new(props.clone()), props)
    }

    /// Wrap a configured component, retaining its callbacks for this instance.
    pub fn from_component(mut component: C, props: C::Props) -> Self {
        let state = component.initial_state(&props);
        Self {
            component,
            supplied_props: props.clone(),
            props,
            state,
            lifecycle: Lifecycle::new(),
            needs_update: true,
        }
    }

    /// Update the component with new props
    pub fn update_props(&mut self, new_props: C::Props) -> bool {
        if self.supplied_props != new_props {
            self.supplied_props = new_props.clone();
            self.props = new_props;
            self.lifecycle.begin_update();
            self.needs_update = self.component.update(&self.props, &mut self.state);
            self.component
                .on_lifecycle(LifecycleEvent::PropsChanged, &mut self.state);
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

    /// Render while preserving component errors for the application owner.
    pub fn try_render(&self) -> crate::error::Result<Element> {
        self.component.try_render(&self.props, &self.state)
    }

    /// Deliver input to this instance without recreating its state.
    pub fn handle_event(
        &mut self,
        event: &crate::event::Event,
    ) -> crate::event::router::EventResult {
        self.component
            .handle_event(event, &mut self.props, &mut self.state)
    }

    /// Update dimensions from the presented frame.
    pub fn layout(&mut self, bounds: super::LayoutInfo) -> bool {
        self.component
            .layout(bounds, &mut self.props, &mut self.state)
    }

    /// Mount the component
    pub fn mount(&mut self) {
        self.lifecycle.mount();
        self.component
            .on_lifecycle(LifecycleEvent::Mount, &mut self.state);
        self.lifecycle.complete_mount();
    }

    /// Unmount the component
    pub fn unmount(&mut self) {
        self.lifecycle.unmount();
        self.component
            .on_lifecycle(LifecycleEvent::Unmount, &mut self.state);
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

impl<C: Component> Drop for ComponentInstance<C> {
    fn drop(&mut self) {
        // Ensure component is properly unmounted when dropped
        if self.lifecycle.is_mounted() {
            self.unmount();
        }
    }
}

/// Type-erased component instance for storing different component types together
pub struct AnyComponentInstance {
    inner: Box<dyn AnyComponent>,
    /// Factory function that can recreate this component with the same props
    factory: Arc<dyn Fn() -> Box<dyn AnyComponent> + Send + Sync>,
    /// The props allocation last applied, so a frame that hands the same
    /// allocation down again (a parent that re-renders an unchanged element)
    /// skips the value comparison.
    last_props: Option<Arc<dyn Any + Send + Sync>>,
}

impl std::fmt::Debug for AnyComponentInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyComponentInstance")
            .field("type_id", &self.type_id())
            .finish()
    }
}

impl AnyComponentInstance {
    /// Create from a typed component instance with factory
    ///
    /// The factory pattern ensures that cloning creates fresh component instances
    /// with the same props but clean state. This is the correct behavior for
    /// component instances - they should not share mutable state.
    pub fn new<C: Component>(instance: ComponentInstance<C>) -> Self
    where
        C::Props: Clone + 'static,
    {
        let props = instance.props.clone();
        let factory = Arc::new(move || -> Box<dyn AnyComponent> {
            let new_instance = ComponentInstance::<C>::new(props.clone());
            Box::new(ComponentInstanceWrapper(new_instance))
        });

        Self {
            inner: Box::new(ComponentInstanceWrapper(instance)),
            factory,
            last_props: None,
        }
    }

    /// Update with type-erased props
    pub fn update(&mut self, props: &dyn Any) -> bool {
        self.last_props = None;
        self.inner.update_any(props)
    }

    /// Update with the shared props allocation an element carries; the same
    /// allocation as last time is known unchanged without comparing values.
    pub fn update_shared(&mut self, props: &Arc<dyn Any + Send + Sync>) -> bool {
        if self
            .last_props
            .as_ref()
            .is_some_and(|last| Arc::ptr_eq(last, props))
        {
            return false;
        }
        let changed = self.inner.update_any(props.as_ref());
        self.last_props = Some(props.clone());
        changed
    }

    /// Render the component
    pub fn render(&self) -> Element {
        self.inner.render_any()
    }

    /// Render while preserving errors across type erasure.
    pub fn try_render(&self) -> crate::error::Result<Element> {
        self.inner.try_render_any()
    }

    /// Deliver input to the retained typed instance.
    pub fn handle_event(
        &mut self,
        event: &crate::event::Event,
    ) -> crate::event::router::EventResult {
        self.inner.handle_event_any(event)
    }

    /// Deliver presented cell bounds to the retained typed instance.
    pub fn layout(&mut self, bounds: super::LayoutInfo) -> bool {
        self.inner.layout_any(bounds)
    }

    /// Handle lifecycle event
    pub fn on_lifecycle(&mut self, event: LifecycleEvent) {
        self.inner.on_lifecycle_any(event)
    }

    /// Check if mounted
    pub fn is_mounted(&self) -> bool {
        self.inner.is_mounted_any()
    }

    /// Get type ID
    pub fn type_id(&self) -> TypeId {
        AnyComponent::type_id(&*self.inner)
    }

    /// Check if this instance supports cloning
    ///
    /// Always returns true since we use the factory pattern for all instances
    pub fn is_clonable(&self) -> bool {
        true
    }
}

impl Clone for AnyComponentInstance {
    fn clone(&self) -> Self {
        // Creates a fresh component instance with the same props but clean state.
        // This is intentional - component instances should not share mutable state.
        // Each clone gets its own lifecycle and state management.
        Self {
            inner: (self.factory)(),
            factory: self.factory.clone(),
            last_props: None,
        }
    }
}

impl Drop for AnyComponentInstance {
    fn drop(&mut self) {
        // Safety net: ensure unmount lifecycle event is called
        // Only unmount if still mounted to prevent double unmount
        if self.is_mounted() {
            self.on_lifecycle(LifecycleEvent::Unmount);
        }
    }
}

/// Wrapper to make ComponentInstance implement AnyComponent
struct ComponentInstanceWrapper<C: Component>(ComponentInstance<C>);

impl<C: Component> AnyComponent for ComponentInstanceWrapper<C> {
    fn update_any(&mut self, props: &dyn Any) -> bool {
        if let Some(typed_props) = props.downcast_ref::<C::Props>() {
            // Compare before copying: a frame that changes nothing costs a
            // comparison, not a copy of the props (a chart's every point).
            if self.0.supplied_props == *typed_props {
                return false;
            }
            self.0.update_props(typed_props.clone())
        } else {
            false
        }
    }

    fn render_any(&self) -> Element {
        self.0.render()
    }

    fn try_render_any(&self) -> crate::error::Result<Element> {
        self.0.try_render()
    }

    fn handle_event_any(
        &mut self,
        event: &crate::event::Event,
    ) -> crate::event::router::EventResult {
        self.0.handle_event(event)
    }

    fn layout_any(&mut self, bounds: super::LayoutInfo) -> bool {
        self.0.layout(bounds)
    }

    fn poll_change_any(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // Enhanced safety implementation with runtime checks

        // First, verify pin stability with a simple pointer check
        let self_ptr = self.as_ref().get_ref() as *const ComponentInstanceWrapper<C>;
        let self_mut_ptr = self.as_ref().get_ref() as *const ComponentInstanceWrapper<C>;

        if self_ptr != self_mut_ptr {
            // Pin stability check failed - this should never happen but provides safety
            log::error!(
                "Pin stability check failed in poll_change_any - potential memory safety issue"
            );
            return Poll::Pending;
        }

        // SAFETY:
        // - ComponentInstanceWrapper is pinned by the caller when this method is invoked.
        // - We've verified pointer stability above as an additional safety check.
        // - We project the pin to the inner component field and promise not to move it
        //   while pinned. Component::poll_change requires a pinned receiver when needed.
        // - This wrapper type does not move the inner component after being pinned.
        // - The component field is at a fixed offset within the wrapper struct.
        unsafe {
            let wrapper = self.as_mut().get_unchecked_mut();
            let component_ptr = &mut wrapper.0.component as *mut C;

            // Additional safety: verify the component pointer is aligned and non-null
            if component_ptr.is_null()
                || !(component_ptr as usize).is_multiple_of(std::mem::align_of::<C>())
            {
                log::error!("Invalid component pointer in poll_change_any");
                return Poll::Pending;
            }

            // Project the pin safely to the component
            Pin::new_unchecked(&mut *component_ptr).poll_change(cx)
        }
    }

    fn on_lifecycle_any(&mut self, event: LifecycleEvent) {
        match event {
            LifecycleEvent::Mount => self.0.mount(),
            LifecycleEvent::Unmount => {
                // Only unmount if mounted to prevent double unmount
                if self.0.lifecycle.is_mounted() {
                    self.0.unmount();
                }
            }
            _ => self.0.component.on_lifecycle(event, &mut self.0.state),
        }
    }

    fn is_mounted_any(&self) -> bool {
        self.0.lifecycle.is_mounted()
    }

    fn type_id(&self) -> TypeId {
        Component::type_id(&self.0.component)
    }

    fn clone_box(&self) -> Box<dyn AnyComponent> {
        // Create a new instance with the same props
        let new_instance = ComponentInstance::<C>::new(self.0.props.clone());
        Box::new(ComponentInstanceWrapper(new_instance))
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}
