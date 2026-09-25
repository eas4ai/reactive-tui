use super::overlay::same_callback;
use crate::component::{Component, Element, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyCode, MouseButton, MouseEventKind, Position};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use taffy::geometry::Rect;

mod live;

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
            && same_callback(&self.on_open, &other.on_open)
            && same_callback(&self.on_close, &other.on_close)
            && same_callback(&self.on_position_change, &other.on_position_change)
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
#[derive(Clone)]
pub struct Popover {
    state: Arc<Mutex<PopoverState>>,
    live: Arc<live::Runtime>,
}

impl Default for Popover {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(PopoverState::default())),
            live: Arc::default(),
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
                trigger_x + (trigger_width - content_width) / 2.0 + offset_x,
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
                trigger_x + (trigger_width - content_width) / 2.0 + offset_x,
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
                trigger_y + (trigger_height - content_height) / 2.0 + offset_y,
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
                trigger_y + (trigger_height - content_height) / 2.0 + offset_y,
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
                    let overflow = |candidate: Rect<f32>| {
                        (container_rect.left - candidate.left).max(0.0)
                            + (candidate.right - container_rect.right).max(0.0)
                            + (container_rect.top - candidate.top).max(0.0)
                            + (candidate.bottom - container_rect.bottom).max(0.0)
                    };
                    if overflow(flipped_rect) < overflow(rect) {
                        adjusted_rect = flipped_rect;
                        adjusted_position = flipped_position;
                    }
                }
                BoundaryBehavior::Shift => {}
                BoundaryBehavior::Hide => {
                    if !fits_in_bounds {
                        adjusted_rect.right = adjusted_rect.left;
                        adjusted_rect.bottom = adjusted_rect.top;
                    }
                }
                BoundaryBehavior::Ignore => {}
            }
            if matches!(
                props.boundary_behavior,
                BoundaryBehavior::Flip | BoundaryBehavior::Shift
            ) {
                let width = adjusted_rect.right - adjusted_rect.left;
                let height = adjusted_rect.bottom - adjusted_rect.top;
                adjusted_rect.left = adjusted_rect.left.clamp(
                    container_rect.left,
                    (container_rect.right - width).max(container_rect.left),
                );
                adjusted_rect.top = adjusted_rect.top.clamp(
                    container_rect.top,
                    (container_rect.bottom - height).max(container_rect.top),
                );
                adjusted_rect.right = adjusted_rect.left + width;
                adjusted_rect.bottom = adjusted_rect.top + height;
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

    /// Set the trigger rectangle in terminal cells. Nonfinite or reversed bounds
    /// restore automatic measurement of the rendered trigger.
    pub fn set_trigger_rect(&self, rect: Rect<f32>) {
        self.live.set_trigger(&self.state, rect);
    }

    /// Check if the popover is currently visible.
    pub fn is_visible(&self) -> bool {
        self.state.lock().unwrap().visible
    }

    /// Show the popover and wake its mounted App.
    pub fn show(&self) {
        self.live.request(&self.state, true);
    }

    /// Hide the popover and wake its mounted App.
    pub fn hide(&self) {
        self.live.request(&self.state, false);
    }
}

impl Component for Popover {
    type Props = PopoverProps;
    type State = PopoverState;

    fn new(_props: Self::Props) -> Self {
        Self::default()
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::typed::<live::LivePopover>(live::LiveProps {
            owner: self.clone(),
            config: props.clone(),
        })
    }

    fn update(&mut self, _props: &Self::Props, _state: &mut Self::State) -> bool {
        self.state.lock().unwrap().is_animating
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        _: &mut Self::State,
    ) -> EventResult {
        self.escape(event, props)
    }

    fn on_lifecycle(&mut self, event: crate::component::LifecycleEvent, _: &mut Self::State) {
        if matches!(event, crate::component::LifecycleEvent::Unmount) {
            self.live.cancel();
        }
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
    /// * `x` - Horizontal offset in terminal cells
    /// * `y` - Vertical offset in terminal cells
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

        assert!(props.visible);
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
    fn test_animation_easing() {
        let popover = Popover::new();

        assert_eq!(popover.ease_out_cubic(0.0), 0.0);
        assert_eq!(popover.ease_out_cubic(1.0), 1.0);
        assert!(popover.ease_out_cubic(0.5) > 0.5); // Should be accelerated

        let bounce_result = popover.ease_out_bounce(0.8);
        assert!((0.0..=1.2).contains(&bounce_result)); // Bounce can overshoot slightly
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

        let state = popover
            .state
            .lock()
            .expect("the popover state lock is not poisoned");
        assert_eq!(state.trigger_rect, rect);
    }

    #[test]
    fn rendered_handle_retains_manual_visibility_until_app_expansion() {
        let popover = Popover::new();
        popover.show();
        let element = popover.render(&PopoverProps::default(), &PopoverState::default());
        assert!(popover.is_visible());
        assert!(matches!(element.element_type, ElementType::Component(_)));
        popover.hide();
        assert!(!popover.is_visible());
    }

    #[test]
    fn callback_replacement_changes_props_identity() {
        let props = PopoverProps {
            on_open: Some(Arc::new(|| {})),
            ..Default::default()
        };
        assert!(props == props.clone());
        let replaced = PopoverProps {
            on_open: Some(Arc::new(|| {})),
            ..props.clone()
        };
        assert!(props != replaced);
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
    fn escape_disabled_and_release_do_not_close() {
        let mut popover = Popover::new();
        popover.show();
        let mut props = PopoverProps {
            close_on_escape: false,
            ..Default::default()
        };
        let mut state = PopoverState::default();
        let escape = Event::Key(crate::event::types::KeyEvent::new(KeyCode::Escape));
        assert_eq!(
            popover.handle_event(&escape, &mut props, &mut state),
            EventResult::Consumed
        );
        props.close_on_escape = true;
        let release = Event::Key(
            crate::event::types::KeyEvent::new(KeyCode::Escape)
                .with_kind(crate::event::types::KeyEventKind::Release),
        );
        assert_eq!(
            popover.handle_event(&release, &mut props, &mut state),
            EventResult::Ignored
        );
        assert!(popover.is_visible());
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
