//! Menu system components for reactive-tui
//!
//! This module provides a comprehensive menu system including:
//! - **menubar**: Traditional desktop-style menubar with dropdown submenus
//! - **context**: Right-click context menus
//! - **popup**: General-purpose popup menus
//! - **dialog**: Dialog-style menus for complex interactions

/// Context menu system
mod context;
mod context_live;
/// Dialog-style menus
mod dialog;
mod dialog_live;
mod invocation;
/// Common menu item types and utilities
mod item;
/// Menubar with dropdown submenus
mod menubar;
mod menubar_live;
mod model;
mod panels;
/// General popup menus
mod popup;
mod popup_live;
pub(crate) use popup_live::RelativePlacement;
mod runtime;
mod state;
/// Menu styling and theming
mod style;
mod view;

pub use context::{ContextMenu, ContextMenuBuilder, ContextMenuProps, ContextMenuState};
pub use dialog::{DialogMenu, DialogMenuBuilder, DialogMenuProps, DialogMenuState, DialogMenuType};
pub use item::{MenuAction, MenuItem, MenuItemBuilder, MenuItemType, MenuSeparator, MenuShortcut};
pub use menubar::{MenuBar, MenuBarBuilder, MenuBarProps, MenuBarState};
pub use popup::{PopupMenu, PopupMenuBuilder, PopupMenuProps, PopupMenuState, PopupPlacement};
pub use style::{MenuStyle, MenuTheme};

// Shared callback signature for menu item IDs and submitted text.
type TextCallback = std::sync::Arc<dyn Fn(&str) + Send + Sync>;
