use super::router::NodeId;
use std::collections::{HashMap, VecDeque};

/// Direction to move focus
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FocusDirection {
    /// Move to next focusable element in tab order
    Next,
    /// Move to previous focusable element in tab order
    Previous,
    /// Move focus upward
    Up,
    /// Move focus downward
    Down,
    /// Move focus to the left
    Left,
    /// Move focus to the right
    Right,
}

/// Focus management for keyboard navigation
pub struct FocusManager {
    /// Currently focused node
    current: Option<NodeId>,

    /// Focus order for tab navigation
    tab_order: Vec<NodeId>,

    /// Spatial navigation map (for arrow keys)
    spatial_map: HashMap<NodeId, SpatialInfo>,

    /// Focus history for back navigation
    history: VecDeque<NodeId>,
    history_limit: usize,

    /// Nodes that can receive focus
    focusable_nodes: HashMap<NodeId, FocusableInfo>,
}

#[derive(Clone, Debug)]
struct SpatialInfo {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Debug)]
struct FocusableInfo {
    tab_index: Option<i32>,
    enabled: bool,
    visible: bool,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            current: None,
            tab_order: Vec::new(),
            spatial_map: HashMap::new(),
            history: VecDeque::with_capacity(10),
            history_limit: 10,
            focusable_nodes: HashMap::new(),
        }
    }

    /// Register a node as focusable
    pub fn register_focusable(&mut self, node_id: NodeId, tab_index: Option<i32>, enabled: bool) {
        self.focusable_nodes.insert(
            node_id,
            FocusableInfo {
                tab_index,
                enabled,
                visible: true,
            },
        );

        self.rebuild_tab_order();
    }

    /// Unregister a focusable node
    pub fn unregister_focusable(&mut self, node_id: NodeId) {
        self.focusable_nodes.remove(&node_id);
        self.spatial_map.remove(&node_id);
        self.tab_order.retain(|&id| id != node_id);

        // Clear focus if this node was focused
        if self.current == Some(node_id) {
            self.current = None;
        }

        // Remove from history
        self.history.retain(|&id| id != node_id);
    }

    /// Update spatial information for a node
    pub fn update_spatial(&mut self, node_id: NodeId, x: f32, y: f32, width: f32, height: f32) {
        self.spatial_map.insert(
            node_id,
            SpatialInfo {
                x,
                y,
                width,
                height,
            },
        );
    }

    /// Set focus to a specific node
    pub fn set_focus(&mut self, node_id: Option<NodeId>) -> bool {
        if let Some(id) = node_id {
            // Check if node is focusable and enabled
            if let Some(info) = self.focusable_nodes.get(&id) {
                if !info.enabled || !info.visible {
                    return false;
                }
            } else {
                return false;
            }

            // Add previous focus to history
            if let Some(prev) = self.current {
                if prev != id {
                    self.add_to_history(prev);
                }
            }
        }

        self.current = node_id;
        true
    }

    /// Get the currently focused node
    pub fn get_focus(&self) -> Option<NodeId> {
        self.current
    }

    /// Move focus in a direction
    pub fn move_focus(&mut self, direction: FocusDirection) -> Option<NodeId> {
        match direction {
            FocusDirection::Next => self.focus_next(),
            FocusDirection::Previous => self.focus_previous(),
            FocusDirection::Up
            | FocusDirection::Down
            | FocusDirection::Left
            | FocusDirection::Right => self.focus_spatial(direction),
        }
    }

    /// Focus the next element in tab order
    pub fn focus_next(&mut self) -> Option<NodeId> {
        if self.tab_order.is_empty() {
            return None;
        }

        let next_index = if let Some(current) = self.current {
            self.tab_order
                .iter()
                .position(|&id| id == current)
                .map(|i| (i + 1) % self.tab_order.len())
                .unwrap_or(0)
        } else {
            0
        };

        let next_node = self.tab_order[next_index];
        self.set_focus(Some(next_node));
        Some(next_node)
    }

    /// Focus the previous element in tab order
    pub fn focus_previous(&mut self) -> Option<NodeId> {
        if self.tab_order.is_empty() {
            return None;
        }

        let prev_index = if let Some(current) = self.current {
            self.tab_order
                .iter()
                .position(|&id| id == current)
                .map(|i| {
                    if i == 0 {
                        self.tab_order.len() - 1
                    } else {
                        i - 1
                    }
                })
                .unwrap_or(self.tab_order.len() - 1)
        } else {
            self.tab_order.len() - 1
        };

        let prev_node = self.tab_order[prev_index];
        self.set_focus(Some(prev_node));
        Some(prev_node)
    }

    /// Focus element in spatial direction
    pub fn focus_spatial(&mut self, direction: FocusDirection) -> Option<NodeId> {
        let current = self.current?;
        let current_spatial = self.spatial_map.get(&current)?;

        // Find the center of current element
        let current_center_x = current_spatial.x + current_spatial.width / 2.0;
        let current_center_y = current_spatial.y + current_spatial.height / 2.0;

        // Find best candidate in the given direction
        let mut best_candidate = None;
        let mut best_score = f32::MAX;

        for (&node_id, spatial) in &self.spatial_map {
            if node_id == current {
                continue;
            }

            // Check if node is focusable and enabled
            if let Some(info) = self.focusable_nodes.get(&node_id) {
                if !info.enabled || !info.visible {
                    continue;
                }
            } else {
                continue;
            }

            let center_x = spatial.x + spatial.width / 2.0;
            let center_y = spatial.y + spatial.height / 2.0;

            // Check if element is in the right direction
            let is_valid = match direction {
                FocusDirection::Up => center_y < current_center_y,
                FocusDirection::Down => center_y > current_center_y,
                FocusDirection::Left => center_x < current_center_x,
                FocusDirection::Right => center_x > current_center_x,
                _ => false,
            };

            if !is_valid {
                continue;
            }

            // Calculate distance (with directional bias)
            let dx = center_x - current_center_x;
            let dy = center_y - current_center_y;

            let score = match direction {
                FocusDirection::Up | FocusDirection::Down => {
                    // Prefer elements that are more vertically aligned
                    dy.abs() + dx.abs() * 0.5
                }
                FocusDirection::Left | FocusDirection::Right => {
                    // Prefer elements that are more horizontally aligned
                    dx.abs() + dy.abs() * 0.5
                }
                _ => dx.abs() + dy.abs(),
            };

            if score < best_score {
                best_score = score;
                best_candidate = Some(node_id);
            }
        }

        if let Some(next_node) = best_candidate {
            self.set_focus(Some(next_node));
            Some(next_node)
        } else {
            None
        }
    }

    /// Focus the first focusable element
    pub fn focus_first(&mut self) -> Option<NodeId> {
        if let Some(&first) = self.tab_order.first() {
            self.set_focus(Some(first));
            Some(first)
        } else {
            None
        }
    }

    /// Focus the last focusable element
    pub fn focus_last(&mut self) -> Option<NodeId> {
        if let Some(&last) = self.tab_order.last() {
            self.set_focus(Some(last));
            Some(last)
        } else {
            None
        }
    }

    /// Go back in focus history
    pub fn focus_back(&mut self) -> Option<NodeId> {
        if let Some(prev) = self.history.pop_back() {
            // Check if still focusable
            if self.focusable_nodes.contains_key(&prev) {
                self.current = Some(prev);
                Some(prev)
            } else {
                // Try next in history
                self.focus_back()
            }
        } else {
            None
        }
    }

    /// Set whether a node is enabled for focus
    pub fn set_enabled(&mut self, node_id: NodeId, enabled: bool) {
        if let Some(info) = self.focusable_nodes.get_mut(&node_id) {
            info.enabled = enabled;

            // Clear focus if disabling current node
            if !enabled && self.current == Some(node_id) {
                self.focus_next();
            }
        }
    }

    /// Set whether a node is visible for focus
    pub fn set_visible(&mut self, node_id: NodeId, visible: bool) {
        if let Some(info) = self.focusable_nodes.get_mut(&node_id) {
            info.visible = visible;

            // Clear focus if hiding current node
            if !visible && self.current == Some(node_id) {
                self.focus_next();
            }
        }
    }

    fn add_to_history(&mut self, node_id: NodeId) {
        // Remove if already in history
        self.history.retain(|&id| id != node_id);

        // Add to back
        self.history.push_back(node_id);

        // Trim if exceeds limit
        while self.history.len() > self.history_limit {
            self.history.pop_front();
        }
    }

    fn rebuild_tab_order(&mut self) {
        let mut entries: Vec<(NodeId, i32)> = Vec::new();

        for (&node_id, info) in &self.focusable_nodes {
            if info.enabled && info.visible {
                let tab_index = info.tab_index.unwrap_or(0);
                entries.push((node_id, tab_index));
            }
        }

        // Sort by tab index (negative values come last)
        entries.sort_by_key(|&(_, index)| if index < 0 { i32::MAX } else { index });

        self.tab_order = entries.into_iter().map(|(id, _)| id).collect();
    }

    /// Get all focusable nodes in tab order
    pub fn get_focusable_nodes(&self) -> Vec<NodeId> {
        self.tab_order.clone()
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_navigation() {
        let mut manager = FocusManager::new();

        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();

        manager.register_focusable(node1, Some(1), true);
        manager.register_focusable(node2, Some(2), true);
        manager.register_focusable(node3, Some(3), true);

        // Focus first
        assert_eq!(manager.focus_first(), Some(node1));
        assert_eq!(manager.get_focus(), Some(node1));

        // Tab forward
        assert_eq!(manager.focus_next(), Some(node2));
        assert_eq!(manager.get_focus(), Some(node2));

        assert_eq!(manager.focus_next(), Some(node3));
        assert_eq!(manager.get_focus(), Some(node3));

        // Wrap around
        assert_eq!(manager.focus_next(), Some(node1));
        assert_eq!(manager.get_focus(), Some(node1));

        // Tab backward
        assert_eq!(manager.focus_previous(), Some(node3));
        assert_eq!(manager.get_focus(), Some(node3));
    }

    #[test]
    fn test_spatial_navigation() {
        let mut manager = FocusManager::new();

        let top = NodeId::new();
        let middle = NodeId::new();
        let bottom = NodeId::new();

        manager.register_focusable(top, Some(1), true);
        manager.register_focusable(middle, Some(2), true);
        manager.register_focusable(bottom, Some(3), true);

        manager.update_spatial(top, 10.0, 10.0, 20.0, 10.0);
        manager.update_spatial(middle, 10.0, 30.0, 20.0, 10.0);
        manager.update_spatial(bottom, 10.0, 50.0, 20.0, 10.0);

        // Start at middle
        manager.set_focus(Some(middle));

        // Move up
        assert_eq!(manager.focus_spatial(FocusDirection::Up), Some(top));
        assert_eq!(manager.get_focus(), Some(top));

        // Move down twice
        manager.set_focus(Some(middle));
        assert_eq!(manager.focus_spatial(FocusDirection::Down), Some(bottom));
        assert_eq!(manager.get_focus(), Some(bottom));
    }

    #[test]
    fn test_focus_history() {
        let mut manager = FocusManager::new();

        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();

        manager.register_focusable(node1, None, true);
        manager.register_focusable(node2, None, true);
        manager.register_focusable(node3, None, true);

        // Build history
        manager.set_focus(Some(node1));
        manager.set_focus(Some(node2));
        manager.set_focus(Some(node3));

        // Go back
        assert_eq!(manager.focus_back(), Some(node2));
        assert_eq!(manager.focus_back(), Some(node1));
    }
}
