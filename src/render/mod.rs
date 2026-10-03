/// Rendering optimization and dirty region management
pub mod optimize;
/// Virtual DOM reconciliation and diffing
pub mod reconcile;
/// Render scheduling and frame management
pub mod scheduler;
/// Render tree construction and management
pub mod tree;

pub use optimize::{DirtyRegion, RenderCache};
pub use reconcile::{DiffResult, PatchOp, Reconciler};
pub use scheduler::{Priority, RenderScheduler, ScheduleHandle};
pub use tree::{NodeKey, RenderNode, RenderTree};
