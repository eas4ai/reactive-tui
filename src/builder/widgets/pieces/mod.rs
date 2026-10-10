//! Builders for the display pieces (docs/spec/display-pieces.md): one free function per
//! piece, such as `icon()`, re-exported from `reactive_tui::builder`.

pub mod alert;
pub mod badge;
pub mod description_list;
pub mod empty;
pub mod icon;
pub mod kbd;
pub mod link;
pub mod pagination;
pub mod separator;
pub mod shimmer;
pub mod skeleton;
pub mod spinner;
pub mod status_bar;
pub mod stepper;

pub use icon::{icon, IconBuilder};
pub use shimmer::{shimmer, ShimmerBuilder};
pub use skeleton::{skeleton, SkeletonBuilder};
pub use spinner::{spinner, SpinnerBuilder};
