//! Widget builders for advanced UI components
//!
//! This module contains builders for complex widgets like data tables, charts,
//! dialogs, and other specialized UI components.

pub mod table;
pub mod chart;
pub mod dialog;
pub mod input;
pub mod display;
pub mod layout;

// Re-export all widget builder functions and types
pub use table::*;
pub use chart::*;
pub use dialog::*;
pub use input::*;
pub use display::*;
pub use layout::*;

// Re-export placeholder builders
// pub use super::placeholders::*; // Module doesn't exist yet
