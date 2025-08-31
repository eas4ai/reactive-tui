pub mod bridge;
pub mod diff;
pub mod node;
pub mod patch;

pub use diff::{diff_vnodes, DiffContext};
pub use node::{VComponent, VElement, VFragment, VNode, VNodeKey, VNodeType, VText};
pub use patch::{apply_patches, Patch, PatchList};

/// Re-export commonly used items
pub mod prelude {
    pub use super::diff::diff_vnodes;
    pub use super::node::{VComponent, VElement, VFragment, VNode, VText};
    pub use super::patch::{apply_patches, Patch};
}
