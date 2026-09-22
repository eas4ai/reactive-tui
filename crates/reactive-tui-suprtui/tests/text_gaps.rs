//! Integration tests for the text-gaps commitment (TXT-011 … TXT-015).
//!
//! Vectors ported from the reference gesture, viewport, iterator,
//! wrap-cache, and editor-view suites; the spec-literal unit tests
//! live in `src/text.rs`.

use suprtui::text::{
    EditorView, GestureBehavior, SelectionOccupancy, TextBuffer, TextView, Viewport, WrapMode,
};

fn make_view(text: &str) -> TextView {
    TextView::new(TextBuffer::from_text(text))
}

fn text_of(view: &TextView) -> Option<&str> {
    view.selected_text()
}

fn rows(view: &mut TextView) -> Vec<String> {
    let cached = view.virtual_lines().to_vec();
    cached
        .iter()
        .map(|v| view.buffer().content()[v.start..v.end].to_string())
        .collect()
}

// ---- TXT-011: gesture selection ----

#[test]
fn req_011_word_press_drag_release() {
    let mut view = make_view("alpha beta gamma");
    // Press on "beta", drag onto "gamma": both words selected.
    view.gesture_press(0, 7, GestureBehavior::Word).unwrap();
    assert!(view.gesture_active());
    view.gesture_move(0, 13).unwrap();
    assert_eq!(Some((6, 16)), view.gesture_release());
    assert_eq!(Some("beta gamma"), text_of(&view));
    assert!(!view.gesture_active());
}

#[test]
fn req_011_backward_drag_matches_forward() {
    let mut backward = make_view("alpha beta gamma");
    let mut fwd = make_view("alpha beta gamma");
    fwd.gesture_press(0, 7, GestureBehavior::Word).unwrap();
    fwd.gesture_move(0, 13).unwrap();
    let fwd_range = fwd.gesture_release();
    backward
        .gesture_press(0, 13, GestureBehavior::Word)
        .unwrap();
    backward.gesture_move(0, 7).unwrap();
    assert_eq!(fwd_range, backward.gesture_release());
    assert_eq!(Some("beta gamma"), text_of(&backward));
}

#[test]
fn req_011_space_run_selects() {
    let mut view = make_view("alpha  beta");
    view.gesture_press(0, 5, GestureBehavior::Word).unwrap();
    assert_eq!(Some((5, 7)), view.gesture_release());
    assert_eq!(Some("  "), text_of(&view));
}

#[test]
fn req_011_padding_click_is_zero_width() {
    let mut view = make_view("ab");
    view.gesture_press(0, 10, GestureBehavior::Word).unwrap();
    assert_eq!(Some((2, 2)), view.gesture_release());
    assert_eq!(Some(""), text_of(&view));
}

#[test]
fn req_011_slash_is_not_a_boundary() {
    let mut view = make_view("a/b cd");
    view.gesture_press(0, 1, GestureBehavior::Word).unwrap();
    assert_eq!(Some((0, 3)), view.gesture_release());
    assert_eq!(Some("a/b"), text_of(&view));
}

#[test]
fn req_011_wide_word_groups() {
    let mut view = make_view("日本語abc x");
    view.gesture_press(0, 1, GestureBehavior::Word).unwrap();
    assert_eq!(Some("日本語abc"), text_of(&view));
}

#[test]
fn req_011_line_drag_unions_lines() {
    let mut view = make_view("one\ntwo\nthree");
    view.gesture_press(0, 1, GestureBehavior::Line).unwrap();
    view.gesture_move(1, 1).unwrap();
    assert_eq!(Some((0, 7)), view.gesture_release());
    assert_eq!(Some("one\ntwo"), text_of(&view));
}

#[test]
fn req_011_convert_word_to_cell_keeps_text() {
    let mut view = make_view("alpha beta");
    view.gesture_press(0, 7, GestureBehavior::Word).unwrap();
    view.gesture_release();
    view.gesture_convert_to_cell();
    assert_eq!(SelectionOccupancy::Cell, view.gesture_occupancy());
    assert_eq!(Some("beta"), text_of(&view));
}

