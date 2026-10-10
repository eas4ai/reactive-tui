//! The builder for the pagination display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-002, DIS-003, DIS-005, DIS-006.

use std::sync::Arc;

use crate::component::Element;
use crate::widgets::display::pieces::pagination::{Pagination, PaginationProps};

/// Create a pagination bar. It has one page until `pages(n)` sets how many there are.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::pagination;
///
/// let bar = pagination()
///     .pages(20)
///     .current(5)
///     .aria_label("Results pages")
///     .on_change(|page| println!("page {page}"))
///     .build();
/// ```
pub fn pagination() -> PaginationBuilder {
    PaginationBuilder {
        props: PaginationProps::default(),
    }
}

/// Builder for a pagination bar. It fills the width its parent allots unless
/// `w-N` or `w-full` is given as a class.
#[derive(Clone)]
pub struct PaginationBuilder {
    props: PaginationProps,
}

impl PaginationBuilder {
    /// Set how many pages there are. At least one.
    pub fn pages(mut self, pages: usize) -> Self {
        self.props.pages = pages.max(1);
        self
    }

    /// Set the current page, 1-based.
    pub fn current(mut self, current: usize) -> Self {
        self.props.current = current;
        self
    }

    /// Set how many numbers the bar shows at most, first and last included.
    /// Five unless set; at least three.
    pub fn visible_pages(mut self, visible: usize) -> Self {
        self.props.visible_pages = visible.max(3);
        self
    }

    /// Show the arrows and `current / pages` instead of the page numbers.
    pub fn compact(mut self, compact: bool) -> Self {
        self.props.compact = compact;
        self
    }

    /// Set the label a screen reader speaks for the bar.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.props.aria_label = Some(label.to_string());
        self
    }

    /// Run `on_change` with the page when a click or Confirm chooses it.
    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        self.props.on_change = Some(Arc::new(on_change));
        self
    }

    /// Add classes to the bar's element, after its own.
    pub fn class(mut self, class: &str) -> Self {
        if !self.props.class.is_empty() {
            self.props.class.push(' ');
        }
        self.props.class.push_str(class);
        self
    }

    /// Build the bar's element.
    pub fn build(self) -> Element {
        Element::typed::<Pagination>(self.props)
    }
}
