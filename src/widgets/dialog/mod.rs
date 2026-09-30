//! Comprehensive Dialog Engine
//!
//! A sophisticated dialog system inspired by r3bl-open-core and rat-salsa that provides:
//! - Modal dialogs with async callbacks
//! - Autocomplete dialogs with HTTP backend support
//! - Confirmation dialogs with customizable buttons
//! - Input dialogs with validation
//! - Progress dialogs with cancellation
//! - Toast notifications and alerts
//! - Context menus and dropdown dialogs
//! - Multi-step wizard dialogs
//!
//! The dialog engine uses a glass layer (high z-index) for rendering and provides
//! comprehensive focus management, keyboard navigation, and event handling.

pub mod autocomplete;
pub mod confirmation;
pub mod dialog_component;
pub mod dialog_types;
mod frame;
mod http;
pub mod input;
pub mod progress;
pub mod toast;
pub mod wizard;

// Re-export main types
pub use autocomplete::*;
pub use confirmation::*;
pub use dialog_component::*;
pub use dialog_types::*;
pub use input::*;
pub use progress::*;
pub use toast::*;
pub use wizard::*;

use crate::core::geometry::Rect;
use std::collections::HashMap;
use std::time::Duration;

mod engine;
pub use engine::{
    DialogAnimationFrame, DialogCompletion, DialogEngine, DialogEngineError, DialogEvents,
    DialogUpdate, WeakDialogEngine,
};

/// Unique identifier for dialogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DialogId(u32);

impl DialogId {
    /// Get the inner value for FFI
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Create from u32 for FFI
    pub fn from_u32(id: u32) -> Self {
        DialogId(id)
    }
}

/// Global configuration for the dialog engine
#[derive(Debug, Clone)]
pub struct DialogEngineConfig {
    /// Default animation duration
    pub animation_duration: Duration,
    /// Dim backdrop cells as the terminal approximation of blur.
    pub backdrop_blur: bool,
    /// Default z-index for dialogs
    pub base_z_index: u16,
    /// Maximum number of concurrent dialogs
    pub max_dialogs: usize,
    /// Enable focus trapping
    pub focus_trap: bool,
    /// Enable escape key to close
    pub escape_to_close: bool,
    /// Default dialog theme
    pub default_theme: DialogTheme,
}

/// Dialog theme configuration
#[derive(Debug, Clone, PartialEq)]
pub struct DialogTheme {
    /// Background color for backdrop
    pub backdrop_color: String,
    /// Dialog background color
    pub dialog_bg: String,
    /// Classes added to the box beside `dialog_bg`. The box's border is the
    /// modal's own, in the theme's `border` role; a `border-<color>` class
    /// here paints the box's background instead.
    pub border_style: String,
    /// Title bar style
    pub title_style: String,
    /// Button styles
    pub button_styles: HashMap<String, String>,
    /// Animation type
    pub animation: DialogAnimation,
}

/// Animation types for dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum DialogAnimation {
    /// No animation
    None,
    /// Fade in/out animation
    Fade,
    /// Slide animation with direction
    Slide(SlideDirection),
    /// Scale animation from center
    Scale,
    /// Bounce animation effect
    Bounce,
    /// Custom animation name
    Custom(String),
}

/// Slide animation directions
#[derive(Debug, Clone, PartialEq)]
pub enum SlideDirection {
    /// Slide from/to top
    Up,
    /// Slide from/to bottom
    Down,
    /// Slide from/to left
    Left,
    /// Slide from/to right
    Right,
    /// Expand from/to center
    Center,
}

/// Focus management for dialogs
#[derive(Debug)]
pub struct DialogFocusManager {
    /// Currently focused dialog
    focused_dialog: Option<DialogId>,
    /// Focus history for restoration
    focus_stack: Vec<DialogId>,
    /// Focusable elements in current dialog
    #[allow(dead_code)]
    focusable_elements: Vec<FocusableElement>,
    /// Current focus index
    #[allow(dead_code)]
    current_focus_index: usize,
}

/// Focusable element in a dialog
#[derive(Debug, Clone)]
pub struct FocusableElement {
    /// Element ID
    pub id: String,
    /// Element type
    pub element_type: FocusableElementType,
    /// Bounding rectangle
    pub bounds: Rect,
    /// Tab index
    pub tab_index: i32,
    /// Whether element is currently focusable
    pub enabled: bool,
}

/// Types of focusable elements
#[derive(Debug, Clone, PartialEq)]
pub enum FocusableElementType {
    /// Button element
    Button,
    /// Text input field
    Input,
    /// Checkbox control
    Checkbox,
    /// Radio button
    Radio,
    /// Select dropdown
    Select,
    /// Clickable link
    Link,
    /// Custom element type
    Custom(String),
}

