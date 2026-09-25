use super::*;
use crate::{
    component::LayoutInfo,
    reactive::{
        component_scope,
        scheduler::{Scheduler, TimerId},
        ThreadSafeSignal,
    },
};
mod events;
mod render;

#[derive(Clone)]
pub(super) struct LiveProps {
    pub owner: Popover,
    pub config: PopoverProps,
}
impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner.state, &other.owner.state) && self.config == other.config
    }
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
pub(super) struct LivePopover(Arc<Runtime>);
impl Component for LivePopover {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        props.owner.live.data.lock().unwrap().scheduler =
            component_scope::current().map(|scope| scope.scheduler());
        Self(props.owner.live)
    }
    fn render(&self, props: &Self::Props, _: &Self::State) -> Element {
        props.owner.render_live(&props.config)
    }
    fn update(&mut self, props: &Self::Props, _: &mut Self::State) -> bool {
        if !Arc::ptr_eq(&self.0, &props.owner.live) {
            self.0.cancel();
            self.0 = props.owner.live.clone();
            self.0.data.lock().unwrap().scheduler =
                component_scope::current().map(|scope| scope.scheduler());
        }
        true
    }
    fn on_lifecycle(&mut self, event: crate::component::LifecycleEvent, _: &mut Self::State) {
        if event == crate::component::LifecycleEvent::Unmount {
            self.0.cancel();
        }
    }
}

#[derive(Default)]
struct Data {
    root: Option<LayoutInfo>,
    trigger: Option<LayoutInfo>,
    body: Option<LayoutInfo>,
    explicit_trigger: bool,
    authored: Option<bool>,
    requested: bool,
    reported: bool,
    focus_dismissed: bool,
    target: bool,
    from: f32,
    changed_at: Option<Instant>,
    hover: Option<(bool, Instant)>,
    scheduler: Option<Arc<Scheduler>>,
    timer: Option<(TimerId, Instant)>,
}

