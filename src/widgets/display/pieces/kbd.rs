//! The key hint display piece (docs/spec/display-pieces.md), carrying DIS-001,
//! DIS-002, DIS-003, DIS-004.
//!
//! A key hint shows one key or chord as text, `Ctrl+S`, on `secondary`. Built
//! from an action it shows the active keymap's first binding for the action, in
//! the binding's display form, so a rebind shows at the next render (DIS-003).

use std::any::Any;

use crate::accessibility::{Node, Role};
use crate::builder::core::span;
use crate::component::{Element, Props};
use crate::keymap::{Action, Keymap};

/// The props of a key hint (DIS-001 to DIS-004).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KbdProps {
    /// The key or chord the hint shows, in the binding's display form.
    pub text: String,
    /// Classes of the hint.
    pub classes: Vec<String>,
}

impl Props for KbdProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl KbdProps {
    /// The props of a hint for `action`: the active keymap's first binding for it,
    /// or no text when the keymap binds nothing to it.
    pub fn for_action(action: Action) -> Self {
        let text = Keymap::active()
            .binding(action)
            .map(|binding| binding.display())
            .unwrap_or_default();
        Self {
            text,
            classes: Vec::new(),
        }
    }
}

/// The element of a key hint: its text on `secondary`, taking the width of the text.
pub fn kbd(props: &KbdProps) -> Element {
    let mut classes = vec![
        "bg-secondary".to_string(),
        "text-secondary-foreground".to_string(),
    ];
    classes.extend(props.classes.iter().cloned());
    let mut node = Node::new(Role::Label);
    node.set_label(props.text.clone());
    span()
        .class(&classes.join(" "))
        .text(&props.text)
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
        assert_eq!(KbdProps::for_action(Action::Copy).text, "F3");
    }
}
