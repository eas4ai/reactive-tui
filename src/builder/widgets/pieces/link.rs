//! The link builder (docs/spec/display-pieces.md, DIS-001 to DIS-004): text
//! underlined in `text-accent` that opens a URL through `on_open` on
//! Confirm, Activate or a click, and through the terminal's own click where
//! the terminal takes OSC 8 hyperlinks.

use crate::component::Element;
use crate::widgets::display::pieces::link::{Link, LinkProps};
use std::sync::Arc;

/// Create a link builder for `text` that opens `url`.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::link;
///
/// let docs = link("Read the docs", "https://example.com/docs")
///     .on_open(|url| println!("open {url}"))
///     .build();
/// ```
pub fn link(text: &str, url: &str) -> LinkBuilder {
    LinkBuilder {
        props: LinkProps::new(text, url),
        classes: Vec::new(),
    }
}

/// Builder for a link. Its element is a [`Link`] built from the same
/// [`LinkProps`], so a link built here looks and acts as one built from
/// its props.
#[derive(Clone)]
pub struct LinkBuilder {
    props: LinkProps,
    classes: Vec<String>,
}

impl LinkBuilder {
    /// Set the callback run with the link's URL when the user opens it with
    /// Confirm, Activate or a click.
    pub fn on_open(mut self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.props.on_open = Some(Arc::new(f));
        self
    }

    /// Set whether the link is disabled: muted, without the focus, and
    /// opened by nothing.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.disabled = disabled;
        self
    }

    /// Set the name a screen reader speaks, when the text is not it.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.props.aria_label = Some(label.to_string());
        self
    }

    /// Add classes to the link's element, after its own; a `w-N` class sets
    /// its width in place of its text's.
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_string());
        self
    }

    /// Build the link's element.
    pub fn build(self) -> Element {
        let element = Element::typed::<Link>(self.props);
        if self.classes.is_empty() {
            element
        } else {
            element.with_class(self.classes.join(" "))
        }
    }
}

impl From<LinkBuilder> for Element {
    fn from(builder: LinkBuilder) -> Self {
        builder.build()
    }
}
