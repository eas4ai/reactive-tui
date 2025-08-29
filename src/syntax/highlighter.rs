//! Syntax highlighter with incremental updates and styled text conversion

use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Rgba};
use crate::syntax::resources::SYNTAX_RESOURCES;
use std::sync::Arc;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style};
use syntect::parsing::{ParseState, SyntaxReference};

/// A highlighted line with style information
#[derive(Debug, Clone)]
pub struct HighlightedLine {
    pub runs: Vec<StyledRun>,
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
    syntax: Arc<SyntaxReference>,
    _parse_state: ParseState,
    _highlight_state: Option<HighlightLines<'static>>,
    cached_lines: Vec<Option<HighlightedLine>>,
    language: String,
}

impl SyntaxHighlighter {
    /// Create a new highlighter for a language
    pub fn new(language: &str) -> Option<Self> {
        let resources = SYNTAX_RESOURCES.read().unwrap();
        let syntax = resources.find_syntax_by_name(language)?;

        // Clone the syntax reference
        let syntax = Arc::new(syntax.clone());
        let parse_state = ParseState::new(syntax.as_ref());

        Some(Self {
            syntax: syntax.clone(),
            _parse_state: parse_state,
            _highlight_state: None,
            cached_lines: Vec::new(),
            language: language.to_string(),
        })
    }

    /// Create a highlighter from file extension
    pub fn from_extension(extension: &str) -> Option<Self> {
        let resources = SYNTAX_RESOURCES.read().unwrap();
        let syntax = resources.find_syntax(extension)?;

        let syntax = Arc::new(syntax.clone());
        let parse_state = ParseState::new(syntax.as_ref());

        Some(Self {
            syntax: syntax.clone(),
            _parse_state: parse_state,
            _highlight_state: None,
            cached_lines: Vec::new(),
            language: syntax.name.clone(),
        })
    }

    /// Highlight a complete text
    pub fn highlight_text(&mut self, text: &str) -> Vec<HighlightedLine> {
        let resources = SYNTAX_RESOURCES.read().unwrap();
        let theme = resources.active_theme();

        let mut highlighter = HighlightLines::new(self.syntax.as_ref(), theme);
        let mut result = Vec::new();

        for (line_num, line) in text.lines().enumerate() {
            let highlighted = highlighter
                .highlight_line(line, &resources.syntax_set)
                .unwrap_or_else(|_| vec![(Style::default(), line)]);

            let runs = highlighted
                .into_iter()
                .map(|(style, text)| {
                    StyledRun::new(
                        text.to_string(),
                        syntect_style_to_rgba(style.foreground),
                        Rgba::transparent(),
                        syntect_font_to_attr(style.font_style),
                    )
                })
                .collect();

            result.push(HighlightedLine {
                runs,
                line_number: line_num,
            });
        }

        // Cache the results
        self.cached_lines = result.iter().map(|line| Some(line.clone())).collect();

        result
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

        let resources = SYNTAX_RESOURCES.read().unwrap();
        let theme = resources.active_theme();
        let mut highlighter = HighlightLines::new(self.syntax.as_ref(), theme);

        let mut result = Vec::new();

        #[allow(clippy::needless_range_loop)]
        for line_num in start_line..end_line.min(lines.len()) {
            // Check cache first
            if let Some(cached) = &self.cached_lines[line_num] {
                result.push(cached.clone());
                continue;
            }

            // Highlight the line
            let line = lines[line_num];
            let highlighted = highlighter
                .highlight_line(line, &resources.syntax_set)
                .unwrap_or_else(|_| vec![(Style::default(), line)]);

            let runs = highlighted
                .into_iter()
                .map(|(style, text)| {
                    StyledRun::new(
                        text.to_string(),
                        syntect_style_to_rgba(style.foreground),
                        Rgba::transparent(),
                        syntect_font_to_attr(style.font_style),
                    )
                })
                .collect();

            let highlighted_line = HighlightedLine {
                runs,
                line_number: line_num,
            };

            // Cache the result
            self.cached_lines[line_num] = Some(highlighted_line.clone());
            result.push(highlighted_line);
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
        &self.language
    }

    /// Re-highlight a single line after edit
    pub fn rehighlight_line(&mut self, line_text: &str, line_num: usize) -> HighlightedLine {
        let resources = SYNTAX_RESOURCES.read().unwrap();
        let theme = resources.active_theme();

        // Need to maintain state for proper context
        let mut highlighter = HighlightLines::new(self.syntax.as_ref(), theme);

        let highlighted = highlighter
            .highlight_line(line_text, &resources.syntax_set)
            .unwrap_or_else(|_| vec![(Style::default(), line_text)]);

        let runs = highlighted
            .into_iter()
            .map(|(style, text)| {
                StyledRun::new(
                    text.to_string(),
                    syntect_style_to_rgba(style.foreground),
                    Rgba::transparent(),
                    syntect_font_to_attr(style.font_style),
                )
            })
            .collect();

        let highlighted_line = HighlightedLine {
            runs,
            line_number: line_num,
        };

        // Update cache
        if line_num < self.cached_lines.len() {
            self.cached_lines[line_num] = Some(highlighted_line.clone());
        }

        highlighted_line
    }
}

/// Convert syntect Color to Rgba
fn syntect_style_to_rgba(color: syntect::highlighting::Color) -> Rgba {
    Rgba {
        r: color.r as f32 / 255.0,
        g: color.g as f32 / 255.0,
        b: color.b as f32 / 255.0,
        a: color.a as f32 / 255.0,
    }
}

/// Convert syntect FontStyle to Attr
fn syntect_font_to_attr(style: FontStyle) -> Attr {
    let mut attr = Attr::empty();

    if style.contains(FontStyle::BOLD) {
        attr |= Attr::BOLD;
    }
    if style.contains(FontStyle::ITALIC) {
        attr |= Attr::ITALIC;
    }
    if style.contains(FontStyle::UNDERLINE) {
        attr |= Attr::UNDERLINE;
    }

    attr
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
        assert!(first_line.runs.len() > 0);

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
