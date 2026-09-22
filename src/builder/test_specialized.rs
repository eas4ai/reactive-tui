//! Tests for specialized widget builders
//!
//! These tests verify that all the specialized widget builders (Tree, Image, etc.)
//! can be created, configured, and built into elements successfully.

#[cfg(test)]
mod tests {
    use super::super::specialized::*;
    use crate::component::{Element, ElementType};
    use crate::widgets::layout::stack::{StackAlignment, StackDirection};
    use crate::widgets::{ImageDisplayMode, ImageFormat, ImageQuality};

    #[test]
    fn test_tree_builder() {
        let tree = tree()
            .selectable(true)
            .multi_select(false)
            .show_icons(true)
            .show_lines(true)
            .checkable(false)
            .build();

        assert!(tree.is_component());
        let props = tree
            .props
            .downcast_ref::<crate::widgets::display::TreeProps>()
            .unwrap();
        assert!(props.selectable && props.show_icons && props.show_lines);
        assert!(!props.multi_select && !props.checkable);
    }

    #[test]
    fn test_image_builder() {
        let image = image()
            .source_file("/path/to/image.png")
            .display_mode(ImageDisplayMode::Auto)
            .quality(ImageQuality::High)
            .format(ImageFormat::PNG)
            .class("w-16 h-8")
            .build();

        assert!(image.is_component());
        assert!(image.metadata.factory.is_some());
        assert_eq!(image.class.as_deref().unwrap().trim(), "w-16 h-8");
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

        assert!(radio.metadata.factory.is_some());
        let props = radio
            .props
            .downcast_ref::<crate::widgets::input::named_radio::NamedRadioProps>()
            .unwrap();
        assert_eq!(props.value, "option1");
        assert_eq!(props.label.as_deref(), Some("Option 1"));
        assert_eq!(props.group.as_deref(), Some("options"));
        assert!(props.checked && !props.disabled);
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

        assert!(slider.metadata.factory.is_some());
        assert!(slider.props.downcast_ref::<SliderBuilder>().is_some());
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

        assert!(scroll_view.metadata.factory.is_some());
        let props = scroll_view
            .props
            .downcast_ref::<crate::widgets::layout::ScrollViewProps>()
            .unwrap();
        assert!(!props.scroll_x && props.scroll_y && props.show_scrollbars);
        let content = props
            .content
            .props
            .downcast_ref::<crate::widgets::layout::StackProps>()
            .unwrap();
        assert_eq!(
            content.children,
            vec![Element::text("Content 1"), Element::text("Content 2")]
        );
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

        assert!(matches!(stack.element_type, ElementType::Layout(_)));
        assert_eq!(
            stack.children,
            vec![Element::text("Child 1"), Element::text("Child 2")]
        );
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

        assert!(dialog.is_component());
        let props = dialog
            .props
            .downcast_ref::<crate::widgets::display::modal::ModalProps>()
            .unwrap();
        assert_eq!(props.title.as_deref(), Some("Test Dialog"));
        assert_eq!(
            props.content.as_ref().unwrap().children,
            vec![Element::text("Dialog content")]
        );
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

        assert!(confirmation.is_component());
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

        assert!(progress.is_component());
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

        assert!(wizard.is_component());
    }

    #[test]
    fn test_builder_from_trait() {
        // Test that builders can be converted to elements using From trait
        let tree: Element = tree().selectable(true).into();
        let image: Element = image().source_file("/test.png").into();
        let radio: Element = RadioButtonBuilder::new().value("test").into();

        assert!(tree.is_component());
        assert!(image.is_component());
        assert!(image.metadata.factory.is_some());
        assert!(radio.metadata.factory.is_some());
    }
}
