//! Confirmation Dialog Implementation
//!
//! Provides confirmation dialogs with customizable buttons, icons, and callbacks.
//! Supports yes/no, ok/cancel, and custom button configurations.

use super::{
    BaseDialogState, DialogBounds, DialogComponent, DialogEventResult, DialogId, DialogPosition,
    DialogResult, DialogTheme, FocusableElementInfo,
};
use crate::component::Element;
use crate::core::geometry::{Point, Rect, Size};
use crate::event::types::{Event, KeyCode, MouseEventKind};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Configuration options for confirmation dialogs
#[derive(Clone)]
pub struct ConfirmationDialogOptions {
    /// Dialog title
    pub title: String,
    /// Main message text
    pub message: String,
    /// Optional detailed description
    pub description: Option<String>,
    /// Dialog icon
    pub icon: Option<ConfirmationIcon>,
    /// Button configuration
    pub buttons: ConfirmationButtons,
    /// Default focused button
    pub default_button: Option<String>,
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
    /// Callback for button clicks
    pub on_button_click: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    /// Callback for dialog close
    pub on_close: Option<Arc<dyn Fn(DialogResult) + Send + Sync>>,
}

impl std::fmt::Debug for ConfirmationDialogOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfirmationDialogOptions")
            .field("title", &self.title)
            .field("message", &self.message)
            .field("description", &self.description)
            .field("icon", &self.icon)
            .field("buttons", &self.buttons)
            .field("default_button", &self.default_button)
            .field("position", &self.position)
            .field("size", &self.size)
            .field("backdrop_closable", &self.backdrop_closable)
            .field("escape_closable", &self.escape_closable)
            .field("css_classes", &self.css_classes)
            .field("on_button_click", &"<function>")
            .field("on_close", &"<function>")
            .finish()
    }
}

/// Icon types for confirmation dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationIcon {
    None,
    Question,
    Warning,
    Error,
    Info,
    Success,
    Custom(String),
}

/// Button configurations for confirmation dialogs
#[derive(Debug, Clone)]
pub enum ConfirmationButtons {
    /// OK button only
    Ok,
    /// OK and Cancel buttons
    OkCancel,
    /// Yes and No buttons
    YesNo,
    /// Yes, No, and Cancel buttons
    YesNoCancel,
    /// Retry and Cancel buttons
    RetryCancel,
    /// Custom button configuration
    Custom(Vec<ConfirmationButton>),
}

/// Individual button configuration
#[derive(Debug, Clone)]
pub struct ConfirmationButton {
    /// Button ID
    pub id: String,
    /// Button text
    pub text: String,
    /// Button style variant
    pub variant: ButtonVariant,
    /// Whether this is the default button
    pub is_default: bool,
    /// Whether this is a cancel button
    pub is_cancel: bool,
    /// Keyboard shortcut
    pub shortcut: Option<KeyCode>,
    /// Custom CSS classes
    pub css_class: Option<String>,
    /// Whether button is enabled
    pub enabled: bool,
}

/// Button style variants
#[derive(Debug, Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
    Info,
    Light,
    Dark,
    Custom(String),
}

/// Confirmation dialog implementation
#[derive(Debug)]
pub struct ConfirmationDialog {
    /// Base dialog state
    state: BaseDialogState,
    /// Dialog options
    options: ConfirmationDialogOptions,
    /// Current button states
    button_states: HashMap<String, ButtonState>,
    /// Currently hovered button
    hovered_button: Option<String>,
    /// Dialog bounds
    bounds: DialogBounds,
}

/// State of individual buttons
#[derive(Debug, Clone)]
struct ButtonState {
    /// Whether button is focused
    focused: bool,
    /// Whether button is pressed
    pressed: bool,
    /// Whether button is hovered
    hovered: bool,
    /// Button bounds
    bounds: Rect,
}

