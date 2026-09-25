use reactive_tui::core::surface::{Cell, DiffWriter, Rgba, Surface};
use reactive_tui::editor::cursor::Movement;
use reactive_tui::editor::{Cursor, GapBuffer, SyntaxEditor, TextEditor};

macro_rules! editor_cases {
    ($module:ident, $constructor:expr) => {
        mod $module {
            use super::*;

            #[test]
            fn mixed_insertion_and_grapheme_deletion() {
                let mut editor = $constructor;
                editor.insert_char('A');
                editor.insert_text("界");
                editor.insert_char('e');
                editor.insert_char('\u{301}');
                editor.insert_text("👩🏽‍💻");
                assert_eq!(editor.content(), "A界e\u{301}👩🏽‍💻");
                for expected in ["A界e\u{301}", "A界", "A", ""] {
                    editor.delete_backward();
                    assert_eq!(editor.content(), expected);
                }
            }

            #[test]
            fn selection_replaces_whole_graphemes_across_lines() {
                let mut editor = $constructor;
                editor.insert_text("A界e\u{301}\n👩🏽‍💻Z");
                editor.move_cursor(Movement::Left, false);
                for _ in 0..3 {
                    editor.move_cursor(Movement::Left, true);
                }
                editor.insert_char('!');
                assert_eq!(editor.content(), "A界!Z");
                editor.move_cursor(Movement::Left, false);
                editor.move_cursor(Movement::Left, true);
                editor.insert_text("好\nQ");
                assert_eq!(editor.content(), "A好\nQ!Z");
            }

            #[test]
            fn painted_columns_and_selection_match_text() {
                let mut editor = $constructor;
                editor.set_size(16, 3);
                editor.insert_text("A界e\u{301}Z");
                editor.move_cursor(Movement::Left, false);
                editor.move_cursor(Movement::Left, true);
                let lines = editor.get_styled_lines();
                let selected: String = lines[0]
                    .runs
                    .iter()
                    .filter(|run| {
                        run.bg
                            == Rgba {
                                r: 0.2,
                                g: 0.4,
                                b: 0.6,
                                a: 1.0,
                            }
                    })
                    .map(|run| run.text.as_str())
                    .collect();
                assert_eq!(selected, "e\u{301}");
                let mut surface = Surface::new(20, 4);
                editor.render(&mut surface, 1, 1);
                assert_eq!(surface.get(6, 1).ch, 'A');
                assert_eq!(surface.get(7, 1).ch, '界');
                assert_eq!(surface.get(8, 1).ch, ' ');
                assert_eq!(surface.get(9, 1).ch, 'e');
                assert_eq!(surface.get(10, 1).ch, 'Z');
                assert_eq!(surface.grapheme(9, 1), "e\u{301}");
            }

            #[test]
            fn forward_delete_and_crlf_are_atomic() {
                let mut editor = $constructor;
                editor.insert_text("e\u{301}👩🏽‍💻\r\n界");
                editor.move_cursor(Movement::DocumentStart, false);
                for expected in ["👩🏽‍💻\r\n界", "\r\n界", "界", ""] {
                    editor.delete_forward();
                    assert_eq!(editor.content(), expected);
                }
                editor.insert_text("A\r\nB");
                editor.move_cursor(Movement::Left, false);
                editor.delete_backward();
                assert_eq!(editor.content(), "AB");
            }

            #[test]
            fn clipping_keeps_wide_graphemes_whole_and_clears_old_text() {
                let mut editor = $constructor;
                editor.set_size(7, 1); // Five-column gutter, then A; 界 cannot fit.
                editor.insert_text("A界Z");
                assert_eq!(editor.get_styled_lines()[0].text(), "   1 A");
                let mut surface = Surface::new(12, 3);
                surface.write_str(
                    0,
                    1,
                    "xxxxxxxxxxxx",
                    Rgba::white(),
                    Rgba::black(),
                    Default::default(),
                );
                editor.render(&mut surface, 1, 1);
                assert_eq!(surface.get(0, 1).ch, 'x');
                assert_eq!(surface.get(6, 1).ch, 'A');
                assert_eq!(surface.get(7, 1).ch, ' ');
                assert_eq!(surface.get(8, 1).ch, 'x');
                editor.set_size(0, 0);
                assert!(editor.get_styled_lines().is_empty());
                editor.render(&mut surface, usize::MAX, usize::MAX);
            }

            #[test]
            fn selection_after_syntax_runs_and_tabs_uses_text_offsets() {
                let mut editor = $constructor;
                editor.insert_text("let 界 = \"e\u{301}\";\n\tZ");
                editor.move_cursor(Movement::DocumentStart, false);
                for _ in 0..4 {
                    editor.move_cursor(Movement::Right, false);
                }
                editor.move_cursor(Movement::Right, true);
                let lines = editor.get_styled_lines();
                let selected: String = lines[0]
                    .runs
                    .iter()
                    .filter(|run| {
                        run.bg
                            == Rgba {
                                r: 0.2,
                                g: 0.4,
                                b: 0.6,
                                a: 1.0,
                            }
                    })
                    .map(|run| run.text.as_str())
                    .collect();
                assert_eq!(selected, "界");
                assert_eq!(lines[1].text(), "   2     Z");
            }

            #[test]
            fn rendered_terminal_cells_retain_unicode_after_updates() {
                let mut editor = $constructor;
                editor.set_size(20, 2);
                editor.insert_text("A界e\u{301}🙂Z");
                let empty = Surface::new(20, 2);
                let mut first = Surface::new(20, 2);
                editor.render(&mut first, 0, 0);
                let mut writer = DiffWriter::new();
                writer.diff(&empty, &first, true);
                let mut parser = vt100::Parser::new(2, 20, 0);
                parser.process(writer.output());
                assert_eq!(parser.screen().cell(0, 5).unwrap().contents(), "A");
                assert_eq!(parser.screen().cell(0, 6).unwrap().contents(), "界");
                assert_eq!(parser.screen().cell(0, 8).unwrap().contents(), "e\u{301}");
                assert_eq!(parser.screen().cell(0, 9).unwrap().contents(), "🙂");
                assert_eq!(parser.screen().cell(0, 11).unwrap().contents(), "Z");
                editor.delete_backward();
                editor.delete_backward();
                editor.insert_char('Q');
                let mut second = first.clone_into_new();
                editor.render(&mut second, 0, 0);
                writer.diff(&first, &second, false);
                parser.process(writer.output());
                assert_eq!(parser.screen().cell(0, 9).unwrap().contents(), "Q");
                assert_eq!(parser.screen().cell(0, 11).unwrap().contents(), " ");
            }

            #[test]
            fn control_text_is_visible_without_becoming_terminal_commands() {
                let mut editor = $constructor;
                editor.insert_text("\u{301}\u{1b}[2J\r");
                assert_eq!(editor.content(), "\u{301}\u{1b}[2J\r");
                assert_eq!(editor.get_styled_lines()[0].text(), "   1 ◌\u{301}�[2J� ");
            }

            #[test]
            fn edits_that_join_graphemes_leave_cursor_at_a_boundary() {
                let mut editor = $constructor;
                editor.insert_text("e\u{301}Z");
                editor.move_cursor(Movement::DocumentStart, false);
                editor.insert_text("界");
                editor.delete_forward();
                assert_eq!(editor.content(), "界Z");
                editor.insert_text("👩");
                editor.insert_char('\u{200d}');
                editor.insert_text("💻");
                editor.delete_backward();
                assert_eq!(editor.content(), "界Z");
            }
        }
    };
}

