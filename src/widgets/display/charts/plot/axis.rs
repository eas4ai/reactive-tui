//! Axis and grid placement in the text layer.

use super::layout::Rect;
use super::tick::Tick;
use super::{Rgba, TextSink};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Which way an axis runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Along the bottom of the plot; labels sit under their ticks.
    Horizontal,
    /// Along the left of the plot; labels sit left of their ticks.
    Vertical,
}

/// Cell width of `text`.
pub fn text_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// `text` cut to at most `max` cells, ending in an ellipsis when cut.
pub fn fit_label(text: &str, max: usize) -> String {
    if text_width(text) <= max {
        return text.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut width = 0;
    for grapheme in text.graphemes(true) {
        let w = text_width(grapheme);
        if width + w > max.saturating_sub(1) {
            break;
        }
        out.push_str(grapheme);
        width += w;
    }
    out.push('…');
    out
}

/// An axis: its ticks in cell coordinates relative to the plot rectangle,
/// the label stride, and an optional title.
#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    /// Direction.
    pub orientation: Orientation,
    /// Ticks with `position` measured in cells from the plot's left (for a
    /// horizontal axis) or top (for a vertical axis).
    pub ticks: Vec<Tick>,
    /// Show every `skip`-th label, starting with the first.
    pub skip: usize,
    /// Axis title.
    pub title: Option<String>,
    /// Whether to draw the axis line itself.
    pub line: bool,
}

impl Axis {
    /// A horizontal axis.
    pub fn horizontal(ticks: Vec<Tick>) -> Self {
        Self {
            orientation: Orientation::Horizontal,
            ticks,
            skip: 1,
            title: None,
            line: true,
        }
    }

    /// A vertical axis.
    pub fn vertical(ticks: Vec<Tick>) -> Self {
        Self {
            orientation: Orientation::Vertical,
            ticks,
            skip: 1,
            title: None,
            line: true,
        }
    }

    /// Set the label stride.
    pub fn with_skip(mut self, skip: usize) -> Self {
        self.skip = skip.max(1);
        self
    }

    /// Set the title.
    pub fn with_title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    /// Widest label in cells.
    pub fn label_width(&self) -> usize {
        self.ticks
            .iter()
            .map(|t| text_width(&t.label))
            .max()
            .unwrap_or(0)
    }

    /// Labels shown after applying the stride.
    pub fn shown(&self) -> impl Iterator<Item = &Tick> {
        self.ticks.iter().step_by(self.skip.max(1))
    }

    /// Draw the axis. For a vertical axis `rect` is the label column left of
    /// `plot`; for a horizontal axis it is the row(s) under `plot`. The axis
    /// line is drawn on the plot's edge cells in the under layer.
    pub fn draw(&self, sink: &mut dyn TextSink, rect: Rect, plot: Rect, color: Option<Rgba>) {
        match self.orientation {
            Orientation::Vertical => {
                if self.line && plot.w > 0 {
                    for row in plot.y..plot.bottom() {
                        sink.under(plot.x, row, "│", color);
                    }
                    if plot.h > 0 {
                        sink.under(plot.x, plot.bottom() - 1, "└", color);
                    }
                }
                if rect.w == 0 {
                    return;
                }
                for tick in self.shown() {
                    let row = tick.position.round();
                    if row < 0.0 || row as usize > plot.h {
                        continue;
                    }
                    let row = row.min(plot.h as f64 - 1.0);
                    let label = fit_label(&tick.label, rect.w);
                    let width = text_width(&label);
                    let x = rect.x + rect.w.saturating_sub(width);
                    sink.text(x, plot.y + row as usize, width, &label, color);
                }
            }
            Orientation::Horizontal => {
                if self.line && plot.h > 0 {
                    for col in plot.x..plot.right() {
                        sink.under(col, plot.bottom() - 1, "─", color);
                    }
                    sink.under(plot.x, plot.bottom() - 1, "└", color);
                }
                if rect.h == 0 {
                    return;
                }
                let mut end = 0usize;
                for tick in self.shown() {
                    let col = tick.position.round();
                    if col < 0.0 || col as usize > plot.w {
                        continue;
                    }
                    let col = col.min(plot.w as f64 - 1.0);
                    let label = fit_label(&tick.label, plot.w);
                    let width = text_width(&label);
                    let centred = (col as usize).saturating_sub(width / 2);
                    let start = centred.min(plot.w.saturating_sub(width));
                    if start < end && end > 0 {
                        continue;
                    }
                    sink.text(plot.x + start, rect.y, width, &label, color);
                    end = start + width + 1;
                }
                if let Some(title) = &self.title {
                    if rect.h > 1 {
                        let title = fit_label(title, plot.w);
                        let width = text_width(&title);
                        sink.text(
                            plot.x + plot.w.saturating_sub(width) / 2,
                            rect.y + 1,
                            width,
                            &title,
                            color,
                        );
                    }
                }
            }
        }
    }
}

