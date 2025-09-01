use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::collections::VecDeque;
use std::sync::Arc;

// Type alias for complex function pointer type
type SuggestionRequestCallback = Arc<dyn Fn(String, usize) -> Vec<Suggestion> + Send + Sync>;
use unicode_segmentation::UnicodeSegmentation;

/// Input mode for different text input behaviors
#[derive(Clone, Debug, PartialEq, Default)]
pub enum InputMode {
    /// Single line input (default)
    #[default]
    SingleLine,
    /// Multi-line input with specified height
    MultiLine { height: u16 },
    /// Password input (masked)
    Password,
    /// Numeric input only
    Numeric,
}

/// Auto-completion suggestion
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    pub text: String,
    pub description: Option<String>,
    pub insert_text: String,
}

/// Properties for TextInput component
#[derive(Clone, Debug, PartialEq)]
pub struct TextInputProps {
    pub value: String,
    pub placeholder: Option<String>,
    pub max_length: Option<usize>,
    pub disabled: bool,
    pub width: Option<u16>,
    pub mode: InputMode,
    pub validator_pattern: Option<String>, // Regex pattern for validation
    pub error_message: Option<String>,
    pub suggestions: Vec<Suggestion>,
    pub show_line_numbers: bool,
    pub wrap_text: bool,
    pub tab_size: usize,
    pub auto_indent: bool,
}

impl Default for TextInputProps {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: Some("Enter text...".to_string()),
            max_length: None,
            disabled: false,
            width: Some(30),
            mode: InputMode::default(),
            validator_pattern: None,
            error_message: None,
            suggestions: Vec::new(),
            show_line_numbers: false,
            wrap_text: true,
            tab_size: 4,
            auto_indent: false,
        }
    }
}

impl Props for TextInputProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Cursor position in a multi-line text input
#[derive(Clone, Debug, PartialEq, Default)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

/// Text selection range
#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

/// Undo/redo command for text editing
#[derive(Clone, Debug)]
pub enum EditCommand {
    Insert {
        position: usize,
        text: String,
    },
    Delete {
        position: usize,
        text: String,
    },
    Replace {
        position: usize,
        old_text: String,
        new_text: String,
    },
}

impl EditCommand {
    /// Create the inverse command for undo
    pub fn inverse(&self) -> Self {
        match self {
            EditCommand::Insert { position, text } => EditCommand::Delete {
                position: *position,
                text: text.clone(),
            },
            EditCommand::Delete { position, text } => EditCommand::Insert {
                position: *position,
                text: text.clone(),
            },
            EditCommand::Replace {
                position,
                old_text,
                new_text,
            } => EditCommand::Replace {
                position: *position,
                old_text: new_text.clone(),
                new_text: old_text.clone(),
            },
        }
    }
}

/// State for TextInput component
#[derive(Clone, Debug)]
pub struct TextInputState {
    /// Current cursor position in the text
    pub cursor: CursorPosition,
    /// Current text selection, if any
    pub selection: Option<Selection>,
    /// Whether the input currently has focus
    pub is_focused: bool,
    /// Whether the current input value is valid
    pub is_valid: bool,
    /// Horizontal scroll offset for long text
    pub scroll_offset_x: usize,
    /// Vertical scroll offset for multi-line text
    pub scroll_offset_y: usize,
    /// Stack of edit commands for undo functionality
    pub undo_stack: VecDeque<EditCommand>,
    /// Stack of edit commands for redo functionality
    pub redo_stack: VecDeque<EditCommand>,
    /// Currently highlighted suggestion index
    pub suggestion_index: Option<usize>,
    /// Whether to show the suggestions dropdown
    pub show_suggestions: bool,
    /// Text content split into lines (for multi-line mode)
    pub lines: Vec<String>,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            cursor: CursorPosition::default(),
            selection: None,
            is_focused: false,
            is_valid: true,
            scroll_offset_x: 0,
            scroll_offset_y: 0,
            undo_stack: VecDeque::with_capacity(100),
            redo_stack: VecDeque::new(),
            suggestion_index: None,
            show_suggestions: false,
            lines: vec![String::new()],
        }
    }
}

/// Text input component with advanced editing capabilities
pub struct TextInput {
    on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    on_submit: Option<Arc<dyn Fn(String) + Send + Sync>>,
    on_suggestion_request: Option<SuggestionRequestCallback>,
    clipboard_content: Option<String>,
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

    /// Create a new TextInput with a suggestion callback
    pub fn with_suggestions(
        mut self,
        f: impl Fn(String, usize) -> Vec<Suggestion> + Send + Sync + 'static,
    ) -> Self {
        self.on_suggestion_request = Some(Arc::new(f));
        self
    }

    /// Get the current text value from state
    fn get_text_value(&self, state: &TextInputState) -> String {
        state.lines.join("\n")
    }

