use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::sync::Arc;

/// Builder for creating RadioButton components with a fluent API
#[derive(Clone, Debug)]
pub struct RadioButtonBuilder<T: Clone + PartialEq + Send + Sync + 'static> {
    options: Vec<RadioOption<T>>,
    selected: Option<T>,
    disabled: bool,
    orientation: RadioOrientation,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Default for RadioButtonBuilder<T> {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            selected: None,
            disabled: false,
            orientation: RadioOrientation::Vertical,
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> RadioButtonBuilder<T> {
    /// Create a new RadioButtonBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a radio option
    pub fn option(mut self, value: T, label: impl Into<String>) -> Self {
        self.options.push(RadioOption {
            value,
            label: label.into(),
            disabled: false,
        });
        self
    }

    /// Add a disabled radio option
    pub fn disabled_option(mut self, value: T, label: impl Into<String>) -> Self {
        self.options.push(RadioOption {
            value,
            label: label.into(),
            disabled: true,
        });
        self
    }

    /// Set all options at once
    pub fn options(mut self, options: Vec<RadioOption<T>>) -> Self {
        self.options = options;
        self
    }

    /// Set the selected value
    pub fn selected(mut self, value: T) -> Self {
        self.selected = Some(value);
        self
    }

    /// Set whether the entire radio group is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the orientation (vertical or horizontal)
    pub fn orientation(mut self, orientation: RadioOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Build the RadioButtonProps
    pub fn build(self) -> RadioButtonProps<T> {
        RadioButtonProps {
            options: self.options,
            selected: self.selected,
            disabled: self.disabled,
            orientation: self.orientation,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("RadioButton").with_props(self.build())
    }
}

/// Orientation for radio button layout
#[derive(Clone, Debug, PartialEq, Default)]
pub enum RadioOrientation {
    /// Stack radio buttons vertically
    #[default]
    Vertical,
    /// Arrange radio buttons horizontally
    Horizontal,
}

/// A single radio button option
#[derive(Clone, Debug, PartialEq)]
pub struct RadioOption<T> {
    /// The value associated with this option
    pub value: T,
    /// Display label for the option
    pub label: String,
    /// Whether this specific option is disabled
    pub disabled: bool,
}

/// Properties for RadioButton component
#[derive(Clone, Debug, PartialEq)]
pub struct RadioButtonProps<T: Clone + PartialEq + Send + Sync + 'static> {
    /// List of radio options
    pub options: Vec<RadioOption<T>>,
    /// Currently selected value
    pub selected: Option<T>,
    /// Whether the entire radio group is disabled
    pub disabled: bool,
    /// Layout orientation
    pub orientation: RadioOrientation,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Default for RadioButtonProps<T> {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            selected: None,
            disabled: false,
            orientation: RadioOrientation::default(),
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Props for RadioButtonProps<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for RadioButton component
#[derive(Clone, Debug, Default)]
pub struct RadioButtonState {
    /// Index of the currently focused option
    pub focused_index: Option<usize>,
    /// Whether any option has hover state
    pub hover_index: Option<usize>,
}

/// RadioButton component with full keyboard and mouse support
pub struct RadioButton<T: Clone + PartialEq + Send + Sync + 'static> {
    state: RadioButtonState,
    on_change: Option<Arc<dyn Fn(T) + Send + Sync>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> RadioButton<T> {
    /// Set the onChange callback for when selection changes
    pub fn with_on_change(mut self, f: impl Fn(T) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Component for RadioButton<T> {
    type Props = RadioButtonProps<T>;
    type State = RadioButtonState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: RadioButtonState::default(),
            on_change: None,
            _phantom: std::marker::PhantomData,
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let mut result = String::new();

        for (index, option) in props.options.iter().enumerate() {
            // Add focus/hover indicators
            if state.focused_index == Some(index) && !props.disabled && !option.disabled {
                result.push_str("▶ ");
            } else {
                result.push_str("  ");
            }

            // Radio button symbol
            result.push('(');
            if props.selected.as_ref() == Some(&option.value) {
                result.push('●'); // Selected
            } else {
                result.push(' '); // Not selected
            }
            result.push(')');

            // Add label
            result.push(' ');
            if props.disabled || option.disabled {
                result.push_str(&format!("({})", option.label)); // Show disabled state
            } else if state.hover_index == Some(index) {
                result.push_str(&format!("_{}_", option.label)); // Show hover state
            } else {
                result.push_str(&option.label);
            }

            // Add separator based on orientation
            match props.orientation {
                RadioOrientation::Horizontal => {
                    if index < props.options.len() - 1 {
                        result.push_str("  ");
                    }
                }
                RadioOrientation::Vertical => {
                    if index < props.options.len() - 1 {
                        result.push('\n');
                    }
                }
            }
        }

        Element::text(result)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if props.disabled {
            return EventResult::Ignored;
        }

        match event {
            Event::Key(key_event) => self.handle_key_event(key_event, props, state),
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            Event::Focus(_) => {
                if state.focused_index.is_none() && !props.options.is_empty() {
                    // Focus first non-disabled option
                    for (i, option) in props.options.iter().enumerate() {
                        if !option.disabled {
                            state.focused_index = Some(i);
                            break;
                        }
                    }
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> RadioButton<T> {
    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut RadioButtonProps<T>,
        state: &mut RadioButtonState,
    ) -> EventResult {
        let Some(current_index) = state.focused_index else {
            return EventResult::Ignored;
        };

        match event.code {
            KeyCode::Up | KeyCode::Left => {
                // Move to previous non-disabled option
                let mut new_index = current_index;
                loop {
                    if new_index == 0 {
                        new_index = props.options.len() - 1;
                    } else {
                        new_index -= 1;
                    }

                    if new_index == current_index {
                        break; // We've wrapped around
                    }

                    if !props.options[new_index].disabled {
                        state.focused_index = Some(new_index);
                        break;
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Down | KeyCode::Right => {
                // Move to next non-disabled option
                let mut new_index = current_index;
                loop {
                    new_index = (new_index + 1) % props.options.len();

                    if new_index == current_index {
                        break; // We've wrapped around
                    }

                    if !props.options[new_index].disabled {
                        state.focused_index = Some(new_index);
                        break;
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                // Select the focused option
                if current_index < props.options.len() && !props.options[current_index].disabled {
                    let value = props.options[current_index].value.clone();
                    props.selected = Some(value.clone());

                    if let Some(on_change) = &self.on_change {
                        on_change(value);
                    }
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut RadioButtonProps<T>,
        state: &mut RadioButtonState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Click => {
                let y = event.position.y() as usize;

                // Determine which option was clicked based on orientation
                let clicked_index = match props.orientation {
                    RadioOrientation::Vertical => {
                        // Each option is on its own line
                        if y < props.options.len() {
                            Some(y)
                        } else {
                            None
                        }
                    }
                    RadioOrientation::Horizontal => {
                        // All options on same line, need to calculate based on x position
                        // This is a simplified implementation
                        if y == 0 {
                            Some(0) // For now, just select first option for horizontal
                        } else {
                            None
                        }
                    }
                };

                if let Some(index) = clicked_index {
                    if index < props.options.len() && !props.options[index].disabled {
                        state.focused_index = Some(index);
                        let value = props.options[index].value.clone();
                        props.selected = Some(value.clone());

                        if let Some(on_change) = &self.on_change {
                            on_change(value);
                        }
                        return EventResult::Consumed;
                    }
                }
                EventResult::Ignored
            }
            MouseEventKind::Move => {
                // Track hover state
                let y = event.position.y() as usize;

                match props.orientation {
                    RadioOrientation::Vertical => {
                        if y < props.options.len() {
                            state.hover_index = Some(y);
                        } else {
                            state.hover_index = None;
                        }
                    }
                    RadioOrientation::Horizontal => {
                        if y == 0 {
                            state.hover_index = Some(0);
                        } else {
                            state.hover_index = None;
                        }
                    }
                }
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
    fn test_radio_selection() {
        let mut radio = RadioButton::<String>::new(RadioButtonProps::default());
        let mut props = RadioButtonProps {
            options: vec![
                RadioOption {
                    value: "option1".to_string(),
                    label: "Option 1".to_string(),
                    disabled: false,
                },
                RadioOption {
                    value: "option2".to_string(),
                    label: "Option 2".to_string(),
                    disabled: false,
                },
            ],
            selected: None,
            disabled: false,
            orientation: RadioOrientation::Vertical,
        };
        let mut state = RadioButtonState {
            focused_index: Some(0),
            hover_index: None,
        };

        // Select first option
        let event = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        let result = radio.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(props.selected, Some("option1".to_string()));
    }

    #[test]
    fn test_radio_navigation() {
        let mut radio = RadioButton::<i32>::new(RadioButtonProps::default());
        let mut props = RadioButtonProps {
            options: vec![
                RadioOption {
                    value: 1,
                    label: "One".to_string(),
                    disabled: false,
                },
                RadioOption {
                    value: 2,
                    label: "Two".to_string(),
                    disabled: false,
                },
                RadioOption {
                    value: 3,
                    label: "Three".to_string(),
                    disabled: false,
                },
            ],
            selected: None,
            disabled: false,
            orientation: RadioOrientation::Vertical,
        };
        let mut state = RadioButtonState {
            focused_index: Some(0),
            hover_index: None,
        };

        // Navigate down
        let event = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        radio.handle_event(&event, &mut props, &mut state);
        assert_eq!(state.focused_index, Some(1));

        // Navigate down again
        radio.handle_event(&event, &mut props, &mut state);
        assert_eq!(state.focused_index, Some(2));

        // Navigate down to wrap around
        radio.handle_event(&event, &mut props, &mut state);
        assert_eq!(state.focused_index, Some(0));
    }
}
