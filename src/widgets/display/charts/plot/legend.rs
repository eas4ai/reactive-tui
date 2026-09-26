//! Legend layout: a swatch and a name per series, on one row (medium) or
//! wrapped over several rows (large), or stacked when placed beside the plot.

use super::axis::{fit_label, text_width};
use super::layout::Rect;
use super::{Rgba, TextSink};

/// The glyph used for legend and tooltip swatches.
pub const SWATCH: &str = "■";

/// One legend entry.
#[derive(Debug, Clone, PartialEq)]
pub struct LegendEntry {
    /// Series name.
    pub name: String,
    /// Series color.
    pub color: Option<Rgba>,
}

/// A legend over a set of entries.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Legend {
    /// Entries in series order.
    pub entries: Vec<LegendEntry>,
}

impl Legend {
    /// A legend for `entries`.
    pub fn new(entries: Vec<LegendEntry>) -> Self {
        Self { entries }
    }

    /// Whether there is anything to draw.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn item_width(entry: &LegendEntry, max_name: usize) -> usize {
        2 + text_width(&fit_label(&entry.name, max_name))
    }

    /// Size needed when entries stack one per row, names cut to `max_name`.
    pub fn stacked_size(&self, max_name: usize) -> (usize, usize) {
        let w = self
            .entries
            .iter()
            .map(|e| Self::item_width(e, max_name))
            .max()
            .unwrap_or(0);
        (w, self.entries.len())
    }

    /// Size needed when entries flow along rows of at most `max_width`
    /// cells, separated by two spaces, wrapping only when `wrap` is set.
    pub fn flowed_size(&self, max_width: usize, max_name: usize, wrap: bool) -> (usize, usize) {
        let mut rows = 1;
        let mut row_width = 0;
        let mut widest = 0;
        for entry in &self.entries {
            let item = Self::item_width(entry, max_name);
            let needed = if row_width == 0 {
                item
            } else {
                row_width + 2 + item
            };
            if row_width > 0 && needed > max_width {
                if !wrap {
                    break;
                }
                rows += 1;
                row_width = item;
            } else {
                row_width = needed;
            }
            widest = widest.max(row_width);
        }
        (widest.min(max_width), rows)
    }

    /// Draw entries one per row inside `rect`.
    pub fn draw_stacked(&self, sink: &mut dyn TextSink, rect: Rect) {
        for (row, entry) in self.entries.iter().take(rect.h).enumerate() {
            let name = fit_label(&entry.name, rect.w.saturating_sub(2));
            sink.text(rect.x, rect.y + row, 1, SWATCH, entry.color);
            sink.text(
                rect.x + 2,
                rect.y + row,
                rect.w.saturating_sub(2),
                &name,
                None,
            );
        }
    }

    /// How many entries, from the first, `draw_stacked` draws in `rect`.
    pub fn stacked_count(&self, rect: Rect) -> usize {
        self.entries.len().min(rect.h)
    }

    /// How many entries, from the first, `draw_flowed` draws in `rect`.
    pub fn flowed_count(&self, rect: Rect, max_name: usize, wrap: bool) -> usize {
        self.flowed_places(rect, max_name, wrap).len()
    }

    /// Where `draw_flowed` puts each entry it draws: the entry's index, its
    /// column and row inside `rect`, and its name cut to `max_name`.
    fn flowed_places(
        &self,
        rect: Rect,
        max_name: usize,
        wrap: bool,
    ) -> Vec<(usize, usize, usize, String)> {
        let mut places = Vec::new();
        let mut x = 0;
        let mut row = 0;
        for (index, entry) in self.entries.iter().enumerate() {
            let name = fit_label(&entry.name, max_name);
            let item = 2 + text_width(&name);
            if x > 0 && x + 2 + item > rect.w {
                if !wrap {
                    break;
                }
                row += 1;
                x = 0;
                if row >= rect.h {
                    break;
                }
            } else if x > 0 {
                x += 2;
            }
            if row >= rect.h {
                break;
            }
            places.push((index, x, row, name));
            x += item;
        }
        places
    }

    /// Draw entries flowing along rows inside `rect`, wrapping when `wrap`.
    pub fn draw_flowed(&self, sink: &mut dyn TextSink, rect: Rect, max_name: usize, wrap: bool) {
        for (index, x, row, name) in self.flowed_places(rect, max_name, wrap) {
            sink.text(
                rect.x + x,
                rect.y + row,
                1,
                SWATCH,
                self.entries[index].color,
            );
            sink.text(
                rect.x + x + 2,
                rect.y + row,
                rect.w.saturating_sub(x + 2),
                &name,
                None,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(names: &[&str]) -> Vec<LegendEntry> {
        names
            .iter()
            .map(|n| LegendEntry {
                name: n.to_string(),
                color: None,
            })
            .collect()
    }

    struct Lines(Vec<(usize, usize, String)>);
    impl TextSink for Lines {
        fn text(&mut self, x: usize, y: usize, _: usize, text: &str, _: Option<Rgba>) {
            self.0.push((x, y, text.to_string()));
        }
        fn under(&mut self, _: usize, _: usize, _: &str, _: Option<Rgba>) {}
    }

    #[test]
    fn flowed_legend_wraps_only_when_asked() {
        let legend = Legend::new(entries(&["alpha", "beta", "gamma"]));
        assert_eq!(legend.flowed_size(100, 20, false), (7 + 2 + 6 + 2 + 7, 1));
        assert_eq!(legend.flowed_size(12, 20, true), (7, 3));
        assert_eq!(legend.flowed_size(12, 20, false), (7, 1));
        assert_eq!(legend.stacked_size(3), (5, 3));
    }

    #[test]
    fn drawing_places_swatch_then_name() {
        let legend = Legend::new(entries(&["S", "T"]));
        let mut sink = Lines(Vec::new());
        legend.draw_stacked(
            &mut sink,
            Rect {
                x: 4,
                y: 1,
                w: 5,
                h: 2,
            },
        );
        assert_eq!(sink.0[0], (4, 1, SWATCH.to_string()));
        assert_eq!(sink.0[1], (6, 1, "S".to_string()));
        assert_eq!(sink.0[3], (6, 2, "T".to_string()));
        let mut sink = Lines(Vec::new());
        legend.draw_flowed(
            &mut sink,
            Rect {
                x: 0,
                y: 0,
                w: 10,
                h: 1,
            },
            5,
            false,
        );
        assert_eq!(sink.0[2], (5, 0, SWATCH.to_string()));
    }

    #[test]
    fn counts_match_what_drawing_draws() {
        let legend = Legend::new(entries(&["alpha", "beta", "gamma"]));
        let row = Rect {
            x: 0,
            y: 0,
            w: 16,
            h: 1,
        };
        // "■ alpha  ■ beta" is 15 columns; gamma does not fit.
        assert_eq!(legend.flowed_count(row, 20, false), 2);
        let mut sink = Lines(Vec::new());
        legend.draw_flowed(&mut sink, row, 20, false);
        assert_eq!(sink.0.len(), 2 * 2);
        let two_rows = Rect { h: 2, ..row };
        assert_eq!(legend.flowed_count(two_rows, 20, true), 3);
        assert_eq!(legend.flowed_count(two_rows, 20, false), 2);
        assert_eq!(legend.stacked_count(Rect { h: 2, ..row }), 2);
        assert_eq!(legend.stacked_count(Rect { h: 9, ..row }), 3);
    }
}
