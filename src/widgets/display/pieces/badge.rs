//! The badge and tag display pieces (docs/spec/display-pieces.md), carrying DIS-001,
//! DIS-002, DIS-003, DIS-004.
//!
//! A badge is a count, a dot or a short text in a filled cell range. It takes the
//! width of its content and hides at a count of zero. A badge that follows an
//! element puts its text into the element's description for the screen reader. A
//! tag is a filled text or, as an outline, its text between brackets.

use crate::accessibility::{Node, Role};
use crate::builder::core::{div, span};
use crate::component::{Element, Props};
use crate::widgets::display::pieces::icon::Icon;
use std::any::Any;

/// The kind of a badge or a tag: the fill it takes, or the color of its outline
/// (DIS-001).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BadgeKind {
    /// A neutral mark, on `secondary`.
    #[default]
    Default,
    /// Information, on `info`.
    Info,
    /// A success, on `success`.
    Success,
    /// A caution, on `warning`.
    Warning,
    /// A failure, on `error`.
    Error,
}

impl BadgeKind {
    /// The classes of a filled badge or tag of this kind: the kind's fill and its
    /// `-foreground` text.
    pub fn fill(self) -> &'static str {
        match self {
            BadgeKind::Default => "bg-secondary text-secondary-foreground",
            BadgeKind::Info => "bg-info text-info-foreground",
            BadgeKind::Success => "bg-success text-success-foreground",
            BadgeKind::Warning => "bg-warning text-warning-foreground",
            BadgeKind::Error => "bg-error text-error-foreground",
        }
    }

    /// The class of an outline tag's text of this kind.
    pub fn outline(self) -> &'static str {
        match self {
            BadgeKind::Default => "text-muted",
            BadgeKind::Info => "text-info",
            BadgeKind::Success => "text-success",
            BadgeKind::Warning => "text-warning",
            BadgeKind::Error => "text-error",
        }
    }
}

/// What a badge shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum BadgeContent {
    /// A count, shown as ` n `. A count of zero hides the badge, and a count above
    /// the badge's `max` shows `max+`.
    Count(u32),
    /// A dot, `●` from the icon catalog.
    Dot,
    /// A short text, shown as ` text `.
    Text(String),
    /// Nothing: the badge is hidden.
    #[default]
    Hidden,
}

/// The props of a badge (DIS-001 to DIS-004).
#[derive(Clone, Debug, PartialEq)]
pub struct BadgeProps {
    /// The kind, which sets the fill.
    pub kind: BadgeKind,
    /// What the badge shows.
    pub content: BadgeContent,
    /// The largest count shown as a number (99 unless set).
    pub max: u32,
    /// Classes of the badge.
    pub classes: Vec<String>,
}

impl Default for BadgeProps {
    fn default() -> Self {
        Self {
            kind: BadgeKind::Default,
            content: BadgeContent::Hidden,
            max: 99,
            classes: Vec::new(),
        }
    }
}

impl Props for BadgeProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The text a badge paints, or `None` when it is hidden.
pub fn badge_text(props: &BadgeProps) -> Option<String> {
    match &props.content {
        BadgeContent::Count(0) | BadgeContent::Hidden => None,
        BadgeContent::Count(count) if *count > props.max => Some(format!(" {}+ ", props.max)),
        BadgeContent::Count(count) => Some(format!(" {count} ")),
        BadgeContent::Dot => Some(Icon::Dot.glyph().to_string()),
        BadgeContent::Text(text) => Some(format!(" {text} ")),
    }
}

/// The element of a badge.
pub fn badge(props: &BadgeProps) -> Element {
    let Some(text) = badge_text(props) else {
        return span().build();
    };
    let mut classes = vec![props.kind.fill().to_string()];
    classes.extend(props.classes.iter().cloned());
    span().class(&classes.join(" ")).text(&text).build()
}

