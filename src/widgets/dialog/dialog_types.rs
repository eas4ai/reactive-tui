//! Common Dialog Types and Utilities
//!
//! Shared types and utilities used across all dialog implementations.

use std::collections::HashMap;
use std::time::Duration;

// Re-export commonly used types from other modules
pub use super::dialog_component::{
    DialogAnimationConfig, DialogAnimationType, DialogBounds, DialogEasing, DialogPosition,
    FocusableElementInfo, FocusableElementType, ValidationResult,
};

/// Quick dialog builder for common dialog types
pub struct DialogBuilder;

impl DialogBuilder {
    /// Create a simple confirmation dialog
    pub fn confirm(title: &str, message: &str) -> super::ConfirmationDialogOptions {
        super::ConfirmationDialogOptions {
            title: title.to_string(),
            message: message.to_string(),
            buttons: super::ConfirmationButtons::YesNo,
            ..Default::default()
        }
    }

    /// Create an OK/Cancel confirmation dialog
    pub fn ok_cancel(title: &str, message: &str) -> super::ConfirmationDialogOptions {
        super::ConfirmationDialogOptions {
            title: title.to_string(),
            message: message.to_string(),
            buttons: super::ConfirmationButtons::OkCancel,
            ..Default::default()
        }
    }

    /// Create a simple input dialog
    pub fn input(title: &str, prompt: &str) -> super::InputDialogOptions {
        super::InputDialogOptions {
            title: title.to_string(),
            prompt: prompt.to_string(),
            ..Default::default()
        }
    }

