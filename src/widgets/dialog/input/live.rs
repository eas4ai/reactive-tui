use super::*;
use crate::{
    accessibility::{Node, Role},
    component::{Component, LayoutInfo, LifecycleEvent, Props},
    event::{router::EventResult, types::FocusEventKind},
    reactive::{
        component_scope,
        scheduler::{Scheduler, TimerId},
        ThreadSafeSignal,
    },
    widgets::{
        display::modal::{ModalButton, ModalButtonAction, ModalProps, ModalSize},
        input::InputMode,
        TextInput, TextInputProps,
    },
};
use std::{sync::Mutex, time::Instant};

mod remote;

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub id: DialogId,
    pub options: InputDialogOptions,
    pub value: String,
    pub bounds: Rect,
    pub theme: DialogTheme,
    pub revision: u64,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct Runtime {
    activity: super::super::frame::Activity,
    options: Mutex<Arc<InputDialogOptions>>,
    value: ThreadSafeSignal<String>,
    error: ThreadSafeSignal<Option<String>>,
    warnings: ThreadSafeSignal<Vec<String>>,
    visible: ThreadSafeSignal<bool>,
    scheduler: Option<Arc<Scheduler>>,
    timer: Mutex<Option<TimerId>>,
    remote: Mutex<remote::State>,
    remote_timer: Mutex<Option<TimerId>>,
    remote_pending: ThreadSafeSignal<bool>,
    content_width: ThreadSafeSignal<Option<usize>>,
}
impl Runtime {
    fn options(&self) -> Arc<InputDialogOptions> {
        self.options.lock().unwrap().clone()
    }
    fn cancel(&self) {
        self.cancel_debounce();
        self.cancel_remote();
    }
    fn cancel_debounce(&self) -> bool {
        let timer = self.timer.lock().unwrap().take();
        let pending = timer.is_some();
        if let (Some(scheduler), Some(timer)) = (&self.scheduler, timer) {
            scheduler.cancel_timer(timer);
        }
        pending
    }
    fn validate_local(&self) -> bool {
        let result = validation::validate(&self.options(), &self.value.get());
        self.error.set(result.message);
        self.warnings.set(result.warnings);
        result.valid
    }
    fn validate(self: &Arc<Self>) -> bool {
        if !self.validate_local() {
            self.cancel_remote();
            return false;
        }
        self.remote_validation(false)
    }
    fn changed(self: &Arc<Self>, value: String) {
        if !self.visible.get() || !self.activity.active() {
            return;
        }
        self.value.set(value.clone());
        self.error.set(None);
        self.warnings.set(Vec::new());
        self.cancel();
        let options = self.options();
        if let Some(callback) = &options.on_change {
            callback(&value);
        }
        if let Some(validation) = options
            .validation
            .as_ref()
            .filter(|validation| validation.validate_on_change)
        {
            self.schedule_validation(validation.debounce_delay);
        }
    }
    fn schedule_validation(self: &Arc<Self>, delay: Duration) {
        self.cancel_debounce();
        if Instant::now().checked_add(delay).is_none() {
            self.error.set(Some(
                "Validation delay exceeds the supported clock range".into(),
            ));
        } else if let Some(scheduler) = &self.scheduler {
            let owner = Arc::downgrade(self);
            *self.timer.lock().unwrap() = Some(scheduler.schedule_timeout(delay, move || {
                if let Some(owner) = owner.upgrade() {
                    owner.timer.lock().unwrap().take();
                    if owner.visible.get() && owner.activity.active() {
                        owner.validate();
                    }
                }
            }));
        } else {
            self.validate();
        }
    }
    fn finish(&self, result: DialogResult) {
        if !self.visible.get() || !self.activity.active() {
            return;
        }
        self.cancel();
        self.visible.set(false);
        let options = self.options();
        if let Some(callback) = &options.on_close {
            callback(result);
        }
    }
    fn submit(self: &Arc<Self>) {
        if !self.visible.get() || !self.activity.active() {
            return;
        }
        self.cancel_debounce();
        if !self.validate_local() {
            self.cancel_remote();
            return;
        }
        if !self.remote_validation(true) {
            return;
        }
        self.complete_submission();
    }
    fn complete_submission(&self) {
        if !self.visible.get() || !self.activity.active() {
            return;
        }
        let options = self.options();
        let value = self.value.get();
        if options
            .on_submit
            .as_ref()
            .is_some_and(|callback| !callback(&value))
        {
            self.cancel_remote();
            return;
        }
        self.finish(DialogResult::Confirmed(Some(value)));
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.cancel();
    }
}

