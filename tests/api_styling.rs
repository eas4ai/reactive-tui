use reactive_tui::{
    app::RootComponent,
    builder::core::div,
    component::{Element, FocusProps},
    event::types::{Event, KeyCode, MouseEvent, MouseEventKind, Position},
};

mod common;
use app_input::{click, key, run, Snapshot};
use common::app_input;
use reactive_tui::event::router::EventResult;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

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

struct DisabledRoot {
    disabled: AtomicBool,
    calls: Arc<AtomicUsize>,
}
impl RootComponent for DisabledRoot {
    fn render(&self) -> Element {
        let calls = self.calls.clone();
        div()
            .class("flex flex-col w-full h-full")
            .children(vec![
                div()
                    .key("disabled")
                    .text("A")
                    .class("w-8 h-1 bg-black disabled:bg-red-500")
                    .disabled(self.disabled.load(Ordering::SeqCst))
                    .on_click(move || {
                        calls.fetch_add(1, Ordering::SeqCst);
                    })
                    .build()
                    .with_focus(FocusProps {
                        auto_focus: true,
                        ..Default::default()
                    }),
                focus_target("B", false),
            ])
            .build()
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        if matches!(event, Event::Key(key) if key.code == KeyCode::Char('d')) {
            self.disabled.fetch_xor(true, Ordering::SeqCst);
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn disabled_nodes_skip_autofocus_tab_and_activation_then_reenable() {
    let calls = Arc::new(AtomicUsize::new(0));
    let frames = run(
        DisabledRoot {
            disabled: AtomicBool::new(true),
            calls: calls.clone(),
        },
        (12, 4),
        vec![
            (1, click(1, 0)),
            (1, key(KeyCode::Enter)),
            (1, key(KeyCode::Tab)),
            (2, key(KeyCode::Char('d'))),
            (3, key(KeyCode::Enter)),
            (4, None),
        ],
    );
    assert_eq!(background(&frames[0], 0, 0), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(background(&frames[1], 1, 0), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(background(&frames[2], 0, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn focus_within_and_compound_variants_require_their_actual_states() {
    let root = div()
        .class("flex flex-col w-full h-full bg-black focus-within:bg-red-500")
        .child(focus_target("A", false).class("w-8 h-1 bg-black focus:hover:bg-red-500"))
        .build();
    let pointer = Some(Event::Mouse(MouseEvent::new(
        MouseEventKind::Move,
        Position::cell(1, 0),
    )));
    let frames = run(
        TreeRoot(root),
        (12, 4),
        vec![(1, pointer), (2, key(KeyCode::Tab)), (3, None)],
    );
    assert_eq!(background(&frames[0], 3, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(background(&frames[1], 0, 0), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(background(&frames[2], 0, 0), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(background(&frames[2], 3, 0), vt100::Color::Rgb(239, 68, 68));
}

#[test]
fn wrapping_measurement_places_the_next_sibling_after_all_text_rows() {
    let root = div()
        .class("flex flex-col w-full h-full")
        .children(vec![
            Element::text("one two three").class("w-full shrink-0 whitespace-normal"),
            Element::text("after").class("w-full h-1 shrink-0"),
        ])
        .build();
    let frames = run(TreeRoot(root), (7, 5), vec![(1, None)]);
    assert_eq!(visible_text(&frames[0]), "one two\nthree\nafter");
    assert_eq!(frames[0].geometry[2].bounds.y, 2.0);
}

#[test]
fn text_justification_expands_word_gaps_only_before_soft_wraps() {
    assert_eq!(
        text_frame("a b cc dddd", "whitespace-normal text-justify", (8, 3)).text,
        "a  b  cc\ndddd"
    );
    assert_eq!(
        text_frame("a b\nc d", "whitespace-pre text-justify", (8, 3)).text,
        "a b\nc d"
    );
}

#[test]
fn leading_tokens_space_baselines_and_explicit_reset_overrides_inheritance() {
    for (token, expected) in [
        ("leading-none", "a\nb"),
        ("leading-tight", "a\nb"),
        ("leading-snug", "a\nb"),
        ("leading-normal", "a\n\nb"),
        ("leading-relaxed", "a\n\nb"),
        ("leading-loose", "a\n\nb"),
        ("leading-3", "a\n\n\nb"),
        ("leading-0", "a\nb"),
    ] {
        assert_eq!(
            text_frame("a\nb", &format!("leading-3 {token}"), (8, 5)).text,
            expected,
            "{token}"
        );
    }
    let root = div()
        .class("flex flex-col w-full h-full leading-3")
        .children(vec![
            Element::text("a\nb").class("w-full shrink-0"),
            Element::text("c\nd").class("w-full shrink-0 leading-none"),
        ])
        .build();
    assert_eq!(
        visible_text(&run(TreeRoot(root), (8, 8), vec![(1, None)])[0]),
        "a\n\n\nb\nc\nd"
    );
    assert_eq!(text_frame("a\nb", "leading-65535", (8, 2)).text, "a");
}

#[test]
fn physical_font_aliases_and_fractional_tracking_keep_terminal_cell_geometry() {
    for token in [
        "text-xs",
        "text-sm",
        "text-base",
        "text-lg",
        "text-xl",
        "text-2xl",
        "text-3xl",
        "text-4xl",
        "text-5xl",
        "text-6xl",
        "text-7xl",
        "text-8xl",
        "text-9xl",
        "font-sans",
        "font-serif",
        "font-mono",
        "tracking-tighter",
        "tracking-tight",
        "tracking-normal",
        "tracking-wide",
        "tracking-wider",
        "tracking-widest",
    ] {
        assert_eq!(
            text_frame("界e\u{301}", token, (8, 2)).text,
            "界e\u{301}",
            "{token}"
        );
    }
}

#[test]
fn tabs_use_four_cell_stops_and_wrapping_keeps_combining_clusters() {
    assert_eq!(text_frame("界\tx", "whitespace-pre", (8, 2)).text, "界  x");
    assert_eq!(
        text_frame(
            "e\u{301}e\u{301}e\u{301}",
            "whitespace-pre-wrap break-all",
            (2, 3)
        )
        .text,
        "e\u{301}e\u{301}\ne\u{301}"
    );
    assert_eq!(text_frame("a\x1bb\x07", "uppercase", (8, 2)).text, "AB");
}

#[test]
fn font_weights_decorations_and_resets_reach_terminal_attributes() {
    for (class, bold) in [
        ("font-thin", false),
        ("font-extralight", false),
        ("font-light", false),
        ("font-normal", false),
        ("font-medium", false),
        ("font-semibold", false),
        ("font-bold", true),
        ("font-extrabold", true),
        ("font-black", true),
        ("font-bold font-normal", false),
    ] {
        assert_eq!(
            text_frame("a", &format!("font-bold {class}"), (4, 2))
                .screen
                .cell(0, 0)
                .unwrap()
                .bold(),
            bold,
            "{class}"
        );
    }
    for (class, italic, underline) in [
        ("italic", true, false),
        ("italic not-italic", false, false),
        ("underline", false, true),
        ("overline", false, true),
        ("underline no-underline", false, false),
    ] {
        let frame = text_frame("a", class, (4, 2));
        let cell = frame.screen.cell(0, 0).unwrap();
        assert_eq!(
            (cell.italic(), cell.underline()),
            (italic, underline),
            "{class}"
        );
    }
    let root = div()
        .class("flex flex-col w-full h-full font-bold italic underline")
        .children(vec![
            Element::text("a").class("h-1"),
            Element::text("b").class("h-1 font-normal not-italic no-underline"),
        ])
        .build();
    let frames = run(TreeRoot(root), (4, 3), vec![(1, None)]);
    for (row, expected) in [(0, true), (1, false)] {
        let cell = frames[0].screen.cell(row, 0).unwrap();
        assert_eq!(
            (cell.bold(), cell.italic(), cell.underline()),
            (expected, expected, expected)
        );
    }
}

#[test]
fn stateless_layout_does_not_invent_interaction_state() {
    use reactive_tui::layout::{css, style::StyleBuilder};
    use reactive_tui::ui::paint::extract_paint_style;
    for token in [
        "focus:bg-red-500",
        "focus-within:bg-red-500",
        "hover:bg-red-500",
        "disabled:bg-red-500",
    ] {
        let mut sb = css::apply_utility_classes(token, StyleBuilder::new());
        let paint = extract_paint_style(&mut sb).unwrap_or_default();
        assert_eq!(
            paint.bg,
            reactive_tui::ui::paint::PaintStyle::default().bg,
            "{token}"
        );
    }
}

#[test]
fn strikethrough_reaches_the_host_as_sgr_nine() {
    // vt100 does not expose strikethrough, so inspect complete SGR commands.
    let sgr = regex::Regex::new("\u{1b}\\[([0-9;]*)m").unwrap();
    for (class, expected) in [("line-through", true), ("", false)] {
        let frame = text_frame("a", class, (4, 2));
        let output = String::from_utf8(frame.output).unwrap();
        assert_eq!(
            sgr.captures_iter(&output)
                .any(|capture| capture[1].split(';').any(|code| code == "9")),
            expected
        );
    }
}

struct MovingHoverRoot(AtomicBool);
impl RootComponent for MovingHoverRoot {
    fn render(&self) -> Element {
        let top = if self.0.load(Ordering::SeqCst) {
            "top-2"
        } else {
            "top-0"
        };
        div()
            .class("relative w-full h-full")
            .child(Element::text("move").key("target").class(format!(
                "absolute {top} left-0 w-8 h-1 bg-black hover:bg-red-500"
            )))
            .build()
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        if matches!(event, Event::Key(key) if key.code == KeyCode::Char('m')) {
            self.0.store(true, Ordering::SeqCst);
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn stationary_pointer_is_retested_after_changed_geometry_is_presented() {
    let pointer = Some(Event::Mouse(MouseEvent::new(
        MouseEventKind::Move,
        Position::cell(1, 0),
    )));
    let frames = run(
        MovingHoverRoot(AtomicBool::new(false)),
        (12, 4),
        vec![(1, pointer), (2, key(KeyCode::Char('m'))), (4, None)],
    );
    assert_eq!(background(&frames[1], 0, 0), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(frames[3].geometry[1].bounds.y, 2.0);
    assert_eq!(background(&frames[3], 2, 0), vt100::Color::Rgb(0, 0, 0));
}
