use super::{Border, ScrollState};
use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyCode, KeyModifiers};
use std::collections::HashMap;
use std::sync::Arc;

mod live;

type NodeActionCallback = dyn Fn(String, &str) + Send + Sync;

/// Tree node representing an item in the tree structure
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode {
    /// Unique identifier for this node
    pub id: String,
    /// Display label for this node
    pub label: String,
    /// Child nodes of this node
    pub children: Vec<TreeNode>,
    /// Whether this node is expanded to show children
    pub expanded: bool,
    /// Whether this node is currently selected
    pub selected: bool,
    /// Checkbox state (None=no checkbox, Some(bool)=checked state)
    pub checked: Option<bool>,
    /// Optional icon identifier for this node
    pub icon: Option<String>,
    /// Optional CSS class or style for this node
    pub style: Option<String>,
    /// Additional data associated with this node
    pub data: HashMap<String, String>,
    /// Whether this node can be selected
    pub selectable: bool,
    /// Whether this node can be checked
    pub checkable: bool,
    /// Whether this node can be expanded
    pub expandable: bool,
    /// Whether this node loads children lazily
    pub lazy: bool,
    /// Whether this node is currently loading children
    pub loading: bool,
    /// Nesting level of this node (0=root)
    pub level: usize,
    /// ID of the parent node (None for root nodes)
    pub parent_id: Option<String>,
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
            expanded: false,
            selected: false,
            checked: None,
            icon: None,
            style: None,
            data: HashMap::new(),
            selectable: true,
            checkable: false,
            expandable: true,
            lazy: false,
            loading: false,
            level: 0,
            parent_id: None,
        }
    }

    /// Add a child node
    pub fn add_child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: Vec<TreeNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set expanded state
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Set selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set checked state
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Set icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set icon (alternative name for compatibility)
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set children (alternative name for compatibility)
    pub fn with_children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }

    /// Set style for the node
    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    /// Find a node by ID in this subtree
    pub fn find_node(&self, id: &str) -> Option<&TreeNode> {
        if self.id == id {
            return Some(self);
        }

        for child in &self.children {
            if let Some(found) = child.find_node(id) {
                return Some(found);
            }
        }

        None
    }

    /// Find a mutable node by ID in this subtree
    pub fn find_node_mut(&mut self, id: &str) -> Option<&mut TreeNode> {
        if self.id == id {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(found) = child.find_node_mut(id) {
                return Some(found);
            }
        }

        None
    }
}

/// Builder for creating Tree components with a fluent API
#[derive(Clone)]
pub struct TreeBuilder {
    root: Option<TreeNode>,
    selected_node: Option<String>,
    expanded_nodes: Vec<String>,
    selectable: bool,
    multi_select: bool,
    show_icons: bool,
    show_lines: bool,
    indent_size: u16,
    lazy_loading: bool,
    checkable: bool,
    drag_drop: bool,
    search_term: Option<String>,
    filter_visible: bool,
    border: Border,
    node_style: Option<String>,
    selected_style: Option<String>,
    expanded_style: Option<String>,
    leaf_style: Option<String>,
    line_style: Option<String>,
    scrollable: bool,
    max_height: Option<u16>,
    virtual_scrolling: bool,
    on_select: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    on_multi_select: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
    on_expand: Option<Arc<dyn Fn(String, bool) + Send + Sync>>,
    on_check: Option<Arc<dyn Fn(String, bool) + Send + Sync>>,
    on_node_action: Option<Arc<NodeActionCallback>>,
    on_load_children: Option<Arc<dyn Fn(String) -> Vec<TreeNode> + Send + Sync>>,
}

impl TreeBuilder {
    /// Create a new TreeBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the root node
    pub fn root(mut self, root: TreeNode) -> Self {
        self.root = Some(root);
        self
    }

    /// Set the selected node ID
    pub fn selected(mut self, id: impl Into<String>) -> Self {
        self.selected_node = Some(id.into());
        self
    }

    /// Add an expanded node ID
    pub fn expand(mut self, id: impl Into<String>) -> Self {
        self.expanded_nodes.push(id.into());
        self
    }

    /// Set expanded nodes
    pub fn expanded_nodes(mut self, nodes: Vec<String>) -> Self {
        self.expanded_nodes = nodes;
        self
    }

    /// Enable or disable selection
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Enable multi-selection
    pub fn multi_select(mut self, multi: bool) -> Self {
        self.multi_select = multi;
        self
    }

