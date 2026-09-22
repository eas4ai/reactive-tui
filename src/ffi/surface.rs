//! Surface FFI functions

use super::*;
use crate::core::surface::Surface;
use crate::ffi::pointer::*;
use std::boxed::Box;
use std::ffi::c_char;

/// Create a new surface
#[no_mangle]
pub extern "C" fn rtui_surface_create(
    width: u16,
    height: u16,
    out_surface: *mut *mut RTuiSurface,
) -> ReactiveError {
    if out_surface.is_null() {
        return ReactiveError::NullPointer;
    }

    if width == 0 || height == 0 {
        return ReactiveError::InvalidParameter;
    }

    catch_panic(AssertUnwindSafe(|| {
        let surface = Surface::new(width as usize, height as usize);
        let boxed = Box::new(surface);
        let raw_ptr = Box::into_raw(boxed);

        // Register the pointer to track its lifetime
        if !trackers::surface_tracker().register(raw_ptr) {
            // Failed to register, clean up and return error
            unsafe {
                let _ = Box::from_raw(raw_ptr);
            }
            return Err(ReactiveError::InternalError);
        }

        unsafe {
            *out_surface = raw_ptr as *mut RTuiSurface;
        }
        Ok(())
    }))
}

/// Destroy a surface
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_surface_destroy(surface: *mut RTuiSurface) {
    if surface.is_null() {
        return;
    }

    let surf_ptr = surface as *mut Surface;

    // Only caller-owned surfaces can be destroyed through this API.
    if !trackers::surface_tracker().unregister(surf_ptr) {
        return;
    }

    unsafe {
        let _ = Box::from_raw(surf_ptr);
    }
}

/// Get surface dimensions
#[no_mangle]
pub extern "C" fn rtui_surface_get_dimensions(
    surface: *const RTuiSurface,
    out_dimensions: *mut RTuiDimensions,
) -> ReactiveError {
    if surface.is_null() || out_dimensions.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let surf_ptr = surface as *const Surface;

        // Validate pointer before use
        if !trackers::is_valid_surface(surf_ptr) {
            return Err(ReactiveError::InvalidPointer);
        }

        let surf = &*surf_ptr;
        let (width, height) = surf.dims();
        *out_dimensions = RTuiDimensions {
            width: width as u16,
            height: height as u16,
        };
        Ok(())
    }))
}

/// Clear the surface
#[no_mangle]
pub extern "C" fn rtui_surface_clear(
    surface: *mut RTuiSurface,
    r: u8,
    g: u8,
    b: u8,
) -> ReactiveError {
    if surface.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let surf_ptr = surface as *mut Surface;

        // Validate pointer before use
        if !trackers::is_valid_surface(surf_ptr) {
            return Err(ReactiveError::InvalidPointer);
        }

        let surf = &mut *surf_ptr;
        let bg = crate::core::surface::Rgba {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        };
        surf.clear(bg);
        Ok(())
    }))
}

/// Set a cell on the surface
#[no_mangle]
pub extern "C" fn rtui_surface_set_cell(
    surface: *mut RTuiSurface,
    x: u16,
    y: u16,
    cell: *const RTuiCell,
) -> ReactiveError {
    if surface.is_null() || cell.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        unsafe {
            let surf_ptr = surface as *mut Surface;

            // Validate pointer before use
            if !trackers::is_valid_surface(surf_ptr) {
                return Err(ReactiveError::InvalidPointer);
            }

            let surf = &mut *surf_ptr;
            let c = &*cell;

            // Convert FFI cell to internal cell
            let ch = if c.ch == 0 {
                ' '
            } else {
                std::char::from_u32(c.ch).unwrap_or('?')
            };

            let internal_cell = crate::core::surface::Cell {
                ch,
                fg: crate::core::surface::Rgba {
                    r: c.fg.r as f32 / 255.0,
                    g: c.fg.g as f32 / 255.0,
                    b: c.fg.b as f32 / 255.0,
                    a: 1.0,
                },
                bg: crate::core::surface::Rgba {
                    r: c.bg.r as f32 / 255.0,
                    g: c.bg.g as f32 / 255.0,
                    b: c.bg.b as f32 / 255.0,
                    a: 1.0,
                },
                attr: crate::core::surface::Attr::from_flags(
                    c.attrs.bold,
                    c.attrs.italic,
                    c.attrs.underline,
                    c.attrs.reverse,
                    c.attrs.strikethrough,
                ),
                image_id: None,
                image_placement: None,
            };

            surf.set(x as usize, y as usize, internal_cell);
            Ok(())
        }
    }))
}

