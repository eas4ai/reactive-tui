//! Virtual DOM implementation for efficient UI updates
//!
//! This module provides a virtual DOM system for efficient diffing and patching
//! of UI trees, enabling React-like declarative rendering with minimal updates.

/// Bridge between virtual DOM and component system
pub mod bridge;
/// Virtual DOM diffing algorithm
pub mod diff;
/// Menu component helpers for VDOM
pub mod menu;
/// Virtual DOM node types and structures
pub mod node;
/// Patch operations for updating the DOM
pub mod patch;

pub use diff::{diff_vnodes, DiffContext};
pub use menu::{
    context_menu, dialog_menu, menubar, popup_menu, simple_context_menu, simple_menubar,
    simple_popup_menu,
};
pub use node::{VComponent, VElement, VElementProps, VFragment, VNode, VNodeKey, VNodeType, VText};
pub use patch::{apply_patches, Patch, PatchList};

/// Re-export commonly used items
pub mod prelude {
    pub use super::diff::diff_vnodes;
    pub use super::node::{VComponent, VElement, VElementProps, VFragment, VNode, VText};
    pub use super::patch::{apply_patches, Patch};
}
