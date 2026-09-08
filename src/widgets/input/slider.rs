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
        self.value = value.clamp(self.min, self.max);
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
        self.value = self.value.clamp(min, max);
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
        SliderProps {
            min: self.min,
            max: self.max,
            value: self.value,
            step: self.step,
            orientation: self.orientation,
            show_value: self.show_value,
            show_labels: self.show_labels,
            disabled: self.disabled,
            width: self.width,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("Slider").with_props(self.build())
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
    state: SliderState,
    on_change: Option<Arc<dyn Fn(f64) + Send + Sync>>,
}

impl Slider {
    /// Set the onChange callback for when the value changes
    pub fn with_on_change(mut self, f: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Calculate the position of the thumb on the track
    fn calculate_thumb_position(&self, props: &SliderProps) -> usize {
        let range = props.max - props.min;
        if range == 0.0 {
            return 0;
        }

        let normalized = (props.value - props.min) / range;
        let track_length = props.width.saturating_sub(1) as f64;
        (normalized * track_length).round() as usize
    }

    /// Calculate value from position
    fn value_from_position(&self, position: usize, props: &SliderProps) -> f64 {
        let track_length = props.width.saturating_sub(1) as f64;
        if track_length == 0.0 {
            return props.min;
        }

        let normalized = position as f64 / track_length;
        let value = props.min + normalized * (props.max - props.min);

        // Round to nearest step
        let steps = ((value - props.min) / props.step).round();
        (props.min + steps * props.step).clamp(props.min, props.max)
    }
}

impl Component for Slider {
    type Props = SliderProps;
    type State = SliderState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: SliderState::default(),
            on_change: None,
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let mut result = String::new();

        // Add focus indicator
        if state.is_focused && !props.disabled {
            result.push_str("▶ ");
        } else {
            result.push_str("  ");
        }

        match props.orientation {
            SliderOrientation::Horizontal => {
                // Show min label if requested
                if props.show_labels {
                    result.push_str(&format!("{:.0} ", props.min));
                }

                // Draw the track
                result.push('[');

                let thumb_pos = self.calculate_thumb_position(props);
                for i in 0..props.width {
                    if i == thumb_pos as u16 {
                        if props.disabled {
                            result.push('○'); // Disabled thumb
                        } else if state.is_dragging {
                            result.push('●'); // Dragging thumb
                        } else if state.is_hover {
                            result.push('◉'); // Hover thumb
                        } else {
                            result.push('●'); // Normal thumb
                        }
                    } else if i < thumb_pos as u16 {
                        result.push('═'); // Filled track
                    } else {
                        result.push('─'); // Empty track
                    }
                }

                result.push(']');

                // Show max label if requested
                if props.show_labels {
                    result.push_str(&format!(" {:.0}", props.max));
                }

                // Show current value if requested
                if props.show_value {
                    if props.disabled {
                        result.push_str(&format!(" ({:.1})", props.value));
                    } else {
                        result.push_str(&format!(" {:.1}", props.value));
                    }
                }
            }
            SliderOrientation::Vertical => {
                // For vertical, we'd need multi-line rendering
                // This is a simplified single-line representation
                result.push('│');
                let thumb_pos = self.calculate_thumb_position(props);
                for i in 0..props.width {
                    if i == thumb_pos as u16 {
                        result.push('●');
                    } else {
                        result.push('│');
                    }
                }
                result.push('│');

                if props.show_value {
                    result.push_str(&format!(" {:.1}", props.value));
                }
            }
        }

        Element::text(result)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if props.disabled {
            return EventResult::Ignored;
        }

        match event {
            Event::Key(key_event) => {
                if !state.is_focused {
                    return EventResult::Ignored;
                }
                self.handle_key_event(key_event, props, state)
            }
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            Event::Focus(_) => {
                state.is_focused = true;
                EventResult::Consumed
            }
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
        let old_value = props.value;

        match event.code {
            KeyCode::Left | KeyCode::Down => {
                // Decrease value by step
                props.value = (props.value - props.step).max(props.min);
            }
            KeyCode::Right | KeyCode::Up => {
                // Increase value by step
                props.value = (props.value + props.step).min(props.max);
            }
            KeyCode::PageDown => {
                // Decrease by larger step (10% of range)
                let large_step = (props.max - props.min) * 0.1;
                props.value = (props.value - large_step).max(props.min);
                // Round to nearest step
                let steps = ((props.value - props.min) / props.step).round();
                props.value = props.min + steps * props.step;
            }
            KeyCode::PageUp => {
                // Increase by larger step (10% of range)
                let large_step = (props.max - props.min) * 0.1;
                props.value = (props.value + large_step).min(props.max);
                // Round to nearest step
                let steps = ((props.value - props.min) / props.step).round();
                props.value = props.min + steps * props.step;
            }
            KeyCode::Home => {
                // Jump to minimum
                props.value = props.min;
            }
            KeyCode::End => {
                // Jump to maximum
                props.value = props.max;
            }
            _ => return EventResult::Ignored,
        }

        if props.value != old_value {
            if let Some(on_change) = &self.on_change {
                on_change(props.value);
            }
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut SliderProps,
        state: &mut SliderState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Down => {
                // Start dragging
                state.is_dragging = true;
                state.is_focused = true;

                // Update value based on click position
                if props.orientation == SliderOrientation::Horizontal {
                    let x = event.position.x() as usize;
                    // Account for the focus indicator and labels
                    let offset = 2 + if props.show_labels { 3 } else { 0 };
                    if x >= offset && x < offset + props.width as usize {
                        let relative_x = x - offset;
                        let new_value = self
                            .value_from_position(relative_x.min(props.width as usize - 1), props);

                        if new_value != props.value {
                            props.value = new_value;
                            if let Some(on_change) = &self.on_change {
                                on_change(props.value);
                            }
                        }
                    }
                }

                EventResult::Consumed
            }
            MouseEventKind::Up => {
                // Stop dragging
                state.is_dragging = false;
                EventResult::Consumed
            }
            MouseEventKind::Drag => {
                // Update value while dragging
                if state.is_dragging && props.orientation == SliderOrientation::Horizontal {
                    let x = event.position.x() as usize;
                    let offset = 2 + if props.show_labels { 3 } else { 0 };
                    if x >= offset && x < offset + props.width as usize {
                        let relative_x = x - offset;
                        let new_value = self
                            .value_from_position(relative_x.min(props.width as usize - 1), props);

                        if new_value != props.value {
                            props.value = new_value;
                            if let Some(on_change) = &self.on_change {
                                on_change(props.value);
                            }
                        }
                    }
                }
                EventResult::Consumed
            }
            MouseEventKind::Move => {
                // Track hover state
                let x = event.position.x() as usize;
                let offset = 2 + if props.show_labels { 3 } else { 0 };
                state.is_hover = x >= offset && x < offset + props.width as usize;
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