/// Get a cell from the surface
#[no_mangle]
pub extern "C" fn rtui_surface_get_cell(
    surface: *const RTuiSurface,
    x: u16,
    y: u16,
    out_cell: *mut RTuiCell,
) -> ReactiveError {
    if surface.is_null() || out_cell.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let surf_ptr = surface as *const Surface;

        // Validate pointer before use
        if !trackers::is_valid_surface(surf_ptr) {
            return Err(ReactiveError::InvalidPointer);
        }

        let surf = &*surf_ptr;
        let cell = surf.get(x as usize, y as usize);

        *out_cell = RTuiCell {
            ch: cell.ch as u32,
            fg: RTuiColor {
                r: (cell.fg.r * 255.0) as u8,
                g: (cell.fg.g * 255.0) as u8,
                b: (cell.fg.b * 255.0) as u8,
            },
            bg: RTuiColor {
                r: (cell.bg.r * 255.0) as u8,
                g: (cell.bg.g * 255.0) as u8,
                b: (cell.bg.b * 255.0) as u8,
            },
            attrs: RTuiTextAttributes {
                bold: cell.attr.contains(crate::core::surface::Attr::BOLD),
                italic: cell.attr.contains(crate::core::surface::Attr::ITALIC),
                underline: cell.attr.contains(crate::core::surface::Attr::UNDERLINE),
                strikethrough: cell.attr.contains(crate::core::surface::Attr::STRIKE),
                reverse: cell.attr.contains(crate::core::surface::Attr::REVERSE),
                blink: false,
                hidden: false,
            },
        };
        Ok(())
    }))
}

/// Draw text on the surface
#[no_mangle]
pub extern "C" fn rtui_surface_draw_text(
    surface: *mut RTuiSurface,
    x: u16,
    y: u16,
    text: *const c_char,
    fg: *const RTuiColor,
    bg: *const RTuiColor,
) -> ReactiveError {
    if surface.is_null() || text.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let surf_ptr = surface as *mut Surface;

        // Validate pointer before use
        if !trackers::is_valid_surface(surf_ptr) {
            return Err(ReactiveError::InvalidPointer);
        }

        let surf = &mut *surf_ptr;
        let text_str = c_str_to_string(text)?;

        let fg_rgba = if fg.is_null() {
            crate::core::surface::Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
        } else {
            let color = &*fg;
            crate::core::surface::Rgba {
                r: color.r as f32 / 255.0,
                g: color.g as f32 / 255.0,
                b: color.b as f32 / 255.0,
                a: 1.0,
            }
        };

        let bg_rgba = if bg.is_null() {
            crate::core::surface::Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else {
            let color = &*bg;
            crate::core::surface::Rgba {
                r: color.r as f32 / 255.0,
                g: color.g as f32 / 255.0,
                b: color.b as f32 / 255.0,
                a: 1.0,
            }
        };

        surf.write_str(
            x as usize,
            y as usize,
            &text_str,
            fg_rgba,
            bg_rgba,
            crate::core::surface::Attr::empty(),
        );

        Ok(())
    }))
}

/// Fill a rectangle on the surface
#[no_mangle]
pub extern "C" fn rtui_surface_fill_rect(
    surface: *mut RTuiSurface,
    rect: *const RTuiRect,
    ch: u32,
    fg: *const RTuiColor,
    bg: *const RTuiColor,
) -> ReactiveError {
    if surface.is_null() || rect.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let surf_ptr = surface as *mut Surface;

        // Validate pointer before use
        if !trackers::is_valid_surface(surf_ptr) {
            return Err(ReactiveError::InvalidPointer);
        }

        let surf = &mut *surf_ptr;
        let r = &*rect;
        let fill_char = std::char::from_u32(ch).unwrap_or(' ');
        let (width, height) = surf.dims();

        let fg_rgba = if fg.is_null() {
            crate::core::surface::Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
        } else {
            let color = &*fg;
            crate::core::surface::Rgba {
                r: color.r as f32 / 255.0,
                g: color.g as f32 / 255.0,
                b: color.b as f32 / 255.0,
                a: 1.0,
            }
        };

        let bg_rgba = if bg.is_null() {
            crate::core::surface::Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else {
            let color = &*bg;
            crate::core::surface::Rgba {
                r: color.r as f32 / 255.0,
                g: color.g as f32 / 255.0,
                b: color.b as f32 / 255.0,
                a: 1.0,
            }
        };

        let cell = crate::core::surface::Cell {
            ch: fill_char,
            fg: fg_rgba,
            bg: bg_rgba,
            attr: crate::core::surface::Attr::empty(),
            image_id: None,
            image_placement: None,
        };

        for y in r.y..(r.y + r.height).min(height as u16) {
            for x in r.x..(r.x + r.width).min(width as u16) {
                surf.set(x as usize, y as usize, cell);
            }
        }

        Ok(())
    }))
}
