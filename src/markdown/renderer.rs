use crate::core::styled_text::StyledLine;
use comrak::{Arena, ExtensionOptions, Options, parse_document};
use std::collections::HashMap;

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
        options.render.full_info_string = false;
        options.render.width = 0; // No wrapping at parser level
        options.render.unsafe_ = false; // Safe by default
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
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &self.options);

        let walker = crate::markdown::ast_walker::AstWalker::new(self.enable_syntax_highlighting);

        walker.walk_document(root)
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
    let options = Options {
        extension: ExtensionOptions {
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
            multiline_block_quotes: false,
            math_dollars: false,
            math_code: false,
            wikilinks_title_after_pipe: false,
            wikilinks_title_before_pipe: true,
            greentext: false,
            underline: false,
            spoiler: false,
            alerts: false,
            subscript: false,
            image_url_rewriter: None,
            link_url_rewriter: None,
            cjk_friendly_emphasis: false,
        },
        ..Default::default()
    };

    let renderer = MarkdownRenderer::with_options(options);
    renderer.render_to_styled_lines(markdown)
}
