//! The sidebar builder (NAV-007 to NAV-010): `builder::sidebar()`. Its
//! element is a `Sidebar` with the builder's callbacks handed to it, so a
//! rebuild with new callbacks keeps the mounted sidebar (CMP-008,
//! CMP-009).

use crate::component::{same_callback, Component, Element, LayoutInfo, Props};
use crate::event::router::EventResult;
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

/// Equal over the settings and not the callbacks, so a rebuild that changes
/// only a callback keeps the mounted sidebar and hands it the new one
/// (CMP-008).
impl PartialEq for SidebarBuilder {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            props,
            class,
            on_select: _,
            on_collapse: _,
        } = self;
        *props == other.props && *class == other.class
    }
}

impl Props for SidebarBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
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

    /// Set an element shown above the items while the bar is expanded.
    pub fn header(mut self, header: Element) -> Self {
        self.props.header = Some(header);
        self
    }

    /// Set an element shown below the items while the bar is expanded.
    pub fn footer(mut self, footer: Element) -> Self {
        self.props.footer = Some(footer);
        self
    }

    /// Set the name the screen reader hears for the bar and its toggle.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.props.aria_label = Some(label.to_string());
        self
    }

    /// Set the CSS-like class names of the element, such as `w-40` or
    /// `h-10`, which size the bar in place of its default.
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the callback called with the id of an item made active, as
    /// `Sidebar::with_on_select` is (CMP-009).
    pub fn on_select(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the collapsed state after a flip, as
    /// `Sidebar::with_on_collapse` is (CMP-009).
    pub fn on_collapse(mut self, f: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_collapse = Some(Arc::new(f));
        self
    }

    /// Build the sidebar's element.
    pub fn build(self) -> Element {
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredSidebar>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }
        element
    }
}

/// A sidebar built through its builder: the widget itself, with the
/// builder's callbacks handed to it (CMP-009).
struct ConfiguredSidebar(Sidebar);

impl ConfiguredSidebar {
    /// Hands the inner sidebar the callbacks the props carry now.
    fn adopt(&mut self, props: &SidebarBuilder) {
        self.0.set_on_select(props.on_select.clone());
        self.0.set_on_collapse(props.on_collapse.clone());
    }
}

impl Component for ConfiguredSidebar {
    type Props = SidebarBuilder;
    type State = ();

    fn new(props: Self::Props) -> Self {
        let mut configured = Self(Sidebar::new(props.props.clone()));
        configured.adopt(&props);
        configured
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        self.0.update(&props.props, state)
    }

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        let select = !same_callback(&props.on_select, &supplied.on_select);
        let collapse = !same_callback(&props.on_collapse, &supplied.on_collapse);
        if select {
            props.on_select = supplied.on_select.clone();
        }
        if collapse {
            props.on_collapse = supplied.on_collapse.clone();
        }
        select || collapse
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.0.render(&props.props, state)
    }

    fn layout(
        &mut self,
        bounds: LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        self.0.layout(bounds, &mut props.props, state)
    }

    fn handle_event(
        &mut self,
        event: &crate::event::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        // The callbacks a rebuild adopted act from this event on (CMP-008).
        self.adopt(props);
        self.0.handle_event(event, &mut props.props, state)
    }
}

impl From<SidebarBuilder> for Element {
    fn from(builder: SidebarBuilder) -> Self {
        builder.build()
    }
}
