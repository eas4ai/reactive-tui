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
use crate::event::types::{KeyCode, KeyEvent, KeyEventKind};
use crate::event::Event;
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
    /// CustomEvent name for navigation. JSON data contains segment_id, path and label.
    /// Without a callback name, App receives breadcrumb_navigation.
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
#[derive(Clone, Debug, Default, PartialEq)]
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

mod live;

impl Component for Breadcrumb {
    type Props = BreadcrumbProps;
    type State = BreadcrumbState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        let mut state = BreadcrumbState::default();
        self.update(props, &mut state);
        state
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        state.visible_segments = props
            .segments
            .iter()
            .map(|segment| segment.id.clone())
            .collect();
        if !props.segments.iter().any(|segment| {
            segment.clickable
                && !segment.current
                && state.focused_segment.as_ref() == Some(&segment.id)
        }) {
            state.focused_segment = props
                .segments
                .iter()
                .rev()
                .find(|segment| segment.clickable && !segment.current)
                .map(|segment| segment.id.clone());
        }
        state.initialized = true;
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveBreadcrumb>(live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
        })
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key) if props.keyboard_navigation => {
                self.handle_keyboard_event(key, props, state)
            }
            _ => EventResult::Ignored,
        }
    }
}

impl Breadcrumb {
    fn handle_keyboard_event(
        &self,
        event: &KeyEvent,
        props: &BreadcrumbProps,
        state: &mut BreadcrumbState,
    ) -> EventResult {
        if event.kind == KeyEventKind::Release {
            return EventResult::Ignored;
        }
        let enabled: Vec<_> = state
            .visible_segments
            .iter()
            .filter(|id| {
                props
                    .segments
                    .iter()
                    .any(|segment| &segment.id == *id && segment.clickable && !segment.current)
            })
            .collect();
        if enabled.is_empty() {
            return EventResult::Ignored;
        }
        let current = enabled
            .iter()
            .position(|id| Some(*id) == state.focused_segment.as_ref())
            .unwrap_or(0);
        let next = match event.code {
            KeyCode::Left => current.saturating_sub(1),
            KeyCode::Right => (current + 1).min(enabled.len() - 1),
            KeyCode::Home => 0,
            KeyCode::End => enabled.len() - 1,
            KeyCode::Enter | KeyCode::Char(' ') => {
                Self::activate_segment(enabled[current], props);
                return EventResult::Consumed;
            }
            _ => return EventResult::Ignored,
        };
        state.focused_segment = Some(enabled[next].clone());
        EventResult::Consumed
    }

    fn activate_segment(id: &str, props: &BreadcrumbProps) -> bool {
        let Some(segment) = props
            .segments
            .iter()
            .find(|segment| segment.id == id && segment.clickable && !segment.current)
        else {
            return false;
        };
        crate::event::notifications::emit(crate::event::CustomEvent::new(
            props.on_click.as_deref().unwrap_or("breadcrumb_navigation"),
            serde_json::json!({"segment_id": segment.id, "path": segment.path, "label": segment.label}).to_string().into_bytes(),
        ));
        true
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

    /// Set the navigation notification name delivered to the App root.
    pub fn on_click(mut self, callback: impl Into<String>) -> Self {
        self.props.on_click = Some(callback.into());
        self
    }

    /// Show the home icon on the original first segment.
    pub fn show_home_icon(mut self, show: bool) -> Self {
        self.props.show_home_icon = show;
        self
    }

    /// Build the breadcrumb element
    pub fn build(self) -> Element {
        Element::typed::<Breadcrumb>(self.props)
    }
}

impl Default for BreadcrumbBuilder {
    fn default() -> Self {
        Self::new()
    }
}
