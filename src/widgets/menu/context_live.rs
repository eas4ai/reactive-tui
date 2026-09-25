use super::{
    popup_live::{LivePopup, LiveProps as PopupProps},
    view::node,
    ContextMenuProps, ContextMenuState, PopupMenuProps, PopupPlacement, TextCallback,
};
use crate::{
    component::{Component, Element, LayoutInfo, Props},
    event::{
        router::EventResult,
        types::{Event, MouseButton, MouseEventKind},
    },
    layout::style::StyleBuilder,
    reactive::{
        component_scope,
        scheduler::{Scheduler, TimerId},
        ThreadSafeSignal,
    },
};
use std::{any::Any, sync::Arc, time::Duration};

type ShowCallback = Arc<dyn Fn(u16, u16) + Send + Sync>;
#[derive(Clone)]
pub(super) struct LiveProps {
    pub config: ContextMenuProps,
    pub seed: ContextMenuState,
    pub auto_close: bool,
    pub selected: Option<TextCallback>,
    pub shown: Option<ShowCallback>,
    pub hidden: Option<Arc<dyn Fn() + Send + Sync>>,
}
impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.seed == other.seed
            && self.auto_close == other.auto_close
            && super::menubar_live::same(&self.selected, &other.selected)
            && super::menubar_live::same(&self.shown, &other.shown)
            && super::menubar_live::same(&self.hidden, &other.hidden)
    }
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) struct LiveContext {
    state: ThreadSafeSignal<Option<(u16, u16)>>,
    seed: ContextMenuState,
    config: ContextMenuProps,
    root: Option<LayoutInfo>,
    scheduler: Option<Arc<Scheduler>>,
    timer: Option<TimerId>,
    press: Option<(u16, u16)>,
}
impl LiveContext {
    fn cancel(&mut self) {
        if let (Some(scheduler), Some(timer)) = (&self.scheduler, self.timer.take()) {
            scheduler.cancel_timer(timer);
        }
        self.press = None;
    }
}
impl Component for LiveContext {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        Self {
            state: ThreadSafeSignal::new(
                props
                    .seed
                    .is_visible
                    .then_some(props.seed.position.unwrap_or((0, 0))),
            ),
            seed: props.seed,
            config: props.config,
            root: None,
            scheduler: component_scope::current().map(|scope| scope.scheduler()),
            timer: None,
            press: None,
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        if self.config != props.config {
            self.cancel();
            self.config = props.config.clone();
        }
        if self.seed != props.seed {
            self.cancel();
            self.state.set(
                props
                    .seed
                    .is_visible
                    .then_some(props.seed.position.unwrap_or((0, 0))),
            );
            self.seed = props.seed.clone();
        }
        if !props.config.enabled || !props.config.show_on_long_press {
            self.cancel();
        }
        if !props.config.enabled {
            self.state.set(None);
        }
        true
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut ()) -> bool {
        let changed = self.root != Some(layout);
        self.root = Some(layout);
        changed
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        let mut children = Vec::new();
        if let Some((x, y)) = self.state.get().filter(|_| props.config.enabled) {
            let hidden = props.hidden.clone();
            let state = self.state.clone();
            let shown = props.shown.clone();
            children.push(
                Element::typed::<LivePopup>(PopupProps {
                    config: PopupMenuProps {
                        items: props.config.items.clone(),
                        style: props.config.style.clone(),
                        visible: true,
                        enabled: props.config.enabled,
                        placement: PopupPlacement::Position { x, y },
                        auto_close: props.auto_close,
                        close_on_outside_click: props.config.close_on_outside_click,
                        max_visible_items: props.config.max_visible_items,
                        width: props.config.width,
                        show_border: props.config.show_border,
                        show_shadow: props.config.show_shadow,
                    },
                    relative_placement: None,
                    seed: props.seed.popup_state.clone(),
                    selected: props.selected.clone(),
                    shown: Some(Arc::new(move || {
                        if let Some(callback) = &shown {
                            callback(x, y);
                        }
                    })),
                    hidden: Some(Arc::new(move || {
                        state.set(None);
                        if let Some(callback) = &hidden {
                            callback();
                        }
                    })),
                })
                .with_key("context-popup"),
            );
        }
        let mut root = node(
            StyleBuilder::new()
                .width_percent(100.0)
                .height_percent(100.0),
            children,
        );
        root.metadata.disabled = !props.config.enabled;
        root
    }
    fn handle_event(&mut self, event: &Event, props: &mut Self::Props, _: &mut ()) -> EventResult {
        if !props.config.enabled {
            return EventResult::Ignored;
        }
        let Event::Mouse(mouse) = event else {
            return EventResult::Ignored;
        };
        if matches!(mouse.kind, MouseEventKind::Up | MouseEventKind::Leave) {
            self.cancel();
            return EventResult::Ignored;
        }
        let Some(root) = self.root else {
            return EventResult::Ignored;
        };
        let [a, b, c, d, tx, ty] = root.transform;
        let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
        let (x, y) = (a * x + c * y + tx, b * x + d * y + ty);
        if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
            return EventResult::Ignored;
        }
        let (x, y) = (x as u16, y as u16);
        let inside = props.config.trigger_areas.is_empty()
            || props
                .config
                .trigger_areas
                .iter()
                .any(|&(left, top, width, height)| {
                    x >= left
                        && y >= top
                        && u32::from(x) < u32::from(left) + u32::from(width)
                        && u32::from(y) < u32::from(top) + u32::from(height)
                });
        match mouse.kind {
            MouseEventKind::Down | MouseEventKind::Click
                if mouse.button == MouseButton::Right
                    && props.config.show_on_right_click
                    && inside =>
            {
                self.cancel();
                self.state.set(Some((x, y)));
                EventResult::Consumed
            }
            MouseEventKind::Down
                if mouse.button == MouseButton::Left
                    && props.config.show_on_long_press
                    && inside
                    && self.state.get().is_none() =>
            {
                self.cancel();
                let Some(scheduler) = &self.scheduler else {
                    return EventResult::Ignored;
                };
                let state = self.state.clone();
                self.timer = Some(scheduler.schedule_timeout(
                    Duration::from_millis(props.config.long_press_duration),
                    move || state.set(Some((x, y))),
                ));
                self.press = Some((x, y));
                EventResult::Handled
            }
            MouseEventKind::Move | MouseEventKind::Drag
                if self
                    .press
                    .is_some_and(|(px, py)| x.abs_diff(px) > 5 || y.abs_diff(py) > 5) =>
            {
                self.cancel();
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }
    fn on_lifecycle(&mut self, event: crate::component::LifecycleEvent, _: &mut ()) {
        if event == crate::component::LifecycleEvent::Unmount {
            self.cancel();
        }
    }
}
impl Drop for LiveContext {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        component::LifecycleEvent,
        event::{
            hit::Bounds,
            types::{MouseEvent, Position},
        },
    };
    fn fixture() -> (LiveContext, LiveProps, Arc<Scheduler>) {
        let props = LiveProps {
            config: ContextMenuProps {
                show_on_long_press: true,
                long_press_duration: 0,
                ..Default::default()
            },
            seed: ContextMenuState::default(),
            auto_close: true,
            selected: None,
            shown: None,
            hidden: None,
        };
        let scheduler = Arc::new(Scheduler::new());
        let mut live = LiveContext::new(props.clone());
        live.scheduler = Some(scheduler.clone());
        live.root = Some(LayoutInfo::from_bounds(Bounds {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 12.0,
        }));
        (live, props, scheduler)
    }
    fn mouse(kind: MouseEventKind, x: u16, y: u16) -> Event {
        Event::Mouse(MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left))
    }
    #[test]
    fn release_movement_prop_change_and_unmount_cancel_ready_long_press() {
        for reason in 0..4 {
            let (mut live, mut props, scheduler) = fixture();
            live.handle_event(&mouse(MouseEventKind::Down, 2, 2), &mut props, &mut ());
            assert!(scheduler.next_deadline().is_some());
            match reason {
                0 => {
                    live.handle_event(&mouse(MouseEventKind::Up, 2, 2), &mut props, &mut ());
                }
                1 => {
                    live.handle_event(&mouse(MouseEventKind::Move, 10, 2), &mut props, &mut ());
                }
                2 => {
                    props.config.enabled = false;
                    live.update(&props, &mut ());
                }
                _ => live.on_lifecycle(LifecycleEvent::Unmount, &mut ()),
            }
            assert!(scheduler.next_deadline().is_none());
            scheduler.process_timers();
            assert_eq!(live.state.get(), None);
        }
    }
    #[test]
    fn long_press_timer_is_owned_by_each_context_menu() {
        let (mut first, mut props, scheduler) = fixture();
        let second = LiveContext::new(props.clone());
        first.handle_event(&mouse(MouseEventKind::Down, 2, 2), &mut props, &mut ());
        scheduler.process_timers();
        assert_eq!(first.state.get(), Some((2, 2)));
        assert_eq!(second.state.get(), None);
    }
}
