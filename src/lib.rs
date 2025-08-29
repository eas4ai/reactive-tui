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
pub mod reactive;
pub mod render;
pub mod screen;
pub mod syntax;
pub mod theme;
pub mod ui;
pub mod vdom;
pub mod widgets;

// Re-export commonly used types
pub use error::{RTuiError, Result};
