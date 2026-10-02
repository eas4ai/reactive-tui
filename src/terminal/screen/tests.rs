use super::*;

fn print(screen: &mut VirtualScreen, text: &str) {
    for ch in text.chars() {
        screen.process_event(AnsiEvent::Print(ch));
    }
}

fn feed(screen: &mut VirtualScreen, text: &str) {
    let mut parser = super::super::parser::AnsiParser::new();
    for event in parser.parse_bytes(text.as_bytes()) {
        screen.process_event(event);
    }
}

#[test]
fn cursor_shape_sequences_preserve_blink_and_ignore_unknown_values() {
    use crate::terminal::cursor::CursorShape;
    let mut screen = VirtualScreen::new(8, 3, 0);
    for (parameter, shape) in [
        (0, CursorShape::BlinkingBlock),
        (1, CursorShape::BlinkingBlock),
        (2, CursorShape::Block),
        (3, CursorShape::BlinkingUnderline),
        (4, CursorShape::Underline),
        (5, CursorShape::BlinkingBar),
        (6, CursorShape::Bar),
    ] {
        feed(&mut screen, &format!("\x1b[{parameter} q"));
        assert_eq!(screen.cursor_shape(), shape);
        feed(&mut screen, "\x1b[99 q\x1b[1q");
        assert_eq!(screen.cursor_shape(), shape);
    }
}

#[test]
fn saved_cursor_is_clamped_after_resize() {
    for (save, restore) in [("\x1b7", "\x1b8"), ("\x1b[s", "\x1b[u")] {
        let mut screen = VirtualScreen::new(12, 6, 0);
        feed(&mut screen, "\x1b[6;12H\x1b[31m");
        feed(&mut screen, save);
        screen.resize(4, 2).unwrap();
        feed(&mut screen, "\x1b[H\x1b[0m");
        feed(&mut screen, restore);
        assert_eq!(screen.cursor_position(), (3, 1));
        feed(&mut screen, "X");
        let cell = screen.cell_at(3, 1).unwrap();
        assert_eq!(cell.character, "X");
        assert_eq!(cell.style.foreground, TerminalColor::Indexed(1));
    }
}

#[test]
fn erase_from_cursor_clears_whole_wide_glyph_with_current_background() {
    let mut screen = VirtualScreen::new(6, 2, 10);
    feed(&mut screen, "a界bc\x1b[1;3H\x1b[44m\x1b[K");
    assert_eq!(screen.cell_at(0, 0).unwrap().character, "a");
    for col in 1..6 {
        let cell = screen.cell_at(col, 0).unwrap();
        assert_eq!(cell.character, " ", "column {col}");
        assert_eq!(cell.width, 1);
        assert_eq!(cell.style.background, TerminalColor::Indexed(4));
    }
}

#[test]
fn erase_to_cursor_includes_cursor_and_preserves_later_cells() {
    let mut screen = VirtualScreen::new(6, 3, 10);
    feed(&mut screen, "abcdef\r\nghijkl\r\nmnopqr\x1b[2;3H\x1b[1J");
    for row in 0..2 {
        for col in 0..if row == 0 { 6 } else { 3 } {
            assert_eq!(screen.cell_at(col, row).unwrap().character, " ");
        }
    }
    assert_eq!(screen.cell_at(3, 1).unwrap().character, "j");
    assert_eq!(screen.cell_at(0, 2).unwrap().character, "m");
    feed(&mut screen, "\x1b[3;3H\x1b[1K");
    assert_eq!(screen.cell_at(2, 2).unwrap().character, " ");
    assert_eq!(screen.cell_at(3, 2).unwrap().character, "p");
    assert_eq!(screen.cursor_position(), (2, 2));
}