    /// Show or hide icons
    pub fn show_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
        self
    }

    /// Show or hide tree lines
    pub fn show_lines(mut self, show: bool) -> Self {
        self.show_lines = show;
        self
    }

    /// Set indent size
    pub fn indent_size(mut self, size: u16) -> Self {
        self.indent_size = size;
        self
    }

    /// Enable lazy loading
    pub fn lazy_loading(mut self, lazy: bool) -> Self {
        self.lazy_loading = lazy;
        self
    }

    /// Enable checkboxes
    pub fn checkable(mut self, checkable: bool) -> Self {
        self.checkable = checkable;
        self
    }

    /// Enable drag and drop
    pub fn drag_drop(mut self, enable: bool) -> Self {
        self.drag_drop = enable;
        self
    }

    /// Set search term
    pub fn search(mut self, term: impl Into<String>) -> Self {
        self.search_term = Some(term.into());
        self
    }

    /// Enable filtering
    pub fn filter_visible(mut self, filter: bool) -> Self {
        self.filter_visible = filter;
        self
    }

    /// Set border style
    pub fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    /// Set node style
    pub fn node_style(mut self, style: impl Into<String>) -> Self {
        self.node_style = Some(style.into());
        self
    }

    /// Set selected node style
    pub fn selected_style(mut self, style: impl Into<String>) -> Self {
        self.selected_style = Some(style.into());
        self
    }

    /// Set expanded node style
    pub fn expanded_style(mut self, style: impl Into<String>) -> Self {
        self.expanded_style = Some(style.into());
        self
    }

    /// Set leaf node style
    pub fn leaf_style(mut self, style: impl Into<String>) -> Self {
        self.leaf_style = Some(style.into());
        self
    }

    /// Set tree line style
    pub fn line_style(mut self, style: impl Into<String>) -> Self {
        self.line_style = Some(style.into());
        self
    }

    /// Enable scrolling
    pub fn scrollable(mut self, scroll: bool) -> Self {
        self.scrollable = scroll;
        self
    }

    /// Set maximum height
    pub fn max_height(mut self, height: u16) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Enable virtual scrolling
    pub fn virtual_scrolling(mut self, virtual_scroll: bool) -> Self {
        self.virtual_scrolling = virtual_scroll;
        self
    }

    /// Set selection callback
    pub fn on_select(mut self, callback: Arc<dyn Fn(Option<String>) + Send + Sync>) -> Self {
        self.on_select = Some(callback);
        self
    }

    /// Set expand callback
    pub fn on_expand(mut self, callback: Arc<dyn Fn(String, bool) + Send + Sync>) -> Self {
        self.on_expand = Some(callback);
        self
    }

    /// Set the multi-selection callback.
    pub fn on_multi_select(mut self, callback: Arc<dyn Fn(Vec<String>) + Send + Sync>) -> Self {
        self.on_multi_select = Some(callback);
        self
    }

    /// Set the checkbox callback.
    pub fn on_check(mut self, callback: Arc<dyn Fn(String, bool) + Send + Sync>) -> Self {
        self.on_check = Some(callback);
        self
    }

    /// Set the activation and in-memory drop callback.
    pub fn on_node_action(mut self, callback: Arc<NodeActionCallback>) -> Self {
        self.on_node_action = Some(callback);
        self
    }

    /// Supply children when an unloaded lazy node is expanded.
    pub fn on_load_children(
        mut self,
        callback: Arc<dyn Fn(String) -> Vec<TreeNode> + Send + Sync>,
    ) -> Self {
        self.on_load_children = Some(callback);
        self
    }

    /// Build the TreeProps
    pub fn build(self) -> TreeProps {
        TreeProps {
            root: self.root,
            selected_node: self.selected_node,
            expanded_nodes: self.expanded_nodes,
            selectable: self.selectable,
            multi_select: self.multi_select,
            show_icons: self.show_icons,
            show_lines: self.show_lines,
            indent_size: self.indent_size,
            lazy_loading: self.lazy_loading,
            checkable: self.checkable,
            drag_drop: self.drag_drop,
            search_term: self.search_term,
            filter_visible: self.filter_visible,
            border: self.border,
            node_style: self.node_style,
            selected_style: self.selected_style,
            expanded_style: self.expanded_style,
            leaf_style: self.leaf_style,
            line_style: self.line_style,
            scrollable: self.scrollable,
            max_height: self.max_height,
            virtual_scrolling: self.virtual_scrolling,
            on_select: self.on_select,
            on_multi_select: self.on_multi_select,
            on_expand: self.on_expand,
            on_check: self.on_check,
            on_node_action: self.on_node_action,
            on_load_children: self.on_load_children,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("Tree").with_props(self.build())
    }
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self {
            root: None,
            selected_node: None,
            expanded_nodes: Vec::new(),
            selectable: true,
            multi_select: false,
            show_icons: true,
            show_lines: true,
            indent_size: 2,
            lazy_loading: false,
            checkable: false,
            drag_drop: false,
            search_term: None,
            filter_visible: false,
            border: Border::default(),
            node_style: None,
            selected_style: Some("bg-blue fg-white".to_string()),
            expanded_style: None,
            leaf_style: None,
            line_style: Some("fg-gray".to_string()),
            scrollable: true,
            max_height: None,
            virtual_scrolling: false,
            on_select: None,
            on_multi_select: None,
            on_expand: None,
            on_check: None,
            on_node_action: None,
            on_load_children: None,
        }
    }
}

