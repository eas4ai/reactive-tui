use reactive_tui::{
    app::RootComponent,
    builder::{div, to_element},
    component::Element,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Animated visual patterns demonstration - shows moving visual elements
pub struct AnimatedPatterns;

impl AnimatedPatterns {
    /// Create a bouncing ball effect
    fn create_bouncing_ball(&self, time: f32, width: usize, height: usize) -> Element {
        let max_x = width.saturating_sub(5);
        let max_y = height.saturating_sub(5);

        let x = ((time * 2.0).sin() * (max_x as f32 / 2.0) + (max_x as f32 / 2.0)) as usize;
        let y = ((time * 3.0).sin().abs() * (max_y as f32 / 2.0) + 3.0) as usize;

        div()
            .class(&format!("absolute left-{} top-{} w-1 h-1", x, y))
            .class("bg-[rgb(255,100,100)]")
            .build()
    }

    /// Create a rotating pattern
    fn create_rotating_pattern(&self, time: f32, width: usize, height: usize) -> Vec<Element> {
        let mut elements = Vec::new();
        let center_x = width / 4;
        let center_y = height / 3;
        let radius = (width.min(height) / 8) as f32;

        for i in 0..8 {
            let angle = time + (i as f32 * std::f32::consts::PI / 4.0);
            let x = (center_x as f32 + angle.cos() * radius) as usize;
            let y = (center_y as f32 + angle.sin() * radius / 2.0) as usize;

            let color = match i % 3 {
                0 => (255, 0, 0), // Red
                1 => (0, 255, 0), // Green
                _ => (0, 0, 255), // Blue
            };

            elements.push(
                div()
                    .class(&format!("absolute left-{} top-{} w-1 h-1", x, y))
                    .class(&format!("bg-[rgb({},{},{})]", color.0, color.1, color.2))
                    .build(),
            );
        }

        elements
    }

    /// Create a wave pattern
    fn create_wave_pattern(&self, time: f32, width: usize, height: usize) -> Vec<Element> {
        let mut elements = Vec::new();

        let wave_width = (width * 3) / 4;
        let wave_start_x = width / 8;
        let wave_center_y = height / 2;
        let wave_amplitude = (height / 8).max(3) as f32;

        for x in 0..wave_width {
            let wave_y = ((x as f32 * 0.3 + time * 2.0).sin() * wave_amplitude
                + wave_center_y as f32) as usize;
            let intensity = ((x as f32 * 0.1 + time).sin() * 0.5 + 0.5) * 255.0;

            elements.push(
                div()
                    .class(&format!(
                        "absolute left-{} top-{} w-1 h-1",
                        wave_start_x + x,
                        wave_y
                    ))
                    .class(&format!("bg-[rgb(0,{},255)]", intensity as u8))
                    .build(),
            );
        }

        elements
    }

    /// Create a pulsing circle
    fn create_pulsing_circle(&self, time: f32, width: usize, height: usize) -> Vec<Element> {
        let mut elements = Vec::new();
        let center_x = (width * 3) / 4;
        let center_y = height / 4;
        let max_radius = (width.min(height) / 8) as f32;
        let pulse_radius = (time * 2.0).sin().abs() * max_radius;

        for y in 0..15 {
            for x in 0..15 {
                let dx = (x as f32 - 7.5).abs();
                let dy = (y as f32 - 7.5).abs() * 2.0; // Adjust for terminal aspect ratio
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= pulse_radius {
                    let intensity = (1.0 - distance / pulse_radius) * 255.0;
                    elements.push(
                        div()
                            .class(&format!(
                                "absolute left-{} top-{} w-1 h-1",
                                center_x + x,
                                center_y + y
                            ))
                            .class(&format!(
                                "bg-[rgb({},0,{})]",
                                intensity as u8,
                                255 - intensity as u8
                            ))
                            .build(),
                    );
                }
            }
        }

        elements
    }

    /// Create a color cycling pattern
    fn create_color_cycle(&self, time: f32, width: usize, height: usize) -> Vec<Element> {
        let mut elements = Vec::new();

        let grid_width = (width / 4).max(10);
        let grid_height = (height / 3).max(8);
        let start_x = width / 2;
        let start_y = (height * 2) / 3;

        for i in 0..grid_width {
            for j in 0..grid_height {
                let hue = (time + i as f32 * 0.1 + j as f32 * 0.05) % (2.0 * std::f32::consts::PI);
                let r = ((hue).sin() * 127.0 + 128.0) as u8;
                let g = ((hue + 2.0 * std::f32::consts::PI / 3.0).sin() * 127.0 + 128.0) as u8;
                let b = ((hue + 4.0 * std::f32::consts::PI / 3.0).sin() * 127.0 + 128.0) as u8;

                elements.push(
                    div()
                        .class(&format!(
                            "absolute left-{} top-{} w-1 h-1",
                            start_x + i,
                            start_y + j
                        ))
                        .class(&format!("bg-[rgb({},{},{})]", r, g, b))
                        .build(),
                );
            }
        }

        elements
    }
}

impl RootComponent for AnimatedPatterns {
    fn render(&self) -> Element {
        // Get current time for animation
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();

        // Get terminal dimensions for responsive layout
        let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let width = term_cols as usize;
        let height = term_rows as usize;

        let mut elements = Vec::new();

        // Add bouncing ball
        elements.push(self.create_bouncing_ball(time, width, height));

        // Add rotating pattern
        elements.extend(self.create_rotating_pattern(time, width, height));

        // Add wave pattern
        elements.extend(self.create_wave_pattern(time, width, height));

        // Add pulsing circle
        elements.extend(self.create_pulsing_circle(time, width, height));

        // Add color cycling pattern
        elements.extend(self.create_color_cycle(time, width, height));

        div()
            .class("w-screen h-screen bg-black")
            .children(elements)
            .child(
                div()
                    .class("absolute left-2 top-1 text-white")
                    .child(to_element("🌊 Animated Visual Patterns"))
                    .build(),
            )
            .child(
                div()
                    .class("absolute bottom-1 left-2 text-gray-400")
                    .child(to_element("Press CTRL+Q to quit"))
                    .build(),
            )
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    let backend = reactive_tui::backend::CrosstermBackend::new()?;
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(AnimatedPatterns)
        .build()?;

    app.run()
}
