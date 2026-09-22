use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::Event;
use std::sync::Arc;

/// Function type for formatting progress bar text
pub type FormatterFn = dyn Fn(f64, f64, f64) -> String + Send + Sync;

/// Progress bar orientation
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressBarOrientation {
    /// Horizontal progress bar
    Horizontal,
    /// Vertical progress bar
    Vertical,
}

/// Builder for creating ProgressBar components with a fluent API
#[derive(Clone)]
pub struct ProgressBarBuilder {
    value: f64,
    max_value: f64,
    min_value: f64,
    show_percentage: bool,
    show_value: bool,
    indeterminate: bool,
    animated: bool,
    label: Option<String>,
    color: Option<String>,
    background_color: Option<String>,
    height: u16,
    width: Option<u16>,
    style: Option<String>,
    bar_style: Option<String>,
    text_style: Option<String>,
    orientation: ProgressBarOrientation,
    segments: Option<u16>,
    striped: bool,
    pulse: bool,
    custom_formatter: Option<Arc<FormatterFn>>,
    on_complete: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ProgressBarBuilder {
    /// Create a new ProgressBarBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current progress value
    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Set the maximum value
    pub fn max_value(mut self, max: f64) -> Self {
        self.max_value = max;
        self
    }

    /// Set the minimum value
    pub fn min_value(mut self, min: f64) -> Self {
        self.min_value = min;
        self
    }

    /// Set value range (min and max)
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min_value = min;
        self.max_value = max;
        self
    }

    /// Show or hide percentage text
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Show or hide current value
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Set as indeterminate progress
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Enable or disable animation
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Set label text
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set progress bar color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set background color
    pub fn background_color(mut self, color: impl Into<String>) -> Self {
        self.background_color = Some(color.into());
        self
    }

    /// Set the height
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Set the width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set container style
    pub fn style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    /// Set progress bar style
    pub fn bar_style(mut self, style: impl Into<String>) -> Self {
        self.bar_style = Some(style.into());
        self
    }

    /// Set text style
    pub fn text_style(mut self, style: impl Into<String>) -> Self {
        self.text_style = Some(style.into());
        self
    }

