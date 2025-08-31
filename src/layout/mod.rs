use crate::error::{ReactiveError, Result};
use taffy::{geometry::Size, prelude::NodeId, style::Style, AvailableSpace, TaffyTree};
pub mod colors;
pub mod direct_grid;
pub mod grid;
pub mod paint_tree;
pub mod renderer;
pub mod style;
pub mod utility_css;

pub struct LayoutEngine {
    tree: TaffyTree<()>,
    root: NodeId,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        let mut tree = TaffyTree::new();
        let root = tree
            .new_leaf_with_context(Style::default(), ())
            .expect("root");
        Self { tree, root }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }
    pub fn set_style(&mut self, node: NodeId, style: Style) -> Result<()> {
        self.tree
            .set_style(node, style)
            .map_err(|e| ReactiveError::layout(format!("Failed to set style: {}", e)))
    }
    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) -> Result<()> {
        self.tree
            .set_children(parent, children)
            .map_err(|e| ReactiveError::layout(format!("Failed to set children: {}", e)))
    }
    pub fn new_leaf(&mut self, style: Style) -> Result<NodeId> {
        self.tree
            .new_leaf(style)
            .map_err(|e| ReactiveError::layout(format!("Failed to create leaf: {}", e)))
    }
    pub fn compute(&mut self, width: Option<f32>) -> Result<()> {
        let size = Size {
            width: width
                .map(AvailableSpace::Definite)
                .unwrap_or(AvailableSpace::MaxContent),
            height: AvailableSpace::MaxContent,
        };
        self.tree
            .compute_layout(self.root, size)
            .map_err(|e| ReactiveError::layout(format!("Failed to compute layout: {}", e)))
    }
}
