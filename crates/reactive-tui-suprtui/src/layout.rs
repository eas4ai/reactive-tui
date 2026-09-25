//! Flex layout on the decided `taffy` engine (LAY-001 … LAY-004).
//!
//! Ports `yoga.zig`'s wrapper surface. Classical Yoga defaults come
//! from the wrapper's style constructor — column direction with a
//! point scale of one and no rounding — mirroring
//! `YGConfigSetUseWebDefaults(config, false)` plus
//! `YGConfigSetPointScaleFactor(config, 1)`. Measure targets are
//! caller-owned boxes routed by node id, replacing the reference
//! global callback routing; there are no process-global configs.

use std::collections::HashMap;
use taffy::{
    AlignContent, AlignItems, AlignSelf, AvailableSpace, Dimension, Display, FlexDirection,
    FlexWrap, JustifyContent, LengthPercentage, LengthPercentageAuto, NodeId as TaffyId, Size,
    Style, TaffyError, TaffyTree,
};

/// Layout failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// Unknown, removed, or foreign node id.
    InvalidNode,
    /// The engine rejected the tree state.
    InvalidTree,
}

impl From<TaffyError> for LayoutError {
    fn from(_: TaffyError) -> Self {
        LayoutError::InvalidTree
    }
}

/// Opaque handle to a node owned by a [`LayoutTree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(TaffyId);

/// Main-axis direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirectionName {
    /// Main axis runs left to right.
    Row,
    /// Classical Yoga default (not the web default).
    #[default]
    Column,
    /// Main axis runs right to left.
    RowReverse,
    /// Main axis runs bottom to top.
    ColumnReverse,
}

/// Wrapping behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Wrap {
    /// Keep all items on one line.
    #[default]
    NoWrap,
    /// Wrap items onto new lines.
    Wrap,
    /// Wrap items onto new lines in reverse cross-axis order.
    WrapReverse,
}

/// Main-axis distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Justify {
    /// Pack items at the main-axis start.
    #[default]
    Start,
    /// Center items on the main axis.
    Center,
    /// Pack items at the main-axis end.
    End,
    /// Equal space between items and none at the edges.
    SpaceBetween,
    /// Equal space around each item, so edge gaps are half the inner gaps.
    SpaceAround,
    /// Equal space between items and at both edges.
    SpaceEvenly,
}

/// Cross-axis alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    /// Align items to the cross-axis start.
    Start,
    /// Center items on the cross axis.
    Center,
    /// Align items to the cross-axis end.
    End,
    /// Stretch items to fill the cross axis.
    #[default]
    Stretch,
}

/// Per-item cross-axis override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignSelfName {
    /// Use the parent's `align_items`.
    #[default]
    Auto,
    /// Align this item to the cross-axis start.
    Start,
    /// Center this item on the cross axis.
    Center,
    /// Align this item to the cross-axis end.
    End,
    /// Stretch this item to fill the cross axis.
    Stretch,
}

/// Multi-line cross-axis distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignContentName {
    /// Pack lines at the cross-axis start.
    #[default]
    Start,
    /// Center lines on the cross axis.
    Center,
    /// Pack lines at the cross-axis end.
    End,
    /// Stretch lines to fill the cross axis.
    Stretch,
    /// Equal space between lines and none at the edges.
    SpaceBetween,
    /// Equal space around each line.
    SpaceAround,
}

/// Auto, points, or percent of the parent axis.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SizeValue {
    /// Size from content and flex rules.
    #[default]
    Auto,
    /// Fixed size in points.
    Points(f32),
    /// Percent of the parent axis, where `50.0` is half.
    Percent(f32),
}

/// Points or percent of the parent axis.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LengthValue {
    /// No length.
    #[default]
    Zero,
    /// Fixed length in points.
    Points(f32),
    /// Percent of the parent axis, where `50.0` is half.
    Percent(f32),
}

/// Edges in left/right/top/bottom order.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Edges<T> {
    /// Left edge.
    pub left: T,
    /// Right edge.
    pub right: T,
    /// Top edge.
    pub top: T,
    /// Bottom edge.
    pub bottom: T,
}

impl<T: Default> Edges<T> {
    /// Use one value for all four edges.
    pub fn uniform(value: T) -> Self
    where
        T: Clone,
    {
        Self {
            left: value.clone(),
            right: value.clone(),
            top: value.clone(),
            bottom: value,
        }
    }
}

