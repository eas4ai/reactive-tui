//! The link display piece (docs/spec/display-pieces.md): text underlined in
//! `text-accent`, in `ring` while it holds the focus (DIS-001), as wide as
//! its text (DIS-002), that runs `on_open` with its URL on Confirm, Activate
//! or a click and is written as an OSC 8 hyperlink where the terminal takes
//! them (DIS-003), and that a screen reader hears as a link described by its
//! URL (DIS-004).

use crate::accessibility::{Node, Role};
use crate::component::{Component, Element, FocusProps, Props};
use crate::event::{
    router::EventResult,
    types::{Event, FocusEventKind, MouseButton, MouseEventKind},
};
use crate::keymap::{Action, Keymap};
use std::any::Any;
use std::sync::Arc;

/// The class of a link's text: underlined, in `text-accent`.
pub const LINK: &str = "text-accent underline";
/// The class of a link's text while it holds the focus: underlined, in `ring`.
pub const LINK_FOCUSED: &str = "text-ring underline";
/// The class of a disabled link's text: underlined, in `text-muted`.
pub const LINK_DISABLED: &str = "text-muted underline";

/// The callback a link runs with its URL when it is opened.
pub type OpenCallback = Arc<dyn Fn(&str) + Send + Sync>;

/// The settings of a [`Link`].
#[derive(Clone, Default)]
pub struct LinkProps {
    /// The text the link paints.
    pub text: String,
    /// The URL the link opens. Where the terminal takes OSC 8 hyperlinks the
    /// text is written as a link to it, so it must be printable ASCII
    /// without spaces; percent-encode anything else, or the text is written
    /// plain.
    pub url: String,
    /// The name a screen reader speaks, when the text is not it.
    pub aria_label: Option<String>,
    /// A disabled link is muted, takes no focus and takes no action: no key,
    /// no click and no terminal hyperlink opens it.
    pub disabled: bool,
    /// Called with `url` when the user opens the link with Confirm, Activate
    /// or a click.
    pub on_open: Option<OpenCallback>,
}

impl LinkProps {
    /// A link that paints `text` and opens `url`.
    pub fn new(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            url: url.into(),
            ..Default::default()
        }
    }
}

/// Equal over the settings and not the callback, so a rebuild that changes
/// only the callback keeps the mounted link, and its new callback acts from
/// the next event on (CMP-008).
impl PartialEq for LinkProps {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            text,
            url,
            aria_label,
            disabled,
            on_open: _,
        } = self;
        *text == other.text
            && *url == other.url
            && *aria_label == other.aria_label
            && *disabled == other.disabled
    }
}

impl Props for LinkProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// What a [`Link`] keeps between frames: whether it holds the focus.
#[derive(Clone, Debug, Default)]
pub struct LinkState {
    focused: bool,
}

/// A link: underlined text that opens a URL through the application's
/// `on_open` callback, and through the terminal's own click where the
/// terminal takes OSC 8 hyperlinks. Build one with
/// [`crate::builder::link()`] or from [`LinkProps`].
pub struct Link;

impl Link {
    /// The node a screen reader is given for `props`: a link labelled by
    /// its text, or `aria_label`, described by its URL, disabled when the
    /// link is and clickable when it is not.
    fn node(props: &LinkProps) -> Node {
        let mut node = Node::new(Role::Link);
        node.set_label(props.aria_label.as_deref().unwrap_or(&props.text));
        node.set_description(props.url.as_str());
        if props.disabled {
            node.set_disabled();
        } else {
            node.set_clickable();
        }
        node
    }
}

