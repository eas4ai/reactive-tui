use crate::component::{Component, Element, ElementType, LayoutInfo, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{FocusEventKind, KeyCode, KeyEventKind, MouseEventKind, WheelDelta};
use crate::event::Event;
use crate::layout::style::{Direction, StyleBuilder};
use std::sync::{Arc, Mutex};

mod motion;
use std::any::Any;

/// Builder for creating ScrollView components with a fluent API
#[derive(Clone, Debug)]
pub struct ScrollViewBuilder {
    content: Element,
    scroll_x: bool,
    scroll_y: bool,
    viewport_width: usize,
    viewport_height: usize,
    show_scrollbars: bool,
    smooth_scroll: bool,
    scroll_speed: usize,
}

impl ScrollViewBuilder {
    /// Create a new ScrollViewBuilder with content
    pub fn new(content: Element) -> Self {
        Self {
            content,
            scroll_x: true,
            scroll_y: true,
            viewport_width: 80,
            viewport_height: 24,
            show_scrollbars: true,
            smooth_scroll: false,
            scroll_speed: 3,
        }
    }

    /// Set the content element
    pub fn content(mut self, content: Element) -> Self {
        self.content = content;
        self
    }

    /// Enable or disable horizontal scrolling
    pub fn scroll_x(mut self, enable: bool) -> Self {
        self.scroll_x = enable;
        self
    }

    /// Enable or disable vertical scrolling
    pub fn scroll_y(mut self, enable: bool) -> Self {
        self.scroll_y = enable;
        self
    }

    /// Set the viewport width
    pub fn viewport_width(mut self, width: usize) -> Self {
        self.viewport_width = width;
        self
    }

    /// Set the viewport height
    pub fn viewport_height(mut self, height: usize) -> Self {
        self.viewport_height = height;
        self
    }

    /// Set the viewport size
    pub fn viewport_size(mut self, width: usize, height: usize) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self
    }

    /// Show or hide scrollbars
    pub fn show_scrollbars(mut self, show: bool) -> Self {
        self.show_scrollbars = show;
        self
    }

    /// Enable or disable smooth scrolling
    pub fn smooth_scroll(mut self, smooth: bool) -> Self {
        self.smooth_scroll = smooth;
        self
    }

    /// Set the scroll speed multiplier
    pub fn scroll_speed(mut self, speed: usize) -> Self {
        self.scroll_speed = speed;
        self
    }

    /// Build the ScrollViewProps
    pub fn build(self) -> ScrollViewProps {
        ScrollViewProps {
            content: self.content,
            scroll_x: self.scroll_x,
            scroll_y: self.scroll_y,
            viewport_width: self.viewport_width,
            viewport_height: self.viewport_height,
            show_scrollbars: self.show_scrollbars,
            smooth_scroll: self.smooth_scroll,
            scroll_speed: self.scroll_speed,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::typed::<ScrollView>(self.build())
    }
}

impl Default for ScrollViewBuilder {
    fn default() -> Self {
        Self::new(Element::text(""))
    }
}

/// Props for the ScrollView component
#[derive(Clone, PartialEq)]
pub struct ScrollViewProps {
    /// Content element to scroll
    pub content: Element,
    /// Whether horizontal scrolling is enabled
    pub scroll_x: bool,
    /// Whether vertical scrolling is enabled
    pub scroll_y: bool,
    /// Width of the viewport
    pub viewport_width: usize,
    /// Height of the viewport
    pub viewport_height: usize,
    /// Whether to show scrollbars
    pub show_scrollbars: bool,
    /// Whether to use smooth scrolling
    pub smooth_scroll: bool,
    /// Scroll speed multiplier
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

/// State for the ScrollView component
#[derive(Clone, Debug, Default)]
pub struct ScrollViewState {
    /// Horizontal scroll position
    pub scroll_x: usize,
    /// Vertical scroll position
    pub scroll_y: usize,
    /// Width of the content
    pub content_width: usize,
    /// Height of the content
    pub content_height: usize,
    /// Whether the scroll view has focus
    pub is_focused: bool,
}

/// Scroll view component for scrollable content.
pub struct ScrollView {
    viewport: Option<LayoutInfo>,
    content_size: Arc<Mutex<(usize, usize)>>,
    motion: motion::ScrollMotion,
}

impl ScrollView {
    fn visible_size(&self, props: &ScrollViewProps) -> (usize, usize) {
        let (width, height) = self
            .viewport
            .map(|v| v.content_size())
            .unwrap_or((props.viewport_width as f32, props.viewport_height as f32));
        (
            (width as usize).saturating_sub(usize::from(props.show_scrollbars && props.scroll_y)),
            (height as usize).saturating_sub(usize::from(props.show_scrollbars && props.scroll_x)),
        )
    }

    fn limits(&self, props: &ScrollViewProps) -> (usize, usize) {
        let content = *self.content_size.lock().unwrap();
        let visible = self.visible_size(props);
        (
            if props.scroll_x {
                content.0.saturating_sub(visible.0)
            } else {
                0
            },
            if props.scroll_y {
                content.1.saturating_sub(visible.1)
            } else {
                0
            },
        )
    }

    fn clamp(&self, props: &ScrollViewProps, state: &mut ScrollViewState) -> bool {
        let old = (state.scroll_x, state.scroll_y);
        let limits = self.limits(props);
        state.scroll_x = state.scroll_x.min(limits.0);
        state.scroll_y = state.scroll_y.min(limits.1);
        (state.content_width, state.content_height) = *self.content_size.lock().unwrap();
        old != (state.scroll_x, state.scroll_y)
    }
}

impl Component for ScrollView {
    type Props = ScrollViewProps;
    type State = ScrollViewState;

    fn new(_props: Self::Props) -> Self {
        Self {
            viewport: None,
            content_size: Arc::new(Mutex::new((0, 0))),
            motion: motion::ScrollMotion::new(),
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        self.clamp(props, state);
        true
    }

    fn layout(
        &mut self,
        bounds: LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        let old = self.visible_size(props);
        self.viewport = Some(bounds);
        self.clamp(props, state) || old != self.visible_size(props)
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let (width, height) = self.visible_size(props);
        let limits = self.limits(props);
        let offset = self.motion.position(
            (state.scroll_x.min(limits.0), state.scroll_y.min(limits.1)),
            props.smooth_scroll,
        );
        let offset = (offset.0.min(limits.0), offset.1.min(limits.1));
        let mut content_style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column)
            .position_absolute()
            .z_index(0)
            .inset_left(-(offset.0 as f32))
            .inset_top(-(offset.1 as f32));
        content_style.unconstrained_width = Some(props.scroll_x);
        let content_style = if props.scroll_x {
            content_style
        } else {
            content_style.width_px(width as f32)
        };
        let content_style = if props.scroll_y {
            content_style
        } else {
            content_style.height_px(height as f32)
        };
        let mut content =
            crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
                .styles(content_style)
                .class("whitespace-pre")
                .child(props.content.clone())
                .build();
        let measured = self.content_size.clone();
        content.metadata.layout.push(Arc::new(move |layout| {
            let mut size = measured.lock().unwrap();
            let next = (
                layout.content_extent.0.max(layout.size.0).ceil() as usize,
                layout.content_extent.1.max(layout.size.1).ceil() as usize,
            );
            let changed = *size != next;
            *size = next;
            changed
        }));
        let viewport = crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .size_px(Some(width as f32), Some(height as f32))
                    .flex_shrink(0.0)
                    .overflow_hidden(),
            )
            .child(content)
            .build();
        let mut children = vec![viewport];
        let measured = *self.content_size.lock().unwrap();
        let insets = self.viewport.map(|v| v.insets).unwrap_or([0.0; 4]);
        if props.show_scrollbars && props.scroll_y && limits.1 > 0 && height > 0 {
            let bar = scrollbar(height, measured.1, offset.1);
            children.push(
                crate::builder::ElementBuilder::new(ElementType::Text(
                    bar.chars()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                ))
                .styles(
                    StyleBuilder::new()
                        .position_absolute()
                        .inset_left(width as f32 + insets[0])
                        .inset_top(insets[1])
                        .size_px(Some(1.0), Some(height as f32)),
                )
                .class("whitespace-pre")
                .build(),
            );
        }
        if props.show_scrollbars && props.scroll_x && limits.0 > 0 && width > 0 {
            children.push(
                crate::builder::ElementBuilder::new(ElementType::Text(scrollbar(
                    width, measured.0, offset.0,
                )))
                .styles(
                    StyleBuilder::new()
                        .position_absolute()
                        .inset_left(insets[0])
                        .inset_top(height as f32 + insets[1])
                        .size_px(Some(width as f32), Some(1.0)),
                )
                .class("whitespace-pre")
                .build(),
            );
        }
        crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .display_flex()
                    .direction(Direction::Column)
                    .size_px(
                        Some(props.viewport_width as f32),
                        Some(props.viewport_height as f32),
                    )
                    .max_width_percent(100.0)
                    .max_height_percent(100.0)
                    .overflow_hidden(),
            )
            .children(children)
            .build()
            .with_accessibility(crate::accessibility::Node::new(
                crate::accessibility::Role::ScrollView,
            ))
            .with_focus(crate::component::FocusProps::input())
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Focus(focus) => {
                match focus.kind {
                    FocusEventKind::Gained => state.is_focused = true,
                    FocusEventKind::Lost => state.is_focused = false,
                    _ => return EventResult::Ignored,
                }
                return EventResult::Consumed;
            }
            Event::Key(key) if state.is_focused && key.kind != KeyEventKind::Release => {
                let (_, height) = self.visible_size(props);
                match key.code {
                    KeyCode::Up if props.scroll_y => {
                        state.scroll_y = state.scroll_y.saturating_sub(1)
                    }
                    KeyCode::Down if props.scroll_y => {
                        state.scroll_y = state.scroll_y.saturating_add(1)
                    }
                    KeyCode::Left if props.scroll_x => {
                        state.scroll_x = state.scroll_x.saturating_sub(1)
                    }
                    KeyCode::Right if props.scroll_x => {
                        state.scroll_x = state.scroll_x.saturating_add(1)
                    }
                    KeyCode::PageUp if props.scroll_y => {
                        state.scroll_y = state.scroll_y.saturating_sub(height.max(1))
                    }
                    KeyCode::PageDown if props.scroll_y => {
                        state.scroll_y = state.scroll_y.saturating_add(height.max(1))
                    }
                    KeyCode::Home => {
                        state.scroll_x = 0;
                        state.scroll_y = 0;
                    }
                    KeyCode::End => {
                        let limits = self.limits(props);
                        state.scroll_y = limits.1;
                        if !props.scroll_y {
                            state.scroll_x = limits.0;
                        }
                    }
                    _ => return EventResult::Ignored,
                }
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Wheel => {
                let (mut x, mut y) = match mouse.wheel.as_ref().map(|wheel| &wheel.delta) {
                    Some(WheelDelta::Lines { x, y }) => (*x as f64, *y as f64),
                    Some(WheelDelta::Pixels { x, y }) => (*x as f64, *y as f64),
                    None => return EventResult::Ignored,
                };
                if mouse.modifiers.shift && x == 0.0 {
                    x = y;
                    y = 0.0;
                }
                if props.scroll_x {
                    state.scroll_x = shifted(state.scroll_x, x, props.scroll_speed);
                }
                if props.scroll_y {
                    state.scroll_y = shifted(state.scroll_y, y, props.scroll_speed);
                }
            }
            _ => return EventResult::Ignored,
        }
        self.clamp(props, state);
        EventResult::Consumed
    }

    fn on_lifecycle(&mut self, event: crate::component::LifecycleEvent, _state: &mut Self::State) {
        if matches!(event, crate::component::LifecycleEvent::Unmount) {
            self.motion.cancel();
        }
    }
}

pub(super) fn shifted(value: usize, delta: f64, speed: usize) -> usize {
    if !delta.is_finite() {
        return value;
    }
    let amount = (delta.abs() * speed as f64).round() as usize;
    if delta < 0.0 {
        value.saturating_sub(amount)
    } else {
        value.saturating_add(amount)
    }
}

fn scrollbar(visible: usize, total: usize, offset: usize) -> String {
    let thumb = ((visible as f64 / total.max(1) as f64) * visible as f64).ceil() as usize;
    let thumb = thumb.clamp(1, visible.max(1));
    let travel = visible.saturating_sub(thumb);
    let start = ((offset as f64 / total.saturating_sub(visible).max(1) as f64) * travel as f64)
        .round() as usize;
    (0..visible)
        .map(|i| {
            if i >= start && i < start + thumb {
                '█'
            } else {
                '░'
            }
        })
        .collect()
}
