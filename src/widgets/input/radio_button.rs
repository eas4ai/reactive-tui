use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

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
        Element::typed::<RadioButton<T>>(self.build())
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
    viewport: Option<crate::component::LayoutInfo>,
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
            viewport: None,
            on_change: None,
            _phantom: std::marker::PhantomData,
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if state.focused_index.is_some() {
            state.focused_index = state
                .focused_index
                .filter(|&i| props.options.get(i).is_some_and(|o| !o.disabled))
                .or_else(|| props.options.iter().position(|o| !o.disabled));
        }
        state.hover_index = state.hover_index.filter(|&i| i < props.options.len());
        true
    }

    fn layout(
        &mut self,
        layout: crate::component::LayoutInfo,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        self.viewport = Some(layout);
        false
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        use crate::accessibility::{Node, Role, Toggled};
        let mut root = Element::layout(crate::component::LayoutType::Flex)
            .with_class(if props.orientation == RadioOrientation::Horizontal {
                "flex flex-row gap-0.5 overflow-hidden"
            } else {
                "flex flex-col overflow-hidden"
            })
            .with_accessibility(Node::new(Role::RadioGroup))
            .with_focus(crate::component::FocusProps::input())
            .disabled(props.disabled || props.options.iter().all(|o| o.disabled));
        for (index, option) in props.options.iter().enumerate() {
            let disabled = props.disabled || option.disabled;
            let mut node = Node::new(Role::RadioButton);
            node.set_label(option.label.clone());
            node.set_toggled(if props.selected.as_ref() == Some(&option.value) {
                Toggled::True
            } else {
                Toggled::False
            });
            if disabled {
                node.set_disabled();
            } else {
                node.set_clickable();
            }
            let mut child = Element::text(Self::option_text(index, props, state))
                .with_key(format!("radio:{index}"))
                .with_class("whitespace-pre shrink-0")
                .with_accessibility(node);
            if !disabled {
                let options = child.metadata.accessibility_options.get_or_insert_default();
                options.focus = state.focused_index == Some(index);
                options.focus_event = Some(crate::event::CustomEvent::new(
                    "reactive_tui.radio.focus",
                    index.to_string().into_bytes(),
                ));
            }
            root = root.with_child(child);
        }
        root
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
            Event::Custom(event) if event.name == "reactive_tui.radio.focus" => {
                let index = std::str::from_utf8(&event.data)
                    .ok()
                    .and_then(|value| value.parse::<usize>().ok());
                if let Some(index) =
                    index.filter(|&i| props.options.get(i).is_some_and(|o| !o.disabled))
                {
                    state.focused_index = Some(index);
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::Key(key_event) => self.handle_key_event(key_event, props, state),
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            Event::Focus(event)
                if matches!(
                    event.kind,
                    crate::event::types::FocusEventKind::Gained
                        | crate::event::types::FocusEventKind::Lost
                ) =>
            {
                state.focused_index = if event.kind == crate::event::types::FocusEventKind::Gained
                    && !props.disabled
                {
                    props
                        .options
                        .iter()
                        .position(|o| !o.disabled && props.selected.as_ref() == Some(&o.value))
                        .or_else(|| props.options.iter().position(|o| !o.disabled))
                } else {
                    None
                };
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> RadioButton<T> {
    fn option_text(index: usize, props: &RadioButtonProps<T>, state: &RadioButtonState) -> String {
        let option = &props.options[index];
        let focus = if state.focused_index == Some(index) && !props.disabled && !option.disabled {
            "▶ "
        } else {
            "  "
        };
        let mark = if props.selected.as_ref() == Some(&option.value) {
            '●'
        } else {
            ' '
        };
        let label = if props.disabled || option.disabled {
            format!("({})", option.label)
        } else if state.hover_index == Some(index) {
            format!("_{}_", option.label)
        } else {
            option.label.clone()
        };
        format!("{focus}({mark}) {label}")
    }

    fn select(
        &self,
        index: usize,
        props: &mut RadioButtonProps<T>,
        state: &mut RadioButtonState,
    ) -> EventResult {
        let Some(option) = props.options.get(index).filter(|o| !o.disabled) else {
            return EventResult::Ignored;
        };
        state.focused_index = Some(index);
        if props.selected.as_ref() != Some(&option.value) {
            let value = option.value.clone();
            props.selected = Some(value.clone());
            if let Some(callback) = &self.on_change {
                callback(value);
            }
        }
        EventResult::Consumed
    }

    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut RadioButtonProps<T>,
        state: &mut RadioButtonState,
    ) -> EventResult {
        let Some(current) = state.focused_index.filter(|&i| i < props.options.len()) else {
            return EventResult::Ignored;
        };
        match event.code {
            KeyCode::Up | KeyCode::Left | KeyCode::Down | KeyCode::Right => {
                let len = props.options.len();
                let backwards = matches!(event.code, KeyCode::Up | KeyCode::Left);
                state.focused_index = (1..=len)
                    .map(|offset| {
                        if backwards {
                            (current + len - offset) % len
                        } else {
                            (current + offset) % len
                        }
                    })
                    .find(|&i| !props.options[i].disabled);
                EventResult::Consumed
            }
            KeyCode::Char(' ') | KeyCode::Space | KeyCode::Enter => {
                self.select(current, props, state)
            }
            _ => EventResult::Ignored,
        }
    }

    fn option_at(
        &self,
        event: &MouseEvent,
        props: &RadioButtonProps<T>,
        state: &RadioButtonState,
    ) -> Option<usize> {
        let (mut x, mut y) = (event.position.x() as usize, event.position.y() as usize);
        if let Some(layout) = self.viewport {
            x = x.checked_sub(layout.insets[0] as usize)?;
            y = y.checked_sub(layout.insets[1] as usize)?;
            let (width, height) = layout.content_size();
            if x >= width as usize || y >= height as usize {
                return None;
            }
        }
        if props.orientation == RadioOrientation::Vertical {
            return (y < props.options.len() && x < Self::option_text(y, props, state).width())
                .then_some(y);
        }
        if y != 0 {
            return None;
        }
        let mut start = 0;
        for index in 0..props.options.len() {
            let end = start + Self::option_text(index, props, state).width();
            if (start..end).contains(&x) {
                return Some(index);
            }
            start = end + 2;
        }
        None
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut RadioButtonProps<T>,
        state: &mut RadioButtonState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Down | MouseEventKind::Click
                if event.button == crate::event::types::MouseButton::Left =>
            {
                self.option_at(event, props, state)
                    .map_or(EventResult::Ignored, |index| {
                        self.select(index, props, state)
                    })
            }
            MouseEventKind::Enter | MouseEventKind::Move => {
                state.hover_index = self
                    .option_at(event, props, state)
                    .filter(|&i| !props.options[i].disabled);
                EventResult::Ignored
            }
            MouseEventKind::Leave => {
                state.hover_index = None;
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
