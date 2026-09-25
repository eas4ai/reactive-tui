//! Text editor components with efficient gap buffer backend
//!
//! This module provides text editing capabilities using a scalar gap buffer. Positions count Unicode scalars; editor movement
//! and deletion use complete graphemes, while vertical movement uses terminal
//! columns. See `manual/text-editing-markdown-and-syntax.md` for the conversion and rendering rules.

pub mod cursor;
pub mod gap_buffer;
mod painting;
mod positions;
pub mod syntax_editor;
pub mod text_editor;

pub use cursor::Cursor;
pub use gap_buffer::GapBuffer;
pub use syntax_editor::SyntaxEditor;
pub use text_editor::TextEditor;
