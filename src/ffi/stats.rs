//! Performance monitoring and debugging FFI functions
//!
//! Provides performance statistics and debugging capabilities

use super::*;
use crate::core::renderer::Renderer;
use std::sync::RwLock;

//
// PERFORMANCE MONITORING
//

/// Update performance statistics
#[reactive_tui_macros::ffi_export]
pub extern "C" fn updateStats(
    renderer: *mut RTuiRenderer,
    _time: f64,
    _fps: u32,
    _frame_callback_time: f64,
) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };

    // Enable detailed stats collection for better performance monitoring
    renderer_ref.enable_detailed_stats();
}

/// Update memory statistics
#[reactive_tui_macros::ffi_export]
pub extern "C" fn updateMemoryStats(
    renderer: *mut RTuiRenderer,
    _heap_used: u32,
    _heap_total: u32,
    _array_buffers: u32,
) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    // Memory stats are automatically tracked by the renderer
    // The renderer estimates its own memory usage internally
}

/// Set render offset for debugging
#[reactive_tui_macros::ffi_export]
pub extern "C" fn setRenderOffset(renderer: *mut RTuiRenderer, offset: u32) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    // Log the render offset setting for debugging purposes
    log_message(LogLevel::Debug, &format!(
        "Render offset set to: {} (Note: Offset rendering requires terminal coordinate modification - not currently implemented in core renderer)",
        offset
    ));

    // Render offset implementation requires:
    // - DiffWriter coordinate transformation for all escape sequences
    // - Renderer state tracking for offset values
    // - Bounds checking integration with offset calculations
    // The current architecture prioritizes performance over coordinate transformation
}

/// Debug overlay corner enumeration (matching OpenTUI)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum DebugOverlayCorner {
    /// Top-left corner of the screen
    TopLeft = 0,
    /// Top-right corner of the screen
    TopRight = 1,
    /// Bottom-left corner of the screen
    BottomLeft = 2,
    /// Bottom-right corner of the screen
    BottomRight = 3,
}

/// Set debug overlay
#[reactive_tui_macros::ffi_export]
pub extern "C" fn setDebugOverlay(renderer: *mut RTuiRenderer, enabled: bool, _corner: u8) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };

    // Enable/disable the debug overlay (corner position is fixed in bottom)
    renderer_ref.set_debug_overlay(enabled);
}

//
// HIT TESTING AND DEBUGGING
//

/// Add element to hit grid for mouse interaction debugging
#[reactive_tui_macros::ffi_export]
pub extern "C" fn addToHitGrid(
    renderer: *mut RTuiRenderer,
    _x: i32,
    _y: i32,
    _width: u32,
    _height: u32,
    _id: u32,
) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    // Hit region tracking requires extending the renderer with a spatial data structure
    // The current renderer focuses on efficient terminal output rather than input handling
    // This function is safe to call but performs no operation
}

/// Check hit at coordinates
#[reactive_tui_macros::ffi_export]
pub extern "C" fn checkHit(renderer: *mut RTuiRenderer, x: u32, y: u32) -> u32 {
    if renderer.is_null() {
        return 0;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return 0;
    }

    let renderer_ref = unsafe { &*(renderer as *const Renderer) };
    let (width, height) = renderer_ref.dims();

    // Basic bounds checking - return 1 if within bounds, 0 if outside
    if (x as usize) < width && (y as usize) < height {
        1 // Hit within renderer bounds
    } else {
        0 // Outside bounds
    }
}

/// Dump hit grid for debugging
#[reactive_tui_macros::ffi_export]
pub extern "C" fn dumpHitGrid(renderer: *mut RTuiRenderer) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &*(renderer as *const Renderer) };
    let (width, height) = renderer_ref.dims();

    // Log hit grid information
    log_message(
        LogLevel::Debug,
        &format!(
            "Hit Grid Debug: Renderer dimensions {}x{}, hit testing available for bounds checking",
            width, height
        ),
    );
}

//
// BUFFER DEBUGGING
//

/// Dump buffers to file for debugging
#[reactive_tui_macros::ffi_export]
pub extern "C" fn dumpBuffers(renderer: *mut RTuiRenderer, timestamp: i64) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &*(renderer as *const Renderer) };
    let (width, height) = renderer_ref.dims();
    let stats = renderer_ref.frame_stats();

    // Log buffer state information
    log_message(
        LogLevel::Debug,
        &format!(
            "Buffer Dump [{}]: {}x{} surface, last frame: {:.2}ms, {} bytes written, {} spans",
            timestamp, width, height, stats.frame_ms, stats.bytes_written, stats.spans_written
        ),
    );
}

/// Dump stdout buffer for debugging
#[reactive_tui_macros::ffi_export]
pub extern "C" fn dumpStdoutBuffer(renderer: *mut RTuiRenderer, timestamp: i64) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &*(renderer as *const Renderer) };
    let write_stats = renderer_ref.write_stats();

    // Log stdout buffer information
    log_message(
        LogLevel::Debug,
        &format!(
            "Stdout Buffer Dump [{}]: {} total writes, {:.1}% buffer utilization, {} total bytes",
            timestamp,
            write_stats.total_writes,
            write_stats.buffer_utilization * 100.0,
            write_stats.total_bytes
        ),
    );
}

