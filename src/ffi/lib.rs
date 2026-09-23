//! Main FFI interface - Modern API
//!
//! This module provides the primary FFI interface with modern patterns:
//! - Function naming without prefixes (createRenderer vs rtui_renderer_create)
//! - Direct memory access for performance
//! - Packed parameter passing
//! - Zero-copy operations where possible

use crate::core::renderer::Renderer;
use crate::core::surface::{Attr, Cell, Rgba, Surface};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// FFI Buffer handle (opaque pointer to Surface)
#[repr(C)]
pub struct RTuiBuffer {
    _private: [u8; 0],
}

fn optimized_buffer_tracker() -> &'static super::pointer::PointerTracker<Surface> {
    static TRACKER: OnceLock<super::pointer::PointerTracker<Surface>> = OnceLock::new();
    TRACKER.get_or_init(super::pointer::PointerTracker::new)
}

fn returned_allocations() -> &'static Mutex<HashMap<(usize, TypeId), usize>> {
    static ALLOCATIONS: OnceLock<Mutex<HashMap<(usize, TypeId), usize>>> = OnceLock::new();
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn export_allocation<T: 'static>(values: Box<[T]>) -> *mut T {
    let length = values.len();
    let pointer = Box::into_raw(values) as *mut T;
    returned_allocations()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert((pointer as usize, TypeId::of::<T>()), length);
    pointer
}

fn release_allocation<T: 'static>(pointer: *mut T) {
    let length = returned_allocations()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&(pointer as usize, TypeId::of::<T>()));
    if let Some(length) = length {
        unsafe {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                pointer, length,
            )));
        }
    }
}

// Re-export core types for OpenTUI-style API

pub use super::types::*;

/// Helper function to convert f32 array pointer to RGBA
pub(crate) fn f32_ptr_to_rgba(ptr: *const f32) -> Rgba {
    unsafe {
        Rgba::new(
            *ptr.offset(0),
            *ptr.offset(1),
            *ptr.offset(2),
            *ptr.offset(3),
        )
    }
}

//
// RENDERER MANAGEMENT
//

