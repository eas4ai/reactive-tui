//! Tests for specialized widget builders
//!
//! These tests verify that all the specialized widget builders (Tree, Image, etc.)
//! can be created, configured, and built into elements successfully.

#[cfg(test)]
mod tests {
    use super::super::specialized::*;
    use crate::widgets::{ImageDisplayMode, ImageFormat, ImageQuality};
    use crate::widgets::layout::stack::{StackDirection, StackAlignment};
    use crate::component::{Element, ElementType};

    #[test]
    fn test_tree_builder() {
        let tree = tree()
            .selectable(true)
            .multi_select(false)
            .show_icons(true)
            .show_lines(true)
            .checkable(false)
            .build();
        
        // Should create a text element describing the tree configuration
        assert!(matches!(tree.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_image_builder() {
        let image = image()
            .source_file("/path/to/image.png")
            .display_mode(ImageDisplayMode::Auto)
            .quality(ImageQuality::High)
            .format(ImageFormat::PNG)
            .build();

        // Should create a text element describing the image configuration
        assert!(matches!(image.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_radio_button_builder() {
        let radio = RadioButtonBuilder::new()
            .value("option1")
            .label("Option 1")
            .checked(true)
            .disabled(false)
            .group("options")
            .build();
        
        // Should create a text element describing the radio button
        assert!(matches!(radio.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_slider_builder() {
        let slider = SliderBuilder::new()
            .value(50.0)
            .min(0.0)
            .max(100.0)
            .step(1.0)
            .label("Volume")
            .disabled(false)
            .build();
        
        // Should create a text element describing the slider
        assert!(matches!(slider.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_scroll_view_builder() {
        let scroll_view = ScrollViewBuilder::new()
            .content(Element::text("Content 1"))
            .content(Element::text("Content 2"))
            .horizontal_scroll(false)
            .vertical_scroll(true)
            .show_scrollbars(true)
            .build();
        
        // Should create a text element describing the scroll view
        assert!(matches!(scroll_view.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_stack_builder() {
        let stack = StackBuilder::new()
            .child(Element::text("Child 1"))
            .child(Element::text("Child 2"))
            .direction(StackDirection::Horizontal)
            .alignment(StackAlignment::Center)
            .spacing(10.0)
            .build();
        
        // Should create a text element describing the stack
        assert!(matches!(stack.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_dialog_builder() {
        let dialog = DialogBuilder::new()
            .title("Test Dialog")
            .content(Element::text("Dialog content"))
            .modal(true)
            .closable(true)
            .width(400)
            .height(300)
            .build();
        
        // Should create a text element describing the dialog
        assert!(matches!(dialog.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_confirmation_dialog_builder() {
        let confirmation = ConfirmationDialogBuilder::new()
            .title("Confirm Action")
            .message("Are you sure you want to proceed?")
            .confirm_text("Yes")
            .cancel_text("No")
            .danger(true)
            .build();
        
        // Should create a text element describing the confirmation dialog
        assert!(matches!(confirmation.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_progress_dialog_builder() {
        let progress = ProgressDialogBuilder::new()
            .title("Processing")
            .message("Please wait...")
            .progress(0.75)
            .indeterminate(false)
            .cancelable(true)
            .show_percentage(true)
            .build();
        
        // Should create a text element describing the progress dialog
        assert!(matches!(progress.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_wizard_builder() {
        let step1 = WizardStep::new("Step 1")
            .content(Element::text("Step 1 content"))
            .can_proceed(true);
        
        let step2 = WizardStep::new("Step 2")
            .content(Element::text("Step 2 content"))
            .can_proceed(false);
        
        let wizard = WizardBuilder::new()
            .title("Setup Wizard")
            .step(step1)
            .step(step2)
            .current_step(0)
            .show_progress(true)
            .cancelable(true)
            .build();
        
        // Should create a text element describing the wizard
        assert!(matches!(wizard.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_builder_from_trait() {
        // Test that builders can be converted to elements using From trait
        let tree: Element = tree().selectable(true).into();
        let image: Element = image().source_file("/test.png").into();
        let radio: Element = RadioButtonBuilder::new().value("test").into();
        
        assert!(matches!(tree.element_type, ElementType::Text(_)));
        assert!(matches!(image.element_type, ElementType::Text(_)));
        assert!(matches!(radio.element_type, ElementType::Text(_)));
    }
}
