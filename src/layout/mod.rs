use taffy::{TaffyTree, style::Style, prelude::NodeId, geometry::Size, AvailableSpace};
pub mod colors;


pub mod style;
pub mod utility_css;
pub mod paint_tree;

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
        let root = tree.new_leaf_with_context(Style::default(), ()).expect("root");
        Self { tree, root }
    }

    pub fn root(&self) -> NodeId { self.root }
    pub fn set_style(&mut self, node: NodeId, style: Style) { self.tree.set_style(node, style).unwrap(); }
    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) { self.tree.set_children(parent, children).unwrap(); }
    pub fn new_leaf(&mut self, style: Style) -> NodeId { self.tree.new_leaf(style).unwrap() }
    pub fn compute(&mut self, width: Option<f32>) {
        let size = Size { width: width.map(AvailableSpace::Definite).unwrap_or(AvailableSpace::MaxContent), height: AvailableSpace::MaxContent };
        self.tree.compute_layout(self.root, size).unwrap();
    }
}

