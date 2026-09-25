use super::tree::{NodeKey, RenderNode, RenderTree};
use std::collections::HashMap;

/// Result of diffing two render trees
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// List of patch operations to apply
    pub patches: Vec<PatchOp>,
    /// Number of nodes that were reused from the old tree
    pub reused_nodes: usize,
    /// Number of new nodes added
    pub new_nodes: usize,
    /// Number of nodes removed
    pub removed_nodes: usize,
}

/// Patch operation to transform old tree into new tree
#[derive(Debug, Clone)]
pub enum PatchOp {
    /// Insert a new node
    Insert {
        /// Parent node key (None for root)
        parent_key: Option<NodeKey>,
        /// Position in parent's children list
        index: usize,
        /// Key of the node to insert
        node_key: NodeKey,
    },

    /// Remove a node
    Remove {
        /// Key of the node to remove
        node_key: NodeKey,
    },

    /// Replace a node with another
    Replace {
        /// Key of the node to replace
        old_key: NodeKey,
        /// Key of the replacement node
        new_key: NodeKey,
    },

    /// Move a node to a different position
    Move {
        /// Key of the node to move
        node_key: NodeKey,
        /// New parent node key (None for root)
        parent_key: Option<NodeKey>,
        /// New position in parent's children list
        index: usize,
    },

    /// Update node properties
    Update {
        /// Key of the node to update
        node_key: NodeKey,
    },

