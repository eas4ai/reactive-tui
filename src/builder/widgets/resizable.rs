//! The resizable split builder (NAV-007 to NAV-010): `builder::resizable()`.
//! Its element is a `Resizable` with the builder's callback handed to it,
//! so a rebuild with a new callback keeps the mounted split (CMP-008,
//! CMP-009).

use crate::component::{same_callback, Component, Element, LayoutInfo, Props};
use crate::event::router::EventResult;
use crate::widgets::layout::resizable::ResizeCallback;
use crate::widgets::layout::{Pane, Resizable, ResizableProps, SplitDirection};
use std::sync::Arc;

/// Builder for resizable splits.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::{div, resizable};
///
/// let split = resizable()
///     .vertical()
///     .aria_label("Panes")
///     .pane(div().build())
///     .pane(div().build())
///     .build();
/// ```
#[derive(Clone, Default)]
pub struct ResizableBuilder {
    props: ResizableProps,
    class: Option<String>,
    on_resize: Option<ResizeCallback>,
}

/// Equal over the settings and not the callback, so a rebuild that changes
/// only the callback keeps the mounted split and hands it the new one
/// (CMP-008).
impl PartialEq for ResizableBuilder {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            props,
            class,
            on_resize: _,
        } = self;
        *props == other.props && *class == other.class
    }
}

impl Props for ResizableBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Create a resizable split builder.
pub fn resizable() -> ResizableBuilder {
    ResizableBuilder::new()
}

impl ResizableBuilder {
    /// Create a builder with the default settings: panes side by side.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lay the panes side by side, with vertical dividers (the default).
    pub fn horizontal(mut self) -> Self {
        self.props.direction = SplitDirection::Row;
        self
    }

    /// Stack the panes, with horizontal dividers.
    pub fn vertical(mut self) -> Self {
        self.props.direction = SplitDirection::Column;
        self
    }

    /// Add a pane with an equal share.
    pub fn pane(self, content: Element) -> Self {
        self.pane_with(Pane::new(content))
    }

    /// Add a pane built with its own size and minimum.
    pub fn pane_with(mut self, pane: Pane) -> Self {
        self.props.panes.push(pane);
        self
    }

    /// Set the name the screen reader hears for each divider.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.props.aria_label = Some(label.to_string());
        self
    }

    /// Set whether the dividers ignore keys, drags and the screen reader.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.disabled = disabled;
        self
    }

    /// Set the CSS-like class names of the element, such as `w-60` or
    /// `h-10`, which size the split in place of what its parent allots.
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the callback called with every pane's size after each move, as
    /// `Resizable::with_on_resize` is (CMP-009).
    pub fn on_resize(mut self, f: impl Fn(Vec<u16>) + Send + Sync + 'static) -> Self {
        self.on_resize = Some(Arc::new(f));
        self
    }

    /// Build the split's element.
    pub fn build(self) -> Element {
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredResizable>(self);
        if let Some(class) = class {
            element = element.with_class(&class);
        }
        element
    }
}

/// A split built through its builder: the widget itself, with the
/// builder's callback handed to it (CMP-009).
struct ConfiguredResizable(Resizable);

impl Component for ConfiguredResizable {
    type Props = ResizableBuilder;
    type State = ();

    fn new(props: Self::Props) -> Self {
        let mut inner = Resizable::new(props.props.clone());
        inner.set_on_resize(props.on_resize.clone());
        Self(inner)
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        self.0.update(&props.props, state)
    }

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        let changed = !same_callback(&props.on_resize, &supplied.on_resize);
        if changed {
            props.on_resize = supplied.on_resize.clone();
        }
        changed
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.0.render(&props.props, state)
    }

    fn layout(
        &mut self,
        bounds: LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        self.0.layout(bounds, &mut props.props, state)
    }

    fn handle_event(
        &mut self,
        event: &crate::event::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        // The callback a rebuild adopted acts from this event on (CMP-008).
        self.0.set_on_resize(props.on_resize.clone());
        self.0.handle_event(event, &mut props.props, state)
    }
}

impl From<ResizableBuilder> for Element {
    fn from(builder: ResizableBuilder) -> Self {
        builder.build()
    }
}
