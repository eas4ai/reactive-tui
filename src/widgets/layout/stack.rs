use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::Event;
use std::any::Any;

/// Stack layout direction
#[derive(Clone, Debug, PartialEq)]
pub enum StackDirection {
    /// Horizontal stack (left to right)
    Horizontal,
    /// Vertical stack (top to bottom)
    Vertical,
}

/// Stack alignment options
#[derive(Clone, Debug, PartialEq)]
pub enum StackAlignment {
    /// Align to start (top for horizontal, left for vertical)
    Start,
    /// Center alignment
    Center,
    /// Align to end (bottom for horizontal, right for vertical)
    End,
    /// Stretch to fill available space
    Stretch,
}

/// Stack justification options
#[derive(Clone, Debug, PartialEq)]
pub enum StackJustify {
    /// Pack items to start
    Start,
    /// Center items with equal space on sides
    Center,
    /// Pack items to end
    End,
    /// Equal space between items, no space on ends
    SpaceBetween,
    /// Equal space around items
    SpaceAround,
    /// Equal space between and around items
    SpaceEvenly,
}

/// Padding configuration for Stack component
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StackPadding {
    /// Top padding in pixels
    pub top: usize,
    /// Right padding in pixels
    pub right: usize,
    /// Bottom padding in pixels
    pub bottom: usize,
    /// Left padding in pixels
    pub left: usize,
}

