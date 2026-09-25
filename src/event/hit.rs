use super::router::NodeId;
use std::collections::BTreeMap;

/// A 2D point for event hit testing
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    /// X coordinate (horizontal position)
    pub x: f32,
    /// Y coordinate (vertical position)
    pub y: f32,
}

impl Point {
    /// Create a new point with the given coordinates
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a point at the origin (0, 0)
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Bounding rectangle for hit testing
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    /// X coordinate of the left edge
    pub x: f32,
    /// Y coordinate of the top edge
    pub y: f32,
    /// Width of the rectangle
    pub width: f32,
    /// Height of the rectangle
    pub height: f32,
}

impl Bounds {
    /// Create a new bounding rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create bounds from two corner points
    pub fn from_points(top_left: Point, bottom_right: Point) -> Self {
        Self {
            x: top_left.x,
            y: top_left.y,
            width: bottom_right.x - top_left.x,
            height: bottom_right.y - top_left.y,
        }
    }

    /// Check if a point is within these bounds
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    /// Check if two bounds intersect
    pub fn intersects(&self, other: &Bounds) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Get the intersection of two bounds
    pub fn intersection(&self, other: &Bounds) -> Option<Bounds> {
        if !self.intersects(other) {
            return None;
        }

        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);

        Some(Bounds::new(x, y, right - x, bottom - y))
    }

    /// Expand bounds by a margin
    pub fn expand(&self, margin: f32) -> Bounds {
        Bounds::new(
            self.x - margin,
            self.y - margin,
            self.width + margin * 2.0,
            self.height + margin * 2.0,
        )
    }

    /// Get the top-left corner point
    pub fn top_left(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Get the bottom-right corner point
    pub fn bottom_right(&self) -> Point {
        Point::new(self.x + self.width, self.y + self.height)
    }

    /// Get the center point of the bounds
    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Spatial index for efficient hit testing using a QuadTree
pub struct HitTest {
    root: QuadNode,
    node_bounds: BTreeMap<NodeId, (Bounds, i32)>, // NodeId -> (Bounds, z-index)
}

struct QuadNode {
    bounds: Bounds,
    nodes: Vec<(NodeId, Bounds, i32)>, // (NodeId, Bounds, z-index)
    children: Option<Box<[QuadNode; 4]>>,
    max_depth: usize,
    max_nodes: usize,
}

impl QuadNode {
    fn new(bounds: Bounds, max_depth: usize, max_nodes: usize) -> Self {
        Self {
            bounds,
            nodes: Vec::new(),
            children: None,
            max_depth,
            max_nodes,
        }
    }

    fn insert(&mut self, node_id: NodeId, bounds: Bounds, z_index: i32, depth: usize) {
        // If we have children, try to insert into them
        if self.children.is_some() {
            if let Some(index) = self.get_child_index(&bounds) {
                if let Some(children) = &mut self.children {
                    children[index].insert(node_id, bounds, z_index, depth + 1);
                    return;
                }
            }
        }

        // Add to this node
        self.nodes.push((node_id, bounds, z_index));

        // Split if we exceed capacity and haven't reached max depth
        if self.children.is_none() && self.nodes.len() > self.max_nodes && depth < self.max_depth {
            self.split(depth);
        }
    }

    fn split(&mut self, depth: usize) {
        let half_width = self.bounds.width / 2.0;
        let half_height = self.bounds.height / 2.0;
        let x = self.bounds.x;
        let y = self.bounds.y;

        // Create four children (NW, NE, SW, SE)
        self.children = Some(Box::new([
            QuadNode::new(
                Bounds::new(x, y, half_width, half_height),
                self.max_depth,
                self.max_nodes,
            ),
            QuadNode::new(
                Bounds::new(x + half_width, y, half_width, half_height),
                self.max_depth,
                self.max_nodes,
            ),
            QuadNode::new(
                Bounds::new(x, y + half_height, half_width, half_height),
                self.max_depth,
                self.max_nodes,
            ),
            QuadNode::new(
                Bounds::new(x + half_width, y + half_height, half_width, half_height),
                self.max_depth,
                self.max_nodes,
            ),
        ]));

        // Move existing nodes to children
        let nodes = std::mem::take(&mut self.nodes);
        for (node_id, bounds, z_index) in nodes {
            self.insert(node_id, bounds, z_index, depth);
        }
    }

    fn get_child_index(&self, bounds: &Bounds) -> Option<usize> {
        let mid_x = self.bounds.x + self.bounds.width / 2.0;
        let mid_y = self.bounds.y + self.bounds.height / 2.0;

        let fits_left = bounds.x + bounds.width <= mid_x;
        let fits_right = bounds.x >= mid_x;
        let fits_top = bounds.y + bounds.height <= mid_y;
        let fits_bottom = bounds.y >= mid_y;

        match (fits_left, fits_right, fits_top, fits_bottom) {
            (true, false, true, false) => Some(0), // NW
            (false, true, true, false) => Some(1), // NE
            (true, false, false, true) => Some(2), // SW
            (false, true, false, true) => Some(3), // SE
            _ => None,                             // Doesn't fit entirely in one quadrant
        }
    }

    fn query(&self, point: Point, results: &mut Vec<(NodeId, i32)>) {
        // Check nodes at this level
        for (node_id, bounds, z_index) in &self.nodes {
            if bounds.contains(point) {
                results.push((*node_id, *z_index));
            }
        }

        // Recursively check children
        if let Some(children) = &self.children {
            for child in children.iter() {
                if child.bounds.contains(point) {
                    child.query(point, results);
                }
            }
        }
    }

    fn query_bounds(&self, query_bounds: &Bounds, results: &mut Vec<(NodeId, i32)>) {
        // Check nodes at this level
        for (node_id, bounds, z_index) in &self.nodes {
            if bounds.intersects(query_bounds) {
                results.push((*node_id, *z_index));
            }
        }

        // Recursively check children
        if let Some(children) = &self.children {
            for child in children.iter() {
                if child.bounds.intersects(query_bounds) {
                    child.query_bounds(query_bounds, results);
                }
            }
        }
    }

    fn remove(&mut self, node_id: NodeId, bounds: Bounds) {
        // Remove from nodes at this level
        self.nodes.retain(|(id, _, _)| *id != node_id);

        // Recursively remove from children
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.bounds.intersects(&bounds) {
                    child.remove(node_id, bounds);
                }
            }
        }
    }
}

