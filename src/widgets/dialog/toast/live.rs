use super::*;
use crate::{
    accessibility::Role,
    component::{Component, LifecycleEvent, Props},
    reactive::{
        component_scope,
        scheduler::{Scheduler, TimerId},
        ThreadSafeSignal,
    },
    widgets::display::modal::{ModalAnimation, ModalPosition, ModalProps},
};
use std::{sync::Mutex, time::Instant};

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub options: ToastOptions,
    pub class: Option<String>,
    pub bounds: Rect,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) struct LiveToast {
    visible: ThreadSafeSignal<bool>,
    options: Arc<Mutex<ToastOptions>>,
    lifetime: Arc<Lifetime>,
    invalid_duration: bool,
}

struct Deadline {
    timer: Option<TimerId>,
    presented: bool,
    mounted: bool,
}

struct Lifetime {
    scheduler: Option<Arc<Scheduler>>,
    deadline: Mutex<Deadline>,
}

fn close(visible: &ThreadSafeSignal<bool>, options: &Mutex<ToastOptions>) {
    if visible.get() {
        visible.set(false);
        let callback = options.lock().unwrap().on_close.clone();
        if let Some(callback) = callback {
            callback();
        }
    }
}

impl Lifetime {
    fn cancel(&self, unmount: bool) {
        let mut deadline = self.deadline.lock().unwrap();
        deadline.mounted &= !unmount;
        if let (Some(scheduler), Some(timer)) = (&self.scheduler, deadline.timer.take()) {
            scheduler.cancel_timer(timer);
        }
    }
    fn presented(&self, visible: &ThreadSafeSignal<bool>, options: &Arc<Mutex<ToastOptions>>) {
        {
            let mut deadline = self.deadline.lock().unwrap();
            if deadline.presented || !deadline.mounted {
                return;
            }
            deadline.presented = true;
        }
        self.schedule(visible, options);
    }
    fn schedule(&self, visible: &ThreadSafeSignal<bool>, options: &Arc<Mutex<ToastOptions>>) {
        let mut deadline = self.deadline.lock().unwrap();
        if let (Some(scheduler), Some(timer)) = (&self.scheduler, deadline.timer.take()) {
            scheduler.cancel_timer(timer);
        }
        if !deadline.mounted || !deadline.presented || !visible.get() {
            return;
        }
        if let (Some(scheduler), Some(duration)) =
            (&self.scheduler, options.lock().unwrap().duration)
        {
            if Instant::now().checked_add(duration).is_none() {
                return;
            }
            let visible = visible.clone();
            let options = options.clone();
            deadline.timer =
                Some(scheduler.schedule_timeout(duration, move || close(&visible, &options)));
        }
    }
}

impl LiveToast {
    fn schedule(&mut self) {
        let duration = self.options.lock().unwrap().duration;
        self.invalid_duration =
            duration.is_some_and(|duration| Instant::now().checked_add(duration).is_none());
        self.lifetime.schedule(&self.visible, &self.options);
    }
}

