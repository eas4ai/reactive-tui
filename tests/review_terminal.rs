//! The code review's terminal and platform findings (docs/spec/roadmap.md,
//! review-terminal-findings), each through its requirement's falsifier
//! (docs/spec/platform.md, PLT). A test's name starts with its requirement,
//! which is how scripts/cairn/review_terminal.py picks it; tests that need a
//! module's private parts live in that module, named the same way.

#[cfg(unix)]
#[allow(dead_code)]
mod pty_support;

#[path = "review_terminal/display.rs"]
mod display;
#[path = "review_terminal/escape.rs"]
mod escape;
#[path = "review_terminal/input.rs"]
mod input;
#[path = "review_terminal/manual.rs"]
mod manual;
#[path = "review_terminal/output.rs"]
mod output;
#[cfg(unix)]
#[path = "review_terminal/startup.rs"]
mod startup;
