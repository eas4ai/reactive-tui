use reactive_tui::core::surface::{Rgba, Surface};
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
