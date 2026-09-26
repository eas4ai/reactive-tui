use reactive_tui::{
    app::RootComponent,
    builder::core::div,
    component,
    component::{props::EmptyProps, Element},
    event::types::{
        Event, MouseButton, MouseEvent, MouseEventKind, Position, WheelDelta, WheelEvent,
        WheelPhase,
    },
    hooks::{
        use_clicks, use_drag, use_drag_and_drop, use_gesture, use_hover, use_long_press,
        use_mouse_position, use_wheel, DragAndDropOptions, GestureType, SwipeDirection,
    },
};
use std::time::Duration;

#[allow(dead_code)]
mod common;
use app_input::run;
use common::app_input;

#[component]
fn HookSource(hooks: &reactive_tui::reactive::Hooks) -> Element {
    let hover = use_hover(hooks).get();
    let drag = use_drag(hooks).get();
    let position = use_mouse_position(hooks).get();
    let clicks = use_clicks(hooks).get();
    let press = use_long_press(hooks, Duration::ZERO).get();
    let gesture = use_gesture(hooks).get();
    let wheel = use_wheel(hooks).get();
    let drop = use_drag_and_drop(
        hooks,
        DragAndDropOptions {
            drag_threshold: 5.0,
            drag_handle_selector: Some("source".to_string()),
            drop_zones: vec!["drop".to_string()],
            allow_drag_outside: false,
        },
    );
    let direction = match gesture.gesture_type {
        GestureType::Swipe(SwipeDirection::Right) => "Right",
        _ => "None",
    };
    let x = position.position.map_or(u32::MAX, |position| position.x());
    div()
        .class("w-70 h-1")
        .text(&format!(
            "H{} P{x} C{} L{} G{direction} W{} D{} Z{} A{} R{}",
            u8::from(hover.is_hovered),
            clicks.click_count,
            u8::from(press.is_long_press),
            wheel.delta_y,
            u8::from(drop.drag.is_dragging),
            drop.drop_target.as_deref().unwrap_or("none"),
            u8::from(drop.can_drop),
            u8::from(drag.is_dragging),
        ))
        .build()
}

#[component]
fn HookDrop(hooks: &reactive_tui::reactive::Hooks) -> Element {
    let _ = hooks;
    div().class("w-8 h-1").text("DROP").build()
}

struct HookRoot;

impl RootComponent for HookRoot {
    fn render(&self) -> Element {
        div()
            .class("flex flex-row w-full h-full")
            .child(Element::typed::<HookSource>(EmptyProps).with_key("source"))
            .child(Element::typed::<HookDrop>(EmptyProps).with_key("drop"))
            .build()
    }

    fn wake_driven(&self) -> bool {
        true
    }
}

fn mouse(kind: MouseEventKind, x: u16) -> Option<Event> {
    Some(Event::Mouse(
        MouseEvent::new(kind, Position::cell(x, 0)).with_button(MouseButton::Left),
    ))
}

#[test]
fn api019_app_routes_mouse_hooks_options_and_local_coordinates() {
    let mut wheel = MouseEvent::new(MouseEventKind::Wheel, Position::cell(8, 0));
    wheel.wheel = Some(WheelEvent {
        delta: WheelDelta::Lines { x: -1.0, y: 2.5 },
        phase: WheelPhase::Changed,
    });
    let frames = run(
        HookRoot,
        (90, 3),
        vec![
            (1, mouse(MouseEventKind::Move, 1)),
            (2, mouse(MouseEventKind::Move, 4)),
            (3, mouse(MouseEventKind::Move, 8)),
            (4, Some(Event::Mouse(wheel))),
            (5, mouse(MouseEventKind::Click, 8)),
            (6, mouse(MouseEventKind::Down, 8)),
            (7, mouse(MouseEventKind::Up, 8)),
            (8, mouse(MouseEventKind::Down, 8)),
            (9, mouse(MouseEventKind::Drag, 74)),
            (10, None),
        ],
    );
    let mut painted = frames.iter().map(|frame| frame.text.as_str());
    assert!(painted.clone().any(|text| text.contains("H1 P8")));
    assert!(painted.clone().any(|text| text.contains("C1")));
    assert!(painted.clone().any(|text| text.contains("L1")));
    assert!(painted.clone().any(|text| text.contains("GRight")));
    assert!(painted.clone().any(|text| text.contains("W2.5")));
    assert!(painted.clone().any(|text| text.contains("D1 Zdrop A1")));
    assert!(
        painted.any(|text| text.contains("P8")),
        "source positions must be component-local"
    );
}
