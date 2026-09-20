use super::*;

#[derive(Clone, Copy)]
struct Position {
    column: usize,
    row: usize,
    pending_wrap: bool,
}

struct Anchor {
    source: Position,
    offset: Option<usize>,
    target: Position,
}

impl Anchor {
    fn new(column: u16, row: u16, pending_wrap: bool) -> Self {
        let source = Position {
            column: usize::from(column),
            row: usize::from(row),
            pending_wrap,
        };
        Self {
            source,
            offset: None,
            target: source,
        }
    }

    fn place(&mut self, column: usize, row: usize, pending_wrap: bool) {
        self.target = Position {
            column,
            row,
            pending_wrap,
        };
        self.offset = None;
    }

    fn viewport_position(&self, top: usize, width: u16, height: u16) -> (u16, u16) {
        (
            self.target.column.min(usize::from(width).saturating_sub(1)) as u16,
            self.target
                .row
                .saturating_sub(top)
                .min(usize::from(height).saturating_sub(1)) as u16,
        )
    }
}

impl VirtualScreen {
    pub(super) fn reflow_main(&mut self, width: u16, height: u16) {
        // DECSET 1049 keeps the main cursor while the alternate grid is active.
        let cursor = if self.using_alt_screen {
            self.saved_cursor.as_ref().unwrap_or(&self.cursor)
        } else {
            &self.cursor
        };
        let mut anchors = vec![Anchor::new(cursor.col, cursor.row, cursor.pending_wrap)];
        if let Some((column, row)) = cursor.saved_position {
            anchors.push(Anchor::new(column, row, false));
        }
        let history_len = self.scrollback.len();
        // Follow the first visible cell, keeping any newly joined historical
        // prefix above it. Height growth alone does not pull history back.
        anchors.push(Anchor::new(0, 0, false));
        for anchor in &mut anchors {
            anchor.source.row += history_len;
        }
        let source = self
            .scrollback
            .drain(..)
            .chain(std::mem::take(&mut self.main_buffer))
            .collect();
        let rows = reflow_rows(source, usize::from(width), &mut anchors);
        // The origin anchor is pushed unconditionally above and
        // `reflow_rows` only mutates anchors in place, so this is always
        // present; an empty stack degrades to the screen origin instead of
        // panicking.
        let origin = anchors.pop().unwrap_or(Anchor::new(0, 0, false));
        let visible_origin = origin.target.row + usize::from(origin.target.column != 0);
        let top = visible_origin.min(anchors[0].target.row).max(
            anchors[0]
                .target
                .row
                .saturating_sub(usize::from(height - 1)),
        );
        for (index, row) in rows.into_iter().enumerate() {
            if index < top {
                if self.max_scrollback != 0 {
                    self.scrollback.push_back(row);
                    while self.scrollback.len() > self.max_scrollback {
                        self.scrollback.pop_front();
                    }
                }
            } else if self.main_buffer.len() < usize::from(height) {
                self.main_buffer.push(row);
            }
        }
        resize_fixed_grid(&mut self.main_buffer, width, height);
        let cursor = if self.using_alt_screen {
            self.saved_cursor.as_mut()
        } else {
            Some(&mut self.cursor)
        };
        if let Some(cursor) = cursor {
            (cursor.col, cursor.row) = anchors[0].viewport_position(top, width, height);
            cursor.pending_wrap = anchors[0].target.pending_wrap;
            if let Some(saved) = anchors.get(1) {
                cursor.saved_position = Some(saved.viewport_position(top, width, height));
            }
        }
    }
}

fn default_blank(cell: &TerminalCell) -> bool {
    cell.character == " "
        && cell.width == 1
        && cell.style == super::super::TerminalStyle::default()
        && cell.hyperlink.is_none()
        && cell.hyperlink_id.is_none()
}

fn reflow_rows(
    rows: Vec<Vec<TerminalCell>>,
    width: usize,
    anchors: &mut [Anchor],
) -> Vec<Vec<TerminalCell>> {
    let mut output = Vec::new();
    let mut logical = Vec::new();
    for (index, row) in rows.into_iter().enumerate() {
        let wrap_end = row.iter().position(|cell| cell.wrapped).map(|end| end + 1);
        let end = wrap_end.unwrap_or_else(|| {
            row.iter()
                .rposition(|cell| !default_blank(cell))
                .map_or(0, |end| end + 1)
        });
        for anchor in anchors
            .iter_mut()
            .filter(|anchor| anchor.source.row == index)
        {
            let column = anchor.source.column + usize::from(anchor.source.pending_wrap);
            anchor.offset = Some(
                logical.len()
                    + if wrap_end.is_some() {
                        column.min(end)
                    } else {
                        column
                    },
            );
        }
        logical.extend(row.into_iter().take(end));
        if wrap_end.is_none() {
            pack_line(std::mem::take(&mut logical), width, &mut output, anchors);
        }
    }
    if !logical.is_empty() {
        pack_line(logical, width, &mut output, anchors);
    }
    output
}