/// Props for the Tree component
#[derive(Clone)]
pub struct TreeProps {
    /// Root node of the tree
    pub root: Option<TreeNode>,
    /// ID of the currently selected node
    pub selected_node: Option<String>,
    /// List of expanded node IDs
    pub expanded_nodes: Vec<String>,
    /// Whether nodes can be selected
    pub selectable: bool,
    /// Whether multiple nodes can be selected
    pub multi_select: bool,
    /// Whether to show icons for nodes
    pub show_icons: bool,
    /// Whether to show tree lines
    pub show_lines: bool,
    /// Indentation size for nested nodes
    pub indent_size: u16,
    /// Whether to enable lazy loading of children
    pub lazy_loading: bool,
    /// Whether nodes can be checked
    pub checkable: bool,
    /// Whether drag and drop is enabled
    pub drag_drop: bool,
    /// Current search term for filtering
    pub search_term: Option<String>,
    /// Whether to filter visible nodes based on search
    pub filter_visible: bool,
    /// Border style for the tree
    pub border: Border,
    /// Style for regular nodes
    pub node_style: Option<String>,
    /// Style for selected nodes
    pub selected_style: Option<String>,
    /// Style for expanded nodes
    pub expanded_style: Option<String>,
    /// Style for leaf nodes
    pub leaf_style: Option<String>,
    /// Style for tree lines
    pub line_style: Option<String>,
    /// Whether the tree is scrollable
    pub scrollable: bool,
    /// Maximum height of the tree
    pub max_height: Option<u16>,
    /// Whether to use virtual scrolling for performance
    pub virtual_scrolling: bool,
    /// Callback for node selection
    pub on_select: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    /// Callback for multi-selection changes
    pub on_multi_select: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
    /// Callback for node expand/collapse
    pub on_expand: Option<Arc<dyn Fn(String, bool) + Send + Sync>>,
    /// Callback for node check/uncheck
    pub on_check: Option<Arc<dyn Fn(String, bool) + Send + Sync>>,
    /// Callback for custom node actions
    pub on_node_action: Option<Arc<NodeActionCallback>>,
    /// Callback for lazy loading children
    pub on_load_children: Option<Arc<dyn Fn(String) -> Vec<TreeNode> + Send + Sync>>,
}

impl Default for TreeProps {
    fn default() -> Self {
        Self {
            root: None,
            selected_node: None,
            expanded_nodes: Vec::new(),
            selectable: true,
            multi_select: false,
            show_icons: true,
            show_lines: true,
            indent_size: 2,
            lazy_loading: false,
            checkable: false,
            drag_drop: false,
            search_term: None,
            filter_visible: false,
            border: Border::default(),
            node_style: None,
            selected_style: Some("bg-blue fg-white".to_string()),
            expanded_style: None,
            leaf_style: None,
            line_style: Some("fg-gray".to_string()),
            scrollable: true,
            max_height: None,
            virtual_scrolling: false,
            on_select: None,
            on_multi_select: None,
            on_expand: None,
            on_check: None,
            on_node_action: None,
            on_load_children: None,
        }
    }
}

impl PartialEq for TreeProps {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
            && self.selected_node == other.selected_node
            && self.expanded_nodes == other.expanded_nodes
            && self.selectable == other.selectable
            && self.multi_select == other.multi_select
            && self.show_icons == other.show_icons
            && self.show_lines == other.show_lines
            && self.indent_size == other.indent_size
            && self.lazy_loading == other.lazy_loading
            && self.checkable == other.checkable
            && self.drag_drop == other.drag_drop
            && self.search_term == other.search_term
            && self.filter_visible == other.filter_visible
            && self.border == other.border
            && self.node_style == other.node_style
            && self.selected_style == other.selected_style
            && self.expanded_style == other.expanded_style
            && self.leaf_style == other.leaf_style
            && self.line_style == other.line_style
            && self.scrollable == other.scrollable
            && self.max_height == other.max_height
            && self.virtual_scrolling == other.virtual_scrolling
        // Skip callback comparisons as they can't be compared
    }
}

