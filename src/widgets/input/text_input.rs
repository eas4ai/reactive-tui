use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

// Type alias for complex function pointer type
type SuggestionRequestCallback = Arc<dyn Fn(String, usize) -> Vec<Suggestion> + Send + Sync>;
use unicode_segmentation::UnicodeSegmentation;
mod accessibility;
mod paint;

/// Input mode for different text input behaviors
#[derive(Clone, Debug, PartialEq, Default)]
pub enum InputMode {
    /// Single line input (default)
    #[default]
    SingleLine,
    /// Multi-line input with specified height
    MultiLine {
        /// Height of the multi-line input in rows
        height: u16,
    },
    /// Password input (masked)
    Password,
    /// Numeric input only
    Numeric,
}

/// Auto-completion suggestion
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    /// Display text for the suggestion
    pub text: String,
    /// Optional description or help text
    pub description: Option<String>,
    /// Text to insert when suggestion is selected
    pub insert_text: String,
}

/// Builder for creating TextInput components with a fluent API
#[derive(Clone, Debug, Default)]
pub struct TextInputBuilder {
    value: String,
    placeholder: Option<String>,
    max_length: Option<usize>,
    disabled: bool,
    width: Option<u16>,
    mode: InputMode,
    validator_pattern: Option<String>,
    error_message: Option<String>,
    suggestions: Vec<Suggestion>,
    show_line_numbers: bool,
    wrap_text: bool,
    tab_size: usize,
    auto_indent: bool,
}

impl TextInputBuilder {
    /// Create a new TextInputBuilder
    pub fn new() -> Self {
        Self {
            wrap_text: true,
            tab_size: 4,
            width: Some(30),
            placeholder: Some("Enter text...".to_string()),
            ..Default::default()
        }
    }

    /// Set the current text value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Set the placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set the maximum text length
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set whether the input is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the fixed width in characters
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the input mode
    pub fn mode(mut self, mode: InputMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set single-line mode
    pub fn single_line(mut self) -> Self {
        self.mode = InputMode::SingleLine;
        self
    }

    /// Set multi-line mode with height
    pub fn multi_line(mut self, height: u16) -> Self {
        self.mode = InputMode::MultiLine { height };
        self
    }

    /// Set password mode
    pub fn password(mut self) -> Self {
        self.mode = InputMode::Password;
        self
    }

    /// Set numeric-only mode
    pub fn numeric(mut self) -> Self {
        self.mode = InputMode::Numeric;
        self
    }

    /// Set validation pattern
    pub fn validator_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.validator_pattern = Some(pattern.into());
        self
    }

    /// Set error message
    pub fn error_message(mut self, message: impl Into<String>) -> Self {
        self.error_message = Some(message.into());
        self
    }

