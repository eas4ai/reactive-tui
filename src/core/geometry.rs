/// Core geometry types for reactive-tui
/// Provides domain objects to replace primitive type usage
use std::ops::{Add, Sub};

/// A point in 2D space with usize coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Point {
    /// X coordinate (horizontal position)
    pub x: usize,
    /// Y coordinate (vertical position)
    pub y: usize,
}

impl Point {
    /// Create a new point
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Origin point (0, 0)
    pub const fn origin() -> Self {
        Self { x: 0, y: 0 }
    }

    /// Offset this point by the given amounts
    pub fn offset(&self, dx: isize, dy: isize) -> Option<Self> {
        let new_x = (self.x as isize).checked_add(dx)?;
        let new_y = (self.y as isize).checked_add(dy)?;

        if new_x >= 0 && new_y >= 0 {
            Some(Self {
                x: new_x as usize,
                y: new_y as usize,
            })
        } else {
            None
        }
    }

    /// Calculate Manhattan distance to another point
    pub fn manhattan_distance(&self, other: &Point) -> usize {
        let dx = self.x.abs_diff(other.x);
        let dy = self.y.abs_diff(other.y);
        dx + dy
    }
}

impl From<(usize, usize)> for Point {
    fn from((x, y): (usize, usize)) -> Self {
        Self::new(x, y)
    }
}

impl From<Point> for (usize, usize) {
    fn from(p: Point) -> Self {
        (p.x, p.y)
    }
}

impl Add<Size> for Point {
    type Output = Point;

    fn add(self, size: Size) -> Self::Output {
        Point {
            x: self.x.saturating_add(size.width),
            y: self.y.saturating_add(size.height),
        }
    }
}

/// Size in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Size {
    /// Width in units
    pub width: usize,
    /// Height in units
    pub height: usize,
}

impl Size {
    /// Create a new size
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Create a square size
    pub const fn square(size: usize) -> Self {
        Self {
            width: size,
            height: size,
        }
    }

    /// Zero size
    pub const fn zero() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }

    /// Check if this size is empty (width or height is 0)
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Calculate the area
    pub fn area(&self) -> usize {
        self.width.saturating_mul(self.height)
    }

    /// Check if a point is within this size (when origin is 0,0)
    pub fn contains_point(&self, point: Point) -> bool {
        point.x < self.width && point.y < self.height
    }

    /// Shrink by the given amount on all sides
    pub fn shrink(&self, amount: usize) -> Self {
        let double_amount = amount.saturating_mul(2);
        Self {
            width: self.width.saturating_sub(double_amount),
            height: self.height.saturating_sub(double_amount),
        }
    }

    /// Expand by the given amount on all sides
    pub fn expand(&self, amount: usize) -> Self {
        let double_amount = amount.saturating_mul(2);
        Self {
            width: self.width.saturating_add(double_amount),
            height: self.height.saturating_add(double_amount),
        }
    }
}

impl From<(usize, usize)> for Size {
    fn from((width, height): (usize, usize)) -> Self {
        Self::new(width, height)
    }
}

impl From<Size> for (usize, usize) {
    fn from(s: Size) -> Self {
        (s.width, s.height)
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, other: Size) -> Self::Output {
        Size {
            width: self.width.saturating_add(other.width),
            height: self.height.saturating_add(other.height),
        }
    }
}

impl Sub for Size {
    type Output = Size;

    fn sub(self, other: Size) -> Self::Output {
        Size {
            width: self.width.saturating_sub(other.width),
            height: self.height.saturating_sub(other.height),
        }
    }
}

/// A rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rect {
    /// Top-left corner position of the rectangle
    pub origin: Point,
    /// Width and height of the rectangle
    pub size: Size,
}

impl Rect {
    /// Create a new rectangle
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// Create a rectangle from coordinates and dimensions
    pub const fn from_coords(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// Create a rectangle from two points (inclusive)
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = p1.x.max(p2.x) - x + 1;
        let height = p1.y.max(p2.y) - y + 1;

        Self::from_coords(x, y, width, height)
    }

    /// Get the left coordinate
    pub fn left(&self) -> usize {
        self.origin.x
    }

    /// Get the top coordinate
    pub fn top(&self) -> usize {
        self.origin.y
    }

    /// Get the right coordinate (exclusive)
    pub fn right(&self) -> usize {
        self.origin.x.saturating_add(self.size.width)
    }

    /// Get the bottom coordinate (exclusive)
    pub fn bottom(&self) -> usize {
        self.origin.y.saturating_add(self.size.height)
    }

    /// Get the width
    pub fn width(&self) -> usize {
        self.size.width
    }

    /// Get the height
    pub fn height(&self) -> usize {
        self.size.height
    }

