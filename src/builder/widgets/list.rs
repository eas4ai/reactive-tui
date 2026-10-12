//! The list builder (DAT-006 to DAT-009): `builder::list()`. Its element is
//! a `List` with the builder's callbacks handed to it.

use crate::component::Element;
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
        let on_select = self.on_select;
        let on_confirm = self.on_confirm;
        let on_cancel = self.on_cancel;
        let on_load_more = self.on_load_more;
        let mut element = Element::typed_with::<List>(self.props, move |props| {
            let mut data = List::new(props);
            data.on_select = on_select.clone();
            data.on_confirm = on_confirm.clone();
            data.on_cancel = on_cancel.clone();
            data.on_load_more = on_load_more.clone();
            data
        });
        if let Some(class) = self.class {
            element = element.with_class(&class);
        }
        element
    }
}
