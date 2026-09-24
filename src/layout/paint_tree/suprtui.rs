//! Grapheme painting over the existing CSS/Taffy layout metadata.

mod cursor;
pub(crate) mod images;

use super::NodeSpec;
use super::{build_nodes_inherited, transform::Affine, NodePaint};
use crate::core::surface::{Attr, Rgba};
use crate::error::{ReactiveError, Result};
use ::suprtui::ansi::{self, TextAttributes};
use ::suprtui::buffer::OptimizedBuffer;
use std::collections::HashMap;
use std::sync::Arc;
use taffy::{geometry::Size, prelude::NodeId, style::Overflow, AvailableSpace, TaffyTree};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, PartialEq)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    fn intersect(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }
}

struct PaintNode {
    mask: Option<Arc<ClipMask>>,
    transform: Affine,
    local: Rect,
    parent_opacity: f32,
    element_index: usize,
    id: NodeId,
    bounds: Rect,
    clip: Rect,
    z: i32,
}

/// Layout results kept across frames: the spec they came from, the Taffy
/// tree, the paint records and the placed nodes. A frame whose layout inputs
/// match the cached spec at the same size paints from them without laying
/// out again (PNT-004).
#[derive(Default)]
pub(crate) struct LayoutCache {
    spec: Option<crate::component::bridge::PaintSpec>,
    size: (u32, u32),
    tree: TaffyTree<()>,
    paints: HashMap<NodeId, NodePaint>,
    nodes: Vec<PaintNode>,
    /// Layouts computed with this cache: counted where Taffy runs, so a
    /// test observes the layout work itself, not the reuse decision.
    runs: u64,
}

/// Whether two specs lay out the same: the same node tree with the same
/// classes and text, the same styles, images and cell grids by identity.
/// Text cursors do not affect layout.
fn same_layout_inputs(
    a: &crate::component::bridge::PaintSpec,
    b: &crate::component::bridge::PaintSpec,
) -> bool {
    fn same_node(a: &NodeSpec<'_>, b: &NodeSpec<'_>) -> bool {
        a.class == b.class
            && a.text == b.text
            && a.children.len() == b.children.len()
            && a.children
                .iter()
                .zip(&b.children)
                .all(|(x, y)| same_node(x, y))
    }
    fn same_handles<T>(a: &[Option<Arc<T>>], b: &[Option<Arc<T>>]) -> bool {
        a.len() == b.len()
            && a.iter().zip(b).all(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Arc::ptr_eq(x, y),
                (None, None) => true,
                _ => false,
            })
    }
    same_node(&a.root, &b.root)
        && a.styles == b.styles
        && a.image_fallbacks == b.image_fallbacks
        && same_handles(&a.images, &b.images)
        && same_handles(&a.cells, &b.cells)
}

/// Write a node's element index plus one into every cell it occupies:
/// its bounds within its clip, exact under masks and transforms, in paint
/// order so a later (higher) node overwrites a lower one (PNT-002).
fn mark_hits(hits: &mut [u32], width: u32, node: &PaintNode, inverse_cells: &mut u64) {
    let id = node.element_index as u32 + 1;
    let visible = node.bounds.intersect(node.clip);
    let plain = node.transform.is_translation() && node.mask.is_none();
    for y in visible.top.max(0)..visible.bottom {
        for x in visible.left.max(0)..visible.right {
            if x as u32 >= width {
                break;
            }
            let Some(cell) = hits.get_mut(y as usize * width as usize + x as usize) else {
                return;
            };
            if !plain {
                if !inside_masks(&node.mask, x, y) {
                    continue;
                }
                *inverse_cells += 1;
                let Some((local_x, local_y)) = node.transform.inverse(x as f32, y as f32) else {
                    continue;
                };
                let (local_x, local_y) = (local_x.round() as i32, local_y.round() as i32);
                if local_x < 0
                    || local_x >= node.local.right
                    || local_y < 0
                    || local_y >= node.local.bottom
                {
                    continue;
                }
            }
            *cell = id;
        }
    }
}

