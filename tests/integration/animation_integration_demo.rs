use reactive_tui::backend::DebugBackend;
use reactive_tui::component::Element;
use reactive_tui::screen::{
    EasingFunction, ScreenId, ScreenManager, TransitionConfig, TransitionType,
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Animation Integration Demo ===");
    println!("Showcasing enhanced easing functions and animation system integration");

    let backend = Box::new(DebugBackend::new(80, 24));
    let mut screen_manager = ScreenManager::new(backend);

    // Create screens with different animation personalities
    create_animation_demo_screens(&mut screen_manager)?;

    // Demonstrate all the new easing functions
    demonstrate_easing_functions(&mut screen_manager)?;

    // Show animation system integration features
    demonstrate_animation_integration(&mut screen_manager)?;

    // Performance and debugging features
    demonstrate_debugging_features(&mut screen_manager)?;

    println!("\n=== Animation Integration Demo Complete ===");
    Ok(())
}

fn create_animation_demo_screens(screen_manager: &mut ScreenManager) -> Result<(), String> {
    println!("\n1. Creating screens with animation personalities...");

    // Smooth screen - for gentle transitions
    let smooth_screen = Element::fragment()
        .child(Element::text("=== SMOOTH SCREEN ==="))
        .child(Element::text(""))
        .child(Element::text(
            "🌊 This screen uses gentle, smooth animations",
        ))
        .child(Element::text(
            "Perfect for: Fade transitions, subtle movements",
        ))
        .child(Element::text("Easing: SpringGentle, EaseOutCubic"))
        .child(Element::text(""))
        .child(Element::text("Animation ID: smooth-screen-transitions"))
        .child(Element::text("Hardware Acceleration: Enabled"));

    screen_manager.create_screen("smooth", "Smooth".to_string(), smooth_screen)?;

    // Bouncy screen - for playful transitions
    let bouncy_screen = Element::fragment()
        .child(Element::text("=== BOUNCY SCREEN ==="))
        .child(Element::text(""))
        .child(Element::text(
            "🏀 This screen uses bouncy, playful animations",
        ))
        .child(Element::text(
            "Perfect for: Scale transitions, attention-grabbing",
        ))
        .child(Element::text("Easing: SnapBounce, EaseOutBounce"))
        .child(Element::text(""))
        .child(Element::text("Animation ID: bouncy-screen-transitions"))
        .child(Element::text("Custom Properties: bounce_intensity=1.5"));

    screen_manager.create_screen("bouncy", "Bouncy".to_string(), bouncy_screen)?;

    // Elastic screen - for dramatic transitions
    let elastic_screen = Element::fragment()
        .child(Element::text("=== ELASTIC SCREEN ==="))
        .child(Element::text(""))
        .child(Element::text(
            "🎯 This screen uses elastic, dramatic animations",
        ))
        .child(Element::text(
            "Perfect for: Slide transitions, dynamic effects",
        ))
        .child(Element::text("Easing: EaseOutElastic, SpringDramatic"))
        .child(Element::text(""))
        .child(Element::text("Animation ID: elastic-screen-transitions"))
        .child(Element::text(
            "Custom Properties: elasticity=0.8, tension=1.2",
        ));

    screen_manager.create_screen("elastic", "Elastic".to_string(), elastic_screen)?;

    // Performance screen - for optimized transitions
    let performance_screen = Element::fragment()
        .child(Element::text("=== PERFORMANCE SCREEN ==="))
        .child(Element::text(""))
        .child(Element::text(
            "⚡ This screen uses performance-optimized animations",
        ))
        .child(Element::text(
            "Perfect for: Fast transitions, mobile devices",
        ))
        .child(Element::text("Easing: Linear, SlideSmooth"))
        .child(Element::text(""))
        .child(Element::text(
            "Animation ID: performance-screen-transitions",
        ))
        .child(Element::text("Hardware Acceleration: Enabled"))
        .child(Element::text("Custom Properties: performance_mode=true"));

    screen_manager.create_screen("performance", "Performance".to_string(), performance_screen)?;

    println!("✅ Created 4 screens with different animation personalities");
    Ok(())
}

fn demonstrate_easing_functions(screen_manager: &mut ScreenManager) -> Result<(), String> {
    println!("\n2. Demonstrating enhanced easing functions...");

    let easing_demos = vec![
        (
            "SpringGentle",
            EasingFunction::SpringGentle,
            "Gentle spring motion - perfect for subtle fades",
        ),
        (
            "SpringDramatic",
            EasingFunction::SpringDramatic,
            "Dramatic spring - attention-grabbing transitions",
        ),
        (
            "SlideSmooth",
            EasingFunction::SlideSmooth,
            "Smooth slide with overshoot - optimized for screen slides",
        ),
        (
            "SnapBounce",
            EasingFunction::SnapBounce,
            "Quick snap with bounce - great for scale effects",
        ),
        (
            "EaseOutElastic",
            EasingFunction::EaseOutElastic,
            "Elastic overshoot - dynamic and playful",
        ),
        (
            "EaseInElastic",
            EasingFunction::EaseInElastic,
            "Elastic wind-up - builds anticipation",
        ),
    ];

    for (name, easing, description) in easing_demos {
        println!("🎨 Testing {}: {}", name, description);

        let config = TransitionConfig {
            transition_type: TransitionType::Fade,
            duration: Duration::from_millis(400),
            easing,
            animation_id: Some(format!("demo-{}", name.to_lowercase())),
            use_hardware_acceleration: true,
            custom_properties: {
                let mut props = std::collections::HashMap::new();
                props.insert("demo_mode".to_string(), 1.0);
                props.insert("easing_showcase".to_string(), 1.0);
                props
            },
        };

        println!("   📊 {}", config.description());

        // Test the easing function at different points
        let test_points = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let values: Vec<String> = test_points
            .iter()
            .map(|&t| format!("{:.3}", easing.apply(t)))
            .collect();
        println!("   📈 Curve: [{}]", values.join(", "));

        // Quick transition to show the easing
        screen_manager.switch_to_with_transition(ScreenId::new("smooth"), Some(config))?;
        simulate_transition_updates(screen_manager)?;

        std::thread::sleep(Duration::from_millis(50));
    }

    println!("✅ Easing function demonstrations complete");
    Ok(())
}

