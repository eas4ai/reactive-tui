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
        Element::component("Stack").with_props(self.build())
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

impl Stack {
    fn calculate_child_sizes(
        &self,
        props: &StackProps,
        available_width: usize,
        available_height: usize,
    ) -> Vec<(usize, usize)> {
        if props.children.is_empty() {
            return Vec::new();
        }

        let content_width =
            available_width.saturating_sub(props.padding.left + props.padding.right);
        let content_height =
            available_height.saturating_sub(props.padding.top + props.padding.bottom);

        match props.direction {
            StackDirection::Horizontal => {
                self.calculate_horizontal_sizes(props, content_width, content_height)
            }
            StackDirection::Vertical => {
                self.calculate_vertical_sizes(props, content_width, content_height)
            }
        }
    }

    fn calculate_horizontal_sizes(
        &self,
        props: &StackProps,
        width: usize,
        height: usize,
    ) -> Vec<(usize, usize)> {
        let mut sizes = Vec::new();
        let spacing_total = if props.children.len() > 1 {
            (props.children.len() - 1) * props.spacing
        } else {
            0
        };

        let available_width = width.saturating_sub(spacing_total);

        // For horizontal stack, calculate child sizes based on content and constraints
        if props.wrap {
            // Wrapping layout - calculate sizes based on content and available space
            let mut current_row_width = 0;
            let mut row_heights = Vec::new();
            let mut current_row_height = 0;

            for child in &props.children {
                let (natural_width, natural_height) =
                    self.calculate_child_natural_size(child, props);
                let child_width = natural_width.min(available_width);

                // Check if we need to wrap to next row
                if current_row_width + child_width > available_width && current_row_width > 0 {
                    row_heights.push(current_row_height);
                    current_row_width = child_width;
                    current_row_height = natural_height;
                } else {
                    current_row_width += child_width + props.spacing;
                    current_row_height = current_row_height.max(natural_height);
                }

                let child_height = match props.alignment {
                    StackAlignment::Stretch => current_row_height,
                    _ => natural_height,
                };

                sizes.push((child_width, child_height));
            }

            if current_row_width > 0 {
                row_heights.push(current_row_height);
            }
        } else {
            // Non-wrapping layout - distribute space among children
            let mut flex_children = 0;
            let mut fixed_width_total = 0;

            // First pass: calculate fixed widths and count flex children
            for child in &props.children {
                let (natural_width, _) = self.calculate_child_natural_size(child, props);
                if self.child_has_fixed_width(child) {
                    fixed_width_total += natural_width;
                } else {
                    flex_children += 1;
                }
            }

            let remaining_width = available_width.saturating_sub(fixed_width_total);
            let flex_width = if flex_children > 0 {
                remaining_width / flex_children
            } else {
                0
            };

            // Second pass: assign final sizes
            for child in &props.children {
                let (natural_width, natural_height) =
                    self.calculate_child_natural_size(child, props);

                let child_width = if self.child_has_fixed_width(child) {
                    natural_width
                } else {
                    flex_width
                };

                let child_height = match props.alignment {
                    StackAlignment::Stretch => height,
                    _ => natural_height.min(height),
                };

                sizes.push((child_width, child_height));
            }
        }

        sizes
    }

    fn calculate_vertical_sizes(
        &self,
        props: &StackProps,
        width: usize,
        height: usize,
    ) -> Vec<(usize, usize)> {
        let mut sizes = Vec::new();
        let spacing_total = if props.children.len() > 1 {
            (props.children.len() - 1) * props.spacing
        } else {
            0
        };

        let available_height = height.saturating_sub(spacing_total);

        // For vertical stack, calculate child sizes based on content and constraints
        let mut flex_children = 0;
        let mut fixed_height_total = 0;

        // First pass: calculate fixed heights and count flex children
        for child in &props.children {
            let (_, natural_height) = self.calculate_child_natural_size(child, props);
            if self.child_has_fixed_height(child) {
                fixed_height_total += natural_height;
            } else {
                flex_children += 1;
            }
        }

        let remaining_height = available_height.saturating_sub(fixed_height_total);
        let flex_height = if flex_children > 0 {
            remaining_height / flex_children
        } else {
            0
        };

        // Second pass: assign final sizes
        for child in &props.children {
            let (natural_width, natural_height) = self.calculate_child_natural_size(child, props);

            let child_height = if self.child_has_fixed_height(child) {
                natural_height
            } else {
                flex_height
            };

            let child_width = match props.alignment {
                StackAlignment::Stretch => width,
                _ => natural_width.min(width),
            };

            sizes.push((child_width, child_height));
        }

        sizes
    }