editor_cases!(plain, TextEditor::new());
editor_cases!(syntax, SyntaxEditor::with_language("", "rust"));

#[test]
fn line_index_tracks_edits_before_and_at_newlines() {
    let mut buffer = GapBuffer::from_string("a\nb\nc");
    buffer.insert_char(0, '界');
    assert_eq!(buffer.get_line(0), "界a");
    buffer.insert_str(2, "X\nY");
    assert_eq!(buffer.to_string(), "界aX\nY\nb\nc");
    assert_eq!(
        (0..buffer.line_count())
            .map(|i| buffer.get_line(i))
            .collect::<Vec<_>>(),
        ["界aX", "Y", "b", "c"]
    );
    buffer.delete_char(0);
    buffer.delete_range(1..5);
    assert_eq!(buffer.to_string(), "ab\nc");
    assert_eq!(buffer.get_line(0), "ab");
    assert_eq!(buffer.line_start(1), 3);
}

#[test]
fn cursor_uses_scalar_offsets_and_display_columns() {
    let buffer = GapBuffer::from_string("a界e\u{301}Z\n12345\na界e\u{301}Z");
    let mut cursor = Cursor::new();
    let positions = [1, 2, 4, 5, 6];
    for position in positions {
        cursor.move_right(&buffer);
        assert_eq!(cursor.position, position);
    }
    cursor.move_to(4, &buffer); // Display column 4, before Z.
    cursor.move_down(&buffer);
    assert_eq!(cursor.position, 10); // Before 5, not before 4.
    cursor.move_down(&buffer);
    assert_eq!(cursor.position, 16);
    cursor.move_to(3, &buffer); // Inside e + combining accent snaps backward.
    assert_eq!(cursor.position, 2);
}

