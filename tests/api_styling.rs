use reactive_tui::{
    app::RootComponent,
    builder::core::div,
    component::{Element, FocusProps},
    event::types::{Event, KeyCode, MouseEvent, MouseEventKind, Position},
};

#[path = "support/app_input.rs"]
mod app_input;
use app_input::{click, key, run, Snapshot};

struct TreeRoot(Element);
impl RootComponent for TreeRoot {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn text_frame(text: &str, class: &str, size: (u16, u16)) -> Snapshot {
    let mut frame = run(
        TreeRoot(Element::text(text).class(format!("w-full h-full {class}"))),
        size,
        vec![(1, None)],
    )
    .remove(0);
    frame.text = visible_text(&frame);
    frame
}

fn visible_text(frame: &Snapshot) -> String {
    // Ignore unused cells at each row's right edge and empty bottom rows,
    // retaining leading alignment spaces and all spaces between characters.
    frame
        .text
        .lines()
        .map(|line| line.trim_end_matches(' '))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end_matches('\n')
        .to_owned()
}

fn background(frame: &Snapshot, row: u16, col: u16) -> vt100::Color {
    frame.screen.cell(row, col).unwrap().bgcolor()
}

#[test]
fn plain_text_and_basic_color_are_an_independent_control() {
    let frame = text_frame("plain 界e\u{301}", "bg-red-500 text-white", (20, 3));
    assert_eq!(frame.text, "plain 界e\u{301}");
    assert_eq!(background(&frame, 0, 0), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(
        frame.screen.cell(0, 0).unwrap().fgcolor(),
        vt100::Color::Rgb(255, 255, 255)
    );
}

#[test]
fn focus_variant_is_inactive_without_focus() {
    let frame = text_frame("idle", "bg-black focus:bg-red-500", (12, 2));
    assert_eq!(background(&frame, 0, 0), vt100::Color::Rgb(0, 0, 0));
}

#[test]
fn hover_variant_is_inactive_without_a_pointer() {
    let frame = text_frame("idle", "bg-black hover:bg-red-500", (12, 2));
    assert_eq!(background(&frame, 0, 0), vt100::Color::Rgb(0, 0, 0));
}

#[test]
fn disabled_variant_is_inactive_on_an_enabled_element() {
    let frame = text_frame("enabled", "bg-black disabled:bg-red-500", (12, 2));
    assert_eq!(background(&frame, 0, 0), vt100::Color::Rgb(0, 0, 0));
}

fn focus_target(label: &str, auto_focus: bool) -> Element {
    Element::text(label)
        .with_key(label)
        .class("w-8 h-1 bg-black focus:bg-red-500")
        .with_focus(FocusProps {
            auto_focus,
            ..Default::default()
        })
}

#[test]
fn tab_moves_the_focus_style_between_actual_nodes() {
    let root = div()
        .class("flex flex-col w-full h-full")
        .children(vec![focus_target("A", false), focus_target("B", false)])
        .build();
    let frames = run(
        TreeRoot(root),
        (12, 4),
        vec![(1, key(KeyCode::Tab)), (2, key(KeyCode::Tab)), (3, None)],
    );
    assert_eq!(background(&frames[0], 0, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(background(&frames[0], 1, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(background(&frames[1], 0, 0), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(background(&frames[1], 1, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(background(&frames[2], 0, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(background(&frames[2], 1, 0), vt100::Color::Rgb(239, 68, 68));
}

#[test]
fn mounting_an_autofocus_target_requests_its_state_repaint() {
    // Focus is registered after the first acknowledged frame. Its new state
    // must schedule a second frame even when the application is otherwise idle.
    let frames = run(TreeRoot(focus_target("A", true)), (12, 3), vec![(2, None)]);
    assert_eq!(background(&frames[1], 0, 0), vt100::Color::Rgb(239, 68, 68));
}

#[test]
fn mouse_focus_repaints_without_an_activation_callback() {
    let root = div()
        .class("flex flex-col w-full h-full")
        .children(vec![focus_target("A", false), focus_target("B", false)])
        .build();
    let frames = run(
        TreeRoot(root),
        (12, 4),
        vec![(1, click(1, 0)), (2, click(1, 1)), (3, None)],
    );
    assert_eq!(background(&frames[1], 0, 0), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(background(&frames[1], 1, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(background(&frames[2], 0, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(background(&frames[2], 1, 0), vt100::Color::Rgb(239, 68, 68));
}

#[test]
fn hover_enter_and_leave_repaint_without_application_handlers() {
    let root = div()
        .class("flex flex-col w-full h-full")
        .child(Element::text("hover").class("w-8 h-1 bg-black hover:bg-red-500"))
        .build();
    let pointer = |x, y| {
        Some(Event::Mouse(MouseEvent::new(
            MouseEventKind::Move,
            Position::cell(x, y),
        )))
    };
    let frames = run(
        TreeRoot(root),
        (12, 4),
        vec![(1, pointer(1, 0)), (2, pointer(1, 3)), (3, None)],
    );
    assert_eq!(background(&frames[0], 0, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(background(&frames[1], 0, 0), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(background(&frames[2], 0, 0), vt100::Color::Rgb(0, 0, 0));
}

#[test]
fn unicode_text_transforms_change_the_painted_text() {
    for (class, input, expected) in [
        ("uppercase", "straße élan", "STRASSE ÉLAN"),
        ("lowercase", "STRAẞE ÉLAN", "straße élan"),
        ("capitalize", "élan mixed-case", "Élan Mixed-Case"),
        ("uppercase normal-case", "MiXeD", "MiXeD"),
    ] {
        assert_eq!(text_frame(input, class, (24, 2)).text, expected, "{class}");
    }
}

#[test]
fn parent_typography_reaches_text_and_child_tokens_override_it() {
    let root = div()
        .class("flex flex-col w-full h-full uppercase")
        .children(vec![
            Element::text("inherited").class("h-1"),
            Element::text("MiXeD").class("h-1 normal-case"),
        ])
        .build();
    let frames = run(TreeRoot(root), (16, 4), vec![(1, None)]);
    assert_eq!(visible_text(&frames[0]), "INHERITED\nMiXeD");
}

#[test]
fn truncation_reserves_an_ellipsis_without_splitting_graphemes() {
    for (text, width, expected) in [
        ("abcdefghi", 5, "abcd…"),
        ("界界界", 5, "界界…"),
        ("e\u{301}e\u{301}e\u{301}", 2, "e\u{301}…"),
        ("🙂🙂", 1, "…"),
        ("short", 5, "short"),
    ] {
        assert_eq!(
            text_frame(text, "truncate", (width, 2)).text,
            expected,
            "{text:?} in {width} cells"
        );
    }
}

#[test]
fn explicit_ellipsis_and_clip_have_distinct_output() {
    assert_eq!(
        text_frame("abcdef", "text-ellipsis whitespace-nowrap", (5, 2)).text,
        "abcd…"
    );
    assert_eq!(
        text_frame("界界界", "text-clip whitespace-nowrap", (5, 2)).text,
        "界界"
    );
}

#[test]
fn whitespace_normal_collapses_and_wraps_at_words() {
    let frame = text_frame("a  b\nc d", "whitespace-normal", (5, 3));
    assert_eq!(frame.text, "a b c\nd");
}

#[test]
fn whitespace_nowrap_collapses_without_creating_a_soft_line() {
    let frame = text_frame("a  b\nc d", "whitespace-nowrap", (5, 3));
    assert_eq!(frame.text, "a b c");
}

#[test]
fn whitespace_pre_preserves_explicit_lines_and_spaces() {
    let frame = text_frame("a  b\nc d", "whitespace-pre", (5, 3));
    assert_eq!(frame.text, "a  b\nc d");
}

#[test]
fn whitespace_pre_line_keeps_newlines_and_collapses_spaces() {
    let frame = text_frame("a  b\nc d", "whitespace-pre-line", (5, 3));
    assert_eq!(frame.text, "a b\nc d");
}

#[test]
fn word_break_modes_use_cell_width_and_keep_graphemes_whole() {
    for (class, text, expected) in [
        ("whitespace-normal break-normal", "ab abcdef", "ab\nabcd"),
        ("whitespace-normal break-words", "ab abcdef", "ab\nabcd\nef"),
        ("whitespace-normal break-all", "abcdef", "abcd\nef"),
        ("whitespace-pre-wrap break-all", "界界界", "界界\n界"),
    ] {
        assert_eq!(text_frame(text, class, (4, 4)).text, expected, "{class}");
    }
}

#[test]
fn text_alignment_positions_cells_instead_of_flex_children() {
    for (class, expected) in [
        ("text-left", "ab"),
        ("text-center", "   ab"),
        ("text-right", "      ab"),
    ] {
        assert_eq!(text_frame("ab", class, (8, 2)).text, expected, "{class}");
    }
}
