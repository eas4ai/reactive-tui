//! Progress Dialog Implementation
//!
//! Provides progress dialogs with cancellation support and customizable progress indicators.

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

/// Configuration options for progress dialogs
#[derive(Clone)]
pub struct ProgressDialogOptions {
    /// Title text displayed at the top of the dialog
    pub title: String,
    /// Main message or description of the operation
    pub message: String,
    /// Whether the user can cancel the operation
    pub cancellable: bool,
    /// Whether to show percentage completion
    pub show_percentage: bool,
    /// Whether to show estimated time remaining
    pub show_time_remaining: bool,
    /// Optional callback function when user cancels
    pub on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl PartialEq for ProgressDialogOptions {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.message == other.message
            && self.cancellable == other.cancellable
            && self.show_percentage == other.show_percentage
            && self.show_time_remaining == other.show_time_remaining
            && crate::widgets::display::overlay::same_callback(&self.on_cancel, &other.on_cancel)
    }
}

impl std::fmt::Debug for ProgressDialogOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressDialogOptions")
            .field("title", &self.title)
            .field("message", &self.message)
            .field("cancellable", &self.cancellable)
            .field("show_percentage", &self.show_percentage)
            .field("show_time_remaining", &self.show_time_remaining)
            .field("on_cancel", &"<function>")
            .finish()
    }
}

/// Progress dialog implementation
#[derive(Debug)]
pub struct ProgressDialog {
    state: BaseDialogState,
    options: ProgressDialogOptions,
    progress: f32, // 0.0 to 1.0
    bounds: DialogBounds,
}

impl ProgressDialog {
    pub(crate) fn element(
        options: ProgressDialogOptions,
        progress: f32,
        indeterminate: bool,
        class: Option<String>,
    ) -> Element {
        Element::typed::<live::LiveProgress>(live::LiveProps {
            id: DialogId::from_u32(0),
            options,
            progress,
            indeterminate,
            class,
            bounds: Rect::default(),
            theme: DialogTheme::default(),
        })
    }
    /// Create a new progress dialog
    pub fn new(id: DialogId, options: ProgressDialogOptions) -> Self {
        Self {
            state: BaseDialogState::new(id),
            options,
            progress: 0.0,
            bounds: DialogBounds::default(),
        }
    }

    /// Set the progress value (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }
}

impl DialogComponent for ProgressDialog {
    fn id(&self) -> DialogId {
        self.state.id
    }
    fn dialog_type(&self) -> &'static str {
        "progress"
    }
    fn render(&self, bounds: Rect, theme: &DialogTheme) -> Element {
        Element::typed::<live::LiveProgress>(live::LiveProps {
            id: self.state.id,
            options: self.options.clone(),
            progress: self.progress,
            indeterminate: false,
            class: None,
            bounds,
            theme: theme.clone(),
        })
    }
    fn handle_event(&mut self, _event: &Event) -> DialogEventResult {
        DialogEventResult::NotHandled
    }
    fn update(&mut self, _delta_time: Duration) -> bool {
        false
    }
    fn get_bounds(&self) -> DialogBounds {
        self.bounds.clone()
    }
    fn is_modal(&self) -> bool {
        true
    }
    fn backdrop_closable(&self) -> bool {
        false
    }
    fn escape_closable(&self) -> bool {
        self.options.cancellable
    }
    fn z_index(&self) -> u16 {
        1000
    }
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

impl Default for ProgressDialogOptions {
    fn default() -> Self {
        Self {
            title: "Progress".to_string(),
            message: "Please wait...".to_string(),
            cancellable: true,
            show_percentage: true,
            show_time_remaining: false,
            on_cancel: None,
        }
    }
}
