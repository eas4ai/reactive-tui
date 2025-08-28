use reactive_tui::backend::DebugBackend;
use reactive_tui::component::Element;
use reactive_tui::event::types as rt_event;
use reactive_tui::screen::{ScreenHooks, ScreenId, ScreenManager, TransitionConfig};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Screen Demo with Transitions ===");

    // Create a debug backend for demonstration
    let backend = Box::new(DebugBackend::new(80, 24));
    let mut screen_manager = ScreenManager::new(backend);

    // Set default transition with enhanced easing
    screen_manager.set_default_transition(
        TransitionConfig::preset_smooth_fade()
            .with_animation_id("default-screen-transition")
            .with_custom_property("smoothness", 0.8),
    );

    // Create multiple screens
    create_demo_screens(&mut screen_manager)?;

    // Set up hotkeys for screen switching
    setup_hotkeys(&mut screen_manager);

    // Demonstrate screen switching with different transitions
    demonstrate_transitions(&mut screen_manager)?;

    // Demonstrate screen updates
    demonstrate_screen_updates(&mut screen_manager)?;

    // Demonstrate event handling
    demonstrate_event_handling(&mut screen_manager)?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn create_demo_screens(screen_manager: &mut ScreenManager) -> Result<(), String> {
    println!("\n1. Creating demo screens...");

    // Main screen
    let main_screen = Element::fragment()
        .child(Element::text("=== MAIN SCREEN ==="))
        .child(Element::text(""))
        .child(Element::text("Welcome to the Multi-Screen Demo!"))
        .child(Element::text(""))
        .child(Element::text("Available screens:"))
        .child(Element::text("• Main (F1)"))
        .child(Element::text("• Settings (F2)"))
        .child(Element::text("• Help (F3)"))
        .child(Element::text("• Dashboard (F4)"))
        .child(Element::text(""))
        .child(Element::text("Press hotkeys to switch screens"));

    screen_manager.create_screen_with_hooks(
        "main",
        "Main Screen".to_string(),
        main_screen,
        ScreenHooks {
            on_activate: Some(Box::new(|| println!("📱 Main screen activated"))),
            on_deactivate: Some(Box::new(|| println!("📱 Main screen deactivated"))),
            on_create: Some(Box::new(|| println!("📱 Main screen created"))),
            ..Default::default()
        },
    )?;

    // Settings screen
    let settings_screen = Element::fragment()
        .child(Element::text("=== SETTINGS SCREEN ==="))
        .child(Element::text(""))
        .child(Element::text("⚙️  Application Settings"))
        .child(Element::text(""))
        .child(Element::text("• Theme: Dark"))
        .child(Element::text("• Font Size: 14px"))
        .child(Element::text("• Auto-save: Enabled"))
        .child(Element::text("• Notifications: On"))
        .child(Element::text(""))
        .child(Element::text("Use F1-F4 to navigate"));

    screen_manager.create_screen_with_hooks(
        "settings",
        "Settings".to_string(),
        settings_screen,
        ScreenHooks {
            on_activate: Some(Box::new(|| println!("⚙️  Settings screen activated"))),
            on_deactivate: Some(Box::new(|| println!("⚙️  Settings screen deactivated"))),
            on_create: Some(Box::new(|| println!("⚙️  Settings screen created"))),
            ..Default::default()
        },
    )?;

    // Help screen
    let help_screen = Element::fragment()
        .child(Element::text("=== HELP SCREEN ==="))
        .child(Element::text(""))
        .child(Element::text("❓ Help & Documentation"))
        .child(Element::text(""))
        .child(Element::text("Keyboard Shortcuts:"))
        .child(Element::text("• F1 - Main Screen"))
        .child(Element::text("• F2 - Settings"))
        .child(Element::text("• F3 - Help (this screen)"))
        .child(Element::text("• F4 - Dashboard"))
        .child(Element::text(""))
        .child(Element::text("Features:"))
        .child(Element::text("• Smooth transitions"))
        .child(Element::text("• Persistent screen state"))
        .child(Element::text("• Global hotkeys"));

    screen_manager.create_screen_with_hooks(
        "help",
        "Help".to_string(),
        help_screen,
        ScreenHooks {
            on_activate: Some(Box::new(|| println!("❓ Help screen activated"))),
            on_deactivate: Some(Box::new(|| println!("❓ Help screen deactivated"))),
            on_create: Some(Box::new(|| println!("❓ Help screen created"))),
            ..Default::default()
        },
    )?;

    // Dashboard screen
    let dashboard_screen = Element::fragment()
        .child(Element::text("=== DASHBOARD ==="))
        .child(Element::text(""))
        .child(Element::text("📊 System Dashboard"))
        .child(Element::text(""))
        .child(Element::text("Status: ✅ All systems operational"))
        .child(Element::text(""))
        .child(Element::text("Metrics:"))
        .child(Element::text("• CPU Usage: 45%"))
        .child(Element::text("• Memory: 2.1GB / 8GB"))
        .child(Element::text("• Disk: 120GB / 500GB"))
        .child(Element::text("• Network: 15 Mbps"))
        .child(Element::text(""))
        .child(Element::text("Last updated: Just now"));

    screen_manager.create_screen_with_hooks(
        "dashboard",
        "Dashboard".to_string(),
        dashboard_screen,
        ScreenHooks {
            on_activate: Some(Box::new(|| println!("📊 Dashboard activated"))),
            on_deactivate: Some(Box::new(|| println!("📊 Dashboard deactivated"))),
            on_create: Some(Box::new(|| println!("📊 Dashboard created"))),
            ..Default::default()
        },
    )?;

    println!(
        "✅ Created {} screens",
        screen_manager.get_screen_ids().len()
    );
    Ok(())
}

