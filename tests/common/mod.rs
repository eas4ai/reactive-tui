//! Shared integration-test harness.
//!
//! Declared from each test binary with plain `mod common;` (Cargo
//! auto-discovers `tests/common/mod.rs`, so no `#[path]` attributes).
//! Re-exported items keep their `app_input::` paths via
//! `use common::app_input;` at each use site.

pub mod app_input;
#[allow(dead_code)]
pub mod digest;