impl Component for LiveToast {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        let mut toast = Self {
            visible: ThreadSafeSignal::new(true),
            options: Arc::new(Mutex::new(props.options)),
            lifetime: Arc::new(Lifetime {
                scheduler: component_scope::current().map(|scope| scope.scheduler()),
                deadline: Mutex::new(Deadline {
                    timer: None,
                    presented: false,
                    mounted: true,
                }),
            }),
            invalid_duration: false,
        };
        toast.schedule();
        toast
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        let previous = std::mem::replace(&mut *self.options.lock().unwrap(), props.options.clone());
        if previous.duration != props.options.duration {
            self.schedule();
        }
        true
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        if !self.visible.get() {
            self.lifetime.cancel(false);
        }
        if self.invalid_duration {
            return Element::text("Toast duration exceeds the supported clock range")
                .class("text-red-500");
        }
        let options = &props.options;
        let position = match options.position {
            ToastPosition::TopLeft => ModalPosition::TopLeft,
            ToastPosition::TopCenter => ModalPosition::Top,
            ToastPosition::TopRight => ModalPosition::TopRight,
            ToastPosition::BottomLeft => ModalPosition::BottomLeft,
            ToastPosition::BottomCenter => ModalPosition::Bottom,
            ToastPosition::BottomRight => ModalPosition::BottomRight,
        };
        let (style, role) = match options.toast_type {
            ToastType::Success => ("bg-green-700 text-white", Role::Status),
            ToastType::Error => ("bg-red-700 text-white", Role::Alert),
            ToastType::Warning => ("bg-yellow-700 text-white", Role::Alert),
            _ => ("bg-blue-700 text-white", Role::Status),
        };
        let visible = self.visible.clone();
        let config = self.options.clone();
        let lifetime = self.lifetime.clone();
        let mut modal = ModalProps {
            visible: self.visible.get(),
            content: Some(Element::text(&options.message).class(match role {
                Role::Alert => "aria-live-assertive",
                _ => "aria-live-polite",
            })),
            position,
            auto_focus: false,
            focus_trap: false,
            backdrop_style: None,
            closable: options.closable,
            keyboard_navigation: options.closable,
            backdrop_clickable: false,
            modal_style: Some(format!("{style} {}", props.class.as_deref().unwrap_or(""))),
            animation: ModalAnimation::None,
            z_index: 2000,
            on_close: Some(Arc::new(move |_| {
                lifetime.cancel(false);
                close(&visible, &config);
            })),
            ..Default::default()
        };
        super::super::frame::apply_bounds(&mut modal, props.bounds);
        let lifetime = self.lifetime.clone();
        let visible = self.visible.clone();
        let options = self.options.clone();
        super::super::frame::modal_with_presented_callback(
            modal,
            props.options.closable,
            role,
            Some(Arc::new(move || lifetime.presented(&visible, &options))),
        )
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut ()) {
        if event == LifecycleEvent::Unmount {
            self.lifetime.cancel(true);
        }
    }
}
impl Drop for LiveToast {
    fn drop(&mut self) {
        self.lifetime.cancel(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmount_and_manual_close_cancel_toast_deadlines_with_retained_output() {
        for unmount in [false, true] {
            let scheduler = Arc::new(Scheduler::new());
            let scope = component_scope::ComponentScope::new(scheduler.clone());
            let _binding = scope.enter(true);
            let props = LiveProps {
                options: ToastOptions::default(),
                class: None,
                bounds: Rect::default(),
            };
            let mut toast = LiveToast::new(props.clone());
            let output = toast.render(&props, &());
            assert!(scheduler.next_deadline().is_none());
            toast.lifetime.presented(&toast.visible, &toast.options);
            assert!(scheduler.next_deadline().is_some());
            if unmount {
                toast.on_lifecycle(LifecycleEvent::Unmount, &mut ());
            } else {
                close(&toast.visible, &toast.options);
                toast.render(&props, &());
            }
            toast.lifetime.presented(&toast.visible, &toast.options);
            assert!(scheduler.next_deadline().is_none());
            drop(output);
            scope.close();
        }
    }

    #[test]
    fn unmount_before_presentation_cannot_start_a_retained_deadline() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _binding = scope.enter(true);
        let props = LiveProps {
            options: ToastOptions::default(),
            class: None,
            bounds: Rect::default(),
        };
        let mut toast = LiveToast::new(props.clone());
        let output = toast.render(&props, &());
        toast.on_lifecycle(LifecycleEvent::Unmount, &mut ());
        toast.lifetime.presented(&toast.visible, &toast.options);
        assert!(scheduler.next_deadline().is_none());
        drop(output);
        scope.close();
    }

    #[test]
    fn presented_toast_preserves_duration_updates_and_does_not_restart_on_redraw() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _binding = scope.enter(true);
        let mut props = LiveProps {
            options: ToastOptions {
                duration: None,
                ..Default::default()
            },
            class: None,
            bounds: Rect::default(),
        };
        let mut toast = LiveToast::new(props.clone());
        toast.lifetime.presented(&toast.visible, &toast.options);
        assert!(scheduler.next_deadline().is_none());
        props.options.duration = Some(Duration::from_secs(30));
        toast.update(&props, &mut ());
        let initial = scheduler.next_deadline().unwrap();
        toast.lifetime.presented(&toast.visible, &toast.options);
        assert_eq!(scheduler.next_deadline(), Some(initial));
        props.options.duration = Some(Duration::from_secs(60));
        toast.update(&props, &mut ());
        assert!(scheduler.next_deadline().unwrap() > initial);
        props.options.duration = None;
        toast.update(&props, &mut ());
        assert!(scheduler.next_deadline().is_none());
        props.options.duration = Some(Duration::ZERO);
        toast.update(&props, &mut ());
        assert!(scheduler.run_ready_timers());
        assert!(!toast.visible.get());
        toast.lifetime.presented(&toast.visible, &toast.options);
        assert!(scheduler.next_deadline().is_none());
        scope.close();
    }
}
