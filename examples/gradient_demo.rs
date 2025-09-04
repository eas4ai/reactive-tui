//! Gradient Demo
//! 
//! Demonstrates CSS gradient rendering in the terminal using reactive-tui's
//! gradient system. Shows various gradient directions and color combinations.

use reactive_tui::prelude::*;
use reactive_tui::backend::CrosstermBackend;
use reactive_tui::builder::{div, to_element};
use reactive_tui::layout::css::gradients::{Gradient, GradientDirection};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Gradient demo showcase
struct GradientDemo {
    _current_gradient: Arc<Mutex<usize>>,
    _start_time: Instant,
}

impl GradientDemo {
    fn new() -> Self {
        Self {
            _current_gradient: Arc::new(Mutex::new(0)),
            _start_time: Instant::now(),
        }
    }
    
    fn create_gradient_block(&self, 
        direction: GradientDirection,
        from: (u8, u8, u8),
        via: Option<(u8, u8, u8)>,
        to: (u8, u8, u8),
        title: &str,
        width: usize,
        height: usize
    ) -> Element {
        let mut gradient = Gradient::new(direction);
        gradient.stops.from = Some((from.0, from.1, from.2, 1.0));
        if let Some(v) = via {
            gradient.stops.via = Some((v.0, v.1, v.2, 1.0));
        }
        gradient.stops.to = Some((to.0, to.1, to.2, 1.0));
        
        let mut elements = Vec::new();
        
        // Add title
        elements.push(
            div()
                .class("absolute top-0 left-2 text-white font-bold z-10")
                .child(to_element(title.to_string()))
                .build()
        );
        
        // Render gradient based on direction
        match direction {
            GradientDirection::ToRight | GradientDirection::ToLeft => {
                // Horizontal gradient
                let colors = gradient.render(width);
                let colors = if matches!(direction, GradientDirection::ToLeft) {
                    colors.into_iter().rev().collect()
                } else {
                    colors
                };
                
                for (x, color) in colors.iter().enumerate() {
                    for y in 0..height {
                        elements.push(
                            div()
                                .class(&format!("absolute left-{} top-{}", x, y))
                                .class(&format!("bg-[rgb({},{},{})]", color.0, color.1, color.2))
                                .child(to_element(" "))
                                .build()
                        );
                    }
                }
            }
            GradientDirection::ToBottom | GradientDirection::ToTop => {
                // Vertical gradient
                let colors = gradient.render(height);
                let colors = if matches!(direction, GradientDirection::ToTop) {
                    colors.into_iter().rev().collect()
                } else {
                    colors
                };
                
                for (y, color) in colors.iter().enumerate() {
                    for x in 0..width {
                        elements.push(
                            div()
                                .class(&format!("absolute left-{} top-{}", x, y))
                                .class(&format!("bg-[rgb({},{},{})]", color.0, color.1, color.2))
                                .child(to_element(" "))
                                .build()
                        );
                    }
                }
            }
            GradientDirection::ToTopRight | GradientDirection::ToBottomLeft => {
                // Diagonal gradient (top-right or bottom-left)
                for y in 0..height {
                    for x in 0..width {
                        // Calculate position along diagonal
                        let pos = ((x as f32 / width as f32) + (y as f32 / height as f32)) / 2.0;
                        let pos = if matches!(direction, GradientDirection::ToBottomLeft) {
                            1.0 - pos
                        } else {
                            pos
                        };
                        
                        if let Some(color) = gradient.color_at(pos) {
                            elements.push(
                                div()
                                    .class(&format!("absolute left-{} top-{}", x, y))
                                    .class(&format!("bg-[rgb({},{},{})]", 
                                        (color.0 as f32 * 255.0) as u8,
                                        (color.1 as f32 * 255.0) as u8,
                                        (color.2 as f32 * 255.0) as u8))
                                    .child(to_element(" "))
                                    .build()
                            );
                        }
                    }
                }
            }
            _ => {
                // Other diagonal gradients
                for y in 0..height {
                    for x in 0..width {
                        let pos = match direction {
                            GradientDirection::ToTopLeft => {
                                1.0 - ((x as f32 / width as f32) + (y as f32 / height as f32)) / 2.0
                            }
                            GradientDirection::ToBottomRight => {
                                ((x as f32 / width as f32) + (1.0 - y as f32 / height as f32)) / 2.0
                            }
                            _ => 0.5
                        };
                        
                        if let Some(color) = gradient.color_at(pos) {
                            elements.push(
                                div()
                                    .class(&format!("absolute left-{} top-{}", x, y))
                                    .class(&format!("bg-[rgb({},{},{})]", 
                                        (color.0 as f32 * 255.0) as u8,
                                        (color.1 as f32 * 255.0) as u8,
                                        (color.2 as f32 * 255.0) as u8))
                                    .child(to_element(" "))
                                    .build()
                            );
                        }
                    }
                }
            }
        }
        
        div()
            .class("relative border border-gray-600")
            .children(elements)
            .build()
    }
}

impl reactive_tui::app::RootComponent for GradientDemo {
    fn render(&self) -> Element {
        // Create different gradient examples
        let examples = vec![
            // Horizontal gradients
            self.create_gradient_block(
                GradientDirection::ToRight,
                (0, 206, 209),     // cyan-500
                Some((59, 130, 246)), // blue-500
                (99, 102, 241),    // indigo-500
                "Horizontal: Cyan → Blue → Indigo",
                40, 8
            ),
            
            // Vertical gradient
            self.create_gradient_block(
                GradientDirection::ToBottom,
                (239, 68, 68),     // red-500
                Some((245, 158, 11)), // amber-500
                (34, 197, 94),     // green-500
                "Vertical: Red → Amber → Green",
                40, 8
            ),
            
            // Diagonal gradient
            self.create_gradient_block(
                GradientDirection::ToTopRight,
                (168, 85, 247),    // purple-500
                None,
                (236, 72, 153),    // pink-500
                "Diagonal: Purple → Pink",
                40, 8
            ),
            
            // Reverse gradient
            self.create_gradient_block(
                GradientDirection::ToLeft,
                (14, 165, 233),    // sky-500
                Some((34, 197, 94)), // green-500
                (245, 158, 11),    // amber-500
                "Reverse: Sky ← Green ← Amber",
                40, 8
            ),
        ];
        
        // Main container
        div()
            .class("w-screen h-screen bg-black p-4")
            .children(vec![
                // Title
                div()
                    .class("text-white text-2xl font-bold mb-4 text-center")
                    .child(to_element("🌈 CSS Gradients in Terminal"))
                    .build(),
                
                // Gradient examples
                div()
                    .class("flex flex-col gap-4")
                    .children(examples)
                    .build(),
                
                // Instructions
                div()
                    .class("text-gray-400 mt-4 text-center")
                    .child(to_element("Press Q or ESC to quit"))
                    .build(),
            ])
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("🌈 Starting Gradient Demo...");
    println!("Press Q or ESC to quit");
    
    let backend = CrosstermBackend::new()?;
    let demo = GradientDemo::new();
    
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(demo)
        .build()?;
    
    app.run()
}