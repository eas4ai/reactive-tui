use crate::core::surface::Surface;
use crate::error::{ReactiveError, Result};
use taffy::style::Overflow;
#[derive(Default)]
pub struct PaintOptions {
    pub debug_overlay: bool,
}

use crate::layout::css::apply_utility_classes;
use crate::layout::style::StyleBuilder;
use crate::ui::paint::{extract_paint_style, PaintStyle};
use std::collections::{BTreeMap, HashMap};
use taffy::{geometry::Size, prelude::NodeId, style::Style, AvailableSpace, TaffyTree};

use std::borrow::Cow;

/// Padding values for a box (left, right, top, bottom)
#[derive(Clone, Copy, Debug, Default)]
struct Padding {
    left: usize,
    right: usize,
    top: usize,
    _bottom: usize,
}

impl Padding {
    fn new(left: usize, right: usize, top: usize, bottom: usize) -> Self {
        Self {
            left,
            right,
            top,
            _bottom: bottom,
        }
    }
}

/// Minimal example: build a grid or flex tree from utility classes and paint text children.
pub struct NodeSpec<'a> {
    pub class: Cow<'a, str>,
    pub text: Option<Cow<'a, str>>,
    pub children: Vec<NodeSpec<'a>>,
}

struct NodePaint {
    style: PaintStyle,
    text: Option<String>,
    pad: Padding,
    z_index: i32,
    overflow_x: Overflow,
    overflow_y: Overflow,
}

pub fn layout_and_paint_with<'a>(
    root: &NodeSpec<'a>,
    surface: &mut Surface,
    width: usize,
    opts: &PaintOptions,
) -> Result<()> {
    let mut taffy = TaffyTree::new();
    let mut map: HashMap<NodeId, NodePaint> = HashMap::new();
    let root_id = build_nodes(&mut taffy, root, &mut map)?;
    let available = Size {
        width: AvailableSpace::Definite(width as f32),
        height: AvailableSpace::MaxContent,
    };
    taffy
        .compute_layout(root_id, available)
        .map_err(|e| ReactiveError::layout(format!("Failed to compute layout: {}", e)))?;
    paint_with_z_index(&taffy, root_id, surface, &map, opts.debug_overlay);
    Ok(())
}

pub fn layout_and_paint<'a>(root: &NodeSpec<'a>, surface: &mut Surface, width: usize) {
    let opts = PaintOptions::default();
    let _ = layout_and_paint_with(root, surface, width, &opts);
}

fn build_nodes<'a>(
    taffy: &mut TaffyTree<()>,
    spec: &NodeSpec<'a>,
    map: &mut HashMap<NodeId, NodePaint>,
) -> Result<NodeId> {
    let mut sb = apply_utility_classes(spec.class.as_ref(), StyleBuilder::new());
    let style: Style = sb.clone().build();
    let id = if spec.children.is_empty() {
        taffy
            .new_leaf(style)
            .map_err(|e| ReactiveError::layout(format!("Failed to create leaf node: {}", e)))?
    } else {
        let id = taffy
            .new_with_children(style, &[])
            .map_err(|e| ReactiveError::layout(format!("Failed to create parent node: {}", e)))?;
        let mut child_ids: Vec<NodeId> = Vec::with_capacity(spec.children.len());
        for child in &spec.children {
            let cid = build_nodes(taffy, child, map)?;
            child_ids.push(cid);
        }
        taffy
            .set_children(id, &child_ids)
            .map_err(|e| ReactiveError::layout(format!("Failed to set children: {}", e)))?;
        id
    };
    // Extract visuals + padding/margin cache (px only)
    let paint_style = extract_paint_style(&mut sb).unwrap_or_default();
    let z_index = sb.get_z_index().unwrap_or(0);
    let overflow_x = sb.get_overflow_x();
    let overflow_y = sb.get_overflow_y();
    let padding = sb.pad_cache();
    let pad = Padding::new(
        padding.left as usize,
        padding.right as usize,
        padding.top as usize,
        padding.bottom as usize,
    );
    let text = spec.text.as_ref().map(|s| s.to_string());
    map.insert(
        id,
        NodePaint {
            style: paint_style,
            text,
            pad,
            z_index,
            overflow_x,
            overflow_y,
        },
    );
    Ok(id)
}