    /// Set orientation
    pub fn orientation(mut self, orientation: ProgressBarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set as vertical progress bar
    pub fn vertical(mut self) -> Self {
        self.orientation = ProgressBarOrientation::Vertical;
        self
    }

    /// Set number of segments
    pub fn segments(mut self, segments: u16) -> Self {
        self.segments = Some(segments);
        self
    }

    /// Enable striped pattern
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// Enable pulse animation
    pub fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    /// Set custom formatter function
    pub fn custom_formatter(mut self, formatter: Arc<FormatterFn>) -> Self {
        self.custom_formatter = Some(formatter);
        self
    }

    /// Set completion callback
    pub fn on_complete(mut self, callback: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_complete = Some(callback);
        self
    }

    /// Build the ProgressBarProps
    pub fn build(self) -> ProgressBarProps {
        ProgressBarProps {
            value: self.value,
            max_value: self.max_value,
            min_value: self.min_value,
            show_percentage: self.show_percentage,
            show_value: self.show_value,
            indeterminate: self.indeterminate,
            animated: self.animated,
            label: self.label,
            color: self.color,
            background_color: self.background_color,
            height: self.height,
            width: self.width,
            style: self.style,
            bar_style: self.bar_style,
            text_style: self.text_style,
            orientation: self.orientation,
            segments: self.segments,
            striped: self.striped,
            pulse: self.pulse,
            custom_formatter: self.custom_formatter,
            on_complete: self.on_complete,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("ProgressBar").with_props(self.build())
    }
}

impl Default for ProgressBarBuilder {
    fn default() -> Self {
        Self {
            value: 0.0,
            max_value: 100.0,
            min_value: 0.0,
            show_percentage: true,
            show_value: false,
            indeterminate: false,
            animated: false,
            label: None,
            color: Some("bg-blue".to_string()),
            background_color: Some("bg-gray-200".to_string()),
            height: 1,
            width: None,
            style: None,
            bar_style: None,
            text_style: None,
            orientation: ProgressBarOrientation::Horizontal,
            segments: None,
            striped: false,
            pulse: false,
            custom_formatter: None,
            on_complete: None,
        }
    }
}

/// Props for the ProgressBar component
#[derive(Clone)]
pub struct ProgressBarProps {
    /// Current progress value
    pub value: f64,
    /// Maximum progress value
    pub max_value: f64,
    /// Minimum progress value
    pub min_value: f64,
    /// Whether to show percentage text
    pub show_percentage: bool,
    /// Whether to show current value
    pub show_value: bool,
    /// Whether progress is indeterminate
    pub indeterminate: bool,
    /// Whether to animate progress changes
    pub animated: bool,
    /// Optional label text
    pub label: Option<String>,
    /// Progress bar color
    pub color: Option<String>,
    /// Background color
    pub background_color: Option<String>,
    /// Height of the progress bar
    pub height: u16,
    /// Optional width constraint
    pub width: Option<u16>,
    /// CSS style for container
    pub style: Option<String>,
    /// CSS style for progress bar
    pub bar_style: Option<String>,
    /// CSS style for text
    pub text_style: Option<String>,
    /// Orientation of the progress bar
    pub orientation: ProgressBarOrientation,
    /// Number of segments for segmented display
    pub segments: Option<u16>,
    /// Whether to show striped pattern
    pub striped: bool,
    /// Whether to show pulsing animation
    pub pulse: bool,
    /// Custom formatter function
    pub custom_formatter: Option<Arc<FormatterFn>>,
    /// Callback when progress completes
    pub on_complete: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Default for ProgressBarProps {
    fn default() -> Self {
        Self {
            value: 0.0,
            max_value: 100.0,
            min_value: 0.0,
            show_percentage: true,
            show_value: false,
            indeterminate: false,
            animated: false,
            label: None,
            color: Some("bg-blue".to_string()),
            background_color: Some("bg-gray-200".to_string()),
            height: 1,
            width: None,
            style: None,
            bar_style: None,
            text_style: None,
            orientation: ProgressBarOrientation::Horizontal,
            segments: None,
            striped: false,
            pulse: false,
            custom_formatter: None,
            on_complete: None,
        }
    }
}

impl PartialEq for ProgressBarProps {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.max_value == other.max_value
            && self.min_value == other.min_value
            && self.show_percentage == other.show_percentage
            && self.show_value == other.show_value
            && self.indeterminate == other.indeterminate
            && self.animated == other.animated
            && self.label == other.label
            && self.color == other.color
            && self.background_color == other.background_color
            && self.height == other.height
            && self.width == other.width
            && self.style == other.style
            && self.bar_style == other.bar_style
            && self.text_style == other.text_style
            && self.orientation == other.orientation
            && self.segments == other.segments
            && self.striped == other.striped
            && self.pulse == other.pulse
            && match (&self.custom_formatter, &other.custom_formatter) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
            && match (&self.on_complete, &other.on_complete) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Props for ProgressBarProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// State for the ProgressBar component
#[derive(Debug, Clone, Default, PartialEq)]
/// State for the ProgressBar component
pub struct ProgressBarState {
    /// Current animation frame counter
    pub animation_frame: u64,
    /// Whether progress has completed
    pub completed: bool,
    /// Last recorded progress value
    pub last_value: f64,
    /// Position for indeterminate animation
    pub indeterminate_position: f64,
    /// Direction of pulse animation (true = forward, false = backward)
    pub pulse_direction: bool,
}

/// ProgressBar component for showing completion progress
pub struct ProgressBar;

impl ProgressBar {
    /// Create a new ProgressBar with default props
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Element {
        Element::component_with_props("ProgressBar", ProgressBarProps::default())
    }

    /// Create a ProgressBar with custom props
    pub fn with_props(props: ProgressBarProps) -> Element {
        Element::component_with_props("ProgressBar", props)
    }

    /// Builder method for value
    pub fn with_value(mut props: ProgressBarProps, value: f64) -> ProgressBarProps {
        props.value = value.max(props.min_value).min(props.max_value);
        props
    }

    /// Builder method for range
    pub fn with_range(mut props: ProgressBarProps, min: f64, max: f64) -> ProgressBarProps {
        props.min_value = min;
        props.max_value = max;
        props.value = props.value.max(min).min(max);
        props
    }

    /// Builder method for label
    pub fn with_label(mut props: ProgressBarProps, label: &str) -> ProgressBarProps {
        props.label = Some(label.to_string());
        props
    }

    /// Builder method for color
    pub fn with_color(mut props: ProgressBarProps, color: &str) -> ProgressBarProps {
        props.color = Some(color.to_string());
        props
    }

    /// Builder method for indeterminate mode
    pub fn indeterminate(mut props: ProgressBarProps, indeterminate: bool) -> ProgressBarProps {
        props.indeterminate = indeterminate;
        props
    }

    /// Builder method for animated mode
    pub fn animated(mut props: ProgressBarProps, animated: bool) -> ProgressBarProps {
        props.animated = animated;
        props
    }

    /// Builder method for orientation
    pub fn with_orientation(
        mut props: ProgressBarProps,
        orientation: ProgressBarOrientation,
    ) -> ProgressBarProps {
        props.orientation = orientation;
        props
    }

    /// Calculate the percentage complete
    fn calculate_percentage(&self, props: &ProgressBarProps) -> f64 {
        if props.indeterminate || !valid_range(props) {
            return 0.0;
        }

        let range = props.max_value - props.min_value;
        let progress = props.value - props.min_value;
        (progress / range * 100.0).clamp(0.0, 100.0)
    }

    /// Format the display text
    fn format_text(&self, props: &ProgressBarProps) -> String {
        if let Some(formatter) = &props.custom_formatter {
            return formatter(props.value, props.min_value, props.max_value);
        }

        if props.indeterminate {
            return "Loading...".to_string();
        }

        let percentage = self.calculate_percentage(props);

        match (props.show_percentage, props.show_value) {
            (true, true) => format!(
                "{:.1}% ({:.1}/{:.1})",
                percentage, props.value, props.max_value
            ),
            (true, false) => format!("{percentage:.1}%"),
            (false, true) => format!("{:.1}/{:.1}", props.value, props.max_value),
            (false, false) => String::new(),
        }
    }

    /// Update animation state
    fn update_animation(&self, props: &ProgressBarProps, state: &mut ProgressBarState) {
        if props.animated || props.indeterminate {
            state.animation_frame = state.animation_frame.wrapping_add(1);
        }

        if props.indeterminate {
            // Update indeterminate position
            state.indeterminate_position += 0.5;
            if state.indeterminate_position >= 100.0 {
                state.indeterminate_position = 0.0;
            }
        }

        if props.pulse {
            // Update pulse direction
            if state.indeterminate_position >= 100.0 {
                state.pulse_direction = false;
            } else if state.indeterminate_position <= 0.0 {
                state.pulse_direction = true;
            }

            if state.pulse_direction {
                state.indeterminate_position += 2.0;
            } else {
                state.indeterminate_position -= 2.0;
            }
        }

        // Check if completed
        let was_completed = state.completed;
        state.completed = !props.indeterminate
            && valid_range(props)
            && live::validate(props).is_ok()
            && props.value >= props.max_value;

        // Trigger completion callback
        if state.completed && !was_completed {
            if let Some(callback) = &props.on_complete {
                callback();
            }
        }

        state.last_value = props.value;
    }
}

mod live;

fn valid_range(props: &ProgressBarProps) -> bool {
    props.value.is_finite()
        && props.min_value.is_finite()
        && props.max_value.is_finite()
        && props.max_value > props.min_value
        && (props.max_value - props.min_value).is_finite()
}

impl Component for ProgressBar {
    type Props = ProgressBarProps;
    type State = ProgressBarState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        let mut state = ProgressBarState::default();
        self.update_animation(props, &mut state);
        state
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveProgress>(live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
        })
    }

    fn handle_event(
        &mut self,
        _event: &Event,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> EventResult {
        // ProgressBar typically doesn't handle events directly
        EventResult::Ignored
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        let old_value = state.last_value;
        let old_frame = state.animation_frame;
        let old_position = state.indeterminate_position;

        self.update_animation(props, state);

        // Re-render if value changed, animation is active, or indeterminate
        old_value != props.value
            || props.animated && old_frame != state.animation_frame
            || props.indeterminate && old_position != state.indeterminate_position
            || props.pulse
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_props() -> ProgressBarProps {
        ProgressBarProps {
            value: 50.0,
            max_value: 100.0,
            min_value: 0.0,
            show_percentage: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_progress_bar_creation() {
        let progress_bar = ProgressBar;
        let props = create_test_props();
        let state = ProgressBarState::default();

        let element = progress_bar.render(&props, &state);
        assert!(matches!(
            element.element_type,
            crate::component::ElementType::Component(_)
        ));
    }

    #[test]
    fn test_percentage_calculation() {
        let progress_bar = ProgressBar;
        let props = create_test_props();

        let percentage = progress_bar.calculate_percentage(&props);
        assert_eq!(percentage, 50.0);

        // Test edge cases
        let mut edge_props = props.clone();
        edge_props.value = 0.0;
        assert_eq!(progress_bar.calculate_percentage(&edge_props), 0.0);

        edge_props.value = 100.0;
        assert_eq!(progress_bar.calculate_percentage(&edge_props), 100.0);

        edge_props.value = 150.0; // Over max
        assert_eq!(progress_bar.calculate_percentage(&edge_props), 100.0);
    }

    #[test]
    fn test_text_formatting() {
        let progress_bar = ProgressBar;
        let props = create_test_props();

        let text = progress_bar.format_text(&props);
        assert_eq!(text, "50.0%");

        // Test with value display
        let mut value_props = props.clone();
        value_props.show_value = true;
        let text = progress_bar.format_text(&value_props);
        assert_eq!(text, "50.0% (50.0/100.0)");

        // Test indeterminate
        let mut indet_props = props.clone();
        indet_props.indeterminate = true;
        let text = progress_bar.format_text(&indet_props);
        assert_eq!(text, "Loading...");
    }

    #[test]
    fn test_bar_creation() {
        let progress_bar = ProgressBar;
        let props = ProgressBarProps {
            value: 50.0,
            width: Some(10),
            ..Default::default()
        };
        let state = ProgressBarState::default();

        let bar: String = (0..10)
            .map(|i| {
                live::bar_char(
                    &props,
                    progress_bar.calculate_percentage(&props) / 100.0,
                    i,
                    10,
                    state.indeterminate_position / 100.0,
                )
                .0
            })
            .collect();
        // Width is in characters, but some Unicode characters might be multi-byte
        // Count actual characters, not bytes
        let char_count = bar.chars().count();
        assert_eq!(char_count, 10);
        assert!(bar.contains('█')); // Should have filled portions
        assert!(bar.contains('▒')); // Should have empty portions
    }

    #[test]
    fn test_indeterminate_bar() {
        let state = ProgressBarState {
            indeterminate_position: 25.0,
            ..Default::default()
        };

        let props = ProgressBarProps {
            indeterminate: true,
            ..Default::default()
        };
        let bar: String = (0..20)
            .map(|i| live::bar_char(&props, 0.0, i, 20, state.indeterminate_position / 100.0).0)
            .collect();
        assert_eq!(bar.chars().count(), 20); // Count characters not bytes
        assert!(bar.contains('█')); // Should have moving segment
        assert!(bar.contains('▒')); // Should have background
    }

    #[test]
    fn test_segmented_bar() {
        let progress_bar = ProgressBar;
        let props = ProgressBarProps {
            value: 30.0,
            segments: Some(10),
            ..Default::default()
        };

        let segments: Vec<_> = (0..10)
            .map(|i| {
                live::bar_char(
                    &props,
                    progress_bar.calculate_percentage(&props) / 100.0,
                    i,
                    10,
                    0.0,
                )
            })
            .collect();
        assert_eq!(segments.len(), 10);
        assert_eq!(segments.iter().filter(|(_, filled)| *filled).count(), 3);
    }

    #[test]
    fn test_animation_update() {
        let progress_bar = ProgressBar;
        let props = ProgressBarProps {
            animated: true,
            indeterminate: true,
            ..Default::default()
        };
        let mut state = ProgressBarState::default();

        let old_frame = state.animation_frame;
        let old_position = state.indeterminate_position;

        progress_bar.update_animation(&props, &mut state);

        assert_ne!(state.animation_frame, old_frame);
        assert_ne!(state.indeterminate_position, old_position);
    }

    #[test]
    fn test_completion_detection() {
        let progress_bar = ProgressBar;
        let props = ProgressBarProps {
            value: 100.0,
            max_value: 100.0,
            ..Default::default()
        };
        let mut state = ProgressBarState {
            completed: false,
            ..Default::default()
        };

        progress_bar.update_animation(&props, &mut state);
        assert!(state.completed);
    }

    #[test]
    fn test_builder_methods() {
        let props = ProgressBar::with_value(ProgressBarProps::default(), 75.0);
        assert_eq!(props.value, 75.0);

        let props = ProgressBar::with_range(props, 0.0, 200.0);
        assert_eq!(props.min_value, 0.0);
        assert_eq!(props.max_value, 200.0);

        let props = ProgressBar::with_label(props, "Loading");
        assert_eq!(props.label, Some("Loading".to_string()));

        let props = ProgressBar::with_color(props, "bg-green");
        assert_eq!(props.color, Some("bg-green".to_string()));

        let props = ProgressBar::indeterminate(props, true);
        assert!(props.indeterminate);

        let props = ProgressBar::animated(props, true);
        assert!(props.animated);

        let props = ProgressBar::with_orientation(props, ProgressBarOrientation::Vertical);
        assert_eq!(props.orientation, ProgressBarOrientation::Vertical);
    }

    #[test]
    fn test_component_update() {
        let mut progress_bar = ProgressBar;
        let props = create_test_props();
        let mut state = ProgressBarState::default();

        // Should update when value changes
        let should_update = progress_bar.update(&props, &mut state);
        assert!(should_update); // First update should always return true

        // Test with animation
        let animated_props = ProgressBarProps {
            animated: true,
            ..props
        };
        let should_update = progress_bar.update(&animated_props, &mut state);
        assert!(should_update); // Animation should cause updates
    }

    #[test]
    fn test_event_handling() {
        let mut progress_bar = ProgressBar;
        let mut props = create_test_props();
        let mut state = ProgressBarState::default();

        let focus_event = crate::event::types::Event::Focus(crate::event::types::FocusEvent {
            kind: crate::event::types::FocusEventKind::Gained,
            timestamp: std::time::Instant::now(),
        });

        let result = progress_bar.handle_event(&focus_event, &mut props, &mut state);
        assert_eq!(result, EventResult::Ignored); // Progress bars don't handle events
    }
}
