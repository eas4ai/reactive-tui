use crate::event::types::{MouseButton, Position};
use crate::reactive::hooks::{use_signal, Hooks, ThreadSafeSignal};
use std::time::{Duration, Instant};

fn routing_context() -> Option<super::processor::MouseHookContext> {
    crate::reactive::component_scope::lookup()
}

/// Hover state for use_hover hook
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HoverState {
    /// Whether the element is currently being hovered
    pub is_hovered: bool,
    /// When the hover state was entered
    pub entered_at: Option<Instant>,
    /// Current mouse position over the element
    pub position: Option<Position>,
}

/// Tracks hover state for a component
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::hooks::mouse::*;
///
/// #[component]
/// fn Card(hooks: &Hooks) -> Element {
///     let state = use_hover(hooks).get();
///     div().class(if state.is_hovered { "bg-gray-100" } else { "bg-white" })
///         .child(Element::text("Hover over me")).build()
/// }
/// ```
pub fn use_hover(hooks: &Hooks) -> ThreadSafeSignal<HoverState> {
    let hover_state = use_signal(hooks, HoverState::default());
    if let Some(context) = routing_context() {
        context
            .processor
            .register_hover(context.owner, hover_state.clone());
    }
    hover_state
}

/// Drag state for use_drag hook
#[derive(Clone, Debug, PartialEq)]
pub struct DragState {
    /// Whether a drag operation is currently active
    pub is_dragging: bool,
    /// Whether the dragged item is over a valid drop zone
    pub is_over_drop_zone: bool,
    /// Position where the drag started
    pub drag_start: Option<Position>,
    /// Current position during drag
    pub current_position: Option<Position>,
    /// Delta in input coordinate units, saturating at i32 limits for large pixel moves.
    pub drag_delta: (i32, i32),
    /// Mouse button used for dragging
    pub button: MouseButton,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            is_dragging: false,
            is_over_drop_zone: false,
            drag_start: None,
            current_position: None,
            drag_delta: (0, 0),
            button: MouseButton::None,
        }
    }
}

impl DragState {
    /// Distance in cells for Cell input or pixels for Pixel input.
    /// Missing positions or a change of coordinate units return zero.
    pub fn distance(&self) -> f64 {
        if let (Some(start), Some(current)) = (self.drag_start, self.current_position) {
            coordinate_delta(start, current).map_or(0.0, |(dx, dy)| dx.hypot(dy))
        } else {
            0.0
        }
    }

    /// Check a threshold expressed in the input positions' coordinate units.
    pub fn is_drag_threshold_met(&self, threshold: f64) -> bool {
        self.distance() >= threshold
    }
}

/// Subtract only positions with matching units, without integer overflow.
pub(crate) fn coordinate_delta(start: Position, end: Position) -> Option<(f64, f64)> {
    match (start, end) {
        (Position::Cell { .. }, Position::Cell { .. })
        | (Position::Pixel { .. }, Position::Pixel { .. }) => Some((
            f64::from(end.x()) - f64::from(start.x()),
            f64::from(end.y()) - f64::from(start.y()),
        )),
        _ => None,
    }
}

/// Tracks drag state for a component
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::hooks::mouse::*;
///
/// #[component]
/// fn DraggableItem(hooks: &Hooks) -> Element {
///     let drag = use_drag(hooks).get();
///     div().class(if drag.is_dragging { "opacity-50" } else { "opacity-100" })
///         .child(Element::text(format!("Drag delta: {:?}", drag.drag_delta))).build()
/// }
/// ```
pub fn use_drag(hooks: &Hooks) -> ThreadSafeSignal<DragState> {
    let drag_state = use_signal(hooks, DragState::default());
    if let Some(context) = routing_context() {
        context
            .processor
            .register_drag(context.owner, drag_state.clone());
    }
    drag_state
}