pub(super) struct Runtime {
    data: Mutex<Data>,
    changed: ThreadSafeSignal<u64>,
}
impl Default for Runtime {
    fn default() -> Self {
        Self {
            data: Mutex::default(),
            changed: ThreadSafeSignal::new(0),
        }
    }
}
impl Runtime {
    fn reject(&self, state: &Mutex<PopoverState>, props: &PopoverProps) {
        {
            let mut data = self.data.lock().unwrap();
            data.requested = false;
            data.authored = None;
            state.lock().unwrap().visible = false;
        }
        self.sample(
            state,
            &PopoverProps {
                visible: false,
                animation: PopoverAnimation::None,
                trigger: PopoverTrigger::Manual,
                ..props.clone()
            },
        );
        self.data.lock().unwrap().authored = None;
        self.cancel();
    }
    pub(super) fn request(&self, state: &Mutex<PopoverState>, visible: bool) {
        {
            let mut data = self.data.lock().unwrap();
            data.requested = true;
            let mut state = state.lock().unwrap();
            if !visible && state.is_focused {
                data.focus_dismissed = true;
            }
            state.visible = visible;
        }
        self.changed
            .update(|revision| *revision = revision.wrapping_add(1));
    }
    pub(super) fn set_trigger(&self, state: &Mutex<PopoverState>, rect: Rect<f32>) {
        {
            let mut data = self.data.lock().unwrap();
            data.explicit_trigger = [rect.left, rect.top, rect.right, rect.bottom]
                .into_iter()
                .all(f32::is_finite)
                && rect.left <= rect.right
                && rect.top <= rect.bottom;
            if data.explicit_trigger {
                state.lock().unwrap().trigger_rect = rect;
            }
        }
        self.changed
            .update(|revision| *revision = revision.wrapping_add(1));
    }
    fn record(&self, part: Part, layout: LayoutInfo) -> bool {
        let mut data = self.data.lock().unwrap();
        let target = match part {
            Part::Root => &mut data.root,
            Part::Trigger => &mut data.trigger,
            Part::Body => &mut data.body,
        };
        let changed = target.is_none_or(|old| match part {
            Part::Body => old.size != layout.size,
            _ => old != layout,
        });
        *target = Some(layout);
        changed
    }
    fn sample(&self, state: &Mutex<PopoverState>, props: &PopoverProps) -> (bool, f32) {
        self.changed.get();
        self.at(state, props, Instant::now())
    }
    fn at(&self, state: &Mutex<PopoverState>, props: &PopoverProps, now: Instant) -> (bool, f32) {
        let (visible, progress, notification) = {
            let mut data = self.data.lock().unwrap();
            let mut state = state.lock().unwrap();
            if data.scheduler.is_none() {
                data.scheduler = component_scope::current().map(|scope| scope.scheduler());
            }
            if data.authored != Some(props.visible) {
                if !data.requested {
                    state.visible = props.visible;
                }
                data.authored = Some(props.visible);
            }
            data.requested = false;
            if props.trigger != PopoverTrigger::Hover {
                data.hover = None;
            }
            if let Some((visible, deadline)) = data.hover {
                if deadline <= now {
                    state.visible = visible;
                    data.hover = None;
                }
            }
            let duration = props.animation_duration.as_secs_f32();
            let fraction = |data: &Data| {
                let t = data
                    .changed_at
                    .map_or(1.0, |at| {
                        now.saturating_duration_since(at).as_secs_f32() / duration.max(f32::EPSILON)
                    })
                    .min(1.0);
                data.from + (f32::from(data.target) - data.from) * t
            };
            if data.target != state.visible {
                data.from = fraction(&data);
                data.target = state.visible;
                data.changed_at = Some(now);
            }
            let progress = if props.animation == PopoverAnimation::None
                || props.animation_duration.is_zero()
                || data.scheduler.is_none()
            {
                f32::from(state.visible)
            } else {
                fraction(&data)
            };
            state.animation_progress = progress;
            state.is_animating = progress != f32::from(state.visible);
            state.animation_start = state.is_animating.then_some(data.changed_at.unwrap_or(now));
            let next = if state.is_animating {
                Some(now + Duration::from_millis(16))
            } else {
                None
            };
            let next = match (next, data.hover.map(|(_, at)| at)) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            if let Some(scheduler) = data.scheduler.clone() {
                if data.timer.is_some_and(|(_, deadline)| {
                    deadline <= now || next.is_none_or(|next| next < deadline)
                }) {
                    if let Some((id, _)) = data.timer.take() {
                        scheduler.cancel_timer(id);
                    }
                }
                if data.timer.is_none() {
                    if let Some(deadline) = next {
                        data.timer = Some((
                            scheduler
                                .schedule_timeout(deadline.saturating_duration_since(now), || {}),
                            deadline,
                        ));
                    }
                }
            }
            let notification = (data.reported != state.visible).then_some(state.visible);
            data.reported = state.visible;
            (state.visible, progress, notification)
        };
        if let Some(visible) = notification {
            if let Some(callback) = if visible {
                &props.on_open
            } else {
                &props.on_close
            } {
                callback();
            }
        }
        (visible, progress)
    }
    pub(super) fn cancel(&self) {
        let mut data = self.data.lock().unwrap();
        if let (Some(scheduler), Some((id, _))) = (data.scheduler.take(), data.timer.take()) {
            scheduler.cancel_timer(id);
        }
        data.hover = None;
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[derive(Clone, Copy)]
enum Part {
    Root,
    Trigger,
    Body,
}

fn validation_error(props: &PopoverProps) -> Option<&'static str> {
    if props
        .min_width
        .zip(props.max_width)
        .is_some_and(|(min, max)| min > max)
    {
        return Some("Invalid Popover: minimum width exceeds maximum");
    }
    if props
        .min_height
        .zip(props.max_height)
        .is_some_and(|(min, max)| min > max)
    {
        return Some("Invalid Popover: minimum height exceeds maximum");
    }
    let now = Instant::now();
    if [
        props.hover_delay,
        props.hover_leave_delay,
        props.animation_duration,
    ]
    .into_iter()
    .any(|duration| now.checked_add(duration).is_none())
    {
        return Some("Invalid Popover: duration exceeds the clock range");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hover_leave_cancels_pending_open_and_reentry_cancels_pending_close() {
        let scheduler = Arc::new(Scheduler::new());
        let owner = Popover::new();
        owner.live.data.lock().unwrap().scheduler = Some(scheduler.clone());
        let props = PopoverProps {
            trigger: PopoverTrigger::Hover,
            animation: PopoverAnimation::None,
            hover_delay: Duration::from_secs(1),
            hover_leave_delay: Duration::from_secs(1),
            ..Default::default()
        };
        let boundary = |kind| {
            Event::Mouse(crate::event::types::MouseEvent::new(
                kind,
                Position::cell(0, 0),
            ))
        };
        owner.observe(&boundary(MouseEventKind::Enter), &props, true);
        assert_eq!(owner.live.sample(&owner.state, &props), (false, 0.0));
        assert!(scheduler.next_deadline().is_some());
        owner.observe(&boundary(MouseEventKind::Leave), &props, true);
        let deadline = owner.live.data.lock().unwrap().hover.unwrap().1;
        assert_eq!(owner.live.at(&owner.state, &props, deadline), (false, 0.0));
        assert!(scheduler.next_deadline().is_none());
        owner.observe(&boundary(MouseEventKind::Enter), &props, true);
        let deadline = owner.live.data.lock().unwrap().hover.unwrap().1;
        assert_eq!(owner.live.at(&owner.state, &props, deadline), (true, 1.0));
        owner.observe(&boundary(MouseEventKind::Leave), &props, true);
        owner.observe(&boundary(MouseEventKind::Enter), &props, false);
        let deadline = owner.live.data.lock().unwrap().hover.unwrap().1;
        assert_eq!(owner.live.at(&owner.state, &props, deadline), (true, 1.0));
        assert!(scheduler.next_deadline().is_none());
    }
    #[test]
    fn replacing_manual_owner_cancels_old_work_and_unmount_cancels_the_new_owner() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _binding = scope.enter(true);
        let props = LiveProps {
            owner: Popover::new(),
            config: PopoverProps {
                visible: true,
                ..Default::default()
            },
        };
        let mut child = LivePopover::new(props.clone());
        let _old_output = child.render(&props, &());
        assert!(scheduler.next_deadline().is_some());
        let next = LiveProps {
            owner: Popover::new(),
            config: props.config.clone(),
        };
        child.update(&next, &mut ());
        assert!(scheduler.next_deadline().is_none());
        let _new_output = child.render(&next, &());
        assert!(scheduler.next_deadline().is_some());
        child.on_lifecycle(crate::component::LifecycleEvent::Unmount, &mut ());
        assert!(scheduler.next_deadline().is_none());
        scope.close();
    }
    #[test]
    fn animation_reverses_from_current_value_and_releases_its_deadline() {
        let scheduler = Arc::new(Scheduler::new());
        let owner = Popover::new();
        owner.live.data.lock().unwrap().scheduler = Some(scheduler.clone());
        let props = PopoverProps {
            visible: true,
            animation_duration: Duration::from_secs(1),
            ..Default::default()
        };
        let start = Instant::now();
        assert_eq!(owner.live.at(&owner.state, &props, start), (true, 0.0));
        let quarter = start + Duration::from_millis(250);
        assert_eq!(owner.live.at(&owner.state, &props, quarter), (true, 0.25));
        owner.hide();
        assert_eq!(owner.live.at(&owner.state, &props, quarter), (false, 0.25));
        let halfway = start + Duration::from_millis(500);
        assert_eq!(
            owner.live.at(&owner.state, &props, halfway),
            (false, 0.1875)
        );
        owner.show();
        assert_eq!(owner.live.at(&owner.state, &props, halfway), (true, 0.1875));
        assert!(scheduler.next_deadline().is_some());
        assert_eq!(
            owner
                .live
                .at(&owner.state, &props, start + Duration::from_secs(2)),
            (true, 1.0)
        );
        assert!(!owner.state.lock().unwrap().is_animating);
        assert!(scheduler.next_deadline().is_none());
    }
    #[test]
    fn mounted_child_cancels_deadlines_while_public_handle_remains_alive() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _binding = scope.enter(true);
        let owner = Popover::new();
        let props = LiveProps {
            owner: owner.clone(),
            config: PopoverProps {
                visible: true,
                ..Default::default()
            },
        };
        let mut child = LivePopover::new(props.clone());
        let output = child.render(&props, &());
        assert!(scheduler.next_deadline().is_some());
        child.on_lifecycle(crate::component::LifecycleEvent::Unmount, &mut ());
        assert!(scheduler.next_deadline().is_none());
        assert!(owner.is_visible());
        drop(output);
        scope.close();
    }
}
