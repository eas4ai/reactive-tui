use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::sync::Arc;

/// Builder for creating Slider components with a fluent API
#[derive(Clone, Debug)]
pub struct SliderBuilder {
    min: f64,
    max: f64,
    value: f64,
    step: f64,
    orientation: SliderOrientation,
    show_value: bool,
    show_labels: bool,
    disabled: bool,
    width: u16,
}

impl Default for SliderBuilder {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            value: 50.0,
            step: 1.0,
            orientation: SliderOrientation::Horizontal,
            show_value: true,
            show_labels: false,
            disabled: false,
            width: 20,
        }
    }
}

impl SliderBuilder {
    /// Create a new SliderBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum value
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    /// Set the maximum value
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// Set the current value
    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Set the step size for increments/decrements
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Set the range (min and max) at once
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Set the orientation (horizontal or vertical)
    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set whether to show the current value
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Set whether to show min/max labels
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Set whether the slider is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the visual width of the slider track
    pub fn width(mut self, width: u16) -> Self {
        self.width = width.max(5); // Minimum width of 5
        self
    }

    /// Build the SliderProps
    pub fn build(self) -> SliderProps {
        let mut props = SliderProps {
            min: self.min,
            max: self.max,
            value: self.value,
            step: self.step,
            orientation: self.orientation,
            show_value: self.show_value,
            show_labels: self.show_labels,
            disabled: self.disabled,
            width: self.width,
        };
        if props.valid() {
            props.value = props.bounded_value();
        }
        props
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::typed::<Slider>(self.build())
    }
}

/// Orientation for slider layout
#[derive(Clone, Debug, PartialEq, Default)]
pub enum SliderOrientation {
    /// Horizontal slider (left to right)
    #[default]
    Horizontal,
    /// Vertical slider (bottom to top)
    Vertical,
}

/// Properties for Slider component
#[derive(Clone, Debug, PartialEq)]
pub struct SliderProps {
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Current value
    pub value: f64,
    /// Step size for increments
    pub step: f64,
    /// Slider orientation
    pub orientation: SliderOrientation,
    /// Whether to show the current value
    pub show_value: bool,
    /// Whether to show min/max labels
    pub show_labels: bool,
    /// Whether the slider is disabled
    pub disabled: bool,
    /// Visual width of the slider track (for horizontal) or height (for vertical)
    pub width: u16,
}

impl Default for SliderProps {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            value: 50.0,
            step: 1.0,
            orientation: SliderOrientation::Horizontal,
            show_value: true,
            show_labels: false,
            disabled: false,
            width: 20,
        }
    }
}

impl Props for SliderProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for Slider component
#[derive(Clone, Debug, Default)]
pub struct SliderState {
    /// Whether the slider has keyboard focus
    pub is_focused: bool,
    /// Whether the slider is being dragged
    pub is_dragging: bool,
    /// Whether the mouse is hovering over the slider
    pub is_hover: bool,
}

/// Slider component with full keyboard and mouse support
pub struct Slider {
    viewport: Option<crate::component::LayoutInfo>,
    label: Option<String>,
    on_change: Option<Arc<dyn Fn(f64) + Send + Sync>>,
}

impl SliderProps {
    fn valid(&self) -> bool {
        self.min.is_finite()
            && self.max.is_finite()
            && self.min <= self.max
            && (self.max - self.min).is_finite()
            && self.value.is_finite()
            && self.step.is_finite()
            && self.step > 0.0
    }

    fn bounded_value(&self) -> f64 {
        self.value.clamp(self.min, self.max)
    }
}

