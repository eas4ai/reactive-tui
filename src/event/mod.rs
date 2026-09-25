//! Event system for terminal input and user interactions
//!
//! This module provides a complete event handling system including keyboard,
//! mouse, and custom event types with routing and focus management.

/// Performance enhancements for event routing
pub mod cache;
/// Focus management and keyboard navigation
pub mod focus;
/// Hit testing for mouse events and click detection
pub mod hit;
pub(crate) mod notifications;
/// Event routing and dispatching to handlers
pub mod router;
/// Core event types (keyboard, mouse, resize, etc.)
pub mod types;

pub use focus::{FocusDirection, FocusManager};
pub use hit::{Bounds, HitTest, Point};
pub use router::{EventHandler, EventPhase, EventRouter};
pub use types::{CustomEvent, Event, FocusEvent, KeyEvent, MouseEvent, PasteEvent, ResizeEvent};
