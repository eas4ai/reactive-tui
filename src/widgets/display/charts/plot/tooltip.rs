//! Tooltip box layout (CHT-018): a bordered box beside the hovered index
//! with one swatch, name and value row per series, at most eight rows before
//! it summarizes, placed right of the anchor and flipped left or above when it
//! would cross the chart's edge.

use super::axis::{fit_label, text_width};
use super::layout::Rect;
use super::legend::SWATCH;
use super::{Rgba, TextSink};

/// Most rows shown before the tooltip summarizes the rest.
pub const MAX_ROWS: usize = 8;

/// One tooltip row.
#[derive(Debug, Clone, PartialEq)]
pub struct TooltipRow {
    /// Swatch color.
    pub color: Option<Rgba>,
    /// Series name.
    pub name: String,
    /// Value text, kept whole; the name is cut first when space is short.
    pub value: String,
}

/// A tooltip for one hovered index.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tooltip {
    /// Optional first line, usually the category label.
    pub title: Option<String>,
    /// One row per series.
    pub rows: Vec<TooltipRow>,
}

/// A laid-out tooltip: its lines and the box around them.
#[derive(Debug, Clone, PartialEq)]
pub struct TooltipBox {
    /// Content lines with their swatch colors.
    pub lines: Vec<(Option<Rgba>, String)>,
    /// Outer width including the border.
    pub width: usize,
    /// Outer height including the border.
    pub height: usize,
}

impl Tooltip {
    /// Lay the tooltip out for a chart `area`, so the box is at most as wide
    /// as the area.
    pub fn layout(&self, area: Rect) -> TooltipBox {
        let inner_max = area.w.saturating_sub(2).max(1);
        let mut lines: Vec<(Option<Rgba>, String)> = Vec::new();
        if let Some(title) = &self.title {
            lines.push((None, fit_label(title, inner_max)));
        }
        let shown = if self.rows.len() > MAX_ROWS {
            MAX_ROWS - 1
        } else {
            self.rows.len()
        };
        for row in &self.rows[..shown] {
            // The swatch takes two cells; the value is kept whole and the
            // name is cut first, then dropped when fewer than two cells
            // remain for it.
            let value = fit_label(&row.value, inner_max.saturating_sub(2));
            let room = inner_max.saturating_sub(2 + 1 + text_width(&value));
            let text = if room >= 2 {
                format!("{} {value}", fit_label(&row.name, room))
            } else {
                value
            };
            lines.push((row.color, text));
        }
        if self.rows.len() > shown {
            let rest = self.rows.len() - shown;
            let total: f64 = self.rows[shown..]
                .iter()
                .filter_map(|r| {
                    r.value
                        .split(':')
                        .next_back()?
                        .split(';')
                        .next()?
                        .trim()
                        .parse::<f64>()
                        .ok()
                })
                .sum();
            lines.push((
                None,
                fit_label(&format!("+{rest} more, sum {total}"), inner_max),
            ));
        }
        let width = lines
            .iter()
            .map(|(color, text)| text_width(text) + if color.is_some() { 2 } else { 0 })
            .max()
            .unwrap_or(0)
            .min(inner_max)
            + 2;
        let height = lines.len() + 2;
        TooltipBox {
            lines,
            width,
            height,
        }
    }
}

impl TooltipBox {
    /// Top-left cell for a box anchored at `anchor` (the hovered cell) inside
    /// `area`: right of the anchor, or left when it would cross the right
    /// edge; below the anchor row, or above when it would cross the bottom.
    pub fn place(&self, anchor: (usize, usize), area: Rect) -> (usize, usize) {
        self.place_beside((anchor.0, anchor.0), anchor.1, area)
    }

    /// Top-left cell for a box beside a shape that spans columns `left` to
    /// `right` on row `row` inside `area`: right of `right`, or left of
    /// `left` when it would cross the right edge, so the box never covers
    /// the shape; below the row, or above when it would cross the bottom.
    pub fn place_beside(
        &self,
        (left, right): (usize, usize),
        row: usize,
        area: Rect,
    ) -> (usize, usize) {
        let anchor = (right, row);
        let x = if right + 2 + self.width <= area.right() {
            right + 2
        } else if left > area.x + self.width {
            left - 1 - self.width
        } else {
            area.right().saturating_sub(self.width).max(area.x)
        };
        let y = if anchor.1 + 1 + self.height <= area.bottom() {
            anchor.1 + 1
        } else if anchor.1 >= area.y + self.height {
            anchor.1 - self.height
        } else {
            area.bottom().saturating_sub(self.height).max(area.y)
        };
        (x, y)
    }

    /// Draw the box with its top-left at `at`, clipped to `area`.
    pub fn draw(
        &self,
        sink: &mut dyn TextSink,
        at: (usize, usize),
        area: Rect,
        color: Option<Rgba>,
    ) {
        let (x0, y0) = at;
        let w = self.width.min(area.right().saturating_sub(x0));
        let h = self.height.min(area.bottom().saturating_sub(y0));
        if w < 2 || h < 2 {
            return;
        }
        let top = format!("╭{}╮", "─".repeat(w - 2));
        let bottom = format!("╰{}╯", "─".repeat(w - 2));
        sink.text(x0, y0, w, &top, color);
        for row in 1..h - 1 {
            sink.text(x0, y0 + row, w, &format!("│{}│", " ".repeat(w - 2)), color);
            if let Some((swatch, text)) = self.lines.get(row - 1) {
                let mut x = x0 + 1;
                if let Some(swatch) = swatch {
                    sink.text(x, y0 + row, 1, SWATCH, Some(*swatch));
                    x += 2;
                }
                let room = (x0 + w - 1).saturating_sub(x);
                sink.text(x, y0 + row, room, &fit_label(text, room), None);
            }
        }
        sink.text(x0, y0 + h - 1, w, &bottom, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tooltip(n: usize) -> Tooltip {
        Tooltip {
            title: None,
            rows: (0..n)
                .map(|i| TooltipRow {
                    color: Some((1.0, 0.0, 0.0, 1.0)),
                    name: format!("series{i}"),
                    value: format!("p1: {i}"),
                })
                .collect(),
        }
    }

    #[test]
    fn rows_beyond_eight_are_summarized() {
        let boxed = tooltip(10).layout(Rect::sized(60, 20));
        assert_eq!(boxed.lines.len(), MAX_ROWS);
        assert!(boxed.lines[7].1.starts_with("+3 more"));
        assert_eq!(boxed.height, MAX_ROWS + 2);
        assert_eq!(tooltip(8).layout(Rect::sized(60, 20)).lines.len(), 8);
    }

    #[test]
    fn narrow_areas_cut_the_name_but_keep_the_value() {
        let boxed = tooltip(1).layout(Rect::sized(14, 10));
        assert!(boxed.width <= 14);
        assert!(boxed.lines[0].1.ends_with("p1: 0"), "{}", boxed.lines[0].1);
    }

    #[test]
    fn placement_flips_at_the_right_and_bottom_edges() {
        let boxed = tooltip(2).layout(Rect::sized(40, 12));
        assert_eq!(boxed.place((5, 3), Rect::sized(40, 12)), (7, 4));
        let (x, _) = boxed.place((38, 3), Rect::sized(40, 12));
        assert!(x + boxed.width <= 38, "flipped left: {x}");
        let (_, y) = boxed.place((5, 11), Rect::sized(40, 12));
        assert!(y + boxed.height <= 11, "flipped above: {y}");
    }
}
