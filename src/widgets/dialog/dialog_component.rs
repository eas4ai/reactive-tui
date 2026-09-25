//! Dialog Component Trait and Base Implementation
//!
//! Defines the core trait that all dialog types must implement and provides
//! common functionality for dialog rendering, event handling, and lifecycle management.

use super::{DialogId, DialogResult, DialogTheme};
use crate::component::Element;
use crate::core::geometry::{Point, Rect, Size};
use crate::event::types::Event;
use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Core trait that all dialog components must implement
pub trait DialogComponent: std::fmt::Debug + Send + Sync {
    /// Get the dialog ID
    fn id(&self) -> DialogId;

    /// Get the dialog type name
    fn dialog_type(&self) -> &'static str;

    /// Render the dialog content
    fn render(&self, bounds: Rect, theme: &DialogTheme) -> Element;

    /// Handle input events
    fn handle_event(&mut self, event: &Event) -> DialogEventResult;

    /// Update dialog state (called each frame)
    fn update(&mut self, delta_time: Duration) -> bool;

    /// Get dialog bounds and positioning
    fn get_bounds(&self) -> DialogBounds;

    /// Check if dialog should be modal (blocks interaction with background)
    fn is_modal(&self) -> bool;

    /// Check if dialog can be closed by clicking backdrop
    fn backdrop_closable(&self) -> bool;

    /// Check if dialog can be closed with escape key
    fn escape_closable(&self) -> bool;

    /// Get dialog z-index for layering
    fn z_index(&self) -> u16;

    /// Get animation configuration
    fn animation(&self) -> Option<DialogAnimationConfig>;

    /// Called when dialog is shown
    fn on_show(&mut self) {}

    /// Called when dialog is hidden
    fn on_hide(&mut self) {}

    /// Called when dialog gains focus
    fn on_focus(&mut self) {}

    /// Called when dialog loses focus
    fn on_blur(&mut self) {}

    /// Get focusable elements in this dialog
    fn get_focusable_elements(&self) -> Vec<FocusableElementInfo>;

    /// Set focus to specific element
    fn set_focus(&mut self, element_id: &str) -> bool;

    /// Get currently focused element
    fn get_focused_element(&self) -> Option<String>;

    /// Validate dialog state (for input dialogs)
    fn validate(&self) -> ValidationResult;

