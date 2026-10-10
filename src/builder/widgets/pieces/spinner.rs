//! The spinner builder (DIS-003, DIS-004): a status glyph that cycles the
//! icon catalog's frames, with its label.

use crate::component::Element;
use crate::widgets::display::pieces::spinner::{Spinner, SpinnerProps};

/// Create a spinner builder. Its label is "Loading" unless `label` sets one.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::spinner;
///
/// let loading = spinner().label("Saving").build();
/// ```
pub fn spinner() -> SpinnerBuilder {
    SpinnerBuilder {
        label: "Loading".to_string(),
        classes: Vec::new(),
        animated: true,
    }
}

/// Builder for a spinner. Its element is text in `text-muted`, as wide as
/// its glyph and label.
#[derive(Clone, Debug)]
pub struct SpinnerBuilder {
    label: String,
    classes: Vec<String>,
    animated: bool,
}

impl SpinnerBuilder {
    /// Set the status label a screen reader announces and the text beside
    /// the glyph shows.
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    /// Add classes to the spinner's element.
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_string());
        self
    }

    /// Set whether the spinner cycles its frames. `false` holds it on its
    /// first frame, as `reduced-motion` does.
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Build the spinner's element.
    pub fn build(self) -> Element {
        Element::typed::<Spinner>(SpinnerProps {
            label: self.label,
            class: self.classes.join(" "),
            animated: self.animated,
        })
    }
}
