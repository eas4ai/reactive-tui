//! Dialog widget builders
//!
//! This module provides builders for various dialog components including modals,
//! toasts, confirmation dialogs, and other overlay components.

use super::super::specialized::{
    ConfirmationDialogBuilder, DialogBuilder, ProgressDialogBuilder, WizardBuilder,
};
use crate::component::{Element, LayoutType};

/// Create a Modal dialog builder
///
/// Returns a `ModalBuilder` for creating modal dialogs with customizable
/// content, sizing, and behavior options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::modal;
///
/// let modal = modal()
///     .title("Settings")
///     .visible(true)
///     .build();
/// ```
pub fn modal() -> ModalBuilder {
    ModalBuilder::new()
}

/// Create a Toast notification builder
///
/// Returns a `ToastBuilder` for creating toast notifications with different
/// types, durations, and positioning options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::toast;
///
/// let toast = toast()
///     .success("Operation completed!")
///     .duration(3000)
///     .build();
/// ```
pub fn toast() -> ToastBuilder {
    ToastBuilder::new()
}

/// Create a Dialog builder
///
/// Returns a `DialogBuilder` for creating basic dialog components.
pub fn dialog() -> DialogBuilder {
    DialogBuilder::new()
}

/// Create a Confirmation Dialog builder
///
/// Returns a `ConfirmationDialogBuilder` for creating confirmation dialogs.
pub fn confirmation_dialog() -> ConfirmationDialogBuilder {
    ConfirmationDialogBuilder::new()
}

/// Create a Progress Dialog builder
///
/// Returns a `ProgressDialogBuilder` for creating progress dialogs.
pub fn progress_dialog() -> ProgressDialogBuilder {
    ProgressDialogBuilder::new()
}

/// Create a Wizard builder
///
/// Returns a `WizardBuilder` for creating multi-step wizard components.
pub fn wizard() -> WizardBuilder {
    WizardBuilder::new()
}

/// Builder for Modal dialog components
///
/// Provides a fluent API for creating modal dialogs with customizable
/// content, sizing, and behavior options.
///
/// # Example
/// ```rust,ignore
/// use reactive_tui::builder::modal;
/// use reactive_tui::component::Element;
///
/// let modal = modal()
///     .title("Settings")
///     .content(Element::text("Configure your preferences"))
///     .visible(true)
///     .size(600, 400)
///     .build();
/// ```
pub struct ModalBuilder {
    title: Option<String>,
    content: Vec<Element>,
    visible: bool,
    closable: bool,
    backdrop_dismissible: bool,
    width: Option<u16>,
    height: Option<u16>,
    class: Option<String>,
}

impl ModalBuilder {
    /// Create a new ModalBuilder with default values
    fn new() -> Self {
        Self {
            title: None,
            content: Vec::new(),
            visible: false,
            closable: true,
            backdrop_dismissible: true,
            width: None,
            height: None,
            class: None,
        }
    }

    /// Set the modal title
    ///
    /// # Arguments
    /// * `title` - The title text to display in the modal header
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Add content element to the modal
    ///
    /// # Arguments
    /// * `element` - The element to add to the modal content area
    pub fn content(mut self, element: Element) -> Self {
        self.content.push(element);
        self
    }

    /// Add multiple content elements to the modal
    ///
    /// # Arguments
    /// * `elements` - Vector of elements to add to the modal content area
    pub fn contents(mut self, elements: Vec<Element>) -> Self {
        self.content.extend(elements);
        self
    }

    /// Set the modal visibility state
    ///
    /// # Arguments
    /// * `visible` - Whether the modal should be visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set whether the modal can be closed by the user
    ///
    /// # Arguments
    /// * `closable` - Whether the modal shows a close button
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set whether clicking the backdrop dismisses the modal
    ///
    /// # Arguments
    /// * `dismissible` - Whether clicking outside the modal closes it
    pub fn backdrop_dismissible(mut self, dismissible: bool) -> Self {
        self.backdrop_dismissible = dismissible;
        self
    }

    /// Set the modal dimensions
    ///
    /// # Arguments
    /// * `width` - Modal width in characters
    /// * `height` - Modal height in characters
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the modal
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Modal element
    ///
    /// Creates a modal dialog element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the modal dialog
    pub fn build(self) -> Element {
        use crate::widgets::display::modal::{Modal, ModalProps, ModalSize};
        Modal::with_props(ModalProps {
            title: self.title,
            content: Some(
                Element::layout(LayoutType::Flex)
                    .class("flex-col")
                    .children(self.content),
            ),
            visible: self.visible,
            closable: self.closable,
            backdrop_clickable: self.backdrop_dismissible,
            width: self.width.map_or(ModalSize::Auto, ModalSize::Fixed),
            height: self.height.map_or(ModalSize::Auto, ModalSize::Fixed),
            modal_style: self.class.or_else(|| ModalProps::default().modal_style),
            ..Default::default()
        })
    }
}

