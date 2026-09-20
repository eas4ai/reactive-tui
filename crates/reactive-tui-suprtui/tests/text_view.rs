//! Integration tests for the text-view commitment (TXT-005 … TXT-010).

use suprtui::text::{TextBuffer, TextView, WrapMode};

fn make_view(text: &str) -> TextView {
    TextView::new(TextBuffer::from_text(text))
}

fn rows(view: &mut TextView) -> Vec<String> {
    let cached = view.virtual_lines().to_vec();
    cached
        .iter()
        .map(|v| view.buffer().content()[v.start..v.end].to_string())
        .collect()
}

#[test]
fn req_005_wrapping() {
    let mut view = make_view("one two three four");
    view.set_wrap_mode(WrapMode::Word);
    view.set_width(9);
    assert_eq!(vec!["one two ", "three ", "four"], rows(&mut view));
    let mut narrow = make_view("abcdefghij");
    narrow.set_wrap_mode(WrapMode::Char);
    narrow.set_width(4);
    assert_eq!(vec!["abcd", "efgh", "ij"], rows(&mut narrow));
}

#[test]
fn req_006_selection() {
    let mut view = make_view("red green blue");
    view.select_word_at(5).unwrap();
    assert_eq!(Some("green"), view.selected_text());
    view.select_line_at(14).unwrap();
    assert_eq!(Some("red green blue"), view.selected_text());
    view.set_selection(0, 3).unwrap();
    assert_eq!(Some("red"), view.selected_text());
}

#[test]
fn req_007_cursor_cluster_safety() {
    let mut view = make_view("caf\u{e9}\nx");
    view.set_cursor(5).unwrap();
    view.backspace().unwrap();
    assert_eq!("caf\nx", view.buffer().content());
    assert_eq!(3, view.cursor());
    assert_eq!(2, view.move_left());
    assert_eq!(3, view.move_right());
    view.insert_text("z").unwrap();
    assert_eq!("cafz\nx", view.buffer().content());
    view.delete_at_cursor().unwrap();
    assert_eq!("cafzx", view.buffer().content());
}

#[test]
fn req_008_highlight_refs() {
    let mut view = make_view("aaa bbb ccc");
    view.add_highlight(7, 0, 3, 1).unwrap();
    view.add_highlight(8, 4, 7, 2).unwrap();
    view.add_highlight(9, 8, 11, 3).unwrap();
    assert!(view.remove_highlight(8));
    let ids: Vec<u32> = view.highlights().iter().map(|h| h.id).collect();
    assert_eq!(vec![7, 9], ids);
    assert_eq!(2, view.resolved_spans().len());
}

#[test]
fn req_009_syntax_style() {
    let mut view = make_view("let x = 1; let y = 2;");
    let count = view.attach_syntax_style("kw", "let", 8).unwrap();
    assert_eq!(2, count);
    assert_eq!(2, view.buffer().spans().len());
    assert_eq!(1, view.syntax_styles().len());
}

#[test]
fn req_010_offset_bounds() {
    let mut view = make_view("line1\nline2\nline3");
    assert_eq!(3, view.line_count());
    assert_eq!(Ok((0, 5)), view.line_range(0));
    assert!(view.set_cursor(100).is_err());
    assert_eq!(6, view.move_down());
    let mut top = make_view("ab");
    top.set_cursor(2).unwrap();
    assert_eq!(0, top.move_up());
    assert_eq!(2, top.move_down());
}