#[test]
fn req_011_cell_press_is_zero_width() {
    let mut view = make_view("alpha beta");
    view.gesture_press(0, 3, GestureBehavior::Cell).unwrap();
    assert_eq!(Some((3, 3)), view.gesture_release());
}

// ---- TXT-012: viewport selection ----

#[test]
fn req_012_vertical_viewport_selection() {
    let mut view = make_view("aaa\nbbb\nccc\nddd");
    view.select_viewport(Viewport::new(1, 0), 0, 0, 1, 3)
        .unwrap();
    assert_eq!(Some("bbb\nccc"), text_of(&view));
}

#[test]
fn req_012_scrolled_matches_unscrolled() {
    let text = "aaa\nbbb\nccc\nddd";
    let mut scrolled = make_view(text);
    scrolled
        .select_viewport(Viewport::new(1, 0), 0, 1, 1, 2)
        .unwrap();
    let mut plain = make_view(text);
    plain.set_selection(5, 10).unwrap();
    assert_eq!(text_of(&plain), text_of(&scrolled));
}

#[test]
fn req_012_horizontal_viewport_offset() {
    let mut view = make_view("abcdefgh");
    view.set_wrap_mode(WrapMode::None);
    view.select_viewport(Viewport::new(0, 3), 0, 0, 0, 2)
        .unwrap();
    assert_eq!(Some("de"), text_of(&view));
}

#[test]
fn req_012_wrapping_ignores_horizontal_offset() {
    let mut wrapped = make_view("abcdefgh");
    wrapped.set_wrap_mode(WrapMode::Char);
    wrapped.set_width(80);
    wrapped
        .select_viewport(Viewport::new(0, 3), 0, 0, 0, 2)
        .unwrap();
    let mut plain = make_view("abcdefgh");
    plain.set_selection(0, 2).unwrap();
    assert_eq!(text_of(&plain), text_of(&wrapped));
}

#[test]
fn req_012_across_empty_line() {
    let mut view = make_view("aa\n\nbb");
    view.select_viewport(Viewport::new(0, 0), 0, 0, 2, 2)
        .unwrap();
    assert_eq!(Some("aa\n\nbb"), text_of(&view));
}

// ---- TXT-013: iterators ----

#[test]
fn req_013_clusters_never_split() {
    // Combining mark joins; ZWJ joins emoji (UAX #29 GB11) but a
    // ZWJ after a plain letter still breaks before the next letter.
    let buf =
        TextBuffer::from_text("a\u{301}\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}b\u{200d}c");
    let clusters = buf.clusters(0, buf.len()).unwrap();
    let texts: Vec<&str> = clusters.iter().map(|c| c.text).collect();
    assert_eq!(
        vec![
            "a\u{301}",
            "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}",
            "b\u{200d}",
            "c"
        ],
        texts
    );
    // Ranges concatenate back to the source slice.
    let joined: String = texts.concat();
    assert_eq!(&joined, buf.content());
}

#[test]
fn req_013_zero_width_prefix_terminates() {
    let buf = TextBuffer::from_text("\u{200d}\u{200d}ab");
    let clusters = buf.clusters(0, buf.len()).unwrap();
    assert!(!clusters.is_empty());
    assert!(clusters.iter().all(|c| !c.text.is_empty()));
    let end = clusters.last().unwrap().end;
    assert_eq!(buf.len(), end);
}

#[test]
fn req_013_lines_and_coords_round_trip() {
    let buf = TextBuffer::from_text("ab\ncde\n\nf");
    assert_eq!(vec![(0, 2), (3, 6), (7, 7), (8, 9)], buf.line_ranges());
    for offset in 0..=buf.len() {
        if buf.content().is_char_boundary(offset) {
            let (row, col) = buf.offset_to_coords(offset).unwrap();
            assert_eq!(offset, buf.coords_to_offset(row, col).unwrap());
        }
    }
    assert!(buf.coords_to_offset(9, 0).is_err());
    assert!(buf.clusters(3, 1).is_err());
}

