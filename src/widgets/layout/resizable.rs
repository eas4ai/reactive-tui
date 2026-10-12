//! The resizable split (docs/spec/layout-widgets.md, NAV-007 and NAV-008):
//! panes side by side or stacked, with a divider between each pair that a
//! key or a drag moves.
//!
//! This is the skeleton of the widget: its props and callbacks are real,
//! and it renders an empty element until the rendering is written.

use crate::component::{Component, Element, Props};
use std::any::Any;
use std::sync::Arc;

/// The callback called with every pane's size after a move.
pub(crate) type ResizeCallback = Arc<dyn Fn(Vec<u16>) + Send + Sync>;

/// The default minimum size of a pane, in cells.
const DEFAULT_MIN: u16 = 4;

/// Which way the panes are laid out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitDirection {
    /// Panes side by side, with a vertical divider `│` between them.
    #[default]
    Row,
    /// Panes stacked, with a horizontal divider `─` between them.
    Column,
}

/// How big a pane is asked to be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaneSize {
    /// A fixed number of cells.
    Cells(u16),
    /// A share of the cells left after the fixed panes, by weight.
    Weight(f32),
}

/// One pane of a split: its content, its size and the least it can take.
#[derive(Clone, PartialEq)]
pub struct Pane {
    /// The element shown in the pane.
    pub content: Element,
    /// The pane's size; none takes an equal share.
    pub size: Option<PaneSize>,
    /// The fewest cells the pane can be moved down to.
    pub min: u16,
}

impl Pane {
    /// Create a pane of `content` with an equal share and the default
    /// minimum of four cells.
    pub fn new(content: Element) -> Self {
        Self {
            content,
            size: None,
            min: DEFAULT_MIN,
        }
    }

    /// Ask for a fixed number of cells.
    pub fn size(mut self, cells: u16) -> Self {
        self.size = Some(PaneSize::Cells(cells));
        self
    }

    /// Ask for a share of the leftover cells by weight.
    pub fn weight(mut self, weight: f32) -> Self {
        self.size = Some(PaneSize::Weight(weight));
        self
    }

    /// Set the fewest cells the pane can take.
    pub fn min(mut self, cells: u16) -> Self {
        self.min = cells;
        self
    }
}

/// Properties for the resizable split.
#[derive(Clone, Default, PartialEq)]
pub struct ResizableProps {
    /// Which way the panes are laid out.
    pub direction: SplitDirection,
    /// The panes, in order.
    pub panes: Vec<Pane>,
    /// The name the screen reader hears for the split.
    pub aria_label: Option<String>,
    /// Whether the dividers ignore keys and drags.
    pub disabled: bool,
}

impl Props for ResizableProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A resizable split: panes with a divider between each pair.
pub struct Resizable {
    /// Called with every pane's size after each move.
    pub(crate) on_resize: Option<ResizeCallback>,
}

impl Resizable {
    /// Create a split from its props, with no callback yet.
    pub fn new(_props: ResizableProps) -> Self {
        Self { on_resize: None }
    }

    /// Set the callback called with every pane's size after each move.
    pub fn with_on_resize(mut self, f: impl Fn(Vec<u16>) + Send + Sync + 'static) -> Self {
        self.on_resize = Some(Arc::new(f));
        self
    }
}

impl Component for Resizable {
    type Props = ResizableProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn render(&self, _props: &Self::Props, _state: &Self::State) -> Element {
        // Skeleton: the split's rendering is written by the widget's job.
        crate::builder::div().build()
    }
}
