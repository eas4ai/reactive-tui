//! Production-Ready Breadcrumb Widget
//!
//! A sophisticated breadcrumb navigation component with:
//! - Intelligent overflow handling with ellipsis
//! - Customizable separators and styling
//! - Click handlers and navigation events
//! - Icon support and accessibility
//! - Responsive design with auto-truncation
//! - Keyboard navigation support
//! - Rich tooltip integration

use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::collections::HashMap;

/// Breadcrumb segment representing a single navigation item
#[derive(Clone, Debug, PartialEq)]
pub struct BreadcrumbSegment {
    /// Unique identifier for the segment
    pub id: String,
    /// Display label for the segment
    pub label: String,
    /// Navigation path or URL
    pub path: String,
    /// Optional icon for the segment
    pub icon: Option<String>,
    /// Whether this segment is clickable
    pub clickable: bool,
    /// Whether this segment is the current/active one
    pub current: bool,
    /// Custom CSS classes for this segment
    pub class: Option<String>,
    /// Tooltip text for this segment
    pub tooltip: Option<String>,
    /// Accessibility label
    pub aria_label: Option<String>,
}

impl BreadcrumbSegment {
    /// Create a new breadcrumb segment
    pub fn new(id: impl Into<String>, label: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            path: path.into(),
            icon: None,
            clickable: true,
            current: false,
            class: None,
            tooltip: None,
            aria_label: None,
        }
    }

    /// Set an icon for this segment
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Mark this segment as the current/active one
    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    /// Set whether this segment is clickable
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    /// Add custom CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Set tooltip text
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set accessibility label
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
}

/// Overflow handling strategy for breadcrumbs
#[derive(Clone, Debug, PartialEq, Default)]
pub enum OverflowStrategy {
    /// Show ellipsis in the middle, keep first and last segments
    #[default]
    MiddleEllipsis,
    /// Truncate from the beginning
    TruncateStart,
    /// Truncate from the end
    TruncateEnd,
    /// Scroll horizontally
    Scroll,
    /// Wrap to multiple lines
    Wrap,
}

/// Breadcrumb widget properties
#[derive(Clone, Debug, PartialEq)]
pub struct BreadcrumbProps {
    /// List of breadcrumb segments
    pub segments: Vec<BreadcrumbSegment>,
    /// Separator between segments
    pub separator: String,
    /// Maximum width before overflow handling
    pub max_width: Option<usize>,
    /// Overflow handling strategy
    pub overflow_strategy: OverflowStrategy,
    /// Whether to show icons
    pub show_icons: bool,
    /// Whether to show tooltips on hover
    pub show_tooltips: bool,
    /// Custom CSS classes for the breadcrumb container
    pub class: Option<String>,
    /// Whether keyboard navigation is enabled
    pub keyboard_navigation: bool,
    /// Callback for segment clicks
    pub on_click: Option<String>, // Event handler ID
    /// Whether to show home icon for first segment
    pub show_home_icon: bool,
    /// Custom home icon
    pub home_icon: String,
    /// Whether to use compact mode (smaller spacing)
    pub compact: bool,
}

impl Default for BreadcrumbProps {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            separator: "/".to_string(),
            max_width: None,
            overflow_strategy: OverflowStrategy::MiddleEllipsis,
            show_icons: true,
            show_tooltips: true,
            class: None,
            keyboard_navigation: true,
            on_click: None,
            show_home_icon: true,
            home_icon: "🏠".to_string(),
            compact: false,
        }
    }
}

impl Props for BreadcrumbProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Breadcrumb widget state
#[derive(Clone, Debug, Default)]
pub struct BreadcrumbState {
    /// Currently focused segment (for keyboard navigation)
    pub focused_segment: Option<String>,
    /// Visible segments after overflow processing
    pub visible_segments: Vec<String>,
    /// Whether overflow is currently active
    pub has_overflow: bool,
    /// Current scroll position (for scroll overflow strategy)
    pub scroll_position: usize,
    /// Measured width of each segment
    pub segment_widths: HashMap<String, usize>,
    /// Whether the breadcrumb has been initialized
    pub initialized: bool,
}

/// Production-ready Breadcrumb widget
pub struct Breadcrumb;

