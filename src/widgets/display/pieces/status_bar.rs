//! The status bar display piece (docs/spec/display-pieces.md), carrying DIS-001,
//! DIS-002, DIS-004.
//!
//! A status bar is one row on `surface` with three regions of text: left, center and
//! right. It fills the width its parent allots. When its regions do not fit, it cuts
//! the center first, then the right and then the left, with `…`. It keeps the left
//! region's first cell and the right region's last cell (DIS-002). The cut needs the
//! width, which the component learns when the App lays it out.

use std::any::Any;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::accessibility::{Node, Role};
use crate::builder::core::{div, span};
use crate::component::{Component, Element, LayoutInfo, Props};
use crate::widgets::display::look;
use crate::widgets::display::pieces::separator::sized;

/// The props of a status bar (DIS-001, DIS-002, DIS-004).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatusBarProps {
    /// The text at the left edge.
    pub left: String,
    /// The text in the middle.
    pub center: String,
    /// The text at the right edge.
    pub right: String,
    /// Classes of the bar. A `w-`, `h-` class sets its size in place of the fill default.
    pub classes: Vec<String>,
}

impl Props for StatusBarProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The status bar component. It keeps the width its layout gave it, so it can cut
/// its regions to fit.
pub struct StatusBar {
    width: usize,
}

impl Component for StatusBar {
    type Props = StatusBarProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self { width: 0 }
    }

    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut ()) -> bool {
        let (width, _) = layout.content_size();
        let width = width.max(0.0) as usize;
        let changed = width != self.width;
        self.width = width;
        changed
    }

    fn render(&self, props: &Self::Props, _: &()) -> Element {
        render(props, self.width)
    }
}

