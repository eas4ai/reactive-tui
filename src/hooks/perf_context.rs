use crate::display::monitor::{PerformanceMetrics, PerformanceMode};
use crate::hooks::fps::{FrameTiming, FpsState};
use crate::reactive::hooks::ThreadSafeSignal;
use std::sync::{Arc, Mutex, RwLock};

/// Shared performance context exposed to hooks and updated by the App
#[derive(Clone)]
pub struct PerformanceContext {
    pub fps_state: ThreadSafeSignal<FpsState>,
    pub metrics: ThreadSafeSignal<PerformanceMetrics>,
    pub frame_timing: ThreadSafeSignal<FrameTiming>,
    pub set_mode: Arc<dyn Fn(PerformanceMode) + Send + Sync>,
}

// Global storage for the performance context
static GLOBAL_CTX: RwLock<Option<Arc<PerformanceContext>>> = RwLock::new(None);

// One-way request channel for performance mode changes from hooks -> App
static REQUESTED_MODE: Mutex<Option<PerformanceMode>> = Mutex::new(None);

/// Set the global performance context (called by App)
pub fn set_global_performance_context(ctx: Arc<PerformanceContext>) {
    let mut guard = GLOBAL_CTX.write().unwrap();
    *guard = Some(ctx);
}

/// Get the global performance context (read by hooks if no local context provided)
pub fn get_global_performance_context() -> Option<Arc<PerformanceContext>> {
    GLOBAL_CTX.read().unwrap().clone()
}

/// Request a performance mode change (called by hooks)
pub fn request_performance_mode(mode: PerformanceMode) {
    let mut guard = REQUESTED_MODE.lock().unwrap();
    *guard = Some(mode);
}

/// Take any requested performance mode (called by App each frame)
pub fn take_requested_performance_mode() -> Option<PerformanceMode> {
    let mut guard = REQUESTED_MODE.lock().unwrap();
    guard.take()
}

