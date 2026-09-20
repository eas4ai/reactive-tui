//! layout-engine commitment tests (LAY-001 … LAY-004), ported from
//! `tests/yoga_test.zig` plus feature vectors for the LAY-001 surface.

use suprtui::layout::{
    Align, FlexDirectionName, FlexStyle, Justify, LayoutTree, LengthValue, MeasureTarget, NodeId,
    SizeValue, Wrap,
};

const EPS: f32 = 0.001;

fn approx(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() <= EPS
}

fn points(v: f32) -> SizeValue {
    SizeValue::Points(v)
}

fn fixed(w: f32, h: f32) -> FlexStyle {
    FlexStyle {
        size: (points(w), points(h)),
        ..FlexStyle::default()
    }
}

fn leaf(tree: &mut LayoutTree, style: &FlexStyle) -> NodeId {
    tree.new_leaf(style).unwrap()
}

// ---------------------------------------------------------------------------
// LAY-001: row/column, grow/shrink, wrap, justification, alignment,
// margins, padding, gaps, min/max.
// ---------------------------------------------------------------------------

#[test]
fn req_001_flex() {
    // Ported fixture: row root 100x100 with a growing child.
    let mut tree = LayoutTree::new();
    let root_style = FlexStyle {
        direction: FlexDirectionName::Row,
        size: (points(100.0), points(100.0)),
        ..FlexStyle::default()
    };
    let root = tree.new_leaf(&root_style).unwrap();
    let child_style = FlexStyle {
        grow: 1.0,
        ..FlexStyle::default()
    };
    let child = leaf(&mut tree, &child_style);
    tree.add_child(root, child).unwrap();

    tree.compute(root, None, None).unwrap();
    let layout = tree.computed(child).unwrap();
    assert!(approx(layout.width, 100.0), "width {}", layout.width);
    assert!(approx(layout.height, 100.0), "height {}", layout.height);

    // Shrink: two children that overflow share the deficit.
    let mut tree = LayoutTree::new();
    let root_style = FlexStyle {
        direction: FlexDirectionName::Row,
        size: (points(100.0), points(50.0)),
        ..FlexStyle::default()
    };
    let root = tree.new_leaf(&root_style).unwrap();
    let wide = FlexStyle {
        size: (points(70.0), points(50.0)),
        shrink: 1.0,
        ..FlexStyle::default()
    };
    let a = leaf(&mut tree, &wide);
    let b = leaf(&mut tree, &wide);
    tree.add_child(root, a).unwrap();
    tree.add_child(root, b).unwrap();
    tree.compute(root, None, None).unwrap();
    let la = tree.computed(a).unwrap();
    let lb = tree.computed(b).unwrap();
    assert!(approx(la.width, 50.0), "shrink a {}", la.width);
    assert!(approx(lb.width, 50.0), "shrink b {}", lb.width);
    assert!(approx(lb.x, 50.0), "shrink b.x {}", lb.x);

    // Wrap: 60-wide children in a 100-wide row break onto two lines.
    let mut tree = LayoutTree::new();
    let root_style = FlexStyle {
        direction: FlexDirectionName::Row,
        wrap: Wrap::Wrap,
        size: (points(100.0), SizeValue::Auto),
        ..FlexStyle::default()
    };
    let root = tree.new_leaf(&root_style).unwrap();
    let kid = fixed(60.0, 30.0);
    let k0 = leaf(&mut tree, &kid);
    let k1 = leaf(&mut tree, &kid);
    let k2 = leaf(&mut tree, &kid);
    tree.add_child(root, k0).unwrap();
    tree.add_child(root, k1).unwrap();
    tree.add_child(root, k2).unwrap();
    tree.compute(root, None, None).unwrap();
    let l0 = tree.computed(k0).unwrap();
    let l1 = tree.computed(k1).unwrap();
    let l2 = tree.computed(k2).unwrap();
    assert!(approx(l0.x, 0.0) && approx(l0.y, 0.0));
    assert!(approx(l1.x, 0.0) && approx(l1.y, 30.0), "wrap {l1:?}");
    assert!(approx(l2.x, 0.0) && approx(l2.y, 60.0), "wrap {l2:?}");

    // Center justification and gaps.
    let mut tree = LayoutTree::new();
    let root_style = FlexStyle {
        direction: FlexDirectionName::Row,
        justify: Justify::Center,
        gap: (LengthValue::Points(10.0), LengthValue::Zero),
        size: (points(100.0), points(40.0)),
        ..FlexStyle::default()
    };
    let root = tree.new_leaf(&root_style).unwrap();
    let small = fixed(20.0, 20.0);
    let c0 = leaf(&mut tree, &small);
    let c1 = leaf(&mut tree, &small);
    tree.add_child(root, c0).unwrap();
    tree.add_child(root, c1).unwrap();
    tree.compute(root, None, None).unwrap();
    let lc0 = tree.computed(c0).unwrap();
    let lc1 = tree.computed(c1).unwrap();
    // Content spans 20 + 10 + 20 = 50, centered in 100.
    assert!(approx(lc0.x, 25.0), "center {}", lc0.x);
    assert!(approx(lc1.x, 55.0), "gap {}", lc1.x);

    // Padding offsets children; min/max clamp sizes.
    let mut tree = LayoutTree::new();
    let root_style = FlexStyle {
        align_items: Align::Start,
        padding: suprtui::layout::Edges::uniform(LengthValue::Points(5.0)),
        size: (points(100.0), points(100.0)),
        ..FlexStyle::default()
    };
    let root = tree.new_leaf(&root_style).unwrap();
    let kid = FlexStyle {
        size: (SizeValue::Auto, points(10.0)),
        min_size: (points(50.0), SizeValue::Auto),
        max_size: (points(1000.0), SizeValue::Auto),
        ..FlexStyle::default()
    };
    let k = leaf(&mut tree, &kid);
    tree.add_child(root, k).unwrap();
    tree.compute(root, None, None).unwrap();
    let lk = tree.computed(k).unwrap();
    assert!(approx(lk.x, 5.0) && approx(lk.y, 5.0), "pad {lk:?}");
    assert!(approx(lk.width, 50.0), "min {}", lk.width);

    // Margins separate siblings.
    let mut tree = LayoutTree::new();
    let root_style = FlexStyle {
        direction: FlexDirectionName::Row,
        size: (points(100.0), points(20.0)),
        ..FlexStyle::default()
    };
    let root = tree.new_leaf(&root_style).unwrap();
    let m0 = leaf(&mut tree, &fixed(20.0, 20.0));
    let margined = FlexStyle {
        margin: suprtui::layout::Edges {
            left: LengthValue::Points(7.0),
            ..Default::default()
        },
        size: (points(20.0), points(20.0)),
        ..FlexStyle::default()
    };
    let m1 = leaf(&mut tree, &margined);
    tree.add_child(root, m0).unwrap();
    tree.add_child(root, m1).unwrap();
    tree.compute(root, None, None).unwrap();
    assert!(approx(tree.computed(m1).unwrap().x, 27.0));

    // Cross-axis centering.
    let mut tree = LayoutTree::new();
    let root_style = FlexStyle {
        direction: FlexDirectionName::Row,
        align_items: Align::Center,
        size: (points(100.0), points(100.0)),
        ..FlexStyle::default()
    };
    let root = tree.new_leaf(&root_style).unwrap();
    let k = leaf(&mut tree, &fixed(20.0, 20.0));
    tree.add_child(root, k).unwrap();
    tree.compute(root, None, None).unwrap();
    assert!(approx(tree.computed(k).unwrap().y, 40.0));
}

