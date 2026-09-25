use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::sync::Arc;

/// Builder for creating Checkbox components with a fluent API
#[derive(Clone, Debug, Default)]
pub struct CheckboxBuilder {
    checked: bool,
    label: Option<String>,
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
        let mut result = String::new();

        // Add state indicator
        if props.disabled {
            result.push_str("🔒 ");
        } else if state.is_focused {
            result.push_str("▶ ");
        } else {
            result.push_str("  ");
        }

        // Render checkbox
        result.push('[');

        if props.indeterminate {
            result.push('▬'); // Indeterminate state
        } else if props.checked {
            result.push('✓'); // Checked
        } else {
            result.push(' '); // Unchecked
        }

        result.push(']');

        // Add label if provided
        if let Some(ref label) = props.label {
            result.push(' ');
            if props.disabled {
                result.push_str(&format!("({label})")); // Show disabled state
            } else if state.is_hover {
                result.push_str(&format!("_{label}_")); // Show hover state
            } else {
                result.push_str(label);
            }
        }

        use crate::accessibility::{Node, Role, Toggled};
        let mut accessible = Node::new(Role::CheckBox);
        if let Some(label) = &props.label {
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
        Element::text(result)
            .with_accessibility(accessible)
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
        match event.code {
            KeyCode::Char(' ') | KeyCode::Space | KeyCode::Enter => {
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
            MouseEventKind::Down | MouseEventKind::Click
                if event.button == crate::event::types::MouseButton::Left =>
            {
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
    use crate::event::types::KeyModifiers;

    #[test]
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
            kind: MouseEventKind::Click,
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
}