impl Component for Link {
    type Props = LinkProps;
    type State = LinkState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if props.disabled {
            *state = LinkState::default();
        }
        true
    }

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if crate::component::same_callback(&props.on_open, &supplied.on_open) {
            return false;
        }
        props.on_open = supplied.on_open.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let look = if props.disabled {
            LINK_DISABLED
        } else if state.focused {
            LINK_FOCUSED
        } else {
            LINK
        };
        // As wide as its text in any parent, a column that stretches its
        // children included (DIS-002).
        let width = crate::core::surface::text_display_width(&props.text);
        let element = Element::text(props.text.clone())
            .with_class(format!("{look} w-{width} shrink-0"))
            .with_accessibility(Self::node(props))
            .with_focus(FocusProps::input())
            .disabled(props.disabled);
        // A disabled link gives the terminal nothing to open either.
        if props.disabled {
            element
        } else {
            element.with_hyperlink(props.url.as_str())
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if let Event::Focus(focus) = event {
            if !matches!(focus.kind, FocusEventKind::Gained | FocusEventKind::Lost) {
                return EventResult::Ignored;
            }
            state.focused = focus.kind == FocusEventKind::Gained && !props.disabled;
            return EventResult::Consumed;
        }
        if props.disabled {
            return EventResult::Ignored;
        }
        let open = match event {
            // The keys mean what the active keymap says (KEY-001).
            Event::Key(key) => {
                state.focused
                    && matches!(
                        Keymap::active().action(key),
                        Some(Action::Confirm | Action::Activate)
                    )
            }
            Event::Mouse(mouse) => {
                mouse.kind == MouseEventKind::Down && mouse.button == MouseButton::Left
            }
            _ => false,
        };
        if !open {
            return EventResult::Ignored;
        }
        if let Some(on_open) = &props.on_open {
            on_open(&props.url);
        }
        EventResult::Consumed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyCode, KeyEvent};

    fn rendered(props: &LinkProps) -> Element {
        Link::new(props.clone()).render(props, &LinkState::default())
    }

    fn node(element: &Element) -> &Node {
        element
            .metadata
            .accessibility
            .as_ref()
            .expect("the link's node")
    }

    /// DIS-004: a link is a link to the screen reader, labelled by its text
    /// and described by its URL, with the click action that its keys also
    /// take.
    #[test]
    fn dis_004_a_link_is_a_link_labelled_by_its_text_and_described_by_its_url() {
        let element = rendered(&LinkProps::new("Docs", "https://example.com/docs"));
        let node = node(&element);
        assert_eq!(node.role(), Role::Link);
        assert_eq!(node.inner.label(), Some("Docs"));
        assert_eq!(node.inner.description(), Some("https://example.com/docs"));
        assert!(!node.inner.is_disabled());
        assert!(
            node.inner.supports_action(accesskit::Action::Click),
            "an enabled link offers the click a screen reader can take"
        );
        assert!(
            element.focus.as_ref().is_some_and(|focus| focus.focusable),
            "the click has a key: the link takes the focus"
        );
    }

    /// DIS-004: the label is `aria_label` when one is set, and a disabled
    /// link is marked disabled and offers no click.
    #[test]
    fn dis_004_a_disabled_link_is_marked_disabled_and_takes_its_aria_label() {
        let props = LinkProps {
            aria_label: Some("Read the docs".into()),
            disabled: true,
            ..LinkProps::new("Docs", "https://example.com/docs")
        };
        let element = rendered(&props);
        let node = node(&element);
        assert_eq!(node.role(), Role::Link);
        assert_eq!(node.inner.label(), Some("Read the docs"));
        assert_eq!(node.inner.description(), Some("https://example.com/docs"));
        assert!(
            node.inner.is_disabled(),
            "a disabled link is marked disabled"
        );
        assert!(!node.inner.supports_action(accesskit::Action::Click));
        assert!(element.metadata.disabled, "a disabled link takes no focus");
        assert!(
            element.metadata.hyperlink.is_none(),
            "a disabled link gives the terminal nothing to open"
        );
    }

    /// DIS-003 and KEY-001: Confirm and Activate open a focused link through
    /// the active keymap, and a rebound key replaces the old one.
    #[test]
    #[serial_test::serial(keymap)]
    fn dis_003_a_link_opens_on_the_keys_the_keymap_binds() {
        use crate::keymap::KeyBinding;
        use std::sync::Mutex;
        let opened = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = opened.clone();
        let mut props = LinkProps {
            on_open: Some(Arc::new(move |url: &str| {
                sink.lock().unwrap().push(url.to_string())
            })),
            ..LinkProps::new("Docs", "https://example.com/docs")
        };
        let mut link = Link::new(props.clone());
        let mut state = LinkState { focused: true };
        let key = |code| Event::Key(KeyEvent::new(code));
        assert_eq!(
            link.handle_event(&key(KeyCode::Enter), &mut props, &mut state),
            EventResult::Consumed
        );
        assert_eq!(
            link.handle_event(&key(KeyCode::Space), &mut props, &mut state),
            EventResult::Consumed
        );
        let mut keymap = Keymap::default();
        keymap.rebind(Action::Confirm, [KeyBinding::new(KeyCode::F(2))]);
        let scope = Keymap::scoped(keymap);
        assert_eq!(
            link.handle_event(&key(KeyCode::Enter), &mut props, &mut state),
            EventResult::Ignored,
            "Enter, no longer Confirm, does nothing"
        );
        assert_eq!(
            link.handle_event(&key(KeyCode::F(2)), &mut props, &mut state),
            EventResult::Consumed
        );
        drop(scope);
        assert_eq!(
            *opened.lock().unwrap(),
            vec!["https://example.com/docs"; 3],
            "Confirm, Activate and the rebound Confirm each open the link"
        );
    }
}
