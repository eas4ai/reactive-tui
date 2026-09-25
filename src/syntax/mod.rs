//! Syntax highlighting integration with Lumis (tree-sitter)
//!
//! Provides efficient syntax highlighting for code blocks with caching,
//! incremental updates, and custom theme support.

pub mod cache;
pub mod highlighter;
pub mod resources;
pub mod theme;

pub use cache::LineCache;
pub use highlighter::{HighlightedLine, SyntaxHighlighter};
pub use resources::{SyntaxResources, ThemeSet, SYNTAX_RESOURCES};
pub use theme::{
    create_syntax_theme, hex_to_rgba, SyntaxElement, SyntaxThemeVariables, ThemedSyntaxStyle,
};
