//! Display utilities and performance monitoring
//!
//! This module provides tools for adaptive rendering, terminal capability detection,
//! and performance monitoring to optimize the display output.

/// Adaptive frame rate management for optimal performance
pub mod adaptive;
/// Terminal display capabilities detection
pub mod capabilities;
/// Performance monitoring and metrics collection
pub mod monitor;

pub use adaptive::{AdaptiveConfig, AdaptiveFpsManager};
pub use capabilities::{ColorDepth, ConnectionType, DisplayCapabilities, TerminalInfo};
pub use monitor::{PerformanceMetrics, PerformanceMonitor};

/// Re-export commonly used items
pub mod prelude {
    pub use super::adaptive::{AdaptiveConfig, AdaptiveFpsManager};
    pub use super::capabilities::{DisplayCapabilities, TerminalInfo};
    pub use super::monitor::PerformanceMetrics;
}
