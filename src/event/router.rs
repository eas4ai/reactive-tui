use super::focus::FocusManager;
use super::hit::HitTest;
use super::types::{Event, EventTrait};
use std::collections::HashMap;
use std::sync::Arc;

/// Event handler function type
pub type EventHandlerFn = Arc<dyn Fn(&Event) -> EventResult + Send + Sync>;

/// Result of handling an event
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventResult {
    /// Event was not handled, continue propagation
    Ignored,
    /// Event was handled, continue propagation
    Handled,
    /// Event was handled, stop propagation
    Consumed,
    /// Event was handled but allow bubbling (capture phase only)
    Captured,
}

/// Phase of event propagation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventPhase {
    /// Capturing phase - top-down from root to target
    Capture,
    /// Target phase - at the target element
    Target,
    /// Bubbling phase - bottom-up from target to root
    Bubble,
}

/// Event handler registration
pub struct EventHandler {
    /// Unique identifier for this handler
    pub id: HandlerId,
    /// Type of event this handler responds to
    pub event_type: String,
    /// Phase of event propagation to handle
    pub phase: EventPhase,
    /// The actual handler function
    pub handler: EventHandlerFn,
    /// Priority for handler execution order (higher = earlier)
    pub priority: i32,
}

/// Unique identifier for event handlers
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HandlerId(usize);

