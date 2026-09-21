//! Input Dialog Implementation
//!
//! Provides input dialogs with validation, different input types, and async validation support.

use super::{
    BaseDialogState, DialogBounds, DialogComponent, DialogEventResult, DialogId, DialogPosition,
    DialogResult, DialogTheme, FocusableElementInfo, ValidationResult,
};
use crate::component::Element;
use crate::core::geometry::{Point, Rect, Size};
use crate::event::types::{Event, KeyCode, MouseEventKind};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

mod live;
mod validation;

// Type aliases for complex function pointer types
type ValidationCallback = Arc<dyn Fn(&str) -> ValidationResult + Send + Sync>;
type OnChangeCallback = Arc<dyn Fn(&str) + Send + Sync>;
type OnSubmitCallback = Arc<dyn Fn(&str) -> bool + Send + Sync>;
type CustomValidatorCallback = Arc<dyn Fn(&str) -> ValidationResult + Send + Sync>;

/// Configuration options for input dialogs
#[derive(Clone)]
pub struct InputDialogOptions {
    /// Dialog title
    pub title: String,
    /// Prompt message
    pub prompt: String,
    /// Input field configuration
    pub input: InputFieldConfig,
    /// Validation rules
    pub validation: Option<ValidationConfig>,
    /// Dialog size
    pub size: Option<Size>,
    /// Position on screen
    pub position: DialogPosition,
    /// Whether dialog is modal
    pub modal: bool,
    /// Whether backdrop can be clicked to close
    pub backdrop_closable: bool,
    /// Whether escape key closes dialog
    pub escape_closable: bool,
    /// Custom CSS classes
    pub css_classes: HashMap<String, String>,
    /// Callback for input validation
    pub on_validate: Option<ValidationCallback>,
    /// Callback for input change
    pub on_change: Option<OnChangeCallback>,
    /// Callback for dialog submit
    pub on_submit: Option<OnSubmitCallback>,
    /// Callback for dialog close
    pub on_close: Option<Arc<dyn Fn(DialogResult) + Send + Sync>>,
}

impl PartialEq for InputDialogOptions {
    fn eq(&self, other: &Self) -> bool {
        use crate::widgets::display::overlay::same_callback;
        self.title == other.title
            && self.prompt == other.prompt
            && self.input == other.input
            && self.validation == other.validation
            && self.size == other.size
            && self.position == other.position
            && self.modal == other.modal
            && self.backdrop_closable == other.backdrop_closable
            && self.escape_closable == other.escape_closable
            && self.css_classes == other.css_classes
            && same_callback(&self.on_validate, &other.on_validate)
            && same_callback(&self.on_change, &other.on_change)
            && same_callback(&self.on_submit, &other.on_submit)
            && same_callback(&self.on_close, &other.on_close)
    }
}

impl std::fmt::Debug for InputDialogOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputDialogOptions")
            .field("title", &self.title)
            .field("prompt", &self.prompt)
            .field("input", &self.input)
            .field("validation", &self.validation)
            .field("position", &self.position)
            .field("size", &self.size)
            .field("backdrop_closable", &self.backdrop_closable)
            .field("escape_closable", &self.escape_closable)
            .field("css_classes", &self.css_classes)
            .field("on_validate", &"<function>")
            .field("on_change", &"<function>")
            .field("on_submit", &"<function>")
            .field("on_close", &"<function>")
            .finish()
    }
}

/// Input field configuration
#[derive(Debug, Clone, PartialEq)]
pub struct InputFieldConfig {
    /// Input type
    pub input_type: InputType,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Default value
    pub default_value: Option<String>,
    /// Maximum length
    pub max_length: Option<usize>,
    /// Whether input is required
    pub required: bool,
    /// Whether input is multiline
    pub multiline: bool,
    /// Number of rows for multiline input
    pub rows: Option<usize>,
    /// Exact input format, checked by validation and before submission.
    /// `#` matches an ASCII digit, `A` a Unicode alphabetic grapheme,
    /// and `*` any non-control grapheme. Backslash escapes the next grapheme;
    /// other graphemes are literals that the user enters (for example `AA-##`).
    /// Editing preserves the user's text; it does not insert separators.
    /// Empty optional values are allowed. A trailing escape is an error.
    pub mask: Option<String>,
    /// Whether to show character count
    pub show_count: bool,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

impl InputFieldConfig {
    fn attribute_enabled(&self, name: &str) -> bool {
        self.attributes
            .get(name)
            .is_some_and(|value| value != "false")
    }
}

/// Types of input fields
#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    /// Plain text input
    Text,
    /// Password input (masked)
    Password,
    /// Email address input
    Email,
    /// Numeric input
    Number,
    /// Phone number input
    Phone,
    /// URL input
    Url,
    /// Search input
    Search,
    /// Custom input type
    Custom(String),
}

