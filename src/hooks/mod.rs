//! React-style hooks for state and lifecycle management
//!
//! This module provides React-like hooks for managing state, effects, and component
//! lifecycle in terminal applications.

/// Animation hooks for managing animated properties
pub mod animation;
/// Clipboard hooks for copy/paste operations
pub mod clipboard;
mod clipboard_process;
/// Frame rate monitoring hooks
pub mod fps;
/// Mouse interaction hooks
pub mod mouse;
/// Performance context and monitoring hooks
pub mod perf_context;
/// Message processor hooks for event handling
pub mod processor;
/// Reference hooks for accessing DOM elements
pub mod refs;
/// Timer and interval hooks
pub mod timer;

pub use crate::reactive::local_hooks::with_local_hooks;

pub use clipboard::{use_clipboard, use_simple_clipboard, ClipboardState};

pub use fps::{
    use_adaptive_quality, use_fps, use_frame_timing, use_performance, use_performance_mode,
    FpsState, FrameTiming, QualityLevel,
};

pub use mouse::{
    use_clicks, use_drag, use_drag_and_drop, use_gesture, use_hover, use_long_press,
    use_mouse_position, use_wheel, ClickState, DragAndDropOptions, DragAndDropState, DragState,
    GestureState, GestureType, HoverState, LongPressState, MousePositionState, SwipeDirection,
    WheelDeltaMode, WheelState,
};

pub use processor::MouseEventProcessor;

pub use refs::{
    use_callback_ref, use_forwarded_ref, use_local_ref, use_multi_ref, use_ref, CallbackRef,
    ForwardedRef, LocalRef, MultiRef, Ref,
};

pub use timer::{
    use_debounce, use_interval, use_throttle, use_timeout, DebouncedFunction, ThrottledFunction,
    TimerHandle,
};

pub use animation::{
    use_animation, use_keyframes, use_spring, use_stagger, use_transition, AnimatableValue,
    AnimationConfig, AnimationHandle, KeyframeHandle, SpringHandle, StaggerHandle,
    TransitionConfig,
};