#[test]
fn erase_saved_lines_preserves_visible_screen() {
    let mut screen = VirtualScreen::new(6, 2, 10);
    feed(&mut screen, "old\r\nkeep\r\nlast");
    assert_eq!(screen.scrollback_len(), 1);
    feed(&mut screen, "\x1b[3J");
    assert_eq!(screen.scrollback_len(), 0);
    assert_eq!(screen.cell_at(0, 0).unwrap().character, "k");
    assert_eq!(screen.cell_at(0, 1).unwrap().character, "l");
}

#[test]
fn osc_hyperlink_and_directory_keep_semicolons() {
    let mut screen = VirtualScreen::new(8, 2, 0);
    feed(
        &mut screen,
        "\x1b]7;file:///work/a;b\x1b\\\x1b]8;id=one;https://example.com/a;b\x1b\\x\x1b]8;;\x1b\\y",
    );
    assert_eq!(screen.working_directory(), Some("file:///work/a;b"));
    assert_eq!(
        screen.cell_at(0, 0).unwrap().hyperlink.as_deref(),
        Some("https://example.com/a;b")
    );
    assert!(screen.cell_at(1, 0).unwrap().hyperlink.is_none());
}

#[test]
fn escape_index_and_reverse_index_respect_scroll_margins() {
    let mut screen = VirtualScreen::new(8, 4, 10);
    feed(
        &mut screen,
        "HEAD\x1b[2;1Hone\x1b[3;1Htwo\x1b[4;1HFOOT\x1b[2;3r\x1b[2;1H\x1bM",
    );
    assert_eq!(screen.cell_at(0, 0).unwrap().character, "H");
    assert_eq!(screen.cell_at(0, 1).unwrap().character, " ");
    assert_eq!(screen.cell_at(0, 2).unwrap().character, "o");
    assert_eq!(screen.cell_at(0, 3).unwrap().character, "F");
    feed(&mut screen, "\x1b[3;1H\x1bD");
    assert_eq!(screen.cell_at(0, 1).unwrap().character, "o");
    assert_eq!(screen.cell_at(0, 2).unwrap().character, " ");
    feed(&mut screen, "\x1b[1;4H\x1bE");
    assert_eq!(screen.cursor_position(), (0, 1));
    feed(&mut screen, "\x1b(D");
    assert_eq!(
        screen.cursor_position(),
        (0, 1),
        "charset designation is not index"
    );
}

#[test]
fn reset_restores_screen_modes_and_cursor() {
    let mut screen = VirtualScreen::new(8, 3, 10);
    feed(
        &mut screen,
        "old\x1b[31m\x1b[?25l\x1b[?1;2004h\x1b[?1049hALT\x1bcX",
    );
    assert_eq!(screen.cell_at(0, 0).unwrap().character, "X");
    assert_eq!(screen.cursor_position(), (1, 0));
    assert_eq!(
        screen.cell_at(0, 0).unwrap().style,
        super::super::TerminalStyle::default()
    );
    assert!(screen.cursor_visible());
    assert!(!screen.modes.alternate_screen);
    assert!(!(screen.modes.application_cursor_keys || screen.modes.bracketed_paste));
    assert_eq!(screen.scrollback_len(), 0);
}

#[test]
fn private_modes_preserve_main_screen_and_cursor() {
    let mut screen = VirtualScreen::new(8, 3, 10);
    feed(&mut screen, "main\x1b[?1049hALT\x1b[?25l\x1b[?1;2004h");
    assert_eq!(screen.cell_at(0, 0).unwrap().character, "A");
    assert!(!screen.cursor_visible());
    assert!(screen.modes.application_cursor_keys && screen.modes.bracketed_paste);
    assert_eq!(screen.scrollback_len(), 0);
    feed(&mut screen, "\x1b[?1049l\x1b[?25h\x1b[?1;2004l");
    assert_eq!(screen.cell_at(0, 0).unwrap().character, "m");
    assert_eq!(screen.cursor_position(), (4, 0));
    assert!(screen.cursor_visible());
    assert!(!screen.modes.application_cursor_keys && !screen.modes.bracketed_paste);
}