// ---------------------------------------------------------------------------
// LAY-002: classical flex defaults, point scale one.
// ---------------------------------------------------------------------------

#[test]
fn req_002_config_defaults() {
    // Classical defaults: column direction, not the web row default.
    assert_eq!(
        LayoutTree::default_style().direction,
        FlexDirectionName::Column
    );
    assert_eq!(LayoutTree::point_scale_factor(), 1.0);

    // An auto-sized node stacks children vertically under classical
    // defaults; web defaults would place them side by side.
    let mut tree = LayoutTree::new();
    let root = tree.new_leaf(&FlexStyle::default()).unwrap();
    let k0 = leaf(&mut tree, &fixed(10.0, 10.0));
    let k1 = leaf(&mut tree, &fixed(10.0, 10.0));
    tree.add_child(root, k0).unwrap();
    tree.add_child(root, k1).unwrap();
    tree.compute(root, None, None).unwrap();
    let l0 = tree.computed(k0).unwrap();
    let l1 = tree.computed(k1).unwrap();
    let lr = tree.computed(root).unwrap();
    assert!(approx(l0.x, 0.0) && approx(l0.y, 0.0));
    assert!(approx(l1.x, 0.0) && approx(l1.y, 10.0), "column {l1:?}");
    assert!(
        approx(lr.width, 10.0) && approx(lr.height, 20.0),
        "auto {lr:?}"
    );
}

// ---------------------------------------------------------------------------
// LAY-003: measure targets.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct TextTarget {
    text: std::rc::Rc<std::cell::RefCell<String>>,
}

