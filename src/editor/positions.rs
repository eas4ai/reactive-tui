//! Conversions between buffer scalar offsets, graphemes and terminal columns.

use super::GapBuffer;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// A line-sized boundary map; LF and CRLF are included as complete graphemes.
fn boundaries(buffer: &GapBuffer, position: usize) -> Vec<usize> {
    let line = buffer.pos_to_line_col(position).0;
    let start = buffer.line_start(line);
    let end = buffer.line_start(line + 1);
    let text = buffer.get_range(start..end);
    let mut offsets = vec![start];
    let mut offset = start;
    for grapheme in text.graphemes(true) {
        offset += grapheme.chars().count();
        offsets.push(offset);
    }
    offsets
}

pub(super) fn floor(buffer: &GapBuffer, position: usize) -> usize {
    let position = position.min(buffer.len());
    boundaries(buffer, position)
        .into_iter()
        .rev()
        .find(|&offset| offset <= position)
        .unwrap_or(0)
}

pub(super) fn ceil(buffer: &GapBuffer, position: usize) -> usize {
    boundaries(buffer, position)
        .into_iter()
        .find(|&offset| offset >= position)
        .unwrap_or(buffer.len())
}

pub(super) fn previous(buffer: &GapBuffer, position: usize) -> usize {
    if position == 0 {
        return 0;
    }
    floor(buffer, position.min(buffer.len()).saturating_sub(1))
}

pub(super) fn next(buffer: &GapBuffer, position: usize) -> usize {
    ceil(buffer, position.saturating_add(1).min(buffer.len()))
}

pub(super) fn display_column(buffer: &GapBuffer, position: usize) -> usize {
    let position = floor(buffer, position);
    let line = buffer.pos_to_line_col(position).0;
    buffer
        .get_range(buffer.line_start(line)..position)
        .graphemes(true)
        .fold(0, |column, grapheme| column + width(grapheme, column))
}

pub(super) fn at_column(buffer: &GapBuffer, line: usize, target: usize) -> usize {
    let mut position = buffer.line_start(line);
    let mut column = 0;
    for grapheme in buffer.get_line(line).graphemes(true) {
        let next_column = column + width(grapheme, column);
        if next_column > target {
            break;
        }
        column = next_column;
        position += grapheme.chars().count();
    }
    floor(buffer, position)
}

pub(super) fn width(grapheme: &str, column: usize) -> usize {
    if grapheme == "\t" {
        4 - column % 4
    } else {
        grapheme.width().max(1)
    }
}
