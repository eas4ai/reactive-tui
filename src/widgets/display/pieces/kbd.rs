//! The key hint display piece (docs/spec/display-pieces.md), carrying DIS-001,
//! DIS-002, DIS-003, DIS-004.
//!
//! A key hint shows one key or chord as text, `Ctrl+S`, on `secondary`. Built
//! from an action it is the `Kbd` component: at each render it shows the active
//! keymap's first binding for the action, in the binding's display form, so a
//! rebind shows at the next render (DIS-003).

use std::any::Any;

use crate::accessibility::{Node, Role};
use crate::builder::core::span;
use crate::component::{Component, Element, Props};
use crate::keymap::{Action, Keymap};

/// The props of a key hint (DIS-001 to DIS-004).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KbdProps {
    /// The key or chord the hint shows, in the binding's display form. Unused
    /// when `action` is set.
    pub text: String,
    /// Classes of the hint.
    pub classes: Vec<String>,
    /// The action whose first active binding the hint shows, read at each render.
    pub action: Option<Action>,
}

impl Props for KbdProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl KbdProps {
    /// The props of a hint for `action`: the active keymap's first binding for it,
    /// read at each render, so a rebind shows at the next render (DIS-003).
    pub fn for_action(action: Action) -> Self {
        Self {
            text: String::new(),
            classes: Vec::new(),
            action: Some(action),
        }
    }

    /// The text the hint shows now: the active keymap's first binding for its
    /// action, or no text when the keymap binds none; or its `text` when it has no
    /// action.
    pub fn shown_text(&self) -> String {
        match self.action {
            Some(action) => Keymap::active()
                .binding(action)
                .map(|binding| binding.display())
                .unwrap_or_default(),
            None => self.text.clone(),
        }
    }
}

/// The key hint component of an action: it reads the active keymap at each render
/// (DIS-003).
pub struct Kbd;

impl Component for Kbd {
    type Props = KbdProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, _state: &()) -> Element {
        hint(props)
    }
}

/// The element of a key hint: the `Kbd` component when it is built from an action,
/// else its text on `secondary`, taking the width of the text.
pub fn kbd(props: &KbdProps) -> Element {
    if props.action.is_some() {
        Element::typed::<Kbd>(props.clone())
    } else {
        hint(props)
    }
}

/// The painted hint: its shown text on `secondary`.
fn hint(props: &KbdProps) -> Element {
    let text = props.shown_text();
    let mut classes = vec![
        "bg-secondary".to_string(),
        "text-secondary-foreground".to_string(),
    ];
    classes.extend(props.classes.iter().cloned());
    let mut node = Node::new(Role::Label);
    node.set_label(text.clone());
    span()
        .class(&classes.join(" "))
        .text(&text)
        .build()
        .with_accessibility(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::KeyCode;
    use crate::keymap::KeyBinding;

    #[test]
    fn dis_004_a_key_hint_is_a_label_with_its_binding_text() {
        let element = kbd(&KbdProps {
            text: "Ctrl+S".into(),
            ..KbdProps::default()
        });
        let node = element.metadata.accessibility.as_ref().expect("a node");
        assert_eq!(node.role(), Role::Label);
        assert_eq!(node.inner.label(), Some("Ctrl+S"));
    }

    #[test]
    fn dis_003_a_key_hint_for_an_action_shows_the_first_active_binding() {
        let mut keymap = Keymap::default();
        keymap.rebind(Action::Copy, [KeyBinding::new(KeyCode::F(3))]);
        let _scope = Keymap::scoped(keymap);
        assert_eq!(KbdProps::for_action(Action::Copy).shown_text(), "F3");
    }

    #[test]
    #[serial_test::serial(keymap)]
    fn dis_003_the_kbd_component_shows_a_rebind_at_its_next_render() {
        let props = KbdProps::for_action(Action::Copy);
        let hint = Kbd::new(props.clone());
        let label = |element: Element| {
            element
                .metadata
                .accessibility
                .as_ref()
                .expect("a node")
                .inner
                .label()
                .map(str::to_string)
        };
        assert_eq!(
            label(hint.render(&props, &())).as_deref(),
            Some("Ctrl+C"),
            "the default keymap binds Copy to Ctrl+C"
        );
        let mut keymap = Keymap::default();
        keymap.rebind(Action::Copy, [KeyBinding::new(KeyCode::F(3))]);
        let _scope = Keymap::scoped(keymap);
        assert_eq!(
            label(hint.render(&props, &())).as_deref(),
            Some("F3"),
            "the same component shows the rebind at its next render"
        );
    }
}
