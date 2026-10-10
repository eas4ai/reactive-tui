//! The alert display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-002, DIS-004, DIS-005.
//!
//! An inline alert is a bar, an icon, a title and a message on `surface`, with
//! an optional `[×]` mark that closes it. Its kind picks the bar's and the
//! icon's color, and whether a screen reader hears it as an alert or a status.

use std::{any::Any, sync::Arc};

use crate::accessibility::{Node, Role};
use crate::builder::core::{button, div, span};
use crate::component::{same_callback, Component, Element, FocusProps, Props};
use crate::reactive::ThreadSafeSignal;
use crate::widgets::display::look;
use crate::widgets::display::pieces::icon::Icon;

/// The kind of an alert (DIS-001): its bar, its icon and its color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AlertKind {
    /// A note that needs no action. Painted in `text-muted`.
    #[default]
    Default,
    /// Information. Painted in `text-info`.
    Info,
    /// A finished step that worked. Painted in `text-success`.
    Success,
    /// A caution that does not stop the work. Painted in `text-warning`.
    Warning,
    /// A failure. Painted in `text-error`.
    Error,
}

impl AlertKind {
    /// The class that paints the bar and the icon of this kind.
    pub(crate) fn text_class(self) -> &'static str {
        match self {
            AlertKind::Default => look::MUTED,
            AlertKind::Info => "text-info",
            AlertKind::Success => "text-success",
            AlertKind::Warning => "text-warning",
            AlertKind::Error => "text-error",
        }
    }

    /// The icon of this kind, from the catalog (DIS-005).
    pub(crate) fn icon(self) -> Icon {
        match self {
            AlertKind::Default | AlertKind::Info => Icon::Info,
            AlertKind::Success => Icon::Success,
            AlertKind::Warning => Icon::Warning,
            AlertKind::Error => Icon::Error,
        }
    }

    /// Whether a screen reader hears this kind as an alert, not a status.
    pub(crate) fn is_alert(self) -> bool {
        matches!(self, AlertKind::Warning | AlertKind::Error)
    }
}

/// The settings of an alert. The callback is left out of equality, so a
/// rebuild that changes only the callback keeps the mounted alert (CMP-008).
#[derive(Clone)]
pub struct AlertProps {
    /// The kind, which picks the color and the icon.
    pub kind: AlertKind,
    /// The title, painted in `foreground`.
    pub title: String,
    /// The message, painted in `text-muted`.
    pub message: String,
    /// Whether the alert shows a `[×]` mark that closes it.
    pub closable: bool,
    /// Classes added to the alert's element, after its own.
    pub class: String,
    /// Called once when the alert closes (CMP-009).
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl PartialEq for AlertProps {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            kind,
            title,
            message,
            closable,
            class,
            on_close: _,
        } = self;
        *kind == other.kind
            && *title == other.title
            && *message == other.message
            && *closable == other.closable
            && *class == other.class
    }
}

impl Props for AlertProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// An inline alert. It shows until its `[×]` mark closes it, then renders
/// nothing.
pub(crate) struct Alert {
    closed: ThreadSafeSignal<bool>,
}

impl Component for Alert {
    type Props = AlertProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self {
            closed: ThreadSafeSignal::new(false),
        }
    }

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if same_callback(&props.on_close, &supplied.on_close) {
            return false;
        }
        props.on_close = supplied.on_close.clone();
        true
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        if self.closed.get() {
            return Element::empty();
        }
        let kind = props.kind;
        let mut node = Node::new(if kind.is_alert() {
            Role::Alert
        } else {
            Role::Status
        });
        node.set_label(props.title.clone());
        if !props.message.is_empty() {
            node.set_description(props.message.clone());
        }

        let mut texts = vec![span().class(look::HEADER).text(&props.title).build()];
        if !props.message.is_empty() {
            texts.push(span().class(look::MUTED).text(&props.message).build());
        }
        let body = div()
            .class("flex flex-col flex-1 min-w-0")
            .children(texts)
            .build();

        let bar = span().class(kind.text_class()).text("▌").build();
        let icon = span()
            .class(kind.text_class())
            .text(&format!(" {} ", kind.icon().glyph()))
            .build();
        let mut children = vec![bar, icon, body];
        if props.closable {
            let closed = self.closed.clone();
            let on_close = props.on_close.clone();
            let mut close_node = Node::new(Role::Button);
            close_node.set_label("Close alert");
            close_node.set_clickable();
            let close = button()
                .text(&format!("[{}]", Icon::Close.glyph()))
                .on_click(move || {
                    closed.set(true);
                    if let Some(callback) = &on_close {
                        callback();
                    }
                })
                .build()
                .with_accessibility(close_node)
                .with_focus(FocusProps::input());
            children.push(close);
        }

        // The alert fills the width its parent allots unless the classes set one (DIS-002).
        let width = if props.class.split_whitespace().any(|t| t.starts_with("w-")) {
            ""
        } else {
            "w-full"
        };
        div()
            .class(&format!("flex flex-row bg-surface {width} {}", props.class))
            .children(children)
            .build()
            .with_accessibility(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::Role;

    fn role_of(kind: AlertKind) -> Role {
        let props = AlertProps {
            kind,
            title: "Disk full".into(),
            message: "Free some space".into(),
            closable: false,
            class: String::new(),
            on_close: None,
        };
        let alert = Alert::new(props.clone());
        let element = alert.render(&props, &());
        element
            .metadata
            .accessibility
            .as_ref()
            .expect("an accessible node")
            .role()
    }

    #[test]
    fn dis_004_error_and_warning_are_alerts_and_the_other_kinds_are_statuses() {
        assert_eq!(role_of(AlertKind::Error), Role::Alert);
        assert_eq!(role_of(AlertKind::Warning), Role::Alert);
        assert_eq!(role_of(AlertKind::Default), Role::Status);
        assert_eq!(role_of(AlertKind::Info), Role::Status);
        assert_eq!(role_of(AlertKind::Success), Role::Status);
    }

    #[test]
    fn dis_004_alert_node_carries_its_title_and_message() {
        let props = AlertProps {
            kind: AlertKind::Error,
            title: "Disk full".into(),
            message: "Free some space".into(),
            closable: false,
            class: String::new(),
            on_close: None,
        };
        let element = Alert::new(props.clone()).render(&props, &());
        let node = element.metadata.accessibility.as_ref().expect("a node");
        assert_eq!(node.inner.label(), Some("Disk full"));
        assert_eq!(node.inner.description(), Some("Free some space"));
    }

    #[test]
    fn dis_005_alert_icons_come_from_the_catalog() {
        assert_eq!(AlertKind::Error.icon(), Icon::Error);
        assert_eq!(AlertKind::Warning.icon(), Icon::Warning);
        assert_eq!(AlertKind::Success.icon(), Icon::Success);
        assert_eq!(AlertKind::Info.icon(), Icon::Info);
    }
}
