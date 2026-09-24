#[cfg(test)]
mod tests {
    use reactive_tui::event::types::{
        MouseButton, MouseEvent, MouseEventKind, Position, WheelDelta, WheelEvent, WheelPhase,
    };
    use reactive_tui::hooks::{
        use_clicks, use_drag, use_gesture, use_hover, use_long_press, use_mouse_position,
        use_wheel, DragAndDropOptions, DragAndDropState, DragState, GestureType,
        MouseEventProcessor, SwipeDirection, WheelDeltaMode,
    };
    use reactive_tui::reactive::hooks::{Hooks, ThreadSafeSignal};
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
        processor.register_press_with_threshold(
            "test_component".to_string(),
            long_press.clone(),
            Duration::ZERO,
        );

        // Start press
        let down_event = MouseEvent::new(MouseEventKind::Down, Position::cell(10, 10));
        processor.process_event(&down_event, Some("test_component"));

        assert!(long_press.get().is_pressing);
        assert!(!long_press.get().is_long_press);

        // End press (simulating after threshold)
        let up_event = MouseEvent::new(MouseEventKind::Up, Position::cell(10, 10));
        processor.process_event(&up_event, Some("test_component"));

        assert!(!long_press.get().is_pressing);
        assert!(long_press.get().is_long_press);
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

        // Four points over 15 cells within 500 ms: the processor detects a
        // swipe from 3 points and 5 cells of travel.
        let state = gesture.get();
        assert_eq!(
            state.gesture_type,
            GestureType::Swipe(SwipeDirection::Right)
        );
    }

    #[test]
    fn test_wheel_hook() {
        let hooks = Hooks::new();
        let wheel = use_wheel(&hooks);

        let processor = MouseEventProcessor::new();
        processor.register_wheel("test_component".to_string(), wheel.clone());

        // Simulate wheel event
        let mut wheel_event = MouseEvent::new(MouseEventKind::Wheel, Position::cell(10, 10));
        wheel_event.wheel = Some(WheelEvent {
            delta: WheelDelta::Lines { x: -1.0, y: 2.5 },
            phase: WheelPhase::Changed,
        });
        processor.process_event(&wheel_event, Some("test_component"));

        assert!(wheel.get().is_scrolling);
        assert_eq!(wheel.get().delta_x, -1.0);
        assert_eq!(wheel.get().delta_y, 2.5);
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

    #[test]
    fn api019_drag_drop_options_and_owner_cleanup_control_routed_state() {
        let processor = MouseEventProcessor::new();
        let state = ThreadSafeSignal::new(DragAndDropState::default());
        processor.register_drag_and_drop(
            "source".to_string(),
            state.clone(),
            DragAndDropOptions {
                drag_threshold: 5.0,
                drag_handle_selector: Some("source".to_string()),
                drop_zones: vec!["drop".to_string()],
                allow_drag_outside: false,
            },
        );
        processor.process_event(
            &MouseEvent::new(MouseEventKind::Down, Position::cell(1, 0))
                .with_button(MouseButton::Left),
            Some("source"),
        );
        processor.process_event(
            &MouseEvent::new(MouseEventKind::Drag, Position::cell(3, 0))
                .with_button(MouseButton::Left),
            Some("drop"),
        );
        assert!(
            !state.get().drag.is_dragging,
            "threshold must delay dragging"
        );

        processor.process_event(
            &MouseEvent::new(MouseEventKind::Drag, Position::cell(8, 0))
                .with_button(MouseButton::Left),
            Some("drop"),
        );
        assert!(state.get().drag.is_dragging);
        assert!(state.get().is_over_valid_drop);
        assert_eq!(state.get().drop_target.as_deref(), Some("drop"));
        assert!(state.get().can_drop);

        processor.process_event(
            &MouseEvent::new(MouseEventKind::Drag, Position::cell(9, 0))
                .with_button(MouseButton::Left),
            Some("outside"),
        );
        assert!(!state.get().is_over_valid_drop);
        assert!(!state.get().can_drop);

        processor.unregister_component("source");
        state.set(DragAndDropState::default());
        processor.process_event(
            &MouseEvent::new(MouseEventKind::Drag, Position::cell(12, 0))
                .with_button(MouseButton::Left),
            Some("drop"),
        );
        assert_eq!(state.get(), DragAndDropState::default());
    }

    #[test]
    fn api019_component_click_and_gesture_history_are_isolated() {
        let hooks1 = Hooks::new();
        let hooks2 = Hooks::new();
        let clicks1 = use_clicks(&hooks1);
        let clicks2 = use_clicks(&hooks2);
        let gesture1 = use_gesture(&hooks1);
        let gesture2 = use_gesture(&hooks2);
        let processor = MouseEventProcessor::new();
        processor.register_clicks("component1".to_string(), clicks1.clone());
        processor.register_clicks("component2".to_string(), clicks2.clone());
        processor.register_gesture("component1".to_string(), gesture1.clone());
        processor.register_gesture("component2".to_string(), gesture2.clone());

        let click = MouseEvent::new(MouseEventKind::Click, Position::cell(5, 5));
        processor.process_event(&click, Some("component1"));
        processor.process_event(&click, Some("component2"));
        assert_eq!(clicks1.get().click_count, 1);
        assert_eq!(clicks2.get().click_count, 1);
        assert!(!clicks2.get().is_double_click);

        for x in [0, 4] {
            processor.process_event(
                &MouseEvent::new(MouseEventKind::Move, Position::cell(x, 0)),
                Some("component1"),
            );
        }
        processor.process_event(
            &MouseEvent::new(MouseEventKind::Move, Position::cell(20, 0)),
            Some("component2"),
        );
        assert_eq!(gesture1.get().gesture_type, GestureType::None);
        assert_eq!(gesture2.get().gesture_type, GestureType::None);

        for x in [24, 30] {
            processor.process_event(
                &MouseEvent::new(MouseEventKind::Move, Position::cell(x, 0)),
                Some("component2"),
            );
        }
        assert_eq!(
            gesture2.get().gesture_type,
            GestureType::Swipe(SwipeDirection::Right)
        );
        assert_eq!(gesture1.get().gesture_type, GestureType::None);
    }
}
