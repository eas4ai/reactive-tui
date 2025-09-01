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
pub mod dialog_buffer;
pub mod dialog_component;
pub mod dialog_types;
pub mod input;
pub mod progress;
pub mod toast;
pub mod wizard;

// Re-export main types
pub use autocomplete::*;
pub use confirmation::*;
pub use dialog_buffer::*;
pub use dialog_component::*;
pub use dialog_types::*;
pub use input::*;
pub use progress::*;
pub use toast::*;
pub use wizard::*;

use crate::core::geometry::Rect;
use std::collections::HashMap;
use std::time::Duration;

/// Main dialog engine that manages all dialog types and their lifecycle
#[derive(Debug)]
pub struct DialogEngine {
    /// Active dialogs by ID
    active_dialogs: HashMap<DialogId, Box<dyn DialogComponent>>,
    /// Dialog stack for proper layering
    dialog_stack: Vec<DialogId>,
    /// Next available dialog ID
    next_id: u32,
    /// Global dialog configuration
    config: DialogEngineConfig,
    /// Event channel for async operations (placeholder)
    event_sender: Option<()>,
    /// Focus management
    focus_manager: DialogFocusManager,
    /// Animation system integration
    #[allow(dead_code)]
    animation_enabled: bool,
}

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
    /// Enable backdrop blur effect
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
#[derive(Debug, Clone)]
pub struct DialogTheme {
    /// Background color for backdrop
    pub backdrop_color: String,
    /// Dialog background color
    pub dialog_bg: String,
    /// Border style
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
    None,
    Fade,
    Slide(SlideDirection),
    Scale,
    Bounce,
    Custom(String),
}

/// Slide animation directions
#[derive(Debug, Clone, PartialEq)]
pub enum SlideDirection {
    Up,
    Down,
    Left,
    Right,
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
    Button,
    Input,
    Checkbox,
    Radio,
    Select,
    Link,
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
    fn default() -> Self {
        Self {
            backdrop_color: "bg-black bg-opacity-50".to_string(),
            dialog_bg: "bg-white".to_string(),
            border_style: "border border-gray-300 rounded-lg shadow-lg".to_string(),
            title_style: "font-bold text-lg border-b border-gray-200 p-4".to_string(),
            button_styles: {
                let mut styles = HashMap::new();
                styles.insert(
                    "primary".to_string(),
                    "bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600".to_string(),
                );
                styles.insert(
                    "secondary".to_string(),
                    "bg-gray-200 text-gray-800 px-4 py-2 rounded hover:bg-gray-300".to_string(),
                );
                styles.insert(
                    "danger".to_string(),
                    "bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600".to_string(),
                );
                styles
            },
            animation: DialogAnimation::Fade,
        }
    }
}

impl DialogEngine {
    /// Create a new dialog engine
    pub fn new() -> Self {
        Self::with_config(DialogEngineConfig::default())
    }

    /// Create a new dialog engine with custom configuration
    pub fn with_config(config: DialogEngineConfig) -> Self {
        Self {
            active_dialogs: HashMap::new(),
            dialog_stack: Vec::new(),
            next_id: 1,
            config,
            event_sender: None,
            focus_manager: DialogFocusManager::new(),
            animation_enabled: true,
        }
    }

    /// Enable async event handling (placeholder)
    pub fn enable_async(&mut self) {
        // Placeholder for async event handling
        self.event_sender = Some(());
    }

    /// Show a confirmation dialog
    pub fn show_confirmation(&mut self, options: ConfirmationDialogOptions) -> DialogId {
        let id = self.next_dialog_id();
        let dialog = Box::new(ConfirmationDialog::new(id, options));
        self.add_dialog(id, dialog);
        id
    }

    /// Show an input dialog
    pub fn show_input(&mut self, options: InputDialogOptions) -> DialogId {
        let id = self.next_dialog_id();
        let dialog = Box::new(InputDialog::new(id, options));
        self.add_dialog(id, dialog);
        id
    }

    /// Show an autocomplete dialog
    pub fn show_autocomplete(&mut self, options: AutocompleteDialogOptions) -> DialogId {
        let id = self.next_dialog_id();
        let dialog = Box::new(AutocompleteDialog::new(id, options));
        self.add_dialog(id, dialog);
        id
    }

    /// Show a progress dialog
    pub fn show_progress(&mut self, options: ProgressDialogOptions) -> DialogId {
        let id = self.next_dialog_id();
        let dialog = Box::new(ProgressDialog::new(id, options));
        self.add_dialog(id, dialog);
        id
    }

    /// Show a toast notification
    pub fn show_toast(&mut self, options: ToastOptions) -> DialogId {
        let id = self.next_dialog_id();
        let dialog = Box::new(Toast::new(id, options));
        self.add_dialog(id, dialog);
        id
    }

    /// Show a wizard dialog
    pub fn show_wizard(&mut self, options: WizardDialogOptions) -> DialogId {
        let id = self.next_dialog_id();
        let dialog = Box::new(WizardDialog::new(id, options));
        self.add_dialog(id, dialog);
        id
    }

    /// Close a dialog
    pub fn close_dialog(&mut self, id: DialogId, _result: DialogResult) {
        if let Some(_dialog) = self.active_dialogs.remove(&id) {
            // Remove from stack
            self.dialog_stack.retain(|&dialog_id| dialog_id != id);

            // Update focus
            self.focus_manager.dialog_closed(id);

            // Send event (placeholder)
            if self.event_sender.is_some() {
                // Would send DialogEvent::Closed(id, result) here
            }
        }
    }

    /// Get the next available dialog ID
    fn next_dialog_id(&mut self) -> DialogId {
        let id = DialogId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Add a dialog to the engine
    fn add_dialog(&mut self, id: DialogId, dialog: Box<dyn DialogComponent>) {
        // Check max dialogs limit
        if self.active_dialogs.len() >= self.config.max_dialogs {
            return;
        }

        self.active_dialogs.insert(id, dialog);
        self.dialog_stack.push(id);
        self.focus_manager.dialog_opened(id);

        // Send event (placeholder)
        if self.event_sender.is_some() {
            // Would send DialogEvent::Opened(id) here
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

impl Default for DialogEngine {
    fn default() -> Self {
        Self::new()
    }
}
