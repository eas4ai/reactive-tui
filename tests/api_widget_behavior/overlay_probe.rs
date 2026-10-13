//! Overlay workflows for real screen-reader delivery.
use reactive_tui::{
    builder,
    component::Element,
    widgets::display::{modal::*, popover::*},
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub(super) fn modal(open: &Arc<AtomicBool>) -> Element {
    let launch = open.clone();
    let close = open.clone();
    builder::div()
        .class("w-full h-full")
        .children(vec![
            builder::button()
                .text("Open details")
                .class("w-14 h-1 p-0")
                .on_click(move || launch.store(true, Ordering::SeqCst))
                .build()
                .auto_focus(),
            Element::typed::<Modal>(ModalProps {
                title: Some("Project details".into()),
                visible: open.load(Ordering::SeqCst),
                content: Some(
                    builder::text_input()
                        .value("draft")
                        .build()
                        .with_accessibility_label("Dialog name"),
                ),
                width: ModalSize::Fixed(36),
                height: ModalSize::Fixed(8),
                animation: ModalAnimation::None,
                on_close: Some(Arc::new(move |_| close.store(false, Ordering::SeqCst))),
                ..Default::default()
            }),
        ])
        .build()
}

pub(super) fn popover() -> Element {
    Element::typed::<Popover>(PopoverProps {
        trigger_element: builder::button()
            .text("Show help")
            .class("w-12 h-1 p-0")
            .build()
            .auto_focus(),
        content: builder::text_input()
            .value("topic")
            .build()
            .with_accessibility_label("Help search"),
        animation: PopoverAnimation::None,
        auto_focus: true,
        min_width: Some(36),
        max_width: Some(36),
        min_height: Some(4),
        max_height: Some(4),
        arrow: PopoverArrow {
            enabled: false,
            ..Default::default()
        },
        ..Default::default()
    })
    .with_accessibility_label("Project help")
}
