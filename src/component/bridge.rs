use super::{Element, ElementType, LayoutType};
use crate::layout::paint_tree::NodeSpec;
use std::borrow::Cow;

pub(crate) struct PaintSpec {
    pub root: NodeSpec<'static>,
    pub styles: Vec<crate::layout::style::StyleBuilder>,
    pub images: Vec<Option<std::sync::Arc<crate::widgets::display::image::paint::ImagePaint>>>,
    pub image_fallbacks: Vec<Option<u32>>,
    pub cursors: Vec<Option<super::element::TextCursor>>,
}

pub(crate) fn element_to_paintspec(element: &Element) -> crate::error::Result<PaintSpec> {
    fn collect(
        element: &Element,
        styles: &mut Vec<crate::layout::style::StyleBuilder>,
        images: &mut Vec<Option<std::sync::Arc<crate::widgets::display::image::paint::ImagePaint>>>,
        fallbacks: &mut Vec<Option<u32>>,
        fallback: Option<u32>,
        cursors: &mut Vec<Option<super::element::TextCursor>>,
    ) -> crate::error::Result<()> {
        let fallback = element.metadata.image_fallback.or(fallback);
        images.push(element.metadata.image.clone());
        fallbacks.push(fallback);
        cursors.push(element.metadata.text_cursor);
        styles.push(match &element.metadata.paint_style {
            Some(style) => style.restore()?,
            None => element_style(element)?,
        });
        for child in &element.children {
            collect(child, styles, images, fallbacks, fallback, cursors)?;
        }
        Ok(())
    }
    let mut styles = Vec::new();
    let mut images = Vec::new();
    let mut image_fallbacks = Vec::new();
    let mut cursors = Vec::new();
    collect(
        element,
        &mut styles,
        &mut images,
        &mut image_fallbacks,
        None,
        &mut cursors,
    )?;
    Ok(PaintSpec {
        root: element_to_nodespec(element),
        styles,
        images,
        image_fallbacks,
        cursors,
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
    let styled = crate::layout::css::apply_utility_classes(
        element.class.as_deref().unwrap_or(default_class),
        base,
    );
    match &element.metadata.inline_styles {
        Some(declarations) => styled.with_inline_declarations(declarations),
        None => Ok(styled),
    }
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
