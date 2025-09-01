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

// Type aliases for complex types
type NodeLayoutInfo = (NodeId, usize, usize, usize, usize);
type LayerMap = BTreeMap<i32, Vec<NodeLayoutInfo>>;
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

/// Layout and paint with explicit height constraint
pub fn layout_and_paint_constrained<'a>(
    root: &NodeSpec<'a>,
    surface: &mut Surface,
    width: usize,
    height: usize,
) {
    let mut taffy = TaffyTree::new();
    let mut map: HashMap<NodeId, NodePaint> = HashMap::new();
    let root_id = build_nodes(&mut taffy, root, &mut map).unwrap_or_else(|_| panic!("Failed to build nodes"));
    let available = Size {
        width: AvailableSpace::Definite(width as f32),
        height: AvailableSpace::Definite(height as f32),
    };
    taffy
        .compute_layout(root_id, available)
        .unwrap_or_else(|_| panic!("Failed to compute layout"));
    paint_with_z_index(&taffy, root_id, surface, &map, false);
}

fn build_nodes<'a>(
    taffy: &mut TaffyTree<()>,
    spec: &NodeSpec<'a>,
    map: &mut HashMap<NodeId, NodePaint>,
) -> Result<NodeId> {
    let mut sb = apply_utility_classes(spec.class.as_ref(), StyleBuilder::new());
    
    // For text nodes, ensure minimum height of 1 cell
    // This ensures text is visible in flexbox column layouts
    if spec.text.is_some() && spec.children.is_empty() {
        // Only set min-height if not already set by the user
        // This is a simple heuristic: if the class doesn't contain 'h-'
        // then we add a minimum height
        if !spec.class.contains("h-") {
            sb = sb.min_height_px(1.0);
        }
    }
    
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
    let mut layers: LayerMap = BTreeMap::new();
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
    layers: &mut LayerMap,
) {
    collect_nodes_by_z_index_recursive(taffy, node, map, layers, 0, 0);
}

