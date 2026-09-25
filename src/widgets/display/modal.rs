use super::{Border, ScrollState};
use crate::component::{Component, Element, Props};
use std::sync::Arc;

mod live;

/// Props for the Modal component
#[derive(Clone)]
pub struct ModalProps {
    /// Whether the modal is visible
    pub visible: bool,
    /// Optional title for the modal
    pub title: Option<String>,
    /// Content element to display
    pub content: Option<Element>,
    /// Width of the modal
    pub width: ModalSize,
    /// Height of the modal
    pub height: ModalSize,
    /// Position of the modal
    pub position: ModalPosition,
    /// Whether modal can be closed
    pub closable: bool,
    /// Whether clicking backdrop closes modal
    pub backdrop_clickable: bool,
    /// Whether keyboard navigation is enabled
    pub keyboard_navigation: bool,
    /// Border configuration
    pub border: Border,
    /// CSS style for backdrop
    pub backdrop_style: Option<String>,
    /// CSS style for modal container
    pub modal_style: Option<String>,
    /// CSS style for header
    pub header_style: Option<String>,
    /// CSS style for content area
    pub content_style: Option<String>,
    /// CSS style for footer
    pub footer_style: Option<String>,
    /// CSS style for close button
    pub close_button_style: Option<String>,
    /// Animation configuration
    pub animation: ModalAnimation,
    /// Z-index for layering
    pub z_index: u16,
    /// Whether content is scrollable
    pub scrollable: bool,
    /// Whether modal can be resized
    pub resizable: bool,
    /// Whether modal can be dragged
    pub draggable: bool,
    /// Optional footer element
    pub footer: Option<Element>,
    /// Action buttons for the modal
    pub buttons: Vec<ModalButton>,
    /// Whether to trap focus within modal
    pub focus_trap: bool,
    /// Whether to auto-focus first element
    pub auto_focus: bool,
    /// Callback when modal is closed
    pub on_close: Option<Arc<dyn Fn(ModalCloseReason) + Send + Sync>>,
    /// Callback when confirmed
    pub on_confirm: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback when cancelled
    pub on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback when button is clicked
    pub on_button_click: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

/// Size specification for modal dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum ModalSize {
    /// Automatic sizing based on content
    Auto,
    /// Fixed size in terminal cells
    Fixed(u16),
    /// Percentage of parent container
    Percent(f32),
    /// Fraction of viewport size
    Viewport(f32),
}

/// Position specification for modal dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum ModalPosition {
    /// Center of the screen
    Center,
    /// Top center
    Top,
    /// Bottom center
    Bottom,
    /// Left center
    Left,
    /// Right center
    Right,
    /// Top-left corner
    TopLeft,
    /// Top-right corner
    TopRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-right corner
    BottomRight,
    /// Custom position with x, y coordinates
    Custom {
        /// X coordinate in terminal cells
        x: u16,
        /// Y coordinate in terminal cells
        y: u16,
    },
}

/// Animation types for modal transitions
#[derive(Debug, Clone, PartialEq)]
pub enum ModalAnimation {
    /// No animation
    None,
    /// Fade in/out animation
    Fade,
    /// Slide animation
    Slide,
    /// Scale animation
    Scale,
    /// Bounce animation
    Bounce,
}

/// Reason why a modal was closed
#[derive(Debug, Clone, PartialEq)]
pub enum ModalCloseReason {
    /// Modal closed via close button
    CloseButton,
    /// Modal closed via escape key
    EscapeKey,
    /// Modal closed by clicking backdrop
    BackdropClick,
    /// Modal closed by clicking a button
    ButtonClick(String),
}

/// Button configuration for modal dialogs
#[derive(Debug, Clone, PartialEq)]
pub struct ModalButton {
    /// Unique identifier for the button
    pub id: String,
    /// Display text for the button
    pub label: String,
    /// Action to perform when clicked
    pub action: ModalButtonAction,
    /// Optional CSS class or style
    pub style: Option<String>,
    /// Whether the button is disabled
    pub disabled: bool,
    /// Whether the button should have autofocus
    pub autofocus: bool,
}

/// Action to perform when a modal button is clicked
#[derive(Debug, Clone, PartialEq)]
pub enum ModalButtonAction {
    /// Close the modal without action
    Close,
    /// Confirm the modal action
    Confirm,
    /// Cancel the modal action
    Cancel,
    /// Custom action with identifier
    Custom(String),
}

