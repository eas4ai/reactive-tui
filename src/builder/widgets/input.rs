//! Input widget builders
//!
//! This module provides builders for various input components including text inputs,
//! checkboxes, radio buttons, select dropdowns, and sliders.

use super::super::specialized::{RadioButtonBuilder, SliderBuilder};
use crate::component::{same_callback, Element};
use std::sync::Arc;

/// A builder's change callback, called with the value the control reports.
type Callback<T> = Arc<dyn Fn(T) + Send + Sync>;

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
#[derive(Clone)]
pub struct TextInputBuilder {
    value: String,
    placeholder: Option<String>,
    aria_label: Option<String>,
    disabled: bool,
    readonly: bool,
    max_length: Option<usize>,
    input_type: String,
    class: Option<String>,
    on_change: Option<Callback<String>>,
    on_submit: Option<Callback<String>>,
}

/// Equal over the settings and not the callbacks, so a rebuild that changes
/// only a callback keeps the mounted input and hands it the new one
/// (CMP-008).
impl PartialEq for TextInputBuilder {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            value,
            placeholder,
            aria_label,
            disabled,
            readonly,
            max_length,
            input_type,
            class,
            on_change: _,
            on_submit: _,
        } = self;
        *value == other.value
            && *placeholder == other.placeholder
            && *aria_label == other.aria_label
            && *disabled == other.disabled
            && *readonly == other.readonly
            && *max_length == other.max_length
            && *input_type == other.input_type
            && *class == other.class
    }
}

impl TextInputBuilder {
    /// Create a new TextInputBuilder with default values
    fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: None,
            aria_label: None,
            disabled: false,
            readonly: false,
            max_length: None,
            input_type: "text".to_string(),
            class: None,
            on_change: None,
            on_submit: None,
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

    /// Set the name the screen reader hears for the field; without it
    /// the placeholder names the field.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.aria_label = Some(label.to_string());
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

    /// Set the callback called with the text after each change the user
    /// makes to it, as `TextInput::with_on_change` is (CMP-009).
    pub fn on_change(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the text when Enter submits it, as
    /// `TextInput::with_on_submit` is (CMP-009).
    pub fn on_submit(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(f));
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
            aria_label: self.aria_label.clone(),
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

impl ConfiguredTextInput {
    /// Hand the input the callbacks `props` carry (CMP-009).
    fn take_callbacks(&mut self, props: &TextInputBuilder) {
        self.0.set_on_change(props.on_change.clone());
        self.0.set_on_submit(props.on_submit.clone());
    }
}

impl crate::component::Component for ConfiguredTextInput {
    type Props = TextInputBuilder;
    type State = crate::widgets::TextInputState;

    fn new(props: Self::Props) -> Self {
        let mut input = Self(crate::widgets::TextInput::new(props.widget_props()));
        input.take_callbacks(&props);
        input
    }
    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if same_callback(&props.on_change, &supplied.on_change)
            && same_callback(&props.on_submit, &supplied.on_submit)
        {
            return false;
        }
        props.on_change = supplied.on_change.clone();
        props.on_submit = supplied.on_submit.clone();
        true
    }
    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        self.0.initial_state(&props.widget_props())
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        self.0.update(&props.widget_props(), state)
    }
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let mut element = self.0.render(&props.widget_props(), state);
        // A read-only field takes no SetValue, so it advertises none
        // (CTL-005).
        if props.readonly {
            if let Some(node) = &mut element.metadata.accessibility {
                node.inner.remove_action(accesskit::Action::SetValue);
            }
            if let Some(options) = &mut element.metadata.accessibility_options {
                options.set_value_event = None;
            }
        }
        element
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
        // The callbacks a rebuild adopted act from this event on (CMP-008).
        self.take_callbacks(props);
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
                Event::Custom(event) => {
                    event.name != crate::widgets::input::TEXT_INPUT_SET_VALUE_EVENT
                }
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
#[derive(Clone)]
pub struct CheckboxBuilder {
    checked: bool,
    label: Option<String>,
    aria_label: Option<String>,
    disabled: bool,
    indeterminate: bool,
    class: Option<String>,
    on_change: Option<Callback<bool>>,
}

/// Equal over the settings and not the callback, so a rebuild that changes
/// only the callback keeps the mounted checkbox and hands it the new one
/// (CMP-008).
impl PartialEq for CheckboxBuilder {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            checked,
            label,
            aria_label,
            disabled,
            indeterminate,
            class,
            on_change: _,
        } = self;
        *checked == other.checked
            && *label == other.label
            && *aria_label == other.aria_label
            && *disabled == other.disabled
            && *indeterminate == other.indeterminate
            && *class == other.class
    }
}

