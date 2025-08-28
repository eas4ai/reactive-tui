use crate::syntax::StyledLine;
use comrak::{
    nodes::{AstNode, NodeValue},
    parse_document, Arena, ComrakOptions,
};
use std::collections::HashMap;

/// Main markdown renderer that converts markdown text to StyledLine output
pub struct MarkdownRenderer {
    options: ComrakOptions,
    syntax_highlighter: Option<crate::syntax::highlighter::SyntaxHighlighter>,
}

impl MarkdownRenderer {
    /// Create a new markdown renderer with default options
    pub fn new() -> Self {
        let mut options = ComrakOptions::default();
        
        // Enable GFM extensions
        options.extension.strikethrough = true;
        options.extension.tagfilter = false; // We handle our own filtering
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.superscript = false; // Keep simple for terminal
        options.extension.header_ids = None;
        options.extension.footnotes = true;
        options.extension.description_lists = true;
        options.extension.front_matter_delimiter = None;
        
        // Parse options
        options.parse.smart = true;
        options.parse.default_info_string = None;
        
        // Render options
        options.render.hardbreaks = false;
        options.render.github_pre_lang = true;
        options.render.width = 0; // No wrapping at parser level
        options.render.unsafe_ = false; // Safe by default
        options.render.escape = true;
        
        Self {
            options,
            syntax_highlighter: Some(crate::syntax::highlighter::SyntaxHighlighter::new()),
        }
    }
    
    /// Create renderer with custom options
    pub fn with_options(options: ComrakOptions) -> Self {
        Self {
            options,
            syntax_highlighter: Some(crate::syntax::highlighter::SyntaxHighlighter::new()),
        }
    }
    
    /// Enable or disable syntax highlighting for code blocks
    pub fn with_syntax_highlighting(mut self, enabled: bool) -> Self {
        if enabled {
            self.syntax_highlighter = Some(crate::syntax::highlighter::SyntaxHighlighter::new());
        } else {
            self.syntax_highlighter = None;
        }
        self
    }
    
    /// Render markdown text to StyledLine vector
    pub fn render_to_styled_lines(&self, markdown: &str) -> Vec<StyledLine> {
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &self.options);
        
        let mut walker = crate::markdown::ast_walker::AstWalker::new(
            &self.syntax_highlighter,
        );
        
        walker.walk_document(root)
    }
    
    /// Render markdown with source position tracking
    pub fn render_with_sourcepos(&self, markdown: &str) -> (Vec<StyledLine>, HashMap<usize, (usize, usize)>) {
        let mut options = self.options.clone();
        options.render.sourcepos = true;
        
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &options);
        
        let mut walker = crate::markdown::ast_walker::AstWalker::new(
            &self.syntax_highlighter,
        );
        
        let styled_lines = walker.walk_document(root);
        let sourcepos_map = walker.take_sourcepos_map();
        
        (styled_lines, sourcepos_map)
    }
    
    /// Get the underlying ComrakOptions
    pub fn options(&self) -> &ComrakOptions {
        &self.options
    }
    
    /// Update ComrakOptions
    pub fn set_options(&mut self, options: ComrakOptions) {
        self.options = options;
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function for quick markdown rendering
pub fn render_markdown(markdown: &str) -> Vec<StyledLine> {
    let renderer = MarkdownRenderer::new();
    renderer.render_to_styled_lines(markdown)
}

/// Render markdown with GFM extensions enabled
pub fn render_markdown_gfm(markdown: &str) -> Vec<StyledLine> {
    let mut options = ComrakOptions::default();
    options.extension = comrak::ComrakExtensionOptions {
        strikethrough: true,
        tagfilter: false,
        table: true,
        autolink: true,
        tasklist: true,
        superscript: false,
        header_ids: None,
        footnotes: true,
        description_lists: true,
        front_matter_delimiter: None,
    };
    
    let renderer = MarkdownRenderer::with_options(options);
    renderer.render_to_styled_lines(markdown)
}