/// Check if a node needs overflow clipping
fn needs_clipping(node_paint: &NodePaint) -> bool {
    matches!(node_paint.overflow_x, Overflow::Hidden)
        || matches!(node_paint.overflow_y, Overflow::Hidden)
}

/// Paint a single node with optional overflow clipping
fn paint_node_with_overflow(
    surface: &mut Surface,
    node_paint: &NodePaint,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    debug_overlay: bool,
) {
    if debug_overlay {
        surface.draw_box(x, y, w, h, None);
    }

    let content_x = x + node_paint.pad.left;
    let content_y = y + node_paint.pad.top;
    let content_w = w.saturating_sub(node_paint.pad.left + node_paint.pad.right);
    let content_h = h.saturating_sub(node_paint.pad.top + node_paint.pad._bottom);

    if let Some(text) = &node_paint.text {
        if needs_clipping(node_paint) {
            // Use clipped subview for overflow:hidden
            let (surface_w, surface_h) = surface.dims();
            let clip_w = match node_paint.overflow_x {
                Overflow::Hidden => content_w,
                _ => surface_w.saturating_sub(content_x),
            };
            let clip_h = match node_paint.overflow_y {
                Overflow::Hidden => content_h,
                _ => surface_h.saturating_sub(content_y),
            };

            // Create a clipped subview and paint within it
            let mut subview = surface.subview_mut(
                content_x as isize,
                content_y as isize,
                content_x as isize,
                content_y as isize,
                clip_w,
                clip_h,
            );
            subview.write_text(
                0,
                0,
                text,
                crate::core::surface::TextStyle {
                    fg: node_paint.style.fg,
                    bg: node_paint.style.bg,
                    attr: node_paint.style.attr,
                    emoji_aware: true,
                },
            );
        } else {
            // Normal text rendering without clipping
            surface.write_text_styled_clipped(
                content_x,
                content_y,
                text,
                content_w,
                &node_paint.style,
            );
        }
    }
}

/// Paint nodes in z-index order for proper layering (modals, popovers, etc.)
fn paint_with_z_index(
    taffy: &TaffyTree<()>,
    root_id: NodeId,
    surface: &mut Surface,
    map: &HashMap<NodeId, NodePaint>,
    debug_overlay: bool,
) {
    // Collect all nodes with their z-index and layout info
    let mut layers: BTreeMap<i32, Vec<(NodeId, usize, usize, usize, usize)>> = BTreeMap::new();
    collect_nodes_by_z_index(taffy, root_id, map, &mut layers);

    // Paint in z-index order (lowest to highest)
    for (_z_index, nodes) in layers {
        for (node_id, x, y, w, h) in nodes {
            if let Some(node_paint) = map.get(&node_id) {
                paint_node_with_overflow(surface, node_paint, x, y, w, h, debug_overlay);
            } else if debug_overlay {
                surface.draw_box(x, y, w, h, None);
            }
        }
    }
}

/// Recursively collect nodes grouped by z-index
fn collect_nodes_by_z_index(
    taffy: &TaffyTree<()>,
    node: NodeId,
    map: &HashMap<NodeId, NodePaint>,
    layers: &mut BTreeMap<i32, Vec<(NodeId, usize, usize, usize, usize)>>,
) {
    if let Ok(layout) = taffy.layout(node) {
        let x = layout.location.x.max(0.0) as usize;
        let y = layout.location.y.max(0.0) as usize;
        let w = layout.size.width.max(0.0) as usize;
        let h = layout.size.height.max(0.0) as usize;

        let z_index = map.get(&node).map(|np| np.z_index).unwrap_or(0);
        layers.entry(z_index).or_default().push((node, x, y, w, h));
    }

    if let Ok(children) = taffy.children(node) {
        for child in children {
            collect_nodes_by_z_index(taffy, child, map, layers);
        }
    }
}
