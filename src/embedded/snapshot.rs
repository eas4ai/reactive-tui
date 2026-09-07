use crate::backend::cell_frame::{CellDecoration, UnderlineStyle};
use crate::backend::{CellFrame, FrameCell};
use crate::error::{ReactiveError, Result};
use libghostty_vt::{
    render::{CellIterator, RowIterator},
    screen::CellWide,
    style::{RgbColor, Style, StyleColor, Underline},
    RenderState, Terminal,
};
use suprtui::ansi::TextAttributes;

pub(super) fn native_error(error: libghostty_vt::Error) -> ReactiveError {
    ReactiveError::terminal(format!("libghostty: {error}"))
}

pub(super) fn capture<'a>(
    terminal: &Terminal<'a, '_>,
    state: &mut RenderState<'a>,
) -> Result<CellFrame> {
    let snapshot = state.update(terminal).map_err(native_error)?;
    let colors = snapshot.colors().map_err(native_error)?;
    let width = snapshot.cols().map_err(native_error)?;
    let height = snapshot.rows().map_err(native_error)?;
    CellFrame::validate_size(width, height)?;
    let mut output = Vec::with_capacity(usize::from(width) * usize::from(height));
    let mut rows = RowIterator::new().map_err(native_error)?;
    let mut cells = CellIterator::new().map_err(native_error)?;
    let mut row_iter = rows.update(&snapshot).map_err(native_error)?;
    while let Some(row) = row_iter.next() {
        let mut cell_iter = cells.update(row).map_err(native_error)?;
        while let Some(cell) = cell_iter.next() {
            let wide = cell
                .raw_cell()
                .map_err(native_error)?
                .wide()
                .map_err(native_error)?;
            let cell_width = match wide {
                CellWide::Wide => 2,
                CellWide::SpacerTail => 0,
                _ => 1,
            };
            let mut text = String::new();
            if cell_width != 0 && wide != CellWide::SpacerHead {
                let count = cell.graphemes_len().map_err(native_error)?;
                if count > CellFrame::MAX_GRAPHEME_BYTES {
                    return Err(ReactiveError::resource(
                        "terminal grapheme exceeds snapshot limit",
                    ));
                }
                // Allocate exactly the queried length before calling the native buffer API.
                let mut chars = vec!['\0'; count];
                cell.graphemes_buf(&mut chars).map_err(native_error)?;
                text.extend(chars.into_iter().filter(|c| *c != '\0'));
            }
            if cell_width != 0 && text.is_empty() {
                text.push(' ');
            }
            let style = cell.style().map_err(native_error)?;
            let foreground = rgb(cell
                .fg_color()
                .map_err(native_error)?
                .unwrap_or(colors.foreground));
            let background = rgb(cell
                .bg_color()
                .map_err(native_error)?
                .unwrap_or(colors.background));
            let underline_color = match style.underline_color {
                StyleColor::None => None,
                StyleColor::Rgb(color) => Some(rgb(color)),
                StyleColor::Palette(index) => Some(rgb(colors.palette[usize::from(index.0)])),
            };
            let underline = match style.underline {
                Underline::None => UnderlineStyle::None,
                Underline::Single => UnderlineStyle::Single,
                Underline::Double => UnderlineStyle::Double,
                Underline::Curly => UnderlineStyle::Curly,
                Underline::Dotted => UnderlineStyle::Dotted,
                Underline::Dashed => UnderlineStyle::Dashed,
                _ => {
                    return Err(ReactiveError::terminal(
                        "unsupported libghostty underline style",
                    ))
                }
            };
            output.push(FrameCell {
                text,
                width: cell_width,
                foreground,
                background,
                attributes: attributes(style),
                decoration: CellDecoration {
                    underline,
                    underline_color,
                    overline: style.overline,
                },
            });
        }
    }
    let cursor = if snapshot.cursor_visible().map_err(native_error)? {
        snapshot
            .cursor_viewport()
            .map_err(native_error)?
            .map(|cursor| (cursor.x, cursor.y))
    } else {
        None
    };
    CellFrame::new(width, height, output, cursor)
}

fn rgb(color: RgbColor) -> [u8; 3] {
    [color.r, color.g, color.b]
}

fn attributes(style: Style) -> u8 {
    let mut bits = 0;
    for (enabled, flag) in [
        (style.bold, TextAttributes::BOLD),
        (style.faint, TextAttributes::DIM),
        (style.italic, TextAttributes::ITALIC),
        (style.blink, TextAttributes::BLINK),
        (style.inverse, TextAttributes::INVERSE),
        (style.invisible, TextAttributes::HIDDEN),
        (style.strikethrough, TextAttributes::STRIKETHROUGH),
    ] {
        if enabled {
            bits |= flag;
        }
    }
    bits
}