impl Default for ModalProps {
    fn default() -> Self {
        Self {
            visible: false,
            title: None,
            content: None,
            width: ModalSize::Auto,
            height: ModalSize::Auto,
            position: ModalPosition::Center,
            closable: true,
            backdrop_clickable: true,
            keyboard_navigation: true,
            border: Border::default(),
            backdrop_style: Some("bg-black/50".to_string()),
            modal_style: Some("bg-white text-black shadow-lg".to_string()),
            header_style: Some("border-b font-bold".to_string()),
            content_style: None,
            footer_style: Some("border-t".to_string()),
            close_button_style: Some("text-gray-500 hover:text-gray-700".to_string()),
            animation: ModalAnimation::Fade,
            z_index: 1000,
            scrollable: true,
            resizable: false,
            draggable: false,
            footer: None,
            buttons: Vec::new(),
            focus_trap: true,
            auto_focus: true,
            on_close: None,
            on_confirm: None,
            on_cancel: None,
            on_button_click: None,
        }
    }
}

impl PartialEq for ModalProps {
    fn eq(&self, other: &Self) -> bool {
        self.visible == other.visible
            && self.content == other.content
            && self.footer == other.footer
            && self.title == other.title
            && self.width == other.width
            && self.height == other.height
            && self.position == other.position
            && self.closable == other.closable
            && self.backdrop_clickable == other.backdrop_clickable
            && self.keyboard_navigation == other.keyboard_navigation
            && self.border == other.border
            && self.backdrop_style == other.backdrop_style
            && self.modal_style == other.modal_style
            && self.header_style == other.header_style
            && self.content_style == other.content_style
            && self.footer_style == other.footer_style
            && self.close_button_style == other.close_button_style
            && self.animation == other.animation
            && self.z_index == other.z_index
            && self.scrollable == other.scrollable
            && self.resizable == other.resizable
            && self.draggable == other.draggable
            && self.buttons == other.buttons
            && self.focus_trap == other.focus_trap
            && self.auto_focus == other.auto_focus
            && super::overlay::same_callback(&self.on_close, &other.on_close)
            && super::overlay::same_callback(&self.on_confirm, &other.on_confirm)
            && super::overlay::same_callback(&self.on_cancel, &other.on_cancel)
            && super::overlay::same_callback(&self.on_button_click, &other.on_button_click)
    }
}

impl Props for ModalProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// State for the Modal component
#[derive(Debug, Clone, Default)]
pub struct ModalState {
    /// Whether the modal has focus
    pub focused: bool,
    /// Scroll state for modal content
    pub scroll_state: ScrollState,
    /// Current animation state
    pub animation_state: ModalAnimationState,
    /// Current position (x, y) of the modal
    pub position: (u16, u16),
    /// Current size (width, height) of the modal
    pub size: (u16, u16),
    /// Whether the modal is being dragged
    pub dragging: bool,
    /// Drag offset from mouse position
    pub drag_offset: (u16, u16),
    /// Whether the modal is being resized
    pub resizing: bool,
    /// Current resize handle being used
    pub resize_handle: Option<ResizeHandle>,
    /// Index of the currently focused button
    pub focused_button: Option<usize>,
    /// Whether to show entrance/exit animations
    pub show_animation: bool,
    /// Current animation frame counter
    pub animation_frame: u64,
}

/// Animation state for modal transitions
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ModalAnimationState {
    /// Modal is completely hidden
    #[default]
    Hidden,
    /// Modal is animating in (showing)
    Showing,
    /// Modal is fully visible
    Visible,
    /// Modal is animating out (hiding)
    Hiding,
}

/// Resize handle positions for modal dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeHandle {
    /// Top-left corner handle
    TopLeft,
    /// Top edge handle
    Top,
    /// Top-right corner handle
    TopRight,
    /// Right edge handle
    Right,
    /// Bottom-right corner handle
    BottomRight,
    /// Bottom center position
    Bottom,
    /// Bottom left corner position
    BottomLeft,
    /// Left center position
    Left,
}

/// Modal component for overlaying content
pub struct Modal;