/// The element of a status bar of `width` cells.
pub(crate) fn render(props: &StatusBarProps, width: usize) -> Element {
    let text = fit(&props.left, &props.center, &props.right, width);
    let classes = sized(
        "flex-row",
        &props.classes,
        &[("w-", "w-full"), ("h-", "h-1")],
    );
    let classes = format!("{classes} {} {}", "bg-surface", look::TEXT);
    let mut node = Node::new(Role::Status);
    node.set_value(
        [&props.left, &props.center, &props.right]
            .iter()
            .filter(|part| !part.is_empty())
            .map(|part| part.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    div()
        .class(&classes)
        .child(span().class(look::TEXT).text(&text).build())
        .build()
        .with_accessibility(node)
}

/// The text of a status bar of `width` cells: the three regions laid on one row,
/// the center in the middle, cut when they do not fit. The order of the cuts is
/// center, then right, then left. A cut region ends in `…` on the cut side: the
/// center and the left on the right, the right on the left. The right region keeps
/// at least its last cell and the left region its first, down to two cells (DIS-002);
/// when the left region alone leaves no room, the right keeps one cell and the left
/// takes the rest.
pub fn fit(left: &str, center: &str, right: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut left = left.to_string();
    let mut center = center.to_string();
    let mut right = right.to_string();
    let (lw, rw) = (cells(&left), cells(&right));
    let center_budget = width.saturating_sub(lw + rw);
    if cells(&center) > center_budget {
        center = cut_end(&center, center_budget);
    }
    if lw + rw > width {
        center.clear();
        // The right region gets what the full left region leaves, but at least one
        // cell so its last cell stays on the row.
        let right_budget = width.saturating_sub(lw).max(1).min(rw);
        if rw > right_budget {
            right = cut_start(&right, right_budget);
        }
        let left_budget = width.saturating_sub(cells(&right));
        if lw > left_budget {
            left = cut_end(&left, left_budget);
        }
    }
    let (lw, cw, rw) = (cells(&left), cells(&center), cells(&right));
    // The center sits in the middle of the row, but never over a region.
    let before = ((width.saturating_sub(cw)) / 2).clamp(lw, width.saturating_sub(cw + rw).max(lw));
    let after = width.saturating_sub(before + cw + rw);
    format!(
        "{left}{}{center}{}{right}",
        " ".repeat(before - lw),
        " ".repeat(after)
    )
}

/// The display width of `text` in cells.
fn cells(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// `text` cut to `budget` cells, its end replaced by `…` when it is cut.
fn cut_end(text: &str, budget: usize) -> String {
    if cells(text) <= budget {
        return text.to_string();
    }
    if budget == 0 {
        return String::new();
    }
    if budget == 1 {
        return lone_cell(text.chars().next());
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > budget - 1 {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push('…');
    out
}

/// `text` cut to `budget` cells, its start replaced by `…` when it is cut.
fn cut_start(text: &str, budget: usize) -> String {
    if cells(text) <= budget {
        return text.to_string();
    }
    if budget == 0 {
        return String::new();
    }
    if budget == 1 {
        return lone_cell(text.chars().last());
    }
    let mut kept: Vec<char> = Vec::new();
    let mut used = 0;
    for ch in text.chars().rev() {
        let w = ch.width().unwrap_or(0);
        if used + w > budget - 1 {
            break;
        }
        kept.push(ch);
        used += w;
    }
    let mut out = String::from('…');
    out.extend(kept.into_iter().rev());
    out
}

/// The one cell a region keeps when it gets one cell: its end glyph, or `…` when
/// that glyph is not one cell wide.
fn lone_cell(ch: Option<char>) -> String {
    match ch {
        Some(ch) if ch.width() == Some(1) => ch.to_string(),
        _ => "…".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dis_002_status_bar_cuts_the_center_then_the_right_and_keeps_both_ends() {
        let left = "L".repeat(60);
        let center = "C".repeat(60);
        let right = "R".repeat(60);
        let text = fit(&left, &center, &right, 100);
        assert_eq!(cells(&text), 100, "the bar fills its width");
        assert!(
            text.starts_with('L'),
            "the left region keeps its first cell"
        );
        assert!(text.ends_with('R'), "the right region keeps its last cell");
        assert!(text.contains('…'), "the cut shows an ellipsis");
        assert!(!text.contains('C'), "the center goes first");
    }

    #[test]
    fn dis_002_status_bar_keeps_both_ends_at_width_40() {
        let text = fit(&"L".repeat(60), &"C".repeat(60), &"R".repeat(60), 40);
        assert_eq!(cells(&text), 40, "the bar fills its width");
        assert!(
            text.starts_with('L'),
            "the left region keeps its first cell"
        );
        assert!(text.ends_with('R'), "the right region keeps its last cell");
        assert!(text.contains('…'), "the cut shows an ellipsis");
        assert!(!text.contains('C'), "the center goes first");
    }

    #[test]
    fn dis_002_status_bar_keeps_both_ends_at_width_10() {
        let text = fit(&"L".repeat(60), &"C".repeat(60), &"R".repeat(60), 10);
        assert_eq!(cells(&text), 10, "the bar fills its width");
        assert!(
            text.starts_with('L'),
            "the left region keeps its first cell"
        );
        assert!(text.ends_with('R'), "the right region keeps its last cell");
        assert!(text.contains('…'), "the cut shows an ellipsis");
    }

    #[test]
    fn dis_002_status_bar_at_width_2_is_its_left_cell_then_its_right_cell() {
        assert_eq!(
            fit(&"L".repeat(60), &"C".repeat(60), &"R".repeat(60), 2),
            "LR"
        );
    }

    #[test]
    fn dis_002_status_bar_fits_its_regions_without_a_cut() {
        assert_eq!(fit("Ready", "", "12:00", 20), "Ready          12:00");
    }

    #[test]
    fn dis_004_status_bar_is_a_status_with_its_regions_as_value() {
        let props = StatusBarProps {
            left: "Ready".into(),
            right: "Ln 4".into(),
            ..StatusBarProps::default()
        };
        let element = render(&props, 30);
        let node = element.metadata.accessibility.as_ref().expect("a node");
        assert_eq!(node.role(), Role::Status);
        assert_eq!(node.inner.value(), Some("Ready Ln 4"));
    }
}