    /// Get dialog as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get dialog as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Result from dialog event handling
#[derive(Debug, Clone)]
pub enum DialogEventResult {
    /// Event was handled, no further action needed
    Handled,
    /// Event was handled and dialog should close with result
    Close(DialogResult),
    /// Event was handled and should trigger callback
    Callback(String, Option<String>),
    /// Event was handled and should send async request
    AsyncRequest(AsyncRequestType),
    /// Event was not handled, pass to next handler
    NotHandled,
    /// Event was handled and dialog state changed
    StateChanged,
    /// Event was handled and focus should change
    FocusChange(String),
}

/// Types of async requests that dialogs can make
#[derive(Debug, Clone)]
pub enum AsyncRequestType {
    /// HTTP request for autocomplete suggestions
    AutocompleteSuggestions {
        /// Search query string
        query: String,
        /// URL endpoint for autocomplete API
        url: String,
        /// Optional HTTP headers for the request
        headers: Option<std::collections::HashMap<String, String>>,
    },
    /// Validation request
    Validation {
        /// Field name being validated
        field: String,
        /// Value to validate
        value: String,
        /// Optional URL for remote validation
        validator_url: Option<String>,
    },
    /// Custom async request
    Custom {
        /// Type identifier for the custom request
        request_type: String,
        /// Request data payload
        data: String,
    },
}

/// Dialog bounds and positioning information
#[derive(Debug, Clone)]
pub struct DialogBounds {
    /// Preferred size (None for auto-sizing)
    pub size: Option<Size>,
    /// Minimum size
    pub min_size: Option<Size>,
    /// Maximum size
    pub max_size: Option<Size>,
    /// Position preference
    pub position: DialogPosition,
    /// Whether dialog can be resized
    pub resizable: bool,
    /// Whether dialog can be dragged
    pub draggable: bool,
    /// Margin from screen edges
    pub margin: DialogMargin,
}

/// Dialog positioning options
#[derive(Clone)]
pub enum DialogPosition {
    /// Center of screen
    Center,
    /// Top center
    TopCenter,
    /// Bottom center
    BottomCenter,
    /// Left center
    LeftCenter,
    /// Right center
    RightCenter,
    /// Specific coordinates
    Fixed(Point),
    /// Relative to another element
    RelativeTo {
        /// ID of the element to position relative to
        element_id: String,
        /// Offset from the anchor point
        offset: Point,
        /// Anchor point on the target element
        anchor: DialogAnchor,
    },
    /// Custom positioning function
    Custom(Arc<dyn Fn(Size) -> Point + Send + Sync>),
}

impl std::fmt::Debug for DialogPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DialogPosition::Center => write!(f, "Center"),
            DialogPosition::TopCenter => write!(f, "TopCenter"),
            DialogPosition::BottomCenter => write!(f, "BottomCenter"),
            DialogPosition::LeftCenter => write!(f, "LeftCenter"),
            DialogPosition::RightCenter => write!(f, "RightCenter"),
            DialogPosition::Fixed(point) => write!(f, "Fixed({:?})", point),
            DialogPosition::RelativeTo {
                element_id,
                offset,
                anchor,
            } => {
                write!(
                    f,
                    "RelativeTo {{ element_id: {:?}, offset: {:?}, anchor: {:?} }}",
                    element_id, offset, anchor
                )
            }
            DialogPosition::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

impl PartialEq for DialogPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DialogPosition::Center, DialogPosition::Center) => true,
            (DialogPosition::TopCenter, DialogPosition::TopCenter) => true,
            (DialogPosition::BottomCenter, DialogPosition::BottomCenter) => true,
            (DialogPosition::LeftCenter, DialogPosition::LeftCenter) => true,
            (DialogPosition::RightCenter, DialogPosition::RightCenter) => true,
            (DialogPosition::Fixed(a), DialogPosition::Fixed(b)) => a == b,
            (
                DialogPosition::RelativeTo {
                    element_id: id1,
                    offset: off1,
                    anchor: anc1,
                },
                DialogPosition::RelativeTo {
                    element_id: id2,
                    offset: off2,
                    anchor: anc2,
                },
            ) => id1 == id2 && off1 == off2 && anc1 == anc2,
            (DialogPosition::Custom(a), DialogPosition::Custom(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// Anchor points for relative positioning
#[derive(Debug, Clone, PartialEq)]
pub enum DialogAnchor {
    /// Anchor to the top-left corner
    TopLeft,
    /// Anchor to the top center
    TopCenter,
    /// Anchor to the top-right corner
    TopRight,
    /// Anchor to the center-left edge
    CenterLeft,
    /// Anchor to the center point
    Center,
    /// Anchor to the center-right edge
    CenterRight,
    /// Anchor to the bottom-left corner
    BottomLeft,
    /// Anchor to the bottom center
    BottomCenter,
    /// Anchor to the bottom-right corner
    BottomRight,
}

/// Dialog margin configuration
#[derive(Debug, Clone)]
pub struct DialogMargin {
    /// Top margin in terminal cells
    pub top: u16,
    /// Right margin in terminal cells
    pub right: u16,
    /// Bottom margin in terminal cells
    pub bottom: u16,
    /// Left margin in terminal cells
    pub left: u16,
}

/// Animation configuration for dialogs
#[derive(Debug, Clone)]
pub struct DialogAnimationConfig {
    /// Animation type
    pub animation_type: DialogAnimationType,
    /// Animation duration
    pub duration: Duration,
    /// Animation easing function
    pub easing: DialogEasing,
    /// Whether to animate on show
    pub animate_in: bool,
    /// Whether to animate on hide
    pub animate_out: bool,
}

/// Types of dialog animations
#[derive(Debug, Clone, PartialEq)]
pub enum DialogAnimationType {
    /// No animation
    None,
    /// Fade in/out animation
    Fade,
    /// Slide up from bottom
    SlideUp,
    /// Slide down from top
    SlideDown,
    /// Slide in from left
    SlideLeft,
    /// Slide in from right
    SlideRight,
    /// Scale up/down animation
    Scale,
    /// Bounce animation effect
    Bounce,
    /// Flip animation effect
    Flip,
    /// Custom animation by name
    Custom(String),
}

/// Animation easing functions
#[derive(Debug, Clone, PartialEq)]
pub enum DialogEasing {
    /// Linear interpolation (constant speed)
    Linear,
    /// Ease in (slow start, fast end)
    EaseIn,
    /// Ease out (fast start, slow end)
    EaseOut,
    /// Ease in-out (slow start and end, fast middle)
    EaseInOut,
    /// Bounce easing effect
    Bounce,
    /// Elastic easing effect
    Elastic,
    /// Custom easing function by name
    Custom(String),
}

/// Information about focusable elements
#[derive(Debug, Clone)]
pub struct FocusableElementInfo {
    /// Element ID
    pub id: String,
    /// Element type
    pub element_type: FocusableElementType,
    /// Tab index for ordering
    pub tab_index: i32,
    /// Whether element is currently enabled
    pub enabled: bool,
    /// Element bounds
    pub bounds: Rect,
    /// Custom properties
    pub properties: std::collections::HashMap<String, String>,
}

/// Types of focusable elements in dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum FocusableElementType {
    /// Button element
    Button,
    /// Text input field
    Input,
    /// Multi-line text area
    Textarea,
    /// Checkbox input
    Checkbox,
    /// Radio button input
    Radio,
    /// Select dropdown
    Select,
    /// Clickable link
    Link,
    /// Tab navigation element
    Tab,
    /// Menu item element
    MenuItem,
    /// Custom focusable element
    Custom(String),
}

/// Validation result for dialog inputs
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Error messages by field
    pub errors: std::collections::HashMap<String, String>,
    /// Warning messages by field
    pub warnings: std::collections::HashMap<String, String>,
    /// Custom validation data
    pub data: Option<String>,
}

