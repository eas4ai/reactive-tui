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
    /// Create a new unique node ID
    pub fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// Event router managing event propagation through component tree
pub struct EventRouter {
    nodes: HashMap<NodeId, EventNode>,
    root: Option<NodeId>,
    focus_manager: FocusManager,
    hit_test: HitTest,
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

    /// Remove an event node and all its descendants
    pub fn remove_node(&mut self, id: NodeId) {
        // Remove from parent's children
        if let Some(node) = self.nodes.get(&id) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.children.retain(|&child| child != id);
                }
            }
        }

        // Remove node and all descendants
        let mut to_remove = vec![id];
        while let Some(node_id) = to_remove.pop() {
            if let Some(node) = self.nodes.remove(&node_id) {
                to_remove.extend(&node.children);
            }
        }

        // Clear root if removed
        if self.root == Some(id) {
            self.root = None;
        }

        // Clear focus if removed
        if self.focus_manager.get_focus() == Some(id) {
            self.focus_manager.set_focus(None);
        }
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

        // Build path with pre-allocated capacity
        let mut path = Vec::with_capacity(16); // Most UI trees are < 16 levels deep
        let mut current = Some(target_id);

        while let Some(node_id) = current {
            path.push(node_id);
            if let Some(node) = self.nodes.get(&node_id) {
                current = node.parent;
            } else {
                break;
            }
        }

        path.reverse(); // Now path goes from root to target

        // Capture phase - root to target (excluding target)
        for &node_id in &path[..path.len().saturating_sub(1)] {
            if let Some(node) = self.nodes.get(&node_id) {
                if let Some(handlers) = node.capture_handlers.get(event_type) {
                    for handler in handlers {
                        match (handler.handler)(event) {
                            EventResult::Consumed => return EventResult::Consumed,
                            EventResult::Captured => {} // Continue to bubble phase
                            EventResult::Handled => {}
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
                        EventResult::Captured => {} // Captured only meaningful in capture phase
                        EventResult::Handled => {}
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
                                    EventResult::Captured => {} // Captured only meaningful in capture phase
                                    EventResult::Handled => {}
                                    EventResult::Ignored => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        EventResult::Ignored
    }

    /// Dispatch an event to the focused node
    pub fn dispatch_to_focus(&self, event: &Event) -> EventResult {
        if let Some(focus_id) = self.focus_manager.get_focus() {
            self.route_event(event, focus_id)
        } else if let Some(root_id) = self.root {
            self.route_event(event, root_id)
        } else {
            EventResult::Ignored
        }
    }

    /// Advance focus to next node id
    /// Returns the focused node and collects events for later emission
    pub fn focus_next(&mut self) -> Option<NodeId> {
        let (node_id, focus_event) = self.focus_manager.focus_next();
        self.emit_focus_operation_events(node_id, focus_event);
        node_id
    }

    /// Move focus to previous node id
    /// Returns the focused node and emits focus event if focus changed
    pub fn focus_prev(&mut self) -> Option<NodeId> {
        let (node_id, focus_event) = self.focus_manager.focus_previous();
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

    /// Helper method to emit focus events from a focus operation result
    fn emit_focus_operation_events(&self, node_id: Option<NodeId>, focus_event: Option<super::types::FocusEvent>) {
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

    /// Process an event - THE central event processing method
    pub fn process_event(&mut self, event: &Event) -> EventResult {
        // 1. Handle system events first (focus traversal, etc.)
        if let Some(result) = self.handle_system_event(event) {
            return result;
        }

        // 2. Determine target node
        let target = self.determine_target(event);

        // 3. Route the event
        self.route_event(event, target)
    }

    /// Handle system-level events (focus traversal, global shortcuts)
    fn handle_system_event(&mut self, event: &Event) -> Option<EventResult> {
        match event {
            Event::Key(key_event) => {
                use super::types::KeyCode;
                if key_event.code == KeyCode::Tab {
                    if key_event.modifiers.shift {
                        if self.focus_prev().is_some() {
                            return Some(EventResult::Handled);
                        }
                    } else if self.focus_next().is_some() {
                        return Some(EventResult::Handled);
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
    fn determine_target(&self, event: &Event) -> NodeId {
        match event {
            Event::Mouse(mouse_event) => {
                // Use hit testing for mouse events
                let point = super::hit::Point::new(
                    mouse_event.position.x() as f32,
                    mouse_event.position.y() as f32,
                );

                if let Some(node_id) = self.hit_test.hit_test(point) {
                    node_id
                } else {
                    // Default to root or focused node
                    self.focus_manager.get_focus().or(self.root).unwrap_or_default()
                }
            }
            _ => {
                // For keyboard and other events, use focused node or root
                self.focus_manager.get_focus().or(self.root).unwrap_or_default()
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
        self.focus_manager.register_focusable(node_id, tab_index, true)
    }



    /// Remove a focusable node
    pub fn remove_focusable(&mut self, node_id: NodeId) {
        self.focus_manager.unregister_focusable(node_id);
    }

    /// Update spatial information for a node (for arrow-key navigation)
    /// Returns true if successful
    pub fn update_spatial(&mut self, node_id: NodeId, x: f32, y: f32, width: f32, height: f32) -> bool {
        self.focus_manager.update_spatial(node_id, x, y, width, height)
    }

    /// Move focus in a specific direction
    /// Returns the focused node and emits focus event if focus changed
    pub fn focus_move(&mut self, direction: super::focus::FocusDirection) -> Option<NodeId> {
        let (node_id, focus_event) = self.focus_manager.move_focus(direction);
        self.emit_focus_operation_events(node_id, focus_event);
        node_id
    }

    /// Create a focus trap for a container (e.g., modal dialog)
    /// This restricts focus navigation to only nodes within the container
    pub fn create_focus_trap(&mut self, container: NodeId, trapped_nodes: Vec<NodeId>) -> bool {
        self.focus_manager.create_focus_trap(container, trapped_nodes)
    }

    /// Remove a focus trap and restore previous focus behavior
    pub fn remove_focus_trap(&mut self, container: NodeId) -> bool {
        self.focus_manager.remove_focus_trap(container)
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