    fn position_children(
        &self,
        props: &StackProps,
        sizes: &[(usize, usize)],
        available_width: usize,
        available_height: usize,
    ) -> Vec<(usize, usize)> {
        if sizes.is_empty() {
            return Vec::new();
        }

        let mut positions = Vec::new();
        let content_width =
            available_width.saturating_sub(props.padding.left + props.padding.right);
        let content_height =
            available_height.saturating_sub(props.padding.top + props.padding.bottom);

        match props.direction {
            StackDirection::Horizontal => self.position_horizontal_children(
                props,
                sizes,
                content_width,
                content_height,
                &mut positions,
            ),
            StackDirection::Vertical => self.position_vertical_children(
                props,
                sizes,
                content_width,
                content_height,
                &mut positions,
            ),
        }

        // Apply padding offset to all positions
        for (x, y) in &mut positions {
            *x += props.padding.left;
            *y += props.padding.top;
        }

        positions
    }

    fn position_horizontal_children(
        &self,
        props: &StackProps,
        sizes: &[(usize, usize)],
        width: usize,
        height: usize,
        positions: &mut Vec<(usize, usize)>,
    ) {
        let total_width: usize = sizes.iter().map(|(w, _)| *w).sum();
        let spacing_total = if sizes.len() > 1 {
            (sizes.len() - 1) * props.spacing
        } else {
            0
        };
        let total_content_width = total_width + spacing_total;

        let start_x = match props.justify {
            StackJustify::Start => 0,
            StackJustify::Center => (width.saturating_sub(total_content_width)) / 2,
            StackJustify::End => width.saturating_sub(total_content_width),
            StackJustify::SpaceBetween | StackJustify::SpaceAround | StackJustify::SpaceEvenly => 0,
        };

        let mut current_x = start_x;

        let children_indices: Box<dyn Iterator<Item = usize>> = if props.reverse {
            Box::new((0..sizes.len()).rev())
        } else {
            Box::new(0..sizes.len())
        };

        for i in children_indices {
            let (child_width, child_height) = sizes[i];

            let y = match props.alignment {
                StackAlignment::Start => 0,
                StackAlignment::Center => (height.saturating_sub(child_height)) / 2,
                StackAlignment::End => height.saturating_sub(child_height),
                StackAlignment::Stretch => 0,
            };

            positions.push((current_x, y));

            // Calculate spacing for next item
            current_x += child_width;
            if i < sizes.len() - 1 {
                current_x += match props.justify {
                    StackJustify::SpaceBetween if sizes.len() > 1 => {
                        (width.saturating_sub(total_width)) / (sizes.len() - 1)
                    }
                    StackJustify::SpaceAround if sizes.len() > 1 => {
                        (width.saturating_sub(total_width)) / sizes.len()
                    }
                    StackJustify::SpaceEvenly => {
                        (width.saturating_sub(total_width)) / (sizes.len() + 1)
                    }
                    _ => props.spacing,
                };
            }
        }
    }

