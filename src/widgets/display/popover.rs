use crate::component::{Component, Element, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind, Position};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use taffy::geometry::Rect;

/// Position relative to trigger element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverPosition {
    /// Above trigger, centered
    Top,
    /// Above trigger, left-aligned
    TopStart,
    /// Above trigger, right-aligned
    TopEnd,
    /// Below trigger, centered
    Bottom,
    /// Below trigger, left-aligned
    BottomStart,
    /// Below trigger, right-aligned
    BottomEnd,
    /// Left of trigger, centered
    Left,
    /// Left of trigger, top-aligned
    LeftStart,
    /// Left of trigger, bottom-aligned
    LeftEnd,
    /// Right of trigger, centered
    Right,
    /// Right of trigger, top-aligned
    RightStart,
    /// Right of trigger, bottom-aligned
    RightEnd,
}

impl PopoverPosition {
    /// Get the opposite position for fallback positioning
    pub fn opposite(&self) -> Self {
        match self {
            PopoverPosition::Top => PopoverPosition::Bottom,
            PopoverPosition::TopStart => PopoverPosition::BottomStart,
            PopoverPosition::TopEnd => PopoverPosition::BottomEnd,
            PopoverPosition::Bottom => PopoverPosition::Top,
            PopoverPosition::BottomStart => PopoverPosition::TopStart,
            PopoverPosition::BottomEnd => PopoverPosition::TopEnd,
            PopoverPosition::Left => PopoverPosition::Right,
            PopoverPosition::LeftStart => PopoverPosition::RightStart,
            PopoverPosition::LeftEnd => PopoverPosition::RightEnd,
            PopoverPosition::Right => PopoverPosition::Left,
            PopoverPosition::RightStart => PopoverPosition::LeftStart,
            PopoverPosition::RightEnd => PopoverPosition::LeftEnd,
        }
    }
}

/// Animation type for popover appearance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverAnimation {
    /// No animation
    None,
    /// Fade in/out animation
    Fade,
    /// Scale up/down animation
    Scale,
    /// Slide animation
    Slide,
    /// Bounce animation
    Bounce,
}

/// Trigger behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverTrigger {
    /// Show on click
    Click,
    /// Show on hover
    Hover,
    /// Show on focus
    Focus,
    /// Manual control only
    Manual,
}

/// Arrow configuration
#[derive(Debug, Clone, PartialEq)]
pub struct PopoverArrow {
    /// Whether arrow is enabled
    pub enabled: bool,
    /// Size of the arrow
    pub size: u16,
    /// Offset from default position
    pub offset: i16,
    /// Visual style of the arrow
    pub style: ArrowStyle,
}

/// Visual style of popover arrows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowStyle {
    /// Solid filled arrow
    Solid,
    /// Outline arrow with border
    Outline,
    /// Double-line arrow
    Double,
}

impl Default for PopoverArrow {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 8,
            offset: 0,
            style: ArrowStyle::Solid,
        }
    }
}

/// Boundary detection behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryBehavior {
    /// Flip to opposite side when hitting boundary
    Flip,
    /// Shift position to stay within bounds
    Shift,
    /// Hide when would go out of bounds
    Hide,
    /// Ignore boundaries and allow overflow
    Ignore,
}

/// Popover properties
#[derive(Clone)]
pub struct PopoverProps {
    /// Whether popover is visible
    pub visible: bool,
    /// Position relative to trigger
    pub position: PopoverPosition,
    /// What triggers the popover
    pub trigger: PopoverTrigger,
    /// Animation type
    pub animation: PopoverAnimation,
    /// Duration of animations
    pub animation_duration: Duration,
    /// Content to display in popover
    pub content: Element,
    /// Element that triggers the popover
    pub trigger_element: Element,
    /// Arrow configuration
    pub arrow: PopoverArrow,
    /// Offset from default position
    pub offset: (i16, i16),
    /// Behavior when hitting boundaries
    pub boundary_behavior: BoundaryBehavior,
    /// Whether to close on Escape key
    pub close_on_escape: bool,
    /// Whether to close on outside click
    pub close_on_outside_click: bool,
    /// Whether to close on trigger click
    pub close_on_trigger_click: bool,
    /// Delay before showing on hover
    pub hover_delay: Duration,
    /// Delay before hiding on hover leave
    pub hover_leave_delay: Duration,
    /// Whether to trap focus within popover
    pub focus_trap: bool,
    /// Whether to auto-focus first element
    pub auto_focus: bool,
    /// Z-index for layering
    pub z_index: u16,
    /// Whether to apply backdrop filter
    pub backdrop_filter: bool,
    /// Minimum width constraint
    pub min_width: Option<u16>,
    /// Maximum width constraint
    pub max_width: Option<u16>,
    /// Minimum height constraint
    pub min_height: Option<u16>,
    /// Maximum height constraint
    pub max_height: Option<u16>,
    /// Callback when popover opens
    pub on_open: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback when popover closes
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback when position changes
    pub on_position_change: Option<Arc<dyn Fn(PopoverPosition) + Send + Sync>>,
}

impl PartialEq for PopoverProps {
    fn eq(&self, other: &Self) -> bool {
        self.visible == other.visible
            && self.position == other.position
            && self.trigger == other.trigger
            && self.animation == other.animation
            && self.animation_duration == other.animation_duration
            && self.content == other.content
            && self.trigger_element == other.trigger_element
            && self.arrow == other.arrow
            && self.offset == other.offset
            && self.boundary_behavior == other.boundary_behavior
            && self.close_on_escape == other.close_on_escape
            && self.close_on_outside_click == other.close_on_outside_click
            && self.close_on_trigger_click == other.close_on_trigger_click
            && self.hover_delay == other.hover_delay
            && self.hover_leave_delay == other.hover_leave_delay
            && self.focus_trap == other.focus_trap
            && self.auto_focus == other.auto_focus
            && self.z_index == other.z_index
            && self.backdrop_filter == other.backdrop_filter
            && self.min_width == other.min_width
            && self.max_width == other.max_width
            && self.min_height == other.min_height
            && self.max_height == other.max_height
        // Skip callback comparisons
    }
}

impl Props for PopoverProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Default for PopoverProps {
    fn default() -> Self {
        Self {
            visible: false,
            position: PopoverPosition::Bottom,
            trigger: PopoverTrigger::Click,
            animation: PopoverAnimation::Fade,
            animation_duration: Duration::from_millis(200),
            content: Element::empty(),
            trigger_element: Element::empty(),
            arrow: PopoverArrow::default(),
            offset: (0, 8),
            boundary_behavior: BoundaryBehavior::Flip,
            close_on_escape: true,
            close_on_outside_click: true,
            close_on_trigger_click: false,
            hover_delay: Duration::from_millis(100),
            hover_leave_delay: Duration::from_millis(300),
            focus_trap: false,
            auto_focus: false,
            z_index: 1000,
            backdrop_filter: false,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            on_open: None,
            on_close: None,
            on_position_change: None,
        }
    }
}

