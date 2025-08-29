pub mod bridge;
pub mod diff;
pub mod node;
pub mod patch;

pub use diff::{DiffContext, diff_vnodes};
pub use node::{VComponent, VElement, VFragment, VNode, VNodeKey, VNodeType, VText};
pub use patch::{Patch, PatchList, apply_patches};

/// Re-export commonly used items
pub mod prelude {
    pub use super::diff::diff_vnodes;
    pub use super::node::{VComponent, VElement, VFragment, VNode, VText};
    pub use super::patch::{Patch, apply_patches};
}
