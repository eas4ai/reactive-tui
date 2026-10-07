//! Performance monitoring and debugging FFI functions
//!
//! Provides performance statistics and debugging capabilities

use super::*;
use crate::core::renderer::Renderer;
use std::sync::RwLock;

//
// PERFORMANCE MONITORING
//

/// Keep the host time, FPS and frame callback time for the debug overlay and buffer dump.
/// Also enable detailed measured frame statistics.
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn updateStats(
    renderer: *mut RTuiRenderer,
    time: f64,
    fps: u32,
    frame_callback_time: f64,
) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };

    let host = renderer_ref.host_stats_mut();
    host.time = Some(time);
    host.fps = Some(fps);
    host.frame_callback_time = Some(frame_callback_time);

    // Enable detailed stats collection for better performance monitoring
    renderer_ref.enable_detailed_stats();
}

/// Keep the host heap-used, heap-total and array-buffer values for the overlay and buffer dump.
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn updateMemoryStats(
    renderer: *mut RTuiRenderer,
    heap_used: u32,
    heap_total: u32,
    array_buffers: u32,
) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };
    let host = renderer_ref.host_stats_mut();
    host.heap_used = Some(heap_used);
    host.heap_total = Some(heap_total);
    host.array_buffers = Some(array_buffers);
}

/// Add `offset` to every emitted cursor row, including the debug overlay.
/// Rows beyond the terminal are still written; changing the offset forces a repaint.
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn setRenderOffset(renderer: *mut RTuiRenderer, offset: u32) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };
    renderer_ref.set_render_offset(offset);
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
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn setDebugOverlay(renderer: *mut RTuiRenderer, enabled: bool, _corner: u8) {
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

/// Write `id` over a region clipped to the renderer in the grid being built.
/// Negative origins clip; outside regions are ignored; later registrations overwrite earlier ones.
/// A successful completed render publishes this grid and starts an empty one.
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn addToHitGrid(
    renderer: *mut RTuiRenderer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    id: u32,
) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &mut *(renderer as *mut Renderer) };
    renderer_ref.add_hit_region(x, y, width, height, id);
}

/// Return the registered id at (x, y) in the last completed frame, or 0 for an empty or outside cell.
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn checkHit(renderer: *mut RTuiRenderer, x: u32, y: u32) -> u32 {
    if renderer.is_null() {
        return 0;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return 0;
    }

    let renderer_ref = unsafe { &*(renderer as *const Renderer) };
    renderer_ref.check_hit(x as usize, y as usize)
}

/// Log each distinct nonzero id in the shown hit grid with its cells' bounding box.
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn dumpHitGrid(renderer: *mut RTuiRenderer) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &*(renderer as *const Renderer) };
    for (id, (left, top, right, bottom)) in renderer_ref.hit_regions() {
        log_message(
            LogLevel::Debug,
            &format!(
                "Hit Grid: id {id}, x={left}, y={top}, width={}, height={}",
                right - left + 1,
                bottom - top + 1
            ),
        );
    }
}

//
// BUFFER DEBUGGING
//

/// Write `rtui-buffers-<timestamp>.txt` in the current directory.
/// The text contains dimensions, front and back surfaces row by row (each cell's
/// grapheme, or a space for an empty cell), last frame statistics and kept host statistics.
/// The dump is written to a sibling file created fresh and then moved over the
/// name, so whatever held that name, a file or a link, is replaced and never
/// written through. Log the path on success or the write error on failure,
/// without panicking.
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn dumpBuffers(renderer: *mut RTuiRenderer, timestamp: i64) {
    if renderer.is_null() {
        return;
    }

    // Validate pointer before using
    if !super::pointer::validate_pointer::<Renderer>(renderer as *const u8) {
        return;
    }

    let renderer_ref = unsafe { &*(renderer as *const Renderer) };
    let path = format!("rtui-buffers-{timestamp}.txt");
    match renderer_ref.dump_buffers(std::path::Path::new(&path)) {
        Ok(()) => log_message(LogLevel::Debug, &format!("Buffer dump written to {path}")),
        Err(error) => log_message(LogLevel::Error, &format!("Failed to write {path}: {error}")),
    }
}

/// Dump stdout buffer for debugging
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn dumpStdoutBuffer(renderer: *mut RTuiRenderer, timestamp: i64) {
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
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn startProfiling(renderer: *mut RTuiRenderer) {
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
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn stopProfiling(renderer: *mut RTuiRenderer) {
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
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle this library
/// returned and has not destroyed, and no call may destroy it until this one
/// returns. `out_avg_frame_time` must be null or a `f32` the caller owns,
/// which this call may write. `out_min_frame_time` must be null or a `f32`
/// the caller owns, which this call may write. `out_max_frame_time` must be
/// null or a `f32` the caller owns, which this call may write.
/// `out_frame_count` must be null or a `u32` the caller owns, which this call
/// may write.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn getFrameStats(
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
///
/// # Safety
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn resetPerformanceCounters(renderer: *mut RTuiRenderer) {
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