fn setup_hotkeys(screen_manager: &mut ScreenManager) {
    println!("\n2. Setting up hotkeys...");

    screen_manager.set_hotkey(rt_event::KeyCode::F(1), ScreenId::new("main"));
    screen_manager.set_hotkey(rt_event::KeyCode::F(2), ScreenId::new("settings"));
    screen_manager.set_hotkey(rt_event::KeyCode::F(3), ScreenId::new("help"));
    screen_manager.set_hotkey(rt_event::KeyCode::F(4), ScreenId::new("dashboard"));

    println!("✅ Hotkeys configured: F1-F4");
}

fn demonstrate_transitions(screen_manager: &mut ScreenManager) -> Result<(), String> {
    println!("\n3. Demonstrating different transitions...");

    // Start with main screen
    println!("📱 Starting with main screen");
    screen_manager.switch_to_immediate(ScreenId::new("main"))?;

    // Fade to settings with spring easing
    println!("🔄 Spring fade transition to settings...");
    screen_manager.switch_to_with_transition(
        ScreenId::new("settings"),
        Some(
            TransitionConfig::preset_smooth_fade()
                .with_animation_id("fade-to-settings")
                .with_custom_property("spring_tension", 0.6),
        ),
    )?;

    // Simulate transition updates
    simulate_transition_updates(screen_manager)?;

    // Slide to help with smooth slide easing
    println!("🔄 Smooth slide transition to help...");
    screen_manager.switch_to_with_transition(
        ScreenId::new("help"),
        Some(
            TransitionConfig::preset_quick_slide()
                .with_animation_id("slide-to-help")
                .with_custom_property("slide_smoothness", 0.9),
        ),
    )?;

    simulate_transition_updates(screen_manager)?;

    // Scale to dashboard with bouncy animation
    println!("🔄 Bouncy scale transition to dashboard...");
    screen_manager.switch_to_with_transition(
        ScreenId::new("dashboard"),
        Some(
            TransitionConfig::preset_bouncy_scale()
                .with_animation_id("scale-to-dashboard")
                .with_hardware_acceleration()
                .with_custom_property("bounce_intensity", 1.2),
        ),
    )?;

    simulate_transition_updates(screen_manager)?;

    println!("✅ Transition demonstrations complete");
    Ok(())
}

fn simulate_transition_updates(screen_manager: &mut ScreenManager) -> Result<(), String> {
    // Simulate transition updates over time
    let mut updates = 0;
    while screen_manager.is_transitioning() && updates < 20 {
        screen_manager.update()?;
        std::thread::sleep(Duration::from_millis(50));
        updates += 1;
    }

    // Ensure transition completes
    screen_manager.update()?;
    Ok(())
}

fn demonstrate_screen_updates(screen_manager: &mut ScreenManager) -> Result<(), String> {
    println!("\n4. Demonstrating screen updates...");

    // Update dashboard with new data
    let updated_dashboard = Element::fragment()
        .child(Element::text("=== DASHBOARD ==="))
        .child(Element::text(""))
        .child(Element::text("📊 System Dashboard (UPDATED)"))
        .child(Element::text(""))
        .child(Element::text("Status: ⚠️  High CPU usage detected"))
        .child(Element::text(""))
        .child(Element::text("Metrics:"))
        .child(Element::text("• CPU Usage: 89% ⚠️"))
        .child(Element::text("• Memory: 6.8GB / 8GB"))
        .child(Element::text("• Disk: 125GB / 500GB"))
        .child(Element::text("• Network: 8 Mbps"))
        .child(Element::text(""))
        .child(Element::text("Last updated: 2 seconds ago"));

    screen_manager.update_screen(&ScreenId::new("dashboard"), updated_dashboard)?;
    println!("✅ Dashboard updated with new metrics");

    // Switch back to main and then to updated dashboard
    screen_manager.switch_to(ScreenId::new("main"))?;
    simulate_transition_updates(screen_manager)?;

    screen_manager.switch_to(ScreenId::new("dashboard"))?;
    simulate_transition_updates(screen_manager)?;

    println!("✅ Screen update demonstration complete");
    Ok(())
}

fn demonstrate_event_handling(screen_manager: &mut ScreenManager) -> Result<(), String> {
    println!("\n5. Demonstrating event handling...");

    // Simulate hotkey events
    let events = vec![
        rt_event::Event::Key(rt_event::KeyEvent::new(rt_event::KeyCode::F(1))),
        rt_event::Event::Key(rt_event::KeyEvent::new(rt_event::KeyCode::F(2))),
        rt_event::Event::Key(rt_event::KeyEvent::new(rt_event::KeyCode::F(3))),
        rt_event::Event::Key(rt_event::KeyEvent::new(rt_event::KeyCode::F(4))),
    ];

    for event in events {
        if let rt_event::Event::Key(ref key_event) = event {
            println!("🎹 Simulating hotkey: {:?}", key_event.code);
        }

        let handled = screen_manager.process_event(&event)?;
        if handled {
            println!("✅ Event handled by screen manager");
            simulate_transition_updates(screen_manager)?;
        } else {
            println!("➡️  Event passed through to active screen");
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("✅ Event handling demonstration complete");
    Ok(())
}
