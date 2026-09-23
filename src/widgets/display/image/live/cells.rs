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

fn color(value: vt100::Color, fallback: (u8, u8, u8)) -> (f32, f32, f32) {
    let (r, g, b) = match value {
        vt100::Color::Default => fallback,
        vt100::Color::Idx(index) => crate::theme::ansi::ansi256_to_rgb(index),
        vt100::Color::Rgb(r, g, b) => (r, g, b),
    };
    (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
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

pub(super) fn element(screen: &vt100::Screen) -> Element {
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
                color(first.fgcolor(), (255, 255, 255)),
                color(first.bgcolor(), (0, 0, 0)),
            );
            if first.inverse() {
                std::mem::swap(&mut fg, &mut bg);
            }
            root = root.child(
                ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
                    .styles(
                        StyleBuilder::new()
                            .position_absolute()
                            .inset_left(start as f32)
                            .inset_top(row as f32)
                            .width_px((column - start) as f32)
                            .height_px(1.0)
                            .fg_rgba(fg.0, fg.1, fg.2, 1.0)
                            .bg_rgba(bg.0, bg.1, bg.2, 1.0)
                            .bold(first.bold())
                            .italic(first.italic())
                            .underline(first.underline())
                            .overflow_hidden(),
                    )
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
}
