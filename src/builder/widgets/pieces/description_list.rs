//! The description list builder (DIS-001 to DIS-004): pairs of a label and a value,
//! side by side, stacked, in columns or in a box.

use crate::component::Element;
use crate::widgets::display::pieces::description_list::{
    DescriptionLayout, DescriptionList, DescriptionListProps,
};

/// Create a description list builder. It is empty until pairs are added.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::widgets::pieces::description_list::description_list;
///
/// let list = description_list().pair("Name", "Ada").pair("Role", "Engineer").build();
/// ```
pub fn description_list() -> DescriptionListBuilder {
    DescriptionListBuilder {
        props: DescriptionListProps::default(),
    }
}

/// Builder for a description list.
#[derive(Clone, Debug, Default)]
pub struct DescriptionListBuilder {
    props: DescriptionListProps,
}

impl DescriptionListBuilder {
    /// Add a pair of a label and its value.
    pub fn pair(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.props.pairs.push((label.into(), value.into()));
        self
    }

    /// Stack each label over its value.
    pub fn vertical(mut self) -> Self {
        self.props.layout = DescriptionLayout::Vertical;
        self
    }

    /// Draw a box of the border color around the list.
    pub fn bordered(mut self) -> Self {
        self.props.bordered = true;
        self
    }

    /// Spread the pairs over `columns` columns, in order.
    pub fn columns(mut self, columns: u16) -> Self {
        self.props.columns = columns;
        self
    }

    /// Add classes to the list. A `w-`, `h-` class sets its size.
    pub fn class(mut self, class: &str) -> Self {
        self.props.classes.push(class.to_string());
        self
    }

    /// Build the list's element.
    pub fn build(self) -> Element {
        Element::typed::<DescriptionList>(self.props)
    }
}
