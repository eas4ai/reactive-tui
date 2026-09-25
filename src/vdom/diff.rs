use super::node::{VNode, VNodeKey};
use super::patch::{Patch, PatchList};
use std::collections::HashMap;

/// Context for diffing operations
pub struct DiffContext {
    /// Path to current node for debugging
    path: Vec<String>,
    /// Statistics
    pub stats: DiffStats,
    /// Current recursion depth for stack overflow prevention
    depth: usize,
    /// Maximum allowed recursion depth
    max_depth: usize,
}

/// Statistics about the diffing process
#[derive(Debug, Default)]
pub struct DiffStats {
    /// Total number of nodes compared during diffing
    pub nodes_compared: usize,
    /// Number of patches generated from the diff
    pub patches_generated: usize,
    /// Number of nodes that were reused (no changes)
    pub nodes_reused: usize,
    /// Number of nodes that were completely replaced
    pub nodes_replaced: usize,
}

impl Default for DiffContext {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffContext {
    /// Create a new diff context
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            stats: DiffStats::default(),
            depth: 0,
            max_depth: 1000, // Reasonable limit for UI trees
        }
    }

    /// Push a path segment
    fn push_path(&mut self, segment: impl Into<String>) {
        self.path.push(segment.into());
    }

    /// Pop a path segment
    fn pop_path(&mut self) {
        self.path.pop();
    }

    /// Get current path as string
    pub fn current_path(&self) -> String {
        self.path.join(".")
    }
}

/// Diff two virtual DOM trees and generate patches
pub fn diff_vnodes(old: &VNode, new: &VNode) -> PatchList {
    let mut ctx = DiffContext::new();
    let mut patches = PatchList::new();

    diff_node(&mut ctx, &mut patches, old, new, 0);

    patches.set_stats(ctx.stats);
    patches
}

/// Diff two nodes
fn diff_node(
    ctx: &mut DiffContext,
    patches: &mut PatchList,
    old: &VNode,
    new: &VNode,
    index: usize,
) {
    ctx.stats.nodes_compared += 1;

    // Stack overflow prevention: check depth limit
    if ctx.depth >= ctx.max_depth {
        #[cfg(debug_assertions)]
        log::warn!(
            "Maximum VDOM diff depth ({}) reached; tree may be too deep or contain cycles",
            ctx.max_depth
        );

        // Replace entire subtree to avoid stack overflow
        patches.push(Patch::Replace {
            index,
            old: old.clone(),
            new: new.clone(),
        });
        ctx.stats.nodes_replaced += 1;
        ctx.stats.patches_generated += 1;
        return;
    }

    // Increment depth for this recursion level
    ctx.depth += 1;

    // Fast path: identical nodes
    if nodes_equal(old, new) {
        ctx.stats.nodes_reused += 1;
        ctx.depth -= 1;
        return;
    }

    // Check if nodes have same type and key
    if !can_patch(old, new) {
        // Complete replacement needed
        patches.push(Patch::Replace {
            index,
            old: old.clone(),
            new: new.clone(),
        });
        ctx.stats.nodes_replaced += 1;
        ctx.stats.patches_generated += 1;
        ctx.depth -= 1;
        return;
    }

    // Nodes can be patched, diff based on type
    match (old, new) {
        (VNode::Element(old_el), VNode::Element(new_el)) => {
            ctx.push_path(format!("{}[{}]", old_el.tag, index));

            // Diff attributes
            diff_attributes(ctx, patches, index, &old_el.attrs, &new_el.attrs);

            // Diff class
            if old_el.class != new_el.class {
                patches.push(Patch::SetClass {
                    index,
                    class: new_el.class.clone(),
                });
                ctx.stats.patches_generated += 1;
            }

            // Diff style
            if old_el.style != new_el.style {
                patches.push(Patch::SetStyle {
                    index,
                    style: new_el.style.clone(),
                });
                ctx.stats.patches_generated += 1;
            }

            // Diff children
            diff_children(ctx, patches, index, &old_el.children, &new_el.children);

            ctx.pop_path();
        }

        (VNode::Text(old_text), VNode::Text(new_text)) => {
            if old_text.content != new_text.content {
                patches.push(Patch::SetText {
                    index,
                    text: new_text.content.clone(),
                });
                ctx.stats.patches_generated += 1;
            }
        }

        (VNode::Component(old_comp), VNode::Component(new_comp)) => {
            ctx.push_path(format!("{}[{}]", old_comp.name, index));

            // Check if props changed
            if !std::sync::Arc::ptr_eq(&old_comp.props, &new_comp.props) {
                patches.push(Patch::UpdateProps {
                    index,
                    props: new_comp.props.clone(),
                });
                ctx.stats.patches_generated += 1;
            }

            // Diff children
            diff_children(ctx, patches, index, &old_comp.children, &new_comp.children);

            ctx.pop_path();
        }

        (VNode::Fragment(old_frag), VNode::Fragment(new_frag)) => {
            ctx.push_path(format!("Fragment[{index}]"));
            diff_children(ctx, patches, index, &old_frag.children, &new_frag.children);
            ctx.pop_path();
        }

        _ => {
            // Different types, should have been caught by can_patch
            patches.push(Patch::Replace {
                index,
                old: old.clone(),
                new: new.clone(),
            });
            ctx.stats.nodes_replaced += 1;
            ctx.stats.patches_generated += 1;
        }
    }

    // Decrement depth after processing
    ctx.depth -= 1;
}

