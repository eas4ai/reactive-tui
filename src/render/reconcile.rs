use super::tree::{NodeKey, RenderNode, RenderTree};
use std::collections::HashMap;

/// Result of diffing two render trees
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub patches: Vec<PatchOp>,
    pub reused_nodes: usize,
    pub new_nodes: usize,
    pub removed_nodes: usize,
}

/// Patch operation to transform old tree into new tree
#[derive(Debug, Clone)]
pub enum PatchOp {
    /// Insert a new node
    Insert {
        parent_key: Option<NodeKey>,
        index: usize,
        node_key: NodeKey,
    },

    /// Remove a node
    Remove { node_key: NodeKey },

    /// Replace a node with another
    Replace { old_key: NodeKey, new_key: NodeKey },

    /// Move a node to a different position
    Move {
        node_key: NodeKey,
        parent_key: Option<NodeKey>,
        index: usize,
    },

    /// Update node properties
    Update { node_key: NodeKey },

    /// Reorder children
    ReorderChildren {
        parent_key: NodeKey,
        new_order: Vec<NodeKey>,
    },
}

/// Reconciler for efficient tree diffing
pub struct Reconciler {
    /// Statistics
    stats: ReconcilerStats,
}

#[derive(Debug, Default)]
struct ReconcilerStats {
    total_diffs: usize,
    cache_hits: usize,
    cache_misses: usize,
}

impl Reconciler {
    pub fn new() -> Self {
        Self {
            stats: ReconcilerStats::default(),
        }
    }

    /// Diff two render trees and produce patch operations
    pub fn diff(&mut self, old_tree: &RenderTree, new_tree: &RenderTree) -> DiffResult {
        self.stats.total_diffs += 1;

        let mut patches = Vec::new();
        let mut reused_nodes = 0;
        let mut new_nodes = 0;
        let mut removed_nodes = 0;

        match (old_tree.root(), new_tree.root()) {
            (Some(old_root), Some(new_root)) => {
                self.diff_nodes(
                    old_root,
                    new_root,
                    None,
                    0,
                    &mut patches,
                    &mut reused_nodes,
                    &mut new_nodes,
                    &mut removed_nodes,
                );
            }
            (None, Some(new_root)) => {
                // Tree was empty, insert everything
                patches.push(PatchOp::Insert {
                    parent_key: None,
                    index: 0,
                    node_key: new_root.key().clone(),
                });
                new_nodes += self.count_nodes(new_root);
            }
            (Some(old_root), None) => {
                // Tree is now empty, remove everything
                patches.push(PatchOp::Remove {
                    node_key: old_root.key().clone(),
                });
                removed_nodes += self.count_nodes(old_root);
            }
            (None, None) => {
                // Both empty, nothing to do
            }
        }

        DiffResult {
            patches,
            reused_nodes,
            new_nodes,
            removed_nodes,
        }
    }

    /// Diff two nodes recursively
    #[allow(clippy::too_many_arguments)]
    fn diff_nodes(
        &mut self,
        old_node: &dyn RenderNode,
        new_node: &dyn RenderNode,
        _parent_key: Option<NodeKey>,
        _index: usize,
        patches: &mut Vec<PatchOp>,
        reused: &mut usize,
        added: &mut usize,
        removed: &mut usize,
    ) {
        // Check if nodes can be reused
        if old_node.key() == new_node.key() && old_node.node_type() == new_node.node_type() {
            // Same node, check if it needs updating
            if !old_node.equals(new_node) {
                patches.push(PatchOp::Update {
                    node_key: new_node.key().clone(),
                });
            }

            *reused += 1;

            // Diff children
            self.diff_children(
                old_node.children(),
                new_node.children(),
                new_node.key().clone(),
                patches,
                reused,
                added,
                removed,
            );
        } else {
            // Different nodes, replace
            patches.push(PatchOp::Replace {
                old_key: old_node.key().clone(),
                new_key: new_node.key().clone(),
            });

            *removed += self.count_nodes(old_node);
            *added += self.count_nodes(new_node);
        }
    }

    /// Diff children using a key-based algorithm
    #[allow(clippy::too_many_arguments)]
    fn diff_children(
        &mut self,
        old_children: &[Box<dyn RenderNode>],
        new_children: &[Box<dyn RenderNode>],
        parent_key: NodeKey,
        patches: &mut Vec<PatchOp>,
        reused: &mut usize,
        added: &mut usize,
        removed: &mut usize,
    ) {
        // Build key maps for efficient lookup
        let old_keys: HashMap<_, _> = old_children
            .iter()
            .enumerate()
            .map(|(i, child)| (child.key().clone(), i))
            .collect();

        let _new_keys: HashMap<_, _> = new_children
            .iter()
            .enumerate()
            .map(|(i, child)| (child.key().clone(), i))
            .collect();

        // Track which old nodes have been matched
        let mut matched_old = vec![false; old_children.len()];
        let mut moves = Vec::new();

        // Process new children
        for (new_idx, new_child) in new_children.iter().enumerate() {
            let new_key = new_child.key();

            if let Some(&old_idx) = old_keys.get(new_key) {
                // Node exists in old tree
                matched_old[old_idx] = true;

                // Check if it needs to move
                if old_idx != new_idx {
                    moves.push((new_key.clone(), new_idx));
                }

                // Recursively diff the node
                self.diff_nodes(
                    old_children[old_idx].as_ref(),
                    new_child.as_ref(),
                    Some(parent_key.clone()),
                    new_idx,
                    patches,
                    reused,
                    added,
                    removed,
                );
            } else {
                // New node, insert it
                patches.push(PatchOp::Insert {
                    parent_key: Some(parent_key.clone()),
                    index: new_idx,
                    node_key: new_key.clone(),
                });
                *added += self.count_nodes(new_child.as_ref());
            }
        }

        // Remove unmatched old nodes
        for (old_idx, old_child) in old_children.iter().enumerate() {
            if !matched_old[old_idx] {
                patches.push(PatchOp::Remove {
                    node_key: old_child.key().clone(),
                });
                *removed += self.count_nodes(old_child.as_ref());
            }
        }

        // Apply moves if necessary
        if !moves.is_empty() {
            let new_order: Vec<_> = new_children
                .iter()
                .map(|child| child.key().clone())
                .collect();

            patches.push(PatchOp::ReorderChildren {
                parent_key,
                new_order,
            });
        }
    }

