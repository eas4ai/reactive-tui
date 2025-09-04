//! Gradient Borders Demo
//! 
//! Demonstrates gradient borders inspired by modern UI frameworks.
//! Shows animated rainbow borders, conic gradients, and various border effects.

use reactive_tui::prelude::*;
use reactive_tui::backend::CrosstermBackend;
use reactive_tui::builder::{div, to_element};
use reactive_tui::layout::css::gradients::{Gradient, GradientDirection, GradientBorder};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Gradient borders showcase
struct GradientBordersDemo {
    start_time: Instant,
    frame_count: Arc<Mutex<u64>>,
}

impl GradientBordersDemo {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            frame_count: Arc::new(Mutex::new(0)),
        }
    }
    
    fn create_gradient_border_box(&self, 
        title: &str,
        width: usize, 
        height: usize,
        gradient_colors: Vec<(u8, u8, u8)>,
        animated: bool,
    ) -> Element {
        let mut elements = Vec::new();
        
        // Create gradient border
        let gradient_border = if animated {
            // Animate by rotating the gradient based on time
            let elapsed = self.start_time.elapsed().as_secs_f32();
            let offset = (elapsed * 0.5).sin() * 0.5 + 0.5; // Oscillate between 0 and 1
            
            let mut gradient = Gradient::new(GradientDirection::ToRight);
            // Shift colors based on animation
            let shifted_colors: Vec<(u8, u8, u8)> = gradient_colors.iter().enumerate().map(|(_i, &color)| {
                let hue_shift = offset * 360.0;
                shift_hue(color, hue_shift as i32)
            }).collect();
            
            gradient.stops.from = Some((shifted_colors[0].0, shifted_colors[0].1, shifted_colors[0].2, 1.0));
            if shifted_colors.len() > 2 {
                gradient.stops.via = Some((shifted_colors[1].0, shifted_colors[1].1, shifted_colors[1].2, 1.0));
                gradient.stops.to = Some((shifted_colors[2].0, shifted_colors[2].1, shifted_colors[2].2, 1.0));
            } else if shifted_colors.len() == 2 {
                gradient.stops.to = Some((shifted_colors[1].0, shifted_colors[1].1, shifted_colors[1].2, 1.0));
            }
            
            GradientBorder::new(gradient, 1)
        } else {
            let mut gradient = Gradient::new(GradientDirection::ToRight);
            gradient.stops.from = Some((gradient_colors[0].0, gradient_colors[0].1, gradient_colors[0].2, 1.0));
            if gradient_colors.len() > 2 {
                gradient.stops.via = Some((gradient_colors[1].0, gradient_colors[1].1, gradient_colors[1].2, 1.0));
                gradient.stops.to = Some((gradient_colors[2].0, gradient_colors[2].1, gradient_colors[2].2, 1.0));
            } else if gradient_colors.len() == 2 {
                gradient.stops.to = Some((gradient_colors[1].0, gradient_colors[1].1, gradient_colors[1].2, 1.0));
            }
            GradientBorder::new(gradient, 1)
        };
        
        // Get border colors
        let border_colors = gradient_border.render_border(width, height);
        
        if border_colors.len() >= 4 {
            let top = &border_colors[0];
            let right = &border_colors[1];
            let bottom = &border_colors[2];
            let left = &border_colors[3];
            
            // Top border
            for (x, color) in top.iter().enumerate() {
                elements.push(
                    div()
                        .class(&format!("absolute left-{} top-0", x))
                        .class(&format!("text-[rgb({},{},{})]", 
                            color.0,
                            color.1,
                            color.2))
                        .child(to_element("─"))
                        .build()
                );
            }
            
            // Bottom border
            for (x, color) in bottom.iter().enumerate() {
                elements.push(
                    div()
                        .class(&format!("absolute left-{} top-{}", x, height - 1))
                        .class(&format!("text-[rgb({},{},{})]", 
                            color.0,
                            color.1,
                            color.2))
                        .child(to_element("─"))
                        .build()
                );
            }
            
            // Left border
            for (y, color) in left.iter().enumerate().skip(1).take(height.saturating_sub(2)) {
                elements.push(
                    div()
                        .class(&format!("absolute left-0 top-{}", y))
                        .class(&format!("text-[rgb({},{},{})]", 
                            color.0,
                            color.1,
                            color.2))
                        .child(to_element("│"))
                        .build()
                );
            }
            
            // Right border
            for (y, color) in right.iter().enumerate().skip(1).take(height.saturating_sub(2)) {
                elements.push(
                    div()
                        .class(&format!("absolute left-{} top-{}", width - 1, y))
                        .class(&format!("text-[rgb({},{},{})]", 
                            color.0,
                            color.1,
                            color.2))
                        .child(to_element("│"))
                        .build()
                );
            }
            
            // Corners with special characters
            // Top-left
            if let Some(color) = top.first() {
                elements.push(
                    div()
                        .class("absolute left-0 top-0")
                        .class(&format!("text-[rgb({},{},{})]", 
                            color.0,
                            color.1,
                            color.2))
                        .child(to_element("╭"))
                        .build()
                );
            }
            
            // Top-right
            if let Some(color) = top.last() {
                elements.push(
                    div()
                        .class(&format!("absolute left-{} top-0", width - 1))
                        .class(&format!("text-[rgb({},{},{})]", 
                            color.0,
                            color.1,
                            color.2))
                        .child(to_element("╮"))
                        .build()
                );
            }
            
            // Bottom-left
            if let Some(color) = bottom.last() {
                elements.push(
                    div()
                        .class(&format!("absolute left-0 top-{}", height - 1))
                        .class(&format!("text-[rgb({},{},{})]", 
                            color.0,
                            color.1,
                            color.2))
                        .child(to_element("╰"))
                        .build()
                );
            }
            
            // Bottom-right
            if let Some(color) = bottom.first() {
                elements.push(
                    div()
                        .class(&format!("absolute left-{} top-{}", width - 1, height - 1))
                        .class(&format!("text-[rgb({},{},{})]", 
                            color.0,
                            color.1,
                            color.2))
                        .child(to_element("╯"))
                        .build()
                );
            }
        }
        
        // Add title in center
        elements.push(
            div()
                .class(&format!("absolute left-{} top-{} text-white", 
                    width / 2 - title.len() / 2, 
                    height / 2))
                .child(to_element(title.to_string()))
                .build()
        );
        
        div()
            .class("relative")
            .children(elements)
            .build()
    }
}

