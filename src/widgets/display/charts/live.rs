use super::*;
use crate::{
    builder::ElementBuilder,
    component::{ElementType, FocusProps, LayoutInfo, LayoutType, LifecycleEvent},
    event::{
        router::EventResult,
        types::{KeyCode, KeyEventKind, MouseEventKind},
        Event,
    },
    layout::style::StyleBuilder,
};
use std::sync::Mutex;

mod canvas;
mod motion;

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub config: ChartProps,
    pub seed: ChartState,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub(super) struct LiveChart {
    viewport: Option<LayoutInfo>,
    previous: ChartProps,
    seed: ChartState,
    picture: Mutex<canvas::Canvas>,
    motion: motion::Reveal,
}

impl Component for LiveChart {
    type Props = LiveProps;
    type State = ChartState;

    fn new(props: Self::Props) -> Self {
        Self {
            viewport: None,
            previous: props.config,
            seed: props.seed,
            picture: Mutex::new(canvas::Canvas::new(0, 0)),
            motion: motion::Reveal::new(),
        }
    }
    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        props.seed.clone()
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if self.seed != props.seed {
            *state = props.seed.clone();
            self.seed = props.seed.clone();
        }
        if self.previous != props.config {
            self.motion.restart();
            state.hovered_point = None;
            state.tooltip = None;
            self.previous = props.config.clone();
        }
        true
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut Self::State) -> bool {
        let changed = self
            .viewport
            .is_none_or(|old| old.content_size() != layout.content_size());
        self.viewport = Some(layout);
        changed
    }
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let config = &props.config;
        let (width, height) = self.viewport.map_or((0, 0), |v| {
            let (w, h) = v.content_size();
            (w.max(0.0) as usize, h.max(0.0) as usize)
        });
        let error = canvas::validate(config).err();
        let progress = self.motion.fraction(
            config,
            error.is_none()
                && width > 0
                && height > 0
                && config
                    .series
                    .iter()
                    .any(|series| series.visible && !series.data.is_empty()),
        );
        let mut picture = canvas::draw(config, width, height, progress);
        if config.show_tooltips {
            if let Some((series, point)) = state.hovered_point {
                if let Some(text) = tooltip(config, series, point) {
                    picture.tooltip(
                        &text,
                        state
                            .tooltip
                            .as_ref()
                            .map(|(_, x, y)| (*x as usize, *y as usize)),
                    );
                }
            } else if let Some((text, x, y)) = &state.tooltip {
                picture.tooltip(text, Some((*x as usize, *y as usize)));
            }
        }
        let elements = picture.elements();
        *self.picture.lock().unwrap() = picture;
        let insets = self.viewport.map_or([0.0; 4], |v| v.insets);
        let content = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(insets[0])
                    .inset_top(insets[1])
                    .width_px(width as f32)
                    .height_px(height as f32)
                    .overflow_hidden(),
            )
            .children(elements)
            .build();
        let mut element = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .display_flex()
                    .width_px(if config.width == 0 || config.height == 0 {
                        24.0
                    } else {
                        config.width as f32
                    })
                    .height_px(if config.width == 0 || config.height == 0 {
                        2.0
                    } else {
                        config.height as f32
                    })
                    .max_width_percent(100.0)
                    .max_height_percent(100.0)
                    .min_width_px(0.0)
                    .min_height_px(0.0)
                    .overflow_hidden(),
            )
            .children(vec![content])
            .build();
        element.focus = Some(FocusProps::button());
        let mut node = crate::accessibility::Node::new(crate::accessibility::Role::Image);
        node.set_label(config.title.as_deref().unwrap_or("Chart"));
        element.metadata.accessibility = Some(node);
        if config.show_tooltips {
            let text = state
                .hovered_point
                .and_then(|(series, point)| tooltip(config, series, point))
                .or_else(|| state.tooltip.as_ref().map(|(text, _, _)| text.clone()))
                .unwrap_or_default();
            if !text.is_empty() {
                element = element.with_child(
                    Element::text(text)
                        .with_key("chart-point-announcement")
                        .class("sr-only aria-live-polite"),
                );
            }
        }
        if let Some(class) = &config.class {
            element = element.with_class(class);
        }
        element
    }
    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if !props.config.show_tooltips {
            return EventResult::Ignored;
        }
        match event {
            Event::Mouse(mouse)
                if matches!(
                    mouse.kind,
                    MouseEventKind::Move | MouseEventKind::Enter | MouseEventKind::Leave
                ) =>
            {
                let hit = if mouse.kind == MouseEventKind::Leave {
                    None
                } else {
                    self.viewport.and_then(|v| {
                        let x = mouse.position.x() as f32;
                        let y = mouse.position.y() as f32;
                        // App dispatch has already converted the event to this
                        // component's local coordinates. Only clipping is global.
                        let [a, b, c, d, tx, ty] = v.transform;
                        let screen = crate::event::hit::Point {
                            x: a * x + c * y + tx,
                            y: b * x + d * y + ty,
                        };
                        if !v.clip.contains(screen) {
                            return None;
                        }
                        let x = (x - v.insets[0]).floor();
                        let y = (y - v.insets[1]).floor();
                        if x < 0.0 || y < 0.0 {
                            return None;
                        }
                        self.picture
                            .lock()
                            .unwrap()
                            .point_at(x as usize, y as usize)
                            .map(|point| (point, x as u16, y as u16))
                    })
                };
                state.hovered_point = hit.map(|(point, _, _)| point);
                state.tooltip = hit.and_then(|((s, p), x, y)| {
                    tooltip(&props.config, s, p).map(|text| (text, x, y))
                });
                EventResult::Consumed
            }
            Event::Key(key)
                if key.kind != KeyEventKind::Release
                    && matches!(
                        key.code,
                        KeyCode::Left
                            | KeyCode::Right
                            | KeyCode::Home
                            | KeyCode::End
                            | KeyCode::Escape
                    ) =>
            {
                if key.code == KeyCode::Escape {
                    state.hovered_point = None;
                    state.tooltip = None;
                    return EventResult::Consumed;
                }
                let points: Vec<_> = props
                    .config
                    .series
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.visible)
                    .flat_map(|(s, series)| (0..series.data.len()).map(move |p| (s, p)))
                    .collect();
                if points.is_empty() {
                    return EventResult::Ignored;
                }
                let current = state
                    .hovered_point
                    .and_then(|p| points.iter().position(|v| *v == p));
                let index = match key.code {
                    KeyCode::Home => 0,
                    KeyCode::End => points.len() - 1,
                    KeyCode::Left => current.unwrap_or(1).saturating_sub(1),
                    _ => current.map_or(0, |i| (i + 1).min(points.len() - 1)),
                };
                state.hovered_point = Some(points[index]);
                state.tooltip = None;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut Self::State) {
        if matches!(event, LifecycleEvent::Unmount) {
            self.motion.cancel();
        }
    }
}

fn tooltip(props: &ChartProps, series: usize, point: usize) -> Option<String> {
    let series = props.series.get(series).filter(|s| s.visible)?;
    let data = series.data.get(point)?;
    let label = data.label.clone().unwrap_or_else(|| point.to_string());
    let mut text = format!("{} / {}: {}", series.name, label, data.value);
    let mut metadata: Vec<_> = data.metadata.iter().collect();
    metadata.sort();
    for (key, value) in metadata {
        text.push_str(&format!("; {key}={value}"));
    }
    Some(text)
}
