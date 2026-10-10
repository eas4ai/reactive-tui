//! The code input builder: `builder::code_input()` builds a code input whose
//! callbacks are handed to the mounted component (CMP-008, CMP-009).

use super::Callback;
use crate::component::{same_callback, Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::Event;
use crate::widgets::{CodeInput, CodeInputProps, CodeInputState};
use std::sync::Arc;

/// Builder for a code input built through `builder::code_input()`.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::code_input;
///
/// let code = code_input()
///     .length(6)
///     .group(3)
///     .on_complete(|code| println!("code {code}"))
///     .build();
/// ```
#[derive(Clone)]
pub struct CodeInputBuilder {
    props: CodeInputProps,
    class: Option<String>,
    on_change: Option<Callback<String>>,
    on_complete: Option<Callback<String>>,
}

/// Equal over the settings and not the callbacks, so a rebuild that changes
/// only a callback keeps the mounted code input and hands it the new one
/// (CMP-008).
impl PartialEq for CodeInputBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.props == other.props && self.class == other.class
    }
}

impl CodeInputBuilder {
    /// A builder with the default settings: six digits, no groups.
    pub(super) fn new() -> Self {
        Self {
            props: CodeInputProps::default(),
            class: None,
            on_change: None,
            on_complete: None,
        }
    }

    /// Set how many digits the code holds.
    pub fn length(mut self, length: usize) -> Self {
        self.props.length = length;
        self
    }

    /// Set the digits per group, with one cell of space between groups.
    pub fn group(mut self, group: usize) -> Self {
        self.props.group = Some(group);
        self
    }

    /// Set the digits typed so far.
    pub fn value(mut self, value: &str) -> Self {
        self.props.value = value.to_string();
        self
    }

    /// Hide the digits behind dots.
    pub fn masked(mut self, masked: bool) -> Self {
        self.props.masked = masked;
        self
    }

    /// Set the name's first part; "Code" when unset.
    pub fn label(mut self, label: &str) -> Self {
        self.props.label = Some(label.to_string());
        self
    }

    /// Set the name the screen reader hears instead of the label.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.props.aria_label = Some(label.to_string());
        self
    }

    /// Set whether the code input is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.disabled = disabled;
        self
    }

    /// Set CSS classes for styling, such as `w-40` or `w-full`.
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the callback called with the code after each change.
    pub fn on_change(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the code when it is complete.
    pub fn on_complete(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_complete = Some(Arc::new(f));
        self
    }

    /// Build the code input element.
    pub fn build(self) -> Element {
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredCodeInput>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }
        element
    }

    fn widget_props(&self) -> CodeInputProps {
        self.props.clone()
    }
}

impl Props for CodeInputBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A code input built through its builder: the code input itself, with the
/// builder's callbacks handed to it (CMP-009).
struct ConfiguredCodeInput(CodeInput);

impl Component for ConfiguredCodeInput {
    type Props = CodeInputBuilder;
    type State = CodeInputState;

    fn new(props: Self::Props) -> Self {
        let mut inner = CodeInput::new(props.widget_props());
        inner.set_on_change(props.on_change.clone());
        inner.set_on_complete(props.on_complete.clone());
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
        let same = same_callback(&props.on_change, &supplied.on_change)
            && same_callback(&props.on_complete, &supplied.on_complete);
        if same {
            return false;
        }
        props.on_change = supplied.on_change.clone();
        props.on_complete = supplied.on_complete.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.0.render(&props.widget_props(), state)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        // The callbacks a rebuild adopted act from this event on (CMP-008).
        self.0.set_on_change(props.on_change.clone());
        self.0.set_on_complete(props.on_complete.clone());
        let mut inner = props.widget_props();
        let result = self.0.handle_event(event, &mut inner, state);
        props.props.value = inner.value;
        result
    }
}

impl From<CodeInputBuilder> for Element {
    fn from(builder: CodeInputBuilder) -> Self {
        builder.build()
    }
}
