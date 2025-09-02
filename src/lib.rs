//! # Reactive TUI
//!
//! A modern, reactive terminal user interface library for Rust that brings React-like
//! component architecture and CSS-style layouts to terminal applications.
//!
//! ## Features
//!
//! - **React-like Components**: Declarative component system with hooks and lifecycle methods
//! - **CSS Utility Classes**: Tailwind-style classes for layout and styling
//! - **Flexbox/Grid Layouts**: Modern layout system powered by the Taffy engine
//! - **Rich Widget Library**: Comprehensive set of production-ready widgets
//! - **24-bit Color Support**: Full RGB color support for modern terminals
//! - **Image Rendering**: Multiple backends for displaying images in terminals
//! - **FFI Support**: Multi-language bindings via stable C ABI
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use reactive_tui::prelude::*;
//! use reactive_tui::backend::DebugBackend;
//!
//! struct MyComponent;
//!
//! impl reactive_tui::app::RootComponent for MyComponent {
//!     fn render(&self) -> Element {
//!         Element::text("Hello, World!")
//!     }
//! }
//!
//! fn main() -> Result<()> {
//!     let app = App::builder()
//!         .backend(DebugBackend::new(80, 24))
//!         .root(MyComponent)
//!         .build()?;
//!     // Your app logic here
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

/// Animation system with keyframes, springs, and easing functions
pub mod animation;
/// Application framework for building TUI applications
pub mod app;
/// Terminal backend implementations for different platforms
pub mod backend;
/// Builder patterns and utilities for constructing UI elements
pub mod builder;
/// React-like component system with lifecycle and state management
pub mod component;
/// Core rendering primitives and terminal abstractions
pub mod core;
/// Display utilities and formatting helpers
pub mod display;
/// Text editor components with syntax highlighting support
pub mod editor;
/// Error types and result handling for the library
pub mod error;
/// Terminal escape sequence handling and parsing
pub mod escape;
/// Event system for keyboard, mouse, and terminal events
pub mod event;
/// Foreign Function Interface for multi-language bindings
#[cfg(feature = "ffi")]
pub mod ffi;
/// React-style hooks for state and lifecycle management
pub mod hooks;
/// CSS-like layout system with flexbox and grid support
pub mod layout;
/// Markdown rendering and parsing for terminal display
#[cfg(not(any(doctest, doc)))]
pub mod markdown;
/// Platform-specific implementations and capabilities detection
pub mod platform;
/// Reactive state management and data flow
pub mod reactive;
/// Rendering pipeline and optimization utilities
pub mod render;
/// Screen management and buffer handling
pub mod screen;
/// Syntax highlighting support for code display
pub mod syntax;
/// Terminal control, capabilities, and raw mode management
pub mod terminal;
/// Theming system with color schemes and styling
pub mod theme;
/// High-level UI utilities and helpers
pub mod ui;
/// Virtual DOM implementation for efficient updates
pub mod vdom;

/// Comprehensive widget library including dialogs, inputs, and displays
pub mod widgets;

// Re-export commonly used types
pub use error::{ReactiveError, Result};

// Re-export the macros
pub use reactive_tui_macros::{component, Props};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::app::App;
    pub use crate::builder::*;
    pub use crate::component::{Component, Element, ElementType, LayoutType};
    pub use crate::error::{ReactiveError, Result};
    pub use crate::hooks::*;
    pub use crate::reactive::*;
    pub use crate::{component, Props};
}
