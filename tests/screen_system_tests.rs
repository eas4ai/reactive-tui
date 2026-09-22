use reactive_tui::backend::DebugBackend;
use reactive_tui::component::Element;
use reactive_tui::event::types as rt_event;
use reactive_tui::screen::{
    EasingFunction, ScreenHooks, ScreenId, ScreenManager, TransitionConfig, TransitionType,
};
use std::time::Duration;

#[test]
fn test_screen_manager_creation() {
    let backend = Box::new(DebugBackend::new(80, 24));
    let screen_manager = ScreenManager::new(backend);

    assert_eq!(screen_manager.get_screen_ids().len(), 0);
    assert!(screen_manager.get_active_screen().is_none());
}

#[test]
fn test_create_and_switch_screens() {
    let backend = Box::new(DebugBackend::new(80, 24));
    let mut screen_manager = ScreenManager::new(backend);

    // Create first screen
    let main_screen = Element::text("Main Screen");
    screen_manager
        .create_screen("main", "Main".to_string(), main_screen)
        .unwrap();

    // Should automatically become active
    assert_eq!(screen_manager.get_screen_ids().len(), 1);
    assert_eq!(
        screen_manager.get_active_screen(),
        Some(&ScreenId::new("main"))
    );

    // Create second screen
    let settings_screen = Element::text("Settings Screen");
    screen_manager
        .create_screen("settings", "Settings".to_string(), settings_screen)
        .unwrap();

    // Should still have main as active
    assert_eq!(screen_manager.get_screen_ids().len(), 2);
    assert_eq!(
        screen_manager.get_active_screen(),
        Some(&ScreenId::new("main"))
    );

    // Switch to settings
    screen_manager
        .switch_to_immediate(ScreenId::new("settings"))
        .unwrap();
    assert_eq!(
        screen_manager.get_active_screen(),
        Some(&ScreenId::new("settings"))
    );
}

#[test]
fn test_screen_with_hooks() {
    let backend = Box::new(DebugBackend::new(80, 24));
    let mut screen_manager = ScreenManager::new(backend);

    let _activated = false;
    let _deactivated = false;

    let hooks = ScreenHooks {
        on_activate: Some(Box::new(|| {
            // In a real test, we'd use a shared state mechanism
            println!("Screen activated");
        })),
        on_deactivate: Some(Box::new(|| {
            println!("Screen deactivated");
        })),
        ..Default::default()
    };

    let screen = Element::text("Test Screen");
    screen_manager
        .create_screen_with_hooks("test", "Test".to_string(), screen, hooks)
        .unwrap();

    assert_eq!(
        screen_manager.get_active_screen(),
        Some(&ScreenId::new("test"))
    );
}

#[test]
fn test_screen_transitions() {
    let backend = Box::new(DebugBackend::new(80, 24));
    let mut screen_manager = ScreenManager::new(backend);

    // Set custom transition
    screen_manager.set_default_transition(TransitionConfig {
        transition_type: TransitionType::Fade,
        duration: Duration::from_millis(100),
        easing: EasingFunction::Linear,
        animation_id: None,
        use_hardware_acceleration: false,
        custom_properties: std::collections::HashMap::new(),
    });

    // Create screens
    screen_manager
        .create_screen("screen1", "Screen 1".to_string(), Element::text("Screen 1"))
        .unwrap();
    screen_manager
        .create_screen("screen2", "Screen 2".to_string(), Element::text("Screen 2"))
        .unwrap();

    // Switch with transition
    screen_manager.switch_to(ScreenId::new("screen2")).unwrap();

    // Should be transitioning initially
    assert!(screen_manager.is_transitioning());

    // Update until transition completes
    let mut updates = 0;
    while screen_manager.is_transitioning() && updates < 50 {
        screen_manager.update().unwrap();
        std::thread::sleep(Duration::from_millis(10));
        updates += 1;
    }

    // Should complete transition
    assert!(!screen_manager.is_transitioning());
    assert_eq!(
        screen_manager.get_active_screen(),
        Some(&ScreenId::new("screen2"))
    );
}