/// Lay `spec` out at `size` into `cache`, reusing the cached layout when
/// its inputs are unchanged (PNT-004). Returns whether it was reused.
fn lay_out(
    spec: crate::component::bridge::PaintSpec,
    size: (u32, u32),
    cache: &mut LayoutCache,
) -> Result<bool> {
    let reused = cache.size == size
        && cache
            .spec
            .as_ref()
            .is_some_and(|previous| same_layout_inputs(previous, &spec));
    if !reused {
        let mut tree = TaffyTree::new();
        let mut paints = HashMap::new();
        let root = build_nodes_inherited(
            &mut tree,
            &spec.root,
            &mut paints,
            &crate::layout::text::TextStyle::default(),
            None,
            false,
            &mut spec.styles.iter().cloned(),
        )?;
        let available = Size {
            width: AvailableSpace::Definite(size.0 as f32),
            height: AvailableSpace::Definite(size.1 as f32),
        };
        tree.compute_layout_with_measure(root, available, |known, available, id, _, _| {
            super::measure_text(&paints[&id], known, available)
        })
        .map_err(|error| ReactiveError::layout(format!("SuprTUI layout: {error}")))?;
        cache.runs += 1;
        let screen = Rect {
            left: 0,
            top: 0,
            right: size.0 as i32,
            bottom: size.1 as i32,
        };
        let mut nodes = Vec::new();
        collect(
            &tree,
            &paints,
            root,
            Placement {
                mask: None,
                transform: Affine::default(),
                clip: screen,
                layer: i32::MIN,
                opacity: 1.0,
            },
            &mut nodes,
        )?;
        // Stable sorting preserves parent-before-child and sibling paint order.
        nodes.sort_by_key(|node| node.z);
        cache.tree = tree;
        cache.paints = paints;
        cache.nodes = nodes;
        cache.size = size;
    }
    cache.spec = Some(spec);
    Ok(reused)
}

/// The layout a component receives for `node`: its size, content extent,
/// clip, transform and insets.
fn presented_layout(
    tree: &TaffyTree<()>,
    node: &PaintNode,
) -> Result<crate::backend::PresentedLayout> {
    let layout = tree
        .layout(node.id)
        .map_err(|error| ReactiveError::layout(error.to_string()))?;
    Ok(crate::backend::PresentedLayout {
        element_index: node.element_index,
        layout: crate::component::LayoutInfo {
            size: (node.local.right as f32, node.local.bottom as f32),
            content_extent: (layout.content_size.width, layout.content_size.height),
            clip: crate::event::hit::Bounds::new(
                node.clip.left as f32,
                node.clip.top as f32,
                (node.clip.right - node.clip.left).max(0) as f32,
                (node.clip.bottom - node.clip.top).max(0) as f32,
            ),
            transform: node.transform.coefficients(),
            insets: [
                layout.padding.left + layout.border.left,
                layout.padding.top + layout.border.top,
                layout.padding.right + layout.border.right,
                layout.padding.bottom + layout.border.bottom,
            ],
        },
    })
}

/// The bounds of `node` that remain visible after its clip.
fn painted_node(node: &PaintNode) -> crate::backend::PaintedNode {
    let visible = node.bounds.intersect(node.clip);
    crate::backend::PaintedNode {
        element_index: node.element_index,
        bounds: crate::event::hit::Bounds::new(
            visible.left as f32,
            visible.top as f32,
            (visible.right - visible.left).max(0) as f32,
            (visible.bottom - visible.top).max(0) as f32,
        ),
    }
}

/// Lay `spec` out at `size` without painting and return each node's
/// visible bounds and each component's layout, as a present of the same
/// spec would report them. The App uses it after a resize so components
/// and anchored overlays take their new geometry before the first frame at
/// the new size is painted; a present of the same spec reuses the layout.
pub(crate) fn layout_frame(
    spec: crate::component::bridge::PaintSpec,
    size: (u32, u32),
    cache: &mut LayoutCache,
) -> Result<crate::backend::FrameLayout> {
    lay_out(spec, size, cache)?;
    Ok(crate::backend::FrameLayout {
        nodes: cache.nodes.iter().map(painted_node).collect(),
        layouts: cache
            .nodes
            .iter()
            .map(|node| presented_layout(&cache.tree, node))
            .collect::<Result<_>>()?,
    })
}

