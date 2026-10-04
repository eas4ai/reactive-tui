use super::{Element, ElementType, LayoutType};
use crate::layout::paint_tree::NodeSpec;
use std::borrow::Cow;

pub(crate) struct PaintSpec {
    pub root: NodeSpec<'static>,
    pub styles: Vec<crate::layout::style::StyleBuilder>,
    pub images: Vec<Option<std::sync::Arc<crate::widgets::display::image::paint::ImagePaint>>>,
    pub image_fallbacks: Vec<Option<u32>>,
    pub cursors: Vec<Option<super::element::TextCursor>>,
    pub cells: Vec<Option<std::sync::Arc<crate::layout::paint_tree::cells::CellGrid>>>,
    /// The canvas each element shows; it takes no part in layout.
    #[cfg(feature = "wgpu-graphics")]
    pub canvases: Vec<Option<std::sync::Arc<crate::graphics::CanvasPaint>>>,
    /// The pixel look each element draws where the terminal takes pixels
    /// (PIX-001), and a number that names the element across frames so its
    /// picture is kept (PIX-002): its id, else its component instance, else
    /// its place in the tree.
    #[cfg(feature = "wgpu-graphics")]
    pub looks: Vec<Option<std::sync::Arc<crate::graphics::look::Look>>>,
    #[cfg(feature = "wgpu-graphics")]
    pub look_ids: Vec<u64>,
}

pub(crate) fn resolve_viewport_styles(
    element: &mut Element,
    width: u16,
) -> crate::error::Result<()> {
    if let Some(style) = &element.metadata.styles {
        if let Some(resolved) = style.resolve_viewport(width)? {
            element.metadata.styles = Some(std::sync::Arc::new(resolved));
        }
    }
    for child in &mut element.children {
        resolve_viewport_styles(child, width)?;
    }
    Ok(())
}

pub(crate) fn element_to_paintspec(element: &Element) -> crate::error::Result<PaintSpec> {
    fn collect(
        element: &Element,
        styles: &mut Vec<crate::layout::style::StyleBuilder>,
        images: &mut Vec<Option<std::sync::Arc<crate::widgets::display::image::paint::ImagePaint>>>,
        fallbacks: &mut Vec<Option<u32>>,
        fallback: Option<u32>,
        cursors: &mut Vec<Option<super::element::TextCursor>>,
        cells: &mut Vec<Option<std::sync::Arc<crate::layout::paint_tree::cells::CellGrid>>>,
    ) -> crate::error::Result<()> {
        let fallback = element.metadata.image_fallback.or(fallback);
        images.push(element.metadata.image.clone());
        fallbacks.push(fallback);
        cursors.push(element.metadata.text_cursor);
        cells.push(element.metadata.cells.clone());
        styles.push(match &element.metadata.paint_style {
            Some(style) => style.restore()?,
            None => element_style(element)?,
        });
        for child in &element.children {
            collect(child, styles, images, fallbacks, fallback, cursors, cells)?;
        }
        Ok(())
    }
    /// The canvases of `element` and its descendants, in the order
    /// `collect` visits them.
    #[cfg(feature = "wgpu-graphics")]
    fn canvases(
        element: &Element,
        found: &mut Vec<Option<std::sync::Arc<crate::graphics::CanvasPaint>>>,
    ) {
        found.push(element.metadata.canvas.clone());
        for child in &element.children {
            canvases(child, found);
        }
    }
    /// The looks of `element` and its descendants with the numbers that
    /// name them, in the order `collect` visits them: a control's own look,
    /// else the look its classes describe (PIX-003). A key names an element
    /// among its siblings only, so a keyed element is named by its key under
    /// its parent's name: two looks with one key under two parents are two
    /// looks, each with its own picture (PIX-002, GFX-003).
    #[cfg(feature = "wgpu-graphics")]
    fn looks(
        element: &Element,
        parent: u64,
        path: &mut Vec<usize>,
        found: &mut Vec<Option<std::sync::Arc<crate::graphics::look::Look>>>,
        ids: &mut Vec<u64>,
    ) {
        use std::hash::{Hash, Hasher};
        let look = element.metadata.look.clone().or_else(|| {
            element
                .class
                .as_deref()
                .and_then(crate::graphics::look::Look::from_classes)
                .map(std::sync::Arc::new)
        });
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        match (&element.key, element.metadata.component_instances.first()) {
            (Some(key), _) => (0u8, parent, key).hash(&mut hasher),
            (None, Some(instance)) => (1u8, instance).hash(&mut hasher),
            (None, None) => (2u8, &*path).hash(&mut hasher),
        }
        let id = hasher.finish();
        found.push(look);
        ids.push(id);
        for (index, child) in element.children.iter().enumerate() {
            path.push(index);
            looks(child, id, path, found, ids);
            path.pop();
        }
    }
    let mut styles = Vec::new();
    let mut images = Vec::new();
    let mut image_fallbacks = Vec::new();
    let mut cursors = Vec::new();
    let mut cells = Vec::new();
    collect(
        element,
        &mut styles,
        &mut images,
        &mut image_fallbacks,
        None,
        &mut cursors,
        &mut cells,
    )?;
    #[cfg(feature = "wgpu-graphics")]
    let looks_and_ids = {
        let (mut found, mut ids, mut path) = (Vec::new(), Vec::new(), Vec::new());
        looks(element, 0, &mut path, &mut found, &mut ids);
        (found, ids)
    };
    Ok(PaintSpec {
        root: element_to_nodespec(element),
        styles,
        images,
        image_fallbacks,
        cursors,
        cells,
        #[cfg(feature = "wgpu-graphics")]
        canvases: {
            let mut found = Vec::new();
            canvases(element, &mut found);
            found
        },
        #[cfg(feature = "wgpu-graphics")]
        looks: looks_and_ids.0,
        #[cfg(feature = "wgpu-graphics")]
        look_ids: looks_and_ids.1,
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