#[test]
fn surface_grapheme_ownership_survives_copy_and_expires_on_overwrite() {
    let mut first = Surface::new(8, 1);
    assert!(first.set_grapheme(1, 0, "👩🏽‍💻", Cell::default()));
    assert_eq!(first.grapheme(1, 0), "👩🏽‍💻");
    assert_eq!(first.grapheme(2, 0), "");
    let mut second = first.clone_into_new();
    second.set(
        2,
        0,
        Cell {
            ch: 'X',
            ..Cell::default()
        },
    );
    assert_eq!(second.grapheme(1, 0), " ");
    assert_eq!(second.grapheme(2, 0), "X");
    second.copy_from(&first);
    assert_eq!(second.grapheme(1, 0), "👩🏽‍💻");
    assert!(!second.set_grapheme(7, 0, "界", Cell::default()));
    assert!(!second.set_grapheme(0, 0, "ab", Cell::default()));
    let mut writer = DiffWriter::new();
    writer.diff(&Surface::new(8, 1), &second, true);
    assert!(String::from_utf8_lossy(writer.output()).contains("👩🏽‍💻"));
    second.clear(Rgba::black());
    assert_eq!(second.grapheme(1, 0), " ");
    second.copy_from(&first);
    second.reinit(4, 2);
    assert_ne!(second.grapheme(1, 0), "👩🏽‍💻");
}

#[test]
fn vertical_motion_preserves_columns_across_short_and_wide_lines() {
    let buffer = GapBuffer::from_string("abcd\n界X\nx\nabcd");
    let mut cursor = Cursor::new();
    cursor.move_to(1, &buffer);
    cursor.move_down(&buffer); // Column 1 falls inside 界, so choose its start.
    assert_eq!(cursor.position, 5);
    cursor.move_down(&buffer);
    assert_eq!(cursor.position, 9);
    cursor.move_down(&buffer);
    assert_eq!(cursor.position, 11);
    cursor.move_to(3, &buffer);
    cursor.move_down(&buffer);
    assert_eq!(cursor.position, 7);
    cursor.move_down(&buffer);
    assert_eq!(cursor.position, 9);
    cursor.move_down(&buffer);
    assert_eq!(cursor.position, 13);
}

#[test]
fn word_movement_retains_unicode_grapheme_boundaries() {
    let buffer = GapBuffer::from_string("e\u{301}界 foo_bar 🙂");
    let mut cursor = Cursor::new();
    cursor.move_word_forward(&buffer);
    assert_eq!(cursor.position, 3);
    cursor.move_word_forward(&buffer);
    assert_eq!(cursor.position, 11);
    cursor.move_word_backward(&buffer);
    assert_eq!(cursor.position, 4);
    cursor.move_word_backward(&buffer);
    assert_eq!(cursor.position, 0);
}