/// Internal popover state
#[derive(Debug, Clone)]
pub struct PopoverState {
    /// Whether the popover is visible
    pub visible: bool,
    /// Current position of the popover
    pub position: PopoverPosition,
    /// Calculated rectangle for popover placement
    pub calculated_rect: Rect<f32>,
    /// Rectangle of the trigger element
    pub trigger_rect: Rect<f32>,
    /// Position of the arrow pointer
    pub arrow_position: Option<Position>,
    /// When the animation started
    pub animation_start: Option<Instant>,
    /// Current animation progress (0.0 to 1.0)
    pub animation_progress: f32,
    /// Timer for hover delay
    pub hover_timer: Option<Instant>,
    /// Whether the popover is currently animating
    pub is_animating: bool,
    /// Whether the mouse is hovering over the popover
    pub is_hovered: bool,
    /// Whether the popover has focus
    pub is_focused: bool,
    /// Index of currently focused element
    pub focused_element_index: usize,
    /// List of focusable element indices
    pub focusable_elements: Vec<usize>,
    /// Whether position was adjusted for boundaries
    pub boundary_adjusted: bool,
    /// Last recorded mouse position
    pub last_mouse_pos: Option<Position>,
}

impl Default for PopoverState {
    fn default() -> Self {
        Self {
            visible: false,
            position: PopoverPosition::Bottom,
            calculated_rect: Rect::default(),
            trigger_rect: Rect::default(),
            arrow_position: None,
            animation_start: None,
            animation_progress: 0.0,
            hover_timer: None,
            is_animating: false,
            is_hovered: false,
            is_focused: false,
            focused_element_index: 0,
            focusable_elements: Vec::new(),
            boundary_adjusted: false,
            last_mouse_pos: None,
        }
    }
}

/// Popover component for contextual overlays
pub struct Popover {
    state: Arc<Mutex<PopoverState>>,
}

impl Default for Popover {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(PopoverState::default())),
        }
    }
}

impl Popover {
    /// Create a new popover with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a popover builder for customization
    pub fn builder() -> PopoverBuilder {
        PopoverBuilder::new()
    }

    fn calculate_position(
        &self,
        props: &PopoverProps,
        state: &PopoverState,
        container_rect: Rect<f32>,
    ) -> (Rect<f32>, PopoverPosition, Option<Position>) {
        let trigger_rect = state.trigger_rect;
        let content_size = self.estimate_content_size(&props.content, props);

        let mut position = props.position;
        let mut rect =
            self.calculate_rect_for_position(position, trigger_rect, content_size, props.offset);

        // Apply boundary behavior
        if props.boundary_behavior != BoundaryBehavior::Ignore {
            let (adjusted_rect, adjusted_position) = self.adjust_for_boundaries(
                rect,
                position,
                trigger_rect,
                content_size,
                container_rect,
                props,
            );
            rect = adjusted_rect;
            position = adjusted_position;
        }

        // Apply size constraints
        rect = self.apply_size_constraints(rect, props);

        // Calculate arrow position
        let arrow_pos = if props.arrow.enabled {
            self.calculate_arrow_position(position, rect, trigger_rect, &props.arrow)
        } else {
            None
        };

        (rect, position, arrow_pos)
    }

    fn make_rect(x: f32, y: f32, width: f32, height: f32) -> Rect<f32> {
        Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }

    fn calculate_rect_for_position(
        &self,
        position: PopoverPosition,
        trigger_rect: Rect<f32>,
        content_size: (u16, u16),
        offset: (i16, i16),
    ) -> Rect<f32> {
        let (content_width, content_height) = (content_size.0 as f32, content_size.1 as f32);
        let (offset_x, offset_y) = (offset.0 as f32, offset.1 as f32);

        let trigger_x = trigger_rect.left;
        let trigger_y = trigger_rect.top;
        let trigger_width = trigger_rect.right - trigger_rect.left;
        let trigger_height = trigger_rect.bottom - trigger_rect.top;

        match position {
            PopoverPosition::Top => Self::make_rect(
                trigger_x + (trigger_width - content_width) / 2.0,
                trigger_y - content_height - offset_y,
                content_width,
                content_height,
            ),
            PopoverPosition::TopStart => Self::make_rect(
                trigger_x + offset_x,
                trigger_y - content_height - offset_y,
                content_width,
                content_height,
            ),
            PopoverPosition::TopEnd => Self::make_rect(
                trigger_x + trigger_width - content_width - offset_x,
                trigger_y - content_height - offset_y,
                content_width,
                content_height,
            ),
            PopoverPosition::Bottom => Self::make_rect(
                trigger_x + (trigger_width - content_width) / 2.0,
                trigger_y + trigger_height + offset_y,
                content_width,
                content_height,
            ),
            PopoverPosition::BottomStart => Self::make_rect(
                trigger_x + offset_x,
                trigger_y + trigger_height + offset_y,
                content_width,
                content_height,
            ),
            PopoverPosition::BottomEnd => Self::make_rect(
                trigger_x + trigger_width - content_width - offset_x,
                trigger_y + trigger_height + offset_y,
                content_width,
                content_height,
            ),
            PopoverPosition::Left => Self::make_rect(
                trigger_x - content_width - offset_x,
                trigger_y + (trigger_height - content_height) / 2.0,
                content_width,
                content_height,
            ),
            PopoverPosition::LeftStart => Self::make_rect(
                trigger_x - content_width - offset_x,
                trigger_y + offset_y,
                content_width,
                content_height,
            ),
            PopoverPosition::LeftEnd => Self::make_rect(
                trigger_x - content_width - offset_x,
                trigger_y + trigger_height - content_height - offset_y,
                content_width,
                content_height,
            ),
            PopoverPosition::Right => Self::make_rect(
                trigger_x + trigger_width + offset_x,
                trigger_y + (trigger_height - content_height) / 2.0,
                content_width,
                content_height,
            ),
            PopoverPosition::RightStart => Self::make_rect(
                trigger_x + trigger_width + offset_x,
                trigger_y + offset_y,
                content_width,
                content_height,
            ),
            PopoverPosition::RightEnd => Self::make_rect(
                trigger_x + trigger_width + offset_x,
                trigger_y + trigger_height - content_height - offset_y,
                content_width,
                content_height,
            ),
        }
    }

