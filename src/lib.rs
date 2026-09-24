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
//! - **Widget Library**: Retained inputs, tables, trees, menus and dialogs
//! - **24-bit Color Support**: Full RGB color support for modern terminals
//! - **Image Rendering**: Multiple backends for displaying images in terminals
//! - **FFI Support**: Multi-language bindings via stable C ABI
//!
//! The repository's `manual/supported-api.md` maps supported routes to behavior
//! checks and platform limits. Catalog-wide acceptance remains subject to the
//! current API remediation commitment; compilation alone is not runtime evidence.
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
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(feature = "simd", feature(portable_simd))]

/// Accessible node semantics and the owned Linux screen-reader connection.
pub mod accessibility;
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

#[cfg(test)]
mod gate_proof;

// Re-export commonly used types
pub use error::{ReactiveError, Result};

// Re-export the macros
pub use reactive_tui_macros::{component, Props};

// Re-export CSS-in-Rust functionality
pub use crate::layout::css::css_in_rust::{apply_css_property, IntoCssValue};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::app::App;
    pub use crate::builder::*;
    pub use crate::component::{Component, Element, ElementType, LayoutType};
    pub use crate::error::{ReactiveError, Result};
    pub use crate::hooks::*;
    pub use crate::reactive::*;
    pub use crate::vdom::{
        context_menu, dialog_menu, menubar, popup_menu, simple_context_menu, simple_menubar,
        simple_popup_menu, VNode,
    };
    pub use crate::{component, Props};

    // CSS-in-Rust support
    pub use crate::layout::css::css_in_rust::{apply_css_property, IntoCssValue};
    pub use taffy::style::{AlignItems, Display, FlexDirection, JustifyContent, Position};
}

/// Owned Unix PTY sessions interpreted by libghostty.
#[cfg(all(feature = "embedded-terminal", unix))]
pub mod embedded;

/// Optional offscreen GPU graphics presented as ordinary terminal cells.
#[cfg(feature = "wgpu-graphics")]
pub mod graphics;
