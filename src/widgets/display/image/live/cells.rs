//! Captured tool output is data, never a host-terminal command stream.
use crate::{
    builder::ElementBuilder,
    component::{Element, ElementType, LayoutType},
    layout::style::StyleBuilder,
};

pub(super) fn validate_size(width: u32, height: u32) -> Result<(), String> {
    if width == 0
        || height == 0
        || width > u16::MAX as u32
        || height >= u16::MAX as u32
        || u64::from(width) * (u64::from(height) + 1) > crate::backend::CellFrame::MAX_CELLS as u64
    {
        return Err("External image layout exceeds the bounded cell grid".into());
    }
    Ok(())
}

pub(super) fn parse(output: &str, width: u32, height: u32) -> Result<vt100::Screen, String> {
    validate_size(width, height)?;
    // A trailing newline must not scroll away the first image row. Child stdout
    // is a pipe, so supply the CR that a terminal's output processing would add.
    let mut parser = vt100::Parser::new(height as u16 + 1, width as u16, 0);
    parser.process(output.replace('\n', "\r\n").as_bytes());
    Ok(parser.screen().clone())
}

/// A cell's color: the tool's own, or for the terminal default the theme
/// color `default` (BAR-003). None when the theme has no such color, so the
/// cell inherits its parent's.
fn color(value: vt100::Color, default: Option<(f32, f32, f32, f32)>) -> Option<(f32, f32, f32)> {
    let (r, g, b) = match value {
        vt100::Color::Default => return default.map(|(r, g, b, _)| (r, g, b)),
        vt100::Color::Idx(index) => crate::theme::ansi::ansi256_to_rgb(index),
        vt100::Color::Rgb(r, g, b) => (r, g, b),
    };
    Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
}

/// One element that paints a blitted grid from the content box's top left.
pub(super) fn grid_element(
    grid: std::sync::Arc<crate::layout::paint_tree::cells::CellGrid>,
) -> Element {
    ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
        .styles(
            StyleBuilder::new()
                .width_percent(100.0)
                .height_percent(100.0)
                .overflow_hidden(),
        )
        .build()
        .with_cells(grid)
}

/// The captured screen as positioned text runs. The terminal's default
/// foreground and background are `theme`'s foreground and background.
pub(super) fn element(screen: &vt100::Screen, theme: &crate::theme::Theme) -> Element {
    let defaults = (
        theme.resolve_color("foreground"),
        theme.resolve_color("background"),
    );
    let (rows, columns) = screen.size();
    let mut root = ElementBuilder::new(ElementType::Layout(LayoutType::Absolute)).styles(
        StyleBuilder::new()
            .width_percent(100.0)
            .height_percent(100.0)
            .overflow_hidden(),
    );
    for row in 0..rows - 1 {
        let mut column = 0;
        while column < columns {
            let first = screen.cell(row, column).unwrap();
            if first.is_wide_continuation() {
                column += 1;
                continue;
            }
            let start = column;
            let mut text = String::new();
            while column < columns {
                let cell = screen.cell(row, column).unwrap();
                if cell.fgcolor() != first.fgcolor()
                    || cell.bgcolor() != first.bgcolor()
                    || cell.bold() != first.bold()
                    || cell.italic() != first.italic()
                    || cell.underline() != first.underline()
                    || cell.inverse() != first.inverse()
                {
                    break;
                }
                if !cell.is_wide_continuation() {
                    text.push_str(if cell.has_contents() {
                        cell.contents()
                    } else {
                        " "
                    });
                }
                column += 1;
            }
            let (mut fg, mut bg) = (
                color(first.fgcolor(), defaults.0),
                color(first.bgcolor(), defaults.1),
            );
            if first.inverse() {
                std::mem::swap(&mut fg, &mut bg);
            }
            let mut style = StyleBuilder::new()
                .position_absolute()
                .inset_left(start as f32)
                .inset_top(row as f32)
                .width_px((column - start) as f32)
                .height_px(1.0)
                .bold(first.bold())
                .italic(first.italic())
                .underline(first.underline())
                .overflow_hidden();
            if let Some((r, g, b)) = fg {
                style = style.fg_rgba(r, g, b, 1.0);
            }
            if let Some((r, g, b)) = bg {
                style = style.bg_rgba(r, g, b, 1.0);
            }
            root = root.child(
                ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
                    .styles(style)
                    .child(Element::text(text).with_class("whitespace-pre"))
                    .build(),
            );
        }
    }
    root.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn api_image_external_cells_preserve_first_row_colors_and_ignore_host_commands() {
        let screen = parse(
            "\x1b]52;c;secret\x07\x1b[38;2;255;0;0m▀▄\n\x1b[34mAB\n",
            2,
            2,
        )
        .unwrap();
        assert_eq!(screen.cell(0, 0).unwrap().contents(), "▀");
        assert_eq!(
            screen.cell(0, 0).unwrap().fgcolor(),
            vt100::Color::Rgb(255, 0, 0)
        );
        assert_eq!(screen.cell(1, 0).unwrap().contents(), "A");
        assert_eq!(screen.cell(1, 0).unwrap().fgcolor(), vt100::Color::Idx(4));
        assert!(parse("", u32::MAX, 1).is_err());
        assert!(parse("", 1024, 1024).is_err());
        assert!(parse("", 0, 0).is_err());
    }

    type Rgba = (f32, f32, f32, f32);

    /// The foreground and background of the first run of `screen` drawn
    /// with `theme`.
    fn run_colors(
        screen: &vt100::Screen,
        theme: &crate::theme::Theme,
    ) -> (Option<Rgba>, Option<Rgba>) {
        let root = element(screen, theme);
        let style = root.children[0]
            .metadata
            .styles
            .as_ref()
            .unwrap()
            .restore()
            .unwrap();
        (style.fg_rgba, style.bg_rgba)
    }

    #[test]
    fn bar_003_default_terminal_colors_come_from_the_theme() {
        use crate::theme::{Theme, ThemeVariables};
        let theme = Theme::new("cells").with_variables(
            ThemeVariables::new()
                .set("--color-foreground", "#336699")
                .set("--color-background", "#102030"),
        );
        let foreground = theme.resolve_color("foreground").unwrap();
        let background = theme.resolve_color("background").unwrap();
        assert_eq!(
            run_colors(&parse("AB\n", 2, 1).unwrap(), &theme),
            (Some(foreground), Some(background))
        );
        // Inverse video swaps the theme's two colors, as a terminal would.
        assert_eq!(
            run_colors(&parse("\x1b[7mAB\n", 2, 1).unwrap(), &theme),
            (Some(background), Some(foreground))
        );
        // A theme without them leaves the run to inherit its parent's colors.
        assert_eq!(
            run_colors(&parse("AB\n", 2, 1).unwrap(), &Theme::new("bare")),
            (None, None)
        );
    }
}
