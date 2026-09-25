//! Additional dialog builders for specialized dialog types
//!
//! This module contains builders for specialized dialog components like
//! progress dialogs and wizard dialogs.

use crate::component::Element;

/// Builder for Progress Dialog components
///
/// Provides a fluent API for creating progress dialog widgets that show
/// the progress of long-running operations.
pub struct ProgressDialogBuilder {
    title: Option<String>,
    message: String,
    progress: f32,
    indeterminate: bool,
    cancelable: bool,
    show_percentage: bool,
    class: Option<String>,
}

impl Default for ProgressDialogBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressDialogBuilder {
    /// Create a new ProgressDialogBuilder with default values
    pub fn new() -> Self {
        Self {
            title: None,
            message: "Processing...".to_string(),
            progress: 0.0,
            indeterminate: false,
            cancelable: false,
            show_percentage: true,
            class: None,
        }
    }

    /// Set the dialog title
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set the progress message
    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// Set the progress value (0.0 to 1.0)
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Set whether the progress is indeterminate
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Set whether the operation can be canceled
    pub fn cancelable(mut self, cancelable: bool) -> Self {
        self.cancelable = cancelable;
        self
    }

    /// Set whether to show percentage text
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the ProgressDialog element
    pub fn build(self) -> Element {
        use crate::widgets::dialog::{ProgressDialog, ProgressDialogOptions};
        ProgressDialog::element(
            ProgressDialogOptions {
                title: self.title.unwrap_or_else(|| "Progress".to_string()),
                message: self.message,
                cancellable: self.cancelable,
                show_percentage: self.show_percentage,
                ..Default::default()
            },
            self.progress,
            self.indeterminate,
            self.class,
        )
    }
}

impl From<ProgressDialogBuilder> for Element {
    fn from(builder: ProgressDialogBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Wizard Dialog components
///
/// Provides a fluent API for creating multi-step wizard dialog widgets.
pub struct WizardBuilder {
    title: Option<String>,
    steps: Vec<WizardStep>,
    current_step: usize,
    show_progress: bool,
    cancelable: bool,
    class: Option<String>,
}

/// Represents a step in a wizard dialog
pub struct WizardStep {
    /// The title of this wizard step
    pub title: String,
    /// The content elements for this step
    pub content: Vec<Element>,
    /// Whether the user can proceed from this step
    pub can_proceed: bool,
}

impl WizardStep {
    /// Create a new wizard step
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            content: Vec::new(),
            can_proceed: true,
        }
    }

    /// Add content to the step
    pub fn content(mut self, element: Element) -> Self {
        self.content.push(element);
        self
    }

    /// Set whether the user can proceed from this step
    pub fn can_proceed(mut self, can_proceed: bool) -> Self {
        self.can_proceed = can_proceed;
        self
    }
}

impl Default for WizardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WizardBuilder {
    /// Create a new WizardBuilder with default values
    pub fn new() -> Self {
        Self {
            title: None,
            steps: Vec::new(),
            current_step: 0,
            show_progress: true,
            cancelable: true,
            class: None,
        }
    }

    /// Set the wizard title
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Add a step to the wizard
    pub fn step(mut self, step: WizardStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add multiple steps to the wizard
    pub fn steps(mut self, steps: Vec<WizardStep>) -> Self {
        self.steps.extend(steps);
        self
    }

    /// Set the current step index
    pub fn current_step(mut self, step: usize) -> Self {
        self.current_step = step;
        self
    }

    /// Set whether to show progress indicator
    pub fn show_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    /// Set whether the wizard can be canceled
    pub fn cancelable(mut self, cancelable: bool) -> Self {
        self.cancelable = cancelable;
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Wizard element
    pub fn build(self) -> Element {
        use crate::widgets::dialog::{
            ValidationResult, WizardDialog, WizardDialogOptions, WizardStep as DialogStep,
        };
        WizardDialog::element(
            WizardDialogOptions {
                title: self.title.unwrap_or_else(|| "Wizard".into()),
                steps: self
                    .steps
                    .into_iter()
                    .enumerate()
                    .map(|(index, step)| {
                        let can_proceed = step.can_proceed;
                        DialogStep {
                            id: format!("step-{index}"),
                            title: step.title,
                            content: crate::builder::div()
                                .class("flex-col")
                                .children(step.content)
                                .build(),
                            can_skip: false,
                            validator: Some(std::sync::Arc::new(move |_| ValidationResult {
                                valid: can_proceed,
                                ..Default::default()
                            })),
                        }
                    })
                    .collect(),
                show_progress: self.show_progress,
                ..Default::default()
            },
            self.current_step,
            self.cancelable,
            self.class,
        )
    }
}

impl From<WizardBuilder> for Element {
    fn from(builder: WizardBuilder) -> Self {
        builder.build()
    }
}
