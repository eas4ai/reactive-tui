pub mod animation;
pub mod app;
pub mod backend;
pub mod component;
pub mod core;
pub mod display;
pub mod editor;
pub mod error;
pub mod escape;
pub mod event;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod hooks;
pub mod layout;
#[cfg(not(any(doctest, doc)))]
pub mod markdown;
pub mod platform;
pub mod reactive;
pub mod render;
pub mod screen;
pub mod syntax;
pub mod terminal;
pub mod theme;
pub mod ui;
pub mod vdom;
pub mod web_api;
pub mod widgets;

// Re-export commonly used types
pub use error::{ReactiveError, Result};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::app::App;
    pub use crate::component::{Component, Element, ElementType, LayoutType};
    pub use crate::error::{ReactiveError, Result};
    pub use crate::hooks::*;
    pub use crate::reactive::*;
    pub use crate::web_api::*;
}
