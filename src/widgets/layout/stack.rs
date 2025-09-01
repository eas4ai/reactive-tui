use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::Event;
use std::any::Any;

/// Properties for Stack layout component
#[derive(Clone, Debug, PartialEq)]
pub struct StackProps {
    pub direction: StackDirection,
    pub spacing: usize,
    pub alignment: StackAlignment,
    pub wrap: bool,            // Allow wrapping to next line/column
    pub justify: StackJustify, // How to distribute space
    pub padding: StackPadding,
    pub reverse: bool, // Reverse the order of children
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

#[derive(Clone, Debug, PartialEq)]
pub enum StackDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StackAlignment {
    Start,   // Top for horizontal, Left for vertical
    Center,  // Center alignment
    End,     // Bottom for horizontal, Right for vertical
    Stretch, // Fill available space
}

#[derive(Clone, Debug, PartialEq)]
pub enum StackJustify {
    Start,        // Pack to start
    Center,       // Center with equal space on sides
    End,          // Pack to end
    SpaceBetween, // Equal space between items, no space on ends
    SpaceAround,  // Equal space around items
    SpaceEvenly,  // Equal space between and around items
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct StackPadding {
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
}

impl StackPadding {
    pub fn all(padding: usize) -> Self {
        Self {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }

    pub fn symmetric(vertical: usize, horizontal: usize) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn horizontal(padding: usize) -> Self {
        Self {
            left: padding,
            right: padding,
            ..Default::default()
        }
    }

    pub fn vertical(padding: usize) -> Self {
        Self {
            top: padding,
            bottom: padding,
            ..Default::default()
        }
    }
}

/// State for Stack component
#[derive(Clone, Debug, Default)]
pub struct StackState {
    pub viewport_width: usize,
    pub viewport_height: usize,
    pub scroll_x: usize,
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
                let (natural_width, natural_height) = self.calculate_child_natural_size(child, props);
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
                let (natural_width, natural_height) = self.calculate_child_natural_size(child, props);

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
                fixed_height_total += natural_height as usize;
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
                natural_height as usize
            } else {
                flex_height
            };

            let child_width = match props.alignment {
                StackAlignment::Stretch => width,
                _ => (natural_width as usize).min(width),
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
                let wrapped_lines: Vec<String> = text
                    .lines()
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

    /// Calculate natural size of a child element
    fn calculate_child_natural_size(
        &self,
        child: &Element,
        _props: &StackProps,
    ) -> (usize, usize) {
        // Production implementation for child size calculation
        // In a real implementation, this would query the child's layout preferences

        // Calculate size based on element type
        match &child.element_type {
            crate::component::ElementType::Text(text) => {
                // Calculate text dimensions
                let lines: Vec<&str> = text.lines().collect();
                let height = lines.len();
                let width = lines.iter()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0);
                (width, height)
            }
            crate::component::ElementType::Component(component_name) => {
                // Estimate size based on component type
                match component_name.as_str() {
                    "Button" => (12, 3), // Typical button with padding
                    "Input" | "TextInput" => (20, 1), // Typical input size
                    "Label" => (10, 1), // Typical label size
                    "Checkbox" => (3, 1), // Checkbox with label space
                    "Radio" => (3, 1), // Radio button with label space
                    _ => (15, 2), // Default for unknown components
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
            crate::component::ElementType::Empty => (0, 0), // Empty elements have no size
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
            ElementType::Fragment => false, // Fragments are flexible
            ElementType::Empty => true, // Empty elements have fixed (zero) width
        }
    }

    /// Check if a child has a fixed height (not flexible)
    fn child_has_fixed_height(&self, child: &Element) -> bool {
        use crate::component::ElementType;

        match &child.element_type {
            ElementType::Text(_) => true, // Text has fixed height based on content
            ElementType::Component(component_name) => {
                // Most components have fixed heights, but some are flexible
                !matches!(component_name.as_str(), "TextArea" | "List" | "Table" | "Tree")
            }
            ElementType::Layout(_) => false, // Layout containers are typically flexible
            ElementType::Fragment => false, // Fragments are flexible
            ElementType::Empty => true, // Empty elements have fixed (zero) height
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