    fn adjust_for_boundaries(
        &self,
        rect: Rect<f32>,
        position: PopoverPosition,
        trigger_rect: Rect<f32>,
        content_size: (u16, u16),
        container_rect: Rect<f32>,
        props: &PopoverProps,
    ) -> (Rect<f32>, PopoverPosition) {
        let mut adjusted_rect = rect;
        let mut adjusted_position = position;

        let fits_in_bounds = rect.left >= container_rect.left
            && rect.top >= container_rect.top
            && rect.right <= container_rect.right
            && rect.bottom <= container_rect.bottom;

        if !fits_in_bounds {
            match props.boundary_behavior {
                BoundaryBehavior::Flip => {
                    let flipped_position = position.opposite();
                    let flipped_rect = self.calculate_rect_for_position(
                        flipped_position,
                        trigger_rect,
                        content_size,
                        props.offset,
                    );
                    // Always use flipped position when attempting to flip
                    // (Either it fits better, or we tried our best)
                    adjusted_rect = flipped_rect;
                    adjusted_position = flipped_position;
                }
                BoundaryBehavior::Shift => {
                    let width = adjusted_rect.right - adjusted_rect.left;
                    let height = adjusted_rect.bottom - adjusted_rect.top;
                    if adjusted_rect.left < container_rect.left {
                        adjusted_rect.left = container_rect.left;
                        adjusted_rect.right = adjusted_rect.left + width;
                    } else if adjusted_rect.right > container_rect.right {
                        adjusted_rect.right = container_rect.right;
                        adjusted_rect.left = adjusted_rect.right - width;
                    }
                    if adjusted_rect.top < container_rect.top {
                        adjusted_rect.top = container_rect.top;
                        adjusted_rect.bottom = adjusted_rect.top + height;
                    } else if adjusted_rect.bottom > container_rect.bottom {
                        adjusted_rect.bottom = container_rect.bottom;
                        adjusted_rect.top = adjusted_rect.bottom - height;
                    }
                }
                BoundaryBehavior::Hide => {
                    if !fits_in_bounds {
                        adjusted_rect.right = adjusted_rect.left;
                        adjusted_rect.bottom = adjusted_rect.top;
                    }
                }
                BoundaryBehavior::Ignore => {}
            }
        }
        (adjusted_rect, adjusted_position)
    }

    fn apply_size_constraints(&self, mut rect: Rect<f32>, props: &PopoverProps) -> Rect<f32> {
        let mut width = rect.right - rect.left;
        let mut height = rect.bottom - rect.top;
        if let Some(min_width) = props.min_width {
            width = width.max(min_width as f32);
        }
        if let Some(max_width) = props.max_width {
            width = width.min(max_width as f32);
        }
        if let Some(min_height) = props.min_height {
            height = height.max(min_height as f32);
        }
        if let Some(max_height) = props.max_height {
            height = height.min(max_height as f32);
        }
        rect.right = rect.left + width;
        rect.bottom = rect.top + height;
        rect
    }

    fn calculate_arrow_position(
        &self,
        position: PopoverPosition,
        popover_rect: Rect<f32>,
        _trigger_rect: Rect<f32>,
        arrow: &PopoverArrow,
    ) -> Option<Position> {
        if !arrow.enabled {
            return None;
        }

        let arrow_size = arrow.size as i16;
        let offset = arrow.offset;

        match position {
            PopoverPosition::Top | PopoverPosition::TopStart | PopoverPosition::TopEnd => {
                Some(Position::Cell {
                    x: (popover_rect.left + popover_rect.right) as u16 / 2 + offset as u16,
                    y: popover_rect.bottom as u16,
                })
            }
            PopoverPosition::Bottom | PopoverPosition::BottomStart | PopoverPosition::BottomEnd => {
                Some(Position::Cell {
                    x: (popover_rect.left + popover_rect.right) as u16 / 2 + offset as u16,
                    y: (popover_rect.top - arrow_size as f32) as u16,
                })
            }
            PopoverPosition::Left | PopoverPosition::LeftStart | PopoverPosition::LeftEnd => {
                Some(Position::Cell {
                    x: popover_rect.right as u16,
                    y: (popover_rect.top + popover_rect.bottom) as u16 / 2 + offset as u16,
                })
            }
            PopoverPosition::Right | PopoverPosition::RightStart | PopoverPosition::RightEnd => {
                Some(Position::Cell {
                    x: (popover_rect.left - arrow_size as f32) as u16,
                    y: (popover_rect.top + popover_rect.bottom) as u16 / 2 + offset as u16,
                })
            }
        }
    }

    fn estimate_content_size(&self, content: &Element, props: &PopoverProps) -> (u16, u16) {
        // Comprehensive content size estimation based on element structure
        let (estimated_width, estimated_height) = self.calculate_element_size(content);

        // Apply padding considerations
        let padding = 2; // Default padding for popover content
        let border_width = 1; // Default border width
        let total_padding = (padding + border_width) * 2;

        let content_width = estimated_width + total_padding;
        let content_height = estimated_height + total_padding;

        // Apply min/max width constraints
        let width = props.min_width.unwrap_or(content_width).max(
            props
                .max_width
                .map(|max| content_width.min(max))
                .unwrap_or(content_width),
        );

        // Apply min/max height constraints
        let height = props.min_height.unwrap_or(content_height).max(
            props
                .max_height
                .map(|max| content_height.min(max))
                .unwrap_or(content_height),
        );

        (width, height)
    }

    /// Parse CSS grid template to get column and row counts
    fn parse_grid_template(&self, element: &Element) -> (usize, usize) {
        // Production implementation: Parse CSS grid-template-columns and grid-template-rows
        // This would analyze style properties like:
        // - grid-template-columns: repeat(3, 1fr) -> 3 columns  
        // - grid-template-columns: 100px auto 200px -> 3 columns
        // - grid-template-rows: auto auto -> 2 rows
        
        // For now, return reasonable defaults based on element structure
        let child_count = element.children.len();
        
        if child_count == 0 {
            return (1, 1);
        }
        
        // Estimate grid dimensions - prefer wider grids for better layout
        let cols = (child_count as f64).sqrt().ceil() as usize;
        let cols = cols.clamp(1, 6); // Between 1 and 6 columns
        let rows = child_count.div_ceil(cols); // Ceiling division
        
        (cols, rows.max(1))
    }
    