/// Grid lines drawn in the under layer at tick rows and columns of the plot.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Grid {
    /// Plot-relative columns that carry a vertical line.
    pub columns: Vec<usize>,
    /// Plot-relative rows that carry a horizontal line.
    pub rows: Vec<usize>,
}

impl Grid {
    /// Grid lines at the given plot-relative columns and rows.
    pub fn new(columns: Vec<usize>, rows: Vec<usize>) -> Self {
        Self { columns, rows }
    }

    /// Draw the grid inside `plot` with `glyph` at every grid cell.
    pub fn draw(&self, sink: &mut dyn TextSink, plot: Rect, glyph: &str, color: Option<Rgba>) {
        for &row in &self.rows {
            if row < plot.h {
                for col in 0..plot.w {
                    sink.under(plot.x + col, plot.y + row, glyph, color);
                }
            }
        }
        for &col in &self.columns {
            if col < plot.w {
                for row in 0..plot.h {
                    sink.under(plot.x + col, plot.y + row, glyph, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct Recorder {
        over: BTreeMap<(usize, usize), String>,
        under: BTreeMap<(usize, usize), String>,
    }
    impl TextSink for Recorder {
        fn text(&mut self, x: usize, y: usize, _: usize, text: &str, _: Option<Rgba>) {
            for (i, g) in text.graphemes(true).enumerate() {
                self.over.insert((x + i, y), g.to_string());
            }
        }
        fn under(&mut self, x: usize, y: usize, glyph: &str, _: Option<Rgba>) {
            self.under.insert((x, y), glyph.to_string());
        }
    }

    fn tick(position: f64, label: &str) -> Tick {
        Tick {
            value: 0.0,
            position,
            label: label.into(),
        }
    }

    #[test]
    fn labels_truncate_with_an_ellipsis() {
        assert_eq!(fit_label("hello", 5), "hello");
        assert_eq!(fit_label("hello world", 6), "hello…");
        assert_eq!(fit_label("hi", 0), "");
    }

    #[test]
    fn vertical_axis_right_aligns_labels_and_draws_the_line() {
        let axis = Axis::vertical(vec![tick(0.0, "10"), tick(3.0, "0")]);
        let mut sink = Recorder::default();
        let plot = Rect {
            x: 3,
            y: 0,
            w: 5,
            h: 4,
        };
        axis.draw(
            &mut sink,
            Rect {
                x: 0,
                y: 0,
                w: 3,
                h: 4,
            },
            plot,
            None,
        );
        assert_eq!(sink.over.get(&(1, 0)).map(String::as_str), Some("1"));
        assert_eq!(sink.over.get(&(2, 3)).map(String::as_str), Some("0"));
        assert_eq!(sink.under.get(&(3, 0)).map(String::as_str), Some("│"));
        assert_eq!(sink.under.get(&(3, 3)).map(String::as_str), Some("└"));
    }

    #[test]
    fn horizontal_axis_skips_labels_by_stride_and_never_overlaps() {
        let ticks: Vec<Tick> = (0..5).map(|i| tick(i as f64 * 2.0, "ab")).collect();
        let axis = Axis::horizontal(ticks).with_skip(2);
        let mut sink = Recorder::default();
        let plot = Rect {
            x: 0,
            y: 0,
            w: 10,
            h: 3,
        };
        axis.draw(
            &mut sink,
            Rect {
                x: 0,
                y: 3,
                w: 10,
                h: 1,
            },
            plot,
            None,
        );
        let shown: Vec<usize> = sink.over.keys().map(|(x, _)| *x).collect();
        assert!(shown.contains(&0) && !shown.contains(&2), "{shown:?}");
        assert_eq!(sink.under.get(&(5, 2)).map(String::as_str), Some("─"));
    }

    #[test]
    fn grid_marks_rows_and_columns_inside_the_plot() {
        let grid = Grid::new(vec![1], vec![0]);
        let mut sink = Recorder::default();
        grid.draw(
            &mut sink,
            Rect {
                x: 2,
                y: 2,
                w: 3,
                h: 2,
            },
            "·",
            None,
        );
        assert_eq!(sink.under.len(), 3 + 2 - 1);
        assert!(sink.under.contains_key(&(3, 3)));
    }
}
