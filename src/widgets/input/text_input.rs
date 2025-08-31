use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::sync::Arc;

/// Properties for TextInput component
#[derive(Clone, Debug, PartialEq)]
pub struct TextInputProps {
    pub value: String,
    pub placeholder: Option<String>,
    pub max_length: Option<usize>,
    pub mask: Option<char>,
    pub disabled: bool,
    pub width: Option<u16>,
    pub validator_pattern: Option<String>, // Regex pattern for validation
    pub error_message: Option<String>,
}

impl Default for TextInputProps {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: Some("Enter text...".to_string()),
            max_length: None,
            mask: None,
            disabled: false,
            width: Some(30),
            validator_pattern: None,
            error_message: None,
        }
    }
}

impl Props for TextInputProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for TextInput component
#[derive(Clone, Debug, Default)]
pub struct TextInputState {
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub is_focused: bool,
    pub is_valid: bool,
    pub scroll_offset: usize,
}

/// Text input component with full editing capabilities
pub struct TextInput {
    state: TextInputState,
    on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    on_submit: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl TextInput {
    /// Create a new TextInput with an onChange callback
    pub fn with_on_change(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Create a new TextInput with an onSubmit callback
    pub fn with_on_submit(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(f));
        self
    }

    /// Validate the input value
    fn validate(&self, value: &str, pattern: &Option<String>) -> bool {
        if let Some(pattern_str) = pattern {
            // Simple pattern matching (in production, use regex crate)
            match pattern_str.as_str() {
                "numeric" => value.chars().all(|c| c.is_numeric()),
                "alpha" => value.chars().all(|c| c.is_alphabetic()),
                "alphanumeric" => value.chars().all(|c| c.is_alphanumeric()),
                "email" => value.contains('@') && value.contains('.'),
                _ => true,
            }
        } else {
            true
        }
    }

    /// Delete the current selection
    fn delete_selection(&mut self, props: &mut TextInputProps, state: &mut TextInputState) {
        if let (Some(start), Some(end)) = (state.selection_start, state.selection_end) {
            let (del_start, del_end) = if start < end {
                (start, end)
            } else {
                (end, start)
            };
            props.value.drain(del_start..del_end);
            state.cursor_position = del_start;
            state.selection_start = None;
            state.selection_end = None;
        }
    }

    /// Update scroll offset to keep cursor visible
    fn update_scroll(&mut self, state: &mut TextInputState, width: usize) {
        if state.cursor_position < state.scroll_offset {
            state.scroll_offset = state.cursor_position;
        } else if state.cursor_position >= state.scroll_offset + width {
            state.scroll_offset = state.cursor_position - width + 1;
        }
    }
}

impl Component for TextInput {
    type Props = TextInputProps;
    type State = TextInputState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: TextInputState::default(),
            on_change: None,
            on_submit: None,
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        // Validate on prop changes
        state.is_valid = self.validate(&props.value, &props.validator_pattern);

        // Update internal state
        self.state = state.clone();

        // Always re-render on update
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let width = props.width.unwrap_or(30) as usize;

        // Apply mask if needed
        let display_value = if let Some(mask_char) = props.mask {
            mask_char.to_string().repeat(props.value.len())
        } else if props.value.is_empty() {
            if let Some(placeholder) = &props.placeholder {
                placeholder.clone()
            } else {
                props.value.clone()
            }
        } else {
            props.value.clone()
        };

        // Calculate visible portion
        let visible_start = state.scroll_offset;
        let visible_end = (visible_start + width).min(display_value.len());
        let visible_text = if visible_end > visible_start {
            &display_value[visible_start..visible_end]
        } else {
            ""
        };

        // Build the display string with cursor and selection
        let mut result = String::new();

        // Add border style based on state
        if !state.is_valid && props.error_message.is_some() {
            result.push_str("❌ ");
        } else if state.is_focused {
            result.push_str("▶ ");
        } else if props.disabled {
            result.push_str("🔒 ");
        } else {
            result.push_str("  ");
        }

        // Build the text with cursor and selection highlighting
        result.push('[');