/// Base dialog state that all dialogs can extend
#[derive(Debug, Clone)]
pub struct BaseDialogState {
    /// Dialog ID
    pub id: DialogId,
    /// Whether dialog is visible
    pub visible: bool,
    /// Whether dialog is focused
    pub focused: bool,
    /// Current animation state
    pub animation_state: DialogAnimationState,
    /// Animation start time
    pub animation_start: Option<Instant>,
    /// Currently focused element ID
    pub focused_element: Option<String>,
    /// Dialog bounds
    pub bounds: Rect,
    /// Whether dialog is being dragged
    pub dragging: bool,
    /// Drag offset
    pub drag_offset: Point,
    /// Whether dialog is being resized
    pub resizing: bool,
    /// Resize handle being used
    pub resize_handle: Option<ResizeHandle>,
}

/// Dialog animation states
#[derive(Debug, Clone, PartialEq)]
pub enum DialogAnimationState {
    /// Dialog is hidden
    Hidden,
    /// Dialog is animating in
    ShowingIn,
    /// Dialog is fully visible
    Visible,
    /// Dialog is animating out
    HidingOut,
}

/// Resize handles for resizable dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeHandle {
    /// Top-left corner handle
    TopLeft,
    /// Top edge handle
    Top,
    /// Top-right corner handle
    TopRight,
    /// Right edge handle
    Right,
    /// Bottom-right corner handle
    BottomRight,
    /// Bottom edge handle
    Bottom,
    /// Bottom-left corner handle
    BottomLeft,
    /// Left edge handle
    Left,
}

impl Default for DialogBounds {
    fn default() -> Self {
        Self {
            size: None,
            min_size: Some(Size::new(200, 100)),
            max_size: None,
            position: DialogPosition::Center,
            resizable: false,
            draggable: false,
            margin: DialogMargin::default(),
        }
    }
}

impl Default for DialogMargin {
    fn default() -> Self {
        Self {
            top: 20,
            right: 20,
            bottom: 20,
            left: 20,
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true,
            errors: std::collections::HashMap::new(),
            warnings: std::collections::HashMap::new(),
            data: None,
        }
    }
}

impl BaseDialogState {
    /// Create a new dialog state
    pub fn new(id: DialogId) -> Self {
        Self {
            id,
            visible: false,
            focused: false,
            animation_state: DialogAnimationState::Hidden,
            animation_start: None,
            focused_element: None,
            bounds: Rect::default(),
            dragging: false,
            drag_offset: Point::default(),
            resizing: false,
            resize_handle: None,
        }
    }

    /// Show the dialog with animation
    pub fn show(&mut self) {
        self.visible = true;
        self.animation_state = DialogAnimationState::ShowingIn;
        self.animation_start = Some(Instant::now());
    }

    /// Hide the dialog with animation
    pub fn hide(&mut self) {
        self.animation_state = DialogAnimationState::HidingOut;
        self.animation_start = Some(Instant::now());
    }

    /// Check if the dialog is currently animating
    pub fn is_animating(&self) -> bool {
        matches!(
            self.animation_state,
            DialogAnimationState::ShowingIn | DialogAnimationState::HidingOut
        )
    }
}
