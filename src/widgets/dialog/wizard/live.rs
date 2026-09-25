use super::*;
use crate::{
    component::{Component, LayoutType, Props},
    reactive::ThreadSafeSignal,
    widgets::display::{
        modal::{ModalButton, ModalButtonAction, ModalProps},
        progress_bar::{ProgressBar, ProgressBarProps},
    },
};
use std::collections::HashSet;

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub id: DialogId,
    pub options: WizardDialogOptions,
    pub data: HashMap<String, String>,
    pub initial_step: usize,
    pub cancelable: bool,
    pub class: Option<String>,
    pub bounds: Rect,
    pub theme: DialogTheme,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
struct Runtime {
    current: ThreadSafeSignal<Option<String>>,
    visited: ThreadSafeSignal<HashSet<String>>,
    visible: ThreadSafeSignal<bool>,
    errors: ThreadSafeSignal<Vec<String>>,
}
impl Runtime {
    fn select(&self, step: &WizardStep) {
        self.current.set(Some(step.id.clone()));
        self.visited.update(|visited| {
            visited.insert(step.id.clone());
        });
        self.errors.set(Vec::new());
    }
    fn cancel(&self, props: &LiveProps) {
        if props.cancelable && self.visible.get() {
            self.visible.set(false);
            if let Some(callback) = &props.options.on_cancel {
                callback();
            }
        }
    }
    fn activate(&self, props: &LiveProps, action: &str) {
        if !self.visible.get() {
            return;
        }
        if action == "cancel" {
            self.cancel(props);
            return;
        }
        if configuration_error(&props.options).is_some() {
            return;
        }
        let Some(index) = props
            .options
            .steps
            .iter()
            .position(|step| Some(&step.id) == self.current.get().as_ref())
        else {
            return;
        };
        if action == "back" {
            if props.options.allow_back && index > 0 {
                self.select(&props.options.steps[index - 1]);
            }
            return;
        }
        let skip = action == "skip" && props.options.steps[index].can_skip;
        if action != "next" && !skip {
            return;
        }
        if !skip {
            let result = validate_step(&props.options, index, &props.data);
            if !result.valid {
                let mut errors: Vec<_> = result.message.into_iter().collect();
                errors.sort();
                if errors.is_empty() {
                    errors.push("Complete this step before continuing".into());
                }
                self.errors.set(errors);
                return;
            }
        }
        if let Some(step) = props.options.steps.get(index + 1) {
            self.select(step);
        } else if props
            .options
            .on_complete
            .as_ref()
            .is_none_or(|callback| callback(&props.data))
        {
            self.visible.set(false);
        }
    }
}