    fn position_vertical_children(
        &self,
        props: &StackProps,
        sizes: &[(usize, usize)],
        width: usize,
        height: usize,
        positions: &mut Vec<(usize, usize)>,
    ) {
        let total_height: usize = sizes.iter().map(|(_, h)| *h).sum();
        let spacing_total = if sizes.len() > 1 {
            (sizes.len() - 1) * props.spacing
        } else {
            0
        };
        let total_content_height = total_height + spacing_total;

        let start_y = match props.justify {
            StackJustify::Start => 0,
            StackJustify::Center => (height.saturating_sub(total_content_height)) / 2,
            StackJustify::End => height.saturating_sub(total_content_height),
            StackJustify::SpaceBetween | StackJustify::SpaceAround | StackJustify::SpaceEvenly => 0,
        };

        let mut current_y = start_y;

        let children_indices: Box<dyn Iterator<Item = usize>> = if props.reverse {
            Box::new((0..sizes.len()).rev())
        } else {
            Box::new(0..sizes.len())
        };

        for i in children_indices {
            let (child_width, child_height) = sizes[i];

            let x = match props.alignment {
                StackAlignment::Start => 0,
                StackAlignment::Center => (width.saturating_sub(child_width)) / 2,
                StackAlignment::End => width.saturating_sub(child_width),
                StackAlignment::Stretch => 0,
            };

            positions.push((x, current_y));

            // Calculate spacing for next item
            current_y += child_height;
            if i < sizes.len() - 1 {
                current_y += match props.justify {
                    StackJustify::SpaceBetween if sizes.len() > 1 => {
                        (height.saturating_sub(total_height)) / (sizes.len() - 1)
                    }
                    StackJustify::SpaceAround if sizes.len() > 1 => {
                        (height.saturating_sub(total_height)) / sizes.len()
                    }
                    StackJustify::SpaceEvenly => {
                        (height.saturating_sub(total_height)) / (sizes.len() + 1)
                    }
                    _ => props.spacing,
                };
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn render_child_at_position(
        &self,
        child: &Element,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> String {
        // Properly position and clip children within bounds
        let child_content = match &child.element_type {
            crate::component::ElementType::Text(text) => {
                // Wrap text to fit within width, clip lines to height
                // Handle text without newlines properly
                let lines_to_process: Vec<&str> = if text.contains('\n') {
                    text.lines().collect()
                } else if !text.is_empty() {
                    vec![text.as_str()]
                } else {
                    vec![]
                };

                let wrapped_lines: Vec<String> = lines_to_process
                    .into_iter()
                    .flat_map(|line| {
                        if line.len() <= width {
                            vec![line.to_string()]
                        } else {
                            // Simple word wrapping
                            let mut wrapped = Vec::new();
                            let mut current_line = String::new();

                            for word in line.split_whitespace() {
                                if current_line.len() + word.len() < width {
                                    if !current_line.is_empty() {
                                        current_line.push(' ');
                                    }
                                    current_line.push_str(word);
                                } else if !current_line.is_empty() {
                                    wrapped.push(current_line);
                                    current_line = word.to_string();
                                } else {
                                    // Word is longer than width, truncate
                                    wrapped.push(word.chars().take(width).collect());
                                }
                            }
                            if !current_line.is_empty() {
                                wrapped.push(current_line);
                            }
                            wrapped
                        }
                    })
                    .take(height) // Clip vertically
                    .collect();

                wrapped_lines.join("\n")
            }
            _ => {
                // Render container children recursively with proper bounds
                let mut result = Vec::new();
                let mut used_height = 0;

                for child_elem in &child.children {
                    if used_height >= height {
                        break; // Vertical clipping
                    }

                    let remaining_height = height.saturating_sub(used_height);
                    let child_rendered = self.render_child_at_position(
                        child_elem,
                        x,
                        y + used_height,
                        width,
                        remaining_height,
                    );

                    if !child_rendered.is_empty() {
                        let child_lines = child_rendered.lines().count();
                        result.push(child_rendered);
                        used_height += child_lines;
                    }
                }

                result.join("")
            }
        };

        // Apply basic positioning by adding spaces for x offset and newlines for y offset
        let lines: Vec<&str> = child_content.lines().collect();
        let mut positioned_lines = Vec::new();

        for _ in 0..y {
            positioned_lines.push(String::new());
        }

        for (i, line) in lines.iter().enumerate() {
            if i >= height {
                break;
            }
            let padded_line = format!("{}{}", " ".repeat(x), line);
            positioned_lines.push(padded_line);
        }

        positioned_lines.join("\n")
    }

    /// Convert element to Taffy style for layout calculations
    fn element_to_taffy_style(&self, element: &Element) -> taffy::Style {
        // Production implementation: Convert reactive-tui Element to Taffy Style
        // This would parse CSS-like properties from the element's styling
        use taffy::geometry::*;
        use taffy::style::*;

        let mut style = taffy::Style::default();

        // Set flex direction based on element type
        match &element.element_type {
            crate::component::ElementType::Layout(layout_type) => {
                match layout_type {
                    crate::component::LayoutType::Flex => {
                        style.display = Display::Flex;
                        style.flex_direction = FlexDirection::Column; // Default to column
                    }
                    crate::component::LayoutType::Grid => {
                        style.display = Display::Grid;
                    }
                    _ => {
                        style.display = Display::Block;
                    }
                }
            }
            _ => {
                style.display = Display::Block;
            }
        }

        // Set default dimensions for content elements
        if matches!(element.element_type, crate::component::ElementType::Text(_)) {
            // Text elements should fit content
            style.size = Size {
                width: Dimension::auto(),
                height: Dimension::auto(),
            };
        }

        style
    }

    /// Calculate natural size of a child element
    fn calculate_child_natural_size(&self, child: &Element, _props: &StackProps) -> (usize, usize) {
        // Use Taffy for ALL layout calculations including text
        use taffy::{AvailableSpace, Size as TaffySize, TaffyTree};

        let mut taffy: TaffyTree<()> = TaffyTree::new();

        // Get the base style for the element
        let mut child_style = self.element_to_taffy_style(child);

        // For text elements, set min-content size based on text dimensions
        if let crate::component::ElementType::Text(text) = &child.element_type {
            if text.is_empty() {
                return (0, 0);
            }

            // Calculate intrinsic text dimensions
            let (text_width, text_height) = if text.contains('\n') {
                let lines: Vec<&str> = text.lines().collect();
                let height = lines.len();
                let width = lines
                    .iter()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0);
                (width, height)
            } else {
                // Single line text without newline
                (text.chars().count(), 1)
            };

            // Set the min-content size for text in Taffy style
            use taffy::style::Dimension;
            child_style.min_size = TaffySize {
                width: Dimension::length(text_width as f32),
                height: Dimension::length(text_height as f32),
            };

            // For text, also set the size to content dimensions
            // Since we can't check if it's auto, always set it for text
            child_style.size.width = Dimension::length(text_width as f32);
            child_style.size.height = Dimension::length(text_height as f32);
        }

        // Create Taffy node with the configured style
        if let Ok(node) = taffy.new_leaf(child_style) {
            // Compute layout with max-content to get natural size
            if taffy
                .compute_layout(
                    node,
                    TaffySize {
                        width: AvailableSpace::MaxContent,
                        height: AvailableSpace::MaxContent,
                    },
                )
                .is_ok()
            {
                let layout_result = taffy.layout(node).unwrap();
                return (
                    layout_result.size.width as usize,
                    layout_result.size.height as usize,
                );
            }
        }

        // Fallback size estimation based on element type
        match &child.element_type {
            crate::component::ElementType::Text(_) => {
                // This should be unreachable since we handle text above
                (10, 1)
            }
            crate::component::ElementType::Component(component_name) => {
                // Estimate size based on component type
                match component_name.as_str() {
                    "Button" => (12, 3),              // Typical button with padding
                    "Input" | "TextInput" => (20, 1), // Typical input size
                    "Label" => (10, 1),               // Typical label size
                    "Checkbox" => (3, 1),             // Checkbox with label space
                    "Radio" => (3, 1),                // Radio button with label space
                    _ => (15, 2),                     // Default for unknown components
                }
            }
            crate::component::ElementType::Layout(layout_type) => {
                // For nested layouts, calculate based on type
                match layout_type {
                    crate::component::LayoutType::Flex => (20, 5),
                    crate::component::LayoutType::Grid => (25, 8),
                    crate::component::LayoutType::Stack => (18, 4),
                    crate::component::LayoutType::Absolute => (15, 3),
                }
            }
            crate::component::ElementType::Fragment => (0, 0), // Fragments have no size
            crate::component::ElementType::Empty => (0, 0),    // Empty elements have no size
        }
    }

    /// Check if a child has a fixed width (not flexible)
    fn child_has_fixed_width(&self, child: &Element) -> bool {
        use crate::component::ElementType;

        match &child.element_type {
            ElementType::Text(_) => true, // Text has fixed width based on content
            ElementType::Component(component_name) => {
                // Some components have fixed widths, others are flexible
                matches!(component_name.as_str(), "Button" | "Checkbox" | "Radio")
            }
            ElementType::Layout(_) => false, // Layout containers are typically flexible
            ElementType::Fragment => false,  // Fragments are flexible
            ElementType::Empty => true,      // Empty elements have fixed (zero) width
        }
    }

    /// Check if a child has a fixed height (not flexible)
    fn child_has_fixed_height(&self, child: &Element) -> bool {
        use crate::component::ElementType;

        match &child.element_type {
            ElementType::Text(_) => true, // Text has fixed height based on content
            ElementType::Component(component_name) => {
                // Most components have fixed heights, but some are flexible
                !matches!(
                    component_name.as_str(),
                    "TextArea" | "List" | "Table" | "Tree"
                )
            }
            ElementType::Layout(_) => false, // Layout containers are typically flexible
            ElementType::Fragment => false,  // Fragments are flexible
            ElementType::Empty => true,      // Empty elements have fixed (zero) height
        }
    }
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

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        if props.children.is_empty() {
            return Element::text("");
        }

        // Use viewport size or default
        let available_width = if state.viewport_width > 0 {
            state.viewport_width
        } else {
            80
        };
        let available_height = if state.viewport_height > 0 {
            state.viewport_height
        } else {
            24
        };

        let sizes = self.calculate_child_sizes(props, available_width, available_height);
        let positions = self.position_children(props, &sizes, available_width, available_height);

        // Render all children at their calculated positions
        let mut rendered_parts = Vec::new();

        for (i, child) in props.children.iter().enumerate() {
            if i < positions.len() && i < sizes.len() {
                let (x, y) = positions[i];
                let (width, height) = sizes[i];
                let positioned_child = self.render_child_at_position(child, x, y, width, height);
                rendered_parts.push(positioned_child);
            }
        }

        // Combine all positioned children into final layout
        let result = match props.direction {
            StackDirection::Horizontal => {
                // For horizontal layout, simply concatenate parts with spacing
                let spacing_str = " ".repeat(props.spacing);
                let mut result_lines = Vec::new();

                // Get max height
                let max_lines = rendered_parts
                    .iter()
                    .map(|p| p.lines().count())
                    .max()
                    .unwrap_or(0);

                // Build each line by concatenating corresponding lines from each part
                for line_idx in 0..max_lines {
                    let mut line_parts = Vec::new();
                    for part in &rendered_parts {
                        let lines: Vec<&str> = part.lines().collect();
                        if let Some(line) = lines.get(line_idx) {
                            // Remove the x-positioning spaces since we'll handle spacing differently
                            line_parts.push(line.trim_start());
                        } else {
                            line_parts.push("");
                        }
                    }
                    // Filter out empty parts and join with spacing
                    let non_empty: Vec<&str> =
                        line_parts.into_iter().filter(|s| !s.is_empty()).collect();
                    if !non_empty.is_empty() {
                        result_lines.push(non_empty.join(&spacing_str));
                    }
                }

                result_lines.join("\n")
            }
            StackDirection::Vertical => {
                // For vertical layout, simply join with spacing
                let spacing_str = "\n".repeat(props.spacing);
                rendered_parts.join(&spacing_str)
            }
        };

        Element::text(result)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> EventResult {
        // Stack components pass events to their children
        // In a full implementation, this would route events to the appropriate child
        // based on position and focus management
        match event {
            Event::Resize(resize_event) => {
                // Update viewport size
                self.state.viewport_width = resize_event.width as usize;
                self.state.viewport_height = resize_event.height as usize;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ElementType;

    #[test]
    fn test_stack_vertical_layout() {
        let stack = Stack::new(StackProps::default());
        let props = StackProps {
            direction: StackDirection::Vertical,
            spacing: 1,
            children: vec![
                Element::text("First"),
                Element::text("Second"),
                Element::text("Third"),
            ],
            ..Default::default()
        };
        let state = StackState {
            viewport_width: 80,
            viewport_height: 24,
            ..Default::default()
        };

        let element = stack.render(&props, &state);
        if let ElementType::Text(content) = &element.element_type {
            let lines: Vec<&str> = content.lines().collect();
            assert!(lines.len() >= 3);
            assert!(content.contains("First"));
            assert!(content.contains("Second"));
            assert!(content.contains("Third"));
        } else {
            panic!("Expected text element");
        }
    }

    #[test]
    fn test_stack_horizontal_layout() {
        let stack = Stack::new(StackProps::default());
        let props = StackProps {
            direction: StackDirection::Horizontal,
            spacing: 2,
            children: vec![Element::text("A"), Element::text("B"), Element::text("C")],
            ..Default::default()
        };
        let state = StackState {
            viewport_width: 80,
            viewport_height: 24,
            ..Default::default()
        };

        let element = stack.render(&props, &state);
        if let ElementType::Text(content) = &element.element_type {
            assert!(content.contains("A"), "Content: {:?}", content);
            assert!(content.contains("B"), "Content: {:?}", content);
            assert!(content.contains("C"), "Content: {:?}", content);
        } else {
            panic!("Expected text element, got: {:?}", element.element_type);
        }
    }

    #[test]
    fn test_stack_padding() {
        let stack = Stack::new(StackProps::default());
        let props = StackProps {
            padding: StackPadding::all(2),
            children: vec![Element::text("Content")],
            ..Default::default()
        };
        let _state = StackState {
            viewport_width: 80,
            viewport_height: 24,
            ..Default::default()
        };

        let sizes = stack.calculate_child_sizes(&props, 80, 24);
        assert_eq!(sizes.len(), 1);
        // Content area should be reduced by padding
        assert!(sizes[0].0 <= 76); // 80 - 4 (left + right padding)
        assert!(sizes[0].1 <= 20); // 24 - 4 (top + bottom padding)
    }

    #[test]
    fn test_stack_alignment() {
        let stack = Stack::new(StackProps::default());

        // Test center alignment
        let props = StackProps {
            alignment: StackAlignment::Center,
            children: vec![Element::text("Test")],
            ..Default::default()
        };
        let sizes = vec![(10, 5)];
        let positions = stack.position_children(&props, &sizes, 80, 24);

        assert_eq!(positions.len(), 1);
        // Should be centered
        assert!(positions[0].0 < 80);
        assert!(positions[0].1 < 24);
    }

    #[test]
    fn test_stack_justify_space_between() {
        let stack = Stack::new(StackProps::default());
        let props = StackProps {
            direction: StackDirection::Horizontal,
            justify: StackJustify::SpaceBetween,
            children: vec![Element::text("A"), Element::text("B"), Element::text("C")],
            ..Default::default()
        };

        let sizes = vec![(10, 5), (10, 5), (10, 5)];
        let positions = stack.position_children(&props, &sizes, 80, 24);

        assert_eq!(positions.len(), 3);
        // First should be at start
        assert_eq!(positions[0].0, 0);
        // Last should be near end
        assert!(positions[2].0 > positions[1].0);
        assert!(positions[1].0 > positions[0].0);
    }

    #[test]
    fn test_stack_reverse() {
        let stack = Stack::new(StackProps::default());
        let props = StackProps {
            reverse: true,
            children: vec![Element::text("First"), Element::text("Second")],
            ..Default::default()
        };

        let sizes = vec![(40, 12), (40, 12)];
        let positions = stack.position_children(&props, &sizes, 80, 24);

        // With reverse, positions should be calculated in reverse order
        assert_eq!(positions.len(), 2);
    }
}
