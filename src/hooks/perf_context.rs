use crate::display::monitor::{PerformanceMetrics, PerformanceMode};
use crate::hooks::fps::{FpsState, FrameTiming};
use crate::reactive::hooks::ThreadSafeSignal;
use std::sync::{Arc, Mutex, RwLock};

/// Shared performance context exposed to hooks and updated by the App
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

// One-way request channel for performance mode changes from hooks -> App
static REQUESTED_MODE: Mutex<Option<PerformanceMode>> = Mutex::new(None);

/// Set the global performance context (called by App)
pub fn set_global_performance_context(ctx: Arc<PerformanceContext>) {
    if let Ok(mut guard) = GLOBAL_CTX.write() {
        *guard = Some(ctx);
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

/// Request a performance mode change (called by hooks)
pub fn request_performance_mode(mode: PerformanceMode) {
    if let Ok(mut guard) = REQUESTED_MODE.lock() {
        *guard = Some(mode);
    } else {
        log::warn!("Requested performance mode lock poisoned during request");
    }
}

/// Take any requested performance mode (called by App each frame)
pub fn take_requested_performance_mode() -> Option<PerformanceMode> {
    REQUESTED_MODE
        .lock()
        .map_err(|_| log::warn!("Requested performance mode lock poisoned during take"))
        .ok()
        .and_then(|mut guard| guard.take())
}
