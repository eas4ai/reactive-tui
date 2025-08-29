use crate::core::geometry::{Point, Size};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct HitId(pub u32);

#[derive(Clone, Debug)]
pub struct HitGrid {
    w: usize,
    h: usize,
    grid: Vec<Option<HitId>>,
}

impl HitGrid {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            grid: vec![None; w * h],
        }
    }

    /// Create a new HitGrid with Size
    pub fn with_size(size: Size) -> Self {
        Self::new(size.width, size.height)
    }

    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.w, self.h)
    }

    /// Get dimensions as Size
    pub fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }

    pub fn clear(&mut self) {
        self.grid.fill(None);
    }

    pub fn set(&mut self, x: usize, y: usize, id: HitId) {
        if x < self.w && y < self.h {
            let i = self.idx(x, y);
            self.grid[i] = Some(id);
        }
    }

    /// Set using Point
    pub fn set_at(&mut self, point: Point, id: HitId) {
        self.set(point.x, point.y, id)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<HitId> {
        if x < self.w && y < self.h {
            let i = self.idx(x, y);
            self.grid[i]
        } else {
            None
        }
    }

    /// Get using Point
    pub fn get_at(&self, point: Point) -> Option<HitId> {
        self.get(point.x, point.y)
    }
}