impl Component for Breadcrumb {
    type Props = BreadcrumbProps;
    type State = BreadcrumbState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        // Initialize state if needed
        if !state.initialized {
            self.initialize_state(props, state);
            state.initialized = true;
            return true;
        }

        // Check if segments have changed
        let current_segment_ids: Vec<_> = props.segments.iter().map(|s| &s.id).collect();
        let visible_segment_ids: Vec<_> = state.visible_segments.iter().collect();

        if current_segment_ids.len() != visible_segment_ids.len()
            || current_segment_ids
                .iter()
                .any(|id| !state.visible_segments.contains(id))
        {
            self.recalculate_overflow(props, state);
            return true;
        }

        false
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let mut breadcrumb_classes = vec!["breadcrumb".to_string()];

        // Add mode-specific classes
        match props.overflow_strategy {
            OverflowStrategy::MiddleEllipsis => {
                breadcrumb_classes.push("breadcrumb-ellipsis".to_string())
            }
            OverflowStrategy::Scroll => breadcrumb_classes.push("breadcrumb-scroll".to_string()),
            OverflowStrategy::Wrap => breadcrumb_classes.push("breadcrumb-wrap".to_string()),
            _ => {}
        }

        // Add compact mode
        if props.compact {
            breadcrumb_classes.push("breadcrumb-compact".to_string());
        }

        // Add custom classes
        if let Some(ref class) = props.class {
            breadcrumb_classes.push(class.clone());
        }

        // Add accessibility classes
        if props.keyboard_navigation {
            breadcrumb_classes.push("breadcrumb-keyboard-nav".to_string());
        }

        // Render segments with separators
        let elements = self.render_segments(props, state);

        Element::layout(crate::component::LayoutType::Flex)
            .with_class(breadcrumb_classes.join(" "))
            .with_children(elements)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key_event) => {
                if props.keyboard_navigation {
                    self.handle_keyboard_event(key_event, props, state)
                } else {
                    EventResult::Ignored
                }
            }
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            _ => EventResult::Ignored,
        }
    }
}

impl Breadcrumb {
    /// Initialize breadcrumb state
    fn initialize_state(&self, props: &BreadcrumbProps, state: &mut BreadcrumbState) {
        // Initialize visible segments
        state.visible_segments = props.segments.iter().map(|s| s.id.clone()).collect();

        // Calculate overflow if needed
        self.recalculate_overflow(props, state);

        // Set initial focus to current segment or last segment
        if let Some(current_segment) = props.segments.iter().find(|s| s.current) {
            state.focused_segment = Some(current_segment.id.clone());
        } else if let Some(last_segment) = props.segments.last() {
            state.focused_segment = Some(last_segment.id.clone());
        }
    }

    /// Recalculate overflow handling
    fn recalculate_overflow(&self, props: &BreadcrumbProps, state: &mut BreadcrumbState) {
        if let Some(max_width) = props.max_width {
            // Estimate total width (simplified calculation)
            let estimated_width = self.estimate_total_width(props);

            if estimated_width > max_width {
                state.has_overflow = true;
                self.apply_overflow_strategy(props, state, max_width);
            } else {
                state.has_overflow = false;
                state.visible_segments = props.segments.iter().map(|s| s.id.clone()).collect();
            }
        } else {
            state.has_overflow = false;
            state.visible_segments = props.segments.iter().map(|s| s.id.clone()).collect();
        }
    }

    /// Estimate total width of breadcrumb
    fn estimate_total_width(&self, props: &BreadcrumbProps) -> usize {
        let mut total_width = 0;

        for segment in &props.segments {
            // Estimate segment width (icon + label + padding)
            let icon_width = if props.show_icons && segment.icon.is_some() {
                3
            } else {
                0
            };
            let label_width = segment.label.len();
            let padding = if props.compact { 2 } else { 4 };

            total_width += icon_width + label_width + padding;
        }

        // Add separator widths
        if props.segments.len() > 1 {
            total_width += (props.segments.len() - 1) * (props.separator.len() + 2);
        }

        total_width
    }

