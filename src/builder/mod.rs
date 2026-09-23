//! Builder API for creating terminal UI elements
//!
//! This module provides a fluent, declarative API for building terminal UI components.
//! It includes core element builders, layout helpers, widget builders, and macros
//! for creating complex user interfaces with a simple, composable syntax.
//!
//! # Core Concepts
//!
//! - **ElementBuilder**: The foundation for creating any UI element
//! - **Layout Helpers**: Pre-styled components for common patterns
//! - **Widget Builders**: Specialized builders for complex components
//! - **Macros**: Declarative syntax for rapid UI development
//! - **VDOM Integration**: Seamless interop with virtual DOM components
//!
//! # Example
//!
//! ```rust
//! use reactive_tui::builder::*;
//!
//! let ui = div()
//!     .class("container")
//!     .children(vec![
//!         h1().text("Welcome").build(),
//!         p().text("This is a paragraph").build(),
//!         primary_button("Click me", || println!("Clicked!")),
//!     ])
//!     .build();
//! ```

// Core builder functionality
pub mod core;
pub mod dialog_builders;
pub mod layout;
pub mod macros;
pub mod mixed;
pub mod specialized;
pub mod widgets;

#[cfg(test)]
mod test_specialized;

// Re-export everything from core for backward compatibility
pub use core::*;

// Re-export layout helpers and convenience functions
pub use layout::*;

// Re-export VDOM integration
pub use mixed::*;

// Re-export all widget builders
pub use widgets::*;

// Re-export macros - these need special handling
pub use crate::{
    button, chart, checkbox, data_table, div, el, input, progress_bar, select, span, tabs,
    text_input, toast,
};

// Additional convenience re-exports for common patterns
// Note: button and input are already re-exported via `pub use core::*;`

// Ensure all builder functions are available at the module root
// This maintains the existing API where users can call `builder::div()`, etc.

// Core traits and utilities
pub use core::{to_element, IntoElement};

/// Builder module version for compatibility tracking
pub const BUILDER_VERSION: &str = "2.0.0";

pub mod docs {
    //! Module documentation and examples
    //!
    //! # Architecture
    //!
    //! The builder module is organized into several sub-modules:
    //!
    //! - `core`: Core ElementBuilder and basic HTML elements
    //! - `layout`: Layout helpers and convenience functions  
    //! - `mixed`: VDOM integration for hybrid applications
    //! - `macros`: Declarative macros for rapid development
    //! - `widgets`: Specialized builders for complex components
    //!

    /// Example of creating a complex layout
    pub fn example_layout() -> crate::component::Element {
        use super::*;

        screen()
            .children(vec![
                header()
                    .class("bg-blue-600 text-white p-4")
                    .child(h1().text("My App").build())
                    .build(),
                div()
                    .class("flex flex-1")
                    .children(vec![
                        sidebar().child(nav().text("Navigation").build()).build(),
                        content()
                            .children(vec![
                                card(vec![
                                    h2().text("Dashboard").build(),
                                    p().text("Welcome to your dashboard").build(),
                                ]),
                                data_table()
                                    .column("Name", "name")
                                    .column("Email", "email")
                                    .simple_row(vec![
                                        ("name", "John"),
                                        ("email", "john@example.com"),
                                    ])
                                    .build(),
                            ])
                            .build(),
                    ])
                    .build(),
                footer()
                    .class("bg-gray-100 p-4 text-center")
                    .child(text("© 2024 My App"))
                    .build(),
            ])
            .build()
    }

    /// Example of using macros for rapid development
    ///
    /// Note: This example is commented out to avoid macro import issues during compilation.
    /// The macros are available and work correctly when imported properly.
    pub fn example_macros_info() -> &'static str {
        "Macros like div!, data_table!, and chart! are available for rapid UI development"
    }
}
