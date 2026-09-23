use reactive_tui::{event::types::Position, hooks::DragState};
use reactive_tui::{
    event::types::{MouseButton, MouseEvent, MouseEventKind},
    hooks::{GestureState, MouseEventProcessor},
    reactive::hooks::ThreadSafeSignal,
};

fn drag(start: Position, end: Position) -> DragState {
    DragState {
        drag_start: Some(start),
        current_position: Some(end),
        ..DragState::default()
    }
}

#[test]
fn distance_uses_input_units_in_both_directions() {
    for (start, end) in [
        (Position::cell(10, 20), Position::cell(13, 24)),
        (Position::pixel(10, 20), Position::pixel(13, 24)),
    ] {
        for state in [drag(start, end), drag(end, start)] {
            assert_eq!(state.distance(), 5.0);
            assert!(state.is_drag_threshold_met(5.0));
            assert!(!state.is_drag_threshold_met(5.01));
        }
    }
}

#[test]
fn distance_handles_full_pixel_range_and_rejects_mixed_units() {
    let maximum = f64::from(u32::MAX);
    assert_eq!(
        drag(Position::pixel(0, 0), Position::pixel(u32::MAX, 0)).distance(),
        maximum
    );
    assert_eq!(
        drag(Position::pixel(u32::MAX, u32::MAX), Position::pixel(0, 0)).distance(),
        maximum.hypot(maximum)
    );
    for state in [
        drag(Position::cell(0, 0), Position::pixel(3, 4)),
        drag(Position::pixel(0, 0), Position::cell(3, 4)),
        DragState::default(),
    ] {
        assert_eq!(state.distance(), 0.0);
        assert!(!state.is_drag_threshold_met(1.0));
    }
}

#[test]
fn processor_restarts_drag_when_units_change_and_saturates_public_delta() {
    let processor = MouseEventProcessor::new();
    let state = ThreadSafeSignal::new(DragState::default());
    processor.register_drag("drag".into(), state.clone());
    for (kind, position) in [
        (MouseEventKind::Down, Position::cell(1, 2)),
        (MouseEventKind::Drag, Position::cell(4, 6)),
        (MouseEventKind::Drag, Position::pixel(100, 200)),
    ] {
        processor.process_event(
            &MouseEvent::new(kind, position).with_button(MouseButton::Left),
            Some("drag"),
        );
    }
    assert_eq!(state.get().distance(), 0.0);
    assert_eq!(state.get().drag_delta, (0, 0));
    processor.process_event(
        &MouseEvent::new(MouseEventKind::Drag, Position::pixel(u32::MAX, 200))
            .with_button(MouseButton::Left),
        Some("drag"),
    );
    assert_eq!(state.get().drag_start, Some(Position::pixel(100, 200)));
    assert_eq!(state.get().drag_delta, (i32::MAX, 0));
    assert_eq!(state.get().distance(), f64::from(u32::MAX) - 100.0);
}

#[test]
fn processor_does_not_join_cell_and_pixel_swipe_history() {
    let processor = MouseEventProcessor::new();
    let state = ThreadSafeSignal::new(GestureState::default());
    processor.register_gesture("swipe".into(), state.clone());
    for position in [
        Position::cell(0, 0),
        Position::cell(3, 0),
        Position::cell(8, 0),
    ] {
        processor.process_event(
            &MouseEvent::new(MouseEventKind::Move, position),
            Some("swipe"),
        );
    }
    assert!(state.get().is_active);
    processor.process_event(
        &MouseEvent::new(MouseEventKind::Move, Position::pixel(1, 1)),
        Some("swipe"),
    );
    assert!(!state.get().is_active);
    assert_eq!(state.get().velocity, (0.0, 0.0));
    for position in [
        Position::pixel(u32::MAX / 2, 1),
        Position::pixel(u32::MAX, 1),
    ] {
        processor.process_event(
            &MouseEvent::new(MouseEventKind::Move, position),
            Some("swipe"),
        );
    }
    assert!(state.get().is_active);
    assert_eq!(state.get().start_position, Some(Position::pixel(1, 1)));
    assert!(state.get().velocity.0.is_finite());
    assert!(state.get().velocity.0 >= 0.0);
}
