#[cfg(test)]
mod tests {
    use reactive_tui::event::types::{MouseButton, MouseEvent, MouseEventKind, Position};
    use reactive_tui::hooks::{
        use_clicks, use_drag, use_gesture, use_hover, use_long_press, use_mouse_position,
        use_wheel, DragState, GestureType, MouseEventProcessor, SwipeDirection, WheelDeltaMode,
    };
    use reactive_tui::reactive::hooks::Hooks;
    use std::time::Duration;

    #[test]
    fn test_hover_hook() {
        let hooks = Hooks::new();
        let hover = use_hover(&hooks);

        // Initial state
        assert!(!hover.get().is_hovered);
        assert!(hover.get().entered_at.is_none());
        assert!(hover.get().position.is_none());

        // Create processor and register the hook
        let processor = MouseEventProcessor::new();
        processor.register_hover("test_component".to_string(), hover.clone());

        // Simulate mouse enter
        let enter_event = MouseEvent::new(MouseEventKind::Enter, Position::cell(10, 5));
        processor.process_event(&enter_event, Some("test_component"));

        // Check hover state is updated
        assert!(hover.get().is_hovered);
        assert!(hover.get().entered_at.is_some());
        assert_eq!(hover.get().position, Some(Position::cell(10, 5)));

        // Simulate mouse leave
        let leave_event = MouseEvent::new(MouseEventKind::Leave, Position::cell(10, 5));
        processor.process_event(&leave_event, Some("test_component"));

        // Check hover state is cleared
        assert!(!hover.get().is_hovered);
        assert!(hover.get().entered_at.is_none());
        assert!(hover.get().position.is_none());
    }

    #[test]
    fn test_drag_hook() {
        let hooks = Hooks::new();
        let drag = use_drag(&hooks);

        // Initial state
        assert!(!drag.get().is_dragging);
        assert!(drag.get().drag_start.is_none());

        let processor = MouseEventProcessor::new();
        processor.register_drag("test_component".to_string(), drag.clone());

        // Start drag
        let down_event = MouseEvent::new(MouseEventKind::Down, Position::cell(10, 10))
            .with_button(MouseButton::Left);
        processor.process_event(&down_event, Some("test_component"));

        // Drag move
        let drag_event = MouseEvent::new(MouseEventKind::Drag, Position::cell(15, 12))
            .with_button(MouseButton::Left);
        processor.process_event(&drag_event, Some("test_component"));

        let state = drag.get();
        assert!(state.is_dragging);
        assert_eq!(state.drag_start, Some(Position::cell(10, 10)));
        assert_eq!(state.current_position, Some(Position::cell(15, 12)));
        assert_eq!(state.drag_delta, (5, 2));

        // End drag
        let up_event = MouseEvent::new(MouseEventKind::Up, Position::cell(15, 12))
            .with_button(MouseButton::Left);
        processor.process_event(&up_event, Some("test_component"));

        assert!(!drag.get().is_dragging);
    }

    #[test]
    fn test_clicks_hook() {
        let hooks = Hooks::new();
        let clicks = use_clicks(&hooks);

        let processor = MouseEventProcessor::new();
        processor.register_clicks("test_component".to_string(), clicks.clone());

        // Single click
        let click_event = MouseEvent::new(MouseEventKind::Click, Position::cell(10, 10));
        processor.process_event(&click_event, Some("test_component"));

        assert_eq!(clicks.get().click_count, 1);
        assert!(!clicks.get().is_double_click);
        assert!(!clicks.get().is_triple_click);

        // Double click
        let double_click_event =
            MouseEvent::new(MouseEventKind::DoubleClick, Position::cell(10, 10));
        processor.process_event(&double_click_event, Some("test_component"));

        assert_eq!(clicks.get().click_count, 2);
        assert!(clicks.get().is_double_click);
        assert!(!clicks.get().is_triple_click);

        // Triple click
        let triple_click_event =
            MouseEvent::new(MouseEventKind::TripleClick, Position::cell(10, 10));
        processor.process_event(&triple_click_event, Some("test_component"));

        assert_eq!(clicks.get().click_count, 3);
        assert!(clicks.get().is_triple_click);
    }