type MotionCallback =
    dyn Fn(f32, &mut crate::layout::style::StyleBuilder) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
pub(in crate::widgets) struct Motion {
    pub duration: std::time::Duration,
    pub apply: Arc<MotionCallback>,
}
impl PartialEq for Motion {
    fn eq(&self, other: &Self) -> bool {
        self.duration == other.duration && Arc::ptr_eq(&self.apply, &other.apply)
    }
}

impl Modal {
    pub(in crate::widgets) fn with_presentation(
        props: ModalProps,
        role: crate::accessibility::Role,
        escape_closable: bool,
        motion: Option<Motion>,
        on_presented: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Element {
        Element::typed::<live::LiveModal>(live::LiveProps {
            config: props,
            seed: ModalState::default(),
            role,
            escape_closable,
            motion,
            on_presented,
        })
    }
    /// Create a Modal element with default props
    pub fn element() -> Element {
        Element::component_with_props("Modal", ModalProps::default())
    }

    /// Create a Modal element with custom props
    pub fn with_props(props: ModalProps) -> Element {
        Element::component_with_props("Modal", props)
    }

    /// Builder method for visibility
    pub fn visible(mut props: ModalProps, visible: bool) -> ModalProps {
        props.visible = visible;
        props
    }

    /// Builder method for title
    pub fn with_title(mut props: ModalProps, title: &str) -> ModalProps {
        props.title = Some(title.to_string());
        props
    }

    /// Builder method for content
    pub fn with_content(mut props: ModalProps, content: Element) -> ModalProps {
        props.content = Some(content);
        props
    }

    /// Builder method for size
    pub fn with_size(mut props: ModalProps, width: ModalSize, height: ModalSize) -> ModalProps {
        props.width = width;
        props.height = height;
        props
    }

    /// Builder method for position
    pub fn with_position(mut props: ModalProps, position: ModalPosition) -> ModalProps {
        props.position = position;
        props
    }

    /// Builder method for closable
    pub fn closable(mut props: ModalProps, closable: bool) -> ModalProps {
        props.closable = closable;
        props
    }

    /// Builder method for buttons
    pub fn with_buttons(mut props: ModalProps, buttons: Vec<ModalButton>) -> ModalProps {
        props.buttons = buttons;
        props
    }

    /// Calculate modal dimensions
    fn calculate_dimensions(
        &self,
        props: &ModalProps,
        viewport_width: u16,
        viewport_height: u16,
    ) -> (u16, u16) {
        let width = match props.width {
            ModalSize::Auto => viewport_width / 2,
            ModalSize::Fixed(w) => w,
            ModalSize::Percent(p) => (viewport_width as f32 * p / 100.0) as u16,
            ModalSize::Viewport(v) => (viewport_width as f32 * v) as u16,
        };

        let height = match props.height {
            ModalSize::Auto => viewport_height / 2,
            ModalSize::Fixed(h) => h,
            ModalSize::Percent(p) => (viewport_height as f32 * p / 100.0) as u16,
            ModalSize::Viewport(v) => (viewport_height as f32 * v) as u16,
        };

        (width.min(viewport_width), height.min(viewport_height))
    }

    /// Calculate modal position
    fn calculate_position(
        &self,
        props: &ModalProps,
        modal_size: (u16, u16),
        viewport_size: (u16, u16),
    ) -> (u16, u16) {
        let (modal_width, modal_height) = modal_size;
        let (viewport_width, viewport_height) = viewport_size;

        match props.position {
            ModalPosition::Center => (
                (viewport_width - modal_width) / 2,
                (viewport_height - modal_height) / 2,
            ),
            ModalPosition::Top => ((viewport_width - modal_width) / 2, 0),
            ModalPosition::Bottom => (
                (viewport_width - modal_width) / 2,
                viewport_height - modal_height,
            ),
            ModalPosition::Left => (0, (viewport_height - modal_height) / 2),
            ModalPosition::Right => (
                viewport_width - modal_width,
                (viewport_height - modal_height) / 2,
            ),
            ModalPosition::TopLeft => (0, 0),
            ModalPosition::TopRight => (viewport_width - modal_width, 0),
            ModalPosition::BottomLeft => (0, viewport_height - modal_height),
            ModalPosition::BottomRight => {
                (viewport_width - modal_width, viewport_height - modal_height)
            }
            ModalPosition::Custom { x, y } => (x, y),
        }
    }

    /// Handle close action
    fn close_modal(&self, props: &ModalProps, reason: ModalCloseReason) {
        if let Some(callback) = &props.on_close {
            callback(reason);
        }
    }

    /// Handle button click
    fn handle_button_click(&self, props: &ModalProps, button: &ModalButton) {
        if button.disabled {
            return;
        }

        if let Some(callback) = &props.on_button_click {
            callback(match &button.action {
                ModalButtonAction::Custom(action) => action.clone(),
                _ => button.id.clone(),
            });
        }
        match &button.action {
            ModalButtonAction::Close => {
                self.close_modal(props, ModalCloseReason::ButtonClick(button.id.clone()));
            }
            ModalButtonAction::Confirm => {
                if let Some(callback) = &props.on_confirm {
                    callback();
                }
                self.close_modal(props, ModalCloseReason::ButtonClick(button.id.clone()));
            }
            ModalButtonAction::Cancel => {
                if let Some(callback) = &props.on_cancel {
                    callback();
                }
                self.close_modal(props, ModalCloseReason::ButtonClick(button.id.clone()));
            }
            ModalButtonAction::Custom(_) => {}
        }
    }
}

impl Component for Modal {
    type Props = ModalProps;
    type State = ModalState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveModal>(live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
            role: crate::accessibility::Role::Dialog,
            escape_closable: props.keyboard_navigation,
            motion: None,
            on_presented: None,
        })
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self
    }
}