#[test]
fn test_screen_hotkeys() {
    let backend = Box::new(DebugBackend::new(80, 24));
    let mut screen_manager = ScreenManager::new(backend);

    // Create screens
    screen_manager
        .create_screen("main", "Main".to_string(), Element::text("Main"))
        .unwrap();
    screen_manager
        .create_screen(
            "settings",
            "Settings".to_string(),
            Element::text("Settings"),
        )
        .unwrap();

    // Set hotkeys
    screen_manager.set_hotkey(rt_event::KeyCode::F(1), ScreenId::new("main"));
    screen_manager.set_hotkey(rt_event::KeyCode::F(2), ScreenId::new("settings"));

    // Test F2 hotkey
    let f2_event = rt_event::Event::Key(rt_event::KeyEvent::new(rt_event::KeyCode::F(2)));
    let handled = screen_manager.process_event(&f2_event).unwrap();

    assert!(handled);

    // Complete any transition
    let mut updates = 0;
    while screen_manager.is_transitioning() && updates < 50 {
        screen_manager.update().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        updates += 1;
    }

    // Ensure transition completes
    screen_manager.update().unwrap();

    assert_eq!(
        screen_manager.get_active_screen(),
        Some(&ScreenId::new("settings"))
    );
}

#[test]
fn test_screen_update() {
    let backend = Box::new(DebugBackend::new(80, 24));
    let mut screen_manager = ScreenManager::new(backend);

    // Create screen
    screen_manager
        .create_screen("test", "Test".to_string(), Element::text("Original"))
        .unwrap();

    // Update screen content
    let updated_content = Element::text("Updated");
    screen_manager
        .update_screen(&ScreenId::new("test"), updated_content)
        .unwrap();

    // Should succeed without error
    assert_eq!(
        screen_manager.get_active_screen(),
        Some(&ScreenId::new("test"))
    );
}

#[test]
fn test_screen_removal() {
    let backend = Box::new(DebugBackend::new(80, 24));
    let mut screen_manager = ScreenManager::new(backend);

    // Create screens
    screen_manager
        .create_screen("screen1", "Screen 1".to_string(), Element::text("Screen 1"))
        .unwrap();
    screen_manager
        .create_screen("screen2", "Screen 2".to_string(), Element::text("Screen 2"))
        .unwrap();

    assert_eq!(screen_manager.get_screen_ids().len(), 2);

    // Remove non-active screen
    screen_manager
        .remove_screen(&ScreenId::new("screen2"))
        .unwrap();
    assert_eq!(screen_manager.get_screen_ids().len(), 1);
    assert_eq!(
        screen_manager.get_active_screen(),
        Some(&ScreenId::new("screen1"))
    );

    // Remove active screen (should switch to remaining screen or none)
    screen_manager
        .remove_screen(&ScreenId::new("screen1"))
        .unwrap();
    assert_eq!(screen_manager.get_screen_ids().len(), 0);
    assert!(screen_manager.get_active_screen().is_none());
}

#[test]
fn test_transition_types() {
    use reactive_tui::screen::transitions::TransitionRenderer;

    let renderer = TransitionRenderer::new(80, 24);

    // Test that all transition types can be created
    let transitions = vec![
        TransitionType::None,
        TransitionType::Fade,
        TransitionType::SlideLeft,
        TransitionType::SlideRight,
        TransitionType::SlideUp,
        TransitionType::SlideDown,
        TransitionType::Scale,
        TransitionType::Flip,
        TransitionType::Cube,
        TransitionType::Push,
    ];

    for transition_type in transitions {
        // Create a simple render tree
        let element = Element::text("Test");
        let root_node = reactive_tui::render::tree::element_to_render_node(element);
        let mut tree = reactive_tui::render::tree::RenderTree::new();
        tree.set_root(root_node);

        // Every transition type composes a surface of the renderer's size
        let result = renderer.render_transition(None, &tree, transition_type, 0.5);
        assert_eq!(
            result.dims(),
            (80, 24),
            "{transition_type:?} composed a surface of the wrong size"
        );
    }
}

#[test]
fn test_easing_functions() {
    use reactive_tui::screen::EasingFunction;

    let functions = vec![
        EasingFunction::Linear,
        EasingFunction::EaseInQuad,
        EasingFunction::EaseOutQuad,
        EasingFunction::EaseInOutQuad,
        EasingFunction::EaseInCubic,
        EasingFunction::EaseOutCubic,
        EasingFunction::EaseInOutCubic,
        EasingFunction::EaseInBack,
        EasingFunction::EaseOutBack,
        EasingFunction::EaseInOutBack,
        EasingFunction::EaseOutBounce,
    ];

    for easing in functions {
        // Test boundary values
        assert_eq!(easing.apply(0.0), 0.0);
        assert!((easing.apply(1.0) - 1.0).abs() < 0.1); // Allow some tolerance for bounce

        // Test mid-point
        let mid_result = easing.apply(0.5);
        assert!((-0.1..=1.5).contains(&mid_result)); // Allow for back easing and bounce overshoot
    }
}
