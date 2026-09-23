use super::*;
use crate::{
    component::{Component, Props},
    reactive::ThreadSafeSignal,
    widgets::display::{
        modal::{ModalButton, ModalProps, ModalSize},
        progress_bar::ProgressBarBuilder,
    },
};
use std::time::Instant;

#[derive(Clone)]
pub(super) struct LiveProps {
    pub id: DialogId,
    pub options: ProgressDialogOptions,
    pub progress: f32,
    pub indeterminate: bool,
    pub class: Option<String>,
    pub bounds: Rect,
    pub theme: DialogTheme,
}
impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.options == other.options
            && self.progress.to_bits() == other.progress.to_bits()
            && self.indeterminate == other.indeterminate
            && self.class == other.class
            && self.bounds == other.bounds
            && self.theme == other.theme
    }
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) struct LiveProgress {
    visible: ThreadSafeSignal<bool>,
    id: DialogId,
    started: Instant,
}
impl Component for LiveProgress {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        Self {
            visible: ThreadSafeSignal::new(true),
            id: props.id,
            started: Instant::now(),
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        if self.id != props.id {
            self.id = props.id;
            self.started = Instant::now();
            self.visible.set(true);
        }
        true
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        let mut content = vec![
            Element::text(&props.options.message),
            Element::typed::<crate::widgets::display::ProgressBar>(
                ProgressBarBuilder::new()
                    .value(f64::from(props.progress))
                    .max_value(1.0)
                    .show_percentage(props.options.show_percentage && !props.indeterminate)
                    .indeterminate(props.indeterminate)
                    .animated(props.indeterminate)
                    .style("w-full")
                    .build(),
            ),
        ];
        if props.options.show_time_remaining {
            let estimate =
                if props.progress.is_finite() && props.progress > 0.0 && !props.indeterminate {
                    let seconds = self.started.elapsed().as_secs_f64()
                        * f64::from(1.0 - props.progress.clamp(0.0, 1.0))
                        / f64::from(props.progress);
                    format!("Remaining: {:.0}s", seconds.ceil())
                } else {
                    "Remaining: --".to_string()
                };
            content.push(Element::text(estimate));
        }
        let mut cancel = ModalButton::cancel();
        cancel.autofocus = true;
        let visible = self.visible.clone();
        let options = props.options.clone();
        let mut modal = ModalProps {
            visible: self.visible.get(),
            title: Some(props.options.title.clone()),
            content: Some(
                crate::builder::div()
                    .class("flex-col")
                    .children(content)
                    .build(),
            ),
            width: ModalSize::Fixed(40),
            closable: props.options.cancellable,
            keyboard_navigation: props.options.cancellable,
            backdrop_clickable: false,
            buttons: if props.options.cancellable {
                vec![cancel]
            } else {
                vec![]
            },
            modal_style: Some(format!(
                "{} {} {}",
                props.theme.dialog_bg,
                props.theme.border_style,
                props.class.as_deref().unwrap_or("")
            )),
            header_style: Some(props.theme.title_style.clone()),
            backdrop_style: Some(props.theme.backdrop_color.clone()),
            on_close: Some(Arc::new(move |_| {
                if visible.get() {
                    visible.set(false);
                    if let Some(callback) = &options.on_cancel {
                        callback();
                    }
                }
            })),
            ..Default::default()
        };
        super::super::frame::apply_bounds(&mut modal, props.bounds);
        super::super::frame::modal(
            modal,
            props.options.cancellable,
            crate::accessibility::Role::Dialog,
        )
    }
}
