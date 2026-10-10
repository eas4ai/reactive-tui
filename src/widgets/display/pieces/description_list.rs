//! The description list display piece (docs/spec/display-pieces.md), carrying DIS-001,
//! DIS-002, DIS-004.
//!
//! A description list shows pairs of a label and a value. Its labels are `text-muted`
//! and its values `foreground`. Each label column is as wide as the longest label. It
//! fills the width its parent allots, and it stacks each label over its value, lays
//! the pairs in columns, or draws a box of `border` around them. The box's sides need
//! the width, which the component learns when the App lays it out.

use std::any::Any;

use unicode_width::UnicodeWidthStr;

use crate::accessibility::{Node, Role};
use crate::builder::core::{div, span};
use crate::component::{Component, Element, LayoutInfo, Props};
use crate::widgets::display::look;
use crate::widgets::display::pieces::separator::sized;

/// How a description list lays out each pair (DIS-002).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DescriptionLayout {
    /// The label and its value side by side, the labels in one column.
    #[default]
    Row,
    /// The label stacked over its value.
    Vertical,
}

/// The props of a description list (DIS-001, DIS-002, DIS-004).
#[derive(Clone, Debug, PartialEq)]
pub struct DescriptionListProps {
    /// The pairs, in order: a label and its value.
    pub pairs: Vec<(String, String)>,
    /// How each pair is laid out.
    pub layout: DescriptionLayout,
    /// The number of columns the pairs are spread over (1 unless set). A bordered
    /// list is one column.
    pub columns: u16,
    /// Whether a box of `border` is drawn around the list.
    pub bordered: bool,
    /// Classes of the list. A `w-`, `h-` class sets its size in place of the fill default.
    pub classes: Vec<String>,
}

impl Default for DescriptionListProps {
    fn default() -> Self {
        Self {
            pairs: Vec::new(),
            layout: DescriptionLayout::Row,
            columns: 1,
            bordered: false,
            classes: Vec::new(),
        }
    }
}

impl Props for DescriptionListProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The description list component. It keeps the width its layout gave it, which a
/// bordered list needs for its sides.
pub struct DescriptionList {
    width: usize,
}

