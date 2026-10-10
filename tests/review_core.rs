//! The code review's core findings (docs/spec/roadmap.md,
//! review-core-findings), each through its requirement's falsifier: the
//! components (docs/spec/components.md, CMP), the styles
//! (docs/spec/styles.md, STY), the painter's explicit cells
//! (docs/spec/painter.md, PNT-006) and the reactive state
//! (docs/spec/reactive.md, SIG-003 to SIG-007). A test's name starts with
//! its requirement, which is how scripts/cairn/review_core.py picks it;
//! tests that need a module's private parts live in that module, named the
//! same way.

mod common;

#[path = "review_core/components.rs"]
mod components;
#[path = "review_core/reactive.rs"]
mod reactive;
#[path = "review_core/styles.rs"]
mod styles;
