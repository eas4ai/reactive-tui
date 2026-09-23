//! Toast Notification Implementation
//!
//! Provides non-modal toast notifications with auto-dismiss and positioning options.

use super::{
    BaseDialogState, DialogBounds, DialogComponent, DialogEventResult, DialogId, DialogTheme,
    FocusableElementInfo,
};
use crate::component::Element;
use crate::core::geometry::Rect;
use crate::event::types::Event;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

mod live;

/// Toast notification types
#[derive(Debug, Clone, PartialEq)]
pub enum ToastType {
    /// Information toast
    Info,
    /// Success toast
    Success,
    /// Warning toast
    Warning,
    /// Error toast
    Error,
    /// Custom toast type
    Custom(String),
}

/// Configuration options for toast notifications
#[derive(Clone)]
pub struct ToastOptions {
    /// Text message to display in the toast
    pub message: String,
    /// Type of toast (info, warning, error, success)
    pub toast_type: ToastType,
    /// How long to show the toast (None for persistent). In App, this starts
    /// after the fully opened toast has been presented.
    pub duration: Option<Duration>,
    /// Where to position the toast on screen
    pub position: ToastPosition,
    /// Whether user can manually close the toast
    pub closable: bool,
    /// Optional callback when toast is closed
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl PartialEq for ToastOptions {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
            && self.toast_type == other.toast_type
            && self.duration == other.duration
            && self.position == other.position
            && self.closable == other.closable
            && crate::widgets::display::overlay::same_callback(&self.on_close, &other.on_close)
    }
}

impl std::fmt::Debug for ToastOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToastOptions")
            .field("message", &self.message)
            .field("toast_type", &self.toast_type)
            .field("duration", &self.duration)
            .field("position", &self.position)
            .field("closable", &self.closable)
            .field("on_close", &"<function>")
            .finish()
    }
}

/// Toast positioning options
#[derive(Debug, Clone, PartialEq)]
pub enum ToastPosition {
    /// Position toast at top-left corner
    TopLeft,
    /// Position toast at top-center
    TopCenter,
    /// Position toast at top-right corner
    TopRight,
    /// Position toast at bottom-left corner
    BottomLeft,
    /// Position toast at bottom-center
    BottomCenter,
    /// Position toast at bottom-right corner
    BottomRight,
}

/// Toast notification implementation
#[derive(Debug)]
pub struct Toast {
    state: BaseDialogState,
    options: ToastOptions,
    bounds: DialogBounds,
    auto_dismiss_time: Option<std::time::Instant>,
}

impl Toast {
    pub(crate) fn element(options: ToastOptions, class: Option<String>) -> Element {
        Element::typed::<live::LiveToast>(live::LiveProps {
            options,
            class,
            bounds: Rect::default(),
        })
    }
    /// Create a new toast notification
    pub fn new(id: DialogId, options: ToastOptions) -> Self {
        let auto_dismiss_time = options.duration.map(|_| std::time::Instant::now());

        Self {
            state: BaseDialogState::new(id),
            options,
            bounds: DialogBounds::default(),
            auto_dismiss_time,
        }
    }
}

impl DialogComponent for Toast {
    fn id(&self) -> DialogId {
        self.state.id
    }
    fn dialog_type(&self) -> &'static str {
        "toast"
    }
    fn render(&self, bounds: Rect, _theme: &DialogTheme) -> Element {
        Element::typed::<live::LiveToast>(live::LiveProps {
            options: self.options.clone(),
            class: None,
            bounds,
        })
    }
    fn handle_event(&mut self, _event: &Event) -> DialogEventResult {
        DialogEventResult::NotHandled
    }
    fn update(&mut self, _delta_time: Duration) -> bool {
        // Check for auto-dismiss
        if let (Some(dismiss_time), Some(duration)) =
            (self.auto_dismiss_time, self.options.duration)
        {
            if dismiss_time.elapsed() >= duration {
                return true; // Signal for removal
            }
        }
        false
    }
    fn get_bounds(&self) -> DialogBounds {
        self.bounds.clone()
    }
    fn is_modal(&self) -> bool {
        false
    }
    fn backdrop_closable(&self) -> bool {
        false
    }
    fn escape_closable(&self) -> bool {
        self.options.closable
    }
    fn z_index(&self) -> u16 {
        2000
    } // Higher than regular dialogs
    fn animation(&self) -> Option<super::DialogAnimationConfig> {
        None
    }
    fn get_focusable_elements(&self) -> Vec<FocusableElementInfo> {
        Vec::new()
    }
    fn set_focus(&mut self, _element_id: &str) -> bool {
        false
    }
    fn get_focused_element(&self) -> Option<String> {
        None
    }
    fn validate(&self) -> super::dialog_component::ValidationResult {
        super::dialog_component::ValidationResult::default()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            message: "Notification".to_string(),
            toast_type: ToastType::Info,
            duration: Some(Duration::from_secs(3)),
            position: ToastPosition::TopRight,
            closable: true,
            on_close: None,
        }
    }
}
