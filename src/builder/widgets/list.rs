//! The list builder (DAT-006 to DAT-009): `builder::list()`. Its element is
//! a `List` with the builder's callbacks handed to it, so a rebuild with new
//! callbacks keeps the mounted list and acts on the new ones (CMP-008,
//! CMP-009).

use crate::component::{same_callback, Component, Element, LayoutInfo};
use crate::event::router::EventResult;
use crate::widgets::display::list::{IdCallback, UnitCallback};
use crate::widgets::display::{List, ListItem, ListProps};
use std::sync::Arc;

/// Builder for lists.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::list;
/// use reactive_tui::widgets::display::ListItem;
///
/// let fruit = list()
///     .item(ListItem::new("apple", "Apple"))
///     .item(ListItem::new("banana", "Banana"))
///     .selected("banana")
///     .filterable(true)
///     .aria_label("Fruit")
///     .build();
/// ```
#[derive(Clone, Default)]
pub struct ListBuilder {
    props: ListProps,
    class: Option<String>,
    on_select: Option<IdCallback>,
    on_confirm: Option<IdCallback>,
    on_cancel: Option<UnitCallback>,
    on_load_more: Option<UnitCallback>,
}

/// Equal over the settings and not the callbacks, so a rebuild that changes
/// only a callback keeps the mounted list and hands it the new one
/// (CMP-008).
impl PartialEq for ListBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.props == other.props && self.class == other.class
    }
}

impl crate::component::Props for ListBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Create a list builder.
pub fn list() -> ListBuilder {
    ListBuilder::new()
}

impl ListBuilder {
    /// Create a builder with no items.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item at the end.
    pub fn item(mut self, item: ListItem) -> Self {
        self.props.items.push(item);
        self
    }

    /// Replace the items.
    pub fn items(mut self, items: Vec<ListItem>) -> Self {
        self.props.items = items;
        self
    }

    /// Set the id of the selected item.
    pub fn selected(mut self, id: &str) -> Self {
        self.props.selected = Some(id.to_string());
        self
    }

    /// Set whether a filter row is shown on top of the list.
    pub fn filterable(mut self, on: bool) -> Self {
        self.props.filterable = on;
        self
    }

    /// Set whether the application has more items to give the list.
    pub fn has_more(mut self, on: bool) -> Self {
        self.props.has_more = on;
        self
    }

    /// Set whether a loading row is shown at the end.
    pub fn loading(mut self, on: bool) -> Self {
        self.props.loading = on;
        self
    }

    /// Set the message shown when no item shows.
    pub fn empty_text(mut self, text: &str) -> Self {
        self.props.empty_text = Some(text.to_string());
        self
    }

    /// Set the name the screen reader hears for the list.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.props.aria_label = Some(label.to_string());
        self
    }

    /// Set the list's width in cells.
    pub fn width(mut self, width: u16) -> Self {
        self.props.width = Some(width);
        self
    }

    /// Set the list's height in cells.
    pub fn height(mut self, height: u16) -> Self {
        self.props.height = Some(height);
        self
    }

    /// Set the CSS-like class names of the element.
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the callback called with the id of the item the cursor moved to.
    pub fn on_select(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the id of a confirmed item.
    pub fn on_confirm(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_confirm = Some(Arc::new(f));
        self
    }

    /// Set the callback called when Escape cancels the list.
    pub fn on_cancel(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(f));
        self
    }

    /// Set the callback called when the list asks for more items.
    pub fn on_load_more(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_load_more = Some(Arc::new(f));
        self
    }

    /// Build the list's element.
    pub fn build(self) -> Element {
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredList>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }
        element
    }

    /// Hands the builder's callbacks to `list`.
    fn configure(&self, list: &mut List) {
        list.on_select = self.on_select.clone();
        list.on_confirm = self.on_confirm.clone();
        list.on_cancel = self.on_cancel.clone();
        list.on_load_more = self.on_load_more.clone();
    }
}

/// A list built through its builder: the widget itself, with the builder's
/// callbacks handed to it (CMP-009).
struct ConfiguredList(List);

impl Component for ConfiguredList {
    type Props = ListBuilder;
    type State = ();

    fn new(props: Self::Props) -> Self {
        let mut inner = List::new(props.props.clone());
        props.configure(&mut inner);
        Self(inner)
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        props.configure(&mut self.0);
        Component::update(&mut self.0, &props.props, state)
    }

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        let changed = !same_callback(&props.on_select, &supplied.on_select)
            || !same_callback(&props.on_confirm, &supplied.on_confirm)
            || !same_callback(&props.on_cancel, &supplied.on_cancel)
            || !same_callback(&props.on_load_more, &supplied.on_load_more);
        if changed {
            props.on_select = supplied.on_select.clone();
            props.on_confirm = supplied.on_confirm.clone();
            props.on_cancel = supplied.on_cancel.clone();
            props.on_load_more = supplied.on_load_more.clone();
        }
        changed
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.0.render(&props.props, state)
    }

    fn layout(
        &mut self,
        layout: LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        // The callbacks a rebuild adopted act from here on (CMP-008).
        props.configure(&mut self.0);
        self.0.layout(layout, &mut props.props, state)
    }

    fn handle_event(
        &mut self,
        event: &crate::event::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        props.configure(&mut self.0);
        self.0.handle_event(event, &mut props.props, state)
    }
}
