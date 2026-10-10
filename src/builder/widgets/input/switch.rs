//! The switch builder (CTL-007 to CTL-010): `builder::switch()`.

use crate::component::{Element, same_callback};
use std::sync::Arc;

/// Builder for Switch components
///
/// Provides a fluent API for creating on/off switches with a label.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::switch;
///
/// let wifi = switch()
///     .label("Wi-Fi")
///     .checked(true)
///     .class("wifi-switch")
///     .build();
/// ```
#[derive(Clone)]
pub struct SwitchBuilder {
    checked: bool,
    label: Option<String>,
    aria_label: Option<String>,
    disabled: bool,
    class: Option<String>,
    on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

/// Equal over the settings and not the callback, so a rebuild that changes
/// only the callback keeps the mounted switch and hands it the new one
/// (CMP-008).
impl PartialEq for SwitchBuilder {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            checked,
            label,
            aria_label,
            disabled,
            class,
            on_change: _,
        } = self;
        *checked == other.checked
            && *label == other.label
            && *aria_label == other.aria_label
            && *disabled == other.disabled
            && *class == other.class
    }
}

impl SwitchBuilder {
    /// Create a new SwitchBuilder with default values
    pub(super) fn new() -> Self {
        Self {
            checked: false,
            label: None,
            aria_label: None,
            disabled: false,
            class: None,
            on_change: None,
        }
    }

    /// Set whether the switch is on
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the label text shown after the switch
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set the name the screen reader hears, when the visible label is
    /// not it ("Wi-Fi" shown, "Wireless network" read).
    pub fn aria_label(mut self, label: &str) -> Self {
        self.aria_label = Some(label.to_string());
        self
    }

    /// Set the disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the callback called with the new on or off state each time the
    /// user flips the switch, as `Switch::with_on_change` is (CMP-009).
    pub fn on_change(mut self, f: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Build the Switch element
    ///
    /// # Returns
    /// An `Element` representing the switch
    pub fn build(self) -> Element {
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredSwitch>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }

        element
    }

    fn widget_props(&self) -> crate::widgets::input::SwitchProps {
        crate::widgets::input::SwitchProps {
            checked: self.checked,
            label: self.label.clone(),
            aria_label: self.aria_label.clone(),
            disabled: self.disabled,
        }
    }
}

impl crate::component::Props for SwitchBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A switch built through its builder: the switch itself, with the
/// builder's callback handed to it (CMP-009).
struct ConfiguredSwitch(crate::widgets::input::Switch);

impl crate::component::Component for ConfiguredSwitch {
    type Props = SwitchBuilder;
    type State = crate::widgets::input::SwitchState;

    fn new(props: Self::Props) -> Self {
        let mut inner = crate::widgets::input::Switch::new(props.widget_props());
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
        result
    }
}

impl From<SwitchBuilder> for Element {
    fn from(builder: SwitchBuilder) -> Self {
        builder.build()
    }
}
