//! Direct Grid Layout Implementation
//!
//! Based on Textual's approach - bypasses CSS and does direct positioning
//! This fixes the issue where only the first child was being rendered

use crate::core::surface::Surface;
use crate::error::Result;
use crate::layout::grid::{DeclarativeGrid, GridArea};

#[derive(Debug, Clone)]
pub struct GridPlacement {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub area: GridArea,
}

#[derive(Debug)]
pub struct DirectGridLayout {
    pub grid: DeclarativeGrid,
    pub placements: Vec<GridPlacement>,
    pub total_width: usize,
    pub total_height: usize,
}

impl DirectGridLayout {
    /// Create a new direct grid layout from a declarative grid
    pub fn new(grid: DeclarativeGrid, container_width: usize, container_height: usize) -> Self {
        let mut layout = Self {
            grid: grid.clone(),
            placements: Vec::new(),
            total_width: container_width,
            total_height: container_height,
        };

        layout.compute_layout();
        layout
    }

    /// Compute the layout directly without CSS
    fn compute_layout(&mut self) {
        // Calculate column and row sizes
        let col_width = if self.grid.cols > 0 {
            (self
                .total_width
                .saturating_sub(self.grid.gap * (self.grid.cols - 1)))
                / self.grid.cols
        } else {
            self.total_width
        };

        let row_height = if self.grid.rows > 0 {
            (self
                .total_height
                .saturating_sub(self.grid.gap * (self.grid.rows - 1)))
                / self.grid.rows
        } else {
            self.total_height
        };

        // Create placements for each area
        for area in &self.grid.areas {
            let placement = self.compute_area_placement(area, col_width, row_height);
            self.placements.push(placement);
        }

        // Sort by z-index for proper rendering order
        self.placements.sort_by_key(|p| p.area.z_index);
    }

    /// Compute placement for a single grid area
    fn compute_area_placement(
        &self,
        area: &GridArea,
        col_width: usize,
        row_height: usize,
    ) -> GridPlacement {
        // Calculate position
        let x = area.col * (col_width + self.grid.gap);
        let y = area.row * (row_height + self.grid.gap);

        // Calculate size (including spans)
        let width = area.col_span * col_width + (area.col_span.saturating_sub(1)) * self.grid.gap;
        let height = area.row_span * row_height + (area.row_span.saturating_sub(1)) * self.grid.gap;

        GridPlacement {
            x,
            y,
            width,
            height,
            area: area.clone(),
        }
    }

    /// Render the grid directly to a surface
    pub fn render_to_surface(&self, surface: &mut Surface) -> Result<()> {
        // Clear the surface
        surface.clear(crate::core::surface::Rgba::black());

        // Render each placement
        for placement in &self.placements {
            self.render_placement(placement, surface)?;
        }

        Ok(())
    }

    /// Render a single grid placement
    fn render_placement(&self, placement: &GridPlacement, surface: &mut Surface) -> Result<()> {
        let (surface_width, surface_height) = surface.dims();

        // Parse background color from CSS class if available
        let bg_color = self.parse_background_color(&placement.area);
        let text_color = self.parse_text_color(&placement.area);

        // Fill the area with background color
        for y in placement.y..placement.y + placement.height {
            for x in placement.x..placement.x + placement.width {
                if x < surface_width && y < surface_height {
                    let mut cell = surface.get(x, y);
                    cell.bg = bg_color;
                    cell.fg = text_color;
                    surface.set(x, y, cell);
                }
            }
        }

        // Render the text content centered in the area
        self.render_text_centered(&placement.area.name, placement, surface, text_color);

        Ok(())
    }

    /// Parse background color from CSS classes
    fn parse_background_color(&self, area: &GridArea) -> crate::core::surface::Rgba {
        if let Some(ref css_class) = area.css_class {
            if css_class.contains("bg-red-500") {
                return crate::core::surface::Rgba::new(0.937, 0.267, 0.267, 1.0);
            // red-500
            } else if css_class.contains("bg-green-500") {
                return crate::core::surface::Rgba::new(0.133, 0.773, 0.369, 1.0);
            // green-500
            } else if css_class.contains("bg-blue-500") {
                return crate::core::surface::Rgba::new(0.231, 0.510, 0.965, 1.0);
            // blue-500
            } else if css_class.contains("bg-yellow-500") {
                return crate::core::surface::Rgba::new(0.918, 0.702, 0.031, 1.0);
            // yellow-500
            } else if css_class.contains("bg-purple-500") {
                return crate::core::surface::Rgba::new(0.659, 0.333, 0.969, 1.0);
            // purple-500
            } else if css_class.contains("bg-pink-500") {
                return crate::core::surface::Rgba::new(0.925, 0.282, 0.600, 1.0);
            // pink-500
            } else if css_class.contains("bg-gray-400") {
                return crate::core::surface::Rgba::new(0.612, 0.639, 0.686, 1.0);
            // gray-400
            } else if css_class.contains("bg-white") {
                return crate::core::surface::Rgba::new(1.0, 1.0, 1.0, 1.0); // white
            }
        }

        // Default background
        crate::core::surface::Rgba::new(0.2, 0.2, 0.2, 1.0) // dark gray
    }

