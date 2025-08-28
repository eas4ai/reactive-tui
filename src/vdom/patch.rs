use super::diff::DiffStats;
use super::node::VNode;
use std::any::Any;
use std::sync::Arc;

/// A patch operation to apply to the DOM
#[derive(Clone, Debug)]
pub enum Patch {
    /// Replace a node entirely
    Replace {
        index: usize,
        old: VNode,
        new: VNode,
    },

    /// Insert a new node
    Insert {
        index: usize,
        parent: usize,
        node: VNode,
    },

    /// Remove a node
    Remove { index: usize, parent: usize },

    /// Move a node to a different position
    Move {
        from: usize,
        to: usize,
        parent: usize,
    },

    /// Update text content
    SetText { index: usize, text: String },

    /// Set an attribute
    SetAttribute {
        index: usize,
        name: String,
        value: String,
    },

    /// Remove an attribute
    RemoveAttribute { index: usize, name: String },

    /// Set the class
    SetClass { index: usize, class: Option<String> },

    /// Set the style
    SetStyle { index: usize, style: Option<String> },

    /// Update component props
    UpdateProps {
        index: usize,
        props: Arc<dyn Any + Send + Sync>,
    },

    /// Add an event listener
    AddEventListener { index: usize, event: String },

    /// Remove an event listener
    RemoveEventListener { index: usize, event: String },
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
        // TODO: Implement patch optimization
        // - Combine adjacent text updates
        // - Combine attribute updates on same node
        // - Remove redundant moves
        // - Batch insertions/removals
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
pub struct TestPatchApplier {
    operations: Vec<String>,
}

#[cfg(test)]
impl Default for TestPatchApplier {
    fn default() -> Self {
        Self {
            operations: Vec::new(),
        }
    }
}

#[cfg(test)]
impl TestPatchApplier {
    pub fn new() -> Self {
        Self::default()
    }

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
