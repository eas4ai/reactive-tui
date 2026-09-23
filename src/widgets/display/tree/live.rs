use super::*;
use crate::{
    component::{ElementType, LayoutInfo},
    event::types::{
        FocusEventKind, KeyEventKind, MouseButton, MouseEvent, MouseEventKind, WheelDelta,
    },
};
use std::collections::HashSet;
use std::sync::Mutex;

mod paint;

#[derive(Clone)]
pub(super) struct LiveProps {
    pub config: TreeProps,
    pub seed: TreeState,
}
impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.seed.selected_nodes == other.seed.selected_nodes
            && self.seed.expanded_nodes == other.seed.expanded_nodes
            && self.seed.checked_nodes == other.seed.checked_nodes
    }
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone)]
struct Row {
    node: FlatTreeNode,
    selectable: bool,
    checkable: bool,
    expandable: bool,
    loading: bool,
    branches: Vec<bool>,
    last: bool,
}
#[derive(Clone, Copy, PartialEq)]
enum Part {
    Label,
    Expand,
    Check,
}

pub(super) struct LiveTree {
    previous: TreeProps,
    seed: TreeState,
    root: Option<TreeNode>,
    viewport: Option<LayoutInfo>,
    rows: Vec<Row>,
    targets: Arc<Mutex<Vec<(String, Part, LayoutInfo)>>>,
    cursor: Option<String>,
    last_press: Option<(String, Part)>,
    loaded: HashSet<String>,
    scroll: usize,
    error: Option<String>,
}

impl LiveTree {
    fn validate(root: &Option<TreeNode>) -> Option<String> {
        let mut ids = HashSet::new();
        let mut pending: Vec<_> = root.iter().collect();
        while let Some(node) = pending.pop() {
            if !ids.insert(&node.id) {
                return Some(format!("Duplicate tree node ID: {}", node.id));
            }
            pending.extend(&node.children);
        }
        None
    }

    fn sync_authored_nodes(&self, root: &Option<TreeNode>, state: &mut TreeState) {
        let mut old = HashMap::new();
        let mut pending: Vec<_> = self.previous.root.iter().collect();
        while let Some(node) = pending.pop() {
            old.insert(&node.id, node);
            pending.extend(&node.children);
        }
        let mut pending: Vec<_> = root.iter().collect();
        while let Some(node) = pending.pop() {
            let previous = old.get(&node.id);
            for (values, value, changed) in [
                (
                    &mut state.expanded_nodes,
                    node.expanded,
                    previous.is_none_or(|old| old.expanded != node.expanded),
                ),
                (
                    &mut state.selected_nodes,
                    node.selected,
                    previous.is_none_or(|old| old.selected != node.selected),
                ),
                (
                    &mut state.checked_nodes,
                    node.checked == Some(true),
                    previous.is_none_or(|old| old.checked != node.checked),
                ),
            ] {
                if changed {
                    values.retain(|id| id != &node.id);
                    if value {
                        values.push(node.id.clone());
                    }
                }
            }
            pending.extend(&node.children);
        }
    }

    fn seed_nodes(&self, state: &mut TreeState) {
        let mut pending: Vec<_> = self.root.iter().collect();
        while let Some(node) = pending.pop() {
            if node.expanded && !state.expanded_nodes.contains(&node.id) {
                state.expanded_nodes.push(node.id.clone());
            }
            if node.selected && !state.selected_nodes.contains(&node.id) {
                state.selected_nodes.push(node.id.clone());
            }
            if node.checked == Some(true) && !state.checked_nodes.contains(&node.id) {
                state.checked_nodes.push(node.id.clone());
            }
            pending.extend(&node.children);
        }
    }

