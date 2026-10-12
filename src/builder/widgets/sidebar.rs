//! The sidebar builder (NAV-009): `builder::sidebar()`. Its element is a
//! `Sidebar` with the builder's callbacks handed to it.

use crate::component::Element;
use crate::widgets::layout::sidebar::{CollapseCallback, SelectCallback};
use crate::widgets::layout::{Sidebar, SidebarItem, SidebarProps};
use std::sync::Arc;

/// Builder for sidebars.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::sidebar;
/// use reactive_tui::widgets::layout::SidebarItem;
///
/// let nav = sidebar()
///     .item(SidebarItem::new("home", "Home"))
///     .item(SidebarItem::new("settings", "Settings"))
///     .active("home")
///     .aria_label("Main")
///     .build();
/// ```
#[derive(Clone, Default)]
pub struct SidebarBuilder {
    props: SidebarProps,
    class: Option<String>,
    on_select: Option<SelectCallback>,
    on_collapse: Option<CollapseCallback>,
}

/// Create a sidebar builder.
pub fn sidebar() -> SidebarBuilder {
    SidebarBuilder::new()
}

impl SidebarBuilder {
    /// Create a builder with no items.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a top-level item.
    pub fn item(mut self, item: SidebarItem) -> Self {
        self.props.items.push(item);
        self
    }

    /// Replace the top-level items.
    pub fn items(mut self, items: Vec<SidebarItem>) -> Self {
        self.props.items = items;
        self
    }

    /// Set the id of the active item.
    pub fn active(mut self, id: &str) -> Self {
        self.props.active = Some(id.to_string());
        self
    }

    /// Set whether the bar is collapsed to its strip of icons.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.props.collapsed = collapsed;
        self
    }

    /// Set an element shown above the items.
    pub fn header(mut self, header: Element) -> Self {
        self.props.header = Some(header);
        self
    }

    /// Set an element shown below the items.
    pub fn footer(mut self, footer: Element) -> Self {
        self.props.footer = Some(footer);
        self
    }

    /// Set the name the screen reader hears for the bar.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.props.aria_label = Some(label.to_string());
        self
    }

    /// Set the CSS-like class names of the element.
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the callback called with the id of an item made active.
    pub fn on_select(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the collapsed state after a flip.
    pub fn on_collapse(mut self, f: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_collapse = Some(Arc::new(f));
        self
    }

    /// Build the sidebar's element.
    pub fn build(self) -> Element {
        let on_select = self.on_select;
        let on_collapse = self.on_collapse;
        let mut element = Element::typed_with::<Sidebar>(self.props, move |props| {
            let mut bar = Sidebar::new(props);
            bar.on_select = on_select.clone();
            bar.on_collapse = on_collapse.clone();
            bar
        });
        if let Some(class) = self.class {
            element = element.with_class(&class);
        }
        element
    }
}
