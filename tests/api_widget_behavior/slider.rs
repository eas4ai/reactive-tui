use super::{click, key, run, Control};
use reactive_tui::{
    builder,
    component::{Component, Element},
    event::types::KeyCode,
    widgets::input::{Slider, SliderBuilder, SliderOrientation, SliderProps},
};
use std::sync::{Arc, Mutex};

#[test]
fn slider_builder_paints_label_and_changes_with_keyboard() {
    for size in [(32, 6), (60, 12)] {
        let frames = run(
            Control(
                builder::slider()
                    .min(-10.0)
                    .max(10.0)
                    .value(0.0)
                    .step(2.0)
                    .label("Gain")
                    .class("w-28 h-2")
                    .build()
                    .auto_focus(),
            ),
            size,
            vec![(1, key(KeyCode::Right)), (2, None)],
        );
        assert!(frames[0].text.contains("Gain"));
        assert!(!frames[0].text.contains("Slider"));
        assert!(
            frames.last().unwrap().text.contains("2.0"),
            "{}",
            frames.last().unwrap().text
        );
    }
}

#[test]
fn slider_mouse_uses_padded_track_and_vertical_rows() {
    for size in [(32, 8), (60, 12)] {
        let values = Arc::new(Mutex::new(Vec::new()));
        let callback = values.clone();
        let props = SliderBuilder::new()
            .range(-100.0, 100.0)
            .value(0.0)
            .width(40)
            .show_labels(true)
            .show_value(false)
            .build();
        let slider = Element::typed_with::<Slider>(props, move |p| {
            let callback = callback.clone();
            Slider::new(p).with_on_change(move |v| callback.lock().unwrap().push(v))
        })
        .with_class("w-28 h-5 p-0.5");
        // 24 content cells: two focus cells, five min-label cells, brackets,
        // four max-label cells leave an 11-cell track from x=10 through x=20.
        let frames = run(Control(slider), size, vec![(1, click(20, 2)), (2, None)]);
        assert_eq!(
            *values.lock().unwrap(),
            vec![100.0],
            "{}",
            frames.last().unwrap().text
        );
        let slider = SliderBuilder::new()
            .range(0.0, 10.0)
            .value(0.0)
            .width(5)
            .orientation(SliderOrientation::Vertical)
            .show_value(false)
            .render()
            .with_class("w-5 h-5");
        let frames = run(Control(slider), size, vec![(1, click(2, 0)), (2, None)]);
        assert!(frames[0]
            .screen
            .cell(4, 2)
            .unwrap()
            .contents()
            .contains('●'));
        assert!(frames
            .last()
            .unwrap()
            .screen
            .cell(0, 2)
            .unwrap()
            .contents()
            .contains('◉'));
    }
}

#[test]
fn slider_callbacks_are_once_per_change_and_invalid_values_are_inert() {
    for size in [(32, 6), (60, 12)] {
        let values = Arc::new(Mutex::new(Vec::new()));
        let callback = values.clone();
        let slider = Element::typed_with::<Slider>(
            SliderProps {
                min: 0.0,
                max: 1.0,
                value: 0.9,
                step: 0.4,
                ..Default::default()
            },
            move |p| {
                let callback = callback.clone();
                Slider::new(p).with_on_change(move |v| callback.lock().unwrap().push(v))
            },
        )
        .auto_focus();
        run(
            Control(slider),
            size,
            vec![
                (1, key(KeyCode::PageUp)),
                (1, key(KeyCode::End)),
                (2, key(KeyCode::End)),
                (2, None),
            ],
        );
        assert!(values
            .lock()
            .unwrap()
            .iter()
            .all(|v| (0.0..=1.0).contains(v)));
        assert_eq!(*values.lock().unwrap(), vec![1.0]);
        assert_eq!(
            values.lock().unwrap().iter().filter(|v| **v == 1.0).count(),
            1
        );
        for props in [
            SliderProps {
                min: 10.0,
                max: 0.0,
                ..Default::default()
            },
            SliderProps {
                step: 0.0,
                ..Default::default()
            },
            SliderProps {
                value: f64::NAN,
                ..Default::default()
            },
        ] {
            let frames = run(
                Control(Element::typed::<Slider>(props).auto_focus()),
                size,
                vec![(1, key(KeyCode::Right)), (1, click(3, 0)), (1, None)],
            );
            assert!(frames.last().unwrap().text.contains("Invalid slider range"));
        }
        let frames = run(
            Control(
                SliderBuilder::new()
                    .value(50.0)
                    .disabled(true)
                    .render()
                    .auto_focus(),
            ),
            size,
            vec![(1, key(KeyCode::Right)), (1, click(3, 0)), (1, None)],
        );
        assert!(frames.last().unwrap().text.contains("50.0"));
    }
}