/// Recursively collect nodes with absolute positioning
fn collect_nodes_by_z_index_recursive(
    taffy: &TaffyTree<()>,
    node: NodeId,
    map: &HashMap<NodeId, NodePaint>,
    layers: &mut LayerMap,
    parent_x: usize,
    parent_y: usize,
) {
    if let Ok(layout) = taffy.layout(node) {
        // Calculate absolute position by adding parent offset
        let x = parent_x + layout.location.x.max(0.0) as usize;
        let y = parent_y + layout.location.y.max(0.0) as usize;
        let w = layout.size.width.max(0.0) as usize;
        let h = layout.size.height.max(0.0) as usize;

        // Debug output for tests (disabled in production)
        #[cfg(test)]
        if std::env::var("PAINT_TREE_DEBUG").is_ok() {
            eprintln!("Node layout: pos=({},{}) size=({},{}) parent=({},{})",
                     x, y, w, h, parent_x, parent_y);
        }

        // Skip nodes with zero size
        if w > 0 && h > 0 {
            let z_index = map.get(&node).map(|np| np.z_index).unwrap_or(0);
            layers.entry(z_index).or_default().push((node, x, y, w, h));
        }

        // Recurse to children with updated absolute position
        if let Ok(children) = taffy.children(node) {
            for child in children {
                collect_nodes_by_z_index_recursive(taffy, child, map, layers, x, y);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::surface::Surface;
    use std::borrow::Cow;

    #[test]
    fn test_padding_creation() {
        let padding = Padding::new(1, 2, 3, 4);
        assert_eq!(padding.left, 1);
        assert_eq!(padding.right, 2);
        assert_eq!(padding.top, 3);
        assert_eq!(padding._bottom, 4);
    }

    #[test]
    fn test_padding_default() {
        let padding = Padding::default();
        assert_eq!(padding.left, 0);
        assert_eq!(padding.right, 0);
        assert_eq!(padding.top, 0);
        assert_eq!(padding._bottom, 0);
    }

    #[test]
    fn test_paint_options_default() {
        let opts = PaintOptions::default();
        assert!(!opts.debug_overlay);
    }

    #[test]
    fn test_paint_options_debug() {
        let opts = PaintOptions {
            debug_overlay: true,
        };
        assert!(opts.debug_overlay);
    }

    #[test]
    fn test_node_spec_creation() {
        let spec = NodeSpec {
            class: Cow::Borrowed("flex p-4"),
            text: Some(Cow::Borrowed("Hello")),
            children: vec![],
        };

        assert_eq!(spec.class, "flex p-4");
        assert_eq!(spec.text, Some(Cow::Borrowed("Hello")));
        assert_eq!(spec.children.len(), 0);
    }

    #[test]
    fn test_node_spec_with_children() {
        let child1 = NodeSpec {
            class: Cow::Borrowed("text-red"),
            text: Some(Cow::Borrowed("Child 1")),
            children: vec![],
        };

        let child2 = NodeSpec {
            class: Cow::Borrowed("text-blue"),
            text: Some(Cow::Borrowed("Child 2")),
            children: vec![],
        };

        let parent = NodeSpec {
            class: Cow::Borrowed("flex flex-col"),
            text: None,
            children: vec![child1, child2],
        };

        assert_eq!(parent.children.len(), 2);
        assert_eq!(parent.children[0].text, Some(Cow::Borrowed("Child 1")));
        assert_eq!(parent.children[1].text, Some(Cow::Borrowed("Child 2")));
    }

    #[test]
    fn test_layout_and_paint_simple() {
        let mut surface = Surface::new(80, 24);

        let spec = NodeSpec {
            class: Cow::Borrowed("p-2"),
            text: Some(Cow::Borrowed("Test")),
            children: vec![],
        };

        // Should not panic
        layout_and_paint(&spec, &mut surface, 80);

        // Surface should have been modified (basic check)
        assert_eq!(surface.dims(), (80, 24));
    }

    #[test]
    fn test_layout_and_paint_with_options() {
        let mut surface = Surface::new(80, 24);

        let spec = NodeSpec {
            class: Cow::Borrowed("flex"),
            text: Some(Cow::Borrowed("Test")),
            children: vec![],
        };

        let opts = PaintOptions {
            debug_overlay: true,
        };

        // Should not panic with debug overlay
        let result = layout_and_paint_with(&spec, &mut surface, 80, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_layout_and_paint_constrained() {
        let mut surface = Surface::new(80, 24);

        let spec = NodeSpec {
            class: Cow::Borrowed("w-full h-full"),
            text: Some(Cow::Borrowed("Constrained")),
            children: vec![],
        };

        // Should not panic with explicit dimensions
        layout_and_paint_constrained(&spec, &mut surface, 40, 12);

        assert_eq!(surface.dims(), (80, 24));
    }

    #[test]
    fn test_layout_with_nested_children() {
        let mut surface = Surface::new(100, 50);

        let spec = NodeSpec {
            class: Cow::Borrowed("flex flex-col p-4"),
            text: None,
            children: vec![
                NodeSpec {
                    class: Cow::Borrowed("text-lg"),
                    text: Some(Cow::Borrowed("Header")),
                    children: vec![],
                },
                NodeSpec {
                    class: Cow::Borrowed("flex flex-row gap-2"),
                    text: None,
                    children: vec![
                        NodeSpec {
                            class: Cow::Borrowed("flex-1"),
                            text: Some(Cow::Borrowed("Left")),
                            children: vec![],
                        },
                        NodeSpec {
                            class: Cow::Borrowed("flex-1"),
                            text: Some(Cow::Borrowed("Right")),
                            children: vec![],
                        },
                    ],
                },
            ],
        };

        // Should handle complex nested layout
        let result = layout_and_paint_with(&spec, &mut surface, 100, &PaintOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_layout_with_z_index() {
        let mut surface = Surface::new(80, 24);

        let spec = NodeSpec {
            class: Cow::Borrowed("relative"),
            text: None,
            children: vec![
                NodeSpec {
                    class: Cow::Borrowed("z-10"),
                    text: Some(Cow::Borrowed("Top layer")),
                    children: vec![],
                },
                NodeSpec {
                    class: Cow::Borrowed("z-1"),
                    text: Some(Cow::Borrowed("Bottom layer")),
                    children: vec![],
                },
            ],
        };

        // Should handle z-index layering
        let result = layout_and_paint_with(&spec, &mut surface, 80, &PaintOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_layout_with_overflow_hidden() {
        let mut surface = Surface::new(80, 24);

        let spec = NodeSpec {
            class: Cow::Borrowed("overflow-hidden w-10 h-5"),
            text: Some(Cow::Borrowed("This is a very long text that should be clipped")),
            children: vec![],
        };

        // Should handle overflow clipping
        let result = layout_and_paint_with(&spec, &mut surface, 80, &PaintOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_node_spec() {
        let mut surface = Surface::new(80, 24);

        let spec = NodeSpec {
            class: Cow::Borrowed(""),
            text: None,
            children: vec![],
        };

        // Should handle empty spec
        let result = layout_and_paint_with(&spec, &mut surface, 80, &PaintOptions::default());
        assert!(result.is_ok());
    }
}
