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
    pub on_validate: Option<Arc<dyn Fn(&str) -> ValidationResult + Send + Sync>>,
    /// Callback for input change
    pub on_change: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    /// Callback for dialog submit
    pub on_submit: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    /// Callback for dialog close
    pub on_close: Option<Arc<dyn Fn(DialogResult) + Send + Sync>>,
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
#[derive(Debug, Clone)]
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
    /// Input mask/format
    pub mask: Option<String>,
    /// Whether to show character count
    pub show_count: bool,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

/// Types of input fields
#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Phone,
    Url,
    Search,
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
    pub custom_validator: Option<Arc<dyn Fn(&str) -> ValidationResult + Send + Sync>>,
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
#[derive(Debug, Clone)]
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
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Email,
    Url,
    Number,
    Phone,
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
    /// Create a new input dialog
    pub fn new(id: DialogId, options: InputDialogOptions) -> Self {
        let input_value = options.input.default_value.clone().unwrap_or_default();
        let cursor_position = input_value.len();

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
            input_focused: true,
            bounds,
            last_validation: None,
            pending_validation: false,
        }
    }

    /// Handle text input
    fn handle_text_input(&mut self, text: &str) -> DialogEventResult {
        // Insert text at cursor position
        if let Some((start, end)) = self.selection {
            // Replace selection
            self.input_value.replace_range(start..end, text);
            self.cursor_position = start + text.len();
            self.selection = None;
        } else {
            // Insert at cursor
            self.input_value.insert_str(self.cursor_position, text);
            self.cursor_position += text.len();
        }

        // Check max length
        if let Some(max_length) = self.options.input.max_length {
            if self.input_value.len() > max_length {
                self.input_value.truncate(max_length);
                self.cursor_position = self.cursor_position.min(max_length);
            }
        }

        // Trigger validation if enabled
        if let Some(validation) = &self.options.validation {
            if validation.validate_on_change {
                self.trigger_validation();
            }
        }

        // Call change callback
        if let Some(callback) = &self.options.on_change {
            callback(&self.input_value);
        }

        DialogEventResult::StateChanged
    }

    /// Handle backspace
    fn handle_backspace(&mut self) -> DialogEventResult {
        if let Some((start, end)) = self.selection {
            // Delete selection
            self.input_value.replace_range(start..end, "");
            self.cursor_position = start;
            self.selection = None;
        } else if self.cursor_position > 0 {
            // Delete character before cursor
            self.input_value.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }

        // Trigger validation if enabled
        if let Some(validation) = &self.options.validation {
            if validation.validate_on_change {
                self.trigger_validation();
            }
        }

        // Call change callback
        if let Some(callback) = &self.options.on_change {
            callback(&self.input_value);
        }

        DialogEventResult::StateChanged
    }

    /// Handle delete key
    fn handle_delete(&mut self) -> DialogEventResult {
        if let Some((start, end)) = self.selection {
            // Delete selection
            self.input_value.replace_range(start..end, "");
            self.cursor_position = start;
            self.selection = None;
        } else if self.cursor_position < self.input_value.len() {
            // Delete character after cursor
            self.input_value.remove(self.cursor_position);
        }

        // Trigger validation if enabled
        if let Some(validation) = &self.options.validation {
            if validation.validate_on_change {
                self.trigger_validation();
            }
        }

        // Call change callback
        if let Some(callback) = &self.options.on_change {
            callback(&self.input_value);
        }

        DialogEventResult::StateChanged
    }

    /// Move cursor left
    fn move_cursor_left(&mut self, select: bool) -> DialogEventResult {
        if select {
            if self.selection.is_none() {
                self.selection = Some((self.cursor_position, self.cursor_position));
            }
        } else {
            self.selection = None;
        }

        if self.cursor_position > 0 {
            self.cursor_position -= 1;

            if select {
                if let Some((start, _)) = &mut self.selection {
                    *start = self.cursor_position;
                }
            }
        }

        DialogEventResult::StateChanged
    }

    /// Move cursor right
    fn move_cursor_right(&mut self, select: bool) -> DialogEventResult {
        if select {
            if self.selection.is_none() {
                self.selection = Some((self.cursor_position, self.cursor_position));
            }
        } else {
            self.selection = None;
        }

        if self.cursor_position < self.input_value.len() {
            self.cursor_position += 1;

            if select {
                if let Some((_, end)) = &mut self.selection {
                    *end = self.cursor_position;
                }
            }
        }

        DialogEventResult::StateChanged
    }

    /// Move cursor to beginning
    fn move_cursor_home(&mut self, select: bool) -> DialogEventResult {
        if select {
            if self.selection.is_none() {
                self.selection = Some((self.cursor_position, self.cursor_position));
            }
            if let Some((start, _)) = &mut self.selection {
                *start = 0;
            }
        } else {
            self.selection = None;
        }

        self.cursor_position = 0;
        DialogEventResult::StateChanged
    }

    /// Move cursor to end
    fn move_cursor_end(&mut self, select: bool) -> DialogEventResult {
        if select {
            if self.selection.is_none() {
                self.selection = Some((self.cursor_position, self.cursor_position));
            }
            if let Some((_, end)) = &mut self.selection {
                *end = self.input_value.len();
            }
        } else {
            self.selection = None;
        }

        self.cursor_position = self.input_value.len();
        DialogEventResult::StateChanged
    }

    /// Select all text
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
    fn validate_input(&self, value: &str, validation: &ValidationConfig) -> ValidationResult {
        let mut result = ValidationResult::default();

        for rule in &validation.rules {
            match &rule.rule_type {
                ValidationRuleType::Required => {
                    if value.trim().is_empty() {
                        result.valid = false;
                        result.message = Some(rule.message.clone());
                    }
                }
                ValidationRuleType::MinLength(min_len) => {
                    if value.len() < *min_len {
                        result.valid = false;
                        result.message = Some(rule.message.clone());
                    }
                }
                ValidationRuleType::MaxLength(max_len) => {
                    if value.len() > *max_len {
                        result.valid = false;
                        result.message = Some(rule.message.clone());
                    }
                }
                ValidationRuleType::Pattern(pattern) => {
                    // Simple pattern matching (would use regex in real implementation)
                    if !value.contains(pattern) {
                        result.valid = false;
                        result.message = Some(rule.message.clone());
                    }
                }
                ValidationRuleType::Email => {
                    if !value.contains('@') || !value.contains('.') {
                        result.valid = false;
                        result.message = Some(rule.message.clone());
                    }
                }
                ValidationRuleType::Url => {
                    if !value.starts_with("http://") && !value.starts_with("https://") {
                        result.valid = false;
                        result.message = Some(rule.message.clone());
                    }
                }
                ValidationRuleType::Number => {
                    if value.parse::<f64>().is_err() {
                        result.valid = false;
                        result.message = Some(rule.message.clone());
                    }
                }
                ValidationRuleType::Phone => {
                    // Simple phone validation
                    let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
                    if digits.len() < 10 {
                        result.valid = false;
                        result.message = Some(rule.message.clone());
                    }
                }
                ValidationRuleType::Custom(_) => {
                    // Custom validation would be handled by callback
                }
            }
        }

        // Call custom validator if provided
        if let Some(validator) = &validation.custom_validator {
            let custom_result = validator(value);
            if !custom_result.valid {
                result.valid = false;
                // Custom validation errors are handled in the valid flag
                result.warnings.extend(custom_result.warnings);
            }
        }

        result
    }

    /// Submit the dialog
    fn submit(&mut self) -> DialogEventResult {
        // Validate before submit
        if let Some(validation) = &self.options.validation {
            self.validation_state = self.validate_input(&self.input_value, validation);
            if !self.validation_state.valid {
                return DialogEventResult::StateChanged;
            }
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

    fn render(&self, _bounds: Rect, theme: &DialogTheme) -> Element {
        use crate::builder::{button, div};

        let mut children = Vec::new();

        // Title bar
        if !self.options.title.is_empty() {
            children.push(
                div()
                    .class(&format!("dialog-title {}", theme.title_style))
                    .text(&self.options.title)
                    .build(),
            );
        }

        // Content area
        let mut content_children = Vec::new();

        // Prompt
        content_children.push(
            div()
                .class("dialog-prompt text-lg font-medium mb-4")
                .text(&self.options.prompt)
                .build(),
        );

        // Input field
        let input_element = self.render_input_field(theme);
        content_children.push(input_element);

        // Validation messages
        if !self.validation_state.valid {
            let error_messages: Vec<String> = if let Some(msg) = &self.validation_state.message {
                vec![msg.clone()]
            } else {
                vec![]
            };
            if !error_messages.is_empty() {
                content_children.push(
                    div()
                        .class("validation-errors text-red-500 text-sm mt-2")
                        .text(&error_messages.join(", "))
                        .build(),
                );
            }
        }

        children.push(
            div()
                .class("dialog-content p-6")
                .children(content_children)
                .build(),
        );

        // Button area
        let button_elements = vec![
            button()
                .class("dialog-button btn-secondary mr-2")
                .text("Cancel")
                .build(),
            button()
                .class("dialog-button btn-primary")
                .text("OK")
                .build(),
        ];

        children.push(
            div()
                .class("dialog-buttons flex justify-end gap-2 p-4 border-t")
                .children(button_elements)
                .build(),
        );

        div()
            .class(&format!(
                "dialog input-dialog {} {}",
                theme.dialog_bg, theme.border_style
            ))
            .children(children)
            .build()
    }

    fn handle_event(&mut self, event: &Event) -> DialogEventResult {
        match event {
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
                KeyCode::Char(c) => self.handle_text_input(&c.to_string()),
                _ => DialogEventResult::NotHandled,
            },
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

    fn update(&mut self, delta_time: Duration) -> bool {
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
                enabled: true,
                bounds: Rect::default(), // Would be calculated during render
                properties: std::collections::HashMap::new(),
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
            "input" => {
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
        if let Some(validation) = &self.options.validation {
            // Convert local ValidationResult to dialog_component::ValidationResult
            let local_result = self.validate_input(&self.input_value, validation);
            let mut warnings_map = std::collections::HashMap::new();
            for (i, warning) in local_result.warnings.iter().enumerate() {
                warnings_map.insert(format!("warning_{}", i), warning.clone());
            }

            super::dialog_component::ValidationResult {
                valid: local_result.valid,
                errors: std::collections::HashMap::new(), // Simplified for now
                warnings: warnings_map,
                data: local_result.message,
            }
        } else {
            super::dialog_component::ValidationResult::default()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl InputDialog {
    /// Render the input field
    fn render_input_field(&self, _theme: &DialogTheme) -> Element {
        use crate::builder::input;

        let mut classes = vec!["dialog-input"];

        // Add input type class
        let type_class = match &self.options.input.input_type {
            InputType::Text => "input-text",
            InputType::Password => "input-password",
            InputType::Email => "input-email",
            InputType::Number => "input-number",
            InputType::Phone => "input-phone",
            InputType::Url => "input-url",
            InputType::Search => "input-search",
            InputType::Custom(class) => class,
        };
        classes.push(type_class);

        // Add state classes
        if self.input_focused {
            classes.push("focused");
        }

        if !self.validation_state.valid {
            classes.push("invalid");
        }

        // Add multiline class
        if self.options.input.multiline {
            classes.push("multiline");
        }

        let mut element = input().class(&classes.join(" "));

        // Add placeholder
        if let Some(placeholder) = &self.options.input.placeholder {
            element = element.placeholder(placeholder);
        }

        element.build()
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
