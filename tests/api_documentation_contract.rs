//! Runtime contracts exercised by the public documentation macros.
use reactive_tui::event::types::{Event, ResizeEvent};
use reactive_tui::{app::RootComponent, component::Element, prelude::*, responsive_css};

#[allow(dead_code)]
mod common;
use common::app_input;

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn responsive_element() -> Element {
    div()
        .styles(responsive_css! {
            base: { background_color: (1.0, 0.0, 0.0, 1.0), padding: 0.0 },
            lg: { background_color: (0.0, 0.0, 1.0, 1.0) },
            md: { background_color: (0.0, 1.0, 0.0, 1.0), padding: 1.0 },
        })
        .class("w-full h-full")
        .child(Element::text("content"))
        .build()
}

#[test]
fn responsive_profiles_follow_viewport_boundaries_and_reverse_resize() {
    let widths = [79, 80, 119, 120, 80, 79];
    let mut steps: Vec<_> = widths
        .iter()
        .enumerate()
        .skip(1)
        .map(|(index, width)| (index, Some(Event::Resize(ResizeEvent::new(*width, 5)))))
        .collect();
    steps.push((widths.len(), None));
    let frames = app_input::run(Root(responsive_element()), (widths[0], 5), steps);
    let mut observed: Vec<_> = frames.iter().map(|frame| frame.screen.size().1).collect();
    observed.dedup();
    assert_eq!(
        observed, widths,
        "every requested viewport must actually be painted"
    );
    for frame in &frames {
        let width = frame.screen.size().1;
        let color = if width >= 120 {
            (0, 0, 255)
        } else if width >= 80 {
            (0, 255, 0)
        } else {
            (255, 0, 0)
        };
        assert_eq!(
            frame.screen.cell(0, 0).unwrap().bgcolor(),
            vt100::Color::Rgb(color.0, color.1, color.2)
        );
        let inset = u16::from(width >= 80);
        assert_eq!(frame.screen.cell(inset, inset).unwrap().contents(), "c");
    }
}

#[test]
fn responsive_profiles_preserve_class_precedence_and_app_isolation() {
    for width in [80, 40, 120, 40] {
        let frames = app_input::run(
            Root(responsive_element().class("w-full h-full bg-white")),
            (width, 5),
            vec![(1, None)],
        );
        assert_eq!(
            frames[0].screen.cell(0, 0).unwrap().bgcolor(),
            vt100::Color::Rgb(255, 255, 255)
        );
        let frames = app_input::run(Root(responsive_element()), (width, 5), vec![(1, None)]);
        let green = width == 80;
        let blue = width == 120;
        assert_eq!(
            frames[0].screen.cell(0, 0).unwrap().bgcolor(),
            vt100::Color::Rgb(
                if green || blue { 0 } else { 255 },
                if green { 255 } else { 0 },
                if blue { 255 } else { 0 }
            )
        );
    }
}

#[test]
fn responsive_blocks_evaluate_once_in_stable_breakpoint_order() {
    let visits = std::cell::RefCell::new(Vec::new());
    let owned = String::from("bg-blue-500");
    let style = responsive_css! {
        base: { padding: 0.0 },
        md: { padding: { visits.borrow_mut().push(80); 2.0 } },
        sm: { padding: { visits.borrow_mut().push(40); 1.0 } },
        md: { class: owned, padding: { visits.borrow_mut().push(81); 3.0 } },
    };
    assert_eq!(*visits.borrow(), [40, 80, 81]);
    for (width, padding) in [(39, 0.0), (40, 1.0), (79, 1.0), (80, 3.0), (160, 3.0)] {
        assert_eq!(
            style
                .clone()
                .at_width(width)
                .build()
                .padding
                .left
                .into_raw()
                .value(),
            padding
        );
    }
    assert_eq!(*visits.borrow(), [40, 80, 81]);
}

#[test]
fn layout_convenience_macros_use_the_facade_and_fill_the_actual_parent() {
    let centered = reactive_tui::flex_center!().build();
    assert_eq!(centered.display, Display::Flex);
    assert_eq!(centered.align_items, Some(AlignItems::Center));
    assert_eq!(
        reactive_tui::flex_column!().build().flex_direction,
        FlexDirection::Column
    );
    let root = div()
        .class("w-7 h-3 relative")
        .child(Element::text("earlier sibling"))
        .child(
            div()
                .styles(reactive_tui::absolute_fill!())
                .class("bg-red-500")
                .build(),
        )
        .build();
    let frame = app_input::run(Root(root), (20, 6), vec![(1, None)]).remove(0);
    assert_eq!(
        frame.screen.cell(2, 6).unwrap().bgcolor(),
        vt100::Color::Rgb(239, 68, 68)
    );
    assert_ne!(
        frame.screen.cell(3, 7).unwrap().bgcolor(),
        vt100::Color::Rgb(239, 68, 68)
    );
}