// Helper implementations
impl ModalButton {
    /// Create a new modal button
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the button
    /// * `label` - Display text for the button
    /// * `action` - Action to perform when clicked
    ///
    /// # Returns
    /// A new `ModalButton` instance
    pub fn new(id: &str, label: &str, action: ModalButtonAction) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            action,
            style: None,
            disabled: false,
            autofocus: false,
        }
    }

    /// Set the button style
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Set whether the button is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether the button has autofocus
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }

    /// Create an OK button
    pub fn ok() -> Self {
        Self::new("ok", "OK", ModalButtonAction::Confirm)
    }

    /// Create a Cancel button
    pub fn cancel() -> Self {
        Self::new("cancel", "Cancel", ModalButtonAction::Cancel)
    }

    /// Create a "Close" button for dialogs
    ///
    /// # Returns
    /// A `ModalButton` configured as a close button
    pub fn close() -> Self {
        Self::new("close", "Close", ModalButtonAction::Close)
    }

    /// Create a "Yes" button for confirmation dialogs
    ///
    /// # Returns
    /// A `ModalButton` configured as a confirmation button
    pub fn yes() -> Self {
        Self::new("yes", "Yes", ModalButtonAction::Confirm)
    }

    /// Create a "No" button for confirmation dialogs
    ///
    /// # Returns
    /// A `ModalButton` configured as a cancel button
    pub fn no() -> Self {
        Self::new("no", "No", ModalButtonAction::Cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_props() -> ModalProps {
        ModalProps {
            visible: true,
            title: Some("Test Modal".to_string()),
            closable: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_modal_creation() {
        let modal = Modal;
        let props = create_test_props();
        let state = ModalState::default();

        let element = modal.render(&props, &state);
        assert!(matches!(
            element.element_type,
            crate::component::ElementType::Component(_)
        ));
        assert!(element.metadata.factory.is_some());
    }

    #[test]
    fn test_modal_visibility() {
        let modal = Modal;
        let mut props = create_test_props();
        let state = ModalState::default();

        // Visible modal
        let element = modal.render(&props, &state);
        assert!(
            element
                .props
                .downcast_ref::<live::LiveProps>()
                .unwrap()
                .config
                .visible
        );

        // Hidden modal
        props.visible = false;
        let element = modal.render(&props, &state);
        assert!(
            !element
                .props
                .downcast_ref::<live::LiveProps>()
                .unwrap()
                .config
                .visible
        );
    }

    #[test]
    fn test_dimension_calculation() {
        let modal = Modal;
        let props = ModalProps {
            width: ModalSize::Percent(50.0),
            height: ModalSize::Fixed(200),
            ..Default::default()
        };

        let (width, height) = modal.calculate_dimensions(&props, 800, 600);
        assert_eq!(width, 400); // 50% of 800
        assert_eq!(height, 200); // Fixed 200
    }

    #[test]
    fn test_position_calculation() {
        let modal = Modal;
        let props = ModalProps {
            position: ModalPosition::Center,
            ..Default::default()
        };

        let (x, y) = modal.calculate_position(&props, (400, 200), (800, 600));
        assert_eq!(x, 200); // (800 - 400) / 2
        assert_eq!(y, 200); // (600 - 200) / 2

        // Test custom position
        let custom_props = ModalProps {
            position: ModalPosition::Custom { x: 100, y: 50 },
            ..Default::default()
        };
        let (x, y) = modal.calculate_position(&custom_props, (400, 200), (800, 600));
        assert_eq!(x, 100);
        assert_eq!(y, 50);
    }

    #[test]
    fn test_button_handling() {
        use std::sync::{Arc, Mutex};
        let modal = Modal;
        let button = ModalButton::new("test", "Test", ModalButtonAction::Close);
        let clicked = Arc::new(Mutex::new(Vec::new()));
        let closed = Arc::new(Mutex::new(Vec::new()));
        let props = ModalProps {
            buttons: vec![button.clone()],
            on_button_click: Some(Arc::new({
                let clicked = Arc::clone(&clicked);
                move |id| clicked.lock().unwrap().push(id)
            })),
            on_close: Some(Arc::new({
                let closed = Arc::clone(&closed);
                move |reason| closed.lock().unwrap().push(reason)
            })),
            ..Default::default()
        };

        // A close button reports its click and then closes the modal
        modal.handle_button_click(&props, &button);
        assert_eq!(*clicked.lock().unwrap(), vec!["test".to_string()]);
        assert_eq!(
            *closed.lock().unwrap(),
            vec![ModalCloseReason::ButtonClick("test".into())]
        );
    }

    #[test]
    fn test_builder_methods() {
        let props = Modal::visible(ModalProps::default(), true);
        assert!(props.visible);

        let props = Modal::with_title(props, "Test Title");
        assert_eq!(props.title, Some("Test Title".to_string()));

        let content = Element::text("Test content");
        let props = Modal::with_content(props, content);
        assert!(props.content.is_some());

        let props = Modal::with_size(props, ModalSize::Fixed(400), ModalSize::Fixed(300));
        assert_eq!(props.width, ModalSize::Fixed(400));
        assert_eq!(props.height, ModalSize::Fixed(300));

        let props = Modal::with_position(props, ModalPosition::TopLeft);
        assert_eq!(props.position, ModalPosition::TopLeft);

        let props = Modal::closable(props, false);
        assert!(!props.closable);

        let buttons = vec![ModalButton::ok(), ModalButton::cancel()];
        let props = Modal::with_buttons(props, buttons);
        assert_eq!(props.buttons.len(), 2);
    }

    #[test]
    fn test_button_constructors() {
        let ok_btn = ModalButton::ok();
        assert_eq!(ok_btn.id, "ok");
        assert_eq!(ok_btn.label, "OK");
        assert_eq!(ok_btn.action, ModalButtonAction::Confirm);

        let cancel_btn = ModalButton::cancel();
        assert_eq!(cancel_btn.id, "cancel");
        assert_eq!(cancel_btn.label, "Cancel");
        assert_eq!(cancel_btn.action, ModalButtonAction::Cancel);

        let close_btn = ModalButton::close();
        assert_eq!(close_btn.id, "close");
        assert_eq!(close_btn.label, "Close");
        assert_eq!(close_btn.action, ModalButtonAction::Close);

        let styled_btn = ModalButton::yes().with_style("btn-primary").disabled(false);
        assert_eq!(styled_btn.style, Some("btn-primary".to_string()));
        assert!(!styled_btn.disabled);
    }

    #[test]
    fn test_component_update() {
        let mut modal = Modal;
        let props = create_test_props();
        let mut state = ModalState::default();

        let should_update = modal.update(&props, &mut state);
        assert!(should_update); // Visibility state should cause update

        // Test with animation
        let animated_props = ModalProps {
            animation: ModalAnimation::Fade,
            ..props
        };
        let should_update = modal.update(&animated_props, &mut state);
        assert!(should_update); // Animation should cause updates
    }
}
