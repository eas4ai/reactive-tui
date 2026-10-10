//! The icon builder (DIS-005): one glyph from the icon catalog, painted muted
//! and read by a screen reader as the icon's name, or the label the application
//! gives.

use crate::accessibility::{Node, Role};
use crate::builder::core::span;
use crate::component::Element;
use crate::widgets::display::look;
use crate::widgets::display::pieces::icon::Icon;

/// Create an icon builder for one mark of the icon catalog.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::icon;
/// use reactive_tui::widgets::display::Icon;
///
/// let mark = icon(Icon::Warning).aria_label("Disk almost full").build();
/// ```
pub fn icon(icon: Icon) -> IconBuilder {
    IconBuilder {
        icon,
        classes: Vec::new(),
        aria_label: None,
    }
}

/// Builder for an icon. Its element is a text element of the glyph, painted in
/// `text-muted` unless the classes say otherwise. It is not focusable.
#[derive(Clone, Debug)]
pub struct IconBuilder {
    icon: Icon,
    classes: Vec<String>,
    aria_label: Option<String>,
}

impl IconBuilder {
    /// Add classes to the icon's element, after `text-muted`.
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_string());
        self
    }

    /// Set the label a screen reader speaks, in place of the icon's name.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.aria_label = Some(label.to_string());
        self
    }

    /// Build the icon's element.
    pub fn build(self) -> Element {
        let mut classes = vec![look::MUTED.to_string()];
        classes.extend(self.classes);
        let mut node = Node::new(Role::Image);
        node.set_label(
            self.aria_label
                .unwrap_or_else(|| self.icon.name().to_string()),
        );
        span()
            .class(&classes.join(" "))
            .text(self.icon.glyph())
            .build()
            .with_accessibility(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label_of(element: &Element) -> Option<String> {
        let node = element.metadata.accessibility.as_ref()?;
        node.inner.label().map(str::to_string)
    }

    #[test]
    fn dis_005_icon_element_is_an_image_labelled_by_the_icon_name() {
        let element = icon(Icon::FolderOpen).build();
        let node = element
            .metadata
            .accessibility
            .as_ref()
            .expect("an accessible node");
        assert_eq!(node.role(), Role::Image);
        assert_eq!(label_of(&element).as_deref(), Some("Folder open"));
        assert!(!element.focus.as_ref().is_some_and(|focus| focus.focusable));
    }

    #[test]
    fn dis_005_icon_element_takes_the_aria_label_when_given() {
        let element = icon(Icon::Error).aria_label("Upload failed").build();
        assert_eq!(label_of(&element).as_deref(), Some("Upload failed"));
    }
}
