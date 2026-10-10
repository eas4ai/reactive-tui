//! The rating builder (CTL-007 to CTL-010), `builder::rating()`.

use crate::component::{same_callback, Component, Element};
use crate::widgets::input::{Rating, RatingProps, RatingState};
use std::sync::Arc;

/// The rating's change callback, called with the new value.
type Callback = Arc<dyn Fn(usize) + Send + Sync>;

/// Builder for Rating components: a row of stars, one cell each.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::rating;
///
/// let stars = rating()
///     .value(3)
///     .max(5)
///     .aria_label("Quality")
///     .on_change(|value| println!("rated {value}"))
///     .build();
/// ```
#[derive(Clone)]
pub struct RatingBuilder {
    value: usize,
    max: usize,
    aria_label: Option<String>,
    disabled: bool,
    class: Option<String>,
    on_change: Option<Callback>,
}

impl Default for RatingBuilder {
    fn default() -> Self {
        Self {
            value: 0,
            max: 5,
            aria_label: None,
            disabled: false,
            class: None,
            on_change: None,
        }
    }
}

/// Equal over the settings and not the callback, so a rebuild that changes
/// only the callback keeps the mounted rating and hands it the new one
/// (CMP-008).
impl PartialEq for RatingBuilder {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            value,
            max,
            aria_label,
            disabled,
            class,
            on_change: _,
        } = self;
        *value == other.value
            && *max == other.max
            && *aria_label == other.aria_label
            && *disabled == other.disabled
            && *class == other.class
    }
}

impl RatingBuilder {
    /// Set the number of lit stars
    pub fn value(mut self, value: usize) -> Self {
        self.value = value;
        self
    }

    /// Set the number of stars
    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }

    /// Set the name the screen reader hears
    pub fn aria_label(mut self, label: &str) -> Self {
        self.aria_label = Some(label.to_string());
        self
    }

    /// Set the disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set CSS classes for styling (`w-N`, `w-full`) on the element
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the callback called with the new value on each change, as
    /// `Rating::with_on_change` is (CMP-009).
    pub fn on_change(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Build the Rating element
    pub fn build(self) -> Element {
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredRating>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }
        element
    }

    fn widget_props(&self) -> RatingProps {
        RatingProps {
            value: self.value,
            max: self.max,
            aria_label: self.aria_label.clone(),
            disabled: self.disabled,
        }
    }
}

impl crate::component::Props for RatingBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A rating built through its builder: the rating itself, with the
/// builder's callback handed to it (CMP-009).
struct ConfiguredRating(Rating);

impl Component for ConfiguredRating {
    type Props = RatingBuilder;
    type State = RatingState;

    fn new(props: Self::Props) -> Self {
        let mut inner = Rating::new(props.widget_props());
        inner.set_on_change(props.on_change.clone());
        Self(inner)
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
        props.value = inner.value;
        result
    }
}

impl From<RatingBuilder> for Element {
    fn from(builder: RatingBuilder) -> Self {
        builder.build()
    }
}