pub(super) struct LiveWizard {
    runtime: Runtime,
    id: DialogId,
    seed: usize,
}
impl Component for LiveWizard {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        let current = props
            .options
            .steps
            .get(props.initial_step)
            .map(|step| step.id.clone());
        Self {
            runtime: Runtime {
                visited: ThreadSafeSignal::new(current.iter().cloned().collect()),
                current: ThreadSafeSignal::new(current),
                visible: ThreadSafeSignal::new(true),
                errors: ThreadSafeSignal::new(Vec::new()),
            },
            id: props.id,
            seed: props.initial_step,
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        if self.id != props.id || self.seed != props.initial_step {
            *self = Self::new(props.clone());
        } else if !props
            .options
            .steps
            .iter()
            .any(|step| Some(&step.id) == self.runtime.current.get().as_ref())
        {
            if let Some(step) = props.options.steps.get(props.initial_step) {
                self.runtime.select(step);
            }
        }
        let visited = self.runtime.visited.get();
        let retained: HashSet<_> = props
            .options
            .steps
            .iter()
            .filter(|step| visited.contains(&step.id))
            .map(|step| step.id.clone())
            .collect();
        if retained != visited {
            self.runtime.visited.set(retained);
        }
        true
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        let current = self.runtime.current.get();
        let index = props
            .options
            .steps
            .iter()
            .position(|step| Some(&step.id) == current.as_ref());
        let error = configuration_error(&props.options).or_else(|| {
            index
                .is_none()
                .then(|| "Wizard step is out of range".into())
        });
        let mut content = Vec::new();
        if let Some(error) = &error {
            content.push(Element::text(error));
        }
        if let Some(index) = index.filter(|_| error.is_none()) {
            if props.options.show_progress {
                content.push(
                    Element::text(format!(
                        "Step {} of {}",
                        index + 1,
                        props.options.steps.len()
                    ))
                    .with_key("progress-label"),
                );
                content.push(
                    ProgressBar::with_props(ProgressBarProps {
                        value: (index + 1) as f64,
                        max_value: props.options.steps.len() as f64,
                        show_percentage: false,
                        ..Default::default()
                    })
                    .with_key("progress"),
                );
            }
            content.push(Element::text(&props.options.steps[index].title).with_key("step-title"));
            let visited = self.runtime.visited.get();
            content.push(
                Element::layout(LayoutType::Flex)
                    .with_key("panels")
                    .with_class("flex-col min-w-0")
                    .with_children(
                        props
                            .options
                            .steps
                            .iter()
                            .filter(|step| visited.contains(&step.id))
                            .map(|step| {
                                Element::layout(LayoutType::Flex)
                                    .with_key(&step.id)
                                    .with_class(if Some(&step.id) == current.as_ref() {
                                        "flex-col min-w-0"
                                    } else {
                                        "hidden"
                                    })
                                    .with_child(step.content.clone())
                            })
                            .collect(),
                    ),
            );
        }
        for (index, error) in self.runtime.errors.get().iter().enumerate() {
            content.push(
                Element::text(error)
                    .with_key(format!("error-{index}"))
                    .with_class("text-red-500 aria-live-assertive")
                    .with_accessibility(crate::accessibility::Node::new(
                        crate::accessibility::Role::Alert,
                    )),
            );
        }
        let button = |id: &str, label: &str| ModalButton {
            id: id.into(),
            label: label.into(),
            action: ModalButtonAction::Custom(id.into()),
            disabled: false,
            autofocus: id == "next",
            style: None,
        };
        let mut buttons = Vec::new();
        if props.cancelable {
            buttons.push(button("cancel", "Cancel"));
        }
        if let Some(index) = index.filter(|_| error.is_none()) {
            if props.options.allow_back && index > 0 {
                buttons.push(button("back", "Back"));
            }
            if props.options.steps[index].can_skip {
                buttons.push(button("skip", "Skip"));
            }
            buttons.push(button(
                "next",
                if index + 1 == props.options.steps.len() {
                    "Finish"
                } else {
                    "Next"
                },
            ));
        }
        let runtime = self.runtime.clone();
        let config = props.clone();
        let closed = self.runtime.clone();
        let close_config = props.clone();
        let mut modal = ModalProps {
            visible: self.runtime.visible.get(),
            title: Some(props.options.title.clone()),
            content: Some(
                Element::layout(LayoutType::Flex)
                    .with_class("flex-col min-w-0")
                    .with_children(content),
            ),
            buttons,
            closable: props.cancelable,
            keyboard_navigation: true,
            backdrop_clickable: false,
            focus_trap: true,
            modal_style: Some(format!(
                "{} {} {}",
                props.theme.dialog_bg,
                props.theme.border_style,
                props.class.as_deref().unwrap_or("")
            )),
            header_style: Some(props.theme.title_style.clone()),
            on_button_click: Some(Arc::new(move |id| runtime.activate(&config, &id))),
            on_close: Some(Arc::new(move |_| closed.cancel(&close_config))),
            ..Default::default()
        };
        super::super::frame::apply_bounds(&mut modal, props.bounds);
        super::super::frame::modal(modal, props.cancelable, crate::accessibility::Role::Dialog)
    }
}
