//! Real App menu fixtures for the Orca workflow.
use reactive_tui::{
    builder,
    component::{Component, Element},
    widgets::menu::{
        ContextMenu, ContextMenuProps, ContextMenuState, DialogMenu, DialogMenuProps,
        DialogMenuState, DialogMenuType, MenuBar, MenuBarProps, MenuItem, MenuStyle, PopupMenu,
        PopupMenuProps, PopupPlacement,
    },
};
use std::sync::{Arc, Mutex};

pub fn render(stage: usize, calls: &Arc<Mutex<Vec<String>>>) -> Element {
    let changed = calls.clone();
    let activated = calls.clone();
    let items = vec![
        MenuItem::checkbox("notify", "Enable notifications", false, move |value| {
            changed
                .lock()
                .unwrap()
                .push(format!("{stage}:checked:{value}"))
        }),
        MenuItem::new("disabled", "Unavailable item").enabled(false),
        MenuItem::submenu(
            "recent",
            "Recent menu",
            vec![MenuItem::action("document", "Recent document", move || {
                activated.lock().unwrap().push(format!("{stage}:document"))
            })],
        ),
    ];
    let style = MenuStyle {
        padding: 0,
        min_width: 0,
        ..Default::default()
    };
    let menu = match stage {
        1 => Element::typed::<MenuBar>(MenuBarProps {
            items: vec![MenuItem::submenu("file", "Menu file", items)],
            style,
            ..Default::default()
        })
        .auto_focus(),
        2 => Element::typed::<PopupMenu>(PopupMenuProps {
            visible: true,
            items,
            style,
            auto_close: false,
            placement: PopupPlacement::Position { x: 1, y: 2 },
            width: Some(27),
            show_border: false,
            show_shadow: false,
            ..Default::default()
        }),
        3 => {
            let props = ContextMenuProps {
                items,
                style,
                width: Some(27),
                show_border: false,
                show_shadow: false,
                ..Default::default()
            };
            ContextMenu::new(props.clone()).render(
                &props,
                &ContextMenuState {
                    is_visible: true,
                    position: Some((1, 2)),
                    ..Default::default()
                },
            )
        }
        _ => {
            let props = DialogMenuProps {
                visible: true,
                items,
                style,
                dialog_type: DialogMenuType::MultiSelection,
                width: Some(27),
                show_border: false,
                show_shadow: false,
                show_close_button: false,
                ..Default::default()
            };
            let confirmed = calls.clone();
            DialogMenu::new(props.clone())
                .with_on_confirmed(move |ids| {
                    confirmed
                        .lock()
                        .unwrap()
                        .push(format!("{stage}:confirmed:{}", ids.join(",")))
                })
                .render(&props, &DialogMenuState::default())
        }
    };
    builder::div()
        .class("relative w-full h-full")
        .child(Element::text(format!("Menu fixture stage {stage}")))
        .child(menu.with_key(format!("menu-stage-{stage}")))
        .build()
}
