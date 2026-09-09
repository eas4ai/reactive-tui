use super::{Element, ElementType, LayoutType};
use crate::layout::paint_tree::NodeSpec;
use std::borrow::Cow;

pub(crate) struct PaintSpec {
    pub root: NodeSpec<'static>,
    pub styles: Vec<crate::layout::style::StyleBuilder>,
}

pub(crate) fn element_to_paintspec(element: &Element) -> crate::error::Result<PaintSpec> {
    fn collect(
        element: &Element,
        styles: &mut Vec<crate::layout::style::StyleBuilder>,
    ) -> crate::error::Result<()> {
        styles.push(match &element.metadata.paint_style {
            Some(style) => style.restore()?,
            None => element_style(element)?,
        });
        for child in &element.children {
            collect(child, styles)?;
        }
        Ok(())
    }
    let mut styles = Vec::new();
    collect(element, &mut styles)?;
    Ok(PaintSpec {
        root: element_to_nodespec(element),
        styles,
    })
}

pub(crate) fn element_style(
    element: &Element,
) -> crate::error::Result<crate::layout::style::StyleBuilder> {
    let mut base = element
        .metadata
        .styles
        .as_deref()
        .map(|style| style.restore())
        .transpose()?
        .unwrap_or_default();
    if element.metadata.gradient.is_some() {
        base.gradient = element.metadata.gradient.clone();
    }
    if element.metadata.gradient_border.is_some() {
        base.gradient_border = element.metadata.gradient_border.clone();
    }
    let default_class = match element.element_type {
        ElementType::Layout(LayoutType::Grid) => "grid",
        _ => "",
    };
    Ok(crate::layout::css::apply_utility_classes(
        element.class.as_deref().unwrap_or(default_class),
        base,
    ))
}

/// Convert an Element tree to a NodeSpec tree that our painter/layout understands.
pub fn element_to_nodespec(elem: &Element) -> NodeSpec<'static> {
    let class_str: String = match (&elem.element_type, elem.class.as_deref()) {
        (ElementType::Layout(LayoutType::Flex), None) => "flex".to_string(),
        (ElementType::Layout(LayoutType::Grid), None) => "grid".to_string(),
        // Default to provided class or empty
        (_, Some(c)) => c.to_string(),
        _ => String::new(),
    };
    let class_cow: Cow<'static, str> = Cow::Owned(class_str);

    // Map text content if this is a Text element
    let text_cow: Option<Cow<'static, str>> = match &elem.element_type {
        ElementType::Text(s) => Some(Cow::Owned(s.clone())),
        _ => None,
    };

    // Recurse children
    let mut children_specs: Vec<NodeSpec<'static>> = Vec::with_capacity(elem.children.len());
    for child in &elem.children {
        children_specs.push(element_to_nodespec(child));
    }

    NodeSpec {
        class: class_cow,
        text: text_cow,
        children: children_specs,
    }
}
