use crate::core::surface::Surface;
use crate::error::{RTuiError, Result};
#[derive(Default)]
pub struct PaintOptions {
    pub debug_overlay: bool,
}

use crate::layout::style::StyleBuilder;
use crate::layout::utility_css::apply_utility_classes;
use crate::ui::paint::{PaintStyle, extract_paint_style};
use std::collections::HashMap;
use taffy::{AvailableSpace, TaffyTree, geometry::Size, prelude::NodeId, style::Style};

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
        .map_err(|e| RTuiError::layout(format!("Failed to compute layout: {}", e)))?;
    paint_recursive_dbg(&taffy, root_id, surface, &map, opts.debug_overlay);
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
            .map_err(|e| RTuiError::layout(format!("Failed to create leaf node: {}", e)))?
    } else {
        let id = taffy
            .new_with_children(style, &[])
            .map_err(|e| RTuiError::layout(format!("Failed to create parent node: {}", e)))?;
        let mut child_ids: Vec<NodeId> = Vec::with_capacity(spec.children.len());
        for child in &spec.children {
            let cid = build_nodes(taffy, child, map)?;
            child_ids.push(cid);
        }
        taffy
            .set_children(id, &child_ids)
            .map_err(|e| RTuiError::layout(format!("Failed to set children: {}", e)))?;
        id
    };
    // Extract visuals + padding/margin cache (px only)
    let paint_style = extract_paint_style(&mut sb).unwrap_or_default();
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
        },
    );
    Ok(id)
}

fn paint_recursive_dbg(
    taffy: &TaffyTree<()>,
    node: NodeId,
    surface: &mut Surface,
    map: &HashMap<NodeId, NodePaint>,
    debug_overlay: bool,
) {
    if let Ok(layout) = taffy.layout(node) {
        let x = layout.location.x.max(0.0) as usize;
        let y = layout.location.y.max(0.0) as usize;
        let w = layout.size.width.max(0.0) as usize;
        let h = layout.size.height.max(0.0) as usize;
        if debug_overlay {
            surface.draw_box(x, y, w, h, None);
        }
        if let Some(np) = map.get(&node) {
            let content_x = x + np.pad.left;
            let content_y = y + np.pad.top;
            let content_w = w.saturating_sub(np.pad.left + np.pad.right);
            if let Some(text) = &np.text {
                surface.write_text_styled_clipped(content_x, content_y, text, content_w, &np.style);
            }
        }
    }
    if let Ok(children) = taffy.children(node) {
        for c in children {
            paint_recursive_dbg(taffy, c, surface, map, debug_overlay);
        }
    }
}