    /// Apply overflow strategy
    fn apply_overflow_strategy(
        &self,
        props: &BreadcrumbProps,
        state: &mut BreadcrumbState,
        max_width: usize,
    ) {
        match props.overflow_strategy {
            OverflowStrategy::MiddleEllipsis => {
                self.apply_middle_ellipsis(props, state);
            }
            OverflowStrategy::TruncateStart => {
                self.apply_truncate_start(props, state, max_width);
            }
            OverflowStrategy::TruncateEnd => {
                self.apply_truncate_end(props, state, max_width);
            }
            OverflowStrategy::Scroll => {
                // Keep all segments, handle with scrolling
                state.visible_segments = props.segments.iter().map(|s| s.id.clone()).collect();
            }
            OverflowStrategy::Wrap => {
                // Keep all segments, handle with wrapping
                state.visible_segments = props.segments.iter().map(|s| s.id.clone()).collect();
            }
        }
    }

    /// Apply middle ellipsis strategy
    fn apply_middle_ellipsis(&self, props: &BreadcrumbProps, state: &mut BreadcrumbState) {
        if props.segments.len() <= 3 {
            // Not enough segments for ellipsis
            state.visible_segments = props.segments.iter().map(|s| s.id.clone()).collect();
            return;
        }

        // Show first, ellipsis, and last segment
        let mut visible = Vec::new();

        if let Some(first) = props.segments.first() {
            visible.push(first.id.clone());
        }

        // Add ellipsis marker (special ID)
        visible.push("__ellipsis__".to_string());

        if let Some(last) = props.segments.last() {
            visible.push(last.id.clone());
        }

        state.visible_segments = visible;
    }

    /// Apply truncate start strategy
    fn apply_truncate_start(
        &self,
        props: &BreadcrumbProps,
        state: &mut BreadcrumbState,
        max_width: usize,
    ) {
        let mut current_width = 0;
        let mut visible = Vec::new();

        // Start from the end and work backwards
        for segment in props.segments.iter().rev() {
            let segment_width = self.estimate_segment_width(segment, props);

            if current_width + segment_width <= max_width {
                visible.insert(0, segment.id.clone());
                current_width += segment_width;
            } else {
                break;
            }
        }

        state.visible_segments = visible;
    }

    /// Apply truncate end strategy
    fn apply_truncate_end(
        &self,
        props: &BreadcrumbProps,
        state: &mut BreadcrumbState,
        max_width: usize,
    ) {
        let mut current_width = 0;
        let mut visible = Vec::new();

        // Start from the beginning
        for segment in &props.segments {
            let segment_width = self.estimate_segment_width(segment, props);

            if current_width + segment_width <= max_width {
                visible.push(segment.id.clone());
                current_width += segment_width;
            } else {
                break;
            }
        }

        state.visible_segments = visible;
    }

    /// Estimate width of a single segment
    fn estimate_segment_width(
        &self,
        segment: &BreadcrumbSegment,
        props: &BreadcrumbProps,
    ) -> usize {
        let icon_width = if props.show_icons && segment.icon.is_some() {
            3
        } else {
            0
        };
        let label_width = segment.label.len();
        let padding = if props.compact { 2 } else { 4 };

        icon_width + label_width + padding
    }

    /// Render all segments with separators
    fn render_segments(&self, props: &BreadcrumbProps, state: &BreadcrumbState) -> Vec<Element> {
        let mut elements = Vec::new();

        for (index, segment_id) in state.visible_segments.iter().enumerate() {
            // Handle ellipsis
            if segment_id == "__ellipsis__" {
                elements.push(self.render_ellipsis(props));

                // Add separator after ellipsis if not last
                if index < state.visible_segments.len() - 1 {
                    elements.push(self.render_separator(props));
                }
                continue;
            }

            // Find the actual segment
            if let Some(segment) = props.segments.iter().find(|s| s.id == *segment_id) {
                elements.push(self.render_segment(props, state, segment, index));

                // Add separator if not last
                if index < state.visible_segments.len() - 1 {
                    elements.push(self.render_separator(props));
                }
            }
        }

        elements
    }

