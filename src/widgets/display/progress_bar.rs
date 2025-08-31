use crate::component::{Component, Element, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::Event;
use std::sync::Arc;

pub type FormatterFn = dyn Fn(f64, f64, f64) -> String + Send + Sync;

/// Props for the ProgressBar component
#[derive(Clone)]
pub struct ProgressBarProps {
    pub value: f64,
    pub max_value: f64,
    pub min_value: f64,
    pub show_percentage: bool,
    pub show_value: bool,
    pub indeterminate: bool,
    pub animated: bool,
    pub label: Option<String>,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub height: u16,
    pub width: Option<u16>,
    pub style: Option<String>,
    pub bar_style: Option<String>,
    pub text_style: Option<String>,
    pub orientation: ProgressBarOrientation,
    pub segments: Option<u16>,
    pub striped: bool,
    pub pulse: bool,
    pub custom_formatter: Option<Arc<FormatterFn>>,
    pub on_complete: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProgressBarOrientation {
    Horizontal,
    Vertical,
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
        // Skip callback comparisons as they can't be compared
    }
}

impl Props for ProgressBarProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// State for the ProgressBar component
#[derive(Debug, Clone, Default)]
pub struct ProgressBarState {
    pub animation_frame: u64,
    pub completed: bool,
    pub last_value: f64,
    pub indeterminate_position: f64,
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
        if props.indeterminate || props.max_value == props.min_value {
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

    /// Create the progress bar visual
    fn create_bar(&self, props: &ProgressBarProps, state: &ProgressBarState) -> String {
        let width = props.width.unwrap_or(40) as usize;

        if props.indeterminate {
            return self.create_indeterminate_bar(width, state);
        }

        let percentage = self.calculate_percentage(props);
        let filled_width = (width as f64 * percentage / 100.0) as usize;
        let empty_width = width.saturating_sub(filled_width);

        let fill_char = if props.striped { '▌' } else { '█' };
        let empty_char = if props.striped { '░' } else { '▒' };

        let mut bar = String::new();

        // Filled portion
        for _ in 0..filled_width {
            bar.push(fill_char);
        }

        // Empty portion
        for _ in 0..empty_width {
            bar.push(empty_char);
        }

        bar
    }

    /// Create indeterminate progress bar animation
    fn create_indeterminate_bar(&self, width: usize, state: &ProgressBarState) -> String {
        let mut bar = vec!['▒'; width];
        let segment_width = (width / 4).max(1); // Ensure at least 1 character

        // Calculate position based on animation frame
        let position = (state.indeterminate_position as usize) % (width + segment_width);

        // Fill the moving segment
        for i in 0..segment_width {
            let pos = position.saturating_sub(segment_width).saturating_add(i);
            if pos < width {
                bar[pos] = '█';
            }
        }

        bar.into_iter().collect()
    }

    /// Create segmented progress bar
    fn create_segmented_bar(&self, props: &ProgressBarProps) -> Vec<Element> {
        let segments = props.segments.unwrap_or(10) as usize;
        let percentage = self.calculate_percentage(props);
        let filled_segments = (segments as f64 * percentage / 100.0) as usize;

        let mut elements = Vec::new();

        for i in 0..segments {
            let is_filled = i < filled_segments;
            let segment_char = if is_filled { '█' } else { '▒' };

            let mut segment_style = String::new();
            if is_filled {
                if let Some(color) = &props.color {
                    segment_style.push_str(color);
                }
            } else if let Some(bg_color) = &props.background_color {
                segment_style.push_str(bg_color);
            }

            elements.push(
                Element::text(segment_char.to_string())
                    .with_class(&segment_style)
                    .with_key(format!("segment-{i}")),
            );
        }

        elements
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
            && props.value >= props.max_value
            && props.max_value > props.min_value;

        // Trigger completion callback
        if state.completed && !was_completed {
            if let Some(callback) = &props.on_complete {
                callback();
            }
        }

        state.last_value = props.value;
    }
}

impl Component for ProgressBar {
    type Props = ProgressBarProps;
    type State = ProgressBarState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        // Create main container
        let mut container_class = String::new();
        if let Some(style) = &props.style {
            container_class.push_str(style);
        }

        let mut children = Vec::new();

        // Add label if present
        if let Some(label) = &props.label {
            let mut label_style = String::new();
            if let Some(text_style) = &props.text_style {
                label_style.push_str(text_style);
            }

            children.push(
                Element::text(label)
                    .with_class(&label_style)
                    .with_key("label"),
            );
        }

        // Create progress bar container
        let orientation_class = match props.orientation {
            ProgressBarOrientation::Horizontal => "flex-row",
            ProgressBarOrientation::Vertical => "flex-col",
        };

        // Progress bar content
        let progress_children = if let Some(_segments) = props.segments {
            self.create_segmented_bar(props)
        } else {
            let bar_text = self.create_bar(props, state);

            let mut bar_style = String::new();
            if let Some(color) = &props.color {
                bar_style.push(' ');
                bar_style.push_str(color);
            }
            if let Some(style) = &props.bar_style {
                bar_style.push(' ');
                bar_style.push_str(style);
            }

            vec![Element::text(&bar_text)
                .with_class(&bar_style)
                .with_key("progress-bar")]
        };

        let progress_bar = Element::layout(LayoutType::Flex)
            .with_class(orientation_class)
            .with_children(progress_children)
            .with_key("progress-container");

        children.push(progress_bar);

        // Add text if should be shown
        let text = self.format_text(props);
        if !text.is_empty() {
            let mut text_style = String::new();
            if let Some(style) = &props.text_style {
                text_style.push_str(style);
            }

            children.push(
                Element::text(&text)
                    .with_class(&text_style)
                    .with_key("progress-text"),
            );
        }

        Element::layout(LayoutType::Flex)
            .with_class(format!("flex-col {container_class}"))
            .with_children(children)
            .with_key("progress-bar-component")
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
        let progress_bar = ProgressBar::default();
        let props = create_test_props();
        let state = ProgressBarState::default();

        let element = progress_bar.render(&props, &state);
        // ProgressBar renders as a flex layout container
        assert_eq!(
            element.element_type,
            crate::component::ElementType::Layout(crate::component::LayoutType::Flex)
        );
    }

    #[test]
    fn test_percentage_calculation() {
        let progress_bar = ProgressBar::default();
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
        let progress_bar = ProgressBar::default();
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
        let progress_bar = ProgressBar::default();
        let props = ProgressBarProps {
            value: 50.0,
            width: Some(10),
            ..Default::default()
        };
        let state = ProgressBarState::default();

        let bar = progress_bar.create_bar(&props, &state);
        // Width is in characters, but some Unicode characters might be multi-byte
        // Count actual characters, not bytes
        let char_count = bar.chars().count();
        assert_eq!(char_count, 10);
        assert!(bar.contains('█')); // Should have filled portions
        assert!(bar.contains('▒')); // Should have empty portions
    }

    #[test]
    fn test_indeterminate_bar() {
        let progress_bar = ProgressBar::default();
        let state = ProgressBarState {
            indeterminate_position: 25.0,
            ..Default::default()
        };

        let bar = progress_bar.create_indeterminate_bar(20, &state);
        assert_eq!(bar.chars().count(), 20); // Count characters not bytes
        assert!(bar.contains('█')); // Should have moving segment
        assert!(bar.contains('▒')); // Should have background
    }

    #[test]
    fn test_segmented_bar() {
        let progress_bar = ProgressBar::default();
        let props = ProgressBarProps {
            value: 30.0,
            segments: Some(10),
            ..Default::default()
        };

        let segments = progress_bar.create_segmented_bar(&props);
        assert_eq!(segments.len(), 10);
        // 30% of 10 segments = 3 filled segments
    }

    #[test]
    fn test_animation_update() {
        let progress_bar = ProgressBar::default();
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
        let progress_bar = ProgressBar::default();
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
        let mut progress_bar = ProgressBar::default();
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
        let mut progress_bar = ProgressBar::default();
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
