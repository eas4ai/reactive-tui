use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::sync::Arc;

/// Builder for creating Switch components with a fluent API
#[derive(Clone, Debug, Default)]
pub struct SwitchBuilder {
    checked: bool,
    label: Option<String>,
    aria_label: Option<String>,
    disabled: bool,
}

impl SwitchBuilder {
    /// Create a new SwitchBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether the switch is on
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the label text shown after the switch
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the name the screen reader hears, when the visible label is
    /// not it ("Wi-Fi" shown, "Wireless network" read).
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Set whether the switch is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Build the SwitchProps
    pub fn build(self) -> SwitchProps {
        SwitchProps {
            checked: self.checked,
            label: self.label,
            aria_label: self.aria_label,
            disabled: self.disabled,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("Switch").with_props(self.build())
    }
}

/// Properties for Switch component
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SwitchProps {
    /// Whether the switch is on
    pub checked: bool,
    /// Optional label text shown after the switch
    pub label: Option<String>,
    /// The name the screen reader hears instead of the visible label
    pub aria_label: Option<String>,
    /// Whether the switch is disabled
    pub disabled: bool,
}

impl Props for SwitchProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for Switch component
#[derive(Clone, Debug, Default)]
pub struct SwitchState {
    /// Whether the switch has keyboard focus
    pub is_focused: bool,
    /// Whether the mouse is hovering over the switch
    pub is_hover: bool,
}

/// Switch component with full keyboard and mouse support
pub struct Switch {
    state: SwitchState,
    on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

impl Switch {
    /// Create a switch from its props, with no callback
    pub fn new(_props: SwitchProps) -> Self {
        Self {
            state: SwitchState::default(),
            on_change: None,
        }
    }

    /// Set the callback called with the new on or off state on each toggle
    ///
    /// # Arguments
    /// * `f` - Callback function that receives the new checked state
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_on_change(mut self, f: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Replace the onChange callback, or remove it with `None`: a builder's
    /// switch takes the one its latest render carried (CMP-009).
    pub(crate) fn set_on_change(&mut self, f: Option<Arc<dyn Fn(bool) + Send + Sync>>) {
        self.on_change = f;
    }
}

impl Component for Switch {
    type Props = SwitchProps;
    type State = SwitchState;

