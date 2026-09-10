use super::*;
use crate::{
    accessibility::Role,
    component::{Component, LifecycleEvent, Props},
    reactive::{
        component_scope,
        scheduler::{Scheduler, TimerId},
        ThreadSafeSignal,
    },
    widgets::display::modal::{Modal, ModalAnimation, ModalPosition, ModalProps},
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
    scheduler: Option<Arc<Scheduler>>,
    timer: Mutex<Option<TimerId>>,
    invalid_duration: bool,
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

impl LiveToast {
    fn cancel(&self) {
        if let (Some(scheduler), Some(timer)) = (&self.scheduler, self.timer.lock().unwrap().take())
        {
            scheduler.cancel_timer(timer);
        }
    }
    fn schedule(&mut self) {
        self.cancel();
        let duration = self.options.lock().unwrap().duration;
        self.invalid_duration =
            duration.is_some_and(|duration| Instant::now().checked_add(duration).is_none());
        if self.invalid_duration || !self.visible.get() {
            return;
        }
        if let (Some(scheduler), Some(duration)) = (&self.scheduler, duration) {
            let visible = self.visible.clone();
            let options = self.options.clone();
            *self.timer.lock().unwrap() =
                Some(scheduler.schedule_timeout(duration, move || close(&visible, &options)));
        }
    }
}

impl Component for LiveToast {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        let mut toast = Self {
            visible: ThreadSafeSignal::new(true),
            options: Arc::new(Mutex::new(props.options)),
            scheduler: component_scope::current().map(|scope| scope.scheduler()),
            timer: Mutex::new(None),
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
            self.cancel();
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
            on_close: Some(Arc::new(move |_| close(&visible, &config))),
            ..Default::default()
        };
        super::super::frame::apply_bounds(&mut modal, props.bounds);
        Modal::with_role(modal, role)
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut ()) {
        if event == LifecycleEvent::Unmount {
            self.cancel();
        }
    }
}
impl Drop for LiveToast {
    fn drop(&mut self) {
        self.cancel();
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
            assert!(scheduler.next_deadline().is_some());
            if unmount {
                toast.on_lifecycle(LifecycleEvent::Unmount, &mut ());
            } else {
                close(&toast.visible, &toast.options);
                toast.render(&props, &());
            }
            assert!(scheduler.next_deadline().is_none());
            drop(output);
            scope.close();
        }
    }
}