impl CheckboxBuilder {
    /// Create a new CheckboxBuilder with default values
    fn new() -> Self {
        Self {
            checked: false,
            label: None,
            aria_label: None,
            disabled: false,
            indeterminate: false,
            class: None,
            on_change: None,
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

    /// Set the name the screen reader hears, when the visible label is
    /// not it ("Agree" shown, "Agree to the terms" read).
    pub fn aria_label(mut self, label: &str) -> Self {
        self.aria_label = Some(label.to_string());
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

    /// Set the callback called with the new checked state each time the
    /// user toggles the box, as `Checkbox::with_on_change` is (CMP-009).
    pub fn on_change(mut self, f: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Build the Checkbox element
    ///
    /// Creates a checkbox element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the checkbox input
    pub fn build(self) -> Element {
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredCheckbox>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }

        element
    }

    fn widget_props(&self) -> crate::widgets::CheckboxProps {
        crate::widgets::CheckboxProps {
            checked: self.checked,
            label: self.label.clone(),
            aria_label: self.aria_label.clone(),
            disabled: self.disabled,
            indeterminate: self.indeterminate,
        }
    }
}

impl crate::component::Props for CheckboxBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A checkbox built through its builder: the checkbox itself, with the
/// builder's callback handed to it (CMP-009).
struct ConfiguredCheckbox(crate::widgets::Checkbox);

impl crate::component::Component for ConfiguredCheckbox {
    type Props = CheckboxBuilder;
    type State = crate::widgets::CheckboxState;

    fn new(props: Self::Props) -> Self {
        let mut inner = crate::widgets::Checkbox::new(props.widget_props());
        inner.set_on_change(props.on_change.clone());
        Self(inner)
    }
    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        self.0.initial_state(&props.widget_props())
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        self.0.update(&props.widget_props(), state)
    }
    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if same_callback(&props.on_change, &supplied.on_change) {
            return false;
        }
        props.on_change = supplied.on_change.clone();
        true
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
        // The callback a rebuild adopted acts from this event on (CMP-008).
        self.0.set_on_change(props.on_change.clone());
        let mut inner = props.widget_props();
        let result = self.0.handle_event(event, &mut inner, state);
        props.checked = inner.checked;
        props.indeterminate = inner.indeterminate;
        result
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
#[derive(Clone)]
pub struct SelectBuilder {
    options: Vec<(String, String)>, // (value, label)
    selected_value: Option<String>,
    placeholder: Option<String>,
    disabled: bool,
    multiple: bool,
    class: Option<String>,
    on_change: Option<Callback<String>>,
}

/// Equal over the settings and not the callback, so a rebuild that changes
/// only the callback keeps the mounted select and hands it the new one
/// (CMP-008).
impl PartialEq for SelectBuilder {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            options,
            selected_value,
            placeholder,
            disabled,
            multiple,
            class,
            on_change: _,
        } = self;
        *options == other.options
            && *selected_value == other.selected_value
            && *placeholder == other.placeholder
            && *disabled == other.disabled
            && *multiple == other.multiple
            && *class == other.class
    }
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
            on_change: None,
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

    /// Set the callback called with the chosen option's value each time
    /// the user chooses another option, as `Select::with_on_change` is
    /// (CMP-009).
    pub fn on_change(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
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
        let mut inner = crate::widgets::Select::new(props.widget_props());
        inner.set_on_change(props.on_change.clone());
        Self {
            inner,
            selected: props.selected_value.clone().into_iter().collect(),
            selection_prop: props.selected_value,
            multiple_prop: props.multiple,
        }
    }
    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if same_callback(&props.on_change, &supplied.on_change) {
            return false;
        }
        props.on_change = supplied.on_change.clone();
        true
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
        // The callback a rebuild adopted acts from this event on (CMP-008).
        self.inner.set_on_change(props.on_change.clone());
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
                        && mouse.kind == MouseEventKind::Down
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
