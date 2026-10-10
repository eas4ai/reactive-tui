//! The builder for the stepper display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-002, DIS-003, DIS-004, DIS-005, DIS-006.

use std::sync::Arc;

use crate::component::Element;
use crate::widgets::display::pieces::stepper::{Stepper, StepperProps};

/// Create a stepper. Add its steps with `step`.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::stepper;
///
/// let steps = stepper()
///     .step("Compose")
///     .step("Capture")
///     .step("Review")
///     .current(2)
///     .on_change(|step| println!("step {step}"))
///     .build();
/// ```
pub fn stepper() -> StepperBuilder {
    StepperBuilder {
        props: StepperProps::default(),
    }
}

/// Builder for a stepper. A row fills the width its parent allots and a column
/// the height; a `w-N`, `h-N`, `w-full` or `h-full` class sets the size instead.
#[derive(Clone)]
pub struct StepperBuilder {
    props: StepperProps,
}

impl StepperBuilder {
    /// Add a step with its label. Steps appear in the order they were added.
    pub fn step(mut self, label: &str) -> Self {
        self.props.steps.push(label.to_string());
        self
    }

    /// Set the current step, 1-based.
    pub fn current(mut self, current: usize) -> Self {
        self.props.current = current;
        self
    }

    /// Stack the steps top to bottom, joined by `│`, instead of in a row.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.props.vertical = vertical;
        self
    }

    /// Run `on_change` with the step, 1-based, when a click or Confirm chooses it.
    /// Set, the arrows move the focused step as well.
    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        self.props.on_change = Some(Arc::new(on_change));
        self
    }

    /// Add classes to the stepper's element, after its own.
    pub fn class(mut self, class: &str) -> Self {
        if !self.props.class.is_empty() {
            self.props.class.push(' ');
        }
        self.props.class.push_str(class);
        self
    }

    /// Build the stepper's element.
    pub fn build(self) -> Element {
        Element::typed::<Stepper>(self.props)
    }
}
