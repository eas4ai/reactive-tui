//! CSS gradient support for terminal rendering
//!
//! Implements CSS gradient utilities like:
//! - `bg-linear-to-r` (linear gradient to right)  
//! - `from-cyan-700` (starting color)
//! - `via-blue-500` (middle color)
//! - `to-indigo-600` (ending color)

use crate::layout::colors::parse_color_token;
use crate::layout::style::StyleBuilder;
use serde::{Deserialize, Serialize};

/// Gradient direction for linear gradients
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GradientDirection {
    /// Gradient flows from left to right
    ToRight,
    /// Gradient flows from right to left
    ToLeft,
    /// Gradient flows from bottom to top
    ToTop,
    /// Gradient flows from top to bottom
    ToBottom,
    /// Gradient flows diagonally to top-right corner
    ToTopRight,
    /// Gradient flows diagonally to top-left corner
    ToTopLeft,
    /// Gradient flows diagonally to bottom-right corner
    ToBottomRight,
    /// Gradient flows diagonally to bottom-left corner
    ToBottomLeft,
}

/// Gradient color stops
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GradientStops {
    /// Starting color (from-*)
    pub from: Option<(u8, u8, u8, f32)>,
    /// Middle color (via-*)
    pub via: Option<(u8, u8, u8, f32)>,
    /// Ending color (to-*)
    pub to: Option<(u8, u8, u8, f32)>,
}

/// Gradient configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    /// Direction of the gradient flow
    pub direction: GradientDirection,
    /// Color stops defining the gradient
    pub stops: GradientStops,
}

impl Gradient {
    /// Create a new gradient with direction
    pub fn new(direction: GradientDirection) -> Self {
        Self {
            direction,
            stops: GradientStops::default(),
        }
    }

    /// Calculate color at a specific position (0.0 to 1.0)
    pub fn color_at(&self, position: f32) -> Option<(u8, u8, u8, f32)> {
        let position = position.clamp(0.0, 1.0);

        let from = self.stops.from.or(self.stops.via).or(self.stops.to)?;
        let to = self.stops.to.or(self.stops.via).unwrap_or(from);
        Some(match self.stops.via {
            Some(via) if position <= 0.5 => interpolate_color(from, via, position * 2.0),
            Some(via) => interpolate_color(via, to, (position - 0.5) * 2.0),
            None => interpolate_color(from, to, position),
        })
    }

    /// Render gradient across a width (returns colors for each cell)
    pub fn render(&self, width: usize) -> Vec<(u8, u8, u8, f32)> {
        if width == 0 {
            return Vec::new();
        }

        let mut colors = Vec::with_capacity(width);

        for i in 0..width {
            let position = if width == 1 {
                0.5 // Single cell gets middle color
            } else {
                i as f32 / (width - 1) as f32
            };

            if let Some(color) = self.color_at(position) {
                colors.push(color);
            } else {
                // Default to transparent if no color defined
                colors.push((0, 0, 0, 0.0));
            }
        }

        colors
    }
}

pub(crate) fn apply_gradient_utility(token: &str, mut sb: StyleBuilder) -> Option<StyleBuilder> {
    if token == "bg-none" {
        sb.gradient = None;
    } else if let Some(direction) = parse_gradient_direction(token) {
        sb.gradient
            .get_or_insert_with(|| Gradient::new(direction))
            .direction = direction;
    } else if let Some((stop, color)) = parse_gradient_stop(token) {
        let gradient = sb
            .gradient
            .get_or_insert_with(|| Gradient::new(GradientDirection::ToRight));
        match stop {
            GradientStopType::From => gradient.stops.from = Some(color),
            GradientStopType::Via => gradient.stops.via = Some(color),
            GradientStopType::To => gradient.stops.to = Some(color),
        }
    } else {
        return None;
    }
    Some(sb)
}

/// Interpolate between two colors
fn interpolate_color(from: (u8, u8, u8, f32), to: (u8, u8, u8, f32), t: f32) -> (u8, u8, u8, f32) {
    let t = t.clamp(0.0, 1.0);
    (
        (from.0 as f32 + (to.0 as f32 - from.0 as f32) * t) as u8,
        (from.1 as f32 + (to.1 as f32 - from.1 as f32) * t) as u8,
        (from.2 as f32 + (to.2 as f32 - from.2 as f32) * t) as u8,
        from.3 + (to.3 - from.3) * t,
    )
}

