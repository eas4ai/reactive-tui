//! Syntax highlighter with incremental updates and styled text conversion
//!
//! Tree-sitter highlighting through Lumis. The public surface of this
//! module is backend-agnostic: callers work with [`HighlightedLine`] and
//! [`StyledRun`] and never see Lumis types.

use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Rgba};
use crate::syntax::cache::hash_line_content;
use crate::syntax::resources::SYNTAX_RESOURCES;
use crate::syntax::theme::hex_to_rgba;
use lumis::highlight::{Highlighter, Style, UnderlineStyle};
use lumis::languages::Language;
use lumis::themes::Appearance;
use std::sync::Arc;

/// Maximum source size accepted by the checked syntax highlighter.
pub const MAX_SYNTAX_BYTES: usize = 1024 * 1024;

/// A highlighted line with style information
#[derive(Debug, Clone)]
pub struct HighlightedLine {
    /// Styled text runs that make up the line
    pub runs: Vec<StyledRun>,
    /// Line number in the source file
    pub line_number: usize,
}

impl HighlightedLine {
    /// Convert to a StyledLine
    pub fn to_styled_line(&self) -> StyledLine {
        let mut line = StyledLine::new();
        for run in &self.runs {
            line.push(run.clone());
        }
        line
    }
}

/// Syntax highlighter for code
pub struct SyntaxHighlighter {
    language: Language,
    language_name: String,
    cached_lines: Vec<Option<HighlightedLine>>,
    /// Document text and theme identity for TXT-002.
    cached_document: Option<(u64, String)>,
    /// How many times the document was parsed, for the tests of TXT-002.
    #[cfg(test)]
    parses: std::sync::atomic::AtomicUsize,
}

