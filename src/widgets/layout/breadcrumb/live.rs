use super::*;
use crate::{
    accessibility::{Node, Role},
    builder::ElementBuilder,
    component::{ElementType, FocusProps, LayoutInfo, LayoutType},
    event::types::{FocusEventKind, MouseButton, MouseEventKind, WheelDelta},
    layout::style::{Direction, StyleBuilder},
};
use std::sync::{Arc, Mutex};
use unicode_width::UnicodeWidthStr;

#[path = "overflow.rs"]
mod overflow;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct LiveProps {
    pub config: BreadcrumbProps,
    pub seed: BreadcrumbState,
}

impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
struct Geometry {
    sizes: HashMap<String, (usize, usize)>,
    targets: HashMap<String, LayoutInfo>,
    visible: Vec<String>,
    overflow: bool,
}

pub(super) struct LiveBreadcrumb {
    viewport: Option<LayoutInfo>,
    geometry: Arc<Mutex<Geometry>>,
    seed: BreadcrumbState,
    focused: bool,
    hovered: Option<String>,
}

fn separator(props: &BreadcrumbProps) -> String {
    if props.compact {
        props.separator.clone()
    } else {
        format!(" {} ", props.separator)
    }
}

impl LiveBreadcrumb {
    fn available(&self, props: &BreadcrumbProps) -> Option<usize> {
        let actual = self
            .viewport
            .map(|layout| layout.content_size().0.floor().max(0.0) as usize);
        match (actual, props.max_width) {
            (Some(actual), Some(maximum)) => Some(actual.min(maximum)),
            (actual, maximum) => actual.or(maximum),
        }
    }

    fn widths(&self, props: &BreadcrumbProps) -> Option<Vec<usize>> {
        let geometry = self.geometry.lock().unwrap();
        props
            .segments
            .iter()
            .map(|segment| geometry.sizes.get(&segment.id).map(|size| size.0))
            .collect()
    }

    fn target_at(&self, mouse: &crate::event::MouseEvent) -> Option<String> {
        let root = self.viewport?;
        let [a, b, c, d, tx, ty] = root.transform;
        let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
        let (x, y) = (a * x + c * y + tx, b * x + d * y + ty);
        self.geometry
            .lock()
            .unwrap()
            .targets
            .iter()
            .find_map(|(id, layout)| {
                let clip = layout.clip;
                (x >= clip.x
                    && y >= clip.y
                    && x < clip.x + clip.width
                    && y < clip.y + clip.height
                    && layout.local_cell(x, y).is_some())
                .then(|| id.clone())
            })
    }

    fn clamp_scroll(
        &self,
        props: &BreadcrumbProps,
        state: &mut BreadcrumbState,
        reveal_focus: bool,
    ) {
        if props.overflow_strategy != OverflowStrategy::Scroll {
            state.scroll_position = 0;
            return;
        }
        let (Some(widths), Some(available)) = (self.widths(props), self.available(props)) else {
            return;
        };
        let spacing = separator(props).width();
        let mut start = 0usize;
        for (segment, width) in props.segments.iter().zip(widths) {
            let end = start.saturating_add(width);
            if reveal_focus && state.focused_segment.as_ref() == Some(&segment.id) {
                if start < state.scroll_position {
                    state.scroll_position = start;
                } else if end > state.scroll_position.saturating_add(available) {
                    state.scroll_position = end.saturating_sub(available);
                }
            }
            start = end.saturating_add(spacing);
        }
        let total = start.saturating_sub(spacing);
        state.scroll_position = state.scroll_position.min(total.saturating_sub(available));
    }

