use super::{
    app_input::{click, key, run},
    Control,
};
use reactive_tui::{
    builder,
    component::Element,
    event::types::{Event, KeyCode, ResizeEvent},
};
use std::sync::{Arc, Mutex};

#[test]
fn css_tab_order_disabled_and_keyboard_only_controls_follow_presented_geometry() {
    for size in [(24, 6), (48, 12)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let button = |label: &'static str, class: &str| {
            let calls = calls.clone();
            builder::button()
                .text(label)
                .class(&format!("w-full h-1 p-0 {class}"))
                .on_click(move || calls.lock().unwrap().push(label))
                .build()
        };
        let root = Element::fragment()
            .class("flex flex-col w-full")
            .children(vec![
                button("first", "tabindex-2"),
                button("second", "tabindex-1"),
                button("disabled", "aria-disabled tabindex-0 disabled:bg-red-500"),
                button("keyboard", "keyboard-only tabindex-0"),
                Element::fragment()
                    .class("aria-disabled")
                    .children(vec![button("nested disabled", "tabindex-0")]),
            ]);
        let frames = run(
            Control(root),
            size,
            vec![
                (1, key(KeyCode::Tab)),
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Tab)),
                (1, key(KeyCode::Enter)),
                (1, click(2, 2)),
                (1, click(2, 3)),
                (1, click(2, 4)),
                (1, key(KeyCode::Tab)),
                (1, key(KeyCode::Enter)),
                (1, Some(Event::Resize(ResizeEvent::new(size.0 - 4, size.1)))),
                (2, click(size.0 - 6, 0)),
                (3, None),
            ],
        );
        assert!(frames[0].text.contains("keyboard"));
        // aria-disabled retains its half-opacity visual treatment: red-500
        // (239, 68, 68) over black rounds to (120, 34, 34).
        assert_eq!(
            frames[0].screen.cell(2, 0).unwrap().bgcolor(),
            vt100::Color::Rgb(120, 34, 34)
        );
        assert_eq!(
            *calls.lock().unwrap(),
            ["second", "first", "keyboard", "first"]
        );
    }
}

#[test]
fn screen_reader_only_text_is_unpainted_and_restorable() {
    for size in [(24, 6), (48, 12)] {
        let root = Element::fragment().class("flex flex-col").children(vec![
            Element::text("SECRET READER TEXT").class("sr-only"),
            Element::text("Restored").class("sr-only not-sr-only"),
        ]);
        let frames = run(Control(root), size, vec![(1, None)]);
        assert!(!frames[0].text.contains("SECRET"));
        assert!(frames[0].text.contains("Restored"));
    }
}