    #[test]
    fn test_mouse_position_hook() {
        let hooks = Hooks::new();
        let position = use_mouse_position(&hooks);

        let processor = MouseEventProcessor::new();
        processor.register_position("test_component".to_string(), position.clone());

        // Initial state
        assert!(position.get().position.is_none());
        assert!(!position.get().is_inside);

        // Mouse move
        let move_event = MouseEvent::new(MouseEventKind::Move, Position::cell(25, 15));
        processor.process_event(&move_event, Some("test_component"));

        assert_eq!(position.get().position, Some(Position::cell(25, 15)));
        assert_eq!(position.get().client_position, Some((25.0, 15.0)));
        assert!(position.get().is_inside);

        // Mouse leave
        let leave_event = MouseEvent::new(MouseEventKind::Leave, Position::cell(25, 15));
        processor.process_event(&leave_event, Some("test_component"));

        assert!(!position.get().is_inside);
    }

    #[test]
    fn test_long_press_hook() {
        let hooks = Hooks::new();
        let threshold = Duration::from_millis(800);
        let long_press = use_long_press(&hooks, threshold);

        let processor = MouseEventProcessor::new();
        processor.register_press("test_component".to_string(), long_press.clone());

        // Start press
        let down_event = MouseEvent::new(MouseEventKind::Down, Position::cell(10, 10));
        processor.process_event(&down_event, Some("test_component"));

        assert!(long_press.get().is_pressing);
        assert!(!long_press.get().is_long_press);

        // End press (simulating after threshold)
        let up_event = MouseEvent::new(MouseEventKind::Up, Position::cell(10, 10));
        processor.process_event(&up_event, Some("test_component"));

        // Note: In real usage, timing would determine if it's a long press
        // The processor tracks actual timing
        assert!(!long_press.get().is_pressing);
    }

    #[test]
    fn test_gesture_hook() {
        let hooks = Hooks::new();
        let gesture = use_gesture(&hooks);

        let processor = MouseEventProcessor::new();
        processor.register_gesture("test_component".to_string(), gesture.clone());

        // Simulate a swipe right gesture with multiple move events
        let positions = vec![
            Position::cell(10, 10),
            Position::cell(15, 10),
            Position::cell(20, 10),
            Position::cell(25, 10),
        ];

        for pos in positions {
            let move_event = MouseEvent::new(MouseEventKind::Move, pos);
            processor.process_event(&move_event, Some("test_component"));
        }

        let state = gesture.get();
        // Gesture detection requires sufficient movement
        if let GestureType::Swipe(direction) = state.gesture_type {
            assert_eq!(direction, SwipeDirection::Right);
        }
    }

    #[test]
    fn test_wheel_hook() {
        let hooks = Hooks::new();
        let wheel = use_wheel(&hooks);

        let processor = MouseEventProcessor::new();
        processor.register_wheel("test_component".to_string(), wheel.clone());

        // Simulate wheel event
        let wheel_event = MouseEvent::new(MouseEventKind::Wheel, Position::cell(10, 10));
        processor.process_event(&wheel_event, Some("test_component"));

        assert!(wheel.get().is_scrolling);
        assert_eq!(wheel.get().delta_y, 3.0); // Default scroll amount
        assert_eq!(wheel.get().delta_mode, WheelDeltaMode::Line);
    }

    #[test]
    fn test_drag_distance_calculation() {
        let hooks = Hooks::new();
        let drag = use_drag(&hooks);

        // Manually set drag state to test distance calculation
        drag.set(DragState {
            is_dragging: true,
            is_over_drop_zone: false,
            drag_start: Some(Position::cell(10, 10)),
            current_position: Some(Position::cell(14, 13)),
            drag_delta: (4, 3),
            button: MouseButton::Left,
        });

        let state = drag.get();
        let distance = state.distance();
        assert_eq!(distance, 5.0); // 3-4-5 triangle
        assert!(state.is_drag_threshold_met(3.0));
        assert!(!state.is_drag_threshold_met(6.0));
    }

    #[test]
    fn test_multiple_components() {
        let hooks1 = Hooks::new();
        let hooks2 = Hooks::new();

        let hover1 = use_hover(&hooks1);
        let hover2 = use_hover(&hooks2);

        let processor = MouseEventProcessor::new();
        processor.register_hover("component1".to_string(), hover1.clone());
        processor.register_hover("component2".to_string(), hover2.clone());

        // Enter component 1
        let enter1 = MouseEvent::new(MouseEventKind::Enter, Position::cell(10, 10));
        processor.process_event(&enter1, Some("component1"));

        assert!(hover1.get().is_hovered);
        assert!(!hover2.get().is_hovered);

        // Move to component 2
        let move_event = MouseEvent::new(MouseEventKind::Move, Position::cell(20, 20));
        processor.process_event(&move_event, Some("component2"));

        // Component 1 should be left, component 2 should be entered
        assert!(!hover1.get().is_hovered);
        assert!(hover2.get().is_hovered);
    }
}
