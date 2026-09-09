//! Shared terminal-cell text processing for measurement and painting.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Transform {
    #[default]
    None,
    Upper,
    Lower,
    Capitalize,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum WhiteSpace {
    Normal,
    NoWrap,
    #[default]
    Pre,
    PreLine,
    PreWrap,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum WordBreak {
    #[default]
    Normal,
    Words,
    All,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Align {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Options remain unset until inheritance has been resolved.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TextStyle {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strike: Option<bool>,
    pub transform: Option<Transform>,
    pub whitespace: Option<WhiteSpace>,
    pub word_break: Option<WordBreak>,
    pub align: Option<Align>,
    pub ellipsis: Option<bool>,
    pub line_height: Option<usize>,
}

impl TextStyle {
    pub fn inherit(&self, parent: &Self) -> Self {
        Self {
            bold: self.bold.or(parent.bold),
            italic: self.italic.or(parent.italic),
            underline: self.underline.or(parent.underline),
            strike: self.strike.or(parent.strike),
            transform: self.transform.or(parent.transform),
            whitespace: self.whitespace.or(parent.whitespace),
            word_break: self.word_break.or(parent.word_break),
            align: self.align.or(parent.align),
            // Text overflow applies to this box, not its descendants.
            ellipsis: self.ellipsis,
            line_height: self.line_height.or(parent.line_height),
        }
    }

    pub fn prepare(&self, text: &str) -> String {
        let transformed = match self.transform.unwrap_or_default() {
            Transform::None => text.to_owned(),
            Transform::Upper => text.to_uppercase(),
            Transform::Lower => text.to_lowercase(),
            Transform::Capitalize => text
                .split_word_bounds()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                        None => String::new(),
                    }
                })
                .collect(),
        };
        // Keep layout separators, but never let application text become controls.
        let clean: String = transformed
            .chars()
            .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
            .collect();
        let whitespace = self.whitespace.unwrap_or_default();
        let clean = match whitespace {
            WhiteSpace::Normal | WhiteSpace::NoWrap => {
                clean.split_whitespace().collect::<Vec<_>>().join(" ")
            }
            WhiteSpace::PreLine => clean
                .split('\n')
                .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join("\n"),
            WhiteSpace::Pre | WhiteSpace::PreWrap => clean
                .split('\n')
                .map(|line| {
                    let mut expanded = String::new();
                    let mut column = 0;
                    for grapheme in line.graphemes(true) {
                        if grapheme == "\t" {
                            let spaces = 4 - column % 4;
                            expanded.extend(std::iter::repeat_n(' ', spaces));
                            column += spaces;
                        } else {
                            expanded.push_str(grapheme);
                            column += UnicodeWidthStr::width(grapheme);
                        }
                    }
                    expanded
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };
        clean
    }

    /// Input is prepared once when the paint tree is built.
    pub fn lines(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 || text.is_empty() {
            return Vec::new();
        }
        let whitespace = self.whitespace.unwrap_or_default();
        let wrap = matches!(
            whitespace,
            WhiteSpace::Normal | WhiteSpace::PreLine | WhiteSpace::PreWrap
        );
        let collapse = matches!(whitespace, WhiteSpace::Normal | WhiteSpace::PreLine);
        let mut output = Vec::new();
        for paragraph in text.split('\n') {
            let lines = if wrap {
                wrap_line(
                    paragraph,
                    width,
                    self.word_break.unwrap_or_default(),
                    collapse,
                )
            } else {
                vec![paragraph.to_owned()]
            };
            let last = lines.len().saturating_sub(1);
            for (index, line) in lines.into_iter().enumerate() {
                let line = if self.ellipsis == Some(true) {
                    ellipsize(&line, width)
                } else {
                    line
                };
                let spare = width.saturating_sub(UnicodeWidthStr::width(line.as_str()));
                let line = match self.align.unwrap_or_default() {
                    Align::Center => format!("{}{line}", " ".repeat(spare / 2)),
                    Align::Right => format!("{}{line}", " ".repeat(spare)),
                    Align::Justify if index < last => justify(&line, spare),
                    _ => line,
                };
                output.push(line);
            }
        }
        output
    }
}

fn wrap_line(text: &str, width: usize, mode: WordBreak, collapse: bool) -> Vec<String> {
    let tokens: Vec<&str> = if mode == WordBreak::All {
        text.graphemes(true).collect()
    } else {
        text.split_word_bounds().collect()
    };
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for token in tokens {
        let blank = token.chars().all(char::is_whitespace);
        let token_width = UnicodeWidthStr::width(token);
        if used > 0 && used + token_width > width {
            lines.push(if collapse {
                line.trim_end().to_owned()
            } else {
                std::mem::take(&mut line)
            });
            line.clear();
            used = 0;
        }
        if collapse && used == 0 && blank {
            continue;
        }
        if token_width > width && mode == WordBreak::Words {
            for grapheme in token.graphemes(true) {
                let cells = UnicodeWidthStr::width(grapheme);
                if used > 0 && used + cells > width {
                    lines.push(std::mem::take(&mut line));
                    used = 0;
                }
                line.push_str(grapheme);
                used += cells;
            }
        } else {
            line.push_str(token);
            used += token_width;
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(if collapse {
            line.trim_end().to_owned()
        } else {
            line
        });
    }
    lines
}

fn ellipsize(text: &str, width: usize) -> String {
    if UnicodeWidthStr::width(text) <= width {
        return text.to_owned();
    }
    let mut output = String::new();
    let mut used = 0;
    for grapheme in text.graphemes(true) {
        let cells = UnicodeWidthStr::width(grapheme);
        if used + cells > width.saturating_sub(1) {
            break;
        }
        output.push_str(grapheme);
        used += cells;
    }
    if width > 0 {
        output.push('…');
    }
    output
}

fn justify(line: &str, spare: usize) -> String {
    let gaps = line.matches(' ').count();
    if gaps == 0 {
        return line.to_owned();
    }
    let mut output = String::new();
    let mut gap = 0;
    for character in line.chars() {
        output.push(character);
        if character == ' ' {
            output.extend(std::iter::repeat_n(
                ' ',
                spare / gaps + usize::from(gap < spare % gaps),
            ));
            gap += 1;
        }
    }
    output
}