impl ConfirmationDialog {
    /// Create a new confirmation dialog
    pub fn new(id: DialogId, options: ConfirmationDialogOptions) -> Self {
        let mut button_states = HashMap::new();

        // Initialize button states
        let buttons = match &options.buttons {
            ConfirmationButtons::Ok => vec![ConfirmationButton::ok()],
            ConfirmationButtons::OkCancel => {
                vec![ConfirmationButton::ok(), ConfirmationButton::cancel()]
            }
            ConfirmationButtons::YesNo => vec![ConfirmationButton::yes(), ConfirmationButton::no()],
            ConfirmationButtons::YesNoCancel => vec![
                ConfirmationButton::yes(),
                ConfirmationButton::no(),
                ConfirmationButton::cancel(),
            ],
            ConfirmationButtons::RetryCancel => {
                vec![ConfirmationButton::retry(), ConfirmationButton::cancel()]
            }
            ConfirmationButtons::Custom(buttons) => buttons.clone(),
        };

        for button in &buttons {
            button_states.insert(
                button.id.clone(),
                ButtonState {
                    focused: button.is_default,
                    pressed: false,
                    hovered: false,
                    bounds: Rect::default(),
                },
            );
        }

        let bounds = DialogBounds {
            size: options.size,
            min_size: Some(Size::new(300, 150)),
            max_size: Some(Size::new(600, 400)),
            position: options.position.clone(),
            resizable: false,
            draggable: true,
            ..Default::default()
        };

        Self {
            state: BaseDialogState::new(id),
            options,
            button_states,
            hovered_button: None,
            bounds,
        }
    }

    /// Handle button click
    fn handle_button_click(&mut self, button_id: &str) -> DialogEventResult {
        // Call button click callback if provided
        if let Some(callback) = &self.options.on_button_click {
            if !callback(button_id) {
                return DialogEventResult::Handled;
            }
        }

        // Determine result based on button
        let result = match button_id {
            "ok" => DialogResult::Confirmed(None),
            "cancel" => DialogResult::Cancelled,
            "yes" => DialogResult::Confirmed(Some("yes".to_string())),
            "no" => DialogResult::Confirmed(Some("no".to_string())),
            "retry" => DialogResult::Confirmed(Some("retry".to_string())),
            _ => DialogResult::Selected(button_id.to_string()),
        };

        DialogEventResult::Close(result)
    }

    /// Get button by ID
    fn get_button(&self, button_id: &str) -> Option<ConfirmationButton> {
        let buttons = match &self.options.buttons {
            ConfirmationButtons::Ok => vec![ConfirmationButton::ok()],
            ConfirmationButtons::OkCancel => {
                vec![ConfirmationButton::ok(), ConfirmationButton::cancel()]
            }
            ConfirmationButtons::YesNo => vec![ConfirmationButton::yes(), ConfirmationButton::no()],
            ConfirmationButtons::YesNoCancel => vec![
                ConfirmationButton::yes(),
                ConfirmationButton::no(),
                ConfirmationButton::cancel(),
            ],
            ConfirmationButtons::RetryCancel => {
                vec![ConfirmationButton::retry(), ConfirmationButton::cancel()]
            }
            ConfirmationButtons::Custom(buttons) => buttons.clone(),
        };

        buttons.into_iter().find(|b| b.id == button_id)
    }

    /// Focus next button
    fn focus_next_button(&mut self) {
        let button_ids: Vec<String> = self.button_states.keys().cloned().collect();
        if button_ids.is_empty() {
            return;
        }

        let current_focused = button_ids.iter().position(|id| {
            self.button_states
                .get(id)
                .is_some_and(|state| state.focused)
        });

        // Clear current focus
        for state in self.button_states.values_mut() {
            state.focused = false;
        }

        // Set next focus
        let next_index = match current_focused {
            Some(index) => (index + 1) % button_ids.len(),
            None => 0,
        };

        if let Some(button_id) = button_ids.get(next_index) {
            if let Some(state) = self.button_states.get_mut(button_id) {
                state.focused = true;
            }
        }
    }

    /// Focus previous button
    fn focus_previous_button(&mut self) {
        let button_ids: Vec<String> = self.button_states.keys().cloned().collect();
        if button_ids.is_empty() {
            return;
        }

        let current_focused = button_ids.iter().position(|id| {
            self.button_states
                .get(id)
                .is_some_and(|state| state.focused)
        });

        // Clear current focus
        for state in self.button_states.values_mut() {
            state.focused = false;
        }

        // Set previous focus
        let prev_index = match current_focused {
            Some(index) => {
                if index == 0 {
                    button_ids.len() - 1
                } else {
                    index - 1
                }
            }
            None => button_ids.len() - 1,
        };

        if let Some(button_id) = button_ids.get(prev_index) {
            if let Some(state) = self.button_states.get_mut(button_id) {
                state.focused = true;
            }
        }
    }

    /// Get currently focused button
    fn get_focused_button(&self) -> Option<String> {
        self.button_states
            .iter()
            .find(|(_, state)| state.focused)
            .map(|(id, _)| id.clone())
    }
}

impl ConfirmationButton {
    /// Create an OK button
    pub fn ok() -> Self {
        Self {
            id: "ok".to_string(),
            text: "OK".to_string(),
            variant: ButtonVariant::Primary,
            is_default: true,
            is_cancel: false,
            shortcut: Some(KeyCode::Enter),
            css_class: None,
            enabled: true,
        }
    }

