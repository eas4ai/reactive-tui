//! Animation system FFI functions

use super::*;
use crate::animation::{
    AnimatedProperty, Animation, AnimationBuilder, AnimationManager, EasingFunction, LoopMode,
    SpringConfig, TransformProperty,
};
use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::time::Duration;

/// Opaque handle to an animation
#[repr(C)]
pub struct RTuiAnimation {
    _private: [u8; 0],
}

/// Opaque handle to an animation manager
#[repr(C)]
pub struct RTuiAnimationManager {
    _private: [u8; 0],
}

/// Easing function types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiEasingType {
    Linear = 0,
    EaseIn = 1,
    EaseOut = 2,
    EaseInOut = 3,
    Bounce = 4,
    Elastic = 5,
    Back = 6,
    Expo = 7,
    Circ = 8,
    Sine = 9,
    Quad = 10,
    Cubic = 11,
    Quart = 12,
    Quint = 13,
    Spring = 14,
}

/// Loop behavior types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiLoopMode {
    None = 0,
    Infinite = 1,
    Count = 2,
    PingPong = 3,
}

/// Animation property types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiAnimatedProperty {
    Opacity = 0,
    TranslateX = 1,
    TranslateY = 2,
    ScaleX = 3,
    ScaleY = 4,
    Rotation = 5,
    Width = 6,
    Height = 7,
}

/// Animation callback function type
pub type RTuiAnimationCallback =
    extern "C" fn(animation_id: *const c_char, progress: f32, user_data: *mut std::ffi::c_void);

/// Animation completion callback function type
pub type RTuiAnimationCompleteCallback =
    extern "C" fn(animation_id: *const c_char, user_data: *mut std::ffi::c_void);

/// Create a new animation manager
#[no_mangle]
pub extern "C" fn rtui_animation_manager_create(
    out_manager: *mut *mut RTuiAnimationManager,
) -> ReactiveError {
    if out_manager.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let manager = AnimationManager::new();
        let boxed = Box::new(manager);
        unsafe {
            *out_manager = Box::into_raw(boxed) as *mut RTuiAnimationManager;
        }
        Ok(())
    }))
}

/// Destroy an animation manager
#[no_mangle]
pub extern "C" fn rtui_animation_manager_destroy(manager: *mut RTuiAnimationManager) {
    if !manager.is_null() {
        unsafe {
            let _ = Box::from_raw(manager as *mut AnimationManager);
        }
    }
}

/// Update the animation manager
#[no_mangle]
pub extern "C" fn rtui_animation_manager_update(
    manager: *mut RTuiAnimationManager,
) -> ReactiveError {
    if manager.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let manager_ref = &mut *(manager as *mut AnimationManager);
        manager_ref.update();
        Ok(())
    }))
}

/// Create a new animation
#[no_mangle]
pub extern "C" fn rtui_animation_create(
    id: *const c_char,
    duration_ms: u32,
    easing: RTuiEasingType,
    loop_mode: RTuiLoopMode,
    loop_count: u32,
    out_animation: *mut *mut RTuiAnimation,
) -> ReactiveError {
    if id.is_null() || out_animation.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let id_str = CStr::from_ptr(id)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;
        let duration = Duration::from_millis(duration_ms as u64);

        let easing_fn = match easing {
            RTuiEasingType::Linear => EasingFunction::Linear,
            RTuiEasingType::EaseIn => EasingFunction::EaseIn,
            RTuiEasingType::EaseOut => EasingFunction::EaseOut,
            RTuiEasingType::EaseInOut => EasingFunction::EaseInOut,
            RTuiEasingType::Bounce => EasingFunction::Bounce,
            RTuiEasingType::Elastic => EasingFunction::Elastic,
            RTuiEasingType::Back => EasingFunction::Back,
            RTuiEasingType::Expo => EasingFunction::Expo,
            RTuiEasingType::Circ => EasingFunction::Circ,
            RTuiEasingType::Sine => EasingFunction::Sine,
            RTuiEasingType::Quad => EasingFunction::Quad,
            RTuiEasingType::Cubic => EasingFunction::Cubic,
            RTuiEasingType::Quart => EasingFunction::Quart,
            RTuiEasingType::Quint => EasingFunction::Quint,
            RTuiEasingType::Spring => EasingFunction::Spring(SpringConfig::default()),
        };

        let loop_mode_enum = match loop_mode {
            RTuiLoopMode::None => LoopMode::None,
            RTuiLoopMode::Infinite => LoopMode::Infinite,
            RTuiLoopMode::Count => LoopMode::Count(loop_count),
            RTuiLoopMode::PingPong => LoopMode::PingPong,
        };

        let animation = AnimationBuilder::new(id_str)
            .duration(duration)
            .easing(easing_fn)
            .loop_mode(loop_mode_enum)
            .build();

        let boxed = Box::new(animation);
        *out_animation = Box::into_raw(boxed) as *mut RTuiAnimation;
        Ok(())
    }))
}

/// Destroy an animation
#[no_mangle]
pub extern "C" fn rtui_animation_destroy(animation: *mut RTuiAnimation) {
    if !animation.is_null() {
        unsafe {
            let _ = Box::from_raw(animation as *mut Animation);
        }
    }
}