    fn new(props: Self::Props) -> Self {
        Switch::new(props)
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        use super::look;
        use crate::widgets::display::pieces::icon::Icon;
        // The box is four cells, `[ ●]` when on and `[● ]` when off: the
        // thumb is the cell on the side it sits. The brackets are `border`,
        // `ring` while focused; the thumb `primary` when on and `text-muted`
        // when off; the label `foreground`, `text-muted` when disabled; the
        // row under the pointer `hover` (CTL-001, CTL-007).
        let focused = state.is_focused && !props.disabled;
        let frame = look::frame(focused);
        let dot = Icon::Dot.glyph();
        let thumb_look = if props.checked {
            look::MARK
        } else {
            look::MARK_OFF
        };
        let (left, left_look, right, right_look) = if props.checked {
            (" ", look::LABEL, dot, thumb_look)
        } else {
            (dot, thumb_look, " ", look::LABEL)
        };
        let label = props.label.as_deref().unwrap_or_default();
        let label_pieces = [
            (if label.is_empty() { "" } else { " " }, look::LABEL),
            (label, look::label(props.disabled)),
        ];
        let pieces = [
            ("[", frame),
            (left, left_look),
            (right, right_look),
            ("]", frame),
            label_pieces[0],
            label_pieces[1],
        ];
        let hover = if state.is_hover && !props.disabled {
            look::HOVER
        } else {
            ""
        };
        // With pixels the box is a picture over its four cells and no
        // glyph: a pill in `primary` with its thumb at the right when on,
        // in `input` bordered with its thumb at the left when off (PIX-007).
        #[cfg(feature = "wgpu-graphics")]
        let pixel_box = look::pixels().then(|| {
            look::spacer(
                4,
                crate::graphics::look::Look::switch(props.checked, focused, props.disabled),
            )
        });
        #[cfg(not(feature = "wgpu-graphics"))]
        let pixel_box: Option<Element> = None;
        let row = match pixel_box {
            Some(pixel_box) => {
                let mut children = vec![pixel_box];
                children.extend(look::pieces(&label_pieces));
                look::row_of(children, hover)
            }
            None => look::row(&pieces, hover),
        };

        use crate::accessibility::{Node, Role, Toggled};
        let mut accessible = Node::new(Role::Switch);
        if let Some(label) = props.aria_label.as_ref().or(props.label.as_ref()) {
            accessible.set_label(label.clone());
        }
        accessible.set_toggled(if props.checked {
            Toggled::True
        } else {
            Toggled::False
        });
        if props.disabled {
            accessible.set_disabled();
        } else {
            accessible.set_clickable();
        }
        row.with_accessibility(accessible)
            .with_focus(crate::component::FocusProps::input())
            .disabled(props.disabled)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if props.disabled && !matches!(event, Event::Focus(_)) {
            return EventResult::Ignored;
        }

        match event {
            Event::Key(key_event) => {
                if !state.is_focused {
                    return EventResult::Ignored;
                }

                self.handle_key_event(key_event, props, state)
            }
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            Event::Focus(event)
                if matches!(
                    event.kind,
                    crate::event::types::FocusEventKind::Gained
                        | crate::event::types::FocusEventKind::Lost
                ) =>
            {
                state.is_focused = event.kind == crate::event::types::FocusEventKind::Gained;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

impl Switch {
    /// Flip the switch and report the new state.
    fn toggle(&self, props: &mut SwitchProps) {
        props.checked = !props.checked;
        if let Some(on_change) = &self.on_change {
            on_change(props.checked);
        }
    }

    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut SwitchProps,
        _state: &mut SwitchState,
    ) -> EventResult {
        // The keys mean what the active keymap says (KEY-001).
        let action = crate::keymap::Keymap::active().action(event);
        use crate::keymap::Action;
        match action {
            Some(Action::Activate | Action::Confirm) => {
                self.toggle(props);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut SwitchProps,
        state: &mut SwitchState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Down if event.button == crate::event::types::MouseButton::Left => {
                // A click anywhere on the row flips it.
                state.is_focused = true;
                self.toggle(props);
                EventResult::Consumed
            }
            MouseEventKind::Enter | MouseEventKind::Move => {
                // App routes only cells within the acknowledged control bounds.
                state.is_hover = true;
                EventResult::Ignored
            }
            MouseEventKind::Leave => {
                state.is_hover = false;
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyCode, KeyModifiers, MouseButton, Position};

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        })
    }

    fn click() -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down,
            button: MouseButton::Left,
            position: Position::cell(2, 0),
            modifiers: KeyModifiers::empty(),
            timestamp: std::time::Instant::now(),
            wheel: None,
        })
    }

