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
pub mod layout;
pub mod mixed;
pub mod macros;
pub mod widgets;
pub mod placeholders;
pub mod dialog_builders;

#[cfg(test)]
mod test_placeholders;

// Re-export everything from core for backward compatibility
pub use core::*;

// Re-export layout helpers and convenience functions
pub use layout::*;

// Re-export VDOM integration
pub use mixed::*;

// Re-export all widget builders
pub use widgets::*;

// Re-export macros - these need special handling
pub use crate::{el, div, span, button, input, data_table, chart, text_input, checkbox, select, progress_bar, toast, tabs};

// Additional convenience re-exports for common patterns

/// Create a button element (alias for core button function)
pub use core::button;

/// Create an input element (alias for core input function)  
pub use core::input;

// Ensure all builder functions are available at the module root
// This maintains the existing API where users can call `builder::div()`, etc.

// Core HTML-like elements (from core module)
// pub use core::{div, span, p, section, article, header, footer, nav, main, aside, grid_builder, flex};

// Layout and convenience functions (from layout module)
// pub use layout::{screen, container, card_builder, sidebar, content, responsive_grid};
// pub use layout::{h1, h2, h3, h4, h5, h6, text, label, card};
// pub use layout::{primary_button, flex_row, flex_col, grid_layout, styled_input, search_input};

// VDOM integration (from mixed module)
// pub use mixed::{from_vdom, mixed_container};

// Widget builders (from widgets module)
// pub use widgets::{data_table, chart, modal, toast, popover, progress_bar, tree, image};
// pub use widgets::{text_input, checkbox, radio_button, select, slider};
// pub use widgets::{scroll_view, stack, tabs, dialog, confirmation_dialog, progress_dialog, wizard};

// Core traits and utilities
pub use core::{IntoElement, to_element};

/// Builder module version for compatibility tracking
pub const BUILDER_VERSION: &str = "2.0.0";

/// Module documentation and examples
///
/// # Architecture
///
/// The builder module is organized into several sub-modules:
///
/// - `core`: Core ElementBuilder and basic HTML elements
/// - `layout`: Layout helpers and convenience functions  
/// - `mixed`: VDOM integration for hybrid applications
/// - `macros`: Declarative macros for rapid development
/// - `widgets`: Specialized builders for complex components
///
/// # Migration Guide
///
/// This refactored module maintains 100% backward compatibility.
/// All existing code using the builder API will continue to work unchanged.
///
/// # Performance
///
/// The modular structure improves compile times by allowing selective imports
/// and reduces binary size through better dead code elimination.
pub mod docs {
    //! Documentation and examples for the builder API
    
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
                        sidebar()
                            .child(nav().text("Navigation").build())
                            .build(),
                            
                        content()
                            .children(vec![
                                card(vec![
                                    h2().text("Dashboard").build(),
                                    p().text("Welcome to your dashboard").build(),
                                ]),
                                
                                data_table()
                                    .column("Name", "name")
                                    .column("Email", "email")
                                    .simple_row(vec![("name", "John"), ("email", "john@example.com")])
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