    /// Count total nodes in a subtree
    #[allow(clippy::only_used_in_recursion)]
    fn count_nodes(&self, node: &dyn RenderNode) -> usize {
        1 + node
            .children()
            .iter()
            .map(|child| self.count_nodes(child.as_ref()))
            .sum::<usize>()
    }

    /// Get reconciler statistics
    pub fn stats(&self) -> String {
        format!(
            "Reconciler Stats: {} diffs, {} cache hits, {} cache misses",
            self.stats.total_diffs, self.stats.cache_hits, self.stats.cache_misses
        )
    }
}

impl Default for Reconciler {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply patches to update the actual UI
pub fn apply_patches(patches: &[PatchOp], _tree: &mut RenderTree) {
    for patch in patches {
        match patch {
            PatchOp::Insert {
                parent_key: _,
                index: _,
                node_key: _,
            } => {
                #[cfg(feature = "debug_patches")]
                eprintln!("INSERT: {patch:?}");
            }
            PatchOp::Remove { node_key: _ } => {
                #[cfg(feature = "debug_patches")]
                eprintln!("REMOVE: {patch:?}");
            }
            PatchOp::Replace {
                old_key: _,
                new_key: _,
            } => {
                #[cfg(feature = "debug_patches")]
                eprintln!("REPLACE: {patch:?}");
            }
            PatchOp::Move {
                node_key: _,
                parent_key: _,
                index: _,
            } => {
                #[cfg(feature = "debug_patches")]
                eprintln!("MOVE: {patch:?}");
            }
            PatchOp::Update { node_key: _ } => {
                #[cfg(feature = "debug_patches")]
                eprintln!("UPDATE: {patch:?}");
            }
            PatchOp::ReorderChildren {
                parent_key: _,
                new_order: _,
            } => {
                #[cfg(feature = "debug_patches")]
                eprintln!("REORDER: {patch:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Element, LayoutType};
    use crate::render::tree::element_to_render_node;

    #[test]
    fn test_diff_identical_trees() {
        let mut reconciler = Reconciler::new();

        let element = Element::layout(LayoutType::Flex).with_key("root");
        let root1 = element_to_render_node(element.clone());
        let root2 = element_to_render_node(element);

        let mut tree1 = RenderTree::new();
        tree1.set_root(root1);

        let mut tree2 = RenderTree::new();
        tree2.set_root(root2);

        let result = reconciler.diff(&tree1, &tree2);

        assert_eq!(result.patches.len(), 0);
        assert_eq!(result.reused_nodes, 1);
        assert_eq!(result.new_nodes, 0);
        assert_eq!(result.removed_nodes, 0);
    }

    #[test]
    fn test_diff_different_trees() {
        let mut reconciler = Reconciler::new();

        let element1 = Element::text("Hello").with_key("text1");
        let element2 = Element::text("World").with_key("text2");

        let root1 = element_to_render_node(element1);
        let root2 = element_to_render_node(element2);

        let mut tree1 = RenderTree::new();
        tree1.set_root(root1);

        let mut tree2 = RenderTree::new();
        tree2.set_root(root2);

        let result = reconciler.diff(&tree1, &tree2);

        assert!(!result.patches.is_empty());
        assert_eq!(result.reused_nodes, 0);
        assert_eq!(result.new_nodes, 1);
        assert_eq!(result.removed_nodes, 1);
    }

    #[test]
    fn test_diff_with_children() {
        let mut reconciler = Reconciler::new();

        // Tree 1: Root with 2 children
        let tree1_root = Element::layout(LayoutType::Flex)
            .with_key("root")
            .with_children(vec![
                Element::text("A").with_key("a"),
                Element::text("B").with_key("b"),
            ]);

        // Tree 2: Root with 3 children (B moved, C added)
        let tree2_root = Element::layout(LayoutType::Flex)
            .with_key("root")
            .with_children(vec![
                Element::text("B").with_key("b"),
                Element::text("A").with_key("a"),
                Element::text("C").with_key("c"),
            ]);

        let mut tree1 = RenderTree::new();
        tree1.set_root(element_to_render_node(tree1_root));

        let mut tree2 = RenderTree::new();
        tree2.set_root(element_to_render_node(tree2_root));

        let result = reconciler.diff(&tree1, &tree2);

        // Should detect reordering and insertion
        assert!(result
            .patches
            .iter()
            .any(|p| matches!(p, PatchOp::ReorderChildren { .. })));
        assert!(result
            .patches
            .iter()
            .any(|p| matches!(p, PatchOp::Insert { .. })));
    }
}