        for (i, ch) in visible_text.chars().enumerate() {
            let abs_pos = visible_start + i;

            // Check if this position is selected
            let is_selected = if let (Some(sel_start), Some(sel_end)) =
                (state.selection_start, state.selection_end)
            {
                let (start, end) = if sel_start < sel_end {
                    (sel_start, sel_end)
                } else {
                    (sel_end, sel_start)
                };
                abs_pos >= start && abs_pos < end
            } else {
                false
            };

            // Add cursor or selection marker
            if state.is_focused && abs_pos == state.cursor_position {
                result.push('│'); // Cursor
            }

            if is_selected {
                // In a real TUI, we'd use background color
                result.push_str(&format!("《{ch}》"));
            } else {
                result.push(ch);
            }
        }

        // Add cursor at end if needed
        if state.is_focused
            && state.cursor_position == display_value.len()
            && state.cursor_position >= visible_start
            && state.cursor_position <= visible_end
        {
            result.push('│');
        }

        // Add scroll indicators
        if visible_start > 0 || visible_end < display_value.len() {
            result.push_str("...");
        }

        result.push(']');

        // Add error message if invalid
        if !state.is_valid && props.error_message.is_some() {
            result.push_str(&format!(" {}", props.error_message.clone().unwrap()));
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
            Event::Key(key_event) => {
                if !state.is_focused {
                    return EventResult::Ignored;
                }

                self.handle_key_event(key_event, props, state)
            }
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            Event::Focus(_) => {
                state.is_focused = true;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

impl TextInput {
    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut TextInputProps,
        state: &mut TextInputState,
    ) -> EventResult {
        let width = props.width.unwrap_or(30) as usize;

        match event.code {
            KeyCode::Char(c) => {
                // Check max length
                if let Some(max_len) = props.max_length {
                    if props.value.len() >= max_len && state.selection_start.is_none() {
                        return EventResult::Consumed;
                    }
                }

                // Delete selection if exists
                if state.selection_start.is_some() {
                    self.delete_selection(props, state);
                }

                // Insert character
                props.value.insert(state.cursor_position, c);
                state.cursor_position += 1;

                // Validate
                state.is_valid = self.validate(&props.value, &props.validator_pattern);

                // Update scroll
                self.update_scroll(state, width);

                // Trigger callback
                if let Some(on_change) = &self.on_change {
                    on_change(props.value.clone());
                }

                EventResult::Consumed
            }
            KeyCode::Backspace => {
                if state.selection_start.is_some() {
                    self.delete_selection(props, state);
                } else if state.cursor_position > 0 {
                    props.value.remove(state.cursor_position - 1);
                    state.cursor_position -= 1;
                    self.update_scroll(state, width);
                }

                state.is_valid = self.validate(&props.value, &props.validator_pattern);

                if let Some(on_change) = &self.on_change {
                    on_change(props.value.clone());
                }

                EventResult::Consumed
            }
            KeyCode::Delete => {
                if state.selection_start.is_some() {
                    self.delete_selection(props, state);
                } else if state.cursor_position < props.value.len() {
                    props.value.remove(state.cursor_position);
                }

                state.is_valid = self.validate(&props.value, &props.validator_pattern);

                if let Some(on_change) = &self.on_change {
                    on_change(props.value.clone());
                }

                EventResult::Consumed
            }
            KeyCode::Left => {
                if event.modifiers.shift {
                    // Start or extend selection
                    if state.selection_start.is_none() {
                        state.selection_start = Some(state.cursor_position);
                    }
                    if state.cursor_position > 0 {
                        state.cursor_position -= 1;
                        state.selection_end = Some(state.cursor_position);
                    }
                } else {
                    // Clear selection and move cursor
                    state.selection_start = None;
                    state.selection_end = None;
                    if state.cursor_position > 0 {
                        state.cursor_position -= 1;
                    }
                }
                self.update_scroll(state, width);
                EventResult::Consumed
            }
            KeyCode::Right => {
                if event.modifiers.shift {
                    // Start or extend selection
                    if state.selection_start.is_none() {
                        state.selection_start = Some(state.cursor_position);
                    }
                    if state.cursor_position < props.value.len() {
                        state.cursor_position += 1;
                        state.selection_end = Some(state.cursor_position);
                    }
                } else {
                    // Clear selection and move cursor
                    state.selection_start = None;
                    state.selection_end = None;
                    if state.cursor_position < props.value.len() {
                        state.cursor_position += 1;
                    }
                }
                self.update_scroll(state, width);
                EventResult::Consumed
            }
            KeyCode::Home => {
                if event.modifiers.shift {
                    if state.selection_start.is_none() {
                        state.selection_start = Some(state.cursor_position);
                    }
                    state.cursor_position = 0;
                    state.selection_end = Some(0);
                } else {
                    state.cursor_position = 0;
                    state.selection_start = None;
                    state.selection_end = None;
                }
                state.scroll_offset = 0;
                EventResult::Consumed
            }
            KeyCode::End => {
                if event.modifiers.shift {
                    if state.selection_start.is_none() {
                        state.selection_start = Some(state.cursor_position);
                    }
                    state.cursor_position = props.value.len();
                    state.selection_end = Some(props.value.len());
                } else {
                    state.cursor_position = props.value.len();
                    state.selection_start = None;
                    state.selection_end = None;
                }
                self.update_scroll(state, width);
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(on_submit) = &self.on_submit {
                    on_submit(props.value.clone());
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut TextInputProps,
        state: &mut TextInputState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Click => {
                state.is_focused = true;

                // Calculate click position in text
                let click_x = event.position.x() as usize;
                let text_start = 2; // Account for border prefix

                if click_x >= text_start {
                    let text_pos = click_x - text_start + state.scroll_offset;
                    state.cursor_position = text_pos.min(props.value.len());
                    state.selection_start = None;
                    state.selection_end = None;
                }

                EventResult::Consumed
            }
            MouseEventKind::Drag => {
                if state.is_focused {
                    let drag_x = event.position.x() as usize;

                    let text_start = 2;

                    if drag_x >= text_start {
                        let text_pos = drag_x - text_start + state.scroll_offset;
                        let drag_pos = text_pos.min(props.value.len());

                        if state.selection_start.is_none() {
                            state.selection_start = Some(state.cursor_position);
                        }

                        state.cursor_position = drag_pos;
                        state.selection_end = Some(drag_pos);
                    }

                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
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
    fn test_text_input_basic() {
        let mut input = TextInput::new(TextInputProps::default());
        let mut props = TextInputProps::default();
        let mut state = TextInputState::default();

        state.is_focused = true;

        // Type 'H'
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('H'),
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        let result = input.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(props.value, "H");
        assert_eq!(state.cursor_position, 1);
    }

    #[test]
    fn test_text_input_max_length() {
        let mut input = TextInput::new(TextInputProps::default());
        let mut props = TextInputProps {
            value: "Test".to_string(),
            max_length: Some(4),
            ..Default::default()
        };
        let mut state = TextInputState {
            cursor_position: 4,
            is_focused: true,
            ..Default::default()
        };

        // Try to type when at max length
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('X'),
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        let result = input.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(props.value, "Test"); // Should not change
    }

    #[test]
    fn test_text_input_selection() {
        let mut input = TextInput::new(TextInputProps::default());
        let mut props = TextInputProps {
            value: "Hello World".to_string(),
            ..Default::default()
        };
        let mut state = TextInputState {
            cursor_position: 5,
            is_focused: true,
            selection_start: Some(0),
            selection_end: Some(5),
            ..Default::default()
        };

        // Type to replace selection
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('T'),
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        let result = input.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(props.value, "T World");
        assert_eq!(state.cursor_position, 1);
        assert!(state.selection_start.is_none());
    }

    #[test]
    fn test_text_input_validation() {
        let mut input = TextInput::new(TextInputProps::default());
        let mut props = TextInputProps {
            validator_pattern: Some("numeric".to_string()),
            ..Default::default()
        };
        let mut state = TextInputState {
            is_focused: true,
            ..Default::default()
        };

        // Type a number
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('1'),
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        input.handle_event(&event, &mut props, &mut state);
        assert!(state.is_valid);

        // Type a letter
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        input.handle_event(&event, &mut props, &mut state);
        assert!(!state.is_valid); // Should be invalid now
    }

    #[test]
    fn test_text_input_masking() {
        let input = TextInput::new(TextInputProps::default());
        let props = TextInputProps {
            value: "secret".to_string(),
            mask: Some('*'),
            ..Default::default()
        };
        let state = TextInputState::default();

        let rendered = input.render(&props, &state);
        let text = match rendered.element_type {
            crate::component::ElementType::Text(ref t) => t,
            _ => panic!("Expected text element"),
        };

        assert!(text.contains("******")); // Should show masked value
    }
}