/// Check if two nodes are equal (no patching needed)
fn nodes_equal(old: &VNode, new: &VNode) -> bool {
    match (old, new) {
        (VNode::Empty, VNode::Empty) => true,
        (VNode::Text(old_t), VNode::Text(new_t)) => {
            old_t.content == new_t.content && old_t.key == new_t.key
        }
        // For elements and components, deep equality is expensive
        // We'll let the diffing algorithm handle it
        _ => false,
    }
}

/// Check if a node can be patched (same type and key)
fn can_patch(old: &VNode, new: &VNode) -> bool {
    old.node_type() == new.node_type() && old.key() == new.key()
}

/// Diff attributes
fn diff_attributes(
    ctx: &mut DiffContext,
    patches: &mut PatchList,
    index: usize,
    old_attrs: &HashMap<String, String>,
    new_attrs: &HashMap<String, String>,
) {
    // Find removed attributes
    for key in old_attrs.keys() {
        if !new_attrs.contains_key(key) {
            patches.push(Patch::RemoveAttribute {
                index,
                name: key.clone(),
            });
            ctx.stats.patches_generated += 1;
        }
    }

    // Find added or changed attributes
    for (key, new_value) in new_attrs {
        match old_attrs.get(key) {
            Some(old_value) if old_value != new_value => {
                patches.push(Patch::SetAttribute {
                    index,
                    name: key.clone(),
                    value: new_value.clone(),
                });
                ctx.stats.patches_generated += 1;
            }
            None => {
                patches.push(Patch::SetAttribute {
                    index,
                    name: key.clone(),
                    value: new_value.clone(),
                });
                ctx.stats.patches_generated += 1;
            }
            _ => {} // Attribute unchanged
        }
    }
}

