pub mod adaptive;
pub mod capabilities;
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
