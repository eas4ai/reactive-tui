//! High-level UI utilities and helpers
//!
//! This module provides utilities for UI rendering, hit testing, and painting operations.

/// Hit testing grid for mouse interaction detection
pub mod hitgrid;
/// Paint utilities for styling and rendering
pub mod paint;

mod updater;
pub(crate) use updater::UpdateRegistry;
pub use updater::{UpdateHandle, UpdateRegistration, Updater};