/// Diff children using a keyed algorithm
fn diff_children(
    ctx: &mut DiffContext,
    patches: &mut PatchList,
    parent_index: usize,
    old_children: &[VNode],
    new_children: &[VNode],
) {
    // Build key maps for efficient lookup
    let mut old_keyed: HashMap<VNodeKey, (usize, &VNode)> = HashMap::new();
    let mut old_unkeyed: Vec<(usize, &VNode)> = Vec::new();

    for (i, child) in old_children.iter().enumerate() {
        match child.key() {
            VNodeKey::None => old_unkeyed.push((i, child)),
            key => {
                old_keyed.insert(key.clone(), (i, child));
            }
        }
    }

    let mut new_keyed: HashMap<VNodeKey, (usize, &VNode)> = HashMap::new();
    let mut new_unkeyed: Vec<(usize, &VNode)> = Vec::new();

    for (i, child) in new_children.iter().enumerate() {
        match child.key() {
            VNodeKey::None => new_unkeyed.push((i, child)),
            key => {
                new_keyed.insert(key.clone(), (i, child));
            }
        }
    }

    // Track which old nodes have been matched
    let mut old_matched = vec![false; old_children.len()];

    // First pass: match keyed nodes
    for (key, (new_idx, new_child)) in &new_keyed {
        if let Some((old_idx, old_child)) = old_keyed.get(key) {
            // Found matching key
            old_matched[*old_idx] = true;

            if *old_idx != *new_idx {
                // Node moved
                patches.push(Patch::Move {
                    from: *old_idx,
                    to: *new_idx,
                    parent: parent_index,
                });
                ctx.stats.patches_generated += 1;
            }

            // Recursively diff the node
            diff_node(ctx, patches, old_child, new_child, *new_idx);
        } else {
            // New keyed node
            patches.push(Patch::Insert {
                index: *new_idx,
                parent: parent_index,
                node: (*new_child).clone(),
            });
            ctx.stats.patches_generated += 1;
        }
    }

    // Second pass: match unkeyed nodes by position
    let mut unkeyed_idx = 0;
    for (new_idx, new_child) in &new_unkeyed {
        // Find next unmatched old unkeyed node
        while unkeyed_idx < old_unkeyed.len() && old_matched[old_unkeyed[unkeyed_idx].0] {
            unkeyed_idx += 1;
        }

        if unkeyed_idx < old_unkeyed.len() {
            let (old_idx, old_child) = old_unkeyed[unkeyed_idx];
            old_matched[old_idx] = true;
            unkeyed_idx += 1;

            // Recursively diff the node
            diff_node(ctx, patches, old_child, new_child, *new_idx);
        } else {
            // No more old unkeyed nodes, insert new one
            patches.push(Patch::Insert {
                index: *new_idx,
                parent: parent_index,
                node: (*new_child).clone(),
            });
            ctx.stats.patches_generated += 1;
        }
    }

    // Third pass: remove unmatched old nodes
    for (i, matched) in old_matched.iter().enumerate() {
        if !matched {
            patches.push(Patch::Remove {
                index: i,
                parent: parent_index,
            });
            ctx.stats.patches_generated += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identical_nodes() {
        let node1 = VNode::text("Hello");
        let node2 = VNode::text("Hello");

        let patches = diff_vnodes(&node1, &node2);
        assert_eq!(patches.len(), 0);
        assert_eq!(patches.stats().nodes_reused, 1);
    }

    #[test]
    fn test_diff_text_change() {
        let node1 = VNode::text("Hello");
        let node2 = VNode::text("World");

        let patches = diff_vnodes(&node1, &node2);
        assert_eq!(patches.len(), 1);

        match &patches.patches()[0] {
            Patch::SetText { text, .. } => assert_eq!(text, "World"),
            _ => panic!("Expected SetText patch"),
        }
    }

    #[test]
    fn test_diff_element_attributes() {
        let node1 = VNode::element("div")
            .attr("id", "old")
            .attr("class", "container")
            .build();

        let node2 = VNode::element("div")
            .attr("id", "new")
            .attr("style", "color: red")
            .build();

        let patches = diff_vnodes(&node1, &node2);

        // Should have patches for: id change, class removal, style addition
        assert!(patches.len() >= 3);
    }

    #[test]
    fn test_diff_with_keys() {
        let node1 = VNode::element("ul")
            .children(vec![
                VNode::element("li")
                    .key("a")
                    .child(VNode::text("A"))
                    .build(),
                VNode::element("li")
                    .key("b")
                    .child(VNode::text("B"))
                    .build(),
                VNode::element("li")
                    .key("c")
                    .child(VNode::text("C"))
                    .build(),
            ])
            .build();

        let node2 = VNode::element("ul")
            .children(vec![
                VNode::element("li")
                    .key("c")
                    .child(VNode::text("C"))
                    .build(),
                VNode::element("li")
                    .key("a")
                    .child(VNode::text("A"))
                    .build(),
                VNode::element("li")
                    .key("b")
                    .child(VNode::text("B"))
                    .build(),
            ])
            .build();

        let patches = diff_vnodes(&node1, &node2);

        // Should detect moves rather than replacements
        let has_move = patches
            .patches()
            .iter()
            .any(|p| matches!(p, Patch::Move { .. }));
        assert!(has_move);
    }

    #[test]
    fn test_diff_node_replacement() {
        let node1 = VNode::element("div").build();
        let node2 = VNode::text("Hello");

        let patches = diff_vnodes(&node1, &node2);
        assert_eq!(patches.len(), 1);

        match &patches.patches()[0] {
            Patch::Replace { .. } => {}
            _ => panic!("Expected Replace patch"),
        }
    }
}