/// Events that can be sent to/from dialogs
#[derive(Debug, Clone)]
pub enum DialogEvent {
    /// Dialog was opened
    Opened(DialogId),
    /// Dialog was closed with result
    Closed(DialogId, DialogResult),
    /// Dialog received input
    Input(DialogId, String),
    /// Dialog button was clicked
    ButtonClicked(DialogId, String),
    /// Dialog focus changed
    FocusChanged(DialogId, Option<String>),
    /// Async operation completed
    AsyncResult(DialogId, AsyncResult),
    /// Dialog animation completed
    AnimationCompleted(DialogId),
    /// Dialog validation result
    ValidationResult(DialogId, ValidationResult),
}

/// Result from dialog operations
#[derive(Debug, Clone)]
pub enum DialogResult {
    /// User confirmed/accepted
    Confirmed(Option<String>),
    /// User cancelled
    Cancelled,
    /// User selected an option
    Selected(String),
    /// Custom result
    Custom(String),
    /// Error occurred
    Error(String),
}

/// Result from async operations
#[derive(Debug, Clone)]
pub enum AsyncResult {
    /// HTTP autocomplete suggestions
    AutocompleteSuggestions(Vec<String>),
    /// Validation result
    ValidationResult(bool, Option<String>),
    /// Custom async result
    Custom(String),
    /// Error occurred
    Error(String),
}

/// Validation result for inputs
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Error message if validation failed
    pub message: Option<String>,
    /// Warnings (non-blocking)
    pub warnings: Vec<String>,
}

impl Default for DialogEngineConfig {
    fn default() -> Self {
        Self {
            animation_duration: Duration::from_millis(200),
            backdrop_blur: true,
            base_z_index: 1000,
            max_dialogs: 10,
            focus_trap: true,
            escape_to_close: true,
            default_theme: DialogTheme::default(),
        }
    }
}

impl Default for DialogTheme {
    /// The look of a dialog whose theme the application did not set
    /// (OVL-001): every color a role of the active theme, one cell of
    /// padding beside a title and inside a button.
    fn default() -> Self {
        use crate::widgets::display::modal::{DANGER_BUTTON, PRIMARY_BUTTON, SECONDARY_BUTTON};
        Self {
            backdrop_color: "bg-overlay".to_string(),
            dialog_bg: "bg-surface text-foreground".to_string(),
            border_style: String::new(),
            title_style: "font-bold border-b px-1".to_string(),
            button_styles: HashMap::from([
                ("primary".to_string(), PRIMARY_BUTTON.to_string()),
                ("secondary".to_string(), SECONDARY_BUTTON.to_string()),
                ("danger".to_string(), DANGER_BUTTON.to_string()),
            ]),
            animation: DialogAnimation::Fade,
        }
    }
}

impl DialogTheme {
    /// The default look with every color taken from `theme` instead of the
    /// active theme: a dialog keeps these colors when the application
    /// changes its theme.
    pub fn of(theme: &crate::theme::Theme) -> Self {
        let button = |fill: &str| {
            format!(
                "px-1 bg-{} text-{} cursor-pointer focus:bg-{} focus:text-{}",
                theme.hex(fill),
                theme.hex(&format!("{fill}-foreground")),
                theme.hex("selection"),
                theme.hex("selection-foreground")
            )
        };
        Self {
            backdrop_color: format!("bg-{}", theme.hex("overlay")),
            dialog_bg: format!(
                "bg-{} text-{}",
                theme.hex("surface"),
                theme.hex("foreground")
            ),
            button_styles: HashMap::from([
                ("primary".to_string(), button("primary")),
                ("secondary".to_string(), button("secondary")),
                ("danger".to_string(), button("error")),
            ]),
            ..Self::default()
        }
    }
}

impl DialogFocusManager {
    fn new() -> Self {
        Self {
            focused_dialog: None,
            focus_stack: Vec::new(),
            focusable_elements: Vec::new(),
            current_focus_index: 0,
        }
    }

    fn dialog_opened(&mut self, id: DialogId) {
        if let Some(current) = self.focused_dialog {
            self.focus_stack.push(current);
        }
        self.focused_dialog = Some(id);
    }

    fn dialog_closed(&mut self, id: DialogId) {
        if self.focused_dialog == Some(id) {
            self.focused_dialog = self.focus_stack.pop();
        }
        self.focus_stack.retain(|&dialog_id| dialog_id != id);
    }
}
