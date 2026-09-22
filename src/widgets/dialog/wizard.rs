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

#[cfg(test)]
mod tests;

mod live;

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

impl PartialEq for WizardStep {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.title == other.title
            && self.content == other.content
            && self.can_skip == other.can_skip
            && crate::widgets::display::overlay::same_callback(&self.validator, &other.validator)
    }
}

impl PartialEq for WizardDialogOptions {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.steps == other.steps
            && self.show_progress == other.show_progress
            && self.allow_back == other.allow_back
            && crate::widgets::display::overlay::same_callback(
                &self.on_complete,
                &other.on_complete,
            )
            && crate::widgets::display::overlay::same_callback(&self.on_cancel, &other.on_cancel)
    }
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
    step_data: HashMap<String, String>,
    bounds: DialogBounds,
}

impl WizardDialog {
    pub(crate) fn element(
        options: WizardDialogOptions,
        initial_step: usize,
        cancelable: bool,
        class: Option<String>,
    ) -> Element {
        Element::typed::<live::LiveWizard>(live::LiveProps {
            id: DialogId::from_u32(0),
            options,
            initial_step,
            cancelable,
            class,
            data: HashMap::new(),
            bounds: Rect::default(),
            theme: DialogTheme::default(),
        })
    }

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
        if self.validate().valid && self.current_step + 1 < self.options.steps.len() {
            self.current_step += 1;
            true
        } else {
            false
        }
    }

    /// Replace the named values supplied to step validators and completion callbacks.
    /// Child controls should update these values through the caller's normal state.
    pub fn set_data(&mut self, data: HashMap<String, String>) {
        self.step_data = data;
    }

    /// Current named values supplied to validators and completion callbacks.
    pub fn data(&self) -> &HashMap<String, String> {
        &self.step_data
    }

    /// Skip an optional step without running its validator.
    pub fn skip_step(&mut self) -> bool {
        if configuration_error(&self.options).is_none()
            && self
                .options
                .steps
                .get(self.current_step)
                .is_some_and(|step| step.can_skip)
            && self.current_step + 1 < self.options.steps.len()
        {
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
    fn render(&self, bounds: Rect, theme: &DialogTheme) -> Element {
        Element::typed::<live::LiveWizard>(live::LiveProps {
            id: self.state.id,
            options: self.options.clone(),
            data: self.step_data.clone(),
            initial_step: self.current_step,
            cancelable: true,
            class: None,
            bounds,
            theme: theme.clone(),
        })
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
        let result = validate_step(&self.options, self.current_step, &self.step_data);
        super::dialog_component::ValidationResult {
            valid: result.valid,
            errors: result
                .message
                .map(|message| HashMap::from([("wizard".into(), message)]))
                .unwrap_or_default(),
            warnings: result
                .warnings
                .into_iter()
                .enumerate()
                .map(|(index, message)| (index.to_string(), message))
                .collect(),
            data: None,
        }
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
                if self.current_step + 1 >= self.options.steps.len() {
                    self.finish_wizard()
                } else if self.next_step() {
                    DialogEventResult::Handled
                } else {
                    DialogEventResult::NotHandled
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
    fn handle_mouse_event(
        &mut self,
        mouse_event: &crate::event::types::MouseEvent,
    ) -> DialogEventResult {
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
        use super::DialogResult;

        if self.current_step + 1 != self.options.steps.len() || !self.validate().valid {
            return DialogEventResult::NotHandled;
        }

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
        use super::DialogResult;

        if let Some(ref on_cancel) = self.options.on_cancel {
            on_cancel();
        }
        DialogEventResult::Close(DialogResult::Cancelled)
    }
}

fn configuration_error(options: &WizardDialogOptions) -> Option<String> {
    if options.steps.is_empty() {
        return Some("Wizard has no steps".into());
    }
    let mut ids = std::collections::HashSet::new();
    for step in &options.steps {
        if step.id.trim().is_empty() || !ids.insert(&step.id) {
            return Some("Wizard step IDs must be nonempty and unique".into());
        }
    }
    None
}

fn validate_step(
    options: &WizardDialogOptions,
    index: usize,
    data: &HashMap<String, String>,
) -> ValidationResult {
    if let Some(error) = configuration_error(options) {
        return ValidationResult {
            valid: false,
            message: Some(error),
            ..Default::default()
        };
    }
    let Some(step) = options.steps.get(index) else {
        return ValidationResult {
            valid: false,
            message: Some("Wizard step is out of range".into()),
            ..Default::default()
        };
    };
    step.validator.as_ref().map_or_else(
        || ValidationResult {
            valid: true,
            ..Default::default()
        },
        |validator| validator(data),
    )
}