impl SyntaxHighlighter {
    /// Create a new highlighter for a language
    pub fn new(language: &str) -> Option<Self> {
        let resources = SYNTAX_RESOURCES.read().ok()?;
        let found = resources.find_language_by_name(language)?;

        Some(Self {
            language: found,
            language_name: found.name().to_string(),
            cached_lines: Vec::new(),
            cached_document: None,
            #[cfg(test)]
            parses: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    /// Create a highlighter from file extension
    pub fn from_extension(extension: &str) -> Option<Self> {
        let resources = SYNTAX_RESOURCES.read().ok()?;
        let found = resources.find_language_for_file(extension);

        Some(Self {
            language: found,
            language_name: found.name().to_string(),
            cached_lines: Vec::new(),
            cached_document: None,
            #[cfg(test)]
            parses: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    /// Highlight a complete text
    pub fn highlight_text(&mut self, text: &str) -> Vec<HighlightedLine> {
        self.try_highlight_text(text).unwrap_or_else(|error| {
            vec![HighlightedLine {
                runs: vec![StyledRun::plain(format!("Syntax highlight error: {error}"))],
                line_number: 0,
            }]
        })
    }

    /// Highlight complete text with an explicit source-size error.
    pub fn try_highlight_text(&mut self, text: &str) -> crate::error::Result<Vec<HighlightedLine>> {
        if text.len() > MAX_SYNTAX_BYTES {
            return Err(crate::error::ReactiveError::invalid_parameter(format!(
                "syntax input exceeds the {MAX_SYNTAX_BYTES}-byte limit"
            )));
        }
        Ok(self.highlight_lines(text, 0, text.lines().count()))
    }

    /// Highlight a visible range, reusing the document parse for unchanged text and theme.
    pub fn highlight_lines(
        &mut self,
        text: &str,
        start_line: usize,
        end_line: usize,
    ) -> Vec<HighlightedLine> {
        let lines: Vec<&str> = text.lines().collect();
        // TXT-002: every line entry point enforces the source limit before parsing.
        if text.len() > MAX_SYNTAX_BYTES {
            return fallback_range(&lines, start_line, end_line);
        }

        let theme = match SYNTAX_RESOURCES.read() {
            Ok(resources) => resources.active_theme(),
            Err(_) => return fallback_range(&lines, start_line, end_line),
        };
        let Some(theme) = theme else {
            return fallback_range(&lines, start_line, end_line);
        };
        let identity = (hash_line_content(text), theme.name.clone());

        // TXT-002: parse once per text or theme change, then serve cached ranges.
        if self.cached_document.as_ref() != Some(&identity) {
            let default_fg = Self::default_foreground(&theme);
            let highlighter = Highlighter::new(self.language, Some(theme));
            self.note_parse();
            let full = match highlighter.highlight(text) {
                Ok(segments) => distribute_segments(text, &segments, default_fg),
                Err(_) => unhighlighted_lines(text),
            };
            self.cached_lines = full.into_iter().map(Some).collect();
            self.cached_document = Some(identity);
        }

        (start_line..end_line.min(lines.len()))
            .map(|line_num| {
                self.cached_lines
                    .get(line_num)
                    .cloned()
                    .flatten()
                    .unwrap_or_else(|| plain_line(lines[line_num], line_num))
            })
            .collect()
    }

    /// Invalidate cached lines in a range (for edits)
    pub fn invalidate_range(&mut self, start: usize, end: usize) {
        self.cached_document = None;
        for i in start..end.min(self.cached_lines.len()) {
            self.cached_lines[i] = None;
        }
    }

    /// Clear all cached lines
    pub fn clear_cache(&mut self) {
        self.cached_lines.clear();
        self.cached_document = None;
    }

    /// Get the language name
    pub fn language(&self) -> &str {
        &self.language_name
    }

    /// Count a parse of the document (TXT-002's tests read the count).
    fn note_parse(&self) {
        #[cfg(test)]
        self.parses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// How many times this highlighter parsed a document.
    #[cfg(test)]
    pub(crate) fn parse_count(&self) -> usize {
        self.parses.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Re-highlight a single line after edit
    pub fn rehighlight_line(&mut self, line_text: &str, line_num: usize) -> HighlightedLine {
        // TXT-002: oversized lines stay plain and never reach tree-sitter.
        if line_text.len() > MAX_SYNTAX_BYTES {
            return plain_line(line_text, line_num);
        }
        let (theme, default_fg) = match SYNTAX_RESOURCES.read() {
            Ok(resources) => {
                let theme = resources.active_theme();
                let default_fg = theme
                    .as_ref()
                    .map(Self::default_foreground)
                    .unwrap_or(Rgba::black());
                (theme, default_fg)
            }
            Err(_) => {
                // Lock poisoned, return unhighlighted line
                return plain_line(line_text, line_num);
            }
        };
        let Some(theme) = theme else {
            return plain_line(line_text, line_num);
        };

        let highlighter = Highlighter::new(self.language, Some(theme));
        self.note_parse();
        let highlighted_line = match highlighter.highlight(line_text) {
            Ok(segments) => {
                let mut distributed = distribute_segments(line_text, &segments, default_fg);
                distributed
                    .pop()
                    .unwrap_or_else(|| plain_line(line_text, line_num))
            }
            Err(_) => plain_line(line_text, line_num),
        };

        let mut highlighted_line = highlighted_line;
        highlighted_line.line_number = line_num;

        // TXT-001: an isolated line must not overwrite the document cache.
        highlighted_line
    }

    /// Default foreground for unstyled tokens under a theme.
    fn default_foreground(theme: &lumis::themes::Theme) -> Rgba {
        if let Some(normal) = theme.highlights.get("normal") {
            if let Some(fg) = normal.fg.as_deref() {
                if let Some(rgba) = hex_to_rgba(fg) {
                    return rgba;
                }
            }
        }
        if matches!(theme.appearance, Appearance::Light) {
            Rgba::black()
        } else {
            Rgba::white()
        }
    }
}

/// Split whole-text Lumis segments into per-source-line runs.
///
/// Segment text may span newlines; pieces are distributed to their source
/// lines so the result has exactly `text.lines().count()` entries.
fn distribute_segments(
    text: &str,
    segments: &[(Arc<Style>, &str)],
    default_fg: Rgba,
) -> Vec<HighlightedLine> {
    let line_count = text.lines().count();
    if line_count == 0 {
        return Vec::new();
    }
    let mut lines: Vec<Vec<StyledRun>> = vec![Vec::new(); line_count];
    let mut line_num = 0;
    for (style, segment) in segments {
        let (fg, attr) = style_to_run(style, default_fg);
        for (index, piece) in segment.split('\n').enumerate() {
            if index > 0 && line_num + 1 < line_count {
                line_num += 1;
            }
            if !piece.is_empty() {
                lines[line_num].push(StyledRun::new(
                    piece.to_string(),
                    fg,
                    Rgba::transparent(),
                    attr,
                ));
            }
        }
    }
    lines
        .into_iter()
        .enumerate()
        .map(|(line_number, runs)| HighlightedLine { runs, line_number })
        .collect()
}

/// Convert a Lumis style to a foreground color and text attributes.
fn style_to_run(style: &Style, default_fg: Rgba) -> (Rgba, Attr) {
    let fg = style
        .fg
        .as_deref()
        .and_then(hex_to_rgba)
        .unwrap_or(default_fg);
    let mut attr = Attr::empty();
    if style.bold {
        attr |= Attr::BOLD;
    }
    if style.italic {
        attr |= Attr::ITALIC;
    }
    if style.text_decoration.underline != UnderlineStyle::None {
        attr |= Attr::UNDERLINE;
    }
    (fg, attr)
}

/// Plain single line used for lock-poisoned and unresolvable fallbacks.
fn plain_line(line_text: &str, line_num: usize) -> HighlightedLine {
    HighlightedLine {
        runs: vec![StyledRun::new(
            line_text.to_string(),
            Rgba::black(),
            Rgba::transparent(),
            Attr::empty(),
        )],
        line_number: line_num,
    }
}

/// Unstyled lines for a whole text (fallback path).
fn unhighlighted_lines(text: &str) -> Vec<HighlightedLine> {
    text.lines()
        .enumerate()
        .map(|(line_num, line)| HighlightedLine {
            runs: vec![StyledRun::new(
                line.to_string(),
                Rgba::black(),
                Rgba::transparent(),
                Attr::empty(),
            )],
            line_number: line_num,
        })
        .collect()
}

/// Plain lines for a visible range (fallback path).
fn fallback_range(lines: &[&str], start_line: usize, end_line: usize) -> Vec<HighlightedLine> {
    (start_line..end_line.min(lines.len()))
        .map(|line_num| HighlightedLine {
            runs: vec![StyledRun::new(
                lines.get(line_num).unwrap_or(&"").to_string(),
                Rgba::black(),
                Rgba::transparent(),
                Attr::empty(),
            )],
            line_number: line_num,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_highlighting() {
        let mut highlighter = SyntaxHighlighter::new("Rust").unwrap();

        let code = r#"fn main() {
    println!("Hello, world!");
}"#;

        let highlighted = highlighter.highlight_text(code);
        assert_eq!(highlighted.len(), 3);

        // First line should have 'fn' keyword highlighted
        let first_line = &highlighted[0];
        assert!(!first_line.runs.is_empty());

        // Verify caching works
        let visible = highlighter.highlight_lines(code, 0, 2);
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn test_incremental_highlighting() {
        let mut highlighter = SyntaxHighlighter::new("Python").unwrap();

        let code = r#"def hello():
    print("Hello")
    return True

def world():
    print("World")"#;

        // Highlight just the visible portion
        let visible = highlighter.highlight_lines(code, 1, 3);
        assert_eq!(visible.len(), 2);

        // Invalidate and re-highlight
        highlighter.invalidate_range(1, 2);
        let rehighlighted = highlighter.highlight_lines(code, 1, 2);
        assert_eq!(rehighlighted.len(), 1);
    }

    #[test]
    fn test_extension_detection() {
        // Should detect Rust from .rs extension
        assert!(SyntaxHighlighter::from_extension("rs").is_some());

        // Should detect Python from .py extension
        assert!(SyntaxHighlighter::from_extension("py").is_some());

        // Should detect JavaScript from .js extension
        assert!(SyntaxHighlighter::from_extension("js").is_some());
    }

    #[test]
    fn txt_002_cache_tracks_document_changes_and_visible_ranges() {
        let mut highlighter = SyntaxHighlighter::new("Rust").unwrap();
        let document = "/*\nfn main() {}\n*/";
        highlighter.highlight_lines(document, 0, 1);
        let contextual = highlighter.highlight_lines(document, 1, 2);
        assert_eq!(highlighter.parse_count(), 1);

        // TXT-001: standalone highlighting must leave document colors intact.
        highlighter.rehighlight_line("fn main() {}", 1);
        let cached = highlighter.highlight_lines(document, 1, 2);
        assert_eq!(highlighter.parse_count(), 2);
        assert_eq!(cached[0].runs, contextual[0].runs);

        let edited = "// comment\nfn main() {}\n";
        let updated = highlighter.highlight_lines(edited, 1, 2);
        assert_eq!(highlighter.parse_count(), 3);
        let expected = SyntaxHighlighter::new("Rust")
            .unwrap()
            .highlight_text(edited);
        assert_eq!(updated[0].runs, expected[1].runs);
        assert!(highlighter.highlight_lines("x", 1, 3).is_empty());
        assert_eq!(highlighter.parse_count(), 4);
    }

    /// TXT-002: an unchanged text is parsed once; later calls serve the cache.
    #[test]
    fn txt_002_highlight_lines_parses_once_for_unchanged_text() {
        let mut highlighter = SyntaxHighlighter::new("Rust").unwrap();
        let text = "fn main() {}\n";
        highlighter.highlight_lines(text, 0, 1);
        highlighter.highlight_lines(text, 0, 1);
        assert_eq!(
            highlighter.parse_count(),
            1,
            "TXT-002: two highlight_lines calls on the same text parsed the document {} times",
            highlighter.parse_count()
        );
    }

    /// TXT-002: the line entry points apply the byte limit as the checked one does.
    #[test]
    fn txt_002_oversized_text_is_not_parsed_by_the_line_entry_points() {
        let mut highlighter = SyntaxHighlighter::new("Rust").unwrap();
        let oversized = "x".repeat(MAX_SYNTAX_BYTES + 1);
        highlighter.highlight_lines(&oversized, 0, 1);
        highlighter.rehighlight_line(&oversized, 0);
        assert_eq!(
            highlighter.parse_count(),
            0,
            "TXT-002: a text over MAX_SYNTAX_BYTES was parsed by highlight_lines or rehighlight_line"
        );
    }
}
