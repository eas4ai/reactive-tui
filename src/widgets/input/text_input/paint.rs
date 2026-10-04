use super::{InputMode, TextInput, TextInputProps, TextInputState};
use crate::component::{Element, FocusProps, LayoutType};
use crate::widgets::input::look;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

struct Glyph {
    text: String,
    start: usize,
    end: usize,
    width: usize,
}

struct Row {
    line: usize,
    start: usize,
    end: usize,
    glyphs: Vec<Glyph>,
}

impl Row {
    fn cursor_column(&self, byte: usize) -> usize {
        self.glyphs
            .iter()
            .take_while(|glyph| glyph.end <= byte)
            .map(|glyph| glyph.width)
            .sum()
    }
}

fn segment(segments: &mut Vec<(String, &'static str)>, text: &str, style: &'static str) {
    if let Some((previous, previous_style)) = segments.last_mut() {
        if *previous_style == style {
            previous.push_str(text);
            return;
        }
    }
    segments.push((text.to_owned(), style));
}

impl TextInput {
    pub(super) fn scroll_rows(
        &self,
        props: &TextInputProps,
        state: &mut TextInputState,
        delta: f32,
    ) {
        let (width, height) = self.view_size(props, state);
        let maximum = self.rows(props, width).len().saturating_sub(height);
        let amount = delta.abs().ceil().max(1.0) as usize;
        state.scroll_offset_y = if delta < 0.0 {
            state.scroll_offset_y.saturating_sub(amount)
        } else {
            state.scroll_offset_y.saturating_add(amount).min(maximum)
        };
    }

    fn number_width(&self, props: &TextInputProps) -> usize {
        if props.show_line_numbers && matches!(props.mode, InputMode::MultiLine { .. }) {
            props.value.split('\n').count().to_string().len() + 1
        } else {
            0
        }
    }

    /// The cells before the field's text: the line numbers and the frame's
    /// first cell.
    fn prefix_width(&self, props: &TextInputProps, _state: &TextInputState) -> usize {
        1 + self.number_width(props)
    }

    /// The field's text width and its rows. A width the props do not set is
    /// what the parent allots, less the frame and the line numbers
    /// (CTL-002); before the first layout, 30 cells.
    pub(super) fn view_size(
        &self,
        props: &TextInputProps,
        state: &TextInputState,
    ) -> (usize, usize) {
        let width = props.width.map_or(usize::MAX, usize::from);
        let height = match props.mode {
            InputMode::MultiLine { height } => usize::from(height).max(1),
            _ => 1,
        };
        self.viewport.map_or((width.min(30), height), |layout| {
            let available_width = (layout.content_size().0 as usize)
                .saturating_sub(self.prefix_width(props, state) + 1);
            let available_height =
                (layout.clip.y + layout.clip.height - layout.transform[5] - layout.insets[1])
                    .max(0.0) as usize;
            (
                width.min(available_width),
                height.min(available_height.max(1)),
            )
        })
    }

    fn rows(&self, props: &TextInputProps, width: usize) -> Vec<Row> {
        let placeholder = props.value.is_empty();
        let text = if placeholder {
            props.placeholder.as_deref().unwrap_or("")
        } else {
            &props.value
        };
        let wrap = props.wrap_text && matches!(props.mode, InputMode::MultiLine { .. });
        let mut result = Vec::new();
        let mut offset = 0;
        for (line_index, line) in text.split('\n').enumerate() {
            let mut row = Row {
                line: line_index,
                start: offset,
                end: offset,
                glyphs: Vec::new(),
            };
            let mut cells = 0;
            for (byte, grapheme) in line.grapheme_indices(true) {
                let display = if !placeholder && props.mode == InputMode::Password {
                    "*".to_owned()
                } else if grapheme == "\t" {
                    let tab = if props.tab_size == 0 {
                        4
                    } else {
                        props.tab_size
                    };
                    " ".repeat((tab - cells % tab).min(width.max(1)))
                } else {
                    grapheme.to_owned()
                };
                let glyph_width = UnicodeWidthStr::width(display.as_str());
                if wrap && cells > 0 && cells + glyph_width > width.max(1) {
                    result.push(row);
                    row = Row {
                        line: line_index,
                        start: offset + byte,
                        end: offset + byte,
                        glyphs: Vec::new(),
                    };
                    cells = 0;
                }
                row.glyphs.push(Glyph {
                    text: display,
                    start: offset + byte,
                    end: offset + byte + grapheme.len(),
                    width: glyph_width,
                });
                row.end = offset + byte + grapheme.len();
                cells += glyph_width;
                if wrap && cells >= width.max(1) {
                    let end = row.end;
                    result.push(row);
                    row = Row {
                        line: line_index,
                        start: end,
                        end,
                        glyphs: Vec::new(),
                    };
                    cells = 0;
                }
            }
            result.push(row);
            offset += line.len() + 1;
        }
        result
    }

    pub(super) fn scroll_to_cursor(&self, props: &TextInputProps, state: &mut TextInputState) {
        let (width, height) = self.view_size(props, state);
        let rows = self.rows(props, width);
        let row_index = rows
            .iter()
            .rposition(|row| {
                row.start <= state.cursor.byte_offset && state.cursor.byte_offset <= row.end
            })
            .unwrap_or(0);
        if row_index < state.scroll_offset_y {
            state.scroll_offset_y = row_index;
        } else if row_index >= state.scroll_offset_y.saturating_add(height) {
            state.scroll_offset_y = row_index + 1 - height;
        }
        if props.wrap_text && matches!(props.mode, InputMode::MultiLine { .. }) {
            state.scroll_offset_x = 0;
        } else {
            let row = &rows[row_index];
            let cursor = row.cursor_column(state.cursor.byte_offset);
            let content_width = row.glyphs.iter().map(|glyph| glyph.width).sum::<usize>();
            let maximum = content_width
                .max(cursor.saturating_add(1))
                .saturating_sub(width);
            state.scroll_offset_x = state.scroll_offset_x.min(maximum);
            if cursor < state.scroll_offset_x {
                state.scroll_offset_x = cursor;
            } else if cursor >= state.scroll_offset_x.saturating_add(width.max(1)) {
                state.scroll_offset_x = cursor + 1 - width.max(1);
            }
            let mut column = 0;
            for glyph in &row.glyphs {
                if column >= state.scroll_offset_x {
                    break;
                }
                column += glyph.width;
            }
            state.scroll_offset_x = column.min(cursor);
        }
    }

    pub(super) fn mouse_byte(
        &self,
        props: &TextInputProps,
        state: &TextInputState,
        x: usize,
        y: usize,
    ) -> Option<usize> {
        let (width, height) = self.view_size(props, state);
        if y >= height {
            return None;
        }
        let rows = self.rows(props, width);
        let row = rows
            .get(state.scroll_offset_y + y)
            .or_else(|| rows.last())?;
        let target = x
            .saturating_sub(self.prefix_width(props, state))
            .saturating_add(state.scroll_offset_x);
        let mut column = 0;
        for glyph in &row.glyphs {
            if target < column + glyph.width {
                return Some(glyph.start.min(props.value.len()));
            }
            column += glyph.width;
        }
        Some(row.end.min(props.value.len()))
    }

    pub(super) fn suggestion_window(
        props: &TextInputProps,
        state: &TextInputState,
    ) -> std::ops::Range<usize> {
        let first = state.suggestion_index.unwrap_or(0).saturating_sub(4);
        first..first.saturating_add(5).min(props.suggestions.len())
    }

    pub(super) fn render_control(&self, props: &TextInputProps, state: &TextInputState) -> Element {
        let (width, height) = self.view_size(props, state);
        // With pixels the field is a picture over its rows: its frame cells
        // and the blank runs of the field hold no glyph, and the picture
        // shows there; its text, placeholder, line numbers, cursor and
        // selection stay cells on `input` (PIX-004).
        let pixels = look::pixels();
        let rows = self.rows(props, width);
        let cursor_row = rows
            .iter()
            .rposition(|row| {
                row.start <= state.cursor.byte_offset && state.cursor.byte_offset <= row.end
            })
            .unwrap_or(0);
        let selection = state.selection.as_ref().map(|selection| {
            (
                selection.start.byte_offset.min(selection.end.byte_offset),
                selection.start.byte_offset.max(selection.end.byte_offset),
            )
        });
        let mut children = Vec::new();
        for visible in 0..height {
            let index = state.scroll_offset_y + visible;
            let row = rows.get(index);
            let mut segments = Vec::new();
            let number_width = self.number_width(props);
            if number_width > 0 {
                let number = row.map_or(String::new(), |row| (row.line + 1).to_string());
                segment(
                    &mut segments,
                    &format!("{number:>digits$} ", digits = number_width - 1),
                    look::LABEL_DISABLED,
                );
            }
            // The field is `input` with `foreground` text between the two
            // cells of its frame, `border` or `ring` while focused; the
            // cursor cell is the field reversed, the selection `selection`,
            // the placeholder `text-muted` (CTL-001).
            // An invalid value shows on the frame, in `error`, also when
            // the field has no error line.
            let frame = if !state.is_valid {
                look::FIELD_FRAME_INVALID
            } else {
                look::field_frame(state.is_focused && !props.disabled)
            };
            // A disabled field's picture is drawn at half, and its text
            // cells, the placeholder's too, take the same half fill
            // (PIX-004).
            let field = match (props.disabled, pixels) {
                (true, true) => look::FIELD_DISABLED_PICTURE,
                (true, false) => look::FIELD_DISABLED,
                (false, _) => look::FIELD,
            };
            let muted = if props.disabled && pixels {
                look::FIELD_DISABLED_PICTURE
            } else {
                look::MUTED
            };
            // The field's blank runs: with pixels they hold no glyph, so
            // they keep a style of their own and are never joined with the
            // text before them.
            let pad = if pixels { look::FIELD_PAD } else { field };
            segment(&mut segments, "[", frame);
            let mut painted = 0;
            let mut column = 0;
            if let Some(row) = row {
                for glyph in &row.glyphs {
                    let start = column;
                    column += glyph.width;
                    if start < state.scroll_offset_x {
                        continue;
                    }
                    let x = start - state.scroll_offset_x;
                    if x + glyph.width > width {
                        break;
                    }
                    if x > painted {
                        segment(&mut segments, &" ".repeat(x - painted), pad);
                    }
                    let style = if state.is_focused
                        && !props.disabled
                        && index == cursor_row
                        && state.cursor.byte_offset == glyph.start
                    {
                        look::CURSOR
                    } else if selection
                        .is_some_and(|(start, end)| glyph.start < end && glyph.end > start)
                    {
                        look::SELECTION
                    } else if props.value.is_empty() {
                        muted
                    } else {
                        field
                    };
                    segment(&mut segments, &glyph.text, style);
                    painted = x + glyph.width;
                }
                let cursor = row
                    .cursor_column(state.cursor.byte_offset)
                    .saturating_sub(state.scroll_offset_x);
                if state.is_focused
                    && !props.disabled
                    && index == cursor_row
                    && state.cursor.byte_offset == row.end
                    && cursor < width
                    && cursor >= painted
                {
                    segment(&mut segments, &" ".repeat(cursor - painted), pad);
                    segment(&mut segments, " ", look::CURSOR);
                    painted = cursor + 1;
                }
            }
            segment(
                &mut segments,
                &" ".repeat(width.saturating_sub(painted)),
                pad,
            );
            segment(&mut segments, "]", frame);
            let mut decoration =
                crate::accessibility::Node::new(crate::accessibility::Role::GenericContainer);
            decoration.set_hidden();
            // An application's class for the field comes after the
            // widget's own, so its colors win.
            let field_class = props.field_class.as_deref().unwrap_or_default();
            children.push(
                Element::layout(LayoutType::Flex)
                    .with_accessibility(decoration)
                    .class("flex flex-row h-1 shrink-0")
                    .children(
                        segments
                            .into_iter()
                            .map(|(text, style)| {
                                let blank = style == frame || style == look::FIELD_PAD;
                                if pixels && blank {
                                    look::blank(unicode_width::UnicodeWidthStr::width(
                                        text.as_str(),
                                    ))
                                } else {
                                    Element::text(text).class(format!(
                                        "shrink-0 whitespace-pre {style} {field_class}"
                                    ))
                                }
                            })
                            .collect(),
                    ),
            );
        }
        #[cfg(feature = "wgpu-graphics")]
        if pixels {
            // The field's picture lies over its rows alone, not over the
            // error line under them.
            let total = self.prefix_width(props, state) + width + 1;
            let mut field = Element::layout(LayoutType::Flex)
                .class(format!("flex flex-col shrink-0 w-{total}"))
                .children(std::mem::take(&mut children));
            field.metadata.look = Some(std::sync::Arc::new(crate::graphics::look::Look::field(
                state.is_focused && !props.disabled,
                !state.is_valid,
                props.disabled,
            )));
            children.push(field);
        }
        if !state.is_valid {
            if let Some(error) = &props.error_message {
                // The error line is an alert to the screen reader (CTL-004).
                children.push(
                    Element::text(error)
                        .class("text-error whitespace-pre aria-live-assertive")
                        .with_accessibility(crate::accessibility::Node::new(
                            crate::accessibility::Role::Alert,
                        )),
                );
            }
        }
        let root = Element::layout(LayoutType::Flex)
            .with_focus(FocusProps::input())
            .disabled(props.disabled);
        let Some(placement) = self.suggestion_placement(props, state) else {
            return root
                .class("text-input flex flex-col w-full overflow-hidden")
                .children(children);
        };
        // The suggestions: a panel under the field, or above it when only
        // the space above holds it, painted whole over the page (CTL-003),
        // its current row in `selection` (CTL-001).
        use crate::widgets::input::panel;
        use unicode_width::UnicodeWidthStr;
        let window = Self::suggestion_window(props, state);
        let lines: Vec<(usize, String, String)> = window
            .map(|index| {
                let suggestion = &props.suggestions[index];
                (
                    index,
                    suggestion.text.clone(),
                    suggestion
                        .description
                        .as_ref()
                        .map_or(String::new(), |description| format!(" — {description}")),
                )
            })
            .collect();
        let body_rows = self.body_rows(props, state);
        let field_width = self
            .viewport
            .map_or(width + self.prefix_width(props, state) + 1, |layout| {
                layout.content_size().0 as usize
            });
        let widest = lines
            .iter()
            .map(|(_, text, description)| {
                UnicodeWidthStr::width(text.as_str()) + UnicodeWidthStr::width(description.as_str())
            })
            .max()
            .unwrap_or(0);
        let panel_width = field_width.max(widest + 4);
        let row_width = panel_width - 2;
        let rows = lines
            .iter()
            .map(|(index, text, description)| {
                let current = state.suggestion_index == Some(*index);
                let padding = " ".repeat(row_width.saturating_sub(
                    2 + UnicodeWidthStr::width(text.as_str())
                        + UnicodeWidthStr::width(description.as_str()),
                ));
                let (text_look, description_look, row_look) = if current {
                    (look::CURRENT_ROW, look::CURRENT_ROW, look::CURRENT_ROW)
                } else {
                    (look::LABEL, look::LABEL_DISABLED, "")
                };
                look::row(
                    &[
                        ("  ", row_look),
                        (text.as_str(), text_look),
                        (description.as_str(), description_look),
                        (padding.as_str(), row_look),
                    ],
                    row_look,
                )
                .with_key(format!("suggestion:{index}"))
            })
            .collect();
        let list = panel::element(placement, panel_width, rows, &self.measured)
            .with_key("text-input-suggestions");
        let mut root = root.class("text-input flex flex-col overflow-visible");
        root.metadata.styles = Some(std::sync::Arc::new(
            panel::host_style(
                placement,
                self.viewport,
                body_rows,
                field_width,
                panel_width,
            )
            .snapshot(),
        ));
        if placement.above {
            let mut ordered = vec![list];
            ordered.extend(children);
            root.children(ordered)
        } else {
            children.push(list);
            root.children(children)
        }
    }

    /// The rows the field and its error line take.
    pub(super) fn body_rows(&self, props: &TextInputProps, state: &TextInputState) -> usize {
        self.view_size(props, state).1
            + usize::from(!state.is_valid && props.error_message.is_some())
    }

    /// Where the suggestion panel stands while suggestions are shown.
    pub(super) fn suggestion_placement(
        &self,
        props: &TextInputProps,
        state: &TextInputState,
    ) -> Option<crate::widgets::input::panel::Placement> {
        if !state.show_suggestions || props.suggestions.is_empty() {
            return None;
        }
        Some(crate::widgets::input::panel::place(
            self.viewport,
            self.measured.screen(),
            self.body_rows(props, state),
            Self::suggestion_window(props, state).len(),
        ))
    }
}