    /// Create a Cancel button
    pub fn cancel() -> Self {
        Self {
            id: "cancel".to_string(),
            text: "Cancel".to_string(),
            variant: ButtonVariant::Secondary,
            is_default: false,
            is_cancel: true,
            shortcut: Some(KeyCode::Escape),
            css_class: None,
            enabled: true,
        }
    }

    /// Create a Yes button
    pub fn yes() -> Self {
        Self {
            id: "yes".to_string(),
            text: "Yes".to_string(),
            variant: ButtonVariant::Primary,
            is_default: true,
            is_cancel: false,
            shortcut: Some(KeyCode::Char('y')),
            css_class: None,
            enabled: true,
        }
    }

    /// Create a No button
    pub fn no() -> Self {
        Self {
            id: "no".to_string(),
            text: "No".to_string(),
            variant: ButtonVariant::Secondary,
            is_default: false,
            is_cancel: true,
            shortcut: Some(KeyCode::Char('n')),
            css_class: None,
            enabled: true,
        }
    }

    /// Create a Retry button
    pub fn retry() -> Self {
        Self {
            id: "retry".to_string(),
            text: "Retry".to_string(),
            variant: ButtonVariant::Warning,
            is_default: true,
            is_cancel: false,
            shortcut: Some(KeyCode::Char('r')),
            css_class: None,
            enabled: true,
        }
    }
}

impl DialogComponent for ConfirmationDialog {
    fn id(&self) -> DialogId {
        self.state.id
    }