    /// Create a password input dialog
    pub fn password(title: &str, prompt: &str) -> super::InputDialogOptions {
        super::InputDialogOptions {
            title: title.to_string(),
            prompt: prompt.to_string(),
            input: super::InputFieldConfig {
                input_type: super::InputType::Password,
                required: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create an autocomplete dialog
    pub fn autocomplete(
        title: &str,
        prompt: &str,
        suggestions: Vec<String>,
    ) -> super::AutocompleteDialogOptions {
        super::AutocompleteDialogOptions {
            title: title.to_string(),
            prompt: prompt.to_string(),
            autocomplete: super::AutocompleteConfig {
                static_suggestions: suggestions,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create an error dialog
    pub fn error(title: &str, message: &str) -> super::ConfirmationDialogOptions {
        super::ConfirmationDialogOptions {
            title: title.to_string(),
            message: message.to_string(),
            icon: Some(super::ConfirmationIcon::Error),
            buttons: super::ConfirmationButtons::Ok,
            ..Default::default()
        }
    }

    /// Create a warning dialog
    pub fn warning(title: &str, message: &str) -> super::ConfirmationDialogOptions {
        super::ConfirmationDialogOptions {
            title: title.to_string(),
            message: message.to_string(),
            icon: Some(super::ConfirmationIcon::Warning),
            buttons: super::ConfirmationButtons::OkCancel,
            ..Default::default()
        }
    }

    /// Create an info dialog
    pub fn info(title: &str, message: &str) -> super::ConfirmationDialogOptions {
        super::ConfirmationDialogOptions {
            title: title.to_string(),
            message: message.to_string(),
            icon: Some(super::ConfirmationIcon::Info),
            buttons: super::ConfirmationButtons::Ok,
            ..Default::default()
        }
    }
}

/// Dialog utility functions
pub struct DialogUtils;

impl DialogUtils {
    /// Calculate optimal dialog size based on content
    pub fn calculate_size(content_length: usize, min_width: u16, max_width: u16) -> (u16, u16) {
        let chars_per_line = 60; // Approximate characters per line
        let lines = (content_length / chars_per_line as usize).max(1);

        let width = (content_length as u16 / lines as u16).clamp(min_width, max_width);
        let height = (lines as u16 * 2 + 8).min(40); // Account for padding and buttons

        (width, height)
    }

    /// Create CSS classes string from map
    pub fn build_css_classes(
        base_classes: &[&str],
        custom_classes: &HashMap<String, String>,
    ) -> String {
        let mut classes = base_classes.to_vec();

        for (key, value) in custom_classes {
            if key == "dialog" || key == "content" || key == "button" {
                classes.push(value);
            }
        }

        classes.join(" ")
    }

    /// Validate dialog configuration
    pub fn validate_config(title: &str, message: &str) -> Result<(), String> {
        if title.is_empty() {
            return Err("Dialog title cannot be empty".to_string());
        }

        if message.is_empty() {
            return Err("Dialog message cannot be empty".to_string());
        }

        if title.len() > 100 {
            return Err("Dialog title is too long (max 100 characters)".to_string());
        }

        if message.len() > 1000 {
            return Err("Dialog message is too long (max 1000 characters)".to_string());
        }

        Ok(())
    }

    /// Generate unique dialog ID
    pub fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("dialog_{}", timestamp)
    }

    /// Parse keyboard shortcut string
    pub fn parse_shortcut(shortcut: &str) -> Option<crate::event::types::KeyCode> {
        match shortcut.to_lowercase().as_str() {
            "enter" | "return" => Some(crate::event::types::KeyCode::Enter),
            "esc" | "escape" => Some(crate::event::types::KeyCode::Escape),
            "tab" => Some(crate::event::types::KeyCode::Tab),
            "space" => Some(crate::event::types::KeyCode::Char(' ')),
            "backspace" => Some(crate::event::types::KeyCode::Backspace),
            "delete" => Some(crate::event::types::KeyCode::Delete),
            s if s.len() == 1 => s.chars().next().map(crate::event::types::KeyCode::Char),
            _ => None,
        }
    }
}

/// Dialog theme presets
pub struct DialogThemes;

impl DialogThemes {
    /// Default light theme
    pub fn light() -> super::DialogTheme {
        super::DialogTheme {
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
            animation: super::DialogAnimation::Fade,
        }
    }

    /// Dark theme
    pub fn dark() -> super::DialogTheme {
        super::DialogTheme {
            backdrop_color: "bg-black bg-opacity-70".to_string(),
            dialog_bg: "bg-gray-800 text-white".to_string(),
            border_style: "border border-gray-600 rounded-lg shadow-xl".to_string(),
            title_style: "font-bold text-lg border-b border-gray-600 p-4".to_string(),
            button_styles: {
                let mut styles = HashMap::new();
                styles.insert(
                    "primary".to_string(),
                    "bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700".to_string(),
                );
                styles.insert(
                    "secondary".to_string(),
                    "bg-gray-600 text-white px-4 py-2 rounded hover:bg-gray-700".to_string(),
                );
                styles.insert(
                    "danger".to_string(),
                    "bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700".to_string(),
                );
                styles
            },
            animation: super::DialogAnimation::Slide(super::SlideDirection::Up),
        }
    }

    /// Minimal theme
    pub fn minimal() -> super::DialogTheme {
        super::DialogTheme {
            backdrop_color: "bg-gray-900 bg-opacity-30".to_string(),
            dialog_bg: "bg-white".to_string(),
            border_style: "border-0 rounded-none shadow-none".to_string(),
            title_style: "font-medium text-base p-3".to_string(),
            button_styles: {
                let mut styles = HashMap::new();
                styles.insert(
                    "primary".to_string(),
                    "text-blue-600 px-3 py-1 hover:bg-blue-50".to_string(),
                );
                styles.insert(
                    "secondary".to_string(),
                    "text-gray-600 px-3 py-1 hover:bg-gray-50".to_string(),
                );
                styles.insert(
                    "danger".to_string(),
                    "text-red-600 px-3 py-1 hover:bg-red-50".to_string(),
                );
                styles
            },
            animation: super::DialogAnimation::Scale,
        }
    }

    /// High contrast theme for accessibility
    pub fn high_contrast() -> super::DialogTheme {
        super::DialogTheme {
            backdrop_color: "bg-black".to_string(),
            dialog_bg: "bg-white text-black".to_string(),
            border_style: "border-4 border-black rounded-none".to_string(),
            title_style: "font-bold text-xl border-b-4 border-black p-4".to_string(),
            button_styles: {
                let mut styles = HashMap::new();
                styles.insert(
                    "primary".to_string(),
                    "bg-black text-white px-6 py-3 border-2 border-black".to_string(),
                );
                styles.insert(
                    "secondary".to_string(),
                    "bg-white text-black px-6 py-3 border-2 border-black".to_string(),
                );
                styles.insert(
                    "danger".to_string(),
                    "bg-red-700 text-white px-6 py-3 border-2 border-black".to_string(),
                );
                styles
            },
            animation: super::DialogAnimation::None,
        }
    }
}

/// Animation presets for dialogs
pub struct DialogAnimations;

impl DialogAnimations {
    /// Fade in/out animation
    pub fn fade(duration: Duration) -> DialogAnimationConfig {
        DialogAnimationConfig {
            animation_type: DialogAnimationType::Fade,
            duration,
            easing: DialogEasing::EaseInOut,
            animate_in: true,
            animate_out: true,
        }
    }

    /// Slide up animation
    pub fn slide_up(duration: Duration) -> DialogAnimationConfig {
        DialogAnimationConfig {
            animation_type: DialogAnimationType::SlideUp,
            duration,
            easing: DialogEasing::EaseOut,
            animate_in: true,
            animate_out: true,
        }
    }

    /// Scale animation
    pub fn scale(duration: Duration) -> DialogAnimationConfig {
        DialogAnimationConfig {
            animation_type: DialogAnimationType::Scale,
            duration,
            easing: DialogEasing::Bounce,
            animate_in: true,
            animate_out: false,
        }
    }

    /// Bounce animation
    pub fn bounce(duration: Duration) -> DialogAnimationConfig {
        DialogAnimationConfig {
            animation_type: DialogAnimationType::Bounce,
            duration,
            easing: DialogEasing::Elastic,
            animate_in: true,
            animate_out: false,
        }
    }

    /// No animation
    pub fn none() -> DialogAnimationConfig {
        DialogAnimationConfig {
            animation_type: DialogAnimationType::None,
            duration: Duration::from_millis(0),
            easing: DialogEasing::Linear,
            animate_in: false,
            animate_out: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::dialog::{ConfirmationButtons, ConfirmationIcon, InputType};

    #[test]
    fn test_dialog_builder_confirm() {
        let dialog = DialogBuilder::confirm("Confirm Action", "Are you sure?");
        assert_eq!(dialog.title, "Confirm Action");
        assert_eq!(dialog.message, "Are you sure?");
        assert!(matches!(dialog.buttons, ConfirmationButtons::YesNo));
    }

    #[test]
    fn test_dialog_builder_ok_cancel() {
        let dialog = DialogBuilder::ok_cancel("Save Changes", "Do you want to save?");
        assert_eq!(dialog.title, "Save Changes");
        assert_eq!(dialog.message, "Do you want to save?");
        assert!(matches!(dialog.buttons, ConfirmationButtons::OkCancel));
    }

    #[test]
    fn test_dialog_builder_input() {
        let dialog = DialogBuilder::input("Enter Name", "Please enter your name:");
        assert_eq!(dialog.title, "Enter Name");
        assert_eq!(dialog.prompt, "Please enter your name:");
    }

    #[test]
    fn test_dialog_builder_password() {
        let dialog = DialogBuilder::password("Login", "Enter password:");
        assert_eq!(dialog.title, "Login");
        assert_eq!(dialog.prompt, "Enter password:");
        assert!(matches!(dialog.input.input_type, InputType::Password));
        assert!(dialog.input.required);
    }

    #[test]
    fn test_dialog_builder_autocomplete() {
        let suggestions = vec!["Option 1".to_string(), "Option 2".to_string()];
        let dialog = DialogBuilder::autocomplete("Select Option", "Choose:", suggestions.clone());
        assert_eq!(dialog.title, "Select Option");
        assert_eq!(dialog.prompt, "Choose:");
        assert_eq!(dialog.autocomplete.static_suggestions, suggestions);
    }

    #[test]
    fn test_dialog_builder_error() {
        let dialog = DialogBuilder::error("Error", "Something went wrong!");
        assert_eq!(dialog.title, "Error");
        assert_eq!(dialog.message, "Something went wrong!");
        assert!(matches!(dialog.icon, Some(ConfirmationIcon::Error)));
        assert!(matches!(dialog.buttons, ConfirmationButtons::Ok));
    }

    #[test]
    fn test_dialog_builder_warning() {
        let dialog = DialogBuilder::warning("Warning", "This action is dangerous!");
        assert_eq!(dialog.title, "Warning");
        assert_eq!(dialog.message, "This action is dangerous!");
        assert!(matches!(dialog.icon, Some(ConfirmationIcon::Warning)));
        assert!(matches!(dialog.buttons, ConfirmationButtons::OkCancel));
    }

    #[test]
    fn test_dialog_builder_info() {
        let dialog = DialogBuilder::info("Information", "Process completed successfully.");
        assert_eq!(dialog.title, "Information");
        assert_eq!(dialog.message, "Process completed successfully.");
        assert!(matches!(dialog.icon, Some(ConfirmationIcon::Info)));
        assert!(matches!(dialog.buttons, ConfirmationButtons::Ok));
    }

    #[test]
    fn test_dialog_utils_calculate_size() {
        // Test small content
        let (width, height) = DialogUtils::calculate_size(30, 20, 100);
        assert!((20..=100).contains(&width));
        assert!(height >= 8);

        // Test large content
        let (width, height) = DialogUtils::calculate_size(500, 40, 120);
        assert!((40..=120).contains(&width));
        assert!(height <= 40);

        // Test minimum constraints
        let (width, height) = DialogUtils::calculate_size(5, 50, 100);
        assert_eq!(width, 50); // Should use minimum width
        assert!(height >= 8);
    }

    #[test]
    fn test_dialog_utils_build_css_classes() {
        let base_classes = ["dialog-base", "rounded"];
        let mut custom_classes = HashMap::new();
        custom_classes.insert("dialog".to_string(), "custom-dialog".to_string());
        custom_classes.insert("content".to_string(), "custom-content".to_string());
        custom_classes.insert("button".to_string(), "custom-button".to_string());
        custom_classes.insert("ignored".to_string(), "should-be-ignored".to_string());

        let result = DialogUtils::build_css_classes(&base_classes, &custom_classes);

        assert!(result.contains("dialog-base"));
        assert!(result.contains("rounded"));
        assert!(result.contains("custom-dialog"));
        assert!(result.contains("custom-content"));
        assert!(result.contains("custom-button"));
        assert!(!result.contains("should-be-ignored"));
    }

    #[test]
    fn test_dialog_utils_validate_config_valid() {
        let result = DialogUtils::validate_config("Valid Title", "Valid message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_dialog_utils_validate_config_empty_title() {
        let result = DialogUtils::validate_config("", "Valid message");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("title cannot be empty"));
    }

    #[test]
    fn test_dialog_utils_validate_config_empty_message() {
        let result = DialogUtils::validate_config("Valid Title", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("message cannot be empty"));
    }

    #[test]
    fn test_dialog_utils_validate_config_title_too_long() {
        let long_title = "a".repeat(101);
        let result = DialogUtils::validate_config(&long_title, "Valid message");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("title is too long"));
    }

    #[test]
    fn test_dialog_utils_validate_config_message_too_long() {
        let long_message = "a".repeat(1001);
        let result = DialogUtils::validate_config("Valid Title", &long_message);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("message is too long"));
    }

    #[test]
    fn test_dialog_utils_generate_id() {
        let id1 = DialogUtils::generate_id();
        std::thread::sleep(std::time::Duration::from_millis(1)); // Ensure different timestamps
        let id2 = DialogUtils::generate_id();

        assert!(id1.starts_with("dialog_"));
        assert!(id2.starts_with("dialog_"));
        assert_ne!(id1, id2); // Should be unique
    }

    #[test]
    fn test_dialog_utils_parse_shortcut() {
        assert_eq!(
            DialogUtils::parse_shortcut("enter"),
            Some(crate::event::types::KeyCode::Enter)
        );
        assert_eq!(
            DialogUtils::parse_shortcut("ENTER"),
            Some(crate::event::types::KeyCode::Enter)
        );
        assert_eq!(
            DialogUtils::parse_shortcut("return"),
            Some(crate::event::types::KeyCode::Enter)
        );
        assert_eq!(
            DialogUtils::parse_shortcut("esc"),
            Some(crate::event::types::KeyCode::Escape)
        );
        assert_eq!(
            DialogUtils::parse_shortcut("escape"),
            Some(crate::event::types::KeyCode::Escape)
        );
        assert_eq!(
            DialogUtils::parse_shortcut("tab"),
            Some(crate::event::types::KeyCode::Tab)
        );
        assert_eq!(
            DialogUtils::parse_shortcut("space"),
            Some(crate::event::types::KeyCode::Char(' '))
        );
        assert_eq!(
            DialogUtils::parse_shortcut("backspace"),
            Some(crate::event::types::KeyCode::Backspace)
        );
        assert_eq!(
            DialogUtils::parse_shortcut("delete"),
            Some(crate::event::types::KeyCode::Delete)
        );
        assert_eq!(
            DialogUtils::parse_shortcut("a"),
            Some(crate::event::types::KeyCode::Char('a'))
        );
        assert_eq!(
            DialogUtils::parse_shortcut("Z"),
            Some(crate::event::types::KeyCode::Char('z'))
        );
        assert_eq!(DialogUtils::parse_shortcut("invalid"), None);
        assert_eq!(DialogUtils::parse_shortcut("toolong"), None);
    }
}