    fn rebuild(&mut self, props: &TreeProps, state: &mut TreeState) {
        self.load_expanded(props, state);
        let term = props.search_term.as_deref().unwrap_or("").to_lowercase();
        let mut all = HashSet::new();
        let mut selectable = HashSet::new();
        let mut matches = HashSet::new();
        let mut paths = HashSet::new();
        // Postorder computes matching subtrees once, without repeated recursive searches.
        let mut pending: Vec<_> = self.root.iter().map(|node| (node, false)).collect();
        while let Some((node, visited)) = pending.pop() {
            if !visited {
                all.insert(node.id.clone());
                if props.selectable && node.selectable {
                    selectable.insert(node.id.clone());
                }
                if !term.is_empty() && node.label.to_lowercase().contains(&term) {
                    matches.insert(node.id.clone());
                }
                pending.push((node, true));
                pending.extend(node.children.iter().map(|child| (child, false)));
            } else if matches.contains(&node.id)
                || node.children.iter().any(|child| paths.contains(&child.id))
            {
                paths.insert(node.id.clone());
            }
        }
        state.selected_nodes.retain(|id| all.contains(id));
        state.expanded_nodes.retain(|id| all.contains(id));
        state.checked_nodes.retain(|id| all.contains(id));
        let expanded: HashSet<_> = state.expanded_nodes.iter().collect();
        let selected: HashSet<_> = state.selected_nodes.iter().collect();
        let checked: HashSet<_> = state.checked_nodes.iter().collect();
        let mut pending: Vec<_> = self
            .root
            .iter()
            .map(|node| (node, 0usize, None, Vec::new(), true))
            .collect();
        self.rows.clear();
        while let Some((node, level, parent_id, branches, last)) = pending.pop() {
            if !term.is_empty() && props.filter_visible && !paths.contains(&node.id) {
                continue;
            }
            let is_expanded = expanded.contains(&node.id);
            let has_children =
                !node.children.is_empty() || (node.lazy && !self.loaded.contains(&node.id));
            self.rows.push(Row {
                node: FlatTreeNode {
                    id: node.id.clone(),
                    label: node.label.clone(),
                    level,
                    parent_id,
                    has_children,
                    expanded: is_expanded,
                    selected: selected.contains(&node.id),
                    checked: (props.checkable || node.checkable || node.checked.is_some())
                        .then_some(checked.contains(&node.id)),
                    icon: node.icon.clone(),
                    style: node.style.clone(),
                    visible: true,
                    matched: matches.contains(&node.id),
                },
                selectable: props.selectable && node.selectable,
                checkable: props.checkable || node.checkable,
                expandable: node.expandable && has_children,
                loading: node.loading,
                branches: branches.clone(),
                last,
            });
            if is_expanded || !term.is_empty() {
                let visible: Vec<_> = node
                    .children
                    .iter()
                    .filter(|child| {
                        term.is_empty() || !props.filter_visible || paths.contains(&child.id)
                    })
                    .collect();
                let mut branches = branches;
                branches.push(!last);
                pending.extend(visible.iter().enumerate().rev().map(|(index, child)| {
                    (
                        *child,
                        level.saturating_add(1),
                        Some(node.id.clone()),
                        branches.clone(),
                        index + 1 == visible.len(),
                    )
                }));
            }
        }
        state.selected_nodes.retain(|id| selectable.contains(id));
        if !props.multi_select {
            state.selected_nodes.truncate(1);
        }
        if self
            .cursor
            .as_ref()
            .is_some_and(|id| !self.rows.iter().any(|row| &row.node.id == id))
        {
            self.cursor = self.rows.first().map(|row| row.node.id.clone());
        }
        state.flat_nodes = self.rows.iter().map(|row| row.node.clone()).collect();
        state.visible_nodes = self.rows.iter().map(|row| row.node.id.clone()).collect();
        state.search_matches = self
            .rows
            .iter()
            .filter(|row| row.node.matched)
            .map(|row| row.node.id.clone())
            .collect();
        self.clamp(props, state);
    }

    fn clamp(&mut self, props: &TreeProps, state: &mut TreeState) {
        let (width, height) = self
            .viewport
            .map_or((0.0, 0.0), |layout| layout.content_size());
        let height = height.max(0.0) as u16;
        state.scroll_state.viewport_height = height;
        state.scroll_state.viewport_width = width.max(0.0) as u16;
        state.scroll_state.content_height = self.rows.len().min(u16::MAX as usize) as u16;
        self.scroll = if props.scrollable {
            self.scroll
                .min(self.rows.len().saturating_sub(height as usize))
        } else {
            0
        };
        state.scroll_state.offset_y = self.scroll.min(u16::MAX as usize) as u16;
        let content_width = self
            .rows
            .iter()
            .map(|row| Self::row_width(row, props))
            .max()
            .unwrap_or(0);
        state.scroll_state.content_width = content_width.min(u16::MAX as usize) as u16;
        state.scroll_state.offset_x = if props.scrollable {
            state.scroll_state.offset_x.min(
                state
                    .scroll_state
                    .content_width
                    .saturating_sub(state.scroll_state.viewport_width),
            )
        } else {
            0
        };
    }