/// The element of `owner` with the badge after it. The badge's text, trimmed,
/// joins the owner's accessible description, so the screen reader says it with
/// the owner (DIS-004). An owner without a node gets a group node that carries
/// the text.
pub fn follow(props: &BadgeProps, owner: Element) -> Element {
    let text = badge_text(props).map(|text| text.trim().to_string());
    let mut owner = owner;
    let mut wrapper_node = None;
    if let Some(text) = text.as_deref() {
        match owner.metadata.accessibility.as_mut() {
            Some(node) => append_description(node, text),
            None => {
                let mut node = Node::new(Role::Group);
                node.set_description(text.to_string());
                wrapper_node = Some(node);
            }
        }
    }
    let wrapper = div()
        .class("flex-row items-center")
        .child(owner)
        .child(badge(props))
        .build();
    match wrapper_node {
        Some(node) => wrapper.with_accessibility(node),
        None => wrapper,
    }
}

fn append_description(node: &mut Node, text: &str) {
    let joined = match node.inner.description() {
        Some(existing) if !existing.is_empty() => format!("{existing} {text}"),
        _ => text.to_string(),
    };
    node.set_description(joined);
}

/// The props of a tag (DIS-001, DIS-004).
#[derive(Clone, Debug, PartialEq)]
pub struct TagProps {
    /// The text of the tag.
    pub text: String,
    /// The kind, which sets the fill or the outline's color.
    pub kind: BadgeKind,
    /// Whether the tag is an outline: its text between brackets, not filled.
    pub outline: bool,
    /// Classes of the tag.
    pub classes: Vec<String>,
}

impl Default for TagProps {
    fn default() -> Self {
        Self {
            text: String::new(),
            kind: BadgeKind::Default,
            outline: false,
            classes: Vec::new(),
        }
    }
}

impl Props for TagProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The element of a standalone tag: a label with its text.
pub fn tag(props: &TagProps) -> Element {
    let (text, classes) = if props.outline {
        (
            format!("[ {} ]", props.text),
            props.kind.outline().to_string(),
        )
    } else {
        (format!(" {} ", props.text), props.kind.fill().to_string())
    };
    let mut classes = vec![classes];
    classes.extend(props.classes.iter().cloned());
    let mut node = Node::new(Role::Label);
    node.set_label(props.text.clone());
    span()
        .class(&classes.join(" "))
        .text(&text)
        .build()
        .with_accessibility(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dis_004_badge_text_joins_the_description_of_the_element_it_follows() {
        let owner = span()
            .text("Inbox")
            .build()
            .with_accessibility(Node::new(Role::Button));
        let props = BadgeProps {
            content: BadgeContent::Count(3),
            ..BadgeProps::default()
        };
        let wrapper = follow(&props, owner);
        let owner_node = wrapper.children[0]
            .metadata
            .accessibility
            .as_ref()
            .expect("the owner keeps its node");
        assert_eq!(owner_node.inner.description(), Some("3"));
        assert_eq!(owner_node.role(), Role::Button);
    }

    #[test]
    fn dis_004_a_standalone_tag_is_a_label_with_its_text() {
        let element = tag(&TagProps {
            text: "done".into(),
            ..TagProps::default()
        });
        let node = element.metadata.accessibility.as_ref().expect("a node");
        assert_eq!(node.role(), Role::Label);
        assert_eq!(node.inner.label(), Some("done"));
    }

    #[test]
    fn dis_003_badge_hides_at_zero_and_caps_at_max() {
        let at = |content| {
            badge_text(&BadgeProps {
                content,
                ..BadgeProps::default()
            })
        };
        assert_eq!(at(BadgeContent::Count(0)), None);
        assert_eq!(at(BadgeContent::Count(3)).as_deref(), Some(" 3 "));
        assert_eq!(at(BadgeContent::Count(120)).as_deref(), Some(" 99+ "));
        assert_eq!(at(BadgeContent::Dot).as_deref(), Some("●"));
    }
}
