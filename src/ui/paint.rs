use crate::core::surface::{Attr, Rgba, Surface};
use crate::layout::style::StyleBuilder;

pub struct PaintStyle {
    pub fg: Rgba,
    pub bg: Rgba,
    pub attr: Attr,
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

pub fn extract_paint_style(sb: &mut StyleBuilder) -> Option<PaintStyle> {
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
            strike: false,
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