/// Validation configuration
#[derive(Clone)]
pub struct ValidationConfig {
    /// Validation rules
    pub rules: Vec<ValidationRule>,
    /// Whether to validate on change
    pub validate_on_change: bool,
    /// Whether to validate on blur
    pub validate_on_blur: bool,
    /// Debounce delay for validation
    pub debounce_delay: Duration,
    /// URL for async validation
    pub async_validation_url: Option<String>,
    /// Custom validation function
    pub custom_validator: Option<CustomValidatorCallback>,
}

impl PartialEq for ValidationConfig {
    fn eq(&self, other: &Self) -> bool {
        self.rules == other.rules
            && self.validate_on_change == other.validate_on_change
            && self.validate_on_blur == other.validate_on_blur
            && self.debounce_delay == other.debounce_delay
            && self.async_validation_url == other.async_validation_url
            && crate::widgets::display::overlay::same_callback(
                &self.custom_validator,
                &other.custom_validator,
            )
    }
}

impl std::fmt::Debug for ValidationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationConfig")
            .field("rules", &self.rules)
            .field("custom_validator", &"<function>")
            .finish()
    }
}

/// Individual validation rule
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationRule {
    /// Rule type
    pub rule_type: ValidationRuleType,
    /// Error message
    pub message: String,
    /// Whether rule is required or optional
    pub required: bool,
}

/// Types of validation rules
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationRuleType {
    /// Field is required and cannot be empty
    Required,
    /// Minimum length validation
    MinLength(usize),
    /// Maximum length validation
    MaxLength(usize),
    /// Regular expression pattern validation
    Pattern(String),
    /// Email format validation
    Email,
    /// URL format validation
    Url,
    /// Number format validation
    Number,
    /// Phone number format validation
    Phone,
    /// Custom validation with identifier
    Custom(String),
}

/// Input dialog implementation
#[derive(Debug)]
pub struct InputDialog {
    /// Base dialog state
    state: BaseDialogState,
    /// Dialog options
    options: InputDialogOptions,
    /// Current input value
    input_value: String,
    /// Cursor position in input
    cursor_position: usize,
    /// Selection range (start, end)
    selection: Option<(usize, usize)>,
    /// Current validation state
    validation_state: ValidationResult,
    /// Whether input is focused
    input_focused: bool,
    /// Dialog bounds
    bounds: DialogBounds,
    /// Last validation time for debouncing
    last_validation: Option<std::time::Instant>,
    /// Pending validation request
    pending_validation: bool,
}

impl InputDialog {
    pub(super) fn render_with_revision(
        &self,
        bounds: Rect,
        theme: &DialogTheme,
        revision: u64,
    ) -> Element {
        Element::typed::<live::LiveInput>(live::LiveProps {
            id: self.state.id,
            options: self.options.clone(),
            value: self.input_value.clone(),
            bounds,
            theme: theme.clone(),
            revision,
        })
    }
    /// Create a new input dialog
    pub fn new(id: DialogId, options: InputDialogOptions) -> Self {
        let input_value = options.input.default_value.clone().unwrap_or_default();
        let cursor_position = input_value.len();
        let input_focused = !options.input.attribute_enabled("disabled");

        let bounds = DialogBounds {
            size: options.size,
            min_size: Some(Size::new(400, 200)),
            max_size: Some(Size::new(800, 600)),
            position: options.position.clone(),
            resizable: false,
            draggable: true,
            ..Default::default()
        };

        Self {
            state: BaseDialogState::new(id),
            options,
            input_value,
            cursor_position,
            selection: None,
            validation_state: ValidationResult::default(),
            input_focused,
            bounds,
            last_validation: None,
            pending_validation: false,
        }
    }

    fn boundary(&self, offset: usize) -> usize {
        self.input_value
            .grapheme_indices(true)
            .map(|(offset, _)| offset)
            .chain(std::iter::once(self.input_value.len()))
            .take_while(|index| *index <= offset)
            .last()
            .unwrap_or(0)
    }