    fn segment(
        &self,
        props: &BreadcrumbProps,
        state: &BreadcrumbState,
        index: usize,
        width: Option<usize>,
    ) -> Element {
        let segment = &props.segments[index];
        let enabled = segment.clickable && !segment.current;
        let mut children = Vec::new();
        if props.show_icons {
            let icon = if index == 0 && props.show_home_icon {
                Some(&props.home_icon)
            } else {
                segment.icon.as_ref()
            };
            if let Some(icon) = icon {
                children
                    .push(Element::text(format!("{icon} ")).with_class("shrink-0 whitespace-pre"));
            }
        }
        children.push(Element::text(&segment.label).with_class("shrink-0 whitespace-pre"));
        let mut decoration = Node::new(Role::GenericContainer);
        decoration.set_hidden();
        let mut natural_style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Row)
            .flex_shrink(0.0);
        natural_style.unconstrained_width = Some(true);
        let mut natural = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(natural_style)
            .class(if props.compact {
                "flex flex-row shrink-0"
            } else {
                "flex flex-row shrink-0 px-1"
            })
            .children(children)
            .build()
            .with_accessibility(decoration);
        if let Some(class) = &segment.class {
            natural.class = Some(format!(
                "{} {class}",
                natural.class.as_deref().unwrap_or_default()
            ));
        }
        let geometry = self.geometry.clone();
        let id = segment.id.clone();
        natural.metadata.layout.push(Arc::new(move |layout| {
            let size = (
                layout.size.0.ceil().max(0.0) as usize,
                layout.size.1.ceil().max(0.0) as usize,
            );
            geometry.lock().unwrap().sizes.insert(id.clone(), size) != Some(size)
        }));
        let mut style = StyleBuilder::new()
            .display_flex()
            .flex_shrink(0.0)
            .overflow_hidden();
        if let Some(width) = width {
            style = style.width_px(width as f32);
        }
        let mut node = Node::new(Role::Link);
        node.set_label(
            segment
                .aria_label
                .as_ref()
                .unwrap_or(&segment.label)
                .clone(),
        );
        if enabled {
            node.set_clickable();
        }
        if !segment.clickable && !segment.current {
            node.set_disabled();
        }
        if segment.current {
            node.inner.set_aria_current(accesskit::AriaCurrent::Page);
        }
        if let Some(tooltip) = &segment.tooltip {
            node.set_description(tooltip.clone());
        }
        let mut result = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(style)
            .class(if segment.current {
                "font-bold"
            } else if enabled {
                "underline"
            } else {
                "opacity-50"
            })
            .child(natural)
            .build()
            .with_key(format!("segment:{}", segment.id))
            .with_accessibility(node);
        if self.focused && state.focused_segment.as_ref() == Some(&segment.id) {
            result.class = Some(format!(
                "{} reverse",
                result.class.as_deref().unwrap_or_default()
            ));
        }
        if enabled {
            let options = result
                .metadata
                .accessibility_options
                .get_or_insert_default();
            options.focus = state.focused_segment.as_ref() == Some(&segment.id);
            options.focus_event = Some(crate::event::CustomEvent::new(
                "reactive_tui.breadcrumb.focus",
                segment.id.as_bytes().to_vec(),
            ));
        }
        let geometry = self.geometry.clone();
        let id = segment.id.clone();
        result.metadata.layout.push(Arc::new(move |layout| {
            geometry.lock().unwrap().targets.insert(id.clone(), layout);
            false
        }));
        result
    }
}

impl Component for LiveBreadcrumb {
    type Props = LiveProps;
    type State = Arc<Mutex<BreadcrumbState>>;

