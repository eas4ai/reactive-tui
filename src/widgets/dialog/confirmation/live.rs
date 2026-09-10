use super::*;
use crate::{
    component::{Component, LayoutInfo, Props},
    event::router::EventResult,
    reactive::ThreadSafeSignal,
    widgets::display::modal::{
        ModalAnimation, ModalButton, ModalButtonAction, ModalProps, ModalSize,
    },
};

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub id: DialogId,
    pub options: ConfirmationDialogOptions,
    pub bounds: Rect,
    pub theme: DialogTheme,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) struct LiveConfirmation {
    visible: ThreadSafeSignal<bool>,
    id: DialogId,
    layout: Option<LayoutInfo>,
}

fn finish(
    visible: &ThreadSafeSignal<bool>,
    options: &ConfirmationDialogOptions,
    result: DialogResult,
) {
    if visible.get() {
        visible.set(false);
        if let Some(callback) = &options.on_close {
            callback(result);
        }
    }
}

fn activate(visible: &ThreadSafeSignal<bool>, options: &ConfirmationDialogOptions, id: &str) {
    if !visible.get() {
        return;
    }
    let Some(button) = options
        .buttons
        .entries()
        .into_iter()
        .find(|button| button.enabled && button.id == id)
    else {
        return;
    };
    if options
        .on_button_click
        .as_ref()
        .is_some_and(|callback| !callback(id))
    {
        return;
    }
    finish(visible, options, button.result());
}

impl Component for LiveConfirmation {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        Self {
            visible: ThreadSafeSignal::new(true),
            id: props.id,
            layout: None,
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        if self.id != props.id {
            self.id = props.id;
            self.visible.set(true);
        }
        true
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut ()) -> bool {
        let changed = self.layout != Some(layout);
        self.layout = Some(layout);
        changed
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        let options = &props.options;
        let entries = options.buttons.entries();
        let focused = options
            .default_button
            .as_deref()
            .filter(|id| {
                entries
                    .iter()
                    .any(|button| button.enabled && button.id == *id)
            })
            .or_else(|| {
                entries
                    .iter()
                    .find(|button| button.enabled && button.is_default)
                    .map(|button| button.id.as_str())
            })
            .or_else(|| {
                entries
                    .iter()
                    .find(|button| button.enabled)
                    .map(|button| button.id.as_str())
            });
        let buttons = entries
            .iter()
            .map(|button| ModalButton {
                id: button.id.clone(),
                label: button.text.clone(),
                action: ModalButtonAction::Custom(button.id.clone()),
                disabled: !button.enabled,
                autofocus: focused == Some(button.id.as_str()),
                style: Some(format!(
                    "{} {} {}",
                    props
                        .theme
                        .button_styles
                        .get(&button.variant.to_string())
                        .map_or("", String::as_str),
                    options.css_classes.get("button").map_or("", String::as_str),
                    button.css_class.as_deref().unwrap_or("")
                )),
            })
            .collect();
        let icon = match options.icon.as_ref() {
            Some(ConfirmationIcon::Question) => "?",
            Some(ConfirmationIcon::Warning) => "!",
            Some(ConfirmationIcon::Error) => "×",
            Some(ConfirmationIcon::Info) => "i",
            Some(ConfirmationIcon::Success) => "✓",
            Some(ConfirmationIcon::Custom(icon)) => icon,
            _ => "",
        };
        let mut content = vec![Element::text(
            format!("{icon} {}", options.message).trim_start(),
        )];
        if let Some(description) = &options.description {
            content.push(Element::text(description));
        }
        let visible = self.visible.clone();
        let config = options.clone();
        let closed = self.visible.clone();
        let close_config = options.clone();
        let position = match super::super::frame::position(&options.position, self.layout) {
            Ok(position) => position,
            Err(error) => return Element::text(error),
        };
        let mut modal = ModalProps {
            visible: self.visible.get(),
            title: Some(options.title.clone()),
            content: Some(
                crate::builder::div()
                    .class("flex-col")
                    .children(content)
                    .build(),
            ),
            width: options.size.map_or(ModalSize::Auto, |size| {
                ModalSize::Fixed(size.width.min(u16::MAX as usize) as u16)
            }),
            height: options.size.map_or(ModalSize::Auto, |size| {
                ModalSize::Fixed(size.height.min(u16::MAX as usize) as u16)
            }),
            position,
            buttons,
            closable: true,
            keyboard_navigation: true,
            backdrop_clickable: options.backdrop_closable,
            focus_trap: options.modal,
            backdrop_style: options.modal.then(|| props.theme.backdrop_color.clone()),
            modal_style: Some(format!(
                "{} {} {}",
                props.theme.dialog_bg,
                props.theme.border_style,
                options.css_classes.get("dialog").map_or("", String::as_str)
            )),
            header_style: Some(props.theme.title_style.clone()),
            content_style: options.css_classes.get("content").cloned(),
            on_button_click: Some(Arc::new(move |id| activate(&visible, &config, &id))),
            on_close: Some(Arc::new(move |_| {
                finish(&closed, &close_config, DialogResult::Cancelled)
            })),
            animation: ModalAnimation::Fade,
            ..Default::default()
        };
        super::super::frame::apply_bounds(&mut modal, props.bounds);
        let mut element = super::super::frame::modal(
            modal,
            options.escape_closable,
            crate::accessibility::Role::Dialog,
        );
        let visible = self.visible.clone();
        let config = options.clone();
        element.metadata.capture_events.push(Arc::new(move |event| {
            let Event::Key(key) = event else {
                return EventResult::Ignored;
            };
            if !visible.get()
                || key.kind == crate::event::types::KeyEventKind::Release
                || key.modifiers.ctrl
                || key.modifiers.alt
                || key.modifiers.meta
                || matches!(
                    key.code,
                    KeyCode::Enter | KeyCode::Escape | KeyCode::Tab | KeyCode::BackTab
                )
            {
                return EventResult::Ignored;
            }
            if let Some(button) = config
                .buttons
                .entries()
                .iter()
                .find(|button| button.enabled && button.matches_shortcut(&key.code))
            {
                activate(&visible, &config, &button.id);
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }));
        element
    }
}
