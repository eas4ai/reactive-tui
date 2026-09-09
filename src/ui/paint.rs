use crate::core::surface::{Attr, Rgba, Surface};
use crate::layout::style::StyleBuilder;

/// Style configuration for painting text and elements
pub struct PaintStyle {
    /// Foreground color for text
    pub fg: Rgba,
    /// Background color for elements
    pub bg: Rgba,
    /// Text attributes (bold, italic, underline, etc.)
    pub attr: Attr,
    /// Whether text should have strikethrough
    pub strike: bool,
}

impl Default for PaintStyle {
    fn default() -> Self {
        Self {
            fg: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            bg: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            attr: Attr::empty(),
            strike: false,
        }
    }
}

/// Extract paint style information from a style builder
pub fn extract_paint_style(sb: &mut StyleBuilder) -> Option<PaintStyle> {
    let strike = sb.get_strike();
    if let Some(visual_style) = sb.take_visuals() {
        let fg = visual_style.fg;
        let bg = visual_style.bg;
        let attr = Attr::from_flags(
            visual_style.decorations.bold,
            visual_style.decorations.italic,
            visual_style.decorations.underline,
            visual_style.decorations.reverse,
            false,
        );
        Some(PaintStyle {
            fg,
            bg,
            attr,
            strike,
        })
    } else {
        None
    }
}

use crate::core::geometry::Point;

/// Convenience function for painting text (deprecated - use Surface::write_text_styled instead)
#[deprecated(note = "Use Surface::write_text_styled instead")]
pub fn paint_text(surface: &mut Surface, x: usize, y: usize, text: &str, style: &PaintStyle) {
    surface.write_text_styled(x, y, text, style)
}

/// Convenience function for painting text at a point (deprecated - use Surface::write_text_styled_at instead)
#[deprecated(note = "Use Surface::write_text_styled_at instead")]
pub fn paint_text_at(surface: &mut Surface, point: Point, text: &str, style: &PaintStyle) {
    surface.write_text_styled(point.x, point.y, text, style)
}
