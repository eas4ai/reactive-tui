use super::*;
use crate::terminal::{TerminalColor, TerminalStyle};
use crate::{
    builder::ElementBuilder,
    component::{ElementType, FocusProps, LayoutType},
    layout::style::StyleBuilder,
};

pub(super) fn content_size(widget: &TerminalWidget) -> (u16, u16) {
    let (w, h) = widget
        .layout
        .map_or((0.0, 0.0), |layout| layout.content_size());
    (
        (w.max(0.0) as u16).saturating_sub(u16::from(widget.props.show_scrollbar)),
        (h.max(0.0) as u16).saturating_sub(u16::from(!widget.props.title.is_empty())),
    )
}
fn color(value: TerminalColor, background: bool) -> (f32, f32, f32) {
    let (r, g, b) = match value {
        TerminalColor::Default if background => (0, 0, 0),
        TerminalColor::Default => (255, 255, 255),
        TerminalColor::Indexed(i) => crate::theme::ansi::ansi256_to_rgb(i),
        TerminalColor::Rgb(r, g, b) => (r, g, b),
    };
    (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}
fn cell(text: String, x: u16, y: u16, width: u16, style: TerminalStyle, cursor: bool) -> Element {
    let mut fg = color(style.foreground, false);
    let mut bg = color(style.background, true);
    if style.reverse ^ cursor {
        std::mem::swap(&mut fg, &mut bg);
    }
    if style.dim {
        fg = (fg.0 * 0.5, fg.1 * 0.5, fg.2 * 0.5);
    }
    let text = if style.invisible {
        " ".repeat(usize::from(width))
    } else {
        text
    };
    ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
        .styles(
            StyleBuilder::new()
                .position_absolute()
                .inset_left(x as f32)
                .inset_top(y as f32)
                .width_px(width as f32)
                .height_px(1.0)
                .overflow_hidden()
                .fg_rgba(fg.0, fg.1, fg.2, 1.0)
                .bg_rgba(bg.0, bg.1, bg.2, 1.0)
                .bold(style.bold)
                .italic(style.italic)
                .underline(style.underline)
                .strike(style.strikethrough),
        )
        .child(Element::text(text).with_class("whitespace-pre"))
        .build()
}
fn cursor_in_view(widget: &TerminalWidget, screen: &crate::terminal::VirtualScreen) -> bool {
    let Some(layout) = widget.layout else {
        return false;
    };
    let (width, height) = content_size(widget);
    let (mut x, y) = screen.cursor_position();
    if x >= width || y >= height {
        return false;
    }
    if screen.cell_at(x, y).is_some_and(|cell| cell.width == 0) {
        x = x.saturating_sub(1);
    }
    let columns = screen
        .cell_at(x, y)
        .map_or(1, |cell| u16::from(cell.width).max(1));
    if x.saturating_add(columns) > width {
        return false;
    }
    let [a, b, c, d, tx, ty] = layout.transform;
    let determinant = a * d - b * c;
    if !determinant.is_finite() || determinant.abs() < f32::EPSILON {
        return false;
    }
    let local_y =
        f32::from(y) + layout.insets[1] + f32::from(u8::from(!widget.props.title.is_empty()));
    (x..x + columns).any(|column| {
        let local_x = f32::from(column) + layout.insets[0];
        layout.clip.contains(crate::event::hit::Point::new(
            (a * local_x + c * local_y + tx).round(),
            (b * local_x + d * local_y + ty).round(),
        ))
    })
}

pub(super) fn render(widget: &TerminalWidget) -> Element {
    let (width, height) = content_size(widget);
    let top = u16::from(!widget.props.title.is_empty());
    let mut children = Vec::new();
    let mut reader_text = String::new();
    let mut reader_cursor = None;
    if top > 0 {
        children.push(cell(
            widget.title(),
            0,
            0,
            width.saturating_add(u16::from(widget.props.show_scrollbar)),
            TerminalStyle::default(),
            false,
        ));
    }
    let error = widget.last_error();
    if let Some(error) = error {
        widget.blink.cancel();
        reader_text = format!("Terminal error: {error}");
        children.push(cell(
            reader_text.clone(),
            0,
            top,
            width,
            TerminalStyle::default(),
            false,
        ));
    } else if let Ok(terminal) = widget.state.terminal.lock() {
        let screen = terminal.screen();
        let offset = widget.state.scroll_position.min(screen.scrollback_len());
        let cursor_visible = widget.blink.visible(
            offset == 0
                && terminal.is_running()
                && cursor_in_view(widget, screen)
                && widget.focused.load(Ordering::Acquire)
                && screen.cursor_visible(),
            screen.cursor_shape(),
            terminal.last_activity(),
        );
        for y in 0..height {
            let mut x = 0;
            while x < width {
                let Some(first) = screen.scrolled_cell_at(x, y, offset) else {
                    break;
                };
                if first.width == 0 {
                    x += 1;
                    continue;
                }
                let start = x;
                let style = first.style;
                let cursor_at = |column: u16| {
                    let (cursor_x, cursor_y) = screen.cursor_position();
                    let columns = screen
                        .scrolled_cell_at(column, y, offset)
                        .map_or(1, |cell| u16::from(cell.width).max(1));
                    cursor_visible
                        && cursor_y == y
                        && cursor_x >= column
                        && cursor_x < column.saturating_add(columns)
                };
                let cursor = cursor_at(x);
                let mut text = String::new();
                while x < width {
                    let Some(value) = screen.scrolled_cell_at(x, y, offset) else {
                        break;
                    };
                    if value.style != style || cursor_at(x) != cursor {
                        break;
                    }
                    let advance = u16::from(value.width).max(1);
                    let (cursor_x, cursor_y) = screen.cursor_position();
                    if offset == 0
                        && screen.cursor_visible()
                        && cursor_y == y
                        && cursor_x >= x
                        && cursor_x < x.saturating_add(advance)
                    {
                        reader_cursor = Some(
                            reader_text.len()
                                + if style.invisible {
                                    usize::from(x - start)
                                } else {
                                    text.len()
                                },
                        );
                    }
                    if x + advance > width {
                        text.push(' ');
                        x += 1;
                        break;
                    }
                    text.push_str(&value.character);
                    x += advance;
                }
                if x == start {
                    break;
                }
                if style.invisible {
                    reader_text.push_str(&" ".repeat(usize::from(x - start)));
                } else {
                    reader_text.push_str(&text);
                }
                let native_style = match screen.cursor_shape() {
                    crate::terminal::cursor::CursorShape::Underline
                    | crate::terminal::cursor::CursorShape::BlinkingUnderline => {
                        Some(::suprtui::render::CursorStyle::Underline)
                    }
                    crate::terminal::cursor::CursorShape::Bar
                    | crate::terminal::cursor::CursorShape::BlinkingBar => {
                        Some(::suprtui::render::CursorStyle::Line)
                    }
                    _ => None,
                }
                .filter(|_| cursor);
                let mut element = cell(
                    text,
                    start,
                    y + top,
                    x - start,
                    style,
                    cursor && native_style.is_none(),
                );
                if let Some(style) = native_style {
                    element.children[0].metadata.text_cursor =
                        Some(crate::component::element::TextCursor {
                            column: screen.cursor_position().0 - start,
                            style,
                        });
                }
                children.push(element);
            }
            if y + 1 < height {
                reader_text.push('\n');
            }
        }
        if widget.props.show_scrollbar && height > 0 {
            let history = screen.scrollback_len();
            let thumb = if history == 0 {
                height - 1
            } else {
                ((history - offset) as u128 * u128::from(height - 1) / history as u128) as u16
            };
            for y in 0..height {
                children.push(cell(
                    if y == thumb { "█" } else { "│" }.into(),
                    width,
                    top + y,
                    1,
                    TerminalStyle::default(),
                    false,
                ));
            }
        }
    }
    let insets = widget.layout.map_or([0.0; 4], |layout| layout.insets);
    let mut hidden_picture =
        crate::accessibility::Node::new(crate::accessibility::Role::GenericContainer);
    hidden_picture.set_hidden();
    let content = ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
        .styles(
            StyleBuilder::new()
                .position_absolute()
                .inset_left(insets[0])
                .inset_top(insets[1])
                .width_px(f32::from(width) + f32::from(u8::from(widget.props.show_scrollbar)))
                .height_px(f32::from(height + top))
                .overflow_hidden(),
        )
        .children(children)
        .build()
        .with_accessibility(hidden_picture);
    let mut root = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .styles(
            StyleBuilder::new()
                .width_percent(100.0)
                .height_percent(100.0)
                .min_width_px(0.0)
                .min_height_px(0.0)
                .overflow_hidden()
                .bg_rgba(0.0, 0.0, 0.0, 1.0),
        )
        .child(content)
        .build();
    let mut accessible = crate::accessibility::Node::new(crate::accessibility::Role::Terminal);
    accessible.set_label(widget.title());
    crate::accessibility::text::append_text_runs(
        &mut root,
        &mut accessible,
        &reader_text,
        false,
        reader_cursor.map(|cursor| [cursor, cursor]),
    );
    root.metadata.accessibility = Some(accessible);
    let focused = widget.focused.clone();
    let blurred = widget.focused.clone();
    let blink = widget.blink.clone();
    let scroll_dragging = widget.scroll_dragging.clone();
    root.focus = Some(FocusProps {
        auto_focus: widget.props.auto_focus,
        on_focus: Some(Arc::new(move || {
            focused.store(true, Ordering::Release);
        })),
        on_blur: Some(Arc::new(move || {
            blurred.store(false, Ordering::Release);
            scroll_dragging.store(false, Ordering::Release);
            blink.cancel();
        })),
        ..FocusProps::input()
    });
    root
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::{
        event::hit::Bounds,
        reactive::{component_scope::ComponentScope, scheduler::Scheduler},
    };

    #[test]
    fn readable_terminal_screen_preserves_graphemes_masks_hidden_cells_and_tracks_scroll() {
        let mut widget = TerminalWidget::new(TerminalProps {
            config: TerminalConfig {
                size: (8, 3),
                ..Default::default()
            },
            title: String::new(),
            show_scrollbar: false,
            ..Default::default()
        });
        widget.layout = Some(LayoutInfo::from_bounds(Bounds::new(0.0, 0.0, 8.0, 3.0)));
        widget
            .state
            .terminal
            .lock()
            .unwrap()
            .process_output("\x1b[31m界e\u{301}\x1b[0m\x1b[8mX\x1b[0mY".as_bytes());
        let element = render(&widget);
        let node = element.metadata.accessibility.as_ref().unwrap();
        assert_eq!(node.inner.role(), crate::accessibility::Role::Terminal);
        assert_eq!(
            node.inner.value(),
            Some("界e\u{301} Y   \n        \n        ")
        );
        assert_eq!(node.text_selection, Some([(0, 4), (0, 4)]));
        assert!(element.children[0]
            .metadata
            .accessibility
            .as_ref()
            .unwrap()
            .inner
            .is_hidden());
        let runs: Vec<_> = element
            .children
            .iter()
            .filter_map(|child| {
                child
                    .metadata
                    .accessibility
                    .as_ref()
                    .filter(|n| n.inner.role() == crate::accessibility::Role::TextRun)
            })
            .collect();
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].inner.character_lengths(), &[3, 3, 1, 1, 1, 1, 1, 1]);
        widget
            .state
            .terminal
            .lock()
            .unwrap()
            .process_output(b"\r\nline2\r\nline3\r\nline4");
        widget.state.scroll_position = 1;
        let scrolled = render(&widget);
        let node = scrolled.metadata.accessibility.as_ref().unwrap();
        assert!(node
            .inner
            .value()
            .unwrap()
            .starts_with("界e\u{301} Y   \nline2"));
        assert_eq!(node.text_selection, None);
    }

    #[test]
    fn invalid_terminal_widget_size_is_reported_until_resize() {
        let mut widget = TerminalWidget::new(TerminalProps {
            config: TerminalConfig {
                size: (0, 4),
                ..Default::default()
            },
            shell_command: Some("/bin/sh".into()),
            ..Default::default()
        });
        assert!(
            widget.last_error().is_some(),
            "invalid size was not reported"
        );
        assert!(matches!(
            widget.start(),
            Err(TerminalError::InvalidSize {
                width: 0,
                height: 4
            })
        ));
        assert!(!widget.is_running());
        assert!(widget.resize(0, 4).is_err());
        widget.resize(10, 4).unwrap();
        assert!(widget.last_error().is_none());
        widget.start().unwrap();
        assert!(widget.is_running());
        widget.stop().unwrap();
    }

    #[test]
    fn terminal_cursor_deadline_tracks_clipping_focus_and_stop() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = ComponentScope::new(scheduler.clone());
        let _guard = scope.enter(true);
        let mut widget = TerminalWidget::new(TerminalProps {
            shell_command: Some("/bin/sh".into()),
            title: String::new(),
            show_scrollbar: false,
            ..Default::default()
        });
        widget.layout = Some(LayoutInfo::from_bounds(Bounds::new(0.0, 0.0, 10.0, 4.0)));
        widget.start().unwrap();
        widget.set_focus(true);
        // Stop output polling while testing a deterministic virtual screen.
        widget.monitor = None;
        widget
            .state
            .terminal
            .lock()
            .unwrap()
            .process_output(b"\x1b[1;1H\x1b[1 q");
        render(&widget);
        assert!(scheduler.next_deadline().is_some());
        widget.layout.as_mut().unwrap().clip = Bounds::new(1.0, 0.0, 9.0, 4.0);
        render(&widget);
        assert!(
            scheduler.next_deadline().is_none(),
            "clipped cursor retained a timer"
        );
        widget.layout.as_mut().unwrap().transform[4] = 2.0;
        render(&widget);
        assert!(
            scheduler.next_deadline().is_some(),
            "translated cursor failed to resume"
        );
        widget.set_focus(false);
        assert!(scheduler.next_deadline().is_none());
        widget.set_focus(true);
        render(&widget);
        assert!(scheduler.next_deadline().is_some());
        widget.layout.as_mut().unwrap().transform[0] = 0.0;
        render(&widget);
        assert!(
            scheduler.next_deadline().is_none(),
            "collapsed cursor retained a timer"
        );
        widget.layout.as_mut().unwrap().transform[0] = 1.0;
        widget
            .state
            .terminal
            .lock()
            .unwrap()
            .process_output(b"\x1b[?25l");
        render(&widget);
        assert!(
            scheduler.next_deadline().is_none(),
            "hidden cursor retained a timer"
        );
        widget
            .state
            .terminal
            .lock()
            .unwrap()
            .process_output(b"\x1b[?25h");
        render(&widget);
        assert!(scheduler.next_deadline().is_some());
        widget.stop().unwrap();
        render(&widget);
        assert!(
            scheduler.next_deadline().is_none(),
            "stopped cursor recreated a timer"
        );
    }
}