    /// Set text value and update state
    fn set_text_value(&mut self, value: &str, state: &mut TextInputState) {
        state.lines = if value.is_empty() {
            vec![String::new()]
        } else {
            value.lines().map(|s| s.to_string()).collect()
        };

        // Ensure cursor is within bounds
        if state.cursor.line >= state.lines.len() {
            state.cursor.line = state.lines.len().saturating_sub(1);
        }

        let line_len = state
            .lines
            .get(state.cursor.line)
            .map(|line| line.graphemes(true).count())
            .unwrap_or(0);

        if state.cursor.column > line_len {
            state.cursor.column = line_len;
        }

        self.update_cursor_byte_offset(state);
    }

    /// Update the byte offset for the cursor position
    fn update_cursor_byte_offset(&self, state: &mut TextInputState) {
        let mut byte_offset = 0;

        // Add bytes from previous lines
        for i in 0..state.cursor.line {
            if let Some(line) = state.lines.get(i) {
                byte_offset += line.len() + 1; // +1 for newline
            }
        }

        // Add bytes from current line up to cursor column
        if let Some(line) = state.lines.get(state.cursor.line) {
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            for grapheme in graphemes
                .iter()
                .take(state.cursor.column.min(graphemes.len()))
            {
                byte_offset += grapheme.len();
            }
        }

        state.cursor.byte_offset = byte_offset;
    }

    /// Validate the input value
    fn validate(&self, value: &str, pattern: &Option<String>) -> bool {
        if let Some(pattern_str) = pattern {
            // Simple pattern matching (in production, use regex crate)
            match pattern_str.as_str() {
                "numeric" => value.chars().all(|c| c.is_numeric() || c.is_whitespace()),
                "alpha" => value
                    .chars()
                    .all(|c| c.is_alphabetic() || c.is_whitespace()),
                "alphanumeric" => value
                    .chars()
                    .all(|c| c.is_alphanumeric() || c.is_whitespace()),
                "email" => value.contains('@') && value.contains('.'),
                _ => true,
            }
        } else {
            true
        }
    }

    /// Execute an edit command and add to undo stack
    fn execute_command(&mut self, command: EditCommand, state: &mut TextInputState) {
        // Clear redo stack when new command is executed
        state.redo_stack.clear();

        // Add inverse command to undo stack
        let inverse = command.inverse();
        if state.undo_stack.len() >= 100 {
            state.undo_stack.pop_front();
        }
        state.undo_stack.push_back(inverse);

        // Execute the command
        self.apply_command(&command, state);
    }

    /// Apply a command to the text
    fn apply_command(&mut self, command: &EditCommand, state: &mut TextInputState) {
        match command {
            EditCommand::Insert { position, text } => {
                let current_text = self.get_text_value(state);
                let mut chars: Vec<char> = current_text.chars().collect();
                let insert_chars: Vec<char> = text.chars().collect();

                for (i, &ch) in insert_chars.iter().enumerate() {
                    chars.insert(position + i, ch);
                }

                let new_text: String = chars.into_iter().collect();
                self.set_text_value(&new_text, state);
            }
            EditCommand::Delete { position, text } => {
                let current_text = self.get_text_value(state);
                let mut chars: Vec<char> = current_text.chars().collect();

                for _ in 0..text.chars().count() {
                    if *position < chars.len() {
                        chars.remove(*position);
                    }
                }

                let new_text: String = chars.into_iter().collect();
                self.set_text_value(&new_text, state);
            }
            EditCommand::Replace {
                position,
                old_text: _,
                new_text,
            } => {
                let current_text = self.get_text_value(state);
                let mut chars: Vec<char> = current_text.chars().collect();

                // Remove old text
                for _ in 0..new_text.chars().count() {
                    if *position < chars.len() {
                        chars.remove(*position);
                    }
                }

                // Insert new text
                let insert_chars: Vec<char> = new_text.chars().collect();
                for (i, &ch) in insert_chars.iter().enumerate() {
                    chars.insert(position + i, ch);
                }

                let new_text: String = chars.into_iter().collect();
                self.set_text_value(&new_text, state);
            }
        }
    }

    /// Undo the last edit
    fn undo(&mut self, state: &mut TextInputState) {
        if let Some(command) = state.undo_stack.pop_back() {
            state.redo_stack.push_back(command.inverse());
            self.apply_command(&command, state);
        }
    }

    /// Redo the last undone edit
    fn redo(&mut self, state: &mut TextInputState) {
        if let Some(command) = state.redo_stack.pop_back() {
            state.undo_stack.push_back(command.inverse());
            self.apply_command(&command, state);
        }
    }

