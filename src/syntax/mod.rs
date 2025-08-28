//! Syntax highlighting integration with syntect
//!
//! Provides efficient syntax highlighting for code blocks with caching,
//! incremental updates, and custom theme support.

pub mod cache;
pub mod highlighter;
pub mod resources;
pub mod theme;

pub use cache::LineCache;
pub use highlighter::{HighlightedLine, SyntaxHighlighter};
pub use resources::{SYNTAX_RESOURCES, SyntaxResources, ThemeSet};
pub use theme::{
    SyntaxElement, SyntaxThemeVariables, ThemedSyntaxStyle, create_syntax_theme, hex_to_rgba,
};