    fn new(props: Self::Props) -> Self {
        Self {
            viewport: None,
            geometry: Arc::default(),
            seed: props.seed,
            focused: false,
            hovered: None,
        }
    }

    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        Arc::new(Mutex::new(props.seed.clone()))
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        let mut state = state.lock().unwrap();
        if self.seed != props.seed {
            let focus = state.focused_segment.clone().filter(|id| {
                props
                    .config
                    .segments
                    .iter()
                    .any(|segment| &segment.id == id && segment.clickable && !segment.current)
            });
            *state = props.seed.clone();
            if focus.is_some() {
                state.focused_segment = focus;
            }
            self.seed = props.seed.clone();
        }
        // Remeasure changed labels, icons or styling, including currently hidden segments.
        self.geometry.lock().unwrap().sizes.clear();
        self.clamp_scroll(&props.config, &mut state, false);
        true
    }

    fn layout(
        &mut self,
        layout: LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        let before = self.available(&props.config);
        self.viewport = Some(layout);
        let mut state = state.lock().unwrap();
        let scroll_before = state.scroll_position;
        self.clamp_scroll(&props.config, &mut state, false);
        before != self.available(&props.config) || scroll_before != state.scroll_position
    }

    fn render(&self, props: &Self::Props, retained: &Self::State) -> Element {
        let state = retained.lock().unwrap().clone();
        let props = &props.config;
        let separator = separator(props);
        let widths = self.widths(props);
        let plan = overflow::Plan::new(
            props,
            widths.as_deref(),
            self.available(props),
            separator.width(),
        );
        {
            let mut geometry = self.geometry.lock().unwrap();
            geometry.targets.clear();
            geometry.visible = plan
                .entries
                .iter()
                .filter_map(|(index, _)| index.map(|index| props.segments[index].id.clone()))
                .collect();
            geometry.overflow = plan.overflow;
        }
        let mut units = Vec::new();
        for (position, (index, width)) in plan.entries.iter().enumerate() {
            let segment = if let Some(index) = index {
                self.segment(props, &state, *index, *width)
            } else {
                let mut node = Node::new(Role::Label);
                node.set_hidden();
                let mut style = StyleBuilder::new().flex_shrink(0.0).overflow_hidden();
                if let Some(width) = width {
                    style = style.width_px(*width as f32);
                }
                ElementBuilder::new(ElementType::Text("...".into()))
                    .styles(style)
                    .class("shrink-0 whitespace-pre")
                    .build()
                    .with_accessibility(node)
            };
            let mut children = vec![segment];
            if position + 1 < plan.entries.len() {
                let mut node = Node::new(Role::Label);
                node.set_hidden();
                children.push(
                    Element::text(&separator)
                        .with_class("shrink-0 whitespace-pre")
                        .with_accessibility(node),
                );
            }
            units.push(
                Element::layout(LayoutType::Flex)
                    .with_key(index.map_or_else(
                        || "ellipsis".into(),
                        |index| format!("unit:{}", props.segments[index].id),
                    ))
                    .with_class("flex flex-row shrink-0")
                    .with_children(children),
            );
        }
        let scroll = props.overflow_strategy == OverflowStrategy::Scroll;
        let wrap = props.overflow_strategy == OverflowStrategy::Wrap;
        let mut row_style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Row)
            .flex_wrap(wrap);
        if scroll {
            row_style = row_style
                .position_absolute()
                .inset_left(-(state.scroll_position as f32))
                .inset_top(0.0);
            row_style.unconstrained_width = Some(true);
        }
        let row = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(row_style)
            .children(units)
            .build();
        let mut viewport_style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column)
            .overflow_hidden();
        if scroll {
            let height = self
                .geometry
                .lock()
                .unwrap()
                .sizes
                .values()
                .map(|size| size.1)
                .max()
                .unwrap_or(1)
                .max(1);
            viewport_style = viewport_style.height_px(height as f32);
        }
        let viewport = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(viewport_style)
            .child(row)
            .build();
        let mut children = vec![viewport];
        if props.show_tooltips {
            if let Some(tooltip) = self
                .hovered
                .as_ref()
                .and_then(|id| props.segments.iter().find(|segment| &segment.id == id))
                .and_then(|segment| segment.tooltip.as_ref())
            {
                children.push(
                    Element::text(tooltip)
                        .with_key("tooltip")
                        .with_class("whitespace-pre bg-gray-800 text-white"),
                );
            }
        }
        let mut style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column);
        if let Some(width) = props.max_width {
            style = style.max_width_px(width as f32);
        }
        let root = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(style)
            .class("flex flex-col min-w-0 w-full")
            .children(children)
            .build()
            .with_accessibility(Node::new(Role::Navigation));
        let mut root = if props
            .segments
            .iter()
            .any(|segment| segment.clickable && !segment.current)
        {
            root.with_focus(FocusProps::input())
        } else {
            root
        };
        if let Some(class) = &props.class {
            root.class = Some(format!(
                "{} {class}",
                root.class.as_deref().unwrap_or_default()
            ));
        }
        let geometry = self.geometry.clone();
        let retained = retained.clone();
        let enabled: Vec<_> = props
            .segments
            .iter()
            .filter(|segment| segment.clickable && !segment.current)
            .map(|segment| segment.id.clone())
            .collect();
        root.metadata.layout.push(Arc::new(move |_| {
            let (visible, overflow, widths) = {
                let geometry = geometry.lock().unwrap();
                (
                    geometry.visible.clone(),
                    geometry.overflow,
                    geometry
                        .sizes
                        .iter()
                        .map(|(id, size)| (id.clone(), size.0))
                        .collect(),
                )
            };
            let mut state = retained.lock().unwrap();
            state.visible_segments = visible;
            state.has_overflow = overflow;
            state.segment_widths = widths;
            let before = state.focused_segment.clone();
            if !state
                .focused_segment
                .as_ref()
                .is_some_and(|id| enabled.contains(id) && state.visible_segments.contains(id))
            {
                state.focused_segment = enabled
                    .iter()
                    .rev()
                    .find(|id| state.visible_segments.contains(id))
                    .cloned();
            }
            before != state.focused_segment
        }));
        root
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        let mut state = state.lock().unwrap();
        let config = &props.config;
        match event {
            Event::Focus(focus) => match focus.kind {
                FocusEventKind::Gained => self.focused = true,
                FocusEventKind::Lost => {
                    self.focused = false;
                    self.hovered = None;
                }
                _ => return EventResult::Ignored,
            },
            Event::Key(key) if self.focused && config.keyboard_navigation => {
                let result = Breadcrumb.handle_keyboard_event(key, config, &mut state);
                self.clamp_scroll(config, &mut state, true);
                return result;
            }
            Event::Custom(event) if event.name == "reactive_tui.breadcrumb.focus" => {
                let Ok(id) = std::str::from_utf8(&event.data) else {
                    return EventResult::Ignored;
                };
                if !config
                    .segments
                    .iter()
                    .any(|segment| segment.id == id && segment.clickable && !segment.current)
                {
                    return EventResult::Ignored;
                }
                state.focused_segment = Some(id.into());
                self.clamp_scroll(config, &mut state, true);
            }
            Event::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click)
                    && mouse.button == MouseButton::Left =>
            {
                let Some(id) = self.target_at(mouse) else {
                    return EventResult::Ignored;
                };
                if !Breadcrumb::activate_segment(&id, config) {
                    return EventResult::Ignored;
                }
                state.focused_segment = Some(id);
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Move => {
                let hovered = self.target_at(mouse);
                if hovered == self.hovered {
                    return EventResult::Ignored;
                }
                self.hovered = hovered;
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Leave => {
                self.hovered = None;
            }
            Event::Mouse(mouse)
                if mouse.kind == MouseEventKind::Wheel
                    && config.overflow_strategy == OverflowStrategy::Scroll =>
            {
                let delta = match mouse.wheel.as_ref().map(|wheel| &wheel.delta) {
                    Some(WheelDelta::Lines { x, y }) => {
                        if *x != 0.0 {
                            *x as f64
                        } else {
                            *y as f64
                        }
                    }
                    Some(WheelDelta::Pixels { x, y }) => {
                        if *x != 0.0 {
                            *x as f64
                        } else {
                            *y as f64
                        }
                    }
                    None => return EventResult::Ignored,
                };
                state.scroll_position =
                    super::super::scroll_view::shifted(state.scroll_position, delta, 3);
                self.clamp_scroll(config, &mut state, false);
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
}