    /// Add suggestions
    pub fn suggestions(mut self, suggestions: Vec<Suggestion>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// Add a single suggestion
    pub fn suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Set whether to show line numbers (multi-line mode)
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Set whether to wrap text
    pub fn wrap_text(mut self, wrap: bool) -> Self {
        self.wrap_text = wrap;
        self
    }

    /// Set tab size in spaces
    pub fn tab_size(mut self, size: usize) -> Self {
        self.tab_size = size;
        self
    }

    /// Set whether to auto-indent new lines
    pub fn auto_indent(mut self, auto: bool) -> Self {
        self.auto_indent = auto;
        self
    }

    /// Build the TextInputProps
    pub fn build(self) -> TextInputProps {
        TextInputProps {
            value: self.value,
            placeholder: self.placeholder,
            max_length: self.max_length,
            disabled: self.disabled,
            width: self.width,
            mode: self.mode,
            validator_pattern: self.validator_pattern,
            error_message: self.error_message,
            suggestions: self.suggestions,
            show_line_numbers: self.show_line_numbers,
            wrap_text: self.wrap_text,
            tab_size: self.tab_size,
            auto_indent: self.auto_indent,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("TextInput").with_props(self.build())
    }
}

/// Properties for TextInput component
#[derive(Clone, Debug, PartialEq)]
pub struct TextInputProps {
    /// Current text value
    pub value: String,
    /// Placeholder text when empty
    pub placeholder: Option<String>,
    /// Maximum allowed text length
    pub max_length: Option<usize>,
    /// Whether the input is disabled
    pub disabled: bool,
    /// Fixed width in characters
    pub width: Option<u16>,
    /// Input mode (single line, multi-line, password, etc.)
    pub mode: InputMode,
    /// Regex pattern for validation
    pub validator_pattern: Option<String>,
    /// Error message to display
    pub error_message: Option<String>,
    /// Auto-completion suggestions
    pub suggestions: Vec<Suggestion>,
    /// Whether to show line numbers (multi-line mode)
    pub show_line_numbers: bool,
    /// Whether to wrap text at boundaries
    pub wrap_text: bool,
    /// Tab size in spaces
    pub tab_size: usize,
    /// Whether to auto-indent new lines
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
    /// Logical line, starting at zero.
    pub line: usize,
    /// Grapheme index within the logical line.
    pub column: usize,
    /// UTF-8 byte offset in the full text, always at a grapheme boundary.
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
    /// Horizontal scroll offset in terminal cells.
    pub scroll_offset_x: usize,
    /// Vertical scroll offset in rendered rows, including wrapped rows.
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
    read_only: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    clipboard_content: Option<String>,
    viewport: Option<crate::component::LayoutInfo>,
    validator: Mutex<Option<(String, Option<regex::Regex>)>>,
}

impl TextInput {
    pub(crate) fn with_read_only(
        mut self,
        read_only: impl Fn() -> bool + Send + Sync + 'static,
    ) -> Self {
        self.read_only = Some(Arc::new(read_only));
        self
    }

    fn is_read_only(&self) -> bool {
        self.read_only.as_ref().is_some_and(|read_only| read_only())
    }

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
            value.split('\n').map(|s| s.to_string()).collect()
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
            match pattern_str.as_str() {
                "numeric" => value.chars().all(|c| c.is_numeric() || c.is_whitespace()),
                "alpha" => value
                    .chars()
                    .all(|c| c.is_alphabetic() || c.is_whitespace()),
                "alphanumeric" => value
                    .chars()
                    .all(|c| c.is_alphanumeric() || c.is_whitespace()),
                pattern => {
                    let pattern = if pattern == "email" {
                        r"^[^\s@]+@[^\s@]+\.[^\s@]+$"
                    } else {
                        pattern
                    };
                    let mut cache = self.validator.lock().unwrap();
                    if cache
                        .as_ref()
                        .is_none_or(|(previous, _)| previous != pattern)
                    {
                        *cache = Some((pattern.to_owned(), regex::Regex::new(pattern).ok()));
                    }
                    cache
                        .as_ref()
                        .and_then(|(_, regex)| regex.as_ref())
                        .is_some_and(|regex| regex.is_match(value))
                }
            }
        } else {
            true
        }
    }

    /// Execute an edit command and add to undo stack
    fn execute_command(&mut self, command: EditCommand, state: &mut TextInputState) {
        if self.is_read_only() {
            return;
        }
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
        let mut current = self.get_text_value(state);
        let (position, removed, inserted) = match command {
            EditCommand::Insert { position, text } => (*position, "", text.as_str()),
            EditCommand::Delete { position, text } => (*position, text.as_str(), ""),
            EditCommand::Replace {
                position,
                old_text,
                new_text,
            } => (*position, old_text.as_str(), new_text.as_str()),
        };
        let end = position.saturating_add(removed.len());
        if current.get(position..end) != Some(removed) {
            return;
        }
        current.replace_range(position..end, inserted);
        self.set_text_value(&current, state);
        state.selection = None;
        self.move_cursor_to_byte_offset(position + inserted.len(), state);
    }

    /// Undo the last edit
    fn undo(&mut self, state: &mut TextInputState) {
        if self.is_read_only() {
            return;
        }
        if let Some(command) = state.undo_stack.pop_back() {
            state.redo_stack.push_back(command.inverse());
            self.apply_command(&command, state);
        }
    }

    /// Redo the last undone edit
    fn redo(&mut self, state: &mut TextInputState) {
        if self.is_read_only() {
            return;
        }
        if let Some(command) = state.redo_stack.pop_back() {
            state.undo_stack.push_back(command.inverse());
            self.apply_command(&command, state);
        }
    }