/// Flex style with classical Yoga defaults: column direction, no wrap,
/// zero grow/shrink, auto sizes. Mirrors the fixed OpenTUI config.
#[derive(Debug, Clone, PartialEq)]
pub struct FlexStyle {
    /// Main-axis direction.
    pub direction: FlexDirectionName,
    /// Whether items wrap onto new lines.
    pub wrap: Wrap,
    /// Main-axis distribution of children.
    pub justify: Justify,
    /// Cross-axis alignment of children.
    pub align_items: Align,
    /// Cross-axis alignment of this node, overriding the parent.
    pub align_self: AlignSelfName,
    /// Cross-axis distribution of wrapped lines.
    pub align_content: AlignContentName,
    /// Share of free main-axis space this node takes.
    pub grow: f32,
    /// Share of overflow this node gives up; `0.0` never shrinks.
    pub shrink: f32,
    /// Main-axis size before grow and shrink apply.
    pub basis: SizeValue,
    /// Width and height.
    pub size: (SizeValue, SizeValue),
    /// Minimum width and height.
    pub min_size: (SizeValue, SizeValue),
    /// Maximum width and height.
    pub max_size: (SizeValue, SizeValue),
    /// Outer spacing per edge.
    pub margin: Edges<LengthValue>,
    /// Inner spacing per edge.
    pub padding: Edges<LengthValue>,
    /// Space between children as (horizontal, vertical).
    pub gap: (LengthValue, LengthValue),
}

impl Default for FlexStyle {
    fn default() -> Self {
        Self {
            direction: FlexDirectionName::Column,
            wrap: Wrap::NoWrap,
            justify: Justify::Start,
            align_items: Align::Stretch,
            align_self: AlignSelfName::Auto,
            align_content: AlignContentName::Start,
            grow: 0.0,
            shrink: 0.0,
            basis: SizeValue::Auto,
            size: (SizeValue::Auto, SizeValue::Auto),
            min_size: (SizeValue::Auto, SizeValue::Auto),
            max_size: (SizeValue::Auto, SizeValue::Auto),
            margin: Edges::default(),
            padding: Edges::default(),
            gap: (LengthValue::Zero, LengthValue::Zero),
        }
    }
}

fn map_size(value: SizeValue) -> Dimension {
    match value {
        SizeValue::Auto => Dimension::auto(),
        SizeValue::Points(v) => Dimension::length(v),
        SizeValue::Percent(v) => Dimension::percent(v / 100.0),
    }
}

fn map_length(value: LengthValue) -> LengthPercentage {
    match value {
        LengthValue::Zero => LengthPercentage::length(0.0),
        LengthValue::Points(v) => LengthPercentage::length(v),
        LengthValue::Percent(v) => LengthPercentage::percent(v / 100.0),
    }
}

fn map_margin(value: LengthValue) -> LengthPercentageAuto {
    match value {
        LengthValue::Zero => LengthPercentageAuto::length(0.0),
        LengthValue::Points(v) => LengthPercentageAuto::length(v),
        LengthValue::Percent(v) => LengthPercentageAuto::percent(v / 100.0),
    }
}

