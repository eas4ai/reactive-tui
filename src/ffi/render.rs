//! Renderer FFI functions

use super::*;
use crate::core::renderer::Renderer;
use crate::core::surface::{Rgba, Surface};
use std::boxed::Box;

/// Create a new renderer
#[no_mangle]
pub extern "C" fn rtui_renderer_create(
    width: u16,
    height: u16,
    out_renderer: *mut *mut RTuiRenderer,
) -> ReactiveError {
    if out_renderer.is_null() {
        return ReactiveError::NullPointer;
    }

    if width == 0 || height == 0 {
        return ReactiveError::InvalidParameter;
    }

    catch_panic(AssertUnwindSafe(|| {
        match Renderer::new(width as usize, height as usize) {
            Ok(renderer) => {
                let raw = Box::into_raw(Box::new(renderer));
                if !pointer::trackers::renderer_tracker().register(raw) {
                    unsafe {
                        drop(Box::from_raw(raw));
                    }
                    return Err(ReactiveError::InternalError);
                }
                unsafe {
                    *out_renderer = raw.cast();
                }
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }))
}

/// Destroy a renderer
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_renderer_destroy(renderer: *mut RTuiRenderer) {
    let raw = renderer.cast::<Renderer>();
    if !pointer::trackers::renderer_tracker().unregister(raw) {
        return;
    }
    unsafe {
        let mut renderer = Box::from_raw(raw);
        pointer::trackers::borrowed_surface_tracker().unregister(renderer.surface_mut());
        drop(renderer);
    }
}

/// Resize the renderer
#[no_mangle]
pub extern "C" fn rtui_renderer_resize(
    renderer: *mut RTuiRenderer,
    width: u16,
    height: u16,
) -> ReactiveError {
    if renderer.is_null() {
        return ReactiveError::NullPointer;
    }

    if width == 0 || height == 0 {
        return ReactiveError::InvalidParameter;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        if !pointer::trackers::renderer_tracker().is_valid(renderer.cast::<Renderer>()) {
            return Err(ReactiveError::InvalidPointer);
        }
        let r = &mut *(renderer as *mut Renderer);
        r.resize(width as usize, height as usize);
        Ok(())
    }))
}

/// Clear the renderer with a color
#[no_mangle]
pub extern "C" fn rtui_renderer_clear(
    renderer: *mut RTuiRenderer,
    r: u8,
    g: u8,
    b: u8,
) -> ReactiveError {
    if renderer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Validate pointer type before casting
        if !pointer::trackers::renderer_tracker().is_valid(renderer.cast::<Renderer>()) {
            return Err(ReactiveError::InvalidParameter);
        }
        let ren = &mut *(renderer as *mut Renderer);
        let color = Rgba {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        };
        ren.clear(color);
        Ok(())
    }))
}

/// Control frame rendering
/// @param begin: true to begin frame, false to end frame
#[no_mangle]
pub extern "C" fn rtui_renderer_frame(renderer: *mut RTuiRenderer, begin: bool) -> ReactiveError {
    if renderer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        if !pointer::trackers::renderer_tracker().is_valid(renderer.cast::<Renderer>()) {
            return Err(ReactiveError::InvalidPointer);
        }
        let r = &mut *(renderer as *mut Renderer);
        if begin {
            match r.begin_frame() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.into()),
            }
        } else {
            match r.end_frame() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }
    }))
}

/// Borrow the drawing surface until renderer shutdown or destruction.
/// The caller must serialize access with renderer operations and must not free it.
#[no_mangle]
pub extern "C" fn rtui_renderer_get_surface(
    renderer: *mut RTuiRenderer,
    out_surface: *mut *mut RTuiSurface,
) -> ReactiveError {
    if renderer.is_null() || out_surface.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        if !pointer::trackers::renderer_tracker().is_valid(renderer.cast::<Renderer>()) {
            return Err(ReactiveError::InvalidPointer);
        }
        let r = &mut *(renderer as *mut Renderer);
        let surface_ptr = r.surface_mut() as *mut Surface;
        if !pointer::trackers::borrowed_surface_tracker().register(surface_ptr) {
            return Err(ReactiveError::InternalError);
        }
        *out_surface = surface_ptr as *mut RTuiSurface;
        Ok(())
    }))
}

/// Restore the terminal without freeing the handle; call destroy afterward.
#[no_mangle]
pub extern "C" fn rtui_renderer_shutdown(renderer: *mut RTuiRenderer) -> ReactiveError {
    if renderer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        if !pointer::trackers::renderer_tracker().is_valid(renderer.cast::<Renderer>()) {
            return Err(ReactiveError::InvalidPointer);
        }
        let r = &mut *(renderer as *mut Renderer);
        pointer::trackers::borrowed_surface_tracker().unregister(r.surface_mut());
        match r.restore_terminal() {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }))
}