#[test]
fn scroll_region_and_origin_mode_keep_header_and_footer() {
    let mut screen = VirtualScreen::new(8, 4, 10);
    feed(&mut screen, "HEAD\x1b[4;1HFOOT\x1b[2;3r\x1b[?6hA\r\nB\r\nC");
    assert_eq!(screen.cell_at(0, 0).unwrap().character, "H");
    assert_eq!(screen.cell_at(0, 1).unwrap().character, "B");
    assert_eq!(screen.cell_at(0, 2).unwrap().character, "C");
    assert_eq!(screen.cell_at(0, 3).unwrap().character, "F");
    feed(&mut screen, "\x1b[1;1H");
    assert_eq!(screen.cursor_position(), (0, 1));
    feed(&mut screen, "\x1b[?6l\x1b[0C");
    assert_eq!(screen.cursor_position(), (1, 0));
    feed(&mut screen, "\x1b[?7l\x1b[1;8Hxy");
    assert_eq!(screen.cell_at(7, 0).unwrap().character, "y");
    assert_eq!(screen.cursor_position(), (7, 0));
}

#[test]
fn sgr_extended_colors_and_resets() {
    let mut screen = VirtualScreen::new(8, 2, 0);
    screen.set_graphics_rendition(&[38, 2, 12, 34, 56, 48, 5, 201]);
    print(&mut screen, "a");
    let style = screen.cell_at(0, 0).unwrap().style;
    assert_eq!(style.foreground, TerminalColor::Rgb(12, 34, 56));
    assert_eq!(style.background, TerminalColor::Indexed(201));
    screen.set_graphics_rendition(&[91, 104]);
    assert_eq!(screen.cursor.style.foreground, TerminalColor::Indexed(9));
    assert_eq!(screen.cursor.style.background, TerminalColor::Indexed(12));
    screen.set_graphics_rendition(&[39, 49]);
    assert_eq!(screen.cursor.style, super::super::TerminalStyle::default());
}

#[test]
fn sgr_attributes_and_malformed_colors() {
    let mut screen = VirtualScreen::new(8, 2, 0);
    screen.set_graphics_rendition(&[1, 2, 3, 4, 5, 7, 8, 9]);
    let s = screen.cursor.style;
    assert!(s.bold && s.dim && s.italic && s.underline && s.blink);
    assert!(s.reverse && s.invisible && s.strikethrough);
    screen.set_graphics_rendition(&[22, 23, 24, 25, 27, 28, 29]);
    assert_eq!(screen.cursor.style, super::super::TerminalStyle::default());
    screen.set_graphics_rendition(&[38, 2, 999, 1, 2, 48, 5, 999]);
    assert_eq!(screen.cursor.style, super::super::TerminalStyle::default());
    screen.set_graphics_rendition(&[38, 2, 1]);
    assert_eq!(screen.cursor.style, super::super::TerminalStyle::default());
}

#[test]
fn streamed_graphemes_occupy_whole_cells() {
    let mut screen = VirtualScreen::new(12, 2, 10);
    print(&mut screen, "e\u{301}👩‍💻🇺🇸x");
    assert_eq!(screen.cell_at(0, 0).unwrap().character, "e\u{301}");
    assert_eq!(screen.cell_at(1, 0).unwrap().character, "👩‍💻");
    assert!(screen.cell_at(2, 0).unwrap().is_wide_continuation());
    assert_eq!(screen.cell_at(3, 0).unwrap().character, "🇺🇸");
    assert!(screen.cell_at(4, 0).unwrap().is_wide_continuation());
    assert_eq!(screen.cell_at(5, 0).unwrap().character, "x");
    assert_eq!(screen.cursor_position(), (6, 0));
}