    fn reveal(&mut self, props: &TreeProps, state: &mut TreeState) {
        if props.scrollable {
            if let Some(index) = self
                .cursor
                .as_ref()
                .and_then(|id| self.rows.iter().position(|row| &row.node.id == id))
            {
                let height = state.scroll_state.viewport_height.max(1) as usize;
                if index < self.scroll {
                    self.scroll = index;
                } else if index >= self.scroll.saturating_add(height) {
                    self.scroll = index.saturating_add(1).saturating_sub(height);
                }
            }
        }
        self.clamp(props, state);
    }

    fn select(
        &mut self,
        props: &TreeProps,
        state: &mut TreeState,
        id: &str,
        extend: bool,
        toggle: bool,
    ) {
        self.cursor = Some(id.to_string());
        if !self
            .rows
            .iter()
            .any(|row| row.node.id == id && row.selectable)
        {
            return;
        }
        let previous = state.selected_nodes.clone();
        if props.multi_select && toggle {
            if state.selected_nodes.iter().any(|node| node == id) {
                state.selected_nodes.retain(|node| node != id);
            } else {
                state.selected_nodes.push(id.to_string());
            }
        } else if props.multi_select && extend {
            if !state.selected_nodes.iter().any(|node| node == id) {
                state.selected_nodes.push(id.to_string());
            }
        } else {
            state.selected_nodes = vec![id.to_string()];
        }
        if previous != state.selected_nodes {
            if props.multi_select {
                if let Some(callback) = &props.on_multi_select {
                    callback(state.selected_nodes.clone());
                }
            } else if let Some(callback) = &props.on_select {
                callback(Some(id.to_string()));
            }
        }
        self.rebuild(props, state);
        self.reveal(props, state);
    }

    fn load(&mut self, props: &TreeProps, id: &str) -> bool {
        let lazy = self
            .root
            .as_ref()
            .and_then(|root| root.find_node(id))
            .is_some_and(|node| node.lazy);
        if !lazy || !props.lazy_loading || self.loaded.contains(id) {
            return true;
        }
        let Some(callback) = &props.on_load_children else {
            self.error = Some(format!("Tree node {id} needs a lazy-load callback"));
            return false;
        };
        let children = callback(id.to_string());
        let mut candidate = self.root.clone();
        if let Some(node) = candidate.as_mut().and_then(|root| root.find_node_mut(id)) {
            node.children = children;
            node.loading = false;
        }
        if let Some(error) = Self::validate(&candidate) {
            self.error = Some(error);
            return false;
        }
        self.root = candidate;
        self.loaded.insert(id.to_string());
        true
    }

    fn load_expanded(&mut self, props: &TreeProps, state: &TreeState) {
        if self.error.is_some() || !props.lazy_loading {
            return;
        }
        let expanded: HashSet<_> = state.expanded_nodes.iter().collect();
        let mut pending: Vec<_> = self.root.iter().collect();
        let mut ids = Vec::new();
        while let Some(node) = pending.pop() {
            if expanded.contains(&node.id) {
                if node.lazy && !self.loaded.contains(&node.id) {
                    ids.push(node.id.clone());
                }
                pending.extend(&node.children);
            }
        }
        for id in ids {
            if !self.load(props, &id) {
                break;
            }
        }
    }

    fn expand(&mut self, props: &TreeProps, state: &mut TreeState, id: &str, expand: bool) {
        if !self
            .rows
            .iter()
            .any(|row| row.node.id == id && row.expandable)
        {
            return;
        }
        if state.expanded_nodes.iter().any(|node| node == id) == expand {
            return;
        }
        if expand {
            if !self.load(props, id) {
                return;
            }
            state.expanded_nodes.push(id.to_string());
        } else {
            state.expanded_nodes.retain(|node| node != id);
        }
        if let Some(callback) = &props.on_expand {
            callback(id.to_string(), expand);
        }
        self.rebuild(props, state);
    }

    fn check(&mut self, props: &TreeProps, state: &mut TreeState, id: &str) {
        if !self
            .rows
            .iter()
            .any(|row| row.node.id == id && row.checkable)
        {
            return;
        }
        let value = !state.checked_nodes.iter().any(|node| node == id);
        if value {
            state.checked_nodes.push(id.to_string());
        } else {
            state.checked_nodes.retain(|node| node != id);
        }
        if let Some(callback) = &props.on_check {
            callback(id.to_string(), value);
        }
        self.rebuild(props, state);
    }