// ---- TXT-014: wrap cache ----

#[test]
fn req_014_repeated_queries_share_layout() {
    let mut view = make_view("aaa bbb ccc");
    view.set_wrap_mode(WrapMode::Word);
    view.set_width(5);
    view.virtual_lines();
    let first = view.recompute_count();
    assert_eq!(1, first);
    for _ in 0..5 {
        view.virtual_lines();
    }
    assert_eq!(first, view.recompute_count());
    view.set_width(5);
    view.set_wrap_mode(WrapMode::Word);
    view.virtual_lines();
    assert_eq!(first, view.recompute_count());
}

#[test]
fn req_014_edits_invalidate_no_stale_rows() {
    let mut view = make_view("aaa bbb");
    view.set_wrap_mode(WrapMode::Word);
    view.set_width(4);
    let before = rows(&mut view);
    assert_eq!(vec!["aaa ", "bbb"], before);
    let count = view.recompute_count();
    view.buffer_mut().insert(0, "z ").unwrap();
    let after = rows(&mut view);
    assert_eq!(count + 1, view.recompute_count());
    assert_ne!(before, after);
}

// ---- TXT-015: editor view ----

#[test]
fn req_015_viewport_scrolls_content() {
    let text = (0..10)
        .map(|i| format!("l{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut editor = EditorView::new(TextBuffer::from_text(&text));
    editor.set_viewport_size(80, 4);
    editor.scroll_to(6, 0);
    assert_eq!(Viewport::new(6, 0), editor.viewport());
}

#[test]
fn req_015_cursor_scrolls_into_view() {
    let text = (0..30)
        .map(|i| format!("line{i:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut editor = EditorView::new(TextBuffer::from_text(&text));
    editor.set_viewport_size(80, 8);
    editor.text_view_mut().set_cursor(150).unwrap();
    editor.ensure_cursor_visible(1);
    let (row, _) = editor.visual_cursor();
    let viewport = editor.viewport();
    assert!(row >= viewport.first_row, "row {row} above window");
    assert!(row < viewport.first_row + 8, "row {row} below window");
}

#[test]
fn req_015_local_selection_set_update_reset() {
    let mut editor = EditorView::new(TextBuffer::from_text("alpha beta gamma"));
    editor
        .set_local_selection(0, 0, 0, 4, GestureBehavior::Cell)
        .unwrap();
    assert_eq!(Some("alph"), editor.text_view().selected_text());
    editor.update_local_selection(0, 9).unwrap();
    assert_eq!(Some("alpha bet"), editor.text_view().selected_text());
    editor.reset_local_selection();
    assert_eq!(None, editor.text_view().selected_text());
}

#[test]
fn req_015_local_word_selection_and_convert() {
    let mut editor = EditorView::new(TextBuffer::from_text("alpha beta"));
    editor
        .set_local_selection(0, 1, 0, 1, GestureBehavior::Word)
        .unwrap();
    assert_eq!(Some("alpha"), editor.text_view().selected_text());
    editor.convert_selection_to_cell();
    // Block occupancy includes the grapheme under the max endpoint
    // (the space): text kept, nothing dropped.
    assert_eq!(Some("alpha "), editor.text_view().selected_text());
}

#[test]
fn req_015_follow_cursor_syncs_focus() {
    let mut editor = EditorView::new(TextBuffer::from_text("alpha beta gamma"));
    editor.set_selection_follow_cursor(true);
    editor
        .set_local_selection(0, 0, 0, 9, GestureBehavior::Cell)
        .unwrap();
    assert_eq!(9, editor.text_view().cursor());
}

#[test]
fn req_015_visual_round_trip() {
    let mut editor = EditorView::new(TextBuffer::from_text("ab\ncdef"));
    editor.set_viewport_size(80, 24);
    for offset in [0, 1, 2, 3, 5, 7] {
        editor.text_view_mut().set_cursor(offset).unwrap();
        let (row, col) = editor.visual_cursor();
        assert_eq!(Some(offset), editor.visual_to_offset(row, col));
    }
}
