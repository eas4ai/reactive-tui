use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::sync::Arc;

/// Builder for creating Checkbox components with a fluent API
#[derive(Clone, Debug, Default)]
pub struct CheckboxBuilder {
    checked: bool,
    label: Option<String>,
    aria_label: Option<String>,
    disabled: bool,
    indeterminate: bool,
}

impl CheckboxBuilder {
    /// Create a new CheckboxBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether the checkbox is checked
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the label text for the checkbox
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the name the screen reader hears, when the visible label is
    /// not it ("Agree" shown, "Agree to the terms" read).
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Set whether the checkbox is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether the checkbox is in indeterminate state
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Build the CheckboxProps
    pub fn build(self) -> CheckboxProps {
        CheckboxProps {
            checked: self.checked,
            label: self.label,
            aria_label: self.aria_label,
            disabled: self.disabled,
            indeterminate: self.indeterminate,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("Checkbox").with_props(self.build())
    }
}

/// Properties for Checkbox component
#[derive(Clone, Debug, PartialEq, Default)]
pub struct CheckboxProps {
    /// Whether the checkbox is checked
    pub checked: bool,
    /// Optional label text for the checkbox
    pub label: Option<String>,
    /// The name the screen reader hears instead of the visible label
    pub aria_label: Option<String>,
    /// Whether the checkbox is disabled
    pub disabled: bool,
    /// Whether the checkbox is in indeterminate state (three-state support)
    pub indeterminate: bool,
}

impl Props for CheckboxProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for Checkbox component
#[derive(Clone, Debug, Default)]
pub struct CheckboxState {
    /// Whether the checkbox has keyboard focus
    pub is_focused: bool,
    /// Whether the mouse is hovering over the checkbox
    pub is_hover: bool,
}

/// Checkbox component with full keyboard and mouse support
pub struct Checkbox {
    state: CheckboxState,
    on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

impl Checkbox {
    /// Set the onChange callback for when the checkbox state changes
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
    /// checkbox takes the one its latest render carried (CMP-009).
    pub(crate) fn set_on_change(&mut self, f: Option<Arc<dyn Fn(bool) + Send + Sync>>) {
        self.on_change = f;
    }
}

impl Component for Checkbox {
    type Props = CheckboxProps;
    type State = CheckboxState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: CheckboxState::default(),
            on_change: None,
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        use super::look;
        // The box is its frame around a mark: `[✓]`, `[▬]` or `[ ]`. The
        // frame is `border`, `ring` while focused; the mark `primary`; the
        // label `foreground`, `text-muted` when disabled; the row under
        // the pointer `hover` (CTL-001).
        let frame = look::frame(state.is_focused && !props.disabled);
        let mark = if props.indeterminate {
            "▬"
        } else if props.checked {
            "✓"
        } else {
            " "
        };
        let label = props.label.as_deref().unwrap_or_default();
        let label_pieces = [
            (if label.is_empty() { "" } else { " " }, look::LABEL),
            (label, look::label(props.disabled)),
        ];
        let pieces = [
            ("[", frame),
            (mark, look::MARK),
            ("]", frame),
            label_pieces[0],
            label_pieces[1],
        ];
        let hover = if state.is_hover && !props.disabled {
            look::HOVER
        } else {
            ""
        };
        // With pixels the box is a picture over its three cells and no
        // glyph: a bordered square, filled with a check mark or a dash
        // when checked or mixed (PIX-004).
        #[cfg(feature = "wgpu-graphics")]
        let pixel_box = look::pixels().then(|| {
            look::spacer(
                3,
                crate::graphics::look::Look::checkbox(
                    props.checked,
                    props.indeterminate,
                    state.is_focused && !props.disabled,
                    props.disabled,
                ),
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
        let mut accessible = Node::new(Role::CheckBox);
        if let Some(label) = props.aria_label.as_ref().or(props.label.as_ref()) {
            accessible.set_label(label.clone());
        }
        accessible.set_toggled(if props.indeterminate {
            Toggled::Mixed
        } else if props.checked {
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

impl Checkbox {
    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut CheckboxProps,
        _state: &mut CheckboxState,
    ) -> EventResult {
        // The keys mean what the active keymap says (KEY-001).
        let action = crate::keymap::Keymap::active().action(event);
        use crate::keymap::Action;
        match action {
            Some(Action::Activate | Action::Confirm) => {
                // Toggle checkbox
                if props.indeterminate {
                    // If indeterminate, go to checked
                    props.indeterminate = false;
                    props.checked = true;
                } else {
                    // Toggle checked state
                    props.checked = !props.checked;
                }

                if let Some(on_change) = &self.on_change {
                    on_change(props.checked);
                }

                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut CheckboxProps,
        state: &mut CheckboxState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Down if event.button == crate::event::types::MouseButton::Left => {
                // Toggle on click
                state.is_focused = true;

                if props.indeterminate {
                    props.indeterminate = false;
                    props.checked = true;
                } else {
                    props.checked = !props.checked;
                }

                if let Some(on_change) = &self.on_change {
                    on_change(props.checked);
                }

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
    use crate::event::types::{KeyCode, KeyModifiers};

    #[test]
    // Sends a default key: kept apart from the tests that rebind it (KEY-001).
    #[serial_test::parallel(keymap)]
    fn test_checkbox_toggle() {
        let mut checkbox = Checkbox::new(CheckboxProps::default());
        let mut props = CheckboxProps::default();
        let mut state = CheckboxState {
            is_focused: true,
            is_hover: false,
        };

        assert!(!props.checked);

        // Toggle with space
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        let result = checkbox.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert!(props.checked);

        // Toggle again
        checkbox.handle_event(&event, &mut props, &mut state);
        assert!(!props.checked);
    }

    #[test]
    fn test_checkbox_disabled() {
        let mut checkbox = Checkbox::new(CheckboxProps::default());
        let mut props = CheckboxProps {
            disabled: true,
            ..Default::default()
        };
        let mut state = CheckboxState::default();

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        let result = checkbox.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Ignored);
        assert!(!props.checked); // Should not change
    }

    #[test]
    // Sends a default key: kept apart from the tests that rebind it (KEY-001).
    #[serial_test::parallel(keymap)]
    fn test_checkbox_indeterminate() {
        let mut checkbox = Checkbox::new(CheckboxProps::default());
        let mut props = CheckboxProps {
            indeterminate: true,
            ..Default::default()
        };
        let mut state = CheckboxState {
            is_focused: true,
            is_hover: false,
        };

        let event = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        checkbox.handle_event(&event, &mut props, &mut state);
        assert!(!props.indeterminate); // Should clear indeterminate
        assert!(props.checked); // Should become checked
    }

    #[test]
    fn test_checkbox_mouse_click() {
        let mut checkbox = Checkbox::new(CheckboxProps::default());
        let mut props = CheckboxProps::default();
        let mut state = CheckboxState::default();

        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down,
            button: crate::event::types::MouseButton::Left,
            position: crate::event::types::Position::cell(2, 0),
            modifiers: KeyModifiers::empty(),
            timestamp: std::time::Instant::now(),
            wheel: None,
        });

        let result = checkbox.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert!(props.checked);
        assert!(state.is_focused);
    }

    /// KEY-001: the checkbox toggles on the key the active keymap binds to
    /// Activate, and no longer on the key it replaced.
    #[test]
    #[serial_test::serial(keymap)]
    fn key_001_checkbox_reads_its_keys_through_the_keymap() {
        use crate::keymap::{Action, KeyBinding, Keymap};
        let mut keymap = Keymap::default();
        keymap.rebind(Action::Activate, [KeyBinding::new(KeyCode::F(2))]);
        let _scope = Keymap::scoped(keymap);
        let mut checkbox = Checkbox::new(CheckboxProps::default());
        let mut props = CheckboxProps::default();
        let mut state = CheckboxState {
            is_focused: true,
            is_hover: false,
        };
        let rebound = checkbox.handle_event(
            &Event::Key(KeyEvent::new(KeyCode::F(2))),
            &mut props,
            &mut state,
        );
        let checked_by_rebound = props.checked;
        let old = checkbox.handle_event(
            &Event::Key(KeyEvent::new(KeyCode::Char(' '))),
            &mut props,
            &mut state,
        );
        let checked_after_old = props.checked;
        // The scope above restores the default keymap when it drops.
        assert_eq!(rebound, EventResult::Consumed);
        assert!(checked_by_rebound, "the new Activate key toggles the box");
        assert_eq!(old, EventResult::Ignored);
        assert!(checked_after_old, "Space, no longer Activate, does nothing");
    }
}
