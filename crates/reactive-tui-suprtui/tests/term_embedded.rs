//! term-embedded commitment tests (TRM-006, TRM-007), ported from
//! `embedded-terminal/tests.zig`. Input-side encoder vectors (key,
//! mouse, paste, focus) have no TRM requirement and stay out of scope.

use std::cell::RefCell;
use std::rc::Rc;
use suprtui::ansi::{self, TextAttributes};
use suprtui::buffer::{InitOptions, OptimizedBuffer};
use suprtui::term_embedded::{EmbeddedError, EmbeddedOptions, EmbeddedTerminal, Point};
use suprtui::uni::pool::GraphemePool;
use suprtui::uni::segments::{is_continuation_char, is_grapheme_char};

fn pool() -> Rc<RefCell<GraphemePool<'static>>> {
    Rc::new(RefCell::new(GraphemePool::new()))
}

fn target(pool: &Rc<RefCell<GraphemePool<'static>>>, w: u32, h: u32) -> OptimizedBuffer<'static> {
    OptimizedBuffer::new(w, h, InitOptions::new(Rc::clone(pool))).unwrap()
}

fn terminal(cols: u16, rows: u16) -> EmbeddedTerminal {
    EmbeddedTerminal::new(EmbeddedOptions::new(cols, rows)).unwrap()
}

fn pt(x: u16, y: u16) -> Point {
    Point { x, y }
}

// ---------------------------------------------------------------------------
// TRM-006: compose dirty rows, resize, scroll, selection, cursor.
// ---------------------------------------------------------------------------

#[test]
fn req_006_embedded_compose() {
    let pool = pool();
    let mut target = target(&pool, 12, 4);
    target.clear(ansi::rgb_color(0, 0, 0, 255), None);

    let mut terminal = terminal(8, 2);
    terminal.write("A\x1b[1;32mB\x1b[0m\r\nwide: \u{738c}".as_bytes());

    terminal.compose(&mut target, 2, 1).unwrap();
    assert_eq!(target.get(2, 1).unwrap().char, 'A' as u32);
    assert_eq!(target.get(3, 1).unwrap().char, 'B' as u32);
    assert_ne!(
        target.get(3, 1).unwrap().attributes & u32::from(TextAttributes::BOLD),
        0
    );
    assert!(ansi::green(target.get(3, 1).unwrap().fg) > ansi::red(target.get(3, 1).unwrap().fg));
    assert!(is_grapheme_char(target.get(8, 2).unwrap().char));
    assert!(is_continuation_char(target.get(9, 2).unwrap().char));

    // A clean row is not redrawn: the sentinel survives recompose.
    let mut sentinel = target.get(2, 1).unwrap();
    sentinel.char = 'X' as u32;
    target.set(2, 1, sentinel);
    terminal.compose(&mut target, 2, 1).unwrap();
    assert_eq!(target.get(2, 1).unwrap().char, 'X' as u32);

    terminal.invalidate();
    terminal.compose(&mut target, 1, 0).unwrap();
    assert_eq!(target.get(1, 0).unwrap().char, 'A' as u32);
}

#[test]
fn req_006_redraw_clip() {
    let pool = pool();
    let mut target = target(&pool, 5, 2);

    let mut terminal = terminal(4, 2);
    terminal.write(b"abcd");
    terminal.compose(&mut target, -1, 0).unwrap();
    assert_eq!(target.get(0, 0).unwrap().char, 'b' as u32);
    assert_eq!(target.get(2, 0).unwrap().char, 'd' as u32);

    // Only the changed row repaints; the sentinel row is untouched.
    terminal.write(b"\x1b[1;2HZ");
    let mut sentinel = target.get(0, 1).unwrap();
    sentinel.char = 'Q' as u32;
    target.set(0, 1, sentinel);
    terminal.compose(&mut target, -1, 0).unwrap();
    assert_eq!(target.get(0, 0).unwrap().char, 'Z' as u32);
    assert_eq!(target.get(0, 1).unwrap().char, 'Q' as u32);

    terminal.resize(5, 2).unwrap();
    terminal.compose(&mut target, 0, 0).unwrap();
    assert_eq!(target.get(0, 0).unwrap().char, 'a' as u32);
    assert_eq!(
        target.get(0, 1).unwrap().char,
        suprtui::buffer::DEFAULT_SPACE_CHAR
    );
}

#[test]
fn req_006_cursor_state() {
    let pool = pool();
    let mut target = target(&pool, 20, 4);

    let mut terminal = terminal(20, 4);
    terminal.write(b"\x1b[2;3H\x1b[5 q");
    terminal.compose(&mut target, 0, 0).unwrap();

    let cursor = terminal.cursor();
    assert!(cursor.has_value);
    assert!(cursor.visible);
    assert_eq!(cursor.x, 2);
    assert_eq!(cursor.y, 1);
    assert_eq!(cursor.style, 0);
}

#[test]
fn req_006_lifecycle_resize_scroll() {
    let mut terminal = terminal(80, 24);
    terminal.write(b"hello");
    terminal.resize(100, 40).unwrap();
    terminal.scroll(-3);
    terminal.scroll(3);

    assert_eq!(terminal.cols(), 100);
    assert_eq!(terminal.rows(), 40);
    assert_eq!(terminal.resize(0, 40), Err(EmbeddedError::InvalidValue));
    assert_eq!(terminal.resize(100, 0), Err(EmbeddedError::InvalidValue));
    assert!(
        EmbeddedTerminal::new(EmbeddedOptions::new(0, 24)).is_err(),
        "zero-col init must fail"
    );
}

#[test]
fn req_006_select_extract() {
    let pool = pool();
    let mut target = target(&pool, 8, 2);

    let mut terminal = terminal(8, 2);
    terminal.write(b"hello");
    terminal.set_selection(pt(1, 0), pt(3, 0)).unwrap();
    assert_eq!(terminal.selected_text(), "ell");

    terminal.compose(&mut target, 0, 0).unwrap();
    let unselected = target.get(0, 0).unwrap();
    let highlighted = target.get(1, 0).unwrap();
    assert!(ansi::red(unselected.fg) > ansi::red(unselected.bg));
    assert!(ansi::red(highlighted.fg) < ansi::red(highlighted.bg));

    terminal.clear_selection();
    terminal.compose(&mut target, 0, 0).unwrap();
    let cleared = target.get(1, 0).unwrap();
    assert!(ansi::red(cleared.fg) > ansi::red(cleared.bg));
}

#[test]
fn req_006_selection_highlights() {
    struct Case {
        output: &'static str,
        start: Point,
        end: Point,
        text: &'static str,
        highlight: [&'static str; 3],
    }
    let cases = [
        Case {
            output: "hello\r\nabc",
            start: pt(1, 0),
            end: pt(7, 2),
            text: "ello\nabc",
            highlight: [".####...", "###.....", "........"],
        },
        Case {
            output: "hello",
            start: pt(7, 0),
            end: pt(7, 0),
            text: "",
            highlight: ["........", "........", "........"],
        },
        Case {
            output: "",
            start: pt(0, 0),
            end: pt(7, 2),
            text: "",
            highlight: ["........", "........", "........"],
        },
        Case {
            output: "  a\x1b[3Cb",
            start: pt(0, 0),
            end: pt(7, 2),
            text: "  a   b",
            highlight: ["#######.", "........", "........"],
        },
        Case {
            // Written spaces remain text even though clipboard
            // extraction trims them.
            output: "a  ",
            start: pt(0, 0),
            end: pt(7, 2),
            text: "a",
            highlight: ["###.....", "........", "........"],
        },
        Case {
            output: "\x1b[44m\x1b[2J\x1b[0mhi",
            start: pt(0, 0),
            end: pt(7, 2),
            text: "hi",
            highlight: ["##......", "........", "........"],
        },
        Case {
            output: "abcdefghi",
            start: pt(0, 0),
            end: pt(7, 2),
            text: "abcdefghi",
            highlight: ["########", "#.......", "........"],
        },
        Case {
            output: "\u{738c}",
            start: pt(1, 0),
            end: pt(1, 0),
            text: "\u{738c}",
            highlight: ["##......", "........", "........"],
        },
        Case {
            output: "e\u{301} \u{1f600}",
            start: pt(0, 0),
            end: pt(7, 2),
            text: "e\u{301} \u{1f600}",
            highlight: ["####....", "........", "........"],
        },
        Case {
            output: "abcdefg\u{738c}",
            start: pt(0, 0),
            end: pt(7, 2),
            text: "abcdefg\u{738c}",
            highlight: ["#######.", "##......", "........"],
        },
    ];

    for case in &cases {
        let pool = pool();
        let mut target = target(&pool, 8, 3);
        let mut terminal = terminal(8, 3);
        terminal.write(case.output.as_bytes());
        terminal.compose(&mut target, 0, 0).unwrap();
        let mut original = [[suprtui::buffer::make_cell(
            0,
            ansi::rgb_color(0, 0, 0, 255),
            ansi::rgb_color(0, 0, 0, 255),
            0,
        ); 8]; 3];
        for (y, row) in original.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                *cell = target.get(x as u32, y as u32).unwrap();
            }
        }

        for reverse in [false, true] {
            let (start, end) = if reverse {
                (case.end, case.start)
            } else {
                (case.start, case.end)
            };
            terminal.set_selection(start, end).unwrap();
            assert_eq!(terminal.selected_text(), case.text);
            terminal.compose(&mut target, 0, 0).unwrap();

            for (y, row) in case.highlight.iter().enumerate() {
                for (x, highlight) in row.bytes().enumerate() {
                    let cell = target.get(x as u32, y as u32).unwrap();
                    let before = original[y][x];
                    if highlight == b'#' {
                        assert_eq!(cell.fg, before.bg, "fg x={x} y={y}");
                        assert_eq!(cell.bg, before.fg, "bg x={x} y={y}");
                    } else {
                        assert_eq!(cell.fg, before.fg, "fg x={x} y={y}");
                        assert_eq!(cell.bg, before.bg, "bg x={x} y={y}");
                    }
                }
            }

            // Moving into an unused row must repaint the previous
            // highlight too.
            terminal.set_selection(pt(0, 2), pt(7, 2)).unwrap();
            terminal.compose(&mut target, 0, 0).unwrap();
            for (y, row) in original.iter().enumerate() {
                for (x, before) in row.iter().enumerate() {
                    let cell = target.get(x as u32, y as u32).unwrap();
                    assert_eq!(cell.fg, before.fg);
                    assert_eq!(cell.bg, before.bg);
                }
            }
            terminal.clear_selection();
            terminal.compose(&mut target, 0, 0).unwrap();
        }
    }
}

