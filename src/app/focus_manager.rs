/// Focus manager that processes declarative focus properties from elements
///
/// This replaces the imperative focus trap system with a declarative one
/// that works during the render cycle.
use crate::component::{Element, FocusProps};
use crate::event::router::NodeId;
use std::collections::{HashMap, VecDeque};

/// Manages focus based on declarative properties from the element tree
pub struct FocusManager {
    /// Currently focused element
    current_focus: Option<NodeId>,

    /// Stack of focus contexts (for nested focus traps)
    focus_stack: Vec<FocusContext>,

    /// Elements requesting auto-focus this frame
    auto_focus_queue: VecDeque<NodeId>,

    /// Map of element IDs to their focus properties
    focus_props_map: HashMap<NodeId, FocusProps>,

    /// Previous focus for restoration
    previous_focus: Option<NodeId>,
}

/// A focus context (typically from trap_focus)
#[derive(Clone, Debug)]
struct FocusContext {
    /// Container that traps focus
    container_id: NodeId,

    /// Elements within this focus trap
    focusable_elements: Vec<NodeId>,

    /// Should restore focus when unmounted
    restore_focus: bool,

    /// Focus to restore to
    restore_to: Option<NodeId>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            current_focus: None,
            focus_stack: Vec::new(),
            auto_focus_queue: VecDeque::new(),
            focus_props_map: HashMap::new(),
            previous_focus: None,
        }
    }

    /// Process focus properties from the rendered element tree
    pub fn process_element_tree(&mut self, root: &Element, root_id: NodeId) {
        // Clear previous frame's data
        self.focus_props_map.clear();
        self.auto_focus_queue.clear();

        // Walk the tree and collect focus properties
        self.collect_focus_props(root, root_id);

        // Process auto-focus requests
        self.process_auto_focus();

        // Update focus traps
        self.update_focus_traps();
    }

    /// Recursively collect focus properties from the element tree
    fn collect_focus_props(&mut self, element: &Element, element_id: NodeId) {
        if let Some(focus) = &element.focus {
            self.focus_props_map.insert(element_id, focus.clone());

            // Queue auto-focus requests
            if focus.auto_focus {
                self.auto_focus_queue.push_back(element_id);
            }

            // Handle focus traps
            if focus.trap_focus {
                self.create_focus_trap(element_id, element, focus.restore_focus);
            }
        }

        // Process children
        for child in element.children.iter() {
            // Generate a unique child ID
            let child_id = NodeId::new();
            self.collect_focus_props(child, child_id);
        }
    }

    /// Process auto-focus queue (first element requesting auto-focus wins)
    fn process_auto_focus(&mut self) {
        if let Some(element_id) = self.auto_focus_queue.pop_front() {
            // Check if element is within current focus trap (if any)
            if self.can_focus(&element_id) {
                self.set_focus(element_id);
            }
        }
    }

    /// Create a focus trap for a container
    fn create_focus_trap(&mut self, container_id: NodeId, element: &Element, restore: bool) {
        let mut focusable_elements = Vec::new();
        self.collect_focusable_children(element, &container_id, &mut focusable_elements);

        let context = FocusContext {
            container_id,
            focusable_elements,
            restore_focus: restore,
            restore_to: self.current_focus,
        };

        self.focus_stack.push(context);
    }

    /// Collect all focusable children within a container
    fn collect_focusable_children(
        &self,
        element: &Element,
        _parent_id: &NodeId,
        focusable: &mut Vec<NodeId>,
    ) {
        for child in element.children.iter() {
            let child_id = NodeId::new();

            if let Some(focus) = &child.focus {
                if focus.focusable && focus.tab_index >= 0 {
                    focusable.push(child_id);
                }
            }

            // Recurse into children
            self.collect_focusable_children(child, &child_id, focusable);
        }
    }

    /// Update focus traps based on current element tree
    fn update_focus_traps(&mut self) {
        // Remove stale focus traps (whose containers no longer exist)
        self.focus_stack
            .retain(|context| self.focus_props_map.contains_key(&context.container_id));

        // If a trap was removed and had restore_focus, restore it
        if let Some(context) = self.focus_stack.last() {
            if !self.focus_props_map.contains_key(&context.container_id) && context.restore_focus {
                if let Some(restore_to) = &context.restore_to {
                    self.set_focus(*restore_to);
                }
            }
        }
    }

    /// Check if an element can receive focus given current constraints
    fn can_focus(&self, element_id: &NodeId) -> bool {
        // If there's a focus trap, element must be within it
        if let Some(trap) = self.focus_stack.last() {
            trap.focusable_elements.contains(element_id)
        } else {
            // No trap, check if element is focusable
            self.focus_props_map
                .get(element_id)
                .map(|props| props.focusable && props.tab_index >= 0)
                .unwrap_or(false)
        }
    }

    /// Set focus to an element
    pub fn set_focus(&mut self, element_id: NodeId) {
        if self.can_focus(&element_id) {
            self.previous_focus = self.current_focus;
            self.current_focus = Some(element_id);

            // Call focus callbacks
            if let Some(props) = self.focus_props_map.get(&element_id) {
                if let Some(on_focus) = &props.on_focus {
                    on_focus();
                }
            }

            // Call blur on previous
            if let Some(prev) = &self.previous_focus {
                if let Some(props) = self.focus_props_map.get(prev) {
                    if let Some(on_blur) = &props.on_blur {
                        on_blur();
                    }
                }
            }
        }
    }

    /// Get the currently focused element
    pub fn current_focus(&self) -> Option<&NodeId> {
        self.current_focus.as_ref()
    }

    /// Move focus to the next focusable element
    pub fn focus_next(&mut self) {
        let focusable = if let Some(trap) = self.focus_stack.last() {
            &trap.focusable_elements
        } else {
            // Get all focusable elements
            return; // For now, skip if no trap
        };

        if focusable.is_empty() {
            return;
        }

        let current_index = self
            .current_focus
            .as_ref()
            .and_then(|f| focusable.iter().position(|e| e == f))
            .unwrap_or(0);

        let next_index = (current_index + 1) % focusable.len();
        self.set_focus(focusable[next_index]);
    }

    /// Move focus to the previous focusable element
    pub fn focus_previous(&mut self) {
        let focusable = if let Some(trap) = self.focus_stack.last() {
            &trap.focusable_elements
        } else {
            return;
        };

        if focusable.is_empty() {
            return;
        }

        let current_index = self
            .current_focus
            .as_ref()
            .and_then(|f| focusable.iter().position(|e| e == f))
            .unwrap_or(0);

        let prev_index = if current_index == 0 {
            focusable.len() - 1
        } else {
            current_index - 1
        };

        self.set_focus(focusable[prev_index]);
    }
}
