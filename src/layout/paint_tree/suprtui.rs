//! Grapheme painting over the existing CSS/Taffy layout metadata.

mod cursor;
pub(crate) mod images;

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

pub(crate) fn paint_frame(
    root: &crate::component::bridge::PaintSpec,
    target: &mut OptimizedBuffer<'_>,
    image_options: crate::backend::ImageOutputOptions,
) -> Result<crate::backend::PresentedGeometry> {
    target.clear(ansi::rgb_color(0, 0, 0, 255), None);
    let mut tree = TaffyTree::new();
    let mut paints = HashMap::new();
    let spec = root;
    let root = build_nodes_inherited(
        &mut tree,
        &root.root,
        &mut paints,
        &crate::layout::text::TextStyle::default(),
        None,
        false,
        &mut root.styles.iter().cloned(),
    )?;
    let available = Size {
        width: AvailableSpace::Definite(target.width() as f32),
        height: AvailableSpace::Definite(target.height() as f32),
    };
    tree.compute_layout_with_measure(root, available, |known, available, id, _, _| {
        super::measure_text(&paints[&id], known, available)
    })
    .map_err(|error| ReactiveError::layout(format!("SuprTUI layout: {error}")))?;
    let screen = Rect {
        left: 0,
        top: 0,
        right: target.width() as i32,
        bottom: target.height() as i32,
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
    for node in nodes {
        let fallback =
            spec.image_fallbacks[node.element_index].is_some_and(|id| selected.contains(&id));
        if !fallback {
            paint_node(
                target,
                &paints[&node.id],
                &node,
                &mut images,
                &mut cursor,
                spec.cursors[node.element_index],
            )?;
            if let Some(grid) = &spec.cells[node.element_index] {
                paint_cells(target, &paints[&node.id], &node, grid, &mut images, &mut cursor)?;
            }
            if let Some(image) = &spec.images[node.element_index] {
                if selected.contains(&image.id) {
                    if let Some(plane) = images::Plane::new(
                        image.clone(),
                        &paints[&node.id],
                        &node,
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
        let visible = hit_bounds(&node);
        let layout = tree
            .layout(node.id)
            .map_err(|error| ReactiveError::layout(error.to_string()))?;
        layouts.push(crate::backend::PresentedLayout {
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
        });
        geometry.push(crate::backend::PaintedNode {
            element_index: node.element_index,
            bounds: crate::event::hit::Bounds::new(
                visible.left as f32,
                visible.top as f32,
                (visible.right - visible.left).max(0) as f32,
                (visible.bottom - visible.top).max(0) as f32,
            ),
        });
    }
    Ok(crate::backend::PresentedGeometry {
        nodes: geometry,
        layouts,
        images: images.into_planes(),
        cursor: cursor.state,
    })
}

fn hit_bounds(node: &PaintNode) -> Rect {
    let visible = node.bounds.intersect(node.clip);
    if node.mask.is_none() {
        return visible;
    }
    // The public event API uses rectangles. Bound the cells that survive local
    // ancestor clipping, rather than registering an entirely clipped child.
    let mut bounds = Rect {
        left: visible.right,
        right: visible.left,
        top: visible.bottom,
        bottom: visible.top,
    };
    for y in visible.top..visible.bottom {
        for x in visible.left..visible.right {
            if inside_masks(&node.mask, x, y) {
                bounds.left = bounds.left.min(x);
                bounds.right = bounds.right.max(x + 1);
                bounds.top = bounds.top.min(y);
                bounds.bottom = bounds.bottom.max(y + 1);
            }
        }
    }
    bounds
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

fn paint_node(
    target: &mut OptimizedBuffer<'_>,
    paint: &NodePaint,
    node: &PaintNode,
    images: &mut images::Layers,
    cursor: &mut cursor::Layer,
    request: Option<crate::component::element::TextCursor>,
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
    // Fill the complete box, including padding, when a background is supplied.
    if paint.gradient.is_some() || paint.gradient_border.is_some() || paint.background_specified {
        for y in visible.top..visible.bottom {
            for x in visible.left..visible.right {
                if !inside_masks(&node.mask, x, y) {
                    continue;
                }
                let Some((local_x, local_y)) = node.transform.inverse(x as f32, y as f32) else {
                    continue;
                };
                let local_x = local_x.round() as i32;
                let local_y = local_y.round() as i32;
                if local_x < 0
                    || local_x >= node.local.right
                    || local_y < 0
                    || local_y >= node.local.bottom
                {
                    continue;
                }
                let Some(source) = background(paint, node.local, local_x, local_y) else {
                    continue;
                };
                let source = with_opacity(source, node.parent_opacity);
                if ansi::alpha(source) == 0 {
                    continue;
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
                    continue;
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
                    .map_err(|error| {
                        ReactiveError::resource(format!("SuprTUI paint: {error:?}"))
                    })?;
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
                    let (paint_x, paint_y) = node
                        .transform
                        .point(x as f32 + (width as f32 - 1.0) / 2.0, y as f32);
                    let paint_x = (paint_x - (width as f32 - 1.0) / 2.0).round() as i32;
                    let paint_y = paint_y.round() as i32;
                    if paint_x < node.clip.left
                        || paint_y < node.clip.top
                        || paint_x.saturating_add(width as i32) > node.clip.right
                        || paint_y >= node.clip.bottom
                    {
                        x = right;
                        continue;
                    }
                    if !(0..width).all(|offset| {
                        inside_masks(&node.mask, paint_x.saturating_add(offset as i32), paint_y)
                    }) {
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

/// Blit a prepared cell grid at the node's content box: each set cell is
/// placed through the node's transform, clipped to the box and the node's
/// clip, masked like text, and drawn with the cell's own color or the
/// node's foreground.
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
    for (gx, gy, glyph, width, fg) in grid.iter() {
        let x = content.left.saturating_add(i32::from(gx));
        let y = content.top.saturating_add(i32::from(gy));
        let right = x.saturating_add(width.min(i32::MAX as usize) as i32);
        if y < content.top || y >= content.bottom || x < content.left || right > content.right {
            continue;
        }
        let (paint_x, paint_y) = node
            .transform
            .point(x as f32 + (width as f32 - 1.0) / 2.0, y as f32);
        let paint_x = (paint_x - (width as f32 - 1.0) / 2.0).round() as i32;
        let paint_y = paint_y.round() as i32;
        if paint_x < node.clip.left
            || paint_y < node.clip.top
            || paint_x.saturating_add(width as i32) > node.clip.right
            || paint_y >= node.clip.bottom
        {
            continue;
        }
        if !(0..width).all(|offset| {
            inside_masks(&node.mask, paint_x.saturating_add(offset as i32), paint_y)
        }) {
            continue;
        }
        for offset in 0..width {
            cursor.cover(paint_x + offset as i32, paint_y, ansi::rgb_color(0, 0, 0, 255));
            images.cover(paint_x + offset as i32, paint_y, ansi::rgb_color(0, 0, 0, 255));
        }
        let bg = target
            .get(paint_x as u32, paint_y as u32)
            .map_or(bg_default, |cell| cell.bg);
        let fg = match fg {
            Some((r, g, b, a)) => {
                with_opacity(ansi::rgba_from_floats(r, g, b, a), node.parent_opacity)
            }
            None => node_fg,
        };
        let width = u8::try_from(width)
            .map_err(|_| ReactiveError::layout("grapheme exceeds 255 cells"))?;
        target
            .draw_grapheme(
                glyph.as_bytes(),
                width,
                paint_x as u32,
                paint_y as u32,
                blend_colors(fg, bg, None),
                bg,
                attr,
            )
            .map_err(|error| ReactiveError::resource(format!("SuprTUI paint: {error:?}")))?;
    }
    Ok(())
}