impl HandlerId {
    fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// Node in the event routing tree
pub struct EventNode {
    /// Unique identifier for this node
    pub id: NodeId,
    /// Parent node ID, if any
    pub parent: Option<NodeId>,
    /// Child node IDs
    pub children: Vec<NodeId>,
    /// Event handlers for bubble phase
    pub handlers: HashMap<String, Vec<EventHandler>>,
    /// Event handlers for capture phase
    pub capture_handlers: HashMap<String, Vec<EventHandler>>,
}

/// Unique identifier for nodes in the event routing tree
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub(super) usize);

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeId {
    pub(crate) fn serial(self) -> usize {
        self.0
    }

    /// Create a new unique node ID
    pub fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// Element index plus one per cell of the presented frame and the node each
/// element maps to (PNT-002).
struct CellHits {
    cells: Vec<u32>,
    width: usize,
    nodes: Vec<Option<NodeId>>,
}

/// Event router managing event propagation through component tree
pub struct EventRouter {
    nodes: HashMap<NodeId, EventNode>,
    root: Option<NodeId>,
    focus_manager: FocusManager,
    hit_test: HitTest,
    /// Per-cell hit ids of the presented frame, preferred over the bounds tree.
    cell_hits: Option<CellHits>,
    pointer: Option<super::hit::Point>,
    hover_path: Vec<NodeId>,
    /// Path cache for event routing optimization
    path_cache: Option<super::cache::PathCache>,
}

impl EventRouter {
    /// Create a new event router with default size
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            focus_manager: FocusManager::new(),
            hit_test: HitTest::new(80.0, 24.0), // Default terminal size
            cell_hits: None,
            pointer: None,
            hover_path: Vec::new(),
            path_cache: None,
        }
    }

    /// Create a new event router with specified size
    pub fn new_with_size(width: u16, height: u16) -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            focus_manager: FocusManager::new(),
            hit_test: HitTest::new(width as f32, height as f32),
            cell_hits: None,
            pointer: None,
            hover_path: Vec::new(),
            path_cache: None,
        }
    }

    /// Enable path caching for improved performance
    pub fn enable_caching(&mut self) {
        self.path_cache = Some(super::cache::PathCache::new());
    }

    /// Create a new event node
    pub fn create_node(&mut self, parent: Option<NodeId>) -> NodeId {
        let id = NodeId::new();

        let node = EventNode {
            id,
            parent,
            children: Vec::new(),
            handlers: HashMap::new(),
            capture_handlers: HashMap::new(),
        };

        // Add to parent's children
        if let Some(parent_id) = parent {
            if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
                parent_node.children.push(id);
            }
        } else {
            // No parent means this is the root
            self.root = Some(id);
        }

        self.nodes.insert(id, node);
        id
    }

    /// Remove an event node and all its descendants, including owned traps.
    pub fn remove_node(&mut self, id: NodeId) {
        self.remove_nodes(&[id]);
    }

    pub(crate) fn remove_nodes(&mut self, roots: &[NodeId]) {
        let mut pending = roots.to_vec();
        let mut removed = std::collections::HashSet::new();
        while let Some(id) = pending.pop() {
            if removed.insert(id) {
                if let Some(node) = self.nodes.get(&id) {
                    pending.extend(&node.children);
                }
            }
        }
        if self.get_focus().is_some_and(|id| removed.contains(&id)) {
            self.set_focus(None);
        }
        let parents: std::collections::HashSet<_> = removed
            .iter()
            .filter_map(|id| self.nodes.get(id).and_then(|node| node.parent))
            .collect();
        for id in &removed {
            self.hit_test.remove_node(*id);
            self.focus_manager.unregister_focusable(*id);
            self.nodes.remove(id);
        }
        for parent in parents {
            if let Some(node) = self.nodes.get_mut(&parent) {
                node.children.retain(|id| !removed.contains(id));
            }
        }
        if self.root.is_some_and(|id| removed.contains(&id)) {
            self.root = None;
        }
        let previous = self.get_focus();
        self.focus_manager.remove_traps_for_nodes(&removed);
        self.emit_focus_change(previous);
    }

    /// Add an event handler to a node
    pub fn add_handler(
        &mut self,
        node_id: NodeId,
        event_type: impl Into<String>,
        phase: EventPhase,
        handler: EventHandlerFn,
    ) -> HandlerId {
        let handler_id = HandlerId::new();
        let event_type = event_type.into();

        let handler = EventHandler {
            id: handler_id,
            event_type: event_type.clone(),
            phase,
            handler,
            priority: 0,
        };

        if let Some(node) = self.nodes.get_mut(&node_id) {
            match phase {
                EventPhase::Capture => {
                    node.capture_handlers
                        .entry(event_type)
                        .or_insert_with(Vec::new)
                        .push(handler);
                }
                EventPhase::Target | EventPhase::Bubble => {
                    node.handlers
                        .entry(event_type)
                        .or_insert_with(Vec::new)
                        .push(handler);
                }
            }
        }

        handler_id
    }

    /// Remove an event handler
    pub(crate) fn clear_handlers(&mut self, node_id: NodeId) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.handlers.clear();
            node.capture_handlers.clear();
        }
    }

    /// Remove an event handler
    pub fn remove_handler(&mut self, node_id: NodeId, handler_id: HandlerId) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            // Remove from regular handlers
            for handlers in node.handlers.values_mut() {
                handlers.retain(|h| h.id != handler_id);
            }

            // Remove from capture handlers
            for handlers in node.capture_handlers.values_mut() {
                handlers.retain(|h| h.id != handler_id);
            }
        }
    }

    /// Route an event through the tree
    pub fn route_event(&self, event: &Event, target_id: NodeId) -> EventResult {
        // Use discriminant for zero-cost type identification
        let event_type = match event {
            Event::Key(_) => "key",
            Event::Mouse(_) => "mouse",
            Event::Resize(_) => "resize",
            Event::Focus(_) => "focus",
            Event::Paste(_) => "paste",
            Event::Custom(_) => "custom",
        };

        // Build path with pre-allocated capacity and cycle detection
        let mut path = Vec::with_capacity(16); // Most UI trees are < 16 levels deep
        let mut visited = std::collections::HashSet::with_capacity(16);
        let mut current = Some(target_id);

        // Maximum depth to prevent infinite loops even with cycle detection
        const MAX_DEPTH: usize = 1000;
        let mut depth = 0;

        while let Some(node_id) = current {
            // Check for cycles
            if !visited.insert(node_id) {
                // Cycle detected - log error and break
                #[cfg(debug_assertions)]
                log::warn!(
                    "Warning: Cycle detected in event router tree at node {:?}",
                    node_id
                );
                break;
            }

            // Check for excessive depth
            if depth >= MAX_DEPTH {
                #[cfg(debug_assertions)]
                log::warn!(
                    "Warning: Maximum depth {} exceeded in event router",
                    MAX_DEPTH
                );
                break;
            }

            path.push(node_id);
            if let Some(node) = self.nodes.get(&node_id) {
                current = node.parent;
            } else {
                break;
            }
            depth += 1;
        }

        path.reverse(); // Now path goes from root to target

        let mut result = EventResult::Ignored;
        // Capture phase - root to target, including target capture handlers.
        for &node_id in &path {
            if let Some(node) = self.nodes.get(&node_id) {
                if let Some(handlers) = node.capture_handlers.get(event_type) {
                    for handler in handlers {
                        match (handler.handler)(event) {
                            EventResult::Consumed => return EventResult::Consumed,
                            EventResult::Captured => result = EventResult::Handled,
                            EventResult::Handled => result = EventResult::Handled,
                            EventResult::Ignored => {}
                        }
                    }
                }
            }
        }

        // Target phase
        if let Some(node) = self.nodes.get(&target_id) {
            if let Some(handlers) = node.handlers.get(event_type) {
                for handler in handlers {
                    match (handler.handler)(event) {
                        EventResult::Consumed => return EventResult::Consumed,
                        EventResult::Captured => result = EventResult::Handled,
                        EventResult::Handled => result = EventResult::Handled,
                        EventResult::Ignored => {}
                    }
                }
            }
        }

        // Bubble phase - target to root (if event bubbles)
        let bubbles = match event {
            Event::Key(e) => e.bubbles(),
            Event::Mouse(e) => e.bubbles(),
            Event::Resize(e) => e.bubbles(),
            Event::Focus(e) => e.bubbles(),
            Event::Paste(e) => e.bubbles(),
            Event::Custom(e) => e.bubbles(),
        };

        if bubbles {
            for &node_id in path.iter().rev().skip(1) {
                if let Some(node) = self.nodes.get(&node_id) {
                    if let Some(handlers) = node.handlers.get(event_type) {
                        for handler in handlers {
                            if handler.phase == EventPhase::Bubble {
                                match (handler.handler)(event) {
                                    EventResult::Consumed => return EventResult::Consumed,
                                    EventResult::Captured => result = EventResult::Handled,
                                    EventResult::Handled => result = EventResult::Handled,
                                    EventResult::Ignored => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Dispatch an event to the focused node
    pub fn dispatch_to_focus(&self, event: &Event) -> EventResult {
        if let Some(focus_id) = self.focus_manager.get_focus() {
            self.route_event(event, focus_id)
        } else if let Some(root_id) = self.root.filter(|_| !self.is_focus_trapped()) {
            self.route_event(event, root_id)
        } else {
            EventResult::Ignored
        }
    }

    /// Advance focus to next node id
    /// Returns the focused node and collects events for later emission
    pub fn focus_next(&mut self) -> Option<NodeId> {
        let previous = self.get_focus();
        let (node_id, focus_event) = self.focus_manager.focus_next();
        self.emit_focus_change(previous);
        self.emit_focus_operation_events(node_id, focus_event);
        node_id
    }

    /// Move focus to previous node id
    /// Returns the focused node and emits focus event if focus changed
    pub fn focus_prev(&mut self) -> Option<NodeId> {
        let previous = self.get_focus();
        let (node_id, focus_event) = self.focus_manager.focus_previous();
        self.emit_focus_change(previous);
        self.emit_focus_operation_events(node_id, focus_event);
        node_id
    }

    /// Set the focused node
    /// Emits focus events if focus changed
    pub fn set_focus(&mut self, node_id: Option<NodeId>) {
        let focus_events = self.focus_manager.set_focus(node_id);

        // Emit all focus events (Lost for old element, Gained for new element)
        for (target_id, focus_event) in focus_events {
            self.emit_focus_event(target_id, focus_event);
        }
    }

    /// Helper method to emit focus events safely
    fn emit_focus_event(&self, target_id: NodeId, focus_event: super::types::FocusEvent) {
        let event = super::types::Event::Focus(focus_event);
        self.route_event(&event, target_id);
    }

    fn emit_focus_change(&self, previous: Option<NodeId>) {
        use super::types::{FocusEvent, FocusEventKind};
        let current = self.get_focus();
        if previous == current {
            return;
        }
        for (id, kind) in [
            (previous, FocusEventKind::Lost),
            (current, FocusEventKind::Gained),
        ] {
            if let Some(id) = id {
                self.emit_focus_event(
                    id,
                    FocusEvent {
                        kind,
                        timestamp: std::time::Instant::now(),
                    },
                );
            }
        }
    }

    /// Helper method to emit focus events from a focus operation result
    fn emit_focus_operation_events(
        &self,
        node_id: Option<NodeId>,
        focus_event: Option<super::types::FocusEvent>,
    ) {
        if let Some(event) = focus_event {
            if let Some(target_id) = node_id {
                self.emit_focus_event(target_id, event);
            }
        }
    }

    /// Get the currently focused node
    pub fn get_focus(&self) -> Option<NodeId> {
        self.focus_manager.get_focus()
    }

    /// Hover follows the last cell position against acknowledged frame bounds.
    pub(crate) fn hovered_node(&self) -> Option<NodeId> {
        self.pointer.and_then(|point| self.resolve_hit(point))
    }

    /// Per-cell hit ids from the backend for the presented frame, with the
    /// node each element index maps to (None for an inert element); `None`
    /// clears them so hit testing falls back to painted bounds (PNT-002).
    pub(crate) fn set_cell_hits(
        &mut self,
        hits: Option<(&[u32], u16)>,
        nodes: Vec<Option<NodeId>>,
    ) {
        self.cell_hits = hits.map(|(cells, width)| CellHits {
            cells: cells.to_vec(),
            width: usize::from(width),
            nodes,
        });
    }

    /// The node under a point: the per-cell grid when the backend offers one,
    /// the painted bounds tree otherwise or when the cell's element is inert.
    fn resolve_hit(&self, point: super::hit::Point) -> Option<NodeId> {
        if let Some(hits) = &self.cell_hits {
            if point.x >= 0.0 && point.y >= 0.0 {
                let (x, y) = (point.x as usize, point.y as usize);
                if x < hits.width {
                    if let Some(&cell) = hits.cells.get(y * hits.width + x) {
                        if cell == 0 {
                            return None;
                        }
                        if let Some(Some(id)) = hits.nodes.get(cell as usize - 1) {
                            return Some(*id);
                        }
                    }
                }
            }
        }
        self.hit_test.hit_test(point)
    }

    /// Deliver boundary events only to nodes whose own hover membership changed.
    /// Moving between two children must not make their shared parent leave.
    pub(crate) fn refresh_hover(&mut self) -> bool {
        use super::types::{MouseEvent, MouseEventKind, Position};
        let mut path = Vec::new();
        let mut current = self.hovered_node();
        while let Some(id) = current {
            if path.contains(&id) {
                break;
            }
            path.push(id);
            current = self.nodes.get(&id).and_then(|node| node.parent);
        }
        if path == self.hover_path {
            return false;
        }
        let position = self.pointer.map_or(Position::cell(0, 0), |point| {
            Position::cell(point.x as u16, point.y as u16)
        });
        for (from, to, kind) in [
            (&self.hover_path, &path, MouseEventKind::Leave),
            (&path, &self.hover_path, MouseEventKind::Enter),
        ] {
            let event = Event::Mouse(MouseEvent::new(kind, position));
            for id in from.iter().filter(|id| !to.contains(id)) {
                if let Some(node) = self.nodes.get(id) {
                    let capture = node.capture_handlers.get("mouse").into_iter().flatten();
                    let bubble = node.handlers.get("mouse").into_iter().flatten();
                    for handler in capture.chain(bubble) {
                        if (handler.handler)(&event) == EventResult::Consumed {
                            break;
                        }
                    }
                }
            }
        }
        self.hover_path = path;
        true
    }

    pub(crate) fn current_focus_ref(&self) -> Option<&NodeId> {
        self.focus_manager.current_ref()
    }

    pub(crate) fn can_focus(&self, id: NodeId) -> bool {
        self.focus_manager.can_focus(id)
    }

    pub(crate) fn set_focus_order(&mut self, order: &[NodeId]) {
        self.focus_manager.set_document_order(order);
    }

    pub(crate) fn set_declarative_trap(
        &mut self,
        container: NodeId,
        nodes: Vec<NodeId>,
        restore: bool,
        preferred: Option<NodeId>,
    ) {
        let previous = self.get_focus();
        self.focus_manager
            .set_declarative_trap(container, nodes, restore, preferred);
        self.emit_focus_change(previous);
    }

    /// Process an event - THE central event processing method
    pub fn process_event(&mut self, event: &Event) -> EventResult {
        if let Event::Mouse(mouse) = event {
            self.pointer = match (mouse.kind.clone(), mouse.position) {
                (super::types::MouseEventKind::Leave, _) => None,
                (_, super::types::Position::Cell { .. }) => Some(super::hit::Point::new(
                    mouse.position.x() as f32,
                    mouse.position.y() as f32,
                )),
                _ => None,
            };
            self.refresh_hover();
            if matches!(
                mouse.kind,
                super::types::MouseEventKind::Enter | super::types::MouseEventKind::Leave
            ) {
                return EventResult::Handled;
            }
        }
        // 1. Handle system events first (focus traversal, etc.)
        if let Some(result) = self.handle_system_event(event) {
            return result;
        }

        // 2. Determine target node
        let target = self.determine_target(event);

        if matches!(event, Event::Mouse(mouse) if mouse.button == super::types::MouseButton::Left && matches!(mouse.kind, super::types::MouseEventKind::Down | super::types::MouseEventKind::Click))
        {
            let mut candidate = Some(target);
            while let Some(id) = candidate {
                self.set_focus(Some(id));
                if self.get_focus() == Some(id) {
                    break;
                }
                candidate = self.nodes.get(&id).and_then(|node| node.parent);
            }
        }

        // 3. Route the event
        self.route_event(event, target)
    }

    /// Handle system-level events (focus traversal, global shortcuts)
    fn handle_system_event(&mut self, event: &Event) -> Option<EventResult> {
        match event {
            Event::Key(key_event) => {
                use super::types::{KeyCode, KeyEventKind};
                if matches!(key_event.code, KeyCode::Tab | KeyCode::BackTab)
                    && key_event.kind != KeyEventKind::Release
                    && !key_event.modifiers.ctrl
                    && !key_event.modifiers.alt
                    && !key_event.modifiers.meta
                {
                    if key_event.code == KeyCode::Tab && !key_event.modifiers.shift {
                        let result = self.dispatch_to_focus(event);
                        if result != EventResult::Ignored {
                            return Some(result);
                        }
                    }
                    if key_event.modifiers.shift || key_event.code == KeyCode::BackTab {
                        if self.focus_prev().is_some() {
                            return Some(EventResult::Handled);
                        }
                    } else if self.focus_next().is_some() {
                        return Some(EventResult::Handled);
                    }
                    if self.is_focus_trapped() {
                        return Some(EventResult::Handled);
                    }
                    if key_event.code == KeyCode::Tab && !key_event.modifiers.shift {
                        // The focused/root handler already saw this Tab above.
                        return Some(EventResult::Ignored);
                    }
                }
            }
            Event::Resize(resize_event) => {
                self.hit_test
                    .resize(resize_event.width as f32, resize_event.height as f32);
                return Some(EventResult::Handled);
            }
            _ => {}
        }
        None
    }

    /// Determine the target node for an event
    pub(crate) fn determine_target(&self, event: &Event) -> NodeId {
        match event {
            Event::Mouse(mouse_event) => {
                if !matches!(mouse_event.position, super::types::Position::Cell { .. }) {
                    return NodeId::new();
                }
                // Use hit testing for mouse events
                let point = super::hit::Point::new(
                    mouse_event.position.x() as f32,
                    mouse_event.position.y() as f32,
                );

                // A miss must not activate the focused element or the root.
                self.resolve_hit(point).unwrap_or_default()
            }
            _ => {
                // For keyboard and other events, use focused node or root
                self.focus_manager
                    .get_focus()
                    .or_else(|| self.root.filter(|_| !self.is_focus_trapped()))
                    .unwrap_or_default()
            }
        }
    }

    /// Add a node to hit testing (for mouse events)
    pub fn add_hit_target(&mut self, node_id: NodeId, bounds: super::hit::Bounds, z_index: i32) {
        self.hit_test.update_bounds(node_id, bounds, z_index);
    }

    /// Remove a node from hit testing
    pub fn remove_hit_target(&mut self, node_id: NodeId) {
        self.hit_test.remove_node(node_id);
    }

    /// Add a focusable node with optional tab index
    /// Returns true if successful
    pub fn add_focusable(&mut self, node_id: NodeId, tab_index: Option<i32>) -> bool {
        self.focus_manager
            .register_focusable(node_id, tab_index, true)
    }

    /// Remove a focusable node
    pub fn remove_focusable(&mut self, node_id: NodeId) {
        if self.get_focus() == Some(node_id) {
            self.set_focus(None);
        }
        self.focus_manager.unregister_focusable(node_id);
    }

    /// Update spatial information for a node (for arrow-key navigation)
    /// Returns true if successful
    pub fn update_spatial(
        &mut self,
        node_id: NodeId,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> bool {
        self.focus_manager
            .update_spatial(node_id, x, y, width, height)
    }

    /// Move focus in a specific direction
    /// Returns the focused node and emits focus event if focus changed
    pub fn focus_move(&mut self, direction: super::focus::FocusDirection) -> Option<NodeId> {
        let previous = self.get_focus();
        let (node_id, focus_event) = self.focus_manager.move_focus(direction);
        self.emit_focus_change(previous);
        self.emit_focus_operation_events(node_id, focus_event);
        node_id
    }

    /// Create a focus trap for a container (e.g., modal dialog)
    /// This restricts focus navigation to only nodes within the container
    pub fn create_focus_trap(&mut self, container: NodeId, trapped_nodes: Vec<NodeId>) -> bool {
        let previous = self.get_focus();
        let created = self
            .focus_manager
            .create_focus_trap(container, trapped_nodes);
        self.emit_focus_change(previous);
        created
    }

    /// Remove a focus trap and restore previous focus behavior
    pub fn remove_focus_trap(&mut self, container: NodeId) -> bool {
        let previous = self.get_focus();
        let removed = self.focus_manager.remove_focus_trap(container);
        self.emit_focus_change(previous);
        removed
    }

    /// Check if focus is currently trapped
    pub fn is_focus_trapped(&self) -> bool {
        self.focus_manager.is_focus_trapped()
    }

    /// Get the active focus trap container, if any
    pub fn get_active_focus_trap(&self) -> Option<NodeId> {
        self.focus_manager.get_active_focus_trap()
    }

    /// Get the root node
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Set the root node
    pub fn set_root(&mut self, node_id: NodeId) {
        self.root = Some(node_id);
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyCode, KeyEvent};
    use std::sync::Mutex;

    #[test]
    fn hover_capture_receives_each_own_boundary_once() {
        use crate::event::{
            hit::Bounds,
            types::{MouseEvent, MouseEventKind, Position},
        };
        let mut router = EventRouter::new();
        let parent = router.create_node(None);
        let child = router.create_node(Some(parent));
        router.add_hit_target(parent, Bounds::new(0.0, 0.0, 10.0, 2.0), 0);
        router.add_hit_target(child, Bounds::new(0.0, 0.0, 4.0, 1.0), 1);
        let observed = Arc::new(Mutex::new(Vec::new()));
        let output = observed.clone();
        router.add_handler(
            parent,
            "mouse",
            EventPhase::Capture,
            Arc::new(move |event| {
                if let Event::Mouse(mouse) = event {
                    if matches!(mouse.kind, MouseEventKind::Enter | MouseEventKind::Leave) {
                        output.lock().unwrap().push(mouse.kind.clone());
                    }
                }
                EventResult::Handled
            }),
        );
        for (x, y) in [(1, 0), (8, 1), (20, 5)] {
            router.process_event(&Event::Mouse(MouseEvent::new(
                MouseEventKind::Move,
                Position::cell(x, y),
            )));
        }
        assert_eq!(
            *observed.lock().unwrap(),
            [MouseEventKind::Enter, MouseEventKind::Leave]
        );
    }

    #[test]
    fn hover_boundaries_preserve_shared_ancestors_and_follow_layout() {
        use crate::event::{
            hit::Bounds,
            types::{MouseEvent, MouseEventKind, Position},
        };
        let mut router = EventRouter::new();
        let parent = router.create_node(None);
        let left = router.create_node(Some(parent));
        let right = router.create_node(Some(parent));
        router.add_hit_target(parent, Bounds::new(0.0, 0.0, 10.0, 1.0), 0);
        router.add_hit_target(left, Bounds::new(0.0, 0.0, 5.0, 1.0), 1);
        router.add_hit_target(right, Bounds::new(5.0, 0.0, 5.0, 1.0), 2);
        let events = Arc::new(Mutex::new(Vec::new()));
        for (id, name) in [(parent, "parent"), (left, "left"), (right, "right")] {
            let events = events.clone();
            router.add_handler(
                id,
                "mouse",
                EventPhase::Bubble,
                Arc::new(move |event| {
                    if let Event::Mouse(mouse) = event {
                        events.lock().unwrap().push((name, mouse.kind.clone()));
                    }
                    EventResult::Ignored
                }),
            );
        }
        for x in [1, 6] {
            router.process_event(&Event::Mouse(MouseEvent::new(
                MouseEventKind::Move,
                Position::cell(x, 0),
            )));
        }
        assert_eq!(
            *events.lock().unwrap(),
            vec![
                ("left", MouseEventKind::Enter),
                ("parent", MouseEventKind::Enter),
                ("left", MouseEventKind::Move),
                ("parent", MouseEventKind::Move),
                ("left", MouseEventKind::Leave),
                ("right", MouseEventKind::Enter),
                ("right", MouseEventKind::Move),
                ("parent", MouseEventKind::Move),
            ]
        );
        events.lock().unwrap().clear();
        router.add_hit_target(right, Bounds::new(20.0, 0.0, 5.0, 1.0), 2);
        assert!(router.refresh_hover());
        assert_eq!(
            *events.lock().unwrap(),
            vec![("right", MouseEventKind::Leave)]
        );
        events.lock().unwrap().clear();
        router.process_event(&Event::Mouse(MouseEvent::new(
            MouseEventKind::Leave,
            Position::cell(6, 0),
        )));
        assert_eq!(
            *events.lock().unwrap(),
            vec![("parent", MouseEventKind::Leave)]
        );
        assert_eq!(router.hovered_node(), None);
    }

    #[test]
    fn test_event_routing() {
        let mut router = EventRouter::new();

        // Create a simple tree: root -> child -> grandchild
        let root = router.create_node(None);
        let child = router.create_node(Some(root));
        let grandchild = router.create_node(Some(child));

        // Track events through phases
        let captured = Arc::new(Mutex::new(Vec::new()));
        let bubbled = Arc::new(Mutex::new(Vec::new()));

        // Add capture handler to root
        let captured_clone = captured.clone();
        router.add_handler(
            root,
            "key",
            EventPhase::Capture,
            Arc::new(move |_| {
                captured_clone.lock().unwrap().push("root_capture");
                EventResult::Handled
            }),
        );

        // Add bubble handler to child
        let bubbled_clone = bubbled.clone();
        router.add_handler(
            child,
            "key",
            EventPhase::Bubble,
            Arc::new(move |_| {
                bubbled_clone.lock().unwrap().push("child_bubble");
                EventResult::Handled
            }),
        );

        // Route an event from grandchild
        let event = Event::Key(KeyEvent::new(KeyCode::Enter));
        router.route_event(&event, grandchild);

        // Check that capture phase happened before bubble phase
        assert_eq!(*captured.lock().unwrap(), vec!["root_capture"]);
        assert_eq!(*bubbled.lock().unwrap(), vec!["child_bubble"]);
    }

    #[test]
    fn test_event_consumption() {
        let mut router = EventRouter::new();

        let root = router.create_node(None);
        let child = router.create_node(Some(root));

        // Add handler that consumes the event
        router.add_handler(
            root,
            "key",
            EventPhase::Capture,
            Arc::new(|_| EventResult::Consumed),
        );

        // Add handler that should not be called
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();
        router.add_handler(
            child,
            "key",
            EventPhase::Target,
            Arc::new(move |_| {
                *called_clone.lock().unwrap() = true;
                EventResult::Handled
            }),
        );

        // Route event
        let event = Event::Key(KeyEvent::new(KeyCode::Space));
        let result = router.route_event(&event, child);

        assert_eq!(result, EventResult::Consumed);
        assert!(!*called.lock().unwrap());
    }
}
