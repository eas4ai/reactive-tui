use super::diff::DiffStats;
use super::node::VNode;
use std::any::Any;
use std::sync::Arc;

/// A patch operation to apply to the DOM
#[derive(Clone, Debug)]
pub enum Patch {
    /// Replace a node entirely
    Replace {
        /// Index of the node to replace
        index: usize,
        /// The old node being replaced
        old: VNode,
        /// The new node to replace with
        new: VNode,
    },

    /// Insert a new node
    Insert {
        /// Index where to insert the node
        index: usize,
        /// Index of the parent node
        parent: usize,
        /// The node to insert
        node: VNode,
    },

    /// Remove a node
    Remove {
        /// Index of the node to remove
        index: usize,
        /// Index of the parent node
        parent: usize,
    },

    /// Move a node to a different position
    Move {
        /// Current index of the node
        from: usize,
        /// Target index for the node
        to: usize,
        /// Index of the parent node
        parent: usize,
    },

    /// Update text content
    SetText {
        /// Index of the node to update
        index: usize,
        /// New text content
        text: String,
    },

    /// Set an attribute
    SetAttribute {
        /// Index of the node to update
        index: usize,
        /// Attribute name
        name: String,
        /// Attribute value
        value: String,
    },

    /// Remove an attribute
    RemoveAttribute {
        /// Index of the node to update
        index: usize,
        /// Attribute name to remove
        name: String,
    },

    /// Set the class
    SetClass {
        /// Index of the node to update
        index: usize,
        /// New class name (None to remove)
        class: Option<String>,
    },

    /// Set the style
    SetStyle {
        /// Index of the node to update
        index: usize,
        /// New style string (None to remove)
        style: Option<String>,
    },

    /// Update component props
    UpdateProps {
        /// Index of the component to update
        index: usize,
        /// New props for the component
        props: Arc<dyn Any + Send + Sync>,
    },

    /// Add an event listener
    AddEventListener {
        /// Index of the node to add listener to
        index: usize,
        /// Event type to listen for
        event: String,
    },

    /// Remove an event listener
    RemoveEventListener {
        /// Index of the node to remove listener from
        index: usize,
        /// Event type to stop listening for
        event: String,
    },
}

/// A list of patches with statistics
#[derive(Debug)]
pub struct PatchList {
    patches: Vec<Patch>,
    stats: DiffStats,
}

impl PatchList {
    /// Create a new empty patch list
    pub fn new() -> Self {
        Self {
            patches: Vec::new(),
            stats: DiffStats::default(),
        }
    }

    /// Add a patch to the list
    pub fn push(&mut self, patch: Patch) {
        self.patches.push(patch);
    }

    /// Get the patches
    pub fn patches(&self) -> &[Patch] {
        &self.patches
    }

    /// Get mutable patches
    pub fn patches_mut(&mut self) -> &mut Vec<Patch> {
        &mut self.patches
    }

    /// Get the number of patches
    pub fn len(&self) -> usize {
        self.patches.len()
    }

    /// Check if the patch list is empty
    pub fn is_empty(&self) -> bool {
        self.patches.is_empty()
    }

    /// Set statistics
    pub fn set_stats(&mut self, stats: DiffStats) {
        self.stats = stats;
    }

    /// Get statistics
    pub fn stats(&self) -> &DiffStats {
        &self.stats
    }

    /// Optimize the patch list by combining adjacent operations
    pub fn optimize(&mut self) {
        let mut optimized = Vec::new();
        let mut pending_attributes: std::collections::HashMap<
            usize,
            std::collections::HashMap<String, String>,
        > = std::collections::HashMap::new();

        for patch in &self.patches {
            match patch {
                Patch::SetAttribute { index, name, value } => {
                    // Collect attribute updates for same node
                    pending_attributes
                        .entry(*index)
                        .or_default()
                        .insert(name.clone(), value.clone());
                }
                _ => {
                    // Flush pending attributes before adding other patches
                    for (index, attrs) in pending_attributes.drain() {
                        for (name, value) in attrs {
                            optimized.push(Patch::SetAttribute { index, name, value });
                        }
                    }
                    optimized.push(patch.clone());
                }
            }
        }

        // Flush any remaining attributes
        for (index, attrs) in pending_attributes.drain() {
            for (name, value) in attrs {
                optimized.push(Patch::SetAttribute { index, name, value });
            }
        }

        // Additional optimizations: remove redundant operations
        self.patches = optimized;
        self.remove_redundant_moves();
        self.combine_adjacent_text_updates();
    }

    fn remove_redundant_moves(&mut self) {
        let mut seen_moves: std::collections::HashSet<usize> = std::collections::HashSet::new();
        self.patches.retain(|patch| {
            if let Patch::Move { from, .. } = patch {
                seen_moves.insert(*from)
            } else {
                true
            }
        });
    }

