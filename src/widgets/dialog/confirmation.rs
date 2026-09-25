//! Confirmation Dialog Implementation
//!
//! Provides confirmation dialogs with customizable buttons, icons, and callbacks.
//! Supports yes/no, ok/cancel, and custom button configurations.

mod live;

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

// Type alias for complex function pointer type
type ButtonClickCallback = Arc<dyn Fn(&str) -> bool + Send + Sync>;

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
    pub on_button_click: Option<ButtonClickCallback>,
    /// Callback for dialog close
    pub on_close: Option<Arc<dyn Fn(DialogResult) + Send + Sync>>,
}

impl PartialEq for ConfirmationDialogOptions {
    fn eq(&self, other: &Self) -> bool {
        use crate::widgets::display::overlay::same_callback;
        self.title == other.title
            && self.message == other.message
            && self.description == other.description
            && self.icon == other.icon
            && self.buttons == other.buttons
            && self.default_button == other.default_button
            && self.size == other.size
            && self.position == other.position
            && self.modal == other.modal
            && self.backdrop_closable == other.backdrop_closable
            && self.escape_closable == other.escape_closable
            && self.css_classes == other.css_classes
            && same_callback(&self.on_button_click, &other.on_button_click)
            && same_callback(&self.on_close, &other.on_close)
    }
}

impl ConfirmationButtons {
    fn entries(&self) -> Vec<ConfirmationButton> {
        match self {
            Self::Ok => vec![ConfirmationButton::ok()],
            Self::OkCancel => vec![ConfirmationButton::ok(), ConfirmationButton::cancel()],
            Self::YesNo => vec![ConfirmationButton::yes(), ConfirmationButton::no()],
            Self::YesNoCancel => vec![
                ConfirmationButton::yes(),
                ConfirmationButton::no(),
                ConfirmationButton::cancel(),
            ],
            Self::RetryCancel => vec![ConfirmationButton::retry(), ConfirmationButton::cancel()],
            Self::Custom(buttons) => buttons.clone(),
        }
    }
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
    /// No icon displayed
    None,
    /// Question mark icon
    Question,
    /// Warning/caution icon
    Warning,
    /// Error/danger icon
    Error,
    /// Information icon
    Info,
    /// Success/checkmark icon
    Success,
    /// Custom icon with specified text
    Custom(String),
}

/// Button configurations for confirmation dialogs
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
    /// Primary action button (emphasized)
    Primary,
    /// Secondary action button (normal)
    Secondary,
    /// Success/positive action button
    Success,
    /// Warning/caution action button
    Warning,
    /// Danger/destructive action button
    Danger,
    /// Information action button
    Info,
    /// Light colored button
    Light,
    /// Dark colored button
    Dark,
    /// Custom styled button
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
        let buttons = options.buttons.entries();

        let focused = options
            .default_button
            .as_deref()
            .filter(|id| {
                buttons
                    .iter()
                    .any(|button| button.enabled && button.id == *id)
            })
            .or_else(|| {
                buttons
                    .iter()
                    .find(|button| button.enabled && button.is_default)
                    .map(|button| button.id.as_str())
            })
            .or_else(|| {
                buttons
                    .iter()
                    .find(|button| button.enabled)
                    .map(|button| button.id.as_str())
            });
        for button in &buttons {
            button_states.insert(
                button.id.clone(),
                ButtonState {
                    focused: focused == Some(button.id.as_str()),
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
        let Some(button) = self.get_button(button_id).filter(|button| button.enabled) else {
            return DialogEventResult::NotHandled;
        };
        // Call button click callback if provided
        if let Some(callback) = &self.options.on_button_click {
            if !callback(button_id) {
                return DialogEventResult::Handled;
            }
        }

        DialogEventResult::Close(button.result())
    }

    /// Get button by ID
    fn get_button(&self, button_id: &str) -> Option<ConfirmationButton> {
        let buttons = self.options.buttons.entries();

        buttons.into_iter().find(|b| b.id == button_id)
    }

    fn move_focus(&mut self, forward: bool) {
        let buttons: Vec<_> = self
            .options
            .buttons
            .entries()
            .into_iter()
            .filter(|button| button.enabled)
            .collect();
        if buttons.is_empty() {
            return;
        }
        let current = buttons.iter().position(|button| {
            self.button_states
                .get(&button.id)
                .is_some_and(|state| state.focused)
        });
        let next = match current {
            Some(index) if forward => (index + 1) % buttons.len(),
            Some(0) | None if !forward => buttons.len() - 1,
            Some(index) => index - 1,
            None => 0,
        };
        for (id, state) in &mut self.button_states {
            state.focused = *id == buttons[next].id;
        }
    }

    fn focus_next_button(&mut self) {
        self.move_focus(true);
    }
    fn focus_previous_button(&mut self) {
        self.move_focus(false);
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
    fn matches_shortcut(&self, code: &KeyCode) -> bool {
        match (&self.shortcut, code) {
            (Some(KeyCode::Char(shortcut)), KeyCode::Char(value)) => {
                value.eq_ignore_ascii_case(shortcut)
            }
            (Some(shortcut), value) => shortcut == value,
            (None, _) => false,
        }
    }

    fn result(&self) -> DialogResult {
        match self.id.as_str() {
            "ok" => DialogResult::Confirmed(None),
            "cancel" => DialogResult::Cancelled,
            "yes" | "no" | "retry" => DialogResult::Confirmed(Some(self.id.clone())),
            _ if self.is_cancel => DialogResult::Cancelled,
            _ => DialogResult::Selected(self.id.clone()),
        }
    }
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

    fn render(&self, bounds: Rect, theme: &DialogTheme) -> Element {
        Element::typed::<live::LiveConfirmation>(live::LiveProps {
            id: self.state.id,
            options: self.options.clone(),
            bounds,
            theme: theme.clone(),
        })
    }

    fn handle_event(&mut self, event: &Event) -> DialogEventResult {
        match event {
            Event::Key(key_event) => {
                if key_event.kind == crate::event::types::KeyEventKind::Release
                    || key_event.modifiers.ctrl
                    || key_event.modifiers.alt
                    || key_event.modifiers.meta
                {
                    return DialogEventResult::NotHandled;
                }
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
                    _ => {
                        match self.options.buttons.entries().into_iter().find(|button| {
                            button.enabled && button.matches_shortcut(&key_event.code)
                        }) {
                            Some(button) => self.handle_button_click(&button.id),
                            None => DialogEventResult::NotHandled,
                        }
                    }
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

    fn update(&mut self, _delta_time: Duration) -> bool {
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

impl std::fmt::Display for ButtonVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Success => "success",
            ButtonVariant::Warning => "warning",
            ButtonVariant::Danger => "danger",
            ButtonVariant::Info => "info",
            ButtonVariant::Light => "light",
            ButtonVariant::Dark => "dark",
            ButtonVariant::Custom(name) => name,
        };
        write!(f, "{}", s)
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
