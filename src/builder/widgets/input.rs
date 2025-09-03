//! Input widget builders
//!
//! This module provides builders for various input components including text inputs,
//! checkboxes, radio buttons, select dropdowns, and sliders.

use crate::component::Element;
use super::super::specialized::{RadioButtonBuilder, SliderBuilder};

/// Create a Text Input builder
///
/// Returns a `TextInputBuilder` for creating text input fields with various
/// input types, validation, and styling options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::text_input;
///
/// let input = text_input()
///     .placeholder("Enter email")
///     .input_type("email")
///     .build();
/// ```
pub fn text_input() -> TextInputBuilder {
    TextInputBuilder::new()
}

/// Create a Checkbox builder
///
/// Returns a `CheckboxBuilder` for creating checkbox inputs with labels
/// and state management.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::checkbox;
///
/// let checkbox = checkbox()
///     .label("I agree")
///     .checked(false)
///     .build();
/// ```
pub fn checkbox() -> CheckboxBuilder {
    CheckboxBuilder::new()
}

/// Create a Radio Button builder
///
/// Returns a `RadioButtonBuilder` for creating radio button inputs.
pub fn radio_button() -> RadioButtonBuilder {
    RadioButtonBuilder::new()
}

/// Create a Select dropdown builder
///
/// Returns a `SelectBuilder` for creating dropdown select inputs with
/// options and selection state.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::select;
///
/// let select = select()
///     .option("us", "United States")
///     .option("ca", "Canada")
///     .build();
/// ```
pub fn select() -> SelectBuilder {
    SelectBuilder::new()
}

/// Create a Slider builder
///
/// Returns a `SliderBuilder` for creating range slider controls.
pub fn slider() -> SliderBuilder {
    SliderBuilder::new()
}

/// Builder for TextInput components
///
/// Provides a fluent API for creating text input fields with various
/// input types, validation, and styling options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::text_input;
///
/// let input = text_input()
///     .placeholder("Enter your name")
///     .value("John Doe")
///     .input_type("email")
///     .max_length(100)
///     .class("email-field")
///     .build();
/// ```
pub struct TextInputBuilder {
    value: String,
    placeholder: Option<String>,
    disabled: bool,
    readonly: bool,
    max_length: Option<usize>,
    input_type: String,
    class: Option<String>,
}

