use reactive_tui::{
    app::RootComponent,
    builder::{div, to_element},
    component::Element,
};

/// Visual gradient blocks demonstration - shows actual colored gradient rectangles
pub struct GradientBlocks;

impl GradientBlocks {
    /// Create a visual gradient block (colored rectangle)
    fn create_gradient_block(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        colors: Vec<(u8, u8, u8)>,
    ) -> Vec<Element> {
        let mut elements = Vec::new();

        for row in 0..height {
            for col in 0..width {
                // Calculate gradient position (0.0 to 1.0)
                let pos = col as f32 / (width - 1) as f32;

                // Interpolate between colors
                let color = if colors.len() >= 2 {
                    let segment_size = 1.0 / (colors.len() - 1) as f32;
                    let segment = (pos / segment_size).floor() as usize;
                    let local_pos = (pos % segment_size) / segment_size;

                    if segment >= colors.len() - 1 {
                        colors[colors.len() - 1]
                    } else {
                        let c1 = colors[segment];
                        let c2 = colors[segment + 1];
                        (
                            (c1.0 as f32 * (1.0 - local_pos) + c2.0 as f32 * local_pos) as u8,
                            (c1.1 as f32 * (1.0 - local_pos) + c2.1 as f32 * local_pos) as u8,
                            (c1.2 as f32 * (1.0 - local_pos) + c2.2 as f32 * local_pos) as u8,
                        )
                    }
                } else {
                    colors[0]
                };

                elements.push(
                    div()
                        .class(&format!(
                            "absolute left-{} top-{} w-1 h-1",
                            x + col,
                            y + row
                        ))
                        .class(&format!("bg-[rgb({},{},{})]", color.0, color.1, color.2))
                        .build(),
                );
            }
        }

        elements
    }

    /// Create a vertical gradient block
    fn create_vertical_gradient(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        colors: Vec<(u8, u8, u8)>,
    ) -> Vec<Element> {
        let mut elements = Vec::new();

        for row in 0..height {
            for col in 0..width {
                // Calculate gradient position (0.0 to 1.0) vertically
                let pos = row as f32 / (height - 1) as f32;

                // Interpolate between colors
                let color = if colors.len() >= 2 {
                    let segment_size = 1.0 / (colors.len() - 1) as f32;
                    let segment = (pos / segment_size).floor() as usize;
                    let local_pos = (pos % segment_size) / segment_size;

                    if segment >= colors.len() - 1 {
                        colors[colors.len() - 1]
                    } else {
                        let c1 = colors[segment];
                        let c2 = colors[segment + 1];
                        (
                            (c1.0 as f32 * (1.0 - local_pos) + c2.0 as f32 * local_pos) as u8,
                            (c1.1 as f32 * (1.0 - local_pos) + c2.1 as f32 * local_pos) as u8,
                            (c1.2 as f32 * (1.0 - local_pos) + c2.2 as f32 * local_pos) as u8,
                        )
                    }
                } else {
                    colors[0]
                };

                elements.push(
                    div()
                        .class(&format!(
                            "absolute left-{} top-{} w-1 h-1",
                            x + col,
                            y + row
                        ))
                        .class(&format!("bg-[rgb({},{},{})]", color.0, color.1, color.2))
                        .build(),
                );
            }
        }

        elements
    }
}

impl RootComponent for GradientBlocks {
    fn render(&self) -> Element {
        let mut elements = Vec::new();

        // Get actual terminal dimensions for true responsiveness
        let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let base_width = term_cols as usize;
        let base_height = term_rows as usize;

        // Calculate responsive block dimensions based on terminal size
        let margin = if base_width > 100 { 3 } else { 2 };
        let available_width = base_width.saturating_sub(margin * 4); // 4 margins between 3 columns
        let available_height = base_height.saturating_sub(6); // Leave space for title and quit text

        // Adaptive sizing based on terminal size
        let block_width = (available_width * 30) / 100; // ~30% of available width
        let block_height = (available_height * 35) / 100; // ~35% of available height
        let small_width = (available_width * 15) / 100; // ~15% of available width
        let tall_height = (available_height * 75) / 100; // ~75% of available height

        // Ensure minimum sizes for readability
        let block_width = block_width.max(15);
        let block_height = block_height.max(6);
        let small_width = small_width.max(8);
        let tall_height = tall_height.max(12);

        // Responsive positioning
        let col1_x = margin;
        let col2_x = col1_x + block_width + margin;
        let col3_x = col2_x + small_width + margin;

        let row1_y = 3;
        let row2_y = row1_y + block_height + 1;

        // Top-left: Horizontal gradient Red to Blue
        elements.extend(self.create_gradient_block(
            col1_x,
            row1_y,
            block_width,
            block_height,
            vec![(255, 0, 0), (0, 0, 255)],
        ));

        // Bottom-left: Horizontal gradient Green to Yellow to Red
        elements.extend(self.create_gradient_block(
            col1_x,
            row2_y,
            block_width,
            block_height,
            vec![(0, 255, 0), (255, 255, 0), (255, 0, 0)],
        ));

        // Center: Vertical gradient Blue to Purple (tall)
        elements.extend(self.create_vertical_gradient(
            col2_x,
            row1_y,
            small_width,
            tall_height,
            vec![(0, 100, 255), (128, 0, 255)],
        ));

        // Top-right: Horizontal gradient Cyan to Magenta
        elements.extend(self.create_gradient_block(
            col3_x,
            row1_y,
            block_width,
            block_height,
            vec![(0, 255, 255), (255, 0, 255)],
        ));

        // Bottom-right: Rainbow gradient
        elements.extend(self.create_gradient_block(
            col3_x,
            row2_y,
            block_width,
            block_height,
            vec![
                (255, 0, 0),   // Red
                (255, 127, 0), // Orange
                (255, 255, 0), // Yellow
                (0, 255, 0),   // Green
                (0, 0, 255),   // Blue
                (75, 0, 130),  // Indigo
                (148, 0, 211), // Violet
            ],
        ));

        div()
            .class("w-screen h-screen bg-black")
            .children(elements)
            .child(
                div()
                    .class("absolute left-5 top-1 text-white")
                    .child(to_element("🎨 Visual Gradient Blocks"))
                    .build(),
            )
            .child(
                div()
                    .class("absolute bottom-1 left-5 text-gray-400")
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
        .root(GradientBlocks)
        .build()?;

    app.run()
}
