//! The behavior the widgets share, from the widget study's adoption changes
//! (docs/spec/roadmap.md, widget-behavior), each through its requirement's
//! falsifier: docs/spec/components.md CMP-009, docs/spec/input-widgets.md
//! CTL-005 and CTL-006, docs/spec/theme.md THM-006,
//! docs/spec/layout-widgets.md NAV-006, docs/spec/clipboard.md CLP-001 and
//! docs/spec/keymap.md KEY-001 and KEY-002. A test's name starts with its
//! requirement, which is how scripts/cairn/widget_behavior.py picks it; a
//! test that needs a module's private parts lives among the library's unit
//! tests under the same name.

mod common;

#[path = "widget_behavior/clipboard.rs"]
mod clipboard;
#[path = "widget_behavior/components.rs"]
mod components;
#[path = "widget_behavior/controls.rs"]
mod controls;
#[path = "widget_behavior/keymap.rs"]
mod keymap;
#[path = "widget_behavior/layout.rs"]
mod layout;