pub(crate) fn paint_frame(
    spec: crate::component::bridge::PaintSpec,
    target: &mut OptimizedBuffer<'_>,
    hits: &mut [u32],
    cache: &mut LayoutCache,
    image_options: crate::backend::ImageOutputOptions,
) -> Result<crate::backend::PresentedGeometry> {
    target.clear(ansi::rgb_color(0, 0, 0, 255), None);
    let size = (target.width(), target.height());
    let reused = lay_out(spec, size, cache)?;
    let spec = cache.spec.as_ref().expect("lay_out stores the spec");
    let (tree, paints, nodes) = (&cache.tree, &cache.paints, &cache.nodes);
    let mut geometry = Vec::with_capacity(nodes.len());
    let mut layouts = Vec::with_capacity(nodes.len());
    let selected: std::collections::HashSet<u32> = spec
        .images
        .iter()
        .flatten()
        .filter(|image| image_options.protocol(image.mode).is_some())
        .map(|image| image.id)
        .collect();
    let mut images = images::Layers::new(target.width() as usize, target.height() as usize);
    let mut cursor = cursor::Layer::default();
    let mut inverse_cells = 0;
    for node in nodes {
        let fallback =
            spec.image_fallbacks[node.element_index].is_some_and(|id| selected.contains(&id));
        if !fallback {
            paint_node(
                target,
                &paints[&node.id],
                node,
                &mut images,
                &mut cursor,
                spec.cursors[node.element_index],
                &mut inverse_cells,
            )?;
            if let Some(grid) = &spec.cells[node.element_index] {
                paint_cells(
                    target,
                    &paints[&node.id],
                    node,
                    grid,
                    &mut images,
                    &mut cursor,
                )?;
            }
            if let Some(image) = &spec.images[node.element_index] {
                if selected.contains(&image.id) {
                    if let Some(plane) = images::Plane::new(
                        image.clone(),
                        &paints[&node.id],
                        node,
                        target,
                        image_options
                            .protocol(image.mode)
                            .expect("selected image protocol"),
                    )? {
                        images.push(plane)?;
                    }
                }
            }
        }
        mark_hits(hits, size.0, node, &mut inverse_cells);
        layouts.push(presented_layout(tree, node)?);
        geometry.push(painted_node(node));
    }
    Ok(crate::backend::PresentedGeometry {
        nodes: geometry,
        layouts,
        images: images.into_planes(),
        cursor: cursor.state,
        hits: Vec::new(),
        layout_reused: reused,
        layout_runs: cache.runs,
        inverse_cells,
    })
}

struct ClipMask {
    parent: Option<Arc<ClipMask>>,
    transform: Affine,
    local: Rect,
    x: bool,
    y: bool,
}

fn inside_masks(mask: &Option<Arc<ClipMask>>, x: i32, y: i32) -> bool {
    let mut current = mask.as_deref();
    while let Some(mask) = current {
        let Some((local_x, local_y)) = mask.transform.inverse(x as f32, y as f32) else {
            return false;
        };
        if (mask.x && (local_x < -0.5 || local_x >= mask.local.right as f32 - 0.5))
            || (mask.y && (local_y < -0.5 || local_y >= mask.local.bottom as f32 - 0.5))
        {
            return false;
        }
        current = mask.parent.as_deref();
    }
    true
}

#[derive(Clone)]
struct Placement {
    mask: Option<Arc<ClipMask>>,
    transform: Affine,
    clip: Rect,
    layer: i32,
    opacity: f32,
}

