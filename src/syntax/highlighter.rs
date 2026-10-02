//! Syntax highlighter with incremental updates and styled text conversion
//!
//! Tree-sitter highlighting through Lumis. The public surface of this
//! module is backend-agnostic: callers work with [`HighlightedLine`] and
//! [`StyledRun`] and never see Lumis types.

use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Rgba};
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
                // Lock poisoned, return unhighlighted text
                return Ok(unhighlighted_lines(text));
            }
        };
        let Some(theme) = theme else {
            return Ok(unhighlighted_lines(text));
        };

        let highlighter = Highlighter::new(self.language, Some(theme));
        let segments = match highlighter.highlight(text) {
            Ok(segments) => segments,
            Err(_) => return Ok(unhighlighted_lines(text)),
        };

        let result = distribute_segments(text, &segments, default_fg);

        // Cache the results
        self.cached_lines = result.iter().map(|line| Some(line.clone())).collect();

        Ok(result)
    }

    /// Highlight only visible lines (incremental)
    pub fn highlight_lines(
        &mut self,
        text: &str,
        start_line: usize,
        end_line: usize,
    ) -> Vec<HighlightedLine> {
        let lines: Vec<&str> = text.lines().collect();

        // Ensure cache is sized correctly
        if self.cached_lines.len() < lines.len() {
            self.cached_lines.resize(lines.len(), None);
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
                // Lock poisoned, return fallback for visible range
                return fallback_range(&lines, start_line, end_line);
            }
        };
        let Some(theme) = theme else {
            return fallback_range(&lines, start_line, end_line);
        };

        // Tree-sitter needs whole-document context, so highlight everything
        // once and serve the requested range (from cache when warm).
        let highlighter = Highlighter::new(self.language, Some(theme));
        if let Ok(segments) = highlighter.highlight(text) {
            let full = distribute_segments(text, &segments, default_fg);
            for (line_num, line) in full.into_iter().enumerate() {
                if line_num < self.cached_lines.len() {
                    self.cached_lines[line_num] = Some(line);
                }
            }
        }

        let mut result = Vec::new();
        let end = end_line.min(lines.len());
        for (line_num, _) in lines.iter().enumerate().take(end).skip(start_line) {
            // Check cache first
            if let Some(cached) = &self.cached_lines[line_num] {
                result.push(cached.clone());
                continue;
            }

            // Not cached (highlight failed above): plain fallback line.
            result.push(HighlightedLine {
                runs: vec![StyledRun::new(
                    lines[line_num].to_string(),
                    Rgba::black(),
                    Rgba::transparent(),
                    Attr::empty(),
                )],
                line_number: line_num,
            });
        }

        result
    }

    /// Invalidate cached lines in a range (for edits)
    pub fn invalidate_range(&mut self, start: usize, end: usize) {
        for i in start..end.min(self.cached_lines.len()) {
            self.cached_lines[i] = None;
        }
    }

    /// Clear all cached lines
    pub fn clear_cache(&mut self) {
        self.cached_lines.clear();
    }

    /// Get the language name
    pub fn language(&self) -> &str {
        &self.language_name
    }

    /// Re-highlight a single line after edit
    pub fn rehighlight_line(&mut self, line_text: &str, line_num: usize) -> HighlightedLine {
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

        // Update cache
        if line_num < self.cached_lines.len() {
            self.cached_lines[line_num] = Some(highlighted_line.clone());
        }

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
}
