//! The empty display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-002, DIS-005.
//!
//! An empty state is a centered icon, a title, a description and a row of
//! actions, shown where a list or a panel has nothing to show yet.

use std::{any::Any, sync::Arc};

use crate::accessibility::{Node, Role};
use crate::builder::core::{button, div, span};
use crate::component::{Component, Element, Props};
use crate::widgets::display::look;
use crate::widgets::display::pieces::icon::Icon;

/// One action of an empty state: a button that runs its callback.
#[derive(Clone)]
pub struct EmptyAction {
    /// The text of the button.
    pub label: String,
    /// Run on Confirm, Activate or a click on the button.
    pub on_action: Arc<dyn Fn() + Send + Sync>,
}

/// The settings of an empty state. Callbacks are left out of equality, so a
/// rebuild that changes only a callback keeps the mounted state (CMP-008).
#[derive(Clone)]
pub struct EmptyProps {
    /// The icon above the title, from the catalog (DIS-005).
    pub icon: Option<Icon>,
    /// The title, painted in `foreground`.
    pub title: String,
    /// The description, painted in `text-muted`.
    pub description: String,
    /// The actions, in the order given.
    pub actions: Vec<EmptyAction>,
    /// Classes added to the element, after its own.
    pub class: String,
}

impl Default for EmptyProps {
    /// The builder's defaults, so props and builder paint the same state
    /// (DIS-001): the catalog's `Info` mark, no text and no actions.
    fn default() -> Self {
        Self {
            icon: Some(Icon::Info),
            title: String::new(),
            description: String::new(),
            actions: Vec::new(),
            class: String::new(),
        }
    }
}

impl PartialEq for EmptyProps {
    fn eq(&self, other: &Self) -> bool {
        self.icon == other.icon
            && self.title == other.title
            && self.description == other.description
            && self.class == other.class
            && self.actions.len() == other.actions.len()
            && self
                .actions
                .iter()
                .zip(&other.actions)
                .all(|(a, b)| a.label == b.label)
    }
}

impl Props for EmptyProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// An empty state. It keeps no state of its own; its buttons run the
/// application's callbacks.
pub struct Empty;

impl Component for Empty {
    type Props = EmptyProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        let same = props.actions.len() == supplied.actions.len()
            && props
                .actions
                .iter()
                .zip(&supplied.actions)
                .all(|(a, b)| Arc::ptr_eq(&a.on_action, &b.on_action));
        if same {
            return false;
        }
        props.actions = supplied.actions.clone();
        true
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        let mut node = Node::new(Role::Group);
        node.set_label(props.title.clone());
        if !props.description.is_empty() {
            node.set_description(props.description.clone());
        }

        let mut children = Vec::new();
        if let Some(icon) = props.icon {
            children.push(span().class(look::MUTED).text(icon.glyph()).build());
        }
        children.push(span().class(look::HEADER).text(&props.title).build());
        if !props.description.is_empty() {
            children.push(span().class(look::MUTED).text(&props.description).build());
        }
        if !props.actions.is_empty() {
            let buttons = props
                .actions
                .iter()
                .map(|action| {
                    let run = action.on_action.clone();
                    let mut action_node = Node::new(Role::Button);
                    action_node.set_label(action.label.clone());
                    action_node.set_clickable();
                    button()
                        .text(&action.label)
                        .on_click(move || run())
                        .build()
                        .with_accessibility(action_node)
                })
                .collect();
            children.push(div().class("flex flex-row gap-1").children(buttons).build());
        }

        // The state fills the width its parent allots unless the classes set one (DIS-002).
        let width = if props.class.split_whitespace().any(|t| t.starts_with("w-")) {
            ""
        } else {
            "w-full"
        };
        div()
            .class(&format!(
                "flex flex-col items-center {width} {}",
                props.class
            ))
            .children(children)
            .build()
            .with_accessibility(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dis_004_empty_state_is_a_group_labelled_by_its_title_and_described_by_its_description() {
        let props = EmptyProps {
            icon: Some(Icon::Info),
            title: "No files".into(),
            description: "Add a file to start".into(),
            actions: Vec::new(),
            class: String::new(),
        };
        let element = Empty::new(props.clone()).render(&props, &());
        let node = element.metadata.accessibility.as_ref().expect("a node");
        assert_eq!(node.role(), Role::Group);
        assert_eq!(node.inner.label(), Some("No files"));
        assert_eq!(node.inner.description(), Some("Add a file to start"));
    }
}
