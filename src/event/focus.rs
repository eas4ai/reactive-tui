use super::router::NodeId;
use super::types::{FocusEvent, FocusEventKind};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

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

    /// Focus trap containers (for modals, dialogs)
    focus_traps: HashMap<NodeId, FocusTrap>,
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

/// Focus trap for containing focus within a specific container
#[derive(Clone, Debug)]
struct FocusTrap {
    /// Nodes that are focusable within this trap
    trapped_nodes: Vec<NodeId>,
    /// Whether this trap is currently active
    active: bool,
    /// Focus to restore when trap is deactivated
    restore_focus: Option<NodeId>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            current: None,
            tab_order: Vec::new(),
            spatial_map: HashMap::new(),
            history: VecDeque::with_capacity(10),
            history_limit: 10,
            focusable_nodes: HashMap::new(),
            focus_traps: HashMap::new(),
        }
    }

    /// Register a node as focusable
    /// Returns true if successful, false if the node ID is invalid
    pub fn register_focusable(&mut self, node_id: NodeId, tab_index: Option<i32>, enabled: bool) -> bool {
        // Validate tab index range
        if let Some(index) = tab_index {
            if index < -1000 || index > 1000 {
                #[cfg(feature = "debug")]
                eprintln!("Warning: Tab index {} is outside recommended range [-1000, 1000]", index);
            }
        }

        let info = FocusableInfo {
            tab_index,
            enabled,
            visible: true,
        };

        // Check if this is an update to existing node
        let is_update = self.focusable_nodes.contains_key(&node_id);

        self.focusable_nodes.insert(node_id, info);

        if is_update {
            // For updates, do a full rebuild to maintain correct order
            self.rebuild_tab_order_optimized();
        } else {
            // For new nodes, try incremental insert
            self.insert_node_in_tab_order(node_id, tab_index, enabled);
        }

        true
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
    /// Returns true if successful, false if coordinates are invalid
    pub fn update_spatial(&mut self, node_id: NodeId, x: f32, y: f32, width: f32, height: f32) -> bool {
        // Validate spatial coordinates
        if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
            #[cfg(feature = "debug")]
            eprintln!("Warning: Invalid spatial coordinates for node {:?}: x={}, y={}, width={}, height={}",
                     node_id, x, y, width, height);
            return false;
        }

        if width < 0.0 || height < 0.0 {
            #[cfg(feature = "debug")]
            eprintln!("Warning: Negative dimensions for node {:?}: width={}, height={}",
                     node_id, width, height);
            return false;
        }

        self.spatial_map.insert(
            node_id,
            SpatialInfo {
                x,
                y,
                width,
                height,
            },
        );

        true
    }

    /// Set focus to a specific node
    /// Returns focus events if focus actually changed (Lost event for old element, Gained for new)
    pub fn set_focus(&mut self, node_id: Option<NodeId>) -> Vec<(NodeId, FocusEvent)> {
        if let Some(id) = node_id {
            // Check if node is focusable and enabled
            if let Some(info) = self.focusable_nodes.get(&id) {
                if !info.enabled || !info.visible {
                    return Vec::new();
                }
            } else {
                return Vec::new();
            }

            // Add previous focus to history
            if let Some(prev) = self.current {
                if prev != id {
                    self.add_to_history(prev);
                }
            }
        }

        // Check if focus actually changed
        if self.current != node_id {
            let old_focus = self.current;
            self.current = node_id;

            let mut events = Vec::new();

            // Emit Lost event for old element (if any)
            if let Some(old_id) = old_focus {
                events.push((old_id, FocusEvent {
                    kind: FocusEventKind::Lost,
                    timestamp: Instant::now(),
                }));
            }

            // Emit Gained event for new element (if any)
            if let Some(new_id) = node_id {
                events.push((new_id, FocusEvent {
                    kind: FocusEventKind::Gained,
                    timestamp: Instant::now(),
                }));
            }

            events
        } else {
            Vec::new() // No change in focus
        }
    }

    /// Get the currently focused node
    pub fn get_focus(&self) -> Option<NodeId> {
        self.current
    }

    /// Move focus in a direction
    /// Returns the focused node ID and optionally a focus event
    pub fn move_focus(&mut self, direction: FocusDirection) -> (Option<NodeId>, Option<FocusEvent>) {
        match direction {
            FocusDirection::Next => self.focus_next(),
            FocusDirection::Previous => self.focus_previous(),
            FocusDirection::Up => {
                let node_id = self.focus_spatial(direction);
                let event = if node_id.is_some() {
                    Some(FocusEvent {
                        kind: FocusEventKind::Up,
                        timestamp: Instant::now(),
                    })
                } else {
                    None
                };
                (node_id, event)
            }
            FocusDirection::Down => {
                let node_id = self.focus_spatial(direction);
                let event = if node_id.is_some() {
                    Some(FocusEvent {
                        kind: FocusEventKind::Down,
                        timestamp: Instant::now(),
                    })
                } else {
                    None
                };
                (node_id, event)
            }
            FocusDirection::Left => {
                let node_id = self.focus_spatial(direction);
                let event = if node_id.is_some() {
                    Some(FocusEvent {
                        kind: FocusEventKind::Left,
                        timestamp: Instant::now(),
                    })
                } else {
                    None
                };
                (node_id, event)
            }
            FocusDirection::Right => {
                let node_id = self.focus_spatial(direction);
                let event = if node_id.is_some() {
                    Some(FocusEvent {
                        kind: FocusEventKind::Right,
                        timestamp: Instant::now(),
                    })
                } else {
                    None
                };
                (node_id, event)
            }
        }
    }

    /// Focus the next element in tab order
    /// Returns the focused node ID and optionally a focus event
    pub fn focus_next(&mut self) -> (Option<NodeId>, Option<FocusEvent>) {
        if self.tab_order.is_empty() {
            return (None, None);
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
        let focus_events = self.set_focus(Some(next_node));

        // Create a Next event if focus changed
        let event = if !focus_events.is_empty() {
            Some(FocusEvent {
                kind: FocusEventKind::Next,
                timestamp: Instant::now(),
            })
        } else {
            None
        };

        (Some(next_node), event)
    }

    /// Focus the previous element in tab order
    /// Returns the focused node ID and optionally a focus event
    pub fn focus_previous(&mut self) -> (Option<NodeId>, Option<FocusEvent>) {
        if self.tab_order.is_empty() {
            return (None, None);
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
        let focus_events = self.set_focus(Some(prev_node));

        // Create a Previous event if focus changed
        let event = if !focus_events.is_empty() {
            Some(FocusEvent {
                kind: FocusEventKind::Previous,
                timestamp: Instant::now(),
            })
        } else {
            None
        };

        (Some(prev_node), event)
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

        // Get nodes to search - respect focus traps
        let search_nodes: Vec<NodeId> = if let Some(active_trap) = self.get_active_trap() {
            // If there's an active trap, only search within trapped nodes that have spatial info
            active_trap.trapped_nodes.iter()
                .filter(|&&node_id| self.spatial_map.contains_key(&node_id))
                .copied()
                .collect()
        } else {
            // No active trap, search all spatial nodes
            self.spatial_map.keys().copied().collect()
        };

        for node_id in search_nodes {
            if node_id == current {
                continue;
            }

            // Get spatial info for this node
            let spatial = match self.spatial_map.get(&node_id) {
                Some(spatial) => spatial,
                None => continue, // Node has no spatial info
            };

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
                // Set focus directly without adding to history
                let old_focus = self.current;
                self.current = Some(prev);
                
                // Emit focus events
                if old_focus != Some(prev) {
                    // Could emit events here if needed, but for now just return the node
                }
                
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

    /// Rebuild tab order considering active focus traps
    fn rebuild_tab_order_with_traps(&mut self) {
        // Check if there are any active focus traps
        let active_trap = self.focus_traps.values().find(|trap| trap.active);

        if let Some(trap) = active_trap {
            // If there's an active trap, only include trapped nodes in tab order
            let mut trapped_entries: Vec<(NodeId, i32)> = Vec::new();

            for &node_id in &trap.trapped_nodes {
                if let Some(info) = self.focusable_nodes.get(&node_id) {
                    if info.enabled && info.visible {
                        let tab_index = info.tab_index.unwrap_or(0);
                        trapped_entries.push((node_id, tab_index));
                    }
                }
            }

            // Sort by tab index (negative values come last)
            trapped_entries.sort_by_key(|&(_, index)| if index < 0 { i32::MAX } else { index });

            self.tab_order = trapped_entries.into_iter().map(|(id, _)| id).collect();
        } else {
            // No active traps, use normal tab order
            self.rebuild_tab_order_optimized();
        }
    }

    /// Optimized tab order rebuild with better performance for large numbers of nodes
    fn rebuild_tab_order_optimized(&mut self) {
        // Pre-allocate with known capacity to avoid reallocations
        let mut entries = Vec::with_capacity(self.focusable_nodes.len());

        // Single pass through focusable nodes to collect enabled/visible entries
        for (&node_id, info) in &self.focusable_nodes {
            if info.enabled && info.visible {
                let tab_index = info.tab_index.unwrap_or(0);
                entries.push((node_id, tab_index));
            }
        }

        // Use unstable sort for better performance (order of equal elements doesn't matter)
        entries.sort_unstable_by_key(|&(_, index)| if index < 0 { i32::MAX } else { index });

        // Pre-allocate tab_order with exact capacity and collect in one pass
        self.tab_order.clear();
        self.tab_order.reserve_exact(entries.len());
        self.tab_order.extend(entries.into_iter().map(|(id, _)| id));
    }

    /// Insert a single node into the tab order at the correct position
    fn insert_node_in_tab_order(&mut self, node_id: NodeId, tab_index: Option<i32>, enabled: bool) {
        if !enabled {
            return; // Don't add disabled nodes to tab order
        }

        let target_index = tab_index.unwrap_or(0);

        // Use the same sorting logic as rebuild_tab_order for consistency
        let sort_key = |index: i32| if index < 0 { i32::MAX } else { index };
        let target_sort_key = sort_key(target_index);

        // Find the correct insertion position using binary search
        let insert_pos = self.tab_order
            .binary_search_by(|&existing_id| {
                let existing_info = self.focusable_nodes.get(&existing_id).unwrap();
                let existing_index = existing_info.tab_index.unwrap_or(0);
                let existing_sort_key = sort_key(existing_index);

                // Compare sort keys (with negative indices mapped to i32::MAX), then node IDs for stability
                existing_sort_key.cmp(&target_sort_key)
                    .then(existing_id.cmp(&node_id))
            })
            .unwrap_or_else(|pos| pos);

        self.tab_order.insert(insert_pos, node_id);
    }

    /// Get all focusable nodes in tab order
    pub fn get_focusable_nodes(&self) -> Vec<NodeId> {
        self.tab_order.clone()
    }

    /// Create a focus trap for a container (e.g., modal dialog)
    /// This restricts focus navigation to only nodes within the container
    pub fn create_focus_trap(&mut self, container: NodeId, trapped_nodes: Vec<NodeId>) -> bool {
        // Validate that trapped nodes are actually focusable
        let valid_trapped_nodes: Vec<NodeId> = trapped_nodes
            .into_iter()
            .filter(|&node_id| {
                self.focusable_nodes
                    .get(&node_id)
                    .map(|info| info.enabled && info.visible)
                    .unwrap_or(false)
            })
            .collect();

        if valid_trapped_nodes.is_empty() {
            return false; // Cannot create trap with no focusable nodes
        }

        // Deactivate any existing traps (only one trap can be active at a time)
        for trap in self.focus_traps.values_mut() {
            trap.active = false;
        }

        // Store current focus to restore later
        let restore_focus = self.current;

        let trap = FocusTrap {
            trapped_nodes: valid_trapped_nodes.clone(),
            active: true,
            restore_focus,
        };

        self.focus_traps.insert(container, trap);

        // Update tab order to only include trapped nodes
        self.rebuild_tab_order_with_traps();

        // Focus first element in trap if no current focus or current focus is outside trap
        if let Some(current) = self.current {
            if !valid_trapped_nodes.contains(&current) {
                if let Some(&first_trapped) = valid_trapped_nodes.first() {
                    self.set_focus(Some(first_trapped));
                }
            }
        } else if let Some(&first_trapped) = valid_trapped_nodes.first() {
            self.set_focus(Some(first_trapped));
        }

        true
    }

    /// Remove a focus trap and restore previous focus behavior
    pub fn remove_focus_trap(&mut self, container: NodeId) -> bool {
        if let Some(trap) = self.focus_traps.remove(&container) {
            // Restore previous focus if it was saved
            if let Some(restore_focus) = trap.restore_focus {
                self.set_focus(Some(restore_focus));
            }

            // Rebuild tab order without the trap
            self.rebuild_tab_order_with_traps();
            true
        } else {
            false
        }
    }

    /// Check if focus is currently trapped
    pub fn is_focus_trapped(&self) -> bool {
        self.focus_traps.values().any(|trap| trap.active)
    }

    /// Get the active focus trap container, if any
    pub fn get_active_focus_trap(&self) -> Option<NodeId> {
        self.focus_traps
            .iter()
            .find(|(_, trap)| trap.active)
            .map(|(&container, _)| container)
    }

    /// Get the active focus trap, if any
    fn get_active_trap(&self) -> Option<&FocusTrap> {
        self.focus_traps.values().find(|trap| trap.active)
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
        let (focused_node, focus_event) = manager.focus_next();
        assert_eq!(focused_node, Some(node2));
        assert_eq!(manager.get_focus(), Some(node2));
        assert!(focus_event.is_some()); // Should emit focus event

        let (focused_node, focus_event) = manager.focus_next();
        assert_eq!(focused_node, Some(node3));
        assert_eq!(manager.get_focus(), Some(node3));
        assert!(focus_event.is_some()); // Should emit focus event

        // Wrap around
        let (focused_node, focus_event) = manager.focus_next();
        assert_eq!(focused_node, Some(node1));
        assert_eq!(manager.get_focus(), Some(node1));
        assert!(focus_event.is_some()); // Should emit focus event

        // Tab backward
        let (focused_node, focus_event) = manager.focus_previous();
        assert_eq!(focused_node, Some(node3));
        assert_eq!(manager.get_focus(), Some(node3));
        assert!(focus_event.is_some()); // Should emit focus event
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

    #[test]
    fn test_focus_events() {
        let mut manager = FocusManager::new();

        let node1 = NodeId::new();
        let node2 = NodeId::new();

        manager.register_focusable(node1, Some(1), true);
        manager.register_focusable(node2, Some(2), true);

        // Initial focus should emit Gained event
        let focus_events = manager.set_focus(Some(node1));
        assert_eq!(focus_events.len(), 1);
        assert_eq!(focus_events[0].0, node1);
        assert_eq!(focus_events[0].1.kind, FocusEventKind::Gained);

        // Moving focus should emit Lost event for old node and Gained for new node
        let focus_events = manager.set_focus(Some(node2));
        assert_eq!(focus_events.len(), 2);
        assert_eq!(focus_events[0].0, node1); // Lost event for old node
        assert_eq!(focus_events[0].1.kind, FocusEventKind::Lost);
        assert_eq!(focus_events[1].0, node2); // Gained event for new node
        assert_eq!(focus_events[1].1.kind, FocusEventKind::Gained);

        // Removing focus should emit Lost event
        let focus_events = manager.set_focus(None);
        assert_eq!(focus_events.len(), 1);
        assert_eq!(focus_events[0].0, node2);
        assert_eq!(focus_events[0].1.kind, FocusEventKind::Lost);

        // Setting same focus should not emit event
        manager.set_focus(Some(node1));
        let focus_events = manager.set_focus(Some(node1));
        assert!(focus_events.is_empty());

        // Tab navigation should emit Next/Previous events
        let (_, focus_event) = manager.focus_next();
        assert!(focus_event.is_some());
        if let Some(event) = focus_event {
            assert_eq!(event.kind, FocusEventKind::Next);
        }

        let (_, focus_event) = manager.focus_previous();
        assert!(focus_event.is_some());
        if let Some(event) = focus_event {
            assert_eq!(event.kind, FocusEventKind::Previous);
        }
    }

    #[test]
    fn test_focus_traps() {
        let mut manager = FocusManager::new();

        // Create nodes: some for general use, some for modal
        let general1 = NodeId::new();
        let general2 = NodeId::new();
        let modal_container = NodeId::new();
        let modal_button1 = NodeId::new();
        let modal_button2 = NodeId::new();

        // Register all nodes as focusable
        assert!(manager.register_focusable(general1, Some(1), true));
        assert!(manager.register_focusable(general2, Some(2), true));
        assert!(manager.register_focusable(modal_container, Some(3), true));
        assert!(manager.register_focusable(modal_button1, Some(4), true));
        assert!(manager.register_focusable(modal_button2, Some(5), true));

        // Initially, all nodes should be in tab order
        assert_eq!(manager.get_focusable_nodes().len(), 5);

        // Focus on a general element
        manager.set_focus(Some(general1));
        assert_eq!(manager.get_focus(), Some(general1));

        // Create focus trap for modal
        let trapped_nodes = vec![modal_button1, modal_button2];
        assert!(manager.create_focus_trap(modal_container, trapped_nodes.clone()));

        // Focus should be trapped - only modal buttons in tab order
        assert_eq!(manager.get_focusable_nodes().len(), 2);
        assert!(manager.is_focus_trapped());
        assert_eq!(manager.get_active_focus_trap(), Some(modal_container));

        // Focus should have moved to first trapped element
        assert_eq!(manager.get_focus(), Some(modal_button1));

        // Tab navigation should only cycle through trapped nodes
        let (focused_node, _) = manager.focus_next();
        assert_eq!(focused_node, Some(modal_button2));

        let (focused_node, _) = manager.focus_next();
        assert_eq!(focused_node, Some(modal_button1)); // Wrap around

        // Remove focus trap
        assert!(manager.remove_focus_trap(modal_container));

        // Focus should be restored and all nodes available again
        assert!(!manager.is_focus_trapped());
        assert_eq!(manager.get_active_focus_trap(), None);
        assert_eq!(manager.get_focusable_nodes().len(), 5);

        // Focus should have been restored to general1
        assert_eq!(manager.get_focus(), Some(general1));
    }

    #[test]
    fn test_focus_edge_cases() {
        let mut manager = FocusManager::new();

        // Test focus on empty manager
        let (node_id, event) = manager.focus_next();
        assert_eq!(node_id, None);
        assert_eq!(event, None);

        let (node_id, event) = manager.focus_previous();
        assert_eq!(node_id, None);
        assert_eq!(event, None);

        // Test focus on disabled/invisible nodes
        let node1 = NodeId::new();
        let node2 = NodeId::new();

        manager.register_focusable(node1, Some(1), false); // disabled
        manager.register_focusable(node2, Some(2), true);

        // Should skip disabled node
        let (focused_node, _) = manager.focus_next();
        assert_eq!(focused_node, Some(node2));

        // Test setting focus to disabled node
        let focus_events = manager.set_focus(Some(node1));
        assert!(focus_events.is_empty()); // Should fail
        assert_eq!(manager.get_focus(), Some(node2)); // Should remain on node2

        // Test unregistering focused node
        manager.unregister_focusable(node2);
        assert_eq!(manager.get_focus(), None); // Should clear focus

        // Test spatial navigation with no spatial info
        let node3 = NodeId::new();
        manager.register_focusable(node3, Some(3), true);
        manager.set_focus(Some(node3));

        let spatial_result = manager.focus_spatial(FocusDirection::Up);
        assert_eq!(spatial_result, None); // No spatial info available
    }

    #[test]
    fn test_focus_system_integration() {
        let mut manager = FocusManager::new();

        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();

        // Test complete workflow
        manager.register_focusable(node1, Some(1), true);
        manager.register_focusable(node2, Some(2), true);
        manager.register_focusable(node3, Some(3), true);

        // Test initial focus
        let focus_events = manager.set_focus(Some(node1));
        assert!(!focus_events.is_empty());
        assert_eq!(manager.get_focus(), Some(node1));

        // Test tab navigation with events
        let (focused_node, focus_event) = manager.focus_next();
        assert_eq!(focused_node, Some(node2));
        assert!(focus_event.is_some());
        if let Some(event) = focus_event {
            assert_eq!(event.kind, FocusEventKind::Next);
        }

        // Test directional movement
        let (focused_node, _) = manager.move_focus(FocusDirection::Previous);
        assert_eq!(focused_node, Some(node1));

        // Test focus history
        manager.set_focus(Some(node2));
        manager.set_focus(Some(node3));
        assert_eq!(manager.focus_back(), Some(node2));

        // Test spatial navigation setup
        manager.update_spatial(node1, 0.0, 0.0, 10.0, 10.0);
        manager.update_spatial(node2, 0.0, 20.0, 10.0, 10.0);
        manager.set_focus(Some(node1));

        let spatial_result = manager.focus_spatial(FocusDirection::Down);
        assert_eq!(spatial_result, Some(node2));
    }
}
