//! The form builder (CTL-007 to CTL-010): `builder::form()` takes its fields
//! and its settings and builds the `Form` element.

use crate::component::Element;
use crate::widgets::input::{Form, FormField, FormLabels, FormProps};

/// Builder for forms with a fluent API.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::{form, text_input};
/// use reactive_tui::widgets::input::FormField;
///
/// let element = form()
///     .field(FormField::new("Name", text_input().build()).required())
///     .field(FormField::new("Email", text_input().build()).description("Work address"))
///     .build();
/// ```
#[derive(Clone)]
pub struct FormBuilder {
    fields: Vec<FormField>,
    labels: FormLabels,
    label_width: Option<u16>,
    aria_label: Option<String>,
    class: Option<String>,
}

impl FormBuilder {
    pub(super) fn new() -> Self {
        Self {
            fields: Vec::new(),
            labels: FormLabels::Beside,
            label_width: None,
            aria_label: None,
            class: None,
        }
    }

    /// Add a field after the fields already added.
    pub fn field(mut self, field: FormField) -> Self {
        self.fields.push(field);
        self
    }

    /// Put every label above its control, whatever the width.
    pub fn labels_above(mut self) -> Self {
        self.labels = FormLabels::Above;
        self
    }

    /// Set the width of the label column, in cells.
    pub fn label_width(mut self, width: u16) -> Self {
        self.label_width = Some(width);
        self
    }

    /// Set the name the screen reader hears for the form.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.aria_label = Some(label.to_string());
        self
    }

    /// Set CSS classes for styling.
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Form element.
    pub fn build(self) -> Element {
        let class = self.class;
        let mut element = Element::typed::<Form>(FormProps {
            fields: self.fields,
            labels: self.labels,
            label_width: self.label_width,
            aria_label: self.aria_label,
        });
        if let Some(class) = class {
            element = element.with_class(&class);
        }
        element
    }
}

impl From<FormBuilder> for Element {
    fn from(builder: FormBuilder) -> Self {
        builder.build()
    }
}
