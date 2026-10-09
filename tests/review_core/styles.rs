//! Part of tests/review_core.rs.
use crate::common::app_input::{click, run_until, run_when_painted, Until};
use reactive_tui::{
    app::RootComponent,
    backend::{Backend, SuprTuiBackend},
    builder::core::div,
    component::Element,
    css,
    event::{
        router::EventResult,
        types::{Event, MouseButton, MouseEvent, MouseEventKind, Position},
    },
    layout::{css::apply_utility_classes, paint_tree::cells::CellGrid, style::StyleBuilder},
};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use taffy::style::Display;

struct TreeRoot(Element);
impl RootComponent for TreeRoot {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
    fn handle_event(&self, _: &Event) -> EventResult {
        EventResult::Handled
    }
}

fn assert_context_free_opacity(variant: &str) {
    let mut style = apply_utility_classes(
        &format!("{variant}:opacity-50"),
        StyleBuilder::new().fg_rgba(1.0, 1.0, 1.0, 1.0),
    );
    let alpha = style.take_visuals().unwrap().fg.a;
    assert_eq!(
        alpha, 1.0,
        "STY-001: {variant}:opacity-50 without context changed foreground alpha to {alpha}"
    );
}

#[test]
fn sty_001_active_without_context_leaves_opacity_unchanged() {
    assert_context_free_opacity("active");
}

#[test]
fn sty_001_visited_without_context_leaves_opacity_unchanged() {
    assert_context_free_opacity("visited");
}

#[test]
fn sty_001_first_without_context_leaves_opacity_unchanged() {
    assert_context_free_opacity("first");
}

#[test]
fn sty_001_last_without_context_leaves_opacity_unchanged() {
    assert_context_free_opacity("last");
}

#[test]
fn sty_001_odd_without_context_leaves_opacity_unchanged() {
    assert_context_free_opacity("odd");
}

#[test]
fn sty_001_even_without_context_leaves_opacity_unchanged() {
    assert_context_free_opacity("even");
}

#[test]
fn sty_001_group_hover_without_context_leaves_opacity_unchanged() {
    assert_context_free_opacity("group-hover");
}

#[test]
fn sty_001_group_focus_without_context_leaves_opacity_unchanged() {
    assert_context_free_opacity("group-focus");
}

#[test]
fn sty_001_group_active_without_context_leaves_opacity_unchanged() {
    assert_context_free_opacity("group-active");
}

fn assert_positional_backgrounds(variant: &str, matches: [bool; 3]) {
    let root = div()
        .class("flex flex-col w-full h-full")
        .children(
            ["A", "B", "C"]
                .into_iter()
                .map(|text| {
                    Element::text(text).class(format!("w-3 h-1 bg-black {variant}:bg-red-500"))
                })
                .collect(),
        )
        .build();
    let frames = run_when_painted(TreeRoot(root), (8, 4), 1);
    let screen = &frames.last().unwrap().screen;
    let observed: Vec<_> = (0..3)
        .map(|row| screen.cell(row, 0).unwrap().bgcolor())
        .collect();
    let expected: Vec<_> = matches
        .into_iter()
        .map(|matches| {
            if matches {
                vt100::Color::Rgb(239, 68, 68)
            } else {
                vt100::Color::Rgb(0, 0, 0)
            }
        })
        .collect();
    assert_eq!(
        observed, expected,
        "STY-001: {variant}: backgrounds for children A, B, C were {observed:?}"
    );
}

#[test]
fn sty_001_first_paints_only_matching_children() {
    assert_positional_backgrounds("first", [true, false, false]);
}

#[test]
fn sty_001_last_paints_only_matching_children() {
    assert_positional_backgrounds("last", [false, false, true]);
}

#[test]
fn sty_001_odd_paints_only_matching_children() {
    assert_positional_backgrounds("odd", [true, false, true]);
}

#[test]
fn sty_001_even_paints_only_matching_children() {
    assert_positional_backgrounds("even", [false, true, false]);
}

fn pointer(kind: MouseEventKind, x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(
        MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left),
    ))
}

