/// Context menu system
mod context;
/// Dialog-style menus
mod dialog;
/// Common menu item types and utilities
mod item;
/// Menu system components for reactive-tui
///
/// This module provides a comprehensive menu system including:
/// - **menubar**: Traditional desktop-style menubar with dropdown submenus
/// - **context**: Right-click context menus
/// - **popup**: General-purpose popup menus
/// - **dialog**: Dialog-style menus for complex interactions

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
