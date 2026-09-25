//! Core rendering primitives and terminal abstractions
//!
//! This module contains the fundamental building blocks for terminal rendering,
//! including geometry types, surface buffers, and the rendering pipeline.

/// Geometric types (Point, Size, Rect) for positioning and layout
pub mod geometry;
/// Unicode grapheme cluster support for proper text rendering
pub mod grapheme_cell;
/// Mouse position and event tracking
pub mod mouse_tracker;
pub(crate) mod owned_process;
/// Low-level rendering operations and commands
pub mod render_ops;
/// Performance statistics and frame timing metrics
pub mod render_stats;
/// Main rendering engine with double-buffering and diff optimization
pub mod renderer;
/// Span-based diffing algorithm for efficient updates
pub mod span_diff;
/// Text with style attributes (colors, formatting)
pub mod styled_text;
/// Cell-based rendering surface and buffer management
pub mod surface;
/// Terminal control, raw mode, and escape sequences
pub mod terminal;
/// Window management and hierarchical rendering
pub mod window;
// pub mod terminal_capabilities;  // Old termwiz-based code, replaced by terminal_query
// pub mod terminal_probe;         // Old termwiz-based code, replaced by terminal_query
/// Terminal capability detection and feature flags
pub mod capabilities;
/// Output writer with buffering and optimization
pub mod writer;
