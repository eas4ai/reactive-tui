//! The form (docs/spec/input-widgets.md, CTL-007 to CTL-010): labelled
//! controls in rows, the labels beside them, or above them when the width
//! is short. The screen reader is told which label and description belong
//! to each control.

use super::look;
use crate::accessibility::{Node, Role};
use crate::component::{Component, Element, FieldPart, FieldRow, LayoutInfo, LayoutType, Props};
use std::any::Any;
use unicode_width::UnicodeWidthStr;

/// The cells a control needs beside a label column before the labels go
/// above it (CTL-008).
const CONTROL_CELLS: usize = 10;

/// One row of a form: a label, the control it names, an optional
/// description under the control, and whether the control is required.
#[derive(Clone, Debug, PartialEq)]
pub struct FormField {
    /// The name of the field, shown beside or above the control.
    pub label: String,
    /// The control the field holds.
    pub control: Element,
    /// The text under the control, when set.
    pub description: Option<String>,
    /// Whether the field must be filled in; its label gets a `*`.
    pub required: bool,
}

impl FormField {
    /// A field named `label` whose control is `control`.
    pub fn new(label: impl Into<String>, control: Element) -> Self {
        Self {
            label: label.into(),
            control,
            description: None,
            required: false,
        }
    }

    /// Mark the field required: its label is followed by a `*`.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set the text that stands under the control.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
}

/// Where a form's labels stand.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FormLabels {
    /// Beside the controls, in a column (the default).
    #[default]
    Beside,
    /// Above each control.
    Above,
}

/// The settings of a form.
#[derive(Clone, Debug, PartialEq)]
pub struct FormProps {
    /// The fields, in the order Tab takes them.
    pub fields: Vec<FormField>,
    /// Where the labels stand, beside the controls by default.
    pub labels: FormLabels,
    /// The width of the label column; the widest label when unset.
    pub label_width: Option<u16>,
    /// The name the screen reader hears for the form.
    pub aria_label: Option<String>,
}

impl Props for FormProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A form. It reads the width its layout allots it, so it can put the labels
/// above when the width is short.
pub struct Form {
    viewport: Option<LayoutInfo>,
}

impl Form {
    /// Create a form from its props
    pub fn new(_props: FormProps) -> Self {
        Self { viewport: None }
    }

    /// The width of the label column: the widest label, plus one cell for
    /// the required mark when a field is required, or the props' width.
    fn label_column(props: &FormProps) -> usize {
        props.label_width.map(usize::from).unwrap_or_else(|| {
            let widest = props
                .fields
                .iter()
                .map(|field| UnicodeWidthStr::width(field.label.as_str()))
                .max()
                .unwrap_or(0);
            widest + usize::from(props.fields.iter().any(|field| field.required))
        })
    }

    /// Whether the labels stand above: when the props ask for it, or when
    /// the allotted width cannot hold the label column, one cell and ten
    /// cells of control (CTL-008).
    fn labels_above(&self, props: &FormProps) -> bool {
        if props.labels == FormLabels::Above {
            return true;
        }
        self.viewport.is_some_and(|bounds| {
            (bounds.content_size().0 as usize) < Self::label_column(props) + 1 + CONTROL_CELLS
        })
    }
}

impl Component for Form {
    type Props = FormProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Form::new(props)
    }

    fn layout(
        &mut self,
        bounds: LayoutInfo,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        let before = self.labels_above(props);
        self.viewport = Some(bounds);
        before != self.labels_above(props)
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        let above = self.labels_above(props);
        let column = Self::label_column(props);
        let rows = props
            .fields
            .iter()
            .map(|field| field_row(field, column, above))
            .collect();
        let mut node = Node::new(Role::Form);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        Element::layout(LayoutType::Flex)
            .class("flex flex-col w-full")
            .children(rows)
            .with_accessibility(node)
    }
}

/// The label of a field, in a row of `width` cells when given: its text in
/// `foreground`, then its required mark in `text-error` with no space.
fn label_element(field: &FormField, width: Option<usize>) -> Element {
    let class = width.map_or_else(String::new, |cells| format!("w-{cells}"));
    look::row_of(
        look::pieces(&[
            (field.label.as_str(), look::LABEL),
            (if field.required { "*" } else { "" }, look::REQUIRED_MARK),
        ]),
        &class,
    )
    .with_field_part(FieldPart::Label)
}

/// The control of a field, filling the width its row leaves it.
fn control_element(field: &FormField, fill: &str) -> Element {
    Element::layout(LayoutType::Flex)
        .class(format!("flex flex-col {fill}"))
        .with_child(field.control.clone())
        .with_field_part(FieldPart::Control)
}

