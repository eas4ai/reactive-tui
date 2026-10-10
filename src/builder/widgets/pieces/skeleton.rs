//! The skeleton builder (DIS-002, DIS-003): rows that hold the place of
//! content still on its way.

use crate::component::Element;
use crate::widgets::display::pieces::skeleton::{Skeleton, SkeletonProps};

/// Create a skeleton builder with three rows.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::skeleton;
///
/// let placeholder = skeleton().rows(2).build();
/// ```
pub fn skeleton() -> SkeletonBuilder {
    SkeletonBuilder {
        rows: 3,
        classes: Vec::new(),
        animated: true,
    }
}

/// Builder for a skeleton. Its element fills its parent's width unless a
/// `w-N` or `w-full` class sets one.
#[derive(Clone, Debug)]
pub struct SkeletonBuilder {
    rows: u16,
    classes: Vec<String>,
    animated: bool,
}

impl SkeletonBuilder {
    /// Set how many rows the skeleton shows.
    pub fn rows(mut self, rows: u16) -> Self {
        self.rows = rows;
        self
    }

    /// Add classes to the skeleton's element, such as `w-40` or `h-6`.
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_string());
        self
    }

    /// Set whether the skeleton pulses. `false` holds it still, as
    /// `reduced-motion` does.
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Build the skeleton's element.
    pub fn build(self) -> Element {
        Element::typed::<Skeleton>(SkeletonProps {
            rows: self.rows,
            class: self.classes.join(" "),
            animated: self.animated,
        })
    }
}