fn demonstrate_animation_integration(screen_manager: &mut ScreenManager) -> Result<(), String> {
    println!("\n3. Demonstrating animation system integration features...");

    // Preset configurations
    println!("🎯 Testing preset configurations...");

    let presets = vec![
        ("Smooth Fade", TransitionConfig::preset_smooth_fade()),
        ("Quick Slide", TransitionConfig::preset_quick_slide()),
        ("Bouncy Scale", TransitionConfig::preset_bouncy_scale()),
        ("Dramatic Flip", TransitionConfig::preset_dramatic_flip()),
    ];

    for (name, mut config) in presets {
        println!("   🎨 {}: {}", name, config.description());

        // Add animation system integration
        config = config
            .with_animation_id(format!("preset-{}", name.to_lowercase().replace(" ", "-")))
            .with_custom_property("preset_demo", 1.0);

        if name.contains("Scale") || name.contains("Flip") {
            config = config.with_hardware_acceleration();
        }

        screen_manager.switch_to_with_transition(ScreenId::new("bouncy"), Some(config))?;
        simulate_transition_updates(screen_manager)?;
    }

    // Custom animation properties
    println!("🔧 Testing custom animation properties...");

    let custom_config = TransitionConfig::default()
        .with_animation_id("custom-demo")
        .with_hardware_acceleration()
        .with_custom_property("spring_tension", 0.8)
        .with_custom_property("damping_ratio", 0.6)
        .with_custom_property("velocity_threshold", 0.01)
        .with_custom_property("performance_mode", 1.0);

    println!(
        "   🎛️  Custom properties: {:?}",
        custom_config.custom_properties
    );
    println!(
        "   ⚡ Hardware acceleration: {}",
        custom_config.use_hardware_acceleration
    );

    screen_manager.switch_to_with_transition(ScreenId::new("elastic"), Some(custom_config))?;
    simulate_transition_updates(screen_manager)?;

    println!("✅ Animation integration demonstrations complete");
    Ok(())
}

fn demonstrate_debugging_features(screen_manager: &mut ScreenManager) -> Result<(), String> {
    println!("\n4. Demonstrating debugging and performance features...");

    // Easing function recommendations
    println!("💡 Easing function recommendations:");

    let transition_types = vec![
        TransitionType::Fade,
        TransitionType::SlideLeft,
        TransitionType::Scale,
        TransitionType::Flip,
    ];

    for transition_type in transition_types {
        let recommendations = EasingFunction::recommended_for_transition(transition_type);
        println!("   {:?}:", transition_type);
        for easing in recommendations {
            println!(
                "     - {} ({})",
                easing.description(),
                format!("{:?}", easing)
            );
        }
    }

    // Performance comparison
    println!("\n⚡ Performance comparison:");

    let performance_configs = vec![
        (
            "Optimized",
            TransitionConfig {
                transition_type: TransitionType::Fade,
                duration: Duration::from_millis(200),
                easing: EasingFunction::Linear,
                animation_id: Some("perf-optimized".to_string()),
                use_hardware_acceleration: true,
                custom_properties: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("performance_mode".to_string(), 1.0);
                    props
                },
            },
        ),
        (
            "Smooth",
            TransitionConfig {
                transition_type: TransitionType::Fade,
                duration: Duration::from_millis(300),
                easing: EasingFunction::EaseOutCubic,
                animation_id: Some("perf-smooth".to_string()),
                use_hardware_acceleration: false,
                custom_properties: std::collections::HashMap::new(),
            },
        ),
        (
            "Dramatic",
            TransitionConfig {
                transition_type: TransitionType::Scale,
                duration: Duration::from_millis(600),
                easing: EasingFunction::SpringDramatic,
                animation_id: Some("perf-dramatic".to_string()),
                use_hardware_acceleration: true,
                custom_properties: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("spring_intensity".to_string(), 1.5);
                    props
                },
            },
        ),
    ];

    for (name, config) in performance_configs {
        println!("   🎯 {}: {}", name, config.description());
        println!("      Hardware Accel: {}", config.use_hardware_acceleration);
        println!("      Animation ID: {:?}", config.animation_id);

        let start_time = std::time::Instant::now();
        screen_manager.switch_to_with_transition(ScreenId::new("performance"), Some(config))?;
        simulate_transition_updates(screen_manager)?;
        let elapsed = start_time.elapsed();

        println!("      Transition Time: {:?}", elapsed);
    }

    println!("✅ Debugging and performance demonstrations complete");
    Ok(())
}

fn simulate_transition_updates(screen_manager: &mut ScreenManager) -> Result<(), String> {
    let mut updates = 0;
    while screen_manager.is_transitioning() && updates < 30 {
        screen_manager.update()?;
        std::thread::sleep(Duration::from_millis(20));
        updates += 1;
    }
    screen_manager.update()?; // Final update
    Ok(())
}