    /// Find word boundaries for navigation
    fn find_word_start(&self, text: &str, position: usize) -> usize {
        let chars: Vec<char> = text.chars().collect();
        if position == 0 || position > chars.len() {
            return position;
        }

        let mut pos = position.saturating_sub(1);

        // Skip whitespace backwards
        while pos > 0 && chars[pos].is_whitespace() {
            pos -= 1;
        }

        // Skip word characters backwards
        while pos > 0 && !chars[pos].is_whitespace() && chars[pos].is_alphanumeric() {
            pos -= 1;
        }

        // If we stopped on a non-alphanumeric, move forward one
        if pos < chars.len() && !chars[pos].is_alphanumeric() && !chars[pos].is_whitespace() {
            pos += 1;
        }

        pos
    }

    fn find_word_end(&self, text: &str, position: usize) -> usize {
        let chars: Vec<char> = text.chars().collect();
        if position >= chars.len() {
            return chars.len();
        }

        let mut pos = position;

        // Skip whitespace forward
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip word characters forward
        while pos < chars.len() && !chars[pos].is_whitespace() && chars[pos].is_alphanumeric() {
            pos += 1;
        }

        pos
    }

    /// Delete the current selection
    fn delete_selection(&mut self, state: &mut TextInputState) -> Option<String> {
        if let Some(selection) = &state.selection {
            let text = self.get_text_value(state);
            let start_offset = selection.start.byte_offset;
            let end_offset = selection.end.byte_offset;

            let (del_start, del_end) = if start_offset < end_offset {
                (start_offset, end_offset)
            } else {
                (end_offset, start_offset)
            };

            let deleted_text = text[del_start..del_end].to_string();
            let new_text = format!("{}{}", &text[..del_start], &text[del_end..]);

            self.set_text_value(&new_text, state);
            state.selection = None;

            // Move cursor to start of deleted selection
            self.move_cursor_to_byte_offset(del_start, state);

            Some(deleted_text)
        } else {
            None
        }
    }

    /// Move cursor to a specific byte offset
    fn move_cursor_to_byte_offset(&self, byte_offset: usize, state: &mut TextInputState) {
        let mut current_offset = 0;

        for (line_idx, line) in state.lines.iter().enumerate() {
            if current_offset + line.len() >= byte_offset {
                // Cursor is in this line
                let line_offset = byte_offset - current_offset;
                let graphemes: Vec<&str> = line.graphemes(true).collect();

                let mut column = 0;
                let mut char_offset = 0;

                for grapheme in graphemes {
                    if char_offset >= line_offset {
                        break;
                    }
                    char_offset += grapheme.len();
                    column += 1;
                }

                state.cursor.line = line_idx;
                state.cursor.column = column;
                state.cursor.byte_offset = byte_offset;
                return;
            }
            current_offset += line.len() + 1; // +1 for newline
        }

        // If we get here, move to end
        if let Some(last_line) = state.lines.last() {
            state.cursor.line = state.lines.len() - 1;
            state.cursor.column = last_line.graphemes(true).count();
            state.cursor.byte_offset = byte_offset;
        }
    }

    /// Update scroll offset to keep cursor visible
    fn update_scroll(&mut self, state: &mut TextInputState, width: usize, height: usize) {
        // Horizontal scrolling
        if state.cursor.column < state.scroll_offset_x {
            state.scroll_offset_x = state.cursor.column;
        } else if state.cursor.column >= state.scroll_offset_x + width {
            state.scroll_offset_x = state.cursor.column - width + 1;
        }

        // Vertical scrolling
        if state.cursor.line < state.scroll_offset_y {
            state.scroll_offset_y = state.cursor.line;
        } else if state.cursor.line >= state.scroll_offset_y + height {
            state.scroll_offset_y = state.cursor.line - height + 1;
        }
    }

    /// Copy selected text to clipboard
    fn copy_selection(&mut self, state: &TextInputState) {
        if let Some(selection) = &state.selection {
            let text = self.get_text_value(state);
            let start_offset = selection.start.byte_offset;
            let end_offset = selection.end.byte_offset;

            let (copy_start, copy_end) = if start_offset < end_offset {
                (start_offset, end_offset)
            } else {
                (end_offset, start_offset)
            };

            if copy_start < text.len() && copy_end <= text.len() {
                self.clipboard_content = Some(text[copy_start..copy_end].to_string());
            }
        }
    }

    /// Cut selected text to clipboard
    fn cut_selection(&mut self, state: &mut TextInputState) {
        if state.selection.is_some() {
            self.copy_selection(state);
            if let Some(deleted) = self.delete_selection(state) {
                let command = EditCommand::Delete {
                    position: state.cursor.byte_offset,
                    text: deleted,
                };
                self.execute_command(command, state);
            }
        }
    }