//
// LOGGING AND DIAGNOSTICS
//

/// Log callback function type (matching OpenTUI)
pub type LogCallback = extern "C" fn(level: u8, msg_ptr: *const u8, msg_len: usize);

/// Global log callback storage
static LOG_CALLBACK: RwLock<Option<LogCallback>> = RwLock::new(None);

/// Set log callback for debugging
#[reactive_tui_macros::ffi_export]
pub extern "C" fn setLogCallback(
    // Spell out the nullable function type so header generation retains its ABI.
    callback: Option<extern "C" fn(level: u8, msg_ptr: *const u8, msg_len: usize)>,
) {
    *LOG_CALLBACK
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = callback;
}

/// Log levels (matching OpenTUI)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    /// Error messages (highest priority)
    Error = 0,
    /// Warning messages
    Warn = 1,
    /// Informational messages
    Info = 2,
    /// Debug messages
    Debug = 3,
    /// Trace messages (lowest priority)
    Trace = 4,
}

/// Internal logging function
pub(crate) fn log_message(level: LogLevel, message: &str) {
    let callback = *LOG_CALLBACK
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(callback) = callback {
        let c_str = std::ffi::CString::new(message).unwrap_or_default();
        let bytes = c_str.as_bytes_with_nul();
        callback(level as u8, bytes.as_ptr(), bytes.len() - 1); // Exclude null terminator from length
        return;
    }

    // Fallback to standard Rust logging
    match level {
        LogLevel::Error => log::error!("{}", message),
        LogLevel::Warn => log::warn!("{}", message),
        LogLevel::Info => log::info!("{}", message),
        LogLevel::Debug => log::debug!("{}", message),
        LogLevel::Trace => log::trace!("{}", message),
    }
}

#[cfg(test)]
mod callback_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CALLS: AtomicUsize = AtomicUsize::new(0);

    extern "C" fn reentrant_callback(_: u8, _: *const u8, _: usize) {
        CALLS.fetch_add(1, Ordering::Relaxed);
        setLogCallback(None);
    }

    #[test]
    fn callback_replacement_is_synchronized_and_reentrant() {
        CALLS.store(0, Ordering::Relaxed);
        setLogCallback(Some(reentrant_callback));
        log_message(LogLevel::Info, "reentrant");
        assert_eq!(CALLS.load(Ordering::Relaxed), 1);

        let setter = std::thread::spawn(|| {
            for index in 0..10_000 {
                setLogCallback((index % 2 == 0).then_some(reentrant_callback));
            }
        });
        let logger = std::thread::spawn(|| {
            for _ in 0..10_000 {
                log_message(LogLevel::Debug, "concurrent");
            }
        });
        setter.join().unwrap();
        logger.join().unwrap();
        setLogCallback(None);
    }
}

//
// PROFILING AND BENCHMARKING
//

/// Start profiling session
#[reactive_tui_macros::ffi_export]
pub extern "C" fn startProfiling(renderer: *mut RTuiRenderer) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };

    // Enable detailed statistics collection for profiling
    renderer_ref.enable_detailed_stats();
    renderer_ref.clear_stats(); // Start fresh
}

/// Stop profiling session
#[reactive_tui_macros::ffi_export]
pub extern "C" fn stopProfiling(renderer: *mut RTuiRenderer) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };

    // Get final metrics before disabling
    let metrics = renderer_ref.performance_metrics();
    let grade = renderer_ref.performance_grade();

    // Log profiling results
    log_message(
        LogLevel::Info,
        &format!(
            "Profiling Results: FPS: {:.1}, Efficiency: {:.1}%, Grade: {}, Frames: {}",
            metrics.fps,
            metrics.efficiency_ratio * 100.0,
            grade,
            metrics.total_frames
        ),
    );

    // Disable detailed stats collection
    renderer_ref.disable_detailed_stats();
}

/// Get frame timing statistics
#[reactive_tui_macros::ffi_export]
pub extern "C" fn getFrameStats(
    renderer: *const RTuiRenderer,
    out_avg_frame_time: *mut f32,
    out_min_frame_time: *mut f32,
    out_max_frame_time: *mut f32,
    out_frame_count: *mut u32,
) {
    if renderer.is_null()
        || out_avg_frame_time.is_null()
        || out_min_frame_time.is_null()
        || out_max_frame_time.is_null()
        || out_frame_count.is_null()
    {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &*(renderer as *const Renderer) };
    let metrics = renderer_ref.performance_metrics();
    let _current_stats = renderer_ref.frame_stats();

    unsafe {
        *out_avg_frame_time = metrics.avg_frame_time.as_secs_f32() * 1000.0;
        *out_min_frame_time = metrics.min_frame_time.as_secs_f32() * 1000.0;
        *out_max_frame_time = metrics.max_frame_time.as_secs_f32() * 1000.0;
        *out_frame_count = metrics.total_frames as u32;
    }
}

/// Reset performance counters
#[reactive_tui_macros::ffi_export]
pub extern "C" fn resetPerformanceCounters(renderer: *mut RTuiRenderer) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };

    // Clear all collected statistics and reset counters
    renderer_ref.clear_stats();
    renderer_ref.reset_write_stats();
}