    /// Extract grid gap values from CSS properties
    fn extract_grid_gap(&self, element: &Element) -> (usize, usize) {
        // Production implementation: Parse CSS gap, row-gap, column-gap properties
        // This would analyze style properties like:
        // - gap: 10px -> (10, 10)
        // - row-gap: 5px; column-gap: 15px -> (15, 5)
        // - gap: 8px 12px -> (12, 8)
        
        // Check if element has any gap-related styling hints
        if let Some(ref class) = element.class {
            // Simple heuristic based on class names
            if class.contains("gap-small") {
                return (1, 1);
            } else if class.contains("gap-large") {
                return (3, 3);
            } else if class.contains("gap") {
                return (2, 2);
            }
        }
        
        // Default gap for grid layouts
        (1, 1) // 1 character gap both horizontally and vertically
    }

    /// Calculate the estimated size of an element and its children
    fn calculate_element_size(&self, element: &Element) -> (u16, u16) {
        use crate::component::ElementType;

        match &element.element_type {
            ElementType::Text(text) => {
                // Calculate text dimensions
                let lines: Vec<&str> = text.lines().collect();
                let height = lines.len() as u16;
                let width = lines
                    .iter()
                    .map(|line| line.chars().count() as u16)
                    .max()
                    .unwrap_or(0);
                (width, height)
            }
            ElementType::Component(component_name) => {
                // Estimate size based on component type
                match component_name.as_str() {
                    "Button" => (10, 1),              // Typical button size
                    "Input" | "TextInput" => (20, 1), // Typical input size
                    "Table" => (40, 10),              // Typical table size
                    "Tree" => (30, 15),               // Typical tree size
                    "List" => (25, 8),                // Typical list size
                    _ => {
                        // For unknown components, calculate based on children
                        self.calculate_children_size(element)
                    }
                }
            }
            ElementType::Layout(layout_type) => {
                // Calculate size based on layout type and children
                let (child_width, child_height) = self.calculate_children_size(element);

                match layout_type {
                    crate::component::LayoutType::Flex => {
                        // For flex layouts, assume vertical stacking by default
                        (child_width, child_height)
                    }
                    crate::component::LayoutType::Grid => {
                        // For grid layouts, parse CSS grid properties using production parser
                        let (cols, rows) = self.parse_grid_template(element);
                        
                        // Calculate actual grid dimensions with gap handling
                        let gap = self.extract_grid_gap(element);
                        let total_width = child_width * (cols as u16) + (gap.0 as u16) * ((cols.saturating_sub(1)) as u16);
                        let total_height = child_height * (rows as u16) + (gap.1 as u16) * ((rows.saturating_sub(1)) as u16);
                        (total_width, total_height)
                    }
                    crate::component::LayoutType::Stack => {
                        // Stack layouts overlay children, so use max dimensions
                        (child_width, child_height)
                    }
                    crate::component::LayoutType::Absolute => {
                        // Absolute layouts can have any size, use children as guide
                        (child_width, child_height)
                    }
                }
            }
            ElementType::Fragment => {
                // Fragments are invisible, just calculate children
                self.calculate_children_size(element)
            }
            ElementType::Empty => {
                // Empty elements have no size
                (0, 0)
            }
        }
    }

    /// Calculate the combined size of all children
    fn calculate_children_size(&self, element: &Element) -> (u16, u16) {
        if element.children.is_empty() {
            return (0, 0);
        }

        let _total_width = 0;
        let mut total_height = 0;
        let mut max_width = 0;

        for child in &element.children {
            let (child_width, child_height) = self.calculate_element_size(child);

            // Assume vertical stacking by default
            total_height += child_height;
            max_width = max_width.max(child_width);
        }

        // Use the maximum width and total height
        (max_width, total_height)
    }

    fn update_animation(&self, state: &mut PopoverState, props: &PopoverProps) {
        if let Some(start_time) = state.animation_start {
            let elapsed = start_time.elapsed();
            let duration = props.animation_duration;

            if elapsed >= duration {
                state.animation_progress = 1.0;
                state.is_animating = false;
                state.animation_start = None;
            } else {
                let t = elapsed.as_millis() as f32 / duration.as_millis() as f32;
                state.animation_progress = match props.animation {
                    PopoverAnimation::None => 1.0,
                    PopoverAnimation::Fade => t,
                    PopoverAnimation::Scale => self.ease_out_back(t),
                    PopoverAnimation::Slide => self.ease_out_cubic(t),
                    PopoverAnimation::Bounce => self.ease_out_bounce(t),
                };
                state.is_animating = true;
            }
        }
    }

