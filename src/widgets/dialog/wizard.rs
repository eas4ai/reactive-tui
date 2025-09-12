//! Wizard Dialog Implementation
//!
//! Provides multi-step wizard dialogs with navigation and validation.

use super::{
    BaseDialogState, DialogBounds, DialogComponent, DialogEventResult, DialogId, DialogTheme,
    FocusableElementInfo, ValidationResult,
};
use crate::component::Element;
use crate::core::geometry::Rect;
use crate::event::types::Event;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Type aliases for complex function pointer types
type StepValidator = Arc<dyn Fn(&HashMap<String, String>) -> ValidationResult + Send + Sync>;
type OnCompleteCallback = Arc<dyn Fn(&HashMap<String, String>) -> bool + Send + Sync>;

/// Wizard step configuration
#[derive(Clone)]
pub struct WizardStep {
    /// Unique identifier for this step
    pub id: String,
    /// Display title for this step
    pub title: String,
    /// UI content for this step
    pub content: Element,
    /// Whether this step can be skipped
    pub can_skip: bool,
    /// Optional validation function for this step
    pub validator: Option<StepValidator>,
}

/// Configuration options for wizard dialogs
#[derive(Clone)]
pub struct WizardDialogOptions {
    /// Title displayed at the top of the wizard
    pub title: String,
    /// Sequence of steps in the wizard
    pub steps: Vec<WizardStep>,
    /// Whether to show progress indicator
    pub show_progress: bool,
    /// Whether to allow going back to previous steps
    pub allow_back: bool,
    /// Callback when wizard is completed
    pub on_complete: Option<OnCompleteCallback>,
    /// Callback when wizard is cancelled
    pub on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl std::fmt::Debug for WizardDialogOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WizardDialogOptions")
            .field("title", &self.title)
            .field("steps", &format!("{} steps", self.steps.len()))
            .field("show_progress", &self.show_progress)
            .field("allow_back", &self.allow_back)
            .field("on_complete", &"<function>")
            .field("on_cancel", &"<function>")
            .finish()
    }
}

/// Wizard dialog implementation
#[derive(Debug)]
pub struct WizardDialog {
    state: BaseDialogState,
    options: WizardDialogOptions,
    current_step: usize,
    #[allow(dead_code)]
    step_data: HashMap<String, String>,
    bounds: DialogBounds,
}

impl WizardDialog {
    /// Create a new wizard dialog
    pub fn new(id: DialogId, options: WizardDialogOptions) -> Self {
        Self {
            state: BaseDialogState::new(id),
            options,
            current_step: 0,
            step_data: HashMap::new(),
            bounds: DialogBounds::default(),
        }
    }

    /// Move to the next step in the wizard
    pub fn next_step(&mut self) -> bool {
        if self.current_step < self.options.steps.len() - 1 {
            self.current_step += 1;
            true
        } else {
            false
        }
    }

    /// Move to the previous step in the wizard
    pub fn previous_step(&mut self) -> bool {
        if self.current_step > 0 && self.options.allow_back {
            self.current_step -= 1;
            true
        } else {
            false
        }
    }
}

