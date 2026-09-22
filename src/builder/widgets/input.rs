//! Input widget builders
//!
//! This module provides builders for various input components including text inputs,
//! checkboxes, radio buttons, select dropdowns, and sliders.

use super::super::specialized::{RadioButtonBuilder, SliderBuilder};
use crate::component::Element;

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
#[derive(Clone, PartialEq)]
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
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredTextInput>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }

        element
    }
}

impl crate::component::Props for TextInputBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TextInputBuilder {
    fn widget_props(&self) -> crate::widgets::TextInputProps {
        use crate::widgets::input::InputMode;
        crate::widgets::TextInputProps {
            value: self.value.clone(),
            placeholder: self.placeholder.clone(),
            disabled: self.disabled,
            max_length: self.max_length,
            mode: match self.input_type.as_str() {
                "password" => InputMode::Password,
                "number" => InputMode::Numeric,
                _ => InputMode::SingleLine,
            },
            validator_pattern: (self.input_type == "email").then(|| "email".to_owned()),
            ..Default::default()
        }
    }
}

struct ConfiguredTextInput(crate::widgets::TextInput);

impl crate::component::Component for ConfiguredTextInput {
    type Props = TextInputBuilder;
    type State = crate::widgets::TextInputState;

    fn new(props: Self::Props) -> Self {
        Self(crate::widgets::TextInput::new(props.widget_props()))
    }
    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        self.0.initial_state(&props.widget_props())
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        self.0.update(&props.widget_props(), state)
    }
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.0.render(&props.widget_props(), state)
    }
    fn layout(
        &mut self,
        bounds: crate::component::LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        self.0.layout(bounds, &mut props.widget_props(), state)
    }
    fn handle_event(
        &mut self,
        event: &crate::event::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> crate::event::router::EventResult {
        use crate::event::{
            router::EventResult,
            types::{Event, KeyCode},
        };
        if props.readonly {
            let allowed = match event {
                Event::Key(key) if key.modifiers.ctrl => matches!(
                    key.code,
                    KeyCode::Char('a' | 'c')
                        | KeyCode::Left
                        | KeyCode::Right
                        | KeyCode::Home
                        | KeyCode::End
                ),
                Event::Key(key) => matches!(
                    key.code,
                    KeyCode::Left
                        | KeyCode::Right
                        | KeyCode::Up
                        | KeyCode::Down
                        | KeyCode::Home
                        | KeyCode::End
                        | KeyCode::PageUp
                        | KeyCode::PageDown
                        | KeyCode::Escape
                        | KeyCode::Tab
                        | KeyCode::BackTab
                ),
                Event::Paste(_) => false,
                _ => true,
            };
            if !allowed {
                return EventResult::Consumed;
            }
        }
        let mut inner = props.widget_props();
        let result = self.0.handle_event(event, &mut inner, state);
        props.value = inner.value;
        result
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
        let mut element =
            Element::typed::<crate::widgets::Checkbox>(crate::widgets::CheckboxProps {
                checked: self.checked,
                label: self.label,
                disabled: self.disabled,
                indeterminate: self.indeterminate,
            });

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
#[derive(Clone, PartialEq)]
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
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredSelect>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }

        element
    }
}

impl crate::component::Props for SelectBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SelectBuilder {
    fn widget_props(&self) -> crate::widgets::SelectProps<String> {
        crate::widgets::SelectProps {
            options: self
                .options
                .iter()
                .map(|(value, label)| {
                    crate::widgets::SelectOption::new(value.clone(), label.clone())
                })
                .collect(),
            selected: self.selected_value.clone(),
            placeholder: self.placeholder.clone(),
            disabled: self.disabled,
            ..Default::default()
        }
    }
}

struct ConfiguredSelect {
    inner: crate::widgets::Select<String>,
    selected: Vec<String>,
    selection_prop: Option<String>,
    multiple_prop: bool,
}

impl crate::component::Component for ConfiguredSelect {
    type Props = SelectBuilder;
    type State = crate::widgets::SelectState;
    fn new(props: Self::Props) -> Self {
        Self {
            inner: crate::widgets::Select::new(props.widget_props()),
            selected: props.selected_value.clone().into_iter().collect(),
            selection_prop: props.selected_value,
            multiple_prop: props.multiple,
        }
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if self.selection_prop != props.selected_value || self.multiple_prop != props.multiple {
            self.selected = props.selected_value.clone().into_iter().collect();
        }
        self.selection_prop.clone_from(&props.selected_value);
        self.multiple_prop = props.multiple;
        self.selected
            .retain(|value| props.options.iter().any(|option| &option.0 == value));
        self.inner.update(&props.widget_props(), state)
    }
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.inner.render_control(
            &props.widget_props(),
            state,
            props.multiple.then_some(self.selected.as_slice()),
        )
    }
    fn layout(
        &mut self,
        bounds: crate::component::LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        self.inner.layout(bounds, &mut props.widget_props(), state)
    }
    fn handle_event(
        &mut self,
        event: &crate::event::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> crate::event::router::EventResult {
        use crate::event::types::{Event, KeyCode, MouseButton, MouseEventKind};
        let choosing = state.is_open
            && match event {
                Event::Key(key) => matches!(
                    key.code,
                    KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Space
                ),
                Event::Mouse(mouse) => {
                    self.inner
                        .option_at(mouse.position, &props.widget_props(), state)
                        .is_some()
                        && mouse.button == MouseButton::Left
                        && matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click)
                }
                _ => false,
            };
        let mut inner = props.widget_props();
        let result = self.inner.handle_event(event, &mut inner, state);
        if props.multiple && choosing && !state.is_open {
            if let Some(value) = &inner.selected {
                if self.selected.contains(value) {
                    self.selected.retain(|selected| selected != value);
                } else {
                    self.selected.push(value.clone());
                }
                state.is_open = true;
            }
        }
        props.selected_value = inner.selected;
        result
    }
}

impl From<SelectBuilder> for Element {
    fn from(builder: SelectBuilder) -> Self {
        builder.build()
    }
}

// Note: Placeholder builders (RadioButtonBuilder, SliderBuilder)
// are now provided by the placeholders module