    fn combine_adjacent_text_updates(&mut self) {
        let mut i = 0;
        while i + 1 < self.patches.len() {
            if let (
                Patch::SetText {
                    index: id1,
                    text: _text1,
                },
                Patch::SetText {
                    index: id2,
                    text: text2,
                },
            ) = (&self.patches[i], &self.patches[i + 1])
            {
                if id1 == id2 {
                    // Combine texts - later update wins
                    self.patches[i] = Patch::SetText {
                        index: *id1,
                        text: text2.clone(),
                    };
                    self.patches.remove(i + 1);
                    continue;
                }
            }
            i += 1;
        }
    }
}

impl Default for PatchList {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply patches to a real DOM or render target
pub trait PatchApplier {
    /// The type of DOM node
    type Node;

    /// Apply a patch to the DOM
    fn apply_patch(&mut self, patch: &Patch, nodes: &mut Vec<Self::Node>);

    /// Create a DOM node from a virtual node
    fn create_node(&mut self, vnode: &VNode) -> Self::Node;

    /// Get a node by index
    fn get_node(&self, index: usize) -> Option<&Self::Node>;

    /// Get a mutable node by index
    fn get_node_mut(&mut self, index: usize) -> Option<&mut Self::Node>;
}

/// Apply patches to a DOM
pub fn apply_patches<A: PatchApplier>(
    applier: &mut A,
    patches: &PatchList,
    nodes: &mut Vec<A::Node>,
) {
    for patch in patches.patches() {
        applier.apply_patch(patch, nodes);
    }
}

/// Example patch applier for testing
#[cfg(test)]
#[derive(Default)]
pub struct TestPatchApplier {
    operations: Vec<String>,
}

#[cfg(test)]
impl TestPatchApplier {
    /// Create a new test patch applier
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the list of patch operations
    ///
    /// # Returns
    /// A slice of operation strings describing the patches applied
    pub fn operations(&self) -> &[String] {
        &self.operations
    }
}

#[cfg(test)]
impl PatchApplier for TestPatchApplier {
    type Node = VNode;

    fn apply_patch(&mut self, patch: &Patch, _nodes: &mut Vec<Self::Node>) {
        match patch {
            Patch::Replace { index, .. } => {
                self.operations.push(format!("Replace at {index}"));
            }
            Patch::Insert { index, parent, .. } => {
                self.operations
                    .push(format!("Insert at {index} in parent {parent}"));
            }
            Patch::Remove { index, parent } => {
                self.operations
                    .push(format!("Remove at {index} from parent {parent}"));
            }
            Patch::Move { from, to, parent } => {
                self.operations
                    .push(format!("Move from {from} to {to} in parent {parent}"));
            }
            Patch::SetText { index, text } => {
                self.operations.push(format!("SetText at {index}: {text}"));
            }
            Patch::SetAttribute { index, name, value } => {
                self.operations
                    .push(format!("SetAttribute at {index}: {name}={value}"));
            }
            Patch::RemoveAttribute { index, name } => {
                self.operations
                    .push(format!("RemoveAttribute at {index}: {name}"));
            }
            Patch::SetClass { index, class } => {
                self.operations
                    .push(format!("SetClass at {index}: {class:?}"));
            }
            Patch::SetStyle { index, style } => {
                self.operations
                    .push(format!("SetStyle at {index}: {style:?}"));
            }
            Patch::UpdateProps { index, .. } => {
                self.operations.push(format!("UpdateProps at {index}"));
            }
            Patch::AddEventListener { index, event } => {
                self.operations
                    .push(format!("AddEventListener at {index}: {event}"));
            }
            Patch::RemoveEventListener { index, event } => {
                self.operations
                    .push(format!("RemoveEventListener at {index}: {event}"));
            }
        }
    }

    fn create_node(&mut self, vnode: &VNode) -> Self::Node {
        vnode.clone()
    }

    fn get_node(&self, _index: usize) -> Option<&Self::Node> {
        None
    }

    fn get_node_mut(&mut self, _index: usize) -> Option<&mut Self::Node> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_list() {
        let mut patches = PatchList::new();

        patches.push(Patch::SetText {
            index: 0,
            text: "Hello".to_string(),
        });

        patches.push(Patch::SetAttribute {
            index: 1,
            name: "id".to_string(),
            value: "main".to_string(),
        });

        assert_eq!(patches.len(), 2);
        assert!(!patches.is_empty());
    }

    #[test]
    fn test_patch_applier() {
        let mut applier = TestPatchApplier::new();
        let mut patches = PatchList::new();

        patches.push(Patch::Insert {
            index: 0,
            parent: 0,
            node: VNode::text("Hello"),
        });

        patches.push(Patch::Move {
            from: 1,
            to: 2,
            parent: 0,
        });

        let mut nodes = Vec::new();
        apply_patches(&mut applier, &patches, &mut nodes);

        assert_eq!(applier.operations().len(), 2);
        assert_eq!(applier.operations()[0], "Insert at 0 in parent 0");
        assert_eq!(applier.operations()[1], "Move from 1 to 2 in parent 0");
    }
}
