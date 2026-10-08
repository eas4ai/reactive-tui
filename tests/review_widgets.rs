//! The widget defects of the code review of 2026-10-04 and of the widget
//! study (docs/spec/roadmap.md, widget-defects), each through its
//! requirement's falsifier: docs/spec/text.md TXT-005 and TXT-006,
//! docs/spec/charts.md CHT-041, docs/spec/theme.md THM-005,
//! docs/spec/layout-widgets.md NAV-005, docs/spec/data-widgets.md DAT-005
//! and docs/spec/components.md CMP-008. A test's name starts with its
//! requirement, which is how scripts/cairn/review_widgets.py picks it; a
//! test that needs a module's private parts lives among the library's unit
//! tests under the same name.

mod common;

#[path = "review_widgets/charts.rs"]
mod charts;
#[path = "review_widgets/scroll.rs"]
mod scroll;
#[path = "review_widgets/text.rs"]
mod text;
#[path = "review_widgets/theme.rs"]
mod theme;
