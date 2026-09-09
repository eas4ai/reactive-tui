//! Grapheme painting over the existing CSS/Taffy layout metadata.

use super::{build_nodes, NodePaint, NodeSpec};
use crate::core::surface::{Attr, Rgba};
use crate::error::{ReactiveError, Result};
use ::suprtui::ansi::{self, TextAttributes};
use ::suprtui::buffer::OptimizedBuffer;
use std::collections::HashMap;
use taffy::{geometry::Size, prelude::NodeId, style::Overflow, AvailableSpace, TaffyTree};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
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
    element_index: usize,
    id: NodeId,
    bounds: Rect,
    clip: Rect,
    z: i32,
}

pub(crate) fn paint_frame(
    root: &NodeSpec<'_>,
    target: &mut OptimizedBuffer<'_>,
) -> Result<Vec<crate::backend::PaintedNode>> {
    target.clear(ansi::rgb_color(0, 0, 0, 255), None);
    let mut tree = TaffyTree::new();
    let mut paints = HashMap::new();
    let root = build_nodes(&mut tree, root, &mut paints)?;
    let available = Size {
        width: AvailableSpace::Definite(target.width() as f32),
        height: AvailableSpace::Definite(target.height() as f32),
    };
    tree.compute_layout_with_measure(root, available, |known, _, id, _, _| {
        let text = paints
            .get(&id)
            .and_then(|paint| paint.text.as_deref())
            .unwrap_or("");
        Size {
            width: known.width.unwrap_or_else(|| {
                text.lines().map(UnicodeWidthStr::width).max().unwrap_or(0) as f32
            }),
            height: known.height.unwrap_or(text.lines().count() as f32),
        }
    })
    .map_err(|error| ReactiveError::layout(format!("SuprTUI layout: {error}")))?;
    let screen = Rect {
        left: 0,
        top: 0,
        right: target.width() as i32,
        bottom: target.height() as i32,
    };
    let mut nodes = Vec::new();
    collect(&tree, &paints, root, (0, 0), screen, i32::MIN, &mut nodes)?;
    // Stable sorting preserves parent-before-child and sibling paint order.
    nodes.sort_by_key(|node| node.z);
    let mut geometry = Vec::with_capacity(nodes.len());
    for node in nodes {
        paint_node(target, &paints[&node.id], &node)?;
        let visible = node.bounds.intersect(node.clip);
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
    Ok(geometry)
}

fn collect(
    tree: &TaffyTree<()>,
    paints: &HashMap<NodeId, NodePaint>,
    id: NodeId,
    origin: (i32, i32),
    clip: Rect,
    parent_layer: i32,
    nodes: &mut Vec<PaintNode>,
) -> Result<()> {
    let layout = tree
        .layout(id)
        .map_err(|error| ReactiveError::layout(error.to_string()))?;
    let left = origin.0.saturating_add(layout.location.x as i32);
    let top = origin.1.saturating_add(layout.location.y as i32);
    let bounds = Rect {
        left,
        top,
        right: left.saturating_add(layout.size.width as i32),
        bottom: top.saturating_add(layout.size.height as i32),
    };
    let paint = &paints[&id];
    // Descendants paint at least on their parent's layer; otherwise a
    // positioned box's background would hide ordinary text children.
    let layer = parent_layer.max(paint.z_index);
    nodes.push(PaintNode {
        element_index: nodes.len(),
        id,
        bounds,
        clip,
        z: layer,
    });
    let child_clip = Rect {
        left: if paint.overflow_x == Overflow::Hidden {
            clip.left.max(bounds.left)
        } else {
            clip.left
        },
        right: if paint.overflow_x == Overflow::Hidden {
            clip.right.min(bounds.right)
        } else {
            clip.right
        },
        top: if paint.overflow_y == Overflow::Hidden {
            clip.top.max(bounds.top)
        } else {
            clip.top
        },
        bottom: if paint.overflow_y == Overflow::Hidden {
            clip.bottom.min(bounds.bottom)
        } else {
            clip.bottom
        },
    };
    for child in tree
        .children(id)
        .map_err(|error| ReactiveError::layout(error.to_string()))?
    {
        // Taffy locations are relative to the parent, including absolute nodes.
        collect(tree, paints, child, (left, top), child_clip, layer, nodes)?;
    }
    Ok(())
}

fn color(value: Rgba) -> ansi::Rgba {
    ansi::rgba_from_floats(value.r, value.g, value.b, value.a)
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

fn paint_node(target: &mut OptimizedBuffer<'_>, paint: &NodePaint, node: &PaintNode) -> Result<()> {
    let fg = color(paint.style.fg);
    let bg = color(paint.style.bg);
    let attr = attributes(paint);
    let visible = node.bounds.intersect(node.clip);
    // Fill the complete box, including padding, when a background is supplied.
    if paint.background_specified || paint.style.bg != crate::ui::paint::PaintStyle::default().bg {
        for y in visible.top..visible.bottom {
            for x in visible.left..visible.right {
                target
                    .draw_grapheme(b" ", 1, x as u32, y as u32, fg, bg, attr)
                    .map_err(|error| {
                        ReactiveError::resource(format!("SuprTUI paint: {error:?}"))
                    })?;
            }
        }
    }
    if let Some(text) = &paint.text {
        let content = Rect {
            left: node.bounds.left.saturating_add(paint.pad.left as i32),
            top: node.bounds.top.saturating_add(paint.pad.top as i32),
            right: node.bounds.right.saturating_sub(paint.pad.right as i32),
            bottom: node.bounds.bottom.saturating_sub(paint.pad._bottom as i32),
        };
        let clip = content.intersect(node.clip);
        for (row, line) in text.lines().enumerate() {
            let y = content
                .top
                .saturating_add(row.min(i32::MAX as usize) as i32);
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
                    let width = u8::try_from(width)
                        .map_err(|_| ReactiveError::layout("grapheme exceeds 255 cells"))?;
                    target
                        .draw_grapheme(grapheme.as_bytes(), width, x as u32, y as u32, fg, bg, attr)
                        .map_err(|error| {
                            ReactiveError::resource(format!("SuprTUI paint: {error:?}"))
                        })?;
                }
                x = right;
            }
        }
    }
    Ok(())
}
