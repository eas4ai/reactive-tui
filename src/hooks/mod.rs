pub mod animation;
pub mod clipboard;
pub mod fps;
pub mod mouse;
pub mod processor;
pub mod refs;
pub mod timer;
pub mod perf_context;


pub use clipboard::{ClipboardState, use_clipboard, use_simple_clipboard};

pub use fps::{
    FpsState, FrameTiming, QualityLevel, use_adaptive_quality, use_fps, use_frame_timing,
    use_performance, use_performance_mode,
};

pub use mouse::{
    ClickState, DragAndDropOptions, DragAndDropState, DragState, GestureState, GestureType,
    HoverState, LongPressState, MousePositionState, SwipeDirection, WheelDeltaMode, WheelState,
    use_clicks, use_drag, use_drag_and_drop, use_gesture, use_hover, use_long_press,
    use_mouse_position, use_wheel,
};

pub use processor::MouseEventProcessor;

pub use refs::{
    CallbackRef, ForwardedRef, LocalRef, MultiRef, Ref, use_callback_ref, use_forwarded_ref,
    use_local_ref, use_multi_ref, use_ref,
};

pub use timer::{
    DebouncedFunction, ThrottledFunction, TimerHandle, use_debounce, use_interval, use_throttle,
    use_timeout,
};

pub use animation::{
    AnimatableValue, AnimationConfig, AnimationHandle, KeyframeHandle, SpringHandle, StaggerHandle,
    TransitionConfig, use_animation, use_keyframes, use_spring, use_stagger, use_transition,
};