#[test]
fn wide_overwrite_clears_both_halves_and_old_links() {
    let mut screen = VirtualScreen::new(6, 2, 10);
    screen
        .cursor
        .set_hyperlink(Some("https://example.com".into()), None);
    print(&mut screen, "界");
    screen.cursor.clear_hyperlink();
    screen.set_cursor_position(1, 0);
    print(&mut screen, "a");
    assert_eq!(screen.cell_at(0, 0).unwrap().character, " ");
    assert_eq!(screen.cell_at(1, 0).unwrap().character, "a");
    assert!(screen.cell_at(1, 0).unwrap().hyperlink.is_none());
}

#[test]
fn wide_glyph_wraps_before_last_column() {
    let mut screen = VirtualScreen::new(4, 2, 10);
    print(&mut screen, "abc界");
    assert_eq!(screen.cell_at(3, 0).unwrap().character, " ");
    assert_eq!(screen.cell_at(0, 1).unwrap().character, "界");
    assert!(screen.cell_at(1, 1).unwrap().is_wide_continuation());
    assert_eq!(screen.cursor_position(), (2, 1));
}

fn row_text(screen: &VirtualScreen, row: u16) -> String {
    (0..screen.size().0)
        .filter_map(|x| screen.cell_at(x, row))
        .map(|cell| cell.character.as_str())
        .collect()
}

#[test]
fn terminal_screen_character_edits_preserve_text_and_wide_boundaries() {
    let mut screen = VirtualScreen::new(10, 3, 10);
    feed(&mut screen, "abcdef\x1b[1;3H\x1b[44m\x1b[2@");
    assert_eq!(row_text(&screen, 0), "ab  cdef  ");
    assert_eq!(
        screen.cell_at(2, 0).unwrap().style.background,
        TerminalColor::Indexed(4)
    );
    feed(&mut screen, "XY\x1b[1;4H\x1b[2P");
    assert_eq!(row_text(&screen, 0), "abXdef    ");
    feed(&mut screen, "\x1b[1;5H\x1b[2X");
    assert_eq!(row_text(&screen, 0), "abXd      ");
    assert_eq!(screen.cursor_position(), (4, 0));
    feed(&mut screen, "\x1b[2;1Ha界bc\x1b[2;3H\x1b[P");
    assert_eq!(row_text(&screen, 1), "a bc      ");
    feed(&mut screen, "\x1b[3;1H12345678界\x1b[3;1H\x1b[@");
    assert_eq!(row_text(&screen, 2), " 12345678 ");
    assert_eq!(screen.cell_at(9, 2).unwrap().width, 1);
}

#[test]
fn terminal_screen_insert_mode_preserves_combining_clusters() {
    let mut screen = VirtualScreen::new(10, 2, 0);
    feed(&mut screen, "abcdef\x1b[1;3H\x1b[4h界e\u{301}\x1b[4lZ");
    assert_eq!(row_text(&screen, 0), "ab界e\u{301}Zdef ");
    assert_eq!(screen.cell_at(3, 0).unwrap().width, 0);
    assert_eq!(screen.cell_at(4, 0).unwrap().character, "e\u{301}");
}

#[test]
fn terminal_screen_line_edits_and_scroll_preserve_margins() {
    let mut screen = VirtualScreen::new(5, 5, 10);
    feed(
        &mut screen,
        "top\r\none\r\ntwo\r\nthree\r\nend\x1b[2;4r\x1b[3;1H\x1b[LNEW",
    );
    assert_eq!(row_text(&screen, 0), "top  ");
    assert_eq!(row_text(&screen, 1), "one  ");
    assert_eq!(row_text(&screen, 2), "NEW  ");
    assert_eq!(row_text(&screen, 3), "two  ");
    assert_eq!(row_text(&screen, 4), "end  ");
    feed(&mut screen, "\x1b[2;1H\x1b[M");
    assert_eq!(row_text(&screen, 1), "NEW  ");
    assert_eq!(row_text(&screen, 2), "two  ");
    assert_eq!(row_text(&screen, 3), "     ");
    feed(&mut screen, "\x1b[S\x1b[T");
    assert_eq!(row_text(&screen, 1), "     ");
    assert_eq!(row_text(&screen, 2), "two  ");
    assert_eq!(row_text(&screen, 4), "end  ");
    assert_eq!(screen.scrollback_len(), 0);
    feed(&mut screen, "\x1b[5;1H\x1b[65535L");
    assert_eq!(
        row_text(&screen, 4),
        "end  ",
        "line editing outside margins must be ignored"
    );
    feed(&mut screen, "\x1b[44m\x1b[65535S");
    for y in 1..4 {
        for x in 0..5 {
            assert_eq!(screen.cell_at(x, y).unwrap().character, " ");
            assert_eq!(
                screen.cell_at(x, y).unwrap().style.background,
                TerminalColor::Indexed(4)
            );
        }
    }
}

