/// Menu system components for reactive-tui
///
/// This module provides a comprehensive menu system including:
/// - **menubar**: Traditional desktop-style menubar with dropdown submenus
/// - **context**: Right-click context menus
/// - **popup**: General-purpose popup menus
/// - **dialog**: Dialog-style menus for complex interactions

/// Menubar with dropdown submenus
mod menubar;
/// Context menu system
mod context;
/// General popup menus
mod popup;
/// Dialog-style menus
mod dialog;
/// Common menu item types and utilities
mod item;
/// Menu styling and theming
mod style;

pub use menubar::{MenuBar, MenuBarProps, MenuBarState, MenuBarBuilder};
pub use context::{ContextMenu, ContextMenuProps, ContextMenuState};
pub use popup::{PopupMenu, PopupMenuProps, PopupMenuState, PopupPlacement};
pub use dialog::{DialogMenu, DialogMenuProps, DialogMenuState, DialogMenuType};
pub use item::{MenuItem, MenuItemType, MenuSeparator, MenuAction, MenuShortcut};
pub use style::{MenuStyle, MenuTheme};