    fn replace(&mut self, start: usize, end: usize, text: &str) -> DialogEventResult {
        if !self.input_focused
            || self.options.input.attribute_enabled("disabled")
            || self.options.input.attribute_enabled("readonly")
        {
            return DialogEventResult::Handled;
        }
        let start = self.boundary(start.min(end));
        let end = self.boundary(end.max(start));
        let mut value = self.input_value.clone();
        value.replace_range(start..end, text);
        if self
            .options
            .input
            .max_length
            .is_some_and(|maximum| value.graphemes(true).count() > maximum)
            || value == self.input_value
        {
            return DialogEventResult::Handled;
        }
        self.input_value = value;
        let cursor = start + text.len();
        self.cursor_position = self
            .input_value
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .chain(std::iter::once(self.input_value.len()))
            .find(|index| *index >= cursor)
            .unwrap_or(self.input_value.len());
        self.selection = None;
        if self
            .options
            .validation
            .as_ref()
            .is_some_and(|validation| validation.validate_on_change)
        {
            self.trigger_validation();
        }
        if let Some(callback) = &self.options.on_change {
            callback(&self.input_value);
        }
        DialogEventResult::StateChanged
    }

    fn handle_text_input(&mut self, text: &str) -> DialogEventResult {
        let (start, end) = self
            .selection
            .unwrap_or((self.cursor_position, self.cursor_position));
        self.replace(start, end, text)
    }

    fn handle_backspace(&mut self) -> DialogEventResult {
        let (start, end) = self.selection.unwrap_or_else(|| {
            let end = self.boundary(self.cursor_position);
            (
                self.input_value[..end]
                    .grapheme_indices(true)
                    .next_back()
                    .map_or(0, |(index, _)| index),
                end,
            )
        });
        self.replace(start, end, "")
    }

    fn handle_delete(&mut self) -> DialogEventResult {
        let (start, end) = self.selection.unwrap_or_else(|| {
            let start = self.boundary(self.cursor_position);
            (
                start,
                start
                    + self.input_value[start..]
                        .graphemes(true)
                        .next()
                        .map_or(0, str::len),
            )
        });
        self.replace(start, end, "")
    }

    fn move_cursor(&mut self, position: usize, select: bool) -> DialogEventResult {
        let position = self.boundary(position);
        self.selection = if select {
            let anchor = self.selection.map_or(self.cursor_position, |(start, end)| {
                if self.cursor_position == start {
                    end
                } else {
                    start
                }
            });
            (anchor != position).then_some((anchor.min(position), anchor.max(position)))
        } else {
            None
        };
        self.cursor_position = position;
        DialogEventResult::StateChanged
    }

    fn move_cursor_left(&mut self, select: bool) -> DialogEventResult {
        let cursor = self.boundary(self.cursor_position);
        let position = if let Some((start, _)) = self.selection.filter(|_| !select) {
            start
        } else {
            self.input_value[..cursor]
                .grapheme_indices(true)
                .next_back()
                .map_or(0, |(index, _)| index)
        };
        self.move_cursor(position, select)
    }
    fn move_cursor_right(&mut self, select: bool) -> DialogEventResult {
        let cursor = self.boundary(self.cursor_position);
        let position = if let Some((_, end)) = self.selection.filter(|_| !select) {
            end
        } else {
            cursor
                + self.input_value[cursor..]
                    .graphemes(true)
                    .next()
                    .map_or(0, str::len)
        };
        self.move_cursor(position, select)
    }
    fn move_cursor_home(&mut self, select: bool) -> DialogEventResult {
        self.move_cursor(0, select)
    }
    fn move_cursor_end(&mut self, select: bool) -> DialogEventResult {
        self.move_cursor(self.input_value.len(), select)
    }
    fn select_all(&mut self) -> DialogEventResult {
        self.selection = Some((0, self.input_value.len()));
        self.cursor_position = self.input_value.len();
        DialogEventResult::StateChanged
    }

    /// Trigger validation
    fn trigger_validation(&mut self) {
        self.last_validation = Some(std::time::Instant::now());

        // Perform synchronous validation first
        if let Some(validation) = &self.options.validation {
            self.validation_state = self.validate_input(&self.input_value, validation);
        }
    }

    /// Validate input value
    fn validate_input(&self, value: &str, _validation: &ValidationConfig) -> ValidationResult {
        validation::validate(&self.options, value)
    }

