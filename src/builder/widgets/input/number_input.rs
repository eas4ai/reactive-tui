//! The number input builder (CTL-007 to CTL-010): `builder::number_input()`.
//! Its element is a `NumberInput` with the builder's callbacks handed to
//! it, so a rebuild with new callbacks keeps the mounted number input
//! (CMP-008, CMP-009).

use super::Callback;
use crate::component::{same_callback, Component, Element};
use crate::event::router::EventResult;
use crate::widgets::input::{NumberInput, NumberInputProps, NumberInputState};
use std::sync::Arc;

/// Builder for NumberInput components
///
/// Provides a fluent API for a number input: its value, its range and step,
/// and the callbacks it reports through.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::number_input;
///
/// let volume = number_input()
///     .aria_label("Volume")
///     .range(0.0, 10.0)
///     .step(0.5)
///     .value(2.0)
///     .build();
/// ```
#[derive(Clone, Default)]
pub struct NumberInputBuilder {
    settings: NumberInputProps,
    class: Option<String>,
    on_change: Option<Callback<f64>>,
    on_submit: Option<Callback<f64>>,
}

/// Equal over the settings and not the callbacks, so a rebuild that changes
/// only a callback keeps the mounted number input and hands it the new one
/// (CMP-008).
impl PartialEq for NumberInputBuilder {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            settings,
            class,
            on_change: _,
            on_submit: _,
        } = self;
        *settings == other.settings && *class == other.class
    }
}

impl NumberInputBuilder {
    /// Create a new NumberInputBuilder with the default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the value
    pub fn value(mut self, value: f64) -> Self {
        self.settings.value = value;
        self
    }

    /// Set the least value
    pub fn min(mut self, min: f64) -> Self {
        self.settings.min = Some(min);
        self
    }

    /// Set the greatest value
    pub fn max(mut self, max: f64) -> Self {
        self.settings.max = Some(max);
        self
    }

    /// Set the least and the greatest value at once
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.settings.min = Some(min);
        self.settings.max = Some(max);
        self
    }

    /// Set the size of one step
    pub fn step(mut self, step: f64) -> Self {
        self.settings.step = step;
        self
    }

    /// Set the name the screen reader hears
    pub fn aria_label(mut self, label: &str) -> Self {
        self.settings.aria_label = Some(label.to_string());
        self
    }

    /// Set whether the number input is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.settings.disabled = disabled;
        self
    }

    /// Set the row's width in cells, as `w-N` does on the element
    pub fn width(mut self, width: u16) -> Self {
        self.settings.width = Some(width);
        self
    }

    /// Set CSS classes for styling, such as `w-40` or `w-full`
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the number input
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the callback called with the value each time it changes, as
    /// `NumberInput::with_on_change` is (CMP-009).
    pub fn on_change(mut self, f: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the value when Confirm submits it, as
    /// `NumberInput::with_on_submit` is (CMP-009).
    pub fn on_submit(mut self, f: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(f));
        self
    }

    /// Build the NumberInput element
    ///
    /// # Returns
    /// An `Element` representing the number input
    pub fn build(self) -> Element {
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredNumberInput>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }
        element
    }

    fn widget_props(&self) -> NumberInputProps {
        self.settings.clone()
    }
}

impl crate::component::Props for NumberInputBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A number input built through its builder: the widget itself, with the
/// builder's callbacks handed to it (CMP-009).
struct ConfiguredNumberInput(NumberInput);

impl Component for ConfiguredNumberInput {
    type Props = NumberInputBuilder;
    type State = NumberInputState;

    fn new(props: Self::Props) -> Self {
        let mut inner = NumberInput::new(props.widget_props());
        inner.set_on_change(props.on_change.clone());
        inner.set_on_submit(props.on_submit.clone());
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
        let change = !same_callback(&props.on_change, &supplied.on_change);
        let submit = !same_callback(&props.on_submit, &supplied.on_submit);
        if change {
            props.on_change = supplied.on_change.clone();
        }
        if submit {
            props.on_submit = supplied.on_submit.clone();
        }
        change || submit
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
    ) -> EventResult {
        // The callbacks a rebuild adopted act from this event on (CMP-008).
        self.0.set_on_change(props.on_change.clone());
        self.0.set_on_submit(props.on_submit.clone());
        let mut inner = props.widget_props();
        let result = self.0.handle_event(event, &mut inner, state);
        props.settings.value = inner.value;
        result
    }
}

impl From<NumberInputBuilder> for Element {
    fn from(builder: NumberInputBuilder) -> Self {
        builder.build()
    }
}
