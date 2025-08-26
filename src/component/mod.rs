use std::any::{Any, TypeId};
use std::pin::Pin;
use std::task::{Context, Poll};

pub mod instance;
pub mod props;
pub mod lifecycle;
pub mod registry;
pub mod element;
pub mod change;

pub use instance::ComponentInstance;
pub use props::Props;
pub use lifecycle::{Lifecycle, LifecycleEvent};
pub use registry::ComponentRegistry;
pub use element::{Element, ElementType, LayoutType};
pub mod bridge;
pub use bridge::element_to_nodespec;

pub use change::{Change, ChangeBatch, TextAttribute};

/// Core trait that all components must implement.
/// Components are the building blocks of the reactive TUI framework.
pub trait Component: Any + Send + Sync + 'static {
    /// The properties type for this component
    type Props: Props;

    /// The state type for this component
    type State: Default + Send + Sync + 'static;

    /// Create a new instance of the component with the given props
    fn new(props: Self::Props) -> Self;

    /// Update the component with new props and state.
    /// Returns true if the component needs to re-render.
    fn update(&mut self, _props: &Self::Props, _state: &mut Self::State) -> bool {
        // Default implementation always re-renders
        true
    }

    /// Render the component into an Element tree
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element;

    /// Poll for async state changes.
    /// Components can implement this to handle async operations.
    fn poll_change(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        // Default implementation never changes
        Poll::Pending
    }

    /// Handle events
    fn handle_event(&mut self, _event: &crate::event::Event, _props: &mut Self::Props, _state: &mut Self::State) -> crate::event::router::EventResult {
        // Default implementation ignores events
        crate::event::router::EventResult::Ignored
    }

    /// Handle lifecycle events
    fn on_lifecycle(&mut self, _event: LifecycleEvent, _state: &mut Self::State) {
        // Default implementation does nothing
    }

    /// Get the type ID of this component
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// A type-erased component that can be stored in collections
pub trait AnyComponent: Any + Send + Sync {
    /// Update the component with type-erased props
    fn update_any(&mut self, props: &dyn Any) -> bool;

    /// Render the component
    fn render_any(&self) -> Element;

    /// Poll for changes
    fn poll_change_any(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>;

    /// Handle lifecycle events
    fn on_lifecycle_any(&mut self, event: LifecycleEvent);

    /// Get the type ID
    fn type_id(&self) -> TypeId;

    /// Downcast to a concrete type
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}