    /// Submit the dialog
    fn submit(&mut self) -> DialogEventResult {
        self.validation_state = validation::validate(&self.options, &self.input_value);
        if !self.validation_state.valid {
            return DialogEventResult::StateChanged;
        }

        // Call submit callback
        if let Some(callback) = &self.options.on_submit {
            if !callback(&self.input_value) {
                return DialogEventResult::StateChanged;
            }
        }

        DialogEventResult::Close(DialogResult::Confirmed(Some(self.input_value.clone())))
    }
}

impl DialogComponent for InputDialog {
    fn id(&self) -> DialogId {
        self.state.id
    }

    fn dialog_type(&self) -> &'static str {
        "input"
    }

    fn render(&self, bounds: Rect, theme: &DialogTheme) -> Element {
        self.render_with_revision(bounds, theme, 0)
    }

    fn handle_event(&mut self, event: &Event) -> DialogEventResult {
        match event {
            Event::Key(key_event)
                if key_event.kind == crate::event::types::KeyEventKind::Release =>
            {
                DialogEventResult::NotHandled
            }
            Event::Key(key_event) => match key_event.code {
                KeyCode::Escape if self.options.escape_closable => {
                    DialogEventResult::Close(DialogResult::Cancelled)
                }
                KeyCode::Enter => {
                    if self.options.input.multiline {
                        self.handle_text_input("\n")
                    } else {
                        self.submit()
                    }
                }
                KeyCode::Backspace => self.handle_backspace(),
                KeyCode::Delete => self.handle_delete(),
                KeyCode::Left => self.move_cursor_left(key_event.modifiers.shift),
                KeyCode::Right => self.move_cursor_right(key_event.modifiers.shift),
                KeyCode::Home => self.move_cursor_home(key_event.modifiers.shift),
                KeyCode::End => self.move_cursor_end(key_event.modifiers.shift),
                KeyCode::Char('a') if key_event.modifiers.ctrl => self.select_all(),
                KeyCode::Char(c)
                    if !key_event.modifiers.ctrl
                        && !key_event.modifiers.alt
                        && !key_event.modifiers.meta =>
                {
                    self.handle_text_input(&c.to_string())
                }
                _ => DialogEventResult::NotHandled,
            },
            Event::Paste(paste) => {
                let text = if self.options.input.multiline {
                    paste.content.replace("\r\n", "\n").replace('\r', "\n")
                } else {
                    paste.content.replace(['\r', '\n'], " ")
                };
                self.handle_text_input(&text)
            }
            Event::Mouse(mouse_event) => {
                match mouse_event.kind {
                    MouseEventKind::Down => {
                        // Check if click is outside dialog (backdrop)
                        if self.options.backdrop_closable
                            && !self.state.bounds.contains_point(Point::new(
                                mouse_event.position.x() as usize,
                                mouse_event.position.y() as usize,
                            ))
                        {
                            return DialogEventResult::Close(DialogResult::Cancelled);
                        }
                        DialogEventResult::NotHandled
                    }
                    _ => DialogEventResult::NotHandled,
                }
            }
            _ => DialogEventResult::NotHandled,
        }
    }

    fn update(&mut self, _delta_time: Duration) -> bool {
        // Handle debounced validation
        if let Some(last_validation) = self.last_validation {
            if let Some(validation) = &self.options.validation {
                if last_validation.elapsed() >= validation.debounce_delay
                    && !self.pending_validation
                {
                    // Trigger async validation if configured
                    if validation.async_validation_url.is_some() {
                        self.pending_validation = true;
                        // Would trigger async validation here
                    }
                }
            }
        }

        // Handle animations
        if self.state.is_animating() {
            if let Some(start_time) = self.state.animation_start {
                let elapsed = start_time.elapsed();
                if elapsed >= Duration::from_millis(200) {
                    match self.state.animation_state {
                        super::DialogAnimationState::ShowingIn => {
                            self.state.animation_state = super::DialogAnimationState::Visible;
                        }
                        super::DialogAnimationState::HidingOut => {
                            self.state.animation_state = super::DialogAnimationState::Hidden;
                            self.state.visible = false;
                        }
                        _ => {}
                    }
                    self.state.animation_start = None;
                    return true;
                }
            }
        }

        false
    }

    fn get_bounds(&self) -> DialogBounds {
        self.bounds.clone()
    }

    fn is_modal(&self) -> bool {
        self.options.modal
    }

    fn backdrop_closable(&self) -> bool {
        self.options.backdrop_closable
    }

    fn escape_closable(&self) -> bool {
        self.options.escape_closable
    }

    fn z_index(&self) -> u16 {
        1000
    }

    fn animation(&self) -> Option<super::DialogAnimationConfig> {
        Some(super::DialogAnimationConfig {
            animation_type: super::DialogAnimationType::Fade,
            duration: Duration::from_millis(200),
            easing: super::DialogEasing::EaseInOut,
            animate_in: true,
            animate_out: true,
        })
    }

    fn get_focusable_elements(&self) -> Vec<FocusableElementInfo> {
        vec![
            super::dialog_component::FocusableElementInfo {
                id: "input".to_string(),
                element_type: super::dialog_component::FocusableElementType::Input,
                tab_index: 0,
                enabled: !self.options.input.attribute_enabled("disabled"),
                bounds: Rect::default(), // Would be calculated during render
                properties: self.options.input.attributes.clone(),
            },
            super::dialog_component::FocusableElementInfo {
                id: "cancel".to_string(),
                element_type: super::dialog_component::FocusableElementType::Button,
                tab_index: 1,
                enabled: true,
                bounds: Rect::default(),
                properties: std::collections::HashMap::new(),
            },
            super::dialog_component::FocusableElementInfo {
                id: "ok".to_string(),
                element_type: super::dialog_component::FocusableElementType::Button,
                tab_index: 2,
                enabled: true,
                bounds: Rect::default(),
                properties: std::collections::HashMap::new(),
            },
        ]
    }

    fn set_focus(&mut self, element_id: &str) -> bool {
        match element_id {
            "input" if !self.options.input.attribute_enabled("disabled") => {
                self.input_focused = true;
                true
            }
            "cancel" | "ok" => {
                self.input_focused = false;
                true
            }
            _ => false,
        }
    }

    fn get_focused_element(&self) -> Option<String> {
        if self.input_focused {
            Some("input".to_string())
        } else {
            None
        }
    }

    fn validate(&self) -> super::dialog_component::ValidationResult {
        let result = validation::validate(&self.options, &self.input_value);
        let errors = result
            .message
            .as_ref()
            .filter(|_| !result.valid)
            .map(|message| ("input".to_string(), message.clone()))
            .into_iter()
            .collect();
        super::dialog_component::ValidationResult {
            valid: result.valid,
            errors,
            warnings: result
                .warnings
                .into_iter()
                .enumerate()
                .map(|(index, warning)| (format!("warning_{index}"), warning))
                .collect(),
            data: result.message,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for InputDialogOptions {
    fn default() -> Self {
        Self {
            title: "Input".to_string(),
            prompt: "Please enter a value:".to_string(),
            input: InputFieldConfig::default(),
            validation: None,
            size: None,
            position: DialogPosition::Center,
            modal: true,
            backdrop_closable: true,
            escape_closable: true,
            css_classes: HashMap::new(),
            on_validate: None,
            on_change: None,
            on_submit: None,
            on_close: None,
        }
    }
}

impl Default for InputFieldConfig {
    fn default() -> Self {
        Self {
            input_type: InputType::Text,
            placeholder: None,
            default_value: None,
            max_length: None,
            required: false,
            multiline: false,
            rows: None,
            mask: None,
            show_count: false,
            attributes: HashMap::new(),
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            validate_on_change: true,
            validate_on_blur: true,
            debounce_delay: Duration::from_millis(300),
            async_validation_url: None,
            custom_validator: None,
        }
    }
}

impl ValidationRule {
    /// Create a required field rule
    pub fn required(message: &str) -> Self {
        Self {
            rule_type: ValidationRuleType::Required,
            message: message.to_string(),
            required: true,
        }
    }

    /// Create a minimum length rule
    pub fn min_length(length: usize, message: &str) -> Self {
        Self {
            rule_type: ValidationRuleType::MinLength(length),
            message: message.to_string(),
            required: false,
        }
    }

    /// Create a maximum length rule
    pub fn max_length(length: usize, message: &str) -> Self {
        Self {
            rule_type: ValidationRuleType::MaxLength(length),
            message: message.to_string(),
            required: false,
        }
    }

    /// Create an email validation rule
    pub fn email(message: &str) -> Self {
        Self {
            rule_type: ValidationRuleType::Email,
            message: message.to_string(),
            required: false,
        }
    }

    /// Create a pattern validation rule
    pub fn pattern(pattern: &str, message: &str) -> Self {
        Self {
            rule_type: ValidationRuleType::Pattern(pattern.to_string()),
            message: message.to_string(),
            required: false,
        }
    }
}