/// Parse gradient direction from token
pub fn parse_gradient_direction(token: &str) -> Option<GradientDirection> {
    match token {
        "bg-linear-to-r" | "bg-gradient-to-r" => Some(GradientDirection::ToRight),
        "bg-linear-to-l" | "bg-gradient-to-l" => Some(GradientDirection::ToLeft),
        "bg-linear-to-t" | "bg-gradient-to-t" => Some(GradientDirection::ToTop),
        "bg-linear-to-b" | "bg-gradient-to-b" => Some(GradientDirection::ToBottom),
        "bg-linear-to-tr" | "bg-gradient-to-tr" => Some(GradientDirection::ToTopRight),
        "bg-linear-to-tl" | "bg-gradient-to-tl" => Some(GradientDirection::ToTopLeft),
        "bg-linear-to-br" | "bg-gradient-to-br" => Some(GradientDirection::ToBottomRight),
        "bg-linear-to-bl" | "bg-gradient-to-bl" => Some(GradientDirection::ToBottomLeft),
        _ => None,
    }
}

/// Parse gradient color stop from token
pub fn parse_gradient_stop(token: &str) -> Option<(GradientStopType, (u8, u8, u8, f32))> {
    if let Some(color_part) = token.strip_prefix("from-") {
        parse_color_token(color_part).map(|(r, g, b, a)| {
            (
                GradientStopType::From,
                ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, a),
            )
        })
    } else if let Some(color_part) = token.strip_prefix("via-") {
        parse_color_token(color_part).map(|(r, g, b, a)| {
            (
                GradientStopType::Via,
                ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, a),
            )
        })
    } else if let Some(color_part) = token.strip_prefix("to-") {
        parse_color_token(color_part).map(|(r, g, b, a)| {
            (
                GradientStopType::To,
                ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, a),
            )
        })
    } else {
        None
    }
}

/// Type of gradient stop
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GradientStopType {
    /// Starting color of the gradient
    From,
    /// Middle color of the gradient (optional)
    Via,
    /// Ending color of the gradient
    To,
}

/// Gradient border configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientBorder {
    /// Gradient to apply to the border
    pub gradient: Gradient,
    /// Border width in terminal cells
    pub width: usize,
    /// Duration of a complete RGB color cycle in App. None or zero is static.
    /// Struct literals must supply this field; older serialized data defaults to None.
    #[serde(default)]
    pub cycle_duration: Option<std::time::Duration>,
}

impl GradientBorder {
    /// Create a new gradient border
    pub fn new(gradient: Gradient, width: usize) -> Self {
        Self {
            gradient,
            width,
            cycle_duration: None,
        }
    }

    /// Calculate border color at a specific position around the perimeter
    /// Position goes from 0.0 (top-left) clockwise around the border to 1.0 (back to top-left)
    pub fn border_color_at(&self, perimeter_position: f32) -> Option<(u8, u8, u8, f32)> {
        self.gradient.color_at(perimeter_position)
    }

    pub(crate) fn color_at_cell(
        &self,
        column: usize,
        row: usize,
        width: usize,
        height: usize,
    ) -> Option<(u8, u8, u8, f32)> {
        if column >= width || row >= height {
            return None;
        }
        let edge = column
            .min(row)
            .min(width - column - 1)
            .min(height - row - 1);
        if edge >= self.width {
            return None;
        }
        let width = width - 2 * edge;
        let height = height - 2 * edge;
        let column = column - edge;
        let row = row - edge;
        let position = if width == 1 && height == 1 {
            0.5
        } else if height == 1 {
            column as f32 / (width - 1) as f32
        } else if width == 1 {
            row as f32 / (height - 1) as f32
        } else {
            let perimeter = 2.0 * ((width - 1) as f32 + (height - 1) as f32);
            let along = if row == 0 {
                column as f32
            } else if column == width - 1 {
                (width - 1) as f32 + row as f32
            } else if row == height - 1 {
                (width - 1) as f32 + (height - 1) as f32 + (width - column - 1) as f32
            } else {
                2.0 * (width - 1) as f32 + (height - 1) as f32 + (height - row - 1) as f32
            };
            along / perimeter
        };
        self.border_color_at(position)
    }

    /// Render the top, right, reversed bottom and reversed left border colors.
    /// Empty dimensions return no edges; a single cell samples the midpoint.
    pub fn render_border(&self, width: usize, height: usize) -> Vec<Vec<(u8, u8, u8, f32)>> {
        if width == 0 || height == 0 {
            return Vec::new();
        }
        vec![
            (0..width)
                .filter_map(|x| self.color_at_cell(x, 0, width, height))
                .collect(),
            (0..height)
                .filter_map(|y| self.color_at_cell(width - 1, y, width, height))
                .collect(),
            (0..width)
                .rev()
                .filter_map(|x| self.color_at_cell(x, height - 1, width, height))
                .collect(),
            (0..height)
                .rev()
                .filter_map(|y| self.color_at_cell(0, y, width, height))
                .collect(),
        ]
    }

