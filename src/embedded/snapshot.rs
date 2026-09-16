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
                // Ghostty may retain UTF-8 C1 controls as printable cell text.
                // Never forward them into a host frame: replace them visibly,
                // preserving native occupancy and the frame's control-byte ban.
                text.extend(chars.into_iter().filter(|c| *c != '\0').map(|c| {
                    if c.is_control() {
                        '\u{fffd}'
                    } else {
                        c
                    }
                }));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_unicode_cells_remain_valid_at_screen_edges_and_after_resize() {
        let mut failures = Vec::new();
        for columns in [1, 2, 3, 8, 96] {
            for text in [
                "界",
                "🙂",
                "👩‍💻",
                "e\u{301}",
                "\u{301}",
                "a\u{200d}b",
                "❤️",
                "🇺🇸",
                "\u{200b}",
                "\u{200e}",
                "\u{200f}",
                "\u{2060}",
                "\u{00ad}",
                "\u{0085}",
                "क्‍ष",
                "\u{2028}",
                "\u{2029}",
                "\u{200c}",
            ] {
                let mut terminal = Terminal::new(columns, 4).unwrap();
                terminal
                    .set_mode(libghostty_vt::terminal::Mode::GRAPHEME_CLUSTER, true)
                    .unwrap();
                let mut state = RenderState::new().unwrap();
                terminal.vt_write(format!("\x1b[1;{columns}H{text}").as_bytes());
                if let Err(error) = capture(&terminal, &mut state) {
                    failures.push(format!("columns={columns}, text={text:?}: {error}"));
                }
                terminal.resize(1, 4, 0, 0).unwrap();
                if let Err(error) = capture(&terminal, &mut state) {
                    failures.push(format!("resized from {columns}, text={text:?}: {error}"));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