impl Props for TreeProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// State for the Tree component
#[derive(Debug, Clone, Default)]
pub struct TreeState {
    /// List of selected node IDs
    pub selected_nodes: Vec<String>,
    /// List of expanded node IDs
    pub expanded_nodes: Vec<String>,
    /// List of checked node IDs
    pub checked_nodes: Vec<String>,
    /// Scroll state for the tree
    pub scroll_state: ScrollState,
    /// Whether the tree has focus
    pub focused: bool,
    /// Node ID under mouse cursor
    pub hover_node: Option<String>,
    /// Flattened tree structure for rendering
    pub flat_nodes: Vec<FlatTreeNode>,
    /// List of currently visible node IDs
    pub visible_nodes: Vec<String>,
    /// List of nodes matching current search
    pub search_matches: Vec<String>,
    /// List of nodes currently loading children
    pub loading_nodes: Vec<String>,
    /// Node being dragged (if any)
    pub drag_source: Option<String>,
    /// Node being targeted for drop (if any)
    pub drop_target: Option<String>,
}

/// Flattened tree node for efficient rendering
#[derive(Debug, Clone)]
pub struct FlatTreeNode {
    /// Unique identifier for this node
    pub id: String,
    /// Display label for this node
    pub label: String,
    /// Nesting level of this node (0=root)
    pub level: usize,
    /// ID of the parent node (None for root nodes)
    pub parent_id: Option<String>,
    /// Whether this node has child nodes
    pub has_children: bool,
    /// Whether this node is expanded to show children
    pub expanded: bool,
    /// Whether this node is currently selected
    pub selected: bool,
    /// Checkbox state (None=no checkbox, Some(bool)=checked state)
    pub checked: Option<bool>,
    /// Optional icon identifier for this node
    pub icon: Option<String>,
    /// Optional CSS class or style for this node
    pub style: Option<String>,
    /// Whether this node is visible (not filtered out)
    pub visible: bool,
    /// Whether this node matches the current search
    pub matched: bool,
}

// TreeState implements Default + Send + Sync automatically

/// Tree display component for hierarchical data
pub struct Tree;

impl Tree {
    /// Create a new Tree with default props
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Element {
        Element::component_with_props("Tree", TreeProps::default())
    }

    /// Create a Tree with custom props
    pub fn with_props(props: TreeProps) -> Element {
        Element::component_with_props("Tree", props)
    }

    /// Builder method for root node
    pub fn with_root(mut props: TreeProps, root: TreeNode) -> TreeProps {
        props.root = Some(root);
        props
    }

    /// Builder method for selection
    pub fn selectable(mut props: TreeProps, selectable: bool) -> TreeProps {
        props.selectable = selectable;
        props
    }

    /// Builder method for checkboxes
    pub fn checkable(mut props: TreeProps, checkable: bool) -> TreeProps {
        props.checkable = checkable;
        props
    }

    /// Builder method for icons
    pub fn with_icons(mut props: TreeProps, show_icons: bool) -> TreeProps {
        props.show_icons = show_icons;
        props
    }

    /// Builder method for lines
    pub fn with_lines(mut props: TreeProps, show_lines: bool) -> TreeProps {
        props.show_lines = show_lines;
        props
    }

    fn flatten_tree(&self, props: &TreeProps, state: &TreeState) -> Vec<FlatTreeNode> {
        let mut flat_nodes = Vec::new();

        if let Some(root) = &props.root {
            self.flatten_node(root, &mut flat_nodes, 0, None, props, state);
        }

        flat_nodes
    }

    fn flatten_node(
        &self,
        node: &TreeNode,
        flat_nodes: &mut Vec<FlatTreeNode>,
        level: usize,
        parent_id: Option<String>,
        props: &TreeProps,
        state: &TreeState,
    ) {
        let is_expanded = state.expanded_nodes.contains(&node.id);
        let is_selected = state.selected_nodes.contains(&node.id);
        let is_checked = state.checked_nodes.contains(&node.id);
        let is_visible = self.is_node_visible(node, props, state);
        let is_matched = state.search_matches.contains(&node.id);

        flat_nodes.push(FlatTreeNode {
            id: node.id.clone(),
            label: node.label.clone(),
            level,
            parent_id,
            has_children: !node.children.is_empty(),
            expanded: is_expanded,
            selected: is_selected,
            checked: if props.checkable {
                Some(is_checked)
            } else {
                None
            },
            icon: node.icon.clone(),
            style: node.style.clone(),
            visible: is_visible,
            matched: is_matched,
        });

        // Add children if:
        // - Node is root (level 0) - always show immediate children
        // - Node is expanded
        // - We're searching and node matches
        // - We're searching (show all nodes to find matches)
        let is_searching =
            props.search_term.is_some() && !props.search_term.as_ref().unwrap().is_empty();
        if (level == 0 || is_expanded || state.search_matches.contains(&node.id) || is_searching)
            && !node.children.is_empty()
        {
            for child in &node.children {
                self.flatten_node(
                    child,
                    flat_nodes,
                    level + 1,
                    Some(node.id.clone()),
                    props,
                    state,
                );
            }
        }
    }

