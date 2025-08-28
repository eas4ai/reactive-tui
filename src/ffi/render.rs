//! Renderer FFI functions

use super::*;
use crate::core::renderer::Renderer;
use crate::core::surface::{Rgba, Surface};
use std::boxed::Box;

/// Create a new renderer
#[unsafe(no_mangle)]
pub extern "C" fn rtui_renderer_create(
    width: u16,
    height: u16,
    out_renderer: *mut *mut RTuiRenderer,
) -> RTuiError {
    if out_renderer.is_null() {
        return RTuiError::NullPointer;
    }
    
    if width == 0 || height == 0 {
        return RTuiError::InvalidParameter;
    }

    catch_panic(AssertUnwindSafe(|| {
        match Renderer::new(width as usize, height as usize) {
            Ok(renderer) => {
                let boxed = Box::new(renderer);
                unsafe {
                    *out_renderer = Box::into_raw(boxed) as *mut RTuiRenderer;
                }
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }))
    
}

/// Destroy a renderer
#[unsafe(no_mangle)]
pub extern "C" fn rtui_renderer_destroy(renderer: *mut RTuiRenderer) {
    if !renderer.is_null() {
        unsafe {
            let r = Box::from_raw(renderer as *mut Renderer);
            // Renderer's drop will call shutdown
            drop(r);
        }
    }
}

/// Resize the renderer
#[unsafe(no_mangle)]
pub extern "C" fn rtui_renderer_resize(
    renderer: *mut RTuiRenderer,
    width: u16,
    height: u16,
) -> RTuiError {
    if renderer.is_null() {
        return RTuiError::NullPointer;
    }
    
    if width == 0 || height == 0 {
        return RTuiError::InvalidParameter;
    }

    catch_panic(AssertUnwindSafe(|| {
        unsafe {
            let r = &mut *(renderer as *mut Renderer);
            r.resize(width as usize, height as usize);
            Ok(())
        }
    }))
    
}

/// Clear the renderer with a color
#[unsafe(no_mangle)]
pub extern "C" fn rtui_renderer_clear(
    renderer: *mut RTuiRenderer,
    r: u8,
    g: u8,
    b: u8,
) -> RTuiError {
    if renderer.is_null() {
        return RTuiError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        unsafe {
            let ren = &mut *(renderer as *mut Renderer);
            let color = Rgba {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            };
            ren.clear(color);
            Ok(())
        }
    }))
    
}

/// Control frame rendering
/// @param begin: true to begin frame, false to end frame
#[unsafe(no_mangle)]
pub extern "C" fn rtui_renderer_frame(renderer: *mut RTuiRenderer, begin: bool) -> RTuiError {
    if renderer.is_null() {
        return RTuiError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        unsafe {
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
        }
    }))
}

/// Get the surface for drawing
#[unsafe(no_mangle)]
pub extern "C" fn rtui_renderer_get_surface(
    renderer: *mut RTuiRenderer,
    out_surface: *mut *mut RTuiSurface,
) -> RTuiError {
    if renderer.is_null() || out_surface.is_null() {
        return RTuiError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        unsafe {
            let r = &mut *(renderer as *mut Renderer);
            let surface_ptr = r.surface_mut() as *mut Surface;
            *out_surface = surface_ptr as *mut RTuiSurface;
            Ok(())
        }
    }))
    
}

/// Shutdown the renderer and exit raw mode
#[unsafe(no_mangle)]
pub extern "C" fn rtui_renderer_shutdown(renderer: *mut RTuiRenderer) -> RTuiError {
    if renderer.is_null() {
        return RTuiError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        unsafe {
            let r = Box::from_raw(renderer as *mut Renderer);
            match r.shutdown() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }
    }))
    
}