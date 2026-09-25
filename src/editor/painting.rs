//! Shared grapheme overlays and clipping for both editor output paths.

use super::{cursor::Cursor, positions, GapBuffer};
use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Cell, Rgba, Surface};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const SELECTION_BG: Rgba = Rgba {
    r: 0.2,
    g: 0.4,
    b: 0.6,
    a: 1.0,
};
const CURSOR_BG: Rgba = Rgba {
    r: 0.8,
    g: 0.8,
    b: 0.8,
    a: 1.0,
};

pub(super) struct LinePainter<'a> {
    pub buffer: &'a GapBuffer,
    pub cursor: &'a Cursor,
    pub width: usize,
    pub gutter: usize,
    pub number_fg: Rgba,
    pub background: Rgba,
}

impl LinePainter<'_> {
    pub fn paint(&self, index: usize, source: StyledLine) -> StyledLine {
        let mut output = StyledLine::new();
        if self.gutter > 0 {
            output.push(StyledRun::new(
                format!("{:>width$} ", index + 1, width = self.gutter - 1),
                self.number_fg,
                self.background,
                Attr::empty(),
            ));
        }
        let text = self.buffer.get_line(index);
        let text = if self.buffer.get_char(self.buffer.line_end(index)) == Some('\n') {
            text.strip_suffix('\r').unwrap_or(&text)
        } else {
            &text
        };
        let mut position = self.buffer.line_start(index);
        let mut column = 0;
        let mut run_index = 0;
        let mut run_end = source.runs.first().map_or(0, |r| r.text.chars().count());
        for grapheme in text.graphemes(true) {
            let local = position - self.buffer.line_start(index);
            while run_end <= local && run_index + 1 < source.runs.len() {
                run_index += 1;
                run_end += source.runs[run_index].text.chars().count();
            }
            let mut run = source.runs.get(run_index).map_or_else(
                || StyledRun::plain(""),
                |run| StyledRun::new("", run.fg, run.bg, run.attr),
            );
            let width = positions::width(grapheme, column);
            run.text = if grapheme == "\t" {
                " ".repeat(width)
            } else if grapheme.chars().any(char::is_control) {
                "�".to_owned()
            } else if grapheme.width() == 0 {
                format!("◌{grapheme}")
            } else {
                grapheme.to_owned()
            };
            let selected = self
                .cursor
                .selection_range()
                .is_some_and(|(start, end)| position >= start && position < end);
            if selected {
                run.bg = SELECTION_BG;
            } else if self.cursor.position == position {
                run.bg = CURSOR_BG;
                run.fg = Rgba::black();
            }
            output.push(run);
            column += width;
            position += grapheme.chars().count();
        }
        if self.cursor.position == position {
            output.push(StyledRun::new(" ", Rgba::black(), CURSOR_BG, Attr::empty()));
        }
        clip(output, self.width)
    }
}

fn clip(line: StyledLine, width: usize) -> StyledLine {
    let mut output = StyledLine::new();
    let mut used = 0;
    for run in line.runs {
        let mut text = String::new();
        for grapheme in run.text.graphemes(true) {
            let next = used + grapheme.width();
            if next > width {
                if !text.is_empty() {
                    output.push(StyledRun::new(text, run.fg, run.bg, run.attr));
                }
                return output;
            }
            text.push_str(grapheme);
            used = next;
        }
        if !text.is_empty() {
            output.push(StyledRun::new(text, run.fg, run.bg, run.attr));
        }
    }
    output
}

pub(super) fn render(
    surface: &mut Surface,
    lines: Vec<StyledLine>,
    origin: (usize, usize),
    size: (usize, usize),
    background: Rgba,
) {
    let (x, y) = origin;
    let (width, height) = size;
    let (surface_width, surface_height) = surface.dims();
    for row in y..y.saturating_add(height).min(surface_height) {
        for column in x..x.saturating_add(width).min(surface_width) {
            surface.set(
                column,
                row,
                Cell {
                    ch: ' ',
                    bg: background,
                    ..Cell::default()
                },
            );
        }
    }
    for (row, line) in lines.into_iter().take(height).enumerate() {
        if y.saturating_add(row) >= surface_height {
            break;
        }
        let line = clip(line, surface_width.saturating_sub(x));
        let mut column = x;
        for run in line.runs {
            for grapheme in run.text.graphemes(true) {
                surface.set_grapheme(
                    column,
                    y + row,
                    grapheme,
                    Cell {
                        fg: run.fg,
                        bg: run.bg,
                        attr: run.attr,
                        ..Cell::default()
                    },
                );
                column += grapheme.width();
            }
        }
    }
}