    /// Reorder children
    ReorderChildren {
        /// Parent node whose children are being reordered
        parent_key: NodeKey,
        /// New order of child node keys
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
    /// Create a new reconciler
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

/// Apply patches to update the actual UI and manage component lifecycle
pub fn apply_patches(patches: &[PatchOp], tree: &mut RenderTree) -> crate::error::Result<()> {
    for patch in patches {
        match patch {
            PatchOp::Insert {
                parent_key: _,
                index: _,
                node_key: _,
            } => {
                #[cfg(feature = "debug_patches")]
                log::debug!("INSERT: {patch:?}");
                // Component instances will be created by the render system
            }
            PatchOp::Remove { node_key } => {
                #[cfg(feature = "debug_patches")]
                log::debug!("REMOVE: {patch:?}");

                // Automatic component cleanup - unregister from global registry
                // Propagate error up instead of suppressing it
                crate::component::registry::get_global_registry()
                    .unregister_instance(node_key)
                    .map_err(|e| {
                        log::error!(
                            "Error: Failed to unregister component during removal: {}",
                            e
                        );
                        e
                    })?;

                tree.remove_node(node_key);
            }
            PatchOp::Replace {
                old_key,
                new_key: _,
            } => {
                #[cfg(feature = "debug_patches")]
                log::debug!("REPLACE: {patch:?}");

                // Automatic component cleanup for replaced node
                // Propagate error up instead of suppressing it
                crate::component::registry::get_global_registry()
                    .unregister_instance(old_key)
                    .map_err(|e| {
                        log::error!("Error: Failed to unregister replaced component: {}", e);
                        e
                    })?;

                tree.remove_node(old_key);
                // Note: New component instance will be created automatically during rendering
            }
            PatchOp::Move {
                node_key: _,
                parent_key: _,
                index: _,
            } => {
                #[cfg(feature = "debug_patches")]
                log::debug!("MOVE: {patch:?}");
                // Component instances stay the same, just moved in tree
            }
            PatchOp::Update { node_key: _ } => {
                #[cfg(feature = "debug_patches")]
                log::debug!("UPDATE: {patch:?}");
                // Component instances will be updated by the render system
            }
            PatchOp::ReorderChildren {
                parent_key: _,
                new_order: _,
            } => {
                #[cfg(feature = "debug_patches")]
                log::debug!("REORDER: {patch:?}");
                // Component instances stay the same, just reordered
            }
        }
    }
    Ok(())
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

    #[test]
    fn test_diff_empty_to_non_empty() {
        let mut reconciler = Reconciler::new();

        let empty_tree = RenderTree::new();

        let element = Element::text("Hello").with_key("text");
        let mut non_empty_tree = RenderTree::new();
        non_empty_tree.set_root(element_to_render_node(element));

        let result = reconciler.diff(&empty_tree, &non_empty_tree);

        assert_eq!(result.patches.len(), 1);
        assert!(matches!(result.patches[0], PatchOp::Insert { .. }));
        assert_eq!(result.new_nodes, 1);
        assert_eq!(result.removed_nodes, 0);
        assert_eq!(result.reused_nodes, 0);
    }

    #[test]
    fn test_diff_non_empty_to_empty() {
        let mut reconciler = Reconciler::new();

        let element = Element::text("Hello").with_key("text");
        let mut non_empty_tree = RenderTree::new();
        non_empty_tree.set_root(element_to_render_node(element));

        let empty_tree = RenderTree::new();

        let result = reconciler.diff(&non_empty_tree, &empty_tree);

        assert_eq!(result.patches.len(), 1);
        assert!(matches!(result.patches[0], PatchOp::Remove { .. }));
        assert_eq!(result.new_nodes, 0);
        assert_eq!(result.removed_nodes, 1);
        assert_eq!(result.reused_nodes, 0);
    }

    #[test]
    fn test_diff_both_empty() {
        let mut reconciler = Reconciler::new();

        let empty_tree1 = RenderTree::new();
        let empty_tree2 = RenderTree::new();

        let result = reconciler.diff(&empty_tree1, &empty_tree2);

        assert_eq!(result.patches.len(), 0);
        assert_eq!(result.new_nodes, 0);
        assert_eq!(result.removed_nodes, 0);
        assert_eq!(result.reused_nodes, 0);
    }

    #[test]
    fn test_diff_node_replacement() {
        let mut reconciler = Reconciler::new();

        let element1 = Element::text("Hello").with_key("text");
        let element2 = Element::layout(LayoutType::Flex).with_key("layout");

        let mut tree1 = RenderTree::new();
        tree1.set_root(element_to_render_node(element1));

        let mut tree2 = RenderTree::new();
        tree2.set_root(element_to_render_node(element2));

        let result = reconciler.diff(&tree1, &tree2);

        assert_eq!(result.patches.len(), 1);
        assert!(matches!(result.patches[0], PatchOp::Replace { .. }));
        assert_eq!(result.new_nodes, 1);
        assert_eq!(result.removed_nodes, 1);
        assert_eq!(result.reused_nodes, 0);
    }

    #[test]
    fn test_diff_node_update() {
        let mut reconciler = Reconciler::new();

        // Same key and type, but different content
        let element1 = Element::text("Hello").with_key("text");
        let element2 = Element::text("World").with_key("text");

        let mut tree1 = RenderTree::new();
        tree1.set_root(element_to_render_node(element1));

        let mut tree2 = RenderTree::new();
        tree2.set_root(element_to_render_node(element2));

        let result = reconciler.diff(&tree1, &tree2);

        // Should detect an update since content changed but key/type same
        assert!(result
            .patches
            .iter()
            .any(|p| matches!(p, PatchOp::Update { .. })));
        assert_eq!(result.reused_nodes, 1);
    }

    #[test]
    fn test_diff_children_removal() {
        let mut reconciler = Reconciler::new();

        // Tree 1: Root with 3 children
        let tree1_root = Element::layout(LayoutType::Flex)
            .with_key("root")
            .with_children(vec![
                Element::text("A").with_key("a"),
                Element::text("B").with_key("b"),
                Element::text("C").with_key("c"),
            ]);

        // Tree 2: Root with 1 child (B and C removed)
        let tree2_root = Element::layout(LayoutType::Flex)
            .with_key("root")
            .with_children(vec![Element::text("A").with_key("a")]);

        let mut tree1 = RenderTree::new();
        tree1.set_root(element_to_render_node(tree1_root));

        let mut tree2 = RenderTree::new();
        tree2.set_root(element_to_render_node(tree2_root));

        let result = reconciler.diff(&tree1, &tree2);

        // Should detect removals
        let remove_count = result
            .patches
            .iter()
            .filter(|p| matches!(p, PatchOp::Remove { .. }))
            .count();
        assert_eq!(remove_count, 2); // B and C removed
        assert_eq!(result.removed_nodes, 2);
        assert_eq!(result.reused_nodes, 2); // root and A
    }

    #[test]
    fn test_diff_children_addition() {
        let mut reconciler = Reconciler::new();

        // Tree 1: Root with 1 child
        let tree1_root = Element::layout(LayoutType::Flex)
            .with_key("root")
            .with_children(vec![Element::text("A").with_key("a")]);

        // Tree 2: Root with 3 children (B and C added)
        let tree2_root = Element::layout(LayoutType::Flex)
            .with_key("root")
            .with_children(vec![
                Element::text("A").with_key("a"),
                Element::text("B").with_key("b"),
                Element::text("C").with_key("c"),
            ]);

        let mut tree1 = RenderTree::new();
        tree1.set_root(element_to_render_node(tree1_root));

        let mut tree2 = RenderTree::new();
        tree2.set_root(element_to_render_node(tree2_root));

        let result = reconciler.diff(&tree1, &tree2);

        // Should detect insertions
        let insert_count = result
            .patches
            .iter()
            .filter(|p| matches!(p, PatchOp::Insert { .. }))
            .count();
        assert_eq!(insert_count, 2); // B and C inserted
        assert_eq!(result.new_nodes, 2);
        assert_eq!(result.reused_nodes, 2); // root and A
    }

    #[test]
    fn test_reconciler_stats() {
        let mut reconciler = Reconciler::new();

        let element = Element::text("Hello").with_key("text");
        let mut tree = RenderTree::new();
        tree.set_root(element_to_render_node(element));

        // Perform multiple diffs
        reconciler.diff(&tree, &tree);
        reconciler.diff(&tree, &tree);
        reconciler.diff(&tree, &tree);

        let stats = reconciler.stats();
        assert!(stats.contains("3 diffs"));
    }

    #[test]
    fn test_count_nodes() {
        let reconciler = Reconciler::new();

        // Create a tree with nested children
        let element = Element::layout(LayoutType::Flex)
            .with_key("root")
            .with_children(vec![
                Element::text("A").with_key("a"),
                Element::layout(LayoutType::Flex)
                    .with_key("nested")
                    .with_children(vec![
                        Element::text("B").with_key("b"),
                        Element::text("C").with_key("c"),
                    ]),
            ]);

        let node = element_to_render_node(element);
        let count = reconciler.count_nodes(node.as_ref());

        // Should count: root + A + nested + B + C = 5 nodes
        assert_eq!(count, 5);
    }

    #[test]
    fn test_apply_patches() {
        let mut tree = RenderTree::new();

        let patches = vec![
            PatchOp::Insert {
                parent_key: None,
                index: 0,
                node_key: NodeKey::named("test"),
            },
            PatchOp::Remove {
                node_key: NodeKey::named("old"),
            },
            PatchOp::Update {
                node_key: NodeKey::named("update"),
            },
        ];

        // Patches naming unknown keys apply cleanly and leave the tree empty
        let result = apply_patches(&patches, &mut tree);
        assert!(result.is_ok(), "apply_patches failed: {result:?}");
        assert!(tree.root().is_none(), "no node was inserted into the tree");
    }
}
