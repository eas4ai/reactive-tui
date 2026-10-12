//! The list (docs/spec/data-widgets.md, DAT-006 to DAT-009): a scrolling
//! list of items with optional section titles, a filter row, a loading row
//! and an empty message, with one focus stop.
//!
//! This is the skeleton of the widget: its props and callbacks are real,
//! and it renders an empty element until the rendering is written.

use crate::component::{Component, Element, Props};
use std::any::Any;
use std::sync::Arc;

/// The callback called with an item's id.
pub(crate) type IdCallback = Arc<dyn Fn(String) + Send + Sync>;

/// The callback called with no arguments.
pub(crate) type UnitCallback = Arc<dyn Fn() + Send + Sync>;

/// One item of a list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListItem {
    /// The id the item is known by to the application.
    pub id: String,
    /// The text the user reads.
    pub label: String,
    /// A text shown after the label, in the muted color, if any.
    pub description: Option<String>,
    /// The title of the section the item sits under, if any. Consecutive
    /// items with one section share one title row.
    pub section: Option<String>,
    /// Whether the item is skipped by the cursor.
    pub disabled: bool,
}

impl ListItem {
    /// Create an item with an id and a label.
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            ..Self::default()
        }
    }

    /// Set the description shown after the label.
    pub fn description(mut self, text: &str) -> Self {
        self.description = Some(text.to_string());
        self
    }

    /// Set the section the item sits under.
    pub fn section(mut self, title: &str) -> Self {
        self.section = Some(title.to_string());
        self
    }

    /// Set whether the item is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Properties for the list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListProps {
    /// The items, in order.
    pub items: Vec<ListItem>,
    /// The id of the selected item, if any.
    pub selected: Option<String>,
    /// Whether a filter row is shown on top of the list.
    pub filterable: bool,
    /// Whether the application has more items to give the list.
    pub has_more: bool,
    /// Whether a loading row is shown at the end.
    pub loading: bool,
    /// The message shown when no item shows, if any.
    pub empty_text: Option<String>,
    /// The name the screen reader hears for the list.
    pub aria_label: Option<String>,
    /// The list's width in cells; none takes what the parent allots.
    pub width: Option<u16>,
    /// The list's height in cells; none takes what the parent allots.
    pub height: Option<u16>,
}

impl Props for ListProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A list: items the user moves through, selects and confirms.
pub struct List {
    /// Called with the id of the item the cursor moved to.
    pub(crate) on_select: Option<IdCallback>,
    /// Called with the id of the item confirmed with Enter or a double click.
    pub(crate) on_confirm: Option<IdCallback>,
    /// Called when Escape cancels the list.
    pub(crate) on_cancel: Option<UnitCallback>,
    /// Called when the list asks for more items.
    pub(crate) on_load_more: Option<UnitCallback>,
}

impl List {
    /// Create a list from its props, with no callbacks yet.
    pub fn new(_props: ListProps) -> Self {
        Self {
            on_select: None,
            on_confirm: None,
            on_cancel: None,
            on_load_more: None,
        }
    }

    /// Set the callback called with the id of the item the cursor moved to.
    pub fn with_on_select(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the id of a confirmed item.
    pub fn with_on_confirm(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_confirm = Some(Arc::new(f));
        self
    }

    /// Set the callback called when Escape cancels the list.
    pub fn with_on_cancel(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(f));
        self
    }

    /// Set the callback called when the list asks for more items.
    pub fn with_on_load_more(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_load_more = Some(Arc::new(f));
        self
    }
}

impl Component for List {
    type Props = ListProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn render(&self, _props: &Self::Props, _state: &Self::State) -> Element {
        // Skeleton: the list's rendering is written by the widget's job.
        crate::builder::div().build()
    }
}