    /// Paste text from clipboard
    fn paste_from_clipboard(&mut self, state: &mut TextInputState) {
        if let Some(clipboard_text) = &self.clipboard_content.clone() {
            // Delete selection if exists
            if state.selection.is_some() {
                self.delete_selection(state);
            }

            let command = EditCommand::Insert {
                position: state.cursor.byte_offset,
                text: clipboard_text.clone(),
            };
            self.execute_command(command, state);

            // Move cursor to end of pasted text
            let new_offset = state.cursor.byte_offset + clipboard_text.len();
            self.move_cursor_to_byte_offset(new_offset, state);
        }
    }

    /// Move cursor left by one grapheme
    fn move_cursor_left(&self, state: &mut TextInputState) {
        if state.cursor.column > 0 {
            state.cursor.column -= 1;
        } else if state.cursor.line > 0 {
            // Move to end of previous line
            state.cursor.line -= 1;
            if let Some(line) = state.lines.get(state.cursor.line) {
                state.cursor.column = line.graphemes(true).count();
            }
        }
        self.update_cursor_byte_offset(state);
    }

    /// Move cursor right by one grapheme
    fn move_cursor_right(&self, state: &mut TextInputState) {
        if let Some(line) = state.lines.get(state.cursor.line) {
            let line_len = line.graphemes(true).count();
            if state.cursor.column < line_len {
                state.cursor.column += 1;
            } else if state.cursor.line + 1 < state.lines.len() {
                // Move to start of next line
                state.cursor.line += 1;
                state.cursor.column = 0;
            }
        }
        self.update_cursor_byte_offset(state);
    }

    /// Move cursor up by one line
    fn move_cursor_up(&self, state: &mut TextInputState) {
        if state.cursor.line > 0 {
            state.cursor.line -= 1;
            if let Some(line) = state.lines.get(state.cursor.line) {
                let line_len = line.graphemes(true).count();
                state.cursor.column = state.cursor.column.min(line_len);
            }
            self.update_cursor_byte_offset(state);
        }
    }

    /// Move cursor down by one line
    fn move_cursor_down(&self, state: &mut TextInputState) {
        if state.cursor.line + 1 < state.lines.len() {
            state.cursor.line += 1;
            if let Some(line) = state.lines.get(state.cursor.line) {
                let line_len = line.graphemes(true).count();
                state.cursor.column = state.cursor.column.min(line_len);
            }
            self.update_cursor_byte_offset(state);
        }
    }

    /// Extend selection left
    fn extend_selection_left(&self, state: &mut TextInputState) {
        if state.selection.is_none() {
            state.selection = Some(Selection {
                start: state.cursor.clone(),
                end: state.cursor.clone(),
            });
        }

        self.move_cursor_left(state);

        if let Some(selection) = &mut state.selection {
            selection.end = state.cursor.clone();
        }
    }

    /// Extend selection right
    fn extend_selection_right(&self, state: &mut TextInputState) {
        if state.selection.is_none() {
            state.selection = Some(Selection {
                start: state.cursor.clone(),
                end: state.cursor.clone(),
            });
        }

        self.move_cursor_right(state);

        if let Some(selection) = &mut state.selection {
            selection.end = state.cursor.clone();
        }
    }

    /// Extend selection up
    fn extend_selection_up(&self, state: &mut TextInputState) {
        if state.selection.is_none() {
            state.selection = Some(Selection {
                start: state.cursor.clone(),
                end: state.cursor.clone(),
            });
        }

        self.move_cursor_up(state);

        if let Some(selection) = &mut state.selection {
            selection.end = state.cursor.clone();
        }
    }

    /// Extend selection down
    fn extend_selection_down(&self, state: &mut TextInputState) {
        if state.selection.is_none() {
            state.selection = Some(Selection {
                start: state.cursor.clone(),
                end: state.cursor.clone(),
            });
        }

        self.move_cursor_down(state);

        if let Some(selection) = &mut state.selection {
            selection.end = state.cursor.clone();
        }
    }