/// Create a new renderer
#[reactive_tui_macros::ffi_export]
pub extern "C" fn createRenderer(width: u32, height: u32) -> *mut RTuiRenderer {
    if width == 0 || height == 0 {
        return std::ptr::null_mut();
    }

    match Renderer::new(width as usize, height as usize) {
        Ok(renderer) => {
            let raw = Box::into_raw(Box::new(renderer));
            if !super::pointer::trackers::renderer_tracker().register(raw) {
                unsafe {
                    drop(Box::from_raw(raw));
                }
                return std::ptr::null_mut();
            }
            raw.cast()
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Destroy a renderer
#[reactive_tui_macros::ffi_export]
pub extern "C" fn destroyRenderer(
    renderer: *mut RTuiRenderer,
    _use_alternate_screen: bool,
    _split_height: u32,
) {
    if renderer.is_null() {
        return;
    }

    let renderer_ptr = renderer as *mut Renderer;
    if !super::pointer::trackers::renderer_tracker().unregister(renderer_ptr) {
        return;
    }
    unsafe {
        let mut renderer_box = Box::from_raw(renderer_ptr);
        super::pointer::trackers::borrowed_surface_tracker().unregister(renderer_box.surface_mut());

        // Handle alternate screen mode - the renderer is already initialized
        // with alternate screen mode in Renderer::new(), so we don't need
        // to explicitly enter it here. The terminal is properly managed
        // by the renderer's lifecycle.

        // Handle split height rendering - adjust viewport if needed
        let (width, height) = renderer_box.dims();
        if height > 0 {
            // For split screen scenarios, we could render only to a portion
            // of the available height. For now, use full height but this
            // provides a foundation for future split-screen functionality.
            //
            // Future enhancement: Add a parameter to control split height:
            // let render_height = if split_mode { height / 2 } else { height };

            // The renderer already handles the full viewport correctly,
            // so no additional viewport adjustment is needed here.
        }

        let _ = renderer_box.end_frame();
    }
}

/// Set renderer background color
#[reactive_tui_macros::ffi_export]
pub extern "C" fn setBackgroundColor(renderer: *mut RTuiRenderer, color: *const f32) {
    if renderer.is_null() || color.is_null() {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };
    let bg_color = f32_ptr_to_rgba(color);

    // Apply background color to renderer's surface
    renderer_ref.surface_mut().clear(bg_color);
}

/// Render the current frame
#[reactive_tui_macros::ffi_export]
pub extern "C" fn render(renderer: *mut RTuiRenderer, force: bool) {
    if renderer.is_null() {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };

    if force {
        let _ = renderer_ref.begin_frame();
    }
    let _ = renderer_ref.end_frame();
}

/// Resize renderer
#[reactive_tui_macros::ffi_export]
pub extern "C" fn resizeRenderer(renderer: *mut RTuiRenderer, width: u32, height: u32) {
    if renderer.is_null() || width == 0 || height == 0 {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };
    let _ = renderer_ref.resize(width as usize, height as usize);
}

//
// BUFFER MANAGEMENT
//

/// Create an optimized buffer
#[reactive_tui_macros::ffi_export]
pub extern "C" fn createOptimizedBuffer(
    width: u32,
    height: u32,
    _respect_alpha: bool,
    _width_method: u8,
    _id_ptr: *const u8,
    _id_len: usize,
) -> *mut RTuiBuffer {
    if width == 0 || height == 0 {
        return std::ptr::null_mut();
    }

    // Create surface (our buffer implementation)
    let surface = Surface::new(width as usize, height as usize);
    let raw = Box::into_raw(Box::new(surface));
    if !optimized_buffer_tracker().register(raw) {
        unsafe {
            drop(Box::from_raw(raw));
        }
        return std::ptr::null_mut();
    }
    raw.cast::<RTuiBuffer>()
}

/// Destroy an optimized buffer
#[reactive_tui_macros::ffi_export]
pub extern "C" fn destroyOptimizedBuffer(buffer: *mut RTuiBuffer) {
    if buffer.is_null() {
        return;
    }

    let buffer_ptr = buffer.cast::<Surface>();
    if optimized_buffer_tracker().unregister(buffer_ptr) {
        unsafe {
            let _ = Box::from_raw(buffer_ptr);
        }
    }
}

/// Get buffer width
#[reactive_tui_macros::ffi_export]
pub extern "C" fn getBufferWidth(buffer: *const RTuiBuffer) -> u32 {
    if buffer.is_null() {
        return 0;
    }

    let surface = unsafe { &*(buffer as *const Surface) };
    surface.size().width as u32
}

/// Get buffer height
#[reactive_tui_macros::ffi_export]
pub extern "C" fn getBufferHeight(buffer: *const RTuiBuffer) -> u32 {
    if buffer.is_null() {
        return 0;
    }

    let surface = unsafe { &*(buffer as *const Surface) };
    surface.size().height as u32
}

/// Clear buffer with background color
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferClear(buffer: *mut RTuiBuffer, bg: *const f32) {
    if buffer.is_null() || bg.is_null() {
        return;
    }

    let surface = unsafe { &mut *(buffer as *mut Surface) };
    let bg_color = f32_ptr_to_rgba(bg);
    surface.clear(bg_color);
}

/// Draw text to buffer
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferDrawText(
    buffer: *mut RTuiBuffer,
    text: *const u8,
    text_len: usize,
    x: u32,
    y: u32,
    fg: *const f32,
    bg: *const f32,
    attributes: u8,
) {
    if buffer.is_null() || text.is_null() || fg.is_null() {
        return;
    }

    let surface = unsafe { &mut *(buffer as *mut Surface) };
    let text_slice = unsafe { std::slice::from_raw_parts(text, text_len) };
    let text_str = match std::str::from_utf8(text_slice) {
        Ok(s) => s,
        Err(_) => return,
    };

    let fg_color = f32_ptr_to_rgba(fg);
    let bg_color = if bg.is_null() {
        Rgba::new(0.0, 0.0, 0.0, 0.0) // Transparent
    } else {
        f32_ptr_to_rgba(bg)
    };

    let attr = Attr::from_bits_truncate(attributes);

    // Draw each character
    for (i, ch) in text_str.chars().enumerate() {
        let cell = Cell {
            ch,
            fg: fg_color,
            bg: bg_color,
            attr,
            image_id: None,
            image_placement: None,
        };
        surface.set(x as usize + i, y as usize, cell);
    }
}

/// Set a single cell with alpha blending
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferSetCellWithAlphaBlending(
    buffer: *mut RTuiBuffer,
    x: u32,
    y: u32,
    character: u32,
    fg: *const f32,
    bg: *const f32,
    attributes: u8,
) {
    if buffer.is_null() || fg.is_null() || bg.is_null() {
        return;
    }

    let surface = unsafe { &mut *(buffer as *mut Surface) };
    let ch = char::from_u32(character).unwrap_or(' ');
    let fg_color = f32_ptr_to_rgba(fg);
    let bg_color = f32_ptr_to_rgba(bg);
    let attr = Attr::from_bits_truncate(attributes);

    let cell = Cell {
        ch,
        fg: fg_color,
        bg: bg_color,
        attr,
        image_id: None,
        image_placement: None,
    };

    surface.set(x as usize, y as usize, cell);
}

/// Fill rectangle with background color
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferFillRect(
    buffer: *mut RTuiBuffer,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    bg: *const f32,
) {
    if buffer.is_null() || bg.is_null() {
        return;
    }

    let surface = unsafe { &mut *(buffer as *mut Surface) };
    let bg_color = f32_ptr_to_rgba(bg);

    for dy in 0..height {
        for dx in 0..width {
            let cell = Cell {
                ch: ' ',
                fg: Rgba::new(1.0, 1.0, 1.0, 1.0),
                bg: bg_color,
                attr: Attr::empty(),
                image_id: None,
                image_placement: None,
            };
            surface.set((x + dx) as usize, (y + dy) as usize, cell);
        }
    }
}

//
// DIRECT MEMORY ACCESS
//

/// Get direct pointer to character buffer
///
/// Returns a pointer to a contiguous array of u32 characters.
/// The array has width * height elements in row-major order.
///
/// # Safety
/// - The returned pointer is valid until its matching release call
/// - The caller must not access beyond width * height elements
/// - Concurrent access must be synchronized by the caller
/// - The caller must call bufferReleaseCharPtr to free the memory
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferGetCharPtr(buffer: *mut RTuiBuffer) -> *mut u32 {
    if buffer.is_null() {
        return std::ptr::null_mut();
    }

    let surface = unsafe { &*(buffer as *const Surface) };

    // Extract characters into a contiguous u32 array
    let chars: Vec<u32> = surface.cells().iter().map(|cell| cell.ch as u32).collect();

    export_allocation(chars.into_boxed_slice())
}

/// Release character buffer pointer
///
/// **REQUIRED:** Must be called for every pointer returned by bufferGetCharPtr()
/// **SAFETY:** Only call this once per pointer, and only with pointers from bufferGetCharPtr()
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferReleaseCharPtr(ptr: *mut u32, _length: usize) {
    if ptr.is_null() {
        return;
    }

    release_allocation(ptr);
}

/// Get direct pointer to foreground color buffer
///
/// Returns a pointer to a contiguous array of f32 RGBA values.
/// The array has width * height * 4 elements (RGBA per cell) in row-major order.
///
/// # Safety
/// - The returned pointer is valid until its matching release call
/// - The caller must not access beyond width * height * 4 elements
/// - The caller must call bufferReleaseFgPtr to free the memory
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferGetFgPtr(buffer: *mut RTuiBuffer) -> *mut f32 {
    if buffer.is_null() {
        return std::ptr::null_mut();
    }

    let surface = unsafe { &*(buffer as *const Surface) };

    // Extract foreground colors into a contiguous f32 array (RGBA)
    let mut colors = Vec::with_capacity(surface.buffer_size() * 4);
    for cell in surface.cells() {
        colors.push(cell.fg.r);
        colors.push(cell.fg.g);
        colors.push(cell.fg.b);
        colors.push(cell.fg.a);
    }

    export_allocation(colors.into_boxed_slice())
}

/// Release foreground color buffer pointer
///
/// **REQUIRED:** Must be called for every pointer returned by bufferGetFgPtr()
/// **SAFETY:** Only call this once per pointer, and only with pointers from bufferGetFgPtr()
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferReleaseFgPtr(ptr: *mut f32, _length: usize) {
    if ptr.is_null() {
        return;
    }

    release_allocation(ptr);
}

/// Get direct pointer to background color buffer
///
/// Returns a pointer to a contiguous array of f32 RGBA values.
/// The array has width * height * 4 elements (RGBA per cell) in row-major order.
///
/// # Safety
/// - The returned pointer is valid until its matching release call
/// - The caller must not access beyond width * height * 4 elements
/// - The caller must call bufferReleaseBgPtr to free the memory
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferGetBgPtr(buffer: *mut RTuiBuffer) -> *mut f32 {
    if buffer.is_null() {
        return std::ptr::null_mut();
    }

    let surface = unsafe { &*(buffer as *const Surface) };

    // Extract background colors into a contiguous f32 array (RGBA)
    let mut colors = Vec::with_capacity(surface.buffer_size() * 4);
    for cell in surface.cells() {
        colors.push(cell.bg.r);
        colors.push(cell.bg.g);
        colors.push(cell.bg.b);
        colors.push(cell.bg.a);
    }

    export_allocation(colors.into_boxed_slice())
}

/// Release background color buffer pointer
///
/// **REQUIRED:** Must be called for every pointer returned by bufferGetBgPtr()
/// **SAFETY:** Only call this once per pointer, and only with pointers from bufferGetBgPtr()
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferReleaseBgPtr(ptr: *mut f32, _length: usize) {
    if ptr.is_null() {
        return;
    }

    release_allocation(ptr);
}

/// Get direct pointer to attributes buffer
///
/// Returns a pointer to a contiguous array of u8 attribute flags.
/// The array has width * height elements in row-major order.
///
/// # Safety
/// - The returned pointer is valid until its matching release call
/// - The caller must not access beyond width * height elements
/// - The caller must call bufferReleaseAttributesPtr to free the memory
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferGetAttributesPtr(buffer: *mut RTuiBuffer) -> *mut u8 {
    if buffer.is_null() {
        return std::ptr::null_mut();
    }

    let surface = unsafe { &*(buffer as *const Surface) };

    // Extract attributes into a contiguous u8 array
    let attrs: Vec<u8> = surface
        .cells()
        .iter()
        .map(|cell| cell.attr.bits())
        .collect();

    export_allocation(attrs.into_boxed_slice())
}

/// Release attributes buffer pointer
///
/// **REQUIRED:** Must be called for every pointer returned by bufferGetAttrPtr()
/// **SAFETY:** Only call this once per pointer, and only with pointers from bufferGetAttrPtr()
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferReleaseAttrPtr(ptr: *mut u8, _length: usize) {
    if ptr.is_null() {
        return;
    }

    release_allocation(ptr);
}

/// Get buffer respect alpha setting
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferGetRespectAlpha(buffer: *const RTuiBuffer) -> bool {
    if buffer.is_null() {
        return false;
    }

    // Surface always respects alpha
    true
}

/// Set buffer respect alpha setting
///
/// Controls whether alpha blending is respected when rendering.
/// When enabled, transparent colors will blend with background.
/// When disabled, alpha values are ignored and colors are rendered opaque.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferSetRespectAlpha(buffer: *mut RTuiBuffer, respect_alpha: bool) {
    if buffer.is_null() {
        return;
    }

    // Surface doesn't currently have a respect_alpha field, but we can
    // implement alpha handling by modifying cell alpha values
    let surface = unsafe { &mut *(buffer as *mut Surface) };

    if !respect_alpha {
        // Force all alpha values to 1.0 (fully opaque)
        let cells = unsafe { surface.cells_mut() };
        for cell in cells {
            cell.fg.a = 1.0;
            cell.bg.a = 1.0;
        }
    }
    // If respect_alpha is true, we preserve existing alpha values
}

/// Resize buffer
#[reactive_tui_macros::ffi_export]
pub extern "C" fn bufferResize(buffer: *mut RTuiBuffer, width: u32, height: u32) {
    if buffer.is_null() || width == 0 || height == 0 {
        return;
    }

    let surface = unsafe { &mut *(buffer as *mut Surface) };
    let (old_width, old_height) = surface.dims();

    // If dimensions haven't changed, nothing to do
    if old_width == width as usize && old_height == height as usize {
        return;
    }

    // Save existing content that will fit in the new dimensions
    let copy_width = old_width.min(width as usize);
    let copy_height = old_height.min(height as usize);

    let mut saved_cells = Vec::new();
    for y in 0..copy_height {
        for x in 0..copy_width {
            saved_cells.push(surface.get(x, y));
        }
    }

    // Resize the surface using the built-in reinit method
    surface.reinit(width as usize, height as usize);

    // Restore the saved content
    let mut cell_idx = 0;
    for y in 0..copy_height {
        for x in 0..copy_width {
            if cell_idx < saved_cells.len() {
                surface.set(x, y, saved_cells[cell_idx]);
                cell_idx += 1;
            }
        }
    }
}

//
// SYSTEM INTEGRATION FUNCTIONS
//

/// Render surface to terminal through renderer
///
/// This integrates Surface → Renderer → Terminal in a single operation
#[reactive_tui_macros::ffi_export]
pub extern "C" fn renderSurfaceToTerminal(
    surface: *const RTuiBuffer,
    terminal: *mut super::terminal::RTuiTerminal,
) -> bool {
    if surface.is_null() || terminal.is_null() {
        return false;
    }

    // Validate pointers
    if !super::pointer::validate_pointer::<Surface>(surface as *const u8) {
        return false;
    }
    if !super::pointer::validate_pointer::<crate::core::terminal::Terminal>(terminal as *const u8) {
        return false;
    }

    let surface_ref = unsafe { &*(surface as *const Surface) };
    let _terminal_ref = unsafe { &mut *(terminal as *mut crate::core::terminal::Terminal) };

    // Get surface dimensions
    let (width, height) = surface_ref.dims();

    // Create a temporary renderer for this operation
    match Renderer::new(width, height) {
        Ok(mut renderer) => {
            // Copy surface data to renderer by copying cells
            let renderer_surface = renderer.surface_mut();
            let cells = surface_ref.cells();
            for (i, cell) in cells.iter().enumerate() {
                let x = i % width;
                let y = i / width;
                if y < height {
                    renderer_surface.set(x, y, *cell);
                }
            }

            // Render to terminal using begin/end frame
            match renderer.begin_frame() {
                Ok(_) => match renderer.end_frame() {
                    Ok(_) => true,
                    Err(_) => false,
                },
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Complete rendering pipeline: TextBuffer → Surface → Renderer → Terminal
///
/// This integrates all systems in a single high-level operation
#[reactive_tui_macros::ffi_export]
pub extern "C" fn renderTextToTerminal(
    text_buffer: *const super::text::RTuiTextBuffer,
    terminal: *mut super::terminal::RTuiTerminal,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> bool {
    if text_buffer.is_null() || terminal.is_null() {
        return false;
    }

    // Create temporary surface
    let surface = createOptimizedBuffer(
        width,
        height,
        false,            // respect_alpha
        0,                // width_method
        std::ptr::null(), // id_ptr
        0,                // id_len
    );
    if surface.is_null() {
        return false;
    }

    // Render text buffer to surface
    let chars_rendered = super::text::renderTextBufferToSurface(text_buffer, surface, x, y, width);

    if chars_rendered == 0 {
        destroyOptimizedBuffer(surface);
        return false;
    }

    // Render surface to terminal
    let success = renderSurfaceToTerminal(surface, terminal);

    // Cleanup
    destroyOptimizedBuffer(surface);

    success
}

/// Integrated renderer with stats collection
///
/// This combines Renderer → Terminal with automatic stats collection
#[reactive_tui_macros::ffi_export]
pub extern "C" fn renderWithStats(
    renderer: *mut super::lib::RTuiRenderer,
    terminal: *mut super::terminal::RTuiTerminal,
    collect_stats: bool,
) -> bool {
    if renderer.is_null() || terminal.is_null() {
        return false;
    }

    // Validate pointers
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return false;
    }
    if !super::pointer::validate_pointer::<crate::core::terminal::Terminal>(terminal as *const u8) {
        return false;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };
    let _terminal_ref = unsafe { &mut *(terminal as *mut crate::core::terminal::Terminal) };

    // Enable stats collection if requested
    if collect_stats {
        renderer_ref.enable_detailed_stats();
    }

    // Render using begin/end frame
    match renderer_ref.begin_frame() {
        Ok(_) => {
            match renderer_ref.end_frame() {
                Ok(_) => {
                    // Log stats if enabled
                    if collect_stats {
                        let metrics = renderer_ref.performance_metrics();
                        super::stats::log_message(
                            super::stats::LogLevel::Debug,
                            &format!(
                                "Render Stats: {:.2}ms, {} bytes, {:.1} FPS",
                                metrics.avg_frame_time.as_secs_f32() * 1000.0,
                                renderer_ref.write_stats().total_bytes,
                                metrics.fps
                            ),
                        );
                    }
                    true
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}