/// The description of a field, in `text-muted`.
fn description_element(text: &str) -> Element {
    Element::text(text)
        .class(format!("shrink-0 whitespace-pre {}", look::DESCRIPTION))
        .with_field_part(FieldPart::Description)
}

/// One field's row. With the labels beside, the label column of `column`
/// cells, a gap and the control share a line, and the description stands
/// under the control. With the labels above, the label, the control and the
/// description stand one under another, the description at column 0.
fn field_row(field: &FormField, column: usize, above: bool) -> Element {
    let mut rows = if above {
        vec![
            label_element(field, None),
            control_element(field, "w-full min-w-0"),
        ]
    } else {
        vec![Element::layout(LayoutType::Flex)
            .class("flex flex-row w-full")
            .children(vec![
                label_element(field, Some(column)),
                look::blank(1),
                control_element(field, "flex-1 min-w-0"),
            ])]
    };
    if let Some(text) = &field.description {
        let description = description_element(text);
        rows.push(if above {
            description
        } else {
            // The description starts at the control's first column.
            Element::layout(LayoutType::Flex)
                .class("flex flex-row w-full")
                .children(vec![look::blank(column + 1), description])
        });
    }
    Element::layout(LayoutType::Flex)
        .class("flex flex-col w-full")
        .children(rows)
        .with_field_row(FieldRow {
            required: field.required,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ElementType;

    /// Every element of `element` in pre-order.
    fn walk(element: &Element, out: &mut Vec<Element>) {
        out.push(element.clone());
        for child in &element.children {
            walk(child, out);
        }
    }

    /// The field parts of `element`, in the order the tree gives them.
    fn parts(element: &Element) -> Vec<FieldPart> {
        let mut all = Vec::new();
        walk(element, &mut all);
        all.iter()
            .filter_map(|node| {
                node.metadata
                    .accessibility_options
                    .as_deref()
                    .and_then(|options| options.field_part)
            })
            .collect()
    }

    fn named_control(name: &str) -> Element {
        Element::text(name).class(name)
    }

    /// CTL-009: the fields' controls come in the order the fields were added,
    /// so Tab moves from one to the next.
    #[test]
    fn ctl_009_a_forms_fields_follow_their_order() {
        let props = FormProps {
            fields: vec![
                FormField::new("First", named_control("a")),
                FormField::new("Second", named_control("b")),
                FormField::new("Third", named_control("c")),
            ],
            labels: FormLabels::Beside,
            label_width: None,
            aria_label: None,
        };
        let element = Form::new(props.clone()).render(&props, &());
        let mut all = Vec::new();
        walk(&element, &mut all);
        let order: Vec<String> = all
            .iter()
            .filter(|node| {
                node.metadata
                    .accessibility_options
                    .as_deref()
                    .and_then(|options| options.field_part)
                    == Some(FieldPart::Control)
            })
            .map(|wrapper| wrapper.children[0].class.clone().unwrap_or_default())
            .collect();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    /// CTL-010: the form root is a Form node labelled by its aria label; each
    /// field's parts are marked, and a required row is marked required.
    #[test]
    fn ctl_010_a_form_marks_its_parts_for_the_label_link() {
        let props = FormProps {
            fields: vec![
                FormField::new("Name", named_control("name")).required(),
                FormField::new("Email", named_control("email")).description("Work address"),
            ],
            labels: FormLabels::Beside,
            label_width: None,
            aria_label: Some("Account".to_owned()),
        };
        let element = Form::new(props.clone()).render(&props, &());
        let root = element
            .metadata
            .accessibility
            .as_ref()
            .expect("the form's node");
        assert_eq!(root.inner.role(), Role::Form);
        assert_eq!(root.inner.label(), Some("Account"));
        assert_eq!(
            parts(&element),
            vec![
                FieldPart::Label,
                FieldPart::Control,
                FieldPart::Label,
                FieldPart::Control,
                FieldPart::Description,
            ]
        );
        let mut all = Vec::new();
        walk(&element, &mut all);
        let rows: Vec<bool> = all
            .iter()
            .filter_map(|node| {
                node.metadata
                    .accessibility_options
                    .as_deref()
                    .and_then(|options| options.field_row)
                    .map(|row| row.required)
            })
            .collect();
        assert_eq!(rows, vec![true, false]);
        assert!(matches!(element.element_type, ElementType::Layout(_)));
    }
}
