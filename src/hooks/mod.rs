pub mod animation;
pub mod clipboard;
pub mod fps;
pub mod mouse;
pub mod perf_context;
pub mod processor;
pub mod refs;
pub mod timer;

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
