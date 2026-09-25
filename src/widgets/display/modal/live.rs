use super::*;
use crate::event::{
    router::EventResult,
    types::{Event, KeyCode, MouseEventKind},
};
use crate::{
    component::{LayoutInfo, LifecycleEvent},
    reactive::{
        component_scope,
        scheduler::{Scheduler, TimerId},
        ThreadSafeSignal,
    },
};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
use taffy::geometry::Rect;

mod events;
mod render;

#[derive(Clone)]
pub(super) struct LiveProps {
    pub config: ModalProps,
    pub seed: ModalState,
    pub role: crate::accessibility::Role,
    pub escape_closable: bool,
    pub motion: Option<Motion>,
    pub on_presented: Option<Arc<dyn Fn() + Send + Sync>>,
}
impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.role == other.role
            && self.escape_closable == other.escape_closable
            && self.motion == other.motion
            && super::super::overlay::same_callback(&self.on_presented, &other.on_presented)
    }
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub(super) struct LiveModal(Arc<Runtime>);
impl Component for LiveModal {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        Self(Arc::new(Runtime {
            data: Mutex::new(Data {
                observed: props.seed,
                ..Default::default()
            }),
            changed: ThreadSafeSignal::new(0),
            scheduler: component_scope::current().map(|scope| scope.scheduler()),
        }))
    }
    fn render(&self, props: &Self::Props, _: &Self::State) -> Element {
        self.0.render(
            &props.config,
            props.role,
            props.escape_closable,
            props.motion.as_ref(),
            props.on_presented.as_ref(),
        )
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut Self::State) {
        if event == LifecycleEvent::Unmount {
            self.0.cancel();
        }
    }
}

#[derive(Clone, Copy)]
enum Part {
    Root,
    Body,
    Content,
    Header,
    Title,
    Footer,
    Buttons,
}
#[derive(Default, Clone, Copy)]
struct Measurements {
    root: Option<LayoutInfo>,
    body: Option<LayoutInfo>,
    content: Option<LayoutInfo>,
    header: Option<LayoutInfo>,
    title: Option<LayoutInfo>,
    footer: Option<LayoutInfo>,
    buttons: Option<LayoutInfo>,
}
struct Drag {
    handle: Option<ResizeHandle>,
    start: (f32, f32),
    rect: Rect<f32>,
}
#[derive(Default)]
struct Data {
    previous: Option<ModalProps>,
    visible: bool,
    measurements: Measurements,
    observed: ModalState,
    position: Option<(u16, u16)>,
    size: Option<(u16, u16)>,
    rect: Rect<f32>,
    bounds: Rect<f32>,
    drag: Option<Drag>,
    target: bool,
    from: f32,
    changed_at: Option<Instant>,
    timer: Option<(TimerId, Instant)>,
}