    fn is_node_visible(&self, node: &TreeNode, props: &TreeProps, _state: &TreeState) -> bool {
        // Always visible if no search
        if let Some(search_term) = &props.search_term {
            if search_term.is_empty() {
                return true;
            }

            // Node matches search
            if node
                .label
                .to_lowercase()
                .contains(&search_term.to_lowercase())
            {
                return true;
            }

            // Has matching children
            if self.has_matching_children(node, search_term) {
                return true;
            }

            // Parent matches (if filter_visible is false)
            if !props.filter_visible {
                return true;
            }

            false
        } else {
            true
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn has_matching_children(&self, node: &TreeNode, search_term: &str) -> bool {
        for child in &node.children {
            if child
                .label
                .to_lowercase()
                .contains(&search_term.to_lowercase())
            {
                return true;
            }
            if self.has_matching_children(child, search_term) {
                return true;
            }
        }
        false
    }

    fn handle_key_navigation(
        &self,
        key: KeyCode,
        modifiers: KeyModifiers,
        props: &TreeProps,
        state: &mut TreeState,
    ) -> EventResult {
        if !props.selectable || state.flat_nodes.is_empty() {
            return EventResult::Ignored;
        }

        let current_selected = state.selected_nodes.first().cloned();
        let current_index = current_selected
            .and_then(|id| state.flat_nodes.iter().position(|n| n.id == id))
            .unwrap_or(0);

        match key {
            KeyCode::Up => {
                let new_index = if current_index > 0 {
                    current_index - 1
                } else {
                    state.flat_nodes.len() - 1
                };
                let node_id = state.flat_nodes.get(new_index).map(|n| n.id.clone());
                if let Some(id) = node_id {
                    self.select_node(props, state, &id, modifiers.shift);
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                let new_index = (current_index + 1) % state.flat_nodes.len();
                let node_id = state.flat_nodes.get(new_index).map(|n| n.id.clone());
                if let Some(id) = node_id {
                    self.select_node(props, state, &id, modifiers.shift);
                }
                EventResult::Consumed
            }
            KeyCode::Left => {
                if let Some(selected_id) = state.selected_nodes.first().cloned() {
                    if state.expanded_nodes.contains(&selected_id) {
                        // Collapse current node
                        self.toggle_expansion(props, state, &selected_id, false);
                    } else {
                        // Select parent
                        let parent_id = state
                            .flat_nodes
                            .iter()
                            .find(|n| n.id == selected_id)
                            .and_then(|n| n.parent_id.clone());
                        if let Some(parent_id) = parent_id {
                            self.select_node(props, state, &parent_id, false);
                        }
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Right => {
                if let Some(selected_id) = state.selected_nodes.first().cloned() {
                    let node_info = state
                        .flat_nodes
                        .iter()
                        .find(|n| n.id == selected_id)
                        .map(|n| (n.has_children, n.expanded))
                        .unwrap_or((false, false));

                    if node_info.0 {
                        // has_children
                        if !node_info.1 {
                            // not expanded
                            // Expand current node
                            self.toggle_expansion(props, state, &selected_id, true);
                        } else {
                            // Select first child
                            let child_id = state
                                .flat_nodes
                                .iter()
                                .find(|n| n.parent_id.as_ref() == Some(&selected_id))
                                .map(|n| n.id.clone());
                            if let Some(child_id) = child_id {
                                self.select_node(props, state, &child_id, false);
                            }
                        }
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                let first_id = state.flat_nodes.first().map(|n| n.id.clone());
                if let Some(id) = first_id {
                    self.select_node(props, state, &id, modifiers.shift);
                }
                EventResult::Consumed
            }
            KeyCode::End => {
                let last_id = state.flat_nodes.last().map(|n| n.id.clone());
                if let Some(id) = last_id {
                    self.select_node(props, state, &id, modifiers.shift);
                }
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(selected_id) = state.selected_nodes.first().cloned() {
                    let node_expanded = state
                        .flat_nodes
                        .iter()
                        .find(|n| n.id == selected_id)
                        .map(|n| (n.has_children, n.expanded))
                        .unwrap_or((false, false));

                    if node_expanded.0 {
                        // has_children
                        self.toggle_expansion(props, state, &selected_id, !node_expanded.1);
                    }

                    if let Some(callback) = &props.on_node_action {
                        callback(selected_id.clone(), "activate");
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Char(' ') => {
                if let Some(selected_id) = state.selected_nodes.first().cloned() {
                    if props.checkable {
                        self.toggle_check(props, state, &selected_id);
                    } else if props.multi_select {
                        self.toggle_node_selection(props, state, &selected_id);
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if let Some(selected_id) = state.selected_nodes.first().cloned() {
                    self.toggle_expansion(props, state, &selected_id, true);
                }
                EventResult::Consumed
            }
            KeyCode::Char('-') => {
                if let Some(selected_id) = state.selected_nodes.first().cloned() {
                    self.toggle_expansion(props, state, &selected_id, false);
                }
                EventResult::Consumed
            }
            KeyCode::Char('*') => {
                // Expand all at current level
                if let Some(selected_id) = state.selected_nodes.first().cloned() {
                    let level = state
                        .flat_nodes
                        .iter()
                        .find(|n| n.id == selected_id)
                        .map(|n| n.level)
                        .unwrap_or(0);
                    self.expand_level(props, state, level);
                }
                EventResult::Consumed
            }
            KeyCode::Char('a') if modifiers.ctrl => {
                if props.multi_select {
                    state.selected_nodes = state.flat_nodes.iter().map(|n| n.id.clone()).collect();
                    if let Some(callback) = &props.on_multi_select {
                        callback(state.selected_nodes.clone());
                    }
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn select_node(&self, props: &TreeProps, state: &mut TreeState, node_id: &str, extend: bool) {
        if props.multi_select && extend {
            if !state.selected_nodes.contains(&node_id.to_string()) {
                state.selected_nodes.push(node_id.to_string());
            }
        } else {
            state.selected_nodes.clear();
            state.selected_nodes.push(node_id.to_string());
        }

        // Trigger callbacks
        if props.multi_select {
            if let Some(callback) = &props.on_multi_select {
                callback(state.selected_nodes.clone());
            }
        } else if let Some(callback) = &props.on_select {
            callback(Some(node_id.to_string()));
        }

        // Auto-scroll to selected node
        self.scroll_to_node(state, node_id);
    }

    fn toggle_node_selection(&self, props: &TreeProps, state: &mut TreeState, node_id: &str) {
        if let Some(pos) = state.selected_nodes.iter().position(|id| id == node_id) {
            state.selected_nodes.remove(pos);
        } else {
            state.selected_nodes.push(node_id.to_string());
        }

        if let Some(callback) = &props.on_multi_select {
            callback(state.selected_nodes.clone());
        }
    }

    fn toggle_expansion(
        &self,
        props: &TreeProps,
        state: &mut TreeState,
        node_id: &str,
        expand: bool,
    ) {
        if expand {
            if !state.expanded_nodes.contains(&node_id.to_string()) {
                state.expanded_nodes.push(node_id.to_string());

                // Handle lazy loading
                if props.lazy_loading {
                    if let Some(callback) = &props.on_load_children {
                        state.loading_nodes.push(node_id.to_string());
                        callback(node_id.to_string());
                    }
                }
            }
        } else {
            state.expanded_nodes.retain(|id| id != node_id);
        }

        if let Some(callback) = &props.on_expand {
            callback(node_id.to_string(), expand);
        }

        // Rebuild flat tree
        state.flat_nodes = self.flatten_tree(props, state);
    }

    fn toggle_check(&self, props: &TreeProps, state: &mut TreeState, node_id: &str) {
        if state.checked_nodes.contains(&node_id.to_string()) {
            state.checked_nodes.retain(|id| id != node_id);
        } else {
            state.checked_nodes.push(node_id.to_string());
        }

        if let Some(callback) = &props.on_check {
            callback(
                node_id.to_string(),
                state.checked_nodes.contains(&node_id.to_string()),
            );
        }
    }

    fn expand_level(&self, props: &TreeProps, state: &mut TreeState, level: usize) {
        for node in &state.flat_nodes {
            if node.level == level && node.has_children && !state.expanded_nodes.contains(&node.id)
            {
                state.expanded_nodes.push(node.id.clone());
            }
        }

        // Rebuild flat tree
        state.flat_nodes = self.flatten_tree(props, state);
    }

    fn scroll_to_node(&self, state: &mut TreeState, node_id: &str) {
        if let Some(index) = state.flat_nodes.iter().position(|n| n.id == node_id) {
            let node_y = index as u16;
            let viewport_top = state.scroll_state.offset_y;
            let viewport_bottom = viewport_top + state.scroll_state.viewport_height;

            if node_y < viewport_top {
                state.scroll_state.offset_y = node_y;
            } else if node_y >= viewport_bottom {
                state.scroll_state.offset_y =
                    node_y.saturating_sub(state.scroll_state.viewport_height.saturating_sub(1));
            }
        }
    }
}

impl Component for Tree {
    type Props = TreeProps;
    type State = TreeState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveTree>(live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
        })
    }

    fn handle_event(
        &mut self,
        event: &crate::event::types::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key_event)
                if key_event.kind != crate::event::types::KeyEventKind::Release =>
            {
                self.handle_key_navigation(
                    key_event.code.clone(),
                    key_event.modifiers,
                    props,
                    state,
                )
            }
            Event::Focus(focus_event) => {
                match focus_event.kind {
                    crate::event::types::FocusEventKind::Gained => state.focused = true,
                    crate::event::types::FocusEventKind::Lost => state.focused = false,
                    _ => {}
                };
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        // Always rebuild flat tree to ensure it's current
        state.flat_nodes = self.flatten_tree(props, state);

        // Update content dimensions for scrolling
        state.scroll_state.content_height = state.flat_nodes.len() as u16;

        // Update search matches
        if let Some(search_term) = &props.search_term {
            if !search_term.is_empty() {
                state.search_matches = state
                    .flat_nodes
                    .iter()
                    .filter(|n| n.label.to_lowercase().contains(&search_term.to_lowercase()))
                    .map(|n| n.id.clone())
                    .collect();
            } else {
                state.search_matches.clear();
            }
        }

        // Sync expanded nodes with props
        state.expanded_nodes = props.expanded_nodes.clone();

        // Update visible nodes
        state.visible_nodes = state
            .flat_nodes
            .iter()
            .filter(|n| n.visible)
            .map(|n| n.id.clone())
            .collect();

        // Intelligent re-render detection for optimal tree performance

        !state.selected_nodes.is_empty() ||
            // Expanded state changed (check if expansion differs from props)
            state.expanded_nodes != props.expanded_nodes ||
            // Visible nodes list changed (indicates tree structure or expansion changes)
            state.visible_nodes.is_empty()
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self
    }
}

// Helper implementations

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> TreeNode {
        TreeNode::new("root", "Root").with_children(vec![
            TreeNode::new("folder1", "Folder 1")
                .with_icon("📁")
                .with_children(vec![
                    TreeNode::new("file1", "File 1.txt").with_icon("📄"),
                    TreeNode::new("file2", "File 2.txt").with_icon("📄"),
                ]),
            TreeNode::new("folder2", "Folder 2")
                .with_icon("📁")
                .with_children(vec![
                    TreeNode::new("subfolder1", "Subfolder 1")
                        .with_icon("📁")
                        .with_children(vec![TreeNode::new("file3", "File 3.txt").with_icon("📄")]),
                    TreeNode::new("file4", "File 4.txt").with_icon("📄"),
                ]),
            TreeNode::new("file5", "File 5.txt").with_icon("📄"),
        ])
    }

    fn create_test_props() -> TreeProps {
        TreeProps {
            root: Some(create_test_tree()),
            selectable: true,
            show_icons: true,
            show_lines: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_tree_creation() {
        let tree = Tree;
        let props = create_test_props();
        let state = TreeState::default();

        let element = tree.render(&props, &state);
        assert!(matches!(
            element.element_type,
            crate::component::ElementType::Component(_)
        ));
    }

    #[test]
    fn test_tree_flattening() {
        let tree = Tree;
        let props = create_test_props();
        let mut state = TreeState::default();

        // Initially only root should be visible
        let flat_nodes = tree.flatten_tree(&props, &state);
        assert_eq!(flat_nodes.len(), 4); // root + 2 folders + 1 file (unexpanded)

        // Expand folder1
        state.expanded_nodes.push("folder1".to_string());
        let flat_nodes = tree.flatten_tree(&props, &state);
        assert!(flat_nodes.len() > 4); // More nodes visible

        // Check levels are correct
        let folder1_node = flat_nodes.iter().find(|n| n.id == "folder1").unwrap();
        assert_eq!(folder1_node.level, 1);

        let file1_node = flat_nodes.iter().find(|n| n.id == "file1").unwrap();
        assert_eq!(file1_node.level, 2);
    }

    #[test]
    fn test_node_selection() {
        let tree = Tree;
        let props = create_test_props();
        let mut state = TreeState::default();

        tree.select_node(&props, &mut state, "folder1", false);
        assert_eq!(state.selected_nodes, vec!["folder1".to_string()]);

        // Multi-select
        let mut multi_props = props.clone();
        multi_props.multi_select = true;

        tree.select_node(&multi_props, &mut state, "folder2", true);
        assert_eq!(
            state.selected_nodes,
            vec!["folder1".to_string(), "folder2".to_string()]
        );
    }

    #[test]
    fn test_node_expansion() {
        let tree = Tree;
        let props = create_test_props();
        let mut state = TreeState::default();

        // Expand folder1
        tree.toggle_expansion(&props, &mut state, "folder1", true);
        assert!(state.expanded_nodes.contains(&"folder1".to_string()));

        // Collapse folder1
        tree.toggle_expansion(&props, &mut state, "folder1", false);
        assert!(!state.expanded_nodes.contains(&"folder1".to_string()));
    }

    #[test]
    fn test_keyboard_navigation() {
        let tree = Tree;
        let props = create_test_props();
        let mut state = TreeState::default();
        state.flat_nodes = tree.flatten_tree(&props, &state);

        // Initial state - should select first node
        let first_id = state.flat_nodes[0].id.clone();
        tree.select_node(&props, &mut state, &first_id, false);

        // Test down arrow
        let result =
            tree.handle_key_navigation(KeyCode::Down, KeyModifiers::empty(), &props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(state.selected_nodes.len(), 1);

        // Test expansion with right arrow
        let _selected_id = state.selected_nodes[0].clone();
        let result =
            tree.handle_key_navigation(KeyCode::Right, KeyModifiers::empty(), &props, &mut state);
        assert_eq!(result, EventResult::Consumed);
    }

    #[test]
    fn test_search_functionality() {
        let tree = Tree;
        let mut props = create_test_props();
        let mut state = TreeState::default();

        props.search_term = Some("File 1".to_string());

        // Update should populate search matches
        let mut tree_mut = tree;
        tree_mut.update(&props, &mut state);

        assert!(!state.search_matches.is_empty());
        assert!(state.search_matches.contains(&"file1".to_string()));
    }

    #[test]
    fn test_checkbox_functionality() {
        let tree = Tree;
        let mut props = create_test_props();
        props.checkable = true;
        let mut state = TreeState::default();

        // Check a node
        tree.toggle_check(&props, &mut state, "file1");
        assert!(state.checked_nodes.contains(&"file1".to_string()));

        // Uncheck the node
        tree.toggle_check(&props, &mut state, "file1");
        assert!(!state.checked_nodes.contains(&"file1".to_string()));
    }

    #[test]
    fn test_node_finding() {
        let root = create_test_tree();

        let found = root.find_node("file1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().label, "File 1.txt");

        let not_found = root.find_node("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_tree_builder() {
        let root = TreeNode::new("root", "Root")
            .with_icon("🏠")
            .with_style("font-bold")
            .expanded(true)
            .selected(true)
            .checked(true);

        assert_eq!(root.icon, Some("🏠".to_string()));
        assert_eq!(root.style, Some("font-bold".to_string()));
        assert!(root.expanded);
        assert!(root.selected);
        assert_eq!(root.checked, Some(true));

        let props = Tree::with_root(TreeProps::default(), root);
        let props = Tree::selectable(props, true);
        let props = Tree::checkable(props, true);
        let props = Tree::with_icons(props, true);
        let props = Tree::with_lines(props, true);

        assert!(props.selectable);
        assert!(props.checkable);
        assert!(props.show_icons);
        assert!(props.show_lines);
    }

    #[test]
    fn test_scrolling() {
        let mut state = TreeState::default();
        state.scroll_state.viewport_height = 10;
        state.scroll_state.content_height = 50;

        // Test scroll to node
        let tree = Tree;
        // Create enough nodes to require scrolling
        state.flat_nodes = (0..25)
            .map(|i| FlatTreeNode {
                id: format!("node{}", i),
                label: format!("Node {}", i),
                level: 0,
                parent_id: None,
                has_children: false,
                expanded: false,
                selected: false,
                checked: None,
                icon: None,
                style: None,
                visible: true,
                matched: false,
            })
            .collect();

        // Set viewport smaller than content
        state.scroll_state.viewport_height = 10;
        state.scroll_state.content_height = 25;

        tree.scroll_to_node(&mut state, "node20");
        // Should scroll to make node visible (node20 is at index 20, so offset should be > 0)
        assert!(state.scroll_state.offset_y > 0);
    }

    #[test]
    fn test_event_handling() {
        let mut tree = Tree;
        let mut props = create_test_props();
        let mut state = TreeState::default();
        state.flat_nodes = tree.flatten_tree(&props, &state);

        // Test key event
        let key_event = Event::Key(crate::event::types::KeyEvent::new(KeyCode::Down));
        let result = tree.handle_event(&key_event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);

        // Test focus event
        let focus_event = Event::Focus(crate::event::types::FocusEvent {
            kind: crate::event::types::FocusEventKind::Gained,
            timestamp: std::time::Instant::now(),
        });
        let result = tree.handle_event(&focus_event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert!(state.focused);
    }

    #[test]
    fn test_component_update() {
        let mut tree = Tree;
        let props = create_test_props();
        let mut state = TreeState::default();

        tree.update(&props, &mut state);

        // Should have flat nodes populated
        assert!(!state.flat_nodes.is_empty());

        // Should have content height set
        assert_eq!(
            state.scroll_state.content_height,
            state.flat_nodes.len() as u16
        );
    }
}