#[test]
fn slider_track_follows_viewport_and_resize() {
    use reactive_tui::event::types::{Event, ResizeEvent};
    for size in [(24, 6), (48, 12)] {
        let values = Arc::new(Mutex::new(Vec::new()));
        let callback = values.clone();
        let control = Element::typed_with::<Slider>(
            SliderBuilder::new()
                .width(80)
                .value(0.0)
                .show_value(false)
                .build(),
            move |p| {
                let callback = callback.clone();
                Slider::new(p).with_on_change(move |v| callback.lock().unwrap().push(v))
            },
        )
        .with_class("w-full h-1");
        let frames = run(
            Control(control),
            size,
            vec![
                (1, click(size.0 - 2, 0)),
                (2, Some(Event::Resize(ResizeEvent::new(20, 6)))),
                (3, click(3, 0)),
                (4, None),
            ],
        );
        assert_eq!(*values.lock().unwrap(), vec![100.0, 0.0]);
        assert!(frames
            .last()
            .unwrap()
            .screen
            .cell(0, 3)
            .unwrap()
            .contents()
            .contains('◉'));
    }
}

#[test]
fn slider_degenerate_range_stays_at_its_only_value() {
    for size in [(24, 6), (48, 12)] {
        let props = SliderBuilder::new().range(5.0, 5.0).value(100.0).build();
        assert_eq!(props.value, 5.0);
        let values = Arc::new(Mutex::new(Vec::new()));
        let callback = values.clone();
        let control = Element::typed_with::<Slider>(props, move |p| {
            let callback = callback.clone();
            Slider::new(p).with_on_change(move |v| callback.lock().unwrap().push(v))
        })
        .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::End)),
                (1, key(KeyCode::PageDown)),
                (1, click(3, 0)),
                (2, None),
            ],
        );
        assert!(frames.last().unwrap().text.contains("5.0"));
        assert!(values.lock().unwrap().is_empty());
    }
}

#[test]
fn slider_drag_ends_on_release_and_focus_loss() {
    use reactive_tui::{
        component::LayoutType,
        event::types::{Event, MouseEventKind},
    };
    fn mouse(x: u16, kind: MouseEventKind) -> Option<Event> {
        let Some(Event::Mouse(mut event)) = click(x, 0) else {
            unreachable!()
        };
        event.kind = kind;
        Some(Event::Mouse(event))
    }
    for size in [(24, 6), (48, 12)] {
        for lose_focus in [false, true] {
            let values = Arc::new(Mutex::new(Vec::new()));
            let callback = values.clone();
            let slider = Element::typed_with::<Slider>(
                SliderBuilder::new().width(80).show_value(false).build(),
                move |p| {
                    let callback = callback.clone();
                    Slider::new(p).with_on_change(move |v| callback.lock().unwrap().push(v))
                },
            )
            .with_class("w-full h-1")
            .auto_focus();
            let root = Element::layout(LayoutType::Flex)
                .with_class("flex flex-col")
                .with_children(vec![
                    slider,
                    builder::button()
                        .text("Next")
                        .class("w-8 h-1 p-0")
                        .build()
                        .with_focus(reactive_tui::component::FocusProps::input()),
                ]);
            let mut events = vec![(1, mouse(3, MouseEventKind::Down))];
            if lose_focus {
                events.extend([
                    (2, key(KeyCode::Tab)),
                    (2, mouse(size.0 - 2, MouseEventKind::Drag)),
                    (2, None),
                ]);
            } else {
                events.extend([
                    (2, mouse(size.0 - 2, MouseEventKind::Drag)),
                    (3, mouse(size.0 - 2, MouseEventKind::Up)),
                    (3, mouse(3, MouseEventKind::Drag)),
                    (3, None),
                ]);
            }
            run(Control(root), size, events);
            assert_eq!(
                *values.lock().unwrap(),
                if lose_focus {
                    vec![0.0]
                } else {
                    vec![0.0, 100.0]
                }
            );
        }
    }
}
