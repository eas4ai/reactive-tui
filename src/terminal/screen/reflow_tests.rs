use super::*;

fn feed(screen: &mut VirtualScreen, bytes: &str) {
    for event in super::super::AnsiParser::new().parse_bytes(bytes.as_bytes()) {
        screen.process_event(event);
    }
}

fn row(screen: &VirtualScreen, y: u16) -> String {
    (0..screen.size().0)
        .map(|x| screen.cell_at(x, y).unwrap().character.as_str())
        .collect::<String>()
        .trim_end()
        .to_string()
}

#[test]
fn wider_main_screen_reflows_the_native_command_banner_before_next_input() {
    let mut screen = VirtualScreen::new(43, 11, 100);
    feed(&mut screen, "Microsoft Windows [Version 10.0.20348.5499]\r\n(c) Microsoft Corporation. All rights reserved.\r\n\r\nREADY>@set /a 41+1\r\n42\r\nREADY>");
    assert_eq!(screen.cursor_position(), (6, 6));
    screen.resize(47, 13).unwrap();
    assert_eq!(screen.cursor_position(), (6, 5));
    feed(&mut screen, "\x1b[6;7H@set /a 52+1\r\n53\r\nREADY>");
    assert_eq!(row(&screen, 6), "53");
    assert_eq!(row(&screen, 7), "READY>");
}

#[test]
fn resizing_preserves_hard_breaks_and_pending_wrap_insertion() {
    let mut screen = VirtualScreen::new(4, 4, 10);
    feed(&mut screen, "ab\r\ncd\r\nWXYZ");
    assert!(screen.cursor.pending_wrap);
    screen.resize(8, 4).unwrap();
    assert_eq!(screen.cursor_position(), (4, 2));
    feed(&mut screen, "!");
    assert_eq!(row(&screen, 0), "ab");
    assert_eq!(row(&screen, 1), "cd");
    assert_eq!(row(&screen, 2), "WXYZ!");
}

#[test]
fn reflow_preserves_styled_graphemes_and_omits_wide_wrap_padding() {
    let mut screen = VirtualScreen::new(5, 6, 10);
    feed(
        &mut screen,
        "\x1b[31m\x1b]8;id=link;https://example.test\x07ab界e\u{301}Z",
    );
    screen.resize(8, 6).unwrap();
    assert_eq!(row(&screen, 0), "ab界e\u{301}Z");
    assert_eq!(screen.cursor_position(), (6, 0));
    screen.resize(3, 6).unwrap();
    assert_eq!(row(&screen, 0), "ab");
    assert_eq!(row(&screen, 1), "界e\u{301}");
    assert_eq!(row(&screen, 2), "Z");
    let wide = screen.cell_at(0, 1).unwrap();
    assert_eq!(wide.width, 2);
    assert_eq!(wide.style.foreground, TerminalColor::Indexed(1));
    assert_eq!(wide.hyperlink.as_deref(), Some("https://example.test"));
    assert_eq!(screen.cell_at(1, 1).unwrap().width, 0);
    screen.resize(8, 6).unwrap();
    assert_eq!(row(&screen, 0), "ab界e\u{301}Z");
}

#[test]
fn shrinking_keeps_the_cursor_visible_and_bounds_new_history() {
    let mut screen = VirtualScreen::new(6, 3, 2);
    feed(&mut screen, "abcdefghi\r\nKEEP");
    screen.resize(3, 3).unwrap();
    assert_eq!(screen.cursor_position(), (1, 2));
    assert_eq!(screen.scrollback_len(), 2);
    assert_eq!(row(&screen, 0), "ghi");
    assert_eq!(row(&screen, 1), "KEE");
    assert_eq!(row(&screen, 2), "P");
    assert_eq!(screen.scrolled_cell_at(0, 0, 2).unwrap().character, "a");
    screen.resize(6, 3).unwrap();
    assert_eq!(row(&screen, 0), "ghi");
    assert_eq!(row(&screen, 1), "KEEP");
    assert_eq!(screen.cursor_position(), (4, 1));
}

#[test]
fn alternate_screen_stays_fixed_while_its_saved_main_cursor_reflows() {
    let mut screen = VirtualScreen::new(4, 4, 10);
    feed(&mut screen, "abcdefg\x1b[?1049hALTXYZ");
    screen.resize(8, 4).unwrap();
    assert_eq!(row(&screen, 0), "ALTX");
    assert_eq!(row(&screen, 1), "YZ");
    feed(&mut screen, "\x1b[?1049l");
    assert_eq!(row(&screen, 0), "abcdefg");
    assert_eq!(screen.cursor_position(), (7, 0));
}

#[test]
fn saved_cursor_follows_its_text_after_reflow() {
    let mut screen = VirtualScreen::new(4, 4, 10);
    feed(&mut screen, "\x1b[31mabcdefg\x1b7\x1b[H\x1b[0m");
    screen.resize(8, 4).unwrap();
    feed(&mut screen, "\x1b8");
    assert_eq!(screen.cursor_position(), (7, 0));
    assert_eq!(screen.cursor.style.foreground, TerminalColor::Indexed(1));
}