/// Options for use_drag_and_drop hook
#[derive(Clone, Debug, PartialEq)]
pub struct DragAndDropOptions {
    /// Minimum distance before dragging: cells for Cell input, pixels for Pixel input.
    pub drag_threshold: f64,
    /// Optional exact routed component identifier required at drag start.
    /// Component keys are identifiers; unkeyed components use their registered name.
    pub drag_handle_selector: Option<String>,
    /// Valid routed component identifiers. Component keys are preferred.
    pub drop_zones: Vec<String>,
    /// Whether dragging outside drop zones is allowed
    pub allow_drag_outside: bool,
}

impl Default for DragAndDropOptions {
    fn default() -> Self {
        Self {
            drag_threshold: 5.0,
            drag_handle_selector: None,
            drop_zones: Vec::new(),
            allow_drag_outside: true,
        }
    }
}

/// Combined drag and drop state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DragAndDropState {
    /// Current drag state
    pub drag: DragState,
    /// Whether the dragged item is over a valid drop zone
    pub is_over_valid_drop: bool,
    /// Identifier of the current drop target
    pub drop_target: Option<String>,
    /// Whether dropping is allowed at current position
    pub can_drop: bool,
}

/// Advanced drag and drop with drop zone detection
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::hooks::mouse::*;
///
/// #[component]
/// fn DragAndDropDemo(hooks: &Hooks) -> Element {
///     let state = use_drag_and_drop(hooks, DragAndDropOptions {
///         drop_zones: vec!["drop-zone-1".into()],
///         ..Default::default()
///     });
///     Element::text(format!("Drop target: {:?}; allowed: {}", state.drop_target, state.can_drop))
/// }
/// ```
pub fn use_drag_and_drop(hooks: &Hooks, options: DragAndDropOptions) -> DragAndDropState {
    let state = use_signal(hooks, DragAndDropState::default());
    if let Some(context) = routing_context() {
        context
            .processor
            .register_drag_and_drop(context.owner, state.clone(), options);
    }
    state.get()
}

/// Mouse position tracking state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MousePositionState {
    /// Current mouse position in terminal coordinates
    pub position: Option<Position>,
    /// Client-relative position (x, y) in floating point
    pub client_position: Option<(f64, f64)>,
    /// Whether the mouse is inside the component bounds
    pub is_inside: bool,
}

/// Track mouse position relative to component
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::hooks::mouse::*;
///
/// #[component]
/// fn MouseTracker(hooks: &Hooks) -> Element {
///     let mouse = use_mouse_position(hooks).get();
///     Element::text(format!("Mouse position: {:?}", mouse.position))
/// }
/// ```
pub fn use_mouse_position(hooks: &Hooks) -> ThreadSafeSignal<MousePositionState> {
    let mouse_state = use_signal(hooks, MousePositionState::default());
    if let Some(context) = routing_context() {
        context
            .processor
            .register_position(context.owner, mouse_state.clone());
    }
    mouse_state
}

/// Click detection with double/triple click support
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClickState {
    /// Number of consecutive clicks (1=single, 2=double, 3=triple)
    pub click_count: usize,
    /// Timestamp of the last click for timing detection
    pub last_click: Option<Instant>,
    /// Position of the last click
    pub position: Option<Position>,
    /// Whether the current click is a double-click
    pub is_double_click: bool,
    /// Whether the current click is a triple-click
    pub is_triple_click: bool,
}

/// Detect various click patterns
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::hooks::mouse::*;
///
/// #[component]
/// fn ClickableItem(hooks: &Hooks) -> Element {
///     let clicks = use_clicks(hooks).get();
///     Element::text(format!("Clicks: {}; double: {}", clicks.click_count, clicks.is_double_click))
/// }
/// ```
pub fn use_clicks(hooks: &Hooks) -> ThreadSafeSignal<ClickState> {
    let click_state = use_signal(hooks, ClickState::default());
    if let Some(context) = routing_context() {
        context
            .processor
            .register_clicks(context.owner, click_state.clone());
    }
    click_state
}

/// Long press detection
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LongPressState {
    /// Whether the mouse button is currently pressed
    pub is_pressing: bool,
    /// Whether a long press has been detected
    pub is_long_press: bool,
    /// When the press started
    pub press_start: Option<Instant>,
    /// Duration of the current press
    pub duration: Duration,
}