impl StackPadding {
    /// Create padding with the same value on all sides
    pub fn all(padding: usize) -> Self {
        Self {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }

    /// Create symmetric padding (vertical and horizontal)
    pub fn symmetric(vertical: usize, horizontal: usize) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create horizontal padding only
    pub fn horizontal(padding: usize) -> Self {
        Self {
            left: padding,
            right: padding,
            ..Default::default()
        }
    }

    /// Create vertical padding only
    pub fn vertical(padding: usize) -> Self {
        Self {
            top: padding,
            bottom: padding,
            ..Default::default()
        }
    }
}

/// Builder for creating Stack components with a fluent API
#[derive(Clone, Debug)]
pub struct StackBuilder {
    direction: StackDirection,
    spacing: usize,
    alignment: StackAlignment,
    wrap: bool,
    justify: StackJustify,
    padding: StackPadding,
    reverse: bool,
    children: Vec<Element>,
}

impl StackBuilder {
    /// Create a new StackBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the stack direction
    pub fn direction(mut self, direction: StackDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Create a horizontal stack (convenience method)
    pub fn horizontal() -> Self {
        Self::new().direction(StackDirection::Horizontal)
    }

    /// Create a vertical stack (convenience method)
    pub fn vertical() -> Self {
        Self::new().direction(StackDirection::Vertical)
    }

    /// Set spacing between elements
    pub fn spacing(mut self, spacing: usize) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set alignment of child elements
    pub fn alignment(mut self, alignment: StackAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Enable or disable wrapping
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Set how to justify content
    pub fn justify(mut self, justify: StackJustify) -> Self {
        self.justify = justify;
        self
    }

    /// Set padding around the stack
    pub fn padding(mut self, padding: StackPadding) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform padding on all sides
    pub fn padding_all(mut self, padding: usize) -> Self {
        self.padding = StackPadding::all(padding);
        self
    }

    /// Set symmetric padding (vertical and horizontal)
    pub fn padding_symmetric(mut self, vertical: usize, horizontal: usize) -> Self {
        self.padding = StackPadding::symmetric(vertical, horizontal);
        self
    }

    /// Set horizontal padding
    pub fn padding_horizontal(mut self, padding: usize) -> Self {
        self.padding = StackPadding::horizontal(padding);
        self
    }

    /// Set vertical padding
    pub fn padding_vertical(mut self, padding: usize) -> Self {
        self.padding = StackPadding::vertical(padding);
        self
    }

    /// Reverse the order of children
    pub fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    /// Add a child element
    pub fn child(mut self, child: Element) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children.extend(children);
        self
    }

    /// Build the StackProps
    pub fn build(self) -> StackProps {
        StackProps {
            direction: self.direction,
            spacing: self.spacing,
            alignment: self.alignment,
            wrap: self.wrap,
            justify: self.justify,
            padding: self.padding,
            reverse: self.reverse,
            children: self.children,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::typed::<Stack>(self.build())
    }
}

impl Default for StackBuilder {
    fn default() -> Self {
        Self {
            direction: StackDirection::Vertical,
            spacing: 0,
            alignment: StackAlignment::Start,
            wrap: false,
            justify: StackJustify::Start,
            padding: StackPadding::default(),
            reverse: false,
            children: Vec::new(),
        }
    }
}

/// Properties for Stack layout component
#[derive(Clone, Debug, PartialEq)]
pub struct StackProps {
    /// Direction of the stack layout
    pub direction: StackDirection,
    /// Spacing between child elements
    pub spacing: usize,
    /// Alignment of child elements
    pub alignment: StackAlignment,
    /// Whether to allow wrapping to next line/column
    pub wrap: bool,
    /// How to distribute space between elements
    pub justify: StackJustify,
    /// Padding around the stack
    pub padding: StackPadding,
    /// Whether to reverse the order of children
    pub reverse: bool,
    /// Child elements to layout
    pub children: Vec<Element>,
}

impl Default for StackProps {
    fn default() -> Self {
        Self {
            direction: StackDirection::Vertical,
            spacing: 0,
            alignment: StackAlignment::Start,
            wrap: false,
            justify: StackJustify::Start,
            padding: StackPadding::default(),
            reverse: false,
            children: Vec::new(),
        }
    }
}

impl Props for StackProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for Stack component
#[derive(Clone, Debug, Default)]
pub struct StackState {
    /// Width of the viewport in pixels
    pub viewport_width: usize,
    /// Height of the viewport in pixels
    pub viewport_height: usize,
    /// Horizontal scroll offset
    pub scroll_x: usize,
    /// Vertical scroll offset
    pub scroll_y: usize,
}

/// Stack layout component - arranges children in horizontal or vertical stacks
pub struct Stack {
    state: StackState,
}

impl Component for Stack {
    type Props = StackProps;
    type State = StackState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: StackState::default(),
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        stack_element(props, props.spacing as f32)
    }

    fn handle_event(
        &mut self,
        _event: &Event,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> EventResult {
        EventResult::Ignored
    }
}

pub(crate) fn stack_element(props: &StackProps, spacing: f32) -> Element {
    use crate::component::{ElementType, LayoutType};
    use crate::layout::style::{AlignItems, Direction, JustifyContent, StyleBuilder};
    let direction = match (&props.direction, props.reverse) {
        (StackDirection::Horizontal, false) => Direction::Row,
        (StackDirection::Horizontal, true) => Direction::RowReverse,
        (StackDirection::Vertical, false) => Direction::Column,
        (StackDirection::Vertical, true) => Direction::ColumnReverse,
    };
    let alignment = match props.alignment {
        StackAlignment::Start => AlignItems::Start,
        StackAlignment::Center => AlignItems::Center,
        StackAlignment::End => AlignItems::End,
        StackAlignment::Stretch => AlignItems::Stretch,
    };
    let justify = match props.justify {
        StackJustify::Start => JustifyContent::Start,
        StackJustify::Center => JustifyContent::Center,
        StackJustify::End => JustifyContent::End,
        StackJustify::SpaceBetween => JustifyContent::SpaceBetween,
        StackJustify::SpaceAround => JustifyContent::SpaceAround,
        StackJustify::SpaceEvenly => JustifyContent::SpaceEvenly,
    };
    let spacing = if spacing.is_finite() {
        spacing.max(0.0)
    } else {
        0.0
    };
    let style = StyleBuilder::new()
        .display_flex()
        .direction(direction)
        .align_items(alignment)
        .justify_content(justify)
        .flex_wrap(props.wrap)
        .gap_px(spacing, spacing)
        .padding_l_px(props.padding.left as f32)
        .padding_r_px(props.padding.right as f32)
        .padding_t_px(props.padding.top as f32)
        .padding_b_px(props.padding.bottom as f32);
    crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .styles(style)
        .children(props.children.clone())
        .build()
}