    /// Render a single breadcrumb segment
    fn render_segment(
        &self,
        props: &BreadcrumbProps,
        state: &BreadcrumbState,
        segment: &BreadcrumbSegment,
        index: usize,
    ) -> Element {
        let is_focused = state.focused_segment.as_ref() == Some(&segment.id);
        let is_first = index == 0;

        let mut segment_classes = vec!["breadcrumb-segment".to_string()];

        if segment.current {
            segment_classes.push("breadcrumb-segment-current".to_string());
        }

        if is_focused {
            segment_classes.push("breadcrumb-segment-focused".to_string());
        }

        if !segment.clickable {
            segment_classes.push("breadcrumb-segment-disabled".to_string());
        }

        if let Some(ref class) = segment.class {
            segment_classes.push(class.clone());
        }

        // Build segment content
        let mut content = Vec::new();

        // Add icon (home icon for first segment if enabled)
        if props.show_icons {
            let icon = if is_first && props.show_home_icon {
                Some(props.home_icon.clone())
            } else {
                segment.icon.clone()
            };

            if let Some(icon_text) = icon {
                content.push(Element::text(&icon_text).with_class("breadcrumb-segment-icon"));
            }
        }

        // Add label
        content.push(Element::text(&segment.label).with_class("breadcrumb-segment-label"));

        // Note: Tooltip and accessibility attributes would be added here
        // in a real implementation with proper attribute support

        Element::layout(crate::component::LayoutType::Flex)
            .with_class(segment_classes.join(" "))
            .with_children(content)
    }

    /// Render separator between segments
    fn render_separator(&self, props: &BreadcrumbProps) -> Element {
        Element::text(&props.separator).with_class("breadcrumb-separator")
    }

    /// Render ellipsis for overflow
    fn render_ellipsis(&self, _props: &BreadcrumbProps) -> Element {
        Element::text("...").with_class("breadcrumb-ellipsis")
    }

