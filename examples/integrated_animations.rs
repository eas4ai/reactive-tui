//! Example demonstrating the integrated animation system
//!
//! This example shows how both hook-based animations and declarative animations
//! now work together through the main render loop without separate threads.

use reactive_tui::animation::{
    fade_in, AnimatedProperty, AnimationBuilder, EasingFunction, LoopMode,
};
use reactive_tui::backend::CrosstermBackend;
use reactive_tui::prelude::*;
use std::time::Duration;

struct AnimatedApp {
    counter: i32,
}

impl reactive_tui::app::RootComponent for AnimatedApp {
    fn render(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .class("flex-col items-center justify-center h-full bg-gray-900")
            .children(vec![
                Element::text("🎬 Integrated Animation System")
                    .class("text-2xl text-blue-400 mb-4"),
                Element::text(format!("Counter: {}", self.counter))
                    .class("text-xl text-white mb-2"),
                Element::text("Both hook-based and declarative animations")
                    .class("text-gray-400 mb-2"),
                Element::text("are now coordinated through the main render loop!")
                    .class("text-gray-400 mb-4"),
                Element::text("Press Ctrl+C or Esc to exit").class("text-sm text-gray-600"),
            ])
    }
}

fn main() -> reactive_tui::Result<()> {
    // Create the application
    let backend = CrosstermBackend::new()?;
    let mut app = App::builder()
        .backend(backend)
        .root(AnimatedApp { counter: 0 })
        .debug(true)
        .build()?;

    // Add some declarative animations to demonstrate integration
    let fade_animation = fade_in("main-fade", Duration::from_millis(1000));
    let _fade_id = app.animation_manager().add_animation(fade_animation);

    // Add a custom animation
    let pulse_animation = AnimationBuilder::new("pulse")
        .animate_property(AnimatedProperty::Scale(1.0, 1.2))
        .duration(Duration::from_millis(800))
        .easing(EasingFunction::EaseInOut)
        .auto_reverse(true)
        .loop_mode(LoopMode::Infinite)
        .build();
    let _pulse_id = app.animation_manager().add_animation(pulse_animation);

    println!("🎬 Starting integrated animation system demo...");
    println!(
        "📊 Active animations: {}",
        app.animation_manager_ref().active_count()
    );

    // Run the application - animations will be updated in the main loop
    app.run()
}
