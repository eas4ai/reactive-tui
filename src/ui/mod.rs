//! High-level UI utilities and helpers
//! 
//! This module provides utilities for UI rendering, hit testing, and painting operations.

/// Hit testing grid for mouse interaction detection
pub mod hitgrid;
/// Paint utilities for styling and rendering
pub mod paint;

/// Trait for components that can receive updates
/// 
/// This trait is a marker trait for types that can be updated in response to state changes
/// or external events. Implementors of this trait can be registered to receive updates
/// when the UI needs to be refreshed.
pub trait Updater {}