#[test]
fn terminal_screen_tab_stops_addressing_and_pending_wrap() {
    let mut screen = VirtualScreen::new(20, 6, 0);
    feed(&mut screen, "\x1b[3g\x1b[5G\x1bH\x1b[12G\x1bH\r\t");
    assert_eq!(screen.cursor_position(), (4, 0));
    feed(&mut screen, "\x1b[I");
    assert_eq!(screen.cursor_position(), (11, 0));
    feed(&mut screen, "\x1b[g\r\x1b[2I");
    assert_eq!(screen.cursor_position(), (19, 0));
    feed(&mut screen, "\x1b[Z");
    assert_eq!(screen.cursor_position(), (4, 0));
    feed(&mut screen, "\x1b[3d\x1b[2E");
    assert_eq!(screen.cursor_position(), (0, 4));
    feed(&mut screen, "\x1b[F");
    assert_eq!(screen.cursor_position(), (0, 3));
    feed(&mut screen, "\x1b[20Gx\x08Z");
    assert_eq!(screen.cell_at(18, 3).unwrap().character, "Z");
    assert_eq!(screen.cursor_position(), (19, 3));
}

#[test]
fn retained_terminal_constructors_reject_zero_dimensions_at_boundary() {
    for (width, height) in [(0, 2), (2, 0)] {
        assert!(
            std::panic::catch_unwind(|| VirtualScreen::new(width, height, 0)).is_err(),
            "screen accepted {width}x{height}"
        );
        assert!(
            std::panic::catch_unwind(|| crate::terminal::Terminal::new(
                crate::terminal::TerminalConfig {
                    size: (width, height),
                    ..Default::default()
                }
            ))
            .is_err(),
            "terminal accepted {width}x{height}"
        );
    }
}

#[test]
fn retained_terminal_fallible_constructors_bound_allocation_and_preserve_valid_sizes() {
    for (width, height) in [(0, 1), (1, 0), (65535, 65535), (1024, 257)] {
        assert!(matches!(
            VirtualScreen::try_new(width, height, 0),
            Err(crate::terminal::TerminalError::InvalidSize { .. })
        ));
        assert!(matches!(
            crate::terminal::Terminal::try_new(crate::terminal::TerminalConfig {
                size: (width, height),
                ..Default::default()
            }),
            Err(crate::terminal::TerminalError::InvalidSize { .. })
        ));
    }
    let screen = VirtualScreen::try_new(1, 1, 0).unwrap();
    assert_eq!(screen.size(), (1, 1));
    let terminal =
        crate::terminal::Terminal::try_new(crate::terminal::TerminalConfig::default()).unwrap();
    assert_eq!(terminal.size(), (80, 24));
    assert!(!terminal.is_running());
}

#[test]
fn resize_preserves_configured_tab_stops() {
    let mut screen = VirtualScreen::new(20, 4, 0);
    feed(&mut screen, "\x1b[3g\x1b[6G\x1bH");
    for (width, height) in [(20, 6), (12, 3), (24, 4)] {
        screen.resize(width, height).unwrap();
        feed(&mut screen, "\r\t");
        assert_eq!(screen.cursor_position().0, 5);
        feed(&mut screen, "\t");
        assert_eq!(screen.cursor_position().0, width - 1);
    }
}
