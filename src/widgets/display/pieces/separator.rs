//! The separator display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-002, DIS-004.
//!
//! A horizontal separator is one row of a line that fills the width its
//! parent allots, and a vertical one a column of the line that fills the
//! height. It is a splitter for the screen reader. The line's glyph comes
//! from [`glyph`], the table the menus share.

use std::any::Any;

use unicode_width::UnicodeWidthStr;

use crate::accessibility::{Node, Role};
use crate::builder::core::{div, span};
use crate::component::{Component, Element, LayoutInfo, Props};
use crate::widgets::display::look;

/// The line a separator draws (DIS-002, DIS-006).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SeparatorStyle {
    /// A single line.
    #[default]
    Line,
    /// A heavy line.
    Thick,
    /// Two parallel lines.
    Double,
    /// A dashed line.
    Dashed,
    /// A dotted line.
    Dotted,
}

/// The glyph a separator of `style` draws: horizontal `─` for the default
/// line, vertical `│` for the same style. The other styles keep their
/// weight in both directions. A menu's separator rows draw through this table.
pub fn glyph(style: SeparatorStyle, vertical: bool) -> char {
    match (style, vertical) {
        (SeparatorStyle::Line, false) => '─',
        (SeparatorStyle::Thick, false) => '━',
        (SeparatorStyle::Double, false) => '═',
        (SeparatorStyle::Dashed, false) => '╌',
        (SeparatorStyle::Dotted, false) => '┄',
        (SeparatorStyle::Line, true) => '│',
        (SeparatorStyle::Thick, true) => '┃',
        (SeparatorStyle::Double, true) => '║',
        (SeparatorStyle::Dashed, true) => '╎',
        (SeparatorStyle::Dotted, true) => '┆',
    }
}

/// The props of a separator (DIS-001 to DIS-004). The builder and these props
/// make the same element.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SeparatorProps {
    /// The line the separator draws.
    pub style: SeparatorStyle,
    /// Whether the separator runs down the height instead of across the width.
    pub vertical: bool,
    /// A label drawn in the middle of a horizontal separator, `──── Label ────`.
    pub label: Option<String>,
    /// Classes of the separator. A `w-`, `h-` class sets its size in place of
    /// the fill default.
    pub classes: Vec<String>,
    /// The name the screen reader speaks, in place of the label.
    pub aria_label: Option<String>,
}

impl Props for SeparatorProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The separator component. It measures the box it is laid out in, so its
/// line fills the width or height its parent allots.
pub struct Separator {
    size: (u16, u16),
}

impl Component for Separator {
    type Props = SeparatorProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self { size: (0, 0) }
    }

    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut ()) -> bool {
        let (width, height) = layout.content_size();
        let size = (width.max(0.0) as u16, height.max(0.0) as u16);
        let changed = size != self.size;
        self.size = size;
        changed
    }

    fn render(&self, props: &Self::Props, _: &()) -> Element {
        render(props, self.size)
    }
}

/// The class string of a piece: its direction, its own classes, then the
/// default of each size axis that the piece's classes leave unset. A
/// `w-`, `h-` class of the piece's own replaces that axis's default.
pub(crate) fn sized(direction: &str, classes: &[String], defaults: &[(&str, &str)]) -> String {
    let own: Vec<&str> = classes
        .iter()
        .flat_map(|class| class.split_whitespace())
        .collect();
    let mut out = vec![direction.to_string()];
    out.extend(own.iter().map(|class| class.to_string()));
    for (prefix, default) in defaults {
        if !own.iter().any(|class| class.starts_with(prefix)) {
            out.push(default.to_string());
        }
    }
    out.join(" ")
}

/// The element of a separator of `size` cells (width, height), before the
/// component measured it.
pub(crate) fn render(props: &SeparatorProps, size: (u16, u16)) -> Element {
    let (width, height) = (usize::from(size.0), usize::from(size.1));
    let line = glyph(props.style, props.vertical);
    let (defaults, children): (&[(&str, &str)], Vec<Element>) = if props.vertical {
        let rows = (0..height)
            .map(|_| span().class(look::BORDER).text(&line.to_string()).build())
            .collect();
        (&[("w-", "w-1"), ("h-", "h-full")], rows)
    } else {
        let row = match &props.label {
            Some(label) => labelled(line, label, width),
            None => vec![span()
                .class(look::BORDER)
                .text(&line.to_string().repeat(width))
                .build()],
        };
        (&[("w-", "w-full"), ("h-", "h-1")], row)
    };
    let direction = if props.vertical {
        "flex-col"
    } else {
        "flex-row"
    };
    let classes = sized(direction, &props.classes, defaults);
    let mut node = Node::new(Role::Splitter);
    if let Some(name) = props.aria_label.as_deref().or(props.label.as_deref()) {
        node.set_label(name);
    }
    div()
        .class(&classes)
        .children(children)
        .build()
        .with_accessibility(node)
}

/// A row that holds the label in the middle of the line: the line's glyph,
/// one space, the label, one space and the line's glyph again. The label
/// is painted `text-muted` and the line `text-border`.
fn labelled(line: char, label: &str, width: usize) -> Vec<Element> {
    let label_width = UnicodeWidthStr::width(label) + 2;
    if width < label_width {
        return vec![span()
            .class(look::MUTED)
            .text(&format!(" {label} "))
            .build()];
    }
    let left = (width - label_width) / 2;
    let right = width - label_width - left;
    vec![
        span()
            .class(look::BORDER)
            .text(&line.to_string().repeat(left))
            .build(),
        span()
            .class(look::MUTED)
            .text(&format!(" {label} "))
            .build(),
        span()
            .class(look::BORDER)
            .text(&line.to_string().repeat(right))
            .build(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dis_004_separator_is_a_splitter_labelled_by_its_label() {
        let props = SeparatorProps {
            label: Some("Results".into()),
            ..SeparatorProps::default()
        };
        let element = render(&props, (30, 1));
        let node = element.metadata.accessibility.as_ref().expect("a node");
        assert_eq!(node.role(), Role::Splitter);
        assert_eq!(node.inner.label(), Some("Results"));
    }

    #[test]
    fn dis_004_vertical_separator_is_a_splitter_without_a_label() {
        let props = SeparatorProps {
            vertical: true,
            ..SeparatorProps::default()
        };
        let element = render(&props, (1, 4));
        let node = element.metadata.accessibility.as_ref().expect("a node");
        assert_eq!(node.role(), Role::Splitter);
        assert_eq!(node.inner.label(), None);
    }

    #[test]
    fn dis_002_glyph_table_has_one_glyph_per_style_and_direction() {
        let styles = [
            SeparatorStyle::Line,
            SeparatorStyle::Thick,
            SeparatorStyle::Double,
            SeparatorStyle::Dashed,
            SeparatorStyle::Dotted,
        ];
        for style in styles {
            assert_eq!(
                UnicodeWidthStr::width(glyph(style, false).to_string().as_str()),
                1
            );
            assert_eq!(
                UnicodeWidthStr::width(glyph(style, true).to_string().as_str()),
                1
            );
        }
        assert_eq!(glyph(SeparatorStyle::Line, false), '─');
        assert_eq!(glyph(SeparatorStyle::Line, true), '│');
    }
}