    fn dialog_type(&self) -> &'static str {
        "confirmation"
    }

    fn render(&self, _bounds: Rect, theme: &DialogTheme) -> Element {
        use crate::builder::div;

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

        // Message
        content_children.push(
            div()
                .class("dialog-message text-lg font-medium mb-2")
                .text(&self.options.message)
                .build(),
        );

        // Description
        if let Some(description) = &self.options.description {
            content_children.push(
                div()
                    .class("dialog-description text-sm text-gray-600 mb-4")
                    .text(description)
                    .build(),
            );
        }

        children.push(
            div()
                .class("dialog-content p-6")
                .children(content_children)
                .build(),
        );

        // Button area
        let button_elements = self.render_buttons(theme);
        children.push(
            div()
                .class("dialog-buttons flex justify-end gap-2 p-4 border-t")
                .children(button_elements)
                .build(),
        );

        div()
            .class(&format!(
                "dialog confirmation-dialog {} {}",
                theme.dialog_bg, theme.border_style
            ))
            .children(children)
            .build()
    }

    fn handle_event(&mut self, event: &Event) -> DialogEventResult {
        match event {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Escape if self.options.escape_closable => {
                        DialogEventResult::Close(DialogResult::Cancelled)
                    }
                    KeyCode::Enter => {
                        if let Some(button_id) = self.get_focused_button() {
                            self.handle_button_click(&button_id)
                        } else {
                            DialogEventResult::NotHandled
                        }
                    }
                    KeyCode::Tab => {
                        self.focus_next_button();
                        DialogEventResult::StateChanged
                    }
                    KeyCode::BackTab => {
                        self.focus_previous_button();
                        DialogEventResult::StateChanged
                    }
                    KeyCode::Char(c) => {
                        // Check for button shortcuts
                        let button_ids: Vec<String> = self.button_states.keys().cloned().collect();
                        for button_id in button_ids {
                            if let Some(button) = self.get_button(&button_id) {
                                if let Some(KeyCode::Char(shortcut)) = button.shortcut {
                                    if c.eq_ignore_ascii_case(&shortcut) {
                                        return self.handle_button_click(&button_id);
                                    }
                                }
                            }
                        }
                        DialogEventResult::NotHandled
                    }
                    _ => DialogEventResult::NotHandled,
                }
            }
            Event::Mouse(mouse_event) => {
                match mouse_event.kind {
                    MouseEventKind::Down => {
                        // Check if click is on a button
                        let button_ids: Vec<String> = self.button_states.keys().cloned().collect();
                        for button_id in button_ids {
                            if let Some(button_state) = self.button_states.get(&button_id) {
                                if button_state.bounds.contains_point(Point::new(
                                    mouse_event.position.x() as usize,
                                    mouse_event.position.y() as usize,
                                )) {
                                    return self.handle_button_click(&button_id);
                                }
                            }
                        }

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
                    MouseEventKind::Move => {
                        // Update hover states
                        let mouse_pos = Point::new(
                            mouse_event.position.x() as usize,
                            mouse_event.position.y() as usize,
                        );

                        let mut state_changed = false;
                        for button_state in self.button_states.values_mut() {
                            let was_hovered = button_state.hovered;
                            button_state.hovered = button_state.bounds.contains_point(mouse_pos);
                            if was_hovered != button_state.hovered {
                                state_changed = true;
                            }
                        }

                        if state_changed {
                            DialogEventResult::StateChanged
                        } else {
                            DialogEventResult::NotHandled
                        }
                    }
                    _ => DialogEventResult::NotHandled,
                }
            }
            _ => DialogEventResult::NotHandled,
        }
    }

    fn update(&mut self, delta_time: Duration) -> bool {
        // Handle animations
        if self.state.is_animating() {
            if let Some(start_time) = self.state.animation_start {
                let elapsed = start_time.elapsed();
                if elapsed >= Duration::from_millis(200) {
                    // Animation complete
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
        1000 // High z-index for dialogs
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
        self.button_states
            .iter()
            .enumerate()
            .map(|(index, (button_id, button_state))| {
                super::dialog_component::FocusableElementInfo {
                    id: button_id.clone(),
                    element_type: super::dialog_component::FocusableElementType::Button,
                    tab_index: index as i32,
                    enabled: true,
                    bounds: button_state.bounds,
                    properties: std::collections::HashMap::new(),
                }
            })
            .collect()
    }

    fn set_focus(&mut self, element_id: &str) -> bool {
        // Clear all focus
        for state in self.button_states.values_mut() {
            state.focused = false;
        }

        // Set focus to specified element
        if let Some(state) = self.button_states.get_mut(element_id) {
            state.focused = true;
            true
        } else {
            false
        }
    }

    fn get_focused_element(&self) -> Option<String> {
        self.get_focused_button()
    }

    fn validate(&self) -> super::dialog_component::ValidationResult {
        super::dialog_component::ValidationResult::default() // Confirmation dialogs don't need validation
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ConfirmationDialog {
    /// Render dialog buttons
    fn render_buttons(&self, theme: &DialogTheme) -> Vec<Element> {
        let buttons = match &self.options.buttons {
            ConfirmationButtons::Ok => vec![ConfirmationButton::ok()],
            ConfirmationButtons::OkCancel => {
                vec![ConfirmationButton::ok(), ConfirmationButton::cancel()]
            }
            ConfirmationButtons::YesNo => vec![ConfirmationButton::yes(), ConfirmationButton::no()],
            ConfirmationButtons::YesNoCancel => vec![
                ConfirmationButton::yes(),
                ConfirmationButton::no(),
                ConfirmationButton::cancel(),
            ],
            ConfirmationButtons::RetryCancel => {
                vec![ConfirmationButton::retry(), ConfirmationButton::cancel()]
            }
            ConfirmationButtons::Custom(buttons) => buttons.clone(),
        };

        buttons
            .into_iter()
            .map(|button| self.render_button(&button, theme))
            .collect()
    }

    /// Render a single button
    fn render_button(&self, button: &ConfirmationButton, _theme: &DialogTheme) -> Element {
        use crate::builder::div;

        // Simplified button rendering using div for now
        div()
            .class("dialog-button btn-primary")
            .text(&button.text)
            .build()
    }
}

impl ToString for ButtonVariant {
    fn to_string(&self) -> String {
        match self {
            ButtonVariant::Primary => "primary".to_string(),
            ButtonVariant::Secondary => "secondary".to_string(),
            ButtonVariant::Success => "success".to_string(),
            ButtonVariant::Warning => "warning".to_string(),
            ButtonVariant::Danger => "danger".to_string(),
            ButtonVariant::Info => "info".to_string(),
            ButtonVariant::Light => "light".to_string(),
            ButtonVariant::Dark => "dark".to_string(),
            ButtonVariant::Custom(name) => name.clone(),
        }
    }
}

impl Default for ConfirmationDialogOptions {
    fn default() -> Self {
        Self {
            title: "Confirm".to_string(),
            message: "Are you sure?".to_string(),
            description: None,
            icon: Some(ConfirmationIcon::Question),
            buttons: ConfirmationButtons::OkCancel,
            default_button: Some("ok".to_string()),
            size: None,
            position: DialogPosition::Center,
            modal: true,
            backdrop_closable: true,
            escape_closable: true,
            css_classes: HashMap::new(),
            on_button_click: None,
            on_close: None,
        }
    }
}
