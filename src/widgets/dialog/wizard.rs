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
    pub id: String,
    pub title: String,
    pub content: Element,
    pub can_skip: bool,
    pub validator: Option<StepValidator>,
}

/// Configuration options for wizard dialogs
#[derive(Clone)]
pub struct WizardDialogOptions {
    pub title: String,
    pub steps: Vec<WizardStep>,
    pub show_progress: bool,
    pub allow_back: bool,
    pub on_complete: Option<OnCompleteCallback>,
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
    pub fn new(id: DialogId, options: WizardDialogOptions) -> Self {
        Self {
            state: BaseDialogState::new(id),
            options,
            current_step: 0,
            step_data: HashMap::new(),
            bounds: DialogBounds::default(),
        }
    }

    pub fn next_step(&mut self) -> bool {
        if self.current_step < self.options.steps.len() - 1 {
            self.current_step += 1;
            true
        } else {
            false
        }
    }

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
        Element::empty()
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