struct Runtime {
    data: Mutex<Data>,
    changed: ThreadSafeSignal<u64>,
    scheduler: Option<Arc<Scheduler>>,
}
impl Runtime {
    fn wake(&self) {
        self.changed
            .update(|revision| *revision = revision.wrapping_add(1));
    }
    fn cancel(&self) {
        let mut data = self.data.lock().unwrap();
        if let (Some(scheduler), Some((id, _))) = (&self.scheduler, data.timer.take()) {
            scheduler.cancel_timer(id);
        }
        data.drag = None;
    }
    fn close(&self, props: &ModalProps, reason: ModalCloseReason) {
        let changed = {
            let mut data = self.data.lock().unwrap();
            let changed = data.visible;
            data.visible = false;
            data.drag = None;
            changed
        };
        if changed {
            self.wake();
            if let Some(callback) = &props.on_close {
                callback(reason);
            }
        }
    }
    fn button(&self, props: &ModalProps, button: &ModalButton) {
        {
            let mut data = self.data.lock().unwrap();
            if button.disabled || !data.visible {
                return;
            }
            if !matches!(button.action, ModalButtonAction::Custom(_)) {
                data.visible = false;
                data.drag = None;
            }
        }
        self.wake();
        Modal.handle_button_click(props, button);
    }
    fn record(&self, part: Part, layout: LayoutInfo) -> bool {
        let mut data = self.data.lock().unwrap();
        let slot = match part {
            Part::Root => &mut data.measurements.root,
            Part::Body => &mut data.measurements.body,
            Part::Content => &mut data.measurements.content,
            Part::Header => &mut data.measurements.header,
            Part::Title => &mut data.measurements.title,
            Part::Footer => &mut data.measurements.footer,
            Part::Buttons => &mut data.measurements.buttons,
        };
        let changed = slot.is_none_or(|old| match part {
            Part::Root => old != layout,
            _ => old.size != layout.size || old.insets != layout.insets,
        });
        *slot = Some(layout);
        changed
    }
    #[cfg(test)]
    fn sample(&self, props: &ModalProps, now: Instant) -> (bool, f32, Measurements) {
        self.sample_with_duration(props, now, Duration::from_millis(200))
    }
    fn sample_with_duration(
        &self,
        props: &ModalProps,
        now: Instant,
        duration: Duration,
    ) -> (bool, f32, Measurements) {
        self.changed.get();
        let mut data = self.data.lock().unwrap();
        if data
            .previous
            .as_ref()
            .is_none_or(|old| old.visible != props.visible)
        {
            data.visible = props.visible;
        }
        if data
            .previous
            .as_ref()
            .is_some_and(|old| old.position != props.position)
        {
            data.position = None;
        }
        if data
            .previous
            .as_ref()
            .is_some_and(|old| old.width != props.width || old.height != props.height)
        {
            data.size = None;
        }
        if !props.draggable && !props.resizable {
            data.drag = None;
        }
        data.previous = Some(props.clone());
        let fraction = |data: &Data| {
            let t = data
                .changed_at
                .map_or(1.0, |start| {
                    if duration.is_zero() {
                        1.0
                    } else {
                        now.saturating_duration_since(start).as_secs_f32() / duration.as_secs_f32()
                    }
                })
                .min(1.0);
            data.from + (f32::from(data.target) - data.from) * t
        };
        if data.target != data.visible {
            data.from = fraction(&data);
            data.target = data.visible;
            data.changed_at = Some(now);
        }
        let reduced = props.modal_style.as_deref().is_some_and(|class| {
            class
                .split_whitespace()
                .any(|token| token == "reduced-motion")
        });
        let progress =
            if props.animation == ModalAnimation::None || reduced || self.scheduler.is_none() {
                f32::from(data.visible)
            } else {
                fraction(&data)
            };
        let moving = progress != f32::from(data.visible);
        data.observed.show_animation = moving;
        data.observed.animation_state = match (data.visible, moving) {
            (true, true) => ModalAnimationState::Showing,
            (false, true) => ModalAnimationState::Hiding,
            (true, false) => ModalAnimationState::Visible,
            (false, false) => ModalAnimationState::Hidden,
        };
        if let Some(scheduler) = &self.scheduler {
            if !moving || data.timer.is_some_and(|(_, deadline)| deadline <= now) {
                if let Some((id, _)) = data.timer.take() {
                    scheduler.cancel_timer(id);
                }
            }
            if moving && data.timer.is_none() {
                let delay = Duration::from_millis(16);
                data.timer = Some((scheduler.schedule_timeout(delay, || {}), now + delay));
            }
        }
        (data.visible, progress, data.measurements)
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn validation_error(props: &ModalProps) -> Option<&'static str> {
    for size in [&props.width, &props.height] {
        match size {
            ModalSize::Percent(value) if !value.is_finite() || *value < 0.0 => {
                return Some("Invalid Modal: percentage must be finite and nonnegative")
            }
            ModalSize::Viewport(value) if !value.is_finite() || *value < 0.0 => {
                return Some("Invalid Modal: viewport fraction must be finite and nonnegative")
            }
            _ => {}
        }
    }
    let mut ids = std::collections::HashSet::new();
    if props
        .buttons
        .iter()
        .any(|button| button.id.is_empty() || !ids.insert(&button.id))
    {
        return Some("Invalid Modal: button IDs must be nonempty and unique");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_duration_controls_opening_closing_and_zero_duration() {
        for duration in [Duration::ZERO, Duration::from_millis(800)] {
            let scheduler = Arc::new(Scheduler::new());
            let scope = component_scope::ComponentScope::new(scheduler.clone());
            let _binding = scope.enter(true);
            let mut props = ModalProps {
                visible: true,
                ..Default::default()
            };
            let child = LiveModal::new(LiveProps {
                config: props.clone(),
                seed: ModalState::default(),
                role: crate::accessibility::Role::Dialog,
                motion: None,
                on_presented: None,
                escape_closable: true,
            });
            let start = Instant::now();
            let sample =
                |props: &ModalProps, at| child.0.sample_with_duration(props, at, duration).1;
            if duration.is_zero() {
                assert_eq!(sample(&props, start), 1.0);
                props.visible = false;
                assert_eq!(sample(&props, start), 0.0);
            } else {
                assert_eq!(sample(&props, start), 0.0);
                assert_eq!(sample(&props, start + duration / 2), 0.5);
                assert_eq!(sample(&props, start + duration), 1.0);
                props.visible = false;
                assert_eq!(sample(&props, start + duration), 1.0);
                assert_eq!(sample(&props, start + duration + duration / 2), 0.5);
                assert_eq!(sample(&props, start + duration * 2), 0.0);
            }
            assert!(scheduler.next_deadline().is_none());
            scope.close();
        }
    }

    #[test]
    fn transition_reverses_from_current_progress_and_releases_deadline() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _binding = scope.enter(true);
        let mut props = ModalProps {
            visible: true,
            ..Default::default()
        };
        let child = LiveModal::new(LiveProps {
            config: props.clone(),
            seed: ModalState::default(),
            role: crate::accessibility::Role::Dialog,
            motion: None,
            on_presented: None,
            escape_closable: true,
        });
        let start = Instant::now();
        assert_eq!(child.0.sample(&props, start).1, 0.0);
        assert_eq!(
            child.0.sample(&props, start + Duration::from_millis(100)).1,
            0.5
        );
        props.visible = false;
        assert_eq!(
            child.0.sample(&props, start + Duration::from_millis(100)).1,
            0.5
        );
        assert_eq!(
            child.0.sample(&props, start + Duration::from_millis(200)).1,
            0.25
        );
        props.visible = true;
        assert_eq!(
            child.0.sample(&props, start + Duration::from_millis(200)).1,
            0.25
        );
        assert!(scheduler.next_deadline().is_some());
        assert_eq!(
            child.0.sample(&props, start + Duration::from_millis(400)).1,
            1.0
        );
        assert!(scheduler.next_deadline().is_none());
        scope.close();
    }

    #[test]
    fn unmount_cancels_transition_while_rendered_output_is_retained() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _binding = scope.enter(true);
        let props = LiveProps {
            config: ModalProps {
                visible: true,
                ..Default::default()
            },
            seed: ModalState::default(),
            role: crate::accessibility::Role::Dialog,
            motion: None,
            on_presented: None,
            escape_closable: true,
        };
        let mut child = LiveModal::new(props.clone());
        let output = child.render(&props, &());
        assert!(scheduler.next_deadline().is_some());
        child.on_lifecycle(LifecycleEvent::Unmount, &mut ());
        assert!(scheduler.next_deadline().is_none());
        drop(output);
        scope.close();
    }

    #[test]
    fn a_closed_owner_ignores_repeated_button_activation() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let confirmed = calls.clone();
        let closed = calls.clone();
        let props = ModalProps {
            visible: true,
            on_confirm: Some(Arc::new(move || confirmed.lock().unwrap().push("confirm"))),
            on_close: Some(Arc::new(move |_| closed.lock().unwrap().push("close"))),
            ..Default::default()
        };
        let child = LiveModal::new(LiveProps {
            config: props.clone(),
            seed: ModalState::default(),
            role: crate::accessibility::Role::Dialog,
            motion: None,
            on_presented: None,
            escape_closable: true,
        });
        child.0.sample(&props, Instant::now());
        child.0.button(&props, &ModalButton::ok());
        child.0.button(&props, &ModalButton::ok());
        assert_eq!(*calls.lock().unwrap(), ["confirm", "close"]);
    }
}