/// Detect long press gestures
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::hooks::mouse::*;
///
/// #[component]
/// fn LongPressButton(hooks: &Hooks) -> Element {
///     let press = use_long_press(hooks, std::time::Duration::from_millis(800)).get();
///     button().child(Element::text(if press.is_long_press { "Long press" } else { "Hold" })).build()
/// }
/// ```
pub fn use_long_press(hooks: &Hooks, threshold: Duration) -> ThreadSafeSignal<LongPressState> {
    let press_state = use_signal(hooks, LongPressState::default());
    if let Some(context) = routing_context() {
        context.processor.register_press_with_threshold(
            context.owner,
            press_state.clone(),
            threshold,
        );
    }
    press_state
}

/// Gesture recognition state
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GestureState {
    /// Type of gesture currently detected
    pub gesture_type: GestureType,
    /// Whether a gesture is currently active
    pub is_active: bool,
    /// Starting position of the gesture
    pub start_position: Option<Position>,
    /// Ending position of the gesture
    pub end_position: Option<Position>,
    /// Velocity of the gesture (x, y) in pixels per second
    pub velocity: (f64, f64),
}

/// Type of mouse gesture detected
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum GestureType {
    /// No gesture detected
    #[default]
    None,
    /// Swipe gesture in a specific direction
    Swipe(SwipeDirection),
    /// Pinch gesture with scale factor
    Pinch {
        /// Scale factor of the pinch
        scale: f64,
    },
    /// Rotation gesture with angle
    Rotate {
        /// Rotation angle in radians
        angle: f64,
    },
}

/// Direction of a swipe gesture
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwipeDirection {
    /// Swipe upward
    Up,
    /// Swipe downward
    Down,
    /// Swipe to the left
    Left,
    /// Swipe to the right
    Right,
}

/// Detect swipe gestures
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::hooks::mouse::*;
///
/// #[component]
/// fn SwipeableCard(hooks: &Hooks) -> Element {
///     let gesture = use_gesture(hooks).get();
///     let message = match gesture.gesture_type {
///         GestureType::Swipe(SwipeDirection::Left) => "Swiped left",
///         GestureType::Swipe(SwipeDirection::Right) => "Swiped right",
///         _ => "Swipe me",
///     };
///     Element::text(message)
/// }
/// ```
pub fn use_gesture(hooks: &Hooks) -> ThreadSafeSignal<GestureState> {
    let gesture_state = use_signal(
        hooks,
        GestureState {
            gesture_type: GestureType::None,
            is_active: false,
            start_position: None,
            end_position: None,
            velocity: (0.0, 0.0),
        },
    );

    if let Some(context) = routing_context() {
        context
            .processor
            .register_gesture(context.owner, gesture_state.clone());
    }
    gesture_state
}

/// Mouse wheel/scroll state
/// Mouse wheel state for scroll tracking
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WheelState {
    /// Horizontal scroll delta
    pub delta_x: f64,
    /// Vertical scroll delta
    pub delta_y: f64,
    /// Mode for interpreting delta values
    pub delta_mode: WheelDeltaMode,
    /// Whether scrolling is currently active
    pub is_scrolling: bool,
}

/// Mode for interpreting mouse wheel delta values
#[derive(Clone, Debug, Default, PartialEq)]
pub enum WheelDeltaMode {
    /// Delta values represent pixels
    #[default]
    Pixel,
    /// Delta values represent lines of text
    Line,
    /// Delta values represent pages
    Page,
}

/// Track mouse wheel/scroll events
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
/// use reactive_tui::hooks::mouse::*;
///
/// #[component]
/// fn ScrollableContent(hooks: &Hooks) -> Element {
///     let wheel = use_wheel(hooks).get();
///     Element::text(format!("Wheel delta: {}, {}", wheel.delta_x, wheel.delta_y))
/// }
/// ```
pub fn use_wheel(hooks: &Hooks) -> ThreadSafeSignal<WheelState> {
    let wheel_state = use_signal(hooks, WheelState::default());
    if let Some(context) = routing_context() {
        context
            .processor
            .register_wheel(context.owner, wheel_state.clone());
    }
    wheel_state
}