    /// Find word boundaries for navigation
    fn find_word_start(&self, text: &str, position: usize) -> usize {
        text.unicode_word_indices()
            .take_while(|(start, _)| *start < position)
            .last()
            .map_or(0, |(start, _)| start)
    }

    fn find_word_end(&self, text: &str, position: usize) -> usize {
        text.unicode_word_indices()
            .find(|(start, word)| start + word.len() > position)
            .map_or(text.len(), |(start, word)| start + word.len())
    }

    /// Delete the current selection
    fn delete_selection(&mut self, state: &mut TextInputState) -> Option<String> {
        if self.is_read_only() {
            return None;
        }
        let selection = state.selection.take()?;
        let text = self.get_text_value(state);
        let start = selection.start.byte_offset.min(selection.end.byte_offset);
        let end = selection.start.byte_offset.max(selection.end.byte_offset);
        let deleted = text.get(start..end)?.to_owned();
        if !deleted.is_empty() {
            self.execute_command(
                EditCommand::Delete {
                    position: start,
                    text: deleted.clone(),
                },
                state,
            );
        }
        self.move_cursor_to_byte_offset(start, state);
        Some(deleted)
    }

    fn insert_text(&mut self, inserted: &str, props: &TextInputProps, state: &mut TextInputState) {
        let text = self.get_text_value(state);
        let (start, end) = state.selection.as_ref().map_or(
            (state.cursor.byte_offset, state.cursor.byte_offset),
            |selection| {
                (
                    selection.start.byte_offset.min(selection.end.byte_offset),
                    selection.start.byte_offset.max(selection.end.byte_offset),
                )
            },
        );
        let Some(removed) = text.get(start..end) else {
            return;
        };
        let mut candidate = text.clone();
        candidate.replace_range(start..end, inserted);
        if props
            .max_length
            .is_some_and(|max| candidate.graphemes(true).count() > max)
        {
            return;
        }
        if matches!(props.mode, InputMode::Numeric)
            && !inserted
                .chars()
                .all(|c| c.is_numeric() || matches!(c, '.' | '-'))
        {
            return;
        }
        if candidate == text {
            return;
        }
        self.execute_command(
            EditCommand::Replace {
                position: start,
                old_text: removed.to_owned(),
                new_text: inserted.to_owned(),
            },
            state,
        );
    }

    /// Move cursor to a specific byte offset
    fn move_cursor_to_byte_offset(&self, byte_offset: usize, state: &mut TextInputState) {
        let mut previous = 0;
        for (line_index, line) in state.lines.iter().enumerate() {
            if byte_offset <= previous + line.len() || line_index + 1 == state.lines.len() {
                let local = byte_offset.saturating_sub(previous).min(line.len());
                let mut bytes = 0;
                let mut column = 0;
                for grapheme in line.graphemes(true) {
                    if bytes + grapheme.len() > local {
                        break;
                    }
                    bytes += grapheme.len();
                    column += 1;
                }
                state.cursor = CursorPosition {
                    line: line_index,
                    column,
                    byte_offset: previous + bytes,
                };
                return;
            }
            previous += line.len() + 1;
        }
        state.cursor = CursorPosition::default();
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
        self.copy_selection(state);
        self.delete_selection(state);
    }

