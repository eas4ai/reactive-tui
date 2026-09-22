//! Size classes and rectangle bookkeeping for the plot area, axes, legend
//! and title (CHT-024).

/// A rectangle in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    /// Left column.
    pub x: usize,
    /// Top row.
    pub y: usize,
    /// Width in columns.
    pub w: usize,
    /// Height in rows.
    pub h: usize,
}

impl Rect {
    /// A rectangle at the origin with the given size.
    pub fn sized(w: usize, h: usize) -> Self {
        Self { x: 0, y: 0, w, h }
    }

    /// Whether the rectangle has area.
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    /// One past the rightmost column.
    pub fn right(&self) -> usize {
        self.x + self.w
    }

    /// One past the bottom row.
    pub fn bottom(&self) -> usize {
        self.y + self.h
    }

    /// Whether the cell at (`x`, `y`) lies inside.
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Remove `rows` from the top and return the removed strip.
    pub fn take_top(&mut self, rows: usize) -> Rect {
        let rows = rows.min(self.h);
        let strip = Rect { h: rows, ..*self };
        self.y += rows;
        self.h -= rows;
        strip
    }

    /// Remove `rows` from the bottom and return the removed strip.
    pub fn take_bottom(&mut self, rows: usize) -> Rect {
        let rows = rows.min(self.h);
        self.h -= rows;
        Rect {
            y: self.y + self.h,
            h: rows,
            ..*self
        }
    }

    /// Remove `cols` from the left and return the removed strip.
    pub fn take_left(&mut self, cols: usize) -> Rect {
        let cols = cols.min(self.w);
        let strip = Rect { w: cols, ..*self };
        self.x += cols;
        self.w -= cols;
        strip
    }

    /// Remove `cols` from the right and return the removed strip.
    pub fn take_right(&mut self, cols: usize) -> Rect {
        let cols = cols.min(self.w);
        self.w -= cols;
        Rect {
            x: self.x + self.w,
            w: cols,
            ..*self
        }
    }
}

/// The layout a chart adopts for its allotted rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeClass {
    /// Under 40 columns or under 8 rows: shapes only, usable down to 8 by 2.
    Mini,
    /// Axes, ticks, a single-row legend and the tooltip.
    Medium,
    /// At least 200 by 40: grid, full labels, multi-row legend, value labels.
    Large,
}

impl SizeClass {
    /// The class for a rectangle of `width` by `height` cells.
    pub fn for_size(width: usize, height: usize) -> Self {
        if width < 40 || height < 8 {
            Self::Mini
        } else if width >= 200 && height >= 40 {
            Self::Large
        } else {
            Self::Medium
        }
    }

    /// Whether axes and their labels are drawn.
    pub fn has_axes(self) -> bool {
        self != Self::Mini
    }

    /// Whether the legend is drawn.
    pub fn has_legend(self) -> bool {
        self != Self::Mini
    }

    /// Whether grid lines are drawn by default.
    pub fn has_grid(self) -> bool {
        self == Self::Large
    }

    /// Whether bars carry value labels.
    pub fn has_value_labels(self) -> bool {
        self == Self::Large
    }

    /// Whether the legend may take several rows.
    pub fn multi_row_legend(self) -> bool {
        self == Self::Large
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_class_boundaries_match_the_contract() {
        assert_eq!(SizeClass::for_size(39, 10), SizeClass::Mini);
        assert_eq!(SizeClass::for_size(80, 7), SizeClass::Mini);
        assert_eq!(SizeClass::for_size(40, 8), SizeClass::Medium);
        assert_eq!(SizeClass::for_size(199, 40), SizeClass::Medium);
        assert_eq!(SizeClass::for_size(200, 39), SizeClass::Medium);
        assert_eq!(SizeClass::for_size(200, 40), SizeClass::Large);
        assert!(!SizeClass::Mini.has_axes() && SizeClass::Large.has_grid());
    }

    #[test]
    fn rect_strips_come_off_the_edges() {
        let mut r = Rect::sized(10, 5);
        assert_eq!(
            r.take_top(1),
            Rect {
                x: 0,
                y: 0,
                w: 10,
                h: 1
            }
        );
        assert_eq!(
            r.take_bottom(1),
            Rect {
                x: 0,
                y: 4,
                w: 10,
                h: 1
            }
        );
        assert_eq!(
            r.take_left(2),
            Rect {
                x: 0,
                y: 1,
                w: 2,
                h: 3
            }
        );
        assert_eq!(
            r.take_right(3),
            Rect {
                x: 7,
                y: 1,
                w: 3,
                h: 3
            }
        );
        assert_eq!(
            r,
            Rect {
                x: 2,
                y: 1,
                w: 5,
                h: 3
            }
        );
        assert!(r.contains(2, 1) && !r.contains(7, 1));
        assert_eq!(r.take_top(9).h, 3);
        assert!(r.is_empty());
    }
}
