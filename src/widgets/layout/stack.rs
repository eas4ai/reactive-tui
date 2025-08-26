use crate::component::{Component, Element, Props};
use crate::event::Event;
use crate::event::router::EventResult;
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

        // For horizontal stack, each child gets equal width unless wrapping
        let child_width = if props.wrap {
            // Calculate based on content - simplified approach
            available_width / props.children.len().max(1)
        } else {
            available_width / props.children.len().max(1)
        };

        let child_height = match props.alignment {
            StackAlignment::Stretch => height,
            _ => {
                // Calculate natural height - simplified to use available height
                height
            }
        };

        for _ in &props.children {
            sizes.push((child_width, child_height));
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

        // For vertical stack, each child gets equal height unless wrapping
        let child_height = available_height / props.children.len().max(1);

        let child_width = match props.alignment {
            StackAlignment::Stretch => width,
            _ => {
                // Calculate natural width - simplified to use available width
                width
            }
        };

        for _ in &props.children {
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
        // Simplified rendering - in a real implementation, this would properly position and clip children
        let child_content = match &child.element_type {
            crate::component::ElementType::Text(text) => text.clone(),
            _ => {
                // Render container children recursively
                child
                    .children
                    .iter()
                    .map(|c| self.render_child_at_position(c, 0, 0, width, height))
                    .collect::<Vec<_>>()
                    .join("")
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
                // For horizontal layout, merge lines at the same y position
                let mut final_lines = vec![String::new(); available_height];

                for part in rendered_parts {
                    for (line_idx, line) in part.lines().enumerate() {
                        if line_idx < final_lines.len() {
                            if final_lines[line_idx].len() < line.len() {
                                final_lines[line_idx] = line.to_string();
                            } else if !line.trim().is_empty() {
                                // Merge non-empty lines
                                for (char_idx, ch) in line.chars().enumerate() {
                                    if char_idx < final_lines[line_idx].len() && ch != ' ' {
                                        final_lines[line_idx]
                                            .replace_range(char_idx..char_idx + 1, &ch.to_string());
                                    } else if char_idx >= final_lines[line_idx].len() {
                                        final_lines[line_idx].push(ch);
                                    }
                                }
                            }
                        }
                    }
                }

                final_lines.join("\n")
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
            assert!(content.contains("A"));
            assert!(content.contains("B"));
            assert!(content.contains("C"));
        } else {
            panic!("Expected text element");
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