    /// Paste from this control's copy/cut buffer.
    fn paste_from_clipboard(&mut self, props: &TextInputProps, state: &mut TextInputState) {
        if let Some(text) = self.clipboard_content.clone() {
            self.insert_text(&text, props, state);
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
                let start = CursorPosition::default();
                let text = self.get_text_value(state);
                self.move_cursor_to_byte_offset(text.len(), state);
                state.selection = Some(Selection {
                    start,
                    end: state.cursor.clone(),
                });
            }
            KeyCode::Char('c') if state.selection.is_some() => self.copy_selection(state),
            KeyCode::Char('x') => self.cut_selection(state),
            KeyCode::Char('v') => self.paste_from_clipboard(props, state),
            KeyCode::Char('z') => self.undo(state),
            KeyCode::Char('y') => self.redo(state),
            KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End => {
                let text = self.get_text_value(state);
                let destination = match event.code {
                    KeyCode::Left => self.find_word_start(&text, state.cursor.byte_offset),
                    KeyCode::Right => self.find_word_end(&text, state.cursor.byte_offset),
                    KeyCode::Home => 0,
                    _ => text.len(),
                };
                let start = state
                    .selection
                    .as_ref()
                    .map_or_else(|| state.cursor.clone(), |selection| selection.start.clone());
                self.move_cursor_to_byte_offset(destination, state);
                state.selection = event.modifiers.shift.then(|| Selection {
                    start,
                    end: state.cursor.clone(),
                });
            }
            KeyCode::Backspace | KeyCode::Delete => {
                if state.selection.is_some() {
                    self.delete_selection(state);
                } else {
                    let text = self.get_text_value(state);
                    let cursor = state.cursor.byte_offset;
                    let (start, end) = if event.code == KeyCode::Backspace {
                        (self.find_word_start(&text, cursor), cursor)
                    } else {
                        (cursor, self.find_word_end(&text, cursor))
                    };
                    if start < end {
                        self.execute_command(
                            EditCommand::Delete {
                                position: start,
                                text: text[start..end].to_owned(),
                            },
                            state,
                        );
                    }
                }
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
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
            read_only: None,
            clipboard_content: None,
            viewport: None,
            validator: Mutex::new(None),
        }
    }

    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        let mut state = TextInputState::default();
        self.set_text_value(&props.value, &mut state);
        state.is_valid = self.validate(&props.value, &props.validator_pattern);
        state
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        // Update text value if it changed
        let current_text = self.get_text_value(state);
        if current_text != props.value {
            self.set_text_value(&props.value, state);
            if let Some(mut selection) = state.selection.take() {
                let cursor = state.cursor.clone();
                self.move_cursor_to_byte_offset(selection.start.byte_offset, state);
                selection.start = state.cursor.clone();
                self.move_cursor_to_byte_offset(selection.end.byte_offset, state);
                selection.end = state.cursor.clone();
                state.cursor = cursor;
                state.selection = (selection.start != selection.end).then_some(selection);
            }
            state.undo_stack.clear();
            state.redo_stack.clear();
            state.show_suggestions = false;
            state.suggestion_index = None;
        }

        // Validate on prop changes
        state.is_valid = self.validate(&props.value, &props.validator_pattern);

        // Always re-render on update
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.render_accessible(props, state)
    }

    fn layout(
        &mut self,
        layout: crate::component::LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        let old = self.view_size(props, state);
        self.viewport = Some(layout);
        let changed = old != self.view_size(props, state);
        self.scroll_to_cursor(props, state);
        changed
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
        let result = match event {
            Event::Key(key) if state.is_focused => self.handle_key_event(key, props, state),
            Event::Mouse(mouse) => self.handle_mouse_event(mouse, props, state),
            Event::Paste(paste) if state.is_focused => {
                let text = if matches!(props.mode, InputMode::MultiLine { .. }) {
                    paste.content.replace("\r\n", "\n").replace('\r', "\n")
                } else {
                    paste.content.replace(['\r', '\n'], " ")
                };
                self.insert_text(&text, props, state);
                EventResult::Consumed
            }
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
        };
        let text = self.get_text_value(state);
        if props.value != text {
            props.value = text.clone();
            state.is_valid = self.validate(&text, &props.validator_pattern);
            if let Some(callback) = &self.on_change {
                callback(text.clone());
            }
            if matches!(event, Event::Key(key) if matches!(key.code, KeyCode::Char(_)) && !key.modifiers.ctrl)
                || matches!(event, Event::Paste(_))
            {
                if let Some(callback) = &self.on_suggestion_request {
                    props.suggestions = callback(text, state.cursor.byte_offset);
                }
                state.show_suggestions = !props.suggestions.is_empty();
                state.suggestion_index = state.show_suggestions.then_some(0);
            }
        }
        if !matches!(event, Event::Mouse(mouse) if mouse.kind == MouseEventKind::Wheel) {
            self.scroll_to_cursor(props, state);
        }
        result
    }
}

