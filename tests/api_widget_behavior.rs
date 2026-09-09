use reactive_tui::{app::RootComponent, builder, component::Element, event::types::KeyCode};

#[path = "support/app_input.rs"]
mod app_input;
use app_input::{click, key, run};

#[test]
fn button_activation_uses_app_keyboard_and_mouse() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(24, 6), (48, 12)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let callback = calls.clone();
        let button = builder::button()
            .text("Apply")
            .class("w-8 h-1 p-0")
            .on_click(move || {
                callback.fetch_add(1, Ordering::SeqCst);
            })
            .build()
            .auto_focus();
        let frames = run(
            Control(button),
            size,
            vec![(1, key(KeyCode::Enter)), (1, click(2, 0))],
        );
        assert!(frames[0].text.contains("Apply"));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}

struct Control(Element);
impl RootComponent for Control {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn text_input_builder_edits_through_app_at_two_sizes() {
    for size in [(24, 6), (48, 12)] {
        let frames = run(
            Control(builder::text_input().value("seed").build().auto_focus()),
            size,
            vec![(1, key(KeyCode::End)), (1, key(KeyCode::Char('X')))],
        );
        assert!(frames[0].text.contains("seed"));
        assert!(
            frames.iter().any(|frame| frame.text.contains("seedX")),
            "typing must edit the painted input: {:?}",
            frames.iter().map(|frame| &frame.text).collect::<Vec<_>>()
        );
    }
}

#[test]
fn checkbox_builder_toggles_through_app_at_two_sizes() {
    for size in [(24, 6), (48, 12)] {
        let frames = run(
            Control(builder::checkbox().label("agree").build().auto_focus()),
            size,
            vec![(1, key(KeyCode::Char(' ')))],
        );
        assert!(frames[0].text.contains("[ ]"), "{}", frames[0].text);
        assert!(frames.iter().any(|frame| frame.text.contains("[✓]")));
    }
}

#[test]
fn select_builder_selects_through_app_at_two_sizes() {
    for size in [(24, 6), (48, 12)] {
        let frames = run(
            Control(
                builder::select()
                    .option("a", "Alpha")
                    .option("b", "Beta")
                    .selected("a")
                    .build()
                    .auto_focus(),
            ),
            size,
            vec![
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Down)),
                (1, key(KeyCode::Enter)),
            ],
        );
        assert!(frames[0].text.contains("Alpha"));
        assert!(frames.last().unwrap().text.contains("Beta"));
    }
}
