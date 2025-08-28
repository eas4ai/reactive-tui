//! Text editor components with efficient gap buffer backend
//!
//! This module provides text editing capabilities using a zero-copy
//! gap buffer for optimal performance with large files.

pub mod cursor;
pub mod gap_buffer;
pub mod syntax_editor;
pub mod text_editor;

pub use cursor::Cursor;
pub use gap_buffer::GapBuffer;
pub use syntax_editor::SyntaxEditor;
pub use text_editor::TextEditor;
