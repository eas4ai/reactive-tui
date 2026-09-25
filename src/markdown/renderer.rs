use crate::core::styled_text::StyledLine;
use comrak::{parse_document, Arena, Options};
use std::collections::HashMap;

/// Maximum source size accepted by the checked Markdown renderer.
pub const MAX_MARKDOWN_BYTES: usize = 1024 * 1024;

/// Main markdown renderer that converts markdown text to StyledLine output
pub struct MarkdownRenderer<'a> {
    options: Options<'a>,
    enable_syntax_highlighting: bool,
}

impl<'a> MarkdownRenderer<'a> {
    /// Create a new markdown renderer with default options
    pub fn new() -> Self {
        let mut options = Options::default();

        // Enable GFM extensions
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.superscript = false; // Keep simple for terminal
        options.extension.header_id_prefix = None;
        options.extension.footnotes = true;
        options.extension.description_lists = true;
        options.extension.front_matter_delimiter = None;

        // Parse options
        options.parse.smart = true;
        options.parse.default_info_string = None;

        // Render options
        options.render.hardbreaks = false;
        options.render.github_pre_lang = true;
        options.render.full_info_string = false;
        options.render.width = 0; // No wrapping at parser level
        options.render.r#unsafe = false; // Safe by default
        options.render.escape = true;

        Self {
            options,
            enable_syntax_highlighting: true,
        }
    }

    /// Create renderer with custom options
    pub fn with_options(options: Options<'a>) -> Self {
        Self {
            options,
            enable_syntax_highlighting: true,
        }
    }

    /// Enable or disable syntax highlighting for code blocks
    pub fn with_syntax_highlighting(mut self, enabled: bool) -> Self {
        self.enable_syntax_highlighting = enabled;
        self
    }

    /// Render markdown text to StyledLine vector
    pub fn render_to_styled_lines(&self, markdown: &str) -> Vec<StyledLine> {
        self.try_render_to_styled_lines(markdown)
            .unwrap_or_else(|error| {
                vec![StyledLine::plain(format!("Markdown render error: {error}"))]
            })
    }

    /// Render Markdown with an explicit source-size error.
    pub fn try_render_to_styled_lines(
        &self,
        markdown: &str,
    ) -> crate::error::Result<Vec<StyledLine>> {
        if markdown.len() > MAX_MARKDOWN_BYTES {
            return Err(crate::error::ReactiveError::invalid_parameter(format!(
                "Markdown input exceeds the {MAX_MARKDOWN_BYTES}-byte limit"
            )));
        }
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &self.options);

        let walker = crate::markdown::ast_walker::AstWalker::new(self.enable_syntax_highlighting);

        Ok(walker.walk_document(root))
    }

    /// Render markdown with source position tracking
    pub fn render_with_sourcepos(
        &self,
        markdown: &str,
    ) -> (Vec<StyledLine>, HashMap<usize, (usize, usize)>) {
        let mut options = self.options.clone();
        options.render.sourcepos = true;

        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &options);

        let mut walker =
            crate::markdown::ast_walker::AstWalker::new(self.enable_syntax_highlighting);

        walker.walk_and_finish(root);
        let sourcepos_map = walker.sourcepos_map.clone();
        let styled_lines = walker.finish();

        (styled_lines, sourcepos_map)
    }

    /// Get the underlying Options
    pub fn options(&self) -> &Options<'a> {
        &self.options
    }

    /// Update Options
    pub fn set_options(&mut self, options: Options<'a>) {
        self.options = options;
    }
}

impl<'a> Default for MarkdownRenderer<'a> {
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
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;

    let renderer = MarkdownRenderer::with_options(options);
    renderer.render_to_styled_lines(markdown)
}