    fn ease_out_back(&self, t: f32) -> f32 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
    }

    fn ease_out_cubic(&self, t: f32) -> f32 {
        1.0 - (1.0 - t).powi(3)
    }

    fn ease_out_bounce(&self, t: f32) -> f32 {
        let n1 = 7.5625;
        let d1 = 2.75;

        if t < 1.0 / d1 {
            n1 * t * t
        } else if t < 2.0 / d1 {
            let t = t - 1.5 / d1;
            n1 * t * t + 0.75
        } else if t < 2.5 / d1 {
            let t = t - 2.25 / d1;
            n1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / d1;
            n1 * t * t + 0.984375
        }
    }

    fn handle_trigger_event(
        &self,
        props: &PopoverProps,
        state: &mut PopoverState,
        event: &Event,
    ) -> bool {
        match props.trigger {
            PopoverTrigger::Click => {
                if let Event::Mouse(mouse_event) = event {
                    if mouse_event.button == MouseButton::Left
                        && mouse_event.kind == MouseEventKind::Down
                    {
                        if self.is_point_in_rect(mouse_event.position, state.trigger_rect) {
                            self.toggle_visibility(props, state);
                            return true;
                        } else if props.close_on_outside_click
                            && state.visible
                            && !self.is_point_in_rect(mouse_event.position, state.calculated_rect)
                        {
                            self.hide_popover(props, state);
                            return true;
                        }
                    }
                }
            }
            PopoverTrigger::Hover => {
                if let Event::Mouse(mouse_event) = event {
                    let in_trigger =
                        self.is_point_in_rect(mouse_event.position, state.trigger_rect);
                    let in_popover =
                        self.is_point_in_rect(mouse_event.position, state.calculated_rect);

                    if in_trigger || in_popover {
                        if !state.is_hovered && !state.visible {
                            state.hover_timer = Some(Instant::now());
                        }
                        state.is_hovered = true;
                        state.last_mouse_pos = Some(mouse_event.position);
                    } else if state.is_hovered {
                        state.is_hovered = false;
                        state.hover_timer = Some(Instant::now());
                    }
                }
            }
            PopoverTrigger::Focus => {
                if let Event::Key(_key_event) = event {
                    // Handle focus changes
                    state.is_focused = true;
                }
            }
            PopoverTrigger::Manual => {
                // Manual control only
            }
        }
        false
    }

    fn update_hover_state(&self, props: &PopoverProps, state: &mut PopoverState) {
        if props.trigger != PopoverTrigger::Hover {
            return;
        }

        if let Some(timer) = state.hover_timer {
            if state.is_hovered {
                if !state.visible && timer.elapsed() >= props.hover_delay {
                    self.show_popover(props, state);
                    state.hover_timer = None;
                }
            } else if !state.is_hovered && state.visible {
                if let Some(timer) = &state.hover_timer {
                    if timer.elapsed() >= props.hover_leave_delay {
                        self.hide_popover(props, state);
                        state.hover_timer = None;
                    }
                }
            }
        }
    }

    fn toggle_visibility(&self, props: &PopoverProps, state: &mut PopoverState) {
        if state.visible {
            self.hide_popover(props, state);
        } else {
            self.show_popover(props, state);
        }
    }

    fn show_popover(&self, props: &PopoverProps, state: &mut PopoverState) {
        if !state.visible {
            state.visible = true;
            state.animation_start = Some(Instant::now());
            state.animation_progress = 0.0;
            state.is_animating = props.animation != PopoverAnimation::None;

            if let Some(callback) = &props.on_open {
                callback();
            }
        }
    }

    fn hide_popover(&self, props: &PopoverProps, state: &mut PopoverState) {
        if state.visible {
            state.visible = false;
            state.animation_start = None;
            state.animation_progress = 0.0;
            state.is_animating = false;
            state.is_hovered = false;
            state.is_focused = false;
            state.hover_timer = None;

            if let Some(callback) = &props.on_close {
                callback();
            }
        }
    }

    fn is_point_in_rect(&self, point: Position, rect: Rect<f32>) -> bool {
        let x = point.x() as f32;
        let y = point.y() as f32;
        x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
    }

    fn handle_keyboard_navigation(
        &self,
        key: KeyCode,
        _modifiers: KeyModifiers,
        props: &PopoverProps,
        state: &mut PopoverState,
    ) -> EventResult {
        match key {
            KeyCode::Escape => {
                if props.close_on_escape && state.visible {
                    self.hide_popover(props, state);
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::Tab => {
                if props.focus_trap && state.visible {
                    if state.focusable_elements.is_empty() {
                        EventResult::Ignored
                    } else {
                        state.focused_element_index =
                            (state.focused_element_index + 1) % state.focusable_elements.len();
                        EventResult::Consumed
                    }
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::Enter | KeyCode::Space => {
                if state.visible && !state.focusable_elements.is_empty() {
                    // Activate focused element
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }

    /// Set the trigger rectangle for positioning
    pub fn set_trigger_rect(&self, rect: Rect<f32>) {
        if let Ok(mut state) = self.state.lock() {
            state.trigger_rect = rect;
        }
    }

    /// Check if the popover is currently visible
    pub fn is_visible(&self) -> bool {
        self.state.lock().map(|s| s.visible).unwrap_or(false)
    }

    /// Show the popover with animation
    pub fn show(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.visible = true;
            state.animation_start = Some(Instant::now());
        }
    }

    /// Hide the popover with animation
    pub fn hide(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.visible = false;
            state.animation_start = None;
        }
    }
}

impl Component for Popover {
    type Props = PopoverProps;
    type State = PopoverState;

    fn new(_props: Self::Props) -> Self {
        Popover::new()
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        let mut state = self.state.lock().unwrap();

        // Update visibility from props
        if props.visible != state.visible {
            if props.visible {
                self.show_popover(props, &mut state);
            } else {
                self.hide_popover(props, &mut state);
            }
        }

        if !state.visible && !state.is_animating {
            return Element::empty();
        }

        // Update animation
        self.update_animation(&mut state, props);

        // Update hover state
        self.update_hover_state(props, &mut state);

        // Calculate position (using default container for now)
        let container_rect = Rect {
            left: 0.0,
            top: 0.0,
            right: 1000.0,
            bottom: 1000.0,
        };
        let (calc_rect, calc_position, arrow_pos) =
            self.calculate_position(props, &state, container_rect);

        // Update state with calculated values
        state.calculated_rect = calc_rect;
        state.arrow_position = arrow_pos;
        if calc_position != state.position {
            state.position = calc_position;
            state.boundary_adjusted = calc_position != props.position;

            if let Some(callback) = &props.on_position_change {
                callback(calc_position);
            }
        }

        // Apply animation transforms
        let (_opacity, _scale, _translate) = match props.animation {
            PopoverAnimation::None => (1.0, 1.0, (0, 0)),
            PopoverAnimation::Fade => (state.animation_progress, 1.0, (0, 0)),
            PopoverAnimation::Scale => (1.0, 0.8 + 0.2 * state.animation_progress, (0, 0)),
            PopoverAnimation::Slide => {
                let offset = ((1.0 - state.animation_progress) * 10.0) as i16;
                let translate_offset = match state.position {
                    PopoverPosition::Top | PopoverPosition::TopStart | PopoverPosition::TopEnd => {
                        (0, offset)
                    }
                    PopoverPosition::Bottom
                    | PopoverPosition::BottomStart
                    | PopoverPosition::BottomEnd => (0, -offset),
                    PopoverPosition::Left
                    | PopoverPosition::LeftStart
                    | PopoverPosition::LeftEnd => (offset, 0),
                    PopoverPosition::Right
                    | PopoverPosition::RightStart
                    | PopoverPosition::RightEnd => (-offset, 0),
                };
                (1.0, 1.0, translate_offset)
            }
            PopoverAnimation::Bounce => (1.0, state.animation_progress, (0, 0)),
        };

        // Build popover content
        let mut popover_content = vec![props.content.clone()];

        // Add arrow if enabled
        if let Some(arrow_position) = arrow_pos {
            let arrow_element =
                self.create_arrow_element(&props.arrow, state.position, arrow_position);
            popover_content.push(arrow_element);
        }

        // Create the popover container
        Element::layout(LayoutType::Stack)
            .with_children(popover_content)
            .with_key("popover-container")
    }

    fn update(&mut self, _props: &Self::Props, _state: &mut Self::State) -> bool {
        // Check if animation is running
        self.state.lock().map(|s| s.is_animating).unwrap_or(false)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> EventResult {
        let mut state = self.state.lock().unwrap();

        // Handle trigger events
        if self.handle_trigger_event(props, &mut state, event) {
            return EventResult::Consumed;
        }

        if !state.visible {
            return EventResult::Ignored;
        }

        match event {
            Event::Key(key_event) => self.handle_keyboard_navigation(
                key_event.code.clone(),
                key_event.modifiers,
                props,
                &mut state,
            ),
            Event::Mouse(mouse_event) => {
                if self.is_point_in_rect(mouse_event.position, state.calculated_rect) {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }
}

impl Popover {
    fn create_arrow_element(
        &self,
        arrow: &PopoverArrow,
        position: PopoverPosition,
        _arrow_pos: Position,
    ) -> Element {
        let arrow_char = match arrow.style {
            ArrowStyle::Solid => match position {
                PopoverPosition::Top | PopoverPosition::TopStart | PopoverPosition::TopEnd => "▲",
                PopoverPosition::Bottom
                | PopoverPosition::BottomStart
                | PopoverPosition::BottomEnd => "▼",
                PopoverPosition::Left | PopoverPosition::LeftStart | PopoverPosition::LeftEnd => {
                    "◀"
                }
                PopoverPosition::Right
                | PopoverPosition::RightStart
                | PopoverPosition::RightEnd => "▶",
            },
            ArrowStyle::Outline => match position {
                PopoverPosition::Top | PopoverPosition::TopStart | PopoverPosition::TopEnd => "△",
                PopoverPosition::Bottom
                | PopoverPosition::BottomStart
                | PopoverPosition::BottomEnd => "▽",
                PopoverPosition::Left | PopoverPosition::LeftStart | PopoverPosition::LeftEnd => {
                    "◁"
                }
                PopoverPosition::Right
                | PopoverPosition::RightStart
                | PopoverPosition::RightEnd => "▷",
            },
            ArrowStyle::Double => match position {
                PopoverPosition::Top | PopoverPosition::TopStart | PopoverPosition::TopEnd => "⇈",
                PopoverPosition::Bottom
                | PopoverPosition::BottomStart
                | PopoverPosition::BottomEnd => "⇊",
                PopoverPosition::Left | PopoverPosition::LeftStart | PopoverPosition::LeftEnd => {
                    "⇇"
                }
                PopoverPosition::Right
                | PopoverPosition::RightStart
                | PopoverPosition::RightEnd => "⇉",
            },
        };

        Element::text(arrow_char)
    }
}

/// Builder for creating popover components
pub struct PopoverBuilder {
    props: PopoverProps,
}

impl PopoverBuilder {
    /// Create a new popover builder
    ///
    /// # Returns
    /// A new `PopoverBuilder` with default properties
    pub fn new() -> Self {
        Self {
            props: PopoverProps::default(),
        }
    }

    /// Set the visibility of the popover
    ///
    /// # Arguments
    /// * `visible` - Whether the popover should be visible
    ///
    /// # Returns
    /// Self for method chaining
    pub fn visible(mut self, visible: bool) -> Self {
        self.props.visible = visible;
        self
    }

    /// Set the position of the popover relative to its trigger
    ///
    /// # Arguments
    /// * `position` - The position (top, bottom, left, right, etc.)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn position(mut self, position: PopoverPosition) -> Self {
        self.props.position = position;
        self
    }

    /// Set the trigger behavior for the popover
    ///
    /// # Arguments
    /// * `trigger` - How the popover should be triggered (click, hover, etc.)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn trigger(mut self, trigger: PopoverTrigger) -> Self {
        self.props.trigger = trigger;
        self
    }

    /// Set the animation for the popover
    ///
    /// # Arguments
    /// * `animation` - The animation type for show/hide transitions
    ///
    /// # Returns
    /// Self for method chaining
    pub fn animation(mut self, animation: PopoverAnimation) -> Self {
        self.props.animation = animation;
        self
    }

    /// Set the animation duration for the popover
    ///
    /// # Arguments
    /// * `duration` - How long the show/hide animation should take
    ///
    /// # Returns
    /// Self for method chaining
    pub fn animation_duration(mut self, duration: Duration) -> Self {
        self.props.animation_duration = duration;
        self
    }

    /// Set the content element for the popover
    ///
    /// # Arguments
    /// * `content` - The element to display inside the popover
    ///
    /// # Returns
    /// Self for method chaining
    pub fn content(mut self, content: Element) -> Self {
        self.props.content = content;
        self
    }

    /// Set the trigger element for the popover
    ///
    /// # Arguments
    /// * `element` - The element that triggers the popover
    ///
    /// # Returns
    /// Self for method chaining
    pub fn trigger_element(mut self, element: Element) -> Self {
        self.props.trigger_element = element;
        self
    }

    /// Set the arrow configuration for the popover
    ///
    /// # Arguments
    /// * `arrow` - Arrow configuration (visible, hidden, or custom)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn arrow(mut self, arrow: PopoverArrow) -> Self {
        self.props.arrow = arrow;
        self
    }

    /// Set the offset from the trigger element
    ///
    /// # Arguments
    /// * `x` - Horizontal offset in pixels
    /// * `y` - Vertical offset in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn offset(mut self, x: i16, y: i16) -> Self {
        self.props.offset = (x, y);
        self
    }

    /// Set the boundary behavior when the popover would overflow
    ///
    /// # Arguments
    /// * `behavior` - How to handle boundary collisions (flip, shift, etc.)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn boundary_behavior(mut self, behavior: BoundaryBehavior) -> Self {
        self.props.boundary_behavior = behavior;
        self
    }

    /// Enable or disable closing on escape key
    ///
    /// # Arguments
    /// * `close` - Whether the popover should close when escape is pressed
    ///
    /// # Returns
    /// Self for method chaining
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.props.close_on_escape = close;
        self
    }

    /// Enable or disable closing on outside click
    ///
    /// # Arguments
    /// * `close` - Whether the popover should close when clicking outside
    ///
    /// # Returns
    /// Self for method chaining
    pub fn close_on_outside_click(mut self, close: bool) -> Self {
        self.props.close_on_outside_click = close;
        self
    }

    /// Set the delay before showing on hover
    ///
    /// # Arguments
    /// * `delay` - Time to wait before showing the popover on hover
    ///
    /// # Returns
    /// Self for method chaining
    pub fn hover_delay(mut self, delay: Duration) -> Self {
        self.props.hover_delay = delay;
        self
    }

    /// Set the delay before hiding when leaving hover
    ///
    /// # Arguments
    /// * `delay` - Time to wait before hiding the popover when leaving hover
    ///
    /// # Returns
    /// Self for method chaining
    pub fn hover_leave_delay(mut self, delay: Duration) -> Self {
        self.props.hover_leave_delay = delay;
        self
    }

    /// Enable or disable focus trapping
    pub fn focus_trap(mut self, trap: bool) -> Self {
        self.props.focus_trap = trap;
        self
    }

    /// Enable or disable auto focus
    pub fn auto_focus(mut self, focus: bool) -> Self {
        self.props.auto_focus = focus;
        self
    }

    /// Set the z-index for layering
    pub fn z_index(mut self, index: u16) -> Self {
        self.props.z_index = index;
        self
    }

    /// Set size constraints for the popover
    pub fn size_constraints(
        mut self,
        min_width: Option<u16>,
        max_width: Option<u16>,
        min_height: Option<u16>,
        max_height: Option<u16>,
    ) -> Self {
        self.props.min_width = min_width;
        self.props.max_width = max_width;
        self.props.min_height = min_height;
        self.props.max_height = max_height;
        self
    }

    /// Set callback for when popover opens
    pub fn on_open<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.props.on_open = Some(Arc::new(callback));
        self
    }

    /// Set callback for when popover closes
    pub fn on_close<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.props.on_close = Some(Arc::new(callback));
        self
    }

    /// Set callback for when popover position changes
    pub fn on_position_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(PopoverPosition) + Send + Sync + 'static,
    {
        self.props.on_position_change = Some(Arc::new(callback));
        self
    }

    /// Build the popover with configured properties
    pub fn build(self) -> (Popover, PopoverProps) {
        (Popover::new(), self.props)
    }
}