#[test]
fn a_full_width_hard_break_and_erased_wrap_boundary_do_not_join_rows() {
    for text in ["abcd\r\nEF", "abcdEF\x1b[1;4H\x1b[K"] {
        let mut screen = VirtualScreen::new(4, 4, 10);
        feed(&mut screen, text);
        screen.resize(8, 4).unwrap();
        assert_eq!(row(&screen, 1), "EF");
        assert!(!row(&screen, 0).contains("EF"));
    }
}

#[test]
fn one_column_resize_never_leaves_half_a_wide_grapheme() {
    let mut screen = VirtualScreen::new(4, 6, 10);
    feed(&mut screen, "a界b");
    screen.resize(1, 6).unwrap();
    for y in 0..6 {
        assert_eq!(screen.cell_at(0, y).unwrap().width, 1);
    }
    assert_eq!(row(&screen, 0), "a");
    assert_eq!(row(&screen, 1), "b");
}

#[test]
fn narrowing_reflows_existing_scrollback_without_hiding_columns() {
    let mut screen = VirtualScreen::new(6, 2, 20);
    feed(&mut screen, "abcdef\r\nGHI\r\nJK");
    assert_eq!(screen.scrollback_len(), 1);
    screen.resize(3, 2).unwrap();
    assert_eq!(screen.scrollback_len(), 2);
    let history: Vec<String> = (0..2)
        .map(|y| {
            (0..3)
                .map(|x| screen.scrolled_cell_at(x, y, 2).unwrap().character.as_str())
                .collect()
        })
        .collect();
    assert_eq!(history, ["abc", "def"]);
    assert_eq!(row(&screen, 0), "GHI");
    assert_eq!(row(&screen, 1), "JK");
    assert_eq!(screen.cursor_position(), (2, 1));
}

#[test]
fn reflow_joins_soft_wraps_across_the_history_viewport_boundary() {
    let mut screen = VirtualScreen::new(4, 2, 20);
    feed(&mut screen, "abcdefghijkl");
    assert_eq!(screen.scrollback_len(), 1);
    screen.resize(6, 2).unwrap();
    assert_eq!(screen.scrollback_len(), 1);
    assert_eq!(screen.scrolled_cell_at(5, 0, 1).unwrap().character, "f");
    assert_eq!(row(&screen, 0), "ghijkl");
    assert_eq!(row(&screen, 1), "");
    feed(&mut screen, "!");
    assert_eq!(row(&screen, 1), "!");
    assert_eq!(screen.scrollback_len(), 1);
}

#[test]
fn reflowed_history_stays_bounded_and_height_growth_keeps_the_origin() {
    let mut screen = VirtualScreen::new(6, 2, 2);
    feed(&mut screen, "abcdef\r\nGHIJKL\r\nMN");
    screen.resize(3, 2).unwrap();
    assert_eq!(screen.scrollback_len(), 2);
    assert_eq!(screen.scrolled_cell_at(0, 0, 2).unwrap().character, "d");
    assert_eq!(screen.scrolled_cell_at(0, 1, 2).unwrap().character, "G");
    assert_eq!(row(&screen, 0), "JKL");
    screen.resize(3, 5).unwrap();
    assert_eq!(screen.scrollback_len(), 2);
    assert_eq!(row(&screen, 0), "JKL");
    assert_eq!(row(&screen, 1), "MN");
    assert_eq!(screen.cursor_position(), (2, 1));
}

#[test]
fn history_reflow_preserves_main_saved_cursor_while_alternate_is_active() {
    let mut screen = VirtualScreen::new(4, 2, 20);
    feed(&mut screen, "abcdefghij\x1b7\x1b[?1049hALT");
    screen.resize(6, 2).unwrap();
    assert_eq!(row(&screen, 0), "ALT");
    feed(&mut screen, "\x1b[?1049l\x1b[H\x1b8!");
    assert_eq!(screen.scrollback_len(), 1);
    assert_eq!(row(&screen, 0), "ghij!");
    assert_eq!(screen.cursor_position(), (5, 0));
}

#[test]
fn conpty_resize_keeps_a_reflowed_historical_prefix_above_the_viewport() {
    let mut screen = VirtualScreen::new(31, 7, 100);
    feed(&mut screen, "Microsoft Windows [Version 10.0.20348.5499]\r\n(c) Microsoft Corporation. All rights reserved.\r\n\r\nREADY>\x1b[6;7H@set /a 41+1\r\n42\r\nREADY>");
    assert_eq!(screen.cursor_position(), (6, 6));
    screen.resize(35, 9).unwrap();
    feed(&mut screen, "\x1b[7;7H@set /a 52+1\r\n53\r\nREADY>");
    assert_eq!(row(&screen, 7), "53");
    assert_eq!(row(&screen, 8), "READY>");
}

#[test]
fn reflow_keeps_the_cursor_line_visible_when_it_joins_a_historical_prefix() {
    let mut screen = VirtualScreen::new(4, 2, 20);
    feed(&mut screen, "abcdefghij\x1b[1;2H");
    screen.resize(6, 2).unwrap();
    feed(&mut screen, "!");
    assert_eq!(row(&screen, 0), "abcde!");
    assert_eq!(row(&screen, 1), "ghij");
}