impl TextInputBuilder {
    /// Create a new TextInputBuilder with default values
    fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: None,
            disabled: false,
            readonly: false,
            max_length: None,
            input_type: "text".to_string(),
            class: None,
        }
    }

    /// Set the input field value
    ///
    /// # Arguments
    /// * `value` - The current text value of the input field
    pub fn value(mut self, value: &str) -> Self {
        self.value = value.to_string();
        self
    }

    /// Set the placeholder text
    ///
    /// # Arguments
    /// * `placeholder` - Text to show when the input is empty
    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    /// Set the disabled state
    ///
    /// # Arguments
    /// * `disabled` - Whether the input should be disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the readonly state
    ///
    /// # Arguments
    /// * `readonly` - Whether the input should be read-only
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Set the maximum character length
    ///
    /// # Arguments
    /// * `max_length` - Maximum number of characters allowed
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set the input type
    ///
    /// # Arguments
    /// * `input_type` - Input type (text, password, email, number, etc.)
    pub fn input_type(mut self, input_type: &str) -> Self {
        self.input_type = input_type.to_string();
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the input
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the TextInput element
    ///
    /// Creates a text input element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the text input field
    pub fn build(self) -> Element {
        let display_text = if self.value.is_empty() {
            self.placeholder.unwrap_or_else(|| "Text Input".to_string())
        } else {
            self.value
        };

        let mut element = Element::text(format!("Input: {}", display_text));

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<TextInputBuilder> for Element {
    fn from(builder: TextInputBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Checkbox components
///
/// Provides a fluent API for creating checkbox inputs with labels,
/// states, and styling options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::checkbox;
///
/// let checkbox = checkbox()
///     .label("I agree to the terms")
///     .checked(false)
///     .class("terms-checkbox")
///     .build();
/// ```
pub struct CheckboxBuilder {
    checked: bool,
    label: Option<String>,
    disabled: bool,
    indeterminate: bool,
    class: Option<String>,
}

impl CheckboxBuilder {
    /// Create a new CheckboxBuilder with default values
    fn new() -> Self {
        Self {
            checked: false,
            label: None,
            disabled: false,
            indeterminate: false,
            class: None,
        }
    }

    /// Set the checked state
    ///
    /// # Arguments
    /// * `checked` - Whether the checkbox should be checked
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the checkbox label text
    ///
    /// # Arguments
    /// * `label` - Text label to display next to the checkbox
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set the disabled state
    ///
    /// # Arguments
    /// * `disabled` - Whether the checkbox should be disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the indeterminate state
    ///
    /// # Arguments
    /// * `indeterminate` - Whether the checkbox should show indeterminate state
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the checkbox
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Checkbox element
    ///
    /// Creates a checkbox element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the checkbox input
    pub fn build(self) -> Element {
        let state = if self.indeterminate {
            "indeterminate"
        } else if self.checked {
            "checked"
        } else {
            "unchecked"
        };

        let display_text = if let Some(label) = &self.label {
            format!("Checkbox ({}): {}", state, label)
        } else {
            format!("Checkbox ({})", state)
        };

        let mut element = Element::text(display_text);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<CheckboxBuilder> for Element {
    fn from(builder: CheckboxBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Select dropdown components
///
/// Provides a fluent API for creating select dropdowns with options,
/// selection state, and styling.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::select;
///
/// let select = select()
///     .option("us", "United States")
///     .option("ca", "Canada")
///     .selected("us")
///     .placeholder("Choose country")
///     .build();
/// ```
pub struct SelectBuilder {
    options: Vec<(String, String)>, // (value, label)
    selected_value: Option<String>,
    placeholder: Option<String>,
    disabled: bool,
    multiple: bool,
    class: Option<String>,
}

impl SelectBuilder {
    /// Create a new SelectBuilder with default values
    fn new() -> Self {
        Self {
            options: Vec::new(),
            selected_value: None,
            placeholder: None,
            disabled: false,
            multiple: false,
            class: None,
        }
    }

    /// Add a single option to the select dropdown
    ///
    /// # Arguments
    /// * `value` - The option value (used for form submission)
    /// * `label` - The display text for the option
    pub fn option(mut self, value: &str, label: &str) -> Self {
        self.options.push((value.to_string(), label.to_string()));
        self
    }

    /// Add multiple options at once
    ///
    /// # Arguments
    /// * `options` - Vector of (value, label) tuples
    pub fn options(mut self, options: Vec<(&str, &str)>) -> Self {
        for (value, label) in options {
            self.options.push((value.to_string(), label.to_string()));
        }
        self
    }

    /// Set the currently selected value
    ///
    /// # Arguments
    /// * `value` - The value of the option to select
    pub fn selected(mut self, value: &str) -> Self {
        self.selected_value = Some(value.to_string());
        self
    }

    /// Set the placeholder text
    ///
    /// # Arguments
    /// * `placeholder` - Text to show when no option is selected
    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    /// Set the disabled state
    ///
    /// # Arguments
    /// * `disabled` - Whether the select should be disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Enable multiple selection mode
    ///
    /// # Arguments
    /// * `multiple` - Whether multiple options can be selected
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the select
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Select element
    ///
    /// Creates a select dropdown element with the configured options and properties.
    ///
    /// # Returns
    /// An `Element` representing the select dropdown
    pub fn build(self) -> Element {
        let selected_label = if let Some(selected) = &self.selected_value {
            self.options
                .iter()
                .find(|(value, _)| value == selected)
                .map(|(_, label)| label.clone())
                .unwrap_or_else(|| selected.clone())
        } else {
            self.placeholder.unwrap_or_else(|| "Select an option".to_string())
        };

        let display_text = format!("Select: {} ({} options)", selected_label, self.options.len());

        let mut element = Element::text(display_text);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<SelectBuilder> for Element {
    fn from(builder: SelectBuilder) -> Self {
        builder.build()
    }
}

// Note: Placeholder builders (RadioButtonBuilder, SliderBuilder)
// are now provided by the placeholders module