/// Set animation property
#[no_mangle]
pub extern "C" fn rtui_animation_set_property(
    animation: *mut RTuiAnimation,
    property: RTuiAnimatedProperty,
    from_value: f32,
    to_value: f32,
) -> ReactiveError {
    if animation.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let animation_ref = &mut *(animation as *mut Animation);

        let animated_property = match property {
            RTuiAnimatedProperty::Opacity => AnimatedProperty::Opacity(from_value, to_value),
            RTuiAnimatedProperty::TranslateX => {
                AnimatedProperty::Transform(TransformProperty::TranslateX(from_value, to_value))
            }
            RTuiAnimatedProperty::TranslateY => {
                AnimatedProperty::Transform(TransformProperty::TranslateY(from_value, to_value))
            }
            RTuiAnimatedProperty::ScaleX => {
                AnimatedProperty::Transform(TransformProperty::ScaleX(from_value, to_value))
            }
            RTuiAnimatedProperty::ScaleY => {
                AnimatedProperty::Transform(TransformProperty::ScaleY(from_value, to_value))
            }
            RTuiAnimatedProperty::Rotation => AnimatedProperty::Rotation(from_value, to_value),
            RTuiAnimatedProperty::Width => {
                AnimatedProperty::Size(from_value as u16, 0, to_value as u16, 0)
            }
            RTuiAnimatedProperty::Height => {
                AnimatedProperty::Size(0, from_value as u16, 0, to_value as u16)
            }
        };

        animation_ref.property = animated_property;
        Ok(())
    }))
}

/// Add animation to manager
#[no_mangle]
pub extern "C" fn rtui_animation_manager_add(
    manager: *mut RTuiAnimationManager,
    animation: *mut RTuiAnimation,
    out_id: *mut *mut c_char,
) -> ReactiveError {
    if manager.is_null() || animation.is_null() || out_id.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let manager_ref = &mut *(manager as *mut AnimationManager);
        let animation_obj = Box::from_raw(animation as *mut Animation);

        let id = manager_ref.add_animation(*animation_obj);
        let c_string = CString::new(id).map_err(|_| ReactiveError::InvalidUtf8)?;
        *out_id = c_string.into_raw();
        Ok(())
    }))
}

/// Remove animation from manager
#[no_mangle]
pub extern "C" fn rtui_animation_manager_remove(
    manager: *mut RTuiAnimationManager,
    animation_id: *const c_char,
) -> ReactiveError {
    if manager.is_null() || animation_id.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let manager_ref = &mut *(manager as *mut AnimationManager);
        let id_str = CStr::from_ptr(animation_id)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        manager_ref.remove_animation(&id_str.to_string());
        Ok(())
    }))
}

/// Play an animation
#[no_mangle]
pub extern "C" fn rtui_animation_play(animation: *mut RTuiAnimation) -> ReactiveError {
    if animation.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let animation_ref = &mut *(animation as *mut Animation);
        animation_ref.play();
        Ok(())
    }))
}

/// Pause an animation
#[no_mangle]
pub extern "C" fn rtui_animation_pause(animation: *mut RTuiAnimation) -> ReactiveError {
    if animation.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let animation_ref = &mut *(animation as *mut Animation);
        animation_ref.pause();
        Ok(())
    }))
}

/// Stop an animation
#[no_mangle]
pub extern "C" fn rtui_animation_stop(animation: *mut RTuiAnimation) -> ReactiveError {
    if animation.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let animation_ref = &mut *(animation as *mut Animation);
        animation_ref.stop();
        Ok(())
    }))
}

/// Check if animation is playing
#[no_mangle]
pub extern "C" fn rtui_animation_is_playing(
    animation: *const RTuiAnimation,
    out_playing: *mut bool,
) -> ReactiveError {
    if animation.is_null() || out_playing.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let animation_ref = &*(animation as *const Animation);
        *out_playing = animation_ref.is_playing();
        Ok(())
    }))
}

/// Get animation progress (0.0 to 1.0)
#[no_mangle]
pub extern "C" fn rtui_animation_get_progress(
    animation: *const RTuiAnimation,
    out_progress: *mut f32,
) -> ReactiveError {
    if animation.is_null() || out_progress.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let animation_ref = &*(animation as *const Animation);
        if let Ok(state) = animation_ref.state.read() {
            *out_progress = state.progress;
        } else {
            *out_progress = 0.0;
        }
        Ok(())
    }))
}

/// Create a spring animation
#[no_mangle]
pub extern "C" fn rtui_animation_create_spring(
    id: *const c_char,
    stiffness: f32,
    damping: f32,
    mass: f32,
    out_animation: *mut *mut RTuiAnimation,
) -> ReactiveError {
    if id.is_null() || out_animation.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let id_str = CStr::from_ptr(id)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let spring_config = SpringConfig {
            stiffness,
            damping,
            mass,
            velocity: 0.0,
            precision: 0.01,
        };

        let animation = AnimationBuilder::new(id_str)
            .duration(Duration::from_millis(1000)) // Default duration for spring
            .easing(EasingFunction::Spring(spring_config))
            .build();

        let boxed = Box::new(animation);
        *out_animation = Box::into_raw(boxed) as *mut RTuiAnimation;
        Ok(())
    }))
}
