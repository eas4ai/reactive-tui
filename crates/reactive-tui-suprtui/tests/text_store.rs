//! Integration tests for the text-store commitment (TXT-001 … TXT-004).

use suprtui::text::{Span, TextBuffer};

#[test]
fn req_001_edits() {
    let mut buf = TextBuffer::from_text("alpha\nbeta\ngamma");
    buf.insert(5, "!").unwrap();
    buf.delete(0, 6).unwrap();
    assert_eq!("\nbeta\ngamma", buf.content());
    let tail = buf.split_off(6).unwrap();
    assert_eq!("\nbeta\n", buf.content());
    assert_eq!("gamma", tail.content());
    assert_eq!(11, buf.len() + tail.len());
    assert!(TextBuffer::new().is_empty());
}

#[test]
fn req_002_span_tracking() {
    let mut buf = TextBuffer::from_text("one two three");
    buf.add_span(4, 7, 2).unwrap();
    buf.insert(4, "big ").unwrap();
    assert_eq!(
        [Span {
            start: 8,
            end: 11,
            style: 2
        }],
        buf.spans()
    );
    assert_eq!("one big two three", buf.content());
    buf.delete(0, 8).unwrap();
    assert_eq!("two three", buf.content());
    assert_eq!(
        [Span {
            start: 0,
            end: 3,
            style: 2
        }],
        buf.spans()
    );
}

#[test]
fn req_003_undo_redo() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "a").unwrap();
    buf.insert(1, "b").unwrap();
    buf.add_span(0, 2, 5).unwrap();
    assert_eq!("ab", buf.content());
    assert!(buf.undo());
    assert!(buf.undo());
    assert_eq!("a", buf.content());
    assert!(buf.spans().is_empty());
    assert!(buf.redo());
    assert_eq!("ab", buf.content());
    buf.delete(0, 2).unwrap();
    assert!(!buf.can_redo());
    assert_eq!("", buf.content());
    assert!(buf.undo());
    assert_eq!("ab", buf.content());
}

#[test]
fn req_004_view_dirty_tracking() {
    let mut buf = TextBuffer::from_text("x");
    let view = buf.register_view();
    assert!(buf.view_dirty(view));
    buf.clear_view_dirty(view);
    let epoch = buf.epoch();
    buf.delete(0, 1).unwrap();
    assert!(buf.epoch() > epoch);
    assert!(buf.view_dirty(view));
    buf.clear_view_dirty(view);
    let tail = buf.split_off(0).unwrap();
    assert!(buf.view_dirty(view));
    assert_eq!("", buf.content());
    assert_eq!("", tail.content());
}