#[test]
fn sty_001_active_applies_only_between_press_and_release() {
    let root = div()
        .class("flex flex-col w-full h-full")
        .child(Element::text("target").class("w-8 h-1 bg-black active:bg-primary"))
        .child(Element::text("control").class("w-8 h-1 bg-primary"))
        .build();
    let frames = run_until(
        TreeRoot(root),
        (12, 4),
        vec![
            Until {
                text: "target",
                cell: None,
                event: pointer(MouseEventKind::Down, 1, 0),
            },
            Until {
                text: "target",
                cell: None,
                event: pointer(MouseEventKind::Up, 1, 0),
            },
            Until {
                text: "target",
                cell: None,
                event: None,
            },
        ],
        Duration::from_secs(30),
    );
    let primary = frames[0].screen.cell(1, 0).unwrap().bgcolor();
    let mut colors: Vec<_> = frames
        .iter()
        .map(|frame| frame.screen.cell(0, 0).unwrap().bgcolor())
        .collect();
    colors.dedup();
    assert_eq!(
        colors,
        [vt100::Color::Rgb(0, 0, 0), primary, vt100::Color::Rgb(0, 0, 0)],
        "STY-001: active:bg-primary backgrounds before press, while held, and after release were {colors:?}"
    );
}

#[test]
fn sty_001_group_hover_tracks_the_ancestor_pointer() {
    let root = div()
        .class("flex flex-col w-full h-full")
        .child(
            div()
                .class("group w-8 h-2")
                .child(Element::text("target").class("w-6 h-1 bg-black group-hover:bg-primary"))
                .build(),
        )
        .child(Element::text("control").class("w-8 h-1 bg-primary"))
        .build();
    let frames = run_until(
        TreeRoot(root),
        (12, 4),
        vec![
            Until {
                text: "target",
                cell: None,
                event: pointer(MouseEventKind::Move, 7, 1),
            },
            Until {
                text: "target",
                cell: None,
                event: pointer(MouseEventKind::Move, 10, 3),
            },
            Until {
                text: "target",
                cell: None,
                event: None,
            },
        ],
        Duration::from_secs(30),
    );
    let primary = frames[0].screen.cell(2, 0).unwrap().bgcolor();
    let mut colors: Vec<_> = frames
        .iter()
        .map(|frame| frame.screen.cell(0, 0).unwrap().bgcolor())
        .collect();
    colors.dedup();
    assert_eq!(
        colors,
        [vt100::Color::Rgb(0, 0, 0), primary, vt100::Color::Rgb(0, 0, 0)],
        "STY-001: group-hover:bg-primary backgrounds before entry, over the ancestor, and outside were {colors:?}"
    );
}

#[test]
fn sty_001_visited_never_paints_its_background() {
    let frames = run_when_painted(
        TreeRoot(Element::text("target").class("w-8 h-1 bg-black visited:bg-red-500")),
        (12, 4),
        1,
    );
    let color = frames.last().unwrap().screen.cell(0, 0).unwrap().bgcolor();
    assert_eq!(
        color,
        vt100::Color::Rgb(0, 0, 0),
        "STY-001: visited:bg-red-500 painted {color:?}"
    );
}

#[test]
fn sty_002_display_none_is_kept_in_the_built_style() {
    let style = css! { display: Display::None }.build();
    assert_eq!(
        style.display,
        Display::None,
        "STY-002: css! display: Display::None built display {:?}",
        style.display
    );
}

#[test]
fn sty_002_display_none_removes_space_paint_and_mouse_targets() {
    let calls = Arc::new(AtomicUsize::new(0));
    let hidden_calls = calls.clone();
    let child_calls = calls.clone();
    let root = div()
        .class("flex flex-row w-full h-full")
        .children(vec![
            Element::text("A").class("w-1 h-1"),
            div()
                .class("w-3 h-1 bg-red-500")
                .styles(css! { display: Display::None })
                .on_click(move || {
                    hidden_calls.fetch_add(1, Ordering::SeqCst);
                })
                .child(
                    div()
                        .class("w-3 h-1")
                        .text("HID")
                        .on_click(move || {
                            child_calls.fetch_add(1, Ordering::SeqCst);
                        })
                        .build(),
                )
                .build(),
            Element::text("B").class("w-1 h-1"),
        ])
        .build();
    let frames = run_until(
        TreeRoot(root),
        (12, 4),
        vec![
            Until {
                text: "A",
                cell: None,
                event: click(2, 0),
            },
            Until {
                text: "A",
                cell: None,
                event: None,
            },
        ],
        Duration::from_secs(30),
    );
    let screen = &frames.last().unwrap().screen;
    assert_eq!(
        screen.cell(0, 1).unwrap().contents(),
        "B",
        "STY-002: hidden middle sibling kept A and B apart: {:?}",
        screen.contents()
    );
    assert!(
        !screen.contents().contains("HID"),
        "STY-002: display-none child painted HID: {:?}",
        screen.contents()
    );
    assert_ne!(
        screen.cell(0, 2).unwrap().bgcolor(),
        vt100::Color::Rgb(239, 68, 68),
        "STY-002: display-none middle sibling painted a red cell"
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "STY-002: display-none element or child received a click"
    );
}