impl From<ModalBuilder> for Element {
    fn from(builder: ModalBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Toast notification components
///
/// Provides a fluent API for creating toast notifications with different
/// types, durations, and positioning options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::toast;
///
/// let toast = toast()
///     .success("Operation completed successfully!")
///     .duration(3000)
///     .position("top-right")
///     .build();
/// ```
pub struct ToastBuilder {
    message: String,
    toast_type: String, // success, error, warning, info
    duration: Option<u64>,
    closable: bool,
    position: String,
    class: Option<String>,
}

impl ToastBuilder {
    /// Create a new ToastBuilder with default values
    fn new() -> Self {
        Self {
            message: String::new(),
            toast_type: "info".to_string(),
            duration: Some(3000),
            closable: true,
            position: "top-right".to_string(),
            class: None,
        }
    }

    /// Set the toast message text
    ///
    /// # Arguments
    /// * `message` - The message text to display in the toast
    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// Set the toast type
    ///
    /// # Arguments
    /// * `toast_type` - The type of toast (success, error, warning, info)
    pub fn toast_type(mut self, toast_type: &str) -> Self {
        self.toast_type = toast_type.to_string();
        self
    }

    /// Create a success toast with message
    ///
    /// # Arguments
    /// * `message` - Success message to display
    pub fn success(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self.toast_type = "success".to_string();
        self
    }

    /// Create an error toast with message
    ///
    /// # Arguments
    /// * `message` - Error message to display
    pub fn error(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self.toast_type = "error".to_string();
        self
    }

    /// Create a warning toast with message
    ///
    /// # Arguments
    /// * `message` - Warning message to display
    pub fn warning(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self.toast_type = "warning".to_string();
        self
    }

    /// Create an info toast with message
    ///
    /// # Arguments
    /// * `message` - Info message to display
    pub fn info(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self.toast_type = "info".to_string();
        self
    }

    /// Set the auto-dismiss duration
    ///
    /// # Arguments
    /// * `duration_ms` - Duration in milliseconds before auto-dismiss
    pub fn duration(mut self, duration_ms: u64) -> Self {
        self.duration = Some(duration_ms);
        self
    }

    /// Make the toast persistent (no auto-dismiss)
    pub fn persistent(mut self) -> Self {
        self.duration = None;
        self
    }

    /// Set whether the toast can be manually closed
    ///
    /// # Arguments
    /// * `closable` - Whether the toast shows a close button
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set the toast position on screen
    ///
    /// # Arguments
    /// * `position` - Position string (top-right, top-left, bottom-center, etc.)
    pub fn position(mut self, position: &str) -> Self {
        self.position = position.to_string();
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the toast
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Toast element
    ///
    /// Creates a toast notification element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the toast notification
    pub fn build(self) -> Element {
        use crate::widgets::dialog::{Toast, ToastOptions, ToastPosition, ToastType};
        let position = match self.position.as_str() {
            "top-left" => ToastPosition::TopLeft,
            "top-center" => ToastPosition::TopCenter,
            "top-right" => ToastPosition::TopRight,
            "bottom-left" => ToastPosition::BottomLeft,
            "bottom-center" => ToastPosition::BottomCenter,
            "bottom-right" => ToastPosition::BottomRight,
            _ => return Element::text(format!("Invalid toast position: {}", self.position)),
        };
        let toast_type = match self.toast_type.as_str() {
            "success" => ToastType::Success,
            "error" => ToastType::Error,
            "warning" => ToastType::Warning,
            "info" => ToastType::Info,
            _ => ToastType::Custom(self.toast_type),
        };
        Toast::element(
            ToastOptions {
                message: self.message,
                toast_type,
                position,
                duration: self.duration.map(std::time::Duration::from_millis),
                closable: self.closable,
                on_close: None,
            },
            self.class,
        )
    }
}

impl From<ToastBuilder> for Element {
    fn from(builder: ToastBuilder) -> Self {
        builder.build()
    }
}

// Note: Placeholder builders (DialogBuilder, ConfirmationDialogBuilder,
// ProgressDialogBuilder, WizardBuilder) are now provided by the placeholders module
