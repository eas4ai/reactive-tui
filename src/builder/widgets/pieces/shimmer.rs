//! The shimmer builder (DIS-001, DIS-003): text with a band of brighter
//! cells that sweeps across it.

use crate::component::Element;
use crate::widgets::display::pieces::shimmer::{Shimmer, ShimmerProps};

/// Create a shimmer builder for `text`.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::shimmer;
///
/// let busy = shimmer("Loading the report").build();
/// ```
pub fn shimmer(text: &str) -> ShimmerBuilder {
    ShimmerBuilder {
        text: text.to_string(),
        classes: Vec::new(),
        animated: true,
    }
}

/// Builder for a shimmer. Its element is as wide as its text.
#[derive(Clone, Debug)]
pub struct ShimmerBuilder {
    text: String,
    classes: Vec<String>,
    animated: bool,
}

impl ShimmerBuilder {
    /// Add classes to the shimmer's element.
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_string());
        self
    }

    /// Set whether the band moves. `false` leaves the text plain, as
    /// `reduced-motion` does.
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Build the shimmer's element.
    pub fn build(self) -> Element {
        Element::typed::<Shimmer>(ShimmerProps {
            text: self.text,
            class: self.classes.join(" "),
            animated: self.animated,
        })
    }
}
