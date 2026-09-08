//! Menu system components for reactive-tui
//!
//! This module provides a comprehensive menu system including:
//! - **menubar**: Traditional desktop-style menubar with dropdown submenus
//! - **context**: Right-click context menus
//! - **popup**: General-purpose popup menus
//! - **dialog**: Dialog-style menus for complex interactions

/// Context menu system
mod context;
/// Dialog-style menus
mod dialog;
/// Common menu item types and utilities
mod item;
/// Menubar with dropdown submenus
mod menubar;
/// General popup menus
mod popup;
/// Menu styling and theming
mod style;

pub use context::{ContextMenu, ContextMenuProps, ContextMenuState};
pub use dialog::{DialogMenu, DialogMenuProps, DialogMenuState, DialogMenuType};
pub use item::{MenuAction, MenuItem, MenuItemType, MenuSeparator, MenuShortcut};
pub use menubar::{MenuBar, MenuBarBuilder, MenuBarProps, MenuBarState};
pub use popup::{PopupMenu, PopupMenuProps, PopupMenuState, PopupPlacement};
pub use style::{MenuStyle, MenuTheme};

// Shared callback signature for menu item IDs and submitted text.
type TextCallback = std::sync::Arc<dyn Fn(&str) + Send + Sync>;
