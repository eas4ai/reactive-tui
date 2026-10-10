//! The key hint builder (DIS-001 to DIS-004): a key or chord as text, or the
//! active keymap's binding for an action.

use crate::component::Element;
use crate::keymap::Action;
use crate::widgets::display::pieces::kbd::{kbd as piece, KbdProps};

/// Create a key hint for a key or chord given as text, such as `Ctrl+S`.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::widgets::pieces::kbd::kbd;
///
/// let hint = kbd("Ctrl+S").build();
/// ```
pub fn kbd(text: impl Into<String>) -> KbdBuilder {
    KbdBuilder {
        props: KbdProps {
            text: text.into(),
            classes: Vec::new(),
            action: None,
        },
    }
}

/// Create a key hint for an action: the active keymap's first binding for it, read
/// at each render, so a rebind shows at the next render (DIS-003).
pub fn kbd_action(action: Action) -> KbdBuilder {
    KbdBuilder {
        props: KbdProps::for_action(action),
    }
}

/// Builder for a key hint.
#[derive(Clone, Debug, Default)]
pub struct KbdBuilder {
    props: KbdProps,
}

impl KbdBuilder {
    /// Add classes to the hint.
    pub fn class(mut self, class: &str) -> Self {
        self.props.classes.push(class.to_string());
        self
    }

    /// Build the hint's element.
    pub fn build(self) -> Element {
        piece(&self.props)
    }
}