/// Simple HSL to RGB conversion for hue shifting
fn shift_hue(rgb: (u8, u8, u8), hue_shift: i32) -> (u8, u8, u8) {
    // Convert RGB to HSL
    let r = rgb.0 as f32 / 255.0;
    let g = rgb.1 as f32 / 255.0;
    let b = rgb.2 as f32 / 255.0;
    
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    
    let l = (max + min) / 2.0;
    
    if delta == 0.0 {
        return rgb; // Gray color, no hue to shift
    }
    
    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };
    
    let mut h = if max == r {
        ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };
    
    // Apply hue shift
    h = (h + hue_shift as f32 / 360.0) % 1.0;
    if h < 0.0 {
        h += 1.0;
    }
    
    // Convert back to RGB
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

impl reactive_tui::app::RootComponent for GradientBordersDemo {
    fn render(&self) -> Element {
        // Update frame count for animation
        let mut frame = self.frame_count.lock().unwrap();
        *frame += 1;
        
        div()
            .class("w-screen h-screen bg-gray-900 p-4")
            .children(vec![
                // Title
                div()
                    .class("text-white text-2xl font-bold mb-4 text-center")
                    .child(to_element("✨ Gradient Borders Demo ✨"))
                    .build(),
                
                // Grid of gradient border examples
                div()
                    .class("grid grid-cols-2 gap-4")
                    .children(vec![
                        // Rainbow animated border
                        self.create_gradient_border_box(
                            "Rainbow Animated",
                            30, 10,
                            vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)],
                            true
                        ),
                        
                        // Cyan to Purple gradient
                        self.create_gradient_border_box(
                            "Cyan → Purple",
                            30, 10,
                            vec![(0, 206, 209), (147, 51, 234)],
                            false
                        ),
                        
                        // Sunset gradient
                        self.create_gradient_border_box(
                            "Sunset",
                            30, 10,
                            vec![(251, 146, 60), (250, 82, 82), (155, 52, 239)],
                            false
                        ),
                        
                        // Ocean gradient (animated)
                        self.create_gradient_border_box(
                            "Ocean Wave",
                            30, 10,
                            vec![(56, 189, 248), (59, 130, 246), (99, 102, 241)],
                            true
                        ),
                    ])
                    .build(),
                
                // Footer
                div()
                    .class("text-gray-400 mt-4 text-center")
                    .child(to_element("Press Q or ESC to quit"))
                    .build(),
            ])
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("✨ Starting Gradient Borders Demo...");
    println!("Press Q or ESC to quit");
    
    let backend = CrosstermBackend::new()?;
    let demo = GradientBordersDemo::new();
    
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(demo)
        .build()?;
    
    app.run()
}