impl Slider {
    /// Set the onChange callback for when the value changes
    pub fn with_on_change(mut self, f: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    pub(crate) fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    fn label_rows(&self) -> usize {
        self.label
            .as_ref()
            .map_or(0, |label| label.split('\n').count())
    }

    // Track coordinates are relative to rendered content; padding is handled
    // once at the mouse boundary. Decorations occupy cells outside the track.
    fn track(&self, props: &SliderProps) -> (usize, usize, usize) {
        let labels = if props.show_labels {
            (
                format!("{:.0} ", props.min).len(),
                format!(" {:.0}", props.max).len(),
            )
        } else {
            (0, 0)
        };
        let value_width = if props.show_value {
            1 + format!("{:.1}", props.min)
                .len()
                .max(format!("{:.1}", props.max).len())
        } else {
            0
        };
        match props.orientation {
            SliderOrientation::Horizontal => {
                let x = 3 + labels.0;
                let available = self.viewport.map_or(props.width as usize, |bounds| {
                    (bounds.content_size().0 as usize)
                        .saturating_sub(x + 1 + labels.1 + value_width)
                });
                (x, self.label_rows(), available.min(props.width as usize))
            }
            SliderOrientation::Vertical => {
                let label_rows = self.label_rows();
                let y = usize::from(props.show_labels);
                let available = self.viewport.map_or(props.width as usize, |bounds| {
                    (bounds.content_size().1 as usize)
                        .saturating_sub(label_rows + y * 2 + usize::from(props.show_value))
                });
                (2, label_rows + y, available.min(props.width as usize))
            }
        }
    }

    fn calculate_thumb_position(&self, props: &SliderProps) -> usize {
        if !props.valid() || props.min == props.max {
            return 0;
        }
        let normalized = (props.bounded_value() - props.min) / (props.max - props.min);
        (normalized * self.track(props).2.saturating_sub(1) as f64).round() as usize
    }

    fn value_from_position(&self, position: usize, props: &SliderProps) -> f64 {
        let last = self.track(props).2.saturating_sub(1);
        if last == 0 {
            return props.min;
        }
        if position >= last {
            return props.max;
        }
        let value = props.min + position as f64 / last as f64 * (props.max - props.min);
        Self::snap(value, props)
    }

    fn snap(value: f64, props: &SliderProps) -> f64 {
        if value <= props.min {
            return props.min;
        }
        if value >= props.max {
            return props.max;
        }
        let steps = ((value - props.min) / props.step).round();
        let snapped = props.min + steps * props.step;
        if snapped.is_finite() {
            snapped.clamp(props.min, props.max)
        } else {
            value.clamp(props.min, props.max)
        }
    }

    fn change(&self, value: f64, props: &mut SliderProps) {
        if value != props.value {
            props.value = value;
            if let Some(callback) = &self.on_change {
                callback(value);
            }
        }
    }

    fn mouse_position(
        &self,
        event: &MouseEvent,
        props: &SliderProps,
        dragging: bool,
    ) -> Option<usize> {
        let (x, y, length) = self.track(props);
        if length == 0 {
            return None;
        }
        let insets = self.viewport.map_or([0.0; 4], |v| v.insets);
        let mx = event.position.x() as f32 - insets[0];
        let my = event.position.y() as f32 - insets[1];
        let (along, across) = match props.orientation {
            SliderOrientation::Horizontal => (mx - x as f32, my - y as f32),
            SliderOrientation::Vertical => (my - y as f32, mx - x as f32),
        };
        if !dragging && (!(0.0..1.0).contains(&across) || along < 0.0 || along >= length as f32) {
            return None;
        }
        let position = along.clamp(0.0, (length - 1) as f32) as usize;
        Some(if props.orientation == SliderOrientation::Vertical {
            length - 1 - position
        } else {
            position
        })
    }
}

impl Component for Slider {
    type Props = SliderProps;
    type State = SliderState;

