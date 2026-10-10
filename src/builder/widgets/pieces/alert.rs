//! The builder for the alert display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-004, DIS-005, DIS-006.

use std::sync::Arc;

use crate::component::Element;
use crate::widgets::display::pieces::alert::{Alert, AlertKind, AlertProps};

/// Create an inline alert of `kind`: a bar, an icon, a title and a message.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::alert;
/// use reactive_tui::widgets::display::pieces::alert::AlertKind;
///
/// let banner = alert(AlertKind::Error)
///     .title("Upload failed")
///     .message("The file is larger than 10 MB.")
///     .closable(true)
///     .build();
/// ```
pub fn alert(kind: AlertKind) -> AlertBuilder {
    AlertBuilder {
        props: AlertProps {
            kind,
            title: String::new(),
            message: String::new(),
            closable: false,
            class: String::new(),
            on_close: None,
        },
    }
}

/// Builder for an inline alert. It fills the width its parent allots unless
/// `w-N` or `w-full` is given as a class.
#[derive(Clone)]
pub struct AlertBuilder {
    props: AlertProps,
}

impl AlertBuilder {
    /// Set the title, painted in `foreground`.
    pub fn title(mut self, title: &str) -> Self {
        self.props.title = title.to_string();
        self
    }

    /// Set the message, painted in `text-muted`.
    pub fn message(mut self, message: &str) -> Self {
        self.props.message = message.to_string();
        self
    }

    /// Show a `[×]` mark that closes the alert.
    pub fn closable(mut self, closable: bool) -> Self {
        self.props.closable = closable;
        self
    }

    /// Run `on_close` once when the alert closes.
    pub fn on_close<F>(mut self, on_close: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.props.on_close = Some(Arc::new(on_close));
        self
    }

    /// Add classes to the alert's element, after its own.
    pub fn class(mut self, class: &str) -> Self {
        if !self.props.class.is_empty() {
            self.props.class.push(' ');
        }
        self.props.class.push_str(class);
        self
    }

    /// Build the alert's element.
    pub fn build(self) -> Element {
        Element::typed::<Alert>(self.props)
    }
}