    /// Handle Ctrl+key combinations
    fn handle_ctrl_key(
        &mut self,
        event: &KeyEvent,
        props: &mut TextInputProps,
        state: &mut TextInputState,
    ) -> EventResult {
        match event.code {
            KeyCode::Char('a') => {
                // Select all
                let text = self.get_text_value(state);
                if !text.is_empty() {
                    state.selection = Some(Selection {
                        start: CursorPosition {
                            line: 0,
                            column: 0,
                            byte_offset: 0,
                        },
                        end: CursorPosition {
                            line: state.lines.len().saturating_sub(1),
                            column: state
                                .lines
                                .last()
                                .map(|l| l.graphemes(true).count())
                                .unwrap_or(0),
                            byte_offset: text.len(),
                        },
                    });
                }
                EventResult::Consumed
            }
            KeyCode::Char('c') => {
                // Copy
                self.copy_selection(state);
                EventResult::Consumed
            }
            KeyCode::Char('x') => {
                // Cut
                self.cut_selection(state);
                let current_text = self.get_text_value(state);
                props.value = current_text.clone();
                if let Some(on_change) = &self.on_change {
                    on_change(current_text);
                }
                EventResult::Consumed
            }
            KeyCode::Char('v') => {
                // Paste
                self.paste_from_clipboard(state);
                let current_text = self.get_text_value(state);
                props.value = current_text.clone();
                if let Some(on_change) = &self.on_change {
                    on_change(current_text);
                }
                EventResult::Consumed
            }
            KeyCode::Char('z') => {
                // Undo
                self.undo(state);
                let current_text = self.get_text_value(state);
                props.value = current_text.clone();
                if let Some(on_change) = &self.on_change {
                    on_change(current_text);
                }
                EventResult::Consumed
            }
            KeyCode::Char('y') => {
                // Redo
                self.redo(state);
                let current_text = self.get_text_value(state);
                props.value = current_text.clone();
                if let Some(on_change) = &self.on_change {
                    on_change(current_text);
                }
                EventResult::Consumed
            }
            KeyCode::Left => {
                // Word left
                let text = self.get_text_value(state);
                let word_start = self.find_word_start(&text, state.cursor.byte_offset);
                self.move_cursor_to_byte_offset(word_start, state);
                EventResult::Consumed
            }
            KeyCode::Right => {
                // Word right
                let text = self.get_text_value(state);
                let word_end = self.find_word_end(&text, state.cursor.byte_offset);
                self.move_cursor_to_byte_offset(word_end, state);
                EventResult::Consumed
            }
            KeyCode::Backspace => {
                // Delete word left
                let text = self.get_text_value(state);
                let word_start = self.find_word_start(&text, state.cursor.byte_offset);
                if word_start < state.cursor.byte_offset {
                    let deleted_text = text[word_start..state.cursor.byte_offset].to_string();
                    let command = EditCommand::Delete {
                        position: word_start,
                        text: deleted_text,
                    };
                    self.execute_command(command, state);
                    self.move_cursor_to_byte_offset(word_start, state);

                    let current_text = self.get_text_value(state);
                    props.value = current_text.clone();
                    if let Some(on_change) = &self.on_change {
                        on_change(current_text);
                    }
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    /// Handle Alt+key combinations
    fn handle_alt_key(
        &mut self,
        _event: &KeyEvent,
        _props: &mut TextInputProps,
        _state: &mut TextInputState,
    ) -> EventResult {
        // Alt combinations can be added here for future features
        EventResult::Ignored
    }
}

impl Component for TextInput {
    type Props = TextInputProps;
    type State = TextInputState;

    fn new(_props: Self::Props) -> Self {
        Self {
            on_change: None,
            on_submit: None,
            on_suggestion_request: None,
            clipboard_content: None,
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        // Update text value if it changed
        let current_text = self.get_text_value(state);
        if current_text != props.value {
            self.set_text_value(&props.value, state);
        }

        // Validate on prop changes
        state.is_valid = self.validate(&props.value, &props.validator_pattern);

        // Always re-render on update
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let width = props.width.unwrap_or(30) as usize;

        // Get display value based on mode
        let display_value = match &props.mode {
            InputMode::Password => "*".repeat(props.value.len()),
            _ => {
                if props.value.is_empty() {
                    if let Some(placeholder) = &props.placeholder {
                        placeholder.clone()
                    } else {
                        props.value.clone()
                    }
                } else {
                    props.value.clone()
                }
            }
        };

        // Calculate visible portion for single line
        let visible_start = state.scroll_offset_x;
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
            let is_selected = if let Some(selection) = &state.selection {
                let start_offset = selection.start.byte_offset;
                let end_offset = selection.end.byte_offset;
                let (start, end) = if start_offset < end_offset {
                    (start_offset, end_offset)
                } else {
                    (end_offset, start_offset)
                };
                abs_pos >= start && abs_pos < end
            } else {
                false
            };

            // Add cursor or selection marker
            if state.is_focused && abs_pos == state.cursor.byte_offset {
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
            && state.cursor.byte_offset == display_value.len()
            && state.cursor.byte_offset >= visible_start
            && state.cursor.byte_offset <= visible_end
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

        // Add suggestions if showing
        if state.show_suggestions && !props.suggestions.is_empty() {
            result.push_str("\n💡 Suggestions:");
            for (i, suggestion) in props.suggestions.iter().enumerate() {
                let marker = if Some(i) == state.suggestion_index {
                    "▶"
                } else {
                    " "
                };
                result.push_str(&format!("\n{marker} {}", suggestion.text));
                if let Some(desc) = &suggestion.description {
                    result.push_str(&format!(" - {desc}"));
                }
            }
        }

        Element::component("div")
            .class(if props.disabled {
                "text-input disabled"
            } else if !state.is_valid {
                "text-input invalid"
            } else if state.is_focused {
                "text-input focused"
            } else {
                "text-input"
            })
            .with_child(Element::text(result))
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
        // Handle modifiers first
        if event.modifiers.ctrl {
            return self.handle_ctrl_key(event, props, state);
        }

        if event.modifiers.alt {
            return self.handle_alt_key(event, props, state);
        }

        let width = props.width.unwrap_or(30) as usize;
        let height = match &props.mode {
            InputMode::MultiLine { height } => *height as usize,
            _ => 1,
        };

        match event.code {
            KeyCode::Char(c) => {
                // Handle numeric mode
                if matches!(props.mode, InputMode::Numeric)
                    && !c.is_numeric()
                    && c != '.'
                    && c != '-'
                {
                    return EventResult::Consumed;
                }

                // Check max length
                if let Some(max_len) = props.max_length {
                    let current_text = self.get_text_value(state);
                    if current_text.len() >= max_len && state.selection.is_none() {
                        return EventResult::Consumed;
                    }
                }

                // Delete selection if exists
                if state.selection.is_some() {
                    self.delete_selection(state);
                }

                // Insert character
                let command = EditCommand::Insert {
                    position: state.cursor.byte_offset,
                    text: c.to_string(),
                };
                self.execute_command(command, state);

                // Move cursor forward
                let new_offset = state.cursor.byte_offset + c.len_utf8();
                self.move_cursor_to_byte_offset(new_offset, state);

                // Update scroll
                self.update_scroll(state, width, height);

                // Validate
                let current_text = self.get_text_value(state);
                state.is_valid = self.validate(&current_text, &props.validator_pattern);

                // Update props value
                props.value = current_text.clone();

                // Trigger onChange
                if let Some(on_change) = &self.on_change {
                    on_change(current_text.clone());
                }

                // Request suggestions if available
                if let Some(suggestion_fn) = &self.on_suggestion_request {
                    let suggestions = suggestion_fn(current_text, state.cursor.byte_offset);
                    props.suggestions = suggestions;
                    state.show_suggestions = !props.suggestions.is_empty();
                    state.suggestion_index = if state.show_suggestions {
                        Some(0)
                    } else {
                        None
                    };
                }

                EventResult::Consumed
            }
            KeyCode::Backspace => {
                if state.selection.is_some() {
                    if let Some(deleted) = self.delete_selection(state) {
                        let command = EditCommand::Delete {
                            position: state.cursor.byte_offset,
                            text: deleted,
                        };
                        self.execute_command(command, state);
                    }
                } else if state.cursor.byte_offset > 0 {
                    let current_text = self.get_text_value(state);
                    let chars: Vec<char> = current_text.chars().collect();
                    if let Some(ch) = chars.get(state.cursor.byte_offset.saturating_sub(1)) {
                        let command = EditCommand::Delete {
                            position: state.cursor.byte_offset - ch.len_utf8(),
                            text: ch.to_string(),
                        };
                        self.execute_command(command, state);

                        let new_offset = state.cursor.byte_offset.saturating_sub(ch.len_utf8());
                        self.move_cursor_to_byte_offset(new_offset, state);
                    }
                }

                self.update_scroll(state, width, height);
                let current_text = self.get_text_value(state);
                state.is_valid = self.validate(&current_text, &props.validator_pattern);
                props.value = current_text.clone();

                if let Some(on_change) = &self.on_change {
                    on_change(current_text);
                }

                EventResult::Consumed
            }
            KeyCode::Delete => {
                if state.selection.is_some() {
                    if let Some(deleted) = self.delete_selection(state) {
                        let command = EditCommand::Delete {
                            position: state.cursor.byte_offset,
                            text: deleted,
                        };
                        self.execute_command(command, state);
                    }
                } else {
                    let current_text = self.get_text_value(state);
                    let chars: Vec<char> = current_text.chars().collect();
                    if let Some(ch) = chars.get(state.cursor.byte_offset) {
                        let command = EditCommand::Delete {
                            position: state.cursor.byte_offset,
                            text: ch.to_string(),
                        };
                        self.execute_command(command, state);
                    }
                }

                self.update_scroll(state, width, height);
                let current_text = self.get_text_value(state);
                state.is_valid = self.validate(&current_text, &props.validator_pattern);
                props.value = current_text.clone();

                if let Some(on_change) = &self.on_change {
                    on_change(current_text);
                }

                EventResult::Consumed
            }
            KeyCode::Enter => {
                if matches!(props.mode, InputMode::MultiLine { .. }) {
                    // Insert newline in multiline mode
                    if state.selection.is_some() {
                        self.delete_selection(state);
                    }

                    let command = EditCommand::Insert {
                        position: state.cursor.byte_offset,
                        text: "\n".to_string(),
                    };
                    self.execute_command(command, state);

                    let new_offset = state.cursor.byte_offset + 1;
                    self.move_cursor_to_byte_offset(new_offset, state);

                    // Auto-indent if enabled
                    if props.auto_indent && state.cursor.line > 0 {
                        if let Some(prev_line) = state.lines.get(state.cursor.line - 1) {
                            let indent = prev_line
                                .chars()
                                .take_while(|&c| c == ' ' || c == '\t')
                                .collect::<String>();

                            if !indent.is_empty() {
                                let indent_command = EditCommand::Insert {
                                    position: state.cursor.byte_offset,
                                    text: indent.clone(),
                                };
                                self.execute_command(indent_command, state);

                                let new_offset = state.cursor.byte_offset + indent.len();
                                self.move_cursor_to_byte_offset(new_offset, state);
                            }
                        }
                    }

                    let current_text = self.get_text_value(state);
                    props.value = current_text.clone();
                    if let Some(on_change) = &self.on_change {
                        on_change(current_text);
                    }
                } else {
                    // Submit in single-line mode
                    if let Some(on_submit) = &self.on_submit {
                        on_submit(self.get_text_value(state));
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Tab => {
                if state.show_suggestions && !props.suggestions.is_empty() {
                    // Accept current suggestion
                    if let Some(index) = state.suggestion_index {
                        if let Some(suggestion) = props.suggestions.get(index) {
                            // Replace current word with suggestion
                            let command = EditCommand::Insert {
                                position: state.cursor.byte_offset,
                                text: suggestion.insert_text.clone(),
                            };
                            self.execute_command(command, state);

                            let new_offset =
                                state.cursor.byte_offset + suggestion.insert_text.len();
                            self.move_cursor_to_byte_offset(new_offset, state);

                            state.show_suggestions = false;
                            state.suggestion_index = None;
                            props.suggestions.clear();

                            let current_text = self.get_text_value(state);
                            props.value = current_text.clone();
                            if let Some(on_change) = &self.on_change {
                                on_change(current_text);
                            }
                        }
                    }
                } else {
                    // Insert tab character or spaces
                    let tab_text = if props.tab_size > 0 {
                        " ".repeat(props.tab_size)
                    } else {
                        "\t".to_string()
                    };

                    if state.selection.is_some() {
                        self.delete_selection(state);
                    }

                    let command = EditCommand::Insert {
                        position: state.cursor.byte_offset,
                        text: tab_text.clone(),
                    };
                    self.execute_command(command, state);

                    let new_offset = state.cursor.byte_offset + tab_text.len();
                    self.move_cursor_to_byte_offset(new_offset, state);

                    let current_text = self.get_text_value(state);
                    props.value = current_text.clone();
                    if let Some(on_change) = &self.on_change {
                        on_change(current_text);
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Escape => {
                // Hide suggestions
                state.show_suggestions = false;
                state.suggestion_index = None;
                props.suggestions.clear();

                // Clear selection
                state.selection = None;

                EventResult::Consumed
            }
            KeyCode::Left => {
                if event.modifiers.shift {
                    self.extend_selection_left(state);
                } else {
                    state.selection = None;
                    self.move_cursor_left(state);
                }
                self.update_scroll(state, width, height);
                EventResult::Consumed
            }
            KeyCode::Right => {
                if event.modifiers.shift {
                    self.extend_selection_right(state);
                } else {
                    state.selection = None;
                    self.move_cursor_right(state);
                }
                self.update_scroll(state, width, height);
                EventResult::Consumed
            }
            KeyCode::Up => {
                if matches!(props.mode, InputMode::MultiLine { .. }) {
                    if event.modifiers.shift {
                        self.extend_selection_up(state);
                    } else {
                        state.selection = None;
                        self.move_cursor_up(state);
                    }
                    self.update_scroll(state, width, height);
                } else if state.show_suggestions && !props.suggestions.is_empty() {
                    // Navigate suggestions
                    if let Some(index) = state.suggestion_index {
                        state.suggestion_index = Some(index.saturating_sub(1));
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if matches!(props.mode, InputMode::MultiLine { .. }) {
                    if event.modifiers.shift {
                        self.extend_selection_down(state);
                    } else {
                        state.selection = None;
                        self.move_cursor_down(state);
                    }
                    self.update_scroll(state, width, height);
                } else if state.show_suggestions && !props.suggestions.is_empty() {
                    // Navigate suggestions
                    if let Some(index) = state.suggestion_index {
                        let max_index = props.suggestions.len().saturating_sub(1);
                        state.suggestion_index = Some((index + 1).min(max_index));
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                if event.modifiers.shift && state.selection.is_none() {
                    state.selection = Some(Selection {
                        start: state.cursor.clone(),
                        end: state.cursor.clone(),
                    });
                }

                // Move to start of current line
                state.cursor.column = 0;
                self.update_cursor_byte_offset(state);

                if event.modifiers.shift {
                    if let Some(selection) = &mut state.selection {
                        selection.end = state.cursor.clone();
                    }
                } else {
                    state.selection = None;
                }

                state.scroll_offset_x = 0;
                EventResult::Consumed
            }
            KeyCode::End => {
                if event.modifiers.shift && state.selection.is_none() {
                    state.selection = Some(Selection {
                        start: state.cursor.clone(),
                        end: state.cursor.clone(),
                    });
                }

                // Move to end of current line
                if let Some(line) = state.lines.get(state.cursor.line) {
                    state.cursor.column = line.graphemes(true).count();
                }
                self.update_cursor_byte_offset(state);

                if event.modifiers.shift {
                    if let Some(selection) = &mut state.selection {
                        selection.end = state.cursor.clone();
                    }
                } else {
                    state.selection = None;
                }

                self.update_scroll(state, width, height);
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
                let click_y = event.position.y() as usize;
                let text_start = 3; // Account for status prefix

                // For multiline, calculate line and column
                if matches!(props.mode, InputMode::MultiLine { .. }) {
                    let line =
                        (click_y + state.scroll_offset_y).min(state.lines.len().saturating_sub(1));
                    let column = if click_x >= text_start {
                        let col_pos = click_x - text_start + state.scroll_offset_x;
                        if let Some(line_text) = state.lines.get(line) {
                            col_pos.min(line_text.graphemes(true).count())
                        } else {
                            0
                        }
                    } else {
                        0
                    };

                    state.cursor.line = line;
                    state.cursor.column = column;
                    self.update_cursor_byte_offset(state);
                } else {
                    // Single line
                    if click_x >= text_start {
                        let text_pos = click_x - text_start + state.scroll_offset_x;
                        let byte_offset = text_pos.min(props.value.len());
                        self.move_cursor_to_byte_offset(byte_offset, state);
                    }
                }

                state.selection = None;
                EventResult::Consumed
            }
            MouseEventKind::Drag => {
                if state.is_focused {
                    let drag_x = event.position.x() as usize;
                    let drag_y = event.position.y() as usize;
                    let text_start = 3;

                    // Start selection if not already started
                    if state.selection.is_none() {
                        state.selection = Some(Selection {
                            start: state.cursor.clone(),
                            end: state.cursor.clone(),
                        });
                    }

                    // Calculate drag position
                    if matches!(props.mode, InputMode::MultiLine { .. }) {
                        let line = (drag_y + state.scroll_offset_y)
                            .min(state.lines.len().saturating_sub(1));
                        let column = if drag_x >= text_start {
                            let col_pos = drag_x - text_start + state.scroll_offset_x;
                            if let Some(line_text) = state.lines.get(line) {
                                col_pos.min(line_text.graphemes(true).count())
                            } else {
                                0
                            }
                        } else {
                            0
                        };

                        state.cursor.line = line;
                        state.cursor.column = column;
                        self.update_cursor_byte_offset(state);
                    } else if drag_x >= text_start {
                        let text_pos = drag_x - text_start + state.scroll_offset_x;
                        let byte_offset = text_pos.min(props.value.len());
                        self.move_cursor_to_byte_offset(byte_offset, state);
                    }

                    // Update selection end
                    if let Some(selection) = &mut state.selection {
                        selection.end = state.cursor.clone();
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
        assert_eq!(state.cursor.byte_offset, 1);
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
            cursor: CursorPosition {
                line: 0,
                column: 4,
                byte_offset: 4,
            },
            is_focused: true,
            ..Default::default()
        };

        // Initialize state from props
        input.update(&props, &mut state);

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
            cursor: CursorPosition {
                line: 0,
                column: 5,
                byte_offset: 5,
            },
            is_focused: true,
            selection: Some(Selection {
                start: CursorPosition {
                    line: 0,
                    column: 0,
                    byte_offset: 0,
                },
                end: CursorPosition {
                    line: 0,
                    column: 5,
                    byte_offset: 5,
                },
            }),
            ..Default::default()
        };

        // Initialize state from props
        input.update(&props, &mut state);

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
        assert_eq!(state.cursor.byte_offset, 1);
        assert!(state.selection.is_none());
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
            mode: InputMode::Password,
            ..Default::default()
        };
        let state = TextInputState::default();

        let rendered = input.render(&props, &state);
        // The render returns a div with text as a child
        if let Some(child) = rendered.children.first() {
            if let crate::component::ElementType::Text(ref t) = child.element_type {
                assert!(t.contains("******")); // Should show masked value
            } else {
                panic!("Expected text child element");
            }
        } else {
            panic!("Expected child element");
        }
    }
}