    /// The text of a rendered row: its text pieces, in order.
    fn row_text(element: &Element) -> String {
        element
            .children
            .iter()
            .filter_map(|child| match &child.element_type {
                crate::component::ElementType::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    /// CTL-007: the thumb is the glyph of the catalog, on the side the
    /// switch's state names.
    #[test]
    fn dis_005_the_switch_thumb_is_the_catalog_dot() {
        use crate::widgets::display::pieces::icon::Icon;
        let dot = Icon::Dot.glyph();
        let props = SwitchProps {
            checked: true,
            label: Some("Wi-Fi".to_owned()),
            ..Default::default()
        };
        let on = Switch::new(props.clone()).render(&props, &SwitchState::default());
        assert_eq!(row_text(&on), format!("[ {dot}] Wi-Fi"));
        let props = SwitchProps {
            checked: false,
            ..props
        };
        let off = Switch::new(props.clone()).render(&props, &SwitchState::default());
        assert_eq!(row_text(&off), format!("[{dot} ] Wi-Fi"));
    }

    /// CTL-009: Space, Enter and a click flip the switch, and each flip
    /// reports the new state.
    #[test]
    #[serial_test::parallel(keymap)]
    fn ctl_009_space_enter_and_a_click_toggle_the_switch() {
        use std::sync::Mutex;
        let reported = Arc::new(Mutex::new(Vec::new()));
        let log = reported.clone();
        let mut switch = Switch::new(SwitchProps::default())
            .with_on_change(move |on| log.lock().unwrap().push(on));
        let mut props = SwitchProps::default();
        let mut state = SwitchState {
            is_focused: true,
            is_hover: false,
        };

        assert_eq!(
            switch.handle_event(&key(KeyCode::Char(' ')), &mut props, &mut state),
            EventResult::Consumed
        );
        assert!(props.checked);
        switch.handle_event(&key(KeyCode::Enter), &mut props, &mut state);
        assert!(!props.checked);
        switch.handle_event(&click(), &mut props, &mut state);
        assert!(props.checked);
        assert_eq!(*reported.lock().unwrap(), vec![true, false, true]);
    }

    /// CTL-009: an unfocused switch ignores keys, and a disabled switch
    /// takes nothing and calls nothing.
    #[test]
    #[serial_test::parallel(keymap)]
    fn ctl_009_an_unfocused_or_disabled_switch_takes_no_key_or_click() {
        use std::sync::Mutex;
        let reported = Arc::new(Mutex::new(Vec::new()));
        let log = reported.clone();
        let mut switch = Switch::new(SwitchProps::default())
            .with_on_change(move |on| log.lock().unwrap().push(on));
        let mut props = SwitchProps::default();
        let mut unfocused = SwitchState::default();
        assert_eq!(
            switch.handle_event(&key(KeyCode::Char(' ')), &mut props, &mut unfocused),
            EventResult::Ignored
        );
        assert!(!props.checked);

        let mut disabled = SwitchProps {
            disabled: true,
            ..Default::default()
        };
        let mut focused = SwitchState {
            is_focused: true,
            is_hover: false,
        };
        assert_eq!(
            switch.handle_event(&key(KeyCode::Char(' ')), &mut disabled, &mut focused),
            EventResult::Ignored
        );
        assert_eq!(
            switch.handle_event(&click(), &mut disabled, &mut focused),
            EventResult::Ignored
        );
        assert!(!disabled.checked, "a disabled switch stays off");
        assert!(reported.lock().unwrap().is_empty());
    }

    /// CTL-010: the node is a switch, toggled on or off, named by its
    /// aria_label when set and its label otherwise, and disabled when it is.
    #[test]
    fn ctl_010_the_node_is_a_switch_named_and_toggled() {
        let props = SwitchProps {
            checked: true,
            label: Some("Wi-Fi".to_owned()),
            ..Default::default()
        };
        let element = Switch::new(props.clone()).render(&props, &SwitchState::default());
        let node = element
            .metadata
            .accessibility
            .as_ref()
            .expect("the switch's node");
        assert_eq!(node.inner.role(), accesskit::Role::Switch);
        assert_eq!(node.inner.toggled(), Some(accesskit::Toggled::True));
        assert_eq!(node.inner.label(), Some("Wi-Fi"));

        let props = SwitchProps {
            checked: false,
            label: Some("Wi-Fi".to_owned()),
            aria_label: Some("Wireless".to_owned()),
            disabled: true,
        };
        let element = Switch::new(props.clone()).render(&props, &SwitchState::default());
        let node = element
            .metadata
            .accessibility
            .as_ref()
            .expect("the switch's node");
        assert_eq!(node.inner.toggled(), Some(accesskit::Toggled::False));
        assert_eq!(node.inner.label(), Some("Wireless"), "aria_label wins");
        assert!(node.inner.is_disabled(), "a disabled switch is disabled");
    }
}