// ---------------------------------------------------------------------------
// TRM-007: exactly-once in-order response drain.
// ---------------------------------------------------------------------------

#[test]
fn req_007_drain_incremental() {
    let mut terminal = terminal(20, 4);
    terminal.write(b"\x1b[5n");

    let mut first = [0u8; 2];
    let mut rest = [0u8; 16];
    let first_len = terminal.drain_responses(&mut first).unwrap();
    let rest_len = terminal.drain_responses(&mut rest).unwrap();

    let mut combined = [0u8; 18];
    combined[..first_len].copy_from_slice(&first[..first_len]);
    combined[first_len..first_len + rest_len].copy_from_slice(&rest[..rest_len]);
    assert_eq!(&combined[..first_len + rest_len], b"\x1b[0n");
}

#[test]
fn req_007_overflow_bound() {
    use suprtui::term_embedded::RESPONSE_LIMIT;
    let mut terminal = terminal(20, 4);

    let query = b"\x1b[5n";
    let count = RESPONSE_LIMIT / query.len() + 1;
    let input = query.repeat(count);
    terminal.write(&input);

    let mut byte = [0u8; 1];
    assert_eq!(
        terminal.drain_responses(&mut byte),
        Err(EmbeddedError::ResponseOverflow)
    );

    // Queued bytes before the overflow are preserved for later drains.
    let mut preserved = [0u8; 16];
    let preserved_len = terminal.drain_responses(&mut preserved).unwrap();
    assert!(preserved_len > 0);
}
