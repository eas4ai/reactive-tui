use crate::core::surface::{Surface, Rgba, Attr};
use crate::layout::style::StyleBuilder;

pub struct PaintStyle {
    pub fg: Rgba,
    pub bg: Rgba,
    pub attr: Attr,
    pub strike: bool,

}

impl Default for PaintStyle {
    fn default() -> Self { Self { fg: Rgba { r:1.0,g:1.0,b:1.0,a:1.0 }, bg: Rgba { r:0.0,g:0.0,b:0.0,a:1.0 }, attr: Attr::empty(), strike: false } }
}

pub fn extract_paint_style(sb: &mut StyleBuilder) -> Option<PaintStyle> {
    if let Some((fg,bg,flags)) = sb.take_visuals() {
        let fg = Rgba { r: fg.0, g: fg.1, b: fg.2, a: fg.3 };
        let bg = Rgba { r: bg.0, g: bg.1, b: bg.2, a: bg.3 };
        let attr = Attr::from_flags(flags.0, flags.1, flags.2, flags.3, false);
        Some(PaintStyle { fg, bg, attr, strike: false })
    } else { None }
}

pub fn paint_text(surface: &mut Surface, x: usize, y: usize, text: &str, style: &PaintStyle) {
    surface.write_str(x, y, text, style.fg, style.bg, style.attr)
}

