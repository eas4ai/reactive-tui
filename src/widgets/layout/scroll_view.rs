use super::look;
use crate::component::{Component, Element, ElementType, LayoutInfo, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{
    FocusEventKind, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind, Position,
    WheelDelta,
};
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
    aria_label: Option<String>,
}

impl ScrollViewBuilder {
    /// Create a new ScrollViewBuilder with content
    pub fn new(content: Element) -> Self {
        Self {
            content,
            scroll_x: true,
            scroll_y: true,
            viewport_width: 0,
            viewport_height: 0,
            show_scrollbars: true,
            smooth_scroll: false,
            scroll_speed: 3,
            aria_label: None,
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

    /// Set the viewport width; zero fills the parent.
    pub fn viewport_width(mut self, width: usize) -> Self {
        self.viewport_width = width;
        self
    }

    /// Set the viewport height; zero fills the parent.
    pub fn viewport_height(mut self, height: usize) -> Self {
        self.viewport_height = height;
        self
    }

    /// Set the viewport size; zero fills that parent axis.
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

    /// Set the accessible name of the scroll view.
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
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
            aria_label: self.aria_label,
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
    /// Width of the viewport; zero fills the parent.
    pub viewport_width: usize,
    /// Height of the viewport; zero fills the parent.
    pub viewport_height: usize,
    /// Whether to show scrollbars
    pub show_scrollbars: bool,
    /// Whether to use smooth scrolling
    pub smooth_scroll: bool,
    /// Scroll speed multiplier
    pub scroll_speed: usize,
    /// Accessible name; absent when the application supplies none.
    pub aria_label: Option<String>,
}

impl Default for ScrollViewProps {
    fn default() -> Self {
        Self {
            content: Element::text(""),
            scroll_x: true,
            scroll_y: true,
            viewport_width: 0,
            viewport_height: 0,
            show_scrollbars: true,
            smooth_scroll: false,
            scroll_speed: 3,
            aria_label: None,
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
    drag: Option<ThumbDrag>,
}

struct ThumbDrag {
    horizontal: bool,
    pointer: f32,
    offset: usize,
    travel: usize,
    limit: usize,
}

impl ScrollView {
    fn full_size(&self, props: &ScrollViewProps) -> (usize, usize) {
        let (width, height) = self
            .viewport
            .map(|v| v.content_size())
            .unwrap_or((props.viewport_width as f32, props.viewport_height as f32));
        (width as usize, height as usize)
    }

    fn bars(&self, props: &ScrollViewProps) -> (bool, bool) {
        let full = self.full_size(props);
        let content = *self.content_size.lock().unwrap();
        // Decide against the full box, before either bar takes layout space.
        (
            props.show_scrollbars && props.scroll_x && content.0 > full.0,
            props.show_scrollbars && props.scroll_y && content.1 > full.1,
        )
    }

    fn visible_size(&self, props: &ScrollViewProps) -> (usize, usize) {
        let (width, height) = self.full_size(props);
        let (horizontal, vertical) = self.bars(props);
        (
            width.saturating_sub(usize::from(vertical)),
            height.saturating_sub(usize::from(horizontal)),
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

    fn handle_pointer(
        &mut self,
        mouse: &MouseEvent,
        props: &ScrollViewProps,
        state: &mut ScrollViewState,
    ) -> EventResult {
        if mouse.kind == MouseEventKind::Up && self.drag.is_some() {
            self.motion
                .position((state.scroll_x, state.scroll_y), false);
            self.drag = None;
            return EventResult::Consumed;
        }
        if mouse.button != MouseButton::Left {
            return EventResult::Ignored;
        }
        let Position::Cell { x, y } = mouse.position else {
            return EventResult::Ignored;
        };
        if mouse.kind == MouseEventKind::Drag {
            let Some(drag) = &self.drag else {
                return EventResult::Ignored;
            };
            if drag.travel == 0 {
                return EventResult::Consumed;
            }
            let pointer = f32::from(if drag.horizontal { x } else { y });
            let offset = (drag.offset as f64
                + f64::from(pointer - drag.pointer) * drag.limit as f64 / drag.travel as f64)
                .round()
                .clamp(0.0, drag.limit as f64) as usize;
            if drag.horizontal {
                state.scroll_x = offset;
            } else {
                state.scroll_y = offset;
            }
            self.clamp(props, state);
            return EventResult::Consumed;
        }
        if mouse.kind != MouseEventKind::Down {
            return EventResult::Ignored;
        }
        self.drag = None;
        let Some(viewport) = self.viewport else {
            return EventResult::Ignored;
        };
        let (width, height) = self.visible_size(props);
        let (horizontal, vertical) = self.bars(props);
        let local_x = f32::from(x) - viewport.insets[0];
        let local_y = f32::from(y) - viewport.insets[1];
        let axis = if vertical
            && height > 0
            && local_x == width as f32
            && local_y >= 0.0
            && local_y < height as f32
        {
            false
        } else if horizontal
            && width > 0
            && local_y == height as f32
            && local_x >= 0.0
            && local_x < width as f32
        {
            true
        } else {
            return EventResult::Ignored;
        };
        let limits = self.limits(props);
        let painted = self.motion.position(
            (state.scroll_x.min(limits.0), state.scroll_y.min(limits.1)),
            props.smooth_scroll,
        );
        let content = *self.content_size.lock().unwrap();
        let (visible, total, offset, limit, cell, pointer) = if axis {
            (
                width,
                content.0,
                painted.0,
                limits.0,
                local_x as usize,
                f32::from(x),
            )
        } else {
            (
                height,
                content.1,
                painted.1,
                limits.1,
                local_y as usize,
                f32::from(y),
            )
        };
        let (start, thumb) = thumb_geometry(visible, total, offset);
        if cell >= start && cell < start + thumb {
            state.scroll_x = painted.0;
            state.scroll_y = painted.1;
            self.motion.position(painted, false);
            self.drag = Some(ThumbDrag {
                horizontal: axis,
                pointer,
                offset,
                travel: visible.saturating_sub(thumb),
                limit,
            });
        } else {
            let next = if cell < start {
                offset.saturating_sub(visible)
            } else {
                offset.saturating_add(visible)
            };
            if axis {
                state.scroll_x = next;
            } else {
                state.scroll_y = next;
            }
            self.clamp(props, state);
        }
        EventResult::Consumed
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
            drag: None,
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
            props.smooth_scroll && self.drag.is_none(),
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
        let (horizontal, vertical) = self.bars(props);
        if vertical && height > 0 {
            children.push(scrollbar(
                height,
                measured.1,
                offset.1,
                false,
                (insets[0] + width as f32, insets[1]),
            ));
        }
        if horizontal && width > 0 {
            children.push(scrollbar(
                width,
                measured.0,
                offset.0,
                true,
                (insets[0], insets[1] + height as f32),
            ));
        }
        let mut style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column)
            .max_width_percent(100.0)
            .max_height_percent(100.0)
            .overflow_hidden();
        style = if props.viewport_width == 0 {
            style.width_percent(100.0).min_width_px(0.0)
        } else {
            style.width_px(props.viewport_width as f32)
        };
        style = if props.viewport_height == 0 {
            style.height_percent(100.0).min_height_px(0.0)
        } else {
            style.height_px(props.viewport_height as f32)
        };
        let mut node = crate::accessibility::Node::new(crate::accessibility::Role::ScrollView);
        node.inner.set_scroll_x(offset.0 as f64);
        node.inner.set_scroll_x_min(0.0);
        node.inner.set_scroll_x_max(limits.0 as f64);
        node.inner.set_scroll_y(offset.1 as f64);
        node.inner.set_scroll_y_min(0.0);
        node.inner.set_scroll_y_max(limits.1 as f64);
        if let Some(label) = &props.aria_label {
            node.inner.set_label(label.clone());
        }
        crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(style)
            .children(children)
            .build()
            .with_accessibility(node)
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
                let before = (state.scroll_x, state.scroll_y);
                if props.scroll_x {
                    state.scroll_x = shifted(state.scroll_x, x, props.scroll_speed);
                }
                if props.scroll_y {
                    state.scroll_y = shifted(state.scroll_y, y, props.scroll_speed);
                }
                self.clamp(props, state);
                // At an edge the wheel passes to an enclosing view (INP-005).
                if (state.scroll_x, state.scroll_y) == before {
                    return EventResult::Ignored;
                }
                return EventResult::Consumed;
            }
            Event::Mouse(mouse) => return self.handle_pointer(mouse, props, state),
            _ => return EventResult::Ignored,
        }
        self.clamp(props, state);
        EventResult::Consumed
    }

    fn on_lifecycle(&mut self, event: crate::component::LifecycleEvent, _state: &mut Self::State) {
        if matches!(event, crate::component::LifecycleEvent::Unmount) {
            self.drag = None;
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

fn scrollbar(
    visible: usize,
    total: usize,
    offset: usize,
    horizontal: bool,
    origin: (f32, f32),
) -> Element {
    let (start, thumb) = thumb_geometry(visible, total, offset);
    let cells = (0..visible)
        .map(|cell| {
            let is_thumb = cell >= start && cell < start + thumb;
            crate::builder::ElementBuilder::new(ElementType::Text(
                if is_thumb { "█" } else { "░" }.into(),
            ))
            .styles(
                StyleBuilder::new()
                    .size_px(Some(1.0), Some(1.0))
                    .flex_shrink(0.0),
            )
            .class(if is_thumb { look::THUMB } else { look::TRACK })
            .build()
        })
        .collect();
    let (width, height, direction) = if horizontal {
        (visible as f32, 1.0, Direction::Row)
    } else {
        (1.0, visible as f32, Direction::Column)
    };
    crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .styles(
            StyleBuilder::new()
                .display_flex()
                .direction(direction)
                .position_absolute()
                .inset_left(origin.0)
                .inset_top(origin.1)
                .size_px(Some(width), Some(height)),
        )
        .children(cells)
        .build()
}

fn thumb_geometry(visible: usize, total: usize, offset: usize) -> (usize, usize) {
    let thumb = ((visible as f64 / total.max(1) as f64) * visible as f64).ceil() as usize;
    let thumb = thumb.clamp(1, visible.max(1));
    let travel = visible.saturating_sub(thumb);
    let start = ((offset as f64 / total.saturating_sub(visible).max(1) as f64) * travel as f64)
        .round() as usize;
    (start.min(travel), thumb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_004_scroll_view_offsets_and_ranges() {
        let props = ScrollViewBuilder::default().viewport_size(20, 10).build();
        let mut view = ScrollView::new(props.clone());
        *view.content_size.lock().unwrap() = (60, 40);
        let mut state = ScrollViewState {
            scroll_x: 7,
            scroll_y: 12,
            ..Default::default()
        };
        view.layout(
            LayoutInfo::from_bounds(crate::event::hit::Bounds::new(0.0, 0.0, 20.0, 10.0)),
            &mut props.clone(),
            &mut state,
        );
        let element = view.render(&props, &state);
        let node = &element.metadata.accessibility.as_ref().unwrap().inner;
        assert_eq!(node.scroll_x(), Some(7.0));
        assert_eq!(node.scroll_x_min(), Some(0.0));
        assert_eq!(node.scroll_x_max(), Some(41.0));
        assert_eq!(node.scroll_y(), Some(12.0));
        assert_eq!(node.scroll_y_min(), Some(0.0));
        assert_eq!(node.scroll_y_max(), Some(31.0));
    }

    #[test]
    fn nav_004_scroll_view_has_no_default_label() {
        let props = ScrollViewProps::default();
        let element = ScrollView::new(props.clone()).render(&props, &ScrollViewState::default());
        assert_eq!(
            element
                .metadata
                .accessibility
                .as_ref()
                .unwrap()
                .inner
                .label(),
            None
        );
    }

    #[test]
    fn nav_004_scroll_view_label_from_both_builders() {
        let props = ScrollViewBuilder::default().aria_label("Log").build();
        let element = ScrollView::new(props.clone()).render(&props, &ScrollViewState::default());
        assert_eq!(
            element
                .metadata
                .accessibility
                .as_ref()
                .unwrap()
                .inner
                .label(),
            Some("Log")
        );
        let element = crate::builder::scroll_view().aria_label("History").build();
        let props = element.props_as::<ScrollViewProps>().unwrap();
        assert_eq!(props.aria_label.as_deref(), Some("History"));
        let rendered = ScrollView::new(props.clone()).render(props, &ScrollViewState::default());
        assert_eq!(
            rendered
                .metadata
                .accessibility
                .as_ref()
                .unwrap()
                .inner
                .label(),
            Some("History")
        );
    }

    #[test]
    fn scroll_view_thumb_without_travel_does_not_move() {
        let mut props = ScrollViewBuilder::default()
            .viewport_size(20, 5)
            .scroll_x(false)
            .build();
        let mut view = ScrollView::new(props.clone());
        *view.content_size.lock().unwrap() = (6, 60);
        let mut state = ScrollViewState::default();
        let mut layout =
            LayoutInfo::from_bounds(crate::event::hit::Bounds::new(0.0, 0.0, 20.0, 5.0));
        layout.insets = [0.0, 2.0, 0.0, 2.0];
        view.layout(layout, &mut props, &mut state);
        for (kind, y) in [(MouseEventKind::Down, 2), (MouseEventKind::Drag, 4)] {
            assert_eq!(
                view.handle_event(
                    &Event::Mouse(
                        MouseEvent::new(kind, Position::cell(19, y)).with_button(MouseButton::Left)
                    ),
                    &mut props,
                    &mut state
                ),
                EventResult::Consumed
            );
        }
        assert_eq!(state.scroll_y, 0, "a one-cell thumb has no travel");
    }

    #[test]
    fn nav_004_scroll_view_fitting_and_disabled_axes_have_zero_ranges() {
        for props in [
            ScrollViewProps::default(),
            ScrollViewProps {
                scroll_x: false,
                scroll_y: false,
                ..Default::default()
            },
        ] {
            let view = ScrollView::new(props.clone());
            if !props.scroll_x {
                *view.content_size.lock().unwrap() = (100, 100);
            }
            let element = view.render(
                &props,
                &ScrollViewState {
                    scroll_x: 50,
                    scroll_y: 50,
                    ..Default::default()
                },
            );
            let node = &element.metadata.accessibility.as_ref().unwrap().inner;
            assert_eq!((node.scroll_x(), node.scroll_y()), (Some(0.0), Some(0.0)));
            assert_eq!(
                (node.scroll_x_max(), node.scroll_y_max()),
                (Some(0.0), Some(0.0))
            );
        }
    }
}