fn collect(
    tree: &TaffyTree<()>,
    paints: &HashMap<NodeId, NodePaint>,
    id: NodeId,
    parent: Placement,
    nodes: &mut Vec<PaintNode>,
) -> Result<()> {
    let layout = tree
        .layout(id)
        .map_err(|error| ReactiveError::layout(error.to_string()))?;
    let paint = &paints[&id];
    let local = Rect {
        left: 0,
        top: 0,
        right: layout.size.width as i32,
        bottom: layout.size.height as i32,
    };
    let transform = parent.transform.placed(
        (layout.location.x, layout.location.y),
        (layout.size.width, layout.size.height),
        paint.transform,
    );
    let (left, top, right, bottom) = transform.bounds(local.right, local.bottom);
    let bounds = Rect {
        left,
        top,
        right,
        bottom,
    };
    let layer = parent.layer.max(paint.z_index);
    nodes.push(PaintNode {
        mask: parent.mask.clone(),
        transform,
        local,
        parent_opacity: parent.opacity,
        element_index: nodes.len(),
        id,
        bounds,
        clip: parent.clip,
        z: layer,
    });
    let clip = parent.clip;
    let child_clip = Rect {
        left: if paint.overflow_x == Overflow::Hidden && paint.overflow_y == Overflow::Hidden {
            clip.left.max(bounds.left)
        } else {
            clip.left
        },
        right: if paint.overflow_x == Overflow::Hidden && paint.overflow_y == Overflow::Hidden {
            clip.right.min(bounds.right)
        } else {
            clip.right
        },
        top: if paint.overflow_x == Overflow::Hidden && paint.overflow_y == Overflow::Hidden {
            clip.top.max(bounds.top)
        } else {
            clip.top
        },
        bottom: if paint.overflow_x == Overflow::Hidden && paint.overflow_y == Overflow::Hidden {
            clip.bottom.min(bounds.bottom)
        } else {
            clip.bottom
        },
    };
    let mask = if paint.overflow_x == Overflow::Hidden || paint.overflow_y == Overflow::Hidden {
        Some(Arc::new(ClipMask {
            parent: parent.mask,
            transform,
            local,
            x: paint.overflow_x == Overflow::Hidden,
            y: paint.overflow_y == Overflow::Hidden,
        }))
    } else {
        parent.mask
    };
    for child in tree
        .children(id)
        .map_err(|error| ReactiveError::layout(error.to_string()))?
    {
        collect(
            tree,
            paints,
            child,
            Placement {
                mask: mask.clone(),
                transform,
                clip: child_clip,
                layer,
                opacity: parent.opacity * paint.opacity,
            },
            nodes,
        )?;
    }
    Ok(())
}

fn color(value: Rgba) -> ansi::Rgba {
    ansi::rgba_from_floats(value.r, value.g, value.b, value.a)
}

fn with_opacity(value: ansi::Rgba, opacity: f32) -> ansi::Rgba {
    ansi::rgb_color(
        ansi::red(value),
        ansi::green(value),
        ansi::blue(value),
        (f32::from(ansi::alpha(value)) * opacity)
            .round()
            .clamp(0.0, 255.0) as u8,
    )
}

fn attributes(paint: &NodePaint) -> u32 {
    let mut flags = 0;
    for (source, dest) in [
        (Attr::BOLD, TextAttributes::BOLD),
        (Attr::ITALIC, TextAttributes::ITALIC),
        (Attr::UNDERLINE, TextAttributes::UNDERLINE),
        (Attr::REVERSE, TextAttributes::INVERSE),
    ] {
        if paint.style.attr.contains(source) {
            flags |= dest;
        }
    }
    if paint.style.strike {
        flags |= TextAttributes::STRIKETHROUGH;
    }
    u32::from(flags)
}