impl Component for DescriptionList {
    type Props = DescriptionListProps;
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

/// The element of a description list `width` cells wide. An unbordered list fills
/// the width it is given through its classes and ignores `width` otherwise.
pub(crate) fn render(props: &DescriptionListProps, width: usize) -> Element {
    if props.bordered {
        return bordered(props, width);
    }
    let columns = usize::from(props.columns.max(1));
    let per_column = props.pairs.len().div_ceil(columns).max(1);
    let label_width = label_width(props);
    let column_elements: Vec<Element> = props
        .pairs
        .chunks(per_column)
        .map(|chunk| {
            div()
                .class("flex-col flex-1")
                .children(
                    chunk
                        .iter()
                        .map(|pair| pair_element(pair, props.layout, label_width))
                        .collect(),
                )
                .build()
        })
        .collect();
    let direction = if columns > 1 { "flex-row" } else { "flex-col" };
    let classes = sized(direction, &props.classes, &[("w-", "w-full")]);
    div()
        .class(&classes)
        .children(column_elements)
        .build()
        .with_accessibility(Node::new(Role::DescriptionList))
}

/// The widest label, in cells.
fn label_width(props: &DescriptionListProps) -> usize {
    props
        .pairs
        .iter()
        .map(|(label, _)| UnicodeWidthStr::width(label.as_str()))
        .max()
        .unwrap_or(0)
}

/// One pair: its label, in `text-muted`, and its value, in `foreground`. Each
/// is a term or a definition for the screen reader.
fn pair_element(pair: &(String, String), layout: DescriptionLayout, label_width: usize) -> Element {
    let (label, value) = pair;
    let direction = match layout {
        DescriptionLayout::Row => "flex-row",
        DescriptionLayout::Vertical => "flex-col",
    };
    let label_width = match layout {
        DescriptionLayout::Row => label_width,
        DescriptionLayout::Vertical => 0,
    };
    div()
        .class(direction)
        .child(term_element(label, label_width))
        .child(definition_element(value))
        .build()
}

fn term_element(label: &str, width: usize) -> Element {
    let mut term = Node::new(Role::Term);
    term.set_label(label.to_string());
    span()
        .class(look::MUTED)
        .text(&pad(label, width))
        .build()
        .with_accessibility(term)
}

/// The term of a box row and the cells it paints: the label padded to `width`.
fn term_part(label: &str, width: usize) -> (Element, usize) {
    let painted = UnicodeWidthStr::width(pad(label, width).as_str());
    (term_element(label, width), painted)
}

fn definition_element(value: &str) -> Element {
    let mut definition = Node::new(Role::Definition);
    definition.set_label(value.to_string());
    span()
        .class(look::TEXT)
        .text(value)
        .build()
        .with_accessibility(definition)
}

/// A bordered list: a box of `border` as wide as `width`, one row of text per
/// label and per value inside it. The label column is as wide as the longest label.
fn bordered(props: &DescriptionListProps, width: usize) -> Element {
    let inner = width.saturating_sub(2);
    let label_width = label_width(props);
    let mut rows = vec![span()
        .class(look::BORDER)
        .text(&format!("┌{}┐", "─".repeat(inner)))
        .build()];
    for (label, value) in &props.pairs {
        match props.layout {
            DescriptionLayout::Row => rows.push(boxed_row(
                vec![
                    term_part(label, label_width),
                    (
                        definition_element(value),
                        UnicodeWidthStr::width(value.as_str()),
                    ),
                ],
                inner,
            )),
            DescriptionLayout::Vertical => {
                rows.push(boxed_row(vec![term_part(label, 0)], inner));
                rows.push(boxed_row(
                    vec![(
                        definition_element(value),
                        UnicodeWidthStr::width(value.as_str()),
                    )],
                    inner,
                ));
            }
        }
    }
    rows.push(
        span()
            .class(look::BORDER)
            .text(&format!("└{}┘", "─".repeat(inner)))
            .build(),
    );
    let classes = sized("flex-col", &props.classes, &[("w-", "w-full")]);
    div()
        .class(&classes)
        .children(rows)
        .build()
        .with_accessibility(Node::new(Role::DescriptionList))
}

/// One row inside a box: the side, the parts with their cell widths, the fill
/// that reaches the other side, and the other side.
fn boxed_row(parts: Vec<(Element, usize)>, inner: usize) -> Element {
    let used: usize = parts.iter().map(|(_, width)| *width).sum();
    let mut children = vec![span().class(look::BORDER).text("│").build()];
    children.extend(parts.into_iter().map(|(element, _)| element));
    let fill = inner.saturating_sub(used);
    children.push(span().text(&" ".repeat(fill)).build());
    children.push(span().class(look::BORDER).text("│").build());
    div().class("flex-row").children(children).build()
}

/// `text` padded with spaces to `width` cells, one space more so the value
/// starts after a gap.
fn pad(text: &str, width: usize) -> String {
    format!("{text}{}", " ".repeat(pad_len(text, width)))
}

/// The number of spaces `pad` adds to `text` for a column of `width` cells.
fn pad_len(text: &str, width: usize) -> usize {
    width.saturating_sub(UnicodeWidthStr::width(text)) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dis_004_description_list_is_a_list_whose_pairs_are_terms_and_definitions() {
        let props = DescriptionListProps {
            pairs: vec![("Name".into(), "Ada".into())],
            ..DescriptionListProps::default()
        };
        let element = render(&props, 0);
        let list = element.metadata.accessibility.as_ref().expect("a node");
        assert_eq!(list.role(), Role::DescriptionList);
        let pair = &element.children[0].children[0];
        let term = pair.children[0].metadata.accessibility.as_ref();
        assert_eq!(term.map(Node::role), Some(Role::Term));
        let value = pair.children[1].metadata.accessibility.as_ref();
        assert_eq!(value.map(Node::role), Some(Role::Definition));
    }

    #[test]
    fn dis_002_the_label_column_is_as_wide_as_the_longest_label() {
        assert_eq!(pad("Id", 6), "Id     ");
        assert_eq!(pad("Status", 6), "Status ");
    }
}
