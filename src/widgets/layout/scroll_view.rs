use crate::component::{Component, Element, ElementType, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;

#[derive(Clone, PartialEq)]
pub struct ScrollViewProps {
    pub content: Element,
    pub scroll_x: bool,
    pub scroll_y: bool,
    pub viewport_width: usize,
    pub viewport_height: usize,
    pub show_scrollbars: bool,
    pub smooth_scroll: bool,
    pub scroll_speed: usize,
}

impl Default for ScrollViewProps {
    fn default() -> Self {
        Self {
            content: Element::text(""),
            scroll_x: true,
            scroll_y: true,
            viewport_width: 80,
            viewport_height: 24,
            show_scrollbars: true,
            smooth_scroll: false,
            scroll_speed: 3,
        }
    }
}

impl Props for ScrollViewProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScrollViewState {
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub content_width: usize,
    pub content_height: usize,
    pub is_focused: bool,
}

pub struct ScrollView {
    state: ScrollViewState,
}

impl Component for ScrollView {
    type Props = ScrollViewProps;
    type State = ScrollViewState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: ScrollViewState::default(),
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let content = match &props.content.element_type {
            ElementType::Text(text) => text.clone(),
            _ => props
                .content
                .children
                .iter()
                .map(|c| match &c.element_type {
                    ElementType::Text(text) => text.clone(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };

        let content_lines: Vec<&str> = content.lines().collect();
        let content_height = content_lines.len();
        let content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        let visible_height = if props.show_scrollbars && props.scroll_x {
            props.viewport_height.saturating_sub(1)
        } else {
            props.viewport_height
        };

        let visible_width = if props.show_scrollbars && props.scroll_y {
            props.viewport_width.saturating_sub(1)
        } else {
            props.viewport_width
        };

        let start_y = state.scroll_y;
        let end_y = (start_y + visible_height).min(content_height);
        let start_x = state.scroll_x;
        let _end_x = start_x + visible_width;

        let mut visible_lines = Vec::new();
        for i in start_y..end_y {
            if let Some(line) = content_lines.get(i) {
                let visible_line = if line.len() > start_x {
                    let line_end = (start_x + visible_width).min(line.len());
                    &line[start_x..line_end]
                } else {
                    ""
                };
                visible_lines.push(visible_line.to_string());
            }
        }

        while visible_lines.len() < visible_height {
            visible_lines.push(String::new());
        }

        if props.show_scrollbars {
            if props.scroll_y && content_height > visible_height {
                let scrollbar_height = visible_height;
                let thumb_size = ((visible_height as f64 / content_height as f64)
                    * scrollbar_height as f64)
                    .ceil() as usize;
                let thumb_pos = ((state.scroll_y as f64 / content_height as f64)
                    * scrollbar_height as f64) as usize;

                for (i, line) in visible_lines.iter_mut().enumerate() {
                    let scrollbar_char = if i >= thumb_pos && i < thumb_pos + thumb_size {
                        '█'
                    } else {
                        '░'
                    };
                    line.push(scrollbar_char);
                }
            }

            if props.scroll_x && content_width > visible_width {
                let scrollbar_width = visible_width;
                let thumb_size = ((visible_width as f64 / content_width as f64)
                    * scrollbar_width as f64)
                    .ceil() as usize;
                let thumb_pos = ((state.scroll_x as f64 / content_width as f64)
                    * scrollbar_width as f64) as usize;

                let mut scrollbar_line = String::new();
                for i in 0..scrollbar_width {
                    let scrollbar_char = if i >= thumb_pos && i < thumb_pos + thumb_size {
                        '█'
                    } else {
                        '░'
                    };
                    scrollbar_line.push(scrollbar_char);
                }

                if props.scroll_y && content_height > visible_height {
                    scrollbar_line.push('█');
                }

                visible_lines.push(scrollbar_line);
            }
        }

        Element::text(visible_lines.join("\n"))
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event, props, state),
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            Event::Focus(_) => {
                state.is_focused = true;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

impl ScrollView {
    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &ScrollViewProps,
        state: &mut ScrollViewState,
    ) -> EventResult {
        if !state.is_focused {
            return EventResult::Ignored;
        }

        let content = match &props.content.element_type {
            ElementType::Text(text) => text.clone(),
            _ => props
                .content
                .children
                .iter()
                .map(|c| match &c.element_type {
                    ElementType::Text(text) => text.clone(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };

        let content_lines: Vec<&str> = content.lines().collect();
        let content_height = content_lines.len();
        let content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        match event.code {
            KeyCode::Up => {
                if props.scroll_y {
                    state.scroll_y = state.scroll_y.saturating_sub(1);
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if props.scroll_y {
                    let max_scroll = content_height.saturating_sub(props.viewport_height);
                    state.scroll_y = (state.scroll_y + 1).min(max_scroll);
                }
                EventResult::Consumed
            }
            KeyCode::Left => {
                if props.scroll_x {
                    state.scroll_x = state.scroll_x.saturating_sub(1);
                }
                EventResult::Consumed
            }
            KeyCode::Right => {
                if props.scroll_x {
                    let max_scroll = content_width.saturating_sub(props.viewport_width);
                    state.scroll_x = (state.scroll_x + 1).min(max_scroll);
                }
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                if props.scroll_y {
                    state.scroll_y = state.scroll_y.saturating_sub(props.viewport_height / 2);
                }
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                if props.scroll_y {
                    let max_scroll = content_height.saturating_sub(props.viewport_height);
                    state.scroll_y = (state.scroll_y + props.viewport_height / 2).min(max_scroll);
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                state.scroll_y = 0;
                state.scroll_x = 0;
                EventResult::Consumed
            }
            KeyCode::End => {
                if props.scroll_y {
                    state.scroll_y = content_height.saturating_sub(props.viewport_height);
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &ScrollViewProps,
        state: &mut ScrollViewState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Wheel => {
                // Mouse wheel always scrolls down by default
                // Use Shift+Wheel to scroll up (common pattern in terminals)
                if props.scroll_y {
                    let content = match &props.content.element_type {
                        ElementType::Text(text) => text.clone(),
                        _ => props
                            .content
                            .children
                            .iter()
                            .map(|c| match &c.element_type {
                                ElementType::Text(text) => text.clone(),
                                _ => String::new(),
                            })
                            .collect::<Vec<_>>()
                            .join("\n"),
                    };
                    let content_height = content.lines().count();
                    let max_scroll = content_height.saturating_sub(props.viewport_height);

                    if event.modifiers.shift {
                        // Shift+Wheel scrolls up
                        state.scroll_y = state.scroll_y.saturating_sub(props.scroll_speed);
                    } else {
                        // Normal wheel scrolls down
                        state.scroll_y = (state.scroll_y + props.scroll_speed).min(max_scroll);
                    }
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_view_basic() {
        let scroll_view = ScrollView::new(ScrollViewProps::default());
        let props = ScrollViewProps {
            content: Element::text("Line 1\nLine 2\nLine 3\nLine 4\nLine 5"),
            viewport_height: 3,
            show_scrollbars: false,  // Disable scrollbars for simple test
            ..Default::default()
        };
        let state = ScrollViewState::default();

        let element = scroll_view.render(&props, &state);
        if let ElementType::Text(content) = &element.element_type {
            // With viewport_height=3, should show 3 lines
            assert!(content.contains("Line 1"), "Content: {:?}", content);
            assert!(content.contains("Line 2"), "Content: {:?}", content);
            assert!(content.contains("Line 3"), "Content: {:?}", content);
            assert!(!content.contains("Line 4"), "Content: {:?}", content);
        }
    }

    #[test]
    fn test_scroll_view_scrolling() {
        let scroll_view = ScrollView::new(ScrollViewProps::default());
        let props = ScrollViewProps {
            content: Element::text("Line 1\nLine 2\nLine 3\nLine 4\nLine 5"),
            viewport_height: 3,
            show_scrollbars: false,  // Disable scrollbars for simple test
            ..Default::default()
        };
        let state = ScrollViewState {
            scroll_y: 2,
            ..Default::default()
        };

        let element = scroll_view.render(&props, &state);
        if let ElementType::Text(content) = &element.element_type {
            assert!(content.contains("Line 3"));
            assert!(content.contains("Line 4"));
            assert!(content.contains("Line 5"));
            assert!(!content.contains("Line 1"));
        }
    }
}