fn gradient_color(
    gradient: &crate::layout::css::gradients::Gradient,
    bounds: Rect,
    x: i32,
    y: i32,
) -> Option<ansi::Rgba> {
    use crate::layout::css::gradients::GradientDirection::*;
    let coordinate = |position: i32, start: i32, end: i32| {
        if end - start <= 1 {
            0.5
        } else {
            (position - start) as f32 / (end - start - 1) as f32
        }
    };
    let horizontal = coordinate(x, bounds.left, bounds.right);
    let vertical = coordinate(y, bounds.top, bounds.bottom);
    let position = match gradient.direction {
        ToRight => horizontal,
        ToLeft => 1.0 - horizontal,
        ToBottom => vertical,
        ToTop => 1.0 - vertical,
        ToTopRight => (horizontal + 1.0 - vertical) / 2.0,
        ToTopLeft => (2.0 - horizontal - vertical) / 2.0,
        ToBottomRight => (horizontal + vertical) / 2.0,
        ToBottomLeft => (1.0 - horizontal + vertical) / 2.0,
    };
    gradient.color_at(position).map(|(r, g, b, a)| {
        ansi::rgba_from_floats(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
    })
}

fn background(paint: &NodePaint, bounds: Rect, x: i32, y: i32) -> Option<ansi::Rgba> {
    if let Some(border) = &paint.gradient_border {
        let width = (bounds.right - bounds.left).max(0) as usize;
        let height = (bounds.bottom - bounds.top).max(0) as usize;
        if let Some((r, g, b, a)) = border.color_at_cell(
            (x - bounds.left).max(0) as usize,
            (y - bounds.top).max(0) as usize,
            width,
            height,
        ) {
            return Some(ansi::rgba_from_floats(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a * paint.opacity,
            ));
        }
    }
    paint
        .gradient
        .as_ref()
        .and_then(|gradient| gradient_color(gradient, bounds, x, y))
        .map(|color| with_opacity(color, paint.opacity))
        .or_else(|| paint.background_specified.then(|| color(paint.style.bg)))
}

/// Paint one background cell at screen `(x, y)`, which maps to `(local_x,
/// local_y)` in the node's box: the node's background color there over the
/// cell below, covering images and the cursor beneath it.
#[allow(clippy::too_many_arguments)]
fn paint_background_cell(
    target: &mut OptimizedBuffer<'_>,
    paint: &NodePaint,
    node: &PaintNode,
    (x, y): (i32, i32),
    (local_x, local_y): (i32, i32),
    fg: ansi::Rgba,
    attr: u32,
    images: &mut images::Layers,
    cursor: &mut cursor::Layer,
) -> Result<()> {
    use ::suprtui::buffer::draw::blend_colors;
    let Some(source) = background(paint, node.local, local_x, local_y) else {
        return Ok(());
    };
    let source = with_opacity(source, node.parent_opacity);
    if ansi::alpha(source) == 0 {
        return Ok(());
    }
    images.cover(x, y, source);
    cursor.cover(x, y, source);
    let below = target
        .get(x as u32, y as u32)
        .expect("visible cell is in the target");
    let background = blend_colors(source, below.bg, None);
    if ansi::alpha(source) < 255 {
        // Preserve the underlying grapheme span while tinting its colors.
        // Character and link ownership do not change on this path.
        let mut cell = below;
        cell.bg = background;
        cell.fg = blend_colors(source, below.fg, None);
        target.set_raw(x as u32, y as u32, cell);
        return Ok(());
    }
    target
        .draw_grapheme(
            b" ",
            1,
            x as u32,
            y as u32,
            blend_colors(fg, background, None),
            background,
            attr,
        )
        .map_err(|error| ReactiveError::resource(format!("SuprTUI paint: {error:?}")))
}

/// Fill the background of a node that is only translated and not masked, by
/// row (PNT-001). Each row's cells inside the node's box form one span, found
/// by subtracting the offset. A solid opaque color, the common case, is
/// written across the span with one buffer call; a gradient or a translucent
/// color needs each cell's color or the cell below, so it is painted per
/// cell, still without an inverse transform.
#[allow(clippy::too_many_arguments)]
fn fill_plain_background(
    target: &mut OptimizedBuffer<'_>,
    paint: &NodePaint,
    node: &PaintNode,
    visible: Rect,
    fg: ansi::Rgba,
    attr: u32,
    images: &mut images::Layers,
    cursor: &mut cursor::Layer,
) -> Result<()> {
    use ::suprtui::buffer::draw::blend_colors;
    let (offset_x, offset_y) = node.transform.offset();
    let local = |screen: i32, offset: f32| (screen as f32 - offset).round() as i32;
    let solid = (paint.gradient.is_none() && paint.gradient_border.is_none())
        .then(|| with_opacity(color(paint.style.bg), node.parent_opacity));
    for y in visible.top..visible.bottom {
        let local_y = local(y, offset_y);
        if local_y < 0 || local_y >= node.local.bottom {
            continue;
        }
        // Rounding is monotonic, so the row's cells inside the box are one
        // span; these loops only step over cells outside it.
        let mut left = visible.left;
        while left < visible.right && local(left, offset_x) < 0 {
            left += 1;
        }
        let mut right = visible.right;
        while right > left && local(right - 1, offset_x) >= node.local.right {
            right -= 1;
        }
        if left >= right {
            continue;
        }
        match solid {
            Some(source) if ansi::alpha(source) == 0 => {}
            Some(source) if ansi::alpha(source) == 255 => {
                images.cover_span(y, left, right, source);
                cursor.cover_span(y, left, right, source);
                target.fill_span(
                    left as u32,
                    y as u32,
                    (right - left) as u32,
                    blend_colors(fg, source, None),
                    source,
                    attr,
                );
            }
            _ => {
                for x in left..right {
                    paint_background_cell(
                        target,
                        paint,
                        node,
                        (x, y),
                        (local(x, offset_x), local_y),
                        fg,
                        attr,
                        images,
                        cursor,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn paint_node(
    target: &mut OptimizedBuffer<'_>,
    paint: &NodePaint,
    node: &PaintNode,
    images: &mut images::Layers,
    cursor: &mut cursor::Layer,
    request: Option<crate::component::element::TextCursor>,
    inverse_cells: &mut u64,
) -> Result<()> {
    use ::suprtui::buffer::draw::blend_colors;
    if paint.opacity * node.parent_opacity <= 0.0 {
        return Ok(());
    }
    if node.transform.inverse(0.0, 0.0).is_none() {
        return Ok(());
    }
    let fg = with_opacity(color(paint.style.fg), node.parent_opacity);
    let bg = color(paint.style.bg);
    let attr = attributes(paint);
    let visible = node.bounds.intersect(node.clip);
    // A node that is only translated and not masked maps cells by plain
    // subtraction and addition, the same values the general path computes,
    // without the per-cell inverse and mask walk (PNT-001).
    let plain = node.transform.is_translation() && node.mask.is_none();
    let (offset_x, offset_y) = node.transform.offset();
    // Fill the complete box, including padding, when a background is supplied.
    if paint.gradient.is_some() || paint.gradient_border.is_some() || paint.background_specified {
        if plain {
            fill_plain_background(target, paint, node, visible, fg, attr, images, cursor)?;
        } else {
            for y in visible.top..visible.bottom {
                for x in visible.left..visible.right {
                    if !inside_masks(&node.mask, x, y) {
                        continue;
                    }
                    *inverse_cells += 1;
                    let Some((local_x, local_y)) = node.transform.inverse(x as f32, y as f32)
                    else {
                        continue;
                    };
                    let (local_x, local_y) = (local_x.round() as i32, local_y.round() as i32);
                    if local_x < 0
                        || local_x >= node.local.right
                        || local_y < 0
                        || local_y >= node.local.bottom
                    {
                        continue;
                    }
                    paint_background_cell(
                        target,
                        paint,
                        node,
                        (x, y),
                        (local_x, local_y),
                        fg,
                        attr,
                        images,
                        cursor,
                    )?;
                }
            }
        }
    }
    if let Some(text) = &paint.text {
        let content = Rect {
            left: node.local.left.saturating_add(paint.pad.left as i32),
            top: node.local.top.saturating_add(paint.pad.top as i32),
            right: node.local.right.saturating_sub(paint.pad.right as i32),
            bottom: node.local.bottom.saturating_sub(paint.pad._bottom as i32),
        };
        let clip = content;
        let width = (content.right - content.left).max(0) as usize;
        for (row, line) in paint.typography.lines(text, width).iter().enumerate() {
            let y = content.top.saturating_add(
                row.saturating_mul(paint.typography.line_height.unwrap_or(1))
                    .min(i32::MAX as usize) as i32,
            );
            if y >= clip.bottom {
                break;
            }
            if y < clip.top {
                continue;
            }
            let mut x = content.left;
            for grapheme in line.graphemes(true) {
                // Never send application text as terminal control sequences.
                if grapheme.chars().any(char::is_control) {
                    continue;
                }
                let width = UnicodeWidthStr::width(grapheme);
                if width == 0 {
                    continue;
                }
                let right = x.saturating_add(width.min(i32::MAX as usize) as i32);
                if right > clip.right {
                    break;
                }
                if x >= clip.left {
                    let half = (width as f32 - 1.0) / 2.0;
                    let (paint_x, paint_y) = if plain {
                        (x as f32 + half + offset_x, y as f32 + offset_y)
                    } else {
                        node.transform.point(x as f32 + half, y as f32)
                    };
                    let paint_x = (paint_x - half).round() as i32;
                    let paint_y = paint_y.round() as i32;
                    if paint_x < node.clip.left
                        || paint_y < node.clip.top
                        || paint_x.saturating_add(width as i32) > node.clip.right
                        || paint_y >= node.clip.bottom
                    {
                        x = right;
                        continue;
                    }
                    if !plain
                        && !(0..width).all(|offset| {
                            inside_masks(&node.mask, paint_x.saturating_add(offset as i32), paint_y)
                        })
                    {
                        x = right;
                        continue;
                    }
                    for offset in 0..width {
                        cursor.cover(
                            paint_x + offset as i32,
                            paint_y,
                            ansi::rgb_color(0, 0, 0, 255),
                        );
                        images.cover(
                            paint_x + offset as i32,
                            paint_y,
                            ansi::rgb_color(0, 0, 0, 255),
                        );
                    }
                    let bg = target
                        .get(paint_x as u32, paint_y as u32)
                        .map_or(bg, |cell| cell.bg);
                    let width = u8::try_from(width)
                        .map_err(|_| ReactiveError::layout("grapheme exceeds 255 cells"))?;
                    target
                        .draw_grapheme(
                            grapheme.as_bytes(),
                            width,
                            paint_x as u32,
                            paint_y as u32,
                            blend_colors(fg, bg, None),
                            bg,
                            attr,
                        )
                        .map_err(|error| {
                            ReactiveError::resource(format!("SuprTUI paint: {error:?}"))
                        })?;
                    cursor.paint(
                        request,
                        (x, y),
                        width,
                        (paint_x, paint_y),
                        blend_colors(fg, bg, None),
                    );
                }
                x = right;
            }
        }
    }
    Ok(())
}

/// Blit a prepared cell grid at the node's content box: each painted cell
/// is placed through the node's transform, clipped to the box and the
/// node's clip, masked like text, and drawn with the cell's own color or
/// the node's foreground, over the cell's own background where it has one.
/// A color packed as `0xRRGGBBAA` in a [`super::cells::CellGrid`].
fn packed(value: u32) -> ansi::Rgba {
    ansi::rgb_color(
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    )
}

fn paint_cells(
    target: &mut OptimizedBuffer<'_>,
    paint: &NodePaint,
    node: &PaintNode,
    grid: &super::cells::CellGrid,
    images: &mut images::Layers,
    cursor: &mut cursor::Layer,
) -> Result<()> {
    use ::suprtui::buffer::draw::blend_colors;
    if paint.opacity * node.parent_opacity <= 0.0 || node.transform.inverse(0.0, 0.0).is_none() {
        return Ok(());
    }
    let node_fg = with_opacity(color(paint.style.fg), node.parent_opacity);
    let bg_default = color(paint.style.bg);
    let attr = attributes(paint);
    let content = Rect {
        left: node.local.left.saturating_add(paint.pad.left as i32),
        top: node.local.top.saturating_add(paint.pad.top as i32),
        right: node.local.right.saturating_sub(paint.pad.right as i32),
        bottom: node.local.bottom.saturating_sub(paint.pad._bottom as i32),
    };
    // A node that is only translated and not masked maps a cell by plain
    // addition, the value the general path computes, without the transform
    // and the mask walk (PNT-001). At full opacity an opaque color is
    // written as it is, with no blend and no read of the cell below.
    let plain = node.transform.is_translation() && node.mask.is_none();
    let (offset_x, offset_y) = node.transform.offset();
    let full = node.parent_opacity >= 1.0;
    // Nothing in this node adds a cursor or an image plane, and what paints
    // later lies above it, so with neither present a cover changes nothing.
    let covers = cursor.state.is_some() || images.has_planes();
    let black = ansi::rgb_color(0, 0, 0, 255);
    let opacity = |value: u32| {
        if full {
            packed(value)
        } else {
            with_opacity(packed(value), node.parent_opacity)
        }
    };
    for (gx, gy, glyph, width, fg, cell_bg) in grid.painted() {
        let x = content.left.saturating_add(i32::from(gx));
        let y = content.top.saturating_add(i32::from(gy));
        let right = x.saturating_add(width.min(i32::MAX as usize) as i32);
        if y < content.top || y >= content.bottom || x < content.left || right > content.right {
            continue;
        }
        let center = (width as f32 - 1.0) / 2.0;
        let (paint_x, paint_y) = if plain {
            (x as f32 + center + offset_x, y as f32 + offset_y)
        } else {
            node.transform.point(x as f32 + center, y as f32)
        };
        let paint_x = (paint_x - center).round() as i32;
        let paint_y = paint_y.round() as i32;
        if paint_x < node.clip.left
            || paint_y < node.clip.top
            || paint_x.saturating_add(width as i32) > node.clip.right
            || paint_y >= node.clip.bottom
        {
            continue;
        }
        if !plain
            && !(0..width).all(|offset| {
                inside_masks(&node.mask, paint_x.saturating_add(offset as i32), paint_y)
            })
        {
            continue;
        }
        if covers {
            for offset in 0..width {
                cursor.cover(paint_x + offset as i32, paint_y, black);
                images.cover(paint_x + offset as i32, paint_y, black);
            }
        }
        let below = || {
            target
                .get(paint_x as u32, paint_y as u32)
                .map_or(bg_default, |cell| cell.bg)
        };
        let bg = match cell_bg {
            Some(value) if full && value & 0xFF == 0xFF => packed(value),
            Some(value) => blend_colors(opacity(value), below(), None),
            None => below(),
        };
        let glyph = if glyph.is_empty() { " " } else { glyph };
        let fg = fg.map_or(node_fg, opacity);
        let fg = if ansi::alpha(fg) == 255 {
            fg
        } else {
            blend_colors(fg, bg, None)
        };
        let width =
            u8::try_from(width).map_err(|_| ReactiveError::layout("grapheme exceeds 255 cells"))?;
        target
            .draw_grapheme(
                glyph.as_bytes(),
                width,
                paint_x as u32,
                paint_y as u32,
                fg,
                bg,
                attr,
            )
            .map_err(|error| ReactiveError::resource(format!("SuprTUI paint: {error:?}")))?;
    }
    Ok(())
}

/// RAS-007: painting a frame clears the next buffer exactly once, whether
/// the layout is computed or reused.
#[cfg(test)]
mod ras_007_tests {
    use super::*;
    use ::suprtui::{buffer::InitOptions, uni::pool::GraphemePool};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn ras_007_painting_a_frame_clears_the_next_buffer_once() {
        let pool = Rc::new(RefCell::new(GraphemePool::new()));
        let mut buffer = OptimizedBuffer::new(20, 4, InitOptions::new(pool)).unwrap();
        let mut hits = vec![0; 20 * 4];
        let mut cache = LayoutCache::default();
        let element = crate::component::Element::text("painted").with_class("w-full h-1");
        for frame in 1..=3 {
            paint_frame(
                crate::component::bridge::element_to_paintspec(&element).unwrap(),
                &mut buffer,
                &mut hits,
                &mut cache,
                crate::backend::ImageOutputOptions::default(),
            )
            .unwrap();
            assert_eq!(buffer.clear_count(), frame, "frame {frame}");
        }
    }
}