fn pack_line(
    line: Vec<TerminalCell>,
    width: usize,
    output: &mut Vec<Vec<TerminalCell>>,
    anchors: &mut [Anchor],
) {
    // Keep intermediate rows compact. Padding every old blank row to the new
    // width could allocate width * old_height cells instead of width * height.
    output.push(Vec::new());
    let mut offset = 0;
    while offset < line.len() {
        let mut cell = line[offset].clone();
        let cells = usize::from(cell.width.max(1));
        if cell.is_wide_continuation() || cells > width {
            for anchor in anchors.iter_mut() {
                if anchor
                    .offset
                    .is_some_and(|point| (offset..offset + cells).contains(&point))
                {
                    // `output` always holds the row pushed above; a missing
                    // row degrades to column zero, and the saturating
                    // subtraction keeps zero-width reflows panic-free.
                    let column = output
                        .last()
                        .map(|row| row.len())
                        .unwrap_or(0)
                        .min(width.saturating_sub(1));
                    anchor.place(column, output.len() - 1, false);
                }
            }
            offset += cells;
            continue;
        }
        if output.last().map(|row| row.len()).unwrap_or(0) + cells > width {
            if let Some(last) = output.last_mut().and_then(|row| row.last_mut()) {
                last.wrapped = true;
            }
            output.push(Vec::new());
        }
        let row = output.len() - 1;
        let column = output[row].len();
        for anchor in anchors.iter_mut() {
            if let Some(point) = anchor
                .offset
                .filter(|point| (offset..offset + cells).contains(point))
            {
                anchor.place(column + point - offset, row, false);
            }
        }
        cell.wrapped = false;
        cell.dirty = true;
        output[row].push(cell.clone());
        for _ in 1..cells {
            let mut continuation = cell.clone();
            continuation.character.clear();
            continuation.width = 0;
            output[row].push(continuation);
        }
        offset += cells;
    }
    let row = output.len() - 1;
    let column = output[row].len();
    for anchor in anchors.iter_mut() {
        if let Some(point) = anchor.offset {
            let extra = point.saturating_sub(line.len());
            anchor.place(
                (column + extra).min(width - 1),
                row,
                extra == 0 && column == width,
            );
        }
    }
}

pub(super) fn resize_fixed_grid(buffer: &mut Vec<Vec<TerminalCell>>, width: u16, height: u16) {
    buffer.resize_with(usize::from(height), Vec::new);
    for row in buffer {
        row.resize_with(usize::from(width), TerminalCell::new);
        for x in 0..row.len() {
            if x + usize::from(row[x].width) > row.len() {
                row[x].erase();
            }
        }
    }
}

pub(super) fn resize_fixed_cursor(cursor: &mut TerminalCursor, width: u16, height: u16) {
    if cursor.pending_wrap && cursor.col + 1 < width {
        cursor.col += 1;
        cursor.pending_wrap = false;
    }
    cursor.col = cursor.col.min(width - 1);
    cursor.row = cursor.row.min(height - 1);
    if let Some((column, row)) = &mut cursor.saved_position {
        *column = (*column).min(width - 1);
        *row = (*row).min(height - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_reflow_rows_remain_compact_before_viewport_selection() {
        let rows = vec![vec![TerminalCell::new(); 2]; 1000];
        let mut anchors = [Anchor::new(0, 0, false)];
        let rows = reflow_rows(rows, 1000, &mut anchors);
        assert_eq!(rows.len(), 1000);
        assert!(rows.iter().all(Vec::is_empty));
    }

    #[test]
    fn zero_width_reflow_never_panics() {
        // `width - 1` used to underflow on a zero-width viewport.
        let rows = vec![vec![TerminalCell::with_char('x'); 2]; 2];
        let mut anchors = [Anchor::new(0, 0, false)];
        let rows = reflow_rows(rows, 0, &mut anchors);
        assert!(rows.iter().all(Vec::is_empty));
    }
}
