use super::{click, key, run, Control};
use reactive_tui::{
    builder,
    component::Element,
    event::types::KeyCode,
    widgets::layout::{StackBuilder, StackPadding},
};

#[test]
fn stack_preserves_nested_input_and_cell_spacing() {
    for size in [(24, 8), (48, 12)] {
        let stack = StackBuilder::vertical()
            .spacing(1)
            .padding_all(1)
            .child(Element::text("Title").with_class("w-10 h-1"))
            .child(
                builder::text_input()
                    .value("seed")
                    .class("w-12 h-1")
                    .build()
                    .auto_focus(),
            )
            .render()
            .with_class("w-full h-full");
        let frames = run(
            Control(stack),
            size,
            vec![
                (1, key(KeyCode::End)),
                (1, key(KeyCode::Char('X'))),
                (2, None),
            ],
        );
        assert!(
            frames[0]
                .screen
                .cell(1, 1)
                .unwrap()
                .contents()
                .contains('T'),
            "{}",
            frames[0].text
        );
        assert!(
            frames.last().unwrap().text.contains("seedX"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn specialized_stack_retains_children_classes_and_clicks() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(24, 8), (48, 12)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let callback = calls.clone();
        let stack = builder::specialized::StackBuilder::new()
            .padding(StackPadding::all(1))
            .child(
                builder::button()
                    .text("Apply")
                    .class("w-8 h-1 p-0")
                    .on_click(move || {
                        callback.fetch_add(1, Ordering::SeqCst);
                    })
                    .build(),
            )
            .class("w-20 h-5")
            .build();
        let frames = run(Control(stack), size, vec![(1, click(2, 1)), (1, None)]);
        assert!(frames[0].text.contains("Apply"), "{}", frames[0].text);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn stack_alignment_justification_reverse_and_wrap_use_real_bounds() {
    use reactive_tui::widgets::layout::{StackAlignment, StackJustify};
    for size in [(24, 8), (48, 12)] {
        for (justify, xs) in [
            (StackJustify::Start, [0, 2]),
            (StackJustify::Center, [4, 6]),
            (StackJustify::End, [8, 10]),
            (StackJustify::SpaceBetween, [0, 10]),
            (StackJustify::SpaceAround, [2, 8]),
            (StackJustify::SpaceEvenly, [3, 7]),
        ] {
            for (alignment, y) in [
                (StackAlignment::Start, 0),
                (StackAlignment::Center, 2),
                (StackAlignment::End, 4),
            ] {
                let stack = StackBuilder::horizontal()
                    .justify(justify.clone())
                    .alignment(alignment)
                    .child(Element::text("A").with_class("w-2 h-1"))
                    .child(Element::text("B").with_class("w-2 h-1"))
                    .render()
                    .with_class("w-12 h-5");
                let frames = run(Control(stack), size, vec![(1, None)]);
                for (x, text) in xs.into_iter().zip(["A", "B"]) {
                    assert_eq!(
                        frames[0].screen.cell(y, x).unwrap().contents(),
                        text,
                        "{justify:?}: {}",
                        frames[0].text
                    );
                }
            }
        }
        let stack = StackBuilder::horizontal()
            .reverse(true)
            .child(Element::text("A").with_class("w-2 h-1"))
            .child(Element::text("B").with_class("w-2 h-1"))
            .render()
            .with_class("w-12 h-2");
        let frames = run(Control(stack), size, vec![(1, None)]);
        assert_eq!(frames[0].screen.cell(0, 10).unwrap().contents(), "A");
        assert_eq!(frames[0].screen.cell(0, 8).unwrap().contents(), "B");
        let stack = StackBuilder::horizontal()
            .wrap(true)
            .children(
                ["A", "B", "C"]
                    .into_iter()
                    .map(|t| Element::text(t).with_class("w-3 h-1 flex-none"))
                    .collect(),
            )
            .render()
            .with_class("w-6 h-2 content-start");
        let frames = run(Control(stack), size, vec![(1, None)]);
        assert_eq!(frames[0].screen.cell(0, 0).unwrap().contents(), "A");
        assert_eq!(frames[0].screen.cell(0, 3).unwrap().contents(), "B");
        assert_eq!(frames[0].screen.cell(1, 0).unwrap().contents(), "C");
    }
}