impl TextInput {
    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut TextInputProps,
        state: &mut TextInputState,
    ) -> EventResult {
        if event.modifiers.ctrl {
            return self.handle_ctrl_key(event, props, state);
        }
        if event.modifiers.alt {
            return self.handle_alt_key(event, props, state);
        }
        match event.code {
            KeyCode::Char(c) => self.insert_text(&c.to_string(), props, state),
            KeyCode::Space => self.insert_text(" ", props, state),
            KeyCode::Backspace | KeyCode::Delete => {
                if state.selection.is_some() {
                    self.delete_selection(state);
                } else {
                    let text = self.get_text_value(state);
                    let cursor = state.cursor.byte_offset;
                    let range = if event.code == KeyCode::Backspace {
                        text.grapheme_indices(true)
                            .take_while(|(index, _)| *index < cursor)
                            .last()
                            .map(|(index, grapheme)| (index, grapheme.to_owned()))
                    } else {
                        text.get(cursor..)
                            .and_then(|tail| tail.graphemes(true).next())
                            .map(|grapheme| (cursor, grapheme.to_owned()))
                    };
                    if let Some((position, text)) = range {
                        self.execute_command(EditCommand::Delete { position, text }, state);
                    }
                }
            }
            KeyCode::Enter => {
                if matches!(props.mode, InputMode::MultiLine { .. }) {
                    let indent = if props.auto_indent {
                        state.lines[state.cursor.line]
                            .chars()
                            .take_while(|c| matches!(c, ' ' | '\t'))
                            .collect::<String>()
                    } else {
                        String::new()
                    };
                    self.insert_text(&format!("\n{indent}"), props, state);
                } else if let Some(callback) = &self.on_submit {
                    callback(self.get_text_value(state));
                }
            }
            KeyCode::Tab => {
                if self.is_read_only() {
                    return EventResult::Ignored;
                }
                if state.show_suggestions {
                    if let Some(index) = state.suggestion_index {
                        self.accept_suggestion(index, props, state);
                    }
                } else if matches!(props.mode, InputMode::MultiLine { .. }) {
                    let text = if props.tab_size == 0 {
                        "\t".to_owned()
                    } else {
                        " ".repeat(props.tab_size)
                    };
                    self.insert_text(&text, props, state);
                } else {
                    return EventResult::Ignored;
                }
            }
            KeyCode::Escape => {
                if !state.show_suggestions && state.selection.is_none() {
                    return EventResult::Ignored;
                }
                state.show_suggestions = false;
                state.suggestion_index = None;
                state.selection = None;
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                if state.show_suggestions && matches!(event.code, KeyCode::Up | KeyCode::Down) {
                    let index = state.suggestion_index.unwrap_or(0);
                    state.suggestion_index = Some(if event.code == KeyCode::Up {
                        index.saturating_sub(1)
                    } else {
                        (index + 1).min(props.suggestions.len().saturating_sub(1))
                    });
                } else {
                    if !event.modifiers.shift {
                        state.selection = None;
                    }
                    match (&event.code, event.modifiers.shift) {
                        (KeyCode::Left, true) => self.extend_selection_left(state),
                        (KeyCode::Left, false) => self.move_cursor_left(state),
                        (KeyCode::Right, true) => self.extend_selection_right(state),
                        (KeyCode::Right, false) => self.move_cursor_right(state),
                        (KeyCode::Up, true) => self.extend_selection_up(state),
                        (KeyCode::Up, false) => self.move_cursor_up(state),
                        (KeyCode::Down, true) => self.extend_selection_down(state),
                        _ => self.move_cursor_down(state),
                    }
                }
            }
            KeyCode::Home | KeyCode::End | KeyCode::PageUp | KeyCode::PageDown => {
                let start = state
                    .selection
                    .as_ref()
                    .map_or_else(|| state.cursor.clone(), |selection| selection.start.clone());
                match event.code {
                    KeyCode::Home => state.cursor.column = 0,
                    KeyCode::End => {
                        state.cursor.column = state.lines[state.cursor.line].graphemes(true).count()
                    }
                    _ => {
                        let height = match props.mode {
                            InputMode::MultiLine { height } => usize::from(height).max(1),
                            _ => 1,
                        };
                        if event.code == KeyCode::PageUp {
                            state.cursor.line = state.cursor.line.saturating_sub(height);
                        } else {
                            state.cursor.line = state
                                .cursor
                                .line
                                .saturating_add(height)
                                .min(state.lines.len().saturating_sub(1));
                        }
                        state.cursor.column = state
                            .cursor
                            .column
                            .min(state.lines[state.cursor.line].graphemes(true).count());
                    }
                }
                self.update_cursor_byte_offset(state);
                state.selection = event.modifiers.shift.then(|| Selection {
                    start,
                    end: state.cursor.clone(),
                });
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }

    fn accept_suggestion(
        &mut self,
        index: usize,
        props: &TextInputProps,
        state: &mut TextInputState,
    ) {
        if self.is_read_only() {
            return;
        }
        let Some(suggestion) = props.suggestions.get(index) else {
            return;
        };
        let text = self.get_text_value(state);
        let end = state.cursor.clone();
        self.move_cursor_to_byte_offset(self.find_word_start(&text, end.byte_offset), state);
        state.selection = Some(Selection {
            start: state.cursor.clone(),
            end,
        });
        self.insert_text(&suggestion.insert_text, props, state);
        state.show_suggestions = false;
        state.suggestion_index = None;
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut TextInputProps,
        state: &mut TextInputState,
    ) -> EventResult {
        use crate::event::types::MouseButton;
        match event.kind {
            MouseEventKind::Down | MouseEventKind::Click | MouseEventKind::Drag
                if event.button == MouseButton::Left =>
            {
                let (inset_x, inset_y) = self.viewport.map_or((0, 0), |layout| {
                    (layout.insets[0] as usize, layout.insets[1] as usize)
                });
                let (x, y) = (event.position.x() as usize, event.position.y() as usize);
                if x < inset_x || y < inset_y {
                    return EventResult::Ignored;
                }
                let y = y - inset_y;
                let suggestion_top = self.view_size(props, state).1
                    + usize::from(!state.is_valid && props.error_message.is_some());
                if state.show_suggestions && y >= suggestion_top {
                    if event.kind != MouseEventKind::Drag {
                        let range = Self::suggestion_window(props, state);
                        if let Some(index) = range.into_iter().nth(y - suggestion_top) {
                            self.accept_suggestion(index, props, state);
                            return EventResult::Consumed;
                        }
                    }
                    return EventResult::Ignored;
                }
                let position = self.mouse_byte(props, state, x - inset_x, y);
                let Some(position) = position else {
                    return EventResult::Ignored;
                };
                let start = state
                    .selection
                    .as_ref()
                    .map_or_else(|| state.cursor.clone(), |selection| selection.start.clone());
                self.move_cursor_to_byte_offset(position, state);
                state.is_focused = true;
                state.selection = (event.kind == MouseEventKind::Drag || event.modifiers.shift)
                    .then(|| Selection {
                        start,
                        end: state.cursor.clone(),
                    });
                EventResult::Consumed
            }
            MouseEventKind::Wheel => {
                let direction = event
                    .wheel
                    .as_ref()
                    .map(|wheel| match wheel.delta {
                        crate::event::types::WheelDelta::Lines { y, .. }
                        | crate::event::types::WheelDelta::Pixels { y, .. } => y,
                    })
                    .unwrap_or_else(|| match event.button {
                        MouseButton::Back => -1.0,
                        MouseButton::Forward => 1.0,
                        _ => 0.0,
                    });
                if direction == 0.0 {
                    return EventResult::Ignored;
                }
                self.scroll_rows(props, state, direction);
                EventResult::Consumed
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
        let mut state = TextInputState {
            is_focused: true,
            ..TextInputState::default()
        };

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
        let mut pending = vec![&rendered];
        let mut text = String::new();
        while let Some(element) = pending.pop() {
            if let crate::component::ElementType::Text(value) = &element.element_type {
                text.push_str(value);
            }
            pending.extend(element.children.iter().rev());
        }
        assert!(text.contains("******"));
        assert!(!text.contains("secret"));
    }
}