    /// Parse text color from CSS classes
    fn parse_text_color(&self, area: &GridArea) -> crate::core::surface::Rgba {
        if let Some(ref css_class) = area.css_class {
            if css_class.contains("text-white") {
                return crate::core::surface::Rgba::white();
            } else if css_class.contains("text-black") {
                return crate::core::surface::Rgba::black();
            }
        }

        // Default text color
        crate::core::surface::Rgba::white()
    }

    /// Render text centered in a placement area
    fn render_text_centered(
        &self,
        text: &str,
        placement: &GridPlacement,
        surface: &mut Surface,
        color: crate::core::surface::Rgba,
    ) {
        let (surface_width, surface_height) = surface.dims();

        // Calculate available space for text (leave 1 char padding on each side)
        let max_text_width = placement.width.saturating_sub(2);
        if max_text_width == 0 {
            return; // Cell too small for any text
        }

        // Truncate text if it's too long for the cell
        let display_text = if text.len() > max_text_width {
            if max_text_width >= 3 {
                format!("{}...", &text[..max_text_width.saturating_sub(3)])
            } else {
                text.chars().take(max_text_width).collect::<String>()
            }
        } else {
            text.to_string()
        };

        // Calculate center position
        let center_x = placement.x + placement.width / 2;
        let center_y = placement.y + placement.height / 2;

        // Calculate text start position (centered)
        let text_start_x = center_x.saturating_sub(display_text.len() / 2);

        // Ensure text stays within cell boundaries
        let cell_left = placement.x;
        let cell_right = placement.x + placement.width;

        // Render each character, but only within cell boundaries
        for (i, ch) in display_text.chars().enumerate() {
            let x = text_start_x + i;
            let y = center_y;

            // Check bounds: within surface AND within cell boundaries
            if x >= cell_left && x < cell_right && x < surface_width && y < surface_height {
                let mut cell = surface.get(x, y);
                cell.ch = ch;
                cell.fg = color;
                surface.set(x, y, cell);
            }
        }
    }
}

/// Render a DeclarativeGrid using direct layout computation
pub fn render_grid_direct(
    grid: &DeclarativeGrid,
    surface: &mut Surface,
    width: usize,
) -> Result<()> {
    let height = surface.dims().1;
    let layout = DirectGridLayout::new(grid.clone(), width, height);
    layout.render_to_surface(surface)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout;

    #[test]
    fn test_direct_grid_layout() {
        let grid = layout! {
            grid(cols: 2, rows: 2, gap: 2) {
                "A" at (0, 0) class "bg-red-500 text-white",
                "B" at (0, 1) class "bg-green-500 text-white",
                "C" at (1, 0) class "bg-blue-500 text-white",
                "D" at (1, 1) class "bg-yellow-500 text-black",
            }
        };

        let layout = DirectGridLayout::new(grid, 80, 20);
        assert_eq!(layout.placements.len(), 4);

        // Check that all placements have valid positions
        for placement in &layout.placements {
            assert!(placement.x < 80);
            assert!(placement.y < 20);
            assert!(placement.width > 0);
            assert!(placement.height > 0);
        }
    }

    #[test]
    fn test_direct_grid_render() {
        let grid = layout! {
            grid(cols: 2, rows: 1, gap: 1) {
                "A" at (0, 0) class "bg-red-500 text-white",
                "B" at (0, 1) class "bg-green-500 text-white",
            }
        };

        let mut surface = Surface::new(40, 10);
        let result = render_grid_direct(&grid, &mut surface, 40);
        assert!(result.is_ok());

        // Check that content was rendered
        let mut has_content = false;
        for y in 0..10 {
            for x in 0..40 {
                let cell = surface.get(x, y);
                if cell.ch != ' ' {
                    has_content = true;
                    break;
                }
            }
            if has_content {
                break;
            }
        }
        assert!(has_content, "Surface should contain rendered content");
    }
}
