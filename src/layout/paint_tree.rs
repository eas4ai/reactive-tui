use crate::core::surface::Surface;
#[derive(Default)]
pub struct PaintOptions { pub debug_overlay: bool }

use crate::ui::paint::{PaintStyle, extract_paint_style};
use crate::layout::style::StyleBuilder;
use crate::layout::utility_css::apply_utility_classes;
use std::collections::HashMap;
use taffy::{TaffyTree, style::Style, prelude::NodeId, geometry::Size, AvailableSpace};

use std::borrow::Cow;
/// Minimal example: build a grid or flex tree from utility classes and paint text children.
pub struct NodeSpec<'a> {
    pub class: Cow<'a, str>,
    pub text: Option<Cow<'a, str>>,
    pub children: Vec<NodeSpec<'a>>,
}

struct NodePaint {
    style: PaintStyle,
    text: Option<String>,
    pad: (usize,usize,usize,usize),
}

pub fn layout_and_paint_with<'a>(root: &NodeSpec<'a>, surface: &mut Surface, width: usize, opts: &PaintOptions) {
    let mut taffy = TaffyTree::new();
    let mut map: HashMap<NodeId, NodePaint> = HashMap::new();
    let root_id = build_nodes(&mut taffy, root, &mut map);
    let available = Size { width: AvailableSpace::Definite(width as f32), height: AvailableSpace::MaxContent };
    let _ = taffy.compute_layout(root_id, available);
    paint_recursive_dbg(&taffy, root_id, surface, &map, opts.debug_overlay);
}


pub fn layout_and_paint<'a>(root: &NodeSpec<'a>, surface: &mut Surface, width: usize) {
    let opts = PaintOptions::default();
    layout_and_paint_with(root, surface, width, &opts)
}

fn build_nodes<'a>(taffy: &mut TaffyTree<()>, spec: &NodeSpec<'a>, map: &mut HashMap<NodeId, NodePaint>) -> NodeId {
    let mut sb = apply_utility_classes(spec.class.as_ref(), StyleBuilder::new());
    let style: Style = sb.clone().build();
    let id = if spec.children.is_empty() {
        taffy.new_leaf(style).unwrap()
    } else {
        let id = taffy.new_with_children(style, &[]).unwrap();
        let mut child_ids: Vec<NodeId> = Vec::with_capacity(spec.children.len());
        for child in &spec.children { let cid = build_nodes(taffy, child, map); child_ids.push(cid); }
        taffy.set_children(id, &child_ids).unwrap();
        id
    };
    // Extract visuals + padding/margin cache (px only)
    let paint_style = extract_paint_style(&mut sb).unwrap_or_default();
    let (pl,pr,pt,pb) = sb.pad_cache();
    let pad = (pl as usize, pr as usize, pt as usize, pb as usize);
    let text = spec.text.as_ref().map(|s| s.to_string());
    map.insert(id, NodePaint { style: paint_style, text, pad });
    id
}

fn paint_recursive_dbg(taffy: &TaffyTree<()>, node: NodeId, surface: &mut Surface, map: &HashMap<NodeId, NodePaint>, debug_overlay: bool) {
    if let Ok(layout) = taffy.layout(node) {
        let x = layout.location.x.max(0.0) as usize;
        let y = layout.location.y.max(0.0) as usize;
        let w = layout.size.width.max(0.0) as usize;
        let h = layout.size.height.max(0.0) as usize;
        if debug_overlay {
            surface.draw_box(x, y, w, h, None);
        }
        if let Some(np) = map.get(&node) {
            let (pl,pr,pt,_pb) = np.pad;
            let content_x = x + pl;
            let content_y = y + pt;
            let content_w = w.saturating_sub(pl + pr);
            if let Some(text) = &np.text {
                paint_text_clipped(surface, content_x, content_y, content_w, text, &np.style);
            }
        }
    }
    if let Ok(children) = taffy.children(node) {
        for c in children { paint_recursive_dbg(taffy, c, surface, map, debug_overlay); }
    }
}

fn paint_text_clipped(surface: &mut Surface, x: usize, y: usize, max_w: usize, text: &str, style: &PaintStyle) {
    // If layout width is zero (no intrinsic size), paint without clip; Surface::write_str will stop at surface width
    let limit = if max_w == 0 { usize::MAX } else { max_w };
    let mut used = 0usize;
    for ch in text.chars() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        if used + w > limit { break; }
        surface.write_str(x + used, y, &ch.to_string(), style.fg, style.bg, style.attr);
        used += w;
    }
}


