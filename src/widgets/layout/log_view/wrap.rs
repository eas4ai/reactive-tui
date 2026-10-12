//! The lines a log view's rows take at a width: a row wider than the view
//! continues on the next lines, cut by the character (NAV-008). The counts
//! are kept between renders, so a long log is measured once and then only
//! for the rows appended since.

use super::Rows;
use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// The text of a row as it is painted: a tab as four spaces, a line break
/// kept as the end of a line, and no other control character, so what is
/// painted takes the cells it was measured to take.
fn painted(text: &str) -> Cow<'_, str> {
    if !text.chars().any(|c| c.is_control() && c != '\n') {
        return Cow::Borrowed(text);
    }
    let mut clean = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\t' => clean.push_str("    "),
            '\n' => clean.push('\n'),
            c if c.is_control() => {}
            c => clean.push(c),
        }
    }
    Cow::Owned(clean)
}

/// Calls `line` with each line `text` takes at `width` cells, in order. A
/// row always takes one line, an empty one when it holds no text; a glyph
/// wider than the view takes a line of its own.
fn each_line(text: &str, width: usize, mut line: impl FnMut(&str)) {
    let width = width.max(1);
    let text = painted(text);
    for part in text.split('\n') {
        let mut start = 0;
        let mut cells = 0;
        for (byte, grapheme) in part.grapheme_indices(true) {
            let glyph = UnicodeWidthStr::width(grapheme);
            if cells > 0 && cells + glyph > width {
                line(&part[start..byte]);
                start = byte;
                cells = 0;
            }
            cells += glyph;
        }
        line(&part[start..]);
    }
}

/// The lines `text` takes at `width` cells.
pub(super) fn lines(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    each_line(text, width, |line| lines.push(line.to_owned()));
    lines
}

/// How many lines `text` takes at `width` cells.
pub(super) fn count(text: &str, width: usize) -> usize {
    let mut count = 0;
    each_line(text, width, |_| count += 1);
    count
}

/// The lines each row of a log takes at one width, from the log's first row,
/// with the line each row starts on.
#[derive(Debug, Default)]
pub(super) struct Wrapped {
    /// The width the rows were measured at.
    width: usize,
    /// The id of the first row measured.
    first: i64,
    /// The log's clear count when the rows were measured.
    epoch: u64,
    /// The lines of each row, in order.
    counts: Vec<usize>,
    /// The line each row starts on.
    starts: Vec<usize>,
    /// The lines of every row measured.
    total: usize,
}

impl Wrapped {
    /// Brings the counts up to date with `rows` at `width`: from scratch when
    /// the width, the first row or the log's clear count changed, else for
    /// the rows appended since the last time.
    pub(super) fn refresh(&mut self, rows: &Rows, width: usize) {
        if self.width != width
            || self.first != rows.first
            || self.epoch != rows.epoch
            || self.counts.len() > rows.rows.len()
        {
            *self = Self {
                width,
                first: rows.first,
                epoch: rows.epoch,
                ..Self::default()
            };
        }
        for row in &rows.rows[self.counts.len()..] {
            let lines = count(&row.text, width);
            self.starts.push(self.total);
            self.counts.push(lines);
            self.total += lines;
        }
    }

    /// The lines of every row.
    pub(super) fn total(&self) -> usize {
        self.total
    }

    /// The line row `index` starts on.
    pub(super) fn start(&self, index: usize) -> usize {
        self.starts.get(index).copied().unwrap_or(self.total)
    }

    /// The lines row `index` takes.
    pub(super) fn lines_of(&self, index: usize) -> usize {
        self.counts.get(index).copied().unwrap_or(0)
    }

    /// The row that holds line `line` and the line within that row; the
    /// last row's last line for a line past the end, none for no rows.
    pub(super) fn locate(&self, line: usize) -> Option<(usize, usize)> {
        let last = self.counts.len().checked_sub(1)?;
        let index = self
            .starts
            .partition_point(|start| *start <= line)
            .saturating_sub(1)
            .min(last);
        let within = (line - self.starts[index].min(line)).min(self.counts[index] - 1);
        Some((index, within))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::layout::LogRow;

    fn rows(texts: &[&str]) -> Rows {
        Rows {
            rows: texts.iter().map(|text| LogRow::new(text)).collect(),
            first: 0,
            epoch: 0,
        }
    }

    #[test]
    fn nav_008_a_row_continues_on_the_next_lines_cut_by_the_character() {
        let long = "0123456789".repeat(25);
        assert_eq!(
            lines(&long, 100),
            vec![
                long[..100].to_owned(),
                long[100..200].to_owned(),
                long[200..].to_owned()
            ]
        );
        assert_eq!(count(&long, 100), 3);
        assert_eq!(
            lines("", 10),
            vec![String::new()],
            "an empty row takes a line"
        );
        assert_eq!(
            lines("ab\ncd", 10),
            vec!["ab".to_owned(), "cd".to_owned()],
            "a line break ends a line"
        );
        assert_eq!(
            lines("a\tb\u{7}c", 20),
            vec!["a    bc".to_owned()],
            "a tab is four spaces and a control character is left out"
        );
        assert_eq!(
            lines("日本語", 4),
            vec!["日本".to_owned(), "語".to_owned()],
            "a wide glyph that does not fit the line starts the next"
        );
    }

    #[test]
    fn nav_008_the_counts_follow_appended_rows_and_restart_on_a_new_first_row() {
        let mut wrapped = Wrapped::default();
        let mut log = rows(&["one", &"x".repeat(25), "three"]);
        wrapped.refresh(&log, 10);
        assert_eq!(wrapped.total(), 5);
        assert_eq!(
            (wrapped.start(2), wrapped.lines_of(1)),
            (4, 3),
            "the third row starts after the second's three lines"
        );
        assert_eq!(wrapped.locate(0), Some((0, 0)));
        assert_eq!(wrapped.locate(3), Some((1, 2)));
        assert_eq!(wrapped.locate(4), Some((2, 0)));
        assert_eq!(
            wrapped.locate(99),
            Some((2, 0)),
            "past the end is the last line"
        );

        log.rows.push(LogRow::new("four"));
        wrapped.refresh(&log, 10);
        assert_eq!(wrapped.total(), 6, "an appended row adds its lines");

        log.rows.insert(0, LogRow::new("zero"));
        log.first -= 1;
        wrapped.refresh(&log, 10);
        assert_eq!(
            (wrapped.total(), wrapped.start(1)),
            (7, 1),
            "a prepended row measures the log again"
        );
        assert_eq!(Wrapped::default().locate(0), None, "no rows hold no line");
    }
}