    fn target(&self, mouse: &MouseEvent) -> Option<(String, Part)> {
        let root = self.viewport?;
        let [a, b, c, d, tx, ty] = root.transform;
        let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
        let (x, y) = (a * x + c * y + tx, b * x + d * y + ty);
        self.targets
            .lock()
            .unwrap()
            .iter()
            .find(|(_, _, layout)| {
                let clip = layout.clip;
                x >= clip.x
                    && y >= clip.y
                    && x < clip.x + clip.width
                    && y < clip.y + clip.height
                    && layout.local_cell(x, y).is_some()
            })
            .map(|(id, part, _)| (id.clone(), *part))
    }

    fn move_node(&mut self, props: &TreeProps, state: &mut TreeState, source: &str, target: &str) {
        let Some(root) = self.root.as_ref() else {
            return;
        };
        let valid = source != root.id
            && source != target
            && root
                .find_node(source)
                .is_some_and(|node| node.find_node(target).is_none())
            && root.find_node(target).is_some_and(|node| {
                node.expandable && (!node.lazy || self.loaded.contains(&node.id))
            });
        if !valid {
            return;
        }
        fn remove(root: &mut TreeNode, id: &str) -> Option<TreeNode> {
            if let Some(index) = root.children.iter().position(|node| node.id == id) {
                return Some(root.children.remove(index));
            }
            root.children.iter_mut().find_map(|child| remove(child, id))
        }
        if let Some(root) = &mut self.root {
            if let Some(node) = remove(root, source) {
                // Both IDs were validated against the same owned tree above.
                root.find_node_mut(target)
                    .expect("validated tree drop target")
                    .children
                    .push(node);
            }
        }
        if !state.expanded_nodes.iter().any(|id| id == target) {
            state.expanded_nodes.push(target.to_string());
        }
        if let Some(callback) = &props.on_node_action {
            callback(source.to_string(), &format!("drop:{target}"));
        }
        self.rebuild(props, state);
    }