impl FlexStyle {
    fn to_taffy(&self) -> Style {
        let direction = match self.direction {
            FlexDirectionName::Row => FlexDirection::Row,
            FlexDirectionName::Column => FlexDirection::Column,
            FlexDirectionName::RowReverse => FlexDirection::RowReverse,
            FlexDirectionName::ColumnReverse => FlexDirection::ColumnReverse,
        };
        let wrap = match self.wrap {
            Wrap::NoWrap => FlexWrap::NoWrap,
            Wrap::Wrap => FlexWrap::Wrap,
            Wrap::WrapReverse => FlexWrap::WrapReverse,
        };
        let justify = match self.justify {
            Justify::Start => JustifyContent::START,
            Justify::Center => JustifyContent::CENTER,
            Justify::End => JustifyContent::END,
            Justify::SpaceBetween => JustifyContent::SPACE_BETWEEN,
            Justify::SpaceAround => JustifyContent::SPACE_AROUND,
            Justify::SpaceEvenly => JustifyContent::SPACE_EVENLY,
        };
        let align_items = match self.align_items {
            Align::Start => AlignItems::START,
            Align::Center => AlignItems::CENTER,
            Align::End => AlignItems::END,
            Align::Stretch => AlignItems::STRETCH,
        };
        let align_self = match self.align_self {
            AlignSelfName::Auto => None,
            AlignSelfName::Start => Some(AlignSelf::START),
            AlignSelfName::Center => Some(AlignSelf::CENTER),
            AlignSelfName::End => Some(AlignSelf::END),
            AlignSelfName::Stretch => Some(AlignSelf::STRETCH),
        };
        let align_content = match self.align_content {
            AlignContentName::Start => AlignContent::START,
            AlignContentName::Center => AlignContent::CENTER,
            AlignContentName::End => AlignContent::END,
            AlignContentName::Stretch => AlignContent::STRETCH,
            AlignContentName::SpaceBetween => AlignContent::SPACE_BETWEEN,
            AlignContentName::SpaceAround => AlignContent::SPACE_AROUND,
        };
        Style {
            display: Display::Flex,
            flex_direction: direction,
            flex_wrap: wrap,
            flex_grow: self.grow,
            flex_shrink: self.shrink,
            flex_basis: map_size(self.basis),
            size: Size {
                width: map_size(self.size.0),
                height: map_size(self.size.1),
            },
            min_size: Size {
                width: map_size(self.min_size.0),
                height: map_size(self.min_size.1),
            },
            max_size: Size {
                width: map_size(self.max_size.0),
                height: map_size(self.max_size.1),
            },
            margin: taffy::Rect {
                left: map_margin(self.margin.left),
                right: map_margin(self.margin.right),
                top: map_margin(self.margin.top),
                bottom: map_margin(self.margin.bottom),
            },
            padding: taffy::Rect {
                left: map_length(self.padding.left),
                right: map_length(self.padding.right),
                top: map_length(self.padding.top),
                bottom: map_length(self.padding.bottom),
            },
            gap: Size {
                width: map_length(self.gap.0),
                height: map_length(self.gap.1),
            },
            justify_content: Some(justify),
            align_items: Some(align_items),
            align_self,
            align_content: Some(align_content),
            ..Default::default()
        }
    }
}

/// Intrinsic-size probe bound to at most one node (LAY-003): a text
/// view or an editor view behind a two-float interface.
pub trait MeasureTarget {
    /// Measure under optional axis constraints; returns content size.
    fn measure(&mut self, width: Option<f32>, height: Option<f32>) -> (f32, f32);
}

/// Absolute computed geometry for one node (LAY-004).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComputedLayout {
    /// Left edge, including all ancestor offsets.
    pub x: f32,
    /// Top edge, including all ancestor offsets.
    pub y: f32,
    /// Computed width.
    pub width: f32,
    /// Computed height.
    pub height: f32,
}

/// Caller-owned flex tree. Nodes are never shared between trees and no
/// configuration is process-global.
pub struct LayoutTree {
    taffy: TaffyTree<()>,
    measures: HashMap<TaffyId, Box<dyn MeasureTarget>>,
    calls: HashMap<TaffyId, u64>,
    live: std::collections::HashSet<TaffyId>,
}

impl LayoutTree {
    /// Fixed OpenTUI configuration: classical defaults with a point
    /// scale of one and no rounding, so floats reach the caller within
    /// the LAY-001 epsilon.
    pub fn new() -> Self {
        let mut taffy = TaffyTree::new();
        taffy.disable_rounding();
        Self {
            taffy,
            measures: HashMap::new(),
            calls: HashMap::new(),
            live: std::collections::HashSet::new(),
        }
    }

    /// Taffy indexes storage directly and panics on stale ids, so every
    /// entry point validates liveness first and reports InvalidNode.
    fn check(&self, node: NodeId) -> Result<TaffyId, LayoutError> {
        if self.live.contains(&node.0) {
            Ok(node.0)
        } else {
            Err(LayoutError::InvalidNode)
        }
    }

    /// Classical Yoga defaults (not web defaults): column direction.
    pub fn default_style() -> FlexStyle {
        FlexStyle::default()
    }

    /// The point scale factor is one by construction: sizes are
    /// unrounded points.
    pub fn point_scale_factor() -> f32 {
        1.0
    }

    /// Create a node with no children.
    pub fn new_leaf(&mut self, style: &FlexStyle) -> Result<NodeId, LayoutError> {
        let id = self
            .taffy
            .new_leaf(style.to_taffy())
            .map_err(|_| LayoutError::InvalidTree)?;
        self.live.insert(id);
        Ok(NodeId(id))
    }

    /// Create a node that owns `children` in the given order.
    /// Fails with [`LayoutError::InvalidNode`] when a child id is not live.
    pub fn new_parent(
        &mut self,
        style: &FlexStyle,
        children: &[NodeId],
    ) -> Result<NodeId, LayoutError> {
        let mut ids = Vec::with_capacity(children.len());
        for child in children {
            ids.push(self.check(*child)?);
        }
        let id = self
            .taffy
            .new_with_children(style.to_taffy(), &ids)
            .map_err(|_| LayoutError::InvalidTree)?;
        self.live.insert(id);
        Ok(NodeId(id))
    }

