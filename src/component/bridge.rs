use super::{Element, ElementType, LayoutType};
use crate::layout::paint_tree::NodeSpec;
use std::borrow::Cow;

pub(crate) struct PaintSpec {
    pub root: NodeSpec<'static>,
    pub styles: Vec<crate::layout::style::StyleBuilder>,
    pub images: Vec<Option<std::sync::Arc<crate::widgets::display::image::paint::ImagePaint>>>,
    pub image_fallbacks: Vec<Option<u32>>,
    pub cursors: Vec<Option<super::element::TextCursor>>,
    /// The URL each element's text links to: its own, else its nearest
    /// ancestor's.
    pub links: Vec<Option<std::sync::Arc<str>>>,
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
    #[allow(clippy::too_many_arguments)]
    fn collect(
        element: &Element,
        styles: &mut Vec<crate::layout::style::StyleBuilder>,
        images: &mut Vec<Option<std::sync::Arc<crate::widgets::display::image::paint::ImagePaint>>>,
        fallbacks: &mut Vec<Option<u32>>,
        fallback: Option<u32>,
        cursors: &mut Vec<Option<super::element::TextCursor>>,
        cells: &mut Vec<Option<std::sync::Arc<crate::layout::paint_tree::cells::CellGrid>>>,
        links: &mut Links,
    ) -> crate::error::Result<()> {
        let fallback = element.metadata.image_fallback.or(fallback);
        images.push(element.metadata.image.clone());
        fallbacks.push(fallback);
        cursors.push(element.metadata.text_cursor);
        cells.push(element.metadata.cells.clone());
        // An element without a link of its own links to its parent's URL.
        let link = element
            .metadata
            .hyperlink
            .clone()
            .or_else(|| links.inherited.clone());
        links.found.push(link.clone());
        let outer = std::mem::replace(&mut links.inherited, link);
        styles.push(match &element.metadata.paint_style {
            Some(style) => style.restore()?,
            None => element_style(element)?,
        });
        for child in &element.children {
            collect(
                child, styles, images, fallbacks, fallback, cursors, cells, links,
            )?;
        }
        links.inherited = outer;
        Ok(())
    }
    /// The URL each element's text links to, in the order `collect` visits
    /// them, and the URL of the element being visited.
    #[derive(Default)]
    struct Links {
        found: Vec<Option<std::sync::Arc<str>>>,
        inherited: Option<std::sync::Arc<str>>,
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
    /// else the look its classes describe (PIX-003), focused while the
    /// element holds the focus (PIX-008). A key names an element
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
                .map(|look| {
                    std::sync::Arc::new(crate::graphics::look::Look {
                        focused: element.metadata.focused,
                        ..look
                    })
                })
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
    let mut links = Links::default();
    collect(
        element,
        &mut styles,
        &mut images,
        &mut image_fallbacks,
        None,
        &mut cursors,
        &mut cells,
        &mut links,
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
        links: links.found,
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

/// The style of `element`: its explicit styles under its utility classes
/// and its inline declarations. A frame asks this of every element in
/// each of its passes over the tree, and most elements come back the same
/// frame after frame, so an element with explicit styles and classes and
/// nothing else keeps its answer on this thread, found by the class string
/// and the styles' bytes together, which alone decide it; the class cache
/// serves an element with classes alone.
pub(crate) fn element_style(
    element: &Element,
) -> crate::error::Result<crate::layout::style::StyleBuilder> {
    let default_class = match element.element_type {
        ElementType::Layout(LayoutType::Grid) => "grid",
        _ => "",
    };
    let class = element.class.as_deref().unwrap_or(default_class);
    let plain = element.metadata.gradient.is_none()
        && element.metadata.gradient_border.is_none()
        && element.metadata.inline_styles.is_none();
    let key = match (&element.metadata.styles, plain) {
        (Some(styles), true) if !class.is_empty() => styles.bytes().map(|bytes| {
            let mut key = Vec::with_capacity(class.len() + 1 + bytes.len());
            key.extend_from_slice(class.as_bytes());
            key.push(0);
            key.extend_from_slice(bytes);
            key
        }),
        _ => None,
    };
    if let Some(key) = &key {
        let generation = crate::theme::Theme::generation();
        let found = STYLED.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.0 != generation {
                cache.1.clear();
                cache.0 = generation;
            }
            cache.1.get(key.as_slice()).cloned()
        });
        if let Some(found) = found {
            return Ok(found);
        }
    }
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
    let styled = crate::layout::css::apply_utility_classes(class, base);
    let styled = match &element.metadata.inline_styles {
        Some(declarations) => styled.with_inline_declarations(declarations)?,
        None => styled,
    };
    if let Some(key) = key {
        // Resolved under the theme in force when the key was read: a theme
        // that changed meanwhile empties the cache on the next read.
        let generation = crate::theme::Theme::generation();
        STYLED.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.0 == generation {
                cache.1.put(key, styled.clone());
            }
        });
    }
    Ok(styled)
}

// The styles resolved on this thread for elements with explicit styles
// and classes, by class and bytes, under one theme generation.
thread_local! {
    static STYLED: std::cell::RefCell<(u64, lru::LruCache<Vec<u8>, crate::layout::style::StyleBuilder>)> =
        std::cell::RefCell::new((
            crate::theme::Theme::generation(),
            lru::LruCache::new(std::num::NonZeroUsize::new(4096).expect("Cache size must be non-zero")),
        ));
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

#[cfg(test)]
mod style_cache_tests {
    use super::*;
    use crate::layout::style::StyleBuilder;

    /// An absolute box `top` rows down with the utility classes `class`.
    fn placed(top: f32, class: &str) -> Element {
        crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(StyleBuilder::new().position_absolute().inset_top(top))
            .class(class)
            .build()
    }

    // The class resolves through no theme, so a theme another test sets
    // meanwhile changes nothing here.
    #[test]
    fn two_elements_with_one_class_string_keep_their_own_explicit_styles() {
        let first = element_style(&placed(1.0, "p-1 overflow-hidden")).expect("a style");
        let second = element_style(&placed(2.0, "p-1 overflow-hidden")).expect("a style");
        let again = element_style(&placed(1.0, "p-1 overflow-hidden")).expect("a style");
        assert_eq!(first, again, "the same element resolves the same twice");
        assert_ne!(
            first.build().inset.top,
            second.build().inset.top,
            "a cached answer must not serve an element with other explicit styles"
        );
    }

    // Another test that sets the theme while this one runs would move the
    // generation under both resolutions alike, so each is read fresh: the
    // check is that the second answer is the light theme's, not the cached
    // dark one.
    #[test]
    fn a_theme_change_resolves_a_cached_element_again() {
        let element = placed(3.0, "text-primary");
        crate::theme::Theme::set_active(crate::theme::dark_theme());
        let dark = element_style(&element).expect("a style");
        crate::theme::Theme::set_active(crate::theme::light_theme());
        let light = element_style(&element).expect("a style");
        let fresh = crate::layout::css::apply_utility_classes(
            "text-primary",
            StyleBuilder::new().position_absolute().inset_top(3.0),
        );
        assert_eq!(
            light.fg_rgba, fresh.fg_rgba,
            "after a theme change the element resolves through the theme now active, not the cached answer"
        );
        assert!(
            dark.fg_rgba.is_some() && light.fg_rgba.is_some(),
            "text-primary gives both a color"
        );
    }
}
