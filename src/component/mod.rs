//! Component system for building reactive terminal user interfaces
//!
//! This module provides a React-like component architecture for terminal applications,
//! including elements, lifecycle management, props, and component registry.

use std::any::{Any, TypeId};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Performance optimizations for component operations
pub mod cache;
/// Component change detection and tracking
pub mod change;
/// Element types and builders for the component tree
pub mod element;
/// Declarative focus management system
pub mod focus;
/// Component instance management and rendering
pub mod instance;
/// Component lifecycle events and hooks
pub mod lifecycle;
/// Component properties system with derive macros
pub mod props;
/// Global component registry for dynamic component creation
pub mod registry;
pub(crate) mod runtime;
/// Tracked component instances with automatic cleanup
pub mod tracked_instance;

pub use element::{Element, ElementMetadata, ElementType, LayoutType};
pub use focus::{FocusProps, FocusPropsBuilder};
pub use instance::{AnyComponentInstance, ComponentInstance};
pub use lifecycle::{Lifecycle, LifecycleEvent};
pub use props::Props;
pub use registry::ComponentRegistry;
pub use tracked_instance::{SharedTrackedInstance, TrackedComponentInstance};
pub(crate) mod anchors;
/// Bridge between component elements and layout system
pub mod bridge;
mod builtin;
mod layout_info;
pub use layout_info::LayoutInfo;
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

    /// Initialize the state owned by an application instance.
    fn initial_state(&mut self, _props: &Self::Props) -> Self::State {
        Self::State::default()
    }

    /// Update the component with new props and state.
    /// Returns true if the component needs to re-render.
    fn update(&mut self, _props: &Self::Props, _state: &mut Self::State) -> bool {
        // Default implementation always re-renders
        true
    }

    /// Render the component into an Element tree
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element;

    /// Render with an observable failure. Existing components retain their
    /// infallible render implementation through this compatibility default.
    fn try_render(
        &self,
        props: &Self::Props,
        state: &Self::State,
    ) -> crate::error::Result<Element> {
        Ok(self.render(props, state))
    }

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

    /// Receive layout and placement from the last presented frame, in terminal cells.
    /// Return true when the new dimensions require another render.
    fn layout(
        &mut self,
        _layout: LayoutInfo,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        false
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

    /// Fallible rendering used by the application runtime.
    fn try_render_any(&self) -> crate::error::Result<Element> {
        Ok(self.render_any())
    }

    /// Dispatch input to the retained component state.
    fn handle_event_any(
        &mut self,
        _event: &crate::event::Event,
    ) -> crate::event::router::EventResult {
        crate::event::router::EventResult::Ignored
    }

    /// Notify a retained component of its presented cell bounds.
    fn layout_any(&mut self, _layout: LayoutInfo) -> bool {
        false
    }

    /// Poll for changes
    fn poll_change_any(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>;

    /// Handle lifecycle events
    fn on_lifecycle_any(&mut self, event: LifecycleEvent);

    /// Get the type ID
    fn type_id(&self) -> TypeId;

    /// Clone this component into a boxed trait object
    fn clone_box(&self) -> Box<dyn AnyComponent>;

    /// Downcast to a concrete type
    /// Get a reference to the component as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    /// Get a mutable reference to the component as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Check if the component is mounted
    fn is_mounted_any(&self) -> bool;
}
