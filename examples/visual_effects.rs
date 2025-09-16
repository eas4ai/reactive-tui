use reactive_tui::{
    app::RootComponent,
    builder::{div, to_element},
    component::Element,
};

/// Visual effects demonstration - shows various visual patterns and effects
pub struct VisualEffects;

impl VisualEffects {
    /// Create a checkerboard pattern
    fn create_checkerboard(&self, x: usize, y: usize, size: usize) -> Vec<Element> {
        let mut elements = Vec::new();
        
        for row in 0..size {
            for col in 0..size {
                let is_dark = (row + col) % 2 == 0;
                let color = if is_dark { (50, 50, 50) } else { (200, 200, 200) };
                
                elements.push(
                    div()
                        .class(&format!("absolute left-{} top-{} w-1 h-1", x + col, y + row))
                        .class(&format!("bg-[rgb({},{},{})]", color.0, color.1, color.2))
                        .build()
                );
            }
        }
        
        elements
    }
    
    /// Create a spiral pattern
    fn create_spiral(&self, center_x: usize, center_y: usize, size: usize) -> Vec<Element> {
        let mut elements = Vec::new();
        let center = size as f32 / 2.0;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = (y as f32 - center) * 2.0; // Adjust for terminal aspect ratio
                let distance = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);
                
                let spiral_value = (distance * 0.3 - angle * 2.0).sin();
                if spiral_value > 0.3 {
                    let intensity = (spiral_value * 255.0) as u8;
                    elements.push(
                        div()
                            .class(&format!("absolute left-{} top-{} w-1 h-1", center_x + x, center_y + y))
                            .class(&format!("bg-[rgb({},0,{})]", intensity, 255 - intensity))
                            .build()
                    );
                }
            }
        }
        
        elements
    }
    
    /// Create a mandala pattern
    fn create_mandala(&self, center_x: usize, center_y: usize, size: usize) -> Vec<Element> {
        let mut elements = Vec::new();
        let center = size as f32 / 2.0;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = (y as f32 - center) * 2.0;
                let distance = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);
                
                let pattern = (distance * 0.5).sin() * (angle * 6.0).sin();
                if pattern > 0.5 {
                    let hue = angle + distance * 0.1;
                    let r = ((hue).sin() * 127.0 + 128.0) as u8;
                    let g = ((hue + 2.0).sin() * 127.0 + 128.0) as u8;
                    let b = ((hue + 4.0).sin() * 127.0 + 128.0) as u8;
                    
                    elements.push(
                        div()
                            .class(&format!("absolute left-{} top-{} w-1 h-1", center_x + x, center_y + y))
                            .class(&format!("bg-[rgb({},{},{})]", r, g, b))
                            .build()
                    );
                }
            }
        }
        
        elements
    }
    
    /// Create a plasma effect
    fn create_plasma(&self, x: usize, y: usize, width: usize, height: usize) -> Vec<Element> {
        let mut elements = Vec::new();
        
        for row in 0..height {
            for col in 0..width {
                let fx = col as f32 / width as f32;
                let fy = row as f32 / height as f32;
                
                let plasma = (fx * 10.0).sin() + (fy * 10.0).sin() + 
                           ((fx * fx + fy * fy).sqrt() * 10.0).sin();
                
                let normalized = (plasma + 3.0) / 6.0;
                let r = (normalized * 255.0) as u8;
                let g = ((normalized * 2.0) % 1.0 * 255.0) as u8;
                let b = ((normalized * 3.0) % 1.0 * 255.0) as u8;
                
                elements.push(
                    div()
                        .class(&format!("absolute left-{} top-{} w-1 h-1", x + col, y + row))
                        .class(&format!("bg-[rgb({},{},{})]", r, g, b))
                        .build()
                );
            }
        }
        
        elements
    }
    
    /// Create a geometric pattern
    fn create_geometric(&self, x: usize, y: usize, size: usize) -> Vec<Element> {
        let mut elements = Vec::new();
        
        for row in 0..size {
            for col in 0..size {
                let pattern = (row ^ col) % 8;
                let color = match pattern {
                    0 => (255, 0, 0),     // Red
                    1 => (255, 127, 0),   // Orange
                    2 => (255, 255, 0),   // Yellow
                    3 => (0, 255, 0),     // Green
                    4 => (0, 255, 255),   // Cyan
                    5 => (0, 0, 255),     // Blue
                    6 => (127, 0, 255),   // Purple
                    _ => (255, 0, 255),   // Magenta
                };
                
                elements.push(
                    div()
                        .class(&format!("absolute left-{} top-{} w-1 h-1", x + col, y + row))
                        .class(&format!("bg-[rgb({},{},{})]", color.0, color.1, color.2))
                        .build()
                );
            }
        }
        
        elements
    }
}

impl RootComponent for VisualEffects {
    fn render(&self) -> Element {
        let mut elements = Vec::new();
        
        // Checkerboard pattern
        elements.extend(self.create_checkerboard(5, 3, 12));
        
        // Spiral pattern
        elements.extend(self.create_spiral(25, 3, 15));
        
        // Mandala pattern
        elements.extend(self.create_mandala(45, 3, 15));
        
        // Plasma effect
        elements.extend(self.create_plasma(65, 3, 20, 12));
        
        // Geometric pattern
        elements.extend(self.create_geometric(5, 20, 16));
        
        // Another geometric pattern
        elements.extend(self.create_geometric(25, 20, 16));

        div()
            .class("w-screen h-screen bg-black")
            .children(elements)
            .child(
                div()
                    .class("absolute left-5 top-1 text-white")
                    .child(to_element("✨ Visual Effects & Patterns"))
                    .build()
            )
            .child(
                div()
                    .class("absolute bottom-1 left-5 text-gray-400")
                    .child(to_element("Press CTRL+Q to quit"))
                    .build()
            )
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    let backend = reactive_tui::backend::CrosstermBackend::new()?;
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(VisualEffects)
        .build()?;

    app.run()
}
