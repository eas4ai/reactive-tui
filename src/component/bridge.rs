use super::{Element, ElementType, LayoutType};
use crate::layout::paint_tree::NodeSpec;
use std::borrow::Cow;

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