    fn new(_props: Self::Props) -> Self {
        Self {
            viewport: None,
            label: None,
            on_change: None,
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if props.disabled || !props.valid() {
            *state = SliderState::default();
        }
        true
    }

    fn layout(
        &mut self,
        bounds: crate::component::LayoutInfo,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        let previous = self.track(props);
        self.viewport = Some(bounds);
        previous != self.track(props)
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        if !props.valid() {
            return Element::text("Invalid slider range").disabled(true);
        }
        let (_, _, length) = self.track(props);
        let thumb = self.calculate_thumb_position(props);
        let marker = if props.disabled {
            '○'
        } else if state.is_hover {
            '◉'
        } else {
            '●'
        };
        let focus = if state.is_focused && !props.disabled {
            "▶ "
        } else {
            "  "
        };
        let value = props.bounded_value();
        let result = match props.orientation {
            SliderOrientation::Horizontal => {
                let mut text = focus.to_owned();
                if props.show_labels {
                    text.push_str(&format!("{:.0} ", props.min));
                }
                text.push('[');
                for i in 0..length {
                    text.push(if i == thumb {
                        marker
                    } else if i < thumb {
                        '═'
                    } else {
                        '─'
                    });
                }
                text.push(']');
                if props.show_labels {
                    text.push_str(&format!(" {:.0}", props.max));
                }
                if props.show_value {
                    text.push_str(&format!(" {value:.1}"));
                }
                text
            }
            SliderOrientation::Vertical => {
                let mut rows = Vec::new();
                if props.show_labels {
                    rows.push(format!("  {:.0}", props.max));
                }
                for row in 0..length {
                    let position = length - 1 - row;
                    let symbol = if position == thumb {
                        marker
                    } else if position < thumb {
                        '┃'
                    } else {
                        '│'
                    };
                    rows.push(format!("{}{symbol}", if row == 0 { focus } else { "  " }));
                }
                if props.show_labels {
                    rows.push(format!("  {:.0}", props.min));
                }
                if props.show_value {
                    rows.push(format!("  {value:.1}"));
                }
                rows.join("\n")
            }
        };
        let result = if let Some(label) = &self.label {
            format!("{label}\n{result}")
        } else {
            result
        };
        use crate::accessibility::{Node, Role};
        let mut accessible = Node::new(Role::Slider);
        if let Some(label) = &self.label {
            accessible.set_label(label.clone());
        }
        accessible.inner.set_numeric_value(value);
        accessible.inner.set_min_numeric_value(props.min);
        accessible.inner.set_max_numeric_value(props.max);
        accessible.inner.set_numeric_value_step(props.step);
        accessible.inner.set_orientation(match props.orientation {
            SliderOrientation::Horizontal => accesskit::Orientation::Horizontal,
            SliderOrientation::Vertical => accesskit::Orientation::Vertical,
        });
        Element::text(result)
            .with_accessibility(accessible)
            .with_class("whitespace-pre overflow-hidden")
            .with_focus(crate::component::FocusProps::input())
            .disabled(props.disabled || length == 0)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if let Event::Focus(event) = event {
            if !matches!(
                event.kind,
                crate::event::types::FocusEventKind::Gained
                    | crate::event::types::FocusEventKind::Lost
            ) {
                return EventResult::Ignored;
            }
            state.is_focused = event.kind == crate::event::types::FocusEventKind::Gained
                && !props.disabled
                && props.valid();
            if !state.is_focused {
                state.is_dragging = false;
                state.is_hover = false;
            }
            return EventResult::Consumed;
        }
        if props.disabled || !props.valid() || self.track(props).2 == 0 {
            return EventResult::Ignored;
        }
        match event {
            Event::Key(event) if state.is_focused => self.handle_key_event(event, props, state),
            Event::Mouse(event) => self.handle_mouse_event(event, props, state),
            _ => EventResult::Ignored,
        }
    }
}

impl Slider {
    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut SliderProps,
        _state: &mut SliderState,
    ) -> EventResult {
        let value = props.bounded_value();
        let new_value = match event.code {
            KeyCode::Left | KeyCode::Down => (value - props.step).max(props.min),
            KeyCode::Right | KeyCode::Up => (value + props.step).min(props.max),
            KeyCode::PageDown => Self::snap(
                (value - ((props.max - props.min) * 0.1).max(props.step)).max(props.min),
                props,
            ),
            KeyCode::PageUp => Self::snap(
                (value + ((props.max - props.min) * 0.1).max(props.step)).min(props.max),
                props,
            ),
            KeyCode::Home => props.min,
            KeyCode::End => props.max,
            _ => return EventResult::Ignored,
        };
        self.change(new_value, props);
        EventResult::Consumed
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut SliderProps,
        state: &mut SliderState,
    ) -> EventResult {
        use crate::event::types::MouseButton;
        match event.kind {
            MouseEventKind::Down | MouseEventKind::Click if event.button == MouseButton::Left => {
                let Some(position) = self.mouse_position(event, props, false) else {
                    return EventResult::Ignored;
                };
                state.is_dragging = event.kind == MouseEventKind::Down;
                self.change(self.value_from_position(position, props), props);
                EventResult::Consumed
            }
            MouseEventKind::Drag if state.is_dragging && event.button == MouseButton::Left => {
                if let Some(position) = self.mouse_position(event, props, true) {
                    self.change(self.value_from_position(position, props), props);
                }
                EventResult::Consumed
            }
            MouseEventKind::Up if state.is_dragging => {
                state.is_dragging = false;
                EventResult::Consumed
            }
            MouseEventKind::Move | MouseEventKind::Enter => {
                state.is_hover = self.mouse_position(event, props, false).is_some();
                EventResult::Ignored
            }
            MouseEventKind::Leave => {
                state.is_hover = false;
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::KeyModifiers;

    #[test]
    fn test_slider_keyboard_control() {
        let mut slider = Slider::new(SliderProps::default());
        let mut props = SliderProps {
            min: 0.0,
            max: 100.0,
            value: 50.0,
            step: 10.0,
            ..Default::default()
        };
        let mut state = SliderState {
            is_focused: true,
            is_dragging: false,
            is_hover: false,
        };

        // Increase value
        let event = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        let result = slider.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(props.value, 60.0);

        // Decrease value
        let event = Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        slider.handle_event(&event, &mut props, &mut state);
        assert_eq!(props.value, 50.0);

        // Jump to minimum
        let event = Event::Key(KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        slider.handle_event(&event, &mut props, &mut state);
        assert_eq!(props.value, 0.0);

        // Jump to maximum
        let event = Event::Key(KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        slider.handle_event(&event, &mut props, &mut state);
        assert_eq!(props.value, 100.0);
    }

    #[test]
    fn test_slider_value_clamping() {
        let props = SliderProps {
            min: 0.0,
            max: 10.0,
            value: 5.0,
            step: 1.0,
            ..Default::default()
        };

        let slider = Slider::new(props.clone());

        // Test thumb position calculation
        let pos = slider.calculate_thumb_position(&props);
        assert_eq!(pos, 10); // Middle of a 20-char track (5.0 is 50% of 0-10 range)

        // Test value from position
        let value = slider.value_from_position(0, &props);
        assert_eq!(value, 0.0);

        let value = slider.value_from_position(19, &props);
        assert_eq!(value, 10.0);
    }

    #[test]
    fn test_slider_disabled() {
        let mut slider = Slider::new(SliderProps::default());
        let mut props = SliderProps {
            disabled: true,
            ..Default::default()
        };
        let mut state = SliderState::default();

        let event = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        let result = slider.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Ignored);
        assert_eq!(props.value, 50.0); // Should not change
    }
}