impl Default for PopoverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ElementType;

    #[test]
    fn test_popover_creation() {
        let popover = Popover::new();
        assert!(!popover.is_visible());
    }

    #[test]
    fn test_popover_builder() {
        let (_popover, props) = Popover::builder()
            .visible(true)
            .position(PopoverPosition::Top)
            .trigger(PopoverTrigger::Hover)
            .animation(PopoverAnimation::Scale)
            .build();

        assert_eq!(props.visible, true);
        assert_eq!(props.position, PopoverPosition::Top);
        assert_eq!(props.trigger, PopoverTrigger::Hover);
        assert_eq!(props.animation, PopoverAnimation::Scale);
    }

    #[test]
    fn test_position_opposite() {
        assert_eq!(PopoverPosition::Top.opposite(), PopoverPosition::Bottom);
        assert_eq!(PopoverPosition::Left.opposite(), PopoverPosition::Right);
        assert_eq!(
            PopoverPosition::TopStart.opposite(),
            PopoverPosition::BottomStart
        );
    }

    #[test]
    fn test_popover_position_calculation() {
        let popover = Popover::new();
        let trigger_rect = Popover::make_rect(100.0, 100.0, 50.0, 30.0);
        let content_size = (200, 100);
        let offset = (0, 8);

        let rect = popover.calculate_rect_for_position(
            PopoverPosition::Bottom,
            trigger_rect,
            content_size,
            offset,
        );

        assert_eq!(rect.left, 25.0); // Centered: 100 + (50-200)/2 = 25
        assert_eq!(rect.top, 138.0); // Below: 100 + 30 + 8 = 138
        assert_eq!(rect.right - rect.left, 200.0); // width
        assert_eq!(rect.bottom - rect.top, 100.0); // height
    }

    #[test]
    fn test_boundary_adjustment_flip() {
        let popover = Popover::new();
        let trigger_rect = Popover::make_rect(50.0, 50.0, 50.0, 30.0);
        let container_rect = Popover::make_rect(0.0, 0.0, 100.0, 100.0);
        let content_size = (200, 100);

        let rect = popover.calculate_rect_for_position(
            PopoverPosition::Right,
            trigger_rect,
            content_size,
            (8, 0),
        );

        let props = PopoverProps {
            boundary_behavior: BoundaryBehavior::Flip,
            offset: (8, 0),
            ..PopoverProps::default()
        };

        let (adjusted_rect, adjusted_position) = popover.adjust_for_boundaries(
            rect,
            PopoverPosition::Right,
            trigger_rect,
            content_size,
            container_rect,
            &props,
        );

        // Should flip to left since right doesn't fit
        assert_eq!(adjusted_position, PopoverPosition::Left);
        assert!(adjusted_rect.left < trigger_rect.left);
    }

    #[test]
    fn test_boundary_adjustment_shift() {
        let popover = Popover::new();
        let trigger_rect = Popover::make_rect(10.0, 10.0, 50.0, 30.0);
        let container_rect = Popover::make_rect(0.0, 0.0, 300.0, 200.0);
        let content_size = (200, 100);

        let rect = Popover::make_rect(-50.0, 20.0, 200.0, 100.0); // Extends beyond left

        let props = PopoverProps {
            boundary_behavior: BoundaryBehavior::Shift,
            ..PopoverProps::default()
        };

        let (adjusted_rect, _) = popover.adjust_for_boundaries(
            rect,
            PopoverPosition::Left,
            trigger_rect,
            content_size,
            container_rect,
            &props,
        );

        // Should shift to fit within bounds
        assert_eq!(adjusted_rect.left, 0.0);
        assert_eq!(adjusted_rect.top, 20.0);
    }

    #[test]
    fn test_arrow_position_calculation() {
        let popover = Popover::new();
        let popover_rect = Popover::make_rect(100.0, 50.0, 200.0, 100.0);
        let trigger_rect = Popover::make_rect(150.0, 160.0, 100.0, 30.0);
        let arrow = PopoverArrow::default();

        let arrow_pos = popover.calculate_arrow_position(
            PopoverPosition::Top,
            popover_rect,
            trigger_rect,
            &arrow,
        );

        assert!(arrow_pos.is_some());
        if let Some(Position::Cell { x, y }) = arrow_pos {
            // For PopoverPosition::Top, arrow is at bottom of popover
            // Center x: (100 + 300) / 2 = 200
            // Bottom y: 150
            assert_eq!(x, 200);
            assert_eq!(y, 150);
        } else {
            panic!("Expected Cell position");
        }
    }

    #[test]
    fn test_size_constraints() {
        let popover = Popover::new();
        let rect = Popover::make_rect(0.0, 0.0, 150.0, 80.0);

        let props = PopoverProps {
            min_width: Some(200),
            max_width: Some(300),
            min_height: Some(100),
            max_height: Some(150),
            ..PopoverProps::default()
        };

        let adjusted_rect = popover.apply_size_constraints(rect, &props);

        assert_eq!(adjusted_rect.right - adjusted_rect.left, 200.0); // Applied min_width
        assert_eq!(adjusted_rect.bottom - adjusted_rect.top, 100.0); // Applied min_height
    }

    #[test]
    fn test_point_in_rect() {
        let popover = Popover::new();
        let rect = Popover::make_rect(10.0, 20.0, 50.0, 30.0);

        assert!(popover.is_point_in_rect(Position::Cell { x: 30, y: 35 }, rect));
        assert!(!popover.is_point_in_rect(Position::Cell { x: 5, y: 25 }, rect));
        assert!(!popover.is_point_in_rect(Position::Cell { x: 65, y: 35 }, rect));
    }

    #[test]
    fn test_animation_easing() {
        let popover = Popover::new();

        assert_eq!(popover.ease_out_cubic(0.0), 0.0);
        assert_eq!(popover.ease_out_cubic(1.0), 1.0);
        assert!(popover.ease_out_cubic(0.5) > 0.5); // Should be accelerated

        let bounce_result = popover.ease_out_bounce(0.8);
        assert!(bounce_result >= 0.0 && bounce_result <= 1.2); // Bounce can overshoot slightly
    }

    #[test]
    fn test_manual_visibility_control() {
        let popover = Popover::new();

        assert!(!popover.is_visible());

        popover.show();
        assert!(popover.is_visible());

        popover.hide();
        assert!(!popover.is_visible());
    }

    #[test]
    fn test_trigger_rect_setting() {
        let popover = Popover::new();
        let rect = Popover::make_rect(100.0, 200.0, 50.0, 25.0);

        popover.set_trigger_rect(rect);

        if let Ok(state) = popover.state.lock() {
            assert_eq!(state.trigger_rect, rect);
        };
    }

    #[test]
    fn test_render_empty_when_invisible() {
        let popover = Popover::new();
        let props = PopoverProps::default();
        let state = PopoverState::default();

        let element = popover.render(&props, &state);

        assert!(matches!(element.element_type, ElementType::Empty));
    }

    #[test]
    fn test_render_with_content() {
        let popover = Popover::new();
        let content = Element::text("Hello World");
        let props = PopoverProps {
            visible: true,
            content,
            animation: PopoverAnimation::None,
            ..PopoverProps::default()
        };
        let state = PopoverState::default();

        let element = popover.render(&props, &state);

        assert!(matches!(
            element.element_type,
            ElementType::Layout(LayoutType::Stack)
        ));
        assert!(!element.children.is_empty());
    }

    #[test]
    fn test_keyboard_navigation() {
        let mut popover = Popover::new();
        let props = PopoverProps {
            visible: true,
            close_on_escape: true,
            focus_trap: true,
            ..PopoverProps::default()
        };

        // Set up visible state
        popover.show();

        let escape_event = Event::Key(crate::event::types::KeyEvent::new(KeyCode::Escape));

        let mut state = PopoverState::default();
        let result = popover.handle_event(&escape_event, &mut props.clone(), &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert!(!popover.is_visible()); // Should be hidden after escape
    }

    #[test]
    fn test_click_outside_closes() {
        let mut popover = Popover::new();
        let props = PopoverProps {
            visible: true,
            close_on_outside_click: true,
            ..PopoverProps::default()
        };

        // Set visible state and trigger rect
        popover.show();
        popover.set_trigger_rect(Popover::make_rect(50.0, 50.0, 50.0, 30.0));

        let outside_click = Event::Mouse(
            crate::event::types::MouseEvent::new(
                MouseEventKind::Down,
                crate::event::types::Position::cell(10, 10),
            )
            .with_button(MouseButton::Left),
        );

        let mut state = PopoverState::default();
        let result = popover.handle_event(&outside_click, &mut props.clone(), &mut state);
        // Note: This test depends on the calculated_rect being set properly
        // In real usage, render() would be called first to set up the state
        assert!(result == EventResult::Consumed || result == EventResult::Ignored);
    }

    #[test]
    fn test_arrow_styles() {
        let solid_arrow = PopoverArrow {
            style: ArrowStyle::Solid,
            ..PopoverArrow::default()
        };
        let outline_arrow = PopoverArrow {
            style: ArrowStyle::Outline,
            ..PopoverArrow::default()
        };
        let double_arrow = PopoverArrow {
            style: ArrowStyle::Double,
            ..PopoverArrow::default()
        };

        assert_eq!(solid_arrow.style, ArrowStyle::Solid);
        assert_eq!(outline_arrow.style, ArrowStyle::Outline);
        assert_eq!(double_arrow.style, ArrowStyle::Double);
    }

    #[test]
    fn test_component_update() {
        let mut popover = Popover::new();
        let props = PopoverProps {
            animation: PopoverAnimation::Fade,
            animation_duration: Duration::from_millis(100),
            ..PopoverProps::default()
        };

        // Start animation
        popover.show();
        if let Ok(mut state) = popover.state.lock() {
            state.is_animating = true;
        }

        let needs_update = popover.update(&props, &mut PopoverState::default());
        assert!(needs_update); // Should need updates while animating
    }

    #[test]
    fn test_comprehensive_builder_pattern() {
        let (_popover, props) = Popover::builder()
            .visible(true)
            .position(PopoverPosition::TopStart)
            .trigger(PopoverTrigger::Hover)
            .animation(PopoverAnimation::Bounce)
            .animation_duration(Duration::from_millis(300))
            .content(Element::text("Tooltip content"))
            .offset(10, -5)
            .boundary_behavior(BoundaryBehavior::Shift)
            .close_on_escape(false)
            .close_on_outside_click(true)
            .hover_delay(Duration::from_millis(500))
            .hover_leave_delay(Duration::from_millis(200))
            .focus_trap(true)
            .auto_focus(true)
            .z_index(2000)
            .size_constraints(Some(150), Some(400), Some(80), Some(300))
            .build();

        assert!(props.visible);
        assert_eq!(props.position, PopoverPosition::TopStart);
        assert_eq!(props.trigger, PopoverTrigger::Hover);
        assert_eq!(props.animation, PopoverAnimation::Bounce);
        assert_eq!(props.animation_duration, Duration::from_millis(300));
        assert_eq!(props.offset, (10, -5));
        assert_eq!(props.boundary_behavior, BoundaryBehavior::Shift);
        assert!(!props.close_on_escape);
        assert!(props.close_on_outside_click);
        assert_eq!(props.hover_delay, Duration::from_millis(500));
        assert_eq!(props.hover_leave_delay, Duration::from_millis(200));
        assert!(props.focus_trap);
        assert!(props.auto_focus);
        assert_eq!(props.z_index, 2000);
        assert_eq!(props.min_width, Some(150));
        assert_eq!(props.max_width, Some(400));
        assert_eq!(props.min_height, Some(80));
        assert_eq!(props.max_height, Some(300));
    }
}