    /// Check if the rectangle is empty
    pub fn is_empty(&self) -> bool {
        self.size.is_empty()
    }

    /// Calculate the area
    pub fn area(&self) -> usize {
        self.size.area()
    }

    /// Check if a point is inside this rectangle
    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x < self.right()
            && point.y < self.bottom()
    }

    /// Check if this rectangle contains another rectangle
    pub fn contains_rect(&self, other: &Rect) -> bool {
        self.contains_point(other.origin)
            && self.contains_point(Point::new(other.right() - 1, other.bottom() - 1))
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// Calculate the intersection with another rectangle
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        let x = self.left().max(other.left());
        let y = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        Some(Rect::from_coords(x, y, right - x, bottom - y))
    }

    /// Calculate the union with another rectangle
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.left().min(other.left());
        let y = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());

        Rect::from_coords(x, y, right - x, bottom - y)
    }

    /// Shrink the rectangle by the given amount on all sides
    pub fn shrink(&self, amount: usize) -> Rect {
        Rect {
            origin: Point::new(
                self.origin.x.saturating_add(amount),
                self.origin.y.saturating_add(amount),
            ),
            size: self.size.shrink(amount),
        }
    }

    /// Expand the rectangle by the given amount on all sides
    pub fn expand(&self, amount: usize) -> Rect {
        Rect {
            origin: Point::new(
                self.origin.x.saturating_sub(amount),
                self.origin.y.saturating_sub(amount),
            ),
            size: self.size.expand(amount),
        }
    }

    /// Clamp this rectangle to be within bounds
    pub fn clamp(&self, bounds: &Rect) -> Rect {
        let x = self.left().max(bounds.left()).min(bounds.right());
        let y = self.top().max(bounds.top()).min(bounds.bottom());
        let right = self.right().min(bounds.right()).max(x);
        let bottom = self.bottom().min(bounds.bottom()).max(y);

        Rect::from_coords(x, y, right - x, bottom - y)
    }
}

impl From<(usize, usize, usize, usize)> for Rect {
    fn from((x, y, width, height): (usize, usize, usize, usize)) -> Self {
        Self::from_coords(x, y, width, height)
    }
}

impl From<Rect> for (usize, usize, usize, usize) {
    fn from(r: Rect) -> Self {
        (r.origin.x, r.origin.y, r.size.width, r.size.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point() {
        let p1 = Point::new(5, 10);
        let p2 = Point::from((3, 7));

        assert_eq!(p1.manhattan_distance(&p2), 5);
        assert_eq!(p1.offset(-2, 3), Some(Point::new(3, 13)));
        assert_eq!(p1.offset(-10, 0), None); // Would be negative

        let p3 = p1 + Size::new(5, 5);
        assert_eq!(p3, Point::new(10, 15));
    }

    #[test]
    fn test_size() {
        let s1 = Size::new(10, 20);
        let s2 = Size::square(15);

        assert_eq!(s1.area(), 200);
        assert!(!s1.is_empty());
        assert!(s1.contains_point(Point::new(5, 10)));
        assert!(!s1.contains_point(Point::new(10, 20)));

        assert_eq!(s1.shrink(2), Size::new(6, 16));
        assert_eq!(s1.expand(3), Size::new(16, 26));

        assert_eq!(s1 + s2, Size::new(25, 35));
        assert_eq!(s1 - Size::new(5, 10), Size::new(5, 10));
    }

    #[test]
    fn test_rect() {
        let r1 = Rect::from_coords(10, 10, 20, 30);
        let r2 = Rect::from_coords(25, 20, 20, 30);

        assert_eq!(r1.right(), 30);
        assert_eq!(r1.bottom(), 40);
        assert_eq!(r1.area(), 600);

        assert!(r1.contains_point(Point::new(15, 20)));
        assert!(!r1.contains_point(Point::new(30, 40)));

        assert!(r1.intersects(&r2));

        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection, Rect::from_coords(25, 20, 5, 20));

        let union = r1.union(&r2);
        assert_eq!(union, Rect::from_coords(10, 10, 35, 40));

        let shrunk = r1.shrink(2);
        assert_eq!(shrunk, Rect::from_coords(12, 12, 16, 26));

        let bounds = Rect::from_coords(0, 0, 50, 50);
        let clamped = Rect::from_coords(45, 45, 20, 20).clamp(&bounds);
        assert_eq!(clamped, Rect::from_coords(45, 45, 5, 5));
    }

    #[test]
    fn test_rect_from_points() {
        let r = Rect::from_points(Point::new(10, 20), Point::new(5, 15));
        assert_eq!(r, Rect::from_coords(5, 15, 6, 6));
    }
}
