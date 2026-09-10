//! VDOM helpers for menu components
//!
//! This module provides convenient functions for creating menu components
//! in the VDOM system, allowing for declarative menu creation.

use super::node::{VComponent, VNode};
use crate::widgets::menu::{
    ContextMenuProps, DialogMenuProps, MenuBarProps, MenuItem, MenuStyle, PopupMenuProps,
    PopupPlacement,
};

/// Create a MenuBar component in VDOM
pub fn menubar() -> VComponent {
    VComponent::new("MenuBar")
}

/// Create a ContextMenu component in VDOM
pub fn context_menu() -> VComponent {
    VComponent::new("ContextMenu")
}

/// Create a PopupMenu component in VDOM
pub fn popup_menu() -> VComponent {
    VComponent::new("PopupMenu")
}

/// Create a DialogMenu component in VDOM
pub fn dialog_menu() -> VComponent {
    VComponent::new("DialogMenu")
}

/// Create a MenuBar VNode with props
pub fn menubar_with_props(props: MenuBarProps) -> VNode {
    VNode::Component(VComponent::new("MenuBar").props(props))
}

/// Create a ContextMenu VNode with props
pub fn context_menu_with_props(props: ContextMenuProps) -> VNode {
    VNode::Component(VComponent::new("ContextMenu").props(props))
}

/// Create a PopupMenu VNode with props
pub fn popup_menu_with_props(props: PopupMenuProps) -> VNode {
    VNode::Component(VComponent::new("PopupMenu").props(props))
}

/// Create a DialogMenu VNode with props
pub fn dialog_menu_with_props(props: DialogMenuProps) -> VNode {
    VNode::Component(VComponent::new("DialogMenu").props(props))
}

/// Convenience function to create a simple menubar with items
pub fn simple_menubar(items: Vec<MenuItem>) -> VNode {
    let props = MenuBarProps {
        items,
        style: MenuStyle::default(),
        visible: true,
        enabled: true,
        title: None,
        show_shortcuts: true,
        max_dropdown_height: 10,
    };
    menubar_with_props(props)
}

/// Convenience function to create a simple context menu with items
pub fn simple_context_menu(items: Vec<MenuItem>) -> VNode {
    let props = ContextMenuProps {
        items,
        style: MenuStyle::default(),
        enabled: true,
        show_on_right_click: true,
        show_on_long_press: false,
        long_press_duration: 500,
        close_on_outside_click: true,
        max_visible_items: 10,
        width: None,
        show_border: true,
        show_shadow: true,
        trigger_areas: vec![],
    };
    context_menu_with_props(props)
}

/// Convenience function to create a simple popup menu with items
pub fn simple_popup_menu(items: Vec<MenuItem>, placement: PopupPlacement) -> VNode {
    let props = PopupMenuProps {
        items,
        style: MenuStyle::default(),
        visible: true,
        enabled: true,
        placement,
        auto_close: true,
        close_on_outside_click: true,
        max_visible_items: 10,
        width: None,
        show_border: true,
        show_shadow: true,
    };
    popup_menu_with_props(props)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::menu::{MenuItem, MenuItemType, MenuSeparator};

    #[test]
    fn test_menubar_creation() {
        let items = vec![MenuItem {
            id: "file".to_string(),
            text: "File".to_string(),
            item_type: MenuItemType::Action,
            enabled: true,
            visible: true,
            shortcut: None,
            action: None,
            submenu: vec![],
            separator: MenuSeparator::None,
            icon: None,
            description: None,
        }];

        let vnode = simple_menubar(items);
        match vnode {
            VNode::Component(comp) => {
                assert_eq!(comp.name, "MenuBar");
            }
            _ => panic!("Expected component node"),
        }
    }

    #[test]
    fn test_context_menu_creation() {
        let items = vec![MenuItem {
            id: "cut".to_string(),
            text: "Cut".to_string(),
            item_type: MenuItemType::Action,
            enabled: true,
            visible: true,
            shortcut: None,
            action: None,
            submenu: vec![],
            separator: MenuSeparator::None,
            icon: None,
            description: None,
        }];

        let vnode = simple_context_menu(items);
        match vnode {
            VNode::Component(comp) => {
                assert_eq!(comp.name, "ContextMenu");
            }
            _ => panic!("Expected component node"),
        }
    }

    #[test]
    fn test_popup_menu_creation() {
        let items = vec![MenuItem {
            id: "option1".to_string(),
            text: "Option 1".to_string(),
            item_type: MenuItemType::Action,
            enabled: true,
            visible: true,
            shortcut: None,
            action: None,
            submenu: vec![],
            separator: MenuSeparator::None,
            icon: None,
            description: None,
        }];

        let vnode = simple_popup_menu(items, PopupPlacement::Cursor);
        match vnode {
            VNode::Component(comp) => {
                assert_eq!(comp.name, "PopupMenu");
            }
            _ => panic!("Expected component node"),
        }
    }
}