#[test]
fn sty_003_aspect_auto_clears_the_earlier_ratio() {
    let style = apply_utility_classes("aspect-square aspect-auto", StyleBuilder::new()).build();
    assert_eq!(
        style.aspect_ratio, None,
        "STY-003: aspect-square aspect-auto left aspect_ratio {:?}",
        style.aspect_ratio
    );
}

#[test]
fn sty_004_doc_comments_describe_runtime_matching_and_ignored_pairs() {
    let docs = include_str!("../../src/layout/css/css_in_rust.rs")
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            line.strip_prefix("//!")
                .or_else(|| line.strip_prefix("///"))
        })
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let promises = [
        "compile-time validation",
        "validated at compile time",
        "type-checked at compile time",
        "type-safe css property",
        "type-safe way to write css properties",
    ];
    let observed: Vec<_> = promises
        .iter()
        .filter(|promise| docs.contains(**promise))
        .collect();
    assert!(
        observed.is_empty(),
        "STY-004: doc comments promise property validation or type checking the macro does not perform: {observed:?}"
    );
    assert!(
        docs.contains("unknown property") && docs.contains("wrong kind") && docs.contains("ignored"),
        "STY-004: doc comments do not explain that an unknown property or a value of the wrong kind is ignored"
    );
    assert!(
        docs.contains("matched") && (docs.contains("built") || docs.contains("runtime")),
        "STY-004: doc comments do not say property names are matched when the style is built"
    );
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// The colors a 4 by 2 SuprTUI backend presents at cell (0, 0) for `frame`.
fn presented_corner(frame: &Element) -> (vt100::Color, vt100::Color) {
    let capture = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(4, 2, capture.clone()).unwrap();
    backend.render_frame(frame).unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
    let mut terminal = vt100::Parser::new(2, 4, 0);
    terminal.process(&capture.0.lock().unwrap());
    let cell = terminal.screen().cell(0, 0).unwrap();
    (cell.fgcolor(), cell.bgcolor())
}

#[test]
fn pnt_006_explicit_cell_colors_include_own_and_ancestor_opacity() {
    let mut grid = CellGrid::new(1, 1);
    let red = Some((1.0, 0.0, 0.0, 1.0));
    grid.set_with_background(0, 0, "X", red, red);
    let grid = Arc::new(grid);
    for (parent, child) in [
        ("w-full h-full bg-black", "w-1 h-1 opacity-50"),
        ("w-full h-full bg-black opacity-50", "w-1 h-1"),
    ] {
        let explicit = presented_corner(
            &div()
                .class(parent)
                .child(div().class(child).build().with_cells(grid.clone()))
                .build(),
        );
        let styled = presented_corner(
            &div()
                .class(parent)
                .child(
                    div()
                        .class(&format!("{child} bg-#ff0000 text-#ff0000"))
                        .text("X")
                        .build(),
                )
                .build(),
        );
        assert_ne!(
            styled,
            (vt100::Color::Rgb(255, 0, 0), vt100::Color::Rgb(255, 0, 0)),
            "the style colors under {parent:?} / {child:?} take the opacity"
        );
        assert_eq!(
            explicit, styled,
            "PNT-006: under {parent:?} / {child:?} the explicit red cell presented {explicit:?}, the same red as style colors {styled:?}"
        );
    }
}