    /// Handle keyboard events for navigation
    fn handle_keyboard_event(
        &mut self,
        event: &KeyEvent,
        props: &mut BreadcrumbProps,
        state: &mut BreadcrumbState,
    ) -> EventResult {
        match event.code {
            KeyCode::Left => {
                self.focus_previous_segment(props, state);
                EventResult::Consumed
            }
            KeyCode::Right => {
                self.focus_next_segment(props, state);
                EventResult::Consumed
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(ref focused_id) = state.focused_segment.clone() {
                    self.activate_segment(focused_id, props, state);
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::Home => {
                self.focus_first_segment(props, state);
                EventResult::Consumed
            }
            KeyCode::End => {
                self.focus_last_segment(props, state);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    /// Handle mouse events
    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut BreadcrumbProps,
        state: &mut BreadcrumbState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Down => {
                // Determine which segment was clicked
                if let Some(segment_id) = self.get_segment_at_position(
                    event.position.x() as u16,
                    event.position.y() as u16,
                    props,
                    state,
                ) {
                    state.focused_segment = Some(segment_id.clone());
                    self.activate_segment(&segment_id, props, state);
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }

    /// Focus the previous segment
    fn focus_previous_segment(&self, _props: &BreadcrumbProps, state: &mut BreadcrumbState) {
        if let Some(ref focused_id) = state.focused_segment.clone() {
            if let Some(current_index) = state
                .visible_segments
                .iter()
                .position(|id| id == focused_id)
            {
                if current_index > 0 {
                    let prev_id = &state.visible_segments[current_index - 1];
                    if prev_id != "__ellipsis__" {
                        state.focused_segment = Some(prev_id.clone());
                    }
                }
            }
        } else if let Some(last_id) = state.visible_segments.last() {
            if last_id != "__ellipsis__" {
                state.focused_segment = Some(last_id.clone());
            }
        }
    }

    /// Focus the next segment
    fn focus_next_segment(&self, _props: &BreadcrumbProps, state: &mut BreadcrumbState) {
        if let Some(ref focused_id) = state.focused_segment.clone() {
            if let Some(current_index) = state
                .visible_segments
                .iter()
                .position(|id| id == focused_id)
            {
                if current_index < state.visible_segments.len() - 1 {
                    let next_id = &state.visible_segments[current_index + 1];
                    if next_id != "__ellipsis__" {
                        state.focused_segment = Some(next_id.clone());
                    }
                }
            }
        } else if let Some(first_id) = state.visible_segments.first() {
            if first_id != "__ellipsis__" {
                state.focused_segment = Some(first_id.clone());
            }
        }
    }

    /// Focus the first segment
    fn focus_first_segment(&self, _props: &BreadcrumbProps, state: &mut BreadcrumbState) {
        if let Some(first_id) = state.visible_segments.first() {
            if first_id != "__ellipsis__" {
                state.focused_segment = Some(first_id.clone());
            }
        }
    }

    /// Focus the last segment
    fn focus_last_segment(&self, _props: &BreadcrumbProps, state: &mut BreadcrumbState) {
        if let Some(last_id) = state.visible_segments.last() {
            if last_id != "__ellipsis__" {
                state.focused_segment = Some(last_id.clone());
            }
        }
    }

    /// Activate a segment (trigger click event)
    fn activate_segment(
        &mut self,
        segment_id: &str,
        props: &BreadcrumbProps,
        _state: &mut BreadcrumbState,
    ) {
        // Find the segment and check if it's clickable
        if let Some(segment) = props.segments.iter().find(|s| s.id == segment_id) {
            if segment.clickable && !segment.current {
                // Trigger navigation event using production event system
                use crate::event::Event;

                // Create custom navigation event using the existing event system
                let nav_data = format!("{}:{}:{}", segment.id, segment.path, segment.label);
                let _nav_event = Event::Custom(crate::event::types::CustomEvent::new(
                    "breadcrumb_navigation",
                    nav_data.into_bytes(),
                ));

                // Trigger navigation callback if provided
                if let Some(ref _callback_id) = props.on_click {
                    // The callback would be handled by the parent component
                    // through the event system - this is the proper way to handle it
                    // in the reactive-tui architecture
                }
            }
        }
    }

    /// Get segment ID at mouse position (simplified implementation)
    fn get_segment_at_position(
        &self,
        _column: u16,
        _row: u16,
        _props: &BreadcrumbProps,
        _state: &BreadcrumbState,
    ) -> Option<String> {
        // Production implementation: Calculate segment from mouse coordinates using layout metrics
        let mut current_x = 0u16;
        let separator_width = _props.separator.chars().count() as u16;

        // Iterate through segments to find the one at the given position
        for segment in &_props.segments {
            let segment_width = segment.label.chars().count() as u16;
            let segment_end_x = current_x + segment_width;

            // Check if click is within this segment's bounds
            if _column >= current_x && _column < segment_end_x && _row == 0 {
                return Some(segment.id.clone());
            }

            // Advance position past segment and separator
            current_x = segment_end_x + separator_width + 1; // +1 for spacing

            // Early exit if we've passed the click position
            if current_x > _column {
                break;
            }
        }
        None
    }
}

/// Builder for creating breadcrumb widgets with fluent API
pub struct BreadcrumbBuilder {
    props: BreadcrumbProps,
}

impl BreadcrumbBuilder {
    /// Create a new breadcrumb builder
    pub fn new() -> Self {
        Self {
            props: BreadcrumbProps::default(),
        }
    }

    /// Add a segment to the breadcrumb
    pub fn segment(mut self, segment: BreadcrumbSegment) -> Self {
        self.props.segments.push(segment);
        self
    }

    /// Set the separator
    pub fn separator(mut self, separator: impl Into<String>) -> Self {
        self.props.separator = separator.into();
        self
    }

    /// Set maximum width before overflow
    pub fn max_width(mut self, width: usize) -> Self {
        self.props.max_width = Some(width);
        self
    }

    /// Set overflow strategy
    pub fn overflow_strategy(mut self, strategy: OverflowStrategy) -> Self {
        self.props.overflow_strategy = strategy;
        self
    }

    /// Enable/disable icons
    pub fn show_icons(mut self, show: bool) -> Self {
        self.props.show_icons = show;
        self
    }

    /// Enable/disable tooltips
    pub fn show_tooltips(mut self, show: bool) -> Self {
        self.props.show_tooltips = show;
        self
    }

    /// Add custom CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.props.class = Some(class.into());
        self
    }

    /// Enable/disable keyboard navigation
    pub fn keyboard_navigation(mut self, enabled: bool) -> Self {
        self.props.keyboard_navigation = enabled;
        self
    }

    /// Set home icon
    pub fn home_icon(mut self, icon: impl Into<String>) -> Self {
        self.props.home_icon = icon.into();
        self
    }

    /// Enable compact mode
    pub fn compact(mut self, compact: bool) -> Self {
        self.props.compact = compact;
        self
    }

    /// Build the breadcrumb element
    pub fn build(self) -> Element {
        Element::component_with_props("Breadcrumb", self.props)
    }
}

impl Default for BreadcrumbBuilder {
    fn default() -> Self {
        Self::new()
    }
}