impl HitTest {
    /// Create a new hit test system
    ///
    /// # Arguments
    /// * `width` - Width of the hit test area
    /// * `height` - Height of the hit test area
    ///
    /// # Returns
    /// A new `HitTest` instance
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            root: QuadNode::new(
                Bounds::new(0.0, 0.0, width, height),
                8,  // max depth
                10, // max nodes per quad
            ),
            node_bounds: BTreeMap::new(),
        }
    }

    /// Update bounds for a node with optimized tree updates
    pub fn update_bounds(&mut self, node_id: NodeId, bounds: Bounds, z_index: i32) {
        // Check if node already exists
        if let Some((old_bounds, old_z_index)) = self.node_bounds.get(&node_id) {
            // If bounds and z-index haven't changed, no update needed
            if *old_bounds == bounds && *old_z_index == z_index {
                return;
            }

            // Remove old node from tree efficiently
            self.root.remove(node_id, *old_bounds);
        }

        // Store new bounds
        self.node_bounds.insert(node_id, (bounds, z_index));

        // Insert new node into tree efficiently
        self.root.insert(node_id, bounds, z_index, 0);
    }

    /// Remove a node from hit testing with optimized tree updates
    pub fn remove_node(&mut self, node_id: NodeId) {
        if let Some((bounds, _)) = self.node_bounds.remove(&node_id) {
            // Remove from tree efficiently
            self.root.remove(node_id, bounds);
        }
    }

    /// Find the topmost node at a point
    pub fn hit_test(&self, point: Point) -> Option<NodeId> {
        let mut results = Vec::new();
        self.root.query(point, &mut results);

        // Sort by z-index (highest first) and return the top one
        results.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        results.first().map(|(node_id, _)| *node_id)
    }

    /// Find all nodes at a point, sorted by z-index
    pub fn hit_test_all(&self, point: Point) -> Vec<NodeId> {
        let mut results = Vec::new();
        self.root.query(point, &mut results);

        // Sort by z-index (highest first)
        results.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        results.into_iter().map(|(node_id, _)| node_id).collect()
    }

    /// Find all nodes intersecting with bounds
    pub fn query_bounds(&self, bounds: Bounds) -> Vec<NodeId> {
        let mut results = Vec::new();
        self.root.query_bounds(&bounds, &mut results);

        // Sort by z-index (highest first)
        results.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        results.into_iter().map(|(node_id, _)| node_id).collect()
    }

    fn rebuild(&mut self) {
        // Create new root
        let bounds = self.root.bounds;
        self.root = QuadNode::new(bounds, 8, 10);

        // Re-insert all nodes
        for (node_id, (bounds, z_index)) in &self.node_bounds {
            self.root.insert(*node_id, *bounds, *z_index, 0);
        }
    }

    /// Resize the hit test area
    pub fn resize(&mut self, width: f32, height: f32) {
        self.root = QuadNode::new(Bounds::new(0.0, 0.0, width, height), 8, 10);
        self.rebuild();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(10.0, 10.0, 20.0, 20.0);

        assert!(bounds.contains(Point::new(15.0, 15.0)));
        assert!(bounds.contains(Point::new(10.0, 10.0)));
        assert!(!bounds.contains(Point::new(5.0, 15.0)));
        assert!(!bounds.contains(Point::new(30.0, 15.0)));
    }

    #[test]
    fn test_bounds_intersection() {
        let a = Bounds::new(10.0, 10.0, 20.0, 20.0);
        let b = Bounds::new(20.0, 20.0, 20.0, 20.0);

        assert!(a.intersects(&b));

        let intersection = a.intersection(&b).unwrap();
        assert_eq!(intersection, Bounds::new(20.0, 20.0, 10.0, 10.0));
    }

    #[test]
    fn test_hit_test() {
        let mut hit_test = HitTest::new(100.0, 100.0);

        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();

        // Add overlapping nodes with different z-indices
        hit_test.update_bounds(node1, Bounds::new(10.0, 10.0, 30.0, 30.0), 1);
        hit_test.update_bounds(node2, Bounds::new(20.0, 20.0, 30.0, 30.0), 2);
        hit_test.update_bounds(node3, Bounds::new(30.0, 30.0, 30.0, 30.0), 3);

        // Test hit at overlapping point
        let hit = hit_test.hit_test(Point::new(35.0, 35.0));
        assert_eq!(hit, Some(node3)); // Highest z-index

        // Test hit at non-overlapping point
        let hit = hit_test.hit_test(Point::new(15.0, 15.0));
        assert_eq!(hit, Some(node1));

        // Test miss
        let hit = hit_test.hit_test(Point::new(5.0, 5.0));
        assert_eq!(hit, None);
    }

    #[test]
    fn test_query_bounds() {
        let mut hit_test = HitTest::new(100.0, 100.0);

        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();

        hit_test.update_bounds(node1, Bounds::new(10.0, 10.0, 20.0, 20.0), 1);
        hit_test.update_bounds(node2, Bounds::new(30.0, 30.0, 20.0, 20.0), 2);
        hit_test.update_bounds(node3, Bounds::new(50.0, 50.0, 20.0, 20.0), 3);

        // Query a region that intersects node1 and node2
        let results = hit_test.query_bounds(Bounds::new(15.0, 15.0, 30.0, 30.0));
        assert_eq!(results.len(), 2);
        assert!(results.contains(&node1));
        assert!(results.contains(&node2));
    }
}