impl DialogComponent for WizardDialog {
    fn id(&self) -> DialogId {
        self.state.id
    }
    fn dialog_type(&self) -> &'static str {
        "wizard"
    }
    fn render(&self, _bounds: Rect, _theme: &DialogTheme) -> Element {
        use crate::builder::core::{div, button};
        use crate::builder::layout::{text, h2};

        if self.options.steps.is_empty() {
            return Element::empty();
        }

        let current_step = &self.options.steps[self.current_step];

        let mut children = Vec::new();

        // Add title
        children.push(
            h2().class("wizard-title text-xl font-bold text-gray-800 mb-4")
                .text(&self.options.title)
                .build()
        );

        // Add progress indicator if enabled
        if self.options.show_progress {
            let progress_text = format!("Step {} of {}", self.current_step + 1, self.options.steps.len());
            children.push(text(&progress_text));

            // Progress bar
            let progress_percent = ((self.current_step + 1) as f32 / self.options.steps.len() as f32 * 100.0) as u32;
            children.push(
                div().class("wizard-progress-bar bg-gray-200 rounded-full h-2 mb-4")
                    .child(
                        div().class("bg-blue-500 h-2 rounded-full")
                            .class(&format!("w-{}", progress_percent.min(100)))
                            .build()
                    )
                    .build()
            );
        }

        // Add current step title
        children.push(
            div().class("step-title text-lg font-semibold text-gray-700 mb-3")
                .text(&current_step.title)
                .build()
        );

        // Add current step content
        children.push(
            div().class("step-content flex-1 mb-4")
                .child(current_step.content.clone())
                .build()
        );

        // Add navigation buttons
        let mut nav_buttons = Vec::new();

        // Back button
        if self.options.allow_back && self.current_step > 0 {
            nav_buttons.push(
                button().class("wizard-back-btn bg-gray-500 text-white px-4 py-2 rounded hover:bg-gray-600")
                    .text("Back")
                    .build()
            );
        } else {
            nav_buttons.push(div().build()); // Spacer
        }

        // Next/Finish button
        let is_last_step = self.current_step >= self.options.steps.len() - 1;
        let next_button_text = if is_last_step { "Finish" } else { "Next" };
        let next_button_class = if is_last_step {
            "wizard-finish-btn bg-green-500 text-white px-4 py-2 rounded hover:bg-green-600"
        } else {
            "wizard-next-btn bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
        };

        nav_buttons.push(
            button().class(next_button_class)
                .text(next_button_text)
                .build()
        );

        children.push(
            div().class("wizard-buttons flex justify-between")
                .children(nav_buttons)
                .build()
        );

        // Create main dialog container
        div().class("wizard-dialog bg-white border border-gray-300 rounded-lg shadow-lg p-4")
            .class("flex flex-col min-w-96 min-h-72")
            .children(children)
            .build()
    }

    fn handle_event(&mut self, event: &Event) -> DialogEventResult {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
            _ => DialogEventResult::NotHandled,
        }
    }

    fn update(&mut self, _delta_time: Duration) -> bool {
        // Wizard dialogs are typically static, but could animate transitions
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
        true
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

impl Default for WizardDialogOptions {
    fn default() -> Self {
        Self {
            title: "Wizard".to_string(),
            steps: Vec::new(),
            show_progress: true,
            allow_back: true,
            on_complete: None,
            on_cancel: None,
        }
    }
}

impl WizardDialog {
    /// Handle keyboard events for wizard navigation
    fn handle_key_event(&mut self, key_event: &crate::event::types::KeyEvent) -> DialogEventResult {
        use crate::event::types::KeyCode;

        match key_event.code {
            KeyCode::Enter => {
                // Move to next step or finish
                if self.current_step >= self.options.steps.len() - 1 {
                    self.finish_wizard()
                } else {
                    if self.next_step() {
                        DialogEventResult::Handled
                    } else {
                        DialogEventResult::NotHandled
                    }
                }
            }
            KeyCode::Escape => {
                // Cancel wizard
                self.cancel_wizard()
            }
            KeyCode::Left | KeyCode::Backspace => {
                // Go back if allowed
                if self.options.allow_back && self.current_step > 0 {
                    if self.previous_step() {
                        DialogEventResult::Handled
                    } else {
                        DialogEventResult::NotHandled
                    }
                } else {
                    DialogEventResult::NotHandled
                }
            }
            _ => DialogEventResult::NotHandled,
        }
    }

    /// Handle mouse events for wizard interaction
    fn handle_mouse_event(&mut self, mouse_event: &crate::event::types::MouseEvent) -> DialogEventResult {
        use crate::event::types::MouseEventKind;

        match mouse_event.kind {
            MouseEventKind::Down => {
                // Handle button clicks based on position
                // This would need proper layout integration to determine which button was clicked
                DialogEventResult::NotHandled
            }
            _ => DialogEventResult::NotHandled,
        }
    }

    /// Finish the wizard and call completion callback
    fn finish_wizard(&mut self) -> DialogEventResult {
        use super::{DialogResult};

        if let Some(ref on_complete) = self.options.on_complete {
            if on_complete(&self.step_data) {
                DialogEventResult::Close(DialogResult::Confirmed(None))
            } else {
                DialogEventResult::NotHandled
            }
        } else {
            DialogEventResult::Close(DialogResult::Confirmed(None))
        }
    }

    /// Cancel the wizard and call cancellation callback
    fn cancel_wizard(&mut self) -> DialogEventResult {
        use super::{DialogResult};

        if let Some(ref on_cancel) = self.options.on_cancel {
            on_cancel();
        }
        DialogEventResult::Close(DialogResult::Cancelled)
    }
}
