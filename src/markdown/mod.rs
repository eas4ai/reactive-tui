//! Markdown rendering support with CommonMark + GFM compliance
//!
//! This module provides markdown rendering capabilities using the comrak crate,
//! which supports CommonMark and GitHub Flavored Markdown (GFM) extensions.
//! The rendered output is converted to our StyledRun/StyledLine model for
//! consistent terminal rendering.
//!
//! # Features
//! - CommonMark compliance
//! - GFM extensions (tables, strikethrough, task lists, footnotes)
//! - Syntax highlighting for code blocks (integrates with syntax module)
//! - Block elements: headings, paragraphs, lists, quotes, code blocks, tables
//! - Inline elements: bold, italic, links, inline code, strikethrough
//! - Source position tracking for selection/anchoring
//!
//! # Example
//! ```rust
//! use reactive_tui::markdown::MarkdownRenderer;
//!
//! let renderer = MarkdownRenderer::new();
//! let markdown = "# Hello\n\nThis is **bold** text.";
//! let styled_lines = renderer.render_to_styled_lines(markdown);
//! ```

/// AST walker for traversing markdown syntax trees
pub mod ast_walker;
/// Markdown to TUI component converter
pub mod converter;
/// Markdown renderer for terminal output
pub mod renderer;

#[cfg(test)]
mod tests;

pub use converter::*;
pub use renderer::*;
