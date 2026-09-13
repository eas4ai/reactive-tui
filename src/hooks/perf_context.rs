use crate::display::monitor::{PerformanceMetrics, PerformanceMode};
use crate::hooks::fps::{FpsState, FrameTiming};
use crate::reactive::hooks::ThreadSafeSignal;
use std::sync::{Arc, Mutex, RwLock};

/// Shared performance snapshots and a mode setter. App owns its own context.
#[derive(Clone)]
pub struct PerformanceContext {
    /// Current FPS state and statistics
    pub fps_state: ThreadSafeSignal<FpsState>,
    /// Performance metrics for monitoring
    pub metrics: ThreadSafeSignal<PerformanceMetrics>,
    /// Frame timing information
    pub frame_timing: ThreadSafeSignal<FrameTiming>,
    /// Function to set performance mode
    pub set_mode: Arc<dyn Fn(PerformanceMode) + Send + Sync>,
}

// Global storage for the performance context
static GLOBAL_CTX: RwLock<Option<Arc<PerformanceContext>>> = RwLock::new(None);

// Explicit standalone compatibility queue; Apps never consume it.
static REQUESTED_MODE: Mutex<Option<PerformanceMode>> = Mutex::new(None);

/// Set the standalone performance context. This does not affect any App.
pub fn set_global_performance_context(ctx: Arc<PerformanceContext>) {
    if let Ok(mut guard) = GLOBAL_CTX.write() {
        let previous = guard.replace(ctx);
        drop(guard);
        drop(previous);
    } else {
        log::error!("Global performance context lock poisoned during set");
    }
}

/// Get the global performance context (read by hooks if no local context provided)
pub fn get_global_performance_context() -> Option<Arc<PerformanceContext>> {
    GLOBAL_CTX
        .read()
        .map_err(|_| log::warn!("Global performance context lock poisoned during read"))
        .ok()
        .and_then(|guard| guard.clone())
}

/// Request a standalone performance mode change. This does not wake an App.
pub fn request_performance_mode(mode: PerformanceMode) {
    if let Ok(mut guard) = REQUESTED_MODE.lock() {
        *guard = Some(mode);
    } else {
        log::warn!("Requested performance mode lock poisoned during request");
    }
}

/// Take a standalone request. A standalone controller must apply it.
pub fn take_requested_performance_mode() -> Option<PerformanceMode> {
    REQUESTED_MODE
        .lock()
        .map_err(|_| log::warn!("Requested performance mode lock poisoned during take"))
        .ok()
        .and_then(|mut guard| guard.take())
}
