//! Component system for building reactive terminal user interfaces
//!
//! This module provides a React-like component architecture for terminal applications,
//! including elements, lifecycle management, props, and component registry.

use std::any::{Any, TypeId};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Component change detection and tracking
pub mod change;
/// Element types and builders for the component tree
pub mod element;
/// Component instance management and rendering
pub mod instance;
/// Component lifecycle events and hooks
pub mod lifecycle;
/// Component properties system with derive macros
pub mod props;
/// Global component registry for dynamic component creation
pub mod registry;

pub use element::{Element, ElementType, LayoutType};
pub use instance::ComponentInstance;
pub use lifecycle::{Lifecycle, LifecycleEvent};
pub use props::Props;
pub use registry::ComponentRegistry;
/// Bridge between component elements and layout system
pub mod bridge;
/// State management flags for component lifecycle
pub mod state_flags;

pub use bridge::element_to_nodespec;
pub use change::{Change, ChangeBatch, TextAttribute};
pub use state_flags::{ComponentState, StateFlagged, StateFlags};

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
    fn handle_event(
        &mut self,
        _event: &crate::event::Event,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> crate::event::router::EventResult {
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
    /// Get a reference to the component as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    /// Get a mutable reference to the component as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