    /// Create an animated rainbow border that cycles through colors
    pub fn rainbow_border(width: usize) -> Self {
        let mut gradient = Gradient::new(GradientDirection::ToRight);
        // Create a rainbow gradient
        gradient.stops.from = Some((255, 0, 0, 1.0)); // Red
        gradient.stops.via = Some((0, 255, 0, 1.0)); // Green
        gradient.stops.to = Some((0, 0, 255, 1.0)); // Blue

        Self {
            gradient,
            width,
            cycle_duration: Some(std::time::Duration::from_secs(2)),
        }
    }

    /// Create a conic gradient border (rotates around the perimeter)
    pub fn conic_gradient(colors: Vec<(u8, u8, u8)>) -> Self {
        let mut gradient = Gradient::new(GradientDirection::ToRight);

        if colors.len() >= 2 {
            gradient.stops.from = Some((colors[0].0, colors[0].1, colors[0].2, 1.0));
            if colors.len() >= 3 {
                gradient.stops.via = Some((colors[1].0, colors[1].1, colors[1].2, 1.0));
                gradient.stops.to = Some((colors[2].0, colors[2].1, colors[2].2, 1.0));
            } else {
                gradient.stops.to = Some((colors[1].0, colors[1].1, colors[1].2, 1.0));
            }
        }

        Self::new(gradient, 1)
    }
}

/// Apply gradient utilities to a style builder
/// This would need to be integrated with the rendering system to actually display gradients
pub fn apply_gradient(
    tokens: &[&str],
    sb: StyleBuilder,
) -> Option<(StyleBuilder, Option<Gradient>)> {
    let mut gradient: Option<Gradient> = None;
    let style = sb;

    for token in tokens {
        // Check for gradient direction
        if let Some(direction) = parse_gradient_direction(token) {
            gradient = Some(Gradient::new(direction));
        }
        // Check for gradient stops
        else if let Some((stop_type, color)) = parse_gradient_stop(token) {
            if let Some(ref mut grad) = gradient {
                match stop_type {
                    GradientStopType::From => grad.stops.from = Some(color),
                    GradientStopType::Via => grad.stops.via = Some(color),
                    GradientStopType::To => grad.stops.to = Some(color),
                }
            }
        }
    }

    if gradient.is_some() {
        Some((style, gradient))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_interpolation() {
        let mut gradient = Gradient::new(GradientDirection::ToRight);
        gradient.stops.from = Some((255, 0, 0, 1.0)); // Red
        gradient.stops.to = Some((0, 0, 255, 1.0)); // Blue

        // At position 0, should be red
        assert_eq!(gradient.color_at(0.0), Some((255, 0, 0, 1.0)));

        // At position 1, should be blue
        assert_eq!(gradient.color_at(1.0), Some((0, 0, 255, 1.0)));

        // At position 0.5, should be purple-ish
        let mid = gradient.color_at(0.5).unwrap();
        assert!(mid.0 > 100 && mid.0 < 150); // Red component
        assert_eq!(mid.1, 0); // Green component
        assert!(mid.2 > 100 && mid.2 < 150); // Blue component
    }

    #[test]
    fn test_three_color_gradient() {
        let mut gradient = Gradient::new(GradientDirection::ToRight);
        gradient.stops.from = Some((255, 0, 0, 1.0)); // Red
        gradient.stops.via = Some((0, 255, 0, 1.0)); // Green
        gradient.stops.to = Some((0, 0, 255, 1.0)); // Blue

        // At 0.25, should be between red and green
        let quarter = gradient.color_at(0.25).unwrap();
        assert!(quarter.0 > 0); // Some red
        assert!(quarter.1 > 0); // Some green
        assert_eq!(quarter.2, 0); // No blue

        // At 0.75, should be between green and blue
        let three_quarter = gradient.color_at(0.75).unwrap();
        assert_eq!(three_quarter.0, 0); // No red
        assert!(three_quarter.1 > 0); // Some green
        assert!(three_quarter.2 > 0); // Some blue
    }

    #[test]
    fn test_gradient_rendering() {
        let mut gradient = Gradient::new(GradientDirection::ToRight);
        gradient.stops.from = Some((255, 0, 0, 1.0)); // Red
        gradient.stops.to = Some((0, 0, 255, 1.0)); // Blue

        let colors = gradient.render(5);
        assert_eq!(colors.len(), 5);

        // First color should be red
        assert_eq!(colors[0], (255, 0, 0, 1.0));

        // Last color should be blue
        assert_eq!(colors[4], (0, 0, 255, 1.0));

        // Middle colors should be interpolated
        assert!(colors[2].0 > 100 && colors[2].0 < 150);
    }
}