impl TextTarget {
    fn of(text: &str) -> (Self, std::rc::Rc<std::cell::RefCell<String>>) {
        let shared = std::rc::Rc::new(std::cell::RefCell::new(text.to_string()));
        (
            Self {
                text: shared.clone(),
            },
            shared,
        )
    }
}

impl MeasureTarget for TextTarget {
    fn measure(&mut self, _width: Option<f32>, _height: Option<f32>) -> (f32, f32) {
        (self.text.borrow().chars().count() as f32 * 10.0, 20.0)
    }
}

#[test]
fn req_003_measure_targets() {
    let mut tree = LayoutTree::new();
    let root_style = FlexStyle {
        direction: FlexDirectionName::Row,
        align_items: Align::Start,
        size: (points(200.0), points(200.0)),
        ..FlexStyle::default()
    };
    let root = tree.new_leaf(&root_style).unwrap();
    let leaf_node = leaf(&mut tree, &FlexStyle::default());
    tree.add_child(root, leaf_node).unwrap();

    // Unbound leaves measure zero without consulting anything.
    tree.compute(root, None, None).unwrap();
    assert!(approx(tree.computed(leaf_node).unwrap().width, 0.0));
    assert_eq!(tree.measure_calls(leaf_node), 0);

    // Binding consults the target during layout.
    let (target, shared) = TextTarget::of("hi");
    tree.bind_measure(leaf_node, Box::new(target)).unwrap();
    tree.compute(root, None, None).unwrap();
    assert!(approx(tree.computed(leaf_node).unwrap().width, 20.0));
    assert!(approx(tree.computed(leaf_node).unwrap().height, 20.0));
    let calls_after_bind = tree.measure_calls(leaf_node);
    assert!(calls_after_bind > 0);

    // Cached layouts do not remeasure.
    tree.compute(root, None, None).unwrap();
    assert_eq!(tree.measure_calls(leaf_node), calls_after_bind);

    // Changing the bound text changes the laid-out size.
    assert!(tree.target_mut(leaf_node).is_some());
    *shared.borrow_mut() = "hello world".to_string();
    tree.mark_dirty(leaf_node).unwrap();
    tree.compute(root, None, None).unwrap();
    assert!(approx(tree.computed(leaf_node).unwrap().width, 110.0));
    assert!(tree.measure_calls(leaf_node) > calls_after_bind);

    // Clearing stops all measuring; the leaf goes back to zero.
    let calls_before_clear = tree.measure_calls(leaf_node);
    tree.clear_measure(leaf_node).unwrap();
    tree.compute(root, None, None).unwrap();
    assert_eq!(tree.measure_calls(leaf_node), calls_before_clear);
    assert!(approx(tree.computed(leaf_node).unwrap().width, 0.0));
}

// ---------------------------------------------------------------------------
// LAY-004: absolute computed positions for nested children.
// ---------------------------------------------------------------------------

#[test]
fn req_004_computed_positions() {
    let mut tree = LayoutTree::new();
    let root_style = FlexStyle {
        padding: suprtui::layout::Edges::uniform(LengthValue::Points(10.0)),
        size: (points(200.0), points(200.0)),
        ..FlexStyle::default()
    };
    let root = tree.new_leaf(&root_style).unwrap();

    let child_style = FlexStyle {
        margin: suprtui::layout::Edges {
            left: LengthValue::Points(5.0),
            top: LengthValue::Points(5.0),
            ..Default::default()
        },
        padding: suprtui::layout::Edges::uniform(LengthValue::Points(3.0)),
        size: (points(100.0), points(100.0)),
        ..FlexStyle::default()
    };
    let child = leaf(&mut tree, &child_style);

    let grandchild = leaf(&mut tree, &fixed(20.0, 20.0));
    tree.add_child(child, grandchild).unwrap();
    tree.add_child(root, child).unwrap();
    tree.compute(root, None, None).unwrap();

    let lr = tree.computed(root).unwrap();
    assert!(approx(lr.x, 0.0) && approx(lr.y, 0.0));
    let lc = tree.computed(child).unwrap();
    assert!(approx(lc.x, 15.0) && approx(lc.y, 15.0), "child {lc:?}");
    let lg = tree.computed(grandchild).unwrap();
    assert!(
        approx(lg.x, 18.0) && approx(lg.y, 18.0),
        "grandchild {lg:?}"
    );
    assert!(approx(lg.width, 20.0) && approx(lg.height, 20.0));

    // Removed nodes fail instead of returning stale geometry.
    let doomed = leaf(&mut tree, &fixed(5.0, 5.0));
    tree.remove(doomed).unwrap();
    assert!(tree.computed(doomed).is_err());
}