pub(super) struct LiveInput {
    runtime: Arc<Runtime>,
    seed: String,
    revision: u64,
    id: DialogId,
    layout: Option<LayoutInfo>,
}
impl Component for LiveInput {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        Self {
            runtime: Arc::new(Runtime {
                activity: super::super::frame::activity(),
                options: Mutex::new(Arc::new(props.options)),
                value: ThreadSafeSignal::new(props.value.clone()),
                error: ThreadSafeSignal::new(None),
                warnings: ThreadSafeSignal::new(Vec::new()),
                visible: ThreadSafeSignal::new(true),
                scheduler: component_scope::current().map(|scope| scope.scheduler()),
                timer: Mutex::new(None),
                remote: Mutex::new(remote::State::default()),
                remote_timer: Mutex::new(None),
                remote_pending: ThreadSafeSignal::new(false),
                content_width: ThreadSafeSignal::new(None),
            }),
            seed: props.value,
            revision: props.revision,
            id: props.id,
            layout: None,
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        let previous = self.runtime.options();
        let endpoint_changed = !remote::same_endpoint(&previous, &props.options);
        let debounce_changed = previous
            .validation
            .as_ref()
            .map(|validation| (validation.validate_on_change, validation.debounce_delay))
            != props
                .options
                .validation
                .as_ref()
                .map(|validation| (validation.validate_on_change, validation.debounce_delay));
        let rules_changed = previous.input.input_type != props.options.input.input_type
            || previous.input.required != props.options.input.required
            || previous.input.max_length != props.options.input.max_length
            || previous.input.mask != props.options.input.mask
            || previous
                .validation
                .as_ref()
                .map(|validation| &validation.rules)
                != props
                    .options
                    .validation
                    .as_ref()
                    .map(|validation| &validation.rules);
        let reschedule = (endpoint_changed || debounce_changed) && self.runtime.cancel_debounce();
        if endpoint_changed {
            self.runtime.cancel_remote();
        }
        if endpoint_changed || rules_changed {
            self.runtime.error.set(None);
            self.runtime.warnings.set(Vec::new());
        }
        *self.runtime.options.lock().unwrap() = Arc::new(props.options.clone());
        if self.seed != props.value || self.id != props.id || self.revision != props.revision {
            self.runtime.cancel();
            self.runtime.value.set(props.value.clone());
            self.runtime.error.set(None);
            self.runtime.warnings.set(Vec::new());
            self.seed = props.value.clone();
            self.revision = props.revision;
        } else if reschedule {
            if let Some(validation) = props
                .options
                .validation
                .as_ref()
                .filter(|validation| validation.validate_on_change)
            {
                self.runtime.schedule_validation(validation.debounce_delay);
            }
        }
        if self.id != props.id {
            self.id = props.id;
            self.runtime.visible.set(true);
        }
        true
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut ()) -> bool {
        let changed = self.layout != Some(layout);
        self.layout = Some(layout);
        changed
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        if !self.runtime.activity.active() {
            self.runtime.cancel();
        }
        let options = &props.options;
        let owner = self.runtime.clone();
        let mode = if options.input.input_type == InputType::Password {
            InputMode::Password
        } else if options.input.multiline {
            InputMode::MultiLine {
                height: options.input.rows.unwrap_or(3).min(u16::MAX as usize) as u16,
            }
        } else if options.input.input_type == InputType::Number {
            InputMode::Numeric
        } else {
            InputMode::SingleLine
        };
        let mut input = Element::typed_with::<TextInput>(
            TextInputProps {
                value: self.runtime.value.get(),
                placeholder: options.input.placeholder.clone(),
                max_length: options.input.max_length,
                width: None,
                mode,
                disabled: options.input.attribute_enabled("disabled"),
                ..Default::default()
            },
            move |props| {
                let changed = owner.clone();
                let submitted = owner.clone();
                let readonly = owner.clone();
                TextInput::new(props)
                    .with_read_only(move || readonly.options().input.attribute_enabled("readonly"))
                    .with_on_change(move |value| changed.changed(value))
                    .with_on_submit(move |_| submitted.submit())
            },
        )
        .auto_focus()
        .with_key("input")
        .with_accessibility_label(
            options
                .input
                .attributes
                .get("aria-label")
                .unwrap_or(&options.prompt),
        )
        .class(
            options
                .css_classes
                .get("input")
                .map_or("w-full", String::as_str),
        );
        let owner = self.runtime.clone();
        input.metadata.capture_events.push(Arc::new(move |event| {
            if matches!(event, Event::Focus(event) if event.kind == FocusEventKind::Lost)
                && owner
                    .options()
                    .validation
                    .as_ref()
                    .is_some_and(|validation| validation.validate_on_blur)
            {
                owner.cancel();
                owner.validate();
            }
            EventResult::Ignored
        }));
        let mut content = vec![Element::text(&options.prompt), input];
        if let Some(mask) = &options.input.mask {
            content
                .push(Element::text(format!("Format: {mask}")).class("whitespace-pre-wrap w-full"));
        }
        if options.input.show_count {
            let count = self.runtime.value.get().graphemes(true).count();
            content.push(Element::text(options.input.max_length.map_or_else(
                || format!("{count} characters"),
                |maximum| format!("{count}/{maximum} characters"),
            )));
        }
        if let Some(error) = self.runtime.error.get() {
            content.push(
                Element::text(error)
                    .class("text-red-500 whitespace-pre-wrap w-full aria-live-assertive")
                    .with_accessibility(Node::new(Role::Alert)),
            );
        }
        for warning in self.runtime.warnings.get() {
            content.push(
                Element::text(warning)
                    .class("text-yellow-500 whitespace-pre-wrap w-full aria-live-polite")
                    .with_accessibility(Node::new(Role::Status)),
            );
        }
        if self.runtime.remote_pending.get() {
            content
                .push(Element::text("Validating...").with_accessibility(Node::new(Role::Status)));
        }
        let position = match super::super::frame::position(&options.position, self.layout) {
            Ok(position) => position,
            Err(error) => return Element::text(error),
        };
        let mut content = crate::builder::div().class("flex-col").children(content);
        if let Some(width) = self.runtime.content_width.get() {
            content =
                content.styles(crate::layout::style::StyleBuilder::new().width_px(width as f32));
        }
        let mut content = content.build();
        let measured = self.runtime.clone();
        content.metadata.layout.push(Arc::new(move |layout| {
            let width = Some(layout.clip.width.max(1.0).floor() as usize);
            let changed = measured.content_width.get() != width;
            if changed {
                measured.content_width.set(width);
            }
            changed
        }));
        let submitted = self.runtime.clone();
        let closed = self.runtime.clone();
        let mut ok = ModalButton::ok();
        ok.autofocus = false;
        ok.action = ModalButtonAction::Custom("ok".to_string());
        let mut cancel = ModalButton::cancel();
        cancel.action = ModalButtonAction::Custom("cancel".to_string());
        let mut modal = ModalProps {
            visible: self.runtime.visible.get(),
            title: Some(options.title.clone()),
            content: Some(content),
            buttons: vec![cancel, ok],
            position,
            width: options.size.map_or(ModalSize::Fixed(40), |size| {
                ModalSize::Fixed(size.width.min(u16::MAX as usize) as u16)
            }),
            height: options.size.map_or(ModalSize::Auto, |size| {
                ModalSize::Fixed(size.height.min(u16::MAX as usize) as u16)
            }),
            focus_trap: options.modal,
            backdrop_clickable: options.backdrop_closable,
            keyboard_navigation: true,
            backdrop_style: options.modal.then(|| props.theme.backdrop_color.clone()),
            modal_style: Some(format!(
                "{} {} {}",
                props.theme.dialog_bg,
                props.theme.border_style,
                options.css_classes.get("dialog").map_or("", String::as_str)
            )),
            header_style: Some(props.theme.title_style.clone()),
            content_style: options.css_classes.get("content").cloned(),
            on_button_click: Some(Arc::new(move |id| {
                if id == "ok" {
                    submitted.submit();
                } else {
                    submitted.finish(DialogResult::Cancelled);
                }
            })),
            on_close: Some(Arc::new(move |_| closed.finish(DialogResult::Cancelled))),
            ..Default::default()
        };
        super::super::frame::apply_bounds(&mut modal, props.bounds);
        super::super::frame::modal(
            modal,
            options.escape_closable,
            crate::accessibility::Role::Dialog,
        )
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut ()) {
        if event == LifecycleEvent::Unmount {
            self.runtime.visible.set(false);
            self.runtime.cancel();
        }
    }
}