    fn key(
        &mut self,
        key: &crate::event::types::KeyEvent,
        props: &TreeProps,
        state: &mut TreeState,
    ) -> EventResult {
        if key.kind == KeyEventKind::Release || self.rows.is_empty() {
            return EventResult::Ignored;
        }
        let current = self
            .cursor
            .as_ref()
            .and_then(|id| self.rows.iter().position(|row| &row.node.id == id));
        let index = current.unwrap_or(0);
        let id = self.rows[index].node.id.clone();
        let step = state.scroll_state.viewport_height.max(1) as usize;
        let destination = match key.code {
            KeyCode::Down => Some(current.map_or(0, |i| (i + 1).min(self.rows.len() - 1))),
            KeyCode::Up => Some(index.saturating_sub(1)),
            KeyCode::Home => Some(0),
            KeyCode::End => Some(self.rows.len() - 1),
            KeyCode::PageDown => Some(index.saturating_add(step).min(self.rows.len() - 1)),
            KeyCode::PageUp => Some(index.saturating_sub(step)),
            _ => None,
        };
        if let Some(mut destination) = destination {
            let backward = matches!(key.code, KeyCode::Up | KeyCode::End | KeyCode::PageUp);
            while !self.rows[destination].selectable {
                let next = if backward {
                    destination.checked_sub(1)
                } else {
                    destination.checked_add(1).filter(|&i| i < self.rows.len())
                };
                let Some(next) = next else {
                    return EventResult::Consumed;
                };
                destination = next;
            }
            let id = self.rows[destination].node.id.clone();
            self.select(props, state, &id, key.modifiers.shift, false);
            return EventResult::Consumed;
        }
        match key.code {
            KeyCode::Left if !key.modifiers.shift => {
                if self.rows[index].node.expanded {
                    self.expand(props, state, &id, false);
                } else if let Some(parent) = self.rows[index].node.parent_id.clone() {
                    self.select(props, state, &parent, false, false);
                }
            }
            KeyCode::Right if !key.modifiers.shift => {
                if !self.rows[index].node.expanded {
                    self.expand(props, state, &id, true);
                } else if let Some(child) = self
                    .rows
                    .get(index + 1)
                    .filter(|row| row.node.parent_id.as_deref() == Some(&id))
                    .map(|row| row.node.id.clone())
                {
                    self.select(props, state, &child, false, false);
                }
            }
            KeyCode::Left | KeyCode::Right if props.scrollable => {
                state.scroll_state.offset_x = if key.code == KeyCode::Left {
                    state.scroll_state.offset_x.saturating_sub(3)
                } else {
                    state.scroll_state.offset_x.saturating_add(3)
                };
                self.clamp(props, state);
            }
            KeyCode::Enter => {
                let expand = !self.rows[index].node.expanded;
                self.expand(props, state, &id, expand);
                if let Some(callback) = &props.on_node_action {
                    callback(id, "activate");
                }
            }
            KeyCode::Char(' ') => {
                if self.rows[index].checkable {
                    self.check(props, state, &id);
                } else if props.multi_select {
                    self.select(props, state, &id, false, true);
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => self.expand(props, state, &id, true),
            KeyCode::Char('-') => self.expand(props, state, &id, false),
            KeyCode::Char('*') => {
                let level = self.rows[index].node.level;
                let ids: Vec<_> = self
                    .rows
                    .iter()
                    .filter(|row| row.node.level == level && row.expandable)
                    .map(|row| row.node.id.clone())
                    .collect();
                for id in ids {
                    self.expand(props, state, &id, true);
                }
            }
            KeyCode::Char('a') if key.modifiers.ctrl && props.multi_select => {
                let selected: Vec<_> = self
                    .rows
                    .iter()
                    .filter(|row| row.selectable)
                    .map(|row| row.node.id.clone())
                    .collect();
                if selected != state.selected_nodes {
                    state.selected_nodes = selected;
                    if let Some(callback) = &props.on_multi_select {
                        callback(state.selected_nodes.clone());
                    }
                    self.rebuild(props, state);
                }
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
}

impl Component for LiveTree {
    type Props = LiveProps;
    type State = TreeState;
    fn new(props: Self::Props) -> Self {
        Self {
            root: props.config.root.clone(),
            error: Self::validate(&props.config.root),
            previous: props.config,
            seed: props.seed,
            viewport: None,
            rows: Vec::new(),
            targets: Arc::default(),
            cursor: None,
            last_press: None,
            loaded: HashSet::new(),
            scroll: 0,
        }
    }
    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        let mut state = props.seed.clone();
        self.seed_nodes(&mut state);
        if let Some(id) = &props.config.selected_node {
            state.selected_nodes = vec![id.clone()];
        }
        self.cursor = state.selected_nodes.first().cloned();
        self.scroll = state.scroll_state.offset_y as usize;
        self.rebuild(&props.config, &mut state);
        state
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        let config = &props.config;
        if self.previous.root != config.root {
            self.sync_authored_nodes(&config.root, state);
            self.root = config.root.clone();
            self.loaded.clear();
            self.error = Self::validate(&self.root);
            state.drag_source = None;
            state.drop_target = None;
        }
        if self.seed.selected_nodes != props.seed.selected_nodes {
            state.selected_nodes = props.seed.selected_nodes.clone();
            self.cursor = state.selected_nodes.first().cloned();
        }
        if self.seed.checked_nodes != props.seed.checked_nodes {
            state.checked_nodes = props.seed.checked_nodes.clone();
        }
        if self.previous.selected_node != config.selected_node {
            state.selected_nodes = config.selected_node.iter().cloned().collect();
            self.cursor = config.selected_node.clone();
        }
        if self.previous.expanded_nodes != config.expanded_nodes {
            state.expanded_nodes = config.expanded_nodes.clone();
        }
        let callback_changed = match (&self.previous.on_load_children, &config.on_load_children) {
            (Some(old), Some(new)) => !Arc::ptr_eq(old, new),
            (None, None) => false,
            _ => true,
        };
        if callback_changed {
            self.error = Self::validate(&self.root);
        }
        let reveal = self.previous.selected_node != config.selected_node;
        if self.previous.search_term != config.search_term
            || self.previous.filter_visible != config.filter_visible
        {
            self.scroll = 0;
        }
        if !config.drag_drop {
            state.drag_source = None;
            state.drop_target = None;
        }
        self.previous = config.clone();
        self.seed = props.seed.clone();
        self.rebuild(config, state);
        if reveal {
            self.reveal(config, state);
        }
        true
    }
    fn layout(
        &mut self,
        layout: LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        let initial = self.viewport.is_none();
        let changed = self.viewport != Some(layout);
        self.viewport = Some(layout);
        self.clamp(&props.config, state);
        if initial {
            self.reveal(&props.config, state);
        }
        changed
    }
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.paint(&props.config, state)
    }
    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        let props = &props.config;
        if self.error.is_some() {
            return EventResult::Ignored;
        }
        match event {
            Event::Custom(event)
                if matches!(
                    event.name.as_str(),
                    "reactive_tui.tree.focus"
                        | "reactive_tui.tree.activate"
                        | "reactive_tui.tree.check"
                ) =>
            {
                let Some(id) = std::str::from_utf8(&event.data).ok().filter(|id| {
                    self.rows.iter().any(|row| {
                        row.node.id == *id && (row.selectable || row.checkable || row.expandable)
                    })
                }) else {
                    return EventResult::Ignored;
                };
                self.cursor = Some(id.to_string());
                state.focused = true;
                self.reveal(props, state);
                if event.name == "reactive_tui.tree.check" {
                    self.check(props, state, id);
                    EventResult::Consumed
                } else if event.name == "reactive_tui.tree.activate" {
                    self.key(
                        &crate::event::types::KeyEvent::new(KeyCode::Enter),
                        props,
                        state,
                    )
                } else {
                    EventResult::Consumed
                }
            }
            Event::Key(key) => {
                self.last_press = None;
                self.key(key, props, state)
            }
            Event::Focus(focus) => {
                state.focused = focus.kind == FocusEventKind::Gained;
                if !state.focused {
                    self.last_press = None;
                    state.drag_source = None;
                    state.drop_target = None;
                }
                EventResult::Consumed
            }
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Wheel && props.scrollable {
                    if let Some(wheel) = &mouse.wheel {
                        let (x, y) = match wheel.delta {
                            WheelDelta::Lines { x, y } | WheelDelta::Pixels { x, y } => (x, y),
                        };
                        if mouse.modifiers.shift {
                            state.scroll_state.offset_x =
                                (state.scroll_state.offset_x as f32 + x + y)
                                    .max(0.0)
                                    .min(u16::MAX as f32) as u16;
                        } else {
                            self.scroll = (self.scroll as f64 + y as f64)
                                .max(0.0)
                                .min(usize::MAX as f64)
                                as usize;
                            state.scroll_state.offset_x = (state.scroll_state.offset_x as f32 + x)
                                .max(0.0)
                                .min(u16::MAX as f32)
                                as u16;
                        }
                        self.clamp(props, state);
                        return EventResult::Consumed;
                    }
                    return EventResult::Ignored;
                }
                if mouse.kind == MouseEventKind::Up {
                    if let Some(source) = state.drag_source.take() {
                        if let Some((target, _)) =
                            self.target(mouse).filter(|_| state.drop_target.is_some())
                        {
                            self.move_node(props, state, &source, &target);
                        }
                        state.drop_target = None;
                        return EventResult::Consumed;
                    }
                    return EventResult::Ignored;
                }
                if mouse.kind == MouseEventKind::Leave {
                    state.hover_node = None;
                    state.drop_target = None;
                    return EventResult::Consumed;
                }
                let Some((id, part)) = self.target(mouse) else {
                    return EventResult::Ignored;
                };
                if mouse.kind == MouseEventKind::Click
                    && self.last_press.take() == Some((id.clone(), part))
                {
                    return EventResult::Consumed;
                }
                if mouse.kind == MouseEventKind::Down {
                    self.last_press = Some((id.clone(), part));
                }
                match mouse.kind {
                    MouseEventKind::Move | MouseEventKind::Enter => {
                        state.hover_node = Some(id);
                    }
                    MouseEventKind::Drag
                        if props.drag_drop && mouse.button == MouseButton::Left =>
                    {
                        if state.drag_source.is_some() {
                            state.drop_target = Some(id);
                        }
                    }
                    MouseEventKind::Down | MouseEventKind::Click | MouseEventKind::DoubleClick
                        if mouse.button == MouseButton::Left =>
                    {
                        state.drag_source = if props.drag_drop
                            && mouse.kind == MouseEventKind::Down
                            && matches!(part, Part::Label)
                        {
                            Some(id.clone())
                        } else {
                            None
                        };
                        state.drop_target = None;
                        self.cursor = Some(id.clone());
                        match part {
                            Part::Expand => {
                                let expanded = state.expanded_nodes.contains(&id);
                                self.expand(props, state, &id, !expanded);
                            }
                            Part::Check => self.check(props, state, &id),
                            Part::Label => {
                                self.select(
                                    props,
                                    state,
                                    &id,
                                    mouse.modifiers.shift,
                                    mouse.modifiers.ctrl,
                                );
                                if mouse.kind == MouseEventKind::DoubleClick {
                                    let expanded = state.expanded_nodes.contains(&id);
                                    self.expand(props, state, &id, !expanded);
                                    if let Some(callback) = &props.on_node_action {
                                        callback(id, "activate");
                                    }
                                }
                            }
                        }
                    }
                    _ => return EventResult::Ignored,
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}
