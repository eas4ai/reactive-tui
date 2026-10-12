//! The sidebar (docs/spec/layout-widgets.md, NAV-009): a tree of items in a
//! bar at the side of the screen, which collapses to a strip of icons.
//!
//! This is the skeleton of the widget: its props and callbacks are real,
//! and it renders an empty element until the rendering is written.

use crate::component::{Component, Element, Props};
use crate::widgets::display::pieces::icon::Icon;
use std::any::Any;
use std::sync::Arc;

/// The callback called with the id of the item the user made active.
pub(crate) type SelectCallback = Arc<dyn Fn(String) + Send + Sync>;

/// The callback called with the collapsed state after the user flips it.
pub(crate) type CollapseCallback = Arc<dyn Fn(bool) + Send + Sync>;

/// One item of a sidebar. An item with children is a group.
#[derive(Clone, Debug, PartialEq)]
pub struct SidebarItem {
    /// The id the item is known by to the application.
    pub id: String,
    /// The text the user reads.
    pub label: String,
    /// The icon shown before the label, if any.
    pub icon: Option<Icon>,
    /// A short text shown at the end of the row, if any.
    pub badge: Option<String>,
    /// Whether the item ignores the keyboard and the pointer.
    pub disabled: bool,
    /// Whether a group starts open.
    pub open: bool,
    /// The item's children; an item with any is a group.
    pub children: Vec<SidebarItem>,
}

impl SidebarItem {
    /// Create an item with an id and a label, with no icon or badge.
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon: None,
            badge: None,
            disabled: false,
            open: false,
            children: Vec::new(),
        }
    }

    /// Set the icon shown before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the badge text shown at the end of the row.
    pub fn badge(mut self, text: &str) -> Self {
        self.badge = Some(text.to_string());
        self
    }

    /// Set whether the item is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether a group starts open.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Add a child item, which makes this item a group.
    pub fn child(mut self, item: SidebarItem) -> Self {
        self.children.push(item);
        self
    }
}

/// Properties for the sidebar.
#[derive(Clone, Default, PartialEq)]
pub struct SidebarProps {
    /// The top-level items, in order.
    pub items: Vec<SidebarItem>,
    /// The id of the active item, if any.
    pub active: Option<String>,
    /// Whether the bar is collapsed to its strip of icons.
    pub collapsed: bool,
    /// An element shown above the items, if any.
    pub header: Option<Element>,
    /// An element shown below the items, if any.
    pub footer: Option<Element>,
    /// The name the screen reader hears for the bar.
    pub aria_label: Option<String>,
    /// The bar's width in cells; none is 28 cells, or 3 while collapsed.
    pub width: Option<u16>,
}

impl Props for SidebarProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A sidebar: a tree of items that the user moves through and makes active.
pub struct Sidebar {
    /// Called with the id of an item the user makes active.
    pub(crate) on_select: Option<SelectCallback>,
    /// Called with the collapsed state after the user flips it.
    pub(crate) on_collapse: Option<CollapseCallback>,
}

impl Sidebar {
    /// Create a sidebar from its props, with no callbacks yet.
    pub fn new(_props: SidebarProps) -> Self {
        Self {
            on_select: None,
            on_collapse: None,
        }
    }

    /// Set the callback called with the id of an item made active.
    pub fn with_on_select(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the collapsed state after a flip.
    pub fn with_on_collapse(mut self, f: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_collapse = Some(Arc::new(f));
        self
    }
}

impl Component for Sidebar {
    type Props = SidebarProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn render(&self, _props: &Self::Props, _state: &Self::State) -> Element {
        // Skeleton: the sidebar's rendering is written by the widget's job.
        crate::builder::div().build()
    }
}
