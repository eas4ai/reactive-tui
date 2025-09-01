use super::{Border, ScrollState};
use crate::component::{Component, Element, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use std::collections::HashMap;
use std::sync::Arc;

type NodeActionCallback = dyn Fn(String, &str) + Send + Sync;

/// Result of hit testing on tree elements
#[derive(Debug, Clone, PartialEq)]
enum TreeHitResult {
    /// Hit a tree node
    Node(String),
    /// Hit an expander/collapse button
    Expander(String),
    /// Hit outside the tree
    #[allow(dead_code)]
    Outside,
}

/// Props for the Tree component
#[derive(Clone)]
pub struct TreeProps {
    pub root: Option<TreeNode>,
    pub selected_node: Option<String>,
    pub expanded_nodes: Vec<String>,
    pub selectable: bool,
    pub multi_select: bool,
    pub show_icons: bool,
    pub show_lines: bool,
    pub indent_size: u16,
    pub lazy_loading: bool,
    pub checkable: bool,
    pub drag_drop: bool,
    pub search_term: Option<String>,
    pub filter_visible: bool,
    pub border: Border,
    pub node_style: Option<String>,
    pub selected_style: Option<String>,
    pub expanded_style: Option<String>,
    pub leaf_style: Option<String>,
    pub line_style: Option<String>,
    pub scrollable: bool,
    pub max_height: Option<u16>,
    pub virtual_scrolling: bool,
    pub on_select: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    pub on_multi_select: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
    pub on_expand: Option<Arc<dyn Fn(String, bool) + Send + Sync>>,
    pub on_check: Option<Arc<dyn Fn(String, bool) + Send + Sync>>,
    pub on_node_action: Option<Arc<NodeActionCallback>>,
    pub on_load_children: Option<Arc<dyn Fn(String) -> Vec<TreeNode> + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub selected: bool,
    pub checked: Option<bool>,
    pub icon: Option<String>,
    pub style: Option<String>,
    pub data: HashMap<String, String>,
    pub selectable: bool,
    pub checkable: bool,
    pub expandable: bool,
    pub lazy: bool,
    pub loading: bool,
    pub level: usize,
    pub parent_id: Option<String>,
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
    pub selected_nodes: Vec<String>,
    pub expanded_nodes: Vec<String>,
    pub checked_nodes: Vec<String>,
    pub scroll_state: ScrollState,
    pub focused: bool,
    pub hover_node: Option<String>,
    pub flat_nodes: Vec<FlatTreeNode>,
    pub visible_nodes: Vec<String>,
    pub search_matches: Vec<String>,
    pub loading_nodes: Vec<String>,
    pub drag_source: Option<String>,
    pub drop_target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FlatTreeNode {
    pub id: String,
    pub label: String,
    pub level: usize,
    pub parent_id: Option<String>,
    pub has_children: bool,
    pub expanded: bool,
    pub selected: bool,
    pub checked: Option<bool>,
    pub icon: Option<String>,
    pub style: Option<String>,
    pub visible: bool,
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

    fn render_node(&self, node: &FlatTreeNode, props: &TreeProps, state: &TreeState) -> Element {
        let mut node_class = String::new();

        // Apply node styling
        if node.selected {
            if let Some(style) = &props.selected_style {
                node_class.push_str(style);
            }
        } else if let Some(style) = &props.node_style {
            node_class.push(' ');
            node_class.push_str(style);
        }

        // Custom node style
        if let Some(style) = &node.style {
            node_class.push(' ');
            node_class.push_str(style);
        }

        let mut content = String::new();

        // Indentation
        for _ in 0..node.level {
            for _ in 0..props.indent_size {
                content.push(' ');
            }
        }

        // Tree lines
        if props.show_lines && node.level > 0 {
            if node.has_children {
                if node.expanded {
                    content.push_str("┬─ ");
                } else {
                    content.push_str("├─ ");
                }
            } else {
                content.push_str("└─ ");
            }
        }

        // Expansion indicator
        if node.has_children {
            if node.expanded {
                content.push_str("▼ ");
            } else {
                content.push_str("▶ ");
            }
        } else {
            content.push_str("  ");
        }

        // Checkbox
        if props.checkable {
            if let Some(checked) = node.checked {
                if checked {
                    content.push_str("☑ ");
                } else {
                    content.push_str("☐ ");
                }
            } else {
                content.push_str("☐ ");
            }
        }

        // Icon
        if props.show_icons {
            if let Some(icon) = &node.icon {
                content.push_str(icon);
                content.push(' ');
            } else if node.has_children {
                content.push_str("📁 ");
            } else {
                content.push_str("📄 ");
            }
        }

        // Label
        content.push_str(&node.label);

        // Loading indicator
        if state.loading_nodes.contains(&node.id) {
            content.push_str(" ⟳");
        }

        Element::text(&content)
            .with_class(&node_class)
            .with_key(format!("node-{}", node.id))
    }

    /// Perform hit testing to determine what tree element was clicked
    fn hit_test_tree(
        &self,
        position: crate::event::types::Position,
        props: &TreeProps,
        state: &TreeState,
    ) -> Option<TreeHitResult> {
        // Convert position to coordinates
        let (x, y) = match position {
            crate::event::types::Position::Cell { x, y } => (x as usize, y as usize),
            crate::event::types::Position::Pixel { x, y } => {
                // Convert pixel to cell coordinates (approximate)
                (x as usize / 8, y as usize / 16)
            }
        };

        // Get the root node
        let root = props.root.as_ref()?;

        // Traverse the visible tree structure to find what was clicked
        let mut current_row = 0;
        self.hit_test_node(root, x, y, &mut current_row, 0, props, state)
    }

    /// Recursively test hit on tree nodes
    fn hit_test_node(
        &self,
        node: &TreeNode,
        click_x: usize,
        click_y: usize,
        current_row: &mut usize,
        level: usize,
        props: &TreeProps,
        state: &TreeState,
    ) -> Option<TreeHitResult> {
        // Check if this row matches the click
        if *current_row == click_y {
            let indent = level * props.indent_size as usize;

            // Check if click is on the expander (first few characters)
            if !node.children.is_empty() && click_x >= indent && click_x < indent + 2 {
                return Some(TreeHitResult::Expander(node.id.clone()));
            }

            // Check if click is on the node content
            if click_x >= indent + 2 {
                return Some(TreeHitResult::Node(node.id.clone()));
            }
        }

        *current_row += 1;

        // If node is expanded, check children
        if state.expanded_nodes.iter().any(|id| id == &node.id) {
            for child in &node.children {
                if let Some(result) = self.hit_test_node(
                    child,
                    click_x,
                    click_y,
                    current_row,
                    level + 1,
                    props,
                    state,
                ) {
                    return Some(result);
                }
            }
        }

        None
    }
}

impl Component for Tree {
    type Props = TreeProps;
    type State = TreeState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        // Hook functionality would be handled by runtime

        // Flatten tree structure
        let flat_nodes = self.flatten_tree(props, state);

        // Calculate visible nodes (for virtual scrolling)
        let start_index = state.scroll_state.offset_y as usize;
        let end_index = if props.virtual_scrolling {
            (start_index + state.scroll_state.viewport_height as usize).min(flat_nodes.len())
        } else {
            flat_nodes.len()
        };

        let mut children = Vec::new();

        // Render visible nodes
        for (_i, node) in flat_nodes
            .iter()
            .enumerate()
            .skip(start_index)
            .take(end_index - start_index)
        {
            if node.visible || !props.filter_visible {
                children.push(self.render_node(node, props, state));
            }
        }

        // Empty state
        if children.is_empty() {
            if let Some(search_term) = &props.search_term {
                if !search_term.is_empty() {
                    children.push(
                        Element::text("No matching nodes found")
                            .with_class("text-muted text-center")
                            .with_key("empty-search"),
                    );
                }
            } else {
                children.push(
                    Element::text("No data")
                        .with_class("text-muted text-center")
                        .with_key("empty"),
                );
            }
        }

        let container_class = if props.border.enabled { "border" } else { "" };

        Element::layout(LayoutType::Flex)
            .with_class(format!("flex-col {container_class}"))
            .with_children(children)
            .with_key("tree-container")
    }

    fn handle_event(
        &mut self,
        event: &crate::event::types::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key_event) => self.handle_key_navigation(
                key_event.code.clone(),
                key_event.modifiers,
                props,
                state,
            ),
            Event::Mouse(mouse_event) => {
                match mouse_event {
                    MouseEvent {
                        kind: MouseEventKind::Click,
                        position,
                        ..
                    } => {
                        // Handle node selection and expansion with proper hit testing
                        if let Some(hit_result) = self.hit_test_tree(*position, props, state) {
                            match hit_result {
                                TreeHitResult::Node(node_id) => {
                                    // Select the node (clear previous selection and add this one)
                                    state.selected_nodes.clear();
                                    state.selected_nodes.push(node_id.clone());
                                    EventResult::Consumed
                                }
                                TreeHitResult::Expander(node_id) => {
                                    // Toggle expansion
                                    if let Some(pos) =
                                        state.expanded_nodes.iter().position(|x| x == &node_id)
                                    {
                                        state.expanded_nodes.remove(pos);
                                    } else {
                                        state.expanded_nodes.push(node_id);
                                    }
                                    EventResult::Consumed
                                }
                                TreeHitResult::Outside => EventResult::Ignored,
                            }
                        } else {
                            EventResult::Ignored
                        }
                    }
                    MouseEvent {
                        kind: MouseEventKind::DoubleClick,
                        ..
                    } => {
                        // Handle expansion on double-click
                        EventResult::Consumed
                    }
                    MouseEvent {
                        kind: MouseEventKind::Wheel,
                        ..
                    } => {
                        if props.scrollable {
                            // Simplified wheel handling
                            state.scroll_state.scroll_down(3);
                            EventResult::Consumed
                        } else {
                            EventResult::Ignored
                        }
                    }
                    _ => EventResult::Ignored,
                }
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
impl TreeNode {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            children: Vec::new(),
            expanded: false,
            selected: false,
            checked: None,
            icon: None,
            style: None,
            data: HashMap::new(),
            selectable: true,
            checkable: true,
            expandable: true,
            lazy: false,
            loading: false,
            level: 0,
            parent_id: None,
        }
    }

    pub fn with_children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn lazy(mut self, lazy: bool) -> Self {
        self.lazy = lazy;
        self
    }

    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

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
        let tree = Tree::default();
        let props = create_test_props();
        let state = TreeState::default();

        let element = tree.render(&props, &state);
        // Tree renders as a flex layout container
        assert_eq!(
            element.element_type,
            crate::component::ElementType::Layout(crate::component::LayoutType::Flex)
        );
    }

    #[test]
    fn test_tree_flattening() {
        let tree = Tree::default();
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
        let tree = Tree::default();
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
        let tree = Tree::default();
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
        let tree = Tree::default();
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
        let tree = Tree::default();
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
        let tree = Tree::default();
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
        let tree = Tree::default();
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
        let mut tree = Tree::default();
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
        let mut tree = Tree::default();
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
