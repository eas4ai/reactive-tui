//! The badge and tag builders (DIS-001 to DIS-004): a count, a dot or a text in a
//! filled cell range or as an outline, a tag as a filled or outline text, and a
//! badge that follows an element.

use crate::component::Element;
use crate::widgets::display::pieces::badge as piece;
use crate::widgets::display::pieces::badge::{BadgeContent, BadgeKind, BadgeProps, TagProps};

/// Create a badge builder. A badge with no content is hidden.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::widgets::pieces::badge::badge;
///
/// let unread = badge().count(3).build();
/// ```
pub fn badge() -> BadgeBuilder {
    BadgeBuilder {
        props: BadgeProps::default(),
    }
}

/// Create a tag builder: `done` as a filled tag, or as an outline with
/// [`TagBuilder::outline`].
pub fn tag(text: impl Into<String>) -> TagBuilder {
    TagBuilder {
        props: TagProps {
            text: text.into(),
            ..TagProps::default()
        },
    }
}

/// Builder for a badge. A count of zero hides it; a count above the maximum shows
/// the maximum with `+`.
#[derive(Clone, Debug, Default)]
pub struct BadgeBuilder {
    props: BadgeProps,
}

impl BadgeBuilder {
    /// Set the kind, which sets the fill.
    pub fn kind(mut self, kind: BadgeKind) -> Self {
        self.props.kind = kind;
        self
    }

    /// Show a count. Zero hides the badge.
    pub fn count(mut self, count: u32) -> Self {
        self.props.content = BadgeContent::Count(count);
        self
    }

    /// Show a dot, `●`.
    pub fn dot(mut self) -> Self {
        self.props.content = BadgeContent::Dot;
        self
    }

    /// Show a short text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.props.content = BadgeContent::Text(text.into());
        self
    }

    /// Set the largest count shown as a number (99 unless set).
    pub fn max(mut self, max: u32) -> Self {
        self.props.max = max;
        self
    }

    /// Paint the badge as an outline: its text after one space, in its kind's
    /// text color with no fill.
    pub fn outline(mut self) -> Self {
        self.props.outline = true;
        self
    }

    /// Paint the kind's mark from the icon catalog before the text: `●` for
    /// the default kind, then the info, check, warning and error marks.
    pub fn mark(mut self) -> Self {
        self.props.mark = true;
        self
    }

    /// Add classes to the badge.
    pub fn class(mut self, class: &str) -> Self {
        self.props.classes.push(class.to_string());
        self
    }

    /// Build the badge's element. A hidden badge builds an empty element.
    pub fn build(self) -> Element {
        piece::badge(&self.props)
    }

    /// Build the element of `owner` with this badge after it. The badge's text joins
    /// the owner's description for the screen reader.
    pub fn on(self, owner: Element) -> Element {
        piece::follow(&self.props, owner)
    }
}

/// Builder for a tag.
#[derive(Clone, Debug)]
pub struct TagBuilder {
    props: TagProps,
}

impl TagBuilder {
    /// Set the kind, which sets the fill or the outline's color.
    pub fn kind(mut self, kind: BadgeKind) -> Self {
        self.props.kind = kind;
        self
    }

    /// Paint the tag as an outline: its text between brackets.
    pub fn outline(mut self) -> Self {
        self.props.outline = true;
        self
    }

    /// Add classes to the tag.
    pub fn class(mut self, class: &str) -> Self {
        self.props.classes.push(class.to_string());
        self
    }

    /// Build the tag's element: a label with its text.
    pub fn build(self) -> Element {
        piece::tag(&self.props)
    }
}