    /// Replace the node's style.
    pub fn set_style(&mut self, node: NodeId, style: &FlexStyle) -> Result<(), LayoutError> {
        let id = self.check(node)?;
        self.taffy
            .set_style(id, style.to_taffy())
            .map_err(|_| LayoutError::InvalidNode)
    }

    /// Append `child` to the end of `parent`'s children.
    pub fn add_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), LayoutError> {
        let parent = self.check(parent)?;
        let child = self.check(child)?;
        self.taffy
            .add_child(parent, child)
            .map_err(|_| LayoutError::InvalidNode)
    }

    /// Remove the node and drop its measure target.
    /// Its children stay in the tree, detached from any parent.
    pub fn remove(&mut self, node: NodeId) -> Result<(), LayoutError> {
        let id = self.check(node)?;
        self.measures.remove(&id);
        self.calls.remove(&id);
        self.live.remove(&id);
        self.taffy
            .remove(id)
            .map(|_| ())
            .map_err(|_| LayoutError::InvalidNode)
    }

    /// Mark the node and its ancestors for relayout.
    pub fn mark_dirty(&mut self, node: NodeId) -> Result<(), LayoutError> {
        let id = self.check(node)?;
        self.taffy
            .mark_dirty(id)
            .map_err(|_| LayoutError::InvalidNode)
    }

    /// Bind the node's single measure target, replacing any previous
    /// one (LAY-003).
    pub fn bind_measure(
        &mut self,
        node: NodeId,
        target: Box<dyn MeasureTarget>,
    ) -> Result<(), LayoutError> {
        let id = self.check(node)?;
        self.measures.insert(id, target);
        self.mark_dirty(node)
    }

    /// Clear the measure target: later layouts never consult it
    /// (LAY-003).
    pub fn clear_measure(&mut self, node: NodeId) -> Result<(), LayoutError> {
        let id = self.check(node)?;
        self.measures.remove(&id);
        self.mark_dirty(node)
    }

    /// Borrow the bound target, e.g. to change its text.
    pub fn target_mut(&mut self, node: NodeId) -> Option<&mut Box<dyn MeasureTarget>> {
        if self.live.contains(&node.0) {
            self.measures.get_mut(&node.0)
        } else {
            None
        }
    }

    /// How many times layout consulted the node's target.
    pub fn measure_calls(&self, node: NodeId) -> u64 {
        self.calls.get(&node.0).copied().unwrap_or(0)
    }

    /// Lay out the subtree. `None` axes are unbounded (Yoga NaN). A removed
    /// root fails with `InvalidNode`.
    pub fn compute(
        &mut self,
        root: NodeId,
        width: Option<f32>,
        height: Option<f32>,
    ) -> Result<(), LayoutError> {
        let root = self.check(root)?;
        let space = Size {
            width: width.map_or(AvailableSpace::MaxContent, AvailableSpace::Definite),
            height: height.map_or(AvailableSpace::MaxContent, AvailableSpace::Definite),
        };
        let taffy = &mut self.taffy;
        let measures = &mut self.measures;
        let calls = &mut self.calls;
        taffy
            .compute_layout_with_measure(root, space, |known, _available, id, _context, _style| {
                if let Some(target) = measures.get_mut(&id) {
                    *calls.entry(id).or_insert(0) += 1;
                    let (w, h) = target.measure(known.width, known.height);
                    Size {
                        width: w,
                        height: h,
                    }
                } else {
                    Size::ZERO
                }
            })
            .map_err(|_| LayoutError::InvalidNode)
    }

    /// Absolute position and size, accumulating ancestor offsets so
    /// nested children report page coordinates (LAY-004).
    pub fn computed(&self, node: NodeId) -> Result<ComputedLayout, LayoutError> {
        let id = self.check(node)?;
        let layout = self
            .taffy
            .layout(id)
            .map_err(|_| LayoutError::InvalidNode)?;
        let (mut x, mut y) = (layout.location.x, layout.location.y);
        let mut cursor = self.taffy.parent(node.0);
        while let Some(id) = cursor {
            let parent = self
                .taffy
                .layout(id)
                .map_err(|_| LayoutError::InvalidNode)?;
            x += parent.location.x;
            y += parent.location.y;
            cursor = self.taffy.parent(id);
        }
        Ok(ComputedLayout {
            x,
            y,
            width: layout.size.width,
            height: layout.size.height,
        })
    }
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}
