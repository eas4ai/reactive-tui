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
    /// Render order, or insertion order for imperative registrations.
    document_order: Vec<NodeId>,

    /// Spatial navigation map (for arrow keys)
    spatial_map: HashMap<NodeId, SpatialInfo>,

    /// Focus history for back navigation
    history: VecDeque<NodeId>,
    history_limit: usize,

    /// Nodes that can receive focus
    focusable_nodes: HashMap<NodeId, FocusableInfo>,

    /// Focus trap containers (for modals, dialogs)
    focus_traps: HashMap<NodeId, FocusTrap>,
    trap_stack: Vec<NodeId>,
}

#[derive(Clone, Debug)]
struct SpatialInfo {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Debug, PartialEq)]
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
    /// Focus to restore when trap is deactivated
    restore_focus: Option<NodeId>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            current: None,
            tab_order: Vec::new(),
            document_order: Vec::new(),
            spatial_map: HashMap::new(),
            history: VecDeque::with_capacity(10),
            history_limit: 10,
            focusable_nodes: HashMap::new(),
            focus_traps: HashMap::new(),
            trap_stack: Vec::new(),
        }
    }

    /// Register a node as focusable
    /// Returns true if successful, false if the node ID is invalid
    pub fn register_focusable(
        &mut self,
        node_id: NodeId,
        tab_index: Option<i32>,
        enabled: bool,
    ) -> bool {
        // Validate tab index range
        if let Some(index) = tab_index {
            if !(-1000..=1000).contains(&index) {
                #[cfg(feature = "debug")]
                log::warn!(
                    "Warning: Tab index {} is outside recommended range [-1000, 1000]",
                    index
                );
            }
        }

        let info = FocusableInfo {
            tab_index,
            enabled,
            visible: true,
        };

        if self.focusable_nodes.get(&node_id) == Some(&info) {
            return true;
        }
        if !self.focusable_nodes.contains_key(&node_id) {
            self.document_order.push(node_id);
        }
        self.focusable_nodes.insert(node_id, info);
        self.rebuild_tab_order_with_traps();

        true
    }

    /// Unregister a focusable node
    pub fn unregister_focusable(&mut self, node_id: NodeId) {
        self.focusable_nodes.remove(&node_id);
        self.spatial_map.remove(&node_id);
        self.tab_order.retain(|&id| id != node_id);
        self.document_order.retain(|&id| id != node_id);
        for trap in self.focus_traps.values_mut() {
            trap.trapped_nodes.retain(|&id| id != node_id);
        }

        // Clear focus if this node was focused
        if self.current == Some(node_id) {
            self.current = None;
        }

        // Remove from history
        self.history.retain(|&id| id != node_id);
    }

    /// Update spatial information for a node
    /// Returns true if successful, false if coordinates are invalid
    pub fn update_spatial(
        &mut self,
        node_id: NodeId,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> bool {
        // Validate spatial coordinates
        if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
            #[cfg(feature = "debug")]
            log::warn!(
                "Invalid spatial coordinates for node {:?}: x={}, y={}, width={}, height={}",
                node_id,
                x,
                y,
                width,
                height
            );
            return false;
        }

        if width < 0.0 || height < 0.0 {
            #[cfg(feature = "debug")]
            log::warn!(
                "Warning: Negative dimensions for node {:?}: width={}, height={}",
                node_id,
                width,
                height
            );
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
            if !self.can_focus(id) {
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
                events.push((
                    old_id,
                    FocusEvent {
                        kind: FocusEventKind::Lost,
                        timestamp: Instant::now(),
                    },
                ));
            }

            // Emit Gained event for new element (if any)
            if let Some(new_id) = node_id {
                events.push((
                    new_id,
                    FocusEvent {
                        kind: FocusEventKind::Gained,
                        timestamp: Instant::now(),
                    },
                ));
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

    pub(crate) fn current_ref(&self) -> Option<&NodeId> {
        self.current.as_ref()
    }

    pub(crate) fn can_focus(&self, id: NodeId) -> bool {
        self.focusable_nodes
            .get(&id)
            .is_some_and(|info| info.enabled && info.visible)
            && self
                .get_active_trap()
                .is_none_or(|trap| trap.trapped_nodes.contains(&id))
    }

    pub(crate) fn set_document_order(&mut self, nodes: &[NodeId]) {
        let mut ordered: Vec<_> = nodes
            .iter()
            .copied()
            .filter(|id| self.focusable_nodes.contains_key(id))
            .collect();
        let present: std::collections::HashSet<_> = ordered.iter().copied().collect();
        ordered.extend(
            self.document_order
                .iter()
                .copied()
                .filter(|id| !present.contains(id)),
        );
        self.document_order = ordered;
        self.rebuild_tab_order_with_traps();
    }

    /// Move focus in a direction
    /// Returns the focused node ID and optionally a focus event
    pub fn move_focus(
        &mut self,
        direction: FocusDirection,
    ) -> (Option<NodeId>, Option<FocusEvent>) {
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

        let search_nodes: Vec<_> = self
            .document_order
            .iter()
            .copied()
            .filter(|id| self.can_focus(*id))
            .collect();

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
        while let Some(previous) = self.history.pop_back() {
            if self.can_focus(previous) {
                self.current = Some(previous);
                return Some(previous);
            }
        }
        None
    }

    /// Set whether a node is enabled for focus
    pub fn set_enabled(&mut self, node_id: NodeId, enabled: bool) {
        if let Some(info) = self.focusable_nodes.get_mut(&node_id) {
            info.enabled = enabled;
        }
        self.refresh_availability();
    }

    /// Set whether a node is visible for focus.
    pub fn set_visible(&mut self, node_id: NodeId, visible: bool) {
        if let Some(info) = self.focusable_nodes.get_mut(&node_id) {
            info.visible = visible;
        }
        self.refresh_availability();
    }

    fn refresh_availability(&mut self) {
        self.rebuild_tab_order_with_traps();
        if self.current.is_some_and(|id| !self.can_focus(id)) {
            self.set_focus(None);
            self.focus_first();
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

    /// Positive indices first, then default indices in stable document order.
    fn rebuild_tab_order_with_traps(&mut self) {
        let mut entries: Vec<_> = self
            .document_order
            .iter()
            .copied()
            .filter(|id| {
                self.can_focus(*id) && self.focusable_nodes[id].tab_index.unwrap_or(0) >= 0
            })
            .collect();
        entries.sort_by_key(|id| {
            let index = self.focusable_nodes[id].tab_index.unwrap_or(0);
            (index == 0, index)
        });
        self.tab_order = entries;
    }

    /// Get all focusable nodes in tab order
    pub fn get_focusable_nodes(&self) -> Vec<NodeId> {
        self.tab_order.clone()
    }

    /// Create or explicitly reactivate a focus trap for this container.
    pub fn create_focus_trap(&mut self, container: NodeId, trapped_nodes: Vec<NodeId>) -> bool {
        if !trapped_nodes.iter().any(|id| {
            self.focusable_nodes
                .get(id)
                .is_some_and(|info| info.enabled && info.visible)
        }) {
            return false;
        }
        // An imperative call activates the requested container. Declarative
        // frame reconciliation below preserves the opening order instead.
        self.focus_traps.remove(&container);
        self.trap_stack.retain(|id| *id != container);
        self.upsert_trap(container, trapped_nodes, true, false, None)
    }

    pub(crate) fn set_declarative_trap(
        &mut self,
        container: NodeId,
        nodes: Vec<NodeId>,
        restore: bool,
        preferred: Option<NodeId>,
    ) {
        self.upsert_trap(container, nodes, restore, true, preferred);
    }

    fn upsert_trap(
        &mut self,
        container: NodeId,
        nodes: Vec<NodeId>,
        restore: bool,
        allow_empty: bool,
        preferred: Option<NodeId>,
    ) -> bool {
        let mut seen = std::collections::HashSet::new();
        let valid: Vec<_> = nodes
            .into_iter()
            .filter(|id| {
                seen.insert(*id)
                    && self
                        .focusable_nodes
                        .get(id)
                        .is_some_and(|info| info.enabled && info.visible)
            })
            .collect();
        if valid.is_empty() && !allow_empty {
            return false;
        }
        let new_trap = !self.focus_traps.contains_key(&container);
        if let Some(trap) = self.focus_traps.get_mut(&container) {
            trap.trapped_nodes = valid;
            if !restore {
                trap.restore_focus = None;
            }
        } else {
            self.focus_traps.insert(
                container,
                FocusTrap {
                    trapped_nodes: valid,
                    restore_focus: self.current.filter(|_| restore),
                },
            );
            self.trap_stack.push(container);
        }
        self.rebuild_tab_order_with_traps();
        if (new_trap && preferred.is_some()) || self.current.is_none_or(|id| !self.can_focus(id)) {
            let target = preferred
                .filter(|id| self.can_focus(*id))
                .or_else(|| self.tab_order.first().copied());
            self.set_focus(target);
        }
        true
    }

    /// Remove a trap. Closing the active trap reactivates the preceding one.
    pub fn remove_focus_trap(&mut self, container: NodeId) -> bool {
        let active = self.get_active_focus_trap() == Some(container);
        let Some(trap) = self.focus_traps.remove(&container) else {
            return false;
        };
        self.trap_stack.retain(|id| *id != container);
        self.rebuild_tab_order_with_traps();
        if active {
            if let Some(id) = trap.restore_focus.filter(|id| self.can_focus(*id)) {
                self.set_focus(Some(id));
            } else if self.current.is_none_or(|id| !self.can_focus(id)) {
                self.set_focus(None);
                self.focus_first();
            }
        }
        true
    }

    /// Check if focus is currently trapped.
    pub fn is_focus_trapped(&self) -> bool {
        !self.trap_stack.is_empty()
    }

    pub(crate) fn remove_traps_for_nodes(&mut self, removed: &std::collections::HashSet<NodeId>) {
        let traps: Vec<_> = self
            .trap_stack
            .iter()
            .rev()
            .copied()
            .filter(|id| removed.contains(id))
            .collect();
        for id in traps {
            self.remove_focus_trap(id);
        }
    }

    /// Get the most recently opened live trap.
    pub fn get_active_focus_trap(&self) -> Option<NodeId> {
        self.trap_stack.last().copied()
    }

    fn get_active_trap(&self) -> Option<&FocusTrap> {
        self.trap_stack
            .last()
            .and_then(|id| self.focus_traps.get(id))
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
